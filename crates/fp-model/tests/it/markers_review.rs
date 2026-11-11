#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! External editor spec EE5: the markers-review flag and the file stamp.

use crate::common::fixture;
use fp_model::{AppState, Command, FileStamp, MarkerKind, MarkersReview, TrackId, apply};

fn flagged(state: &mut AppState, track: TrackId) {
    state.library.get_mut(track).unwrap().markers_review = Some(MarkersReview {
        dropped: vec![MarkerKind::IntroEnd],
    });
}

fn review(state: &AppState, track: TrackId) -> Option<MarkersReview> {
    state.library.get(track).unwrap().markers_review.clone()
}

fn ids(state: &AppState) -> Vec<TrackId> {
    state.library.ids()
}

#[test]
fn mark_reviewed_clears_the_flag_of_each_track() {
    let mut s = fixture(3);
    let t = ids(&s);
    for id in &t {
        flagged(&mut s, *id);
    }
    apply(
        &mut s,
        Command::MarkReviewed {
            tracks: vec![t[0], t[2], TrackId(9999)],
        },
    )
    .unwrap();
    assert_eq!(review(&s, t[0]), None);
    assert!(review(&s, t[1]).is_some());
    assert_eq!(review(&s, t[2]), None);
}

#[test]
fn setting_a_marker_clears_the_flag() {
    let mut s = fixture(1);
    let t = ids(&s)[0];
    flagged(&mut s, t);
    apply(
        &mut s,
        Command::SetMarker {
            track: t,
            kind: MarkerKind::IntroEnd,
            secs: Some(5.0),
        },
    )
    .unwrap();
    assert_eq!(review(&s, t), None);
}

#[test]
fn clearing_a_marker_clears_the_flag() {
    let mut s = fixture(1);
    let t = ids(&s)[0];
    flagged(&mut s, t);
    apply(
        &mut s,
        Command::SetMarker {
            track: t,
            kind: MarkerKind::IntroEnd,
            secs: None,
        },
    )
    .unwrap();
    assert_eq!(review(&s, t), None);
}

#[test]
fn reset_markers_clears_the_flag() {
    let mut s = fixture(1);
    let t = ids(&s)[0];
    flagged(&mut s, t);
    apply(&mut s, Command::ResetMarkers { track: t }).unwrap();
    assert_eq!(review(&s, t), None);
}

#[test]
fn a_refused_marker_keeps_the_flag() {
    let mut s = fixture(1);
    let t = ids(&s)[0];
    flagged(&mut s, t);
    let r = apply(
        &mut s,
        Command::SetMarker {
            track: t,
            kind: MarkerKind::IntroEnd,
            secs: Some(f64::NAN),
        },
    );
    assert!(r.is_err());
    assert!(review(&s, t).is_some());
}

#[test]
fn set_file_stamp_fills_only_a_missing_stamp() {
    let mut s = fixture(1);
    let t = ids(&s)[0];
    let first = FileStamp {
        size: 10,
        modified_ns: 20,
    };
    let second = FileStamp {
        size: 11,
        modified_ns: 21,
    };
    s.library.get_mut(t).unwrap().file_stamp = None;
    apply(
        &mut s,
        Command::SetFileStamp {
            track: t,
            stamp: first,
        },
    )
    .unwrap();
    assert_eq!(s.library.get(t).unwrap().file_stamp, Some(first));
    apply(
        &mut s,
        Command::SetFileStamp {
            track: t,
            stamp: second,
        },
    )
    .unwrap();
    assert_eq!(s.library.get(t).unwrap().file_stamp, Some(first));
    apply(
        &mut s,
        Command::SetFileStamp {
            track: TrackId(9999),
            stamp: second,
        },
    )
    .unwrap();
}
