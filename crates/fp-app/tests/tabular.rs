#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Times keep their width as they change (feedback spec F1).

use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use fp_app::ui::format;
use fp_app::ui::theme;
use fp_app::ui::widgets::{font, font_medium, tabular_size};

mod support;

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

/// Runs `measure` for each text with the app's fonts installed.
fn measured<T: Clone + 'static>(
    texts: Vec<String>,
    measure: impl Fn(&egui::Ui, &str) -> T + 'static,
) -> Vec<T> {
    let mut h = Harness::new_ui_state(
        move |ui, (themed, out): &mut (bool, Vec<T>)| {
            if !*themed {
                theme::apply(ui.ctx());
                *themed = true;
                return;
            }
            *out = texts.iter().map(|t| measure(ui, t)).collect();
        },
        (false, Vec::new()),
    );
    h.run_steps(3);
    h.state().1.clone()
}

/// The rectangle `paint_tabular_right` covers for each text.
fn right_rects(texts: Vec<String>, font: egui::FontId) -> Vec<egui::Rect> {
    measured(texts, move |ui, t| {
        fp_app::ui::widgets::paint_tabular_right(
            ui.painter(),
            egui::pos2(200.0, 10.0),
            t,
            &font,
            egui::Color32::WHITE,
        )
    })
}

fn strings(texts: &[&str]) -> Vec<String> {
    texts.iter().map(|t| (*t).to_owned()).collect()
}

fn badge_widths(texts: &[&str]) -> Vec<f32> {
    measured(strings(texts), |ui, t| {
        fp_app::ui::widgets::time_badge_width(ui.painter(), "INTRO", t)
    })
}

#[test]
fn a_right_aligned_time_keeps_its_left_edge_as_digits_change() {
    let r = right_rects(strings(&["-00:09", "-08:88", "-11:11"]), font(10.0));
    assert_eq!(r[0].right(), 200.0);
    assert_eq!(r[0].width(), r[1].width());
    assert_eq!(r[1].width(), r[2].width());
}

#[test]
fn hour_times_keep_their_width_inside_their_format() {
    let r = right_rects(strings(&["1:11:11", "8:08:08", "-1:11:11"]), font(10.0));
    assert_eq!(r[0].width(), r[1].width());
    assert!(r[2].width() > r[0].width()); // the sign is its own glyph
}

#[test]
fn a_nan_time_still_has_a_width() {
    // `format::clock` maps NaN and negatives to 00:00.
    let r = right_rects(vec![format::clock(f64::NAN)], font(10.0));
    assert!(r[0].width() > 0.0);
}

#[test]
fn the_intro_badge_keeps_its_width_from_9_9_to_10_0() {
    let w = badge_widths(&["9.9", "1.1", "8.8", "10.0", "12.3"]);
    assert_eq!(w[0], w[1]);
    assert_eq!(w[1], w[2]);
    assert_eq!(w[2], w[3]); // the box is sized for 00.0
    assert_eq!(w[3], w[4]);
}

#[test]
fn the_top_bar_clock_keeps_its_width() {
    let (mut h, _fake) = support::harness(support::state(1, 0));
    h.run_steps(2);
    let is_clock = |s: &str| {
        s.len() == 8
            && s.char_indices().all(|(i, c)| {
                if i == 2 || i == 5 {
                    c == ':'
                } else {
                    c.is_ascii_digit()
                }
            })
    };
    let clock = h
        .get_all_by_role(egui::accesskit::Role::Label)
        .find(|n| {
            n.accesskit_node()
                .value()
                .is_some_and(|l| is_clock(l.as_str()))
        })
        .expect("the clock label");
    let text = clock.accesskit_node().value().unwrap();
    let expected = measured(vec![text.to_string()], |ui, t| {
        fp_app::ui::widgets::tabular_size(ui.painter(), t, &font(13.0)).x
    })[0];
    let got = clock.rect().width();
    assert!((got - expected).abs() < 0.5, "{got} vs {expected}");
}
