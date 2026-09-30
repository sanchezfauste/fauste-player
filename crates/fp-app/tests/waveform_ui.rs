#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Waveform drag to seek, zoom and trimmed regions (feedback spec §3.3).

mod support;

use egui::{Event, Key, Modifiers, PointerButton, Pos2, Rect, pos2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_model::{AppState, Command, TrackAnalysis, apply};
use support::{Fake, harness, state};

const WAVE: &str = "Waveform: click to seek";

/// P1 plays a 180 s track trimmed to 10–170 s.
fn playing() -> AppState {
    let mut s = state(1, 2);
    let e = s.playlists.iter().next().unwrap().entries[0].clone();
    let analysis = TrackAnalysis {
        duration_secs: 180.0,
        cue_in: Some(10.0),
        cue_out: Some(170.0),
        ..TrackAnalysis::default()
    };
    apply(
        &mut s,
        Command::ApplyAnalysis {
            track: e.track,
            analysis: Box::new(analysis),
        },
    )
    .unwrap();
    let p = s.players[0].id;
    apply(&mut s, Command::Play(p)).unwrap();
    s
}

fn wave(h: &Harness<'_, AppUi>) -> Rect {
    h.get_by_label(WAVE).rect()
}

/// The x of `secs` in the full view.
fn x_of(wave: Rect, secs: f32) -> f32 {
    wave.left() + 1.0 + (wave.width() - 2.0) * secs / 180.0
}

fn press(h: &mut Harness<'_, AppUi>, at: Pos2, pressed: bool) {
    h.event(Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    });
}

/// Presses at `from`, moves to `to` in steps, and leaves the button down.
fn drag_to(h: &mut Harness<'_, AppUi>, from: Pos2, to: Pos2) {
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    press(h, from, true);
    h.run_steps(1);
    for step in 1..=10 {
        let t = step as f32 / 10.0;
        h.event(Event::PointerMoved(pos2(
            from.x + (to.x - from.x) * t,
            from.y + (to.y - from.y) * t,
        )));
        h.run_steps(1);
    }
}

fn seeks(fake: &Fake) -> Vec<f64> {
    fake.take_sent()
        .into_iter()
        .filter_map(|c| match c {
            Command::Seek(_, secs) => Some(secs),
            _ => None,
        })
        .collect()
}

#[test]
fn releasing_a_drag_inside_seeks_to_the_release_point() {
    let (mut h, fake) = harness(playing());
    let w = wave(&h);
    let (from, to) = (
        pos2(x_of(w, 40.0), w.center().y),
        pos2(x_of(w, 120.0), w.center().y),
    );
    drag_to(&mut h, from, to);
    assert!(seeks(&fake).is_empty(), "nothing moves while dragging");
    press(&mut h, to, false);
    h.run_steps(2);
    let s = seeks(&fake);
    assert_eq!(s.len(), 1, "{s:?}");
    assert!((s[0] - 120.0).abs() < 1.5, "{s:?}");
}

#[test]
fn releasing_a_drag_outside_the_waveform_cancels_it() {
    let (mut h, fake) = harness(playing());
    let w = wave(&h);
    let from = pos2(x_of(w, 40.0), w.center().y);
    let outside = pos2(x_of(w, 120.0), w.bottom() + 60.0);
    drag_to(&mut h, from, outside);
    press(&mut h, outside, false);
    h.run_steps(2);
    assert!(seeks(&fake).is_empty());
}

#[test]
fn escape_during_a_drag_cancels_it() {
    let (mut h, fake) = harness(playing());
    let w = wave(&h);
    let (from, to) = (
        pos2(x_of(w, 40.0), w.center().y),
        pos2(x_of(w, 120.0), w.center().y),
    );
    drag_to(&mut h, from, to);
    h.key_press(Key::Escape);
    h.run_steps(1);
    press(&mut h, to, false);
    h.run_steps(2);
    assert!(seeks(&fake).is_empty());
}

#[test]
fn a_plain_click_still_seeks_at_once() {
    let (mut h, fake) = harness(playing());
    let w = wave(&h);
    let at = pos2(x_of(w, 90.0), w.center().y);
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    press(&mut h, at, true);
    press(&mut h, at, false);
    h.run_steps(2);
    let s = seeks(&fake);
    assert_eq!(s.len(), 1);
    assert!((s[0] - 90.0).abs() < 1.5, "{s:?}");
}
