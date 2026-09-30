#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback spec R28: which transport actions make sense now.

mod common;

use common::{entries, fixture, p0};
use fp_model::{
    AppState, Availability, Command, EngineEvent, PlayMode, PlayerId, apply, availability,
    command_available, on_event,
};

fn three() -> (AppState, PlayerId) {
    let state = fixture(3);
    let p = p0(&state);
    (state, p)
}

fn next(state: &mut AppState, p: PlayerId) {
    apply(state, Command::Play(p)).unwrap();
    on_event(state, EngineEvent::FadeCompleted { player: p });
}

#[test]
fn a_stopped_player_with_a_next_can_play_cue_and_arm_stop_after() {
    let (s, p) = three();
    assert_eq!(
        availability(&s, p),
        Availability {
            play: true,
            cue: true,
            stop_after_current: true,
            ..Availability::default()
        }
    );
}

#[test]
fn an_empty_playlist_offers_only_stop_after() {
    let s = fixture(0);
    let a = availability(&s, p0(&s));
    assert_eq!(
        a,
        Availability {
            stop_after_current: true,
            ..Availability::default()
        }
    );
}

#[test]
fn a_playing_player_without_history_cannot_go_back() {
    let (mut s, p) = three();
    next(&mut s, p);
    let a = availability(&s, p);
    assert!(a.play && a.pause && a.stop && a.fade_stop && a.restart && a.cue);
    assert!(!a.previous);
}

#[test]
fn previous_needs_a_playable_history_entry() {
    let (mut s, p) = three();
    next(&mut s, p);
    next(&mut s, p);
    assert!(availability(&s, p).previous);
    let first = entries(&s)[0];
    apply(&mut s, Command::RemoveEntry(first)).unwrap();
    assert!(!availability(&s, p).previous);
}

#[test]
fn a_zero_length_history_never_offers_previous() {
    let (mut s, p) = three();
    s.config.players.history_len = 0;
    next(&mut s, p);
    next(&mut s, p);
    assert!(!availability(&s, p).previous);
}

#[test]
fn during_a_crossfade_play_pause_and_previous_wait() {
    let (mut s, p) = three();
    next(&mut s, p);
    next(&mut s, p);
    apply(&mut s, Command::Play(p)).unwrap();
    let a = availability(&s, p);
    assert!(!a.play && !a.pause && !a.previous);
    assert!(a.stop && a.fade_stop && a.restart);
}

#[test]
fn during_a_fade_stop_fade_stop_is_unavailable_but_stop_is_not() {
    let (mut s, p) = three();
    next(&mut s, p);
    apply(&mut s, Command::FadeStop(p)).unwrap();
    let a = availability(&s, p);
    assert!(!a.fade_stop && !a.pause);
    assert!(a.stop);
}

#[test]
fn a_paused_player_can_resume_restart_and_stop_but_not_fade_or_go_back() {
    let (mut s, p) = three();
    next(&mut s, p);
    next(&mut s, p);
    apply(&mut s, Command::Pause(p)).unwrap();
    let a = availability(&s, p);
    assert!(a.play && a.pause && a.restart && a.stop);
    assert!(!a.fade_stop && !a.previous);
}

#[test]
fn stop_after_current_needs_continuous_mode() {
    let (mut s, p) = three();
    apply(&mut s, Command::SetMode(p, PlayMode::Single)).unwrap();
    assert!(!availability(&s, p).stop_after_current);
}

#[test]
fn a_running_cue_can_always_be_stopped() {
    let (mut s, p) = three();
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    assert!(s.player(p).unwrap().cue.is_some());
    let i = s.players.iter().position(|x| x.id == p).unwrap();
    s.players[i].next = None;
    assert!(availability(&s, p).cue);
    s.players[i].cue = None;
    assert!(!availability(&s, p).cue);
}

#[test]
fn command_available_filters_transport_commands_only() {
    let (s, p) = three();
    assert!(!command_available(&s, &Command::Stop(p)));
    assert!(!command_available(&s, &Command::Previous(p)));
    assert!(command_available(&s, &Command::Play(p)));
    assert!(command_available(&s, &Command::SetVolume(p, 0.5)));
}

#[test]
fn an_unplayable_history_entry_does_not_offer_previous() {
    let (mut s, p) = three();
    next(&mut s, p);
    next(&mut s, p);
    let first = entries(&s)[0];
    let track = s.playlists.entry(first).unwrap().track;
    apply(
        &mut s,
        Command::SetFileState {
            track,
            state: fp_model::FileState::Unreadable,
        },
    )
    .unwrap();
    assert!(!availability(&s, p).previous);
}

#[test]
fn restart_waits_while_a_fade_stop_runs() {
    let (mut s, p) = three();
    next(&mut s, p);
    apply(&mut s, Command::FadeStop(p)).unwrap();
    assert!(!availability(&s, p).restart);
    assert!(apply(&mut s, Command::Restart(p)).unwrap().is_empty());
}
