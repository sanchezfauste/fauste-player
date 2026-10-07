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

fn dac() -> fp_model::OutputDevice {
    fp_model::OutputDevice {
        backend: "offline".into(),
        device: "dac".into(),
    }
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
    outputs_in(fp_model::OutputsView::Advanced, bit_perfect)
}

/// `outputs_with` in the given view.
fn outputs_in(
    view: fp_model::OutputsView,
    bit_perfect: Vec<fp_model::OutputDevice>,
) -> (
    egui_kittest::Harness<'static, fp_app::ui::app::AppUi>,
    Arc<support::Fake>,
) {
    outputs_view_routed(view, bit_perfect, "speakers", None)
}

/// The Audio outputs tab with player 1's Main on `dac` and its Cue on
/// `cue`, and the cartwall's `(main, cue)` devices when given.
fn outputs_routed(
    bit_perfect: Vec<fp_model::OutputDevice>,
    cue: &str,
    cartwall: Option<(&str, &str)>,
) -> (
    egui_kittest::Harness<'static, fp_app::ui::app::AppUi>,
    Arc<support::Fake>,
) {
    outputs_view_routed(fp_model::OutputsView::Advanced, bit_perfect, cue, cartwall)
}

fn outputs_view_routed(
    view: fp_model::OutputsView,
    bit_perfect: Vec<fp_model::OutputDevice>,
    cue: &str,
    cartwall: Option<(&str, &str)>,
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
    s.config.ui.outputs_view = view;
    s.config.outputs.backend = Some("offline".into());
    s.config.outputs.bit_perfect = bit_perfect;
    s.config.outputs.routes = vec![PlayerRoutes {
        player: s.players[0].id,
        main: Some(route("dac")),
        cue: Some(route(cue)),
    }];
    if let Some((main, cue)) = cartwall {
        s.config.outputs.cartwall = fp_model::CartwallRoutes {
            main: Some(route(main)),
            cue: Some(route(cue)),
        };
    }
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let (mut h, fake) = harness_with_backends(s, backends);
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Audio outputs")
        .click();
    // Devices are listed by a helper thread; the routes show once they are.
    for _ in 0..200 {
        h.run_steps(1);
        if h.query_all_by_label("Test Main").next().is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    h.run_steps(2);
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
        .scroll_to_me();
    h.run_steps(5);
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
fn the_null_backend_is_offered_as_no_output() {
    let mut h = outputs_with_null(Some("offline"));
    h.get_by_value("Offline").click();
    h.run_steps(2);
    assert!(h.query_by_label("No output (silent)").is_some());
    assert!(
        h.query_by_label("Null").is_none(),
        "never by its internal name"
    );
}

#[test]
fn a_configured_null_backend_shows_as_no_output() {
    let h = outputs_with_null(Some("null"));
    assert!(h.query_by_value("No output (silent)").is_some());
}

#[test]
fn the_cue_markers_toggle_updates_the_config() {
    let (mut h, fake) = harness(state(1, 1));
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Players").click();
    h.run_steps(2);
    assert!(fake.state.load().config.players.use_cue_markers);
    h.get_by_role_and_label(Role::CheckBox, "Use cue-in and cue-out")
        .click();
    h.run_steps(2);
    let sent: Vec<bool> = fake
        .take_sent()
        .into_iter()
        .filter_map(|c| match c {
            Command::UpdateConfig(config) => Some(config.players.use_cue_markers),
            _ => None,
        })
        .collect();
    assert_eq!(sent.last(), Some(&false));
    assert!(!fake.state.load().config.players.use_cue_markers);
    assert!(
        h.query_by_role_and_label(Role::CheckBox, "Automatic mix at the MIX point")
            .is_some(),
        "next to the automatic mix switch"
    );
}

#[test]
fn a_bit_perfect_device_offers_its_dsd_modes_and_the_choice_updates_the_config() {
    let (mut h, fake) = outputs_with(vec![dac()]);
    h.get_by_role_and_label(Role::ComboBox, "DSD: dac")
        .scroll_to_me();
    h.run_steps(5);
    h.get_by_role_and_label(Role::ComboBox, "DSD: dac").click();
    h.run_steps(2);
    assert!(h.query_by_label("DoP").is_some());
    assert!(
        h.query_by_label("Native DSD").is_none(),
        "the device reports no DSD format"
    );
    h.get_by_label("DoP").click();
    h.run_steps(2);
    let c = fake.state.load().config.clone();
    assert_eq!(
        c.outputs.dsd_output_for("offline", "dac"),
        fp_model::DsdOutput::Dop
    );
}

fn rate_choices(fake: &support::Fake) -> Vec<Vec<fp_model::DeviceOverride>> {
    fake.take_sent()
        .into_iter()
        .filter_map(|c| match c {
            Command::UpdateConfig(config) => Some(config.outputs.device_overrides),
            _ => None,
        })
        .collect()
}

#[test]
fn a_device_can_be_given_its_own_rate_and_back_the_global_one() {
    let (mut h, fake) = outputs_with(Vec::new());
    h.get_by_role_and_label(Role::ComboBox, "Sample rate: dac")
        .scroll_to_me();
    h.run_steps(5);
    assert!(
        h.query_all_by_value("Global (48000 Hz)").count() >= 2,
        "dac and speakers both use the global rate"
    );
    h.get_by_role_and_label(Role::ComboBox, "Sample rate: dac")
        .click();
    h.run_steps(2);
    h.get_by_label("96000 Hz").click();
    h.run_steps(2);
    let c = fake.state.load().config.clone();
    assert_eq!(c.outputs.rate_for("offline", "dac"), 96_000);
    assert_eq!(c.outputs.rate_for("offline", "speakers"), 48_000);
    assert_eq!(
        c.outputs.sample_rate, 48_000,
        "the global rate is untouched"
    );
    assert_eq!(rate_choices(&fake).len(), 1, "one UpdateConfig");
    h.get_by_role_and_label(Role::ComboBox, "Sample rate: dac")
        .click();
    h.run_steps(2);
    h.get_by_label("Global (48000 Hz)").click();
    h.run_steps(2);
    assert!(fake.state.load().config.outputs.device_overrides.is_empty());
}

#[test]
fn a_device_can_be_given_its_own_buffer() {
    let (mut h, fake) = outputs_with(Vec::new());
    h.get_by_role_and_label(Role::ComboBox, "Buffer size: dac")
        .scroll_to_me();
    h.run_steps(5);
    h.get_by_role_and_label(Role::ComboBox, "Buffer size: dac")
        .click();
    h.run_steps(2);
    h.get_by_label("1024").click();
    h.run_steps(2);
    let c = fake.state.load().config.clone();
    assert_eq!(c.outputs.buffer_for("offline", "dac"), 1024);
    assert_eq!(c.outputs.buffer_for("offline", "speakers"), 512);
    assert!(
        h.query_by_label_contains("21.3 ms").is_some(),
        "1024 frames at 48 kHz"
    );
}

#[test]
fn the_dsd_silence_slider_updates_the_config() {
    let (mut h, fake) = outputs_with(Vec::new());
    h.get_by_role_and_label(Role::Slider, "DSD silence")
        .scroll_to_me();
    h.run_steps(5);
    h.get_by_role_and_label(Role::Slider, "DSD silence").focus();
    h.run_steps(1);
    h.key_press(Key::ArrowRight);
    h.run_steps(2);
    assert_eq!(fake.state.load().config.outputs.dsd_silence_ms, 210.0);
}

#[test]
fn the_dsd_mix_choice_updates_the_config() {
    let (mut h, fake) = outputs_with(vec![dac()]);
    h.get_by_value("Continue the DSD track as PCM")
        .scroll_to_me();
    h.run_steps(5);
    h.get_by_value("Continue the DSD track as PCM").click();
    h.run_steps(2);
    h.get_by_label("Keep DSD and mute the other sources")
        .click();
    h.run_steps(2);
    let c = fake.state.load().config.clone();
    assert_eq!(c.outputs.dsd_mix, fp_model::DsdMix::HoldOthers);
}

#[test]
fn the_basic_view_hides_the_device_rows() {
    let (h, _) = outputs_in(fp_model::OutputsView::Basic, vec![dac()]);
    assert!(
        h.query_by_role_and_label(Role::RadioButton, "Basic")
            .is_some()
    );
    assert!(
        h.query_by_value("48000 Hz").is_some(),
        "the global rate stays"
    );
    assert!(h.query_by_value("512").is_some(), "the global buffer stays");
    assert!(
        h.query_all_by_label("Test Main").next().is_some(),
        "the routes stay"
    );
    assert!(
        h.query_by_role_and_label(Role::CheckBox, "Bit-perfect: dac")
            .is_none()
    );
    assert!(h.query_by_value("Continue the DSD track as PCM").is_none());
}

#[test]
fn switching_views_changes_only_the_view() {
    let (mut h, fake) = outputs_with(vec![dac()]);
    let before = fake.state.load().config.clone();
    assert!(
        h.query_by_role_and_label(Role::CheckBox, "Bit-perfect: dac")
            .is_some()
    );
    h.get_by_role_and_label(Role::RadioButton, "Basic").click();
    h.run_steps(2);
    let basic = fake.state.load().config.clone();
    assert_eq!(basic.ui.outputs_view, fp_model::OutputsView::Basic);
    assert_eq!(basic.outputs, before.outputs, "no output setting changes");
    assert!(fp_model::restart_pending(&before, &basic).is_empty());
    assert!(
        h.query_by_role_and_label(Role::CheckBox, "Bit-perfect: dac")
            .is_none()
    );
    h.get_by_role_and_label(Role::RadioButton, "Advanced")
        .click();
    h.run_steps(2);
    assert_eq!(fake.state.load().config, before);
    assert!(
        h.query_by_role_and_label(Role::CheckBox, "Bit-perfect: dac")
            .is_some()
    );
}

#[test]
fn the_basic_view_says_when_advanced_settings_are_in_use() {
    let note = "Some devices have advanced settings";
    let (h, _) = outputs_in(fp_model::OutputsView::Basic, Vec::new());
    assert!(h.query_by_label_contains(note).is_none());
    let (h, _) = outputs_in(fp_model::OutputsView::Basic, vec![dac()]);
    assert!(h.query_by_label_contains(note).is_some());
}

const CUE_EQUALS_MAIN: &str = "The Cue output is the same as the Main output";

#[test]
fn a_cue_output_equal_to_main_is_warned_about() {
    let count = |cue, cartwall| {
        let (h, _fake) = outputs_routed(Vec::new(), cue, cartwall);
        h.query_all_by_label_contains(CUE_EQUALS_MAIN).count()
    };
    assert_eq!(count("speakers", None), 0, "separate outputs");
    assert_eq!(count("dac", None), 1, "a player's Cue on its Main");
    assert_eq!(count("speakers", Some(("dac", "dac"))), 1, "the cartwall's");
    assert_eq!(count("dac", Some(("dac", "dac"))), 2, "both");
    assert_eq!(count("speakers", Some(("dac", "speakers"))), 0);
}
