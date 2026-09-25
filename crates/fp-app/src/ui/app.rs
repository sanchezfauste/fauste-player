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
use fp_engine::bus::BusHealth;
use fp_engine::conductor::Telemetry;
use fp_model::{AppState, Command, EntryId, ModelError, PlayerId, PlaylistId, TrackId, Transport};

use super::controller::Controller;
use super::files::{AUDIO_EXTENSIONS, audio_paths};
use super::player;
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
                let paths = pollster::block_on(dialog.pick_files())
                    .map(|files| files.iter().map(|f| f.path().to_path_buf()).collect())
                    .unwrap_or_default();
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
        }
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

    /// Table rows built during the last frame (virtualisation check).
    pub fn rows_built(&self) -> usize {
        self.view.rows_built
    }

    pub fn ui(&mut self, ui: &mut Ui) {
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
        while let Ok(picked) = self.picks_rx.try_recv() {
            let paths = audio_paths(&picked.paths);
            if !paths.is_empty() {
                self.ctl.send(Command::InsertPaths {
                    playlist: picked.playlist,
                    index: picked.index,
                    paths,
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
        status_bar(&mut status_ui, &scene, &self.view, &self.platform);
        ui.allocate_rect(full, Sense::hover());
        self.file_drops(&ctx);
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
            let pressed: Vec<usize> = NUMBERS
                .iter()
                .enumerate()
                .filter(|(_, k)| i.key_pressed(**k) && i.modifiers.is_none())
                .map(|(n, _)| n)
                .collect();
            (
                pressed,
                i.key_pressed(Key::Delete) || i.key_pressed(Key::Backspace),
                i.key_pressed(Key::Escape),
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
            self.ctl.send(Command::RemoveEntry(entry));
        }
        if escape {
            self.view.selection.clear();
        }
    }

    fn file_drops(&mut self, ctx: &egui::Context) {
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
        let Some(target) = self.view.file_drop else {
            return;
        };
        let paths = audio_paths(&dropped);
        if !paths.is_empty() {
            self.ctl.send(Command::InsertPaths {
                playlist: target.playlist,
                index: target.index,
                paths,
            });
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
        other => i18n.tr_args("error-other", &[("detail", other.to_string().into())]),
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
    let height = ui.available_height();
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

fn status_bar(ui: &mut Ui, scene: &Scene<'_>, view_state: &ViewState, platform: &str) {
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
    let alerts: Vec<String> = scene
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
