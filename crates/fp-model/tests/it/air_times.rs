#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Automation spec AU35: what counts as aired, and when.

use std::path::PathBuf;

use crate::common::{entries, fixture, p0};
use fp_model::{AirTimes, Command, SplitMix64, TrackId, WallTime, apply};

fn track_of(state: &fp_model::AppState, entry: fp_model::EntryId) -> TrackId {
    state.track_for_entry(entry).unwrap().id
}

#[test]
fn a_player_start_records_the_track_at_wall_now() {
    let mut s = fixture(2);
    s.wall_now = WallTime(1_000);
    let first = entries(&s)[0];
    let p = p0(&s);
    apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(s.air_times.get(track_of(&s, first)), Some(WallTime(1_000)));
    assert_eq!(s.air_times.len(), 1);
}

#[test]
fn a_cart_fire_records_its_track() {
    let mut s = fixture(0);
    s.wall_now = WallTime(5_000);
    let page = s.cartwall.pages[0].id;
    apply(
        &mut s,
        Command::AssignCartFile {
            page,
            index: 0,
            path: PathBuf::from("/carts/id.wav"),
        },
    )
    .unwrap();
    let cart = s.cartwall.pages[0].carts[0].id;
    let track = s.cartwall.pages[0].carts[0].track.unwrap();
    apply(&mut s, Command::FireCart(cart)).unwrap();
    assert_eq!(s.air_times.get(track), Some(WallTime(5_000)));
}

#[test]
fn a_cue_does_not_record() {
    let mut s = fixture(2);
    s.wall_now = WallTime(1_000);
    let first = entries(&s)[0];
    let p = p0(&s);
    apply(&mut s, Command::CueEntry(p, first)).unwrap();
    let track = track_of(&s, first);
    apply(&mut s, Command::SelectLibraryCue(Some(track))).unwrap();
    let _ = apply(&mut s, Command::PlayLibraryCue);
    assert!(s.air_times.is_empty());
}

#[test]
fn removing_a_track_drops_its_air_time() {
    let mut s = fixture(2);
    s.wall_now = WallTime(1_000);
    let second = entries(&s)[1];
    let track = track_of(&s, second);
    s.air_times.record(track, WallTime(10));
    apply(&mut s, Command::RemoveTracks(vec![track])).unwrap();
    assert_eq!(s.air_times.get(track), None);
}

#[test]
fn record_keeps_the_later_time() {
    let mut a = AirTimes::default();
    a.record(TrackId(1), WallTime(500));
    a.record(TrackId(1), WallTime(100));
    assert_eq!(a.get(TrackId(1)), Some(WallTime(500)));
    a.record(TrackId(1), WallTime(900));
    assert_eq!(a.get(TrackId(1)), Some(WallTime(900)));
}

#[test]
fn splitmix_is_deterministic_and_in_range() {
    let mut a = SplitMix64(42);
    let mut b = SplitMix64(42);
    let xs: Vec<u64> = (0..10).map(|_| a.next_u64()).collect();
    let ys: Vec<u64> = (0..10).map(|_| b.next_u64()).collect();
    assert_eq!(xs, ys);
    let mut r = SplitMix64(7);
    let draws: Vec<u8> = (0..1_000).map(|_| r.in_range(2, 3)).collect();
    assert!(draws.iter().all(|d| *d == 2 || *d == 3));
    assert!(draws.contains(&2) && draws.contains(&3));
    assert!((0..100).all(|_| (2..=3).contains(&r.in_range(3, 2))));
    assert_eq!(r.below(0), 0);
    assert!((0..100).all(|_| r.below(5) < 5));
}

#[test]
fn wall_time_arithmetic_saturates() {
    assert_eq!(WallTime(1_000).plus_secs(1.5), WallTime(2_500));
    assert_eq!(WallTime(1_000).plus_secs(f64::NAN), WallTime(1_000));
    assert_eq!(WallTime(i64::MAX).plus_secs(10.0), WallTime(i64::MAX));
    assert_eq!(WallTime(1_000).millis_since(WallTime(400)), 600);
    assert_eq!(WallTime(400).millis_since(WallTime(1_000)), -600);
    assert_eq!(
        WallTime(i64::MIN).millis_since(WallTime(i64::MAX)),
        i64::MIN
    );
}
