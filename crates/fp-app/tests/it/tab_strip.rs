#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
//! The layout of the playlist tabs (feedback 2 spec O35).

use fp_app::ui::tab_strip::{ARROW_WIDTH, MIN_TAB_WIDTH, clamp_offset, layout, reveal, step};

#[test]
fn few_tabs_fill_the_strip_without_arrows() {
    let l = layout(600.0, 3);
    assert_eq!(l.tab_width, 200.0);
    assert!(!l.overflow);
    assert_eq!(l.view_width, 600.0);
    assert_eq!(layout(600.0, 1).tab_width, 600.0);
    assert!(!layout(600.0, 0).overflow);
}

#[test]
fn tabs_shrink_to_the_minimum_then_scroll() {
    let fits = layout(MIN_TAB_WIDTH * 4.0, 4);
    assert!(!fits.overflow);
    assert_eq!(fits.tab_width, MIN_TAB_WIDTH);
    let l = layout(MIN_TAB_WIDTH * 4.0, 5);
    assert!(l.overflow);
    assert_eq!(l.tab_width, MIN_TAB_WIDTH);
    assert_eq!(l.view_width, MIN_TAB_WIDTH * 4.0 - 2.0 * ARROW_WIDTH);
    assert_eq!(l.content_width, MIN_TAB_WIDTH * 5.0);
}

#[test]
fn a_strip_narrower_than_its_arrows_does_not_panic() {
    let l = layout(10.0, 30);
    assert!(l.overflow);
    assert_eq!(l.view_width, 0.0);
    assert_eq!(
        clamp_offset(5000.0, l.content_width, l.view_width),
        l.content_width
    );
    assert_eq!(reveal(0.0, 29, &l), l.content_width); // no negative or NaN
    let nan = layout(f32::NAN, 3);
    assert!(nan.tab_width.is_finite() && nan.view_width.is_finite());
}

#[test]
fn the_offset_is_clamped_when_the_tabs_get_fewer() {
    assert_eq!(clamp_offset(900.0, 360.0, 200.0), 160.0);
    assert_eq!(clamp_offset(-5.0, 360.0, 200.0), 0.0);
    assert_eq!(clamp_offset(30.0, 100.0, 200.0), 0.0); // fits: no scroll
}

#[test]
fn reveal_scrolls_the_least_that_shows_the_tab() {
    let l = layout(MIN_TAB_WIDTH * 3.0, 10); // overflow
    let view = l.view_width;
    // Already whole in view: unchanged.
    assert_eq!(reveal(0.0, 0, &l), 0.0);
    // Off to the right: its right edge meets the view's right edge.
    let o = reveal(0.0, 9, &l);
    assert!((o + view - 10.0 * MIN_TAB_WIDTH).abs() < 0.01, "{o}");
    // Off to the left: its left edge meets the view's left edge.
    let o = reveal(o, 2, &l);
    assert!((o - 2.0 * MIN_TAB_WIDTH).abs() < 0.01, "{o}");
}

#[test]
fn an_arrow_moves_one_tab_and_stops_at_the_ends() {
    let l = layout(MIN_TAB_WIDTH * 3.0, 10);
    assert_eq!(step(0.0, 1, &l), MIN_TAB_WIDTH);
    assert_eq!(step(0.0, -1, &l), 0.0);
    let max = l.content_width - l.view_width;
    assert_eq!(step(max - 5.0, 1, &l), max);
}
