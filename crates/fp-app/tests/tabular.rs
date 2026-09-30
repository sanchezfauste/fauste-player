#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Times keep their width as they change (feedback spec F1).

use egui_kittest::Harness;
use fp_app::ui::theme;
use fp_app::ui::widgets::{font, font_medium, tabular_size};

/// Widths of `texts` in `font`, measured with the app's fonts installed:
/// (plain proportional layout, tabular layout).
fn widths(texts: &'static [&'static str], font: egui::FontId) -> Vec<(f32, f32)> {
    let mut h = Harness::new_ui_state(
        move |ui, (themed, out): &mut (bool, Vec<(f32, f32)>)| {
            if !*themed {
                // Fonts are bound from the next frame on (as in `AppUi::ui`).
                theme::apply(ui.ctx());
                *themed = true;
                return;
            }
            *out = texts
                .iter()
                .map(|t| {
                    let plain = ui
                        .painter()
                        .layout_no_wrap((*t).to_owned(), font.clone(), egui::Color32::WHITE)
                        .size()
                        .x;
                    (plain, tabular_size(ui.painter(), t, &font).x)
                })
                .collect();
        },
        (false, Vec::new()),
    );
    h.run_steps(3);
    h.state().1.clone()
}

#[test]
fn the_countdown_keeps_its_width_as_digits_change() {
    let w = widths(&["-00:00", "-11:11", "-88:88"], font_medium(38.0));
    // The proportional font really does move (the defect).
    assert!(w[1].0 < w[2].0, "{w:?}");
    // The tabular layout does not.
    assert_eq!(w[0].1, w[1].1);
    assert_eq!(w[1].1, w[2].1);
}

#[test]
fn tenths_and_small_times_keep_their_width() {
    let w = widths(&[".1", ".8"], font_medium(17.0));
    assert_eq!(w[0].1, w[1].1);
    let w = widths(&["-11:11", "-40:08", "1:11:11", "8:08:08"], font(12.0));
    assert_eq!(w[0].1, w[1].1);
    // The hour format is wider, but steady within itself.
    assert_eq!(w[2].1, w[3].1);
}
