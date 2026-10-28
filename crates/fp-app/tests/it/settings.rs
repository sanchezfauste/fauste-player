#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The Settings modal (spec §8.4).

use crate::support;

use egui::Key;
use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT, Queryable};
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
    h.get_by_role_and_label(Role::ComboBox, "Language")
        .scroll_to_me();
    h.run_steps(5);
    h.get_by_role_and_label(Role::ComboBox, "Language").click();
    h.run_steps(2);
    // "System" first, then every registered language in its own name.
    let mut expected = vec!["System".to_owned()];
    expected.extend(fp_app::i18n::LOCALES.iter().map(|l| l.name.to_owned()));
    for name in &expected {
        assert!(h.query_by_label(name).is_some(), "{name} is offered");
    }
    let tops: Vec<f32> = expected
        .iter()
        .map(|n| h.get_by_label(n).rect().top())
        .collect();
    assert!(tops.is_sorted(), "listed in registry order: {tops:?}");
    h.get_by_label("Español").click();
    h.run_steps(4);
    assert_eq!(
        fake.state.load().config.ui.language.as_deref(),
        Some("es-ES")
    );
    assert!(h.query_all_by_label("Configuración").next().is_some());
    // Back to "System" (now in Spanish): the setting is cleared.
    h.get_by_role_and_label(Role::ComboBox, "Idioma").click();
    h.run_steps(2);
    h.get_by_label("Sistema").click();
    h.run_steps(4);
    assert_eq!(fake.state.load().config.ui.language, None);
}

/// A configured tag that no locale matches behaves as "System", and the
/// drop-down says so instead of showing the raw tag.
#[test]
fn an_unregistered_language_shows_as_system() {
    let mut s = state(1, 0);
    s.config.ui.language = Some("xx-YY".to_owned());
    let (mut h, _fake) = harness(s);
    open_section(&mut h, "Players");
    h.get_by_role_and_label(Role::ComboBox, "Language")
        .scroll_to_me();
    h.run_steps(5);
    assert!(h.query_by_value("System").is_some());
    assert!(h.query_by_value("xx-YY").is_none());
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

/// Types `text` into the number box of the grid slider labelled `label`,
/// one character per frame, then presses `end`.
fn type_grid_size(
    h: &mut egui_kittest::Harness<'static, fp_app::ui::app::AppUi>,
    label: &str,
    text: &str,
    end: Key,
) {
    h.get_by_role_and_label(Role::Slider, label).scroll_to_me();
    h.run_steps(3);
    h.get_by_role_and_label(Role::Slider, label).focus();
    h.run_steps(1);
    // Tab moves on to the slider's number box, which selects its text.
    h.key_press(Key::Tab);
    h.run_steps(2);
    for c in text.chars() {
        h.event(egui::Event::Text(c.to_string()));
        h.run_steps(1);
    }
    h.key_press(end);
    h.run_steps(3);
}

fn resizes(fake: &support::Fake) -> Vec<(u16, u16)> {
    fake.take_sent()
        .into_iter()
        .filter_map(|c| match c {
            Command::ResizeCartPage { rows, cols, .. } => Some((rows, cols)),
            _ => None,
        })
        .collect()
}

#[test]
fn a_typed_grid_size_resizes_the_page_once_it_is_complete() {
    let mut s = with_cart();
    let page = s.cartwall.pages[0].id;
    let (rows, cols) = (s.cartwall.pages[0].rows, s.cartwall.pages[0].cols);
    // A file in the last cart: a one-column page would lose it.
    let last = usize::from(rows) * usize::from(cols) - 1;
    fp_model::apply(
        &mut s,
        Command::AssignCartFile {
            page,
            index: last,
            path: std::path::PathBuf::from("/carts/last.wav"),
        },
    )
    .unwrap();
    let (mut h, fake) = harness(s);
    open_section(&mut h, "Cartwall");
    fake.take_sent();
    type_grid_size(&mut h, "Columns", "12", Key::Enter);
    assert_eq!(resizes(&fake), vec![(rows, 12)]);
    let p = &fake.state.load().cartwall.pages[0];
    assert_eq!((p.rows, p.cols), (rows, 12));
}

#[test]
fn leaving_the_number_box_applies_the_typed_grid_size() {
    let (mut h, fake) = harness(with_cart());
    open_section(&mut h, "Cartwall");
    let cols = fake.state.load().cartwall.pages[0].cols;
    fake.take_sent();
    type_grid_size(&mut h, "Rows", "6", Key::Tab);
    assert_eq!(resizes(&fake), vec![(6, cols)]);
}

#[test]
fn escape_keeps_the_grid_size_being_typed_from_applying() {
    let (mut h, fake) = harness(with_cart());
    open_section(&mut h, "Cartwall");
    let before = {
        let p = &fake.state.load().cartwall.pages[0];
        (p.rows, p.cols)
    };
    fake.take_sent();
    type_grid_size(&mut h, "Rows", "6", Key::Escape);
    assert_eq!(resizes(&fake), Vec::new());
    let p = &fake.state.load().cartwall.pages[0];
    assert_eq!((p.rows, p.cols), before);
}

#[test]
fn a_typed_player_count_applies_once_it_is_complete() {
    let (mut h, fake) = harness(state(4, 1));
    open_section(&mut h, "Players");
    let kept = fake.state.load().players[1].id;
    fake.take_sent();
    h.get_by_role_and_label(Role::Slider, "Number of players")
        .focus();
    h.run_steps(1);
    h.key_press(Key::Tab);
    h.run_steps(2);
    for c in "12".chars() {
        h.event(egui::Event::Text(c.to_string()));
        h.run_steps(1);
    }
    h.key_press(Key::Enter);
    h.run_steps(3);
    let counts: Vec<usize> = fake
        .take_sent()
        .into_iter()
        .filter_map(|c| match c {
            Command::SetPlayerCount(n) => Some(n),
            _ => None,
        })
        .collect();
    assert_eq!(counts, vec![12]);
    let state = fake.state.load();
    assert_eq!(state.players.len(), 12);
    assert_eq!(state.players[1].id, kept, "the second player is kept");
}

fn press(h: &egui_kittest::Harness<'static, fp_app::ui::app::AppUi>, pos: egui::Pos2, down: bool) {
    h.event(egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed: down,
        modifiers: egui::Modifiers::NONE,
    });
}

#[test]
fn the_player_count_applies_when_the_slider_is_released() {
    let (mut h, fake) = harness(state(4, 1));
    open_section(&mut h, "Players");
    let kept = fake.state.load().players[3].id;
    fake.take_sent();
    let rail = h
        .get_by_role_and_label(Role::Slider, "Number of players")
        .rect();
    // Down to one player and back up to the far end, in one drag.
    let y = rail.center().y;
    let start = egui::pos2(rail.left() + 2.0, y);
    h.hover_at(start);
    h.step();
    press(&h, start, true);
    h.step();
    let mut x = rail.left() + 2.0;
    while x < rail.right() + 200.0 {
        x += 20.0;
        h.event(egui::Event::PointerMoved(egui::pos2(x, y)));
        h.step();
    }
    let counts = |fake: &support::Fake| -> Vec<usize> {
        fake.take_sent()
            .into_iter()
            .filter_map(|c| match c {
                Command::SetPlayerCount(n) => Some(n),
                _ => None,
            })
            .collect()
    };
    assert_eq!(counts(&fake), Vec::<usize>::new(), "nothing while dragging");
    press(&h, egui::pos2(x, y), false);
    h.run_steps(3);
    let max = fake.state.load().config.limits.max_players;
    assert_eq!(counts(&fake), vec![max]);
    let state = fake.state.load();
    assert_eq!(state.players.len(), max);
    assert_eq!(state.players[3].id, kept, "the fourth player is kept");
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
    outputs_built(view, bit_perfect, cue, cartwall, |_, _| {})
}

/// `outputs_view_routed`, with `tweak` applied to the `dac` test handle and
/// the state before the app starts.
fn outputs_built(
    view: fp_model::OutputsView,
    bit_perfect: Vec<fp_model::OutputDevice>,
    cue: &str,
    cartwall: Option<(&str, &str)>,
    tweak: impl FnOnce(&fp_backends::OfflineDevice, &mut fp_model::AppState),
) -> (
    egui_kittest::Harness<'static, fp_app::ui::app::AppUi>,
    Arc<support::Fake>,
) {
    let backend = OfflineBackend::new();
    let dac_handle = backend.add_device("dac", 2);
    dac_handle.set_exclusive_capable(true);
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
    tweak(&dac_handle, &mut s);
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
    assert_eq!(rate_choices(&fake).len(), 1, "one UpdateConfig");
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
    for combo in ["Sample rate: dac", "Buffer size: dac", "DSD: dac"] {
        assert!(
            h.query_by_role_and_label(Role::ComboBox, combo).is_none(),
            "{combo}"
        );
    }
    assert!(
        h.query_by_role_and_label(Role::Slider, "DSD silence")
            .is_none()
    );
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

const CUE_ON_AIR: &str = "This Cue output shares an on-air output";

#[test]
fn a_cue_output_on_an_on_air_output_is_noted() {
    let count = |cue, cartwall| {
        let (h, _fake) = outputs_routed(Vec::new(), cue, cartwall);
        h.query_all_by_label_contains(CUE_ON_AIR).count()
    };
    assert_eq!(count("speakers", None), 0, "separate outputs");
    assert_eq!(count("dac", None), 1, "a player's Cue on its Main");
    assert_eq!(count("speakers", Some(("dac", "dac"))), 1, "the cartwall's");
    assert_eq!(count("dac", Some(("dac", "dac"))), 2, "both");
    assert_eq!(count("speakers", Some(("dac", "speakers"))), 0);
    assert_eq!(
        count("speakers", Some(("speakers", "dac"))),
        2,
        "each Cue on the other's Main"
    );
}

/// `dac` reports only 44.1 to 48 kHz and 64 to 1024 frames, and the
/// configuration still holds 192 kHz and 4096 frames for it.
fn a_device_that_narrowed() -> (
    egui_kittest::Harness<'static, fp_app::ui::app::AppUi>,
    Arc<support::Fake>,
) {
    outputs_built(
        fp_model::OutputsView::Advanced,
        Vec::new(),
        "speakers",
        None,
        |dac_handle, s| {
            dac_handle.set_reported_rates(vec![(44_100, 48_000)]);
            dac_handle.set_reported_buffers((64, 1024));
            s.config.outputs.set_device_rate(&dac(), Some(192_000));
            s.config.outputs.set_device_buffer(&dac(), Some(4096));
        },
    )
}

#[test]
fn a_saved_rate_the_device_no_longer_reports_is_kept_and_flagged() {
    let (mut h, _) = a_device_that_narrowed();
    h.get_by_role_and_label(Role::ComboBox, "Sample rate: dac")
        .scroll_to_me();
    h.run_steps(5);
    assert!(
        h.query_by_label("This device no longer reports 192000 Hz; it may not open.")
            .is_some()
    );
    assert!(
        h.query_by_label_contains("a buffer of 4096 frames")
            .is_some()
    );
    h.get_by_role_and_label(Role::ComboBox, "Sample rate: dac")
        .click();
    h.run_steps(2);
    assert!(h.query_by_label("96000 Hz").is_none(), "not reported");
    // A closed combo box now carries its selected text as its label too, so
    // look at the list items only.
    assert!(
        h.query_by_role_and_label(Role::Button, "48000 Hz")
            .is_some(),
        "reported"
    );
    assert!(
        h.query_by_label("192000 Hz").is_some(),
        "the saved one stays"
    );
}

#[test]
fn a_reported_rate_raises_no_note() {
    let (h, _) = outputs_with(Vec::new());
    assert!(h.query_by_label_contains("no longer reports").is_none());
}

#[test]
fn a_dsd_mode_the_device_cannot_take_shows_as_pcm() {
    let (mut h, fake) = outputs_built(
        fp_model::OutputsView::Advanced,
        vec![dac()],
        "speakers",
        None,
        |dac_handle, s| {
            dac_handle.set_exclusive_capable(false);
            s.config.outputs.dsd_output = vec![fp_model::DsdDevice {
                backend: "offline".into(),
                device: "dac".into(),
                mode: fp_model::DsdOutput::Dop,
            }];
        },
    );
    h.get_by_role_and_label(Role::ComboBox, "DSD: dac")
        .scroll_to_me();
    h.run_steps(5);
    assert!(h.query_by_value("DoP").is_none(), "the engine plays PCM");
    assert!(
        h.query_all_by_value("Convert to PCM").count() >= 2,
        "dac and speakers"
    );
    assert!(
        fake.take_sent().is_empty(),
        "showing it as PCM rewrites nothing"
    );
}

#[test]
fn an_unplugged_device_says_it_is_not_connected() {
    let (mut h, _) = outputs_routed(Vec::new(), "gone", None);
    h.get_by_role_and_label(Role::ComboBox, "DSD: gone")
        .scroll_to_me();
    h.run_steps(5);
    assert!(
        h.query_all_by_label_contains("need the device to be plugged in")
            .count()
            >= 1
    );
    assert!(h.query_by_label_contains("opened exclusively").is_none());
}

/// Audio outputs in a `width`-wide main window, with the DSD settings shown
/// (`dac` is bit-perfect).
fn narrow_outputs(width: f32) -> egui_kittest::Harness<'static, fp_app::ui::app::AppUi> {
    let backend = OfflineBackend::new();
    backend.add_device("dac", 2).set_exclusive_capable(true);
    backend.add_device("speakers", 2);
    let mut s = state(1, 1);
    let route = |device: &str| Route {
        backend: "offline".into(),
        device: device.into(),
        first_channel: 0,
    };
    s.config.ui.outputs_view = fp_model::OutputsView::Advanced;
    s.config.outputs.backend = Some("offline".into());
    s.config.outputs.bit_perfect = vec![dac()];
    s.config.outputs.dsd_mix = fp_model::DsdMix::HoldOthers;
    s.config.outputs.routes = vec![PlayerRoutes {
        player: s.players[0].id,
        main: Some(route("dac")),
        cue: Some(route("speakers")),
    }];
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let (mut h, _) =
        support::harness_sized(s, egui::vec2(width, 700.0), |ui| ui.with_backends(backends));
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
    h.run_steps(2);
    h
}

/// Every list on Audio outputs, the audio system first, stays inside a
/// narrow Settings window: none runs past the section's right margin.
#[test]
fn the_output_lists_stay_inside_a_narrow_settings_window() {
    let width = 700.0;
    let h = narrow_outputs(width);
    // The Settings window is centred, `SCREEN_MARGIN.x` (48) narrower than
    // the main window and at most `WINDOW_SIZE.x` (900) wide
    // (`src/ui/settings.rs`); the section body keeps a 24-point margin on
    // the right. Valid while the window is wider than the label column.
    let window = (width - 48.0).min(900.0);
    let content_right = (width + window) / 2.0 - 24.0;
    let backend = h.get_by_value("Offline").rect();
    assert!(
        backend.right() <= content_right + 0.5,
        "audio system {backend:?} past {content_right}"
    );
    let mix = h.get_by_value("Keep DSD and mute the other sources").rect();
    assert!(
        mix.right() <= content_right + 0.5,
        "DSD mix {mix:?} past {content_right}"
    );
    for combo in h.get_all_by_role(Role::ComboBox) {
        let rect = combo.rect();
        assert!(
            rect.right() <= content_right + 0.5,
            "{combo:?} {rect:?} past {content_right}"
        );
    }
}

/// The Settings window in a `width`-wide main window is centred,
/// `SCREEN_MARGIN.x` (48) narrower than the main window and at most
/// `WINDOW_SIZE.x` (900) wide (`src/ui/settings.rs`).
fn settings_window(width: f32) -> f32 {
    (width - 48.0).min(900.0)
}

/// The right edge of the section body: the body keeps a 24-point margin on
/// the right.
fn content_right(width: f32) -> f32 {
    (width + settings_window(width)) / 2.0 - 24.0
}

/// The left edge of the rows' control column: past the 200-point section
/// list and its 1-point line, the body's 24-point margin, the 180-point
/// label column and its 8 + 16 points of space.
fn control_left(width: f32) -> f32 {
    (width - settings_window(width)) / 2.0 + 201.0 + 24.0 + 180.0 + 24.0
}

/// A narrow main window whose Settings rows still leave the control column
/// some room (59 points beside the label column and the section list).
/// Narrower, down to the 420-point minimum, the label column itself no
/// longer fits and the body scrolls sideways.
const NARROW: f32 = 560.0;

/// Every list on Audio outputs (the audio system, the rate and buffer, the
/// device pickers and their channel pairs, the per-device rate, buffer and
/// DSD mode, the DSD mix) stays inside the control column: none runs past
/// the body's right margin, and none starts over the label column.
fn assert_lists_inside(width: f32) {
    let h = narrow_outputs(width);
    let (left, right) = (control_left(width), content_right(width));
    for label in ["Sample rate: dac", "Buffer size: dac", "DSD: dac"] {
        h.get_by_role_and_label(Role::ComboBox, label);
    }
    for combo in h.get_all_by_role(Role::ComboBox) {
        let rect = combo.rect();
        assert!(
            rect.left() >= left - 0.5 && rect.right() <= right + 0.5,
            "{width}: {combo:?} {rect:?} outside {left}..{right}"
        );
    }
}

#[test]
fn the_output_lists_stay_inside_the_column_of_a_narrow_window() {
    assert_lists_inside(700.0);
}

#[test]
fn the_output_lists_stay_inside_the_column_of_the_narrowest_window() {
    assert_lists_inside(NARROW);
}

/// The language list, set to a long name, stays inside the narrowest
/// window that still has a control column.
#[test]
fn the_language_list_stays_inside_a_narrow_settings_window() {
    let mut s = state(1, 0);
    s.config.ui.language = Some("nl-NL".to_owned());
    let (mut h, _) = support::harness_sized(s, egui::vec2(NARROW, 700.0), |ui| ui);
    open_section(&mut h, "Players");
    let right = content_right(NARROW);
    let rect = h.get_by_value("Nederlands").rect();
    assert!(
        rect.right() <= right + 0.5,
        "language {rect:?} past {right}"
    );
    hover_shows_whole(&mut h, None, "Nederlands");
}

/// Hovering the list `label` names (or, without one, the list showing
/// `value`) adds one more `value` on screen: the tooltip with the whole
/// choice.
fn hover_shows_whole(
    h: &mut egui_kittest::Harness<'static, fp_app::ui::app::AppUi>,
    label: Option<&str>,
    value: &str,
) {
    let list = || {
        let by = egui_kittest::kittest::by()
            .role(Role::ComboBox)
            .value(value);
        match label {
            Some(label) => by.label(label),
            None => by,
        }
    };
    let before = h.query_all_by_label(value).count();
    h.get(list()).scroll_to_me();
    h.run_steps(5);
    h.get(list()).hover();
    h.run_steps(40);
    let after = h.query_all_by_label(value).count();
    assert_eq!(after, before + 1, "hovering {value} shows it whole");
}

/// A device picker and a per-device list shortened in a narrow window
/// can still be read whole on hover.
#[test]
fn shortened_device_choices_show_whole_on_hover() {
    let mut h = narrow_outputs(NARROW);
    hover_shows_whole(&mut h, None, "speakers");
    let mut h = narrow_outputs(NARROW);
    hover_shows_whole(&mut h, Some("Sample rate: dac"), "Global (48000 Hz)");
}

/// A shortened choice can still be read whole: hovering the list shows it.
#[test]
fn a_shortened_choice_shows_whole_on_hover() {
    let mut h = narrow_outputs(700.0);
    let full = "Keep DSD and mute the other sources";
    // The closed combo box carries its selected text as its label too, so
    // count the nodes instead of looking for any.
    let before = h.query_all_by_label(full).count();
    // The list is below the fold: scroll to it first, as a user would.
    h.get_by_value(full).scroll_to_me();
    h.run_steps(5);
    h.get_by_value(full).hover();
    h.run_steps(40);
    assert_eq!(
        h.query_all_by_label(full).count(),
        before + 1,
        "the tooltip gives the whole choice"
    );
}

/// A colour chosen right after a rename, before the snapshot shows the new
/// name, keeps the new name.
#[test]
fn a_colour_chosen_right_after_a_rename_keeps_the_new_name() {
    let (mut h, fake) = harness(with_cart());
    open_section(&mut h, "Cartwall");
    h.get_by_role_and_label(Role::Button, "Cart 1").click();
    h.run_steps(2);
    h.get_all_by_role(Role::TextInput)
        .nth(1)
        .unwrap()
        .scroll_to_me();
    h.run_steps(3);
    fake.hold.store(true, std::sync::atomic::Ordering::SeqCst);
    h.get_all_by_role(Role::TextInput).nth(1).unwrap().focus();
    h.run_steps(1);
    h.get_all_by_role(Role::TextInput)
        .nth(1)
        .unwrap()
        .type_text("Station ID");
    h.run_steps(1);
    h.key_press(Key::Enter);
    h.run_steps(2);
    h.get_by_role_and_label(Role::RadioButton, "Pink")
        .scroll_to_me();
    h.run_steps(3);
    h.get_by_role_and_label(Role::RadioButton, "Teal").click();
    h.run_steps(3);
    let last = fake
        .take_sent()
        .into_iter()
        .filter_map(|c| match c {
            Command::SetCart { edit, .. } => Some(edit),
            _ => None,
        })
        .next_back()
        .unwrap();
    assert_eq!(last.color, Some(fp_model::CartColor::Teal));
    assert_eq!(last.name, "Station ID");
}

/// Cart colour spec CC5: nine swatches, the chosen one marked.
#[test]
fn cc5_the_colour_row_sets_and_clears_the_cart_colour() {
    use egui::accesskit::Toggled;
    use fp_model::CartColor;
    let (mut h, fake) = harness(with_cart());
    open_section(&mut h, "Cartwall");
    h.get_by_role_and_label(Role::Button, "Cart 1").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::RadioButton, "Pink")
        .scroll_to_me();
    h.run_steps(3);
    let toggled = |h: &egui_kittest::Harness<'static, fp_app::ui::app::AppUi>, name: &str| {
        h.get_by_role_and_label(Role::RadioButton, name)
            .accesskit_node()
            .toggled()
    };
    for name in [
        "No colour",
        "Red",
        "Orange",
        "Amber",
        "Green",
        "Teal",
        "Blue",
        "Violet",
        "Pink",
    ] {
        let expected = if name == "No colour" {
            Toggled::True
        } else {
            Toggled::False
        };
        assert_eq!(toggled(&h, name), Some(expected), "{name}");
    }
    h.get_by_role_and_label(Role::RadioButton, "Teal").click();
    h.run_steps(3);
    assert_eq!(
        fake.state.load().cartwall.pages[0].carts[0].color,
        Some(CartColor::Teal)
    );
    assert_eq!(toggled(&h, "Teal"), Some(Toggled::True));
    assert_eq!(toggled(&h, "No colour"), Some(Toggled::False));
    h.get_by_role_and_label(Role::RadioButton, "No colour")
        .click();
    h.run_steps(3);
    assert_eq!(fake.state.load().cartwall.pages[0].carts[0].color, None);
    // Each swatch says its name on hover too.
    h.get_by_role_and_label(Role::RadioButton, "Violet").hover();
    h.run_steps(40);
    assert!(
        h.query_all_by_label("Violet").count() >= 2,
        "the tooltip repeats the name"
    );
}

#[test]
fn a_shared_output_shows_the_least_buffer_it_asks_for() {
    // `dac` goes through a sound server; `speakers` is the hardware.
    let (mut h, _fake) = outputs_built(
        fp_model::OutputsView::Advanced,
        Vec::new(),
        "speakers",
        None,
        |dac, _| dac.set_shared(true),
    );
    let shared = "Latency: 10.7 ms (shared outputs: at least 50 ms)";
    h.get_by_role_and_label(Role::ComboBox, "Buffer size: dac")
        .scroll_to_me();
    h.run_steps(5);
    assert_eq!(
        h.query_all_by_label(shared).count(),
        2,
        "the global buffer and dac's, not speakers'"
    );
    assert!(h.query_all_by_label("Latency: 10.7 ms").next().is_some());
}

#[test]
fn a_shared_output_with_its_own_buffer_uses_it_as_it_is() {
    let (mut h, _fake) = outputs_built(
        fp_model::OutputsView::Advanced,
        Vec::new(),
        "speakers",
        None,
        |dac, s| {
            dac.set_shared(true);
            let dac = fp_model::OutputDevice {
                backend: "offline".into(),
                device: "dac".into(),
            };
            s.config.outputs.set_device_buffer(&dac, Some(1024));
        },
    );
    h.get_by_role_and_label(Role::ComboBox, "Buffer size: dac")
        .scroll_to_me();
    h.run_steps(5);
    assert!(h.query_all_by_label("Latency: 21.3 ms").next().is_some());
    assert_eq!(
        h.query_all_by_label("Latency: 10.7 ms (shared outputs: at least 50 ms)")
            .count(),
        1,
        "only the global buffer"
    );
}

#[test]
fn the_least_shared_buffer_shown_is_what_the_device_takes() {
    for (largest, shown) in [
        (
            2_048,
            Some("Latency: 10.7 ms (shared outputs: at least 43 ms)"),
        ),
        // A shared engine that only reports its period keeps the buffer.
        (480, None),
    ] {
        let (mut h, _fake) = outputs_built(
            fp_model::OutputsView::Advanced,
            Vec::new(),
            "speakers",
            None,
            |dac, _| {
                dac.set_shared(true);
                dac.set_reported_buffers((32, largest));
            },
        );
        h.get_by_role_and_label(Role::ComboBox, "Buffer size: dac")
            .scroll_to_me();
        h.run_steps(5);
        let floored = h.query_all_by_label_contains("shared outputs").count();
        match shown {
            Some(text) => assert_eq!(h.query_all_by_label(text).count(), 2, "{largest}"),
            None => assert_eq!(floored, 0, "{largest}"),
        }
    }
}
