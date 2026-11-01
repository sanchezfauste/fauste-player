#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! A phone card's countdown and its meter and fader scales (phone layout
//! spec PL39).

use crate::support;

use egui::{Rect, Vec2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_app::ui::widgets;
use support::{harness_sized, state};

/// A narrow phone, a Samsung A56, a common phone and a large one, upright,
/// in dp.
const NARROW: Vec2 = vec2(360.0, 780.0);
const A56: Vec2 = vec2(384.0, 832.0);
const PORTRAIT: Vec2 = vec2(412.0, 892.0);
const LARGE: Vec2 = vec2(480.0, 1040.0);

/// One player whose tracks last `secs` each, on a phone of `size`.
fn card(size: Vec2, secs: f64) -> Harness<'static, AppUi> {
    card_with(size, secs, fp_model::MeterBallistics::DigitalPeak)
}

/// As [`card`], with a meter of type `meter`.
fn card_with(size: Vec2, secs: f64, meter: fp_model::MeterBallistics) -> Harness<'static, AppUi> {
    let mut s = state(1, 3);
    s.config.meter.ballistics = meter;
    for t in s.library.iter_mut() {
        t.duration_secs = secs;
    }
    // The meter's node is named "Level meter" without a loudness line.
    s.config.meter.loudness = fp_model::LoudnessReadout::Off;
    let (mut h, _fake) = harness_sized(s, size, |ui| ui.with_touch());
    h.run_steps(3);
    h
}

fn rect(h: &Harness<'static, AppUi>, label: &str) -> Rect {
    h.get_by_label(label).rect()
}

fn countdown(h: &Harness<'static, AppUi>, text: &str) -> Rect {
    rect(h, text)
}

/// The right edge of the transport: Pause ends the first row of small
/// buttons.
fn buttons_right(h: &Harness<'static, AppUi>) -> f32 {
    rect(h, "Pause").right()
}

fn meter_width(h: &Harness<'static, AppUi>) -> f32 {
    rect(h, "Level meter").width()
}

fn fader_width(h: &Harness<'static, AppUi>) -> f32 {
    h.get_all_by_label_contains("Volume")
        .next()
        .unwrap()
        .rect()
        .width()
}

/// PL39: the countdown ends where the transport buttons end, for a song
/// and for a ten-hour programme, on every phone width.
#[test]
fn the_countdown_ends_with_the_buttons() {
    for size in [NARROW, A56, PORTRAIT, LARGE] {
        for (secs, text) in [(180.0, "-03:00.0"), (36_000.0, "-10:00:00.0")] {
            let h = card(size, secs);
            let shown = countdown(&h, text);
            let right = buttons_right(&h);
            assert!(
                (shown.right() - right).abs() <= 0.5,
                "{size:?} {text}: the countdown ends at {} and the buttons at {right}",
                shown.right()
            );
        }
    }
}

/// PL39: a long countdown shrinks so it never runs into the elapsed time.
#[test]
fn a_ten_hour_countdown_stays_clear_of_the_elapsed_time() {
    for size in [NARROW, A56, PORTRAIT, LARGE] {
        let h = card(size, 36_000.0);
        let elapsed = rect(&h, "00:00 / 10:00:00");
        let shown = countdown(&h, "-10:00:00.0");
        assert!(
            shown.left() >= elapsed.right(),
            "{size:?}: {shown:?} overlaps {elapsed:?}"
        );
    }
}

/// PL39: a large phone has room for both scales: the meter's ruler and
/// the fader's numbers.
#[test]
fn a_large_phone_shows_both_scales() {
    let h = card(LARGE, 180.0);
    assert!(meter_width(&h) > widgets::COMPACT_METER_WIDTH);
    assert!(fader_width(&h) > widgets::COMPACT_FADER_WIDTH);
}

/// PL39: a common phone (412 dp) has room for the meter's scale but not
/// for the fader's as well: the meter's comes first.
#[test]
fn a_common_phone_shows_the_meter_scale_only() {
    let h = card(PORTRAIT, 180.0);
    assert!(meter_width(&h) > widgets::COMPACT_METER_WIDTH);
    assert_eq!(fader_width(&h), widgets::COMPACT_FADER_WIDTH);
}

/// The rects of the texts a frame drew.
fn texts(h: &Harness<'static, AppUi>) -> Vec<(String, Rect)> {
    fn walk(shape: &egui::Shape, out: &mut Vec<(String, Rect)>) {
        match shape {
            egui::Shape::Text(t) => {
                out.push((t.galley.text().to_owned(), t.visual_bounding_rect()))
            }
            egui::Shape::Vec(v) => v.iter().for_each(|s| walk(s, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    for c in &h.output().shapes {
        walk(&c.shape, &mut out);
    }
    out
}

/// PL39: the scales' labels stay within the meter and the fader, for the
/// widest labels too (EBU's TEST), on the phones that show them.
#[test]
fn the_scale_labels_stay_in_their_column() {
    for meter in [
        fp_model::MeterBallistics::DigitalPeak,
        fp_model::MeterBallistics::EbuPpm,
    ] {
        for size in [PORTRAIT, LARGE] {
            let h = card_with(size, 180.0, meter);
            let meter_rect = rect(&h, "Level meter");
            let fader = h.get_all_by_label_contains("Volume").next().unwrap().rect();
            let right = buttons_right(&h);
            let below_buttons = rect(&h, "Play").bottom() + 1.0;
            for (text, r) in texts(&h) {
                // Only what is drawn beside the buttons: the column. A
                // blank text has no bounds.
                if text.trim().is_empty()
                    || !r.is_finite()
                    || r.left() < right
                    || r.top() > below_buttons
                    || r.bottom() < meter_rect.top()
                {
                    continue;
                }
                assert!(
                    r.left() >= meter_rect.left() - 0.5 && r.right() <= fader.right() + 0.5,
                    "{meter:?} {size:?}: {text:?} at {r:?} is outside {meter_rect:?}..{fader:?}"
                );
            }
        }
    }
}

/// PL39: a narrow phone has no room beside the buttons: both scales stay
/// off, as before.
#[test]
fn a_narrow_phone_shows_no_scale() {
    let h = card(NARROW, 180.0);
    assert_eq!(meter_width(&h), widgets::COMPACT_METER_WIDTH);
    assert_eq!(fader_width(&h), widgets::COMPACT_FADER_WIDTH);
}

/// PL39: the meter's scale comes first: a width with room for one scale
/// shows the meter's, never the fader's alone.
#[test]
fn the_meter_scale_comes_before_the_fader_scale() {
    for width in (360..=520).step_by(4) {
        let h = card(vec2(width as f32, 900.0), 180.0);
        let meter = meter_width(&h) > widgets::COMPACT_METER_WIDTH;
        let fader = fader_width(&h) > widgets::COMPACT_FADER_WIDTH;
        assert!(
            meter || !fader,
            "{width} dp: the fader's scale without the meter's"
        );
    }
}

/// PL39: the scales never take room from the buttons or overlap them.
#[test]
fn the_scales_stay_clear_of_the_buttons() {
    for width in (360..=520).step_by(4) {
        let h = card(vec2(width as f32, 900.0), 180.0);
        let meter = rect(&h, "Level meter");
        assert!(
            meter.left() >= buttons_right(&h),
            "{width} dp: the meter at {} starts before the buttons end",
            meter.left()
        );
        let play = rect(&h, "Play");
        assert_eq!(play.width(), play.height(), "{width} dp: Play is square");
    }
}
