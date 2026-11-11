#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Automation spec AU27–AU29: the music filler.

use fp_model::filler::ensure;
use fp_model::{AppState, Config, EngineAction, Transport};

fn state() -> AppState {
    let mut s = AppState::new(Config::default(), "P");
    fp_model::rotation::ensure_default(&mut s, "Music");
    s
}

#[test]
fn ensure_creates_a_stopped_filler_last_once() {
    let mut s = state();
    let first = ensure(&mut s, "Music filler");
    let filler = s.filler.clone().unwrap();
    assert_eq!(s.players.last().unwrap().id, filler.player);
    assert_eq!(s.players.last().unwrap().transport, Transport::Stopped);
    assert_eq!(
        first,
        vec![EngineAction::AddPlayer {
            player: filler.player
        }]
    );
    assert!(ensure(&mut s, "Music filler").is_empty());
    assert_eq!(s.players.iter().filter(|p| s.is_filler(p.id)).count(), 1);
    assert_eq!(s.aux_players().count(), s.config.players.count);
    assert!(s.is_filler_playlist(filler.playlist));
    assert_eq!(
        s.playlists.get(filler.playlist).unwrap().name,
        "Music filler"
    );
}

#[test]
fn filler_settings_defaults_and_ranges() {
    let mut c = Config::default();
    assert_eq!(
        (
            c.automation.filler_lookahead_mins,
            c.automation.filler_played_kept
        ),
        (30, 50)
    );
    c.automation.filler_lookahead_mins = 4;
    c.automation.filler_played_kept = 2_000;
    c.validate();
    assert_eq!(c.automation.filler_lookahead_mins, 5);
    assert_eq!(c.automation.filler_played_kept, 1_000);
}

#[test]
fn player_count_changes_leave_the_filler_last() {
    let mut s = state();
    ensure(&mut s, "Music filler");
    let filler = s.filler.as_ref().unwrap().player;
    fp_model::apply(&mut s, fp_model::Command::SetPlayerCount(2)).unwrap();
    assert_eq!(s.players.last().unwrap().id, filler);
    assert_eq!(s.aux_players().count(), 2);
    assert_eq!(s.players.len(), 3);
    fp_model::apply(&mut s, fp_model::Command::SetPlayerCount(5)).unwrap();
    assert_eq!(s.players.last().unwrap().id, filler);
    assert_eq!(s.aux_players().count(), 5);
    assert_eq!(s.config.players.count, 5);
}

#[test]
fn numbered_shortcuts_never_reach_the_filler() {
    let mut s = state();
    ensure(&mut s, "Music filler");
    assert_eq!(s.aux_players().count(), 4);
    assert_eq!(
        fp_model::player_command(&s, fp_model::ShortcutAction::PlayPlayer(5)),
        None
    );
    let first = s.players[0].id;
    assert_eq!(
        fp_model::player_command(&s, fp_model::ShortcutAction::PlayPlayer(1)),
        Some(fp_model::Command::Play(first))
    );
}

#[test]
fn sessions_leave_the_filler_out() {
    let mut s = state();
    ensure(&mut s, "Music filler");
    let filler = s.filler.as_ref().unwrap().player;
    let sessions = s.sessions(|_| 0.0);
    assert_eq!(sessions.len(), 4);
    assert!(sessions.iter().all(|p| p.id != filler));
}
