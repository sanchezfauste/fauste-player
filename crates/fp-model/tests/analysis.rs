#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Applying analysis results (spec §6) to the model.

mod common;

use common::{entries, fixture, p0};
use fp_model::{
    AudioFormat, Command, EngineAction, FileState, MarkerKind, MarkerSource, TrackAnalysis,
    TrackId, TransitionPlan, apply,
};

fn analysis(duration: f64) -> TrackAnalysis {
    TrackAnalysis {
        title: Some("Real Title".into()),
        artist: Some("Real Artist".into()),
        album: None,
        duration_secs: duration,
        cue_in: Some(0.5),
        cue_out: Some(duration - 1.0),
        segue_start: Some(duration - 6.0),
        outro_start: Some(duration - 20.0),
        intro_end: None,
        format: Some(AudioFormat {
            sample_rate: 44_100,
            bits: Some(16),
            channels: 2,
        }),
        version: 1,
    }
}

fn track_of(state: &fp_model::AppState, n: usize) -> TrackId {
    state.playlists.entry(entries(state)[n]).unwrap().track
}

#[test]
fn analysis_fills_metadata_and_markers() {
    let mut state = fixture(2);
    let t = track_of(&state, 0);
    apply(
        &mut state,
        Command::ApplyAnalysis {
            track: t,
            analysis: Box::new(analysis(200.0)),
        },
    )
    .unwrap();
    let track = state.library.get(t).unwrap();
    assert_eq!(
        (track.title.as_str(), track.artist.as_str()),
        ("Real Title", "Real Artist")
    );
    assert_eq!(track.duration_secs, 200.0);
    assert_eq!(track.cue_in_secs(), 0.5);
    assert_eq!(track.cue_out_secs(), 199.0);
    assert_eq!(track.segue_start_secs(), Some(194.0));
    assert_eq!(track.outro_start_secs(), Some(180.0));
    assert!(track.analyzed);
    assert_eq!(
        track.markers.get(MarkerKind::SegueStart).unwrap().source,
        MarkerSource::Auto
    );
}

#[test]
fn analysis_does_not_erase_known_metadata_with_nothing() {
    let mut state = fixture(1);
    let t = track_of(&state, 0);
    let before = state.library.get(t).unwrap().title.clone();
    let mut a = analysis(100.0);
    a.title = None;
    apply(
        &mut state,
        Command::ApplyAnalysis {
            track: t,
            analysis: Box::new(a),
        },
    )
    .unwrap();
    assert_eq!(state.library.get(t).unwrap().title, before);
}

#[test]
fn re_analysis_never_overwrites_manual_markers() {
    let mut state = fixture(1);
    let t = track_of(&state, 0);
    state
        .library
        .get_mut(t)
        .unwrap()
        .markers
        .set_manual(MarkerKind::SegueStart, Some(150.0));
    apply(
        &mut state,
        Command::ApplyAnalysis {
            track: t,
            analysis: Box::new(analysis(200.0)),
        },
    )
    .unwrap();
    assert_eq!(
        state.library.get(t).unwrap().segue_start_secs(),
        Some(150.0)
    );
}

#[test]
fn a_newly_known_segue_reschedules_the_playing_transition() {
    let mut state = fixture(2);
    let p = p0(&state);
    apply(&mut state, Command::Play(p)).unwrap();
    let t = track_of(&state, 0);
    let actions = apply(
        &mut state,
        Command::ApplyAnalysis {
            track: t,
            analysis: Box::new(analysis(200.0)),
        },
    )
    .unwrap();
    assert!(actions.iter().any(|a| matches!(
        a,
        EngineAction::Schedule { player, plan: Some(TransitionPlan::StartNextAt { at_secs, fade_current_until_secs: Some(_) }) }
            if *player == p && *at_secs == 194.0
    )), "{actions:?}");
}

#[test]
fn a_missing_file_is_skipped_as_next() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    let t = track_of(&state, 1);
    apply(
        &mut state,
        Command::SetFileState {
            track: t,
            state: FileState::Missing,
        },
    )
    .unwrap();
    assert_eq!(state.player(p).unwrap().next, Some(e[2]));
}

#[test]
fn results_for_removed_tracks_are_ignored() {
    let mut state = fixture(1);
    let actions = apply(
        &mut state,
        Command::ApplyAnalysis {
            track: TrackId(987_654),
            analysis: Box::new(analysis(10.0)),
        },
    );
    assert!(actions.is_ok());
    assert!(
        apply(
            &mut state,
            Command::SetFileState {
                track: TrackId(987_654),
                state: FileState::Missing
            }
        )
        .is_ok()
    );
}

#[test]
fn a_newly_known_cue_in_re_preloads_the_next() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    let t = track_of(&state, 1);
    let mut a = analysis(200.0);
    a.cue_in = Some(2.5);
    let actions = apply(
        &mut state,
        Command::ApplyAnalysis {
            track: t,
            analysis: Box::new(a),
        },
    )
    .unwrap();
    assert!(actions.iter().any(|a| matches!(
        a,
        EngineAction::Preload { player, request: Some(r) } if *player == p && r.entry == e[1] && r.from_secs == 2.5
    )), "{actions:?}");
}

#[test]
fn an_automatic_segue_outside_a_manual_cue_out_is_not_used() {
    let mut state = fixture(2);
    let p = p0(&state);
    let t = track_of(&state, 0);
    state
        .library
        .get_mut(t)
        .unwrap()
        .markers
        .set_manual(MarkerKind::CueOut, Some(150.0));
    apply(
        &mut state,
        Command::ApplyAnalysis {
            track: t,
            analysis: Box::new(analysis(206.0)),
        },
    )
    .unwrap();
    let actions = apply(&mut state, Command::Play(p)).unwrap();
    assert!(actions.iter().any(|a| matches!(
        a,
        EngineAction::Schedule { plan: Some(TransitionPlan::StartNextAt { at_secs, fade_current_until_secs: None }), .. }
            if *at_secs == 150.0
    )), "the manual cue-out wins and the stale segue is ignored: {actions:?}");
}

#[test]
fn requests_carry_the_track_format() {
    let mut state = fixture(2);
    let p = p0(&state);
    let t = track_of(&state, 0);
    apply(
        &mut state,
        Command::ApplyAnalysis {
            track: t,
            analysis: Box::new(analysis(200.0)),
        },
    )
    .unwrap();
    assert_eq!(
        state.library.get(t).unwrap().format,
        Some(AudioFormat {
            sample_rate: 44_100,
            bits: Some(16),
            channels: 2,
        })
    );
    let actions = apply(&mut state, Command::Play(p)).unwrap();
    let request = actions
        .iter()
        .find_map(|a| match a {
            EngineAction::StartCurrent { request, .. } => Some(request.clone()),
            _ => None,
        })
        .unwrap();
    assert_eq!(request.format.map(|f| f.sample_rate), Some(44_100));
}

#[test]
fn a_library_without_formats_loads() {
    let track: fp_model::Track = serde_json::from_str(
        r#"{"id":1,"path":"/music/a.flac","title":"A","artist":"","album":"",
            "duration_secs":10.0,"kind":"Music","file_state":"Ok",
            "markers":{},"analyzed":true}"#,
    )
    .unwrap();
    assert_eq!(track.format, None);
}
