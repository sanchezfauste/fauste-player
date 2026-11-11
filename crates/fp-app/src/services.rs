//! The services thread (spec §6, §7): feeds the analyzer, routes its results
//! into the model and the UI's media cache, and saves the state. It never
//! touches the audio path; it only talks to the conductor through its
//! non-blocking handle.

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError, RwLock};
use std::thread::JoinHandle;
use std::time::{Duration, Instant, SystemTime};

use crossbeam_channel::{Receiver, Sender};
use fp_analysis::analyzer::{AnalysisResult, Analyzer, LoudnessResult};
use fp_analysis::{AnalysisError, WavePeak};
use fp_engine::conductor::ConductorHandle;
use fp_model::{
    AnalysisSettings, AppState, Command, FileStamp, FileState, LOUDNESS_VERSION, Library, PlayerId,
    TrackId, Transport, track_sounding,
};
use fp_store::Store;

use crate::activity::{ActivityCell, AnalysisActivity, PassProgress};
use crate::changes::ChangeCheck;
use crate::header::HeaderReader;
use crate::tags::{TagJob, TagOutcome, TagWorker};
use crate::watch::folders::FolderWatch;
use crate::watch::{FolderWatcher, WatchCell, idle_watch_cell};

/// How often the services thread wakes up when running on its own.
const PERIOD: Duration = Duration::from_millis(50);

/// How many header reads wait on the reader at once: few enough that a
/// track loaded on a player goes ahead of a large folder just added.
const HEADER_READS_IN_FLIGHT: usize = 8;

/// How many files one change check looks at (EE7): a large library is
/// checked in several batches, so a track loaded on a player meanwhile is
/// checked after the batch running, not after the whole library.
const CHANGE_BATCH: usize = 256;

/// What a published activity was built from (`Services::publish_activity`).
type ActivityKey = (u64, usize, usize, usize, usize, Option<(u64, usize)>, bool);

/// A *Look for changed files* pass (EE7.5) still running.
struct LookPass {
    id: u64,
    /// Its tracks without an answer yet.
    pending: HashSet<TrackId>,
    /// How many of the answered ones changed.
    found: usize,
}

/// A queue of tracks that holds each one once: pushing a queued track does
/// nothing (M8: O(1), however long the queue).
#[derive(Default)]
struct UniqueQueue {
    order: VecDeque<TrackId>,
    queued: HashSet<TrackId>,
}

impl UniqueQueue {
    fn push_back(&mut self, id: TrackId) {
        if self.queued.insert(id) {
            self.order.push_back(id);
        }
    }

    fn pop_front(&mut self) -> Option<TrackId> {
        let id = self.order.pop_front()?;
        self.queued.remove(&id);
        Some(id)
    }

    #[cfg(any(test, feature = "test-hooks"))]
    fn len(&self) -> usize {
        self.order.len()
    }

    fn clear(&mut self) {
        self.order.clear();
        self.queued.clear();
    }
}

/// The thread that looks for files not found and at unreadable files, off
/// the services thread: a share that is offline can make each look block
/// for seconds.
struct Probe {
    paths: Sender<Job>,
    answers: Receiver<Answer>,
    /// A look was sent and its answer has not come back.
    busy: bool,
}

impl Probe {
    fn spawn() -> Option<Self> {
        let (paths, jobs) = crossbeam_channel::unbounded::<Job>();
        let (answer_tx, answers) = crossbeam_channel::unbounded::<Answer>();
        let spawned = std::thread::Builder::new()
            .name("fp-file-probe".to_owned())
            .spawn(move || {
                // Ends when the services drop their side.
                while let Ok(job) = jobs.recv() {
                    if answer_tx.send(run(job)).is_err() {
                        break;
                    }
                }
            });
        match spawned {
            Ok(_) => Some(Self {
                paths,
                answers,
                busy: false,
            }),
            Err(e) => {
                tracing::error!(error = %e, "cannot start the file probe; missing files stay missing");
                None
            }
        }
    }
}

/// The tracks of `batch` whose file exists. Each folder is looked at once,
/// from the root down, and the folders under one that is gone are not
/// looked at: a drive that is not mounted costs one look, not one per
/// album or file.
fn present(batch: Vec<(TrackId, PathBuf)>) -> Vec<TrackId> {
    let mut folders: HashMap<PathBuf, bool> = HashMap::new();
    batch
        .into_iter()
        .filter(|(_, path)| {
            path.parent()
                .is_none_or(|dir| folder_exists(dir, &mut folders))
                && path.is_file()
        })
        .map(|(id, _)| id)
        .collect()
}

fn folder_exists(dir: &std::path::Path, seen: &mut HashMap<PathBuf, bool>) -> bool {
    if let Some(known) = seen.get(dir) {
        return *known;
    }
    let exists = match dir.parent() {
        Some(up) if !up.as_os_str().is_empty() && !folder_exists(up, seen) => false,
        // A bare relative name: the file itself decides.
        _ if dir.as_os_str().is_empty() => true,
        _ => dir.is_dir(),
    };
    seen.insert(dir.to_path_buf(), exists);
    exists
}

/// A file's size and modification time: what tells that it changed.
type Stamp = (u64, SystemTime);

/// What the probe has seen of an unreadable file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Seen {
    /// Not looked at yet.
    New,
    /// Looked at, and the `stat` failed.
    Unknown,
    At(Stamp),
}

/// What one look at an unreadable file means (operator feedback 4, Q10).
#[derive(Debug, PartialEq)]
enum Look {
    /// First stat (or the first that worked): remember it.
    Record(Seen),
    /// Nothing to do.
    Keep,
    /// The size or the modification time changed: analyse it again.
    Changed,
    /// The file is not there any more.
    Gone,
}

fn look(seen: Seen, now: std::io::Result<Stamp>) -> Look {
    match (now, seen) {
        (Ok(now), Seen::New | Seen::Unknown) => Look::Record(Seen::At(now)),
        (Ok(now), Seen::At(before)) if now == before => Look::Keep,
        (Ok(_), Seen::At(_)) => Look::Changed,
        (Err(e), _) if e.kind() == std::io::ErrorKind::NotFound => Look::Gone,
        (Err(_), Seen::New) => Look::Record(Seen::Unknown),
        (Err(_), _) => Look::Keep,
    }
}

/// The `stat` of `path`. It never opens or decodes the file.
fn stamp_of(path: &Path) -> std::io::Result<Stamp> {
    let meta = std::fs::metadata(path)?;
    Ok((meta.len(), meta.modified()?))
}

/// One look of the probe thread: files not found, and unreadable files with
/// what was seen of them.
struct Job {
    missing: Vec<(TrackId, PathBuf)>,
    unreadable: Vec<(TrackId, PathBuf, Seen)>,
}

#[derive(Default)]
struct Answer {
    /// Missing files that exist now.
    found: Vec<TrackId>,
    /// Unreadable files looked at for the first time (or the first time the
    /// `stat` worked).
    recorded: Vec<(TrackId, Seen)>,
    /// Unreadable files whose size or modification time changed.
    changed: Vec<TrackId>,
    /// Unreadable files that are not there any more.
    gone: Vec<TrackId>,
}

/// One look, on the probe thread.
fn run(job: Job) -> Answer {
    let mut answer = Answer {
        found: present(job.missing),
        ..Answer::default()
    };
    for (id, path, seen) in job.unreadable {
        match look(seen, stamp_of(&path)) {
            Look::Record(s) => answer.recorded.push((id, s)),
            Look::Keep => {}
            Look::Changed => answer.changed.push(id),
            Look::Gone => answer.gone.push(id),
        }
    }
    answer
}

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

    /// Replaces the cover thumbnail of `track` (`None` removes it), keeping
    /// its peaks: the tag editor changed the cover in the file. A track with
    /// no entry has nothing to refresh: the analysis that fills the cache
    /// when it is shown next reads the new cover from the file (the analysis
    /// cache is keyed on the file's size and modification time, so the
    /// edited file is analysed again).
    pub fn set_cover(&self, track: TrackId, cover: Option<Arc<[u8]>>) {
        let mut items = self.items.write().unwrap_or_else(PoisonError::into_inner);
        let Some(current) = items.get(&track) else {
            return;
        };
        let updated = TrackMedia {
            peaks: current.peaks.clone(),
            peak_bucket_secs: current.peak_bucket_secs,
            cover_png: cover,
        };
        items.insert(track, Arc::new(updated));
        self.version.fetch_add(1, Ordering::AcqRel);
    }

    /// Puts `media` in the cache as the services thread does after an
    /// analysis (tests).
    #[cfg(feature = "test-hooks")]
    pub fn seed(&self, track: TrackId, media: TrackMedia) {
        self.insert(track, media);
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
    /// Analyse this track at once, whatever its state (the row menu's
    /// Re-analyse).
    ReanalyseTrack(TrackId),
    /// Analyse every unreadable track again: the app may read files it
    /// could not before (Android: the media permission was just granted),
    /// which changes neither their size nor their modification time.
    RecheckUnreadable,
    /// Check this track's file for a change now (external editor spec
    /// EE7.4: its markers were opened for editing).
    CheckTrack(TrackId),
    /// *Look for changed files* (EE7.5): check every track's file and tell
    /// how many changed (`AnalysisActivity::changed_found`).
    LookForChangedFiles,
    /// Follow the folder of a file just opened in the external editor
    /// (EE7.3).
    FollowEditedFile(TrackId),
}

/// Whether `track` was analysed by an earlier version: before formats were
/// recorded (Phase 4), or under other marker rules. A track whose file
/// cannot be read is left out: it cannot be analysed again, and counting it
/// would bring the start-up notice back at every start.
pub fn outdated(track: &fp_model::Track) -> bool {
    track.analyzed
        && track.file_state.is_playable()
        && track.format.is_none_or(|f| {
            !fp_analysis::cache::analysis_is_current(track.analysis_version, f.channels)
        })
}

/// How many tracks an earlier version analysed. They keep that analysis,
/// still usable, until the operator asks for a new one: re-analysing a
/// library takes the processor for a while on an on-air machine.
pub fn outdated_tracks(state: &AppState) -> usize {
    state.library.iter().filter(|t| outdated(t)).count()
}

/// A count over the library, remembered for the model snapshot it was
/// worked out on: it scans once per new snapshot instead of on every frame.
/// It never compares libraries: a new snapshot is a new scan.
#[derive(Default)]
struct SnapshotCount {
    /// Kept (not just its address), so that the address cannot be reused
    /// by another snapshot while it is remembered.
    snapshot: Option<Arc<AppState>>,
    count: usize,
    scans: u32,
}

impl SnapshotCount {
    fn get(&mut self, state: &Arc<AppState>, count: fn(&AppState) -> usize) -> usize {
        if !self
            .snapshot
            .as_ref()
            .is_some_and(|known| Arc::ptr_eq(known, state))
        {
            self.count = count(state);
            self.snapshot = Some(Arc::clone(state));
            self.scans += 1;
        }
        self.count
    }
}

/// `outdated_tracks`, remembered for the model snapshot it was worked out
/// on. Counting scans the whole library, which the interface would do on
/// every frame while the start-up notice or Settings > Analysis is open;
/// with this it scans once per new snapshot.
#[derive(Default)]
pub struct OutdatedCount(SnapshotCount);

impl OutdatedCount {
    /// How many tracks an earlier version analysed, in `state`.
    pub fn get(&mut self, state: &Arc<AppState>) -> usize {
        self.0.get(state, outdated_tracks)
    }

    /// How many times the library was scanned.
    pub fn scans(&self) -> u32 {
        self.0.scans
    }
}

/// `fp_model::loudness_unmeasured`, remembered for the snapshot it was
/// worked out on (Settings > Loudness reads it every frame; loudness
/// normalization spec N22).
#[derive(Default)]
pub struct UnmeasuredCount(SnapshotCount);

impl UnmeasuredCount {
    /// How many playable tracks still wait for a loudness measurement.
    pub fn get(&mut self, state: &Arc<AppState>) -> usize {
        self.0.get(state, fp_model::loudness_unmeasured)
    }

    /// How many times the library was scanned.
    pub fn scans(&self) -> u32 {
        self.0.scans
    }
}

/// What the whole-library walks of a step (the tag, header and loudness
/// passes and the submission of new tracks) depend on, as of the end of the
/// last step.
///
/// A step walks the library again when any of these changed, or when
/// `Services::walk_again` says a pass left work behind. The inputs, kept
/// explicit here so that a new one is not forgotten:
///
/// - the library itself (`Library::shares_storage`: O(1));
/// - what the screen draws: the players' tracks, the cart tracks and the
///   carts' waveforms (they decide what is `wanted`);
/// - the flag *analyse outdated*;
/// - the size of every set of tracks the passes keep (in flight, done,
///   failed, forced, fresh, urgent, the pass, the tag, bit-rate, header and
///   loudness jobs), as a backstop for a change made anywhere else.
///
/// Correctness rests on this: every change to a pass set between two steps
/// must set `walk_again` (or change the library). The sizes only back that
/// up; they do not catch a swap that keeps the count.
///
/// Events that may create work without changing any of them set
/// `walk_again`: operator requests, analysis results, probe, tag, header and
/// loudness answers, a changed analysis setting, and a pass with work left
/// (a full header queue, a full loudness share).
struct WalkInputs {
    library: Library,
    on_players: HashSet<TrackId>,
    on_carts: HashSet<TrackId>,
    carts_drawn: HashSet<TrackId>,
    wanted: HashSet<TrackId>,
    analyse_outdated: bool,
    sizes: [usize; 17],
}

impl WalkInputs {
    fn same_as(&self, other: &Self) -> bool {
        self.library.shares_storage(&other.library)
            && self.analyse_outdated == other.analyse_outdated
            && self.sizes == other.sizes
            && self.on_players == other.on_players
            && self.on_carts == other.on_carts
            && self.carts_drawn == other.carts_drawn
            && self.wanted == other.wanted
    }
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
    /// Of `in_flight`, analysed tracks submitted only for the peaks of the
    /// cart page shown: their result fills the media cache and is not
    /// applied to the model again.
    peaks_only: HashSet<TrackId>,
    /// Of `in_flight`, analysed tracks submitted only because the screen
    /// draws them (a player's or a cart's waveform): not analysis activity
    /// (main spec §6.1), and a Re-analyse of one goes ahead.
    shown_only: HashSet<TrackId>,
    /// Tracks whose result reached the model since the last reset.
    done: HashSet<TrackId>,
    /// Tracks whose file could not be analysed since the last reset.
    failed: HashSet<TrackId>,
    /// Tracks to analyse again even if already analysed.
    forced: HashSet<TrackId>,
    /// Tracks whose next analysis skips the cache lookup: an explicit
    /// re-analysis (main spec §6.1, itr06 I11).
    fresh: HashSet<TrackId>,
    /// The tracks of the library pass still without an answer (main spec
    /// §6.1); empty when no pass runs.
    library_pass: HashSet<TrackId>,
    /// How many tracks the library pass started with.
    pass_total: usize,
    /// What the analysis is doing, for the interface.
    activity: ActivityCell,
    /// What the published activity was built from: the pool's version,
    /// the forced and shown-only counts, the pass and the last count of
    /// changed files; nothing is rebuilt while it stays the same.
    activity_key: Option<ActivityKey>,
    /// How many times the activity was rebuilt (a test hook reads it).
    activity_builds: u64,
    /// The library pass came from *Re-analyse all*: a settings change that
    /// restarts it keeps it fresh.
    pass_fresh: bool,
    /// What the probe saw of each track whose file is unreadable in the
    /// model (from the analysis or from playback): a change of size or
    /// modification time sends it to the analysis again.
    stamps: HashMap<TrackId, Seen>,
    /// The size and modification time an analysis that failed saw, kept
    /// until a snapshot shows the track unreadable and then recorded in
    /// `stamps`: a file that changed after it was read (a copy that
    /// completed) is analysed again.
    failed_at: HashMap<TrackId, Stamp>,
    /// Unreadable tracks reported gone (`Missing` sent) whose snapshot does
    /// not show it yet: neither looked at nor reported again meanwhile.
    gone: HashSet<TrackId>,
    /// Tracks the operator asked to analyse at once (the row menu's
    /// Re-analyse): they go ahead of the library.
    urgent: HashSet<TrackId>,
    /// The operator asked for the outdated tracks to be analysed again.
    analyse_outdated: bool,
    /// The inputs of the library walks as of the end of the last step; `None`
    /// until the first step, and after a step that panicked.
    walked: Option<WalkInputs>,
    /// Something may have created work for the library walks without
    /// changing `walked`: the next step walks again.
    walk_again: bool,
    /// How many steps walked the whole library.
    library_walks: u64,
    /// A pass walked the whole library in the current step.
    walk_counted: bool,
    /// Tracks whose result was lost to a panic once already: a second loss
    /// gives up on them (as failed) instead of retrying forever.
    retried: HashSet<TrackId>,
    /// Tracks seen analysed in a snapshot. One that turns unanalysed again
    /// (markers reset) is analysed again.
    seen_analyzed: HashSet<TrackId>,
    settings: Option<AnalysisSettings>,
    /// The limits the analysis was last given (live settings spec §6).
    limits: Option<fp_model::Limits>,
    /// The crash report cap (live settings spec §6), when `main` gave it.
    report_cap: Option<crate::crash::ReportCap>,
    /// When files not found were last looked for again.
    last_recheck: Option<Instant>,
    /// Looks for them; `None` if its thread could not start.
    probe: Option<Probe>,
    /// The categories' watched folders; `None` until `set_watcher`.
    folders: Option<FolderWatch>,
    /// What the watched folders are doing, for the interface.
    watch_status: WatchCell,
    /// Reads the tags of tracks that wait for the tag-only pass (feedback 2
    /// spec O23); `None` if its thread could not start.
    tag_worker: Option<TagWorker>,
    /// Tracks sent to it and not answered yet.
    tags_in_flight: HashSet<TrackId>,
    /// Tracks whose tag read was answered and sent to the model, but whose
    /// snapshot does not show it yet. Without it a snapshot taken before the
    /// command lands still says "unread" and the track is read twice.
    tags_answered: HashSet<TrackId>,
    /// The same two sets for the bit-rate-only pass, which shares the tag
    /// worker.
    bitrate_in_flight: HashSet<TrackId>,
    bitrate_answered: HashSet<TrackId>,
    /// Loudness-only jobs queued and not answered yet (loudness
    /// normalization spec N6b).
    loudness_in_flight: HashSet<TrackId>,
    /// Tracks whose loudness answer was sent to the model but whose
    /// snapshot does not show it yet: not queued twice meanwhile.
    loudness_answered: HashSet<TrackId>,
    /// Reads the length of tracks not analysed yet (operator feedback 4,
    /// Q1.1); `None` if its thread could not start.
    header_reader: Option<HeaderReader>,
    /// Tracks whose header was asked for (answered or not): once each.
    headers_asked: HashSet<TrackId>,
    /// Reads sent and not answered yet.
    headers_in_flight: usize,
    /// Lengths read whose command could not be queued (the conductor's
    /// queue was full): sent again on a later round, not read again.
    headers_unsent: HashMap<TrackId, f64>,
    /// Looks at library files for changes (external editor spec EE7);
    /// `None` if its thread could not start.
    change_check: Option<ChangeCheck>,
    /// Tracks to check that are in use (EE7.4) or asked for
    /// (`CheckTrack`): sent before `change_bulk`.
    change_urgent: UniqueQueue,
    /// Tracks to check for the start (EE7.1) and on-demand (EE7.5) passes.
    change_bulk: VecDeque<TrackId>,
    /// The tracks of the batch the change check is looking at.
    change_sent: Vec<TrackId>,
    /// The start pass was queued.
    change_started: bool,
    /// The tracks in use at the last step (EE7.4): a track that joins them
    /// is checked.
    in_use: HashSet<TrackId>,
    /// The *Look for changed files* pass running, if any.
    look_pass: Option<LookPass>,
    /// How many such passes were started: the id of the last one.
    look_passes: u64,
    /// The last such pass that ended: its id and its count.
    changed_found: Option<(u64, usize)>,
    /// Changed tracks that sound (EE9): analysed once they stop.
    changed_waiting: HashSet<TrackId>,
    /// Tracks sent to the analysis because their file changed.
    change_analysing: HashSet<TrackId>,
    /// Results of those analyses for tracks that sound now (EE9): applied
    /// once they stop.
    held: HashMap<TrackId, AnalysisResult>,
    /// Tracks whose file changed while it was analysed (EE10: an editor
    /// still writing it), with when they are analysed once more: the watch
    /// debounce interval later.
    changed_retry: HashMap<TrackId, Instant>,
    /// Tracks whose one retry is used: a file still changing then waits
    /// for the next event, use or look.
    changed_retried: HashSet<TrackId>,
    /// Tracks whose file was still changing at their retry: neither
    /// analysed nor read for the screen until a real trigger (an event, a
    /// check, a look, a Re-analyse) takes them up.
    unsettled: HashSet<TrackId>,
    /// The time of the current step.
    clock: Instant,
    /// Steps that panicked (shown by the UI as an alert).
    faults: Arc<AtomicU64>,
    #[cfg(feature = "test-hooks")]
    fail_steps: u32,
    #[cfg(feature = "test-hooks")]
    fail_routes: u32,
    #[cfg(feature = "test-hooks")]
    gone_reports: usize,
    /// Reads sent since the start (a test hook reads it).
    header_reads_sent: u64,
    saved_version: u64,
    dirty_since: Option<Instant>,
    /// A save failed and none has worked since: the failure was logged.
    save_failing: bool,
    /// Earliest time of the next attempt after a failed save.
    next_retry: Option<Instant>,
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
            peaks_only: HashSet::new(),
            shown_only: HashSet::new(),
            done: HashSet::new(),
            failed: HashSet::new(),
            forced: HashSet::new(),
            fresh: HashSet::new(),
            library_pass: HashSet::new(),
            pass_total: 0,
            activity: crate::activity::idle_cell(),
            activity_key: None,
            activity_builds: 0,
            pass_fresh: false,
            stamps: HashMap::new(),
            failed_at: HashMap::new(),
            gone: HashSet::new(),
            urgent: HashSet::new(),
            analyse_outdated: false,
            walked: None,
            walk_again: false,
            library_walks: 0,
            walk_counted: false,
            retried: HashSet::new(),
            seen_analyzed: HashSet::new(),
            settings: None,
            limits: None,
            report_cap: None,
            last_recheck: None,
            probe: Probe::spawn(),
            folders: None,
            watch_status: idle_watch_cell(),
            tag_worker: TagWorker::spawn(Box::new(|| {}))
                .map_err(|e| {
                    tracing::error!(error = %e, "cannot start the tag worker; tags stay unread");
                })
                .ok(),
            tags_in_flight: HashSet::new(),
            tags_answered: HashSet::new(),
            bitrate_in_flight: HashSet::new(),
            bitrate_answered: HashSet::new(),
            loudness_in_flight: HashSet::new(),
            loudness_answered: HashSet::new(),
            header_reader: HeaderReader::spawn()
                .map_err(|e| {
                    tracing::error!(error = %e, "cannot start the header reader; lengths wait for the analysis");
                })
                .ok(),
            headers_asked: HashSet::new(),
            headers_in_flight: 0,
            headers_unsent: HashMap::new(),
            change_check: ChangeCheck::spawn(),
            change_urgent: UniqueQueue::default(),
            change_bulk: VecDeque::new(),
            change_sent: Vec::new(),
            change_started: false,
            in_use: HashSet::new(),
            look_pass: None,
            look_passes: 0,
            changed_found: None,
            changed_waiting: HashSet::new(),
            change_analysing: HashSet::new(),
            held: HashMap::new(),
            changed_retry: HashMap::new(),
            changed_retried: HashSet::new(),
            unsettled: HashSet::new(),
            clock: Instant::now(),
            faults: Arc::new(AtomicU64::new(0)),
            #[cfg(feature = "test-hooks")]
            fail_steps: 0,
            #[cfg(feature = "test-hooks")]
            fail_routes: 0,
            #[cfg(feature = "test-hooks")]
            gone_reports: 0,
            header_reads_sent: 0,
            saved_version: 0,
            dirty_since: None,
            save_failing: false,
            next_retry: None,
            last_session_save: None,
        }
    }

    /// Keeps the crash report cap in step with `limits.max_crash_reports`.
    pub fn with_report_cap(mut self, cap: crate::crash::ReportCap) -> Self {
        self.report_cap = Some(cap);
        self
    }

    /// Where the UI sends its requests.
    pub fn requests(&self) -> Sender<ServiceRequest> {
        self.request_tx.clone()
    }

    /// What the analysis is doing (main spec §6.1), updated every step.
    pub fn activity(&self) -> ActivityCell {
        self.activity.clone()
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

    /// Runs the tag pass on `state` alone. Used to test the pass against a
    /// snapshot older than its own answers.
    #[cfg(feature = "test-hooks")]
    pub fn tag_pass_on(&mut self, state: &AppState) {
        self.tag_pass(state, true);
    }

    /// How many folder scans were sent. Used to test that an idle watch
    /// does no file-system work.
    #[cfg(feature = "test-hooks")]
    pub fn folder_scans(&self) -> u64 {
        self.folders.as_ref().map_or(0, FolderWatch::scans)
    }

    /// How many steps walked the whole library. Used to test that an
    /// unchanged library is not walked again.
    #[cfg(feature = "test-hooks")]
    pub fn library_walks(&self) -> u64 {
        self.library_walks
    }

    /// How many header reads were sent. Used to test that a track is
    /// asked once.
    #[cfg(feature = "test-hooks")]
    pub fn header_reads_sent(&self) -> u64 {
        self.header_reads_sent
    }

    /// How many header reads were sent and not answered yet.
    #[cfg(feature = "test-hooks")]
    pub fn header_reads_in_flight(&self) -> usize {
        self.headers_in_flight
    }

    /// How many unreadable tracks have had their first look answered.
    #[cfg(feature = "test-hooks")]
    pub fn looked_at(&self) -> usize {
        self.stamps.values().filter(|s| **s != Seen::New).count()
    }

    /// How many times the activity was rebuilt.
    #[cfg(feature = "test-hooks")]
    pub fn activity_builds(&self) -> u64 {
        self.activity_builds
    }

    /// How many tracks wait with a Re-analyse urgent mark.
    #[cfg(feature = "test-hooks")]
    pub fn urgent_marks(&self) -> usize {
        self.urgent.len()
    }

    /// How many tracks wait, their file still changing at their retry,
    /// for a trigger (EE10).
    #[cfg(feature = "test-hooks")]
    pub fn unsettled_tracks(&self) -> usize {
        self.unsettled.len()
    }

    /// How many change checks wait or run (EE7).
    #[cfg(feature = "test-hooks")]
    pub fn change_checks_pending(&self) -> usize {
        let busy = self.change_check.as_ref().is_some_and(ChangeCheck::busy);
        self.change_urgent.len() + self.change_bulk.len() + usize::from(busy)
    }

    /// How many analysis results wait for their track to stop sounding
    /// (EE9).
    #[cfg(feature = "test-hooks")]
    pub fn held_results(&self) -> usize {
        self.held.len()
    }

    /// How many tracks a folder scan found gone after an event and waits
    /// to look at again (EE7.2).
    #[cfg(feature = "test-hooks")]
    pub fn folder_doubts(&self) -> usize {
        self.folders.as_ref().map_or(0, FolderWatch::doubts)
    }

    /// How many times an unreadable file was reported gone to the model.
    #[cfg(feature = "test-hooks")]
    pub fn gone_reports(&self) -> usize {
        self.gone_reports
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
            self.analysis_step(&state, now);
        }));
        let saving = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.autosave(&state, version, now);
        }));
        if analysis.is_err() {
            // Nothing is known about what the failed step did.
            self.walked = None;
        }
        for outcome in [analysis, saving] {
            if outcome.is_err() {
                self.faults.fetch_add(1, Ordering::AcqRel);
                tracing::error!("a services step panicked; continuing");
            }
        }
    }

    /// Watches the categories' folders (desktop). Without it nothing is
    /// watched.
    pub fn set_watcher(&mut self, watcher: Box<dyn FolderWatcher>) {
        self.folders = Some(FolderWatch::new(watcher, self.watch_status.clone()));
    }

    /// What the watched folders are doing.
    pub fn watch_status(&self) -> WatchCell {
        self.watch_status.clone()
    }

    /// EE7.3: the folder of a file opened in the external editor is
    /// followed for the rest of the run; its events go through the same
    /// debounce and scan as a watched folder's.
    fn follow_edited_file(&mut self, state: &AppState, id: TrackId) {
        let folder = state
            .library
            .get(id)
            .and_then(|t| t.path.parent())
            .map(Path::to_path_buf);
        match (&mut self.folders, folder) {
            (Some(folders), Some(folder)) => folders.follow_edited(folder),
            (None, _) => tracing::debug!(?id, "no folder watcher; the edited file is not followed"),
            (_, None) => tracing::debug!(?id, "an edited track not in the library is not followed"),
        }
    }

    /// One round of the folder watch: its commands go to the conductor and
    /// the files that came back are analysed at once.
    fn watch_step(&mut self, state: &AppState, now: Instant) {
        let Some(folders) = &mut self.folders else {
            return;
        };
        let outcome = folders.step(state, now);
        let mut unsent = Vec::new();
        for command in outcome.commands {
            // Order is kept: once one is refused, the rest wait with it.
            if !unsent.is_empty() {
                unsent.push(command);
            } else if let Err(refused) = self.conductor.offer_background(command) {
                unsent.push(refused);
            }
        }
        if !unsent.is_empty() {
            folders.requeue(unsent);
        }
        // EE7.2: a stamp check, as `CheckTrack`; an unchanged file gives
        // nothing.
        for id in outcome.touched {
            self.check_track(state, id);
        }
        for id in outcome.returned {
            if self.in_flight.contains(&id) {
                continue;
            }
            self.failed.remove(&id);
            self.forced.insert(id);
            self.walk_again = true;
        }
    }

    fn analysis_step(&mut self, state: &AppState, now: Instant) {
        #[cfg(feature = "test-hooks")]
        if self.fail_steps > 0 {
            self.fail_steps -= 1;
            #[allow(clippy::panic)]
            {
                panic!("injected services failure");
            }
        }
        self.clock = now;
        while let Ok(request) = self.requests.try_recv() {
            self.walk_again = true;
            match request {
                ServiceRequest::ReanalyseAll => self.reanalyse_all(state),
                ServiceRequest::AnalyseOutdated => self.analyse_outdated = true,
                ServiceRequest::ReanalyseTrack(id) => self.reanalyse_track(state, id),
                ServiceRequest::CheckTrack(id) => self.check_track(state, id),
                ServiceRequest::LookForChangedFiles => self.look_for_changed_files(state),
                ServiceRequest::FollowEditedFile(id) => self.follow_edited_file(state, id),
                ServiceRequest::RecheckUnreadable => {
                    let unreadable: Vec<TrackId> = state
                        .library
                        .iter()
                        .filter(|t| t.file_state == FileState::Unreadable)
                        .map(|t| t.id)
                        .collect();
                    for id in unreadable {
                        self.reanalyse_track(state, id);
                    }
                }
            }
        }
        self.follow_settings(state);
        self.follow_limits(state);
        self.recheck_missing(state, now);
        self.watch_step(state, now);
        self.retry_changed(state, now);
        self.change_step(state);
        let now_inputs = self.walk_inputs(state);
        let walk = std::mem::take(&mut self.walk_again)
            || !self
                .walked
                .as_ref()
                .is_some_and(|before| before.same_as(&now_inputs));
        self.submit_new(state, walk);
        self.header_pass(state, walk);
        self.tag_pass(state, walk);
        self.loudness_pass(state, walk);
        let wanted = now_inputs.wanted.clone();
        let results: Vec<AnalysisResult> = self.analyzer.results().try_iter().collect();
        if !results.is_empty() {
            self.walk_again = true;
        }
        for result in results {
            self.route_guarded(result, state, &wanted);
        }
        // Nothing queued and nothing to queue: the pass has nothing left
        // to wait for, whatever happened to its tracks.
        if self.in_flight.is_empty() && self.forced.is_empty() {
            self.library_pass.clear();
        }
        self.walked = Some(self.walk_inputs(state));
        if std::mem::take(&mut self.walk_counted) {
            self.library_walks += 1;
        }
        self.publish_activity();
    }

    /// Applies `result`. A result taken from the analyzer is gone from it: if
    /// applying it panics, the track is asked for again (once) instead of
    /// staying "in progress" until the next start.
    fn route_guarded(
        &mut self,
        result: AnalysisResult,
        state: &AppState,
        wanted: &HashSet<TrackId>,
    ) {
        let track = result.track;
        let routed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.route(result, state, wanted);
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

    /// External editor spec EE7, EE9: one round of the change checks. The
    /// first round checks every analysed track (EE7.1); later rounds check
    /// the tracks that came into use (EE7.4). Answers send the changed
    /// tracks to the analysis, unless they sound, and store the stamp of
    /// tracks that had none. There is no polling: nothing is checked
    /// unless something asks for it.
    fn change_step(&mut self, state: &AppState) {
        if self.change_check.is_none() {
            return;
        }
        self.take_change_answers(state);
        let in_use = Self::in_use(state);
        if self.change_started {
            let joined: Vec<TrackId> = in_use.difference(&self.in_use).copied().collect();
            // A check, or at once for a file left unsettled (R1).
            for id in joined {
                self.check_track(state, id);
            }
        } else {
            self.change_started = true;
            // The tracks in use first.
            for id in &in_use {
                self.check_track(state, *id);
            }
            self.change_bulk.extend(
                state
                    .library
                    .iter()
                    .filter(|t| Self::checkable(t) && !in_use.contains(&t.id))
                    .map(|t| t.id),
            );
        }
        self.in_use = in_use;
        self.release_changed(state);
        self.send_change_batch(state);
    }

    /// The tracks in use (EE7.4): the players' current, next and CUE
    /// entries, the carts of the page shown and the library pre-listen.
    fn in_use(state: &AppState) -> HashSet<TrackId> {
        let mut used = Self::on_players(state);
        used.extend(
            state
                .cartwall
                .shown_page()
                .into_iter()
                .flat_map(|page| page.carts.iter())
                .filter_map(|cart| cart.track),
        );
        used.extend(state.library_cue.track);
        used
    }

    /// Whether a track's file is worth a change check: one not analysed
    /// yet gets its analysis anyway, and missing and unreadable files keep
    /// their own rules.
    fn checkable(track: &fp_model::Track) -> bool {
        track.analyzed && track.file_state.is_playable()
    }

    /// *Look for changed files* (EE7.5): every analysed track is checked;
    /// the count is published when the last one is answered. Ignored while
    /// a pass runs.
    fn look_for_changed_files(&mut self, state: &AppState) {
        if self.change_check.is_none() {
            tracing::error!("the change check is not running; changed files cannot be looked for");
            return;
        }
        if self.look_pass.is_some() {
            tracing::debug!("look for changed files ignored: a pass is running");
            return;
        }
        self.look_passes += 1;
        // Files still changing at their retry are analysed now.
        let unsettled: Vec<TrackId> = self.unsettled.iter().copied().collect();
        for id in unsettled {
            self.take_up_changed(state, id);
        }
        let pending: HashSet<TrackId> = state
            .library
            .iter()
            .filter(|t| Self::checkable(t))
            .map(|t| t.id)
            .collect();
        self.change_bulk.extend(pending.iter().copied());
        self.look_pass = Some(LookPass {
            id: self.look_passes,
            pending,
            found: 0,
        });
        self.finish_look_pass();
    }

    /// Publishes the count of the pass when none of its tracks is left.
    fn finish_look_pass(&mut self) {
        if let Some(pass) = &self.look_pass
            && pass.pending.is_empty()
        {
            tracing::info!(found = pass.found, "looked for changed files");
            self.changed_found = Some((pass.id, pass.found));
            self.look_pass = None;
        }
    }

    fn take_change_answers(&mut self, state: &AppState) {
        let Some(check) = &mut self.change_check else {
            return;
        };
        let answers = check.answers();
        if check.gone() {
            self.change_check_stopped();
            return;
        }
        if answers.is_empty() {
            return;
        }
        let sent = std::mem::take(&mut self.change_sent);
        for answer in answers {
            for (track, stamp) in answer.unstamped {
                // A track gone or stamped meanwhile needs nothing: the
                // command would still make a new model version.
                let needed = state
                    .library
                    .get(track)
                    .is_some_and(|t| t.file_stamp.is_none());
                // Not queued: the next start stores it.
                if needed
                    && !self
                        .conductor
                        .send_background(Command::SetFileStamp { track, stamp })
                {
                    tracing::debug!(?track, "the stamp of a file could not be queued");
                }
            }
            if let Some(pass) = &mut self.look_pass {
                pass.found += answer
                    .changed
                    .iter()
                    .filter(|id| pass.pending.contains(id))
                    .count();
            }
            for id in answer.changed {
                self.on_changed(state, id);
            }
        }
        if let Some(pass) = &mut self.look_pass {
            for id in &sent {
                pass.pending.remove(id);
            }
        }
        self.finish_look_pass();
    }

    /// EE9: a changed track is analysed again at once, unless it sounds.
    fn on_changed(&mut self, state: &AppState, id: TrackId) {
        if state.library.get(id).is_none() {
            return;
        }
        tracing::info!(?id, "a library file changed; analysing it again");
        self.unsettled.remove(&id);
        if track_sounding(state, id) {
            self.changed_waiting.insert(id);
        } else {
            self.analyse_changed(state, id);
        }
    }

    /// Re-analyse of a changed track: urgent, through the cache (its key
    /// changed with the file).
    fn analyse_changed(&mut self, state: &AppState, id: TrackId) {
        self.reanalyse(state, id, false);
        self.change_analysing.insert(id);
        self.walk_again = true;
    }

    /// EE9: changed tracks and held results whose track stopped sounding go
    /// ahead.
    fn release_changed(&mut self, state: &AppState) {
        let known = |id: &TrackId| state.library.get(*id).is_some();
        self.changed_waiting.retain(known);
        self.change_analysing.retain(known);
        self.held.retain(|id, _| known(id));
        self.changed_retry.retain(|id, _| known(id));
        self.changed_retried.retain(known);
        self.unsettled.retain(known);
        let ready: Vec<TrackId> = self
            .changed_waiting
            .iter()
            .copied()
            .filter(|id| !track_sounding(state, *id))
            .collect();
        for id in ready {
            self.changed_waiting.remove(&id);
            // A result already held for it reads the changed file.
            if !self.held.contains_key(&id) {
                self.analyse_changed(state, id);
            }
        }
        let ready: Vec<TrackId> = self
            .held
            .keys()
            .copied()
            .filter(|id| !track_sounding(state, *id))
            .collect();
        if ready.is_empty() {
            return;
        }
        let wanted = Self::wanted(state);
        for id in ready {
            if let Some(result) = self.held.remove(&id) {
                self.walk_again = true;
                self.route_guarded(result, state, &wanted);
            }
        }
    }

    /// Sends the next batch when the change check is free: tracks in use
    /// first.
    fn send_change_batch(&mut self, state: &AppState) {
        if self.change_check.as_ref().is_none_or(ChangeCheck::busy) {
            return;
        }
        let mut batch = Vec::new();
        let mut seen = HashSet::new();
        // Tracks being read for something else (a cart's peaks, say): looked
        // at once that read is over. (urgent, id)
        let mut later: Vec<(bool, TrackId)> = Vec::new();
        while batch.len() < CHANGE_BATCH {
            let (urgent, id) = match self.change_urgent.pop_front() {
                Some(id) => (true, id),
                None => match self.change_bulk.pop_front() {
                    Some(id) => (false, id),
                    None => break,
                },
            };
            if !seen.insert(id) {
                continue;
            }
            let Some(t) = state.library.get(id).filter(|t| Self::checkable(t)) else {
                if let Some(pass) = &mut self.look_pass {
                    pass.pending.remove(&id);
                }
                continue;
            };
            // A change already found and not applied yet counts as found.
            if self.changed_waiting.contains(&id)
                || self.held.contains_key(&id)
                || self.change_analysing.contains(&id)
            {
                if let Some(pass) = &mut self.look_pass
                    && pass.pending.remove(&id)
                {
                    pass.found += 1;
                }
                continue;
            }
            if self.in_flight.contains(&id) {
                // A read applied to the model reads the file as it is now:
                // for a pass it counts as checked. A check asked for now
                // (an event, a use), or a read only for a waveform, waits
                // for it to end. Each track is popped once per step.
                if urgent || self.peaks_only.contains(&id) {
                    later.push((urgent, id));
                } else if let Some(pass) = &mut self.look_pass {
                    pass.pending.remove(&id);
                }
                continue;
            }
            batch.push((id, t.path.clone(), t.file_stamp));
        }
        for (urgent, id) in later {
            if urgent {
                self.change_urgent.push_back(id);
            } else {
                self.change_bulk.push_back(id);
            }
        }
        if batch.is_empty() {
            self.finish_look_pass();
            return;
        }
        let ids: Vec<TrackId> = batch.iter().map(|(id, _, _)| *id).collect();
        let Some(check) = &mut self.change_check else {
            return;
        };
        if check.send(batch) {
            self.change_sent = ids;
        } else {
            self.change_check_stopped();
        }
    }

    /// The change-check thread is gone: nothing more is looked at, and a
    /// running look ends without a count (so `looking` does not stick).
    fn change_check_stopped(&mut self) {
        tracing::error!("the change check stopped; changed files go unnoticed");
        self.change_check = None;
        self.change_urgent.clear();
        self.change_bulk.clear();
        self.change_sent.clear();
        self.look_pass = None;
    }

    /// Replaces the change-check thread (tests).
    #[cfg(feature = "test-hooks")]
    pub fn set_change_check(&mut self, check: ChangeCheck) {
        self.change_check = Some(check);
    }

    /// The inputs of the library walks now (see `WalkInputs`).
    fn walk_inputs(&self, state: &AppState) -> WalkInputs {
        let on_players = Self::on_players(state);
        let wanted = Self::wanted(state);
        WalkInputs {
            library: state.library.clone(),
            on_players,
            on_carts: Self::cart_tracks(state),
            carts_drawn: Self::cart_waveforms(state, false),
            wanted,
            analyse_outdated: self.analyse_outdated,
            sizes: [
                self.in_flight.len(),
                self.peaks_only.len(),
                self.shown_only.len(),
                self.done.len(),
                self.failed.len(),
                self.forced.len(),
                self.fresh.len(),
                self.urgent.len(),
                self.library_pass.len(),
                self.tags_in_flight.len(),
                self.tags_answered.len(),
                self.bitrate_in_flight.len(),
                self.bitrate_answered.len(),
                self.loudness_in_flight.len(),
                self.loudness_answered.len(),
                self.headers_asked.len(),
                self.headers_unsent.len() + self.headers_in_flight,
            ],
        }
    }

    /// Main spec §6.1: stores what the analysis is doing when it changed.
    /// Reads of an analysed track for its waveform are not activity. On a
    /// large library nothing is rebuilt while the pool, the forced tracks
    /// and the pass stay as they were.
    fn publish_activity(&mut self) {
        let key = (
            self.analyzer.activity_version(),
            self.forced.len(),
            self.shown_only.len(),
            self.library_pass.len(),
            self.pass_total,
            self.changed_found,
            self.look_pass.is_some(),
        );
        if self.activity_key == Some(key) {
            return;
        }
        self.activity_key = Some(key);
        self.activity_builds += 1;
        let pool = self.analyzer.activity();
        let counted = |id: &TrackId| !self.shown_only.contains(id);
        let running: Vec<TrackId> = pool.running.into_iter().filter(counted).collect();
        let queued: HashSet<TrackId> = pool
            .queued
            .into_iter()
            .filter(counted)
            .chain(self.forced.iter().copied())
            .filter(|id| !running.contains(id))
            .collect();
        let pass = (!self.library_pass.is_empty()).then(|| PassProgress {
            done: self.pass_total.saturating_sub(self.library_pass.len()),
            total: self.pass_total,
        });
        let activity = AnalysisActivity {
            queued,
            running,
            loudness: pool.loudness,
            pass,
            changed_found: self.changed_found,
            looking: self.look_pass.is_some(),
        };
        if **self.activity.load() != activity {
            self.activity.store(Arc::new(activity));
        }
    }

    /// Final save, with the positions the engine reports right now.
    pub fn shutdown(&mut self) {
        let state = self.conductor.model.load_full();
        // The final save always tries once, whatever the retry throttle says.
        let now = Instant::now();
        let result = self.save_all(&state, now);
        self.note_save(&state, result, now);
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
            thread: Mutex::new(Some(thread)),
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
            self.restart_analysis(state, false);
            self.walk_again = true;
        }
        self.settings = Some(current.clone());
    }

    /// Live settings spec §6: new cover and tag limits reach the analysis
    /// and its cache for jobs that start after them; nothing is analysed
    /// again.
    fn follow_limits(&mut self, state: &AppState) {
        let current = &state.config.limits;
        if self.limits.as_ref() == Some(current) {
            return;
        }
        self.analyzer.update_limits(current.clone());
        if let Some(cap) = &self.report_cap {
            cap.set(current.max_crash_reports);
        }
        self.limits = Some(current.clone());
    }

    /// The limits the analysis has now. Used by tests.
    #[cfg(feature = "test-hooks")]
    pub fn analyzer_limits(&self) -> fp_model::Limits {
        self.analyzer.limits()
    }

    /// *Re-analyse all* (main spec §6.1): ignored while a library pass
    /// runs; otherwise every track is analysed again from its file.
    fn reanalyse_all(&mut self, state: &AppState) {
        if !self.library_pass.is_empty() {
            tracing::debug!("re-analyse all ignored: a library pass is running");
            return;
        }
        self.restart_analysis(state, true);
    }

    /// Analyses every track again as a library pass; running analyses are
    /// cancelled so that none finishes with outdated settings.
    /// `fresh`: *Re-analyse all*, which reads every file again. A settings
    /// change during such a pass keeps the restarted pass fresh.
    fn restart_analysis(&mut self, state: &AppState, fresh: bool) {
        self.pass_fresh = fresh || (self.pass_fresh && !self.library_pass.is_empty());
        if self.pass_fresh {
            self.fresh = state.library.iter().map(|t| t.id).collect();
        }
        for id in self.in_flight.drain() {
            self.analyzer.cancel(id);
        }
        self.peaks_only.clear();
        self.shown_only.clear();
        self.done.clear();
        self.failed.clear();
        self.retried.clear();
        self.stamps.clear();
        self.failed_at.clear();
        self.urgent.clear();
        self.changed_retry.clear();
        self.changed_retried.clear();
        self.unsettled.clear();
        self.forced = state.library.iter().map(|t| t.id).collect();
        self.library_pass = self.forced.clone();
        self.pass_total = self.library_pass.len();
    }

    /// The row menu's Re-analyse: forget the failure and analyse this track
    /// now from its file, ahead of the library. Manual markers are kept by
    /// the model. A track already queued or running is left alone (main
    /// spec §6.1); one read only for its waveform is analysed.
    fn reanalyse_track(&mut self, state: &AppState, id: TrackId) {
        self.reanalyse(state, id, true);
    }

    /// `reanalyse_track`; `fresh` skips the cache lookup.
    fn reanalyse(&mut self, state: &AppState, id: TrackId, fresh: bool) {
        if state.library.get(id).is_none() {
            return;
        }
        if self.in_flight.contains(&id) && !self.shown_only.contains(&id) {
            tracing::debug!(?id, "re-analyse ignored: the track is queued or running");
            return;
        }
        if self.in_flight.remove(&id) {
            self.analyzer.cancel(id);
        }
        self.peaks_only.remove(&id);
        self.shown_only.remove(&id);
        self.done.remove(&id);
        self.failed.remove(&id);
        self.retried.remove(&id);
        self.stamps.remove(&id);
        self.failed_at.remove(&id);
        // A result held for the track (EE9) or a retry waiting for its file
        // to settle (EE10) is superseded by this analysis.
        self.held.remove(&id);
        self.changed_retry.remove(&id);
        self.unsettled.remove(&id);
        self.forced.insert(id);
        if fresh {
            self.fresh.insert(id);
        }
        self.urgent.insert(id);
    }

    /// Every `tuning.missing_recheck_ms`, the probe thread looks again for
    /// files not found (a drive not mounted yet) and at unreadable files
    /// (operator feedback 4, Q10). A missing file found goes to the analysis
    /// pool, which answers from the cache when it was analysed before. An
    /// unreadable file is only `stat`ed, never opened: its size and
    /// modification time are the ones the failed analysis read (or, for a
    /// playback failure, the first look's), and a change of either clears
    /// the failure and analyses it again; one that
    /// disappeared becomes missing. Neither this thread nor the pool waits
    /// on a share that is offline, so autosave and real analyses go on.
    fn recheck_missing(&mut self, state: &AppState, now: Instant) {
        let Some(probe) = &mut self.probe else {
            return;
        };
        loop {
            match probe.answers.try_recv() {
                Ok(answer) => {
                    probe.busy = false;
                    self.walk_again = true;
                    // One in flight already (Re-analyse all) needs no second
                    // analysis.
                    for id in answer
                        .found
                        .into_iter()
                        .filter(|id| !self.in_flight.contains(id))
                    {
                        self.failed.remove(&id);
                        self.forced.insert(id);
                    }
                    for (id, seen) in answer.recorded {
                        // Only a track still waiting for its look: one that
                        // was re-analysed meanwhile starts again from `New`.
                        if let Some(known) = self.stamps.get_mut(&id) {
                            *known = seen;
                        }
                    }
                    for id in answer
                        .changed
                        .into_iter()
                        .filter(|id| !self.in_flight.contains(id))
                    {
                        self.stamps.remove(&id);
                        self.failed.remove(&id);
                        self.forced.insert(id);
                    }
                    for id in answer.gone {
                        // Q10.3: it follows the missing-file recheck from
                        // here.
                        self.stamps.remove(&id);
                        if self.gone.contains(&id) {
                            continue;
                        }
                        #[cfg(feature = "test-hooks")]
                        {
                            self.gone_reports += 1;
                        }
                        // Not queued: the next look reports it again.
                        if self.conductor.send_background(Command::SetFileState {
                            track: id,
                            state: FileState::Missing,
                        }) {
                            self.gone.insert(id);
                        }
                    }
                }
                Err(crossbeam_channel::TryRecvError::Empty) => break,
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    tracing::error!("the file probe stopped; missing files stay missing");
                    self.probe = None;
                    return;
                }
            }
        }
        // Every unreadable track, from the analysis or from playback, has an
        // entry; one that is not unreadable any more loses it.
        self.stamps.retain(|id, _| {
            state
                .library
                .get(*id)
                .is_some_and(|t| t.file_state == FileState::Unreadable)
        });
        self.gone.retain(|id| {
            state
                .library
                .get(*id)
                .is_some_and(|t| t.file_state == FileState::Unreadable)
        });
        // Depends on the library, `failed_at` (filled when a result is
        // routed, which sets `walk_again`) and `gone` (changed by a probe
        // answer, which sets it too): nothing to do while they are as they
        // were.
        let library_same = self
            .walked
            .as_ref()
            .is_some_and(|w| w.library.shares_storage(&state.library));
        if self.walk_again || !library_same {
            self.walk_counted = true;
            for t in state
                .library
                .iter()
                .filter(|t| t.file_state == FileState::Unreadable && !self.gone.contains(&t.id))
            {
                // What the failed analysis read, when it could `stat` the
                // file; otherwise (and for a playback failure) a first look.
                match self.failed_at.remove(&t.id) {
                    Some(stamp) => {
                        self.stamps.insert(t.id, Seen::At(stamp));
                    }
                    None => {
                        self.stamps.entry(t.id).or_insert(Seen::New);
                    }
                }
            }
        }
        let interval =
            Duration::from_secs_f64(state.config.tuning.missing_recheck_ms.max(0.0) / 1000.0);
        let due = match self.last_recheck {
            Some(last) => now.saturating_duration_since(last) >= interval,
            None => {
                self.last_recheck = Some(now);
                false
            }
        };
        // A failure without the analysis's stat (a playback failure, or a
        // `stat` that failed) is looked at once, without waiting for the
        // timer, so that the recorded stat is close to the failure.
        let first_look = self
            .stamps
            .iter()
            .any(|(id, s)| *s == Seen::New && !self.in_flight.contains(id));
        // One look at a time: a hung mount only delays the next one.
        if probe.busy || !(due || first_look) {
            return;
        }
        if due {
            self.last_recheck = Some(now);
        }
        let missing: Vec<(TrackId, PathBuf)> = if due {
            state
                .library
                .iter()
                .filter(|t| t.file_state == FileState::Missing && !self.in_flight.contains(&t.id))
                .map(|t| (t.id, t.path.clone()))
                .collect()
        } else {
            Vec::new()
        };
        let unreadable: Vec<(TrackId, PathBuf, Seen)> = self
            .stamps
            .iter()
            .filter(|(id, seen)| !self.in_flight.contains(id) && (due || **seen == Seen::New))
            .filter_map(|(id, seen)| state.library.get(*id).map(|t| (*id, t.path.clone(), *seen)))
            .collect();
        if missing.is_empty() && unreadable.is_empty() {
            return;
        }
        if probe
            .paths
            .send(Job {
                missing,
                unreadable,
            })
            .is_ok()
        {
            probe.busy = true;
        } else {
            tracing::error!("the file probe stopped; missing files stay missing");
            self.probe = None;
        }
    }

    /// Tracks the screen draws: the players' tracks and the carts of the
    /// page shown (cart colour spec CC8).
    fn wanted(state: &AppState) -> HashSet<TrackId> {
        let mut wanted = Self::on_players(state);
        wanted.extend(Self::cart_waveforms(state, true));
        // The library window's preview bar draws its track's waveform.
        if state.library_view.open {
            wanted.extend(state.library_cue.track);
        }
        wanted
    }

    /// The current, next and cue entries' tracks of the players.
    fn on_players(state: &AppState) -> HashSet<TrackId> {
        state
            .players
            .iter()
            .flat_map(|p| [p.current, p.next, p.cue.map(|c| c.entry)])
            .flatten()
            .filter_map(|e| state.playlists.entry(e))
            .map(|e| e.track)
            .collect()
    }

    /// The playable carts' tracks that draw a waveform: those of the page
    /// shown on an expanded cartwall (`shown_only`), or of every page.
    fn cart_waveforms(state: &AppState, shown_only: bool) -> HashSet<TrackId> {
        let pages: Vec<&fp_model::CartPage> = if shown_only {
            state
                .cartwall
                .shown_page()
                .filter(|_| state.cartwall.open)
                .into_iter()
                .collect()
        } else {
            state.cartwall.pages.iter().collect()
        };
        pages
            .into_iter()
            .flat_map(|page| page.carts.iter())
            .filter_map(|cart| cart.track)
            // A cart that cannot play draws no waveform: no peaks for it.
            .filter(|t| {
                state
                    .library
                    .get(*t)
                    .is_some_and(|t| t.file_state.is_playable())
            })
            .collect()
    }

    /// Tracks assigned to a cart.
    fn cart_tracks(state: &AppState) -> HashSet<TrackId> {
        state
            .cartwall
            .pages
            .iter()
            .flat_map(|page| page.carts.iter())
            .filter_map(|cart| cart.track)
            .collect()
    }

    /// The tag-only pass: tracks analysed but not read since (new ones, ones
    /// of an older library, re-analysed ones) get their tags read on the tag
    /// worker. A read that cannot parse the file still answers, with the
    /// file-name title and empty fields, so a track is never asked twice for
    /// the same analysis.
    ///
    /// The bit-rate-only pass rides on the same worker: tracks analysed
    /// before the format carried a bit rate (`needs_bitrate_read`) get it
    /// read from the file's header, once per analysis.
    fn tag_pass(&mut self, state: &AppState, walk: bool) {
        let Some(worker) = &self.tag_worker else {
            return;
        };
        self.tags_in_flight
            .retain(|id| state.library.get(*id).is_some());
        // An answer stays here until a snapshot shows it (`tags_read`), so a
        // later reset (a new analysis) brings the read back.
        self.tags_answered
            .retain(|id| state.library.get(*id).is_some_and(|t| !t.tags_read));
        self.bitrate_in_flight
            .retain(|id| state.library.get(*id).is_some());
        self.bitrate_answered
            .retain(|id| state.library.get(*id).is_some_and(|t| !t.bitrate_read));
        if walk {
            self.walk_counted = true;
            for track in state.library.iter() {
                if track.needs_bitrate_read()
                    && !self.bitrate_answered.contains(&track.id)
                    && self.bitrate_in_flight.insert(track.id)
                {
                    let job = TagJob::ReadBitrate {
                        track: track.id,
                        path: track.path.clone(),
                    };
                    if !worker.submit(job) {
                        tracing::error!("the tag worker stopped; bit rates stay unread");
                        self.tag_worker = None;
                        return;
                    }
                }
                if track.needs_tag_read()
                    && !self.tags_answered.contains(&track.id)
                    && self.tags_in_flight.insert(track.id)
                {
                    let job = TagJob::Read {
                        track: track.id,
                        path: track.path.clone(),
                        limits: state.config.limits.clone(),
                    };
                    if !worker.submit(job) {
                        tracing::error!("the tag worker stopped; tags stay unread");
                        self.tag_worker = None;
                        return;
                    }
                }
            }
        }
        let outcomes: Vec<TagOutcome> = worker.results().try_iter().collect();
        if !outcomes.is_empty() {
            self.walk_again = true;
        }
        for outcome in outcomes {
            match outcome {
                TagOutcome::Read { track, tags } => {
                    // Answered either way. If the command is not queued the
                    // track still needs its read, so the next round asks
                    // again.
                    self.tags_in_flight.remove(&track);
                    if self.conductor.send_background(Command::ApplyTags {
                        track,
                        tags: Box::new(tags),
                    }) {
                        self.tags_answered.insert(track);
                    }
                }
                TagOutcome::Bitrate { track, bitrate } => {
                    self.bitrate_in_flight.remove(&track);
                    if self
                        .conductor
                        .send_background(Command::ApplyBitrate { track, bitrate })
                    {
                        self.bitrate_answered.insert(track);
                    }
                }
                _ => {}
            }
        }
    }

    /// N6b: tracks analysed before loudness was measured (or by an older
    /// `LOUDNESS_VERSION`) get a loudness-only measurement, below every
    /// normal analysis: nothing is queued while a normal job is in flight,
    /// and never more than the pool's share at once. Failures and files
    /// too short to measure are recorded too, so they are not retried; a
    /// file gone since its analysis is marked missing instead, and measured
    /// when it comes back. It runs whether normalization is on or off.
    fn loudness_pass(&mut self, state: &AppState, walk: bool) {
        let answers: Vec<LoudnessResult> = self.analyzer.loudness_results().try_iter().collect();
        if !answers.is_empty() {
            self.walk_again = true;
        }
        for answer in answers {
            // A job whose track went to a full analysis meanwhile is stale:
            // the analysis's measurement must not be overwritten.
            if !self.loudness_in_flight.remove(&answer.track) {
                continue;
            }
            let Some(track) = state.library.get(answer.track) else {
                continue; // removed meanwhile: nothing to record
            };
            let command = match answer.outcome {
                Ok(loudness) => Command::ApplyLoudness {
                    track: answer.track,
                    loudness,
                    version: LOUDNESS_VERSION,
                },
                // The measurement never reports a changed file; nothing
                // to record either way.
                Err(AnalysisError::Cancelled | AnalysisError::Changed) => continue,
                Err(AnalysisError::Missing) => {
                    if track.file_state == FileState::Missing {
                        continue;
                    }
                    Command::SetFileState {
                        track: answer.track,
                        state: FileState::Missing,
                    }
                }
                Err(AnalysisError::Unreadable(reason)) => {
                    tracing::warn!(
                        path = %answer.path.display(),
                        %reason,
                        "cannot measure the loudness"
                    );
                    Command::ApplyLoudness {
                        track: answer.track,
                        loudness: None,
                        version: LOUDNESS_VERSION,
                    }
                }
            };
            // If the command is not queued the track still needs the pass,
            // so a later round measures it again.
            if self.conductor.send_background(command) {
                self.loudness_answered.insert(answer.track);
            }
        }
        let gone: Vec<TrackId> = self
            .loudness_in_flight
            .iter()
            .copied()
            .filter(|id| {
                state
                    .library
                    .get(*id)
                    .is_none_or(|t| !t.needs_loudness_pass())
            })
            .collect();
        for id in gone {
            self.analyzer.cancel_loudness(id);
            self.loudness_in_flight.remove(&id);
        }
        // An answer stays here until a snapshot shows it.
        self.loudness_answered.retain(|id| {
            state
                .library
                .get(*id)
                .is_some_and(|t| t.needs_loudness_pass())
        });
        // Below every normal job: only once the normal queue is empty (a
        // full analysis measures its track anyway).
        if !self.in_flight.is_empty() {
            return;
        }
        let room = self
            .analyzer
            .loudness_share()
            .saturating_sub(self.loudness_in_flight.len());
        if room == 0 || !walk {
            return;
        }
        self.walk_counted = true;
        let todo: Vec<(TrackId, PathBuf)> = state
            .library
            .iter()
            .filter(|t| {
                t.needs_loudness_pass()
                    && !self.loudness_in_flight.contains(&t.id)
                    && !self.loudness_answered.contains(&t.id)
            })
            .take(room)
            .map(|t| (t.id, t.path.clone()))
            .collect();
        // The share is full: there may be more waiting.
        if todo.len() >= room {
            self.walk_again = true;
        }
        for (id, path) in todo {
            self.analyzer.submit_loudness(id, path);
            self.loudness_in_flight.insert(id);
        }
    }

    /// Cancels the loudness-only job of `id`, if any, so that its answer is
    /// never applied over a full analysis's measurement.
    fn drop_loudness_job(&mut self, id: TrackId) {
        if self.loudness_in_flight.remove(&id) {
            self.analyzer.cancel_loudness(id);
        }
    }

    /// Operator feedback 4, Q1.1: tracks with no length and no analysis get
    /// the length their file's header declares, read on the header reader,
    /// the ones on a player first. Each track is read once per session; a
    /// length whose command could not be queued is sent on a later round.
    fn header_pass(&mut self, state: &AppState, walk: bool) {
        let Some(reader) = &self.header_reader else {
            return;
        };
        // Lengths still waiting go first, while their track still needs one.
        let conductor = &self.conductor;
        self.headers_unsent.retain(|track, secs| {
            state
                .library
                .get(*track)
                .is_some_and(|t| t.needs_header_duration())
                && !conductor.send_background(Command::SetDuration {
                    track: *track,
                    secs: *secs,
                })
        });
        let answers: Vec<(TrackId, Option<f64>)> = reader.answers().try_iter().collect();
        if !answers.is_empty() {
            self.walk_again = true;
        }
        for (track, secs) in answers {
            self.headers_in_flight = self.headers_in_flight.saturating_sub(1);
            // A length that comes after the analysis (or after the file was
            // found missing or unreadable) is dropped: the model would ignore
            // it, but the command would still make a new model version, so a
            // new snapshot and a save for nothing.
            let needed = state
                .library
                .get(track)
                .is_some_and(|t| t.needs_header_duration());
            if let Some(secs) = secs
                && needed
                && !self
                    .conductor
                    .send_background(Command::SetDuration { track, secs })
            {
                // The queue is full: keep the length for a later round.
                self.headers_unsent.insert(track, secs);
            }
        }
        self.headers_asked
            .retain(|id| state.library.get(*id).is_some());
        if !walk {
            return;
        }
        self.walk_counted = true;
        let wanted = Self::wanted(state);
        let mut todo: Vec<&fp_model::Track> = state
            .library
            .iter()
            .filter(|t| t.needs_header_duration() && !self.headers_asked.contains(&t.id))
            .collect();
        // The tracks on a player first; the sort is stable.
        todo.sort_by_key(|t| !wanted.contains(&t.id));
        let room = HEADER_READS_IN_FLIGHT.saturating_sub(self.headers_in_flight);
        // Reads wait for room: look again when one is answered.
        // (With no room, the answer that frees one asks for the walk.)
        if room > 0 && todo.len() > room {
            self.walk_again = true;
        }
        for track in todo.into_iter().take(room) {
            if !reader.submit(track.id, track.path.clone()) {
                tracing::error!("the header reader stopped; lengths wait for the analysis");
                self.header_reader = None;
                return;
            }
            self.headers_asked.insert(track.id);
            self.headers_in_flight += 1;
            self.header_reads_sent += 1;
        }
    }

    fn submit_new(&mut self, state: &AppState, walk: bool) {
        if !walk {
            return;
        }
        self.walk_counted = true;
        let known: HashSet<TrackId> = state.library.iter().map(|t| t.id).collect();
        for gone in self.in_flight.difference(&known) {
            self.analyzer.cancel(*gone);
        }
        self.in_flight.retain(|id| known.contains(id));
        self.done.retain(|id| known.contains(id));
        self.failed.retain(|id| known.contains(id));
        self.forced.retain(|id| known.contains(id));
        self.shown_only.retain(|id| self.in_flight.contains(id));
        self.fresh.retain(|id| known.contains(id));
        self.library_pass.retain(|id| known.contains(id));
        self.retried.retain(|id| known.contains(id));
        self.stamps.retain(|id, _| known.contains(id));
        self.failed_at.retain(|id, _| known.contains(id));
        self.urgent.retain(|id| known.contains(id));
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
        // Peaks and covers are kept for what the screen shows (the players and
        // the cart page shown) and for every cart, so going back to a page
        // reads nothing again; the analysis cache brings a player's back
        // when its track is shown again.
        let on_players = Self::on_players(state);
        let wanted = Self::wanted(state);
        let carts_drawn = Self::cart_waveforms(state, false);
        self.media
            .retain(|id| wanted.contains(&id) || carts_drawn.contains(&id));
        let on_carts = Self::cart_tracks(state);
        // The players' tracks first: a page of carts asked for at once does
        // not hold back a player's waveform.
        for players_pass in [true, false] {
            for track in state.library.iter() {
                if on_players.contains(&track.id) == players_pass {
                    self.consider(track, &wanted, &on_players, &on_carts);
                }
            }
        }
    }

    /// Submits `track` to the analysis when it needs one, or when the screen
    /// draws it and its peaks are not in the media cache.
    fn consider(
        &mut self,
        track: &fp_model::Track,
        wanted: &HashSet<TrackId>,
        on_players: &HashSet<TrackId>,
        on_carts: &HashSet<TrackId>,
    ) {
        let id = track.id;
        // Its new analysis waits for it to stop sounding (EE9), or for its
        // file to settle (EE10).
        if self.held.contains_key(&id)
            || self.changed_retry.contains_key(&id)
            || self.unsettled.contains(&id)
        {
            return;
        }
        if self.in_flight.contains(&id) {
            // Loaded on a player while queued with the library: its
            // waveform is needed now.
            if wanted.contains(&id) {
                self.analyzer.promote(id, &track.path);
            }
            return;
        }
        // Tracks an earlier version analysed are analysed again, once,
        // when the operator asks; the ones on screen are anyway (`show`).
        // A cart's track without a format is analysed at once: the cart
        // bus opens bit-perfect only for a known format, and a cart must
        // not wait for the operator's answer to play without resampling.
        let stale = outdated(track)
            && (self.analyse_outdated || (track.format.is_none() && on_carts.contains(&id)));
        let analyse = self.forced.contains(&id)
            || ((!track.analyzed || stale)
                && !self.done.contains(&id)
                && !self.failed.contains(&id));
        let show = wanted.contains(&id) && !self.media.contains(id) && !self.failed.contains(&id);
        if analyse || show {
            self.forced.remove(&id);
            self.in_flight.insert(id);
            // A cart's analysed track read only for its waveform: the
            // model already has its analysis.
            if !analyse && track.analyzed {
                self.shown_only.insert(id);
            } else {
                self.shown_only.remove(&id);
            }
            if !analyse && track.analyzed && !on_players.contains(&id) {
                self.peaks_only.insert(id);
            } else {
                self.peaks_only.remove(&id);
                // The analysis measures the loudness too.
                self.drop_loudness_job(id);
            }
            // The mark is used up whichever way the track goes urgent.
            let asked = self.urgent.remove(&id);
            // An explicit re-analysis reads the file (I11); a read for the
            // peaks alone keeps the mark for the analysis.
            let fresh = analyse && self.fresh.remove(&id);
            let path = track.path.clone();
            match (wanted.contains(&id) || asked, fresh) {
                (true, false) => self.analyzer.submit_urgent(id, path),
                (true, true) => self.analyzer.submit_urgent_fresh(id, path),
                (false, false) => self.analyzer.submit(id, path),
                (false, true) => self.analyzer.submit_fresh(id, path),
            }
        }
    }

    fn route(&mut self, result: AnalysisResult, state: &AppState, wanted: &HashSet<TrackId>) {
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
        if matches!(result.outcome, Err(AnalysisError::Changed)) {
            self.changed_during_analysis(state, &result);
            return;
        }
        self.changed_retried.remove(&result.track);
        self.unsettled.remove(&result.track);
        let stamp = result.stamp;
        if self.hold_while_sounding(&result, state) {
            // EE9: it waits for the track to stop sounding; a waveform
            // shows meanwhile, but one already shown (the audio that
            // plays) is kept.
            if wanted.contains(&result.track)
                && !self.media.contains(result.track)
                && let Ok(analysis) = &result.outcome
            {
                self.media.insert(
                    result.track,
                    TrackMedia {
                        peaks: analysis.peaks.clone(),
                        peak_bucket_secs: analysis.peak_bucket_secs,
                        cover_png: analysis.cover_png.clone().map(Arc::from),
                    },
                );
            }
            self.shown_only.remove(&result.track);
            self.held.insert(result.track, result);
            return;
        }
        // An answer, whatever it is, settles the track in the pass; a
        // command that cannot be queued puts it back below.
        let in_pass = self.library_pass.remove(&result.track);
        self.shown_only.remove(&result.track);
        let peaks_only = self.peaks_only.remove(&result.track)
            && state.library.get(result.track).is_some_and(|t| t.analyzed);
        if peaks_only && let Ok(analysis) = &result.outcome {
            if wanted.contains(&result.track) {
                self.media.insert(
                    result.track,
                    TrackMedia {
                        peaks: analysis.peaks.clone(),
                        peak_bucket_secs: analysis.peak_bucket_secs,
                        cover_png: analysis.cover_png.clone().map(Arc::from),
                    },
                );
            }
            return;
        }
        self.done.insert(result.track);
        self.failed_at.remove(&result.track);
        let command = match result.outcome {
            Ok(analysis) => {
                self.failed.remove(&result.track);
                self.drop_loudness_job(result.track);
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
                // It resets `tags_read`: the tags are read again.
                self.tags_answered.remove(&result.track);
                // What the analysis read (EE6): a later check compares the
                // file with it.
                let mut applied = analysis.analysis;
                applied.file_stamp =
                    stamp.and_then(|(size, modified)| FileStamp::from_metadata(size, modified));
                Command::ApplyAnalysis {
                    track: result.track,
                    analysis: Box::new(applied),
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
                if let Some(stamp) = result.stamp {
                    self.failed_at.insert(result.track, stamp);
                }
                tracing::warn!(path = %result.path.display(), %reason, "unreadable file");
                Command::SetFileState {
                    track: result.track,
                    state: FileState::Unreadable,
                }
            }
            // Both were handled above.
            Err(AnalysisError::Cancelled | AnalysisError::Changed) => return,
        };
        // A file still missing changes nothing: no new model version, so
        // nothing is saved every recheck.
        if let Command::SetFileState {
            track,
            state: file_state,
        } = &command
            && state.library.get(*track).map(|t| t.file_state) == Some(*file_state)
        {
            return;
        }
        if !self.conductor.send_background(command) {
            // The queue is full: try again on a later round.
            self.done.remove(&result.track);
            self.forced.insert(result.track);
            if in_pass {
                self.library_pass.insert(result.track);
            }
        }
    }

    /// EE10: the file changed while it was analysed (an editor still
    /// writing it), so nothing from that read is applied. The track is
    /// analysed once more after the watch debounce interval; if its file
    /// still moved then, it waits for the next event, use or look (no
    /// polling).
    fn changed_during_analysis(&mut self, state: &AppState, result: &AnalysisResult) {
        let id = result.track;
        // The pass has its answer: the retry is not part of it.
        self.library_pass.remove(&id);
        self.shown_only.remove(&id);
        self.peaks_only.remove(&id);
        // Not a change waiting to be applied any more: a later check finds
        // it again (the retry asks for it anew).
        self.change_analysing.remove(&id);
        if self.changed_retried.insert(id) {
            tracing::info!(path = %result.path.display(), "the file changed while it was analysed; analysing it again shortly");
            let quiet = Duration::from_millis(state.config.library.watch_debounce_ms);
            self.changed_retry.insert(id, self.clock + quiet);
        } else {
            tracing::info!(path = %result.path.display(), "the file is still changing; it is analysed on its next change");
            self.unsettled.insert(id);
        }
    }

    /// EE10: the retries whose time came (see `take_up_changed`).
    fn retry_changed(&mut self, state: &AppState, now: Instant) {
        if self.changed_retry.is_empty() {
            return;
        }
        let due: Vec<TrackId> = self
            .changed_retry
            .iter()
            .filter(|(_, at)| now >= **at)
            .map(|(id, _)| *id)
            .collect();
        for id in due {
            self.changed_retry.remove(&id);
            self.take_up_changed(state, id);
        }
    }

    /// Analyses a track whose file changed: an analysed one goes through
    /// the changed-file path (EE9: not while it sounds); one never
    /// analysed is analysed.
    fn take_up_changed(&mut self, state: &AppState, id: TrackId) {
        self.unsettled.remove(&id);
        match state.library.get(id) {
            Some(t) if t.analyzed => self.on_changed(state, id),
            Some(_) => {
                self.reanalyse(state, id, false);
                self.walk_again = true;
            }
            None => {}
        }
    }

    /// An event on a known track or `CheckTrack`: a stamp check, or, for a
    /// track whose file was still changing at its retry (whose stored stamp
    /// may be none at all), an analysis at once.
    fn check_track(&mut self, state: &AppState, id: TrackId) {
        if self.unsettled.contains(&id) {
            self.take_up_changed(state, id);
        } else {
            self.change_urgent.push_back(id);
        }
    }

    /// EE9: whether `result` would change a sounding track's analysis with
    /// a changed file's: one asked for because the file changed, or any
    /// other read applied to the model (a player's waveform, say) of a file
    /// whose stamp differs from the stored one. A read only for the peaks
    /// changes nothing, and *Re-analyse all* (the library pass) is the
    /// operator's explicit call.
    fn hold_while_sounding(&mut self, result: &AnalysisResult, state: &AppState) -> bool {
        let id = result.track;
        let asked = self.change_analysing.remove(&id);
        let Some(track) = state.library.get(id) else {
            return false;
        };
        if !track_sounding(state, id) || (self.peaks_only.contains(&id) && track.analyzed) {
            return false;
        }
        let read = result
            .stamp
            .and_then(|(size, modified)| FileStamp::from_metadata(size, modified));
        let other_file =
            matches!((track.file_stamp, read), (Some(stored), Some(read)) if stored != read);
        asked || (other_file && !self.library_pass.contains(&id))
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
            // Recorded only once the files are written: while a save fails
            // the state stays dirty and is retried, no more often than the
            // debounce interval.
            if self.next_retry.is_none_or(|at| now >= at) {
                let result = self.save_all(state, now);
                if self.note_save(state, result, now) {
                    self.saved_version = version;
                    self.dirty_since = None;
                }
                self.last_session_save = Some(now);
            }
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
            let result = self.save_session(state, now);
            self.note_save(state, result, now);
            self.last_session_save = Some(now);
        }
    }

    /// Keeps the failure state of the saves: the first failure of a run is
    /// logged, then silence until a save works again (one info line). A
    /// failure delays the next autosave attempt by the debounce interval.
    /// Returns whether the save worked.
    fn note_save(&mut self, state: &AppState, result: Result<(), String>, now: Instant) -> bool {
        match result {
            Ok(()) => {
                if self.save_failing {
                    tracing::info!("saving works again");
                }
                self.save_failing = false;
                self.next_retry = None;
                true
            }
            Err(e) => {
                if !self.save_failing {
                    tracing::error!(error = %e, "saving failed; retrying at the autosave pace");
                }
                self.save_failing = true;
                let debounce =
                    Duration::from_secs_f64(state.config.tuning.save_debounce_ms.max(0.0) / 1000.0);
                self.next_retry = Some(now + debounce);
                false
            }
        }
    }

    /// Saves the configuration and the session; the error of the first
    /// that failed. The caller logs it.
    fn save_all(&mut self, state: &AppState, now: Instant) -> Result<(), String> {
        let config = self
            .store
            .save_config(state)
            .map_err(|e| format!("the configuration: {e}"));
        let session = self.save_session(state, now);
        config.and(session)
    }

    /// Saves the playlists, library, carts, players and layout through the
    /// database: only the rows that changed are written. On failure the
    /// autosave keeps the state dirty and retries (see `note_save`).
    fn save_session(&mut self, state: &AppState, now: Instant) -> Result<(), String> {
        let telemetry = self.conductor.telemetry.load();
        let position_of = |player: PlayerId| {
            telemetry
                .players
                .iter()
                .find(|(id, _)| *id == player)
                .and_then(|(_, t)| t.position_secs)
                .unwrap_or(0.0)
        };
        self.store
            .save_state(state, position_of, now)
            .map_err(|e| format!("the state: {e}"))
    }
}

/// The running services thread.
pub struct ServicesHandle {
    requests: Sender<ServiceRequest>,
    stop: Arc<AtomicBool>,
    /// Behind a lock so a shared session can stop it (`stop`).
    thread: Mutex<Option<JoinHandle<()>>>,
}

impl ServicesHandle {
    /// Queues a request; never blocks.
    pub fn request(&self, request: ServiceRequest) -> bool {
        self.requests.try_send(request).is_ok()
    }

    /// Stops the thread after its final save.
    pub fn shutdown(self) {
        self.stop();
    }

    /// Stops the thread after its final save and waits for it; once
    /// stopped, returns at once. The lock is held through the join, so a
    /// second caller meanwhile waits for the save too.
    pub fn stop(&self) {
        self.stop.store(true, Ordering::Release);
        let mut thread = self.thread.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(thread) = thread.take()
            && thread.join().is_err()
        {
            tracing::error!("the services thread panicked");
        }
    }
}

impl Drop for ServicesHandle {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use std::io::{Error, ErrorKind};

    fn at(secs: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(secs)
    }

    /// M8: a track already queued is not queued again; once taken it
    /// can be queued anew.
    #[test]
    fn the_unique_queue_holds_each_track_once() {
        let mut q = UniqueQueue::default();
        q.push_back(TrackId(1));
        q.push_back(TrackId(2));
        q.push_back(TrackId(1));
        assert_eq!(q.len(), 2);
        assert_eq!(q.pop_front(), Some(TrackId(1)));
        q.push_back(TrackId(1));
        assert_eq!(q.pop_front(), Some(TrackId(2)));
        assert_eq!(q.pop_front(), Some(TrackId(1)));
        assert_eq!(q.pop_front(), None);
        q.push_back(TrackId(3));
        q.clear();
        q.push_back(TrackId(3));
        assert_eq!(q.len(), 1);
    }

    #[test]
    fn the_first_look_records_what_it_sees() {
        assert_eq!(
            look(Seen::New, Ok((10, at(5)))),
            Look::Record(Seen::At((10, at(5))))
        );
        assert_eq!(
            look(Seen::Unknown, Ok((10, at(5)))),
            Look::Record(Seen::At((10, at(5))))
        );
    }

    #[test]
    fn the_same_size_and_time_is_unchanged_and_a_new_one_is_a_change() {
        let seen = Seen::At((10, at(5)));
        assert_eq!(look(seen, Ok((10, at(5)))), Look::Keep);
        assert_eq!(look(seen, Ok((11, at(5)))), Look::Changed);
        assert_eq!(look(seen, Ok((10, at(6)))), Look::Changed);
    }

    #[test]
    fn a_file_that_is_gone_is_gone_whatever_was_seen() {
        for seen in [Seen::New, Seen::Unknown, Seen::At((1, at(1)))] {
            let gone = Err(Error::from(ErrorKind::NotFound));
            assert_eq!(look(seen, gone), Look::Gone);
        }
    }

    #[test]
    fn look_keeps_what_it_knows_when_the_stat_fails() {
        let denied = || Err(Error::from(ErrorKind::PermissionDenied));
        assert_eq!(look(Seen::New, denied()), Look::Record(Seen::Unknown));
        assert_eq!(look(Seen::Unknown, denied()), Look::Keep);
        assert_eq!(look(Seen::At((1, at(1))), denied()), Look::Keep);
    }
}
