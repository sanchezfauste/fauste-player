#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Itr05 UI polish: the BP badge, the settings buttons, the column arrows,
//! the mix marker colour.

use fp_app::ui::theme;
use fp_app::ui::widgets::bp_tile_style;

#[test]
fn the_bp_badge_is_green_on_and_grey_off() {
    let on = bp_tile_style(true);
    assert_eq!(on.border, theme::PLAY_BORDER);
    assert_eq!(on.content, theme::PLAY_HOVER);
    assert!(!on.dim_when_disabled, "drawn at full strength");
    let off = bp_tile_style(false);
    assert_eq!(off.border, theme::NEUTRAL_700);
    assert_eq!(off.content, theme::NEUTRAL_600);
}
