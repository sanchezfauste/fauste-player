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
use fp_app::ui::widgets::{Zone, loudness_line, meter_segments};
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

#[test]
fn the_meter_draws_the_telemetry_levels() {
    let c = MeterConfig::default(); // floor −60, warning −9, danger −3
    let segments = meter_segments(-30.0, -6.0, &c, 20);
    let lit: Vec<usize> = (0..20).filter(|i| segments[*i].lit).collect();
    // −30 dBFS is halfway up a −60 … 0 scale.
    assert_eq!(lit.len(), 10, "{lit:?}");
    assert!(segments.iter().any(|s| s.hold), "the hold is drawn");
    assert_eq!(segments[0].zone, Zone::Normal);
    assert_eq!(segments[17].zone, Zone::Warning, "−9 … −3 dBFS");
    assert_eq!(segments[19].zone, Zone::Danger);
    let silent = meter_segments(-120.0, -120.0, &c, 20);
    assert!(silent.iter().all(|s| !s.lit && !s.hold));
}

#[test]
fn the_reference_level_is_marked() {
    let c = MeterConfig::default();
    let segments = meter_segments(-120.0, -120.0, &c, 20);
    let marked: Vec<usize> = (0..20).filter(|i| segments[*i].reference).collect();
    assert_eq!(
        marked,
        vec![14],
        "−18 dBFS on a −60 … 0 scale of 20 (3 dB each)"
    );
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
