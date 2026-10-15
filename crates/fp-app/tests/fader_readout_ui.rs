#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The level readout under the player's volume fader.

mod support;

use egui_kittest::kittest::Queryable;
use fp_app::ui::view::volume_readout;
use support::{harness, lock_player, state};

type H = egui_kittest::Harness<'static, fp_app::ui::app::AppUi>;

/// The colour the text `text` is painted in on the last frame.
fn text_colour(h: &H, text: &str) -> egui::Color32 {
    fn find(shape: &egui::Shape, text: &str, out: &mut Vec<egui::Color32>) {
        match shape {
            egui::Shape::Text(t) if t.galley.text() == text => {
                out.push(t.galley.job.sections[0].format.color);
            }
            egui::Shape::Vec(v) => v.iter().for_each(|s| find(s, text, out)),
            _ => {}
        }
    }
    let mut found = Vec::new();
    for clipped in &h.output().shapes {
        find(&clipped.shape, text, &mut found);
    }
    assert_eq!(found.len(), 1, "one {text:?} painted: {found:?}");
    found[0]
}

#[test]
fn the_readout_has_one_decimal_and_no_negative_zero() {
    assert_eq!(volume_readout(1.0).as_deref(), Some("0.0"));
    assert_eq!(volume_readout(0.4732).as_deref(), Some("-6.5"));
    assert_eq!(volume_readout(0.9999).as_deref(), Some("0.0"));
    assert_eq!(volume_readout(0.0), None);
    assert_eq!(volume_readout(f32::NAN), None);
}

#[test]
fn the_fader_shows_its_level_in_db() {
    let mut s = state(1, 1);
    s.players[0].volume = 0.4732;
    let (mut h, _) = harness(s);
    h.run_steps(2);
    h.get_by_label("-6.5");
    assert_eq!(text_colour(&h, "-6.5"), fp_app::ui::theme::NEUTRAL_400);
}

#[test]
fn a_silent_fader_shows_minus_infinity() {
    let mut s = state(1, 1);
    s.players[0].volume = 0.0;
    let (mut h, _) = harness(s);
    h.run_steps(2);
    h.get_by_label("−∞");
}

#[test]
fn a_locked_fader_shows_zero_dimmed() {
    let mut s = state(1, 1);
    s.players[0].volume = 0.5;
    lock_player(&mut s, 0);
    let (mut h, _) = harness(s);
    h.run_steps(2);
    h.get_by_label("0.0");
    assert_eq!(text_colour(&h, "0.0"), fp_app::ui::theme::NEUTRAL_600);
}
