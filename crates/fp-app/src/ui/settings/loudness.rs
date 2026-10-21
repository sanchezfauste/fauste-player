//! Settings → Loudness (loudness normalization spec N22): normalization on
//! or off, the target, the true-peak ceiling and the maximum boost and
//! trim. Changes apply to tracks loaded afterwards (N3); a bit-perfect
//! output is never normalized (N7).

use egui::Ui;
use fp_model::Config;

use super::{note, row, slider, toggle, update};
use crate::ui::app::Scene;
use crate::ui::theme;

/// One slider row: its title, its hint, the value, its range (N1), the step, the
/// unit and where the value goes.
type Field = (
    &'static str,
    &'static str,
    f32,
    std::ops::RangeInclusive<f32>,
    f64,
    &'static str,
    fn(&mut Config, f32),
);

pub(super) fn section(ui: &mut Ui, scene: &Scene<'_>, unmeasured: usize) {
    let t = scene.i18n;
    let l = scene.state.config.loudness.clone();
    let mut on = l.normalize;
    let label = t.tr("settings-normalize");
    row(ui, &label, Some(&t.tr("settings-hint-normalize")), |ui| {
        if toggle(ui, &mut on, &label) {
            update(scene, |c| c.loudness.normalize = on);
        }
    });
    let fields: [Field; 4] = [
        (
            "settings-normalize-target",
            "settings-hint-normalize-target",
            l.target_lufs,
            -36.0..=-6.0,
            0.5,
            " LUFS",
            |c, v| c.loudness.target_lufs = v,
        ),
        (
            "settings-normalize-ceiling",
            "settings-hint-normalize-ceiling",
            l.true_peak_ceiling_dbtp,
            -12.0..=0.0,
            0.1,
            " dBTP",
            |c, v| c.loudness.true_peak_ceiling_dbtp = v,
        ),
        (
            "settings-normalize-max-boost",
            "settings-hint-normalize-max-boost",
            l.max_boost_db,
            0.0..=24.0,
            0.5,
            " dB",
            |c, v| c.loudness.max_boost_db = v,
        ),
        (
            "settings-normalize-max-trim",
            "settings-hint-normalize-max-trim",
            l.max_trim_db,
            0.0..=24.0,
            0.5,
            " dB",
            |c, v| c.loudness.max_trim_db = v,
        ),
    ];
    for (key, hint, value, range, step, suffix, set) in fields {
        let mut v = value;
        let label = t.tr(key);
        row(ui, &label, Some(&t.tr(hint)), |ui| {
            if slider(ui, &mut v, range, step, suffix, &label) {
                update(scene, |c| set(c, v));
            }
        });
    }
    ui.add_space(4.0);
    note(
        ui,
        &t.tr("settings-hint-normalize-apply"),
        theme::NEUTRAL_400,
    );
    note(ui, &t.tr("settings-hint-normalize-bp"), theme::NEUTRAL_400);
    if unmeasured > 0 {
        note(
            ui,
            &t.tr_args(
                "settings-normalize-unmeasured",
                &[("count", unmeasured.into())],
            ),
            theme::NEUTRAL_400,
        );
    }
}
