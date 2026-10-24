#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::float_cmp
)]
//! Operator feedback 4, Q1: a track's length from its file header, until
//! its analysis replaces it.

use std::path::PathBuf;

use fp_model::{
    AppState, Command, Config, FileState, Track, TrackAnalysis, TrackId, TransitionPlan, apply,
    plan_for,
};

/// One playlist of `n` tracks, none analysed, none with a length.
fn unanalysed(n: usize) -> (AppState, Vec<TrackId>) {
    let mut s = AppState::new(Config::default(), "Main");
    let playlist = s.playlists.first_id().unwrap();
    let paths = (0..n)
        .map(|i| PathBuf::from(format!("/music/t{i}.mp3")))
        .collect();
    apply(
        &mut s,
        Command::InsertPaths {
            playlist,
            index: 0,
            paths,
        },
    )
    .unwrap();
    let tracks = s
        .playlists
        .get(playlist)
        .unwrap()
        .entries
        .iter()
        .map(|e| e.track)
        .collect();
    (s, tracks)
}

fn length(s: &AppState, t: TrackId) -> f64 {
    s.library.get(t).unwrap().duration_secs
}

fn analysis(duration_secs: f64) -> Box<TrackAnalysis> {
    Box::new(TrackAnalysis {
        duration_secs,
        ..TrackAnalysis::default()
    })
}

#[test]
fn q1_2_a_header_duration_is_stored_before_the_analysis() {
    let (mut s, t) = unanalysed(1);
    apply(
        &mut s,
        Command::SetDuration {
            track: t[0],
            secs: 200.0,
        },
    )
    .unwrap();
    assert_eq!(length(&s, t[0]), 200.0);
    assert!(!s.library.get(t[0]).unwrap().analyzed);
}

#[test]
fn q1_2_the_analysis_replaces_the_header_duration() {
    let (mut s, t) = unanalysed(1);
    apply(
        &mut s,
        Command::SetDuration {
            track: t[0],
            secs: 200.0,
        },
    )
    .unwrap();
    apply(
        &mut s,
        Command::ApplyAnalysis {
            track: t[0],
            analysis: analysis(198.5),
        },
    )
    .unwrap();
    assert_eq!(length(&s, t[0]), 198.5);
}

#[test]
fn an_analysis_without_a_length_keeps_the_header_duration() {
    let (mut s, t) = unanalysed(1);
    apply(
        &mut s,
        Command::SetDuration {
            track: t[0],
            secs: 200.0,
        },
    )
    .unwrap();
    apply(
        &mut s,
        Command::ApplyAnalysis {
            track: t[0],
            analysis: analysis(0.0),
        },
    )
    .unwrap();
    assert_eq!(length(&s, t[0]), 200.0);
}

#[test]
fn q1_2_a_header_duration_after_the_analysis_is_ignored() {
    let (mut s, t) = unanalysed(1);
    apply(
        &mut s,
        Command::ApplyAnalysis {
            track: t[0],
            analysis: analysis(198.5),
        },
    )
    .unwrap();
    apply(
        &mut s,
        Command::SetDuration {
            track: t[0],
            secs: 250.0,
        },
    )
    .unwrap();
    assert_eq!(length(&s, t[0]), 198.5);
}

#[test]
fn q1_2_a_duration_that_is_not_finite_and_positive_is_rejected() {
    let (mut s, t) = unanalysed(1);
    for bad in [0.0, -3.0, f64::NAN, f64::INFINITY] {
        apply(
            &mut s,
            Command::SetDuration {
                track: t[0],
                secs: bad,
            },
        )
        .unwrap();
        assert_eq!(length(&s, t[0]), 0.0, "{bad}");
    }
}

#[test]
fn q1_2_an_unknown_track_is_ignored() {
    let (mut s, _) = unanalysed(1);
    let before = s.clone();
    apply(
        &mut s,
        Command::SetDuration {
            track: TrackId(999_999),
            secs: 10.0,
        },
    )
    .unwrap();
    assert_eq!(s.library, before.library);
}

#[test]
fn q1_5_the_play_range_of_such_a_track_is_zero_to_its_duration() {
    let (mut s, t) = unanalysed(1);
    apply(
        &mut s,
        Command::SetDuration {
            track: t[0],
            secs: 200.0,
        },
    )
    .unwrap();
    for use_markers in [true, false] {
        let range = s.library.get(t[0]).unwrap().play_range(use_markers);
        assert_eq!(range.cue_in, 0.0);
        assert_eq!(range.known_end(), Some(200.0), "{use_markers}");
    }
}

#[test]
fn q1_5_the_transition_is_planned_at_the_header_duration() {
    let (mut s, t) = unanalysed(2);
    let p = s.players[0].id;
    apply(&mut s, Command::Play(p)).unwrap();
    apply(
        &mut s,
        Command::SetDuration {
            track: t[0],
            secs: 200.0,
        },
    )
    .unwrap();
    let at = match plan_for(&s, s.player(p).unwrap()) {
        Some(TransitionPlan::StartNextAt { at_secs, .. } | TransitionPlan::StopAt { at_secs }) => {
            at_secs
        }
        other => panic!("{other:?}"),
    };
    assert_eq!(at, 200.0);
}

#[test]
fn a_track_needs_a_header_read_only_while_it_has_no_length_and_no_analysis() {
    let mut t = Track::new(TrackId(1), PathBuf::from("/music/a.mp3"));
    assert!(t.needs_header_duration());
    t.duration_secs = 10.0;
    assert!(!t.needs_header_duration(), "it has a length");
    t.duration_secs = 0.0;
    t.file_state = FileState::Missing;
    assert!(!t.needs_header_duration(), "no file to read");
    t.file_state = FileState::Ok;
    t.analyzed = true;
    assert!(!t.needs_header_duration(), "analysed");
}
