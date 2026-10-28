//! One player column (spec §8.3): header, info row, transport, waveform,
//! playlist tabs, track table and footer.

use std::collections::HashMap;

use egui::{
    Align, Color32, Layout, Rect, RichText, Sense, Stroke, StrokeKind, TextureHandle, Ui,
    UiBuilder, pos2, vec2,
};
use egui_phosphor::regular as icon;
use fp_model::layout::Panel;
use fp_model::{Command, PlayMode, PlayerId, PlaylistId, TrackId};

use super::app::{DragEntry, FollowScroll, Scene, ViewState};
use super::format;
use super::glyphs::{self, TransportAction};
use super::header_layout;
use super::tab_strip;
use super::table;
use super::theme;
use super::view::{self, PlayerStatus, PlayerView};
use super::wave_panel::{self, WaveBadges, WavePanelInput};
use super::wave_view::WaveKey;
use super::widgets::{self, TileStyle, font, font_medium, font_semibold};

const PLAY_SIZE: f32 = 64.0;
const GAP: f32 = 6.0;
const WAVE_HEIGHT: f32 = 56.0;
const TABS_HEIGHT: f32 = 30.0;
/// Width of the "+" (new playlist) button at the right end of the tabs.
const ADD_TAB_WIDTH: f32 = 30.0;
/// Width of the new-playlist popup.
const NEW_PLAYLIST_POPUP_WIDTH: f32 = 220.0;
const FOOTER_HEIGHT: f32 = 24.0;

/// A player's desktop column. Without `fader` its fader only shows the
/// level (a phone's scrolling columns, where a scroll must not move it).
pub(crate) fn column(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    covers: &mut HashMap<TrackId, TextureHandle>,
    index: usize,
    fader: bool,
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
            header(ui, scene, view_state, id, index, &pv);
            top_block(ui, scene, covers, id, &pv, &telemetry, fader);
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

/// A phone's compact panel (mobile spec §6): the player block without the
/// waveform and the playlist; a bar shows the progress.
pub(crate) fn compact(
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
    egui::Frame::new()
        .fill(theme::SURFACE)
        .stroke(Stroke::new(1.0, theme::NEUTRAL_800))
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 8.0);
            header(ui, scene, view_state, id, index, &pv);
            // A finger scrolling the list must not move a level on air: the
            // fader is adjusted in the open player.
            top_block(ui, scene, covers, id, &pv, &telemetry, false);
            progress_bar(ui, &pv);
            time_row(ui, scene, &pv);
        });
}

/// The share of the track played, as a thin bar.
fn progress_bar(ui: &mut Ui, pv: &PlayerView) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 4.0), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, theme::NEUTRAL_800);
    let share = pv
        .total
        .filter(|t| *t > 0.0)
        .map_or(0.0, |t| (pv.elapsed / t).clamp(0.0, 1.0)) as f32;
    let played = Rect::from_min_size(rect.min, vec2(rect.width() * share, rect.height()));
    ui.painter().rect_filled(played, 0.0, theme::NEUTRAL_400);
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
    // A badge is named by its sentence when it has one, else by its text.
    let name = tip.unwrap_or(text).to_owned();
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, name.clone()));
    if let Some(tip) = tip {
        response.on_hover_text(tip);
    }
}

/// The items of the header's left group, in drawing order. Their `rank` for
/// `header_layout::fit` is the order in which they collapse when the column
/// is narrow: the entry notice first, then "stop after", the fade badge
/// (the Next caption says it too), the DSD hold notice (nothing else shows
/// it), the state text (truncated before it goes) and the dot; the player's
/// number never.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LeftItem {
    Number,
    Dot,
    Status,
    Fading,
    StopAfter,
    EntryNotice,
    DsdHold,
}

impl LeftItem {
    fn rank(self) -> u8 {
        match self {
            Self::Number => 0,
            Self::Dot => 1,
            Self::Status => 2,
            Self::DsdHold => 3,
            Self::Fading => 4,
            Self::StopAfter => 5,
            Self::EntryNotice => 6,
        }
    }
}

/// The width `outlined` takes for `text`.
fn outlined_width(ui: &Ui, text: &str) -> f32 {
    ui.painter()
        .layout_no_wrap(text.to_uppercase(), font(10.0), theme::NEUTRAL_200)
        .size()
        .x
        + 12.0
}

fn header(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    id: PlayerId,
    index: usize,
    pv: &PlayerView,
) {
    const SPACING: f32 = 6.0;
    const CUE_WIDTH: f32 = 50.0;
    const BP_WIDTH: f32 = 28.0;
    const STATUS_MIN_WIDTH: f32 = 30.0;
    // A point of slack, so rounding never lets two items touch.
    const SLACK: f32 = 1.0;
    let t = scene.i18n;
    // RL16: a right click on the header's background opens the panel menu;
    // the tiles, added after it, keep their own clicks. It is no Tab stop.
    let row = Rect::from_min_size(ui.cursor().min, vec2(ui.available_width(), 20.0));
    let background = ui.interact(row, egui::Id::unique(("player-header", id.0)), Sense::CLICK);

    // What the header shows, measured before anything is drawn, so that no
    // two items overlap (the row fits them; the lowest priority collapses).
    let number = t.tr_args("player-label", &[("n", (index + 1).into())]);
    let (dot, status) = match pv.status {
        PlayerStatus::OnAir => (theme::ON_AIR_TEXT, t.tr("status-on-air")),
        PlayerStatus::Paused => (theme::AMBER, t.tr("status-paused")),
        PlayerStatus::Stopped => (theme::NEUTRAL_700, t.tr("status-stopped")),
    };
    let fade_stopping = scene.state.player(id).is_ok_and(|p| p.fade_stopping());
    let fade_text = t.tr(if fade_stopping {
        "badge-fading"
    } else {
        "badge-mixing"
    });
    let stop_after_text = t.tr("badge-stop-after");
    let notice = pv.entry_notice.map(|notice| match notice {
        fp_model::EntryNotice::Repeats => (t.tr("badge-entry-repeat"), t.tr("tip-entry-repeat")),
        fp_model::EntryNotice::StopsAfter => (t.tr("badge-entry-stop"), t.tr("tip-entry-stop")),
    });
    let dsd_hold_text = t.tr("badge-dsd-hold");
    let status_text = small_caps(&status, theme::NEUTRAL_400);
    let status_width = ui
        .painter()
        .layout_no_wrap(status.to_uppercase(), font(10.0), theme::NEUTRAL_400)
        .size()
        .x;
    let mut left: Vec<(LeftItem, f32)> = vec![
        (LeftItem::Number, outlined_width(ui, &number)),
        (LeftItem::Dot, 7.0),
        // The status text truncates down to a least width before it goes.
        (LeftItem::Status, status_width.min(STATUS_MIN_WIDTH)),
    ];
    if pv.fading {
        left.push((LeftItem::Fading, outlined_width(ui, &fade_text)));
    }
    if pv.stop_after_current {
        left.push((LeftItem::StopAfter, outlined_width(ui, &stop_after_text)));
    }
    if let Some((text, _)) = &notice {
        left.push((LeftItem::EntryNotice, outlined_width(ui, text)));
    }
    if pv.dsd_holds_others {
        left.push((LeftItem::DsdHold, outlined_width(ui, &dsd_hold_text)));
    }

    let single_text = t.tr("mode-single");
    let cont_text = t.tr("mode-cont");
    let single_tip = t.tr("tip-single");
    let cont_tip = t.tr("tip-cont");
    let segments = [
        widgets::Segment {
            text: &single_text,
            label: &single_tip,
        },
        widgets::Segment {
            text: &cont_text,
            label: &cont_tip,
        },
    ];
    // B5: the BP badge exists only where the player can be bit-perfect: its
    // Main device runs bit-perfect (the telemetry terms cover a frame of lag
    // after the device changes).
    let show_bp = fp_model::bit_perfect_player(scene.state, id) || pv.bit_perfect || pv.dsd;
    let gain = pv
        .gain
        .as_ref()
        .map(|badge| GainBadgeLayout::new(ui, t, badge));
    let mut right_widths = vec![
        super::panels::header_buttons_width(view_state),
        CUE_WIDTH,
        widgets::segmented_width(ui, &segments),
    ];
    if show_bp {
        right_widths.push(BP_WIDTH);
    }
    if let Some(gain) = &gain {
        right_widths.push(gain.width);
    }
    let right_width =
        right_widths.iter().sum::<f32>() + SPACING * (right_widths.len() as f32 - 1.0);
    let items: Vec<header_layout::Item> = left
        .iter()
        .map(|(kind, width)| header_layout::Item {
            width: *width,
            rank: kind.rank(),
        })
        .collect();
    let room = row.width() - right_width - SPACING - SLACK;
    let shown = header_layout::fit(&items, room, SPACING);
    // What the shown items leave over goes to the status text.
    let used: f32 = items
        .iter()
        .zip(&shown)
        .filter(|(_, shown)| **shown)
        .map(|(item, _)| item.width + SPACING)
        .sum::<f32>()
        - SPACING;
    let status_room = (STATUS_MIN_WIDTH + (room - used).max(0.0)).min(status_width);
    let has = |wanted: LeftItem| {
        left.iter()
            .zip(&shown)
            .any(|((kind, _), shown)| *kind == wanted && *shown)
    };

    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), 20.0),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing = vec2(SPACING, 0.0);
            outlined(ui, &number, theme::NEUTRAL_700, theme::NEUTRAL_200);
            if has(LeftItem::Dot) {
                let (r, _) = ui.allocate_exact_size(vec2(7.0, 7.0), Sense::hover());
                ui.painter().rect_filled(r, 0.0, dot);
            }
            if has(LeftItem::Status) {
                ui.allocate_ui_with_layout(
                    vec2(status_room, 20.0),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        ui.add(egui::Label::new(status_text).selectable(false).truncate());
                    },
                );
            }
            if has(LeftItem::Fading) {
                outlined(ui, &fade_text, theme::ACCENT, theme::ACCENT_300);
            }
            if has(LeftItem::StopAfter) {
                outlined(ui, &stop_after_text, theme::AMBER, theme::AMBER_TEXT);
            }
            if let Some((text, tip)) = &notice
                && has(LeftItem::EntryNotice)
            {
                outlined_with_tip(ui, text, Some(tip), theme::AMBER, theme::AMBER_TEXT);
            }
            if has(LeftItem::DsdHold) {
                outlined_with_tip(
                    ui,
                    &dsd_hold_text,
                    Some(&t.tr("tip-dsd-hold")),
                    theme::AMBER,
                    theme::AMBER_TEXT,
                );
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.spacing_mut().item_spacing = vec2(SPACING, 0.0);
                // The panel's buttons: at the right end, never collapsed.
                ui.scope(|ui| {
                    super::panels::header_buttons(ui, scene, view_state, Panel::Player(id))
                });
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
                // Without a Cue route the CUE is dimmed (spec §4.6, CA1);
                // on an on-air output its tooltip says where it goes (§5).
                let cue_tip = if fp_model::cue_on_air::cue_is_on_air(
                    scene.state,
                    fp_model::Holder::PlayerCue(id),
                ) {
                    t.tr("tip-cue-on-air")
                } else if pv.cueing || scene.state.config.outputs.player_has_cue(id) {
                    t.tr("tip-cue")
                } else {
                    t.tr("tip-cue-no-output")
                };
                if widgets::tile(
                    ui,
                    vec2(CUE_WIDTH, 20.0),
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
                let modes = [PlayMode::Single, PlayMode::Continuous];
                let selected = usize::from(pv.mode == PlayMode::Continuous);
                if let Some(mode) =
                    widgets::segmented(ui, egui::Id::unique(("play-mode", id)), &segments, selected)
                        .and_then(|i| modes.get(i).copied())
                {
                    scene.ctl.send(Command::SetMode(id, mode));
                }
                if show_bp {
                    let (bp, tip, on) = match fp_model::bp_badge(pv.bit_perfect, pv.dsd) {
                        fp_model::BpBadge::Off => (t.tr("badge-bp"), t.tr("tip-bp-off"), false),
                        fp_model::BpBadge::Pcm => (t.tr("badge-bp"), t.tr("tip-bp-on"), true),
                        fp_model::BpBadge::Dsd => (t.tr("badge-dsd"), t.tr("tip-dsd-on"), true),
                    };
                    widgets::tile(
                        ui,
                        vec2(BP_WIDTH, 20.0),
                        &tip,
                        false,
                        widgets::bp_tile_style(on),
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
                }
                // N18: the gain of the source on air, left of the BP badge.
                if let Some(gain) = &gain {
                    gain.draw(ui, t);
                }
            });
        },
    );
    super::panels::header_menu(&background, scene, view_state, Panel::Player(id));
}

/// The width of the gain badge's tooltip.
const GAIN_TIP_WIDTH: f32 = 320.0;

/// The gain badge of the header (loudness normalization spec N18), measured:
/// a tile named by its tooltip, as the BP badge, with a border in its tone.
/// Its width is that of the widest gain ("-00.0 dB"), so the header does not
/// shift when the gain changes.
struct GainBadgeLayout {
    text: String,
    /// The tooltip as plain text: the tile's accessible name.
    name: String,
    tip: Vec<view::InfoLine>,
    content: Color32,
    width: f32,
}

impl GainBadgeLayout {
    fn new(ui: &Ui, t: &crate::i18n::I18n, badge: &view::GainBadge) -> Self {
        let text = widgets::tr_message(t, &badge.text);
        let name = widgets::info_text(t, &badge.tip);
        let content = match badge.tone {
            view::BadgeTone::Normal => theme::ACCENT,
            view::BadgeTone::Amber => theme::AMBER,
        };
        let measure = |text: &str| {
            ui.painter()
                .layout_no_wrap(text.to_owned(), font_semibold(9.0), content)
                .size()
                .x
        };
        let widest = widgets::tr_message(
            t,
            &view::Message {
                key: "badge-gain",
                args: vec![("gain", "\u{2212}00.0".to_owned())],
            },
        );
        Self {
            width: measure(&text).max(measure(&widest)) + 12.0,
            text,
            name,
            tip: badge.tip.clone(),
            content,
        }
    }

    fn draw(&self, ui: &mut Ui, t: &crate::i18n::I18n) {
        let response = widgets::tile_without_tooltip(
            ui,
            vec2(self.width, 20.0),
            &self.name,
            false,
            TileStyle {
                border: self.content,
                content: self.content,
                dim_when_disabled: false,
                ..TileStyle::plain()
            },
            |p, r, c| {
                p.text(
                    r.center(),
                    egui::Align2::CENTER_CENTER,
                    &self.text,
                    font_semibold(9.0),
                    c,
                );
            },
        );
        response.on_hover_ui(|ui| {
            ui.set_width(GAIN_TIP_WIDTH.min((ui.ctx().content_rect().width() - 16.0).max(120.0)));
            super::loudness_edit::info_block(ui, t, &self.tip);
        });
    }
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

/// Width of the meter and fader column for meter `c`.
fn meter_column_width(c: &fp_model::MeterConfig) -> f32 {
    widgets::meter_width(c) + METER_FADER_GAP + widgets::FADER_WIDTH
}
/// The gap between the meter and the fader, which holds their divider with
/// room on both sides.
const METER_FADER_GAP: f32 = 10.0;
/// The divider's width.
const DIVIDER_WIDTH: f32 = 1.0;
/// Gap between the left part and the meter and fader column.
const METER_COLUMN_GAP: f32 = 6.0;
/// Vertical gap between the info row and the transport.
const ROW_GAP: f32 = 8.0;
/// The countdown shrinks to fit down to this share of its size (38 → 10.5 px:
/// a ten-hour time in a player at its minimum width, beside the widest
/// meter, EBU's, with its two rulers, and the fader with its scale).
const COUNTDOWN_MIN_SCALE: f32 = 10.5 / 38.0;

/// The info row and the transport on the left, the meter and fader column
/// on the right spanning both (feedback spec §3.2).
fn top_block(
    ui: &mut Ui,
    scene: &Scene<'_>,
    covers: &mut HashMap<TrackId, TextureHandle>,
    id: PlayerId,
    pv: &PlayerView,
    telemetry: &fp_engine::engine::PlayerTelemetry,
    fader: bool,
) {
    let height = PLAY_SIZE * 2.0 + ROW_GAP;
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
    let split = (rect.right() - meter_column_width(&scene.state.config.meter)).max(rect.left());
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
    meter_column(&mut right_ui, scene, id, telemetry, height, fader);
}

/// The level meter and the volume fader, `height` tall; the fader only
/// shows the level unless `fader`.
fn meter_column(
    ui: &mut Ui,
    scene: &Scene<'_>,
    id: PlayerId,
    telemetry: &fp_engine::engine::PlayerTelemetry,
    height: f32,
    fader: bool,
) {
    let t = scene.i18n;
    ui.spacing_mut().item_spacing = vec2(METER_FADER_GAP, 0.0);
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
    let meter_top = ui.cursor().top();
    if widgets::vu(ui, height, &telemetry.meter, meter, &labels) {
        scene.ctl.reset_meter_max(id);
    }
    // A hairline in the gap between the meter and the fader, so the two
    // read as separate instruments (itr07 J5).
    // Centred on what shows between the two: from the meter's right edge
    // (its labels end there) to the fader's knob. It stops above the
    // level readout under the fader, which may overhang the gap.
    let fader_left = ui.cursor().left();
    let meter_right = fader_left - METER_FADER_GAP;
    let divider_x = (meter_right + widgets::fader_knob_left(fader_left) - DIVIDER_WIDTH) / 2.0;
    let divider_height = (height - widgets::FADER_READOUT_HEIGHT).max(0.0);
    ui.painter().rect_filled(
        Rect::from_min_size(
            pos2(divider_x, meter_top),
            vec2(DIVIDER_WIDTH, divider_height),
        ),
        0.0,
        theme::DIVIDER,
    );
    let volume = scene.state.player(id).map_or(1.0, |p| p.volume);
    let db = match view::volume_db(volume) {
        Some(db) => t.tr_args("unit-db", &[("value", format!("{db:.1}").into())]),
        None => t.tr("volume-silent"),
    };
    // BF7: a bit-perfect output holds the volume at 100 %; the fader is
    // drawn at the top (whatever a state not caught up yet says) and
    // disabled.
    let locked = fp_model::fader_locked(scene.state, id);
    let tip = if locked {
        t.tr("tip-volume-locked")
    } else {
        t.tr_args("tip-volume", &[("db", db.into())])
    };
    let position = if locked {
        1.0
    } else {
        view::fader_from_gain(volume)
    };
    // The level in dB under the fader, on the row of the meter's loudness
    // line; a locked fader shows 0.0, dimmed.
    let readout = if locked {
        view::volume_readout(1.0)
    } else {
        view::volume_readout(volume)
    }
    .unwrap_or_else(|| t.tr("volume-readout-silent"));
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
        let moved = widgets::fader(
            ui,
            (height - widgets::FADER_READOUT_HEIGHT).max(0.0),
            position,
            &tip,
            !locked && fader,
        );
        widgets::fader_readout(ui, &readout, !locked);
        if let Some(pos) = moved {
            scene
                .ctl
                .send(Command::SetVolume(id, view::gain_from_fader(pos)));
        }
    });
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

/// The look of the big Play / Next button (itr07 J4): flat green, no
/// outline, a white icon. Idle it starts the next track; on air it is Next,
/// in the same green. Hover, pressed, disabled and keyboard focus (the ring
/// `widgets::tile` draws) are tokens in `theme`.
pub fn play_button_style() -> TileStyle {
    TileStyle {
        fill: theme::PLAY_FILL,
        border: Color32::TRANSPARENT,
        border_width: 0.0,
        content: theme::PLAY_ICON,
        hover_fill: theme::PLAY_FILL_HOVER,
        hover_border: Color32::TRANSPARENT,
        hover_content: theme::PLAY_ICON,
        active_fill: theme::PLAY_FILL_PRESSED,
        dim_when_disabled: true,
        disabled_fill: Some(theme::PLAY_FILL_DISABLED),
    }
}

/// The Play button of a paused player: it blinks with `blink`, in phase
/// with Pause (main spec rule 6), between its green and a dim one.
pub fn paused_play_style(blink: bool) -> TileStyle {
    TileStyle {
        fill: if blink {
            theme::PLAY_FILL
        } else {
            theme::PLAY_FILL_PAUSED_DIM
        },
        ..play_button_style()
    }
}

/// The part of `r` a fade countdown still covers: anchored to the right edge
/// so the band's leading edge sweeps from left to right, like a cart's.
pub fn sweep_band(r: Rect, left: f32) -> Rect {
    let width = r.width() * left.clamp(0.0, 1.0);
    Rect::from_min_max(pos2(r.right() - width, r.top()), r.max)
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
            // BR15: on air, a bit-perfect player starts the next track at
            // once, without a crossfade.
            let play_label = if !playing {
                t.tr("tip-play")
            } else if fp_model::bit_perfect_player(scene.state, id) {
                t.tr("tip-next-bit-perfect")
            } else {
                t.tr("tip-next")
            };
            let fade_stopping = scene.state.player(id).is_ok_and(|p| p.fade_stopping());
            let caption = if pv.fading {
                // As the header's badge: a fade stop fades, a crossfade mixes.
                t.tr(if fade_stopping {
                    "badge-fading"
                } else {
                    "badge-mixing"
                })
                .to_uppercase()
            } else if playing {
                t.tr("next-caption")
            } else {
                t.tr("play-caption")
            };
            // While a fade runs, the button that started it drains a line
            // from full to empty over the fade: Fade stop for a fade stop,
            // Previous for a crossfade it started, Next/Play for any other
            // crossfade or a segue overlap (the UI times it from the frame that first showed the fade; a
            // new fade (a fade stop pressed during a crossfade) restarts it,
            // on Fade stop).
            let fade_key = egui::Id::unique(("fade-started", id.0));
            let fade_left = if pv.fading {
                let (_, started) = ui.data_mut(|d| {
                    let entry =
                        d.get_temp_mut_or_insert_with(fade_key, || (pv.fade_seq, scene.time));
                    if entry.0 != pv.fade_seq {
                        *entry = (pv.fade_seq, scene.time);
                    }
                    *entry
                });
                Some(view::fade_remaining_fraction(
                    started, scene.time, pv.fade_ms,
                ))
            } else {
                ui.data_mut(|d| d.remove_temp::<(u64, f64)>(fade_key));
                None
            };
            let stop_left = fade_left.filter(|_| fade_stopping);
            // I20: the countdown sits on the button that started the fade;
            // a segue overlap, which no button started, drains on Next.
            let next_left = fade_left
                .filter(|_| !fade_stopping && pv.fade_origin != fp_model::FadeOrigin::Previous);
            let previous_left = fade_left
                .filter(|_| !fade_stopping && pv.fade_origin == fp_model::FadeOrigin::Previous);
            let mut play_style = if pv.status == PlayerStatus::Paused {
                paused_play_style(blink)
            } else {
                play_button_style()
            };
            if next_left.is_some() {
                // Next cannot be pressed while it fades, but it is the
                // button counting the fade down: keep the on-air look and
                // an undimmed icon instead of the muted disabled green.
                play_style.disabled_fill = Some(play_style.fill);
                play_style.dim_when_disabled = false;
            }
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
                    if let Some(left) = next_left {
                        // Full strength even though Next is unavailable during a
                        // fade: the countdown must stay visible.
                        p.rect_filled(sweep_band(r, left), 0.0, theme::play_sweep());
                    }
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
            let fade_tone = fade_style.content;
            let previous_tone = TileStyle::plain().content;
            let fade_button: PaintFn = Box::new(move |p, r, c| {
                if let Some(left) = stop_left {
                    p.rect_filled(sweep_band(r, left), 0.0, theme::sweep_tone(fade_tone));
                }
                glyphs::paint(p, r, TransportAction::FadeStop, 13.0, c);
            });
            let button = |action: TransportAction| -> PaintFn {
                Box::new(move |p, r, c| glyphs::paint(p, r, action, 13.0, c))
            };
            let previous_button: PaintFn = Box::new(move |p, r, c| {
                if let Some(left) = previous_left {
                    p.rect_filled(sweep_band(r, left), 0.0, theme::sweep_tone(previous_tone));
                }
                glyphs::paint(p, r, TransportAction::Previous, 13.0, c);
            });
            // Two rows of three (feedback spec §3.2): Previous and Restart
            // first, then Stop and Pause over Fade stop and Stop after.
            let rows: [[GridButton; 3]; 2] = [
                [
                    GridButton::new(
                        t.tr("tip-previous"),
                        available.previous,
                        TileStyle::plain(),
                        previous_button,
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
                        fade_button,
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
                    egui::WidgetInfo::labeled(egui::Role::Label, true, spoken.clone())
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
    // The current track, or the next one waiting while stopped.
    let entry = view::shown_entry(scene.state, id);
    let track = entry
        .and_then(|e| scene.state.playlists.entry(e))
        .map(|e| e.track);
    let media = track.and_then(|t| scene.media.get(t));
    let input = WavePanelInput {
        key: WaveKey::Player(id),
        entry,
        track,
        media: media.as_ref(),
        total: pv.total,
        markers: pv.markers,
        mix_active: view::mix_active(scene.state, id),
        // A stopped player's position is pinned (its cue-in or its pending
        // start, rule 3a): following it would undo a zoom made to prepare
        // the next track.
        follow: pv.status != PlayerStatus::Stopped,
        badges: WaveBadges {
            intro: pv.intro,
            intro_blink: pv.intro_blink,
            outro: pv.outro,
        },
        editable_markers: true,
        height: WAVE_HEIGHT,
    };
    if let Some(secs) = wave_panel::show(ui, scene, view_state, &input).seek {
        scene.ctl.send(Command::Seek(id, secs));
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
    // The tabs and their arrows share the strip with the "+" at its end.
    let tabs_area = Rect::from_min_max(
        strip.min,
        pos2(
            (strip.right() - ADD_TAB_WIDTH).max(strip.left()),
            strip.bottom(),
        ),
    );
    let l = tab_strip::layout(tabs_area.width(), count);
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
    if l.overflow && ui.rect_contains_pointer(tabs_area) {
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
                tabs_area.right() - tab_strip::ARROW_WIDTH,
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
                ui.scope_id().with(("tab", id.0, pl.id.0)),
                Sense::click(),
            )
            .on_hover_text(&pl.name);
        let name = pl.name.clone();
        response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, name.clone()));
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
    new_playlist_button(ui, scene, view_state, id, strip);
    let bottom = Rect::from_min_size(
        pos2(strip.left(), strip.bottom() - 1.0),
        vec2(strip.width(), 1.0),
    );
    ui.painter().rect_filled(bottom, 0.0, theme::NEUTRAL_800);
}

/// The "+" at the right end of the tab `strip` and its popup: a name, then
/// Create (or Enter) makes a playlist and shows it in this player (rule 29);
/// Escape or a click outside cancels.
fn new_playlist_button(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    id: PlayerId,
    strip: Rect,
) {
    let t = scene.i18n;
    let area = Rect::from_min_max(pos2(strip.right() - ADD_TAB_WIDTH, strip.top()), strip.max);
    let label = t.tr("tip-new-playlist");
    let response = ui
        .scope_builder(UiBuilder::new().max_rect(area), |ui| {
            widgets::tile(
                ui,
                area.size(),
                &label,
                true,
                TileStyle {
                    border_width: 0.0,
                    ..TileStyle::plain()
                },
                |p, r, c| {
                    p.text(
                        r.center(),
                        egui::Align2::CENTER_CENTER,
                        icon::PLUS,
                        font(12.0),
                        c,
                    );
                },
            )
        })
        .inner;
    let opening = response.clicked();
    if opening {
        view_state.new_playlist.insert(id, String::new());
    }
    let name = view_state.new_playlist.entry(id).or_default();
    let hint = t.tr("new-playlist-name");
    let create = t.tr("new-playlist-create");
    egui::Popup::from_toggle_button_response(&response)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.set_width(NEW_PLAYLIST_POPUP_WIDTH);
            let edit = ui.add(
                egui::TextEdit::singleline(name)
                    .hint_text(hint)
                    .desired_width(f32::INFINITY),
            );
            let enter = edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            // Keep the focus on the name while the popup is open, so the
            // app's shortcuts do not fire under what the operator types (and
            // Escape, which closes the popup, is not read as the app's own).
            let escape = ui.input(|i| i.key_pressed(egui::Key::Escape));
            if opening || (!escape && ui.memory(|m| m.focused().is_none())) {
                edit.request_focus();
            }
            let trimmed = name.trim().to_owned();
            let clicked =
                widgets::text_button(ui, None, &create, !trimmed.is_empty(), TileStyle::plain())
                    .clicked();
            if (enter || clicked) && !trimmed.is_empty() {
                scene.ctl.send(Command::CreatePlaylistIn {
                    name: trimmed,
                    player: id,
                });
                // Like a tab click: follow-current leaves the new playlist be.
                view_state.table_touched.insert(id, scene.time);
                name.clear();
                ui.close();
            }
        });
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
