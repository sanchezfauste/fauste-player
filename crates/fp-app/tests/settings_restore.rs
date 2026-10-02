#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O2: "Restore defaults" in the Settings sections.

mod support;

use egui::Key;
use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_model::{Command, SettingsSection};
use support::{harness, state};

const QUESTION: &str = "Restore the default values of this section?";

fn open(
    section: &str,
    s: fp_model::AppState,
) -> (Harness<'static, AppUi>, std::sync::Arc<support::Fake>) {
    let (mut h, fake) = harness(s);
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, section).click();
    h.run_steps(2);
    fake.take_sent();
    (h, fake)
}

#[test]
fn players_restores_its_defaults_after_confirmation() {
    let mut s = state(1, 1);
    s.config.players.fade_ms = 3000;
    let (mut h, fake) = open("Players", s);
    h.get_by_label("Restore defaults").click();
    h.run_steps(2);
    assert!(h.query_by_label(QUESTION).is_some());
    assert!(fake.take_sent().is_empty());
    h.get_by_label("Restore").click();
    h.run_steps(2);
    assert_eq!(
        fake.take_sent(),
        vec![Command::RestoreDefaults(SettingsSection::Players)]
    );
    assert_eq!(fake.state.load().config.players.fade_ms, 1000);
    assert!(h.query_by_label(QUESTION).is_none());
}

#[test]
fn cancel_and_escape_change_nothing_and_keep_settings_open() {
    let mut s = state(1, 1);
    s.config.meter.floor_db = -40.0;
    let (mut h, fake) = open("Meters", s);
    h.get_by_label("Restore defaults").click();
    h.run_steps(2);
    h.get_by_label("Cancel").click();
    h.run_steps(2);
    assert!(h.query_by_label(QUESTION).is_none());
    h.get_by_label("Restore defaults").click();
    h.run_steps(2);
    h.key_press(Key::Escape);
    h.run_steps(3);
    assert!(h.query_by_label(QUESTION).is_none());
    // Settings is still open.
    assert!(h.query_by_role_and_label(Role::Button, "Meters").is_some());
    assert!(fake.take_sent().is_empty());
    assert_eq!(fake.state.load().config.meter.floor_db, -40.0);
}

#[test]
fn each_section_with_the_button_restores_itself() {
    for (section, expected) in [
        ("Meters", SettingsSection::Meters),
        ("Analysis", SettingsSection::Analysis),
        ("Keyboard shortcuts", SettingsSection::Shortcuts),
    ] {
        let (mut h, fake) = open(section, state(1, 1));
        h.get_by_label("Restore defaults").click();
        h.run_steps(2);
        h.get_by_label("Restore").click();
        h.run_steps(2);
        assert_eq!(
            fake.take_sent(),
            vec![Command::RestoreDefaults(expected)],
            "{section}"
        );
    }
}

#[test]
fn hardware_security_and_show_data_sections_have_no_button() {
    for section in ["Audio outputs", "Playlists", "Cartwall", "MIDI", "Remote"] {
        let (h, _) = open(section, state(1, 1));
        assert!(h.query_by_label("Restore defaults").is_none(), "{section}");
    }
}
