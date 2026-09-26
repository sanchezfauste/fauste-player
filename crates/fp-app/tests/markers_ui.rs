#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Phase 2 spec P2.8: marker editing on the waveform.

mod support;

use egui::{Event, Modifiers, PointerButton, pos2};
use egui_kittest::kittest::Queryable;
use fp_model::{AppState, Command, MarkerKind, TrackAnalysis, TrackId, apply};
use support::{harness, state};

/// P1 plays a 180 s track with a MIX point at 170 s.
fn playing() -> (AppState, TrackId) {
    let mut s = state(1, 2);
    let e = s.playlists.iter().next().unwrap().entries[0];
    let analysis = TrackAnalysis {
        duration_secs: 180.0,
        cue_in: Some(0.0),
        cue_out: Some(180.0),
        segue_start: Some(170.0),
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
    (s, e.track)
}

fn set_markers(sent: &[Command]) -> Vec<(TrackId, MarkerKind, Option<f64>)> {
    sent.iter()
        .filter_map(|c| match c {
            Command::SetMarker { track, kind, secs } => Some((*track, *kind, *secs)),
            _ => None,
        })
        .collect()
}

#[test]
fn the_waveform_menu_sets_the_intro_here() {
    let (s, track) = playing();
    let (mut h, fake) = harness(s);
    let wave = h.get_by_label("Waveform: click to seek").rect();
    let at = pos2(wave.left() + wave.width() * 0.25, wave.center().y);
    h.event(Event::PointerMoved(at));
    h.event(Event::PointerButton {
        pos: at,
        button: PointerButton::Secondary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    h.event(Event::PointerButton {
        pos: at,
        button: PointerButton::Secondary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(2);
    h.get_by_label("Set intro end here").click();
    h.run_steps(2);
    let set = set_markers(&fake.take_sent());
    assert_eq!(set.len(), 1);
    let (t, kind, secs) = set[0];
    assert_eq!((t, kind), (track, MarkerKind::IntroEnd));
    assert!((secs.unwrap() - 45.0).abs() < 2.0, "{secs:?}");
}

#[test]
fn reset_markers_is_sent() {
    let (s, track) = playing();
    let (mut h, fake) = harness(s);
    let wave = h.get_by_label("Waveform: click to seek").rect();
    h.event(Event::PointerMoved(wave.center()));
    h.event(Event::PointerButton {
        pos: wave.center(),
        button: PointerButton::Secondary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    h.event(Event::PointerButton {
        pos: wave.center(),
        button: PointerButton::Secondary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(2);
    h.get_by_label("Reset markers to automatic").click();
    h.run_steps(2);
    assert!(fake.take_sent().contains(&Command::ResetMarkers { track }));
}

#[test]
fn alt_dragging_the_mix_marker_moves_it() {
    let (s, track) = playing();
    let (mut h, fake) = harness(s);
    let wave = h.get_by_label("Waveform: click to seek").rect();
    let x_of = |secs: f32| wave.left() + 1.0 + (wave.width() - 2.0) * secs / 180.0;
    let from = pos2(x_of(170.0), wave.center().y);
    let to = pos2(x_of(150.0), wave.center().y);
    h.event(Event::ModifiersChanged(Modifiers::ALT));
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    h.event(Event::PointerButton {
        pos: from,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::ALT,
    });
    h.run_steps(1);
    for step in 1..=10 {
        let x = from.x + (to.x - from.x) * step as f32 / 10.0;
        h.event(Event::PointerMoved(pos2(x, from.y)));
        h.run_steps(1);
    }
    h.event(Event::PointerButton {
        pos: to,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::ALT,
    });
    h.run_steps(2);
    let sent = fake.take_sent();
    let set = set_markers(&sent);
    assert_eq!(set.len(), 1, "one command on release: {sent:?}");
    let (t, kind, secs) = set[0];
    assert_eq!((t, kind), (track, MarkerKind::SegueStart));
    assert!((secs.unwrap() - 150.0).abs() < 2.0, "{secs:?}");
    assert!(
        !sent.iter().any(|c| matches!(c, Command::Seek(..))),
        "a marker drag does not seek"
    );
}
