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
    // Rework AR6: running until what is on air ends.
    assert!(r.s.automation.as_ref().unwrap().running);
    r.end_current(p);
    r.at_hms(10, 3, 0);
    assert!(!r.s.automation.as_ref().unwrap().running);
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

/// A 10:00 block holding only a "Spots" category item (track 901), and an
/// 11:00 block with a song; `kind` is the category's kind.
fn spots_block(kind: TrackKind) -> (Rig, CategoryId) {
    let mut r = Rig::new(vec![(11, 0, vec![song(3)])]);
    r.apply(Command::CreateCategory {
        parent: None,
        name: "Spots".into(),
        color: None,
        kind,
    });
    let spots: CategoryId = r.s.categories.iter().last().unwrap().id;
    let mut t = Track::new(TrackId(901), PathBuf::from("/m/spot.wav"));
    t.duration_secs = 30.0;
    t.category = Some(spots);
    t.kind = kind;
    r.s.library.insert(t);
    let base = r.base;
    r.apply(Command::AddBlock {
        template: base,
        start: tod(10, 0),
    });
    let id =
        r.s.schedule
            .template(base)
            .unwrap()
            .blocks
            .iter()
            .find(|b| b.start == tod(10, 0))
            .unwrap()
            .id;
    r.blocks.insert((10, 0), id);
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
    (r, spots)
}

#[test]
fn play_on_a_category_item_set_as_next_plays_its_track() {
    let (mut r, _) = spots_block(TrackKind::Ad);
    r.at_hms(8, 0, 0);
    r.apply(Command::SetScheduleNext(r.item(today(), 10, 0, 0)));
    r.apply(Command::Play(r.schedule_player()));
    assert_eq!(r.playing(r.schedule_player()), Some(TrackId(901)));
    assert_eq!(r.playing(r.filler_player()), None);
}

/// The row of the 10:00 Spots item in the Log's view.
fn spots_row(r: &Rig) -> fp_model::automation::ScheduleRow {
    let want = r.item(today(), 10, 0, 0);
    fp_model::automation::schedule_view(&r.s)
        .into_iter()
        .find(|row| matches!(row, fp_model::automation::ScheduleRow::Item { item, .. } if *item == want))
        .unwrap()
}

#[test]
fn a_category_item_set_as_next_shows_its_track_and_plays_it() {
    let (mut r, _) = spots_block(TrackKind::Ad);
    let mut other = Track::new(TrackId(902), PathBuf::from("/m/spot2.wav"));
    other.duration_secs = 20.0;
    other.category = r.s.library.get(TrackId(901)).unwrap().category;
    other.kind = TrackKind::Ad;
    r.s.library.insert(other);
    r.at_hms(8, 0, 0);
    r.apply(Command::SetScheduleNext(r.item(today(), 10, 0, 0)));
    let fp_model::automation::ScheduleRow::Item {
        track, length_secs, ..
    } = spots_row(&r)
    else {
        panic!()
    };
    let shown = track.expect("chosen as soon as it is next");
    assert!(length_secs.is_some());
    // What was shown is what plays.
    r.at_hms(8, 0, 1);
    r.apply(Command::Play(r.schedule_player()));
    assert_eq!(r.playing(r.schedule_player()), Some(shown));
}

#[test]
fn the_clocks_next_category_item_shows_its_track() {
    let (mut r, _) = spots_block(TrackKind::Ad);
    r.at_hms(9, 0, 0);
    let fp_model::automation::ScheduleRow::Item { track, .. } = spots_row(&r) else {
        panic!()
    };
    assert_eq!(track, Some(TrackId(901)));
}

#[test]
fn a_category_with_nothing_to_play_says_so() {
    let (mut r, _) = spots_block(TrackKind::Ad);
    r.s.library.get_mut(TrackId(901)).unwrap().category = None;
    r.at_hms(9, 0, 0);
    let fp_model::automation::ScheduleRow::Item {
        track,
        nothing_to_play,
        ..
    } = spots_row(&r)
    else {
        panic!()
    };
    assert_eq!(track, None);
    assert!(nothing_to_play);
}

#[test]
fn during_the_fill_a_category_item_set_as_next_shows_its_track_and_follows_the_song() {
    let (mut r, _) = spots_block(TrackKind::Ad);
    r.at_hms(8, 0, 0);
    r.apply(Command::StartAutomation);
    r.at_hms(8, 0, 1);
    let f = r.filler_player();
    assert!(r.playing(f).is_some());
    r.apply(Command::SetScheduleNext(r.item(today(), 10, 0, 0)));
    r.at_hms(8, 0, 2);
    let fp_model::automation::ScheduleRow::Item { track, .. } = spots_row(&r) else {
        panic!()
    };
    assert_eq!(track, Some(TrackId(901)), "shown while the fill plays");
    // The filler ends its song (held), then the Log plays the item.
    assert!(r.player(f).stop_after_current);
    r.end_current(f);
    r.at_hms(8, 0, 3);
    assert_eq!(r.playing(r.schedule_player()), Some(TrackId(901)));
}

#[test]
fn during_the_fill_an_empty_category_set_as_next_says_so() {
    let (mut r, _) = spots_block(TrackKind::Ad);
    r.s.library.get_mut(TrackId(901)).unwrap().category = None;
    r.at_hms(8, 0, 0);
    r.apply(Command::StartAutomation);
    r.at_hms(8, 0, 1);
    r.apply(Command::SetScheduleNext(r.item(today(), 10, 0, 0)));
    r.at_hms(8, 0, 2);
    let fp_model::automation::ScheduleRow::Item {
        nothing_to_play, ..
    } = spots_row(&r)
    else {
        panic!()
    };
    assert!(nothing_to_play);
}

/// The longest run of `track` in a row in the filler's list.
fn longest_run(r: &Rig, track: TrackId) -> usize {
    let f = r.s.filler.as_ref().unwrap();
    let list = r.s.playlists.get(f.playlist).unwrap();
    let (mut best, mut run) = (0, 0);
    for e in &list.entries {
        if e.track == track {
            run += 1;
            best = best.max(run);
        } else {
            run = 0;
        }
    }
    best
}

/// A rotation of an empty "Music" category (1–5) and a station ID (1),
/// as the default rotation or as another one the filler plays.
fn id_rotation(as_default: bool) -> Rig {
    use fp_model::rotation::{RotationStep, StepSource};
    let mut r = Rig::new(vec![(10, 0, vec![song(1)])]);
    let category = |r: &mut Rig, name: &str| {
        r.apply(Command::CreateCategory {
            parent: None,
            name: name.into(),
            color: None,
            kind: TrackKind::Music,
        });
        r.s.categories.iter().last().unwrap().id
    };
    let empty: CategoryId = category(&mut r, "Music");
    let ids: CategoryId = category(&mut r, "ID");
    let mut t = Track::new(TrackId(950), PathBuf::from("/m/id.wav"));
    t.duration_secs = 5.0;
    t.category = Some(ids);
    t.kind = TrackKind::Jingle;
    r.s.library.insert(t);
    let steps = vec![
        RotationStep {
            source: StepSource::Category(empty),
            min: 1,
            max: 5,
        },
        RotationStep {
            source: StepSource::Category(ids),
            min: 1,
            max: 1,
        },
    ];
    if as_default {
        let rot = r.s.rotations.default_rotation().unwrap().clone();
        r.apply(Command::UpdateRotation(fp_model::rotation::Rotation {
            steps,
            ..rot
        }));
    } else {
        r.apply(Command::CreateRotation {
            name: "With ID".into(),
            steps,
        });
        let id = r.s.rotations.iter().last().unwrap().id;
        r.apply(Command::SetFillerRotation(Some(id)));
    }
    r.at_hms(8, 0, 0);
    r.apply(Command::StartAutomation);
    for s in 1..30 {
        r.at_hms(8, 0, s);
    }
    r
}

#[test]
fn an_empty_step_never_plays_the_other_step_back_to_back() {
    for as_default in [true, false] {
        let r = id_rotation(as_default);
        assert_eq!(longest_run(&r, TrackId(950)), 1, "default: {as_default}");
        let f = r.s.filler.as_ref().unwrap();
        let list = r.s.playlists.get(f.playlist).unwrap();
        // Music stood in for the empty step.
        assert!(
            list.entries.iter().any(|e| e.track.0 <= 40),
            "default: {as_default}"
        );
    }
}
