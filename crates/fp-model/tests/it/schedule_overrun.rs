#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Automation spec AU37: a block longer than its slot says by how much.

use std::path::PathBuf;

use fp_model::local_time::TimeOfDay;
use fp_model::schedule::{Item, overrun_secs};
use fp_model::{AppState, Command, Config, Track, TrackId, TrackKind, apply};

fn hms(h: u32, m: u32) -> TimeOfDay {
    TimeOfDay::hms(h, m, 0).unwrap()
}

/// Tracks 1..=n, each `secs` long.
fn state(lengths: &[f64]) -> AppState {
    let mut s = AppState::new(Config::default(), "P");
    for (i, secs) in lengths.iter().enumerate() {
        let id = i as u64 + 1;
        let mut t = Track::new(TrackId(id), PathBuf::from(format!("/m/{id}.wav")));
        t.duration_secs = *secs;
        s.library.insert(t);
    }
    s
}

fn track(n: u64) -> Item {
    Item::Track {
        track: TrackId(n),
        time_critical: false,
    }
}

#[test]
fn a_block_that_fits_has_no_overrun() {
    let s = state(&[120.0, 180.0]);
    let items = [track(1), track(2)];
    assert_eq!(
        overrun_secs(&s, &items, hms(10, 0), Some(hms(10, 10))),
        None
    );
}

#[test]
fn a_block_longer_than_its_slot_says_by_how_much() {
    let s = state(&[400.0, 300.0]);
    let items = [track(1), track(2)];
    assert_eq!(
        overrun_secs(&s, &items, hms(10, 0), Some(hms(10, 10))),
        Some(100.0)
    );
}

#[test]
fn the_last_block_runs_to_midnight() {
    let s = state(&[400.0]);
    let items = [track(1)];
    assert_eq!(overrun_secs(&s, &items, hms(23, 55), None), Some(100.0));
    // A next block earlier in the day is tomorrow's.
    assert_eq!(
        overrun_secs(&s, &items, hms(23, 55), Some(hms(0, 30))),
        None
    );
}

#[test]
fn a_category_counts_its_shortest_track() {
    let mut s = state(&[]);
    apply(
        &mut s,
        Command::CreateCategory {
            parent: None,
            name: "Spots".into(),
            color: None,
            kind: TrackKind::Music,
        },
    )
    .unwrap();
    let spots = s.categories.iter().map(|c| c.id).max().unwrap();
    for (id, secs) in [(1u64, 30.0), (2, 60.0)] {
        let mut t = Track::new(TrackId(id), PathBuf::from(format!("/m/{id}.wav")));
        t.duration_secs = secs;
        t.category = Some(spots);
        s.library.insert(t);
    }
    let items = [
        Item::Category {
            category: spots,
            time_critical: false,
        },
        Item::Category {
            category: spots,
            time_critical: false,
        },
    ];
    // Two spots of at least 30 s in a 50 s slot.
    let start = TimeOfDay::hms(10, 0, 0).unwrap();
    let next = TimeOfDay::hms(10, 0, 50).unwrap();
    assert_eq!(overrun_secs(&s, &items, start, Some(next)), Some(10.0));
}
