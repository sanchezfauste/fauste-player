//! Settings → Meters (meters spec M3): the meter type after a broadcast
//! standard, true peak, the scale and the loudness readout. Changes apply at
//! once.

use egui::{Color32, Ui, vec2};
use fp_model::{LoudnessReadout, MeterBallistics};

use super::{heading, row, slider, toggle, update};
use crate::ui::app::Scene;
use crate::ui::theme;
use crate::ui::widgets::{self, TileStyle, font};

const PRESETS: [(MeterBallistics, &str); 5] = [
    (MeterBallistics::DigitalPeak, "meter-digital-peak"),
    (MeterBallistics::EbuPpm, "meter-ebu-ppm"),
    (MeterBallistics::DinPpm, "meter-din-ppm"),
    (MeterBallistics::Vu, "meter-vu"),
    (MeterBallistics::Custom, "meter-custom"),
];

pub(super) fn section(ui: &mut Ui, scene: &Scene<'_>) {
    let t = scene.i18n;
    let m = scene.state.config.meter.clone();
    heading(ui, &t.tr("settings-tab-meters"));

    row(
        ui,
        &t.tr("settings-meter-type"),
        Some(&t.tr("settings-hint-meter-type")),
        |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
                for (ballistics, key) in PRESETS {
                    let on = m.ballistics == ballistics;
                    let text = t.tr(key);
                    let style = TileStyle {
                        fill: if on {
                            theme::NEUTRAL_700
                        } else {
                            Color32::TRANSPARENT
                        },
                        content: if on { theme::TEXT } else { theme::NEUTRAL_400 },
                        ..TileStyle::plain()
                    };
                    if widgets::tile(ui, vec2(280.0, 28.0), &text, true, style, |p, r, c| {
                        p.text(
                            r.left_center() + vec2(10.0, 0.0),
                            egui::Align2::LEFT_CENTER,
                            &text,
                            font(12.0),
                            c,
                        );
                    })
                    .clicked()
                    {
                        update(scene, |c| c.meter.ballistics = ballistics);
                    }
                }
            });
        },
    );
    if m.ballistics == MeterBallistics::Custom {
        let mut attack = m.attack_ms;
        let label = t.tr("settings-meter-attack");
        row(ui, &label, None, |ui| {
            if slider(ui, &mut attack, 0.0..=1000.0, 1.0, " ms", &label) {
                update(scene, |c| c.meter.attack_ms = attack);
            }
        });
        let mut release = m.release_db_per_sec;
        let label = t.tr("settings-meter-release");
        row(ui, &label, None, |ui| {
            if slider(ui, &mut release, 1.0..=100.0, 0.1, " dB/s", &label) {
                update(scene, |c| c.meter.release_db_per_sec = release);
            }
        });
    }
    let mut true_peak = m.true_peak;
    row(
        ui,
        &t.tr("settings-true-peak"),
        Some(&t.tr("settings-hint-true-peak")),
        |ui| {
            if toggle(ui, &mut true_peak, &t.tr("settings-true-peak")) {
                update(scene, |c| c.meter.true_peak = true_peak);
            }
        },
    );
    let db_slider = |ui: &mut Ui,
                     key: &str,
                     hint: Option<&str>,
                     value: f32,
                     range: std::ops::RangeInclusive<f32>,
                     suffix: &str,
                     set: fn(&mut fp_model::MeterConfig, f32)| {
        let mut v = value;
        let label = t.tr(key);
        let hint = hint.map(|h| t.tr(h));
        row(ui, &label, hint.as_deref(), |ui| {
            if slider(ui, &mut v, range, 0.5, suffix, &label) {
                update(scene, |c| set(&mut c.meter, v));
            }
        });
    };
    db_slider(
        ui,
        "settings-meter-floor",
        None,
        m.floor_db,
        -96.0..=-20.0,
        " dBFS",
        |c, v| {
            c.floor_db = v;
        },
    );
    db_slider(
        ui,
        "settings-meter-hold",
        Some("settings-hint-meter-hold"),
        m.peak_hold_secs,
        0.0..=10.0,
        " s",
        |c, v| c.peak_hold_secs = v,
    );
    db_slider(
        ui,
        "settings-meter-reference",
        Some("settings-hint-meter-reference"),
        m.reference_dbfs,
        -30.0..=0.0,
        " dBFS",
        |c, v| c.reference_dbfs = v,
    );
    db_slider(
        ui,
        "settings-meter-warning",
        Some("settings-hint-meter-warning"),
        m.warning_dbfs,
        -30.0..=0.0,
        " dBFS",
        |c, v| c.warning_dbfs = v,
    );
    db_slider(
        ui,
        "settings-meter-danger",
        Some("settings-hint-meter-danger"),
        m.danger_dbfs,
        -30.0..=0.0,
        " dBFS",
        |c, v| c.danger_dbfs = v,
    );
    row(
        ui,
        &t.tr("settings-loudness"),
        Some(&t.tr("settings-hint-loudness")),
        |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            for (readout, key) in [
                (LoudnessReadout::Off, "loudness-off"),
                (LoudnessReadout::Momentary, "loudness-momentary"),
                (LoudnessReadout::ShortTerm, "loudness-short-term"),
            ] {
                let on = m.loudness == readout;
                let text = t.tr(key);
                let style = TileStyle {
                    fill: if on {
                        theme::NEUTRAL_700
                    } else {
                        Color32::TRANSPARENT
                    },
                    content: if on { theme::TEXT } else { theme::NEUTRAL_400 },
                    ..TileStyle::plain()
                };
                if widgets::tile(ui, vec2(130.0, 30.0), &text, true, style, |p, r, c| {
                    p.text(
                        r.center(),
                        egui::Align2::CENTER_CENTER,
                        &text,
                        font(12.0),
                        c,
                    );
                })
                .clicked()
                {
                    update(scene, |c| c.meter.loudness = readout);
                }
            }
        },
    );
    db_slider(
        ui,
        "settings-loudness-target",
        Some("settings-hint-loudness-target"),
        m.loudness_target_lufs,
        -36.0..=-10.0,
        " LUFS",
        |c, v| c.loudness_target_lufs = v,
    );
}
