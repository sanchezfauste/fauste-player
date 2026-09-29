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
use fp_app::ui::widgets::{Zone, loudness_line, max_readout, meter_position, scale_marks, zone_of};
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
    assert_eq!(zone_of(-16.0, &c), Zone::Danger, "above +4");
}

#[test]
fn the_maximum_reads_with_one_decimal() {
    assert_eq!(max_readout(-3.24), "-3.2");
    assert_eq!(max_readout(0.4), "+0.4");
    assert_eq!(max_readout(-0.04), "0.0");
    assert_eq!(max_readout(-120.0), "—", "nothing measured");
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
