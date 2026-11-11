#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Automation spec AU2, AU34: walking a rotation's steps.

use std::path::PathBuf;

use fp_model::rotation::{RotationStep, StepSource, ensure_default};
use fp_model::rotation_cursor::{RotationCursor, next_track};
use fp_model::{
    AppState, CategoryId, Command, Config, RotationId, SplitMix64, Track, TrackId, TrackKind,
    WallTime, apply,
};

const NOW: WallTime = WallTime(1_000_000_000_000);

fn state() -> AppState {
    let mut s = AppState::new(Config::default(), "P");
    // Rules off: these tests are about the walk, not separation.
    s.config.automation.track_separation_mins = 0;
    s.config.automation.track_separation_percent = 0;
    s.config.automation.artist_separation_mins = 0;
    s
}

fn category(s: &mut AppState, name: &str) -> CategoryId {
    apply(
        s,
        Command::CreateCategory {
            parent: None,
            name: name.into(),
            color: None,
            kind: TrackKind::Music,
        },
    )
    .unwrap();
    s.categories.iter().map(|c| c.id).max().unwrap()
}

fn fill(s: &mut AppState, category: CategoryId, first: u64, n: u64) {
    for id in first..first + n {
        let mut t = Track::new(TrackId(id), PathBuf::from(format!("/m/{id}.flac")));
        t.category = Some(category);
        s.library.insert(t);
    }
}

fn rotation(s: &mut AppState, steps: Vec<RotationStep>) -> RotationId {
    apply(
        s,
        Command::CreateRotation {
            name: "R".into(),
            steps,
        },
    )
    .unwrap();
    s.rotations.iter().map(|r| r.id).max().unwrap()
}

fn step(category: CategoryId, min: u8, max: u8) -> RotationStep {
    RotationStep {
        source: StepSource::Category(category),
        min,
        max,
    }
}

/// The category of each track the cursor yields, `n` times.
fn walk(
    s: &AppState,
    cursor: &mut RotationCursor,
    n: usize,
    rng: &mut SplitMix64,
) -> Vec<CategoryId> {
    (0..n)
        .map(|_| {
            let p = next_track(s, cursor, NOW, &[], rng).picked.unwrap();
            s.library.get(p.track).unwrap().category.unwrap()
        })
        .collect()
}

#[test]
fn steps_play_in_order_and_wrap() {
    let mut s = state();
    let a = category(&mut s, "A");
    let b = category(&mut s, "B");
    fill(&mut s, a, 1, 3);
    fill(&mut s, b, 10, 3);
    let r = rotation(&mut s, vec![step(a, 1, 1), step(b, 1, 1)]);
    let mut cursor = RotationCursor::new(r);
    let got = walk(&s, &mut cursor, 5, &mut SplitMix64(3));
    assert_eq!(got, vec![a, b, a, b, a]);
}

#[test]
fn counts_are_drawn_in_their_range_each_time() {
    let mut s = state();
    let a = category(&mut s, "A");
    let b = category(&mut s, "B");
    fill(&mut s, a, 1, 3);
    fill(&mut s, b, 10, 3);
    let r = rotation(&mut s, vec![step(a, 2, 3), step(b, 1, 1)]);
    let mut cursor = RotationCursor::new(r);
    let got = walk(&s, &mut cursor, 200, &mut SplitMix64(9));
    // Lengths of the runs of A between the B's.
    let mut runs = Vec::new();
    let mut run = 0;
    for c in &got {
        if *c == a {
            run += 1;
        } else {
            assert_eq!(*c, b);
            runs.push(run);
            run = 0;
        }
    }
    assert!(runs.len() > 40);
    assert!(runs.iter().all(|r| *r == 2 || *r == 3), "{runs:?}");
    assert!(runs.contains(&2) && runs.contains(&3));
}

#[test]
fn a_step_on_a_deleted_category_is_skipped() {
    let mut s = state();
    let gone = category(&mut s, "Gone");
    let b = category(&mut s, "B");
    fill(&mut s, b, 10, 3);
    let r = rotation(&mut s, vec![step(gone, 1, 1), step(b, 1, 1)]);
    apply(&mut s, Command::DeleteCategory(gone)).unwrap();
    let mut cursor = RotationCursor::new(r);
    let out = next_track(&s, &mut cursor, NOW, &[], &mut SplitMix64(1));
    let picked = out.picked.unwrap();
    assert_eq!(s.library.get(picked.track).unwrap().category, Some(b));
    assert_eq!(out.skipped_steps, vec![0]);
    assert!(!out.used_default);
}

#[test]
fn an_unknown_rotation_uses_the_default() {
    let mut s = state();
    ensure_default(&mut s, "Music");
    let a = category(&mut s, "A");
    fill(&mut s, a, 1, 2);
    let mut cursor = RotationCursor::new(RotationId(4_242));
    let out = next_track(&s, &mut cursor, NOW, &[], &mut SplitMix64(1));
    assert!(out.picked.is_some());
    assert!(out.used_default);
}

#[test]
fn a_fruitless_rotation_falls_back_to_the_default_once() {
    let mut s = state();
    ensure_default(&mut s, "Music");
    let empty = category(&mut s, "Empty");
    let a = category(&mut s, "A");
    fill(&mut s, a, 1, 2);
    let r = rotation(&mut s, vec![step(empty, 1, 1)]);
    let mut cursor = RotationCursor::new(r);
    let before = cursor.clone();
    let out = next_track(&s, &mut cursor, NOW, &[], &mut SplitMix64(1));
    assert!(
        out.picked.is_some(),
        "the default rotation (all music) yields"
    );
    assert!(out.used_default);
    assert_eq!(out.skipped_steps, vec![0]);
    assert_eq!(
        cursor.rotation, before.rotation,
        "the caller's cursor keeps its rotation"
    );
}

#[test]
fn an_empty_library_ends_the_pass() {
    let mut s = state();
    ensure_default(&mut s, "Music");
    let a = category(&mut s, "A");
    let b = category(&mut s, "B");
    let r = rotation(&mut s, vec![step(a, 1, 2), step(b, 1, 1)]);
    let mut cursor = RotationCursor::new(r);
    let out = next_track(&s, &mut cursor, NOW, &[], &mut SplitMix64(1));
    assert_eq!(out.picked, None);
    let mut skipped = out.skipped_steps.clone();
    skipped.sort();
    assert_eq!(skipped, vec![0, 1]);
    assert!(out.used_default);
}

#[test]
fn a_rotation_without_steps_falls_back_to_the_default() {
    let mut s = state();
    ensure_default(&mut s, "Music");
    let a = category(&mut s, "A");
    fill(&mut s, a, 1, 1);
    let r = rotation(&mut s, vec![]);
    let mut cursor = RotationCursor::new(r);
    let out = next_track(&s, &mut cursor, NOW, &[], &mut SplitMix64(1));
    assert!(out.picked.is_some());
    assert!(out.used_default);
}
