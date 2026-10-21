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
    // From -100 dB down the tenths go, so the readout stays as narrow as
    // "-60.0" (the bottom of the fader's travel reaches there).
    assert_eq!(
        volume_readout(10f32.powf(-114.2 / 20.0)).as_deref(),
        Some("-114")
    );
    assert_eq!(
        volume_readout(10f32.powf(-99.9 / 20.0)).as_deref(),
        Some("-99.9")
    );
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

/// The x of the centre of the text `text` and of the fader's knob on the
/// last frame.
fn readout_and_knob_x(h: &H, text: &str) -> (f32, f32) {
    let (centre, knob, _) = readout_knob_and_left(h, text);
    (centre, knob)
}

/// As `readout_and_knob_x`, with the left edge of the readout too.
fn readout_knob_and_left(h: &H, text: &str) -> (f32, f32, f32) {
    fn walk(
        shape: &egui::Shape,
        text: &str,
        readout: &mut Vec<f32>,
        knobs: &mut Vec<f32>,
        lefts: &mut Vec<f32>,
    ) {
        match shape {
            egui::Shape::Text(t) if t.galley.text() == text => {
                readout.push(t.pos.x + t.galley.size().x / 2.0);
                lefts.push(t.pos.x);
            }
            egui::Shape::Rect(r)
                if (r.rect.width() - 12.0).abs() < 0.01
                    && r.fill == fp_app::ui::theme::NEUTRAL_100 =>
            {
                knobs.push(r.rect.center().x);
            }
            egui::Shape::Vec(v) => v.iter().for_each(|s| walk(s, text, readout, knobs, lefts)),
            _ => {}
        }
    }
    let (mut readout, mut knobs, mut lefts) = (Vec::new(), Vec::new(), Vec::new());
    for clipped in &h.output().shapes {
        walk(&clipped.shape, text, &mut readout, &mut knobs, &mut lefts);
    }
    assert_eq!(readout.len(), 1, "one {text:?}: {readout:?}");
    assert!(!knobs.is_empty(), "a knob");
    (readout[0], knobs[0], lefts[0])
}

#[test]
fn the_readout_is_centred_on_the_knob() {
    let mut s = state(1, 1);
    s.players[0].volume = 0.4732;
    let (mut h, _) = harness(s);
    h.run_steps(2);
    let (readout, knob) = readout_and_knob_x(&h, "-6.5");
    assert!((readout - knob).abs() <= 0.5, "{readout} vs {knob}");
}

#[test]
fn the_widest_readout_stays_within_the_gap_before_the_meter() {
    let mut s = state(1, 1);
    s.players[0].volume = 10f32.powf(-60.0 / 20.0);
    let (mut h, _) = harness(s);
    h.run_steps(2);
    let (_, knob, left) = readout_knob_and_left(&h, "-60.0");
    // The fader's column starts 8 px left of the knob axis; the gap
    // before the meter is 6 px.
    let column_left = knob - 8.0;
    assert!(left >= column_left - 6.0, "{left} vs column {column_left}");
}
