#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Automation spec AU15, AU17, AU18, AU26: Start, a block in order, Stop
//! after current.

use std::path::PathBuf;

use fp_model::schedule::{Block, Item};
use fp_model::{CategoryId, Command, EngineAction, FileState, Track, TrackId, TrackKind};

use crate::automation_rig::{Rig, song, tod, today};

fn two_blocks() -> Rig {
    Rig::new(vec![
        (10, 0, vec![song(1), song(2)]),
        (11, 0, vec![song(3)]),
    ])
}

#[test]
fn start_with_a_coming_block_plays_music_until_it() {
    let mut r = two_blocks();
    r.at_hms(10, 58, 0);
    r.apply(Command::StartAutomation);
    assert!(r.playing(r.filler_player()).is_some());
    assert_eq!(r.playing(r.schedule_player()), None);
    r.at_hms(10, 59, 30);
    assert_eq!(r.playing(r.schedule_player()), None);
    r.at_hms(11, 0, 0);
    assert_eq!(r.playing(r.schedule_player()), Some(TrackId(3)));
}

#[test]
fn start_with_a_hand_set_next_plays_it_now() {
    let mut r = two_blocks();
    r.at_hms(11, 30, 0);
    let item = r.item(today(), 10, 0, 0);
    r.apply(Command::SetScheduleNext(item));
    r.apply(Command::StartAutomation);
    assert_eq!(r.playing(r.schedule_player()), Some(TrackId(1)));
    assert!(!r.s.automation.as_ref().unwrap().next_by_hand);
}

#[test]
fn a_block_plays_its_items_in_order() {
    let mut r = two_blocks();
    r.at_hms(9, 59, 0);
    r.apply(Command::StartAutomation);
    r.at_hms(10, 0, 0);
    let p = r.schedule_player();
    assert_eq!(r.playing(p), Some(TrackId(1)));
    r.end_current(p);
    assert_eq!(r.playing(p), Some(TrackId(2)));
}

#[test]
fn stop_after_current_ends_the_item_and_stops() {
    let mut r = two_blocks();
    r.at_hms(9, 59, 0);
    r.apply(Command::StartAutomation);
    r.at_hms(10, 0, 0);
    let p = r.schedule_player();
    r.apply(Command::StopAutomationAfterCurrent);
    assert!(!r.s.automation.as_ref().unwrap().running);
    r.end_current(p);
    r.at_hms(10, 3, 0);
    assert_eq!(r.playing(p), None);
    assert_eq!(r.playing(r.filler_player()), None);
}

#[test]
fn stop_after_current_lets_the_filler_song_end() {
    let mut r = two_blocks();
    r.at_hms(10, 30, 0);
    r.apply(Command::StartAutomation);
    let f = r.filler_player();
    assert!(r.playing(f).is_some());
    r.apply(Command::StopAutomationAfterCurrent);
    assert!(r.player(f).stop_after_current);
    r.end_current(f);
    assert_eq!(r.playing(f), None);
    r.at_hms(11, 0, 0);
    assert_eq!(r.playing(r.schedule_player()), None);
}

#[test]
fn a_category_item_picks_a_track_when_its_block_loads() {
    let mut r = Rig::new(vec![]);
    r.apply(Command::CreateCategory {
        parent: None,
        name: "Spots".into(),
        color: None,
        kind: TrackKind::Ad,
    });
    let spots: CategoryId = r.s.categories.iter().last().unwrap().id;
    let mut t = Track::new(TrackId(901), PathBuf::from("/m/spot.wav"));
    t.duration_secs = 30.0;
    t.category = Some(spots);
    t.kind = TrackKind::Ad;
    r.s.library.insert(t);
    let base = r.base;
    r.apply(Command::AddBlock {
        template: base,
        start: tod(10, 0),
    });
    let id = r.s.schedule.template(base).unwrap().blocks[0].id;
    r.apply(Command::UpdateBlock {
        template: base,
        block: Block {
            id,
            start: tod(10, 0),
            items: vec![Item::Category {
                category: spots,
                time_critical: false,
            }],
        },
    });
    r.at_hms(9, 59, 0);
    r.apply(Command::StartAutomation);
    r.at_hms(10, 0, 0);
    assert_eq!(r.playing(r.schedule_player()), Some(TrackId(901)));
}

#[test]
fn a_block_with_nothing_playable_moves_on() {
    let mut r = Rig::new(vec![(10, 0, vec![song(5)]), (11, 0, vec![song(3)])]);
    r.s.library.get_mut(TrackId(5)).unwrap().file_state = FileState::Missing;
    r.at_hms(9, 59, 0);
    r.apply(Command::StartAutomation);
    r.at_hms(10, 0, 0);
    r.at_hms(10, 0, 1);
    assert_eq!(r.playing(r.schedule_player()), None);
    assert!(r.missed().contains(&r.item(today(), 10, 0, 0)));
    assert!(r.playing(r.filler_player()).is_some());
    assert_eq!(
        r.s.automation.as_ref().unwrap().next,
        Some(r.item(today(), 11, 0, 0))
    );
}

#[test]
fn editing_the_schedule_never_cuts_the_item_on_air() {
    let mut r = two_blocks();
    r.at_hms(9, 59, 0);
    r.apply(Command::StartAutomation);
    r.at_hms(10, 0, 0);
    let p = r.schedule_player();
    let base = r.base;
    let id = r.block(10, 0);
    let actions = r.apply(Command::UpdateBlock {
        template: base,
        block: Block {
            id,
            start: tod(10, 0),
            items: vec![song(7)],
        },
    });
    assert!(!actions.iter().any(|a| matches!(a,
        EngineAction::StopNow { player } | EngineAction::FadeOutAndStop { player, .. } if *player == p)));
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, EngineAction::Crossfade { player, .. } if *player == p))
    );
    assert_eq!(r.playing(p), Some(TrackId(1)));
}

#[test]
fn nothing_plays_by_itself() {
    let mut r = two_blocks();
    for h in [9, 10, 11, 12] {
        let actions = r.at_hms(h, 0, 0);
        assert!(!actions.iter().any(|a| matches!(
            a,
            EngineAction::StartCurrent { .. } | EngineAction::Crossfade { .. }
        )));
    }
}

#[test]
fn after_a_restart_automation_is_stopped_and_silent() {
    let mut r = two_blocks();
    r.at_hms(9, 59, 0);
    r.apply(Command::StartAutomation);
    r.at_hms(10, 0, 0);
    assert!(r.playing(r.schedule_player()).is_some());
    let s = &r.s;
    let parts = fp_model::RestoreParts {
        rotations: s.rotations.clone(),
        air_times: Default::default(),
        filler: s.filler_session(|_| 10.0),
        automation: s.automation_session(|_| 10.0),
        schedule: s.schedule.clone(),
        library_view: Default::default(),
        config: s.config.clone(),
        library: s.library.clone(),
        categories: s.categories.clone(),
        playlists: s.playlists.clone(),
        cart_pages: s.cartwall.pages.clone(),
        cartwall_session: s.cartwall.session(),
        layout: s.layout.clone(),
        ids: s.ids.clone(),
    };
    let (mut back, actions) = fp_model::AppState::restore(parts, &s.sessions(|_| 10.0), "Main");
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, EngineAction::StartCurrent { .. }))
    );
    let a = back.automation.clone().unwrap();
    assert_eq!(a.player, r.schedule_player());
    assert!(!a.running);
    assert!(fp_model::automation::ensure(&mut back, "Schedule").is_empty());
    let list = back.playlists.get(a.playlist).unwrap();
    assert!(list.entries.is_empty());
    let player = back.players.iter().find(|p| p.id == a.player).unwrap();
    assert_eq!(player.transport, fp_model::Transport::Stopped);
    // The schedule's player comes before the filler's, which is last.
    assert_eq!(back.players.last().unwrap().id, r.filler_player());
    back.local_now = today().and_hms_opt(11, 0, 0).unwrap();
    let (actions, _) = fp_model::automation::tick(&mut back);
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, EngineAction::StartCurrent { .. }))
    );
}
