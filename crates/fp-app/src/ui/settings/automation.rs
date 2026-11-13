//! Settings → Automation (automation spec §8.2, §11; rework AR20): the
//! templates and days, the rotations and their steps, then the music
//! fill's rules.
//! Every edit sends the whole rotation (`UpdateRotation`); the model checks
//! it and a refusal shows like any other.

use egui::{RichText, Ui, vec2};
use egui_phosphor::regular as icon;
use fp_model::rotation::{STEP_MAX_RANGE, STEP_MIN_RANGE};
use fp_model::{Command, Rotation, RotationId, RotationStep, StepSource, TrackKind};

use super::{SettingsState, caption, fitted_combo, note, row, slider, update};
use crate::ui::app::{Scene, ViewState};
use crate::ui::library::category_dialog::{KINDS, kind_key};
use crate::ui::theme;
use crate::ui::widgets::{self, TileStyle, font};

/// The rotation being edited and its name while it is typed.
#[derive(Default)]
pub(crate) struct AutomationState {
    selected: Option<RotationId>,
    name: Option<(RotationId, String)>,
    plans: super::plans::PlansState,
}

pub(super) fn section(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    st: &mut SettingsState,
) {
    rotations(ui, scene, &mut st.automation);
    ui.add_space(16.0);
    super::plans::section(ui, scene, view_state, &mut st.automation.plans);
    ui.add_space(16.0);
    rules(ui, scene);
}

/// AU27, AU31, AU32: the filler's look-ahead and kept rows, and the
/// separation rules.
fn rules(ui: &mut Ui, scene: &Scene<'_>) {
    let t = scene.i18n;
    let a = scene.state.config.automation.clone();
    caption(ui, &t.tr("settings-automation-rules"));
    ui.add_space(4.0);
    // Units are symbols, as the other sliders' " ms".
    let minutes = " min";
    let mut v = a.track_separation_mins;
    let label = t.tr("settings-track-separation-mins");
    row(
        ui,
        &label,
        Some(&t.tr("settings-hint-track-separation-mins")),
        |ui| {
            if slider(ui, &mut v, 0..=10_080, 15.0, minutes, &label) {
                update(scene, |c| c.automation.track_separation_mins = v);
            }
        },
    );
    let mut v = a.track_separation_percent;
    let label = t.tr("settings-track-separation-percent");
    row(
        ui,
        &label,
        Some(&t.tr("settings-hint-track-separation-percent")),
        |ui| {
            if slider(ui, &mut v, 0..=95, 1.0, " %", &label) {
                update(scene, |c| c.automation.track_separation_percent = v);
            }
        },
    );
    let mut v = a.artist_separation_mins;
    let label = t.tr("settings-artist-separation-mins");
    row(
        ui,
        &label,
        Some(&t.tr("settings-hint-artist-separation-mins")),
        |ui| {
            if slider(ui, &mut v, 0..=1_440, 5.0, minutes, &label) {
                update(scene, |c| c.automation.artist_separation_mins = v);
            }
        },
    );
    let mut v = a.filler_lookahead_mins;
    let label = t.tr("settings-filler-lookahead");
    row(
        ui,
        &label,
        Some(&t.tr("settings-hint-filler-lookahead")),
        |ui| {
            if slider(ui, &mut v, 5..=240, 5.0, minutes, &label) {
                update(scene, |c| c.automation.filler_lookahead_mins = v);
            }
        },
    );
    let mut v = a.filler_played_kept;
    let label = t.tr("settings-filler-kept");
    row(ui, &label, Some(&t.tr("settings-hint-filler-kept")), |ui| {
        if slider(ui, &mut v, 10..=1_000, 10.0, "", &label) {
            update(scene, |c| c.automation.filler_played_kept = v);
        }
    });
}

/// AU1–AU4: the rotations, the selected one's name and steps.
fn rotations(ui: &mut Ui, scene: &Scene<'_>, st: &mut AutomationState) {
    let t = scene.i18n;
    let rotations = &scene.state.rotations;
    caption(ui, &t.tr("settings-rotations"));
    ui.add_space(4.0);
    note(ui, &t.tr("settings-rotations-hint"), theme::NEUTRAL_400);
    ui.add_space(6.0);
    // The selection follows a rotation deleted meanwhile to the default.
    if st.selected.and_then(|id| rotations.get(id)).is_none() {
        st.selected = rotations.default_id();
    }
    let default = rotations.default_id();
    for r in rotations.iter() {
        let mut text = r.name.clone();
        if Some(r.id) == default {
            text = t.tr_args("settings-rotation-default", &[("name", text.into())]);
        }
        if ui
            .selectable_label(
                st.selected == Some(r.id),
                RichText::new(text).font(font(13.0)),
            )
            .clicked()
        {
            st.selected = Some(r.id);
            st.name = None;
        }
    }
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
        let new = t.tr("settings-rotation-new");
        if widgets::text_button(ui, Some(icon::PLUS), &new, true, TileStyle::plain()).clicked() {
            scene.ctl.send(Command::CreateRotation {
                name: t.tr("settings-rotation-new-name"),
                steps: vec![music_step()],
            });
        }
        let selected = st.selected;
        let make_default = t.tr("settings-rotation-make-default");
        let can = selected.is_some() && selected != default;
        if widgets::text_button(ui, Some(icon::STAR), &make_default, can, TileStyle::plain())
            .clicked()
            && let Some(id) = selected
        {
            scene.ctl.send(Command::SetDefaultRotation(id));
        }
        let delete = t.tr("settings-rotation-delete");
        if widgets::text_button(
            ui,
            Some(icon::TRASH),
            &delete,
            selected.is_some(),
            TileStyle::plain(),
        )
        .clicked()
            && let Some(id) = selected
        {
            scene.ctl.send(Command::DeleteRotation(id));
        }
    });
    let Some(rotation) = st.selected.and_then(|id| rotations.get(id)).cloned() else {
        return;
    };
    ui.add_space(10.0);
    name_row(ui, scene, st, &rotation);
    steps(ui, scene, &rotation);
}

fn music_step() -> RotationStep {
    RotationStep {
        source: StepSource::Kind(TrackKind::Music),
        min: 1,
        max: 1,
    }
}

/// The selected rotation's name, sent when the box is left or Enter
/// pressed.
fn name_row(ui: &mut Ui, scene: &Scene<'_>, st: &mut AutomationState, rotation: &Rotation) {
    let t = scene.i18n;
    let label = t.tr("settings-rotation-name");
    row(ui, &label, None, |ui| {
        let draft = match &mut st.name {
            Some((id, text)) if *id == rotation.id => text,
            other => {
                *other = Some((rotation.id, rotation.name.clone()));
                match other {
                    Some((_, text)) => text,
                    None => return,
                }
            }
        };
        let response = ui.add(egui::TextEdit::singleline(draft).desired_width(220.0));
        if response.lost_focus() && draft.trim() != rotation.name {
            scene.ctl.send(Command::UpdateRotation(Rotation {
                name: draft.clone(),
                ..rotation.clone()
            }));
        }
    });
}

/// One row per step: its source, its counts, and moving or removing it.
fn steps(ui: &mut Ui, scene: &Scene<'_>, rotation: &Rotation) {
    let t = scene.i18n;
    let send = |steps: Vec<RotationStep>| {
        scene.ctl.send(Command::UpdateRotation(Rotation {
            steps,
            ..rotation.clone()
        }));
    };
    let count = rotation.steps.len();
    for (i, step) in rotation.steps.iter().enumerate() {
        let n = i + 1;
        let title = t.tr_args("settings-rotation-step", &[("n", n.into())]);
        row(ui, &title, None, |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = vec2(6.0, 4.0);
                let shown = source_name(scene, &step.source);
                let mut chosen = None;
                fitted_combo(
                    ui,
                    ("rotation-step-source", rotation.id.0, i),
                    shown,
                    Some(260.0),
                    |ui| {
                        for (source, name) in sources(scene) {
                            if ui.selectable_label(source == step.source, name).clicked() {
                                chosen = Some(source);
                            }
                        }
                    },
                );
                if let Some(source) = chosen.filter(|s| *s != step.source) {
                    let mut steps = rotation.steps.clone();
                    if let Some(s) = steps.get_mut(i) {
                        s.source = source;
                    }
                    send(steps);
                }
                if let StepSource::Category(id) = step.source
                    && scene.state.categories.get(id).is_none()
                {
                    note(ui, &t.tr("rotation-step-missing"), theme::AMBER);
                }
                let (mut min, mut max) = (step.min, step.max);
                let at_least = t.tr_args("settings-rotation-step-min", &[("n", n.into())]);
                let at_most = t.tr_args("settings-rotation-step-max", &[("n", n.into())]);
                // Each count named on screen, the names in one column so the
                // two sliders line up.
                let shown_least = t.tr("settings-rotation-at-least");
                let shown_most = t.tr("settings-rotation-at-most");
                let name_width = [&shown_least, &shown_most]
                    .iter()
                    .map(|s| {
                        ui.painter()
                            .layout_no_wrap((*s).clone(), font(12.0), theme::NEUTRAL_400)
                            .size()
                            .x
                    })
                    .fold(0.0_f32, f32::max);
                // AU1: at least 0 (the step may sit out), at most 1 or more.
                let count_row = |ui: &mut Ui,
                                 name: &str,
                                 value: &mut u8,
                                 range: std::ops::RangeInclusive<u8>,
                                 label: &str| {
                    ui.horizontal(|ui| {
                        ui.add_sized(
                            vec2(name_width, 18.0),
                            egui::Label::new(
                                RichText::new(name)
                                    .font(font(12.0))
                                    .color(theme::NEUTRAL_400),
                            ),
                        );
                        slider(ui, value, range, 1.0, "", label)
                    })
                    .inner
                };
                let min_changed = count_row(ui, &shown_least, &mut min, STEP_MIN_RANGE, &at_least);
                let max_changed = count_row(ui, &shown_most, &mut max, STEP_MAX_RANGE, &at_most);
                if min_changed || max_changed {
                    // The counts stay in order: the one not moved follows.
                    if min_changed {
                        max = max.max(min);
                    } else {
                        min = min.min(max);
                    }
                    let mut steps = rotation.steps.clone();
                    if let Some(s) = steps.get_mut(i) {
                        s.min = min;
                        s.max = max;
                    }
                    send(steps);
                }
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = vec2(4.0, 0.0);
                    let up = t.tr("settings-rotation-step-up");
                    if widgets::text_button(
                        ui,
                        Some(icon::ARROW_UP),
                        &up,
                        i > 0,
                        TileStyle::plain(),
                    )
                    .clicked()
                    {
                        let mut steps = rotation.steps.clone();
                        steps.swap(i, i - 1);
                        send(steps);
                    }
                    let down = t.tr("settings-rotation-step-down");
                    if widgets::text_button(
                        ui,
                        Some(icon::ARROW_DOWN),
                        &down,
                        n < count,
                        TileStyle::plain(),
                    )
                    .clicked()
                    {
                        let mut steps = rotation.steps.clone();
                        steps.swap(i, i + 1);
                        send(steps);
                    }
                    let remove = t.tr("settings-rotation-step-remove");
                    if widgets::text_button(
                        ui,
                        Some(icon::X),
                        &remove,
                        count > 1,
                        TileStyle::plain(),
                    )
                    .clicked()
                    {
                        let mut steps = rotation.steps.clone();
                        steps.remove(i);
                        send(steps);
                    }
                });
            });
        });
    }
    let add = t.tr("settings-rotation-step-add");
    if widgets::text_button(ui, Some(icon::PLUS), &add, true, TileStyle::plain()).clicked() {
        let mut steps = rotation.steps.clone();
        steps.push(music_step());
        send(steps);
    }
}

/// Every source a step can take: the categories as indented paths, then
/// each kind.
fn sources(scene: &Scene<'_>) -> Vec<(StepSource, String)> {
    let categories = &scene.state.categories;
    let mut out: Vec<(StepSource, String)> = categories
        .iter()
        .map(|c| {
            let mut depth = 0;
            let mut parent = c.parent;
            while let Some(p) = parent.and_then(|p| categories.get(p)) {
                depth += 1;
                parent = p.parent;
                if depth > 64 {
                    break;
                }
            }
            (
                StepSource::Category(c.id),
                format!("{}{}", "   ".repeat(depth), c.name),
            )
        })
        .collect();
    out.extend(KINDS.iter().map(|k| {
        (
            StepSource::Kind(*k),
            source_name(scene, &StepSource::Kind(*k)),
        )
    }));
    out
}

fn source_name(scene: &Scene<'_>, source: &StepSource) -> String {
    let t = scene.i18n;
    match source {
        StepSource::Category(id) => scene
            .state
            .categories
            .get(*id)
            .map_or_else(|| t.tr("rotation-step-missing"), |c| c.name.clone()),
        StepSource::Kind(kind) => t.tr_args(
            "rotation-source-kind",
            &[("kind", t.tr(kind_key(*kind)).into())],
        ),
    }
}
