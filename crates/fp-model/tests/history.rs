#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback spec §2.2: R23 Restart, R24 Previous, R25 History.

mod common;

use common::{entries, fixture, p0};
use fp_model::{AppState, Command, EngineEvent, EntryId, PlayerId, apply, on_event};

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
