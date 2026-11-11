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
