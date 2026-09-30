#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback spec §2: per-entry repeat (R26) and stop after (R27).

mod common;

use common::{entries, fixture, p0};
use fp_model::{
    AppState, Command, EngineAction, EngineEvent, EntryId, PlayMode, PlayerId, SOURCE_END,
    TransitionPlan, Transport, apply, on_event, plan_for,
};

fn three() -> (AppState, PlayerId, [EntryId; 3]) {
    let state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    (state, p, [e[0], e[1], e[2]])
}

fn entry(s: &AppState, e: EntryId) -> &fp_model::PlaylistEntry {
    s.playlists.entry(e).unwrap()
}

#[test]
fn the_toggles_flip_each_flag() {
    let (mut s, _, [a, _, _]) = three();
    assert!(!entry(&s, a).repeat && !entry(&s, a).stop_after);
    apply(&mut s, Command::ToggleEntryRepeat(a)).unwrap();
    apply(&mut s, Command::ToggleEntryStopAfter(a)).unwrap();
    assert!(entry(&s, a).repeat && entry(&s, a).stop_after);
    apply(&mut s, Command::ToggleEntryRepeat(a)).unwrap();
    assert!(!entry(&s, a).repeat);
    assert!(apply(&mut s, Command::ToggleEntryRepeat(EntryId(999_999))).is_err());
}

#[test]
fn duplicating_an_entry_copies_its_flags() {
    let (mut s, _, [a, _, _]) = three();
    apply(&mut s, Command::ToggleEntryRepeat(a)).unwrap();
    apply(&mut s, Command::ToggleEntryStopAfter(a)).unwrap();
    apply(&mut s, Command::DuplicateEntry(a)).unwrap();
    let copy = entries(&s)[1];
    assert_ne!(copy, a);
    assert!(entry(&s, copy).repeat && entry(&s, copy).stop_after);
}

#[test]
fn flags_are_saved_only_when_set_and_old_files_load_without_them() {
    let (mut s, _, [a, b, _]) = three();
    apply(&mut s, Command::ToggleEntryRepeat(a)).unwrap();
    let json = serde_json::to_value(entry(&s, a)).unwrap();
    assert_eq!(json["repeat"], true);
    assert!(json.get("stop_after").is_none());
    assert!(
        serde_json::to_value(entry(&s, b))
            .unwrap()
            .get("repeat")
            .is_none()
    );
    let old: fp_model::PlaylistEntry =
        serde_json::from_value(serde_json::json!({"id": 7, "track": 3})).unwrap();
    assert!(!old.repeat && !old.stop_after);
}

fn plan(s: &AppState, p: PlayerId) -> Option<TransitionPlan> {
    plan_for(s, s.player(p).unwrap())
}

/// The last `Preload` for `p` in `out`, if any: `Some(None)` drops it.
fn preload(out: &[EngineAction], p: PlayerId) -> Option<Option<EntryId>> {
    out.iter().rev().find_map(|a| match a {
        EngineAction::Preload { player, request } if *player == p => {
            Some(request.as_ref().map(|r| r.entry))
        }
        _ => None,
    })
}

const HARD_END: TransitionPlan = TransitionPlan::StartNextAt {
    at_secs: 180.0,
    fade_current_until_secs: None,
};

// R27

#[test]
fn r27_a_stop_after_entry_stops_in_continuous_mode() {
    let (mut s, p, [a, b, _]) = three();
    apply(&mut s, Command::ToggleEntryStopAfter(a)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(plan(&s, p), Some(TransitionPlan::StopAt { at_secs: 180.0 }));
    on_event(
        &mut s,
        EngineEvent::ReachedEnd {
            player: p,
            entry: a,
        },
    );
    let player = s.player(p).unwrap();
    assert_eq!(player.transport, Transport::Stopped);
    assert_eq!(player.next, Some(b));
    assert!(entry(&s, a).stop_after, "the flag stays for the next time");
}

#[test]
fn r27_a_stop_after_entry_stops_in_single_mode_and_wins_over_repeat() {
    let (mut s, p, [a, _, _]) = three();
    apply(&mut s, Command::SetMode(p, PlayMode::Single)).unwrap();
    apply(&mut s, Command::ToggleEntryStopAfter(a)).unwrap();
    apply(&mut s, Command::ToggleEntryRepeat(a)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(plan(&s, p), Some(TransitionPlan::StopAt { at_secs: 180.0 }));
}

// R26

#[test]
fn r26_a_repeating_entry_preloads_itself_and_restarts_at_cue_out() {
    for mode in [PlayMode::Continuous, PlayMode::Single] {
        let (mut s, p, [a, b, _]) = three();
        apply(&mut s, Command::SetMode(p, mode)).unwrap();
        apply(&mut s, Command::ToggleEntryRepeat(a)).unwrap();
        let out = apply(&mut s, Command::Play(p)).unwrap();
        assert_eq!(preload(&out, p), Some(Some(a)), "{mode:?}: {out:?}");
        assert_eq!(plan(&s, p), Some(HARD_END), "{mode:?}");
        assert_eq!(
            s.player(p).unwrap().next,
            Some(b),
            "the next line is unchanged"
        );
    }
}

#[test]
fn r26_each_pass_keeps_the_entry_and_plans_the_next_pass() {
    let (mut s, p, [a, _, _]) = three();
    apply(&mut s, Command::ToggleEntryRepeat(a)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    for _ in 0..3 {
        let out = on_event(
            &mut s,
            EngineEvent::TransitionStarted {
                player: p,
                entry: a,
            },
        );
        let player = s.player(p).unwrap();
        assert_eq!(player.current, Some(a));
        assert!(
            !entry(&s, a).played_by.contains(&p),
            "not played while repeating"
        );
        assert!(player.history.is_empty());
        assert_eq!(preload(&out, p), Some(Some(a)), "{out:?}");
        assert!(
            out.iter().any(|x| matches!(x,
            EngineAction::Schedule { player, plan: Some(pl) } if *player == p && *pl == HARD_END)),
            "{out:?}"
        );
    }
}

#[test]
fn r26_pause_keeps_the_repeat() {
    let (mut s, p, [a, _, _]) = three();
    apply(&mut s, Command::ToggleEntryRepeat(a)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::Pause(p)).unwrap();
    apply(&mut s, Command::Pause(p)).unwrap();
    assert_eq!(plan(&s, p), Some(HARD_END));
}

#[test]
fn r26_next_ends_the_repeat_and_marks_the_entry_played() {
    let (mut s, p, [a, b, _]) = three();
    apply(&mut s, Command::ToggleEntryRepeat(a)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(s.player(p).unwrap().current, Some(b));
    assert!(entry(&s, a).played_by.contains(&p));
    assert!(entry(&s, a).repeat, "the flag stays");
}

#[test]
fn r26_stop_after_current_ends_the_pass() {
    let (mut s, p, [a, _, _]) = three();
    apply(&mut s, Command::ToggleEntryRepeat(a)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    let out = apply(&mut s, Command::ToggleStopAfterCurrent(p)).unwrap();
    assert_eq!(plan(&s, p), Some(TransitionPlan::StopAt { at_secs: 180.0 }));
    assert_eq!(preload(&out, p), Some(s.player(p).unwrap().next), "{out:?}");
}

#[test]
fn r26_toggling_repeat_while_playing_re_plans() {
    let (mut s, p, [a, b, _]) = three();
    apply(&mut s, Command::Play(p)).unwrap();
    let out = apply(&mut s, Command::ToggleEntryRepeat(a)).unwrap();
    assert_eq!(preload(&out, p), Some(Some(a)));
    assert_eq!(plan(&s, p), Some(HARD_END));
    let out = apply(&mut s, Command::ToggleEntryRepeat(a)).unwrap();
    assert_eq!(preload(&out, p), Some(Some(b)));
}

#[test]
fn r26_an_entry_of_unknown_length_restarts_at_the_source_end() {
    let (mut s, p, [a, _, _]) = three();
    for t in s.library.iter_mut() {
        t.duration_secs = 0.0;
    }
    apply(&mut s, Command::ToggleEntryRepeat(a)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(
        plan(&s, p),
        Some(TransitionPlan::StartNextAt {
            at_secs: SOURCE_END,
            fade_current_until_secs: None
        })
    );
}

#[test]
fn r26_two_players_repeat_the_same_entry_independently() {
    let (mut s, p, [a, _, _]) = three();
    let q = s.players[1].id;
    apply(&mut s, Command::ToggleEntryRepeat(a)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::Play(q)).unwrap();
    let out = on_event(
        &mut s,
        EngineEvent::TransitionStarted {
            player: p,
            entry: a,
        },
    );
    assert_eq!(preload(&out, p), Some(Some(a)));
    assert_eq!(preload(&out, q), None, "the other player is untouched");
    assert_eq!(s.player(q).unwrap().current, Some(a));
}
