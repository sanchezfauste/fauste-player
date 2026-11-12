#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Automation spec AU19–AU22, AU24, AU27: blocks in time order, music
//! between them, overruns, late time-critical items, the hand-over.

use chrono::{Days, NaiveDate};
use fp_model::schedule::{Block, Item};
use fp_model::{Command, Config, EngineAction, TrackId};

use crate::automation_rig::{Rig, hourly_id, song, tod, today};

fn tomorrow() -> NaiveDate {
    today().checked_add_days(Days::new(1)).unwrap()
}

fn fades(actions: &[EngineAction], player: fp_model::PlayerId) -> Option<u32> {
    actions.iter().find_map(|a| match a {
        EngineAction::FadeOutAndStop { player: p, fade_ms } if *p == player => Some(*fade_ms),
        _ => None,
    })
}

#[test]
fn after_a_block_music_until_the_next() {
    let mut r = Rig::new(vec![(10, 0, vec![song(1)]), (11, 0, vec![song(2)])]);
    r.at_hms(9, 59, 0);
    r.apply(Command::StartAutomation);
    r.at_hms(10, 0, 0);
    r.at_hms(10, 3, 0);
    r.end_current(r.schedule_player());
    r.at_hms(10, 3, 1);
    assert_eq!(r.playing(r.schedule_player()), None);
    assert!(r.playing(r.filler_player()).is_some());
    r.at_hms(11, 0, 0);
    assert_eq!(r.playing(r.schedule_player()), Some(TrackId(2)));
    assert!(r.missed().is_empty(), "{:?}", r.missed());
}

#[test]
fn the_filler_fades_out_before_the_block() {
    let mut r = Rig::new(vec![(11, 0, vec![song(2)])]);
    r.at_hms(10, 30, 0);
    r.apply(Command::StartAutomation);
    let f = r.filler_player();
    assert!(fades(&r.at_hms(10, 59, 57), f).is_none());
    assert_eq!(fades(&r.at_hms(10, 59, 58), f), Some(2000));
    // The fade ran out: the filler does not start again before the block.
    let actions = r.at_hms(10, 59, 59);
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, EngineAction::StartCurrent { player, .. } if *player == f))
    );
    assert_eq!(r.playing(f), None);
    r.at_hms(11, 0, 0);
    assert_eq!(r.playing(r.schedule_player()), Some(TrackId(2)));
}

#[test]
fn a_block_played_by_hand_late_skips_to_the_next_unstarted() {
    let mut r = Rig::new(vec![
        (10, 0, vec![song(1), song(2)]),
        (11, 0, vec![song(3)]),
        (12, 0, vec![song(4)]),
    ]);
    r.at_hms(11, 0, 0);
    let ten = r.item(today(), 10, 0, 0);
    r.apply(Command::SetScheduleNext(ten));
    r.apply(Command::StartAutomation);
    let p = r.schedule_player();
    assert_eq!(r.playing(p), Some(TrackId(1)));
    r.at_hms(11, 30, 0);
    r.end_current(p);
    r.at_hms(11, 55, 0);
    r.end_current(p);
    r.at_hms(11, 55, 1);
    assert_eq!(r.playing(p), None);
    assert!(r.playing(r.filler_player()).is_some());
    assert!(r.missed().contains(&r.item(today(), 11, 0, 0)));
    r.at_hms(12, 0, 0);
    assert_eq!(r.playing(p), Some(TrackId(4)));
}

#[test]
fn an_overrun_plays_the_next_block_when_it_ends() {
    let mut r = Rig::new(vec![
        (11, 0, vec![song(3)]),
        (12, 0, vec![song(4), song(5)]),
    ]);
    r.at_hms(10, 59, 0);
    r.apply(Command::StartAutomation);
    r.at_hms(11, 0, 0);
    r.at_hms(12, 4, 0);
    // The 12:00 block waits for the 11:00 block, which overran.
    assert_eq!(r.playing(r.schedule_player()), Some(TrackId(3)));
    r.end_current(r.schedule_player());
    r.at_hms(12, 4, 1);
    assert_eq!(r.playing(r.schedule_player()), Some(TrackId(4)));
    assert!(r.missed().is_empty(), "{:?}", r.missed());
}

#[test]
fn a_late_hourly_id_is_skipped() {
    let mut r = Rig::new(vec![
        (11, 0, vec![song(3)]),
        (12, 0, vec![hourly_id(10), song(11)]),
    ]);
    r.s.config.automation.late_tolerance_secs = 120;
    r.at_hms(10, 59, 0);
    r.apply(Command::StartAutomation);
    r.at_hms(11, 0, 0);
    r.at_hms(12, 3, 0);
    r.end_current(r.schedule_player());
    r.at_hms(12, 3, 1);
    assert_eq!(r.playing(r.schedule_player()), Some(TrackId(11)));
    assert!(r.missed().contains(&r.item(today(), 12, 0, 0)));
}

#[test]
fn a_time_critical_item_later_in_a_block_is_skipped_when_it_would_be_late() {
    // 11:00: two 180 s songs, then the hourly ID; the block was started by
    // hand at 11:00 but its first song ends late (here: at 11:06).
    let mut r = Rig::new(vec![(
        11,
        0,
        vec![song(3), song(4), hourly_id(10), song(12)],
    )]);
    r.s.config.automation.late_tolerance_secs = 120;
    r.at_hms(10, 59, 0);
    r.apply(Command::StartAutomation);
    r.at_hms(11, 0, 0);
    let p = r.schedule_player();
    r.at_hms(11, 6, 0);
    r.end_current(p);
    r.at_hms(11, 6, 1);
    assert_eq!(r.playing(p), Some(TrackId(4)));
    // Song 4 ends after 11:02: the hourly ID would be late.
    r.at_hms(11, 6, 2);
    r.end_current(p);
    assert_eq!(r.playing(p), Some(TrackId(12)));
    assert!(r.missed().contains(&r.item(today(), 11, 0, 2)));
}

#[test]
fn an_overrun_chains_across_midnight() {
    let mut r = Rig::new(vec![(23, 30, vec![song(1)]), (0, 0, vec![song(2)])]);
    r.at_hms(23, 29, 0);
    r.apply(Command::StartAutomation);
    r.at_hms(23, 30, 0);
    assert_eq!(r.playing(r.schedule_player()), Some(TrackId(1)));
    r.at(tomorrow().and_hms_opt(0, 1, 0).unwrap());
    r.end_current(r.schedule_player());
    r.at(tomorrow().and_hms_opt(0, 1, 1).unwrap());
    assert_eq!(r.playing(r.schedule_player()), Some(TrackId(2)));
    let on_air = r.s.automation.as_ref().unwrap().on_air.unwrap();
    assert_eq!(on_air.date, tomorrow());
}

#[test]
fn the_rotation_item_sets_the_filler_rotation() {
    let mut r = Rig::new(vec![(10, 0, vec![song(1)]), (11, 0, vec![song(2)])]);
    r.apply(Command::CreateRotation {
        name: "Night".into(),
        steps: r.s.rotations.default_rotation().unwrap().steps.clone(),
    });
    let night = r.s.rotations.iter().last().unwrap().id;
    let base = r.base;
    let ten = r.block(10, 0);
    r.apply(Command::UpdateBlock {
        template: base,
        block: Block {
            id: ten,
            start: tod(10, 0),
            items: vec![song(1), Item::Rotation { rotation: night }],
        },
    });
    r.at_hms(9, 59, 0);
    r.apply(Command::StartAutomation);
    r.at_hms(10, 0, 0);
    r.end_current(r.schedule_player());
    r.at_hms(10, 3, 1);
    assert_eq!(r.s.filler.as_ref().unwrap().rotation, Some(night));
    r.at_hms(11, 0, 0);
    r.end_current(r.schedule_player());
    r.at_hms(11, 3, 1);
    assert_eq!(r.s.filler.as_ref().unwrap().rotation, None);
}

#[test]
fn a_next_set_by_hand_between_blocks_follows_the_filler_song() {
    let mut r = Rig::new(vec![(10, 0, vec![song(1)]), (12, 0, vec![song(2)])]);
    r.at_hms(10, 30, 0);
    r.apply(Command::StartAutomation);
    let f = r.filler_player();
    let item = r.item(today(), 10, 0, 0);
    r.apply(Command::SetScheduleNext(item));
    r.at_hms(10, 30, 1);
    assert!(r.playing(f).is_some());
    assert_eq!(r.playing(r.schedule_player()), None);
    r.end_current(f);
    r.at_hms(10, 33, 0);
    assert_eq!(r.playing(r.schedule_player()), Some(TrackId(1)));
}

#[test]
fn block_fade_ms_is_clamped() {
    let mut c = Config::default();
    assert_eq!(c.automation.block_fade_ms, 2000);
    c.automation.block_fade_ms = 20_000;
    c.validate();
    assert_eq!(c.automation.block_fade_ms, 10_000);
}

#[test]
fn the_filler_looks_ahead_only_to_the_coming_block() {
    let mut r = Rig::new(vec![(11, 0, vec![song(2)])]);
    r.at_hms(10, 50, 0);
    r.apply(Command::StartAutomation);
    r.at_hms(10, 50, 1);
    let f = r.s.filler.as_ref().unwrap();
    let rows = r.s.playlists.get(f.playlist).unwrap().entries.len();
    // The song on air and ten minutes of 3-minute songs ahead (four rows),
    // not 30 minutes' worth.
    assert_eq!(rows, 5, "{rows} rows queued");
}
