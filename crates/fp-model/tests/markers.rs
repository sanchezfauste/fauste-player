#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Phase 2 spec P2.8: manual markers.

mod common;

use common::{entries, fixture, p0};
use fp_model::{
    AppState, Command, EngineAction, MarkerKind, MarkerSource, ModelError, TrackAnalysis, TrackId,
    TransitionPlan, apply,
};

fn track(state: &AppState, n: usize) -> TrackId {
    state.playlists.entry(entries(state)[n]).unwrap().track
}

fn marker(state: &AppState, t: TrackId, kind: MarkerKind) -> Option<(f64, MarkerSource)> {
    state
        .library
        .get(t)
        .unwrap()
        .markers
        .get(kind)
        .map(|m| (m.secs, m.source))
}

fn set(
    state: &mut AppState,
    t: TrackId,
    kind: MarkerKind,
    secs: Option<f64>,
) -> Result<Vec<EngineAction>, ModelError> {
    apply(
        state,
        Command::SetMarker {
            track: t,
            kind,
            secs,
        },
    )
}

#[test]
fn a_manual_mix_point_changes_the_scheduled_transition() {
    let mut state = fixture(3);
    let p = p0(&state);
    apply(&mut state, Command::Play(p)).unwrap();
    let t = track(&state, 0);
    let actions = set(&mut state, t, MarkerKind::SegueStart, Some(170.0)).unwrap();
    assert!(actions.iter().any(|a| matches!(
        a,
        EngineAction::Schedule { player, plan: Some(TransitionPlan::StartNextAt { at_secs, .. }) }
            if *player == p && *at_secs == 170.0
    )), "{actions:?}");
    assert_eq!(
        marker(&state, t, MarkerKind::SegueStart),
        Some((170.0, MarkerSource::Manual))
    );
}

#[test]
fn cue_in_after_cue_out_is_refused() {
    let mut state = fixture(1);
    let t = track(&state, 0);
    assert_eq!(
        set(&mut state, t, MarkerKind::CueIn, Some(200.0)),
        Err(ModelError::InvalidMarker)
    );
    set(&mut state, t, MarkerKind::CueIn, Some(10.0)).unwrap();
    assert_eq!(
        set(&mut state, t, MarkerKind::CueOut, Some(5.0)),
        Err(ModelError::InvalidMarker)
    );
    assert_eq!(
        marker(&state, t, MarkerKind::CueOut),
        None,
        "nothing changed"
    );
}

#[test]
fn markers_outside_the_cue_range_are_clamped() {
    let mut state = fixture(1);
    let t = track(&state, 0);
    set(&mut state, t, MarkerKind::CueIn, Some(2.0)).unwrap();
    set(&mut state, t, MarkerKind::IntroEnd, Some(500.0)).unwrap();
    set(&mut state, t, MarkerKind::OutroStart, Some(-3.0)).unwrap();
    assert_eq!(marker(&state, t, MarkerKind::IntroEnd).unwrap().0, 180.0);
    assert_eq!(marker(&state, t, MarkerKind::OutroStart).unwrap().0, 2.0);
}

#[test]
fn non_finite_values_are_refused() {
    let mut state = fixture(1);
    let t = track(&state, 0);
    assert_eq!(
        set(&mut state, t, MarkerKind::IntroEnd, Some(f64::NAN)),
        Err(ModelError::InvalidMarker)
    );
}

#[test]
fn clearing_a_manual_marker_removes_it() {
    let mut state = fixture(1);
    let t = track(&state, 0);
    set(&mut state, t, MarkerKind::IntroEnd, Some(12.0)).unwrap();
    set(&mut state, t, MarkerKind::IntroEnd, None).unwrap();
    assert_eq!(marker(&state, t, MarkerKind::IntroEnd), None);
}

#[test]
fn reset_markers_requests_a_new_analysis_and_keeps_nothing_manual() {
    let mut state = fixture(1);
    let t = track(&state, 0);
    state.library.get_mut(t).unwrap().analyzed = true;
    set(&mut state, t, MarkerKind::IntroEnd, Some(12.0)).unwrap();
    set(&mut state, t, MarkerKind::SegueStart, Some(170.0)).unwrap();
    apply(&mut state, Command::ResetMarkers { track: t }).unwrap();
    let tr = state.library.get(t).unwrap();
    assert!(!tr.analyzed, "analysis runs again");
    for kind in [
        MarkerKind::CueIn,
        MarkerKind::IntroEnd,
        MarkerKind::OutroStart,
        MarkerKind::SegueStart,
        MarkerKind::CueOut,
    ] {
        assert!(
            tr.markers
                .get(kind)
                .is_none_or(|m| m.source == MarkerSource::Auto),
            "{kind:?}"
        );
    }
}

#[test]
fn analysis_never_overwrites_a_manual_intro() {
    let mut state = fixture(1);
    let t = track(&state, 0);
    set(&mut state, t, MarkerKind::IntroEnd, Some(12.0)).unwrap();
    let analysis = TrackAnalysis {
        duration_secs: 180.0,
        intro_end: Some(20.0),
        ..TrackAnalysis::default()
    };
    apply(
        &mut state,
        Command::ApplyAnalysis {
            track: t,
            analysis: Box::new(analysis),
        },
    )
    .unwrap();
    assert_eq!(
        marker(&state, t, MarkerKind::IntroEnd),
        Some((12.0, MarkerSource::Manual))
    );
}

#[test]
fn an_intro_from_analysis_is_automatic() {
    let mut state = fixture(1);
    let t = track(&state, 0);
    let analysis = TrackAnalysis {
        duration_secs: 180.0,
        intro_end: Some(20.0),
        ..TrackAnalysis::default()
    };
    apply(
        &mut state,
        Command::ApplyAnalysis {
            track: t,
            analysis: Box::new(analysis),
        },
    )
    .unwrap();
    assert_eq!(
        marker(&state, t, MarkerKind::IntroEnd),
        Some((20.0, MarkerSource::Auto))
    );
}

#[test]
fn unknown_tracks_are_refused() {
    let mut state = fixture(0);
    assert_eq!(
        set(
            &mut state,
            TrackId(123_456),
            MarkerKind::IntroEnd,
            Some(1.0)
        ),
        Err(ModelError::UnknownTrack(TrackId(123_456)))
    );
}
