#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O15: players may ignore cue-in and cue-out.

mod common;

use common::fixture;
use fp_model::{Command, Config, MarkerKind, TrackId, apply};

/// Gives every track of `state` a cue-in and a cue-out (manual).
fn mark_all(state: &mut fp_model::AppState, cue_in: f64, cue_out: f64) {
    let tracks: Vec<TrackId> = state.library.iter().map(|t| t.id).collect();
    for track in tracks {
        for (kind, secs) in [(MarkerKind::CueIn, cue_in), (MarkerKind::CueOut, cue_out)] {
            apply(
                state,
                Command::SetMarker {
                    track,
                    kind,
                    secs: Some(secs),
                },
            )
            .unwrap();
        }
    }
}

#[test]
fn the_setting_defaults_to_on() {
    assert!(Config::default().players.use_cue_markers);
}

#[test]
fn the_range_follows_the_markers_when_they_are_used() {
    let mut state = fixture(1);
    mark_all(&mut state, 5.0, 170.0);
    let t = state.library.iter().next().unwrap();
    let r = t.play_range(true);
    assert_eq!((r.cue_in, r.cue_out), (5.0, 170.0));
    assert_eq!(r.known_end(), Some(170.0));
    assert_eq!(r.length(), 165.0);
}

#[test]
fn the_range_is_the_whole_file_when_the_markers_are_ignored_and_they_are_kept() {
    let mut state = fixture(1);
    mark_all(&mut state, 5.0, 170.0);
    let t = state.library.iter().next().unwrap();
    let r = t.play_range(false);
    assert_eq!((r.cue_in, r.cue_out), (0.0, 180.0));
    assert_eq!(r.known_end(), Some(180.0));
    assert_eq!(r.length(), 180.0);
    assert_eq!(t.cue_in_secs(), 5.0, "kept");
    assert_eq!(t.cue_out_secs(), 170.0, "kept");
}

#[test]
fn an_unknown_duration_has_no_known_end() {
    let mut state = fixture(1);
    for t in state.library.iter_mut() {
        t.duration_secs = 0.0;
    }
    let t = state.library.iter().next().unwrap();
    for use_markers in [true, false] {
        let r = t.play_range(use_markers);
        assert_eq!(r.known_end(), None, "{use_markers}");
        assert_eq!(r.length(), 0.0);
        assert!(r.cue_in.is_finite() && r.cue_out.is_finite());
    }
}
