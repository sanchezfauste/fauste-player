#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! What the level meter draws (meters spec M4).

mod support;

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable;
use fp_app::ui::widgets::{
    MeterTick, Side, TickKind, Zone, alignment_dbfs, label_column, loudness_line, mark_label,
    max_readout, meter_layout, meter_position, meter_width, minor_marks, scale_marks, scale_zero,
    tick_colour, zone_of,
};
use fp_engine::conductor::Telemetry;
use fp_engine::engine::PlayerTelemetry;
use fp_engine::meter::MeterReading;
use fp_model::{Command, LoudnessReadout, MeterBallistics, MeterConfig};
use support::{harness, state};

fn reading(level: f32, hold: f32) -> MeterReading {
    MeterReading {
        level_db: [level; 2],
        hold_db: [hold; 2],
        ..MeterReading::default()
    }
}

fn meter(ballistics: MeterBallistics) -> MeterConfig {
    MeterConfig {
        ballistics,
        ..MeterConfig::default() // floor −60, alignment −18 dBFS
    }
}

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-3
}

#[test]
fn the_digital_scale_is_the_iec_60268_18_deflection() {
    let c = meter(MeterBallistics::DigitalPeak);
    // In % of 0 dBFS: −20 at 50, −40 at 15, −60 at 2.5 (the floor).
    assert_eq!(meter_position(0.0, &c), 1.0);
    assert!(near(meter_position(-20.0, &c), 47.5 / 97.5));
    assert!(near(meter_position(-40.0, &c), 12.5 / 97.5));
    assert_eq!(meter_position(-60.0, &c), 0.0);
    assert_eq!(meter_position(-80.0, &c), 0.0);
    assert_eq!(
        scale_marks(&c),
        vec![
            -60.0, -50.0, -40.0, -35.0, -30.0, -25.0, -20.0, -15.0, -10.0, -5.0, 0.0
        ]
    );
}

#[test]
fn the_ebu_scale_runs_linearly_from_minus_12_to_plus_12() {
    let c = meter(MeterBallistics::EbuPpm);
    assert_eq!(meter_position(-30.0, &c), 0.0, "−12");
    assert!(near(meter_position(-18.0, &c), 0.5), "TEST");
    assert_eq!(meter_position(-6.0, &c), 1.0, "+12");
    assert_eq!(meter_position(-40.0, &c), 0.0);
    assert_eq!(
        scale_marks(&c),
        vec![-30.0, -26.0, -22.0, -18.0, -14.0, -10.0, -6.0]
    );
}

#[test]
fn the_din_scale_follows_the_fourth_root_of_the_voltage() {
    let c = meter(MeterBallistics::DinPpm);
    // 0 dB (permitted maximum) is 9 dB above alignment: −9 dBFS.
    assert_eq!(meter_position(-4.0, &c), 1.0, "+5");
    assert_eq!(meter_position(-59.0, &c), 0.0, "−50");
    let quarter = |db: f32| 10f32.powf(db / 80.0);
    let expected = (quarter(-5.0) - quarter(-55.0)) / (1.0 - quarter(-55.0));
    assert!(near(meter_position(-9.0, &c), expected), "0 dB");
    assert_eq!(
        scale_marks(&c),
        vec![-59.0, -49.0, -39.0, -29.0, -19.0, -14.0, -9.0, -4.0]
    );
}

#[test]
fn the_vu_scale_is_proportional_to_the_voltage() {
    let c = meter(MeterBallistics::Vu);
    assert_eq!(meter_position(-15.0, &c), 1.0, "+3 VU");
    assert!(
        near(meter_position(-18.0, &c), 10f32.powf(-3.0 / 20.0)),
        "0 VU"
    );
    assert!(
        near(meter_position(-38.0, &c), 10f32.powf(-23.0 / 20.0)),
        "−20 VU"
    );
    assert_eq!(scale_marks(&c).len(), 11);
    assert_eq!(scale_marks(&c)[0], -38.0);
}

#[test]
fn the_k_scale_is_linear_from_the_top_to_minus_24() {
    let c = meter(MeterBallistics::K20);
    assert_eq!(meter_position(0.0, &c), 1.0, "+20");
    assert!(near(meter_position(-44.0, &c), 0.2), "−24");
    assert!(
        near(meter_position(-20.0, &c), 0.2 + 0.8 * 24.0 / 44.0),
        "0"
    );
    assert!(near(meter_position(-62.0, &c), 0.1), "−42");
    assert_eq!(meter_position(-80.0, &c), 0.0, "−60");
    let marks = scale_marks(&meter(MeterBallistics::K12));
    assert_eq!(marks.first(), Some(&-72.0));
    assert!(marks.contains(&-12.0) && marks.contains(&-8.0) && marks.contains(&0.0));
}

#[test]
fn every_scale_is_monotonic_and_ends_at_its_top() {
    for ballistics in [
        MeterBallistics::DigitalPeak,
        MeterBallistics::EbuPpm,
        MeterBallistics::DinPpm,
        MeterBallistics::Vu,
        MeterBallistics::K20,
        MeterBallistics::K14,
        MeterBallistics::K12,
        MeterBallistics::Custom,
    ] {
        for floor in [-96.0, -60.0, -30.0, -20.0] {
            let c = MeterConfig {
                ballistics,
                floor_db: floor,
                ..MeterConfig::default()
            };
            let mut last = 0.0;
            let mut db = -120.0;
            while db <= 6.0 {
                let p = meter_position(db, &c);
                assert!((0.0..=1.0).contains(&p), "{ballistics:?} {db}: {p}");
                assert!(
                    p >= last,
                    "{ballistics:?}, floor {floor}, {db} dB: {p} < {last}"
                );
                last = p;
                db += 0.25;
            }
            assert_eq!(last, 1.0, "{ballistics:?}: overs sit at the top");
        }
    }
}

#[test]
fn levels_take_the_colour_of_their_zone() {
    let c = MeterConfig::default(); // warning −9, danger −3
    assert_eq!(zone_of(-20.0, &c), Zone::Normal);
    assert_eq!(zone_of(-9.0, &c), Zone::Warning);
    assert_eq!(zone_of(-3.0, &c), Zone::Danger);
    assert_eq!(zone_of(0.4, &c), Zone::Danger);
}

#[test]
fn the_k_system_has_its_own_zones() {
    let c = meter(MeterBallistics::K20);
    assert_eq!(zone_of(-20.5, &c), Zone::Normal, "below 0");
    assert_eq!(zone_of(-20.0, &c), Zone::Warning, "0 to +4: amber");
    assert_eq!(
        zone_of(-16.0, &c),
        Zone::Warning,
        "+4 itself is still amber"
    );
    assert_eq!(zone_of(-15.9, &c), Zone::Danger, "above +4");
}

#[test]
fn the_digital_scale_reaches_down_to_a_low_floor() {
    let c = MeterConfig {
        floor_db: -96.0,
        ..meter(MeterBallistics::DigitalPeak)
    };
    let (p96, p90, p80, p70) = (
        meter_position(-96.0, &c),
        meter_position(-90.0, &c),
        meter_position(-80.0, &c),
        meter_position(-70.0, &c),
    );
    assert_eq!(p96, 0.0, "the floor is the bottom");
    assert!(0.0 < p90 && p90 < p80 && p80 < p70, "{p90} {p80} {p70}");
}

#[test]
fn the_maximum_reads_with_one_decimal() {
    assert_eq!(max_readout(-3.24), "-3.2");
    assert_eq!(max_readout(0.4), "+0.4");
    assert_eq!(max_readout(-0.04), "0.0");
    assert_eq!(max_readout(-120.0), "—", "nothing measured");
    assert_eq!(max_readout(f32::NAN), "—", "never a NaN on screen");
}

/// Label rows are this far apart at least (monospace 9 px text).
const LABEL_ROW: f32 = 10.0;

/// The rect of a default (digital) meter `height` tall.
fn column(height: f32) -> egui::Rect {
    column_for(&MeterConfig::default(), height)
}

/// The rect of meter `c`, `height` tall.
fn column_for(c: &MeterConfig, height: f32) -> egui::Rect {
    egui::Rect::from_min_size(egui::pos2(100.0, 20.0), egui::vec2(meter_width(c), height))
}

#[test]
fn labels_never_overlap_and_the_alignment_line_stays() {
    for ballistics in ALL_METERS {
        for height in [64.0, 136.0] {
            for loudness in [false, true] {
                let c = meter(ballistics);
                let l = meter_layout(column_for(&c, height), &c, loudness);
                let ys: Vec<f32> = l
                    .lines
                    .iter()
                    .filter(|m| !m.label.is_empty())
                    .map(|m| m.label_centre())
                    .collect();
                for pair in ys.windows(2) {
                    assert!(
                        pair[1] - pair[0] >= LABEL_ROW,
                        "{ballistics:?} {height} {loudness}: {ys:?}"
                    );
                }
                assert_eq!(
                    l.lines.iter().filter(|m| m.alignment).count(),
                    1,
                    "{ballistics:?} {height}"
                );
            }
        }
    }
}

#[test]
fn end_labels_stay_inside_the_rect_on_their_line() {
    for ballistics in ALL_METERS {
        for height in [64.0, 136.0, 300.0] {
            for loudness in [false, true] {
                let c = meter(ballistics);
                let rect = column_for(&c, height);
                let l = meter_layout(rect, &c, loudness);
                for m in l.lines.iter().filter(|m| !m.label.is_empty()) {
                    let centre = m.label_centre();
                    assert!(
                        centre - LABEL_ROW / 2.0 >= rect.top() - 0.01
                            && centre + LABEL_ROW / 2.0 <= rect.bottom() + 0.01,
                        "{ballistics:?} {height} {loudness}: {} outside the rect",
                        m.label
                    );
                    // Never shifted off its line: the line is within the
                    // label's own height.
                    assert!(
                        (centre - m.y).abs() <= LABEL_ROW / 2.0 + 0.01,
                        "{ballistics:?} {height}: {} is off its line",
                        m.label
                    );
                    assert_eq!(m.label_y, m.y, "the anchor is the line");
                }
            }
        }
    }
    // Without the loudness line the bars end at the rect's bottom, so the
    // bottom label rests on its line instead of being centred over it.
    let c = meter(MeterBallistics::DigitalPeak);
    let l = meter_layout(column_for(&c, 136.0), &c, false);
    let bottom = l.lines.iter().find(|m| m.label == "-60").unwrap();
    assert_eq!(bottom.label_align, egui::Align::Max);
    // With room above and below, a label is centred on its line.
    let middle = l.lines.iter().find(|m| m.label == "-30").unwrap();
    assert_eq!(middle.label_align, egui::Align::Center);
}

#[test]
fn labels_stay_between_the_readouts() {
    for loudness in [false, true] {
        let l = meter_layout(
            column(136.0),
            &meter(MeterBallistics::DigitalPeak),
            loudness,
        );
        let bars = l.bars[0];
        let rect = column(136.0);
        for m in &l.lines {
            assert!(
                m.label_centre() - LABEL_ROW / 2.0 >= rect.top() - 0.01,
                "{}",
                m.label
            );
            assert!(
                m.label_centre() + LABEL_ROW / 2.0 <= rect.bottom() + 0.01,
                "{}",
                m.label
            );
        }
        assert!(l.max.bottom() <= bars.top());
        if let Some(r) = l.loudness {
            assert!(r.top() >= bars.bottom());
        }
        assert_eq!(l.loudness.is_some(), loudness);
    }
}

#[test]
fn both_ends_of_every_scale_are_labelled() {
    for ballistics in ALL_METERS {
        for loudness in [false, true] {
            let c = meter(ballistics);
            let marks = scale_marks(&c);
            let l = meter_layout(column_for(&c, 136.0), &c, loudness);
            let labels: Vec<&str> = l.lines.iter().map(|m| m.label.as_str()).collect();
            for end in [marks.first().unwrap(), marks.last().unwrap()] {
                let want = mark_label(*end, &c);
                assert!(
                    labels.contains(&want.as_str()),
                    "{ballistics:?}: {want} in {labels:?}"
                );
            }
        }
    }
}

#[test]
fn the_digital_floor_is_always_the_bottom_mark() {
    for ballistics in [MeterBallistics::DigitalPeak, MeterBallistics::Custom] {
        for floor in [-96.0, -90.0, -55.0, -52.0, -45.0, -20.0] {
            let c = MeterConfig {
                ballistics,
                floor_db: floor,
                reference_dbfs: floor + 10.0,
                ..MeterConfig::default()
            };
            let marks = scale_marks(&c);
            assert_eq!(marks.first(), Some(&floor), "{ballistics:?} {floor}");
            assert_eq!(marks.last(), Some(&0.0));
            assert!(
                marks.windows(2).all(|w| w[0] < w[1]),
                "strictly increasing: {marks:?}"
            );
        }
    }
}

/// Spec O11: every meter type at 64, 136 and 300 px, with and without the
/// loudness line, and for several digital floors.
#[test]
fn both_ends_stay_labelled_at_every_height() {
    for ballistics in ALL_METERS {
        for floor in [-96.0, -60.0, -55.0, -20.0] {
            for height in [64.0, 136.0, 300.0] {
                for loudness in [false, true] {
                    let c = MeterConfig {
                        ballistics,
                        floor_db: floor,
                        reference_dbfs: if floor > -30.0 { floor + 5.0 } else { -18.0 },
                        ..MeterConfig::default()
                    };
                    let marks = scale_marks(&c);
                    let l = meter_layout(column_for(&c, height), &c, loudness);
                    let labelled: Vec<&str> = l
                        .lines
                        .iter()
                        .filter(|m| !m.label.is_empty())
                        .map(|m| m.label.as_str())
                        .collect();
                    for end in [marks.first().unwrap(), marks.last().unwrap()] {
                        let want = mark_label(*end, &c);
                        assert!(
                            labelled.contains(&want.as_str()),
                            "{ballistics:?} floor {floor} {height}px loudness {loudness}: \
                             {want} in {labelled:?}"
                        );
                    }
                    // Every label has its line, and the lines are on the bars.
                    for m in l.lines.iter().filter(|m| !m.label.is_empty()) {
                        assert!(m.y >= l.bars[0].top() && m.y <= l.bars[0].bottom());
                    }
                }
            }
        }
    }
}

/// An end that is also the alignment level keeps its label when the other
/// end crowds the alignment line.
#[test]
fn an_end_that_is_the_alignment_level_stays_labelled() {
    for ballistics in [MeterBallistics::DigitalPeak, MeterBallistics::Custom] {
        for (floor, reference) in [
            (-20.0, -20.0),
            (-40.0, -40.0),
            (-30.0, -30.0),
            (-60.0, 0.0),
            (-20.0, 0.0),
            (-60.0, -60.0),
            (-30.0, 0.0),
        ] {
            for height in [24.0, 32.0, 40.0, 48.0, 64.0, 136.0, 300.0] {
                for loudness in [false, true] {
                    let c = MeterConfig {
                        ballistics,
                        floor_db: floor,
                        reference_dbfs: reference,
                        ..MeterConfig::default()
                    };
                    let marks = scale_marks(&c);
                    let l = meter_layout(column_for(&c, height), &c, loudness);
                    let labelled: Vec<&str> = l
                        .lines
                        .iter()
                        .filter(|m| !m.label.is_empty())
                        .map(|m| m.label.as_str())
                        .collect();
                    for end in [marks.first().unwrap(), marks.last().unwrap()] {
                        // Below 64 px the two ends cannot both fit: the one
                        // that is the alignment level is the one kept.
                        let is_alignment = (end - alignment_dbfs(&c)).abs() < 1.0;
                        if height < 64.0 && !is_alignment {
                            continue;
                        }
                        let want = mark_label(*end, &c);
                        assert!(
                            labelled.contains(&want.as_str()),
                            "{ballistics:?} floor {floor} {height}px loudness {loudness}: \
                             {want} in {labelled:?}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn a_tall_digital_meter_labels_its_main_marks() {
    let l = meter_layout(
        column_for(&meter(MeterBallistics::DigitalPeak), 136.0),
        &meter(MeterBallistics::DigitalPeak),
        false,
    );
    let labels: Vec<&str> = l.lines.iter().map(|m| m.label.as_str()).collect();
    for want in ["0", "-10", "-20", "-30", "-40", "-60"] {
        assert!(labels.contains(&want), "{labels:?}");
    }
    // A very tall meter labels every mark.
    let c = meter(MeterBallistics::DigitalPeak);
    let tall = meter_layout(column_for(&c, 600.0), &c, false);
    let tall_labels: Vec<&str> = tall.lines.iter().map(|m| m.label.as_str()).collect();
    for mark in scale_marks(&c) {
        let want = mark_label(mark, &c);
        assert!(
            tall_labels.contains(&want.as_str()),
            "{want} in {tall_labels:?}"
        );
    }
    // The digital meter's alignment (−18 dBFS) is a heavier, unlabelled
    // line; the scale's round labels stay.
    let alignment = l.lines.iter().find(|m| m.alignment).unwrap();
    assert!(alignment.label.is_empty());
}

/// The labels a layout shows, top down.
fn shown_labels(l: &fp_app::ui::widgets::MeterLayout) -> Vec<String> {
    l.lines
        .iter()
        .filter(|m| !m.label.is_empty())
        .map(|m| m.label.clone())
        .collect()
}

/// The labels of the scale's own marks.
fn mark_labels(c: &MeterConfig) -> Vec<String> {
    scale_marks(c).iter().map(|m| mark_label(*m, c)).collect()
}

#[test]
fn a_tall_digital_meter_labels_every_db_near_the_top() {
    for ballistics in [MeterBallistics::DigitalPeak, MeterBallistics::Custom] {
        let c = meter(ballistics);
        let l = meter_layout(column_for(&c, 600.0), &c, false);
        let labels = shown_labels(&l);
        for db in -5..=0 {
            let want = db.to_string();
            assert!(
                labels.contains(&want),
                "{ballistics:?}: {want} in {labels:?}"
            );
        }
        // Each level is labelled once, the alignment level included.
        let mut unique = labels.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), labels.len(), "{labels:?}");
    }
}

#[test]
fn a_short_digital_meter_keeps_the_round_labels() {
    for ballistics in [MeterBallistics::DigitalPeak, MeterBallistics::Custom] {
        for height in [64.0, 136.0] {
            for loudness in [false, true] {
                let c = meter(ballistics);
                let l = meter_layout(column_for(&c, height), &c, loudness);
                let marks = mark_labels(&c);
                for label in shown_labels(&l) {
                    assert!(
                        marks.contains(&label),
                        "{ballistics:?} {height}: {label} is not a scale mark"
                    );
                }
            }
        }
    }
}

#[test]
fn intermediate_labels_never_crowd_at_any_height() {
    for ballistics in [MeterBallistics::DigitalPeak, MeterBallistics::Custom] {
        for (floor, reference) in [
            (-60.0, -18.0),
            (-96.0, -18.0),
            (-48.0, -20.0),
            (-20.0, -9.0),
        ] {
            for height in (24..=900).step_by(7) {
                for loudness in [false, true] {
                    let c = MeterConfig {
                        ballistics,
                        floor_db: floor,
                        reference_dbfs: reference,
                        ..MeterConfig::default()
                    };
                    let l = meter_layout(column_for(&c, height as f32), &c, loudness);
                    let ys: Vec<f32> = l
                        .lines
                        .iter()
                        .filter(|m| !m.label.is_empty())
                        .map(|m| m.label_centre())
                        .collect();
                    for pair in ys.windows(2) {
                        assert!(
                            pair[1] - pair[0] >= LABEL_ROW - 0.01,
                            "{ballistics:?} floor {floor} {height}px {loudness}: {ys:?}"
                        );
                    }
                    // Every label is a whole dB with a major tick.
                    for m in l.lines.iter().filter(|m| !m.label.is_empty()) {
                        assert!(
                            l.ticks
                                .iter()
                                .any(|t| near(t.y, m.y) && t.kind != TickKind::Minor),
                            "{ballistics:?} {height}: no major tick for {}",
                            m.label
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn intermediate_labels_follow_a_custom_floor() {
    let c = MeterConfig {
        ballistics: MeterBallistics::Custom,
        floor_db: -48.0,
        reference_dbfs: -20.0,
        ..MeterConfig::default()
    };
    let labels = shown_labels(&meter_layout(column_for(&c, 900.0), &c, false));
    for want in ["-48", "-45", "-40", "-20", "-1", "0"] {
        assert!(labels.contains(&want.to_owned()), "{want} in {labels:?}");
    }
    // Below -20 dBFS the scale's spacing is 5 dB: no finer labels.
    for unwanted in ["-47", "-46", "-44", "-42"] {
        assert!(
            !labels.contains(&unwanted.to_owned()),
            "{unwanted} in {labels:?}"
        );
    }
}

#[test]
fn standard_scales_keep_their_own_labels_at_every_height() {
    for ballistics in [
        MeterBallistics::EbuPpm,
        MeterBallistics::DinPpm,
        MeterBallistics::Vu,
        MeterBallistics::K20,
        MeterBallistics::K14,
        MeterBallistics::K12,
    ] {
        let c = meter(ballistics);
        let marks = mark_labels(&c);
        for height in [64.0, 136.0, 300.0, 600.0, 900.0] {
            let labels = shown_labels(&meter_layout(column_for(&c, height), &c, false));
            for label in &labels {
                assert!(marks.contains(label), "{ballistics:?} {height}: {label}");
            }
            if height >= 600.0 {
                assert_eq!(labels.len(), marks.len(), "{ballistics:?}: {labels:?}");
            }
        }
    }
}

#[test]
fn each_meter_labels_its_own_units() {
    let ebu = meter(MeterBallistics::EbuPpm);
    assert_eq!(mark_label(ebu.reference_dbfs + 8.0, &ebu), "+8");
    assert_eq!(mark_label(ebu.reference_dbfs, &ebu), "TEST");
    let din = meter(MeterBallistics::DinPpm);
    assert_eq!(mark_label(din.reference_dbfs + 9.0, &din), "0");
    assert_eq!(mark_label(din.reference_dbfs + 9.0 - 50.0, &din), "-50");
    let vu = meter(MeterBallistics::Vu);
    assert_eq!(mark_label(vu.reference_dbfs - 20.0, &vu), "-20");
    assert_eq!(mark_label(vu.reference_dbfs + 3.0, &vu), "+3");
    let k = meter(MeterBallistics::K14);
    assert_eq!(mark_label(-14.0, &k), "0");
    let layout = meter_layout(column_for(&k, 136.0), &k, false);
    let alignment = layout.lines.iter().find(|m| m.alignment).unwrap();
    assert_eq!(alignment.label, "0", "K-System names its alignment level");
    assert_eq!(layout.lines.iter().filter(|m| m.label == "0").count(), 1);
    assert_eq!(mark_label(-10.0, &k), "+4");
    assert_eq!(mark_label(0.0, &k), "+14");
    let d = meter(MeterBallistics::DigitalPeak);
    assert_eq!(mark_label(-18.0, &d), "-18");
    assert_eq!(mark_label(0.0, &d), "0");
}

#[test]
fn clicking_the_maximum_restarts_it() {
    let (mut h, fake) = harness(state(1, 1));
    let player = fake.player(0);
    fake.telemetry.store(std::sync::Arc::new(Telemetry {
        players: vec![(
            player,
            PlayerTelemetry {
                meter: MeterReading {
                    max_db: -3.2,
                    ..reading(-12.0, -6.0)
                },
                ..PlayerTelemetry::default()
            },
        )],
        ..Telemetry::default()
    }));
    h.run_steps(2);
    h.get_by_label("Maximum -3.2 dBFS").click();
    h.run_steps(2);
    assert_eq!(*fake.meter_resets.lock().unwrap(), vec![player]);
}

#[test]
fn the_loudness_line_shows_lufs() {
    let mut c = MeterConfig::default();
    let r = MeterReading {
        short_term_lufs: Some(-23.4),
        momentary_lufs: Some(-19.0),
        ..MeterReading::default()
    };
    let (text, on_target) = loudness_line(&r, &c).unwrap();
    assert_eq!(text, "-23.4");
    assert!(on_target, "within ±1 LU of −23");
    c.loudness = LoudnessReadout::Momentary;
    let (text, on_target) = loudness_line(&r, &c).unwrap();
    assert_eq!(text, "-19.0");
    assert!(!on_target);
    c.loudness = LoudnessReadout::Off;
    assert_eq!(loudness_line(&r, &c), None);
    c.loudness = LoudnessReadout::ShortTerm;
    let (text, _) = loudness_line(&reading(-120.0, -120.0), &c).unwrap();
    assert_eq!(text, "—", "nothing measured");
}

#[test]
fn the_player_meter_reads_its_telemetry() {
    let (mut h, fake) = harness(state(1, 1));
    let player = fake.player(0);
    let meter = MeterReading {
        short_term_lufs: Some(-22.5),
        ..reading(-12.0, -6.0)
    };
    fake.telemetry.store(std::sync::Arc::new(Telemetry {
        players: vec![(
            player,
            PlayerTelemetry {
                meter,
                ..PlayerTelemetry::default()
            },
        )],
        ..Telemetry::default()
    }));
    h.run_steps(2);
    assert!(h.query_by_label("Loudness -22.5 LUFS").is_some());
}

#[test]
fn choosing_a_vu_meter_updates_the_config() {
    let (mut h, fake) = harness(state(1, 1));
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Meters").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "VU (IEC 60268-17)")
        .click();
    h.run_steps(2);
    assert_eq!(
        fake.state.load().config.meter.ballistics,
        MeterBallistics::Vu
    );
    assert!(
        fake.take_sent()
            .iter()
            .any(|c| matches!(c, Command::UpdateConfig(_)))
    );
}

#[test]
fn k_system_meters_can_be_chosen() {
    let (mut h, fake) = harness(state(1, 1));
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Meters").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "K-14 (K-System)")
        .click();
    h.run_steps(2);
    assert_eq!(
        fake.state.load().config.meter.ballistics,
        MeterBallistics::K14
    );
}

#[test]
fn a_long_loudness_value_fits_the_meter() {
    let c = MeterConfig::default();
    let r = MeterReading {
        short_term_lufs: Some(-100.3),
        ..MeterReading::default()
    };
    let (text, _) = loudness_line(&r, &c).unwrap();
    assert!(text.chars().count() <= 5, "{text}");
}

const ALL_METERS: [MeterBallistics; 8] = [
    MeterBallistics::DigitalPeak,
    MeterBallistics::EbuPpm,
    MeterBallistics::DinPpm,
    MeterBallistics::Vu,
    MeterBallistics::K20,
    MeterBallistics::K14,
    MeterBallistics::K12,
    MeterBallistics::Custom,
];

#[test]
fn every_meter_shows_its_alignment_level() {
    for ballistics in ALL_METERS {
        let c = meter(ballistics);
        let at = alignment_dbfs(&c);
        let p = meter_position(at, &c);
        assert!(p > 0.0 && p < 1.0, "{ballistics:?}: {at} dBFS at {p}");
    }
    assert_eq!(alignment_dbfs(&meter(MeterBallistics::DigitalPeak)), -18.0);
    assert_eq!(alignment_dbfs(&meter(MeterBallistics::K14)), -14.0);
}

#[test]
fn programme_meters_turn_red_where_their_scale_does() {
    // VU: the red arc from 0 VU; PPMs: from the permitted maximum, 9 dB
    // above alignment (DIN 0, EBU +9). All inside their scales.
    let vu = meter(MeterBallistics::Vu);
    assert_eq!(zone_of(-18.5, &vu), Zone::Normal);
    assert_eq!(zone_of(-18.0, &vu), Zone::Danger);
    for ppm in [MeterBallistics::DinPpm, MeterBallistics::EbuPpm] {
        let c = meter(ppm);
        assert_eq!(zone_of(-9.5, &c), Zone::Normal, "{ppm:?}");
        assert_eq!(zone_of(-9.0, &c), Zone::Danger, "{ppm:?}");
        assert!(meter_position(-9.0, &c) < 1.0, "{ppm:?}: on the scale");
    }
}

#[test]
fn the_meter_and_fader_fill_the_height_they_are_given() {
    let mut h = egui_kittest::Harness::new_ui(|ui| {
        ui.horizontal(|ui| {
            let labels = fp_app::ui::widgets::MeterLabels {
                meter: "Level meter".to_owned(),
                max: "Maximum".to_owned(),
                max_tip: String::new(),
            };
            fp_app::ui::widgets::vu(
                ui,
                120.0,
                &MeterReading::default(),
                &MeterConfig::default(),
                &labels,
            );
            fp_app::ui::widgets::fader(ui, 120.0, 0.5, "Volume");
        });
    });
    h.run();
    let meter = h.get_by_label("Level meter").rect();
    let fader = h.get_by_label("Volume").rect();
    assert_eq!(meter.height(), 120.0);
    assert_eq!(meter.width(), meter_width(&MeterConfig::default()));
    assert_eq!(fader.height(), 120.0);
    assert!(fader.left() >= meter.right());
}

#[test]
fn a_meter_too_short_to_draw_lays_out_without_panicking() {
    for ballistics in ALL_METERS {
        for height in [0.0, 1.0, 5.0, 12.0, 24.0] {
            for loudness in [false, true] {
                let l = meter_layout(
                    column_for(&meter(ballistics), height),
                    &meter(ballistics),
                    loudness,
                );
                for m in &l.lines {
                    assert!(
                        m.y.is_finite() && m.label_centre().is_finite(),
                        "{ballistics:?} {height}"
                    );
                }
            }
        }
    }
}

/// Settings > Meters shows only what the chosen type uses (meters spec M3).
#[test]
fn each_meter_type_shows_only_its_settings() {
    let (mut h, _) = harness(state(1, 1));
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Meters").click();
    h.run_steps(2);
    let shown = |h: &egui_kittest::Harness<'_, fp_app::ui::app::AppUi>, label: &str| {
        h.query_all_by_label(label).next().is_some()
    };
    for label in [
        "Scale floor",
        "Peak hold",
        "Alignment level",
        "Warning from",
        "Danger from",
        "True peak",
    ] {
        assert!(shown(&h, label), "digital peak shows {label}");
    }
    assert!(!shown(&h, "Rise time"));
    h.get_by_role_and_label(Role::Button, "VU (IEC 60268-17)")
        .click();
    h.run_steps(3);
    assert!(shown(&h, "Alignment level"));
    for label in [
        "Scale floor",
        "Peak hold",
        "Warning from",
        "Danger from",
        "True peak",
    ] {
        assert!(!shown(&h, label), "VU hides {label}");
    }
    h.get_by_role_and_label(Role::Button, "K-20 (K-System)")
        .click();
    h.run_steps(3);
    assert!(shown(&h, "Peak hold") && shown(&h, "True peak"));
    assert!(!shown(&h, "Alignment level") && !shown(&h, "Warning from"));
    h.get_by_role_and_label(Role::Button, "Custom").click();
    h.run_steps(3);
    assert!(shown(&h, "Rise time") && shown(&h, "Scale floor"));
}

fn has(marks: &[f32], x: f32) -> bool {
    marks.iter().any(|m| near(*m, x))
}

fn minors_between(marks: &[f32], low: f32, high: f32) -> usize {
    marks
        .iter()
        .filter(|m| **m > low + 1e-3 && **m < high - 1e-3)
        .count()
}

#[test]
fn the_meter_has_a_ruler_on_each_side_of_the_bars() {
    for ballistics in ALL_METERS {
        let c = meter(ballistics);
        let rect = column_for(&c, 136.0);
        let l = meter_layout(rect, &c, false);
        let [left, right] = l.bars;
        let [lr, rr] = &l.rulers;
        assert_eq!((lr.side, rr.side), (Side::Left, Side::Right));
        // The tick strips are flush with the bars, outside them.
        assert!(near(lr.ticks.max, left.left()), "{ballistics:?}");
        assert!(near(rr.ticks.min, right.right()), "{ballistics:?}");
        // A clear gap between the ticks and the labels, so a label's minus
        // never reads as part of a tick.
        assert!(lr.labels_x + LABEL_TICK_GAP <= lr.ticks.min + 0.01);
        assert!(rr.labels_x >= rr.ticks.max + LABEL_TICK_GAP - 0.01);
        assert_eq!(lr.label_halign, egui::Align::Max);
        assert_eq!(rr.label_halign, egui::Align::Min);
        // The label columns fit in the meter's rect.
        assert!(near(lr.labels_x - label_column(&c), rect.left()));
        assert!(near(rr.labels_x + label_column(&c), rect.right()));
        // The gap between the bars is unchanged and empty.
        assert!(near(right.left() - left.right(), 2.0));
    }
    // Three-character labels on every scale but EBU, whose TEST is four.
    assert_eq!(meter_width(&meter(MeterBallistics::DigitalPeak)), 74.0);
    assert_eq!(meter_width(&meter(MeterBallistics::EbuPpm)), 84.0);
}

/// The clear gap (px) between a ruler's ticks and its labels: wide enough
/// that a 4 px tick and a label's minus never read as a double dash.
const LABEL_TICK_GAP: f32 = 3.0;

/// Every label of every scale fits its ruler's label column, measured in
/// the font it is drawn in (the column is sized from the widest label).
#[test]
fn every_label_fits_its_column() {
    use std::cell::RefCell;
    for c in ruled_scales() {
        let widths: RefCell<Vec<(String, f32)>> = RefCell::new(Vec::new());
        let mut h = egui_kittest::Harness::new_ui(|ui| {
            *widths.borrow_mut() = scale_marks(&c)
                .iter()
                .map(|m| {
                    let label = mark_label(*m, &c);
                    let w = ui
                        .painter()
                        .layout_no_wrap(
                            label.clone(),
                            egui::FontId::monospace(8.0),
                            egui::Color32::WHITE,
                        )
                        .size()
                        .x;
                    (label, w)
                })
                .collect();
        });
        h.run();
        for (label, w) in widths.borrow().iter() {
            assert!(
                *w <= label_column(&c) + 0.01,
                "{:?}: {label} is {w} px, the column {}",
                c.ballistics,
                label_column(&c)
            );
        }
    }
}

#[test]
fn every_labelled_mark_has_a_tick_on_both_rulers() {
    for ballistics in ALL_METERS {
        for height in [64.0, 136.0, 300.0] {
            let l = meter_layout(
                column_for(&meter(ballistics), height),
                &meter(ballistics),
                false,
            );
            for m in l.lines.iter().filter(|m| !m.label.is_empty()) {
                let tick = l
                    .ticks
                    .iter()
                    .find(|t| near(t.y, m.y) && t.kind != TickKind::Minor)
                    .unwrap_or_else(|| panic!("{ballistics:?} {height}: no tick for {}", m.label));
                let [lr, rr] = &l.rulers;
                let (a, b) = (lr.tick_rect(tick), rr.tick_rect(tick));
                assert!(near(a.center().y, tick.y) && near(b.center().y, tick.y));
                assert!(near(a.width(), b.width()), "the rulers mirror each other");
                assert!(near(a.right(), lr.ticks.max), "flush with the bar side");
                assert!(near(b.left(), rr.ticks.min), "flush with the bar side");
            }
        }
    }
}

#[test]
fn the_alignment_tick_is_thicker_and_unique() {
    for ballistics in ALL_METERS {
        let c = meter(ballistics);
        let l = meter_layout(column_for(&c, 136.0), &c, false);
        let alignment: Vec<&MeterTick> = l
            .ticks
            .iter()
            .filter(|t| t.kind == TickKind::Alignment)
            .collect();
        assert_eq!(alignment.len(), 1, "{ballistics:?}");
        let line = l.lines.iter().find(|m| m.alignment).unwrap();
        assert!(near(alignment[0].y, line.y), "{ballistics:?}");
        let [lr, rr] = &l.rulers;
        let a = lr.tick_rect(alignment[0]);
        let major = l.ticks.iter().find(|t| t.kind == TickKind::Major).unwrap();
        let m = lr.tick_rect(major);
        assert!(a.height() > m.height(), "{ballistics:?}: thicker");
        assert!(a.width() >= m.width(), "{ballistics:?}");
        assert!(rr.tick_rect(alignment[0]).height() > m.height());
        // Never inside a bar (flush with its edge at most).
        for bar in l.bars.map(|b| b.shrink(0.01)) {
            assert!(!a.intersects(bar) && !rr.tick_rect(alignment[0]).intersects(bar));
        }
    }
}

#[test]
fn minor_ticks_sit_between_the_marks_at_a_spacing_per_scale() {
    // Digital: 1 dB from -20 up, 5 dB below it (spec Q11.4).
    let c = meter(MeterBallistics::DigitalPeak);
    let m = minor_marks(&c);
    assert!(has(&m, -55.0) && has(&m, -45.0));
    assert!(has(&m, -19.0) && has(&m, -16.0) && has(&m, -1.0));
    assert!(!has(&m, -20.0) && !has(&m, -52.0), "never on a mark");
    assert_eq!(minors_between(&m, -60.0, -20.0), 2);
    assert_eq!(minors_between(&m, -20.0, 0.0), 16);
    // EBU: every 1 dB, three between each pair of marks, six gaps.
    let c = meter(MeterBallistics::EbuPpm);
    assert_eq!(minor_marks(&c).len(), 18);
    // DIN: 5 dB below -20 relative to its 0, 1 dB above.
    let c = meter(MeterBallistics::DinPpm);
    assert_eq!(minor_marks(&c).len(), 24);
    // VU: 5 dB below -10, 1 dB to -3, 0.5 dB above.
    let c = meter(MeterBallistics::Vu);
    let m = minor_marks(&c);
    assert_eq!(m.len(), 11);
    assert!(has(&m, c.reference_dbfs - 2.5) && has(&m, c.reference_dbfs - 15.0));
    // K-20 (0 at -20 dBFS): 1 dB between -24 and -20 relative to its 0.
    let c = meter(MeterBallistics::K20);
    let k = alignment_dbfs(&c);
    let m = minor_marks(&c);
    assert_eq!(minors_between(&m, k - 24.0, k - 20.0), 3);
    assert!(has(&m, k - 55.0));
}

#[test]
fn minor_marks_follow_a_custom_floor_and_reference() {
    let mut c = meter(MeterBallistics::DigitalPeak);
    c.floor_db = -57.0;
    let m = minor_marks(&c);
    assert!(has(&m, -55.0), "on the 5 dB grid, not 5 dB above the floor");
    assert!(!has(&m, -52.0), "never off the grid");
    assert!(!has(&m, -57.0), "the floor is a mark");
    let mut c = meter(MeterBallistics::EbuPpm);
    c.reference_dbfs = -20.0;
    let m = minor_marks(&c);
    assert!(has(&m, -20.0 + 1.0) && !has(&m, -20.0), "TEST is a mark");
    assert_eq!(m.len(), 18);
}

#[test]
fn ticks_stay_inside_the_rect_and_apart() {
    for ballistics in ALL_METERS {
        for height in [64.0, 136.0, 300.0] {
            for loudness in [false, true] {
                let c = meter(ballistics);
                let rect = column_for(&c, height);
                let l = meter_layout(rect, &c, loudness);
                let bars = l.bars[0];
                for (i, t) in l.ticks.iter().enumerate() {
                    assert!(
                        t.y >= bars.top() - 0.01 && t.y <= bars.bottom() + 0.01,
                        "{ballistics:?} {height} {loudness}: {t:?} outside the bars' height"
                    );
                    for r in &l.rulers {
                        assert!(rect.contains_rect(r.tick_rect(t)), "{t:?}");
                    }
                    if t.kind == TickKind::Minor {
                        for (j, o) in l.ticks.iter().enumerate() {
                            if i != j {
                                assert!(
                                    (o.y - t.y).abs() >= 3.0 - 0.01,
                                    "{ballistics:?} {height} {loudness}: minor {t:?} next to {o:?}"
                                );
                            }
                        }
                    }
                }
                let ys: Vec<f32> = l.ticks.iter().map(|t| t.y).collect();
                assert!(ys.windows(2).all(|p| p[0] <= p[1]), "sorted by y");
            }
        }
    }
}

#[test]
fn crowded_marks_keep_a_minor_tick() {
    // At 136 px some marks lose their label; every scale mark still has a
    // tick.
    for ballistics in [MeterBallistics::DigitalPeak, MeterBallistics::DinPpm] {
        let c = meter(ballistics);
        let l = meter_layout(column_for(&c, 136.0), &c, false);
        let bars = l.bars[0];
        for mark in scale_marks(&c) {
            let y = bars.bottom() - meter_position(mark, &c) * bars.height();
            assert!(
                l.ticks.iter().any(|t| (t.y - y).abs() < 1.0),
                "{ballistics:?}: no tick at {mark}"
            );
        }
    }
    let c = meter(MeterBallistics::DigitalPeak);
    let small = meter_layout(column_for(&c, 64.0), &c, false);
    let labelled = small.lines.iter().filter(|m| !m.label.is_empty()).count();
    let majors = small
        .ticks
        .iter()
        .filter(|t| t.kind == TickKind::Major)
        .count();
    assert!(majors <= labelled, "a major tick only for a label");
}

#[test]
fn a_taller_meter_has_more_minor_ticks() {
    let c = meter(MeterBallistics::DigitalPeak);
    let count = |h: f32| {
        meter_layout(column_for(&c, h), &c, false)
            .ticks
            .iter()
            .filter(|t| t.kind == TickKind::Minor)
            .count()
    };
    assert!(count(300.0) > count(64.0));
}

#[test]
fn tick_colours_follow_the_kind() {
    assert_eq!(tick_colour(TickKind::Major), fp_app::ui::theme::METER_TICK);
    assert_eq!(
        tick_colour(TickKind::Minor),
        fp_app::ui::theme::METER_TICK.gamma_multiply(fp_app::ui::theme::METER_TICK_MINOR_ALPHA)
    );
    assert_eq!(tick_colour(TickKind::Alignment), egui::Color32::WHITE);
}

#[test]
fn the_alignment_tick_is_white_on_both_rulers() {
    // The digital meter's alignment level (-18 dBFS) has no label: its tick
    // is still there, and white.
    let l = meter_layout(
        column_for(&meter(MeterBallistics::DigitalPeak), 136.0),
        &meter(MeterBallistics::DigitalPeak),
        false,
    );
    let a = l.lines.iter().find(|m| m.alignment).unwrap();
    assert!(a.label.is_empty());
    let tick = l
        .ticks
        .iter()
        .find(|t| t.kind == TickKind::Alignment)
        .unwrap();
    assert_eq!(tick_colour(tick.kind), egui::Color32::WHITE);
    for r in &l.rulers {
        assert!(near(r.tick_rect(tick).center().y, a.y));
    }
}

/// Whether `c` is `base` at some opacity (premultiplied, as
/// `gamma_multiply` gives it).
fn is_tint_of(c: egui::Color32, base: egui::Color32) -> bool {
    if base.a() == 0 {
        return false;
    }
    let tint = base.gamma_multiply(c.a() as f32 / base.a() as f32);
    c.to_array()
        .iter()
        .zip(tint.to_array())
        .all(|(a, b)| a.abs_diff(b) <= 2)
}

/// Whether `shape` is part of a bar's own content: a filled rect without a
/// stroke, in the bar background or a zone colour at any opacity (the
/// level, the K-System's dimmed peak, the hold).
fn is_bar_content(shape: &egui::Shape) -> bool {
    let egui::Shape::Rect(r) = shape else {
        return false;
    };
    let allowed = [
        fp_app::ui::theme::NEUTRAL_800,
        fp_app::ui::theme::METER_NORMAL,
        fp_app::ui::theme::METER_WARNING,
        fp_app::ui::theme::METER_DANGER,
    ];
    r.stroke.is_empty() && allowed.iter().any(|base| is_tint_of(r.fill, *base))
}

/// Spec Q11.1: of everything `vu` paints, only the bars' own content (the
/// background, the level, the hold and the zones) touches a bar, and nothing
/// touches the gap between them.
#[test]
fn nothing_is_drawn_over_the_bars_or_between_them() {
    use std::cell::RefCell;
    for ballistics in ALL_METERS {
        for loudness in [false, true] {
            let c = MeterConfig {
                loudness: if loudness {
                    LoudnessReadout::Momentary
                } else {
                    LoudnessReadout::Off
                },
                ..meter(ballistics)
            };
            let seen: RefCell<Vec<egui::Shape>> = RefCell::new(Vec::new());
            let bars: RefCell<[egui::Rect; 2]> = RefCell::new([egui::Rect::NOTHING; 2]);
            let mut h = egui_kittest::Harness::new_ui(|ui| {
                let labels = fp_app::ui::widgets::MeterLabels {
                    meter: "Level meter".to_owned(),
                    max: "Maximum".to_owned(),
                    max_tip: String::new(),
                };
                let mut r = reading(-9.0, -3.0);
                r.rms_db = [-14.0; 2];
                r.max_db = -3.0;
                r.momentary_lufs = Some(-23.0);
                let layer = ui.layer_id();
                let before = ui
                    .ctx()
                    .graphics_mut(|g| g.entry(layer).all_entries().len());
                fp_app::ui::widgets::vu(ui, 136.0, &r, &c, &labels);
                let rect = ui.min_rect();
                let l = meter_layout(
                    egui::Rect::from_min_size(rect.min, egui::vec2(meter_width(&c), 136.0)),
                    &c,
                    loudness,
                );
                *bars.borrow_mut() = l.bars;
                *seen.borrow_mut() = ui.ctx().graphics_mut(|g| {
                    g.entry(layer)
                        .all_entries()
                        .skip(before)
                        .map(|s| s.shape.clone())
                        .collect()
                });
            });
            h.run();
            let [left, right] = *bars.borrow();
            let gap = egui::Rect::from_x_y_ranges(left.right()..=right.left(), left.y_range());
            let shapes = seen.borrow();
            assert!(!shapes.is_empty(), "{ballistics:?}: nothing painted");
            for shape in shapes.iter() {
                let s = shape.visual_bounding_rect();
                assert!(
                    !(gap.width() > 0.0 && s.width() > 0.0 && s.intersects(gap.shrink(0.01))),
                    "{ballistics:?} {loudness}: {s:?} touches the gap {gap:?}"
                );
                for bar in [left, right] {
                    if s.intersects(bar.shrink(0.01)) {
                        assert!(
                            bar.expand(0.01).contains_rect(s),
                            "{ballistics:?} {loudness}: {s:?} crosses the edge of {bar:?}"
                        );
                        assert!(
                            is_bar_content(shape),
                            "{ballistics:?} {loudness}: {shape:?} is drawn over {bar:?}"
                        );
                    }
                }
            }
        }
    }
}

/// The steps a ruler segment's minor ticks may take, in the scale's units.
const LADDER: [f32; 6] = [0.5, 1.0, 2.0, 2.5, 5.0, 10.0];

/// The multiples of `step` (in the scale's units, counted from `zero`)
/// strictly between `lo` and `hi` dBFS.
fn multiples(lo: f32, hi: f32, step: f32, zero: f32) -> Vec<f32> {
    let mut out = Vec::new();
    let mut k = ((lo - zero) / step).floor() as i32;
    loop {
        let v = zero + k as f32 * step;
        if v >= hi - 0.01 {
            return out;
        }
        if v > lo + 0.01 {
            out.push(v);
        }
        k += 1;
    }
}

fn is_mark(c: &MeterConfig, db: f32) -> bool {
    (db - alignment_dbfs(c)).abs() < 0.01 || scale_marks(c).iter().any(|m| (m - db).abs() < 0.01)
}

/// The scales to rule: every meter, plus an odd digital floor and an EBU
/// alignment level off the whole dB.
fn ruled_scales() -> Vec<MeterConfig> {
    let mut configs: Vec<MeterConfig> = ALL_METERS.iter().map(|b| meter(*b)).collect();
    let mut odd = meter(MeterBallistics::DigitalPeak);
    odd.floor_db = -57.0;
    configs.push(odd);
    let mut ebu = meter(MeterBallistics::EbuPpm);
    ebu.reference_dbfs = -20.5;
    configs.push(ebu);
    configs
}

/// Spec Q11.4, as on a measuring ruler: between two ticks of the scale's
/// marks, the minor ticks are every multiple of one step, evenly spaced, or
/// none at all.
#[test]
fn minor_ticks_rule_each_segment_evenly() {
    for c in ruled_scales() {
        let zero = scale_zero(&c);
        let heights = (40..=400)
            .step_by(3)
            .map(|h| h as f32)
            .chain([120.0, 136.0]);
        for height in heights {
            for loudness in [false, true] {
                let l = meter_layout(column_for(&c, height), &c, loudness);
                let mut ticks = l.ticks.clone();
                ticks.sort_by(|a, b| a.db.total_cmp(&b.db));
                let bounds: Vec<f32> = ticks
                    .iter()
                    .filter(|t| t.kind != TickKind::Minor || is_mark(&c, t.db))
                    .map(|t| t.db)
                    .collect();
                for pair in bounds.windows(2) {
                    let (lo, hi) = (pair[0], pair[1]);
                    let inside: Vec<f32> = ticks
                        .iter()
                        .filter(|t| t.db > lo + 0.01 && t.db < hi - 0.01)
                        .map(|t| t.db)
                        .collect();
                    if inside.is_empty() {
                        continue;
                    }
                    let regular = LADDER.iter().any(|s| {
                        let m = multiples(lo, hi, *s, zero);
                        m.len() == inside.len()
                            && m.iter().zip(&inside).all(|(a, b)| (a - b).abs() < 0.01)
                    });
                    assert!(
                        regular,
                        "{:?} floor {} ref {} at {height} {loudness}: {inside:?} between {lo} and {hi}",
                        c.ballistics, c.floor_db, c.reference_dbfs
                    );
                }
            }
        }
    }
}

/// Where there is room, every step of the scale's own spacing is ruled.
#[test]
fn a_tall_meter_rules_every_step_of_its_scale() {
    for c in ruled_scales() {
        let l = meter_layout(column_for(&c, 1500.0), &c, false);
        // A step is ruled by a minor tick, or on the digital scale by a
        // major one where it is labelled.
        let mut minors: Vec<f32> = l
            .ticks
            .iter()
            .filter(|t| t.kind != TickKind::Alignment && !is_mark(&c, t.db))
            .map(|t| t.db)
            .collect();
        minors.sort_by(f32::total_cmp);
        let expected: Vec<f32> = minor_marks(&c)
            .into_iter()
            .filter(|m| (m - alignment_dbfs(&c)).abs() >= 0.01)
            .collect();
        assert_eq!(
            minors.len(),
            expected.len(),
            "{:?}: {minors:?}",
            c.ballistics
        );
        for (a, b) in minors.iter().zip(&expected) {
            assert!((a - b).abs() < 0.01, "{:?}: {minors:?}", c.ballistics);
        }
    }
}

/// The two cases a review found: the top of the digital scale at a bar
/// height of about 109 px kept −2 and −7 alone, and at 136 px dropped −1.
#[test]
fn the_top_of_the_digital_scale_is_ruled_evenly() {
    let c = meter(MeterBallistics::DigitalPeak);
    for height in [120.0, 136.0] {
        let l = meter_layout(column_for(&c, height), &c, false);
        for (lo, hi) in [(-10.0, -5.0), (-5.0, 0.0)] {
            let mut inside: Vec<f32> = l
                .ticks
                .iter()
                .filter(|t| t.db > lo + 0.01 && t.db < hi - 0.01)
                .map(|t| t.db)
                .collect();
            inside.sort_by(f32::total_cmp);
            let regular = LADDER.iter().any(|s| {
                let m = multiples(lo, hi, *s, 0.0);
                m.len() == inside.len() && m.iter().zip(&inside).all(|(a, b)| (a - b).abs() < 0.01)
            });
            assert!(regular, "{height}: {inside:?} between {lo} and {hi}");
        }
    }
}

/// Spec Q11.1 on the layout: no tick of any kind, on either ruler, and no
/// label column touches a bar or the gap between them.
#[test]
fn the_rulers_keep_clear_of_the_bars_and_the_gap() {
    for c in ruled_scales() {
        for height in [64.0, 136.0, 300.0] {
            for loudness in [false, true] {
                let l = meter_layout(column_for(&c, height), &c, loudness);
                let [left, right] = l.bars;
                let gap = egui::Rect::from_x_y_ranges(left.right()..=right.left(), left.y_range());
                let inner = [left, right, gap].map(|r| r.shrink(0.01));
                let [lr, rr] = &l.rulers;
                for ruler in [lr, rr] {
                    for t in &l.ticks {
                        let r = ruler.tick_rect(t);
                        assert!(
                            inner.iter().all(|b| !r.intersects(*b)),
                            "{:?} {height} {loudness}: {t:?} at {r:?} on the {:?} ruler",
                            c.ballistics,
                            ruler.side
                        );
                    }
                }
                // The labels hang away from the bars, from anchors outside
                // them.
                let label_column = label_column(&c);
                let labels = [
                    (lr.labels_x - label_column)..=lr.labels_x,
                    rr.labels_x..=(rr.labels_x + label_column),
                ];
                assert_eq!(lr.label_halign, egui::Align::Max);
                assert_eq!(rr.label_halign, egui::Align::Min);
                for x in labels {
                    let column = egui::Rect::from_x_y_ranges(x, left.y_range());
                    assert!(
                        inner.iter().all(|b| !column.intersects(*b)),
                        "{:?} {height} {loudness}: labels at {column:?}",
                        c.ballistics
                    );
                }
            }
        }
    }
}
