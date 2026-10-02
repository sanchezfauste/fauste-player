#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Waveform click to seek, drag to pan, zoom and trimmed regions (feedback 2 spec O10).

mod support;

use egui::{Event, Modifiers, PointerButton, Pos2, Rect, pos2};
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

/// A stopped player shows its next track, which always starts at its
/// cue-in: its waveform shows times but does not seek.
#[test]
fn a_stopped_players_waveform_does_not_seek() {
    let mut s = playing();
    let p = s.players[0].id;
    apply(&mut s, Command::Stop(p)).unwrap();
    let next = s.players[0].next.unwrap();
    let track = s.playlists.entry(next).unwrap().track;
    let analysis = TrackAnalysis {
        duration_secs: 180.0,
        ..TrackAnalysis::default()
    };
    apply(
        &mut s,
        Command::ApplyAnalysis {
            track,
            analysis: Box::new(analysis),
        },
    )
    .unwrap();
    let (mut h, fake) = harness(s);
    let w = wave(&h);
    let at = pos2(w.center().x, w.center().y);
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    press(&mut h, at, true);
    press(&mut h, at, false);
    h.run_steps(2);
    drag_to(&mut h, at, pos2(at.x + 60.0, at.y));
    press(&mut h, pos2(at.x + 60.0, at.y), false);
    h.run_steps(2);
    assert!(seeks(&fake).is_empty());
}

fn wheel(h: &mut Harness<'_, AppUi>, at: Pos2, dx: f32, dy: f32, modifiers: Modifiers) {
    h.event(Event::PointerMoved(at));
    h.event(Event::MouseWheel {
        unit: egui::MouseWheelUnit::Line,
        delta: egui::vec2(dx, dy),
        phase: egui::TouchPhase::Move,
        modifiers,
    });
    h.run_steps(1);
}

fn click(h: &mut Harness<'_, AppUi>, at: Pos2) {
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    press(h, at, true);
    press(h, at, false);
    h.run_steps(2);
}

#[test]
fn the_wheel_zooms_around_the_pointer_and_clicks_seek_in_the_zoomed_view() {
    let (mut h, fake) = harness(playing());
    let w = wave(&h);
    let centre = pos2(x_of(w, 90.0), w.center().y);
    for _ in 0..3 {
        wheel(&mut h, centre, 0.0, 1.0, Modifiers::NONE);
    }
    // 0.8³ of 180 s = 92.16 s around 90 s: from 43.92 s. A click three
    // quarters across lands at 43.92 + 0.75 · 92.16 ≈ 113 s, not 135 s.
    click(&mut h, pos2(x_of(w, 135.0), w.center().y));
    let s = seeks(&fake);
    assert_eq!(s.len(), 1, "{s:?}");
    assert!((s[0] - 113.0).abs() < 2.0, "{s:?}");
}

#[test]
fn shift_wheel_pans_the_zoomed_view() {
    let (mut h, fake) = harness(playing());
    let w = wave(&h);
    let centre = pos2(x_of(w, 90.0), w.center().y);
    for _ in 0..3 {
        wheel(&mut h, centre, 0.0, 1.0, Modifiers::NONE);
    }
    wheel(&mut h, centre, 0.0, -1.0, Modifiers::SHIFT);
    click(&mut h, centre);
    let s = seeks(&fake);
    assert_eq!(s.len(), 1);
    assert!((s[0] - 90.0).abs() > 5.0, "the view moved: {s:?}");
}

#[test]
fn full_view_appears_only_while_zoomed_and_resets_it() {
    let (mut h, fake) = harness(playing());
    assert!(h.query_by_label("Full view").is_none());
    let w = wave(&h);
    let centre = pos2(x_of(w, 90.0), w.center().y);
    wheel(&mut h, centre, 0.0, 1.0, Modifiers::NONE);
    h.get_by_label("Full view").click();
    h.run_steps(2);
    assert!(h.query_by_label("Full view").is_none());
    fake.take_sent();
    click(&mut h, pos2(x_of(w, 135.0), w.center().y));
    let s = seeks(&fake);
    assert!((s[0] - 135.0).abs() < 1.5, "{s:?}");
}

#[test]
fn zooming_out_fully_returns_to_the_full_view() {
    let (mut h, _) = harness(playing());
    let w = wave(&h);
    let centre = pos2(x_of(w, 90.0), w.center().y);
    wheel(&mut h, centre, 0.0, 1.0, Modifiers::NONE);
    assert!(h.query_by_label("Full view").is_some());
    for _ in 0..3 {
        wheel(&mut h, centre, 0.0, -1.0, Modifiers::NONE);
    }
    assert!(h.query_by_label("Full view").is_none());
}

#[test]
fn a_new_track_returns_to_the_full_view() {
    let (mut h, fake) = harness(playing());
    let w = wave(&h);
    wheel(
        &mut h,
        pos2(x_of(w, 90.0), w.center().y),
        0.0,
        1.0,
        Modifiers::NONE,
    );
    assert!(h.query_by_label("Full view").is_some());
    let mut s = (*fake.state.load_full()).clone();
    let p = s.players[0].id;
    let second = s.playlists.iter().next().unwrap().entries[1].id;
    apply(&mut s, Command::SetNext(p, second)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    fake.state.store(std::sync::Arc::new(s));
    h.run_steps(2);
    assert!(h.query_by_label("Full view").is_none());
}

/// Reports `secs` as the playhead of P1.
fn at_position(fake: &Fake, secs: f64) {
    let p = fake.player(0);
    fake.telemetry
        .store(std::sync::Arc::new(fp_engine::conductor::Telemetry {
            players: vec![(
                p,
                fp_engine::engine::PlayerTelemetry {
                    position_secs: Some(secs),
                    ..Default::default()
                },
            )],
            ..Default::default()
        }));
}

#[test]
fn a_zoomed_view_follows_the_playhead_after_the_grace() {
    let mut s = playing();
    s.config.ui.follow_current_grace_secs = 0.01;
    let (mut h, fake) = harness(s);
    at_position(&fake, 20.0);
    h.run_steps(1);
    let w = wave(&h);
    for _ in 0..6 {
        wheel(
            &mut h,
            pos2(x_of(w, 20.0), w.center().y),
            0.0,
            1.0,
            Modifiers::NONE,
        );
    }
    // The playhead moves out of the zoomed stretch: the view goes with it.
    at_position(&fake, 150.0);
    h.run_steps(2);
    fake.take_sent();
    click(&mut h, pos2(x_of(w, 90.0), w.center().y));
    let s = seeks(&fake);
    assert_eq!(s.len(), 1);
    assert!(
        s[0] > 140.0 && s[0] < 180.0,
        "the view follows the playhead: {s:?}"
    );
}

#[test]
fn a_recent_pan_holds_the_view_still() {
    let (mut h, fake) = harness(playing());
    at_position(&fake, 20.0);
    h.run_steps(1);
    let w = wave(&h);
    for _ in 0..6 {
        wheel(
            &mut h,
            pos2(x_of(w, 20.0), w.center().y),
            0.0,
            1.0,
            Modifiers::NONE,
        );
    }
    at_position(&fake, 150.0);
    h.run_steps(2);
    fake.take_sent();
    click(&mut h, pos2(x_of(w, 90.0), w.center().y));
    let s = seeks(&fake);
    assert!(s[0] < 60.0, "within the 10 s grace the view stays: {s:?}");
}

#[test]
fn alt_dragging_a_marker_in_a_zoomed_view_sets_the_zoomed_time() {
    let mut s = playing();
    let track = s.library.iter().next().unwrap().id;
    apply(
        &mut s,
        Command::SetMarker {
            track,
            kind: fp_model::MarkerKind::SegueStart,
            secs: Some(100.0),
        },
    )
    .unwrap();
    let (mut h, fake) = harness(s);
    let w = wave(&h);
    // Zoom twice around 100 s: 0.64 · 180 = 115.2 s, from 100 − 0.5·115.2.
    let mix = pos2(x_of(w, 100.0), w.center().y);
    wheel(&mut h, mix, 0.0, 1.0, Modifiers::NONE);
    wheel(&mut h, mix, 0.0, 1.0, Modifiers::NONE);
    let to = pos2(mix.x + (w.width() - 2.0) * 0.25, mix.y);
    h.event(Event::ModifiersChanged(Modifiers::ALT));
    h.event(Event::PointerMoved(mix));
    h.run_steps(1);
    h.event(Event::PointerButton {
        pos: mix,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::ALT,
    });
    h.run_steps(1);
    for step in 1..=10 {
        h.event(Event::PointerMoved(pos2(
            mix.x + (to.x - mix.x) * step as f32 / 10.0,
            mix.y,
        )));
        h.run_steps(1);
    }
    h.event(Event::PointerButton {
        pos: to,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::ALT,
    });
    h.run_steps(2);
    let set: Vec<f64> = fake
        .take_sent()
        .into_iter()
        .filter_map(|c| match c {
            Command::SetMarker { secs, .. } => secs,
            _ => None,
        })
        .collect();
    // A quarter of the width is a quarter of 115.2 s: about 128.8 s.
    assert_eq!(set.len(), 1, "{set:?}");
    assert!((set[0] - 128.8).abs() < 2.0, "{set:?}");
}

#[test]
fn a_trackpad_swipe_zooms_in_proportion() {
    let (mut h, fake) = harness(playing());
    let w = wave(&h);
    let at = pos2(x_of(w, 90.0), w.center().y);
    h.event(Event::PointerMoved(at));
    for _ in 0..30 {
        h.event(Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(0.0, 2.0),
            phase: egui::TouchPhase::Move,
            modifiers: Modifiers::NONE,
        });
        h.run_steps(1);
    }
    // 60 points of scrolling is about one notch, not the deepest zoom
    // (which would put a click three quarters across near 92 s).
    fake.take_sent();
    click(&mut h, pos2(x_of(w, 135.0), w.center().y));
    let s = seeks(&fake);
    assert!(s[0] > 110.0, "{s:?}");
}

/// Zooms three notches around 90 s: the view is then 43.92 s to 136.08 s.
fn zoom_in(h: &mut Harness<'_, AppUi>, w: Rect) {
    let centre = pos2(x_of(w, 90.0), w.center().y);
    for _ in 0..3 {
        wheel(h, centre, 0.0, 1.0, Modifiers::NONE);
    }
}

#[test]
fn dragging_pans_the_zoomed_view_and_never_seeks() {
    let (mut h, fake) = harness(playing());
    let w = wave(&h);
    zoom_in(&mut h, w);
    fake.take_sent();
    // 30 s of the full view to the left: the zoomed view moves later.
    let (from, to) = (
        pos2(x_of(w, 100.0), w.center().y),
        pos2(x_of(w, 70.0), w.center().y),
    );
    drag_to(&mut h, from, to);
    press(&mut h, to, false);
    h.run_steps(2);
    assert!(seeks(&fake).is_empty(), "a drag never seeks");
    // The centre showed 90 s; the view has moved about 15 s later (egui
    // drops the first few pixels of a drag), so it now shows about 105 s.
    click(&mut h, pos2(w.center().x, w.center().y));
    let s = seeks(&fake);
    assert_eq!(s.len(), 1, "{s:?}");
    assert!(s[0] > 98.0 && s[0] < 112.0, "{s:?}");
}

#[test]
fn a_drag_without_zoom_does_nothing() {
    let (mut h, fake) = harness(playing());
    let w = wave(&h);
    drag_to(
        &mut h,
        pos2(x_of(w, 40.0), w.center().y),
        pos2(x_of(w, 120.0), w.center().y),
    );
    press(&mut h, pos2(x_of(w, 120.0), w.center().y), false);
    h.run_steps(2);
    assert!(seeks(&fake).is_empty());
    assert!(
        h.query_by_label("Full view").is_none(),
        "still the whole track"
    );
    click(&mut h, pos2(x_of(w, 90.0), w.center().y));
    let s = seeks(&fake);
    assert!((s[0] - 90.0).abs() < 1.5, "the view did not move: {s:?}");
}

#[test]
fn a_drag_released_outside_the_waveform_never_seeks() {
    let (mut h, fake) = harness(playing());
    let w = wave(&h);
    let outside = pos2(x_of(w, 120.0), w.bottom() + 60.0);
    drag_to(&mut h, pos2(x_of(w, 40.0), w.center().y), outside);
    press(&mut h, outside, false);
    h.run_steps(2);
    assert!(seeks(&fake).is_empty());
}

#[test]
fn a_click_that_moves_within_the_drag_threshold_still_seeks() {
    let (mut h, fake) = harness(playing());
    let w = wave(&h);
    let at = pos2(x_of(w, 90.0), w.center().y);
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    press(&mut h, at, true);
    let moved = pos2(at.x + 2.0, at.y);
    h.event(Event::PointerMoved(moved));
    h.run_steps(1);
    press(&mut h, moved, false);
    h.run_steps(2);
    let s = seeks(&fake);
    assert_eq!(s.len(), 1, "{s:?}");
    assert!((s[0] - 90.0).abs() < 2.0, "{s:?}");
}

#[test]
fn a_stopped_players_zoomed_waveform_still_pans() {
    let mut s = playing();
    let p = s.players[0].id;
    apply(&mut s, Command::Stop(p)).unwrap();
    let next = s.players[0].next.unwrap();
    let track = s.playlists.entry(next).unwrap().track;
    apply(
        &mut s,
        Command::ApplyAnalysis {
            track,
            analysis: Box::new(TrackAnalysis {
                duration_secs: 180.0,
                ..TrackAnalysis::default()
            }),
        },
    )
    .unwrap();
    let (mut h, fake) = harness(s);
    let w = wave(&h);
    zoom_in(&mut h, w);
    drag_to(
        &mut h,
        pos2(x_of(w, 100.0), w.center().y),
        pos2(x_of(w, 70.0), w.center().y),
    );
    press(&mut h, pos2(x_of(w, 70.0), w.center().y), false);
    h.run_steps(2);
    assert!(seeks(&fake).is_empty(), "it never seeks");
    // Panned later: Full view is still offered.
    assert!(h.query_by_label("Full view").is_some());
}

#[test]
fn dragging_from_the_full_view_button_does_not_pan() {
    let (mut h, fake) = harness(playing());
    let w = wave(&h);
    wheel(
        &mut h,
        pos2(x_of(w, 90.0), w.center().y),
        0.0,
        1.0,
        Modifiers::NONE,
    );
    let button = h.get_by_label("Full view").rect();
    fake.take_sent();
    let to = pos2(button.center().x - 40.0, button.center().y);
    drag_to(&mut h, button.center(), to);
    press(&mut h, to, false);
    h.run_steps(2);
    assert!(seeks(&fake).is_empty());
    // One notch around 90 s shows 18 s to 162 s: the centre is still 90 s.
    click(&mut h, pos2(w.center().x, w.center().y));
    let s = seeks(&fake);
    assert!((s[0] - 90.0).abs() < 1.5, "the view did not move: {s:?}");
}

#[test]
fn the_view_does_not_follow_while_a_drag_is_held() {
    let mut s = playing();
    s.config.ui.follow_current_grace_secs = 0.3;
    let (mut h, fake) = harness(s);
    at_position(&fake, 20.0);
    h.run_steps(1);
    let w = wave(&h);
    for _ in 0..6 {
        wheel(
            &mut h,
            pos2(x_of(w, 20.0), w.center().y),
            0.0,
            1.0,
            Modifiers::NONE,
        );
    }
    // The view is 14.8 s to 62 s. A small drag keeps the button held.
    let (from, to) = (
        pos2(x_of(w, 60.0), w.center().y),
        pos2(x_of(w, 66.0), w.center().y),
    );
    drag_to(&mut h, from, to);
    at_position(&fake, 150.0);
    // Held for 0.8 s, well past the grace: without the hold the view would
    // have followed the playhead to 150 s.
    h.run_steps(40);
    press(&mut h, to, false);
    h.run_steps(1);
    fake.take_sent();
    click(&mut h, pos2(w.center().x, w.center().y));
    let s = seeks(&fake);
    assert!(
        s[0] < 60.0,
        "the view stayed near the start while the drag was held: {s:?}"
    );
}

#[test]
fn a_zero_grace_never_follows_the_playhead() {
    let mut s = playing();
    s.config.ui.follow_current_grace_secs = 0.0;
    let (mut h, fake) = harness(s);
    at_position(&fake, 20.0);
    h.run_steps(1);
    let w = wave(&h);
    for _ in 0..6 {
        wheel(
            &mut h,
            pos2(x_of(w, 20.0), w.center().y),
            0.0,
            1.0,
            Modifiers::NONE,
        );
    }
    at_position(&fake, 150.0);
    h.run_steps(30);
    fake.take_sent();
    click(&mut h, pos2(x_of(w, 90.0), w.center().y));
    let s = seeks(&fake);
    assert!(s[0] < 60.0, "{s:?}");
}
