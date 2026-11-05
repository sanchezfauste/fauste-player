//! The phone's pre-listen card (track library spec §11.3, LB54–LB56): the
//! library CUE's track under the Library tab's rows, as the Playlists
//! screen's player card sits under its playlist. The waveform is the
//! players' panel (`WaveKey::Library`): a tap seeks (or, stopped, sets
//! where Play starts), a long press edits a marker, a pinch zooms, and the
//! padlock stops seeking and editing.

use egui::{Align, Layout, Rect, RichText, Stroke, Ui, UiBuilder, pos2, vec2};
use fp_engine::conductor::LibraryCueTelemetry;
use fp_model::{Command, Transport};

use super::app::{Scene, ViewState};
use super::glyphs::{self, TransportAction};
use super::player::{paused_play_style, play_button_style};
use super::view::{self, LibraryCueView};
use super::wave_panel::{self, WaveBadges, WavePanelInput};
use super::wave_view::{WaveKey, WaveSubject};
use super::widgets::{self, TileStyle, font, font_semibold};
use super::{theme, touch};

/// The share of the rows' room an upright phone gives the card, at most.
pub(crate) const CARD_SHARE: f32 = 0.5;
/// The waveform's height, between these.
const WAVE_MIN: f32 = 40.0;
const WAVE_MAX: f32 = 160.0;
const GAP: f32 = 6.0;
const PAD: f32 = 8.0;
/// The title and the artist.
const NAMES: f32 = 36.0;
const TIME_ROW: f32 = 20.0;
const COUNTDOWN: f32 = 20.0;
/// Between the buttons and the meter.
const METER_GAP: f32 = 12.0;

/// The card's height with no waveform: what the waveform leaves.
fn rest() -> f32 {
    2.0 * PAD + NAMES + TIME_ROW + touch::TARGET + 3.0 * GAP
}

/// The height an upright phone gives the card in `room` (the rows' and the
/// card's): its waveform as tall as half the room allows, within limits.
pub(crate) fn upright_height(room: f32) -> f32 {
    let wave = (room * CARD_SHARE - rest()).clamp(WAVE_MIN, WAVE_MAX);
    (rest() + wave).min(room.max(0.0))
}

/// The library CUE's telemetry while it is the card's track.
fn telemetry(scene: &Scene<'_>) -> Option<LibraryCueTelemetry> {
    scene
        .telemetry
        .library_cue
        .filter(|t| Some(t.track) == scene.state.library_cue.track)
}

/// What the card shows, or `None` when the library CUE has no track.
pub(crate) fn card_view(scene: &Scene<'_>) -> Option<LibraryCueView> {
    view::library_cue_view(scene.state, telemetry(scene).map(|t| t.position_secs))
}

/// Draws the card over `rect`.
pub(crate) fn card(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    v: &LibraryCueView,
    rect: Rect,
) {
    ui.painter().rect(
        rect,
        6.0,
        theme::NEUTRAL_900,
        Stroke::new(1.0, theme::DIVIDER),
        egui::StrokeKind::Inside,
    );
    let inner = rect.shrink(PAD);
    let mut body = ui.new_child(
        UiBuilder::new()
            .max_rect(inner)
            .layout(Layout::top_down(Align::Min)),
    );
    body.set_clip_rect(inner.intersect(ui.clip_rect()));
    let ui = &mut body;
    ui.spacing_mut().item_spacing = vec2(0.0, GAP);
    names(ui, scene, v);
    let wave_height = (inner.height() - (rest() - 2.0 * PAD)).clamp(WAVE_MIN, WAVE_MAX);
    wave(ui, scene, view_state, v, wave_height);
    // The times over the buttons on the left, the meter spanning both on
    // the right, as on a player's card.
    let bottom = Rect::from_min_size(
        ui.cursor().min,
        vec2(ui.available_width(), TIME_ROW + GAP + touch::TARGET),
    );
    ui.allocate_rect(bottom, egui::Sense::hover());
    let c = &scene.state.config.meter;
    let meter_width = if v.has_route {
        widgets::compact_meter_width(c, true)
    } else {
        0.0
    };
    let split = (bottom.right() - meter_width - METER_GAP).max(bottom.left());
    let mut left = ui.new_child(
        UiBuilder::new()
            .max_rect(Rect::from_min_max(bottom.min, pos2(split, bottom.bottom())))
            .layout(Layout::top_down(Align::Min)),
    );
    left.spacing_mut().item_spacing = vec2(0.0, GAP);
    time_row(&mut left, scene, v);
    transport_row(&mut left, scene, v);
    if v.has_route {
        let rect = Rect::from_min_max(pos2(bottom.right() - meter_width, bottom.top()), bottom.max);
        let mut meter_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(rect)
                .layout(Layout::top_down(Align::Min)),
        );
        meter(&mut meter_ui, scene, bottom.height());
    }
}

fn names(ui: &mut Ui, scene: &Scene<'_>, v: &LibraryCueView) {
    let t = scene.i18n;
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), NAMES),
        Layout::top_down(Align::Min),
        |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            ui.add(
                egui::Label::new(
                    RichText::new(v.title.clone())
                        .font(font_semibold(15.0))
                        .color(theme::TEXT),
                )
                .selectable(false)
                .truncate(),
            );
            ui.add(
                egui::Label::new(
                    RichText::new(v.artist.clone().unwrap_or_else(|| t.tr("unknown-artist")))
                        .font(font(12.0))
                        .color(theme::NEUTRAL_400),
                )
                .selectable(false)
                .truncate(),
            );
        },
    );
}

/// LB56: the players' panel on the card's track.
fn wave(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState, v: &LibraryCueView, h: f32) {
    let media = scene.media.get(v.track);
    let input = WavePanelInput {
        key: WaveKey::Library,
        subject: Some(WaveSubject::Track(v.track)),
        track: Some(v.track),
        media: media.as_ref(),
        total: v.total,
        markers: v.markers,
        // A preview is never in a Continuous sequence.
        mix_active: false,
        follow: v.transport == Transport::Playing,
        badges: WaveBadges {
            intro: v.intro,
            // The talk-over warning is for audio on air; a preview is not.
            intro_blink: None,
            outro: v.outro,
        },
        editable_markers: true,
        height: h,
        locked: touch::locked(scene, view_state),
    };
    if let Some(secs) = wave_panel::show(ui, scene, view_state, &input).seek {
        scene.ctl.send(Command::SeekLibraryCue(secs));
    }
}

/// The elapsed time and the length on the left, the time left on the right.
fn time_row(ui: &mut Ui, scene: &Scene<'_>, v: &LibraryCueView) {
    let text = view::time_text(v.elapsed, v.total, &scene.i18n.tr("placeholder-none"));
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), TIME_ROW),
        Layout::left_to_right(Align::Center),
        |ui| {
            widgets::tabular_label(ui, &text, &font(13.0), theme::NEUTRAL_300);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                widgets::countdown(ui, v.remaining.unwrap_or(f64::NAN), theme::TEXT, COUNTDOWN);
            });
        },
    );
}

/// Play, Pause and Stop at a finger's size, the meter on the right.
fn transport_row(ui: &mut Ui, scene: &Scene<'_>, v: &LibraryCueView) {
    let t = scene.i18n;
    let size = vec2(touch::TARGET, touch::TARGET);
    let paused = v.transport == Transport::Paused;
    let running = v.transport != Transport::Stopped;
    let blink = widgets::blink(scene.time);
    let row = Rect::from_min_size(ui.cursor().min, vec2(ui.available_width(), touch::TARGET));
    ui.allocate_rect(row, egui::Sense::hover());
    let mut buttons = ui.new_child(
        UiBuilder::new()
            .max_rect(row)
            .layout(Layout::left_to_right(Align::Center)),
    );
    buttons.spacing_mut().item_spacing = vec2(GAP, 0.0);
    let can_play = fp_model::command_available(scene.state, &Command::PlayLibraryCue);
    let play_style = if paused {
        paused_play_style(blink)
    } else {
        play_button_style()
    };
    let play = widgets::tile(
        &mut buttons,
        size,
        &t.tr("library-preview-play"),
        can_play,
        play_style,
        |p, r, c| glyphs::paint(p, r, TransportAction::Play, 16.0, c),
    );
    if touch::tapped(&buttons, &play) {
        // A held preview is released; a stopped one starts.
        scene.ctl.send(Command::PlayLibraryCue);
    }
    let pause_style = if paused {
        widgets::paused_style(blink)
    } else {
        TileStyle::plain()
    };
    let pause = widgets::tile(
        &mut buttons,
        size,
        &t.tr("library-preview-pause"),
        running,
        pause_style,
        |p, r, c| glyphs::paint(p, r, TransportAction::Pause, 16.0, c),
    );
    if touch::tapped(&buttons, &pause) {
        scene.ctl.send(Command::SetLibraryCuePaused(!paused));
    }
    let stop = widgets::tile(
        &mut buttons,
        size,
        &t.tr("library-preview-stop"),
        running,
        TileStyle::plain(),
        |p, r, c| glyphs::paint(p, r, TransportAction::Stop, 16.0, c),
    );
    if touch::tapped(&buttons, &stop) {
        scene.ctl.send(Command::StopLibraryCue);
    }
}

/// The library CUE's meter, `height` tall; a tap on its maximum resets it.
fn meter(ui: &mut Ui, scene: &Scene<'_>, height: f32) {
    let t = scene.i18n;
    let c = &scene.state.config.meter;
    let reading = telemetry(scene).map(|t| t.meter).unwrap_or_default();
    let name = widgets::loudness_line(&reading, c)
        .map(|(value, _)| t.tr_args("meter-loudness", &[("value", value.into())]))
        .unwrap_or_else(|| t.tr("meter-label"));
    let labels = widgets::MeterLabels {
        meter: name,
        max: t.tr_args(
            "meter-max",
            &[("value", widgets::max_readout(reading.max_db).into())],
        ),
        max_tip: t.tr("tip-meter-max"),
    };
    if widgets::vu_compact(ui, height, &reading, c, &labels, true) {
        scene.ctl.reset_library_cue_meter_max();
    }
}
