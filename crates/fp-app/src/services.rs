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
use fp_analysis::AnalysisError;
use fp_analysis::analyzer::{AnalysisResult, Analyzer};
use fp_engine::conductor::ConductorHandle;
use fp_model::{AnalysisSettings, AppState, Command, FileState, PlayerId, TrackId, Transport};
use fp_store::Store;

/// How often the services thread wakes up when running on its own.
const PERIOD: Duration = Duration::from_millis(50);

/// What the UI draws for a track besides its model data.
#[derive(Debug, Clone, PartialEq)]
pub struct TrackMedia {
    /// Min/max per bucket, full scale = `i16::MAX`.
    pub peaks: Vec<(i16, i16)>,
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
}

pub struct Services {
    conductor: Arc<ConductorHandle>,
    store: Store,
    analyzer: Analyzer,
    media: MediaCache,
    requests: Receiver<ServiceRequest>,
    request_tx: Sender<ServiceRequest>,
    /// Tracks submitted and not yet answered, or answered.
    submitted: HashSet<TrackId>,
    settings: Option<AnalysisSettings>,
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
            submitted: HashSet::new(),
            settings: None,
            saved_version: 0,
            dirty_since: None,
            last_session_save: None,
        }
    }

    /// Where the UI sends its requests.
    pub fn requests(&self) -> Sender<ServiceRequest> {
        self.request_tx.clone()
    }

    /// One round of work. Never blocks on the analyzer.
    pub fn step(&mut self, now: Instant) {
        let state = self.conductor.model.load_full();
        while let Ok(request) = self.requests.try_recv() {
            match request {
                ServiceRequest::ReanalyseAll => self.submitted.clear(),
            }
        }
        self.follow_settings(&state);
        self.submit_new(&state);
        while let Ok(result) = self.analyzer.results().try_recv() {
            self.route(result);
        }
        self.autosave(&state, now);
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
            self.submitted.clear();
        }
        self.settings = Some(current.clone());
    }

    fn submit_new(&mut self, state: &AppState) {
        let known: HashSet<TrackId> = state.library.iter().map(|t| t.id).collect();
        for gone in self.submitted.difference(&known) {
            self.analyzer.cancel(*gone);
        }
        self.submitted.retain(|id| known.contains(id));
        self.media.retain(|id| known.contains(&id));
        for track in state.library.iter() {
            if self.submitted.insert(track.id) {
                self.analyzer.submit(track.id, track.path.clone());
            }
        }
    }

    fn route(&mut self, result: AnalysisResult) {
        let command = match result.outcome {
            Ok(analysis) => {
                self.media.insert(
                    result.track,
                    TrackMedia {
                        peaks: analysis.peaks,
                        peak_bucket_secs: analysis.peak_bucket_secs,
                        cover_png: analysis.cover_png.map(Arc::from),
                    },
                );
                Command::ApplyAnalysis {
                    track: result.track,
                    analysis: Box::new(analysis.analysis),
                }
            }
            Err(AnalysisError::Missing) => Command::SetFileState {
                track: result.track,
                state: FileState::Missing,
            },
            Err(AnalysisError::Unreadable(reason)) => {
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
            self.submitted.remove(&result.track);
        }
    }

    fn autosave(&mut self, state: &AppState, now: Instant) {
        let version = self.conductor.telemetry.load().model_version;
        if version != self.saved_version && self.dirty_since.is_none() {
            self.dirty_since = Some(now);
        }
        let debounce =
            Duration::from_secs_f64(state.config.tuning.save_debounce_ms.max(0.0) / 1000.0);
        if self
            .dirty_since
            .is_some_and(|since| now.saturating_duration_since(since) >= debounce)
        {
            self.saved_version = version;
            self.dirty_since = None;
            self.save_all(state);
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
