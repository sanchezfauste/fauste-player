#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O12 (CUE window commands) and O17 (the CUE follows).

mod common;

use common::{entries, fixture, p0};
use fp_model::{AppState, Command, EngineAction, FileState, ModelError, PlayerId, apply};

/// A state whose player 0 is cueing its next entry (the first of three).
fn cueing() -> (AppState, PlayerId, Vec<fp_model::EntryId>) {
    let mut s = fixture(3);
    let p = p0(&s);
    let e = entries(&s);
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    (s, p, e)
}

fn cue(s: &AppState) -> Option<fp_model::CueState> {
    s.players[0].cue
}

#[test]
fn a_new_cue_starts_unpaused() {
    let (s, _, e) = cueing();
    let c = cue(&s).unwrap();
    assert_eq!(c.entry, e[0]);
    assert!(!c.paused);
}

#[test]
fn pausing_and_resuming_the_cue_tell_the_engine_once() {
    let (mut s, p, _) = cueing();
    let a = apply(&mut s, Command::SetCuePaused(p, true)).unwrap();
    assert_eq!(
        a,
        vec![EngineAction::SetCuePaused {
            player: p,
            paused: true
        }]
    );
    assert!(cue(&s).unwrap().paused);
    assert!(
        apply(&mut s, Command::SetCuePaused(p, true))
            .unwrap()
            .is_empty(),
        "idempotent"
    );
    let a = apply(&mut s, Command::SetCuePaused(p, false)).unwrap();
    assert_eq!(
        a,
        vec![EngineAction::SetCuePaused {
            player: p,
            paused: false
        }]
    );
    assert!(!cue(&s).unwrap().paused);
}

#[test]
fn commands_without_a_cue_do_nothing() {
    let mut s = fixture(3);
    let p = p0(&s);
    for command in [
        Command::SetCuePaused(p, true),
        Command::SeekCue(p, 30.0),
        Command::CueToNext(p),
    ] {
        assert_eq!(apply(&mut s, command).unwrap(), vec![]);
    }
    assert!(cue(&s).is_none());
    assert!(!s.players[0].next_explicit);
}

#[test]
fn commands_for_an_unknown_player_are_refused() {
    let mut s = fixture(1);
    let ghost = PlayerId(99);
    assert!(matches!(
        apply(&mut s, Command::SeekCue(ghost, 1.0)),
        Err(ModelError::UnknownPlayer(_))
    ));
    assert!(matches!(
        apply(&mut s, Command::SetCuePaused(ghost, true)),
        Err(ModelError::UnknownPlayer(_))
    ));
    assert!(matches!(
        apply(&mut s, Command::CueToNext(ghost)),
        Err(ModelError::UnknownPlayer(_))
    ));
}

#[test]
fn seeking_the_cue_asks_the_engine_for_the_position_clamped_to_the_file() {
    let (mut s, p, _) = cueing();
    for (asked, want) in [(42.5, 42.5), (500.0, 180.0), (-3.0, 0.0)] {
        let a = apply(&mut s, Command::SeekCue(p, asked)).unwrap();
        assert_eq!(
            a,
            vec![EngineAction::SeekCue {
                player: p,
                secs: want
            }],
            "asked {asked}"
        );
    }
}

#[test]
fn a_broken_seek_is_ignored() {
    let (mut s, p, _) = cueing();
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(apply(&mut s, Command::SeekCue(p, bad)).unwrap(), vec![]);
    }
}

#[test]
fn seeking_a_paused_cue_keeps_it_paused() {
    let (mut s, p, _) = cueing();
    apply(&mut s, Command::SetCuePaused(p, true)).unwrap();
    apply(&mut s, Command::SeekCue(p, 10.0)).unwrap();
    assert!(cue(&s).unwrap().paused, "the model does not resume it");
}

#[test]
fn load_as_next_makes_the_cued_entry_the_explicit_next_and_keeps_the_cue() {
    let (mut s, p, e) = cueing();
    // Cue the third entry, whose row is not the next.
    apply(&mut s, Command::CueEntry(p, e[2])).unwrap();
    assert_ne!(s.players[0].next, Some(e[2]));
    let a = apply(&mut s, Command::CueToNext(p)).unwrap();
    assert_eq!(s.players[0].next, Some(e[2]));
    assert!(s.players[0].next_explicit);
    assert_eq!(cue(&s).unwrap().entry, e[2], "the CUE keeps running");
    assert!(
        !a.iter().any(|a| matches!(
            a,
            EngineAction::StopCue { .. } | EngineAction::StartCue { .. }
        )),
        "{a:?}"
    );
}

#[test]
fn load_as_next_of_the_current_entry_is_refused() {
    let (mut s, p, e) = cueing();
    apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(s.players[0].current, Some(e[0]));
    apply(&mut s, Command::CueEntry(p, e[0])).unwrap();
    assert_eq!(
        apply(&mut s, Command::CueToNext(p)),
        Err(ModelError::NextIsCurrent)
    );
}

#[test]
fn setting_next_moves_a_running_cue_to_the_new_next_from_its_cue_in() {
    let (mut s, p, e) = cueing();
    let a = apply(&mut s, Command::SetNext(p, e[2])).unwrap();
    assert_eq!(cue(&s).unwrap().entry, e[2]);
    let start = a.iter().find_map(|a| match a {
        EngineAction::StartCue { player, request } if *player == p => Some(request),
        _ => None,
    });
    let request = start.expect("a StartCue");
    assert_eq!(request.entry, e[2]);
    assert_eq!(request.from_secs, 0.0, "the entry's cue-in (none set)");
}

#[test]
fn moving_a_paused_cue_plays_the_new_one() {
    let (mut s, p, e) = cueing();
    apply(&mut s, Command::SetCuePaused(p, true)).unwrap();
    apply(&mut s, Command::SetNext(p, e[1])).unwrap();
    let c = cue(&s).unwrap();
    assert_eq!(c.entry, e[1]);
    assert!(!c.paused);
}

#[test]
fn setting_the_cued_entry_as_next_keeps_it_running() {
    let (mut s, p, e) = cueing();
    let a = apply(&mut s, Command::SetNext(p, e[0])).unwrap();
    assert_eq!(cue(&s).unwrap().entry, e[0]);
    assert!(
        !a.iter().any(|a| matches!(a, EngineAction::StartCue { .. })),
        "no restart: {a:?}"
    );
}

#[test]
fn setting_next_without_a_cue_starts_none() {
    let mut s = fixture(3);
    let p = p0(&s);
    let e = entries(&s);
    let a = apply(&mut s, Command::SetNext(p, e[2])).unwrap();
    assert!(cue(&s).is_none());
    assert!(
        !a.iter().any(|a| matches!(a, EngineAction::StartCue { .. })),
        "{a:?}"
    );
}

#[test]
fn setting_next_to_an_unplayable_entry_leaves_the_cue() {
    let (mut s, p, e) = cueing();
    let track = s.playlists.entry(e[2]).unwrap().track;
    apply(
        &mut s,
        Command::SetFileState {
            track,
            state: FileState::Missing,
        },
    )
    .unwrap();
    let a = apply(&mut s, Command::SetNext(p, e[2])).unwrap();
    assert_eq!(s.players[0].next, Some(e[2]));
    assert_eq!(cue(&s).unwrap().entry, e[0], "the CUE stays where it was");
    assert!(
        !a.iter().any(|a| matches!(a, EngineAction::StartCue { .. })),
        "{a:?}"
    );
}
