#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Regressions found by the whole-branch review of Phase 1 plan 1.

mod common;

use std::path::PathBuf;

use common::{entries, fixture, p0};
use fp_model::{
    AppState, Command, Config, EngineAction, EngineEvent, MarkerKind, PlayMode, PlayerId,
    RestoreParts, SOURCE_END, TransitionPlan, Transport, apply, on_event,
};

fn parts(state: &AppState) -> RestoreParts {
    RestoreParts {
        config: state.config.clone(),
        library: state.library.clone(),
        playlists: state.playlists.clone(),
        cart_pages: state.cartwall.pages.clone(),
        ids: state.ids.clone(),
    }
}

fn scheduled(actions: &[EngineAction], p: PlayerId) -> Option<Option<TransitionPlan>> {
    actions.iter().rev().find_map(|a| match a {
        EngineAction::Schedule { player, plan } if *player == p => Some(*plan),
        _ => None,
    })
}

fn starts(actions: &[EngineAction]) -> bool {
    actions
        .iter()
        .any(|a| matches!(a, EngineAction::StartCurrent { .. }))
}

// C1 — nothing goes on air by itself.

#[test]
fn restored_player_whose_source_fails_never_goes_on_air() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    let (mut restored, _) = AppState::restore(parts(&state), &state.sessions(|_| 10.0), "Main");
    let actions = on_event(
        &mut restored,
        EngineEvent::SourceFailed {
            player: p,
            entry: e[0],
        },
    );
    assert!(!starts(&actions));
    assert_ne!(restored.player(p).unwrap().transport, Transport::Playing);
}

#[test]
fn paused_player_whose_source_fails_stops_instead_of_skipping() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    apply(&mut state, Command::Pause(p)).unwrap();
    let actions = on_event(
        &mut state,
        EngineEvent::SourceFailed {
            player: p,
            entry: e[0],
        },
    );
    assert!(!starts(&actions));
    assert_eq!(state.player(p).unwrap().transport, Transport::Stopped);
    assert_eq!(state.player(p).unwrap().next, Some(e[1]));
}

// I1 — next is never the player's own current.

#[test]
fn removing_the_next_never_makes_next_the_current() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::SetNext(p, e[1])).unwrap();
    apply(&mut state, Command::Play(p)).unwrap();
    apply(&mut state, Command::SetNext(p, e[0])).unwrap();
    apply(&mut state, Command::RemoveEntry(e[0])).unwrap();
    assert_eq!(state.player(p).unwrap().current, Some(e[1]));
    assert_eq!(state.player(p).unwrap().next, Some(e[2]));
}

#[test]
fn a_failing_next_never_makes_next_the_current() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::SetNext(p, e[1])).unwrap();
    apply(&mut state, Command::Play(p)).unwrap();
    apply(&mut state, Command::SetNext(p, e[0])).unwrap();
    on_event(
        &mut state,
        EngineEvent::SourceFailed {
            player: p,
            entry: e[0],
        },
    );
    assert_eq!(state.player(p).unwrap().next, Some(e[2]));
}

// I2 — derived next follows playlist edits; explicit next is kept.

#[test]
fn appending_while_on_the_last_entry_sets_next_and_reschedules() {
    let mut state = fixture(1);
    let p = p0(&state);
    let playlist = state.playlists.first_id().unwrap();
    apply(&mut state, Command::Play(p)).unwrap();
    assert_eq!(state.player(p).unwrap().next, None);
    let actions = apply(
        &mut state,
        Command::InsertPaths {
            playlist,
            index: 99,
            paths: vec![PathBuf::from("/music/late.flac")],
        },
    )
    .unwrap();
    let added = entries(&state)[1];
    assert_eq!(state.player(p).unwrap().next, Some(added));
    assert!(matches!(
        scheduled(&actions, p),
        Some(Some(TransitionPlan::StartNextAt { .. }))
    ));
}

#[test]
fn a_derived_next_follows_an_insert_right_after_current() {
    let mut state = fixture(3);
    let p = p0(&state);
    let playlist = state.playlists.first_id().unwrap();
    apply(&mut state, Command::Play(p)).unwrap();
    apply(
        &mut state,
        Command::InsertPaths {
            playlist,
            index: 1,
            paths: vec![PathBuf::from("/music/inserted.flac")],
        },
    )
    .unwrap();
    let inserted = entries(&state)[1];
    assert_eq!(state.player(p).unwrap().next, Some(inserted));
    apply(&mut state, Command::Stop(p)).unwrap();
    assert_eq!(state.player(p).unwrap().next, Some(inserted));
}

#[test]
fn an_explicit_next_survives_playlist_edits() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    let playlist = state.playlists.first_id().unwrap();
    apply(&mut state, Command::Play(p)).unwrap();
    apply(&mut state, Command::SetNext(p, e[2])).unwrap();
    apply(
        &mut state,
        Command::InsertPaths {
            playlist,
            index: 1,
            paths: vec![PathBuf::from("/music/inserted.flac")],
        },
    )
    .unwrap();
    assert_eq!(state.player(p).unwrap().next, Some(e[2]));
}

// I3 — unknown durations never schedule a transition at 0 s.

#[test]
fn unanalysed_tracks_transition_at_the_end_of_the_source() {
    let mut state = AppState::new(Config::default(), "Main");
    let playlist = state.playlists.first_id().unwrap();
    let paths = vec![PathBuf::from("/m/a.flac"), PathBuf::from("/m/b.flac")];
    apply(
        &mut state,
        Command::InsertPaths {
            playlist,
            index: 0,
            paths,
        },
    )
    .unwrap();
    let p = state.players[0].id;
    let actions = apply(&mut state, Command::Play(p)).unwrap();
    assert_eq!(
        scheduled(&actions, p),
        Some(Some(TransitionPlan::StartNextAt {
            at_secs: SOURCE_END,
            fade_current_until_secs: None
        }))
    );
    let actions = apply(&mut state, Command::Seek(p, 999.0)).unwrap();
    assert_eq!(
        actions,
        vec![EngineAction::Seek {
            player: p,
            secs: 999.0
        }]
    );
    let actions = apply(&mut state, Command::SetMode(p, PlayMode::Single)).unwrap();
    assert_eq!(
        scheduled(&actions, p),
        Some(Some(TransitionPlan::StopAt {
            at_secs: SOURCE_END
        }))
    );
}

// I5 — fade stop cancels the transition and works during an overlap.

fn with_segue(state: &mut AppState, secs: f64) {
    for t in state.library.iter_mut() {
        t.markers.set_auto(MarkerKind::SegueStart, Some(secs));
    }
}

#[test]
fn fade_stop_cancels_the_scheduled_transition() {
    let mut state = fixture(3);
    with_segue(&mut state, 172.0);
    let p = p0(&state);
    apply(&mut state, Command::Play(p)).unwrap();
    let actions = apply(&mut state, Command::FadeStop(p)).unwrap();
    assert_eq!(scheduled(&actions, p), Some(None));
}

#[test]
fn fade_stop_works_during_a_segue_overlap() {
    let mut state = fixture(3);
    with_segue(&mut state, 172.0);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    on_event(
        &mut state,
        EngineEvent::TransitionStarted {
            player: p,
            entry: e[1],
        },
    );
    assert!(state.player(p).unwrap().fading);
    let actions = apply(&mut state, Command::FadeStop(p)).unwrap();
    assert!(actions.contains(&EngineAction::FadeOutAndStop {
        player: p,
        fade_ms: 1000
    }));
}

// I7 — engine events name the entry and stale ones are ignored.

#[test]
fn a_stale_reached_end_is_ignored() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    apply(&mut state, Command::Stop(p)).unwrap();
    apply(&mut state, Command::Play(p)).unwrap();
    on_event(
        &mut state,
        EngineEvent::ReachedEnd {
            player: p,
            entry: e[0],
        },
    );
    let player = state.player(p).unwrap();
    assert_eq!(
        (player.current, player.transport),
        (Some(e[1]), Transport::Playing)
    );
}

#[test]
fn a_transition_follows_what_the_engine_actually_started() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    apply(&mut state, Command::SetNext(p, e[2])).unwrap();
    // The engine had already started the old preload (e1) when the new next arrived.
    on_event(
        &mut state,
        EngineEvent::TransitionStarted {
            player: p,
            entry: e[1],
        },
    );
    let player = state.player(p).unwrap();
    assert_eq!((player.current, player.next), (Some(e[1]), Some(e[2])));
    assert!(state.playlists.entry(e[0]).unwrap().played);
}

// Restore gaps (re-graded from minor).

#[test]
fn restore_sends_the_restored_volume_to_the_engine() {
    let mut state = fixture(2);
    let p = p0(&state);
    apply(&mut state, Command::SetVolume(p, 0.5)).unwrap();
    let (_, actions) = AppState::restore(parts(&state), &state.sessions(|_| 0.0), "Main");
    assert!(actions.contains(&EngineAction::SetVolume {
        player: p,
        volume: 0.5
    }));
}

#[test]
fn restore_drops_a_current_entry_whose_track_is_gone() {
    let mut state = fixture(2);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    let sessions = state.sessions(|_| 5.0);
    let mut broken = parts(&state);
    let track = state.playlists.entry(e[0]).unwrap().track;
    broken.library.remove(track);
    let (restored, _) = AppState::restore(broken, &sessions, "Main");
    let player = restored.player(p).unwrap();
    assert_eq!(
        (player.current, player.transport),
        (None, Transport::Stopped)
    );
}

// Plan 2 review — a transition the engine reports after the model stopped is stale.
#[test]
fn a_transition_reported_after_stop_does_not_restart_the_player() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    apply(&mut state, Command::Stop(p)).unwrap();
    let actions = on_event(
        &mut state,
        EngineEvent::TransitionStarted {
            player: p,
            entry: e[1],
        },
    );
    assert_eq!(state.player(p).unwrap().transport, Transport::Stopped);
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, EngineAction::StartCurrent { .. }))
    );
}
