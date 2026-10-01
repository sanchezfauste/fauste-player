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
use std::sync::Arc;

use fp_backends::{AudioBackend, OfflineBackend};
use fp_model::{Command, PlayerRoutes, Route};
use support::{harness, harness_with_backends, state};

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

#[test]
fn delete_backspace_and_escape_cannot_be_bound() {
    let (mut h, fake) = harness(state(1, 0));
    open_section(&mut h, "Keyboard shortcuts");
    for key in [Key::Delete, Key::Backspace] {
        h.get_by_role_and_label(Role::Button, "Stop P1").click();
        h.run_steps(2);
        h.key_press(key);
        h.run_steps(2);
    }
    assert!(
        !fake
            .take_sent()
            .iter()
            .any(|c| matches!(c, Command::SetShortcut { .. }))
    );
    assert!(
        h.query_all_by_label_contains("is reserved")
            .next()
            .is_some()
    );
}

#[test]
fn escape_while_waiting_for_a_key_cancels_the_capture_only() {
    let (mut h, fake) = harness(state(1, 0));
    open_section(&mut h, "Keyboard shortcuts");
    h.get_by_role_and_label(Role::Button, "Pause P1").click();
    h.run_steps(2);
    h.key_press(Key::Escape);
    h.run_steps(3);
    assert!(
        h.query_by_role_and_label(Role::Button, "Pause P1")
            .is_some(),
        "Settings stays open"
    );
    assert!(
        !fake
            .take_sent()
            .iter()
            .any(|c| matches!(c, Command::SetShortcut { .. }))
    );
}

#[test]
fn reserved_keys_in_the_config_are_ignored() {
    let mut s = state(1, 2);
    s.config.shortcuts.push(fp_model::Shortcut {
        action: fp_model::ShortcutAction::StopPlayer(1),
        chord: fp_model::KeyChord::key("Delete"),
    });
    let (mut h, fake) = harness(s);
    h.key_press(Key::Delete);
    h.run_steps(2);
    assert!(
        !fake
            .take_sent()
            .iter()
            .any(|c| matches!(c, Command::Stop(_)))
    );
}

#[test]
fn a_typed_cart_name_is_kept_when_another_cart_is_selected() {
    let (mut h, fake) = harness(with_cart());
    open_section(&mut h, "Cartwall");
    h.get_by_role_and_label(Role::Button, "Cart 1").click();
    h.run_steps(2);
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
        .type_text("Typed");
    h.run_steps(1);
    h.get_by_role_and_label(Role::Button, "Cart 2")
        .scroll_to_me();
    h.run_steps(3);
    h.get_by_role_and_label(Role::Button, "Cart 2").click();
    h.run_steps(3);
    assert_eq!(fake.state.load().cartwall.pages[0].carts[0].name, "Typed");
}

/// Player 1 plays on `dac` (exclusive-capable) and pre-listens on
/// `speakers` (shared); Settings is open on Audio outputs.
fn outputs() -> (
    egui_kittest::Harness<'static, fp_app::ui::app::AppUi>,
    Arc<support::Fake>,
) {
    outputs_with(Vec::new())
}

fn outputs_with(
    bit_perfect: Vec<fp_model::OutputDevice>,
) -> (
    egui_kittest::Harness<'static, fp_app::ui::app::AppUi>,
    Arc<support::Fake>,
) {
    let backend = OfflineBackend::new();
    backend.add_device("dac", 2).set_exclusive_capable(true);
    backend.add_device("speakers", 2);
    let mut s = state(1, 1);
    let route = |device: &str| Route {
        backend: "offline".into(),
        device: device.into(),
        first_channel: 0,
    };
    s.config.outputs.backend = Some("offline".into());
    s.config.outputs.bit_perfect = bit_perfect;
    s.config.outputs.routes = vec![PlayerRoutes {
        player: s.players[0].id,
        main: Some(route("dac")),
        cue: Some(route("speakers")),
    }];
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let (mut h, fake) = harness_with_backends(s, backends);
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Audio outputs")
        .click();
    // Devices are listed by a helper thread.
    for _ in 0..200 {
        h.run_steps(1);
        if h.query_by_role_and_label(Role::CheckBox, "Bit-perfect: dac")
            .is_some()
        {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    (h, fake)
}

fn bit_perfect_devices(fake: &support::Fake) -> Vec<String> {
    fake.state
        .load()
        .config
        .outputs
        .bit_perfect
        .iter()
        .map(|d| d.device.clone())
        .collect()
}

#[test]
fn a_device_can_be_marked_bit_perfect() {
    let (mut h, fake) = outputs();
    h.get_by_role_and_label(Role::CheckBox, "Bit-perfect: dac")
        .scroll_to_me();
    h.run_steps(5);
    h.get_by_role_and_label(Role::CheckBox, "Bit-perfect: dac")
        .click();
    h.run_steps(2);
    assert_eq!(bit_perfect_devices(&fake), vec!["dac".to_owned()]);
}

#[test]
fn a_shared_device_cannot_be_bit_perfect() {
    let (mut h, fake) = outputs();
    h.get_by_role_and_label(Role::CheckBox, "Bit-perfect: speakers")
        .scroll_to_me();
    h.run_steps(5);
    h.get_by_role_and_label(Role::CheckBox, "Bit-perfect: speakers")
        .click();
    h.run_steps(2);
    assert!(bit_perfect_devices(&fake).is_empty());
    // The same click on a capable device does take (the switch was reached).
    h.get_by_role_and_label(Role::CheckBox, "Bit-perfect: dac")
        .click();
    h.run_steps(2);
    assert_eq!(bit_perfect_devices(&fake), vec!["dac".to_owned()]);
}

#[test]
fn a_listed_device_can_always_be_turned_off() {
    // Listed earlier (or edited by hand) although it cannot be exclusive.
    let (mut h, fake) = outputs_with(vec![fp_model::OutputDevice {
        backend: "offline".into(),
        device: "speakers".into(),
    }]);
    h.get_by_role_and_label(Role::CheckBox, "Bit-perfect: speakers")
        .scroll_to_me();
    h.run_steps(5);
    h.get_by_role_and_label(Role::CheckBox, "Bit-perfect: speakers")
        .click();
    h.run_steps(2);
    assert!(bit_perfect_devices(&fake).is_empty());
}

#[test]
fn each_playlist_export_button_names_its_playlist() {
    let mut state = state(1, 1);
    fp_model::apply(
        &mut state,
        Command::CreatePlaylist {
            name: "Night".into(),
        },
    )
    .unwrap();
    let (mut h, _fake) = harness(state);
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Playlists").click();
    h.run_steps(2);
    assert!(
        h.query_by_role_and_label(Role::Button, "Export Main as M3U8")
            .is_some()
    );
    assert!(
        h.query_by_role_and_label(Role::Button, "Export Night as M3U8")
            .is_some()
    );
}

fn outputs_with_null(
    configured: Option<&str>,
) -> egui_kittest::Harness<'static, fp_app::ui::app::AppUi> {
    let mut s = state(1, 1);
    s.config.outputs.backend = configured.map(str::to_owned);
    let backends: Vec<Arc<dyn AudioBackend>> = vec![
        Arc::new(OfflineBackend::new()),
        Arc::new(fp_backends::NullBackend),
    ];
    let (mut h, _) = harness_with_backends(s, backends);
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Audio outputs")
        .click();
    for _ in 0..200 {
        h.run_steps(1);
        if h.query_all_by_label("Test Main").next().is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    h
}

#[test]
fn the_null_backend_is_not_offered() {
    let mut h = outputs_with_null(Some("offline"));
    h.get_by_value("Offline").click();
    h.run_steps(2);
    assert!(h.query_by_label("System default").is_some());
    assert!(h.query_by_label("Null").is_none());
    assert!(h.query_by_label("No output (silent)").is_none());
}

#[test]
fn a_configured_null_backend_shows_as_no_output() {
    let h = outputs_with_null(Some("null"));
    assert!(h.query_by_value("No output (silent)").is_some());
}
