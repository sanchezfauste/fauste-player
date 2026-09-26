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
use fp_model::{AppState, Command, EntryId, ModelError, PlayerId, PlaylistId, TrackId, Transport};

use super::controller::Controller;
use super::files::{AUDIO_EXTENSIONS, audio_paths};
use super::player;
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
/// Peak-hold fall rate, in meter fractions per second.
const HOLD_DECAY_PER_SEC: f32 = 0.35;

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

#[derive(Debug, Clone, Copy, Default)]
struct Meter {
    holds: [f32; 2],
    at: f64,
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
    meters: HashMap<PlayerId, Meter>,
    notice: Option<(String, f64)>,
}

impl ViewState {
    /// Peak hold for a player's meter, decaying over time.
    pub fn meter(&mut self, player: PlayerId, levels: [f32; 2], now: f64) -> [f32; 2] {
        let m = self.meters.entry(player).or_default();
        let dt = (now - m.at).clamp(0.0, 1.0) as f32;
        m.at = now;
        for (hold, level) in m.holds.iter_mut().zip(levels) {
            *hold = if level >= *hold {
                level
            } else {
                (*hold - dt * HOLD_DECAY_PER_SEC).max(0.0)
            };
        }
        m.holds
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
    themed: bool,
    #[cfg(feature = "test-hooks")]
    fail_next_frame: bool,
    settings: SettingsState,
    settings_shown: bool,
    backends: Vec<Arc<dyn AudioBackend>>,
    service_faults: Option<Arc<std::sync::atomic::AtomicU64>>,
}

impl AppUi {
    pub fn new(ctl: Arc<dyn Controller>, i18n: I18n, media: MediaCache) -> Self {
        let (picks_tx, picks_rx) = crossbeam_channel::unbounded();
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
            themed: false,
            #[cfg(feature = "test-hooks")]
            fail_next_frame: false,
            settings: SettingsState::default(),
            settings_shown: false,
            backends: Vec::new(),
            service_faults: None,
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
        let mut players_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(middle.shrink(8.0))
                .layout(Layout::left_to_right(Align::Min)),
        );
        players_row(&mut players_ui, &scene, &mut self.view, &mut self.covers);
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
            self.file_drops(&ctx, &state);
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
        const NUMBERS: [Key; 9] = [
            Key::Num1,
            Key::Num2,
            Key::Num3,
            Key::Num4,
            Key::Num5,
            Key::Num6,
            Key::Num7,
            Key::Num8,
            Key::Num9,
        ];
        let (pressed, delete, escape) = ctx.input(|i| {
            // Only the first press counts: holding a key must not repeat it
            // (a repeated Play would skip tracks on air).
            let first_press = |wanted: Key| {
                i.events.iter().any(|e| {
                    matches!(e, egui::Event::Key { key, pressed: true, repeat: false, modifiers, .. }
                        if *key == wanted && modifiers.is_none())
                })
            };
            let pressed: Vec<usize> = NUMBERS
                .iter()
                .enumerate()
                .filter(|(_, k)| first_press(**k))
                .map(|(n, _)| n)
                .collect();
            (
                pressed,
                first_press(Key::Delete) || first_press(Key::Backspace),
                first_press(Key::Escape),
            )
        });
        if self.view.settings_open {
            if escape {
                self.view.settings_open = false;
            }
            return;
        }
        for n in pressed {
            if let Some(p) = state.players.get(n) {
                self.ctl.send(Command::Play(p.id));
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

    fn file_drops(&mut self, ctx: &egui::Context, state: &AppState) {
        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .map(|f| f.path().to_path_buf())
                .filter(|p| !p.as_os_str().is_empty())
                .collect()
        });
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
        | ModelError::UnknownEntry(_) => i18n.tr("error-not-found"),
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
    ui.add(
        egui::Label::new(
            RichText::new(scene.i18n.tr("app-name"))
                .font(font_medium(12.0))
                .color(theme::TEXT),
        )
        .selectable(false),
    );
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
