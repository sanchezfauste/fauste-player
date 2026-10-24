#![allow(clippy::indexing_slicing)]
//! The header's fit: the lowest-priority items collapse first.

use fp_app::ui::header_layout::{Item, fit};

fn item(width: f32, rank: u8) -> Item {
    Item { width, rank }
}

#[test]
fn everything_stays_when_it_fits() {
    let items = [item(20.0, 0), item(30.0, 3), item(40.0, 2)];
    // 20 + 30 + 40 + 2 gaps of 6.
    assert_eq!(fit(&items, 102.0, 6.0), vec![true, true, true]);
}

#[test]
fn the_highest_rank_collapses_first() {
    let items = [item(20.0, 0), item(30.0, 3), item(40.0, 2)];
    assert_eq!(fit(&items, 101.0, 6.0), vec![true, false, true]);
    // 20 + 40 + 1 gap.
    assert_eq!(fit(&items, 66.0, 6.0), vec![true, false, true]);
    assert_eq!(fit(&items, 65.0, 6.0), vec![true, false, false]);
}

#[test]
fn equal_ranks_collapse_from_the_right() {
    let items = [item(20.0, 0), item(30.0, 2), item(30.0, 2)];
    assert_eq!(fit(&items, 70.0, 6.0), vec![true, true, false]);
}

#[test]
fn rank_zero_is_kept_even_when_alone_it_does_not_fit() {
    let items = [item(20.0, 0), item(30.0, 1)];
    assert_eq!(fit(&items, 5.0, 6.0), vec![true, false]);
    assert!(fit(&[], 5.0, 6.0).is_empty());
}

#[test]
fn the_shown_items_never_exceed_the_room() {
    let items = [
        item(22.0, 0),
        item(7.0, 8),
        item(55.0, 9),
        item(92.0, 4),
        item(60.0, 5),
        item(80.0, 6),
        item(70.0, 7),
    ];
    for room in 0..450 {
        let room = room as f32;
        let shown = fit(&items, room, 6.0);
        let widths: Vec<f32> = items
            .iter()
            .zip(&shown)
            .filter(|(_, s)| **s)
            .map(|(i, _)| i.width)
            .collect();
        let total: f32 = widths.iter().sum::<f32>() + 6.0 * (widths.len() as f32 - 1.0).max(0.0);
        assert!(
            total <= room || widths.len() == 1,
            "{room}: {total} for {shown:?}"
        );
    }
}
