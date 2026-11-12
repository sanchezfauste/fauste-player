#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Automation spec §8.1, AU15: the schedule's own player.

use fp_model::automation::ensure;
use fp_model::{AppState, Command, Config, EngineAction, ModelError, Transport, apply};

fn state() -> AppState {
    let mut s = AppState::new(Config::default(), "P");
    fp_model::rotation::ensure_default(&mut s, "Music");
    fp_model::filler::ensure(&mut s, "Music filler");
    s
}

#[test]
fn ensure_adds_a_hidden_schedule_player_once() {
    let mut s = state();
    let first = ensure(&mut s, "Schedule");
    let a = s.automation.clone().unwrap();
    assert_eq!(first, vec![EngineAction::AddPlayer { player: a.player }]);
    assert!(ensure(&mut s, "Schedule").is_empty());
    assert_eq!(s.players.iter().filter(|p| p.id == a.player).count(), 1);
    let player = s.players.iter().find(|p| p.id == a.player).unwrap();
    assert_eq!(player.transport, Transport::Stopped);
    assert!(s.layout.hidden_players.contains(&a.player));
    assert!(s.aux_players().all(|p| p.id != a.player));
    assert_eq!(s.aux_players().count(), s.config.players.count);
    let filler = s.filler.as_ref().unwrap().player;
    assert_eq!(s.player_ids()[0], a.player);
    assert_eq!(s.player_ids()[1], filler);
    assert!(s.is_automation(a.player) && s.is_automation(filler));
    assert!(s.is_automation_playlist(a.playlist));
    // The filler stays last in `players`.
    assert_eq!(s.players.last().unwrap().id, filler);
}

#[test]
fn the_schedule_list_is_not_an_operator_list() {
    let mut s = state();
    ensure(&mut s, "Schedule");
    let list = s.automation.as_ref().unwrap().playlist;
    let p1 = s.aux_players().next().unwrap().id;
    assert_eq!(
        apply(
            &mut s,
            Command::RenamePlaylist {
                playlist: list,
                name: "x".into()
            }
        ),
        Err(ModelError::FillerPlaylist(list))
    );
    assert_eq!(
        apply(&mut s, Command::DeletePlaylist(list)),
        Err(ModelError::FillerPlaylist(list))
    );
    assert_eq!(
        apply(&mut s, Command::ShowPlaylist(p1, list)),
        Err(ModelError::FillerPlaylist(list))
    );
}

#[test]
fn automation_starts_stopped() {
    let mut s = state();
    ensure(&mut s, "Schedule");
    let a = s.automation.as_ref().unwrap();
    assert!(!a.running);
    assert!(a.next.is_none());
    assert!(!a.next_by_hand);
}
