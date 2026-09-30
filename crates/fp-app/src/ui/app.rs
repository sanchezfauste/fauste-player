//! The main screen (spec §8.3). The UI reads model and telemetry snapshots
//! every frame, sends commands, and keeps only view state.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender};
use egui::{
    Align, Color32, Key, Layout, Rect, RichText, Sense, TextureHandle, Ui, UiBuilder, vec2,
};
use fp_backends::AudioBackend;
use fp_engine::bus::BusHealth;
use fp_engine::conductor::Telemetry;
use fp_model::{
    AppState, Command, EntryId, KeyChord, ModelError, PlayerId, PlaylistId, ShortcutAction,
    TrackId, Transport,
};

use super::about::{self, NoticeOpener};
use super::cartwall;
use super::controller::Controller;
use super::files::{AUDIO_EXTENSIONS, audio_paths};
use super::player;
use super::playlist_files::{self, FileOutcome};
use super::settings::{self, SettingsDeps, SettingsState};
use super::theme;
use super::widgets::{self, TileStyle, font, font_medium};
use crate::i18n::I18n;
use crate::services::{MediaCache, ServiceRequest};

const MIN_COLUMN_WIDTH: f32 = 380.0;
const TOP_BAR_HEIGHT: f32 = 34.0;
const STATUS_BAR_HEIGHT: f32 = 24.0;
const NOTICE_SECS: f64 = 5.0;
/// Idle repaint period (the clock).
const IDLE_REPAINT: Duration = Duration::from_millis(100);

/// The payload of an entry being dragged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DragEntry {
    pub entry: EntryId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DropTarget {
    pub playlist: PlaylistId,
    pub index: usize,
}

/// Files chosen in a file dialog, to insert at `index` of `playlist`.
pub(crate) struct Picked {
    playlist: PlaylistId,
    index: usize,
    paths: Vec<PathBuf>,
}

/// Everything the UI remembers between frames.
#[derive(Default)]
pub(crate) struct ViewState {
    pub selection: HashMap<PlayerId, EntryId>,
    pub active_player: Option<PlayerId>,
    pub drop: Option<DropTarget>,
    pub file_drop: Option<DropTarget>,
    pub widths: HashMap<PlayerId, [f32; 4]>,
    pub resizing: HashSet<PlayerId>,
    pub rows_built: usize,
    pub settings_open: bool,
    pub about_open: bool,
    /// Where each player's waveform menu was opened, in seconds.
    pub wave_menu: HashMap<PlayerId, f64>,
    /// The table width and column fractions each player's table was last
    /// laid out with (a change resets egui's column widths).
    pub table_layout: HashMap<PlayerId, (f32, Option<[f32; 4]>)>,
    /// Zoomed waveforms; a player without one shows the whole track.
    pub wave_zoom: HashMap<PlayerId, super::wave_view::WaveZoom>,
    /// A marker being dragged on a waveform, and the track it belongs to.
    pub marker_drag: Option<(PlayerId, fp_model::MarkerKind, TrackId)>,
    /// A cart to open in Settings → Cartwall (`Edit…` on a cart).
    pub edit_cart: Option<(fp_model::CartPageId, usize)>,
    notice: Option<(String, f64)>,
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
}

impl Scene<'_> {
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
}

impl AppUi {
    pub fn new(ctl: Arc<dyn Controller>, i18n: I18n, media: MediaCache) -> Self {
        let (picks_tx, picks_rx) = crossbeam_channel::unbounded();
        let (files_tx, files_rx) = crossbeam_channel::unbounded();
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
            ctx: None,
            pending_imports: Vec::new(),
            inbox: None,
            language: None,
            backends: Vec::new(),
            service_faults: None,
            notices: None,
            opener: about::system_opener(),
        }
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
        self.view.file_drop = None;
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
        self.keyboard(&ctx, &state);
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
        top_bar(&mut top_ui, &scene, &mut self.view);
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
            };
            self.view.settings_open = settings::show(&ctx, &scene, &mut self.settings, &deps);
        } else {
            self.settings_shown = false;
            if self.view.about_open {
                self.view.about_open =
                    about::show(&ctx, &scene, self.notices.as_deref(), &self.opener);
            } else {
                self.file_drops(&ctx, &state);
            }
        }
        let busy = state
            .players
            .iter()
            .any(|p| p.transport == Transport::Playing || p.fading || p.cue.is_some());
        if busy {
            ctx.request_repaint();
        } else {
            ctx.request_repaint_after(IDLE_REPAINT);
        }
    }

    fn keyboard(&mut self, ctx: &egui::Context, state: &AppState) {
        if ctx.text_edit_focused() {
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
    let player = |n: u16| {
        state
            .players
            .get(usize::from(n).checked_sub(1)?)
            .map(|p| p.id)
    };
    let wall = &state.cartwall;
    let page_step = |step: isize| {
        let shown = wall.shown_page().map(|p| p.id);
        let index = wall.pages.iter().position(|p| Some(p.id) == shown)?;
        let count = wall.pages.len() as isize;
        let next = (index as isize + step).rem_euclid(count.max(1));
        wall.pages.get(usize::try_from(next).ok()?).map(|p| p.id)
    };
    Some(match action {
        ShortcutAction::PlayPlayer(n) => Command::Play(player(n)?),
        ShortcutAction::PausePlayer(n) => Command::Pause(player(n)?),
        ShortcutAction::StopPlayer(n) => Command::Stop(player(n)?),
        ShortcutAction::FadeStopPlayer(n) => Command::FadeStop(player(n)?),
        ShortcutAction::CuePlayer(n) => Command::ToggleCue(player(n)?),
        ShortcutAction::RestartPlayer(n) => Command::Restart(player(n)?),
        ShortcutAction::PreviousPlayer(n) => Command::Previous(player(n)?),
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

pub(crate) fn error_text(i18n: &I18n, error: &ModelError) -> String {
    match error {
        ModelError::LastPlaylist => i18n.tr("error-last-playlist"),
        ModelError::PlaylistOnAir(_) => i18n.tr("error-playlist-on-air"),
        ModelError::EntryOnAir(_) => i18n.tr("error-entry-on-air"),
        ModelError::NextIsCurrent => i18n.tr("error-next-is-current"),
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

fn top_bar(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState) {
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
    ui.add(
        egui::Label::new(
            RichText::new(egui_phosphor::fill::BROADCAST)
                .family(egui::FontFamily::Name(theme::ICONS_FILL.into()))
                .size(15.0)
                .color(theme::ACCENT),
        )
        .selectable(false),
    );
    let name = ui.add(
        egui::Label::new(
            RichText::new(scene.i18n.tr("app-name"))
                .font(font_medium(12.0))
                .color(theme::TEXT),
        )
        .selectable(false),
    );
    let version = ui.add(
        egui::Label::new(
            RichText::new(format!("v{}", about::VERSION))
                .font(font(11.0))
                .color(theme::NEUTRAL_500),
        )
        .selectable(false),
    );
    let about_label = scene.i18n.tr("tip-about");
    let response = ui
        .interact(
            name.rect.union(version.rect),
            ui.id().with("about"),
            Sense::click(),
        )
        .on_hover_text(&about_label);
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, about_label.clone())
    });
    if response.clicked() {
        view_state.about_open = true;
    }

    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        ui.spacing_mut().item_spacing = vec2(12.0, 0.0);
        ui.add_space(10.0);
        let clock = chrono::Local::now().format("%H:%M:%S").to_string();
        ui.add(
            egui::Label::new(
                RichText::new(clock)
                    .font(font(13.0))
                    .color(theme::NEUTRAL_200),
            )
            .selectable(false),
        );
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
