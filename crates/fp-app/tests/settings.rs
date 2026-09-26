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

fn with_cart() -> fp_model::AppState {
    let mut s = state(1, 0);
    let page = s.cartwall.pages[0].id;
    fp_model::apply(
        &mut s,
        Command::AssignCartFile {
            page,
            index: 0,
            path: std::path::PathBuf::from("/carts/id.wav"),
        },
    )
    .unwrap();
    s
}

fn open_section(h: &mut egui_kittest::Harness<'static, fp_app::ui::app::AppUi>, name: &str) {
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, name).click();
    h.run_steps(2);
}

#[test]
fn a_cart_can_be_renamed_and_made_exclusive() {
    let (mut h, fake) = harness(with_cart());
    open_section(&mut h, "Cartwall");
    h.get_by_role_and_label(Role::Button, "Cart 1").click();
    h.run_steps(2);
    // The editor is below the grid: scroll to it first, as a user would.
    h.get_by_role_and_label(Role::CheckBox, "Stop other carts when fired")
        .scroll_to_me();
    h.run_steps(3);
    h.get_by_role_and_label(Role::CheckBox, "Stop other carts when fired")
        .click();
    h.run_steps(2);
    // Text fields in order: the page name, then the cart name.
    h.get_all_by_role(Role::TextInput)
        .nth(1)
        .unwrap()
        .scroll_to_me();
    h.run_steps(3);
    h.get_all_by_role(Role::TextInput).nth(1).unwrap().focus();
    h.run_steps(1);
    h.get_all_by_role(Role::TextInput)
        .nth(1)
        .unwrap()
        .type_text("Station ID");
    h.run_steps(1);
    h.key_press(Key::Enter);
    h.run_steps(3);
    let cart = &fake.state.load().cartwall.pages[0].carts[0];
    assert!(cart.exclusive);
    assert_eq!(cart.name, "Station ID");
}

#[test]
fn deleting_the_last_cart_page_shows_the_refusal() {
    let (mut h, _fake) = harness(with_cart());
    open_section(&mut h, "Cartwall");
    h.get_by_role_and_label(Role::Button, "Delete page").click();
    h.run_steps(3);
    assert!(
        h.query_all_by_label("The last cart page cannot be deleted.")
            .next()
            .is_some()
    );
}

#[test]
fn binding_a_shortcut_shows_and_resolves_the_conflict() {
    let (mut h, fake) = harness(state(1, 0));
    open_section(&mut h, "Keyboard shortcuts");
    h.get_by_role_and_label(Role::Button, "Pause P1").click();
    h.run_steps(2);
    h.key_press(Key::Num1);
    h.run_steps(2);
    assert!(
        h.query_all_by_label_contains("Play P1").next().is_some(),
        "the conflict is named"
    );
    h.get_by_role_and_label(Role::Button, "Assign").click();
    h.run_steps(2);
    assert!(fake.take_sent().iter().any(|c| matches!(
        c,
        Command::SetShortcut { action: fp_model::ShortcutAction::PausePlayer(1), chord: Some(k) } if k.key == "1"
    )));
}

#[test]
fn choosing_spanish_switches_the_interface() {
    let (mut h, fake) = harness(state(1, 0));
    open_section(&mut h, "Players");
    h.get_by_role_and_label(Role::Button, "Español").click();
    h.run_steps(4);
    assert_eq!(
        fake.state.load().config.ui.language.as_deref(),
        Some("es-ES")
    );
    assert!(h.query_all_by_label("Configuración").next().is_some());
}
