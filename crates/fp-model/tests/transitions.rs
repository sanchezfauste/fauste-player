#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Spec §3 rules 9–12 and decode failures.

mod common;

use common::{entries, fixture, p0};
use fp_model::{
    AppState, Command, EngineAction, EngineEvent, FileState, MarkerKind, ModelError, PlayMode,
    PlayerId, TransitionPlan, Transport, apply, on_event,
};

fn with_segue(state: &mut AppState, secs: f64) {
    for t in state.library.iter_mut() {
        t.markers.set_auto(MarkerKind::SegueStart, Some(secs));
    }
}

/// The last plan scheduled for `p` in `actions`, if any.
fn scheduled(actions: &[EngineAction], p: PlayerId) -> Option<Option<TransitionPlan>> {
    actions.iter().rev().find_map(|a| match a {
        EngineAction::Schedule { player, plan } if *player == p => Some(*plan),
        _ => None,
    })
}

#[test]
fn rule11_continuous_with_segue_overlaps_at_segue_start() {
    let mut state = fixture(3);
    with_segue(&mut state, 172.0);
    let p = p0(&state);
    let actions = apply(&mut state, Command::Play(p)).unwrap();
    assert_eq!(
        scheduled(&actions, p),
        Some(Some(TransitionPlan::StartNextAt {
            at_secs: 172.0,
            fade_current_until_secs: Some(180.0)
        }))
    );
}

#[test]
fn rule11_without_segue_the_next_starts_at_cue_out() {
    let mut state = fixture(3);
    let p = p0(&state);
    let actions = apply(&mut state, Command::Play(p)).unwrap();
    assert_eq!(
        scheduled(&actions, p),
        Some(Some(TransitionPlan::StartNextAt {
            at_secs: 180.0,
            fade_current_until_secs: None
        }))
    );
}

#[test]
fn rule11_auto_segue_disabled_ignores_the_segue_marker() {
    let mut state = fixture(3);
    with_segue(&mut state, 172.0);
    state.config.players.auto_segue = false;
    let p = p0(&state);
    let actions = apply(&mut state, Command::Play(p)).unwrap();
    assert_eq!(
        scheduled(&actions, p),
        Some(Some(TransitionPlan::StartNextAt {
            at_secs: 180.0,
            fade_current_until_secs: None
        }))
    );
}

#[test]
fn rule10_single_mode_stops_at_cue_out() {
    let mut state = fixture(3);
    with_segue(&mut state, 172.0);
    let p = p0(&state);
    apply(&mut state, Command::SetMode(p, PlayMode::Single)).unwrap();
    let actions = apply(&mut state, Command::Play(p)).unwrap();
    assert_eq!(
        scheduled(&actions, p),
        Some(Some(TransitionPlan::StopAt { at_secs: 180.0 }))
    );
}

#[test]
fn rule11_last_entry_schedules_a_stop() {
    let mut state = fixture(1);
    let p = p0(&state);
    let actions = apply(&mut state, Command::Play(p)).unwrap();
    assert_eq!(
        scheduled(&actions, p),
        Some(Some(TransitionPlan::StopAt { at_secs: 180.0 }))
    );
}

#[test]
fn rule9_stop_after_current_is_refused_in_single_mode() {
    let mut state = fixture(3);
    let p = p0(&state);
    apply(&mut state, Command::SetMode(p, PlayMode::Single)).unwrap();
    assert_eq!(
        apply(&mut state, Command::ToggleStopAfterCurrent(p)),
        Err(ModelError::StopAfterInSingle)
    );
}

#[test]
fn rule9_switching_to_single_clears_stop_after_current() {
    let mut state = fixture(3);
    let p = p0(&state);
    apply(&mut state, Command::ToggleStopAfterCurrent(p)).unwrap();
    assert!(state.player(p).unwrap().stop_after_current);
    apply(&mut state, Command::SetMode(p, PlayMode::Single)).unwrap();
    assert!(!state.player(p).unwrap().stop_after_current);
}

#[test]
fn rule9_stop_after_current_reschedules_a_stop_and_keeps_next_green() {
    let mut state = fixture(3);
    with_segue(&mut state, 172.0);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    let actions = apply(&mut state, Command::ToggleStopAfterCurrent(p)).unwrap();
    assert_eq!(
        scheduled(&actions, p),
        Some(Some(TransitionPlan::StopAt { at_secs: 180.0 }))
    );
    // Rule 2: choosing another next keeps the flag.
    apply(&mut state, Command::SetNext(p, e[2])).unwrap();
    assert!(state.player(p).unwrap().stop_after_current);
    on_event(&mut state, EngineEvent::ReachedEnd { player: p });
    let player = state.player(p).unwrap();
    assert_eq!(
        (player.current, player.next, player.stop_after_current),
        (None, Some(e[2]), false)
    );
}

#[test]
fn rule11_transition_started_advances_marks_played_and_tracks_the_fade() {
    let mut state = fixture(3);
    with_segue(&mut state, 172.0);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    let actions = on_event(&mut state, EngineEvent::TransitionStarted { player: p });
    let player = state.player(p).unwrap();
    assert_eq!(
        (player.current, player.next, player.fading),
        (Some(e[1]), Some(e[2]), true)
    );
    assert!(state.playlists.entry(e[0]).unwrap().played);
    assert!(
        scheduled(&actions, p).is_some(),
        "a plan for the new current must be sent"
    );
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, EngineAction::StartCurrent { .. }))
    );
}

#[test]
fn rule11_transition_without_overlap_does_not_mark_fading() {
    let mut state = fixture(3);
    let p = p0(&state);
    apply(&mut state, Command::Play(p)).unwrap();
    on_event(&mut state, EngineEvent::TransitionStarted { player: p });
    assert!(!state.player(p).unwrap().fading);
}

#[test]
fn rule12_advance_skips_missing_entries() {
    let mut state = fixture(4);
    let (e, p) = (entries(&state), p0(&state));
    let missing = state.playlists.entry(e[1]).unwrap().track;
    state.library.get_mut(missing).unwrap().file_state = FileState::Missing;
    apply(&mut state, Command::Play(p)).unwrap();
    assert_eq!(state.player(p).unwrap().next, Some(e[2]));
}

#[test]
fn a_failing_current_source_skips_to_next_in_continuous_mode() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    let actions = on_event(
        &mut state,
        EngineEvent::SourceFailed {
            player: p,
            entry: e[0],
        },
    );
    assert!(
        matches!(actions.first(), Some(EngineAction::StartCurrent { request, .. }) if request.entry == e[1])
    );
    let failed = state.playlists.entry(e[0]).unwrap().track;
    assert_eq!(
        state.library.get(failed).unwrap().file_state,
        FileState::Unreadable
    );
}

#[test]
fn a_failing_current_source_stops_in_single_mode() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::SetMode(p, PlayMode::Single)).unwrap();
    apply(&mut state, Command::Play(p)).unwrap();
    let actions = on_event(
        &mut state,
        EngineEvent::SourceFailed {
            player: p,
            entry: e[0],
        },
    );
    assert_eq!(actions.first(), Some(&EngineAction::StopNow { player: p }));
    assert_eq!(state.player(p).unwrap().transport, Transport::Stopped);
}

#[test]
fn a_failing_next_source_moves_next_forward_on_every_player() {
    let mut state = fixture(3);
    let e = entries(&state);
    let p = p0(&state);
    on_event(
        &mut state,
        EngineEvent::SourceFailed {
            player: p,
            entry: e[0],
        },
    );
    assert!(state.players.iter().all(|pl| pl.next == Some(e[1])));
}

#[test]
fn playlist_of_unreadable_entries_never_starts() {
    let mut state = fixture(3);
    let e = entries(&state);
    for t in state.library.iter_mut() {
        t.file_state = FileState::Unreadable;
    }
    for pl in &mut state.players {
        pl.next = None;
    }
    let p = p0(&state);
    let actions = apply(&mut state, Command::Play(p)).unwrap();
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, EngineAction::StartCurrent { .. }))
    );
    assert_eq!(state.player(p).unwrap().transport, Transport::Stopped);
    assert_eq!(
        state.playlists.next_playable_after(e[0], &state.library),
        None
    );
    // Repeated failures terminate and leave the player stopped.
    for entry in &e {
        on_event(
            &mut state,
            EngineEvent::SourceFailed {
                player: p,
                entry: *entry,
            },
        );
    }
    assert_eq!(state.player(p).unwrap().transport, Transport::Stopped);
}

#[test]
fn stopping_cancels_the_schedule() {
    let mut state = fixture(3);
    let p = p0(&state);
    apply(&mut state, Command::Play(p)).unwrap();
    let actions = apply(&mut state, Command::Stop(p)).unwrap();
    assert_eq!(scheduled(&actions, p), Some(None));
}
