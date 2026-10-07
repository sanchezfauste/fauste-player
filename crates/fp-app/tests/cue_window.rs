#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O12: the CUE window.

mod support;

use std::sync::Arc;

use egui::{Color32, Event, Modifiers, PointerButton, Pos2, Rect, pos2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use fp_app::ui::app::AppUi;
use fp_app::ui::controller::Controller;
use fp_app::ui::theme;
use fp_model::{Command, MarkerKind, TrackId};
use support::{Fake, harness, state};

const PAUSE: &str = "Pause CUE";
const RESUME: &str = "Resume CUE";
const STOP: &str = "Stop CUE";
const LOAD_NEXT: &str = "Set as next";
const CLOSE: &str = "Close and stop CUE";
const WAVE: &str = "CUE waveform: click to seek";
const PLAYER_WAVE: &str = "Waveform: click to seek";
const FULL_VIEW: &str = "Full view";

/// P1 is cueing its next entry; every track is 180 s long.
fn cueing(players: usize) -> (Harness<'static, AppUi>, Arc<Fake>) {
    let mut s = state(players, 3);
    for t in s.library.iter_mut() {
        t.duration_secs = 180.0;
    }
    let (mut h, fake) = harness(s);
    fake.send(Command::ToggleCue(fake.player(0)));
    fake.take_sent();
    h.run_steps(2);
    (h, fake)
}

fn sent(fake: &Fake) -> Vec<Command> {
    fake.take_sent()
}

#[test]
fn there_is_no_window_without_a_cue() {
    let (h, _) = harness(state(1, 3));
    for label in [PAUSE, STOP, LOAD_NEXT, CLOSE, WAVE] {
        assert!(h.query_by_label(label).is_none(), "{label}");
    }
}

#[test]
fn a_running_cue_opens_its_window_and_stopping_it_closes_it() {
    let (mut h, fake) = cueing(1);
    for label in [PAUSE, STOP, LOAD_NEXT, CLOSE, WAVE] {
        assert!(h.query_by_label(label).is_some(), "{label}");
    }
    h.get_by_label(STOP).click();
    h.run_steps(2);
    assert_eq!(sent(&fake), vec![Command::SetCue(fake.player(0), false)]);
    assert!(h.query_by_label(STOP).is_none(), "the window closed");
}

#[test]
fn the_close_button_stops_the_cue() {
    let (mut h, fake) = cueing(1);
    h.get_by_label(CLOSE).click();
    h.run_steps(2);
    assert_eq!(sent(&fake), vec![Command::SetCue(fake.player(0), false)]);
    assert!(h.query_by_label(CLOSE).is_none());
}

#[test]
fn pause_becomes_resume() {
    let (mut h, fake) = cueing(1);
    let p = fake.player(0);
    h.get_by_label(PAUSE).click();
    h.run_steps(2);
    assert_eq!(sent(&fake), vec![Command::SetCuePaused(p, true)]);
    assert!(h.query_by_label(PAUSE).is_none());
    h.get_by_label(RESUME).click();
    h.run_steps(2);
    assert_eq!(sent(&fake), vec![Command::SetCuePaused(p, false)]);
    assert!(h.query_by_label(PAUSE).is_some());
}

#[test]
fn load_as_next_sends_the_command_and_dims_once_it_is_explicit() {
    let (mut h, fake) = cueing(1);
    let p = fake.player(0);
    h.get_by_label(LOAD_NEXT).click();
    h.run_steps(2);
    assert_eq!(sent(&fake), vec![Command::CueToNext(p)]);
    assert!(h.get_by_label(LOAD_NEXT).accesskit_node().is_disabled());
    assert!(h.query_by_label(STOP).is_some(), "the CUE keeps running");
}

#[test]
fn clicking_the_waveform_seeks_the_cue() {
    let (mut h, fake) = cueing(1);
    let p = fake.player(0);
    let wave = h.get_by_label(WAVE).rect();
    let at = pos2(
        wave.left() + 1.0 + (wave.width() - 2.0) * 0.5,
        wave.center().y,
    );
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    for pressed in [true, false] {
        h.event(Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        });
    }
    h.run_steps(2);
    let seeks: Vec<f64> = sent(&fake)
        .into_iter()
        .filter_map(|c| match c {
            Command::SeekCue(id, secs) if id == p => Some(secs),
            _ => None,
        })
        .collect();
    assert_eq!(seeks.len(), 1, "{seeks:?}");
    assert!((seeks[0] - 90.0).abs() < 2.0, "{seeks:?}");
}

#[test]
fn the_window_shows_the_position_from_the_engine() {
    let (mut h, fake) = cueing(1);
    fake.telemetry
        .store(Arc::new(fp_engine::conductor::Telemetry {
            players: vec![(
                fake.player(0),
                fp_engine::engine::PlayerTelemetry {
                    cue_position_secs: Some(65.0),
                    ..Default::default()
                },
            )],
            ..Default::default()
        }));
    h.run_steps(2);
    // The player column paints the same clock text: the window is the second.
    assert!(h.query_all_by_label("01:05").count() >= 2, "elapsed");
    assert!(h.query_by_label("-01:55").is_some(), "remaining");
}

#[test]
fn two_cues_stack_two_windows() {
    let (mut h, fake) = cueing(2);
    fake.send(Command::ToggleCue(fake.player(1)));
    h.run_steps(2);
    assert_eq!(h.get_all_by_label(STOP).count(), 2);
    assert_eq!(h.get_all_by_label(WAVE).count(), 2);
    // Stopping the second leaves the first.
    h.get_all_by_label(STOP).nth(1).unwrap().click();
    h.run_steps(2);
    assert_eq!(h.get_all_by_label(STOP).count(), 1);
}

#[test]
fn a_cue_without_a_known_length_shows_zero_and_cannot_seek() {
    let mut s = state(1, 3);
    let first = s.playlists.iter().next().unwrap().entries[0].track;
    s.library.get_mut(first).unwrap().duration_secs = 0.0;
    let (mut h, fake) = harness(s);
    fake.send(Command::ToggleCue(fake.player(0)));
    fake.take_sent();
    h.run_steps(2);
    assert!(h.query_by_label(STOP).is_some());
    assert!(h.query_by_label(PAUSE).is_some());
    // The player column paints "00:00" too: the window is the second match.
    assert!(h.get_all_by_label("00:00").count() >= 2);
    let wave = h.get_by_label(WAVE).rect();
    let at = wave.center();
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    for pressed in [true, false] {
        h.event(Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        });
    }
    h.run_steps(2);
    assert!(sent(&fake).is_empty());
}

#[test]
fn a_cue_that_ends_by_itself_closes_its_window() {
    let (mut h, fake) = cueing(1);
    let p = fake.player(0);
    let entry = fake.state.load().players[0].cue.unwrap().entry;
    let mut s = (*fake.state.load_full()).clone();
    fp_model::on_event(&mut s, fp_model::EngineEvent::CueEnded { player: p, entry });
    fake.state.store(Arc::new(s));
    h.run_steps(2);
    assert!(h.query_by_label(STOP).is_none());
}

#[test]
fn the_set_as_next_button_is_wide_enough_for_its_icon_and_text() {
    let (h, _) = cueing(1);
    let rect = h.get_by_label(LOAD_NEXT).rect();
    let text = format!(
        "{}  {LOAD_NEXT}",
        egui_phosphor::regular::ARROW_BEND_DOWN_RIGHT
    );
    let needed = h.ctx.fonts_mut(|f| {
        f.layout_no_wrap(
            text,
            fp_app::ui::widgets::font_semibold(11.0),
            egui::Color32::WHITE,
        )
        .size()
        .x
    });
    assert!(
        rect.width() >= needed + 16.0,
        "button {} px, text {needed} px",
        rect.width()
    );
}

/// The fill the tile named `label` is drawn with in the last frame: the
/// first rectangle painted at its rectangle is its background.
fn tile_fill(h: &Harness<'_, AppUi>, label: &str) -> Color32 {
    let rect = h.get_by_label(label).rect();
    h.output()
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            egui::Shape::Rect(r)
                if r.rect.expand(0.5).contains_rect(rect)
                    && rect.expand(0.5).contains_rect(r.rect) =>
            {
                Some(r.fill)
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("no tile drawn at {rect:?}"))
}

/// The fills `label` takes over 1.2 s of frames (60 steps of 20 ms).
fn fills_over_a_second(
    h: &mut Harness<'_, AppUi>,
    label: &str,
) -> std::collections::HashSet<Color32> {
    let mut fills = std::collections::HashSet::new();
    for _ in 0..60 {
        h.step();
        fills.insert(tile_fill(h, label));
    }
    fills
}

#[test]
fn a_paused_cue_blinks_its_resume_button_amber() {
    let (mut h, fake) = cueing(1);
    // Sent to the model directly: a click would leave the pointer on the
    // tile and its hover fill would hide the blink.
    fake.send(Command::SetCuePaused(fake.player(0), true));
    h.run_steps(2);
    let fills = fills_over_a_second(&mut h, RESUME);
    assert!(
        fills.contains(&theme::AMBER_BG) && fills.contains(&Color32::TRANSPARENT),
        "{fills:?}"
    );
}

#[test]
fn a_running_cue_does_not_blink() {
    let (mut h, _fake) = cueing(1);
    let fills = fills_over_a_second(&mut h, PAUSE);
    assert_eq!(fills.len(), 1, "{fills:?}");
}

#[test]
fn a_paused_cue_keeps_the_interface_repainting() {
    let (_h, fake) = cueing(1);
    fake.send(Command::SetCuePaused(fake.player(0), true));
    assert!(fp_app::ui::view::animating(&fake.state.load()));
    assert!(!fp_app::ui::view::animating(&state(1, 3)));
}

/// P1 cues its next entry, a 180 s track whose intro ends at 30 s.
fn cueing_with_intro() -> (Harness<'static, AppUi>, Arc<Fake>, TrackId) {
    let mut s = state(1, 3);
    for t in s.library.iter_mut() {
        t.duration_secs = 180.0;
    }
    let track = s.playlists.iter().next().unwrap().entries[0].track;
    fp_model::apply(
        &mut s,
        Command::SetMarker {
            track,
            kind: MarkerKind::IntroEnd,
            secs: Some(30.0),
        },
    )
    .unwrap();
    let (mut h, fake) = harness(s);
    fake.send(Command::ToggleCue(fake.player(0)));
    fake.take_sent();
    h.run_steps(2);
    (h, fake, track)
}

/// The x of `secs` on a whole-file waveform of 180 s.
fn x_of(wave: Rect, secs: f32) -> f32 {
    wave.left() + 1.0 + (wave.width() - 2.0) * secs / 180.0
}

fn wheel(h: &mut Harness<'_, AppUi>, at: Pos2, dy: f32) {
    h.event(Event::PointerMoved(at));
    h.event(Event::MouseWheel {
        unit: egui::MouseWheelUnit::Line,
        delta: egui::vec2(0.0, dy),
        phase: egui::TouchPhase::Move,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(1);
}

fn button(
    h: &mut Harness<'_, AppUi>,
    at: Pos2,
    button: PointerButton,
    pressed: bool,
    modifiers: Modifiers,
) {
    h.event(Event::PointerButton {
        pos: at,
        button,
        pressed,
        modifiers,
    });
}

fn click(h: &mut Harness<'_, AppUi>, at: Pos2) {
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    button(h, at, PointerButton::Primary, true, Modifiers::NONE);
    button(h, at, PointerButton::Primary, false, Modifiers::NONE);
    h.run_steps(2);
}

/// Holds Alt, presses at `from` and moves to `to` in steps; the button
/// stays down.
fn alt_drag_to(h: &mut Harness<'_, AppUi>, from: Pos2, to: Pos2) {
    h.event(Event::ModifiersChanged(Modifiers::ALT));
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    button(h, from, PointerButton::Primary, true, Modifiers::ALT);
    h.run_steps(1);
    for step in 1..=10 {
        let x = from.x + (to.x - from.x) * step as f32 / 10.0;
        h.event(Event::PointerMoved(pos2(x, from.y)));
        h.run_steps(1);
    }
}

fn set_markers(sent: &[Command]) -> Vec<(TrackId, MarkerKind, Option<f64>)> {
    sent.iter()
        .filter_map(|c| match c {
            Command::SetMarker { track, kind, secs } => Some((*track, *kind, *secs)),
            _ => None,
        })
        .collect()
}

fn cue_seeks(sent: &[Command], p: fp_model::PlayerId) -> Vec<f64> {
    sent.iter()
        .filter_map(|c| match c {
            Command::SeekCue(id, secs) if *id == p => Some(*secs),
            _ => None,
        })
        .collect()
}

#[test]
fn the_wheel_zooms_the_cue_waveform_and_a_click_seeks_the_cue_in_the_zoomed_view() {
    let (mut h, fake) = cueing(1);
    let p = fake.player(0);
    let w = h.get_by_label(WAVE).rect();
    let centre = pos2(x_of(w, 90.0), w.center().y);
    for _ in 0..3 {
        wheel(&mut h, centre, 1.0);
    }
    assert_eq!(
        h.get_all_by_label(FULL_VIEW).count(),
        1,
        "only the CUE's waveform zoomed"
    );
    // 0.8³ of 180 s = 92.16 s around 90 s: from 43.92 s. A click three
    // quarters across lands at 43.92 + 0.75 · 92.16 ≈ 113 s.
    click(&mut h, pos2(x_of(w, 135.0), w.center().y));
    let sent = sent(&fake);
    assert!(
        !sent.iter().any(|c| matches!(c, Command::Seek(..))),
        "the CUE seeks, not the player: {sent:?}"
    );
    let seeks = cue_seeks(&sent, p);
    assert_eq!(seeks.len(), 1, "{sent:?}");
    assert!((seeks[0] - 113.0).abs() < 2.0, "{seeks:?}");
}

#[test]
fn the_cue_full_view_button_shows_the_whole_file_again() {
    let (mut h, fake) = cueing(1);
    let w = h.get_by_label(WAVE).rect();
    wheel(&mut h, pos2(x_of(w, 90.0), w.center().y), 1.0);
    h.get_by_label(FULL_VIEW).click();
    h.run_steps(2);
    assert!(h.query_by_label(FULL_VIEW).is_none());
    sent(&fake);
    click(&mut h, pos2(x_of(w, 135.0), w.center().y));
    let seeks = cue_seeks(&sent(&fake), fake.player(0));
    assert!((seeks[0] - 135.0).abs() < 1.5, "{seeks:?}");
}

#[test]
fn the_player_and_its_cue_zoom_independently() {
    let (mut h, _) = cueing(1);
    let cue = h.get_by_label(WAVE).rect();
    wheel(&mut h, pos2(x_of(cue, 90.0), cue.center().y), 1.0);
    // The stopped player shows the same track; its right end is clear of
    // the CUE window.
    let player = h.get_by_label(PLAYER_WAVE).rect();
    wheel(&mut h, pos2(x_of(player, 170.0), player.center().y), 1.0);
    assert_eq!(h.get_all_by_label(FULL_VIEW).count(), 2);
}

#[test]
fn a_new_cue_opens_on_the_whole_file() {
    let (mut h, fake) = cueing(1);
    let w = h.get_by_label(WAVE).rect();
    wheel(&mut h, pos2(x_of(w, 90.0), w.center().y), 1.0);
    assert!(h.query_by_label(FULL_VIEW).is_some());
    h.get_by_label(STOP).click();
    h.run_steps(2);
    fake.send(Command::ToggleCue(fake.player(0)));
    h.run_steps(2);
    assert!(h.query_by_label(WAVE).is_some(), "the CUE runs again");
    assert!(h.query_by_label(FULL_VIEW).is_none(), "on the same entry");
}

#[test]
fn a_cue_moved_to_another_entry_shows_it_whole() {
    let (mut h, fake) = cueing(1);
    let w = h.get_by_label(WAVE).rect();
    wheel(&mut h, pos2(x_of(w, 90.0), w.center().y), 1.0);
    let second = fake.entries()[1];
    fake.send(Command::CueEntry(fake.player(0), second));
    h.run_steps(2);
    assert!(h.query_by_label(FULL_VIEW).is_none());
}

#[test]
fn a_cue_without_a_known_length_does_not_zoom() {
    let mut s = state(1, 3);
    let first = s.playlists.iter().next().unwrap().entries[0].track;
    s.library.get_mut(first).unwrap().duration_secs = 0.0;
    let (mut h, fake) = harness(s);
    fake.send(Command::ToggleCue(fake.player(0)));
    h.run_steps(2);
    let w = h.get_by_label(WAVE).rect();
    wheel(&mut h, w.center(), 1.0);
    assert!(h.query_by_label(FULL_VIEW).is_none());
}

#[test]
fn the_cue_waveform_menu_sets_the_intro_here() {
    let (mut h, fake, track) = cueing_with_intro();
    let w = h.get_by_label(WAVE).rect();
    let at = pos2(x_of(w, 45.0), w.center().y);
    h.event(Event::PointerMoved(at));
    button(&mut h, at, PointerButton::Secondary, true, Modifiers::NONE);
    button(&mut h, at, PointerButton::Secondary, false, Modifiers::NONE);
    h.run_steps(2);
    h.get_by_label("Set intro end here").click();
    h.run_steps(2);
    let set = set_markers(&fake.take_sent());
    assert_eq!(set.len(), 1, "{set:?}");
    let (t, kind, secs) = set[0];
    assert_eq!((t, kind), (track, MarkerKind::IntroEnd));
    assert!((secs.unwrap() - 45.0).abs() < 2.0, "{secs:?}");
}

#[test]
fn alt_dragging_the_intro_on_the_cue_waveform_moves_it() {
    let (mut h, fake, track) = cueing_with_intro();
    let w = h.get_by_label(WAVE).rect();
    let from = pos2(x_of(w, 30.0), w.center().y);
    let to = pos2(x_of(w, 60.0), w.center().y);
    alt_drag_to(&mut h, from, to);
    button(&mut h, to, PointerButton::Primary, false, Modifiers::ALT);
    h.run_steps(2);
    let sent = fake.take_sent();
    let set = set_markers(&sent);
    assert_eq!(set.len(), 1, "one command on release: {sent:?}");
    let (t, kind, secs) = set[0];
    assert_eq!((t, kind), (track, MarkerKind::IntroEnd));
    assert!((secs.unwrap() - 60.0).abs() < 2.0, "{secs:?}");
    assert!(
        !sent
            .iter()
            .any(|c| matches!(c, Command::Seek(..) | Command::SeekCue(..))),
        "a marker drag does not seek"
    );
}

#[test]
fn stopping_the_cue_mid_drag_moves_no_marker() {
    let (mut h, fake, _) = cueing_with_intro();
    let p = fake.player(0);
    let w = h.get_by_label(WAVE).rect();
    let from = pos2(x_of(w, 30.0), w.center().y);
    let to = pos2(x_of(w, 60.0), w.center().y);
    alt_drag_to(&mut h, from, to);
    fake.send(Command::SetCue(p, false));
    h.run_steps(2);
    button(&mut h, to, PointerButton::Primary, false, Modifiers::ALT);
    h.event(Event::ModifiersChanged(Modifiers::NONE));
    h.run_steps(2);
    fake.send(Command::ToggleCue(p));
    h.run_steps(2);
    assert!(h.query_by_label(WAVE).is_some(), "the CUE runs again");
    assert!(set_markers(&fake.take_sent()).is_empty());
}
