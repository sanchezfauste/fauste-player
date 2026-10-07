//! One player column (spec §8.3): header, info row, transport, waveform,
//! playlist tabs, track table and footer.

use std::collections::HashMap;

use egui::{
    Align, Color32, Layout, Rect, RichText, Sense, Stroke, StrokeKind, TextureHandle, Ui,
    UiBuilder, pos2, vec2,
};
use egui_phosphor::regular as icon;
use fp_model::{Command, MarkerKind, PlayMode, PlayerId, PlaylistId, TrackId};

use super::app::{DragEntry, FollowScroll, Scene, ViewState};
use super::format;
use super::glyphs::{self, TransportAction};
use super::tab_strip;
use super::table;
use super::theme;
use super::view::{self, PlayerStatus, PlayerView};
use super::wave_view::{WaveKey, WaveView, WaveZoom, min_span};
use super::widgets::{self, TileStyle, font, font_medium, font_semibold};

const PLAY_SIZE: f32 = 64.0;
/// One wheel notch zooms the waveform in to this share of its span.
const WAVE_ZOOM_STEP: f64 = 0.8;
/// One sideways notch pans the waveform by this share of its width.
const WAVE_PAN_STEP: f32 = 0.1;
/// Smooth-scrolling wheels and trackpads report points: this many make a
/// notch.
const WAVE_POINTS_PER_NOTCH: f32 = 50.0;
/// A wheel that reports pages: one page is this many notches.
const WAVE_NOTCHES_PER_PAGE: f32 = 3.0;
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
    pv.dsd = telemetry.dsd;
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
            wave(ui, scene, view_state, id, &pv);
            time_row(ui, scene, &pv);
        });
    scroll_to_next_once(scene, view_state, id);
    follow_current(scene, view_state, id, player.playlist);
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
    footer(&mut footer_ui, scene, view_state, id, player.playlist);
}

/// Follows the player's current entry in its table (feedback spec F18):
/// when it changes, and once the operator has left the table and tabs alone
/// for `ui.follow_current_grace_secs` (0 never follows), the tab of its
/// playlist is shown and its row scrolled to the top.
fn follow_current(scene: &Scene<'_>, view_state: &mut ViewState, id: PlayerId, shown: PlaylistId) {
    let current = scene.state.player(id).ok().and_then(|p| p.current);
    if view_state.followed.get(&id) != Some(&current) {
        view_state.followed.insert(id, current);
        match current {
            Some(entry) => {
                view_state.follow_pending.insert(id, entry);
            }
            None => {
                view_state.follow_pending.remove(&id);
            }
        }
    }
    let Some(entry) = view_state.follow_pending.get(&id).copied() else {
        return;
    };
    let grace = scene.state.config.ui.follow_current_grace_secs;
    if grace <= 0.0 {
        view_state.follow_pending.remove(&id);
        return;
    }
    let idle = view_state
        .table_touched
        .get(&id)
        .is_none_or(|t| scene.time - t >= grace);
    if !idle {
        return;
    }
    view_state.follow_pending.remove(&id);
    let Some((playlist, _)) = scene.state.playlists.find(entry) else {
        return;
    };
    if playlist != shown {
        scene.ctl.send(Command::ShowPlaylist(id, playlist));
    }
    view_state.follow_scroll.insert(
        id,
        FollowScroll {
            entry,
            align: Align::TOP,
            animated: true,
        },
    );
}

/// O7: the first time a player's column is drawn (the start of the
/// application, or a player added later), its table scrolls so that the
/// next entry is in the middle. Never again: after that the operator, and
/// the following of the current entry, own the scroll position.
fn scroll_to_next_once(scene: &Scene<'_>, view_state: &mut ViewState, id: PlayerId) {
    if !view_state.startup_scrolled.insert(id) {
        return;
    }
    if let Some(entry) = view::start_scroll_target(scene.state, id) {
        view_state.follow_scroll.insert(
            id,
            FollowScroll {
                entry,
                align: Align::Center,
                animated: false,
            },
        );
    }
}

fn small_caps(text: &str, color: Color32) -> RichText {
    RichText::new(text.to_uppercase())
        .font(font(10.0))
        .color(color)
}

fn outlined(ui: &mut Ui, text: &str, border: Color32, color: Color32) {
    outlined_with_tip(ui, text, None, border, color);
}

/// As `outlined`, with a sentence that is the badge's tooltip and
/// accessible name.
fn outlined_with_tip(ui: &mut Ui, text: &str, tip: Option<&str>, border: Color32, color: Color32) {
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_uppercase(), font(10.0), color);
    let size = vec2(galley.size().x + 12.0, 18.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter()
        .rect_stroke(rect, 0.0, Stroke::new(1.0, border), StrokeKind::Inside);
    ui.painter().galley(
        pos2(rect.left() + 6.0, rect.center().y - galley.size().y / 2.0),
        galley,
        color,
    );
    if let Some(tip) = tip {
        let owned = tip.to_owned();
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Label, true, owned.clone())
        });
        response.on_hover_text(tip);
    }
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
            if let Some(notice) = pv.entry_notice {
                let (text, tip) = match notice {
                    fp_model::EntryNotice::Repeats => ("badge-entry-repeat", "tip-entry-repeat"),
                    fp_model::EntryNotice::StopsAfter => ("badge-entry-stop", "tip-entry-stop"),
                };
                outlined_with_tip(
                    ui,
                    &t.tr(text),
                    Some(&t.tr(tip)),
                    theme::AMBER,
                    theme::AMBER_TEXT,
                );
            }
            if pv.dsd_holds_others {
                outlined_with_tip(
                    ui,
                    &t.tr("badge-dsd-hold"),
                    Some(&t.tr("tip-dsd-hold")),
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
                // Without a Cue output apart from Main the CUE is dimmed
                // (spec §4.6); its tooltip says how to get one.
                let cue_tip = if pv.cueing || scene.state.config.outputs.player_has_cue(id) {
                    t.tr("tip-cue")
                } else {
                    t.tr("tip-cue-no-output")
                };
                if widgets::tile(
                    ui,
                    vec2(50.0, 20.0),
                    &cue_tip,
                    fp_model::availability(scene.state, id).cue,
                    cue_style,
                    |p, r, c| {
                        p.text(
                            r.center(),
                            egui::Align2::CENTER_CENTER,
                            format!("{} {cue_label}", glyphs::glyph_text(TransportAction::Cue)),
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
                let (bp, tip, content) = match fp_model::bp_badge(pv.bit_perfect, pv.dsd) {
                    fp_model::BpBadge::Off => {
                        (t.tr("badge-bp"), t.tr("tip-bp-off"), theme::NEUTRAL_600)
                    }
                    fp_model::BpBadge::Pcm => (t.tr("badge-bp"), t.tr("tip-bp-on"), theme::ACCENT),
                    fp_model::BpBadge::Dsd => {
                        (t.tr("badge-dsd"), t.tr("tip-dsd-on"), theme::ACCENT)
                    }
                };
                widgets::tile(
                    ui,
                    vec2(28.0, 20.0),
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
    let tip = if telemetry.dsd {
        t.tr("tip-volume-dsd")
    } else {
        t.tr_args("tip-volume", &[("db", db.into())])
    };
    if let Some(pos) = widgets::fader(ui, height, view::fader_from_gain(volume), &tip) {
        scene
            .ctl
            .send(Command::SetVolume(id, view::gain_from_fader(pos)));
    }
}

/// `elapsed / total` under the waveform, right-aligned, close to it.
fn time_row(ui: &mut Ui, scene: &Scene<'_>, pv: &PlayerView) {
    ui.add_space(-4.0);
    let text = view::time_text(pv.elapsed, pv.total, &scene.i18n.tr("placeholder-none"));
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), 14.0),
        Layout::right_to_left(Align::Center),
        |ui| {
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
    let current_track = view::shown_entry(scene.state, id)
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
                                    RichText::new(glyphs::glyph_text(TransportAction::Cue))
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
    let blink = widgets::blink(scene.time);
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
            let play_action = if playing {
                TransportAction::Next
            } else {
                TransportAction::Play
            };
            if widgets::tile(
                ui,
                vec2(PLAY_SIZE, PLAY_SIZE),
                &play_label,
                available.play,
                play_style,
                |p, r, c| {
                    glyphs::paint(p, r.translate(vec2(0.0, -6.0)), play_action, 28.0, c);
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
                widgets::paused_style(blink)
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
            let sa_tip = if pv.mode == PlayMode::Single && !available.stop_after_current {
                t.tr("tip-stop-after-single")
            } else {
                t.tr("tip-stop-after")
            };
            let button = |action: TransportAction| -> PaintFn {
                Box::new(move |p, r, c| glyphs::paint(p, r, action, 13.0, c))
            };
            // Two rows of three (feedback spec §3.2): Previous and Restart
            // first, then Stop and Pause over Fade stop and Stop after.
            let rows: [[GridButton; 3]; 2] = [
                [
                    GridButton::new(
                        t.tr("tip-previous"),
                        available.previous,
                        TileStyle::plain(),
                        button(TransportAction::Previous),
                        Command::Previous(id),
                    ),
                    GridButton::new(
                        t.tr("tip-stop"),
                        available.stop,
                        TileStyle::plain(),
                        button(TransportAction::Stop),
                        Command::Stop(id),
                    ),
                    GridButton::new(
                        t.tr("tip-pause"),
                        available.pause,
                        pause_style,
                        button(TransportAction::Pause),
                        Command::Pause(id),
                    ),
                ],
                [
                    GridButton::new(
                        t.tr("tip-restart"),
                        available.restart,
                        TileStyle::plain(),
                        button(TransportAction::Restart),
                        Command::Restart(id),
                    ),
                    GridButton::new(
                        t.tr("tip-fade-stop"),
                        available.fade_stop,
                        fade_style,
                        button(TransportAction::FadeStop),
                        Command::FadeStop(id),
                    ),
                    GridButton::new(
                        sa_tip,
                        available.stop_after_current,
                        sa_style,
                        button(TransportAction::StopAfter),
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
    // The current track, or the next one waiting while stopped.
    let current = view::shown_entry(scene.state, id);
    let track = current
        .and_then(|e| scene.state.playlists.entry(e))
        .map(|e| e.track);
    let media = track.and_then(|t| scene.media.get(t));
    let total = pv.total.filter(|t| *t > 0.0);
    let key = WaveKey::Player(id);
    // A zoom belongs to the entry it was made on.
    let mut zoom = view_state
        .wave_zoom
        .get(key, current)
        .filter(|_| total.is_some());
    let wave_id = key.id();
    // While zoomed, follow the playhead once the operator's last move is
    // older than the grace (feedback spec F17), but never under a held drag.
    let dragging = widgets::pan_dragging(ui, wave_id)
        || view_state.marker_drag.is_some_and(|(k, _, _)| k == key);
    if let (Some(z), Some(total), Some(f)) = (zoom.as_mut(), total, pv.markers.position) {
        let grace = scene.state.config.ui.follow_current_grace_secs;
        if dragging {
            z.moved_at = scene.time;
        } else if grace > 0.0
            && scene.time - z.moved_at >= grace
            && pv.status != PlayerStatus::Stopped
        {
            // A stopped player's position is pinned (its cue-in or its pending
            // start, rule 3a): following it would undo a zoom made to prepare
            // the next track.
            z.view = z.view.follow(f64::from(f) * total, total);
        }
    }
    let view = zoom.map(|z| z.view);
    // Where the Full view button goes while zoomed: no seek starts under it.
    let wave_rect = Rect::from_min_size(ui.cursor().min, vec2(ui.available_width(), WAVE_HEIGHT));
    let full_view_text = t.tr("wave-full-view");
    let full_view_width = ui
        .painter()
        .layout_no_wrap(full_view_text.clone(), font(10.0), theme::TEXT)
        .size()
        .x
        + 12.0;
    let button_rect = Rect::from_min_size(
        pos2(
            wave_rect.right() - 4.0 - full_view_width,
            wave_rect.top() + 4.0,
        ),
        vec2(full_view_width, 18.0),
    );
    let full_view_button = zoom.map(|_| button_rect);
    let mix_label = t.tr("mix-marker");
    let label = t.tr("tip-waveform");
    let input = widgets::WaveInput {
        id: wave_id,
        media: media.as_ref(),
        total: pv.total,
        markers: pv.markers,
        colors: theme::wave_colors(&scene.state.config.ui.wave_color),
        mix_active: pv.mode == PlayMode::Continuous,
        mix_label: &mix_label,
        accessible_label: &label,
        view,
        shield: full_view_button,
        seekable: true,
    };
    let output = widgets::waveform(ui, WAVE_HEIGHT, &input);
    let (response, seek, pan_dx) = (output.response, output.seek, output.pan_dx);
    if let Some(secs) = seek {
        scene.ctl.send(Command::Seek(id, secs));
    }
    let rect = response.rect;
    if let (Some(track), Some(total)) = (track, total) {
        let shown = view.unwrap_or_else(|| WaveView::full(total));
        edit_markers(ui, scene, view_state, key, track, shown, pv, &response);
    }
    // The wheel zooms around the pointer; Shift or a sideways wheel pans.
    if let (Some(entry), Some(total), Some(p)) = (current, total, response.hover_pos()) {
        let inner = rect.shrink(1.0);
        let bucket = media.as_ref().map_or(
            f64::from(scene.state.config.analysis.peak_bucket_ms) / 1000.0,
            |m| m.peak_bucket_secs,
        );
        let min = min_span(bucket, inner.width());
        // In notches: a line is one, points and pages are converted, so a
        // trackpad zooms in proportion instead of one step per event.
        let wheels: Vec<(egui::Vec2, bool)> = ui.input(|i| {
            i.events
                .iter()
                .filter_map(|e| match e {
                    egui::Event::MouseWheel {
                        unit,
                        delta,
                        modifiers,
                        ..
                    } if *delta != egui::Vec2::ZERO && !modifiers.command => {
                        let notches = match unit {
                            egui::MouseWheelUnit::Line => *delta,
                            egui::MouseWheelUnit::Point => *delta / WAVE_POINTS_PER_NOTCH,
                            egui::MouseWheelUnit::Page => *delta * WAVE_NOTCHES_PER_PAGE,
                        };
                        Some((notches, modifiers.shift))
                    }
                    _ => None,
                })
                .collect()
        });
        if !wheels.is_empty() {
            let mut v = view.unwrap_or_else(|| WaveView::full(total));
            for (notches, shift) in wheels {
                if shift || notches.x.abs() > notches.y.abs() {
                    let step = if notches.x.abs() > notches.y.abs() {
                        notches.x
                    } else {
                        notches.y
                    };
                    v = v.pan(step * inner.width() * WAVE_PAN_STEP, inner, total);
                } else {
                    let factor = WAVE_ZOOM_STEP.powf(f64::from(notches.y));
                    v = v.zoom_at(p.x, inner, factor, total, min);
                }
            }
            zoom = (!v.is_full(total)).then_some(WaveZoom {
                view: v,
                entry,
                moved_at: scene.time,
            });
            // The wheel was for the waveform, not for a scroll area around it.
            ui.ctx()
                .input_mut(|i| i.smooth_scroll_delta = egui::Vec2::ZERO);
            ui.ctx().request_repaint();
        }
    }
    // Dragging pans a zoomed view; without zoom a drag does nothing (O10).
    if pan_dx != 0.0
        && let (Some(z), Some(total)) = (zoom.as_mut(), total)
    {
        z.view = z.view.pan(pan_dx, rect.shrink(1.0), total);
        z.moved_at = scene.time;
    }
    let painter = ui.painter_at(rect);
    let mut badge_right = rect.right() - 4.0;
    if zoom.is_some() {
        let button = button_rect;
        let text = full_view_text;
        let mut child = ui.new_child(UiBuilder::new().max_rect(button));
        if widgets::tile(
            &mut child,
            button.size(),
            &text,
            true,
            TileStyle::plain(),
            |p, r, c| {
                p.text(
                    r.center(),
                    egui::Align2::CENTER_CENTER,
                    &text,
                    font(10.0),
                    c,
                );
            },
        )
        .clicked()
        {
            zoom = None;
        }
        badge_right = button.left() - 4.0;
    }
    view_state.wave_zoom.set(key, zoom);
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
            pos2(badge_right, rect.top() + 4.0),
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
    ui.painter().rect_filled(strip, 0.0, theme::NEUTRAL_900);
    let count = scene.state.playlists.len();
    let l = tab_strip::layout(strip.width(), count);
    let left_gap = if l.overflow {
        tab_strip::ARROW_WIDTH
    } else {
        0.0
    };
    let view = Rect::from_min_size(
        pos2(strip.left() + left_gap, strip.top()),
        vec2(l.view_width, TABS_HEIGHT),
    );
    let st = view_state.tab_scroll.entry(id).or_default();
    let key = (shown, count, l.view_width.to_bits());
    if st.revealed != Some(key) {
        if let Some(index) = scene.state.playlists.iter().position(|pl| pl.id == shown) {
            st.offset = tab_strip::reveal(st.offset, index, &l);
        }
        st.revealed = Some(key);
    }
    if l.overflow && ui.rect_contains_pointer(strip) {
        let delta = ui.input(|i| i.smooth_scroll_delta);
        if delta != egui::Vec2::ZERO {
            st.offset -= delta.x + delta.y;
            // The wheel was for the tabs, not for the table below.
            ui.ctx()
                .input_mut(|i| i.smooth_scroll_delta = egui::Vec2::ZERO);
        }
    }
    st.offset = if l.overflow {
        tab_strip::clamp_offset(st.offset, l.content_width, l.view_width)
    } else {
        0.0
    };
    if l.overflow {
        let max = (l.content_width - l.view_width).max(0.0);
        for (direction, at, glyph, key) in [
            (-1, strip.left(), icon::CARET_LEFT, "tip-tabs-scroll-left"),
            (
                1,
                strip.right() - tab_strip::ARROW_WIDTH,
                icon::CARET_RIGHT,
                "tip-tabs-scroll-right",
            ),
        ] {
            let enabled = if direction < 0 {
                st.offset > 0.0
            } else {
                st.offset < max
            };
            let area = Rect::from_min_size(
                pos2(at, strip.top()),
                vec2(tab_strip::ARROW_WIDTH, TABS_HEIGHT),
            );
            let label = scene.i18n.tr(key);
            let clicked = ui
                .scope_builder(UiBuilder::new().max_rect(area), |ui| {
                    widgets::tile(
                        ui,
                        area.size(),
                        &label,
                        enabled,
                        TileStyle {
                            border_width: 0.0,
                            ..TileStyle::plain()
                        },
                        |p, r, c| {
                            p.text(
                                r.center(),
                                egui::Align2::CENTER_CENTER,
                                glyph,
                                font(12.0),
                                c,
                            );
                        },
                    )
                })
                .inner
                .clicked();
            if clicked {
                st.offset = tab_strip::step(st.offset, direction, &l);
            }
        }
    }
    let offset = st.offset;
    for (i, pl) in scene.state.playlists.iter().enumerate() {
        let rect = Rect::from_min_size(
            pos2(view.left() - offset + i as f32 * l.tab_width, strip.top()),
            vec2(l.tab_width, TABS_HEIGHT),
        );
        if !rect.intersects(view) {
            continue;
        }
        let response = ui
            .interact(
                rect.intersect(view),
                ui.id().with(("tab", id.0, pl.id.0)),
                Sense::click(),
            )
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
        if response.clicked() {
            view_state.table_touched.insert(id, scene.time);
        }
        if response.clicked() && pl.id != shown {
            scene.ctl.send(Command::ShowPlaylist(id, pl.id));
        }
        let on = pl.id == shown;
        let painter = ui.painter().with_clip_rect(view);
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
        let mut job = egui::text::LayoutJob::simple_singleline(pl.name.clone(), font(11.0), color);
        job.wrap = egui::text::TextWrapping {
            max_width: (l.tab_width - 24.0).max(8.0),
            max_rows: 1,
            break_anywhere: true,
            overflow_character: Some('…'),
        };
        let galley = painter.layout_job(job);
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
        painter
            .with_clip_rect(text_rect.intersect(view))
            .galley(text_rect.min, galley, color);
    }
    let bottom = Rect::from_min_size(
        pos2(strip.left(), strip.bottom() - 1.0),
        vec2(strip.width(), 1.0),
    );
    ui.painter().rect_filled(bottom, 0.0, theme::NEUTRAL_800);
}

fn footer(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    id: PlayerId,
    playlist: PlaylistId,
) {
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
    // O22: clear the played marks of this playlist, after a question.
    let reset = t.tr("footer-reset-played");
    let reset_width = ui
        .painter()
        .layout_no_wrap(reset.clone(), font(10.0), theme::NEUTRAL_300)
        .size()
        .x
        + 26.0;
    if widgets::tile(
        ui,
        vec2(reset_width, 18.0),
        &t.tr("tip-reset-played"),
        fp_model::can_reset_played(scene.state, playlist),
        TileStyle::plain(),
        |p, r, c| {
            p.text(
                r.center(),
                egui::Align2::CENTER_CENTER,
                format!("{} {reset}", icon::ARROW_COUNTER_CLOCKWISE),
                font(10.0),
                c,
            );
        },
    )
    .clicked()
    {
        view_state.confirm_reset = Some(playlist);
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
    key: WaveKey,
    track: TrackId,
    view: WaveView,
    pv: &PlayerView,
    response: &egui::Response,
) {
    let t = scene.i18n;
    let inner = response.rect.shrink(1.0);
    let total = pv.total.unwrap_or(0.0);
    let secs_at = |x: f32| view.secs_at(x, inner);
    let x_of = |f: f32| view.x_of(f64::from(f) * total, inner);
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
        view_state.wave_menu.insert(key, secs_at(p.x));
    }
    let at = view_state.wave_menu.get(&key).copied();
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
        view_state.wave_menu.remove(&key);
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
        view_state.marker_drag = nearest.map(|(kind, _)| (key, kind, track));
    }
    // A drag belongs to the track it started on: if the player moved on,
    // it is dropped.
    if view_state
        .marker_drag
        .is_some_and(|(k, _, t)| k == key && t != track)
    {
        view_state.marker_drag = None;
    }
    let dragging = view_state
        .marker_drag
        .filter(|(k, _, _)| *k == key)
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
        let drag_time = format::clock(secs_at(x));
        let drag_font = font(10.0);
        let drag_h = widgets::tabular_size(&painter, &drag_time, &drag_font).y;
        widgets::paint_tabular(
            &painter,
            pos2(x + 4.0, inner.bottom() - 4.0 - drag_h),
            &drag_time,
            &drag_font,
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
