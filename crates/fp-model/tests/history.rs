#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback spec §2.2: R23 Restart, R24 Previous, R25 History.

mod common;

use common::{entries, fixture, p0};
use fp_model::{
    AppState, Command, EngineAction, EngineEvent, EntryId, PlayerId, Transport, apply, on_event,
};

/// One player's view of a fixture with three entries A, B and C.
fn three() -> (AppState, PlayerId, [EntryId; 3]) {
    let state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    (state, p, [e[0], e[1], e[2]])
}

/// Play (or Next) and let the crossfade finish.
fn next(state: &mut AppState, p: PlayerId) {
    apply(state, Command::Play(p)).unwrap();
    on_event(state, EngineEvent::FadeCompleted { player: p });
}

fn history(state: &AppState, p: PlayerId) -> Vec<EntryId> {
    state.player(p).unwrap().history.clone()
}

#[test]
fn r25_every_advance_records_the_entry_that_was_left() {
    let (mut s, p, [a, b, c]) = three();
    next(&mut s, p);
    assert!(history(&s, p).is_empty());
    next(&mut s, p);
    assert_eq!(history(&s, p), vec![a]);
    on_event(
        &mut s,
        EngineEvent::TransitionStarted {
            player: p,
            entry: c,
        },
    );
    assert_eq!(history(&s, p), vec![a, b]);
}

#[test]
fn r25_a_stop_records_the_entry_that_stopped() {
    let (mut s, p, [a, _, _]) = three();
    next(&mut s, p);
    apply(&mut s, Command::Stop(p)).unwrap();
    assert_eq!(history(&s, p), vec![a]);
}

#[test]
fn r25_the_history_keeps_only_the_configured_length() {
    let (mut s, p, [_, b, c]) = three();
    s.config.players.history_len = 2;
    for _ in 0..3 {
        next(&mut s, p);
    }
    // A and B were left; C is current. One more stop leaves C too.
    apply(&mut s, Command::Stop(p)).unwrap();
    assert_eq!(history(&s, p), vec![b, c]);
}

#[test]
fn r25_a_zero_length_history_records_nothing() {
    let (mut s, p, _) = three();
    s.config.players.history_len = 0;
    next(&mut s, p);
    next(&mut s, p);
    assert!(history(&s, p).is_empty());
}

#[test]
fn r23_restart_seeks_to_cue_in_while_playing() {
    let (mut s, p, [a, _, _]) = three();
    next(&mut s, p);
    let cue_in = s.request_from_cue_in(a).unwrap().from_secs;
    let out = apply(&mut s, Command::Restart(p)).unwrap();
    assert_eq!(
        out,
        vec![EngineAction::Seek {
            player: p,
            secs: cue_in
        }]
    );
    assert_eq!(s.player(p).unwrap().transport, Transport::Playing);
    assert_eq!(s.player(p).unwrap().current, Some(a));
}

#[test]
fn r23_a_paused_player_restarts_paused() {
    let (mut s, p, _) = three();
    next(&mut s, p);
    apply(&mut s, Command::Pause(p)).unwrap();
    let out = apply(&mut s, Command::Restart(p)).unwrap();
    assert!(
        matches!(out.as_slice(), [EngineAction::Seek { .. }]),
        "{out:?}"
    );
    assert_eq!(s.player(p).unwrap().transport, Transport::Paused);
}

#[test]
fn r23_restart_does_nothing_when_stopped() {
    let (mut s, p, _) = three();
    assert!(apply(&mut s, Command::Restart(p)).unwrap().is_empty());
    next(&mut s, p);
    apply(&mut s, Command::Stop(p)).unwrap();
    assert!(apply(&mut s, Command::Restart(p)).unwrap().is_empty());
}

/// A played, then B: history [A], B current.
fn playing_b() -> (AppState, PlayerId, [EntryId; 3]) {
    let (mut s, p, e) = three();
    next(&mut s, p);
    next(&mut s, p);
    (s, p, e)
}

#[test]
fn r24_previous_crossfades_back_and_queues_the_entry_it_left() {
    let (mut s, p, [a, b, _]) = playing_b();
    let fade_ms = s.config.players.fade_ms;
    let out = apply(&mut s, Command::Previous(p)).unwrap();
    assert!(
        out.iter().any(|x| matches!(x,
            EngineAction::Crossfade { player, request, fade_ms: f }
                if *player == p && request.entry == a && *f == fade_ms)),
        "{out:?}"
    );
    let player = s.player(p).unwrap();
    assert_eq!(player.current, Some(a));
    assert_eq!(player.next, Some(b));
    assert!(player.next_explicit);
    assert!(player.fading);
    assert!(player.history.is_empty(), "B is not recorded");
    assert!(
        s.playlists.entry(b).unwrap().played_by.contains(&p),
        "B counts as played"
    );
}

#[test]
fn r24_repeated_previous_keeps_going_back() {
    let (mut s, p, [a, b, c]) = three();
    for _ in 0..3 {
        next(&mut s, p);
    }
    assert_eq!(s.player(p).unwrap().current, Some(c));
    apply(&mut s, Command::Previous(p)).unwrap();
    on_event(&mut s, EngineEvent::FadeCompleted { player: p });
    assert_eq!(s.player(p).unwrap().current, Some(b));
    apply(&mut s, Command::Previous(p)).unwrap();
    on_event(&mut s, EngineEvent::FadeCompleted { player: p });
    assert_eq!(s.player(p).unwrap().current, Some(a));
}

#[test]
fn r24_gone_entries_are_skipped_and_discarded() {
    let (mut s, p, [a, b, c]) = three();
    for _ in 0..3 {
        next(&mut s, p);
    }
    assert_eq!(history(&s, p), vec![a, b]);
    apply(&mut s, Command::RemoveEntry(b)).unwrap();
    apply(&mut s, Command::Previous(p)).unwrap();
    assert_eq!(s.player(p).unwrap().current, Some(a));
    assert!(history(&s, p).is_empty());
    assert_eq!(s.player(p).unwrap().next, Some(c));
}

#[test]
fn r24_previous_with_nothing_playable_left_does_nothing() {
    let (mut s, p, [a, b, _]) = playing_b();
    apply(&mut s, Command::RemoveEntry(a)).unwrap();
    assert!(apply(&mut s, Command::Previous(p)).unwrap().is_empty());
    assert_eq!(s.player(p).unwrap().current, Some(b));
}

#[test]
fn r24_previous_does_nothing_stopped_paused_or_fading() {
    let (mut s, p, _) = three();
    assert!(
        apply(&mut s, Command::Previous(p)).unwrap().is_empty(),
        "stopped"
    );
    next(&mut s, p);
    assert!(
        apply(&mut s, Command::Previous(p)).unwrap().is_empty(),
        "no history"
    );
    let (mut s, p, _) = playing_b();
    apply(&mut s, Command::Pause(p)).unwrap();
    assert!(
        apply(&mut s, Command::Previous(p)).unwrap().is_empty(),
        "paused"
    );
    let (mut s, p, _) = playing_b();
    apply(&mut s, Command::Play(p)).unwrap();
    assert!(
        apply(&mut s, Command::Previous(p)).unwrap().is_empty(),
        "crossfading"
    );
    let (mut s, p, _) = playing_b();
    apply(&mut s, Command::FadeStop(p)).unwrap();
    assert!(
        apply(&mut s, Command::Previous(p)).unwrap().is_empty(),
        "fade stop"
    );
}
