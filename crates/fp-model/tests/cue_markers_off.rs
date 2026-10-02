#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O15: players may ignore cue-in and cue-out.

mod common;

use common::{fixture, p0};
use fp_model::{
    AppState, Command, Config, EngineAction, MarkerKind, PlayMode, PlayerId, PlayerSession,
    RestoreParts, SOURCE_END, TrackId, TransitionPlan, Transport, apply,
};

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

fn set_use(state: &mut AppState, on: bool) -> Vec<EngineAction> {
    let mut config = state.config.clone();
    config.players.use_cue_markers = on;
    apply(state, Command::UpdateConfig(Box::new(config))).unwrap()
}

fn start_secs(actions: &[EngineAction]) -> Option<f64> {
    actions.iter().find_map(|a| match a {
        EngineAction::StartCurrent { request, .. } => Some(request.from_secs),
        _ => None,
    })
}

/// The last plan scheduled for `p` in `actions`, if any.
fn scheduled(actions: &[EngineAction], p: PlayerId) -> Option<Option<TransitionPlan>> {
    actions.iter().rev().find_map(|a| match a {
        EngineAction::Schedule { player, plan } if *player == p => Some(*plan),
        _ => None,
    })
}

fn preload_secs(actions: &[EngineAction], p: PlayerId) -> Option<f64> {
    actions.iter().find_map(|a| match a {
        EngineAction::Preload {
            player,
            request: Some(r),
        } if *player == p => Some(r.from_secs),
        _ => None,
    })
}

#[test]
fn the_engine_starts_a_player_entry_at_0_when_the_markers_are_ignored() {
    for (on, want) in [(true, 5.0), (false, 0.0)] {
        let mut state = fixture(2);
        mark_all(&mut state, 5.0, 170.0);
        set_use(&mut state, on);
        let p = p0(&state);
        let actions = apply(&mut state, Command::Play(p)).unwrap();
        assert_eq!(start_secs(&actions), Some(want), "use markers {on}");
    }
}

#[test]
fn restart_goes_back_to_the_start_of_the_range() {
    for (on, want) in [(true, 5.0), (false, 0.0)] {
        let mut state = fixture(2);
        mark_all(&mut state, 5.0, 170.0);
        set_use(&mut state, on);
        let p = p0(&state);
        apply(&mut state, Command::Play(p)).unwrap();
        let actions = apply(&mut state, Command::Restart(p)).unwrap();
        assert!(
            actions.iter().any(|a| matches!(
                a,
                EngineAction::Seek { player, secs } if *player == p && *secs == want
            )),
            "use markers {on}: {actions:?}"
        );
    }
}

#[test]
fn the_player_cue_starts_at_the_play_range_start() {
    for (on, want) in [(true, 5.0), (false, 0.0)] {
        let mut state = fixture(2);
        mark_all(&mut state, 5.0, 170.0);
        set_use(&mut state, on);
        let p = p0(&state);
        let actions = apply(&mut state, Command::ToggleCue(p)).unwrap();
        let from = actions.iter().find_map(|a| match a {
            EngineAction::StartCue { request, .. } => Some(request.from_secs),
            _ => None,
        });
        assert_eq!(from, Some(want), "use markers {on}");
    }
}

#[test]
fn a_transition_is_planned_at_the_end_of_the_range() {
    for (on, want) in [(true, 170.0), (false, 180.0)] {
        let mut state = fixture(3);
        mark_all(&mut state, 5.0, 170.0);
        set_use(&mut state, on);
        let p = p0(&state);
        let actions = apply(&mut state, Command::Play(p)).unwrap();
        assert_eq!(
            scheduled(&actions, p),
            Some(Some(TransitionPlan::StartNextAt {
                at_secs: want,
                fade_current_until_secs: None
            })),
            "use markers {on}"
        );
        // Single mode stops at the same end.
        apply(&mut state, Command::SetMode(p, PlayMode::Single)).unwrap();
        let plan = fp_model::plan_for(&state, state.player(p).unwrap());
        assert_eq!(plan, Some(TransitionPlan::StopAt { at_secs: want }));
    }
}

#[test]
fn a_mix_point_past_the_cue_out_is_used_only_while_the_markers_are_off() {
    let mut state = fixture(3);
    mark_all(&mut state, 5.0, 170.0);
    for t in state.library.iter_mut() {
        t.markers.set_auto(MarkerKind::SegueStart, Some(175.0));
    }
    let p = p0(&state);
    apply(&mut state, Command::Play(p)).unwrap();
    let plan = |s: &AppState| fp_model::plan_for(s, s.player(p).unwrap());
    assert_eq!(
        plan(&state),
        Some(TransitionPlan::StartNextAt {
            at_secs: 170.0,
            fade_current_until_secs: None
        }),
        "stale against the cue-out"
    );
    set_use(&mut state, false);
    assert_eq!(
        plan(&state),
        Some(TransitionPlan::StartNextAt {
            at_secs: 175.0,
            fade_current_until_secs: Some(180.0)
        })
    );
    set_use(&mut state, true);
    assert_eq!(
        plan(&state),
        Some(TransitionPlan::StartNextAt {
            at_secs: 170.0,
            fade_current_until_secs: None
        })
    );
}

#[test]
fn the_mix_point_is_validated_against_the_effective_range() {
    for (on, want) in [(true, 170.0), (false, 178.0)] {
        let mut state = fixture(2);
        mark_all(&mut state, 5.0, 170.0);
        set_use(&mut state, on);
        let t = state.library.iter().next().unwrap().id;
        apply(
            &mut state,
            Command::SetMarker {
                track: t,
                kind: MarkerKind::SegueStart,
                secs: Some(178.0),
            },
        )
        .unwrap();
        let m = state.library.get(t).unwrap().markers.segue_start.unwrap();
        assert_eq!(m.secs, want, "use markers {on}");
    }
}

#[test]
fn the_cue_points_themselves_are_still_validated_against_the_kept_markers() {
    let mut state = fixture(2);
    mark_all(&mut state, 5.0, 170.0);
    set_use(&mut state, false);
    let t = state.library.iter().next().unwrap().id;
    let refused = apply(
        &mut state,
        Command::SetMarker {
            track: t,
            kind: MarkerKind::CueIn,
            secs: Some(171.0),
        },
    );
    assert!(refused.is_err(), "cue-in past the kept cue-out");
}

#[test]
fn a_restored_position_past_the_end_comes_back_at_the_start_of_the_range() {
    for (saved, want) in [(179.0, 179.0), (240.0, 0.0)] {
        let mut state = fixture(3);
        mark_all(&mut state, 0.5, 100.0);
        state.config.players.use_cue_markers = false;
        let p = p0(&state);
        apply(&mut state, Command::Play(p)).unwrap();
        let sessions: Vec<PlayerSession> = state.sessions(|_| saved);
        let parts = RestoreParts {
            config: state.config.clone(),
            library: state.library.clone(),
            playlists: state.playlists.clone(),
            cart_pages: state.cartwall.pages.clone(),
            cartwall_session: state.cartwall.session(),
            ids: state.ids.clone(),
        };
        let (restored, actions) = AppState::restore(parts, &sessions, "Main");
        let at = actions.iter().find_map(|a| match a {
            EngineAction::LoadPaused { player, request } if *player == p => Some(request.from_secs),
            _ => None,
        });
        assert_eq!(at, Some(want), "saved at {saved}");
        assert_eq!(restored.player(p).unwrap().transport, Transport::Paused);
    }
}

#[test]
fn an_unknown_duration_plans_to_the_source_end() {
    let mut state = fixture(3);
    for t in state.library.iter_mut() {
        t.duration_secs = 0.0;
    }
    set_use(&mut state, false);
    let p = p0(&state);
    let actions = apply(&mut state, Command::Play(p)).unwrap();
    assert_eq!(
        scheduled(&actions, p),
        Some(Some(TransitionPlan::StartNextAt {
            at_secs: SOURCE_END,
            fade_current_until_secs: None
        }))
    );
}

#[test]
fn toggling_re_preloads_the_next_entry_from_the_new_start() {
    let mut state = fixture(3);
    mark_all(&mut state, 5.0, 170.0);
    let p = p0(&state);
    let off = set_use(&mut state, false);
    assert_eq!(preload_secs(&off, p), Some(0.0));
    let on = set_use(&mut state, true);
    assert_eq!(preload_secs(&on, p), Some(5.0));
}

#[test]
fn toggling_while_on_air_replans_without_touching_the_sound() {
    let mut state = fixture(3);
    mark_all(&mut state, 5.0, 170.0);
    let p = p0(&state);
    apply(&mut state, Command::Play(p)).unwrap();
    let actions = set_use(&mut state, false);
    assert_eq!(
        scheduled(&actions, p),
        Some(Some(TransitionPlan::StartNextAt {
            at_secs: 180.0,
            fade_current_until_secs: None
        }))
    );
    assert!(
        !actions.iter().any(|a| matches!(
            a,
            EngineAction::StartCurrent { .. }
                | EngineAction::Seek { .. }
                | EngineAction::StopNow { .. }
                | EngineAction::Crossfade { .. }
        )),
        "{actions:?}"
    );
    assert_eq!(state.player(p).unwrap().transport, Transport::Playing);
}
