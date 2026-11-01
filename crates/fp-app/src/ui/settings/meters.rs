//! Settings → Meters (meters spec M3): the meter type after a broadcast
//! standard, true peak, the scale and the loudness readout. Changes apply at
//! once.

use egui::{Color32, Ui, pos2, vec2};
use fp_model::{LoudnessReadout, MeterBallistics};

use super::{column_width, row, slider, toggle, update};
use crate::ui::app::Scene;
use crate::ui::theme;
use crate::ui::widgets::{self, TileStyle, font};

/// The room between a loudness readout tile's name and its border.
const LOUDNESS_TILE_PAD: f32 = 6.0;

/// A loudness readout's name, centred and wrapped to `wrap` points.
fn name_job(text: &str, wrap: f32, colour: Color32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::simple(text.to_owned(), font(12.0), colour, wrap);
    job.halign = egui::Align::Center;
    job
}

const PRESETS: [(MeterBallistics, &str); 8] = [
    (MeterBallistics::DigitalPeak, "meter-digital-peak"),
    (MeterBallistics::EbuPpm, "meter-ebu-ppm"),
    (MeterBallistics::DinPpm, "meter-din-ppm"),
    (MeterBallistics::Vu, "meter-vu"),
    (MeterBallistics::K20, "meter-k20"),
    (MeterBallistics::K14, "meter-k14"),
    (MeterBallistics::K12, "meter-k12"),
    (MeterBallistics::Custom, "meter-custom"),
];

pub(super) fn section(ui: &mut Ui, scene: &Scene<'_>) {
    let t = scene.i18n;
    let m = scene.state.config.meter.clone();

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
    // Only what the chosen type's standard defines (meters spec M3).
    let uses = m.ballistics.settings();
    if uses.custom_ballistics {
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
    if uses.true_peak {
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
    }
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
    if uses.floor {
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
    }
    if uses.peak_hold {
        db_slider(
            ui,
            "settings-meter-hold",
            Some("settings-hint-meter-hold"),
            m.peak_hold_secs,
            0.0..=10.0,
            " s",
            |c, v| c.peak_hold_secs = v,
        );
    }
    if uses.alignment {
        db_slider(
            ui,
            "settings-meter-reference",
            Some("settings-hint-meter-reference"),
            m.reference_dbfs,
            -30.0..=0.0,
            " dBFS",
            |c, v| c.reference_dbfs = v,
        );
    }
    if uses.zones {
        db_slider(
            ui,
            "settings-meter-warning",
            Some("settings-hint-meter-warning"),
            m.warning_dbfs,
            -30.0..=0.0,
            " dBFS",
            |c, v| c.warning_dbfs = v,
        );
    }
    if uses.zones {
        db_slider(
            ui,
            "settings-meter-danger",
            Some("settings-hint-meter-danger"),
            m.danger_dbfs,
            -30.0..=0.0,
            " dBFS",
            |c, v| c.danger_dbfs = v,
        );
    }
    row(
        ui,
        &t.tr("settings-loudness"),
        Some(&t.tr("settings-hint-loudness")),
        |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            // Three to a line, narrower on a phone (PL33).
            let width = (column_width(ui) / 3.0).min(130.0);
            let options = [
                (LoudnessReadout::Off, "loudness-off"),
                (LoudnessReadout::Momentary, "loudness-momentary"),
                (LoudnessReadout::ShortTerm, "loudness-short-term"),
            ];
            // A name too long for its tile wraps, and every tile grows to
            // the tallest name, so no two names run into each other.
            let texts: Vec<String> = options.iter().map(|(_, key)| t.tr(key)).collect();
            let wrap = width - 2.0 * LOUDNESS_TILE_PAD;
            let height = texts
                .iter()
                .map(|text| {
                    ui.painter()
                        .layout_job(name_job(text, wrap, theme::TEXT))
                        .size()
                        .y
                        + 2.0 * LOUDNESS_TILE_PAD
                })
                .fold(30.0, f32::max);
            for ((readout, _), text) in options.into_iter().zip(texts) {
                let on = m.loudness == readout;
                let style = TileStyle {
                    fill: if on {
                        theme::NEUTRAL_700
                    } else {
                        Color32::TRANSPARENT
                    },
                    content: if on { theme::TEXT } else { theme::NEUTRAL_400 },
                    ..TileStyle::plain()
                };
                if widgets::tile(ui, vec2(width, height), &text, true, style, |p, r, c| {
                    let galley = p.layout_job(name_job(&text, wrap, c));
                    p.galley(
                        pos2(r.center().x, r.center().y - galley.size().y / 2.0),
                        galley,
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
