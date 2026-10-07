#![allow(clippy::unwrap_used)]
//! Operator feedback 4, Q5: the blink shared by the player and the CUE window.

use egui::Color32;
use fp_app::ui::theme;
use fp_app::ui::widgets::{blink, paused_style};

#[test]
fn the_blink_is_on_for_half_a_second_then_off() {
    assert!(blink(0.0));
    assert!(blink(0.49));
    assert!(!blink(0.5));
    assert!(!blink(0.99));
    assert!(blink(1.0));
    assert!(!blink(1.5));
}

#[test]
fn a_broken_clock_never_panics() {
    let _ = blink(f64::NAN);
    let _ = blink(f64::INFINITY);
    let _ = blink(-3.2);
}

#[test]
fn the_paused_style_is_amber_when_lit_and_transparent_when_not() {
    let on = paused_style(true);
    assert_eq!(on.fill, theme::AMBER_BG);
    assert_eq!(on.content, theme::AMBER_TEXT);
    let off = paused_style(false);
    assert_eq!(off.fill, Color32::TRANSPARENT);
    assert_eq!(off.content, theme::AMBER_DIM);
}
