//! The services thread (spec §6, §7): feeds the analyzer, routes its results
//! into the model and the UI's media cache, and saves the state. It never
//! touches the audio path; it only talks to the conductor through its
//! non-blocking handle.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, PoisonError, RwLock};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender};
use fp_analysis::analyzer::{AnalysisResult, Analyzer};
use fp_analysis::{AnalysisError, WavePeak};
use fp_engine::conductor::ConductorHandle;
use fp_model::{AnalysisSettings, AppState, Command, FileState, PlayerId, TrackId, Transport};
use fp_store::Store;

/// How often the services thread wakes up when running on its own.
const PERIOD: Duration = Duration::from_millis(50);

/// What the UI draws for a track besides its model data.
#[derive(Debug, Clone, PartialEq)]
pub struct TrackMedia {
    /// Min, max and RMS per bucket, full scale = `i16::MAX`.
    pub peaks: Vec<WavePeak>,
    pub peak_bucket_secs: f64,
    pub cover_png: Option<Arc<[u8]>>,
}

/// Peaks and covers shared with the UI. The version increases on every
/// change so the UI can refresh its textures cheaply.
#[derive(Debug, Clone, Default)]
pub struct MediaCache {
    items: Arc<RwLock<HashMap<TrackId, Arc<TrackMedia>>>>,
    version: Arc<AtomicU64>,
}

impl MediaCache {
    pub fn get(&self, track: TrackId) -> Option<Arc<TrackMedia>> {
        let items = self.items.read().unwrap_or_else(PoisonError::into_inner);
        items.get(&track).cloned()
    }

    pub fn contains(&self, track: TrackId) -> bool {
        let items = self.items.read().unwrap_or_else(PoisonError::into_inner);
        items.contains_key(&track)
    }

    pub fn version(&self) -> u64 {
        self.version.load(Ordering::Acquire)
    }

    fn insert(&self, track: TrackId, media: TrackMedia) {
        let mut items = self.items.write().unwrap_or_else(PoisonError::into_inner);
        items.insert(track, Arc::new(media));
        self.version.fetch_add(1, Ordering::AcqRel);
    }

    fn retain(&self, keep: impl Fn(TrackId) -> bool) {
        let mut items = self.items.write().unwrap_or_else(PoisonError::into_inner);
        let before = items.len();
        items.retain(|id, _| keep(*id));
        if items.len() != before {
            self.version.fetch_add(1, Ordering::AcqRel);
        }
    }
}

/// Requests from the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceRequest {
    /// Analyse every track again (manual markers are kept by the model).
    ReanalyseAll,
    /// Analyse the tracks an earlier version analysed (`outdated_tracks`).
    AnalyseOutdated,
}

/// Whether `track` was analysed by an earlier version: before formats were
/// recorded (Phase 4), or under other marker rules. A track whose file
/// cannot be read is left out: it cannot be analysed again, and counting it
/// would bring the start-up notice back at every start.
pub fn outdated(track: &fp_model::Track) -> bool {
    track.analyzed
        && track.file_state.is_playable()
        && (track.format.is_none() || track.analysis_version < fp_analysis::cache::ANALYSIS_VERSION)
}

/// How many tracks an earlier version analysed. They keep that analysis,
/// still usable, until the operator asks for a new one: re-analysing a
/// library takes the processor for a while on an on-air machine.
pub fn outdated_tracks(state: &AppState) -> usize {
    state.library.iter().filter(|t| outdated(t)).count()
}

pub struct Services {
    conductor: Arc<ConductorHandle>,
    store: Store,
    analyzer: Analyzer,
    media: MediaCache,
    requests: Receiver<ServiceRequest>,
    request_tx: Sender<ServiceRequest>,
    /// Tracks submitted and not answered yet.
    in_flight: HashSet<TrackId>,
    /// Tracks whose result reached the model since the last reset.
    done: HashSet<TrackId>,
    /// Tracks whose file could not be analysed since the last reset.
    failed: HashSet<TrackId>,
    /// Tracks to analyse again even if already analysed.
    forced: HashSet<TrackId>,
    /// The operator asked for the outdated tracks to be analysed again.
    analyse_outdated: bool,
    /// Tracks whose result was lost to a panic once already: a second loss
    /// gives up on them (as failed) instead of retrying forever.
    retried: HashSet<TrackId>,
    /// Tracks seen analysed in a snapshot. One that turns unanalysed again
    /// (markers reset) is analysed again.
    seen_analyzed: HashSet<TrackId>,
    settings: Option<AnalysisSettings>,
    /// Steps that panicked (shown by the UI as an alert).
    faults: Arc<AtomicU64>,
    #[cfg(feature = "test-hooks")]
    fail_steps: u32,
    #[cfg(feature = "test-hooks")]
    fail_routes: u32,
    saved_version: u64,
    dirty_since: Option<Instant>,
    last_session_save: Option<Instant>,
}

impl Services {
    pub fn new(
        conductor: Arc<ConductorHandle>,
        store: Store,
        analyzer: Analyzer,
        media: MediaCache,
    ) -> Self {
        let (request_tx, requests) = crossbeam_channel::bounded(16);
        Self {
            conductor,
            store,
            analyzer,
            media,
            requests,
            request_tx,
            in_flight: HashSet::new(),
            done: HashSet::new(),
            failed: HashSet::new(),
            forced: HashSet::new(),
            analyse_outdated: false,
            retried: HashSet::new(),
            seen_analyzed: HashSet::new(),
            settings: None,
            faults: Arc::new(AtomicU64::new(0)),
            #[cfg(feature = "test-hooks")]
            fail_steps: 0,
            #[cfg(feature = "test-hooks")]
            fail_routes: 0,
            saved_version: 0,
            dirty_since: None,
            last_session_save: None,
        }
    }

    /// Where the UI sends its requests.
    pub fn requests(&self) -> Sender<ServiceRequest> {
        self.request_tx.clone()
    }

    /// Counts the steps that panicked.
    pub fn faults(&self) -> Arc<AtomicU64> {
        self.faults.clone()
    }

    /// Makes the analysis part of the next `count` steps panic. Used to test
    /// panic isolation.
    #[cfg(feature = "test-hooks")]
    pub fn fail_steps(&mut self, count: u32) {
        self.fail_steps = count;
    }

    /// Makes applying the next `count` analysis results panic, after the
    /// result was taken. Used to test that no result is lost for good.
    #[cfg(feature = "test-hooks")]
    pub fn fail_routes(&mut self, count: u32) {
        self.fail_routes = count;
    }

    /// One round of work. Never blocks on the analyzer. A panic inside is
    /// logged and counted, and the next step runs normally, so saving never
    /// stops for the rest of the session.
    pub fn step(&mut self, now: Instant) {
        // The version is read before the snapshot: the snapshot saved can
        // only be newer than the version recorded as saved, never older.
        let version = self.conductor.telemetry.load().model_version;
        let state = self.conductor.model.load_full();
        // Analysis and saving are isolated from each other: a fault in one
        // never stops the other.
        let analysis = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.analysis_step(&state);
        }));
        let saving = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.autosave(&state, version, now);
        }));
        for outcome in [analysis, saving] {
            if outcome.is_err() {
                self.faults.fetch_add(1, Ordering::AcqRel);
                tracing::error!("a services step panicked; continuing");
            }
        }
    }

    fn analysis_step(&mut self, state: &AppState) {
        #[cfg(feature = "test-hooks")]
        if self.fail_steps > 0 {
            self.fail_steps -= 1;
            #[allow(clippy::panic)]
            {
                panic!("injected services failure");
            }
        }
        while let Ok(request) = self.requests.try_recv() {
            match request {
                ServiceRequest::ReanalyseAll => self.restart_analysis(state),
                ServiceRequest::AnalyseOutdated => self.analyse_outdated = true,
            }
        }
        self.follow_settings(state);
        self.submit_new(state);
        let wanted = Self::wanted(state);
        let results: Vec<AnalysisResult> = self.analyzer.results().try_iter().collect();
        for result in results {
            // A result taken from the analyzer is gone from it: if applying
            // it panics, the track is asked for again (once) instead of
            // staying "in progress" until the next start.
            let track = result.track;
            let routed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.route(result, &wanted);
            }));
            if routed.is_err() {
                self.faults.fetch_add(1, Ordering::AcqRel);
                tracing::error!(?track, "applying an analysis result panicked");
                self.in_flight.remove(&track);
                self.done.remove(&track);
                if !self.retried.insert(track) {
                    self.failed.insert(track);
                }
            }
        }
    }

    /// Final save, with the positions the engine reports right now.
    pub fn shutdown(&mut self) {
        let state = self.conductor.model.load_full();
        self.save_all(&state);
    }

    /// Runs on its own thread until the handle is shut down or dropped.
    pub fn spawn(mut self) -> std::io::Result<ServicesHandle> {
        let requests = self.requests();
        let stop = Arc::new(AtomicBool::new(false));
        let flag = stop.clone();
        let thread = std::thread::Builder::new()
            .name("fp-services".to_owned())
            .spawn(move || {
                while !flag.load(Ordering::Acquire) {
                    self.step(Instant::now());
                    std::thread::sleep(PERIOD);
                }
                self.shutdown();
            })?;
        Ok(ServicesHandle {
            requests,
            stop,
            thread: Some(thread),
        })
    }

    fn follow_settings(&mut self, state: &AppState) {
        let current = &state.config.analysis;
        if self.settings.as_ref() == Some(current) {
            return;
        }
        if self.settings.is_some() {
            // New thresholds change the automatic markers of every track.
            self.analyzer.update_settings(current.clone());
            self.restart_analysis(state);
        }
        self.settings = Some(current.clone());
    }

    /// Analyses every track again; running analyses are cancelled so that
    /// none finishes with outdated settings.
    fn restart_analysis(&mut self, state: &AppState) {
        for id in self.in_flight.drain() {
            self.analyzer.cancel(id);
        }
        self.done.clear();
        self.failed.clear();
        self.retried.clear();
        self.forced = state.library.iter().map(|t| t.id).collect();
    }

    /// Tracks the screen draws: current, next and cue entries of the players.
    fn wanted(state: &AppState) -> HashSet<TrackId> {
        state
            .players
            .iter()
            .flat_map(|p| [p.current, p.next, p.cue.map(|c| c.entry)])
            .flatten()
            .filter_map(|e| state.playlists.entry(e))
            .map(|e| e.track)
            .collect()
    }

    fn submit_new(&mut self, state: &AppState) {
        let known: HashSet<TrackId> = state.library.iter().map(|t| t.id).collect();
        for gone in self.in_flight.difference(&known) {
            self.analyzer.cancel(*gone);
        }
        self.in_flight.retain(|id| known.contains(id));
        self.done.retain(|id| known.contains(id));
        self.failed.retain(|id| known.contains(id));
        self.forced.retain(|id| known.contains(id));
        self.retried.retain(|id| known.contains(id));
        self.seen_analyzed.retain(|id| known.contains(id));
        for track in state.library.iter() {
            if track.analyzed {
                self.seen_analyzed.insert(track.id);
            } else if self.seen_analyzed.remove(&track.id) {
                // Analysed before, not any more: the model asked for a new
                // analysis (manual markers reset or cleared).
                self.done.remove(&track.id);
                self.failed.remove(&track.id);
            }
        }
        // Peaks and covers are kept only for what the players show; the
        // analysis cache brings them back when a track is shown again.
        let wanted = Self::wanted(state);
        self.media.retain(|id| wanted.contains(&id));
        for track in state.library.iter() {
            let id = track.id;
            if self.in_flight.contains(&id) {
                continue;
            }
            // Tracks an earlier version analysed are analysed again, once,
            // when the operator asks; the ones on screen are anyway (`show`).
            let stale = self.analyse_outdated && outdated(track);
            let analyse = self.forced.contains(&id)
                || ((!track.analyzed || stale)
                    && !self.done.contains(&id)
                    && !self.failed.contains(&id));
            let show =
                wanted.contains(&id) && !self.media.contains(id) && !self.failed.contains(&id);
            if analyse || show {
                self.forced.remove(&id);
                self.in_flight.insert(id);
                self.analyzer.submit(id, track.path.clone());
            }
        }
    }

    fn route(&mut self, result: AnalysisResult, wanted: &HashSet<TrackId>) {
        if matches!(result.outcome, Err(AnalysisError::Cancelled)) {
            return;
        }
        #[cfg(feature = "test-hooks")]
        if self.fail_routes > 0 {
            self.fail_routes -= 1;
            #[allow(clippy::panic)]
            {
                panic!("injected routing failure");
            }
        }
        self.in_flight.remove(&result.track);
        self.done.insert(result.track);
        let command = match result.outcome {
            Ok(analysis) => {
                self.failed.remove(&result.track);
                if wanted.contains(&result.track) {
                    self.media.insert(
                        result.track,
                        TrackMedia {
                            peaks: analysis.peaks,
                            peak_bucket_secs: analysis.peak_bucket_secs,
                            cover_png: analysis.cover_png.map(Arc::from),
                        },
                    );
                }
                Command::ApplyAnalysis {
                    track: result.track,
                    analysis: Box::new(analysis.analysis),
                }
            }
            Err(AnalysisError::Missing) => {
                self.failed.insert(result.track);
                Command::SetFileState {
                    track: result.track,
                    state: FileState::Missing,
                }
            }
            Err(AnalysisError::Unreadable(reason)) => {
                self.failed.insert(result.track);
                tracing::warn!(path = %result.path.display(), %reason, "unreadable file");
                Command::SetFileState {
                    track: result.track,
                    state: FileState::Unreadable,
                }
            }
            Err(AnalysisError::Cancelled) => return,
        };
        if !self.conductor.send(command) {
            // The queue is full: try again on a later round.
            self.done.remove(&result.track);
            self.forced.insert(result.track);
        }
    }

    fn autosave(&mut self, state: &AppState, version: u64, now: Instant) {
        if version != self.saved_version && self.dirty_since.is_none() {
            self.dirty_since = Some(now);
        }
        let debounce =
            Duration::from_secs_f64(state.config.tuning.save_debounce_ms.max(0.0) / 1000.0);
        if self
            .dirty_since
            .is_some_and(|since| now.saturating_duration_since(since) >= debounce)
        {
            // Recorded only once the files are written: a failed save is
            // retried on the next step.
            self.save_all(state);
            self.saved_version = version;
            self.dirty_since = None;
            self.last_session_save = Some(now);
            return;
        }
        let playing = state
            .players
            .iter()
            .any(|p| p.transport == Transport::Playing);
        let session_due = self
            .last_session_save
            .is_none_or(|last| now.saturating_duration_since(last) >= debounce);
        if playing && session_due {
            self.save_session(state);
            self.last_session_save = Some(now);
        }
    }

    fn save_all(&self, state: &AppState) {
        if let Err(e) = self.store.save_config(state) {
            tracing::error!(error = %e, "saving the configuration failed");
        }
        if let Err(e) = self.store.save_playlists(state) {
            tracing::error!(error = %e, "saving the playlists failed");
        }
        if let Err(e) = self.store.save_carts(state) {
            tracing::error!(error = %e, "saving the cart pages failed");
        }
        self.save_session(state);
    }

    fn save_session(&self, state: &AppState) {
        let telemetry = self.conductor.telemetry.load();
        let position_of = |player: PlayerId| {
            telemetry
                .players
                .iter()
                .find(|(id, _)| *id == player)
                .and_then(|(_, t)| t.position_secs)
                .unwrap_or(0.0)
        };
        if let Err(e) = self.store.save_session(state, position_of) {
            tracing::error!(error = %e, "saving the session failed");
        }
    }
}

/// The running services thread.
pub struct ServicesHandle {
    requests: Sender<ServiceRequest>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl ServicesHandle {
    /// Queues a request; never blocks.
    pub fn request(&self, request: ServiceRequest) -> bool {
        self.requests.try_send(request).is_ok()
    }

    /// Stops the thread after its final save.
    pub fn shutdown(mut self) {
        self.stop_and_join();
    }

    fn stop_and_join(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take()
            && thread.join().is_err()
        {
            tracing::error!("the services thread panicked");
        }
    }
}

impl Drop for ServicesHandle {
    fn drop(&mut self) {
        self.stop_and_join();
    }
}
