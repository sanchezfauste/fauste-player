#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Automation spec AU31–AU34: separation rules, fair choice, relaxing.

use std::path::PathBuf;

use fp_model::rotation::StepSource;
use fp_model::selection::{PICK_SPREAD, Relaxed, pick, pool};
use fp_model::{
    AppState, CategoryId, Command, Config, FileState, SplitMix64, Track, TrackId, TrackKind,
    WallTime, apply,
};

const NOW: WallTime = WallTime(1_000_000_000_000);
const MIN: i64 = 60_000;

fn state() -> AppState {
    AppState::new(Config::default(), "P")
}

fn category(s: &mut AppState, name: &str, parent: Option<CategoryId>) -> CategoryId {
    apply(
        s,
        Command::CreateCategory {
            parent,
            name: name.into(),
            color: None,
            kind: TrackKind::Music,
        },
    )
    .unwrap();
    s.categories.iter().map(|c| c.id).max().unwrap()
}

/// Adds track `id` by `artist` in `category`.
fn track(s: &mut AppState, id: u64, artist: &str, category: Option<CategoryId>) -> TrackId {
    let mut t = Track::new(TrackId(id), PathBuf::from(format!("/m/{id}.flac")));
    t.artist = artist.into();
    t.category = category;
    t.duration_secs = 200.0;
    s.library.insert(t);
    TrackId(id)
}

/// `n` tracks with ids `first..first + n`, each by its own artist.
fn tracks(s: &mut AppState, first: u64, n: u64, category: Option<CategoryId>) -> Vec<TrackId> {
    (first..first + n)
        .map(|id| track(s, id, &format!("Artist {id}"), category))
        .collect()
}

fn aired(s: &mut AppState, id: TrackId, minutes_ago: i64) {
    s.air_times.record(id, WallTime(NOW.0 - minutes_ago * MIN));
}

fn cfg(s: &mut AppState, mins: u32, pct: u8, artist: u32) {
    s.config.automation.track_separation_mins = mins;
    s.config.automation.track_separation_percent = pct;
    s.config.automation.artist_separation_mins = artist;
}

fn pick_one(s: &AppState, source: StepSource) -> Option<fp_model::selection::Picked> {
    pick(s, &source, NOW, &[], &mut SplitMix64(7))
}

fn eligible(s: &AppState, source: StepSource) -> Vec<TrackId> {
    fp_model::selection::eligible(s, &source, NOW, &[])
}

fn all_music() -> StepSource {
    StepSource::Kind(TrackKind::Music)
}

#[test]
fn a_never_aired_track_is_preferred() {
    let mut s = state();
    let ids = tracks(&mut s, 1, 2, None);
    cfg(&mut s, 0, 0, 0);
    aired(&mut s, ids[0], 600);
    assert_eq!(eligible(&s, all_music()), vec![ids[1], ids[0]]);
}

#[test]
fn track_separation_by_time() {
    let mut s = state();
    let ids = tracks(&mut s, 1, 10, None);
    cfg(&mut s, 180, 0, 0);
    aired(&mut s, ids[0], 179);
    assert!(!eligible(&s, all_music()).contains(&ids[0]));
    let mut s = state();
    let ids = tracks(&mut s, 1, 10, None);
    cfg(&mut s, 180, 0, 0);
    aired(&mut s, ids[0], 180);
    assert!(eligible(&s, all_music()).contains(&ids[0]));
}

#[test]
fn track_separation_by_percent_of_the_category() {
    let mut s = state();
    let ids = tracks(&mut s, 1, 35, None);
    cfg(&mut s, 0, 50, 0);
    // The candidate aired 100 min ago and 16 others since; the rest never
    // aired. Needed: ceil(50 % of the 34 others) = 17.
    aired(&mut s, ids[0], 100);
    for (k, id) in ids[1..17].iter().enumerate() {
        aired(&mut s, *id, 99 - k as i64);
    }
    assert!(!eligible(&s, all_music()).contains(&ids[0]));
    aired(&mut s, ids[17], 50);
    assert!(eligible(&s, all_music()).contains(&ids[0]));
}

#[test]
fn either_criterion_is_enough() {
    let mut s = state();
    let ids = tracks(&mut s, 1, 3, None);
    cfg(&mut s, 60, 50, 0);
    // By time: aired 61 min ago, nothing since.
    aired(&mut s, ids[0], 61);
    aired(&mut s, ids[1], 61);
    aired(&mut s, ids[2], 61);
    let e = eligible(&s, all_music());
    assert_eq!(e.len(), 3);
    // By percent: aired 1 min ago... and the 2 others aired since.
    let mut s = state();
    let ids = tracks(&mut s, 1, 3, None);
    cfg(&mut s, 60, 50, 0);
    aired(&mut s, ids[0], 3);
    aired(&mut s, ids[1], 2);
    aired(&mut s, ids[2], 1);
    // Needed: ceil(50 % of 2) = 1. The newest has none aired since.
    let e = eligible(&s, all_music());
    assert_eq!(e, vec![ids[0], ids[1]]);
}

#[test]
fn artist_separation_across_the_library() {
    let mut s = state();
    let other = category(&mut s, "Other", None);
    let pop = category(&mut s, "Pop", None);
    let blocker = track(&mut s, 1, "  Ada Lindqvist ", Some(other));
    let candidate = track(&mut s, 2, "ada lindqvist", Some(pop));
    cfg(&mut s, 0, 0, 30);
    aired(&mut s, blocker, 29);
    let p = pick_one(&s, StepSource::Category(pop)).unwrap();
    assert_eq!((p.track, p.relaxed), (candidate, Relaxed::Artist));
    let mut s = s.clone();
    s.air_times = Default::default();
    aired(&mut s, blocker, 30);
    let p = pick_one(&s, StepSource::Category(pop)).unwrap();
    assert_eq!((p.track, p.relaxed), (candidate, Relaxed::None));
}

#[test]
fn relaxing_order_artist_then_track() {
    // A one-track category aired a minute ago: relaxed down to Track.
    let mut s = state();
    let solo = category(&mut s, "Solo", None);
    let only = track(&mut s, 1, "A", Some(solo));
    // Percent off: with no other track, "half of the others" is always met.
    cfg(&mut s, 180, 0, 30);
    aired(&mut s, only, 1);
    let p = pick_one(&s, StepSource::Category(solo)).unwrap();
    assert_eq!((p.track, p.relaxed), (only, Relaxed::Track));
    // Every track by an artist aired a minute ago: relaxed to Artist.
    let mut s = state();
    let cat = category(&mut s, "One artist", None);
    for id in 1..=3 {
        track(&mut s, id, "Same", Some(cat));
    }
    let elsewhere = track(&mut s, 10, "Same", None);
    cfg(&mut s, 180, 50, 30);
    aired(&mut s, elsewhere, 1);
    let p = pick_one(&s, StepSource::Category(cat)).unwrap();
    assert_eq!(p.relaxed, Relaxed::Artist);
}

#[test]
fn nothing_playable_gives_none() {
    let s = state();
    assert_eq!(pick_one(&s, all_music()), None);
    let mut s = state();
    let ids = tracks(&mut s, 1, 3, None);
    for id in ids {
        s.library.get_mut(id).unwrap().file_state = FileState::Missing;
    }
    assert!(pool(&s, &all_music()).is_empty());
    assert_eq!(pick_one(&s, all_music()), None);
}

#[test]
fn fair_choice_stays_among_the_oldest() {
    let mut s = state();
    let ids = tracks(&mut s, 1, 20, None);
    cfg(&mut s, 0, 0, 0);
    // Track k aired (20 - k) * 10 minutes ago: ids[0] is the oldest.
    for (k, id) in ids.iter().enumerate() {
        aired(&mut s, *id, (20 - k as i64) * 10);
    }
    let oldest: Vec<TrackId> = ids[..PICK_SPREAD].to_vec();
    let mut rng = SplitMix64(11);
    let mut seen = std::collections::HashSet::new();
    for _ in 0..200 {
        let p = pick(&s, &all_music(), NOW, &[], &mut rng).unwrap();
        assert!(oldest.contains(&p.track), "{:?}", p.track);
        seen.insert(p.track);
    }
    assert_eq!(
        seen.len(),
        PICK_SPREAD,
        "every one of the oldest gets a turn"
    );
}

#[test]
fn planned_tracks_count_as_aired() {
    let mut s = state();
    let ids = tracks(&mut s, 1, 2, None);
    cfg(&mut s, 180, 0, 0);
    aired(&mut s, ids[1], 600);
    let planned = [(ids[0], WallTime(NOW.0 + 3 * MIN))];
    let p = pick(&s, &all_music(), NOW, &planned, &mut SplitMix64(7)).unwrap();
    assert_eq!((p.track, p.relaxed), (ids[1], Relaxed::None));
}

#[test]
fn an_air_time_in_the_future_counts_as_just_aired() {
    let mut s = state();
    let ids = tracks(&mut s, 1, 2, None);
    cfg(&mut s, 180, 0, 0);
    s.air_times.record(ids[0], WallTime(NOW.0 + 60 * MIN));
    aired(&mut s, ids[1], 600);
    let p = pick_one(&s, all_music()).unwrap();
    assert_eq!(p.track, ids[1]);
    let e = eligible(&s, all_music());
    assert!(!e.contains(&ids[0]));
}

#[test]
fn a_category_includes_its_subcategories() {
    let mut s = state();
    let music = category(&mut s, "Music", None);
    let pop = category(&mut s, "Pop", Some(music));
    let other = category(&mut s, "Other", None);
    let a = track(&mut s, 1, "A", Some(music));
    let b = track(&mut s, 2, "B", Some(pop));
    track(&mut s, 3, "C", Some(other));
    let mut p = pool(&s, &StepSource::Category(music));
    p.sort();
    assert_eq!(p, vec![a, b]);
}

#[test]
fn a_kind_source_takes_every_track_of_that_kind() {
    let mut s = state();
    let a = track(&mut s, 1, "A", None);
    let b = track(&mut s, 2, "B", None);
    s.library.get_mut(b).unwrap().kind = TrackKind::Jingle;
    assert_eq!(pool(&s, &StepSource::Kind(TrackKind::Music)), vec![a]);
    assert_eq!(pool(&s, &StepSource::Kind(TrackKind::Jingle)), vec![b]);
}
