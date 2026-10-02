#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O37: the entry on air can be set as next and then plays
//! once more from its cue-in (a hard transition like a repeat, R26), as an
//! ordinary play.

mod common;

use common::{entries, fixture, p0};
use fp_model::{
    AppState, Command, EngineAction, EngineEvent, EntryId, PlayMode, PlayerId, TransitionPlan,
    Transport, apply, on_event, plan_for,
};

fn three() -> (AppState, PlayerId, [EntryId; 3]) {
    let state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    (state, p, [e[0], e[1], e[2]])
}

fn plan(s: &AppState, p: PlayerId) -> Option<TransitionPlan> {
    plan_for(s, s.player(p).unwrap())
}

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

fn started(s: &mut AppState, p: PlayerId, e: EntryId) -> Vec<EngineAction> {
    on_event(
        s,
        EngineEvent::TransitionStarted {
            player: p,
            entry: e,
        },
    )
}

/// Entry `a` is on air with itself as next.
fn replaying() -> (AppState, PlayerId, [EntryId; 3]) {
    let (mut s, p, e) = three();
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::SetNext(p, e[0])).unwrap();
    (s, p, e)
}

#[test]
fn o37_the_entry_on_air_can_be_set_as_next() {
    let (mut s, p, [a, _, _]) = three();
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::SetNext(p, a)).unwrap();
    let player = s.player(p).unwrap();
    assert_eq!(
        (player.current, player.next, player.next_explicit),
        (Some(a), Some(a), true)
    );
}

#[test]
fn o37_a_self_next_preloads_its_own_cue_in_and_plans_a_hard_transition_at_cue_out() {
    let (mut s, p, [a, _, _]) = three();
    apply(&mut s, Command::Play(p)).unwrap();
    let out = apply(&mut s, Command::SetNext(p, a)).unwrap();
    assert_eq!(preload(&out, p), Some(Some(a)), "{out:?}");
    assert_eq!(plan(&s, p), Some(HARD_END));
}

#[test]
fn o37_a_self_next_never_plans_a_segue() {
    let (mut s, p, [a, _, _]) = three();
    let t = s.track_for_entry(a).unwrap().id;
    // An automatic segue strictly inside the range would overlap the entry
    // with itself: the plan must stay the hard one at cue-out.
    apply(
        &mut s,
        Command::SetMarker {
            track: t,
            kind: fp_model::MarkerKind::SegueStart,
            secs: Some(170.0),
        },
    )
    .unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::SetNext(p, a)).unwrap();
    assert_eq!(plan(&s, p), Some(HARD_END));
}

#[test]
fn o37_the_second_pass_is_an_ordinary_play() {
    let (mut s, p, [a, b, _]) = replaying();
    let out = started(&mut s, p, a);
    let player = s.player(p).unwrap();
    assert_eq!(player.current, Some(a), "it is current again");
    assert!(
        s.playlists.entry(a).unwrap().played_by.contains(&p),
        "marked played"
    );
    assert_eq!(player.history, vec![a], "entered the history");
    assert_eq!(
        (player.next, player.next_explicit),
        (Some(b), false),
        "the next is worked out as usual"
    );
    assert_eq!(player.transport, Transport::Playing);
    assert_eq!(preload(&out, p), Some(Some(b)), "{out:?}");
    assert_eq!(
        plan(&s, p),
        Some(HARD_END),
        "and goes on with the following entry"
    );
}

#[test]
fn o37_it_acts_once_the_third_pass_never_comes() {
    let (mut s, p, [a, b, _]) = replaying();
    started(&mut s, p, a);
    started(&mut s, p, b);
    assert_eq!(s.player(p).unwrap().current, Some(b));
}

#[test]
fn the_replay_of_the_last_entry_leaves_no_next_and_stops_after_the_second_pass() {
    let (mut s, p, [_, _, c]) = three();
    apply(&mut s, Command::SetNext(p, c)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::SetNext(p, c)).unwrap();
    started(&mut s, p, c);
    assert_eq!(s.player(p).unwrap().next, None);
    assert_eq!(plan(&s, p), Some(TransitionPlan::StopAt { at_secs: 180.0 }));
    on_event(
        &mut s,
        EngineEvent::ReachedEnd {
            player: p,
            entry: c,
        },
    );
    assert_eq!(s.player(p).unwrap().transport, Transport::Stopped);
}

#[test]
fn o37_a_repeating_entry_keeps_repeating_and_the_self_next_waits() {
    let (mut s, p, [a, b, _]) = three();
    apply(&mut s, Command::ToggleEntryRepeat(a)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::SetNext(p, a)).unwrap();
    started(&mut s, p, a);
    let player = s.player(p).unwrap();
    assert_eq!(
        (player.next, player.next_explicit),
        (Some(a), true),
        "still waiting"
    );
    assert!(player.history.is_empty(), "a repeat pass is not recorded");
    // Repeat off: the next boundary is the replay, then the player goes on.
    apply(&mut s, Command::ToggleEntryRepeat(a)).unwrap();
    started(&mut s, p, a);
    let player = s.player(p).unwrap();
    assert_eq!((player.next, player.history.clone()), (Some(b), vec![a]));
}

#[test]
fn o37_next_while_on_air_restarts_the_entry_with_a_crossfade() {
    let (mut s, p, [a, b, _]) = replaying();
    let out = apply(&mut s, Command::Play(p)).unwrap();
    assert!(
        out.iter().any(
            |x| matches!(x, EngineAction::Crossfade { player, request, .. }
            if *player == p && request.entry == a && request.from_secs == 0.0)
        ),
        "{out:?}"
    );
    let player = s.player(p).unwrap();
    assert_eq!(
        (player.current, player.next, player.history.clone()),
        (Some(a), Some(b), vec![a])
    );
}

#[test]
fn o37_paused_with_a_self_next_resumes_without_a_replay() {
    let (mut s, p, [a, _, _]) = replaying();
    apply(&mut s, Command::Pause(p)).unwrap();
    let out = apply(&mut s, Command::Play(p)).unwrap();
    assert!(
        out.iter().any(|x| matches!(x, EngineAction::Resume { .. })),
        "{out:?}"
    );
    assert!(s.player(p).unwrap().history.is_empty());
    assert_eq!(s.player(p).unwrap().next, Some(a));
}

// Everything that already wins over the next entry still wins.

#[test]
fn o37_single_mode_stops_at_the_end_and_play_starts_the_entry_again() {
    let (mut s, p, [a, _, _]) = three();
    apply(&mut s, Command::SetMode(p, PlayMode::Single)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::SetNext(p, a)).unwrap();
    assert_eq!(plan(&s, p), Some(TransitionPlan::StopAt { at_secs: 180.0 }));
    on_event(
        &mut s,
        EngineEvent::ReachedEnd {
            player: p,
            entry: a,
        },
    );
    let player = s.player(p).unwrap();
    assert_eq!(
        (player.transport, player.next),
        (Transport::Stopped, Some(a))
    );
    let out = apply(&mut s, Command::Play(p)).unwrap();
    assert!(
        out.iter()
            .any(|x| matches!(x, EngineAction::StartCurrent { request, .. } if request.entry == a)),
        "{out:?}"
    );
}

#[test]
fn o37_stop_after_current_wins() {
    let (mut s, p, [a, _, _]) = replaying();
    apply(&mut s, Command::ToggleStopAfterCurrent(p)).unwrap();
    assert_eq!(plan(&s, p), Some(TransitionPlan::StopAt { at_secs: 180.0 }));
    on_event(
        &mut s,
        EngineEvent::ReachedEnd {
            player: p,
            entry: a,
        },
    );
    assert_eq!(s.player(p).unwrap().transport, Transport::Stopped);
}

#[test]
fn o37_an_entry_stop_after_wins() {
    let (mut s, p, [a, _, _]) = replaying();
    apply(&mut s, Command::ToggleEntryStopAfter(a)).unwrap();
    assert_eq!(plan(&s, p), Some(TransitionPlan::StopAt { at_secs: 180.0 }));
}

#[test]
fn o37_a_fade_stop_wins() {
    let (mut s, p, _) = replaying();
    apply(&mut s, Command::FadeStop(p)).unwrap();
    assert_eq!(plan(&s, p), None);
}
