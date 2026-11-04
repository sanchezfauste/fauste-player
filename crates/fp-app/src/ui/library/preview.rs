//! The library window's preview bar (track library spec §8, LB32–LB35):
//! Play, Pause and Stop for the library CUE, the big remaining time, the
//! title, artist and category of the bar's track, its own small meter, and
//! the waveform the players use, with click-to-seek and marker editing.
//! The bar reads `AppState` and `Telemetry` and sends commands; the model
//! decides what the preview does.

use egui::{Align, Color32, Layout, Rect, RichText, Stroke, Ui, UiBuilder, pos2, vec2};
use fp_model::{Command, Transport};

use super::super::app::{Scene, ViewState};
use super::super::glyphs::{self, TransportAction};
use super::super::player::{paused_play_style, play_button_style};
use super::super::theme;
use super::super::view::{self, LibraryCueView};
use super::super::wave_panel::{self, WaveBadges, WavePanelInput};
use super::super::wave_view::{WaveKey, WaveSubject};
use super::super::widgets::{self, TileStyle, font, font_semibold};
use super::state::LibraryUi;

/// The bar's height, under the window's title bar.
pub(crate) const PREVIEW_BAR_HEIGHT: f32 = 112.0;
/// The margins inside the bar, across and down.
const PAD_X: f32 = 10.0;
const PAD_Y: f32 = 6.0;
/// The row of the buttons, the time and the track's names.
const TOP_ROW: f32 = 42.0;
/// The gap under the top row and the one under the waveform.
const ROW_GAP: f32 = 4.0;
const WAVE_GAP: f32 = 2.0;
/// The `elapsed / total` line under the waveform.
const TIME_ROW: f32 = 14.0;
/// A transport button, as the CUE window's.
const BUTTON: egui::Vec2 = vec2(40.0, 28.0);
const BUTTON_GAP: f32 = 4.0;
/// The remaining time's digits.
const COUNTDOWN_SIZE: f32 = 26.0;
/// The gap between the groups of the top row, and before the meter.
const GROUP_GAP: f32 = 12.0;
/// The share of the top row's room after the time that the notes on the
/// right may take, at most.
const NOTE_SHARE: f32 = 0.5;

/// LB34: reports the selected row (the last one selected) to the preview
/// when it changed since the last report; the model decides what the
/// preview does with it. Called after the table, every frame the window's
/// body is drawn.
pub(crate) fn follow_selection(scene: &Scene<'_>, library: &mut LibraryUi) {
    let selected = library.selected.last().copied();
    if library.preview_sent != Some(selected) {
        library.preview_sent = Some(selected);
        scene.ctl.send(Command::SelectLibraryCue(selected));
    }
}

/// Draws the bar over the whole of `ui`'s rectangle.
pub(crate) fn preview_bar(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState) {
    let rect = ui.max_rect();
    ui.painter().hline(
        rect.x_range(),
        rect.bottom() - 0.5,
        Stroke::new(1.0, theme::DIVIDER),
    );
    let inner = rect.shrink2(vec2(PAD_X, PAD_Y));
    // The engine's position and meter only while they are the bar's track.
    let telemetry = scene
        .telemetry
        .library_cue
        .filter(|t| Some(t.track) == scene.state.library_cue.track);
    let Some(v) = view::library_cue_view(scene.state, telemetry.map(|t| t.position_secs)) else {
        let mut line = ui.new_child(
            UiBuilder::new()
                .max_rect(inner)
                .layout(Layout::left_to_right(Align::Center)),
        );
        line.add(
            egui::Label::new(
                RichText::new(scene.i18n.tr("library-preview-empty"))
                    .font(font(12.0))
                    .color(theme::NEUTRAL_500),
            )
            .selectable(false)
            .truncate(),
        );
        return;
    };
    let meter_config = &scene.state.config.meter;
    // Without a route the notes take the meter's place.
    let meter_width = if v.has_route {
        widgets::compact_meter_width(meter_config, false)
    } else {
        0.0
    };
    let left = Rect::from_min_max(
        inner.min,
        pos2(
            inner.right() - meter_width - if v.has_route { GROUP_GAP } else { 0.0 },
            inner.bottom(),
        ),
    );
    let top = Rect::from_min_size(left.min, vec2(left.width(), TOP_ROW));
    let mut row = ui.new_child(
        UiBuilder::new()
            .max_rect(top)
            .layout(Layout::left_to_right(Align::Center)),
    );
    row.set_clip_rect(top.intersect(ui.clip_rect()));
    top_row(&mut row, scene, view_state, &v);
    let wave_height = (left.height() - TOP_ROW - ROW_GAP - WAVE_GAP - TIME_ROW).max(0.0);
    let wave_rect = Rect::from_min_size(
        pos2(left.left(), top.bottom() + ROW_GAP),
        vec2(left.width(), wave_height),
    );
    let mut wave_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(wave_rect)
            .layout(Layout::top_down(Align::Min)),
    );
    wave(&mut wave_ui, scene, view_state, &v, wave_height);
    let time = Rect::from_min_max(
        pos2(left.left(), wave_rect.bottom() + WAVE_GAP),
        pos2(left.right(), left.bottom()),
    );
    let mut time_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(time)
            .layout(Layout::right_to_left(Align::Min)),
    );
    let text = view::time_text(v.elapsed, v.total, &scene.i18n.tr("placeholder-none"));
    widgets::tabular_label(&mut time_ui, &text, &font(12.0), theme::NEUTRAL_300);
    if v.has_route {
        let meter = Rect::from_min_max(pos2(inner.right() - meter_width, inner.top()), inner.max);
        let mut meter_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(meter)
                .layout(Layout::top_down(Align::Min)),
        );
        let reading = telemetry.map(|t| t.meter).unwrap_or_default();
        meter_column(&mut meter_ui, scene, &reading, inner.height());
    }
}

/// Play, Pause and Stop, the remaining time, the track's names, and the
/// notes at the right (no route, a file that cannot play).
fn top_row(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState, v: &LibraryCueView) {
    ui.spacing_mut().item_spacing = vec2(BUTTON_GAP, 0.0);
    transport(ui, scene, v);
    ui.add_space(GROUP_GAP - BUTTON_GAP);
    let colour = if v.transport == Transport::Stopped {
        theme::NEUTRAL_300
    } else {
        theme::TEXT
    };
    widgets::countdown(ui, v.remaining.unwrap_or(f64::NAN), colour, COUNTDOWN_SIZE);
    // The countdown sets its own spacing after it.
    ui.add_space((GROUP_GAP - ui.spacing().item_spacing.x).max(0.0));
    // The notes from the right edge, then the names in the room left.
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        ui.spacing_mut().item_spacing = vec2(GROUP_GAP, 0.0);
        let room = ui.available_width().max(0.0);
        let notes = notes(scene, v);
        if !notes.is_empty() {
            let width = notes_width(ui, scene, &notes, v.has_route).min(room * NOTE_SHARE);
            ui.allocate_ui_with_layout(
                vec2(width, TOP_ROW),
                Layout::top_down(Align::Max).with_main_align(Align::Center),
                |ui| {
                    ui.set_width(width);
                    ui.spacing_mut().item_spacing = vec2(0.0, 1.0);
                    // Cut short, never wrapped, when the bar is narrow.
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);
                    for note in &notes {
                        note_label(ui, note, theme::AMBER_TEXT);
                    }
                    if !v.has_route {
                        // The accent, given here: the theme's text colour
                        // override would paint a link like plain text.
                        let link = ui.add(egui::Link::new(
                            RichText::new(scene.i18n.tr("library-preview-open-settings"))
                                .font(font(11.0))
                                .color(theme::ACCENT_400),
                        ));
                        if link.clicked() {
                            view_state.open_outputs = true;
                        }
                    }
                },
            );
        }
        let width = ui.available_width().max(0.0);
        ui.allocate_ui_with_layout(vec2(width, TOP_ROW), Layout::top_down(Align::Min), |ui| {
            ui.set_width(width);
            names(ui, scene, v);
        });
    });
}

/// Why the bar cannot play: no route, then a file that cannot play.
fn notes(scene: &Scene<'_>, v: &LibraryCueView) -> Vec<String> {
    let t = scene.i18n;
    let mut notes = Vec::new();
    if !v.has_route {
        notes.push(t.tr("library-preview-no-route"));
    }
    if !v.playable {
        notes.push(t.tr("library-preview-not-available"));
    }
    notes
}

/// The width the notes and the link take at their natural size.
fn notes_width(ui: &Ui, scene: &Scene<'_>, notes: &[String], has_route: bool) -> f32 {
    let measure = |text: String| {
        ui.painter()
            .layout_no_wrap(text, font(11.0), theme::TEXT)
            .size()
            .x
    };
    let mut width = notes.iter().map(|n| measure(n.clone())).fold(0.0, f32::max);
    if !has_route {
        width = width.max(measure(scene.i18n.tr("library-preview-open-settings")));
    }
    width + 2.0
}

fn note_label(ui: &mut Ui, text: &str, colour: Color32) {
    ui.add(
        egui::Label::new(RichText::new(text).font(font(11.0)).color(colour))
            .selectable(false)
            .truncate(),
    );
}

/// The title, the artist and the category, each on its own line and cut
/// short when the bar is narrow.
fn names(ui: &mut Ui, scene: &Scene<'_>, v: &LibraryCueView) {
    let t = scene.i18n;
    ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
    let line = |ui: &mut Ui, text: String, font: egui::FontId, colour: Color32| {
        ui.add(
            egui::Label::new(RichText::new(text).font(font).color(colour))
                .selectable(false)
                .truncate(),
        );
    };
    line(ui, v.title.clone(), font_semibold(13.0), theme::TEXT);
    line(
        ui,
        v.artist.clone().unwrap_or_else(|| t.tr("unknown-artist")),
        font(11.0),
        theme::NEUTRAL_400,
    );
    line(
        ui,
        v.category
            .clone()
            .unwrap_or_else(|| t.tr("library-tree-uncategorised")),
        font(11.0),
        theme::NEUTRAL_500,
    );
}

/// Play, Pause and Stop. A dimmed Play says why in its tooltip; while the
/// preview is held, Play blinks green and Pause amber, as a paused
/// player's.
fn transport(ui: &mut Ui, scene: &Scene<'_>, v: &LibraryCueView) {
    let t = scene.i18n;
    let state = scene.state;
    let paused = v.transport == Transport::Paused;
    let running = v.transport != Transport::Stopped;
    let blink = widgets::blink(scene.time);
    let play_label = t.tr("library-preview-play");
    let can_play = fp_model::command_available(state, &Command::PlayLibraryCue);
    let tip = if can_play {
        play_label.clone()
    } else if !v.has_route {
        t.tr("library-preview-no-route")
    } else if !v.playable {
        t.tr("library-preview-not-available")
    } else {
        play_label.clone()
    };
    let play_style = if paused {
        paused_play_style(blink)
    } else {
        play_button_style()
    };
    if widgets::tile_without_tooltip(ui, BUTTON, &play_label, can_play, play_style, |p, r, c| {
        glyphs::paint(p, r, TransportAction::Play, 14.0, c)
    })
    .on_hover_text(tip)
    .clicked()
    {
        scene.ctl.send(Command::PlayLibraryCue);
    }
    let pause_style = if paused {
        widgets::paused_style(blink)
    } else {
        TileStyle::plain()
    };
    if widgets::tile(
        ui,
        BUTTON,
        &t.tr("library-preview-pause"),
        running,
        pause_style,
        |p, r, c| glyphs::paint(p, r, TransportAction::Pause, 14.0, c),
    )
    .clicked()
    {
        scene.ctl.send(Command::SetLibraryCuePaused(!paused));
    }
    if widgets::tile(
        ui,
        BUTTON,
        &t.tr("library-preview-stop"),
        running,
        TileStyle::plain(),
        |p, r, c| glyphs::paint(p, r, TransportAction::Stop, 14.0, c),
    )
    .clicked()
    {
        scene.ctl.send(Command::StopLibraryCue);
    }
}

/// The players' waveform panel on the bar's track: a click seeks (or sets
/// where Play starts while stopped), the markers are edited on the track,
/// and a zoomed view follows the position while it plays.
fn wave(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    v: &LibraryCueView,
    height: f32,
) {
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
        height,
        locked: super::super::touch::locked(scene, view_state),
    };
    // The panel reports a click once, never a drag.
    if let Some(secs) = wave_panel::show(ui, scene, view_state, &input).seek {
        scene.ctl.send(Command::SeekLibraryCue(secs));
    }
}

/// The library CUE's own meter, without a ruler; a click on its maximum
/// restarts it.
fn meter_column(
    ui: &mut Ui,
    scene: &Scene<'_>,
    reading: &fp_engine::meter::MeterReading,
    height: f32,
) {
    let t = scene.i18n;
    let meter = &scene.state.config.meter;
    let name = widgets::loudness_line(reading, meter)
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
    if widgets::vu_compact(ui, height, reading, meter, &labels, false) {
        scene.ctl.reset_library_cue_meter_max();
    }
}
