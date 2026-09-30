//! One player column (spec §8.3): header, info row, transport, waveform,
//! playlist tabs, track table and footer.

use std::collections::HashMap;

use egui::{
    Align, Color32, Layout, Rect, RichText, Sense, Stroke, StrokeKind, TextureHandle, Ui,
    UiBuilder, pos2, vec2,
};
use egui_phosphor::regular as icon;
use fp_model::{Command, MarkerKind, PlayMode, PlayerId, PlaylistId, TrackId};

use super::app::{DragEntry, Scene, ViewState};
use super::format;
use super::icons;
use super::table;
use super::theme;
use super::view::{self, PlayerStatus, PlayerView};
use super::widgets::{self, TileStyle, font, font_medium, font_semibold};

const PLAY_SIZE: f32 = 64.0;
const GAP: f32 = 6.0;
const WAVE_HEIGHT: f32 = 56.0;
const TABS_HEIGHT: f32 = 30.0;
const FOOTER_HEIGHT: f32 = 24.0;

pub(crate) fn column(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    covers: &mut HashMap<TrackId, TextureHandle>,
    index: usize,
) {
    let Some(player) = scene.state.players.get(index) else {
        return;
    };
    let id = player.id;
    let telemetry = scene
        .telemetry
        .players
        .iter()
        .find(|(p, _)| *p == id)
        .map(|(_, t)| *t)
        .unwrap_or_default();
    let Some(mut pv) = view::player_view(scene.state, id, telemetry.position_secs, scene.time)
    else {
        return;
    };
    pv.bit_perfect = telemetry.bit_perfect;
    let rect = ui.available_rect_before_wrap();
    ui.painter().rect_filled(rect, 0.0, theme::COLUMN_BG);
    ui.painter().rect_stroke(
        rect,
        0.0,
        Stroke::new(1.0, theme::NEUTRAL_800),
        StrokeKind::Inside,
    );
    let mut ui = ui.new_child(
        UiBuilder::new()
            .max_rect(rect.shrink(1.0))
            .layout(Layout::top_down(Align::Min)),
    );
    let ui = &mut ui;
    ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
    egui::Frame::new()
        .fill(theme::SURFACE)
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 8.0);
            header(ui, scene, id, index, &pv);
            top_block(ui, scene, covers, id, &pv, &telemetry);
            time_row(ui, &pv);
            wave(ui, scene, view_state, id, &pv);
        });
    tabs(ui, scene, view_state, id, player.playlist);
    let footer_top = ui.max_rect().bottom() - FOOTER_HEIGHT;
    let table_rect = Rect::from_min_max(
        ui.available_rect_before_wrap().min,
        pos2(ui.max_rect().right(), footer_top),
    );
    let mut table_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(table_rect)
            .layout(Layout::top_down(Align::Min)),
    );
    table::track_table(&mut table_ui, scene, view_state, id, player.playlist);
    let footer_rect = Rect::from_min_max(pos2(ui.max_rect().left(), footer_top), ui.max_rect().max);
    let mut footer_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(footer_rect)
            .layout(Layout::left_to_right(Align::Center)),
    );
    footer(&mut footer_ui, scene, id, player.playlist);
}

fn small_caps(text: &str, color: Color32) -> RichText {
    RichText::new(text.to_uppercase())
        .font(font(10.0))
        .color(color)
}

fn outlined(ui: &mut Ui, text: &str, border: Color32, color: Color32) {
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_uppercase(), font(10.0), color);
    let size = vec2(galley.size().x + 12.0, 18.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter()
        .rect_stroke(rect, 0.0, Stroke::new(1.0, border), StrokeKind::Inside);
    ui.painter().galley(
        pos2(rect.left() + 6.0, rect.center().y - galley.size().y / 2.0),
        galley,
        color,
    );
}

fn header(ui: &mut Ui, scene: &Scene<'_>, id: PlayerId, index: usize, pv: &PlayerView) {
    let t = scene.i18n;
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), 20.0),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
            outlined(
                ui,
                &t.tr_args("player-label", &[("n", (index + 1).into())]),
                theme::NEUTRAL_700,
                theme::NEUTRAL_200,
            );
            let (dot, label) = match pv.status {
                PlayerStatus::OnAir => (theme::ON_AIR_TEXT, t.tr("status-on-air")),
                PlayerStatus::Paused => (theme::AMBER, t.tr("status-paused")),
                PlayerStatus::Stopped => (theme::NEUTRAL_700, t.tr("status-stopped")),
            };
            let (r, _) = ui.allocate_exact_size(vec2(7.0, 7.0), Sense::hover());
            ui.painter().rect_filled(r, 0.0, dot);
            ui.add(
                egui::Label::new(small_caps(&label, theme::NEUTRAL_400))
                    .selectable(false)
                    .truncate(),
            );
            if pv.fading {
                let fade_stopping = scene.state.player(id).is_ok_and(|p| p.fade_stopping());
                let key = if fade_stopping {
                    "badge-fading"
                } else {
                    "badge-mixing"
                };
                outlined(ui, &t.tr(key), theme::ACCENT, theme::ACCENT_300);
            }
            if pv.stop_after_current {
                outlined(
                    ui,
                    &t.tr("badge-stop-after"),
                    theme::AMBER,
                    theme::AMBER_TEXT,
                );
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
                let cue_style = if pv.cueing {
                    TileStyle {
                        fill: theme::CUE_BG,
                        border: theme::CUE,
                        content: theme::CUE,
                        ..TileStyle::plain()
                    }
                } else {
                    TileStyle {
                        content: theme::NEUTRAL_400,
                        ..TileStyle::plain()
                    }
                };
                let cue_label = t.tr("cue");
                if widgets::tile(
                    ui,
                    vec2(50.0, 20.0),
                    &t.tr("tip-cue"),
                    fp_model::availability(scene.state, id).cue,
                    cue_style,
                    |p, r, c| {
                        p.text(
                            r.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("{} {cue_label}", icon::HEADPHONES),
                            font_semibold(9.0),
                            c,
                        );
                    },
                )
                .clicked()
                {
                    scene.ctl.send(Command::ToggleCue(id));
                }
                let single_text = t.tr("mode-single");
                let cont_text = t.tr("mode-cont");
                let single_tip = t.tr("tip-single");
                let cont_tip = t.tr("tip-cont");
                let modes = [PlayMode::Single, PlayMode::Continuous];
                let selected = usize::from(pv.mode == PlayMode::Continuous);
                if let Some(mode) = widgets::segmented(
                    ui,
                    egui::Id::new(("play-mode", id)),
                    &[
                        widgets::Segment {
                            text: &single_text,
                            label: &single_tip,
                        },
                        widgets::Segment {
                            text: &cont_text,
                            label: &cont_tip,
                        },
                    ],
                    selected,
                )
                .and_then(|i| modes.get(i).copied())
                {
                    scene.ctl.send(Command::SetMode(id, mode));
                }
                let bp = t.tr("badge-bp");
                let (tip, content) = if pv.bit_perfect {
                    (t.tr("tip-bp-on"), theme::ACCENT)
                } else {
                    (t.tr("tip-bp-off"), theme::NEUTRAL_600)
                };
                widgets::tile(
                    ui,
                    vec2(24.0, 20.0),
                    &tip,
                    false,
                    TileStyle {
                        content,
                        ..TileStyle::plain()
                    },
                    |p, r, c| {
                        p.text(
                            r.center(),
                            egui::Align2::CENTER_CENTER,
                            &bp,
                            font_semibold(9.0),
                            c,
                        );
                    },
                );
            });
        },
    );
}

fn cover(
    ui: &mut Ui,
    scene: &Scene<'_>,
    covers: &mut HashMap<TrackId, TextureHandle>,
    track: Option<TrackId>,
) {
    let (rect, _) = ui.allocate_exact_size(vec2(64.0, 64.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, 0.0, theme::NEUTRAL_900);
    let texture = track.and_then(|track| {
        if let Some(t) = covers.get(&track) {
            return Some(t.clone());
        }
        let media = scene.media.get(track)?;
        let png = media.cover_png.as_ref()?;
        let decoded = image::load_from_memory_with_format(png, image::ImageFormat::Png).ok()?;
        let rgba = decoded.to_rgba8();
        let size = [rgba.width() as usize, rgba.height() as usize];
        let image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
        let handle = ui.ctx().load_texture(
            format!("cover-{}", track.0),
            image,
            egui::TextureOptions::LINEAR,
        );
        covers.insert(track, handle.clone());
        Some(handle)
    });
    match texture {
        Some(t) => {
            ui.painter().image(
                t.id(),
                rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }
        None => widgets::glyph(
            ui.painter(),
            rect,
            icon::VINYL_RECORD,
            28.0,
            theme::TEXT.gamma_multiply(0.35),
            false,
        ),
    }
}

/// Width of the meter and fader column, and its gap to the left part.
const METER_COLUMN_WIDTH: f32 = widgets::METER_WIDTH + 6.0 + widgets::FADER_WIDTH;
const METER_COLUMN_GAP: f32 = 10.0;
/// Vertical gap between the info row and the transport.
const ROW_GAP: f32 = 8.0;
/// The countdown shrinks to fit down to this share of its size (38 → 18 px:
/// an hour-long time in a player at its minimum width).
const COUNTDOWN_MIN_SCALE: f32 = 18.0 / 38.0;

/// The info row and the transport on the left, the meter and fader column
/// on the right spanning both (feedback spec §3.2).
fn top_block(
    ui: &mut Ui,
    scene: &Scene<'_>,
    covers: &mut HashMap<TrackId, TextureHandle>,
    id: PlayerId,
    pv: &PlayerView,
    telemetry: &fp_engine::engine::PlayerTelemetry,
) {
    let height = PLAY_SIZE * 2.0 + ROW_GAP;
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    let split = (rect.right() - METER_COLUMN_WIDTH).max(rect.left());
    let left = Rect::from_min_max(
        rect.min,
        pos2((split - METER_COLUMN_GAP).max(rect.left()), rect.bottom()),
    );
    let right = Rect::from_min_max(pos2(split, rect.top()), rect.max);
    let mut left_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(left)
            .layout(Layout::top_down(Align::Min)),
    );
    left_ui.spacing_mut().item_spacing = vec2(6.0, ROW_GAP);
    info_row(&mut left_ui, scene, covers, id, pv, telemetry);
    transport(&mut left_ui, scene, id, pv);
    let mut right_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(right)
            .layout(Layout::left_to_right(Align::Min)),
    );
    meter_column(&mut right_ui, scene, id, telemetry, height);
}

/// The level meter and the volume fader, `height` tall.
fn meter_column(
    ui: &mut Ui,
    scene: &Scene<'_>,
    id: PlayerId,
    telemetry: &fp_engine::engine::PlayerTelemetry,
    height: f32,
) {
    let t = scene.i18n;
    ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
    let meter = &scene.state.config.meter;
    let name = widgets::loudness_line(&telemetry.meter, meter)
        .map(|(value, _)| t.tr_args("meter-loudness", &[("value", value.into())]))
        .unwrap_or_else(|| t.tr("meter-label"));
    let labels = widgets::MeterLabels {
        meter: name,
        max: t.tr_args(
            "meter-max",
            &[("value", widgets::max_readout(telemetry.meter.max_db).into())],
        ),
        max_tip: t.tr("tip-meter-max"),
    };
    if widgets::vu(ui, height, &telemetry.meter, meter, &labels) {
        scene.ctl.reset_meter_max(id);
    }
    let volume = scene.state.player(id).map_or(1.0, |p| p.volume);
    let db = match view::volume_db(volume) {
        Some(db) => t.tr_args("unit-db", &[("value", format!("{db:.1}").into())]),
        None => t.tr("volume-silent"),
    };
    let tip = t.tr_args("tip-volume", &[("db", db.into())]);
    if let Some(pos) = widgets::fader(ui, height, view::fader_from_gain(volume), &tip) {
        scene
            .ctl
            .send(Command::SetVolume(id, view::gain_from_fader(pos)));
    }
}

/// `elapsed / total` under the transport, right-aligned (mockup C).
fn time_row(ui: &mut Ui, pv: &PlayerView) {
    ui.add_space(-4.0);
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), 14.0),
        Layout::right_to_left(Align::Center),
        |ui| {
            let text = format!(
                "{} / {}",
                format::clock(pv.elapsed),
                format::clock(pv.total.unwrap_or(0.0))
            );
            widgets::tabular_label(ui, &text, &font(12.0), theme::NEUTRAL_400);
        },
    );
}

fn info_row(
    ui: &mut Ui,
    scene: &Scene<'_>,
    covers: &mut HashMap<TrackId, TextureHandle>,
    id: PlayerId,
    pv: &PlayerView,
    telemetry: &fp_engine::engine::PlayerTelemetry,
) {
    let t = scene.i18n;
    let current_track = scene
        .state
        .player(id)
        .ok()
        .and_then(|p| p.current)
        .and_then(|e| scene.state.playlists.entry(e))
        .map(|e| e.track);
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), 64.0),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing = vec2(10.0, 0.0);
            cover(ui, scene, covers, current_track);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = vec2(6.0, 2.0);
                let title = pv.title.clone().unwrap_or_else(|| t.tr("no-track"));
                ui.add(
                    egui::Label::new(
                        RichText::new(title)
                            .font(font_medium(15.0))
                            .color(theme::TEXT),
                    )
                    .selectable(false)
                    .truncate(),
                );
                let artist = match (&pv.title, &pv.artist) {
                    (_, Some(artist)) => artist.clone(),
                    (Some(_), None) => t.tr("unknown-artist"),
                    (None, None) => t.tr("placeholder-none"),
                };
                ui.add(
                    egui::Label::new(
                        RichText::new(artist)
                            .font(font(13.0))
                            .color(theme::NEUTRAL_400),
                    )
                    .selectable(false)
                    .truncate(),
                );
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
                    let (r, _) = ui.allocate_exact_size(vec2(6.0, 6.0), Sense::hover());
                    ui.painter().rect_filled(r, 0.0, theme::NEXT_TEXT);
                    if pv.cueing {
                        let cue = telemetry
                            .cue_position_secs
                            .map(format::clock)
                            .unwrap_or_default();
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            widgets::tabular_label(ui, &cue, &font(11.0), theme::CUE);
                            ui.add(
                                egui::Label::new(
                                    RichText::new(icon::HEADPHONES)
                                        .font(font(11.0))
                                        .color(theme::CUE),
                                )
                                .selectable(false),
                            );
                            next_line(ui, scene, pv);
                        });
                    } else {
                        next_line(ui, scene, pv);
                    }
                });
            });
        },
    );
}

fn next_line(ui: &mut Ui, scene: &Scene<'_>, pv: &PlayerView) {
    let text = pv
        .next_line
        .clone()
        .unwrap_or_else(|| scene.i18n.tr("placeholder-none"));
    ui.add(
        egui::Label::new(
            RichText::new(text)
                .font(font(11.0))
                .color(theme::NEUTRAL_300),
        )
        .selectable(false)
        .truncate(),
    );
}

/// Draws a transport button's content in its rectangle and colour.
type PaintFn = Box<dyn Fn(&egui::Painter, Rect, Color32)>;

/// One button of the transport grid.
struct GridButton {
    tip: String,
    enabled: bool,
    style: TileStyle,
    paint: PaintFn,
    command: Command,
}

impl GridButton {
    fn new(tip: String, enabled: bool, style: TileStyle, paint: PaintFn, command: Command) -> Self {
        Self {
            tip,
            enabled,
            style,
            paint,
            command,
        }
    }
}

fn transport(ui: &mut Ui, scene: &Scene<'_>, id: PlayerId, pv: &PlayerView) {
    let t = scene.i18n;
    let blink = (scene.time * 2.0).floor() as i64 % 2 == 0;
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), PLAY_SIZE),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing = vec2(GAP, GAP);
            let available = fp_model::availability(scene.state, id);
            let playing = pv.status == PlayerStatus::OnAir;
            let play_label = if playing {
                t.tr("tip-next")
            } else {
                t.tr("tip-play")
            };
            let caption = if pv.fading {
                t.tr("badge-fading").to_uppercase()
            } else if playing {
                t.tr("next-caption")
            } else {
                t.tr("play-caption")
            };
            let play_style = TileStyle {
                fill: Color32::TRANSPARENT,
                border: theme::PLAY_BORDER,
                border_width: 2.0,
                content: theme::PLAY,
                hover_fill: theme::PLAY_HOVER_BG,
                hover_content: theme::PLAY_HOVER,
                active_fill: theme::PLAY_ACTIVE_BG,
            };
            let glyph = if playing {
                egui_phosphor::fill::FAST_FORWARD
            } else {
                egui_phosphor::fill::PLAY
            };
            if widgets::tile(
                ui,
                vec2(PLAY_SIZE, PLAY_SIZE),
                &play_label,
                available.play,
                play_style,
                |p, r, c| {
                    widgets::glyph(p, r.translate(vec2(0.0, -6.0)), glyph, 28.0, c, true);
                    p.text(
                        pos2(r.center().x, r.bottom() - 12.0),
                        egui::Align2::CENTER_CENTER,
                        &caption,
                        font_semibold(8.0),
                        c,
                    );
                },
            )
            .clicked()
            {
                scene.ctl.send(Command::Play(id));
            }
            let small = vec2((PLAY_SIZE - GAP) / 2.0, (PLAY_SIZE - GAP) / 2.0);
            let paused = pv.status == PlayerStatus::Paused;
            let pause_style = if paused {
                TileStyle {
                    fill: if blink {
                        theme::AMBER_BG
                    } else {
                        Color32::TRANSPARENT
                    },
                    content: if blink {
                        theme::AMBER_TEXT
                    } else {
                        theme::AMBER_DIM
                    },
                    ..TileStyle::plain()
                }
            } else {
                TileStyle::plain()
            };
            let fade_stopping = scene.state.player(id).is_ok_and(|p| p.fade_stopping());
            let fade_style = if fade_stopping {
                TileStyle {
                    fill: theme::NEUTRAL_800,
                    content: theme::TEXT,
                    ..TileStyle::plain()
                }
            } else {
                TileStyle::plain()
            };
            let sa_style = if pv.stop_after_current {
                TileStyle {
                    fill: theme::AMBER_BG,
                    border: theme::AMBER,
                    content: theme::AMBER_TEXT,
                    ..TileStyle::plain()
                }
            } else {
                TileStyle::plain()
            };
            let sa_tip = if pv.mode == PlayMode::Single {
                t.tr("tip-stop-after-single")
            } else {
                t.tr("tip-stop-after")
            };
            let glyph = |g: &'static str| -> PaintFn {
                Box::new(move |p, r, c| widgets::glyph(p, r, g, 13.0, c, true))
            };
            let drawn = |f: fn(Rect, Color32) -> Vec<egui::Shape>, w: f32, h: f32| -> PaintFn {
                Box::new(move |p, r, c| {
                    p.extend(f(Rect::from_center_size(r.center(), vec2(w, h)), c))
                })
            };
            // Two rows of three (feedback spec §3.2): Previous and Restart
            // first, then Stop and Pause over Fade stop and Stop after.
            let rows: [[GridButton; 3]; 2] = [
                [
                    GridButton::new(
                        t.tr("tip-previous"),
                        available.previous,
                        TileStyle::plain(),
                        drawn(icons::previous, 16.0, 12.0),
                        Command::Previous(id),
                    ),
                    GridButton::new(
                        t.tr("tip-stop"),
                        available.stop,
                        TileStyle::plain(),
                        glyph(egui_phosphor::fill::STOP),
                        Command::Stop(id),
                    ),
                    GridButton::new(
                        t.tr("tip-pause"),
                        available.pause,
                        pause_style,
                        glyph(egui_phosphor::fill::PAUSE),
                        Command::Pause(id),
                    ),
                ],
                [
                    GridButton::new(
                        t.tr("tip-restart"),
                        available.restart,
                        TileStyle::plain(),
                        drawn(icons::restart, 14.0, 12.0),
                        Command::Restart(id),
                    ),
                    GridButton::new(
                        t.tr("tip-fade-stop"),
                        available.fade_stop,
                        fade_style,
                        drawn(icons::fade_stop, 16.0, 12.0),
                        Command::FadeStop(id),
                    ),
                    GridButton::new(
                        sa_tip,
                        available.stop_after_current,
                        sa_style,
                        drawn(icons::stop_after, 18.0, 13.0),
                        Command::ToggleStopAfterCurrent(id),
                    ),
                ],
            ];
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = vec2(GAP, GAP);
                for row in rows {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = vec2(GAP, GAP);
                        for b in row {
                            if widgets::tile(ui, small, &b.tip, b.enabled, b.style, |p, r, c| {
                                (b.paint)(p, r, c);
                            })
                            .clicked()
                            {
                                scene.ctl.send(b.command);
                            }
                        }
                    });
                }
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.spacing_mut().item_spacing = vec2(8.0, 0.0);
                let (main, tenths) = format::countdown(pv.remaining);
                let colour = if pv.end_warning && (pv.remaining * 2.0).floor() as i64 % 2 == 1 {
                    theme::ON_AIR_TEXT
                } else {
                    theme::TEXT
                };
                // Full size when it fits; smaller in a narrow player or for an
                // hour-long time, so it never runs into the buttons.
                let natural = widgets::tabular_size(ui.painter(), &main, &font_medium(38.0)).x
                    + widgets::tabular_size(ui.painter(), &tenths, &font_medium(17.0)).x;
                let scale =
                    (ui.available_width() / natural.max(1.0)).clamp(COUNTDOWN_MIN_SCALE, 1.0);
                let big = font_medium(38.0 * scale);
                let tenths_font = font_medium(17.0 * scale);
                let main_size = widgets::tabular_size(ui.painter(), &main, &big);
                let tenths_size = widgets::tabular_size(ui.painter(), &tenths, &tenths_font);
                let (rect, response) = ui.allocate_exact_size(
                    vec2(main_size.x + tenths_size.x, main_size.y),
                    Sense::hover(),
                );
                let spoken = format!("{main}{tenths}");
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::WidgetType::Label, true, spoken.clone())
                });
                widgets::paint_tabular(ui.painter(), rect.left_top(), &main, &big, colour);
                // The tenths sit on the bottom of the big digits, smaller and dimmed.
                widgets::paint_tabular(
                    ui.painter(),
                    pos2(
                        rect.left() + main_size.x,
                        rect.bottom() - tenths_size.y - 4.0 * scale,
                    ),
                    &tenths,
                    &tenths_font,
                    theme::NEUTRAL_500,
                );
            });
        },
    );
}

fn wave(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState, id: PlayerId, pv: &PlayerView) {
    let t = scene.i18n;
    let track = scene
        .state
        .player(id)
        .ok()
        .and_then(|p| p.current)
        .and_then(|e| scene.state.playlists.entry(e))
        .map(|e| e.track);
    let media = track.and_then(|t| scene.media.get(t));
    let mix_label = t.tr("mix-marker");
    let label = t.tr("tip-waveform");
    let input = widgets::WaveInput {
        id: egui::Id::new(("waveform", id)),
        media: media.as_ref(),
        total: pv.total,
        markers: pv.markers,
        colors: theme::wave_colors(&scene.state.config.ui.wave_color),
        mix_active: pv.mode == PlayMode::Continuous,
        mix_label: &mix_label,
        accessible_label: &label,
        view: None,
    };
    let (response, seek) = widgets::waveform(ui, WAVE_HEIGHT, &input);
    if let Some(secs) = seek {
        scene.ctl.send(Command::Seek(id, secs));
    }
    if let (Some(track), Some(total)) = (track, pv.total.filter(|t| *t > 0.0)) {
        edit_markers(ui, scene, view_state, id, track, total, pv, &response);
    }
    let rect = response.rect;
    let painter = ui.painter_at(rect);
    if let Some(left) = pv.intro {
        let fill = if pv.intro_blink == Some(true) {
            theme::INTRO_BADGE_BLINK
        } else {
            theme::INTRO_BADGE_BG
        };
        widgets::time_badge(
            &painter,
            rect.left_top() + vec2(4.0, 4.0),
            false,
            &t.tr("wave-intro"),
            &format!("{left:.1}"),
            (
                fill.gamma_multiply(0.95),
                theme::INTRO,
                theme::INTRO_BADGE_TEXT,
            ),
        );
    }
    if let Some(left) = pv.outro {
        widgets::time_badge(
            &painter,
            rect.right_top() + vec2(-4.0, 4.0),
            true,
            &t.tr("wave-outro"),
            &format!("{left:.1}"),
            (
                theme::OUTRO_BADGE_BG.gamma_multiply(0.9),
                theme::OUTRO_LINE,
                theme::OUTRO_BADGE_TEXT,
            ),
        );
    }
}

fn tabs(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    id: PlayerId,
    shown: PlaylistId,
) {
    let player = scene.state.player(id).ok();
    let current_pl = player
        .and_then(|p| p.current)
        .and_then(|e| scene.state.playlists.find(e))
        .map(|(pl, _)| pl);
    let next_pl = player
        .and_then(|p| p.next)
        .and_then(|e| scene.state.playlists.find(e))
        .map(|(pl, _)| pl);
    let (strip, _) =
        ui.allocate_exact_size(vec2(ui.available_width(), TABS_HEIGHT), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(strip, 0.0, theme::NEUTRAL_900);
    let count = scene.state.playlists.len().max(1);
    let width = strip.width() / count as f32;
    for (i, pl) in scene.state.playlists.iter().enumerate() {
        let rect = Rect::from_min_size(
            pos2(strip.left() + i as f32 * width, strip.top()),
            vec2(width, TABS_HEIGHT),
        );
        let response = ui
            .interact(rect, ui.id().with(("tab", id.0, pl.id.0)), Sense::click())
            .on_hover_text(&pl.name);
        let name = pl.name.clone();
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Button, true, name.clone())
        });
        let dropping = response.dnd_hover_payload::<DragEntry>().is_some();
        if let Some(payload) = response.dnd_release_payload::<DragEntry>() {
            scene.ctl.send(Command::MoveEntry {
                entry: payload.entry,
                to: pl.id,
                index: pl.entries.len(),
            });
            view_state.drop = None;
        }
        if response.clicked() && pl.id != shown {
            scene.ctl.send(Command::ShowPlaylist(id, pl.id));
        }
        let on = pl.id == shown;
        let painter = ui.painter();
        let fill = if dropping {
            theme::ACCENT_900
        } else if on {
            theme::COLUMN_BG
        } else if response.hovered() {
            theme::NEUTRAL_800
        } else {
            Color32::TRANSPARENT
        };
        painter.rect_filled(rect, 0.0, fill);
        if on || dropping {
            painter.rect_filled(
                Rect::from_min_size(rect.left_top(), vec2(rect.width(), 2.0)),
                0.0,
                theme::ACCENT,
            );
        }
        painter.rect_filled(
            Rect::from_min_size(
                pos2(rect.right() - 1.0, rect.top()),
                vec2(1.0, rect.height()),
            ),
            0.0,
            theme::NEUTRAL_800,
        );
        let color = if on || dropping {
            theme::TEXT
        } else {
            theme::NEUTRAL_500
        };
        let galley = painter.layout(
            pl.name.clone(),
            font(11.0),
            color,
            (rect.width() - 24.0).max(8.0),
        );
        let dot = if current_pl == Some(pl.id) {
            Some(theme::ON_AIR_TEXT)
        } else if next_pl == Some(pl.id) {
            Some(theme::NEXT_TEXT)
        } else {
            None
        };
        let text_w = galley.size().x.min(rect.width() - 24.0);
        let start = rect.center().x - (text_w + 12.0) / 2.0;
        if let Some(dot) = dot {
            painter.rect_filled(
                Rect::from_min_size(pos2(start, rect.center().y - 3.0), vec2(6.0, 6.0)),
                0.0,
                dot,
            );
        }
        let text_rect = Rect::from_min_size(
            pos2(start + 12.0, rect.center().y - galley.size().y / 2.0),
            vec2(text_w, galley.size().y),
        );
        ui.painter()
            .with_clip_rect(text_rect)
            .galley(text_rect.min, galley, color);
    }
    let bottom = Rect::from_min_size(
        pos2(strip.left(), strip.bottom() - 1.0),
        vec2(strip.width(), 1.0),
    );
    ui.painter().rect_filled(bottom, 0.0, theme::NEUTRAL_800);
}

fn footer(ui: &mut Ui, scene: &Scene<'_>, id: PlayerId, playlist: PlaylistId) {
    let t = scene.i18n;
    let rect = ui.max_rect();
    ui.painter().rect_filled(rect, 0.0, theme::NEUTRAL_900);
    ui.painter().rect_filled(
        Rect::from_min_size(rect.left_top(), vec2(rect.width(), 1.0)),
        0.0,
        theme::NEUTRAL_800,
    );
    ui.add_space(10.0);
    ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
    let add = t.tr("footer-add");
    let add_width = ui
        .painter()
        .layout_no_wrap(add.clone(), font(10.0), theme::NEUTRAL_300)
        .size()
        .x
        + 26.0;
    if widgets::tile(
        ui,
        vec2(add_width, 18.0),
        &t.tr("tip-add"),
        true,
        TileStyle::plain(),
        |p, r, c| {
            p.text(
                r.center(),
                egui::Align2::CENTER_CENTER,
                format!("{} {add}", icon::PLUS),
                font(10.0),
                c,
            );
        },
    )
    .clicked()
    {
        let len = scene
            .state
            .playlists
            .get(playlist)
            .map_or(0, |p| p.entries.len());
        scene.pick_files(playlist, len);
    }
    let len = scene
        .state
        .playlists
        .get(playlist)
        .map_or(0, |p| p.entries.len());
    ui.add(
        egui::Label::new(
            RichText::new(t.tr_args("footer-count", &[("count", len.into())]))
                .font(font(11.0))
                .color(theme::NEUTRAL_500),
        )
        .selectable(false)
        .truncate(),
    );
    let positions: Vec<(PlayerId, f64)> = scene
        .telemetry
        .players
        .iter()
        .filter_map(|(p, t)| t.position_secs.map(|s| (*p, s)))
        .collect();
    let times = view::playlist_times(scene.state, id, playlist, &positions);
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        ui.add_space(10.0);
        ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
        widgets::tabular_label(
            ui,
            &format::clock(times.total),
            &font(11.0),
            theme::NEUTRAL_300,
        );
        ui.add(
            egui::Label::new(
                RichText::new(t.tr("footer-total"))
                    .font(font(11.0))
                    .color(theme::NEUTRAL_500),
            )
            .selectable(false),
        );
        let (r, _) = ui.allocate_exact_size(vec2(1.0, 14.0), Sense::hover());
        ui.painter().rect_filled(r, 0.0, theme::NEUTRAL_800);
        widgets::tabular_label(
            ui,
            &format!("-{}", format::clock(times.remaining)),
            &font_medium(13.0),
            theme::TEXT,
        );
    });
}

/// Marker editing on the waveform (Phase 2 spec P2.8): a context menu that
/// places a marker where it was opened, and Alt-drag on marker handles.
#[allow(clippy::too_many_arguments)]
fn edit_markers(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    id: PlayerId,
    track: TrackId,
    total: f64,
    pv: &PlayerView,
    response: &egui::Response,
) {
    let t = scene.i18n;
    let inner = response.rect.shrink(1.0);
    let secs_at = |x: f32| f64::from(((x - inner.left()) / inner.width()).clamp(0.0, 1.0)) * total;
    let x_of = |f: f32| inner.left() + f * inner.width();
    let m = pv.markers;
    let handles = [
        (MarkerKind::CueIn, m.cue_in),
        (MarkerKind::IntroEnd, m.intro_end),
        (MarkerKind::OutroStart, m.outro_start),
        (MarkerKind::SegueStart, m.segue_start),
        (MarkerKind::CueOut, m.cue_out),
    ];
    if response.secondary_clicked()
        && let Some(p) = response.interact_pointer_pos()
    {
        view_state.wave_menu.insert(id, secs_at(p.x));
    }
    let at = view_state.wave_menu.get(&id).copied();
    let open = response.context_menu(|ui| {
        ui.set_min_width(220.0);
        let item = |ui: &mut Ui, key: &str| {
            let text = t.tr(key);
            let r = ui.button(&text);
            r.widget_info(|| {
                egui::WidgetInfo::labeled(egui::WidgetType::Button, true, text.clone())
            });
            r.clicked()
        };
        if let Some(secs) = at {
            ui.add(
                egui::Label::new(
                    RichText::new(format::clock(secs))
                        .font(font(11.0))
                        .color(theme::NEUTRAL_400),
                )
                .selectable(false),
            );
            ui.separator();
            for (kind, key) in [
                (MarkerKind::CueIn, "wave-set-cue-in"),
                (MarkerKind::IntroEnd, "wave-set-intro"),
                (MarkerKind::OutroStart, "wave-set-outro"),
                (MarkerKind::SegueStart, "wave-set-mix"),
                (MarkerKind::CueOut, "wave-set-cue-out"),
            ] {
                if item(ui, key) {
                    scene.ctl.send(Command::SetMarker {
                        track,
                        kind,
                        secs: Some(secs),
                    });
                    ui.close();
                }
            }
            ui.separator();
        }
        if item(ui, "wave-reset") {
            scene.ctl.send(Command::ResetMarkers { track });
            ui.close();
        }
    });
    // Once the menu is closed, the point it was opened at is forgotten.
    if open.is_none() && !response.secondary_clicked() {
        view_state.wave_menu.remove(&id);
    }
    let alt = ui.input(|i| i.modifiers.alt);
    // The drag starts once the pointer has moved; pick the marker under
    // the point where the button went down.
    if alt
        && response.drag_started_by(egui::PointerButton::Primary)
        && let Some(p) = ui.input(|i| i.pointer.press_origin())
    {
        // The nearest marker within reach of the pointer.
        let nearest = handles
            .iter()
            .filter_map(|(kind, f)| f.map(|f| (*kind, (x_of(f) - p.x).abs())))
            .filter(|(_, d)| *d <= 8.0)
            .min_by(|a, b| a.1.total_cmp(&b.1));
        view_state.marker_drag = nearest.map(|(kind, _)| (id, kind, track));
    }
    // A drag belongs to the track it started on: if the player moved on,
    // it is dropped.
    if view_state
        .marker_drag
        .is_some_and(|(p, _, t)| p == id && t != track)
    {
        view_state.marker_drag = None;
    }
    let dragging = view_state
        .marker_drag
        .filter(|(p, _, _)| *p == id)
        .map(|(_, k, _)| k);
    let painter = ui.painter_at(response.rect);
    if alt || dragging.is_some() {
        for (kind, f) in handles {
            if let Some(f) = f {
                let x = x_of(f);
                let active = dragging == Some(kind);
                let color = if active {
                    theme::TEXT
                } else {
                    theme::ACCENT_300
                };
                painter.add(egui::Shape::convex_polygon(
                    vec![
                        pos2(x - 5.0, inner.top()),
                        pos2(x + 5.0, inner.top()),
                        pos2(x, inner.top() + 7.0),
                    ],
                    color,
                    Stroke::NONE,
                ));
            }
        }
    }
    if let Some(kind) = dragging
        && let Some(p) = ui.ctx().pointer_latest_pos()
    {
        let x = p.x.clamp(inner.left(), inner.right());
        painter.rect_filled(
            Rect::from_min_size(pos2(x, inner.top()), vec2(1.0, inner.height())),
            0.0,
            theme::TEXT,
        );
        painter.text(
            pos2(x + 4.0, inner.bottom() - 4.0),
            egui::Align2::LEFT_BOTTOM,
            format::clock(secs_at(x)),
            font(10.0),
            theme::TEXT,
        );
        if response.drag_stopped() || !ui.input(|i| i.pointer.primary_down()) {
            scene.ctl.send(Command::SetMarker {
                track,
                kind,
                secs: Some(secs_at(x)),
            });
            view_state.marker_drag = None;
        }
    }
}
