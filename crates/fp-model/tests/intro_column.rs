#![allow(clippy::unwrap_used)]
//! Feedback 2 spec O24: the Intro column shows how long the intro lasts.

use std::path::PathBuf;

use fp_model::{MarkerKind, Track, TrackId};

fn track(cue_in: Option<f64>, intro_end: Option<f64>) -> Track {
    let mut t = Track::new(TrackId(1), PathBuf::from("/m/a.flac"));
    t.duration_secs = 200.0;
    t.markers.set_manual(MarkerKind::CueIn, cue_in);
    t.markers.set_manual(MarkerKind::IntroEnd, intro_end);
    t
}

#[test]
fn the_intro_runs_from_the_start_to_its_marker() {
    assert_eq!(track(None, Some(12.0)).intro_secs(true), Some(12.0));
}

#[test]
fn with_cue_markers_on_it_runs_from_the_cue_in() {
    let t = track(Some(2.0), Some(12.0));
    assert_eq!(t.intro_secs(true), Some(10.0));
    assert_eq!(t.intro_secs(false), Some(12.0), "markers off: from 0");
}

#[test]
fn without_an_intro_marker_there_is_none() {
    assert_eq!(track(None, None).intro_secs(true), None);
}

#[test]
fn a_marker_not_after_the_start_gives_none() {
    assert_eq!(track(Some(5.0), Some(5.0)).intro_secs(true), None);
    assert_eq!(track(Some(8.0), Some(5.0)).intro_secs(true), None);
}
