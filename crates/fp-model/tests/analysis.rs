#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Applying analysis results (spec §6) to the model.

mod common;

use common::{entries, fixture, p0};
use fp_model::{
    AudioFormat, Command, EngineAction, FileState, LOUDNESS_VERSION, LoudnessMeasurement,
    MarkerKind, MarkerSource, TrackAnalysis, TrackId, TransitionPlan, apply, loudness_unmeasured,
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
            dsd_rate: None,
            bitrate: None,
        }),
        version: 1,
        loudness: None,
        loudness_pass: None,
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
        EngineAction::Schedule { player, plan: Some(TransitionPlan::StartNextAt { at_secs, fade_current_until_secs: Some(_), ..  }) }
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
        EngineAction::Schedule { plan: Some(TransitionPlan::StartNextAt { at_secs, fade_current_until_secs: None, ..  }), .. }
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
            dsd_rate: None,
            bitrate: None,
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

fn measured(lufs: f32) -> LoudnessMeasurement {
    LoudnessMeasurement {
        integrated_lufs: Some(lufs),
        true_peak_dbtp: -1.5,
    }
}

fn with_loudness(duration: f64, lufs: Option<f32>) -> TrackAnalysis {
    TrackAnalysis {
        loudness: lufs.map(measured),
        loudness_pass: Some(LOUDNESS_VERSION),
        ..analysis(duration)
    }
}

#[test]
fn n6_an_analysis_stores_the_loudness_measurement() {
    let mut state = fixture(1);
    let t = track_of(&state, 0);
    apply(
        &mut state,
        Command::ApplyAnalysis {
            track: t,
            analysis: Box::new(with_loudness(200.0, Some(-11.0))),
        },
    )
    .unwrap();
    let track = state.library.get(t).unwrap();
    assert_eq!(track.loudness, Some(measured(-11.0)));
    assert_eq!(track.loudness_pass, Some(LOUDNESS_VERSION));
    assert!(!track.needs_loudness_pass());
}

#[test]
fn an_analysis_without_a_loudness_pass_clears_the_measurement() {
    // An entry cached before this feature carries no measurement, and the
    // one the track holds may belong to other content (M1): both go, and
    // the loudness-only pass measures the file again.
    let mut state = fixture(1);
    let t = track_of(&state, 0);
    state
        .library
        .get_mut(t)
        .unwrap()
        .apply_loudness(Some(measured(-9.0)), LOUDNESS_VERSION);
    apply(
        &mut state,
        Command::ApplyAnalysis {
            track: t,
            analysis: Box::new(analysis(200.0)),
        },
    )
    .unwrap();
    let track = state.library.get(t).unwrap();
    assert_eq!((track.loudness, track.loudness_pass), (None, None));
    assert!(track.needs_loudness_pass());
}

/// I2: a measurement that is not a finite number is not kept (JSON cannot
/// hold it), by either rule.
#[test]
fn a_non_finite_measurement_is_dropped() {
    let bad = [
        LoudnessMeasurement {
            integrated_lufs: Some(f32::NAN),
            true_peak_dbtp: -1.0,
        },
        LoudnessMeasurement {
            integrated_lufs: Some(f32::NEG_INFINITY),
            true_peak_dbtp: -1.0,
        },
        LoudnessMeasurement {
            integrated_lufs: None,
            true_peak_dbtp: f32::NAN,
        },
    ];
    for m in bad {
        let mut state = fixture(1);
        let t = track_of(&state, 0);
        apply(
            &mut state,
            Command::ApplyLoudness {
                track: t,
                loudness: Some(m),
                version: LOUDNESS_VERSION,
            },
        )
        .unwrap();
        let track = state.library.get(t).unwrap();
        assert_eq!(track.loudness, None);
        assert_eq!(
            track.loudness_pass,
            Some(LOUDNESS_VERSION),
            "the attempt stays"
        );

        let mut state = fixture(1);
        let t = track_of(&state, 0);
        let a = TrackAnalysis {
            loudness: Some(m),
            loudness_pass: Some(LOUDNESS_VERSION),
            ..analysis(200.0)
        };
        apply(
            &mut state,
            Command::ApplyAnalysis {
                track: t,
                analysis: Box::new(a),
            },
        )
        .unwrap();
        assert_eq!(state.library.get(t).unwrap().loudness, None);
    }
}

#[test]
fn n6_a_track_analysed_before_this_feature_needs_the_loudness_pass() {
    let mut state = fixture(1);
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
    assert_eq!((track.loudness, track.loudness_pass), (None, None));
    assert!(track.needs_loudness_pass());
}

#[test]
fn n6b_a_loudness_only_result_changes_no_marker() {
    let mut state = fixture(1);
    let t = track_of(&state, 0);
    apply(
        &mut state,
        Command::ApplyAnalysis {
            track: t,
            analysis: Box::new(analysis(200.0)),
        },
    )
    .unwrap();
    apply(
        &mut state,
        Command::SetMarker {
            track: t,
            kind: MarkerKind::IntroEnd,
            secs: Some(12.0),
        },
    )
    .unwrap();
    let before = state.library.get(t).unwrap().clone();
    apply(
        &mut state,
        Command::ApplyLoudness {
            track: t,
            loudness: Some(measured(-14.0)),
            version: LOUDNESS_VERSION,
        },
    )
    .unwrap();
    let after = state.library.get(t).unwrap();
    assert_eq!(after.markers, before.markers);
    assert_eq!(
        (after.title.as_str(), after.duration_secs),
        (before.title.as_str(), before.duration_secs)
    );
    assert_eq!(after.loudness, Some(measured(-14.0)));
    assert!(!after.needs_loudness_pass());
}

#[test]
fn n6b_a_failed_or_short_measurement_is_recorded_and_not_asked_again() {
    let mut state = fixture(1);
    let t = track_of(&state, 0);
    apply(
        &mut state,
        Command::ApplyAnalysis {
            track: t,
            analysis: Box::new(analysis(200.0)),
        },
    )
    .unwrap();
    apply(
        &mut state,
        Command::ApplyLoudness {
            track: t,
            loudness: None,
            version: LOUDNESS_VERSION,
        },
    )
    .unwrap();
    let track = state.library.get(t).unwrap();
    assert_eq!(track.loudness, None);
    assert!(!track.needs_loudness_pass());
}

#[test]
fn n6b_a_newer_loudness_version_asks_for_the_pass_again() {
    let mut state = fixture(1);
    let t = track_of(&state, 0);
    let track = state.library.get_mut(t).unwrap();
    track.analyzed = true;
    track.apply_loudness(Some(measured(-14.0)), LOUDNESS_VERSION - 1);
    assert!(track.needs_loudness_pass());
}

#[test]
fn a_loudness_result_for_a_gone_track_is_ignored() {
    let mut state = fixture(1);
    let before = state.clone();
    apply(
        &mut state,
        Command::ApplyLoudness {
            track: TrackId(999),
            loudness: Some(measured(-14.0)),
            version: LOUDNESS_VERSION,
        },
    )
    .unwrap();
    assert_eq!(state.library, before.library);
}

#[test]
fn n22_tracks_waiting_for_a_measurement_are_counted() {
    let mut state = fixture(3);
    let ids: Vec<TrackId> = (0..3).map(|n| track_of(&state, n)).collect();
    // One measured, one tried and failed, one never tried.
    state
        .library
        .get_mut(ids[0])
        .unwrap()
        .apply_loudness(Some(measured(-14.0)), LOUDNESS_VERSION);
    state
        .library
        .get_mut(ids[1])
        .unwrap()
        .apply_loudness(None, LOUDNESS_VERSION);
    assert_eq!(loudness_unmeasured(&state), 1);
    // A missing file is not waiting for anything.
    state.library.get_mut(ids[2]).unwrap().file_state = FileState::Missing;
    assert_eq!(loudness_unmeasured(&state), 0);
}

#[test]
fn n6_a_library_saved_before_loudness_loads_without_it() {
    let json = r#"[{"id":1,"path":"/a.flac","title":"A","artist":"","album":"",
        "duration_secs":10.0,"kind":"Music","file_state":"Ok","markers":{},
        "analyzed":true}]"#;
    let lib: fp_model::Library = serde_json::from_str(json).unwrap();
    let t = lib.get(TrackId(1)).unwrap();
    assert_eq!((t.loudness, t.loudness_pass), (None, None));
}
