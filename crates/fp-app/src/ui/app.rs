//! The main screen (spec §8.3). The UI reads model and telemetry snapshots
//! every frame, sends commands, and keeps only view state.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender};
use egui::{
    Align, Color32, Key, Layout, Rect, RichText, Sense, TextureHandle, Ui, UiBuilder, vec2,
};
use fp_backends::AudioBackend;
use fp_engine::bus::BusHealth;
use fp_engine::conductor::Telemetry;
use fp_model::{
    AppState, Command, EntryId, KeyChord, ModelError, PlayerId, PlaylistId, RestartReason,
    ShortcutAction, TrackId,
};

use super::about::{self, NoticeOpener};
use super::cartwall;
use super::controller::Controller;
use super::cue_window;
use super::exit_guard::{self, ExitIntent};
use super::files::{AUDIO_EXTENSIONS, audio_paths};
use super::notice;
use super::player;
use super::playlist_files::{self, FileOutcome};
use super::reset_played;
use super::settings::{self, SettingsDeps, SettingsState};
use super::tag_editor;
use super::theme;
use super::widgets::{self, TileStyle, font};
use crate::i18n::I18n;
use crate::services::{MediaCache, ServiceRequest};
use crate::tags::{TagJob, TagOutcome, TagWorker};

const MIN_COLUMN_WIDTH: f32 = 380.0;
const TOP_BAR_HEIGHT: f32 = 34.0;
const STATUS_BAR_HEIGHT: f32 = 24.0;
const NOTICE_SECS: f64 = 5.0;
/// Idle repaint period (the clock).
const IDLE_REPAINT: Duration = Duration::from_millis(100);

/// The payload of a column header being dragged (feedback 2 spec O24): its
/// position in the list of columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DragColumn {
    pub index: usize,
}

/// The payload of an entry being dragged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DragEntry {
    pub entry: EntryId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DropTarget {
    /// The player whose table shows the target: only that table draws the
    /// drop line (operator feedback 4, Q4.3).
    pub player: PlayerId,
    pub playlist: PlaylistId,
    pub index: usize,
}

/// Files chosen in a file dialog, to insert at `index` of `playlist`.
pub(crate) struct Picked {
    playlist: PlaylistId,
    index: usize,
    paths: Vec<PathBuf>,
}

/// A row a player's table scrolls to once its playlist is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FollowScroll {
    pub entry: EntryId,
    /// Where the row ends up: the top (following the current entry) or the
    /// middle (the next entry at start-up).
    pub align: Align,
    /// Whether the table glides to the row; start-up jumps.
    pub animated: bool,
}

/// The scroll of one player's playlist tabs.
#[derive(Default)]
pub(crate) struct TabScroll {
    pub offset: f32,
    /// The shown playlist, the tab count and the view width (as bits) the
    /// shown tab was last brought into view for; it is revealed again when
    /// this key changes.
    pub revealed: Option<(PlaylistId, usize, u32)>,
}

/// Everything the UI remembers between frames.
#[derive(Default)]
pub(crate) struct ViewState {
    pub selection: HashMap<PlayerId, EntryId>,
    pub active_player: Option<PlayerId>,
    pub drop: Option<DropTarget>,
    pub file_drop: Option<DropTarget>,
    /// The pointer is over a track table but not on a drop target (its
    /// header, a column edge, the scroll bar, or a window over it): files
    /// dropped there are not inserted anywhere (Q4.6).
    pub file_drop_refused: bool,
    /// The pixel widths of each player's table columns in the last frame,
    /// in the order of `ui.table_columns`.
    pub widths: HashMap<PlayerId, Vec<f32>>,
    /// The column edge being dragged (feedback 2 spec O16).
    pub(crate) live_resize: Option<super::table::LiveResize>,
    pub rows_built: usize,
    /// The row a double-click set as next in the current burst of clicks
    /// (see `table::set_next_on_click`).
    pub(crate) set_next_sent: Option<(PlayerId, EntryId)>,
    pub settings_open: bool,
    pub about_open: bool,
    /// The close guard is asking the operator (feedback 2 spec O6).
    pub exit_guard: Option<ExitIntent>,
    /// The operator confirmed the close: let the window go.
    pub close_confirmed: bool,
    /// Restart now was pressed (Settings footer or the top-bar pill).
    pub restart_requested: bool,
    /// The notice about tracks an earlier version analysed is open.
    pub outdated_open: bool,
    /// Where each waveform's menu was opened, in seconds.
    pub wave_menu: HashMap<super::wave_view::WaveKey, f64>,
    /// When the operator last used each player's table or tabs (scroll,
    /// entry drag, row menu, tab click), in `Scene::time`.
    pub table_touched: HashMap<PlayerId, f64>,
    /// The current entry each player's table last saw.
    pub followed: HashMap<PlayerId, Option<EntryId>>,
    /// A current entry the table will follow once the operator's grace has
    /// passed (feedback spec F18).
    pub follow_pending: HashMap<PlayerId, EntryId>,
    /// A row the table scrolls to once its playlist is shown.
    pub follow_scroll: HashMap<PlayerId, FollowScroll>,
    /// The players whose table already had its start-up scroll to the next
    /// entry (feedback 2 spec O7); it happens once, on their first frame.
    pub startup_scrolled: HashSet<PlayerId>,
    /// Zoomed waveforms; one without a zoom shows the whole track.
    pub wave_zoom: super::wave_view::WaveZooms,
    /// A marker being dragged on a waveform, and the track it belongs to.
    pub marker_drag: Option<(super::wave_view::WaveKey, fp_model::MarkerKind, TrackId)>,
    /// A cart to open in Settings → Cartwall (`Edit…` on a cart).
    pub edit_cart: Option<(fp_model::CartPageId, usize)>,
    /// A track whose tags the operator asked to edit (feedback 2 spec O23);
    /// the next frame opens the editor.
    pub edit_tags: Option<TrackId>,
    /// The tag editor, while it is open.
    pub(crate) tag_editor: Option<tag_editor::TagEditor>,
    /// The playlist whose Reset played waits for the operator's answer
    /// (feedback 2 spec O22).
    pub confirm_reset: Option<PlaylistId>,
    /// How far each player's playlist tabs are scrolled (feedback 2 spec
    /// O35).
    pub tab_scroll: HashMap<PlayerId, TabScroll>,
    notice: Option<(String, f64)>,
    /// Dropouts the engine counted, as the status bar alerts show them.
    dropouts: Dropouts,
}

/// Where a dropout was counted.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum DropoutSource {
    /// Source underruns on a player (decoding did not keep up).
    Player(PlayerId),
    /// Xruns reported by an output device.
    Device(fp_engine::bus::BusKey),
}

/// The count last seen for each place that can drop out, and until when its
/// alert shows: `NOTICE_SECS` after the last increase (audit A8).
#[derive(Default)]
struct Dropouts(HashMap<DropoutSource, (u64, f64)>);

impl Dropouts {
    /// Notes this frame's counts. Counters start at zero, so a place seen
    /// for the first time with a count above zero has dropped out since the
    /// last frame we saw (a bus opened mid-session): that raises an alert.
    /// A count that fell is a new source on the player, which starts again at zero.
    fn observe(&mut self, telemetry: &Telemetry, time: f64) {
        let counts = telemetry
            .players
            .iter()
            .map(|(id, t)| (DropoutSource::Player(*id), t.underruns))
            .chain(
                telemetry
                    .buses
                    .iter()
                    .map(|b| (DropoutSource::Device(b.key.clone()), b.counters.xruns)),
            );
        for (source, count) in counts {
            match self.0.entry(source) {
                std::collections::hash_map::Entry::Vacant(v) => {
                    let until = if count > 0 {
                        time + NOTICE_SECS
                    } else {
                        f64::NEG_INFINITY
                    };
                    v.insert((count, until));
                }
                std::collections::hash_map::Entry::Occupied(mut o) => {
                    let (seen, until) = o.get_mut();
                    if count > *seen || (count < *seen && count > 0) {
                        *until = time + NOTICE_SECS;
                    }
                    *seen = count;
                }
            }
        }
    }

    /// The alerts to show at `time`, players first, in a fixed order.
    fn alerts(&self, time: f64, players: &[PlayerId], t: &I18n) -> Vec<String> {
        let mut active: Vec<_> = self
            .0
            .iter()
            .filter(|(_, (_, until))| *until > time)
            .collect();
        active.sort_by(|a, b| a.0.cmp(b.0));
        active
            .into_iter()
            .filter_map(|(source, (count, _))| match source {
                DropoutSource::Player(id) => {
                    let n = players.iter().position(|p| p == id)? + 1;
                    let player = t.tr_args("player-label", &[("n", n.into())]);
                    Some(t.tr_args(
                        "alert-underruns",
                        &[("player", player.into()), ("count", (*count).into())],
                    ))
                }
                DropoutSource::Device(device) => Some(t.tr_args(
                    "alert-xruns",
                    &[
                        ("device", device.device.clone().into()),
                        ("count", (*count).into()),
                    ],
                )),
            })
            .collect()
    }
}

/// What a frame draws from, shared by the parts of the screen.
pub(crate) struct Scene<'a> {
    pub ctl: &'a dyn Controller,
    pub i18n: &'a I18n,
    pub media: &'a MediaCache,
    pub state: &'a AppState,
    pub telemetry: &'a Telemetry,
    /// Seconds since start, for blinking.
    pub time: f64,
    pub ctx: egui::Context,
    picks: &'a Sender<Picked>,
    pub files: &'a Sender<FileOutcome>,
    pub services: Option<&'a Sender<ServiceRequest>>,
}

impl Scene<'_> {
    /// Asks the services thread for something. Without services (tests) or
    /// with a full queue the request is dropped: it is only a convenience.
    pub fn request(&self, request: ServiceRequest) {
        if let Some(services) = self.services {
            let _ = services.try_send(request);
        }
    }

    /// Why `track` cannot be played, with its path, for the tooltip of its
    /// warning icon; `None` when it can.
    pub fn file_tip(&self, track: TrackId) -> Option<String> {
        let track = self.state.library.get(track)?;
        let key = super::view::file_problem(track)?;
        let path = track.path.display().to_string();
        Some(self.i18n.tr_args(key, &[("path", path.into())]))
    }

    /// Why `track` cannot be played, without its path, for the row popup
    /// (which has a Path row of its own); `None` when it can.
    pub fn file_reason(&self, track: TrackId) -> Option<String> {
        let track = self.state.library.get(track)?;
        let key = match track.file_state {
            fp_model::FileState::Ok => return None,
            fp_model::FileState::Missing => "file-missing-reason",
            fp_model::FileState::Unreadable => "file-unreadable-reason",
        };
        Some(self.i18n.tr(key))
    }

    /// Sends the new list of table columns (feedback 2 spec O24), unless it
    /// is the one in use. The model repairs it (`Config::validate`).
    pub fn set_table_columns(&self, columns: Vec<fp_model::TableColumn>) {
        let mut config = self.state.config.clone();
        config.ui.table_columns = columns;
        let _ = config.validate();
        if config.ui.table_columns != self.state.config.ui.table_columns {
            self.ctl.send(Command::UpdateConfig(Box::new(config)));
        }
    }

    /// Opens the native file dialog without blocking the interface.
    pub fn pick_files(&self, playlist: PlaylistId, index: usize) {
        let tx = self.picks.clone();
        let ctx = self.ctx.clone();
        let filter = self.i18n.tr("dialog-audio-files");
        let dir = self.state.config.ui.music_dir.clone();
        let spawned = std::thread::Builder::new()
            .name("fp-file-dialog".to_owned())
            .spawn(move || {
                let mut dialog = rfd::AsyncFileDialog::new().add_filter(filter, AUDIO_EXTENSIONS);
                if let Some(dir) = dir {
                    dialog = dialog.set_directory(dir);
                }
                let picked: Vec<PathBuf> = pollster::block_on(dialog.pick_files())
                    .map(|files| files.iter().map(|f| f.path().to_path_buf()).collect())
                    .unwrap_or_default();
                let paths = audio_paths(&picked);
                let _ = tx.send(Picked {
                    playlist,
                    index,
                    paths,
                });
                ctx.request_repaint();
            });
        if let Err(e) = spawned {
            tracing::error!(error = %e, "could not open the file dialog");
        }
    }
}

pub struct AppUi {
    ctl: Arc<dyn Controller>,
    i18n: I18n,
    media: MediaCache,
    services: Option<Sender<ServiceRequest>>,
    platform: String,
    pub(crate) view: ViewState,
    covers: HashMap<TrackId, TextureHandle>,
    covers_version: u64,
    picks_tx: Sender<Picked>,
    picks_rx: Receiver<Picked>,
    files_tx: Sender<FileOutcome>,
    files_rx: Receiver<FileOutcome>,
    themed: bool,
    #[cfg(feature = "test-hooks")]
    fail_next_frame: bool,
    settings: SettingsState,
    settings_shown: bool,
    /// Whether the start-up check for outdated analyses has run.
    outdated_checked: bool,
    /// How many tracks an earlier version analysed, per model snapshot.
    outdated: crate::services::OutdatedCount,
    /// The egui context, once the first frame has run.
    ctx: Option<egui::Context>,
    /// Imports asked for before the first frame.
    pending_imports: Vec<PathBuf>,
    /// Playlists handed over by a second start of the application.
    inbox: Option<Receiver<PathBuf>>,
    /// The `config.ui.language` the interface strings follow.
    language: Option<Option<String>>,
    backends: Vec<Arc<dyn AudioBackend>>,
    service_faults: Option<Arc<std::sync::atomic::AtomicU64>>,
    /// The installed third-party notices, found at start-up.
    notices: Option<PathBuf>,
    opener: NoticeOpener,
    /// The MIDI service's handle (Settings > MIDI), when it started.
    midi: Option<fp_control::service::MidiHandle>,
    /// The remote servers' state (Settings > Remote), when they started.
    remote_status: Option<Arc<arc_swap::ArcSwap<fp_remote::RemoteStatus>>>,
    /// The configuration the engine was built with (feedback 2 spec O4).
    started: fp_model::Config,
    /// Set once a restart is confirmed; `main` reads it after the window
    /// closes.
    restart: Arc<AtomicBool>,
    /// Reads and writes tags off the interface thread; started by the first
    /// use of the tag editor.
    tag_worker: Option<TagWorker>,
    /// The image the operator chose for a cover (or `None`: the dialog was
    /// closed), from the dialog's helper thread.
    cover_picks_tx: Sender<CoverPick>,
    cover_picks_rx: Receiver<CoverPick>,
    /// Counts the openings of the tag editor.
    tag_sessions: u64,
    /// Stands in for the native image dialog in tests.
    #[cfg(feature = "test-hooks")]
    cover_picker: Option<CoverPicker>,
}

/// The answer of the image dialog: the track, the editor session that opened
/// it, and the chosen file (`None` if the dialog was closed).
type CoverPick = (TrackId, u64, Option<PathBuf>);

/// Chooses an image file; runs on the dialog's helper thread.
type CoverPicker = Arc<dyn Fn() -> Option<PathBuf> + Send + Sync>;

impl AppUi {
    pub fn new(ctl: Arc<dyn Controller>, i18n: I18n, media: MediaCache) -> Self {
        let (picks_tx, picks_rx) = crossbeam_channel::unbounded();
        let (files_tx, files_rx) = crossbeam_channel::unbounded();
        let (cover_picks_tx, cover_picks_rx) = crossbeam_channel::unbounded();
        let started = ctl.model().config.clone();
        Self {
            ctl,
            i18n,
            media,
            services: None,
            platform: default_platform(),
            view: ViewState::default(),
            covers: HashMap::new(),
            covers_version: 0,
            picks_tx,
            picks_rx,
            files_tx,
            files_rx,
            themed: false,
            #[cfg(feature = "test-hooks")]
            fail_next_frame: false,
            settings: SettingsState::default(),
            settings_shown: false,
            outdated_checked: false,
            outdated: crate::services::OutdatedCount::default(),
            ctx: None,
            pending_imports: Vec::new(),
            inbox: None,
            language: None,
            backends: Vec::new(),
            service_faults: None,
            notices: None,
            opener: about::system_opener(),
            midi: None,
            remote_status: None,
            started,
            restart: Arc::new(AtomicBool::new(false)),
            tag_worker: None,
            cover_picks_tx,
            cover_picks_rx,
            tag_sessions: 0,
            #[cfg(feature = "test-hooks")]
            cover_picker: None,
        }
    }

    /// The configuration the audio engine was built with; a change to a
    /// start-up setting after it shows "Restart pending".
    pub fn with_started_config(mut self, config: fp_model::Config) -> Self {
        self.started = config;
        self
    }

    /// True once the operator confirmed a restart: `main` starts the
    /// application again after its normal shutdown.
    pub fn restart_flag(&self) -> Arc<AtomicBool> {
        self.restart.clone()
    }

    /// The services thread's fault counter, shown as a status-bar alert.
    pub fn with_service_faults(mut self, faults: Arc<std::sync::atomic::AtomicU64>) -> Self {
        self.service_faults = Some(faults);
        self
    }

    /// The audio systems listed in Settings.
    pub fn with_backends(mut self, backends: Vec<Arc<dyn AudioBackend>>) -> Self {
        self.backends = backends;
        self
    }

    /// Where "Re-analyse all" goes.
    pub fn with_services(mut self, services: Sender<ServiceRequest>) -> Self {
        self.services = Some(services);
        self
    }

    /// The backend/OS label in the status bar.
    pub fn with_platform(mut self, platform: String) -> Self {
        self.platform = platform;
        self
    }

    /// The installed third-party notices file (see `about::find_notices`).
    pub fn with_notices(mut self, notices: Option<PathBuf>) -> Self {
        self.notices = notices;
        self
    }

    /// The MIDI service, for Settings > MIDI.
    pub fn with_midi(mut self, midi: fp_control::service::MidiHandle) -> Self {
        self.midi = Some(midi);
        self
    }

    /// The remote servers' state, for Settings > Remote.
    pub fn with_remote_status(
        mut self,
        status: Arc<arc_swap::ArcSwap<fp_remote::RemoteStatus>>,
    ) -> Self {
        self.remote_status = Some(status);
        self
    }

    /// How the About window opens the notices file.
    pub fn with_notice_opener(mut self, opener: NoticeOpener) -> Self {
        self.opener = opener;
        self
    }

    pub fn i18n(&self) -> &I18n {
        &self.i18n
    }

    /// Forgets all view state (selection, drags, open dialogs); the next
    /// frame is rebuilt from the current snapshot.
    pub fn reset_view(&mut self) {
        self.view = ViewState::default();
        self.settings = SettingsState::default();
        self.settings_shown = false;
        self.covers.clear();
    }

    /// Makes the **Change…** button of the tag editor take its image from
    /// `picker` instead of opening the native dialog. Used by tests.
    #[cfg(feature = "test-hooks")]
    pub fn set_cover_picker(
        &mut self,
        picker: impl Fn() -> Option<PathBuf> + Send + Sync + 'static,
    ) {
        self.cover_picker = Some(Arc::new(picker));
    }

    /// Makes the next frame panic. Used to test panic isolation.
    #[cfg(feature = "test-hooks")]
    pub fn fail_next_frame(&mut self) {
        self.fail_next_frame = true;
    }

    /// Playlists another start of the application hands over (see
    /// `instance::watch`), imported as they arrive.
    pub fn with_inbox(mut self, inbox: Receiver<PathBuf>) -> Self {
        self.inbox = Some(inbox);
        self
    }

    /// Imports a playlist file (M3U, M3U8, PLS) as a new playlist; the file
    /// is read on a helper thread.
    pub fn import_playlist(&mut self, path: PathBuf) {
        let limits = self.ctl.model().config.limits.clone();
        if let Some(ctx) = self.ctx.clone() {
            playlist_files::import(&ctx, path, limits, self.files_tx.clone());
        } else {
            self.pending_imports.push(path);
        }
    }

    /// Table rows built during the last frame (virtualisation check).
    pub fn rows_built(&self) -> usize {
        self.view.rows_built
    }

    /// The pixel widths of a player's table columns in the last frame, in
    /// the order of `ui.table_columns`.
    pub fn column_widths(&self, player: PlayerId) -> Option<Vec<f32>> {
        self.view.widths.get(&player).cloned()
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        #[cfg(feature = "test-hooks")]
        if std::mem::take(&mut self.fail_next_frame) {
            #[allow(clippy::panic)]
            {
                panic!("injected interface failure");
            }
        }
        let ctx = ui.ctx().clone();
        if !self.themed {
            // Fonts are bound from the next frame on; draw nothing until then.
            theme::apply(&ctx);
            self.themed = true;
            ctx.request_repaint();
            return;
        }
        let state = self.ctl.model();
        // Follow a language change made in Settings (the first frame only
        // records what the interface was built with).
        let wanted = state.config.ui.language.clone();
        match &self.language {
            None => self.language = Some(wanted),
            Some(applied) if *applied != wanted => {
                self.i18n = I18n::new(wanted.as_deref());
                self.language = Some(wanted);
            }
            Some(_) => {}
        }
        let telemetry = self.ctl.telemetry();
        let time = ctx.input(|i| i.time);
        self.view.rows_built = 0;
        self.view.dropouts.observe(&telemetry, time);
        self.view.file_drop = None;
        self.view.file_drop_refused = false;
        if !egui::DragAndDrop::has_any_payload(&ctx) {
            self.view.drop = None;
        }
        if self.media.version() != self.covers_version {
            self.covers.clear();
            self.covers_version = self.media.version();
        }
        while let Some(error) = self.ctl.take_rejection() {
            self.view.notice = Some((error_text(&self.i18n, &error), time + NOTICE_SECS));
        }
        // Paths arrive already filtered (and folders expanded) off this thread.
        if self.ctx.is_none() {
            self.ctx = Some(ctx.clone());
            for path in std::mem::take(&mut self.pending_imports) {
                self.import_playlist(path);
            }
        }
        let handed: Vec<PathBuf> = self
            .inbox
            .as_ref()
            .map(|rx| rx.try_iter().collect())
            .unwrap_or_default();
        for path in handed {
            tracing::info!(path = %path.display(), "importing a playlist handed over by another start");
            self.import_playlist(path);
        }
        while let Ok(outcome) = self.files_rx.try_recv() {
            let text = self.file_outcome(outcome);
            self.view.notice = Some((text, time + NOTICE_SECS));
        }
        while let Ok(picked) = self.picks_rx.try_recv() {
            if !picked.paths.is_empty() {
                self.ctl.send(Command::InsertPaths {
                    playlist: picked.playlist,
                    index: picked.index,
                    paths: picked.paths,
                });
            }
        }
        self.tag_outcomes(&state, time);
        self.cover_picks(&ctx, &state);
        if let Some(track) = self.view.edit_tags.take() {
            self.open_tag_editor(&ctx, &state, track);
        }
        self.keyboard(&ctx, &state);
        let pending = fp_model::restart_pending(&self.started, &state.config);
        let scene = Scene {
            ctl: self.ctl.as_ref(),
            i18n: &self.i18n,
            media: &self.media,
            state: &state,
            telemetry: &telemetry,
            time,
            ctx: ctx.clone(),
            picks: &self.picks_tx,
            files: &self.files_tx,
            services: self.services.as_ref(),
        };
        let full = ui.available_rect_before_wrap();
        ui.painter().rect_filled(full, 0.0, theme::BG);
        let top = Rect::from_min_size(full.min, vec2(full.width(), TOP_BAR_HEIGHT));
        let status = Rect::from_min_max(
            egui::pos2(full.left(), full.bottom() - STATUS_BAR_HEIGHT),
            full.max,
        );
        let middle = Rect::from_min_max(
            egui::pos2(full.left(), top.bottom()),
            egui::pos2(full.right(), status.top()),
        );
        let mut top_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(top)
                .layout(Layout::left_to_right(Align::Center)),
        );
        top_bar(&mut top_ui, &scene, &mut self.view, &pending);
        // The cartwall strip takes the bottom of the middle area.
        let inner = middle.shrink(8.0);
        let cart_height = cartwall::height(&scene).min(inner.height() * 0.6);
        let players_rect = Rect::from_min_max(
            inner.min,
            egui::pos2(inner.right(), inner.bottom() - cart_height - 8.0),
        );
        let cart_rect = Rect::from_min_max(
            egui::pos2(inner.left(), inner.bottom() - cart_height),
            inner.max,
        );
        let mut players_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(players_rect)
                .layout(Layout::left_to_right(Align::Min)),
        );
        players_row(&mut players_ui, &scene, &mut self.view, &mut self.covers);
        let mut cart_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(cart_rect)
                .id_salt("cartwall")
                .layout(Layout::top_down(Align::Min)),
        );
        cart_ui.set_clip_rect(cart_rect);
        cartwall::strip(&mut cart_ui, &scene, &mut self.view);
        let mut status_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(status)
                .layout(Layout::left_to_right(Align::Center)),
        );
        let faults = self
            .service_faults
            .as_ref()
            .map_or(0, |f| f.load(std::sync::atomic::Ordering::Acquire));
        status_bar(&mut status_ui, &scene, &self.view, &self.platform, faults);
        ui.allocate_rect(full, Sense::hover());
        // The CUE windows float over the screen; the dialogs below stay on top.
        cue_window::show_all(&ctx, &scene, &mut self.view);
        // Once, at start-up: tracks an earlier version analysed wait for
        // the operator (they cost the processor for a while to redo).
        if !self.outdated_checked {
            self.outdated_checked = true;
            self.view.outdated_open = self.services.is_some() && self.outdated.get(&state) > 0;
        }
        // Files dropped on the window, when no dialog is up (handled once
        // the dialogs are drawn).
        let mut take_drops = false;
        if self.view.settings_open {
            if !self.settings_shown {
                self.settings.reset();
                self.settings_shown = true;
            }
            if let Some((page, index)) = self.view.edit_cart.take() {
                self.settings.edit_cart(page, index);
            }
            let deps = SettingsDeps {
                backends: &self.backends,
                services: self.services.as_ref(),
                notice: self
                    .view
                    .notice
                    .as_ref()
                    .filter(|(_, until)| *until > time)
                    .map(|(text, _)| text.clone()),
                midi: self.midi.as_ref(),
                remote: self.remote_status.as_ref().map(|s| (**s.load()).clone()),
                restart_pending: !pending.is_empty(),
                outdated: self.outdated.get(&state),
            };
            let outcome = settings::show(&ctx, &scene, &mut self.settings, &deps);
            self.view.settings_open = outcome.open;
            self.view.restart_requested |= outcome.restart;
        } else {
            self.settings_shown = false;
            // Closing Settings ends MIDI learn: the next press on a surface
            // must act, not bind.
            if self.settings.midi_learning.take().is_some()
                && let Some(midi) = &self.midi
            {
                let _ = midi
                    .requests
                    .send(fp_control::service::MidiRequest::CancelLearn);
            }
            if self.view.about_open {
                self.view.about_open =
                    about::show(&ctx, &scene, self.notices.as_deref(), &self.opener);
            } else if self.view.outdated_open {
                let count = self.outdated.get(&state);
                match notice::show(&ctx, &scene, count) {
                    Some(notice::Answer::AnalyseNow) => {
                        if let Some(services) = &self.services {
                            let _ = services.try_send(ServiceRequest::AnalyseOutdated);
                        }
                        self.view.outdated_open = false;
                    }
                    Some(notice::Answer::Later) => self.view.outdated_open = false,
                    // Tracks on screen are analysed anyway: nothing left.
                    None if count == 0 => self.view.outdated_open = false,
                    None => {}
                }
            } else {
                take_drops = true;
            }
        }
        // O23: the tag editor, under the dialogs that must stay above it.
        if !self.view.settings_open
            && let Some(editor) = &mut self.view.tag_editor
        {
            if state.library.get(editor.track).is_none() {
                self.view.tag_editor = None;
            } else {
                let block = super::view::tag_edit_availability(&state, editor.track);
                match tag_editor::show(&ctx, &scene, editor, block) {
                    tag_editor::EditorAnswer::Cancel => self.view.tag_editor = None,
                    tag_editor::EditorAnswer::Save => {
                        let job = state.library.get(editor.track).and_then(|track| {
                            editor.save_job(
                                &track.path,
                                &state.config.limits,
                                state.config.analysis.cover_thumb_px,
                            )
                        });
                        // The rule is judged again now: the track may have
                        // gone on air since the modal last drew.
                        let started = block.is_none()
                            && job
                                .is_some_and(|job| start_tag_job(&mut self.tag_worker, &ctx, job));
                        if started {
                            editor.saving = true;
                            editor.error = None;
                        } else if block.is_none() {
                            tracing::error!("the tag worker is not running");
                            editor.error = Some(scene.i18n.tr("tags-error-worker"));
                        }
                    }
                    tag_editor::EditorAnswer::ChangeCover => {
                        if !editor.cover_busy {
                            #[cfg(feature = "test-hooks")]
                            let hook = self.cover_picker.clone();
                            #[cfg(not(feature = "test-hooks"))]
                            let hook: Option<CoverPicker> = None;
                            let filter = scene.i18n.tr("dialog-image-files");
                            let folder = state
                                .library
                                .get(editor.track)
                                .and_then(|t| t.path.parent().map(Path::to_path_buf));
                            let pick = move || match hook {
                                Some(picker) => picker(),
                                None => pick_cover_image(filter, folder),
                            };
                            if start_cover_dialog(
                                &self.cover_picks_tx,
                                &ctx,
                                (editor.track, editor.session),
                                pick,
                            ) {
                                editor.picking_cover();
                            }
                        }
                    }
                    tag_editor::EditorAnswer::Open => {}
                }
            }
        }
        // O22: Reset played asks before it clears the marks of a playlist.
        if let Some(playlist) = self.view.confirm_reset {
            if state.playlists.get(playlist).is_none() {
                // The playlist went away meanwhile: nothing left to confirm.
                self.view.confirm_reset = None;
            } else {
                match reset_played::show(&ctx, &scene) {
                    Some(true) => {
                        scene.ctl.send(Command::ResetPlayed(playlist));
                        self.view.confirm_reset = None;
                    }
                    Some(false) => self.view.confirm_reset = None,
                    None => {}
                }
            }
        }
        // O4: Restart now asks the close guard first when audio is on air.
        if std::mem::take(&mut self.view.restart_requested) {
            if fp_model::on_air(&state).is_empty() {
                begin_restart(&self.restart, &mut self.view, &ctx);
            } else {
                self.view.exit_guard = Some(ExitIntent::Restart);
            }
        }
        // The guard takes precedence over Settings and About: drawn last, it
        // is the top modal.
        if let Some(intent) = self.view.exit_guard {
            let items = fp_model::on_air(&state);
            if items.is_empty() {
                // Everything stopped meanwhile: nothing left to confirm.
                self.view.exit_guard = None;
            } else {
                match exit_guard::show(&ctx, &scene, intent, &items) {
                    Some(true) => {
                        for command in exit_guard::stop_commands(&items) {
                            scene.ctl.send(command);
                        }
                        self.view.exit_guard = None;
                        match intent {
                            ExitIntent::Close => {
                                self.view.close_confirmed = true;
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                            ExitIntent::Restart => {
                                begin_restart(&self.restart, &mut self.view, &ctx);
                            }
                        }
                    }
                    Some(false) => self.view.exit_guard = None,
                    None => {}
                }
            }
        }
        // Files dropped under the tag editor are discarded, like shortcuts.
        if take_drops && self.view.tag_editor.is_none() && self.view.confirm_reset.is_none() {
            self.file_drops(&ctx, &state);
        }
        let busy = super::view::animating(&state);
        if busy {
            ctx.request_repaint();
        } else {
            ctx.request_repaint_after(IDLE_REPAINT);
        }
    }

    /// O23: opens the tag editor on `track` and asks the tag worker for its
    /// sheet.
    fn open_tag_editor(&mut self, ctx: &egui::Context, state: &AppState, track: TrackId) {
        let Some(t) = state.library.get(track) else {
            return;
        };
        let job = TagJob::ReadSheet {
            track,
            path: t.path.clone(),
            limits: state.config.limits.clone(),
            thumb_px: state.config.analysis.cover_thumb_px,
        };
        self.tag_sessions += 1;
        if start_tag_job(&mut self.tag_worker, ctx, job) {
            self.view.tag_editor = Some(tag_editor::TagEditor::reading(track, self.tag_sessions));
        } else {
            tracing::error!("the tag worker is not running");
            self.view.notice = Some((
                self.i18n.tr("tags-error-worker"),
                ctx.input(|i| i.time) + NOTICE_SECS,
            ));
        }
    }

    /// O23: takes what the tag worker finished: a sheet for the editor, or
    /// the result of a save.
    fn tag_outcomes(&mut self, state: &AppState, time: f64) {
        let outcomes: Vec<TagOutcome> = self
            .tag_worker
            .as_ref()
            .map(|w| w.results().try_iter().collect())
            .unwrap_or_default();
        for outcome in outcomes {
            match outcome {
                TagOutcome::SheetRead { track, sheet } => {
                    if let Some(editor) = &mut self.view.tag_editor
                        && editor.track == track
                    {
                        editor.arrived(sheet.map(|s| *s));
                    }
                }
                TagOutcome::SheetWritten { track, result } => {
                    self.tag_saved(state, time, track, result);
                }
                TagOutcome::CoverLoaded {
                    track,
                    session,
                    result,
                } => {
                    if let Some(editor) = &mut self.view.tag_editor
                        && (editor.track, editor.session) == (track, session)
                    {
                        editor.cover_loaded(result);
                    }
                }
                // The services thread's reads and the summary writes have
                // their own consumers.
                TagOutcome::Read { .. } | TagOutcome::Written { .. } => {}
            }
        }
    }

    /// O23: hands the image the operator chose to the tag worker, which
    /// checks and decodes it; a closed dialog frees the Change button.
    fn cover_picks(&mut self, ctx: &egui::Context, state: &AppState) {
        let picks: Vec<CoverPick> = self.cover_picks_rx.try_iter().collect();
        for (track, session, path) in picks {
            let Some(editor) = self
                .view
                .tag_editor
                .as_mut()
                .filter(|e| (e.track, e.session) == (track, session))
            else {
                continue;
            };
            let Some(path) = path else {
                editor.cover_not_picked();
                continue;
            };
            let job = TagJob::LoadCover {
                track,
                session,
                path,
                limits: state.config.limits.clone(),
                thumb_px: state.config.analysis.cover_thumb_px,
            };
            if !start_tag_job(&mut self.tag_worker, ctx, job) {
                tracing::error!("the tag worker is not running");
                editor.cover_not_picked();
            }
        }
    }

    fn tag_saved(
        &mut self,
        state: &AppState,
        time: f64,
        track: TrackId,
        result: Result<Box<crate::tags::SheetSaved>, fp_analysis::tags::TagWriteError>,
    ) {
        let title = state
            .library
            .get(track)
            .map(|t| t.title.clone())
            .unwrap_or_default();
        let editor = self.view.tag_editor.as_mut().filter(|e| e.track == track);
        match result {
            Ok(saved) => {
                let unstored = editor
                    .as_ref()
                    .map(|e| e.unstored(saved.sheet.as_ref(), &state.config.limits))
                    .unwrap_or_default();
                let cover_unstored = editor
                    .as_ref()
                    .is_some_and(|e| e.cover_unstored(saved.sheet.as_ref()));
                // The cover shown elsewhere (the player, the remote API)
                // follows the file.
                if let Some(sheet) = &saved.sheet
                    && editor.as_ref().is_some_and(|e| e.cover_changed())
                {
                    self.media
                        .set_cover(track, sheet.cover().and_then(|c| c.thumb_shared()));
                }
                let text = if unstored.is_empty() && !cover_unstored {
                    self.i18n.tr_args("tags-saved", &[("title", title.into())])
                } else {
                    let mut names: Vec<String> = unstored
                        .iter()
                        .map(|f| self.i18n.tr(&tag_editor::field_key(*f)))
                        .collect();
                    if cover_unstored {
                        names.push(self.i18n.tr("tags-cover"));
                    }
                    self.i18n.tr_args(
                        "tags-saved-partly",
                        &[("title", title.into()), ("fields", names.join(", ").into())],
                    )
                };
                self.ctl.send(Command::ApplyTags {
                    track,
                    tags: Box::new(saved.tags),
                });
                self.view.notice = Some((text, time + NOTICE_SECS));
                if editor.is_some() {
                    self.view.tag_editor = None;
                }
            }
            Err(e) => {
                let text = self.i18n.tr_args(
                    "tags-save-failed",
                    &[
                        ("title", title.into()),
                        ("error", tag_error_text(&self.i18n, &e).into()),
                    ],
                );
                if let Some(editor) = editor {
                    editor.saving = false;
                    editor.error = Some(text.clone());
                }
                self.view.notice = Some((text, time + NOTICE_SECS));
            }
        }
    }

    /// O6: a close request while something is on air waits for the
    /// operator. Runs once per frame from `eframe::App::logic`, which
    /// eframe also calls while the window is minimized or hidden (when
    /// `ui` does not run), so a close can never bypass it.
    pub fn guard_close(&mut self, ctx: &egui::Context) {
        let close_requested = ctx.input(|i| i.viewport().close_requested());
        if !close_requested || self.view.close_confirmed {
            return;
        }
        if fp_model::on_air(&self.ctl.model()).is_empty() {
            // Nothing to cut: let the window close.
            return;
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        // The dialog must be visible even if the window was minimized.
        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        self.view.exit_guard = Some(ExitIntent::Close);
    }

    fn keyboard(&mut self, ctx: &egui::Context, state: &AppState) {
        // The close guard owns the keyboard while it is open, even over a
        // focused text field: Esc cancels it and no shortcut acts. (Esc is
        // answered here, before the modal is drawn, so the modal never
        // sees it; its own `should_close` covers the backdrop click.)
        if self.view.exit_guard.is_some() {
            let escape = ctx.input(|i| {
                i.events.iter().any(|e| {
                    matches!(e, egui::Event::Key {
                        key: Key::Escape,
                        pressed: true,
                        repeat: false,
                        modifiers,
                        ..
                    } if modifiers.is_none())
                })
            });
            if escape {
                self.view.exit_guard = None;
            }
            return;
        }
        if ctx.text_edit_focused() {
            return;
        }
        // The tag editor is modal: no shortcut, not even Delete, acts under it.
        if self.view.tag_editor.is_some() {
            return;
        }
        // So is the Reset played question: Esc cancels it, nothing else acts.
        if self.view.confirm_reset.is_some() {
            let escape = ctx.input(|i| {
                i.events.iter().any(|e| {
                    matches!(e, egui::Event::Key {
                        key: Key::Escape,
                        pressed: true,
                        repeat: false,
                        modifiers,
                        ..
                    } if modifiers.is_none())
                })
            });
            if escape {
                self.view.confirm_reset = None;
            }
            return;
        }
        // Configured shortcuts whose key the toolkit knows.
        let bindings: Vec<(Key, &KeyChord, ShortcutAction)> = state
            .config
            .shortcuts
            .iter()
            .filter_map(|s| Key::from_name(&s.chord.key).map(|k| (k, &s.chord, s.action)))
            // Delete, Backspace and Esc keep their fixed meaning.
            .filter(|(k, _, _)| !super::settings::RESERVED_KEYS.contains(k))
            .collect();
        let (fired, held, delete, escape) = ctx.input(|i| {
            // Only the first press counts: holding a key must not repeat it
            // (a repeated Play would skip tracks on air).
            let pressed = |wanted: Key, chord: Option<&KeyChord>, repeats: bool| {
                i.events.iter().any(|e| match e {
                    egui::Event::Key {
                        key,
                        pressed: true,
                        repeat,
                        modifiers,
                        ..
                    } if *key == wanted && (repeats || !*repeat) => match chord {
                        Some(c) => chord_matches(modifiers, c),
                        None => modifiers.is_none(),
                    },
                    _ => false,
                })
            };
            let first_press = |wanted: Key, chord: Option<&KeyChord>| pressed(wanted, chord, false);
            let fired: Vec<ShortcutAction> = bindings
                .iter()
                .filter(|(key, chord, _)| first_press(*key, Some(chord)))
                .map(|(_, _, action)| *action)
                .collect();
            // Keys held on a shortcut, first press or repeat.
            let held: Vec<(Key, KeyChord)> = bindings
                .iter()
                .filter(|(key, chord, _)| pressed(*key, Some(chord), true))
                .map(|(key, chord, _)| (*key, (*chord).clone()))
                .collect();
            (
                fired,
                held,
                first_press(Key::Delete, None) || first_press(Key::Backspace, None),
                first_press(Key::Escape, None),
            )
        });
        if self.view.settings_open {
            if escape && !self.settings.capturing() {
                self.view.settings_open = false;
            }
            return;
        }
        if self.view.about_open {
            if escape {
                self.view.about_open = false;
            }
            return;
        }
        if self.view.outdated_open {
            // Esc is "Later".
            if escape {
                self.view.outdated_open = false;
            }
            return;
        }
        // A key held on a shortcut is taken out of the frame's input, so a
        // focused button or list does not act on it too (Space, Enter), nor
        // on its repeats.
        // Only presses of the shortcut's own chord (Shift+Space still reaches
        // the focused widget when Space is the shortcut), and the releases.
        ctx.input_mut(|i| {
            i.events.retain(|e| match e {
                egui::Event::Key {
                    key,
                    pressed,
                    modifiers,
                    ..
                } => !held
                    .iter()
                    .any(|(k, c)| k == key && (!*pressed || chord_matches(modifiers, c))),
                _ => true,
            });
        });
        // Tab and the arrows move the focus before this runs; a shortcut on
        // them keeps it where it was.
        let moves_focus = [
            Key::Tab,
            Key::ArrowUp,
            Key::ArrowDown,
            Key::ArrowLeft,
            Key::ArrowRight,
        ];
        if held.iter().any(|(k, _)| moves_focus.contains(k)) {
            ctx.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
        }
        for action in fired {
            // An action that makes no sense now (R28) is ignored.
            if let Some(command) = shortcut_command(state, action)
                && fp_model::command_available(state, &command)
            {
                self.ctl.send(command);
            }
        }
        if delete
            && let Some(player) = self.view.active_player
            && let Some(entry) = self.view.selection.remove(&player)
        {
            // Only an entry of the playlist the player shows can be removed.
            let shown = state
                .player(player)
                .ok()
                .and_then(|p| state.playlists.get(p.playlist))
                .is_some_and(|l| l.position(entry).is_some());
            if shown {
                self.ctl.send(Command::RemoveEntry(entry));
            }
        }
        if escape {
            self.view.selection.clear();
        }
    }

    /// Applies a finished import or export and says what happened.
    fn file_outcome(&mut self, outcome: FileOutcome) -> String {
        let t = &self.i18n;
        match outcome {
            FileOutcome::Imported {
                name,
                result: Ok(list),
            } => {
                let count = list.entries.len();
                let mut text = if count == 0 {
                    t.tr_args("playlist-import-empty", &[("name", name.clone().into())])
                } else {
                    t.tr_args(
                        "playlist-imported",
                        &[("name", name.clone().into()), ("count", count.into())],
                    )
                };
                if list.skipped_streams > 0 {
                    text.push(' ');
                    text.push_str(&t.tr_args(
                        "playlist-streams-skipped",
                        &[("streams", list.skipped_streams.into())],
                    ));
                }
                if count > 0 {
                    let paths = list.entries.into_iter().map(|e| e.path).collect();
                    self.ctl
                        .send(Command::CreatePlaylistFromPaths { name, paths });
                }
                text
            }
            FileOutcome::Imported {
                name,
                result: Err(e),
            } => t.tr_args(
                "playlist-import-failed",
                &[("name", name.into()), ("error", e.text(t).into())],
            ),
            FileOutcome::Exported(Ok(path)) => t.tr_args(
                "playlist-exported",
                &[("path", path.display().to_string().into())],
            ),
            FileOutcome::Exported(Err(e)) => {
                t.tr_args("playlist-export-failed", &[("error", e.text(t).into())])
            }
        }
    }

    fn file_drops(&mut self, ctx: &egui::Context, state: &AppState) {
        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .map(|f| f.path().to_path_buf())
                .filter(|p| !p.as_os_str().is_empty())
                .collect()
        });
        // Playlist files dropped on the window become new playlists.
        let (lists, dropped): (Vec<PathBuf>, Vec<PathBuf>) = dropped
            .into_iter()
            .partition(|p| playlist_files::is_playlist_file(p));
        for list in lists {
            self.import_playlist(list);
        }
        if dropped.is_empty() {
            return;
        }
        // Over a table but not on a target: nothing is inserted (Q4.6).
        if self.view.file_drop.is_none() && self.view.file_drop_refused {
            return;
        }
        // Pointer positions are not reported during an OS drag on every
        // platform: without one, the files go to the end of the list shown
        // by the last player used (or the first player).
        let fallback = || {
            let player = self
                .view
                .active_player
                .and_then(|id| state.player(id).ok())
                .or_else(|| state.players.first())?;
            let list = state.playlists.get(player.playlist)?;
            Some(DropTarget {
                player: player.id,
                playlist: list.id,
                index: list.entries.len(),
            })
        };
        let Some(target) = self.view.file_drop.or_else(fallback) else {
            return;
        };
        // Reading folders can be slow (network shares, sleeping disks): it
        // happens on a helper thread and the result comes back as a pick.
        let tx = self.picks_tx.clone();
        let ctx = ctx.clone();
        let spawned = std::thread::Builder::new()
            .name("fp-drop-scan".to_owned())
            .spawn(move || {
                let _ = tx.send(Picked {
                    playlist: target.playlist,
                    index: target.index,
                    paths: audio_paths(&dropped),
                });
                ctx.request_repaint();
            });
        if let Err(e) = spawned {
            tracing::error!(error = %e, "could not read the dropped files");
        }
    }
}

/// Whether `modifiers` are exactly those of `chord`.
fn chord_matches(modifiers: &egui::Modifiers, chord: &KeyChord) -> bool {
    modifiers.ctrl == chord.ctrl
        && modifiers.alt == chord.alt
        && modifiers.shift == chord.shift
        && modifiers.mac_cmd == chord.command
}

/// The command a shortcut stands for, resolving 1-based positions against
/// the players and the cart page shown.
fn shortcut_command(state: &AppState, action: ShortcutAction) -> Option<Command> {
    let wall = &state.cartwall;
    let page_step = |step: isize| {
        let shown = wall.shown_page().map(|p| p.id);
        let index = wall.pages.iter().position(|p| Some(p.id) == shown)?;
        let count = wall.pages.len() as isize;
        let next = (index as isize + step).rem_euclid(count.max(1));
        wall.pages.get(usize::try_from(next).ok()?).map(|p| p.id)
    };
    if let Some(command) = fp_model::player_command(state, action) {
        return Some(command);
    }
    Some(match action {
        ShortcutAction::PlayPlayer(_)
        | ShortcutAction::PausePlayer(_)
        | ShortcutAction::StopPlayer(_)
        | ShortcutAction::FadeStopPlayer(_)
        | ShortcutAction::CuePlayer(_)
        | ShortcutAction::RestartPlayer(_)
        | ShortcutAction::PreviousPlayer(_) => return None,
        ShortcutAction::FireCart(n) => {
            let cart = wall
                .shown_page()?
                .carts
                .get(usize::from(n).checked_sub(1)?)?;
            Command::FireCart(cart.id)
        }
        ShortcutAction::StopAllCarts => Command::StopAllCarts,
        ShortcutAction::ToggleCartwall => Command::SetCartwallOpen(!wall.open),
        ShortcutAction::NextCartPage => Command::ShowCartPage(page_step(1)?),
        ShortcutAction::PreviousCartPage => Command::ShowCartPage(page_step(-1)?),
    })
}

fn default_platform() -> String {
    match std::env::consts::OS {
        "linux" => "Linux".to_owned(),
        "windows" => "Windows".to_owned(),
        "macos" => "macOS".to_owned(),
        other => other.to_owned(),
    }
}

/// Queues `job` on the tag worker, starting the worker first if it is not
/// running yet. `false` if there is no worker to give it to.
fn start_tag_job(worker: &mut Option<TagWorker>, ctx: &egui::Context, job: TagJob) -> bool {
    if worker.is_none() {
        let repaint = ctx.clone();
        *worker = TagWorker::spawn(Box::new(move || repaint.request_repaint())).ok();
    }
    worker.as_ref().is_some_and(|w| w.submit(job))
}

/// Opens the image dialog on a helper thread and sends the chosen path (or
/// `None`) back. `false` if the thread could not start.
fn start_cover_dialog(
    tx: &Sender<CoverPick>,
    ctx: &egui::Context,
    (track, session): (TrackId, u64),
    pick: impl FnOnce() -> Option<PathBuf> + Send + 'static,
) -> bool {
    let (tx, ctx) = (tx.clone(), ctx.clone());
    let spawned = std::thread::Builder::new()
        .name("fp-cover-dialog".to_owned())
        .spawn(move || {
            let _ = tx.send((track, session, pick()));
            ctx.request_repaint();
        });
    if let Err(e) = &spawned {
        tracing::error!(error = %e, "could not open the image dialog");
    }
    spawned.is_ok()
}

/// The native dialog for a cover image (JPEG or PNG), starting in `folder`.
fn pick_cover_image(filter: String, folder: Option<PathBuf>) -> Option<PathBuf> {
    let mut dialog = rfd::AsyncFileDialog::new().add_filter(filter, &["jpg", "jpeg", "png"]);
    if let Some(folder) = folder {
        dialog = dialog.set_directory(folder);
    }
    pollster::block_on(dialog.pick_file()).map(|file| file.path().to_path_buf())
}

/// Why a save failed, in the interface language.
fn tag_error_text(i18n: &I18n, error: &fp_analysis::tags::TagWriteError) -> String {
    use fp_analysis::tags::TagWriteError as E;
    match error {
        E::Unsupported => i18n.tr("tags-error-unsupported"),
        E::NotFound => i18n.tr("tags-error-not-found"),
        E::Denied => i18n.tr("tags-error-denied"),
        E::InvalidDate => i18n.tr("tags-error-invalid-date"),
        E::InvalidField(field) => i18n.tr_args(
            "tags-error-invalid-field",
            &[("field", i18n.tr(&tag_editor::field_key(*field)).into())],
        ),
        E::CoverNotStorable => i18n.tr("tags-error-cover-not-stored"),
        E::InvalidCover(_) => i18n.tr("tags-error-cover"),
        E::Other(detail) => i18n.tr_args("tags-error-other", &[("detail", detail.clone().into())]),
    }
}

pub(crate) fn error_text(i18n: &I18n, error: &ModelError) -> String {
    match error {
        ModelError::LastPlaylist => i18n.tr("error-last-playlist"),
        ModelError::PlaylistOnAir(_) => i18n.tr("error-playlist-on-air"),
        ModelError::EntryOnAir(_) => i18n.tr("error-entry-on-air"),
        ModelError::StopAfterInSingle => i18n.tr("error-stop-after-single"),
        ModelError::PlayerCountOutOfRange { max, .. } => {
            i18n.tr_args("error-player-count", &[("max", (*max).into())])
        }
        ModelError::PlayerBusy(_) => i18n.tr("error-player-busy"),
        ModelError::UnknownPlayer(_)
        | ModelError::UnknownPlaylist(_)
        | ModelError::UnknownEntry(_)
        | ModelError::UnknownCart(_)
        | ModelError::UnknownCartPage(_)
        | ModelError::UnknownCartPosition(_)
        | ModelError::UnknownTrack(_) => i18n.tr("error-not-found"),
        ModelError::InvalidMarker => i18n.tr("error-invalid-marker"),
        ModelError::LastCartPage => i18n.tr("error-last-cart-page"),
        ModelError::CartsWouldBeLost => i18n.tr("error-carts-would-be-lost"),
        ModelError::CartGridOutOfRange => i18n.tr("error-cart-grid"),
        ModelError::NoPlaylists => i18n.tr("error-no-playlists"),
    }
}

/// Closes the window for a restart. The close guard lets it through: what
/// was on air has been stopped, or nothing was.
fn begin_restart(restart: &AtomicBool, view: &mut ViewState, ctx: &egui::Context) {
    restart.store(true, Ordering::Release);
    view.close_confirmed = true;
    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
}

fn restart_reason_key(reason: RestartReason) -> &'static str {
    match reason {
        RestartReason::AudioSystem => "restart-reason-audio-system",
        RestartReason::SampleRate => "restart-reason-sample-rate",
        RestartReason::BufferSize => "restart-reason-buffer-size",
        RestartReason::Routes => "restart-reason-routes",
        RestartReason::BitPerfect => "restart-reason-bit-perfect",
        RestartReason::DsdOutput => "restart-reason-dsd",
        RestartReason::Limits => "restart-reason-limits",
        RestartReason::Tuning => "restart-reason-tuning",
    }
}

fn top_bar(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState, pending: &[RestartReason]) {
    let rect = ui.max_rect();
    ui.painter().rect_filled(rect, 0.0, theme::NEUTRAL_900);
    ui.painter().rect_filled(
        Rect::from_min_size(
            egui::pos2(rect.left(), rect.bottom() - 1.0),
            vec2(rect.width(), 1.0),
        ),
        0.0,
        theme::NEUTRAL_800,
    );
    ui.spacing_mut().item_spacing = vec2(8.0, 0.0);
    ui.add_space(10.0);

    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        ui.spacing_mut().item_spacing = vec2(12.0, 0.0);
        ui.add_space(10.0);
        let clock = chrono::Local::now().format("%H:%M:%S").to_string();
        widgets::tabular_label(ui, &clock, &font(13.0), theme::NEUTRAL_200);
        let label = scene.i18n.tr("top-settings");
        let width = ui
            .painter()
            .layout_no_wrap(label.clone(), font(12.0), theme::NEUTRAL_300)
            .size()
            .x
            + 34.0;
        let style = TileStyle {
            border: theme::NEUTRAL_800,
            hover_fill: theme::NEUTRAL_800,
            ..TileStyle::plain()
        };
        if widgets::tile(ui, vec2(width, 24.0), &label, true, style, |p, r, c| {
            p.text(
                r.center(),
                egui::Align2::CENTER_CENTER,
                format!("{} {label}", egui_phosphor::regular::GEAR_SIX),
                font(12.0),
                c,
            );
        })
        .clicked()
        {
            view_state.settings_open = true;
        }
        let about = scene.i18n.tr("tip-about");
        let style = TileStyle {
            border: theme::NEUTRAL_800,
            hover_fill: theme::NEUTRAL_800,
            ..TileStyle::plain()
        };
        if widgets::tile(ui, vec2(24.0, 24.0), &about, true, style, |p, r, c| {
            p.text(
                r.center(),
                egui::Align2::CENTER_CENTER,
                egui_phosphor::regular::INFO,
                font(14.0),
                c,
            );
        })
        .on_hover_text(&about)
        .clicked()
        {
            view_state.about_open = true;
        }
        if !pending.is_empty() {
            let i18n = scene.i18n;
            let label = i18n.tr("top-restart-pending");
            let reasons = pending
                .iter()
                .map(|r| i18n.tr(restart_reason_key(*r)))
                .collect::<Vec<_>>()
                .join(", ");
            let tip = i18n.tr_args("tip-restart-pending", &[("reasons", reasons.into())]);
            let width = ui
                .painter()
                .layout_no_wrap(label.clone(), font(12.0), theme::AMBER)
                .size()
                .x
                + 34.0;
            let style = TileStyle {
                border: theme::AMBER,
                content: theme::AMBER,
                hover_fill: theme::NEUTRAL_800,
                ..TileStyle::plain()
            };
            if widgets::tile(ui, vec2(width, 24.0), &label, true, style, |p, r, c| {
                p.text(
                    r.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{} {label}", egui_phosphor::regular::ARROWS_CLOCKWISE),
                    font(12.0),
                    c,
                );
            })
            .on_hover_text(tip)
            .clicked()
            {
                view_state.restart_requested = true;
            }
        }
    });
}

fn players_row(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    covers: &mut HashMap<TrackId, TextureHandle>,
) {
    let count = scene.state.players.len();
    if count == 0 {
        return;
    }
    let gap = 8.0;
    let available = ui.available_width();
    let width = ((available - gap * (count as f32 - 1.0)) / count as f32).max(MIN_COLUMN_WIDTH);
    let overflow = width * count as f32 + gap * (count as f32 - 1.0) > available + 0.5;
    // The horizontal scroll bar gets its own strip instead of covering the footers.
    let bar = if overflow {
        ui.spacing().scroll.bar_width + ui.spacing().scroll.bar_inner_margin + 2.0
    } else {
        0.0
    };
    let height = ui.available_height() - bar;
    egui::ScrollArea::horizontal()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing = vec2(gap, 0.0);
                for index in 0..count {
                    let (rect, _) =
                        ui.allocate_exact_size(vec2(width, height - 2.0), Sense::hover());
                    if !ui.is_rect_visible(rect) {
                        continue;
                    }
                    let mut column_ui = ui.new_child(
                        UiBuilder::new()
                            .max_rect(rect)
                            .id_salt(("player", index))
                            .layout(Layout::top_down(Align::Min)),
                    );
                    column_ui.set_clip_rect(rect.intersect(ui.clip_rect()));
                    player::column(&mut column_ui, scene, view_state, covers, index);
                }
            });
        });
}

fn legend(ui: &mut Ui, color: Color32, label: &str) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
        let (r, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
        ui.painter().rect_filled(r, 0.0, color);
        ui.add(
            egui::Label::new(
                RichText::new(label)
                    .font(font(11.0))
                    .color(theme::NEUTRAL_500),
            )
            .selectable(false),
        );
    });
}

fn status_bar(ui: &mut Ui, scene: &Scene<'_>, view_state: &ViewState, platform: &str, faults: u64) {
    let rect = ui.max_rect();
    ui.painter().rect_filled(rect, 0.0, theme::NEUTRAL_900);
    ui.painter().rect_filled(
        Rect::from_min_size(rect.min, vec2(rect.width(), 1.0)),
        0.0,
        theme::NEUTRAL_800,
    );
    let t = scene.i18n;
    ui.spacing_mut().item_spacing = vec2(18.0, 0.0);
    ui.add_space(12.0);
    let mut alerts: Vec<String> = scene
        .telemetry
        .buses
        .iter()
        .filter(|b| b.health == BusHealth::Lost)
        .map(|b| {
            t.tr_args(
                "alert-device-lost",
                &[("device", b.key.device.clone().into())],
            )
        })
        .collect();
    let ids: Vec<PlayerId> = scene.state.players.iter().map(|p| p.id).collect();
    alerts.extend(view_state.dropouts.alerts(scene.time, &ids, t));
    if faults > 0 {
        alerts.push(t.tr_args("alert-services-fault", &[("count", faults.into())]));
    }
    let notice = view_state
        .notice
        .as_ref()
        .filter(|(_, until)| *until > scene.time)
        .map(|(text, _)| text.clone());
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        ui.spacing_mut().item_spacing = vec2(18.0, 0.0);
        ui.add_space(12.0);
        ui.add(
            egui::Label::new(
                RichText::new(platform)
                    .font(font(11.0))
                    .color(theme::NEUTRAL_500),
            )
            .selectable(false),
        );
        legend(ui, theme::AMBER, &t.tr("legend-mix"));
        legend(ui, theme::INTRO, &t.tr("legend-intro"));
        legend(ui, theme::NEXT_ROW, &t.tr("legend-next"));
        legend(ui, theme::ON_AIR_ROW, &t.tr("legend-on-air"));
        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            let (text, color) = if let Some(alert) = alerts.first() {
                (alert.clone(), theme::ON_AIR_TEXT)
            } else if let Some(notice) = notice {
                (notice, theme::AMBER)
            } else {
                (t.tr("status-hints"), theme::NEUTRAL_500)
            };
            ui.add(
                egui::Label::new(RichText::new(text).font(font(11.0)).color(color))
                    .selectable(false)
                    .truncate(),
            );
        });
    });
}
