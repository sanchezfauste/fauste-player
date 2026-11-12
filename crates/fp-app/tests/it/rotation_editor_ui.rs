#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Automation spec §8.2 (rotation part): Settings → Automation.

use crate::support;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_model::{
    AppState, CategoryId, Command, RotationStep, SettingsSection, StepSource, TrackKind, apply,
};
use support::{Fake, harness, state};

fn with_rotation() -> AppState {
    let mut s = state(2, 1);
    fp_model::rotation::ensure_default(&mut s, "Music");
    s
}

fn open(s: AppState) -> (Harness<'static, AppUi>, std::sync::Arc<Fake>) {
    let (mut h, fake) = harness(s);
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Automation").click();
    h.run_steps(2);
    fake.take_sent();
    (h, fake)
}

#[test]
fn new_rotation_sends_create_with_one_music_step() {
    let (mut h, fake) = open(with_rotation());
    h.get_by_label("New rotation").click();
    h.run_steps(2);
    let sent = fake.take_sent();
    assert!(
        sent.iter().any(|c| matches!(
            c,
            Command::CreateRotation { steps, .. }
                if steps == &vec![RotationStep {
                    source: StepSource::Kind(TrackKind::Music),
                    min: 1,
                    max: 1,
                }]
        )),
        "{sent:?}"
    );
}

#[test]
fn editing_a_step_count_sends_update_rotation() {
    let (mut h, fake) = open(with_rotation());
    h.get_by_role_and_label(Role::Slider, "Step 1, at most")
        .focus();
    h.run_steps(1);
    h.key_press(egui::Key::ArrowRight);
    h.run_steps(2);
    let sent = fake.take_sent();
    assert!(
        sent.iter().any(|c| matches!(
            c,
            Command::UpdateRotation(r) if r.steps[0].max == 2 && r.steps[0].min == 1
        )),
        "{sent:?}"
    );
}

/// The two counts of a step are named on screen, not only for a screen
/// reader: two bare sliders do not say which is the least and which the most.
#[test]
fn a_step_names_its_two_counts() {
    let (h, _fake) = open(with_rotation());
    h.get_by_role_and_label(Role::Label, "At least");
    h.get_by_role_and_label(Role::Label, "At most");
}

#[test]
fn deleting_the_last_rotation_shows_the_error() {
    let (mut h, _fake) = open(with_rotation());
    h.get_by_label("Delete rotation").click();
    h.run_steps(3);
    assert!(
        h.query_all_by_label_contains("The last rotation cannot be deleted")
            .count()
            >= 1
    );
}

#[test]
fn a_step_on_a_deleted_category_is_flagged() {
    let mut s = with_rotation();
    let rotation = s.rotations.default_rotation().unwrap().clone();
    apply(
        &mut s,
        Command::UpdateRotation(fp_model::Rotation {
            steps: vec![RotationStep {
                source: StepSource::Category(CategoryId(987_654)),
                min: 1,
                max: 1,
            }],
            ..rotation
        }),
    )
    .unwrap();
    let (h, _fake) = open(s);
    // The step's source and the warning under it both say so.
    assert!(h.query_all_by_label_contains("Category deleted").count() >= 1);
}

#[test]
fn restore_defaults_resets_the_automation_rules() {
    let mut s = with_rotation();
    s.config.automation.artist_separation_mins = 5;
    let (mut h, fake) = open(s);
    h.get_by_label("Restore defaults").click();
    h.run_steps(2);
    h.get_by_label("Restore").click();
    h.run_steps(2);
    assert_eq!(
        fake.take_sent(),
        vec![Command::RestoreDefaults(SettingsSection::Automation)]
    );
    assert_eq!(
        fake.state.load().config.automation.artist_separation_mins,
        30
    );
}
