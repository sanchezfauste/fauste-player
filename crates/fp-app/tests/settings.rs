#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The Settings modal (spec §8.4).

mod support;

use egui::Key;
use egui::accesskit::Role;
use egui_kittest::kittest::Queryable;
use fp_model::Command;
use support::{harness, state};

#[test]
fn changing_fade_time_updates_the_config() {
    let (mut h, fake) = harness(state(1, 1));
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Players").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Slider, "Fade time").focus();
    h.run_steps(1);
    h.key_press(Key::ArrowRight);
    h.run_steps(2);
    let fades: Vec<u32> = fake
        .take_sent()
        .into_iter()
        .filter_map(|c| match c {
            Command::UpdateConfig(config) => Some(config.players.fade_ms),
            _ => None,
        })
        .collect();
    assert_eq!(fades.last(), Some(&1100));
    assert_eq!(fake.state.load().config.players.fade_ms, 1100);
}

#[test]
fn deleting_the_last_playlist_shows_the_refusal() {
    let (mut h, fake) = harness(state(1, 1));
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Playlists").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Delete").click();
    h.run_steps(3);
    assert!(
        fake.take_sent()
            .iter()
            .any(|c| matches!(c, Command::DeletePlaylist(_)))
    );
    assert!(
        h.query_all_by_label("The last playlist cannot be deleted.")
            .next()
            .is_some()
    );
    assert_eq!(fake.state.load().playlists.len(), 1);
}

#[test]
fn escape_closes_the_settings() {
    let (mut h, _fake) = harness(state(1, 1));
    h.get_by_label("Settings").click();
    h.run_steps(2);
    assert!(h.query_by_role_and_label(Role::Button, "Players").is_some());
    h.key_press(Key::Escape);
    h.run_steps(3);
    assert!(h.query_by_role_and_label(Role::Button, "Players").is_none());
}

#[test]
fn escape_cancels_a_playlist_rename() {
    let (mut h, fake) = harness(state(1, 1));
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Playlists").click();
    h.run_steps(2);
    // The second text field is the name of the first playlist.
    h.get_all_by_role(Role::TextInput).nth(1).unwrap().focus();
    h.run_steps(1);
    h.get_all_by_role(Role::TextInput)
        .nth(1)
        .unwrap()
        .type_text(" renamed");
    h.run_steps(1);
    h.key_press(Key::Escape);
    h.run_steps(3);
    assert!(
        !fake
            .take_sent()
            .iter()
            .any(|c| matches!(c, Command::RenamePlaylist { .. }))
    );
    assert_eq!(
        fake.state.load().playlists.iter().next().unwrap().name,
        "Main"
    );
}
