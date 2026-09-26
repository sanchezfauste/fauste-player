#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Spec §3 rules 2–8: next selection, play, pause, stop, fade stop.

mod common;

use common::{entries, fixture, p0};
use fp_model::{Command, EngineAction, EngineEvent, ModelError, Transport, apply, on_event};

#[test]
fn inserting_into_an_empty_playlist_marks_the_first_entry_as_next() {
    let state = fixture(3);
    let e = entries(&state);
    assert!(state.players.iter().all(|p| p.next == Some(e[0])));
}

#[test]
fn rule2_double_click_sets_next_but_never_the_current_entry() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::SetNext(p, e[2])).unwrap();
    assert_eq!(state.player(p).unwrap().next, Some(e[2]));
    apply(&mut state, Command::Play(p)).unwrap();
    assert_eq!(
        apply(&mut state, Command::SetNext(p, e[2])),
        Err(ModelError::NextIsCurrent)
    );
}

#[test]
fn rule3_play_while_stopped_starts_next_and_advances() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    let actions = apply(&mut state, Command::Play(p)).unwrap();
    let player = state.player(p).unwrap();
    assert_eq!(player.current, Some(e[0]));
    assert_eq!(player.next, Some(e[1]));
    assert_eq!(player.transport, Transport::Playing);
    assert!(matches!(
        actions.first(),
        Some(EngineAction::StartCurrent { player, request }) if *player == p && request.entry == e[0] && request.from_secs == 0.0
    ));
    assert!(actions.iter().any(
        |a| matches!(a, EngineAction::Preload { player, request: Some(r) } if *player == p && r.entry == e[1])
    ));
}

#[test]
fn rule3_play_without_next_does_nothing() {
    let mut state = fixture(0);
    let p = p0(&state);
    assert!(apply(&mut state, Command::Play(p)).unwrap().is_empty());
    assert_eq!(state.player(p).unwrap().transport, Transport::Stopped);
}

#[test]
fn rule4_play_while_paused_resumes() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    apply(&mut state, Command::Pause(p)).unwrap();
    let actions = apply(&mut state, Command::Play(p)).unwrap();
    assert_eq!(actions, vec![EngineAction::Resume { player: p }]);
    assert_eq!(state.player(p).unwrap().current, Some(e[0]));
    assert_eq!(state.player(p).unwrap().transport, Transport::Playing);
}

#[test]
fn rule5_play_while_playing_crossfades_into_next() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    let actions = apply(&mut state, Command::Play(p)).unwrap();
    assert!(matches!(
        actions.first(),
        Some(EngineAction::Crossfade { player, request, fade_ms: 1000 }) if *player == p && request.entry == e[1]
    ));
    let player = state.player(p).unwrap();
    assert_eq!(
        (player.current, player.next, player.fading),
        (Some(e[1]), Some(e[2]), true)
    );
    assert!(state.playlists.entry(e[0]).unwrap().played);
}

#[test]
fn rule5_play_during_a_fade_is_ignored_until_the_fade_completes() {
    let mut state = fixture(3);
    let p = p0(&state);
    apply(&mut state, Command::Play(p)).unwrap();
    apply(&mut state, Command::Play(p)).unwrap();
    assert!(apply(&mut state, Command::Play(p)).unwrap().is_empty());
    on_event(&mut state, EngineEvent::FadeCompleted { player: p });
    assert!(!state.player(p).unwrap().fading);
}

#[test]
fn rule6_pause_toggles_and_is_ignored_during_a_fade() {
    let mut state = fixture(3);
    let p = p0(&state);
    apply(&mut state, Command::Play(p)).unwrap();
    assert_eq!(
        apply(&mut state, Command::Pause(p)).unwrap(),
        vec![EngineAction::Pause { player: p }]
    );
    assert_eq!(state.player(p).unwrap().transport, Transport::Paused);
    assert_eq!(
        apply(&mut state, Command::Pause(p)).unwrap(),
        vec![EngineAction::Resume { player: p }]
    );
    apply(&mut state, Command::Play(p)).unwrap(); // crossfade → fading
    assert!(apply(&mut state, Command::Pause(p)).unwrap().is_empty());
}

#[test]
fn rule7_stop_marks_played_and_keeps_the_green_next() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    let actions = apply(&mut state, Command::Stop(p)).unwrap();
    assert_eq!(actions.first(), Some(&EngineAction::StopNow { player: p }));
    let player = state.player(p).unwrap();
    assert_eq!(
        (player.current, player.next, player.transport),
        (None, Some(e[1]), Transport::Stopped)
    );
    assert!(state.playlists.entry(e[0]).unwrap().played);
}

#[test]
fn rule7_stop_keeps_an_explicitly_chosen_next() {
    let mut state = fixture(4);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    apply(&mut state, Command::SetNext(p, e[3])).unwrap();
    apply(&mut state, Command::Stop(p)).unwrap();
    assert_eq!(state.player(p).unwrap().next, Some(e[3]));
}

#[test]
fn rule7_stopping_an_idle_player_does_nothing() {
    let mut state = fixture(1);
    let p = p0(&state);
    assert!(apply(&mut state, Command::Stop(p)).unwrap().is_empty());
}

#[test]
fn rule8_fade_stop_fades_then_stops_when_the_engine_reports_the_end() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    let actions = apply(&mut state, Command::FadeStop(p)).unwrap();
    assert_eq!(
        actions.first(),
        Some(&EngineAction::FadeOutAndStop {
            player: p,
            fade_ms: 1000
        })
    );
    assert!(state.player(p).unwrap().fading);
    on_event(
        &mut state,
        EngineEvent::ReachedEnd {
            player: p,
            entry: e[0],
        },
    );
    let player = state.player(p).unwrap();
    assert_eq!(
        (player.current, player.next, player.fading),
        (None, Some(e[1]), false)
    );
}

#[test]
fn unknown_player_is_refused() {
    let mut state = fixture(1);
    let ghost = fp_model::PlayerId(9999);
    assert_eq!(
        apply(&mut state, Command::Play(ghost)),
        Err(ModelError::UnknownPlayer(ghost))
    );
}

#[test]
fn a_fade_stop_is_told_apart_from_a_crossfade() {
    let mut state = fixture(3);
    let p = p0(&state);
    apply(&mut state, Command::Play(p)).unwrap();
    apply(&mut state, Command::Play(p)).unwrap();
    assert!(state.player(p).unwrap().fading);
    assert!(!state.player(p).unwrap().fade_stopping(), "a crossfade");
    let mut state = fixture(3);
    apply(&mut state, Command::Play(p)).unwrap();
    apply(&mut state, Command::FadeStop(p)).unwrap();
    assert!(state.player(p).unwrap().fade_stopping());
}

#[test]
fn rule22_an_idle_player_does_not_pick_an_entry_on_air_elsewhere() {
    let mut state = fixture(3);
    let e = entries(&state);
    let (p1, p2) = (state.players[0].id, state.players[1].id);
    apply(&mut state, Command::Play(p1)).unwrap();
    assert_eq!(state.player(p1).unwrap().current, Some(e[0]));
    assert_eq!(state.player(p2).unwrap().next, Some(e[1]));
}

#[test]
fn rule22_derived_next_skips_entries_on_air_elsewhere() {
    let mut state = fixture(4);
    let e = entries(&state);
    let (p1, p2) = (state.players[0].id, state.players[1].id);
    apply(&mut state, Command::SetNext(p1, e[1])).unwrap();
    apply(&mut state, Command::Play(p1)).unwrap();
    apply(&mut state, Command::SetNext(p2, e[0])).unwrap();
    apply(&mut state, Command::Play(p2)).unwrap();
    assert_eq!(state.player(p2).unwrap().current, Some(e[0]));
    assert_eq!(
        state.player(p2).unwrap().next,
        Some(e[2]),
        "e[1] is on air on P1"
    );
}

#[test]
fn rule22_an_explicit_next_is_kept_even_if_on_air_elsewhere() {
    let mut state = fixture(3);
    let e = entries(&state);
    let (p1, p2) = (state.players[0].id, state.players[1].id);
    apply(&mut state, Command::SetNext(p2, e[0])).unwrap();
    apply(&mut state, Command::Play(p1)).unwrap();
    assert_eq!(state.player(p2).unwrap().next, Some(e[0]));
}
