#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The safe area: the screen less the system bars and the display cutout
//! (mobile spec MB10).

use egui::{Rect, pos2};
use fp_mobile::insets::{Insets, safe_area};

#[test]
fn safe_area_shrinks_by_the_insets_in_points() {
    let screen = Rect::from_min_max(pos2(0.0, 0.0), pos2(400.0, 800.0));
    let insets = Insets {
        left: 0.0,
        top: 96.0,
        right: 0.0,
        bottom: 48.0,
    };
    assert_eq!(
        safe_area(screen, insets, 2.0),
        Rect::from_min_max(pos2(0.0, 48.0), pos2(400.0, 776.0))
    );
}

#[test]
fn zero_insets_keep_the_screen() {
    let screen = Rect::from_min_max(pos2(0.0, 0.0), pos2(400.0, 800.0));
    assert_eq!(safe_area(screen, Insets::default(), 2.0), screen);
}
