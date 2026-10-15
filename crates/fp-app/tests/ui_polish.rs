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

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::widgets::{
    SETTINGS_BUTTON_HEIGHT, SETTINGS_BUTTON_PAD, TileStyle, font, padded_button_width, text_button,
};

#[test]
fn a_button_is_its_text_plus_the_padding_on_both_sides() {
    assert_eq!(padded_button_width(50.0), 50.0 + 2.0 * SETTINGS_BUTTON_PAD);
}

#[test]
fn a_settings_button_clears_its_text_by_the_padding_with_and_without_a_glyph() {
    for glyph in [None, Some(egui_phosphor::regular::FOLDER_OPEN)] {
        for label in ["OK", "Import M3U / PLS…", "Re-analyse all tracks"] {
            let seen = std::cell::Cell::new((0.0_f32, 0.0_f32, 0.0_f32));
            let mut h = Harness::new_ui(|ui| {
                let shown = match glyph {
                    Some(g) => format!("{g} {label}"),
                    None => label.to_owned(),
                };
                let text_width = ui
                    .painter()
                    .layout_no_wrap(shown, font(12.0), theme::TEXT)
                    .size()
                    .x;
                let r = text_button(ui, glyph, label, true, TileStyle::plain());
                seen.set((r.rect.width(), r.rect.height(), text_width));
            });
            h.run();
            let _ = h.get_by_label(label);
            let (width, height, text_width) = seen.get();
            assert!(
                width >= text_width + 2.0 * SETTINGS_BUTTON_PAD - 0.01,
                "{label}: {width} vs {text_width}"
            );
            assert_eq!(height, SETTINGS_BUTTON_HEIGHT);
        }
    }
}
