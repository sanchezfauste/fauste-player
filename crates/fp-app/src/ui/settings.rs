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
use fp_model::{Command, Config, PlayMode, PlayerId, PlayerRoutes, PlaylistId, Route};

use super::app::Scene;
use super::format;
use super::theme;
use super::widgets::{self, TileStyle, font, font_medium};
use crate::services::ServiceRequest;

/// Test tone frequencies (spec §8.4).
const MAIN_TONE_HZ: f32 = 1000.0;
const CUE_TONE_HZ: f32 = 440.0;
const SAMPLE_RATES: [u32; 6] = [44_100, 48_000, 88_200, 96_000, 176_400, 192_000];
const BUFFER_SIZES: [u32; 7] = [64, 128, 256, 512, 1024, 2048, 4096];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Section {
    #[default]
    Outputs,
    Players,
    Analysis,
    Playlists,
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
    backends: Option<Vec<BackendChoice>>,
    loading: Option<Receiver<Vec<BackendChoice>>>,
    names: HashMap<PlaylistId, String>,
    new_name: String,
    /// A folder dialog is open; it answers `None` when cancelled.
    folder: Option<Receiver<Option<PathBuf>>>,
}

impl SettingsState {
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
}

/// Draws the modal; returns `false` once it should close.
pub(crate) fn show(
    ctx: &egui::Context,
    scene: &Scene<'_>,
    st: &mut SettingsState,
    deps: &SettingsDeps<'_>,
) -> bool {
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
    let screen = ctx.content_rect();
    let size = vec2(
        (screen.width() - 48.0).clamp(320.0, 980.0),
        (screen.height() - 82.0).clamp(300.0, 680.0),
    );
    let modal = egui::Modal::new(egui::Id::new("settings"))
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
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                egui::Frame::new()
                                    .inner_margin(egui::Margin::symmetric(24, 20))
                                    .show(ui, |ui| {
                                        ui.spacing_mut().item_spacing = vec2(8.0, 6.0);
                                        match st.section {
                                            Section::Outputs => {
                                                st.load_devices(deps.backends, ctx);
                                                outputs(ui, scene, st);
                                            }
                                            Section::Players => players(ui, scene),
                                            Section::Analysis => analysis(ui, scene, deps),
                                            Section::Playlists => playlists(ui, scene, st),
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
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.add_space(16.0);
                        let (text, color) = match &deps.notice {
                            Some(n) => (n.clone(), theme::AMBER),
                            None => (t.tr("settings-applies-now"), theme::NEUTRAL_500),
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
    if modal.should_close() {
        open = false;
    }
    open
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
        for (section, glyph, key) in [
            (Section::Outputs, icon::SPEAKER_HIGH, "settings-tab-outputs"),
            (
                Section::Players,
                icon::SLIDERS_HORIZONTAL,
                "settings-tab-players",
            ),
            (Section::Analysis, icon::WAVEFORM, "settings-tab-analysis"),
            (Section::Playlists, icon::PLAYLIST, "settings-tab-playlists"),
        ] {
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
                st.section = section;
            }
        }
    });
}

fn heading(ui: &mut Ui, text: &str) {
    ui.add(
        egui::Label::new(
            RichText::new(text)
                .font(font_medium(20.0))
                .color(theme::TEXT),
        )
        .selectable(false),
    );
    ui.add_space(8.0);
}

/// A settings row: label (and hint) on the left, the control on the right.
fn row(ui: &mut Ui, label: &str, hint: Option<&str>, control: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.set_min_height(44.0);
        ui.allocate_ui_with_layout(vec2(220.0, 40.0), Layout::top_down(Align::Min), |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
            ui.add_space(4.0);
            ui.add(
                egui::Label::new(RichText::new(label).font(font(13.0)).color(theme::TEXT))
                    .selectable(false),
            );
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
        });
        ui.add_space(16.0);
        control(ui);
    });
    let width = ui.available_width();
    let (r, _) = ui.allocate_exact_size(vec2(width, 1.0), Sense::hover());
    ui.painter()
        .rect_filled(r, 0.0, theme::TEXT.gamma_multiply(0.06));
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

fn outputs(ui: &mut Ui, scene: &Scene<'_>, st: &mut SettingsState) {
    let t = scene.i18n;
    let config = &scene.state.config;
    heading(ui, &t.tr("settings-tab-outputs"));
    ui.add(
        egui::Label::new(
            RichText::new(format!("{}  {}", icon::INFO, t.tr("settings-restart-note")))
                .font(font(12.0))
                .color(theme::AMBER),
        )
        .selectable(false)
        .wrap(),
    );
    ui.add_space(8.0);
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
    let current = config.outputs.backend.clone();
    row(ui, &t.tr("settings-backend"), None, |ui| {
        let shown = current
            .clone()
            .unwrap_or_else(|| t.tr("settings-default-backend"));
        egui::ComboBox::from_id_salt("backend")
            .selected_text(shown)
            .width(260.0)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(current.is_none(), t.tr("settings-default-backend"))
                    .clicked()
                {
                    update(scene, |c| c.outputs.backend = None);
                }
                for b in &backends {
                    let text = match &b.unavailable {
                        Some(_) => t.tr_args(
                            "settings-backend-unavailable",
                            &[("name", b.id.clone().into())],
                        ),
                        None => b.id.clone(),
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
        egui::ComboBox::from_id_salt("rate")
            .selected_text(t.tr_args("unit-hz", &[("value", rate.into())]))
            .show_ui(ui, |ui| {
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
    let latency_ms = f64::from(buffer) / f64::from(rate.max(1)) * 1000.0;
    let latency = t.tr_args(
        "settings-latency",
        &[("ms", format!("{latency_ms:.1}").into())],
    );
    row(ui, &t.tr("settings-buffer"), Some(&latency), |ui| {
        egui::ComboBox::from_id_salt("buffer")
            .selected_text(buffer.to_string())
            .show_ui(ui, |ui| {
                for b in BUFFER_SIZES {
                    if ui.selectable_label(b == buffer, b.to_string()).clicked() {
                        update(scene, |c| c.outputs.buffer_frames = b);
                    }
                }
            });
    });
    ui.add_space(12.0);
    ui.add(
        egui::Label::new(
            RichText::new(t.tr("settings-player-routes").to_uppercase())
                .font(font(10.0))
                .color(theme::NEUTRAL_500),
        )
        .selectable(false),
    );
    ui.add_space(4.0);
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
                    player.id,
                    Bus::Main,
                    routes.as_ref().and_then(|r| r.main.clone()),
                    backend_id.as_deref(),
                    &devices,
                );
                route_picker(
                    ui,
                    scene,
                    player.id,
                    Bus::Cue,
                    routes.as_ref().and_then(|r| r.cue.clone()),
                    backend_id.as_deref(),
                    &devices,
                );
            });
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Bus {
    Main,
    Cue,
}

fn set_route(config: &mut Config, player: PlayerId, bus: Bus, route: Option<Route>) {
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

fn route_picker(
    ui: &mut Ui,
    scene: &Scene<'_>,
    player: PlayerId,
    bus: Bus,
    route: Option<Route>,
    backend: Option<&str>,
    devices: &[DeviceInfo],
) {
    let t = scene.i18n;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(8.0, 0.0);
        let bus_label = match bus {
            Bus::Main => t.tr("settings-main"),
            Bus::Cue => t.tr("settings-cue"),
        };
        ui.add_sized(
            vec2(40.0, 20.0),
            egui::Label::new(
                RichText::new(bus_label.to_uppercase())
                    .font(widgets::font_semibold(10.0))
                    .color(if bus == Bus::Main {
                        theme::ACCENT_400
                    } else {
                        theme::CUE
                    }),
            )
            .selectable(false),
        );
        let none_text = match bus {
            Bus::Main => t.tr("settings-default-device"),
            Bus::Cue => t.tr("settings-none"),
        };
        let device = route.as_ref().map(|r| r.device.clone());
        let shown = device
            .as_ref()
            .map(|d| {
                devices
                    .iter()
                    .find(|x| &x.id.0 == d)
                    .map_or_else(|| d.clone(), |x| x.name.clone())
            })
            .unwrap_or_else(|| none_text.clone());
        egui::ComboBox::from_id_salt(("device", player.0, bus == Bus::Main))
            .selected_text(shown)
            .width(260.0)
            .show_ui(ui, |ui| {
                if ui.selectable_label(device.is_none(), &none_text).clicked() {
                    update(scene, |c| set_route(c, player, bus, None));
                }
                for d in devices {
                    let on = device.as_deref() == Some(d.id.0.as_str());
                    if ui.selectable_label(on, &d.name).clicked()
                        && let Some(backend) = backend
                    {
                        let r = Route {
                            backend: backend.to_owned(),
                            device: d.id.0.clone(),
                            first_channel: 0,
                        };
                        update(scene, |c| set_route(c, player, bus, Some(r)));
                    }
                }
            });
        if let Some(r) = &route {
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
            egui::ComboBox::from_id_salt(("channels", player.0, bus == Bus::Main))
                .selected_text(pair(r.first_channel))
                .show_ui(ui, |ui| {
                    for first in (0..channels.saturating_sub(1)).step_by(2) {
                        if ui
                            .selectable_label(first == r.first_channel, pair(first))
                            .clicked()
                        {
                            let mut changed = r.clone();
                            changed.first_channel = first;
                            update(scene, |c| set_route(c, player, bus, Some(changed)));
                        }
                    }
                });
        }
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
        let width = ui
            .painter()
            .layout_no_wrap(label.clone(), font(12.0), theme::TEXT)
            .size()
            .x
            + 36.0;
        let enabled = target.is_some();
        if widgets::tile(
            ui,
            vec2(width, 26.0),
            &label,
            enabled,
            TileStyle::plain(),
            |p, r, c| {
                p.text(
                    r.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{} {label}", icon::WAVEFORM),
                    font(12.0),
                    c,
                );
            },
        )
        .clicked()
            && let Some(target) = target
        {
            scene.ctl.test_tone(target, freq);
        }
    });
    ui.add_space(4.0);
}

fn players(ui: &mut Ui, scene: &Scene<'_>) {
    let t = scene.i18n;
    let config = &scene.state.config;
    heading(ui, &t.tr("settings-tab-players"));
    let max = config.limits.max_players;
    let mut count = scene.state.players.len();
    let label = t.tr("settings-player-count");
    row(
        ui,
        &label,
        Some(&t.tr("settings-hint-player-count")),
        |ui| {
            if slider(ui, &mut count, 1..=max, 1.0, "", &label) {
                scene.ctl.send(Command::SetPlayerCount(count));
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
    let mut warning = config.players.end_warning_secs;
    let label = t.tr("settings-end-warning");
    row(ui, &label, Some(&t.tr("settings-hint-end-warning")), |ui| {
        if slider(ui, &mut warning, 0.0..=60.0, 1.0, " s", &label) {
            update(scene, |c| c.players.end_warning_secs = warning);
        }
    });
    ui.add_space(12.0);
    ui.add(
        egui::Label::new(
            RichText::new(t.tr_args("settings-language", &[("lang", t.lang().into())]))
                .font(font(11.0))
                .color(theme::NEUTRAL_500),
        )
        .selectable(false),
    );
}

/// An on/off switch drawn like the design's toggles.
fn toggle(ui: &mut Ui, on: &mut bool, label: &str) -> bool {
    let (rect, response) = ui.allocate_exact_size(vec2(36.0, 20.0), Sense::click());
    let value = *on;
    let owned = label.to_owned();
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, true, value, owned.clone())
    });
    if response.clicked() {
        *on = !*on;
    }
    let p = ui.painter();
    p.rect_filled(
        rect,
        0.0,
        if *on {
            theme::ACCENT
        } else {
            theme::NEUTRAL_700
        },
    );
    let knob_x = if *on {
        rect.right() - 18.0
    } else {
        rect.left() + 2.0
    };
    p.rect_filled(
        egui::Rect::from_min_size(egui::pos2(knob_x, rect.top() + 2.0), vec2(16.0, 16.0)),
        0.0,
        if *on {
            theme::NEUTRAL_100
        } else {
            theme::NEUTRAL_400
        },
    );
    response.clicked()
}

fn analysis(ui: &mut Ui, scene: &Scene<'_>, deps: &SettingsDeps<'_>) {
    let t = scene.i18n;
    let a = scene.state.config.analysis.clone();
    heading(ui, &t.tr("settings-tab-analysis"));
    type Field = (
        &'static str,
        f64,
        std::ops::RangeInclusive<f64>,
        f64,
        &'static str,
    );
    let fields: [Field; 6] = [
        (
            "settings-silence",
            a.silence_threshold_db,
            -96.0..=-10.0,
            1.0,
            " dB",
        ),
        (
            "settings-segue-threshold",
            a.segue_threshold_db,
            -60.0..=0.0,
            1.0,
            " dB",
        ),
        (
            "settings-segue-max",
            a.segue_max_secs,
            0.0..=60.0,
            0.5,
            " s",
        ),
        (
            "settings-outro-drop",
            a.outro_drop_db,
            0.0..=40.0,
            1.0,
            " dB",
        ),
        (
            "settings-outro-max",
            a.outro_max_secs,
            0.0..=300.0,
            1.0,
            " s",
        ),
        (
            "settings-min-duration",
            a.markers_min_duration_secs,
            0.0..=3600.0,
            5.0,
            " s",
        ),
    ];
    for (i, (key, value, range, step, suffix)) in fields.into_iter().enumerate() {
        let mut v = value;
        let label = t.tr(key);
        row(ui, &label, None, |ui| {
            if slider(ui, &mut v, range, step, suffix, &label) {
                update(scene, |c| {
                    let a = &mut c.analysis;
                    match i {
                        0 => a.silence_threshold_db = v,
                        1 => a.segue_threshold_db = v,
                        2 => a.segue_max_secs = v,
                        3 => a.outro_drop_db = v,
                        4 => a.outro_max_secs = v,
                        _ => a.markers_min_duration_secs = v,
                    }
                });
            }
        });
    }
    let label = t.tr("settings-reanalyse");
    row(ui, &label, Some(&t.tr("settings-hint-reanalyse")), |ui| {
        let width = ui
            .painter()
            .layout_no_wrap(label.clone(), font(12.0), theme::TEXT)
            .size()
            .x
            + 36.0;
        let enabled = deps.services.is_some();
        if widgets::tile(
            ui,
            vec2(width, 30.0),
            &label,
            enabled,
            TileStyle::plain(),
            |p, r, c| {
                p.text(
                    r.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{} {label}", icon::ARROWS_CLOCKWISE),
                    font(12.0),
                    c,
                );
            },
        )
        .clicked()
            && let Some(services) = deps.services
        {
            let _ = services.try_send(ServiceRequest::ReanalyseAll);
        }
    });
}

fn playlists(ui: &mut Ui, scene: &Scene<'_>, st: &mut SettingsState) {
    let t = scene.i18n;
    heading(ui, &t.tr("settings-tab-playlists"));
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
        let width = ui
            .painter()
            .layout_no_wrap(browse.clone(), font(12.0), theme::TEXT)
            .size()
            .x
            + 36.0;
        if widgets::tile(
            ui,
            vec2(width, 28.0),
            &browse,
            st.folder.is_none(),
            TileStyle::plain(),
            |p, r, c| {
                p.text(
                    r.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{} {browse}", icon::FOLDER_OPEN),
                    font(12.0),
                    c,
                );
            },
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
        let width = ui
            .painter()
            .layout_no_wrap(label.clone(), font(12.0), theme::TEXT)
            .size()
            .x
            + 36.0;
        let name = st.new_name.trim().to_owned();
        if widgets::tile(
            ui,
            vec2(width, 28.0),
            &label,
            !name.is_empty(),
            TileStyle::plain(),
            |p, r, c| {
                p.text(
                    r.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{} {label}", icon::PLUS),
                    font(12.0),
                    c,
                );
            },
        )
        .clicked()
        {
            scene.ctl.send(Command::CreatePlaylist { name });
            st.new_name.clear();
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
                .map(|t| t.play_length_secs())
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
}

fn pick_folder(scene: &Scene<'_>) -> Option<Receiver<Option<PathBuf>>> {
    let (tx, rx) = crossbeam_channel::bounded(1);
    let ctx = scene.ctx.clone();
    let start = scene.state.config.ui.music_dir.clone();
    let spawned = std::thread::Builder::new()
        .name("fp-folder-dialog".to_owned())
        .spawn(move || {
            let mut dialog = rfd::AsyncFileDialog::new();
            if let Some(dir) = start {
                dialog = dialog.set_directory(dir);
            }
            let folder = pollster::block_on(dialog.pick_folder());
            let _ = tx.send(folder.map(|f| f.path().to_path_buf()));
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
