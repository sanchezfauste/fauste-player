//! The Settings modal (spec §8.4, Phase 1 subset). Every edit goes through
//! the model: `UpdateConfig` with a validated copy of the configuration,
//! `SetPlayerCount`, and the playlist commands.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crossbeam_channel::{Receiver, Sender};
use egui::{Align, Color32, Layout, RichText, Sense, Ui, vec2};
use egui_phosphor::regular as icon;
use fp_backends::{AudioBackend, Availability, DeviceInfo};
use fp_model::{
    Command, Config, OutputsView, PlayMode, PlayerId, PlayerRoutes, PlaylistId, Route,
    SettingsSection,
};

use super::app::Scene;

mod carts;
mod columns;
mod devices;
mod keys;
mod loudness;
mod meters;
#[cfg(not(target_os = "android"))]
mod midi;
#[cfg(not(target_os = "android"))]
mod remote;
use super::format;
use super::theme;
use super::widgets::{self, TileStyle, font, font_medium, toggle};
use crate::i18n::Arg;
use crate::services::ServiceRequest;
pub(crate) use keys::RESERVED_KEYS;

/// Test tone frequencies (spec §8.4).
const MAIN_TONE_HZ: f32 = 1000.0;
const CUE_TONE_HZ: f32 = 440.0;
const SAMPLE_RATES: [u32; 6] = [44_100, 48_000, 88_200, 96_000, 176_400, 192_000];
const BUFFER_SIZES: [u32; 7] = [64, 128, 256, 512, 1024, 2048, 4096];

/// The Settings window has one size whatever the section (feedback 2 spec
/// O3); the section body scrolls inside it.
const WINDOW_SIZE: egui::Vec2 = vec2(900.0, 640.0);
/// Room kept around the window inside the main window.
const SCREEN_MARGIN: egui::Vec2 = vec2(48.0, 82.0);
const MIN_WINDOW_SIZE: egui::Vec2 = vec2(320.0, 300.0);

/// `WINDOW_SIZE`, shrunk to fit `screen`.
pub(crate) fn window_size(screen: egui::Vec2) -> egui::Vec2 {
    (screen - SCREEN_MARGIN)
        .min(WINDOW_SIZE)
        .max(MIN_WINDOW_SIZE)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Section {
    #[default]
    Outputs,
    Players,
    Meters,
    Analysis,
    Loudness,
    Playlists,
    Cartwall,
    Shortcuts,
    #[cfg(not(target_os = "android"))]
    Midi,
    #[cfg(not(target_os = "android"))]
    Remote,
}

/// One audio system and what it offers.
#[derive(Debug, Clone)]
pub(crate) struct BackendChoice {
    pub id: String,
    pub unavailable: Option<String>,
    pub devices: Vec<DeviceInfo>,
}

#[derive(Default)]
pub(crate) struct SettingsState {
    pub section: Section,
    pub(super) carts: carts::CartsState,
    pub(super) keys: keys::KeysState,
    /// The MIDI action waiting for a control (MIDI learn).
    pub(super) midi_learning: Option<fp_model::MidiAction>,
    #[cfg(not(target_os = "android"))]
    pub(super) remote: remote::RemoteState,
    /// The section whose "Restore defaults" waits for an answer.
    pub(super) confirm_restore: Option<SettingsSection>,
    backends: Option<Vec<BackendChoice>>,
    loading: Option<Receiver<Vec<BackendChoice>>>,
    names: HashMap<PlaylistId, String>,
    new_name: String,
    /// A folder dialog is open; it answers `None` when cancelled.
    folder: Option<Receiver<Option<PathBuf>>>,
    /// Number of players being dragged, sent on release.
    player_count_draft: Option<usize>,
}

impl SettingsState {
    /// True while Settings waits for a key to bind (Esc then cancels the
    /// capture, not the dialog).
    pub fn capturing(&self) -> bool {
        self.keys.capturing() || self.midi_learning.is_some() || self.confirm_restore.is_some()
    }

    /// The name the device pickers give `device`, once the device lists
    /// have been read (Settings opened once); its id until then. Nothing
    /// is enumerated for this (rule 8).
    pub(crate) fn device_label(&self, device: &fp_model::OutputDevice) -> String {
        match &self.backends {
            Some(backends) => devices::find(backends, device).1,
            None => device.device.clone(),
        }
    }

    /// Opens the Cartwall section on one cart (`Edit…` on a cart button).
    pub fn edit_cart(&mut self, page: fp_model::CartPageId, index: usize) {
        self.section = Section::Cartwall;
        self.carts.select(page, index);
    }

    /// Called when the modal opens: device lists are read again.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Enumerates devices on a helper thread: some systems take a while.
    fn load_devices(&mut self, backends: &[Arc<dyn AudioBackend>], ctx: &egui::Context) {
        if self.backends.is_some() || self.loading.is_some() {
            return;
        }
        let (tx, rx) = crossbeam_channel::bounded(1);
        let backends = backends.to_vec();
        let ctx = ctx.clone();
        let spawned = std::thread::Builder::new()
            .name("fp-device-scan".to_owned())
            .spawn(move || {
                let list = backends
                    .iter()
                    .map(|b| {
                        let unavailable = match b.availability() {
                            Availability::Available => None,
                            Availability::Unavailable(why) => Some(why),
                        };
                        let devices = if unavailable.is_none() {
                            b.enumerate_devices().unwrap_or_default()
                        } else {
                            Vec::new()
                        };
                        BackendChoice {
                            id: b.id().0,
                            unavailable,
                            devices,
                        }
                    })
                    .collect();
                let _ = tx.send(list);
                ctx.request_repaint();
            });
        match spawned {
            Ok(_) => self.loading = Some(rx),
            Err(e) => {
                tracing::error!(error = %e, "could not scan audio devices");
                self.backends = Some(Vec::new());
            }
        }
    }

    fn poll(&mut self) {
        if let Some(rx) = &self.loading
            && let Ok(list) = rx.try_recv()
        {
            self.backends = Some(list);
            self.loading = None;
        }
    }
}

/// Applies `edit` to a copy of the configuration and sends it if it changed.
fn update(scene: &Scene<'_>, edit: impl FnOnce(&mut Config)) {
    let mut config = scene.state.config.clone();
    edit(&mut config);
    let _ = config.validate();
    if config != scene.state.config {
        scene.ctl.send(Command::UpdateConfig(Box::new(config)));
    }
}

/// Everything the modal needs besides the scene.
pub(crate) struct SettingsDeps<'a> {
    pub backends: &'a [Arc<dyn AudioBackend>],
    pub services: Option<&'a Sender<ServiceRequest>>,
    pub notice: Option<String>,
    #[cfg(not(target_os = "android"))]
    pub midi: Option<&'a fp_control::service::MidiHandle>,
    #[cfg(not(target_os = "android"))]
    pub remote: Option<fp_remote::RemoteStatus>,
    /// An output change waits (live settings spec §9.1).
    pub pending: bool,
    /// One of them is an output change Apply now can apply.
    pub can_apply: bool,
    /// What waits, one item per line (Apply now's tooltip).
    pub pending_tip: String,
    /// How many tracks an earlier version analysed.
    pub outdated: usize,
    /// How many tracks wait for a loudness measurement (loudness
    /// normalization spec N22).
    pub unmeasured: usize,
}

/// What the modal asks of the application after a frame.
pub(crate) struct Outcome {
    /// `false` once the modal should close.
    pub open: bool,
    /// The operator pressed Apply now.
    pub apply_now: bool,
}

/// Draws the modal for one frame.
pub(crate) fn show(
    ctx: &egui::Context,
    scene: &Scene<'_>,
    st: &mut SettingsState,
    deps: &SettingsDeps<'_>,
) -> Outcome {
    let capturing = st.capturing();
    st.poll();
    if let Some(rx) = &st.folder
        && let Ok(picked) = rx.try_recv()
    {
        if let Some(dir) = picked {
            update(scene, |c| c.ui.music_dir = Some(dir));
        }
        st.folder = None;
    }
    let t = scene.i18n;
    let mut open = true;
    let mut apply_now = false;
    let size = window_size(ctx.content_rect().size());
    let modal = egui::Modal::new(egui::Id::unique("settings"))
        .frame(egui::Frame::new().fill(theme::SURFACE))
        .backdrop_color(theme::NEUTRAL_900.gamma_multiply(0.7))
        .show(ctx, |ui| {
            ui.set_min_size(size);
            ui.set_max_size(size);
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            // Header.
            ui.allocate_ui_with_layout(
                vec2(size.x, 44.0),
                Layout::left_to_right(Align::Center),
                |ui| {
                    ui.spacing_mut().item_spacing = vec2(10.0, 0.0);
                    ui.add_space(16.0);
                    ui.add(
                        egui::Label::new(
                            RichText::new(icon::GEAR_SIX)
                                .size(16.0)
                                .color(theme::ACCENT),
                        )
                        .selectable(false),
                    );
                    ui.add(
                        egui::Label::new(
                            RichText::new(t.tr("settings-title"))
                                .font(font_medium(15.0))
                                .color(theme::TEXT),
                        )
                        .selectable(false),
                    );
                },
            );
            separator(ui, size.x);
            let body_height = size.y - 44.0 - 52.0 - 2.0;
            ui.allocate_ui_with_layout(
                vec2(size.x, body_height),
                Layout::left_to_right(Align::Min),
                |ui| {
                    nav(ui, scene, st, body_height);
                    let (line, _) = ui.allocate_exact_size(vec2(1.0, body_height), Sense::hover());
                    ui.painter().rect_filled(line, 0.0, theme::NEUTRAL_800);
                    ui.vertical(|ui| {
                        ui.set_min_size(vec2(ui.available_width(), body_height));
                        section_header(ui, scene, st);
                        egui::ScrollArea::both()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                egui::Frame::new()
                                    .inner_margin(egui::Margin {
                                        left: 24,
                                        right: 24,
                                        top: 0,
                                        bottom: 20,
                                    })
                                    .show(ui, |ui| {
                                        ui.spacing_mut().item_spacing = vec2(8.0, 6.0);
                                        set_body_right(ui);
                                        match st.section {
                                            Section::Outputs => {
                                                st.load_devices(deps.backends, ctx);
                                                outputs(ui, scene, st);
                                            }
                                            Section::Players => players(ui, scene, st),
                                            Section::Meters => meters::section(ui, scene),
                                            Section::Analysis => analysis(ui, scene, deps),
                                            Section::Loudness => {
                                                loudness::section(ui, scene, deps.unmeasured)
                                            }
                                            Section::Playlists => playlists(ui, scene, st),
                                            Section::Cartwall => carts::section(ui, scene, st),
                                            Section::Shortcuts => keys::section(ui, scene, st),
                                            #[cfg(not(target_os = "android"))]
                                            Section::Midi => {
                                                midi::section(ui, scene, st, deps.midi)
                                            }
                                            #[cfg(not(target_os = "android"))]
                                            Section::Remote => {
                                                remote::section(ui, scene, st, deps.remote.as_ref())
                                            }
                                        }
                                    });
                            });
                    });
                },
            );
            separator(ui, size.x);
            // Footer.
            ui.allocate_ui_with_layout(
                vec2(size.x, 52.0),
                Layout::right_to_left(Align::Center),
                |ui| {
                    ui.spacing_mut().item_spacing = vec2(8.0, 0.0);
                    ui.add_space(16.0);
                    let close = t.tr("settings-close");
                    let style = TileStyle {
                        fill: theme::ACCENT,
                        border: theme::ACCENT,
                        content: theme::NEUTRAL_900,
                        hover_fill: theme::ACCENT_400,
                        hover_content: theme::NEUTRAL_900,
                        active_fill: theme::ACCENT_300,
                        ..TileStyle::plain()
                    };
                    if widgets::tile(ui, vec2(80.0, 30.0), &close, true, style, |p, r, c| {
                        p.text(
                            r.center(),
                            egui::Align2::CENTER_CENTER,
                            &close,
                            font_medium(13.0),
                            c,
                        );
                    })
                    .clicked()
                    {
                        open = false;
                    }
                    if deps.can_apply {
                        let label = t.tr("settings-apply-now");
                        let width = ui
                            .painter()
                            .layout_no_wrap(label.clone(), font_medium(13.0), theme::TEXT)
                            .size()
                            .x
                            + 28.0;
                        let style = TileStyle {
                            border: theme::AMBER,
                            ..TileStyle::plain()
                        };
                        if widgets::tile_without_tooltip(
                            ui,
                            vec2(width, 30.0),
                            &label,
                            true,
                            style,
                            |p, r, c| {
                                p.text(
                                    r.center(),
                                    egui::Align2::CENTER_CENTER,
                                    &label,
                                    font_medium(13.0),
                                    c,
                                );
                            },
                        )
                        .on_hover_text(&deps.pending_tip)
                        .clicked()
                        {
                            apply_now = true;
                        }
                    }
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.add_space(16.0);
                        let (text, color) = match (&deps.notice, deps.pending) {
                            (Some(n), _) => (n.clone(), theme::AMBER),
                            (None, true) => (t.tr("settings-pending"), theme::AMBER),
                            (None, false) => (t.tr("settings-applies-now"), theme::NEUTRAL_500),
                        };
                        ui.add(
                            egui::Label::new(RichText::new(text).font(font(11.0)).color(color))
                                .selectable(false)
                                .truncate(),
                        );
                    });
                },
            );
        });
    confirm_restore(ctx, scene, st);
    if modal.should_close() && !capturing {
        open = false;
    }
    Outcome { open, apply_now }
}

fn separator(ui: &mut Ui, width: f32) {
    let (r, _) = ui.allocate_exact_size(vec2(width, 1.0), Sense::hover());
    ui.painter().rect_filled(r, 0.0, theme::NEUTRAL_800);
}

fn nav(ui: &mut Ui, scene: &Scene<'_>, st: &mut SettingsState, height: f32) {
    let t = scene.i18n;
    ui.allocate_ui_with_layout(vec2(200.0, height), Layout::top_down(Align::Min), |ui| {
        ui.painter()
            .rect_filled(ui.max_rect(), 0.0, theme::NEUTRAL_900);
        ui.add_space(8.0);
        for &(section, glyph, key) in SECTIONS {
            let on = st.section == section;
            let label = t.tr(key);
            let style = TileStyle {
                fill: if on {
                    theme::SURFACE
                } else {
                    Color32::TRANSPARENT
                },
                border: Color32::TRANSPARENT,
                content: if on { theme::TEXT } else { theme::NEUTRAL_400 },
                hover_fill: if on {
                    theme::SURFACE
                } else {
                    Color32::TRANSPARENT
                },
                ..TileStyle::plain()
            };
            let response = widgets::tile(ui, vec2(200.0, 36.0), &label, true, style, |p, r, c| {
                p.text(
                    r.left_center() + vec2(16.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    format!("{glyph}   {label}"),
                    font(13.0),
                    c,
                );
                if on {
                    p.rect_filled(
                        egui::Rect::from_min_size(r.left_top(), vec2(2.0, r.height())),
                        0.0,
                        theme::ACCENT,
                    );
                }
            });
            if response.clicked() {
                #[cfg(not(target_os = "android"))]
                if st.section == Section::Remote && section != Section::Remote {
                    remote::flush(scene, &mut st.remote);
                }
                st.section = section;
            }
        }
    });
}

/// A plain Settings button, as wide as its label; true when clicked.
fn button(ui: &mut Ui, text: &str) -> bool {
    widgets::text_button(ui, None, text, true, TileStyle::plain()).clicked()
}

/// Every section: its nav glyph and its title.
const SECTIONS: &[(Section, &str, &str)] = &[
    (Section::Outputs, icon::SPEAKER_HIGH, "settings-tab-outputs"),
    (
        Section::Players,
        icon::SLIDERS_HORIZONTAL,
        "settings-tab-players",
    ),
    (Section::Meters, icon::GAUGE, "settings-tab-meters"),
    (Section::Analysis, icon::WAVEFORM, "settings-tab-analysis"),
    (
        Section::Loudness,
        icon::SPEAKER_SIMPLE_HIGH,
        "settings-tab-loudness",
    ),
    (Section::Playlists, icon::PLAYLIST, "settings-tab-playlists"),
    (
        Section::Cartwall,
        icon::SQUARES_FOUR,
        "settings-tab-cartwall",
    ),
    (Section::Shortcuts, icon::KEYBOARD, "settings-tab-shortcuts"),
    #[cfg(not(target_os = "android"))]
    (Section::Midi, icon::PIANO_KEYS, "settings-tab-midi"),
    #[cfg(not(target_os = "android"))]
    (Section::Remote, icon::BROADCAST, "settings-tab-remote"),
];

const SECTION_HEADER_HEIGHT: f32 = 56.0;

/// The model section a Settings section restores; `None` where the spec
/// gives no button (hardware, security, show data).
fn restorable(section: Section) -> Option<SettingsSection> {
    match section {
        Section::Players => Some(SettingsSection::Players),
        Section::Meters => Some(SettingsSection::Meters),
        Section::Analysis => Some(SettingsSection::Analysis),
        Section::Loudness => Some(SettingsSection::Loudness),
        Section::Shortcuts => Some(SettingsSection::Shortcuts),
        Section::Outputs | Section::Playlists | Section::Cartwall => None,
        #[cfg(not(target_os = "android"))]
        Section::Midi | Section::Remote => None,
    }
}

/// The section's title, and "Restore defaults" on the right where the
/// section has one. It stays put while the body scrolls.
fn section_header(ui: &mut Ui, scene: &Scene<'_>, st: &mut SettingsState) {
    let t = scene.i18n;
    let key = SECTIONS
        .iter()
        .find(|(s, _, _)| *s == st.section)
        .map_or("settings-title", |(_, _, key)| *key);
    let width = ui.available_width();
    ui.allocate_ui_with_layout(
        vec2(width, SECTION_HEADER_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.set_width(width);
            ui.add_space(24.0);
            ui.add(
                egui::Label::new(
                    RichText::new(t.tr(key))
                        .font(font_medium(20.0))
                        .color(theme::TEXT),
                )
                .selectable(false),
            );
            if let Some(section) = restorable(st.section) {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add_space(24.0);
                    if button(ui, &t.tr("settings-restore")) {
                        st.confirm_restore = Some(section);
                    }
                });
            }
        },
    );
}

/// Asks before restoring; drawn over Settings.
fn confirm_restore(ctx: &egui::Context, scene: &Scene<'_>, st: &mut SettingsState) {
    let Some(section) = st.confirm_restore else {
        return;
    };
    let t = scene.i18n;
    let mut answer = None;
    let modal = egui::Modal::new(egui::Id::unique("settings-restore"))
        .frame(egui::Frame::new().fill(theme::SURFACE).inner_margin(20.0))
        .backdrop_color(theme::NEUTRAL_900.gamma_multiply(0.5))
        .show(ctx, |ui| {
            ui.set_width(360.0);
            ui.spacing_mut().item_spacing = vec2(8.0, 12.0);
            ui.add(
                egui::Label::new(
                    RichText::new(t.tr("settings-restore-question"))
                        .font(font(13.0))
                        .color(theme::TEXT),
                )
                .selectable(false)
                .wrap(),
            );
            ui.horizontal(|ui| {
                if button(ui, &t.tr("settings-restore-cancel")) {
                    answer = Some(false);
                }
                if button(ui, &t.tr("settings-restore-confirm")) {
                    answer = Some(true);
                }
            });
        });
    // Esc or a click on the backdrop is "cancel".
    if answer.is_none() && modal.should_close() {
        answer = Some(false);
    }
    match answer {
        Some(true) => {
            scene.ctl.send(Command::RestoreDefaults(section));
            st.confirm_restore = None;
        }
        Some(false) => st.confirm_restore = None,
        None => {}
    }
}

fn body_right_id() -> egui::Id {
    egui::Id::unique("settings-body-right")
}

/// Notes where the section body ends on the right, before any row is drawn.
fn set_body_right(ui: &Ui) {
    let right = ui.max_rect().right();
    ui.data_mut(|d| d.insert_temp(body_right_id(), right));
}

/// The width left on the current line up to the body's right edge. A row
/// whose fixed parts (a slider, the test buttons) run past that edge widens
/// the body, and the body scrolls sideways; measured against the edge rather
/// than `available_width`, the rows below are not widened with it and their
/// lists keep to the column.
pub(super) fn column_width(ui: &Ui) -> f32 {
    let available = ui.available_width();
    // Right to left, the room ends at the cursor, not at the body's edge.
    if ui.layout().main_dir() == egui::Direction::RightToLeft {
        return available;
    }
    ui.data(|d| d.get_temp::<f32>(body_right_id()))
        .map_or(available, |right| {
            available.min(right - ui.cursor().min.x).max(0.0)
        })
}

/// The label column of every Settings row (feedback 2 spec O3).
pub(super) const LABEL_WIDTH: f32 = 180.0;

/// A settings row: the label (and hint) in a fixed column on the left, the
/// control filling the rest. `control` gets the label's id, for
/// `labelled_by`.
pub(super) fn labelled_row<R>(
    ui: &mut Ui,
    label: &str,
    hint: Option<&str>,
    control: impl FnOnce(&mut Ui, egui::Id) -> R,
) -> R {
    let out = ui
        .horizontal(|ui| {
            ui.set_min_height(44.0);
            let label_id = ui
                .allocate_ui_with_layout(
                    vec2(LABEL_WIDTH, 40.0),
                    Layout::top_down(Align::Min),
                    |ui| {
                        // The column keeps its width whatever the label's.
                        ui.set_width(LABEL_WIDTH);
                        ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
                        ui.add_space(4.0);
                        let id = ui
                            .add(
                                egui::Label::new(
                                    RichText::new(label).font(font(13.0)).color(theme::TEXT),
                                )
                                .selectable(false),
                            )
                            .id;
                        if let Some(hint) = hint {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(hint)
                                        .font(font(11.0))
                                        .color(theme::NEUTRAL_500),
                                )
                                .selectable(false)
                                .wrap(),
                            );
                        }
                        id
                    },
                )
                .inner;
            ui.add_space(16.0);
            let rest = column_width(ui);
            ui.allocate_ui_with_layout(
                vec2(rest, 40.0),
                Layout::left_to_right(Align::Center),
                |ui| {
                    ui.set_width(rest);
                    control(ui, label_id)
                },
            )
            .inner
        })
        .inner;
    let width = column_width(ui);
    let (r, _) = ui.allocate_exact_size(vec2(width, 1.0), Sense::hover());
    ui.painter()
        .rect_filled(r, 0.0, theme::TEXT.gamma_multiply(0.06));
    out
}

/// What a shared `device` asks for beyond `buffer` frames at `rate`, in ms,
/// as the backend works it out (`outputs.shared_buffer_ms`, no more than
/// the largest buffer the device reports); `None` when it asks for no more.
pub(super) fn shared_floor_ms(
    device: &DeviceInfo,
    buffer: u32,
    rate: u32,
    shared_ms: f64,
) -> Option<f64> {
    if !device.shared || shared_ms.is_nan() || shared_ms <= 0.0 {
        return None;
    }
    let rate = f64::from(rate.max(1));
    let mut floor = (shared_ms * rate / 1000.0).ceil();
    if let Some((_, max)) = device.buffer_frames {
        floor = floor.min(f64::from(max));
    }
    (floor > f64::from(buffer)).then(|| floor / rate * 1000.0)
}

/// The latency of `buffer` frames at `rate`, and the least a shared output
/// asks for (`shared_ms`, when it asks for more than `buffer`).
pub(super) fn latency_hint(
    t: &crate::i18n::I18n,
    buffer: u32,
    rate: u32,
    shared_ms: Option<f64>,
) -> String {
    let ms = format!("{:.1}", f64::from(buffer) / f64::from(rate.max(1)) * 1000.0);
    match shared_ms {
        Some(floor) => t.tr_args(
            "settings-latency-shared",
            &[("ms", ms.into()), ("floor", format!("{floor:.0}").into())],
        ),
        None => t.tr_args("settings-latency", &[("ms", ms.into())]),
    }
}

/// `labelled_row` for controls that name themselves.
pub(super) fn row(ui: &mut Ui, label: &str, hint: Option<&str>, control: impl FnOnce(&mut Ui)) {
    labelled_row(ui, label, hint, |ui, _| control(ui));
}

/// A drop-down that stays inside its column: its usual `width` (`None`:
/// egui's default) where the column has room, wider for a longer choice,
/// never past the column; the shown choice is shortened when
/// the column is narrower, and whole on hover. The open list (`menu`) shows
/// every choice in full.
pub(super) fn fitted_combo(
    ui: &mut Ui,
    id_salt: impl egui::AsIdSalt,
    shown: String,
    width: Option<f32>,
    menu: impl FnOnce(&mut Ui),
) -> egui::Response {
    let room = column_width(ui);
    let width = width.unwrap_or(ui.spacing().combo_width);
    let combo = |ui: &mut Ui| {
        egui::ComboBox::from_id_salt(id_salt)
            .selected_text(shown.clone())
            .width(room.min(width))
            .truncate()
            .show_ui(ui, menu)
            .response
    };
    // The combo box shortens its choice to the room its `Ui` has: where a
    // wider row has pushed that room out, give it the column instead. Both
    // ways go through a child `Ui`, so the combo keeps one id (and an open
    // list stays open) when a resize switches between them.
    let response = if ui.available_width() > room {
        let height = ui.spacing().interact_size.y;
        ui.allocate_ui(vec2(room, height), combo).inner
    } else {
        ui.scope(combo).inner
    };
    // A disabled list (one choice only) still gives its choice whole.
    response
        .on_hover_text(shown.clone())
        .on_disabled_hover_text(shown)
}

/// A small upper-case heading over a group of rows.
pub(super) fn caption(ui: &mut Ui, text: &str) {
    ui.add(
        egui::Label::new(
            RichText::new(text.to_uppercase())
                .font(font(10.0))
                .color(theme::NEUTRAL_500),
        )
        .selectable(false),
    );
}

/// A line of explanation, wrapped to the section's width.
pub(super) fn note(ui: &mut Ui, text: &str, color: Color32) {
    ui.add(
        egui::Label::new(RichText::new(text).font(font(12.0)).color(color))
            .selectable(false)
            .wrap(),
    );
}

/// Basic | Advanced (operator feedback 4, Q12.1). Only `ui.outputs_view`
/// changes: the advanced settings are hidden, never reset (Q12.7).
fn view_selector(ui: &mut Ui, scene: &Scene<'_>, view: OutputsView) {
    let t = scene.i18n;
    let basic = t.tr("settings-outputs-basic");
    let advanced = t.tr("settings-outputs-advanced");
    let views = [OutputsView::Basic, OutputsView::Advanced];
    row(ui, &t.tr("settings-outputs-view"), None, |ui| {
        let selected = usize::from(view == OutputsView::Advanced);
        if let Some(chosen) = widgets::segmented(
            ui,
            egui::Id::unique("outputs-view"),
            &[
                widgets::Segment {
                    text: &basic,
                    label: &basic,
                },
                widgets::Segment {
                    text: &advanced,
                    label: &advanced,
                },
            ],
            selected,
        )
        .and_then(|i| views.get(i).copied())
        {
            update(scene, |c| c.ui.outputs_view = chosen);
        }
    });
}

/// A slider whose accessible name is `label` (the row shows the same text).
fn slider<T: egui::emath::Numeric>(
    ui: &mut Ui,
    value: &mut T,
    range: std::ops::RangeInclusive<T>,
    step: f64,
    suffix: &str,
    label: &str,
) -> bool {
    let response = ui.add(egui::Slider::new(value, range).step_by(step).suffix(suffix));
    let owned = label.to_owned();
    let v = value.to_f64();
    response.widget_info(|| egui::WidgetInfo::slider(true, v, owned.clone()));
    response.changed()
}

/// The Number of players slider. Every count it sends removes or adds
/// players, so it sends one only when the count is settled: when the slider
/// is released (the count being dragged is kept in `draft` and shown), or
/// when a number typed into its box is complete (Enter or leaving the box;
/// Escape cancels it). Dragging through smaller counts, or typing the first
/// digit of a two-digit count, would otherwise remove the players above it
/// on the way. Returns the count to send.
fn count_slider(
    ui: &mut Ui,
    draft: &mut Option<usize>,
    current: usize,
    max: usize,
    label: &str,
) -> Option<usize> {
    let mut value = draft.unwrap_or(current);
    let before = value;
    let response = ui.add(
        egui::Slider::new(&mut value, 1..=max)
            .step_by(1.0)
            .update_while_editing(false),
    );
    let owned = label.to_owned();
    let v = egui::emath::Numeric::to_f64(value);
    response.widget_info(|| egui::WidgetInfo::slider(true, v, owned.clone()));
    if response.dragged() {
        *draft = Some(value);
        return None;
    }
    // The box reports each keystroke as a change; only a new value is one.
    if response.drag_stopped() || (response.changed() && value != before) {
        *draft = None;
        return (value != current).then_some(value);
    }
    None
}

/// The audio systems the list offers: the real systems in their order, then
/// `null` as "No output (silent)" (feedback 2 spec O27, which reverses the
/// hiding part of O5).
fn listed_backends(all: &[BackendChoice]) -> Vec<&BackendChoice> {
    all.iter()
        .filter(|b| b.id != "null")
        .chain(all.iter().filter(|b| b.id == "null"))
        .collect()
}

fn backend_name(t: &crate::i18n::I18n, id: &str) -> String {
    if id == "null" {
        t.tr("settings-backend-null")
    } else {
        fp_backends::display_name(id).to_owned()
    }
}

fn outputs(ui: &mut Ui, scene: &Scene<'_>, st: &mut SettingsState) {
    let t = scene.i18n;
    let config = &scene.state.config;
    let Some(backends) = st.backends.clone() else {
        ui.add(
            egui::Label::new(
                RichText::new(t.tr("settings-loading-devices"))
                    .font(font(13.0))
                    .color(theme::NEUTRAL_400),
            )
            .selectable(false),
        );
        return;
    };
    let view = config.ui.outputs_view;
    view_selector(ui, scene, view);
    let current = config.outputs.backend.clone();
    row(ui, &t.tr("settings-backend"), None, |ui| {
        let shown = current
            .as_deref()
            .map(|id| backend_name(t, id))
            .unwrap_or_else(|| t.tr("settings-default-backend"));
        // 260 points (as the language list), never past a narrow window; a
        // shortened name shows whole on hover.
        fitted_combo(ui, "backend", shown.clone(), Some(260.0), |ui| {
            if ui
                .selectable_label(current.is_none(), t.tr("settings-default-backend"))
                .clicked()
            {
                update(scene, |c| c.outputs.backend = None);
            }
            for b in listed_backends(&backends) {
                let text = match &b.unavailable {
                    Some(_) => t.tr_args(
                        "settings-backend-unavailable",
                        &[("name", backend_name(t, &b.id).into())],
                    ),
                    None => backend_name(t, &b.id),
                };
                let response = ui.add_enabled(
                    b.unavailable.is_none(),
                    egui::Button::selectable(current.as_deref() == Some(b.id.as_str()), text),
                );
                let response = match &b.unavailable {
                    Some(why) => response.on_disabled_hover_text(why),
                    None => response,
                };
                if response.clicked() {
                    let id = b.id.clone();
                    update(scene, |c| c.outputs.backend = Some(id));
                }
            }
        });
    });
    let rate = config.outputs.sample_rate;
    let buffer = config.outputs.buffer_frames;
    row(ui, &t.tr("settings-rate"), None, |ui| {
        let shown = t.tr_args("unit-hz", &[("value", rate.into())]);
        fitted_combo(ui, "rate", shown.clone(), None, |ui| {
            for r in SAMPLE_RATES {
                if ui
                    .selectable_label(r == rate, t.tr_args("unit-hz", &[("value", r.into())]))
                    .clicked()
                {
                    update(scene, |c| c.outputs.sample_rate = r);
                }
            }
        });
    });
    let backend_id = current.clone().or_else(|| {
        backends
            .iter()
            .find(|b| b.unavailable.is_none() && b.id != "null")
            .map(|b| b.id.clone())
    });
    let devices: Vec<DeviceInfo> = backends
        .iter()
        .find(|b| Some(&b.id) == backend_id.as_ref())
        .map(|b| b.devices.clone())
        .unwrap_or_default();
    // Outputs through a sound server may ask for more than the buffer below.
    let floor = devices
        .iter()
        .filter_map(|d| shared_floor_ms(d, buffer, rate, config.outputs.shared_buffer_ms))
        .reduce(f64::max);
    let latency = latency_hint(t, buffer, rate, floor);
    row(ui, &t.tr("settings-buffer"), Some(&latency), |ui| {
        let shown = buffer.to_string();
        fitted_combo(ui, "buffer", shown.clone(), None, |ui| {
            for b in BUFFER_SIZES {
                if ui.selectable_label(b == buffer, b.to_string()).clicked() {
                    update(scene, |c| c.outputs.buffer_frames = b);
                }
            }
        });
    });
    ui.add_space(12.0);
    caption(ui, &t.tr("settings-player-routes"));
    ui.add_space(4.0);
    for (n, player) in scene.state.players.iter().enumerate() {
        let routes = config
            .outputs
            .routes
            .iter()
            .find(|r| r.player == player.id)
            .cloned();
        let label = t.tr_args("player-label", &[("n", (n + 1).into())]);
        row(ui, &label, None, |ui| {
            ui.vertical(|ui| {
                route_picker(
                    ui,
                    scene,
                    Owner::Player(player.id),
                    Bus::Main,
                    routes.as_ref().and_then(|r| r.main.clone()),
                    backend_id.as_deref(),
                    &devices,
                );
                route_picker(
                    ui,
                    scene,
                    Owner::Player(player.id),
                    Bus::Cue,
                    routes.as_ref().and_then(|r| r.cue.clone()),
                    backend_id.as_deref(),
                    &devices,
                );
                let cue = fp_model::Holder::PlayerCue(player.id);
                if config.outputs.cue_shares_main(cue)
                    || fp_model::cue_on_air::cue_is_on_air(scene.state, cue)
                {
                    cue_on_air_note(ui, scene);
                }
            });
        });
    }
    let cart_routes = config.outputs.cartwall.clone();
    row(ui, &t.tr("settings-cartwall-outputs"), None, |ui| {
        ui.vertical(|ui| {
            route_picker(
                ui,
                scene,
                Owner::Cartwall,
                Bus::Main,
                cart_routes.main.clone(),
                backend_id.as_deref(),
                &devices,
            );
            route_picker(
                ui,
                scene,
                Owner::Cartwall,
                Bus::Cue,
                cart_routes.cue.clone(),
                backend_id.as_deref(),
                &devices,
            );
            let cue = fp_model::Holder::CartwallCue;
            if config.outputs.cue_shares_main(cue)
                || fp_model::cue_on_air::cue_is_on_air(scene.state, cue)
            {
                cue_on_air_note(ui, scene);
            }
        });
    });
    match view {
        OutputsView::Advanced => devices::section(ui, scene, &backends),
        OutputsView::Basic => {
            if config.outputs.advanced_in_use() {
                ui.add_space(8.0);
                note(
                    ui,
                    &t.tr("settings-outputs-advanced-in-use"),
                    theme::NEUTRAL_400,
                );
            }
        }
    }
}

/// On-air pre-listen spec §5: a Cue route that shares an on-air output is
/// usable; say that its pre-listen is heard on air and asks first while
/// something plays there.
fn cue_on_air_note(ui: &mut Ui, scene: &Scene<'_>) {
    ui.add(
        egui::Label::new(
            RichText::new(scene.i18n.tr("settings-cue-on-air"))
                .font(font(11.0))
                .color(theme::AMBER_TEXT),
        )
        .wrap(),
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Bus {
    Main,
    Cue,
}

/// Whose outputs a route picker edits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Owner {
    Player(PlayerId),
    Cartwall,
}

fn set_route(config: &mut Config, owner: Owner, bus: Bus, route: Option<Route>) {
    let player = match owner {
        Owner::Player(p) => p,
        Owner::Cartwall => {
            let c = &mut config.outputs.cartwall;
            match bus {
                Bus::Main => c.main = route,
                Bus::Cue => c.cue = route,
            }
            return;
        }
    };
    let routes = &mut config.outputs.routes;
    let index = match routes.iter().position(|r| r.player == player) {
        Some(i) => i,
        None => {
            routes.push(PlayerRoutes {
                player,
                main: None,
                cue: None,
            });
            routes.len() - 1
        }
    };
    if let Some(r) = routes.get_mut(index) {
        match bus {
            Bus::Main => r.main = route,
            Bus::Cue => r.cue = route,
        }
    }
}

/// Width of the channel-pair slot, kept empty when a bus has no device, so
/// that the test buttons line up (feedback 2 spec O3).
const CHANNELS_WIDTH: f32 = 96.0;
/// Width of the bus label (MAIN, CUE) in front of a route picker.
const BUS_LABEL_WIDTH: f32 = 40.0;
/// The narrowest device picker kept on the bus label's line next to the
/// channel pair and the test button; a narrower column moves those two to
/// a line of their own.
const DEVICE_MIN_WIDTH: f32 = 96.0;

fn route_picker(
    ui: &mut Ui,
    scene: &Scene<'_>,
    owner: Owner,
    bus: Bus,
    route: Option<Route>,
    backend: Option<&str>,
    devices: &[DeviceInfo],
) {
    let t = scene.i18n;
    // Both test buttons get the wider label's width: one column each.
    let test_width = [t.tr("settings-test-main"), t.tr("settings-test-cue")]
        .into_iter()
        .map(|l| {
            ui.painter()
                .layout_no_wrap(format!("{} {l}", icon::WAVEFORM), font(12.0), theme::TEXT)
                .size()
                .x
        })
        .fold(0.0, f32::max);
    let test_width = widgets::padded_button_width(test_width);
    const GAP: f32 = 8.0;
    let tail_width = CHANNELS_WIDTH + GAP + test_width;
    let one_line = BUS_LABEL_WIDTH + GAP + DEVICE_MIN_WIDTH + GAP + tail_width;
    let none_text = match bus {
        Bus::Main => t.tr("settings-default-device"),
        Bus::Cue => t.tr("settings-none"),
    };
    let device = |ui: &mut Ui| {
        device_box(
            ui,
            scene,
            owner,
            bus,
            route.as_ref(),
            backend,
            devices,
            &none_text,
        );
    };
    let test = |ui: &mut Ui| {
        let (label, freq) = match bus {
            Bus::Main => (t.tr("settings-test-main"), MAIN_TONE_HZ),
            Bus::Cue => (t.tr("settings-test-cue"), CUE_TONE_HZ),
        };
        // Main without a route plays on the default output; Cue needs one.
        let target = route.clone().or_else(|| {
            (bus == Bus::Main).then(|| Route {
                backend: backend.unwrap_or_default().to_owned(),
                device: String::new(),
                first_channel: 0,
            })
        });
        if widgets::text_button_at_least(
            ui,
            Some(icon::WAVEFORM),
            &label,
            target.is_some(),
            TileStyle::plain(),
            test_width,
        )
        .clicked()
            && let Some(target) = target
        {
            scene.ctl.test_tone(target, freq);
        }
    };
    // The channel pair's slot, kept when there is no pair to show.
    let channels = |ui: &mut Ui| {
        let width = CHANNELS_WIDTH.min(column_width(ui));
        ui.allocate_ui_with_layout(
            vec2(width, 26.0),
            Layout::left_to_right(Align::Center),
            |ui| {
                ui.set_width(width);
                if let Some(r) = &route {
                    channel_pair(ui, scene, owner, bus, r, devices);
                }
            },
        );
    };
    let bus_label = |ui: &mut Ui| {
        let text = match bus {
            Bus::Main => t.tr("settings-main"),
            Bus::Cue => t.tr("settings-cue"),
        };
        ui.add_sized(
            vec2(BUS_LABEL_WIDTH, 20.0),
            egui::Label::new(
                RichText::new(text.to_uppercase())
                    .font(widgets::font_semibold(10.0))
                    .color(if bus == Bus::Main {
                        theme::ACCENT_400
                    } else {
                        theme::CUE
                    }),
            )
            .selectable(false),
        );
    };
    let available = column_width(ui);
    if available >= one_line {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(GAP, 0.0);
            bus_label(ui);
            // Right to left: the test button, the channel pair, the device.
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                test(ui);
                channels(ui);
                device(ui);
            });
        });
        ui.add_space(4.0);
        return;
    }
    // A narrow column: the device picker beside the bus label (or under it,
    // where both do not fit), then the channel pair and the test button on a
    // line of their own (right-aligned as above), or one under the other
    // where even that line does not fit.
    if available >= BUS_LABEL_WIDTH + GAP + DEVICE_MIN_WIDTH {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(GAP, 0.0);
            bus_label(ui);
            device(ui);
        });
    } else {
        bus_label(ui);
        device(ui);
    }
    ui.add_space(4.0);
    if available >= tail_width {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(GAP, 0.0);
            ui.add_space(available - tail_width);
            channels(ui);
            test(ui);
        });
    } else {
        if route.is_some() {
            channels(ui);
            ui.add_space(4.0);
        }
        test(ui);
    }
    ui.add_space(4.0);
}

fn channel_pair(
    ui: &mut Ui,
    scene: &Scene<'_>,
    owner: Owner,
    bus: Bus,
    r: &Route,
    devices: &[DeviceInfo],
) {
    let t = scene.i18n;
    let channels = devices
        .iter()
        .find(|d| d.id.0 == r.device)
        .map_or(2, |d| d.channels.max(2));
    let pair = |first: u16| {
        t.tr_args(
            "settings-channels",
            &[
                ("first", (first + 1).into()),
                ("second", (first + 2).into()),
            ],
        )
    };
    let shown = pair(r.first_channel);
    fitted_combo(
        ui,
        ("channels", owner, bus == Bus::Main),
        shown.clone(),
        Some(CHANNELS_WIDTH - 8.0),
        |ui| {
            for first in (0..channels.saturating_sub(1)).step_by(2) {
                if ui
                    .selectable_label(first == r.first_channel, pair(first))
                    .clicked()
                {
                    let mut changed = r.clone();
                    changed.first_channel = first;
                    update(scene, |c| set_route(c, owner, bus, Some(changed)));
                }
            }
        },
    );
}

// The picker's parts, split for layout only.
#[allow(clippy::too_many_arguments)]
fn device_box(
    ui: &mut Ui,
    scene: &Scene<'_>,
    owner: Owner,
    bus: Bus,
    route: Option<&Route>,
    backend: Option<&str>,
    devices: &[DeviceInfo],
    none_text: &str,
) {
    let device = route.map(|r| r.device.clone());
    let labels = fp_backends::device_labels(devices);
    let shown = device
        .as_ref()
        .map(|d| {
            devices
                .iter()
                .zip(&labels)
                .find(|(x, _)| &x.id.0 == d)
                .map_or_else(|| d.clone(), |(_, label)| label.clone())
        })
        .unwrap_or_else(|| none_text.to_owned());
    // As wide as the room beside the bus label (or the channel pair); a
    // shortened name shows whole on hover.
    fitted_combo(
        ui,
        ("device", owner, bus == Bus::Main),
        shown.clone(),
        Some(f32::INFINITY),
        |ui| {
            if ui.selectable_label(device.is_none(), none_text).clicked() {
                update(scene, |c| set_route(c, owner, bus, None));
            }
            for (d, label) in devices.iter().zip(&labels) {
                let on = device.as_deref() == Some(d.id.0.as_str());
                if ui.selectable_label(on, label).clicked()
                    && let Some(backend) = backend
                {
                    let r = Route {
                        backend: backend.to_owned(),
                        device: d.id.0.clone(),
                        first_channel: 0,
                    };
                    update(scene, |c| set_route(c, owner, bus, Some(r)));
                }
            }
        },
    );
}

fn players(ui: &mut Ui, scene: &Scene<'_>, st: &mut SettingsState) {
    let t = scene.i18n;
    let config = &scene.state.config;
    let max = config.limits.max_players;
    let count = scene.state.players.len();
    let label = t.tr("settings-player-count");
    row(
        ui,
        &label,
        Some(&t.tr("settings-hint-player-count")),
        |ui| {
            if let Some(n) = count_slider(ui, &mut st.player_count_draft, count, max, &label) {
                scene.ctl.send(Command::SetPlayerCount(n));
            }
        },
    );
    row(
        ui,
        &t.tr("settings-default-mode"),
        Some(&t.tr("settings-hint-default-mode")),
        |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            for (mode, key) in [
                (PlayMode::Single, "mode-single"),
                (PlayMode::Continuous, "mode-cont"),
            ] {
                let on = config.players.default_mode == mode;
                let text = t.tr(key);
                let style = TileStyle {
                    fill: if on {
                        theme::NEUTRAL_700
                    } else {
                        Color32::TRANSPARENT
                    },
                    content: if on { theme::TEXT } else { theme::NEUTRAL_400 },
                    ..TileStyle::plain()
                };
                if widgets::tile(ui, vec2(70.0, 30.0), &text, true, style, |p, r, c| {
                    p.text(
                        r.center(),
                        egui::Align2::CENTER_CENTER,
                        &text,
                        font(12.0),
                        c,
                    );
                })
                .clicked()
                {
                    update(scene, |c| c.players.default_mode = mode);
                }
            }
        },
    );
    let mut fade = config.players.fade_ms;
    let label = t.tr("settings-fade");
    row(ui, &label, Some(&t.tr("settings-hint-fade")), |ui| {
        if slider(ui, &mut fade, 100..=5000, 100.0, " ms", &label) {
            update(scene, |c| c.players.fade_ms = fade);
        }
    });
    let mut auto = config.players.auto_segue;
    row(
        ui,
        &t.tr("settings-auto-segue"),
        Some(&t.tr("settings-hint-auto-segue")),
        |ui| {
            if toggle(ui, &mut auto, &t.tr("settings-auto-segue")) {
                update(scene, |c| c.players.auto_segue = auto);
            }
        },
    );
    // BR15: the two settings above are global; say they do not apply to
    // the players on a bit-perfect output, which never mix. Both stay
    // editable for the other players.
    if scene
        .state
        .players
        .iter()
        .any(|p| fp_model::bit_perfect_player(scene.state, p.id))
    {
        note(
            ui,
            &t.tr("settings-players-bit-perfect-note"),
            theme::NEUTRAL_400,
        );
    }
    let mut cue_markers = config.players.use_cue_markers;
    row(
        ui,
        &t.tr("settings-use-cue-markers"),
        Some(&t.tr("settings-hint-use-cue-markers")),
        |ui| {
            if toggle(ui, &mut cue_markers, &t.tr("settings-use-cue-markers")) {
                update(scene, |c| c.players.use_cue_markers = cue_markers);
            }
        },
    );
    let mut warning = config.players.end_warning_secs;
    let label = t.tr("settings-end-warning");
    row(ui, &label, Some(&t.tr("settings-hint-end-warning")), |ui| {
        if slider(ui, &mut warning, 0.0..=60.0, 1.0, " s", &label) {
            update(scene, |c| c.players.end_warning_secs = warning);
        }
    });
    let configured = config.ui.language.clone();
    // A tag no locale is registered for behaves as "System" and shows so.
    let current = configured
        .as_deref()
        .and_then(crate::i18n::locale)
        .map(|l| l.tag);
    labelled_row(ui, &t.tr("settings-language-title"), None, |ui, label| {
        // "System", then every language in its own name, in registry order.
        let system = t.tr("settings-language-system");
        let mut choices = vec![(None, system.as_str())];
        choices.extend(crate::i18n::LOCALES.iter().map(|l| (Some(l.tag), l.name)));
        let shown = choices
            .iter()
            .find(|(tag, _)| current == *tag)
            .map_or(system.as_str(), |(_, n)| *n)
            .to_owned();
        fitted_combo(ui, "language", shown.clone(), Some(260.0), |ui| {
            for (tag, name) in &choices {
                let on = current == *tag;
                if ui.selectable_label(on, *name).clicked() && configured.as_deref() != *tag {
                    let tag = tag.map(str::to_owned);
                    update(scene, |c| c.ui.language = tag);
                }
            }
        })
        .labelled_by(label);
    });
}

fn analysis(ui: &mut Ui, scene: &Scene<'_>, deps: &SettingsDeps<'_>) {
    let t = scene.i18n;
    let a = scene.state.config.analysis.clone();
    type Field = (
        &'static str,
        &'static str,
        f64,
        std::ops::RangeInclusive<f64>,
        f64,
        &'static str,
    );
    let fields: [Field; 7] = [
        (
            "settings-trim-threshold",
            "settings-hint-trim-threshold",
            a.trim_threshold_db,
            -120.0..=-20.0,
            1.0,
            " dB",
        ),
        (
            "settings-trim-margin",
            "settings-hint-trim-margin",
            f64::from(a.trim_margin_ms),
            0.0..=1000.0,
            5.0,
            " ms",
        ),
        (
            "settings-segue-drop",
            "settings-hint-segue-drop",
            a.segue_drop_db,
            3.0..=40.0,
            1.0,
            " dB",
        ),
        (
            "settings-segue-max",
            "settings-hint-segue-max",
            a.segue_max_secs,
            0.0..=60.0,
            0.5,
            " s",
        ),
        (
            "settings-outro-drop",
            "settings-hint-outro-drop",
            a.outro_drop_db,
            0.0..=40.0,
            1.0,
            " dB",
        ),
        (
            "settings-outro-max",
            "settings-hint-outro-max",
            a.outro_max_secs,
            0.0..=300.0,
            1.0,
            " s",
        ),
        (
            "settings-min-duration",
            "settings-hint-min-duration",
            a.markers_min_duration_secs,
            0.0..=3600.0,
            5.0,
            " s",
        ),
    ];
    for (i, (key, hint, value, range, step, suffix)) in fields.into_iter().enumerate() {
        let mut v = value;
        let label = t.tr(key);
        row(ui, &label, Some(&t.tr(hint)), |ui| {
            if slider(ui, &mut v, range, step, suffix, &label) {
                update(scene, |c| {
                    let a = &mut c.analysis;
                    match i {
                        0 => a.trim_threshold_db = v,
                        1 => a.trim_margin_ms = v.round().clamp(0.0, 1000.0) as u32,
                        2 => a.segue_drop_db = v,
                        3 => a.segue_max_secs = v,
                        4 => a.outro_drop_db = v,
                        5 => a.outro_max_secs = v,
                        _ => a.markers_min_duration_secs = v,
                    }
                });
            }
        });
    }
    let label = t.tr("settings-reanalyse");
    row(ui, &label, Some(&t.tr("settings-hint-reanalyse")), |ui| {
        // Main spec §6.1: one library pass at a time, with its progress.
        let pass = scene.analysis.pass;
        let enabled = deps.services.is_some() && pass.is_none();
        let text = pass.map_or_else(
            || label.clone(),
            |p| {
                t.tr_args(
                    "settings-reanalyse-running",
                    &[("done", p.done.into()), ("total", p.total.into())],
                )
            },
        );
        if widgets::text_button(
            ui,
            Some(icon::ARROWS_CLOCKWISE),
            &text,
            enabled,
            TileStyle::plain(),
        )
        .clicked()
            && let Some(services) = deps.services
        {
            let _ = services.try_send(ServiceRequest::ReanalyseAll);
        }
    });
    // Tracks an earlier version analysed wait for the operator (the
    // start-up notice offers the same).
    let outdated = deps.outdated;
    let label = t.tr_args("settings-analyse-outdated", &[("count", outdated.into())]);
    row(
        ui,
        &label,
        Some(&t.tr("settings-hint-analyse-outdated")),
        |ui| {
            let enabled = deps.services.is_some() && outdated > 0;
            if widgets::text_button(
                ui,
                Some(icon::ARROWS_CLOCKWISE),
                &label,
                enabled,
                TileStyle::plain(),
            )
            .clicked()
                && let Some(services) = deps.services
            {
                let _ = services.try_send(ServiceRequest::AnalyseOutdated);
            }
        },
    );
}

fn playlists(ui: &mut Ui, scene: &Scene<'_>, st: &mut SettingsState) {
    let t = scene.i18n;
    let dir = scene
        .state
        .config
        .ui
        .music_dir
        .as_ref()
        .map(|d| d.display().to_string())
        .unwrap_or_default();
    row(ui, &t.tr("settings-music-dir"), None, |ui| {
        ui.add(
            egui::Label::new(
                RichText::new(dir)
                    .font(font(12.0))
                    .color(theme::NEUTRAL_300),
            )
            .selectable(false)
            .truncate(),
        );
        let browse = t.tr("settings-browse");
        if widgets::text_button(
            ui,
            Some(icon::FOLDER_OPEN),
            &browse,
            st.folder.is_none(),
            TileStyle::plain(),
        )
        .clicked()
        {
            st.folder = pick_folder(scene);
        }
    });
    ui.add_space(12.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(8.0, 0.0);
        ui.add(
            egui::TextEdit::singleline(&mut st.new_name)
                .hint_text(t.tr("settings-new-playlist"))
                .desired_width(240.0),
        );
        let label = t.tr("settings-new-playlist");
        let name = st.new_name.trim().to_owned();
        if widgets::text_button(
            ui,
            Some(icon::PLUS),
            &label,
            !name.is_empty(),
            TileStyle::plain(),
        )
        .clicked()
        {
            scene.ctl.send(Command::CreatePlaylist { name });
            st.new_name.clear();
        }
        let import = t.tr("settings-import-playlist");
        // Read-only music on mobile (spec §2): no import or export.
        if scene.platform.can_write_user_files()
            && widgets::text_button(
                ui,
                Some(icon::DOWNLOAD_SIMPLE),
                &import,
                true,
                TileStyle::plain(),
            )
            .clicked()
        {
            super::playlist_files::import_with_dialog(
                &scene.ctx,
                scene.platform.clone(),
                scene.state.config.limits.clone(),
                scene.files.clone(),
            );
        }
    });
    ui.add_space(12.0);
    ui.horizontal(|ui| {
        for (key, width) in [
            ("settings-playlist-name", 300.0),
            ("settings-playlist-tracks", 70.0),
            ("settings-playlist-duration", 80.0),
        ] {
            ui.add_sized(
                vec2(width, 16.0),
                egui::Label::new(
                    RichText::new(t.tr(key).to_uppercase())
                        .font(font(10.0))
                        .color(theme::NEUTRAL_500),
                )
                .selectable(false),
            );
        }
    });
    let lists: Vec<_> = scene
        .state
        .playlists
        .iter()
        .map(|p| {
            let total: f64 = p
                .entries
                .iter()
                .filter_map(|e| scene.state.library.get(e.track))
                .map(|t| {
                    t.play_range(scene.state.config.players.use_cue_markers)
                        .length()
                })
                .sum();
            (p.id, p.name.clone(), p.entries.len(), total)
        })
        .collect();
    for (id, name, count, total) in lists {
        ui.horizontal(|ui| {
            ui.set_min_height(36.0);
            ui.spacing_mut().item_spacing = vec2(8.0, 0.0);
            let edit = st.names.entry(id).or_insert_with(|| name.clone());
            let response = ui.add(egui::TextEdit::singleline(edit).desired_width(300.0));
            let cancelled = ui.input(|i| i.key_pressed(egui::Key::Escape));
            let commit = !cancelled
                && response.lost_focus()
                && edit.trim() != name
                && !edit.trim().is_empty();
            if commit {
                scene.ctl.send(Command::RenamePlaylist {
                    playlist: id,
                    name: edit.trim().to_owned(),
                });
            }
            if !response.has_focus() && !commit && *edit != name {
                *edit = name.clone();
            }
            ui.add_sized(
                vec2(70.0, 20.0),
                egui::Label::new(
                    RichText::new(count.to_string())
                        .font(font(13.0))
                        .color(theme::NEUTRAL_300),
                )
                .selectable(false),
            );
            ui.add_sized(
                vec2(80.0, 20.0),
                egui::Label::new(
                    RichText::new(format::clock(total))
                        .font(font(13.0))
                        .color(theme::NEUTRAL_300),
                )
                .selectable(false),
            );
            let export = t.tr("settings-export-playlist");
            if scene.platform.can_write_user_files()
                && widgets::tile(
                    ui,
                    vec2(56.0, 30.0),
                    &t.tr_args("tip-export-playlist", &[("name", Arg::Text(name.clone()))]),
                    true,
                    TileStyle {
                        border: Color32::TRANSPARENT,
                        content: theme::NEUTRAL_300,
                        ..TileStyle::plain()
                    },
                    |p, r, c| {
                        p.text(
                            r.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("{} {export}", icon::EXPORT),
                            font(12.0),
                            c,
                        );
                    },
                )
                .clicked()
            {
                super::playlist_files::export_with_dialog(
                    &scene.ctx,
                    scene.platform.clone(),
                    format!("{}.m3u8", super::playlist_files::safe_file_name(&name)),
                    super::playlist_files::export_entries(scene.state, id),
                    scene.files.clone(),
                );
            }
            let delete = t.tr("settings-delete");
            let only = scene.state.playlists.len() <= 1;
            let style = TileStyle {
                border: Color32::TRANSPARENT,
                content: if only {
                    theme::NEUTRAL_600
                } else {
                    theme::NEUTRAL_400
                },
                ..TileStyle::plain()
            };
            if widgets::tile(ui, vec2(32.0, 30.0), &delete, true, style, |p, r, c| {
                widgets::glyph(p, r, icon::TRASH, 15.0, c, false);
            })
            .clicked()
            {
                scene.ctl.send(Command::DeletePlaylist(id));
            }
        });
    }
    // O24: the columns of every player's table.
    ui.add_space(16.0);
    columns::section(ui, scene);
}

fn pick_folder(scene: &Scene<'_>) -> Option<Receiver<Option<PathBuf>>> {
    let (tx, rx) = crossbeam_channel::bounded(1);
    let ctx = scene.ctx.clone();
    let start = scene.state.config.ui.music_dir.clone();
    let platform = scene.platform.clone();
    let spawned = std::thread::Builder::new()
        .name("fp-folder-dialog".to_owned())
        .spawn(move || {
            let _ = tx.send(platform.pick_folder(start));
            ctx.request_repaint();
        });
    match spawned {
        Ok(_) => Some(rx),
        Err(e) => {
            tracing::error!(error = %e, "could not open the folder dialog");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_window_has_one_size_clamped_to_the_screen() {
        assert_eq!(window_size(vec2(1600.0, 940.0)), WINDOW_SIZE);
        assert_eq!(window_size(vec2(700.0, 500.0)), vec2(652.0, 418.0));
        assert_eq!(window_size(vec2(100.0, 100.0)), MIN_WINDOW_SIZE);
    }

    fn choice(id: &str) -> BackendChoice {
        BackendChoice {
            id: id.to_owned(),
            unavailable: None,
            devices: Vec::new(),
        }
    }

    #[test]
    fn null_is_always_listed_after_the_real_systems() {
        let all = [choice("null"), choice("alsa"), choice("jack")];
        let ids: Vec<&str> = listed_backends(&all)
            .into_iter()
            .map(|b| b.id.as_str())
            .collect();
        assert_eq!(ids, vec!["alsa", "jack", "null"]);
    }
}
