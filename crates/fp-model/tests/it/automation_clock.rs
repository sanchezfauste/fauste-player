#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Automation spec AU29, AU40, AU41: the filler on its own, clock jumps,
//! the auxiliary players.

use chrono::Days;
use fp_model::playlist::PlaylistEntry;
use fp_model::{Command, EngineAction, EntryId, TrackId};

use crate::automation_rig::{Rig, song, today};

#[test]
fn starting_automation_keeps_the_filler_song() {
    let mut r = Rig::new(vec![(11, 0, vec![song(2)])]);
    r.at_hms(10, 30, 0);
    let f = r.filler_player();
    r.apply(Command::Play(f));
    let on_air = r.playing(f);
    assert!(on_air.is_some());
    let actions = r.apply(Command::StartAutomation);
    assert!(!actions.iter().any(|a| matches!(a,
        EngineAction::StartCurrent { player, .. }
        | EngineAction::Crossfade { player, .. }
        | EngineAction::FadeOutAndStop { player, .. }
        | EngineAction::StopNow { player } if *player == f)));
    assert_eq!(r.playing(f), on_air);
}

#[test]
fn a_backward_jump_replays_nothing() {
    let mut r = Rig::new(vec![(10, 0, vec![song(1)]), (11, 0, vec![song(2)])]);
    r.at_hms(9, 59, 0);
    r.apply(Command::StartAutomation);
    r.at_hms(10, 0, 0);
    r.end_current(r.schedule_player());
    r.at_hms(10, 3, 1);
    r.at_hms(11, 0, 0);
    r.end_current(r.schedule_player());
    r.at_hms(11, 3, 1);
    // The clock is set back an hour.
    r.at_hms(10, 3, 30);
    let tomorrow = today().checked_add_days(Days::new(1)).unwrap();
    assert_eq!(
        r.s.automation.as_ref().unwrap().next,
        Some(r.item(tomorrow, 10, 0, 0))
    );
    r.at_hms(11, 0, 0);
    assert_eq!(r.playing(r.schedule_player()), None);
}

#[test]
fn a_forward_jump_misses_the_skipped_blocks() {
    let mut r = Rig::new(vec![
        (11, 0, vec![song(2)]),
        (12, 0, vec![song(3)]),
        (13, 0, vec![song(4)]),
    ]);
    r.at_hms(10, 30, 0);
    r.apply(Command::StartAutomation);
    // The clock jumps from 10:30 to 12:30.
    r.jump_hms(12, 30, 0);
    assert_eq!(r.playing(r.schedule_player()), None);
    assert!(r.missed().contains(&r.item(today(), 11, 0, 0)));
    assert!(r.missed().contains(&r.item(today(), 12, 0, 0)));
    assert_eq!(
        r.s.automation.as_ref().unwrap().next,
        Some(r.item(today(), 13, 0, 0))
    );
    r.at_hms(13, 0, 0);
    assert_eq!(r.playing(r.schedule_player()), Some(TrackId(4)));
}

#[test]
fn aux_players_do_not_delay_a_block() {
    let mut r = Rig::new(vec![(11, 0, vec![song(2)])]);
    let p1 = r.s.aux_players().next().unwrap().id;
    let list = r.s.aux_players().next().unwrap().playlist;
    r.s.playlists
        .get_mut(list)
        .unwrap()
        .entries
        .push(PlaylistEntry::new(EntryId(5000), TrackId(30)));
    r.at_hms(10, 59, 0);
    r.apply(Command::StartAutomation);
    r.apply(Command::Play(p1));
    assert!(r.playing(p1).is_some());
    r.at_hms(11, 0, 0);
    assert_eq!(r.playing(r.schedule_player()), Some(TrackId(2)));
    assert!(r.playing(p1).is_some());
}
