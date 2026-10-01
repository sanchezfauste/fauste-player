#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Settings > Remote (remote control spec §8).

mod support;

use std::sync::Arc;

use arc_swap::ArcSwap;
use egui::accesskit::Role;
use egui_kittest::kittest::Queryable;
use fp_model::Command;
use fp_remote::{RemoteStatus, ServerError, ServerStatus};
use support::{harness_from, state};

fn open(
    status: RemoteStatus,
) -> (
    egui_kittest::Harness<'static, fp_app::ui::app::AppUi>,
    Arc<support::Fake>,
    Arc<ArcSwap<RemoteStatus>>,
) {
    let cell = Arc::new(ArcSwap::from_pointee(status));
    let given = cell.clone();
    let (mut h, fake) = harness_from(state(1, 0), move |ui| ui.with_remote_status(given));
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Remote").click();
    h.run_steps(2);
    (h, fake, cell)
}

fn last_config(fake: &support::Fake) -> fp_model::Config {
    fake.take_sent()
        .into_iter()
        .rev()
        .find_map(|c| match c {
            Command::UpdateConfig(c) => Some(*c),
            _ => None,
        })
        .expect("no UpdateConfig sent")
}

#[test]
fn the_switches_turn_the_servers_on() {
    let (mut h, fake, _) = open(RemoteStatus::default());
    h.get_by_label("Allow remote control over HTTP").click();
    h.run_steps(2);
    assert!(last_config(&fake).remote.http.enabled);
    h.get_by_label("Allow OSC control").click();
    h.run_steps(2);
    assert!(last_config(&fake).remote.osc.enabled);
}

#[test]
fn the_status_of_each_server_is_shown() {
    let status = RemoteStatus {
        http: ServerStatus::Listening("127.0.0.1:7380".parse().unwrap()),
        osc: ServerStatus::Error(ServerError::TokenRequired),
    };
    let (h, _, _) = open(status);
    assert!(
        h.query_by_label_contains("Listening on 127.0.0.1:7380")
            .is_some()
    );
    assert!(h.query_by_label_contains("a token is required").is_some());
}

#[test]
fn generate_makes_a_long_random_token() {
    let (mut h, fake, _) = open(RemoteStatus::default());
    h.get_by_label("Generate").click();
    h.run_steps(2);
    let token = last_config(&fake).remote.http.token;
    assert_eq!(token.len(), 43);
    assert!(
        token
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    );
}

#[test]
fn the_token_is_masked_until_shown() {
    let mut s = state(1, 0);
    s.config.remote.http.token = "s3cret-token-value-0123".into();
    let cell = Arc::new(ArcSwap::from_pointee(RemoteStatus::default()));
    let (mut h, _) = harness_from(s, move |ui| ui.with_remote_status(cell));
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Remote").click();
    h.run_steps(2);
    assert!(h.query_by_label_contains("s3cret-token").is_none());
    assert_eq!(h.query_all_by_value("s3cret-token-value-0123").count(), 0);
    h.get_by_label("Show").click();
    h.run_steps(2);
    assert!(h.query_all_by_value("s3cret-token-value-0123").count() > 0);
}

#[test]
fn a_warning_appears_beyond_this_computer_without_a_token() {
    let mut s = state(1, 0);
    s.config.remote.http.bind = "0.0.0.0".into();
    let cell = Arc::new(ArcSwap::from_pointee(RemoteStatus::default()));
    let (mut h, _) = harness_from(s, move |ui| ui.with_remote_status(cell));
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Remote").click();
    h.run_steps(2);
    assert!(
        h.query_by_label_contains("A token is required to listen beyond this computer")
            .is_some()
    );
}

#[test]
fn new_tokens_differ() {
    let a = fp_app::remote::new_token().unwrap();
    let b = fp_app::remote::new_token().unwrap();
    assert_ne!(a, b);
    assert_eq!(a.len(), 43);
}

const VALID: &str = "0123456789abcdef0123456789abcdef0123456789a";

fn opened(
    s: fp_model::AppState,
) -> (
    egui_kittest::Harness<'static, fp_app::ui::app::AppUi>,
    Arc<support::Fake>,
) {
    let cell = Arc::new(ArcSwap::from_pointee(RemoteStatus::default()));
    let (mut h, fake) = harness_from(s, move |ui| ui.with_remote_status(cell));
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Remote").click();
    h.run_steps(2);
    (h, fake)
}

/// Replaces the text of the `nth` field labelled `label`, then presses `end`.
fn retype(
    h: &mut egui_kittest::Harness<'static, fp_app::ui::app::AppUi>,
    role: Role,
    label: &str,
    nth: usize,
    text: &str,
    end: egui::Key,
) {
    h.get_all_by_role_and_label(role, label)
        .nth(nth)
        .unwrap()
        .focus();
    h.run_steps(2);
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.run_steps(1);
    // One character per frame, as a keyboard sends them.
    for c in text.chars() {
        h.get_all_by_role_and_label(role, label)
            .nth(nth)
            .unwrap()
            .type_text(&c.to_string());
        h.run_steps(1);
    }
    h.key_press(end);
    h.run_steps(3);
}

fn sent_configs(fake: &support::Fake) -> Vec<fp_model::Config> {
    fake.take_sent()
        .into_iter()
        .filter_map(|c| match c {
            Command::UpdateConfig(c) => Some(*c),
            _ => None,
        })
        .collect()
}

#[test]
fn a_valid_address_applies_when_the_field_is_left() {
    let (mut h, fake) = opened(state(1, 0));
    retype(
        &mut h,
        Role::TextInput,
        "Address",
        0,
        "10.0.0.9",
        egui::Key::Tab,
    );
    assert_eq!(fake.state.load().config.remote.http.bind, "10.0.0.9");
}

#[test]
fn a_half_typed_address_keeps_the_previous_one() {
    let mut s = state(1, 0);
    s.config.remote.http.bind = "0.0.0.0".into();
    s.config.remote.http.token = VALID.into();
    let (mut h, fake) = opened(s);
    retype(
        &mut h,
        Role::TextInput,
        "Address",
        0,
        "192.168.1.",
        egui::Key::Tab,
    );
    assert_eq!(fake.state.load().config.remote.http.bind, "0.0.0.0");
}

#[test]
fn a_short_token_does_not_replace_the_token() {
    let mut s = state(1, 0);
    s.config.remote.http.token = VALID.into();
    let (mut h, fake) = opened(s);
    h.get_by_label("Show").click();
    h.run_steps(2);
    retype(&mut h, Role::TextInput, "Token", 0, "short", egui::Key::Tab);
    assert_eq!(fake.state.load().config.remote.http.token, VALID);
}

#[test]
fn escape_discards_the_typed_text() {
    let (mut h, fake) = opened(state(1, 0));
    retype(
        &mut h,
        Role::TextInput,
        "Address",
        0,
        "10.0.0.9",
        egui::Key::Escape,
    );
    assert_eq!(fake.state.load().config.remote.http.bind, "127.0.0.1");
    assert!(sent_configs(&fake).is_empty());
}

#[test]
fn typing_a_port_applies_only_the_final_value() {
    let (mut h, fake) = opened(state(1, 0));
    retype(
        &mut h,
        Role::SpinButton,
        "Port",
        0,
        "9000",
        egui::Key::Enter,
    );
    let ports: Vec<u16> = sent_configs(&fake)
        .iter()
        .map(|c| c.remote.http.port)
        .collect();
    assert_eq!(
        ports,
        vec![9000],
        "every intermediate value restarts the server"
    );
}

/// Types `text` into the `nth` field labelled `label`, leaving it focused.
fn type_into(
    h: &mut egui_kittest::Harness<'static, fp_app::ui::app::AppUi>,
    label: &str,
    nth: usize,
    text: &str,
) {
    h.get_all_by_role_and_label(Role::TextInput, label)
        .nth(nth)
        .unwrap()
        .focus();
    h.run_steps(2);
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.run_steps(1);
    for c in text.chars() {
        h.get_all_by_role_and_label(Role::TextInput, label)
            .nth(nth)
            .unwrap()
            .type_text(&c.to_string());
        h.run_steps(1);
    }
}

#[test]
fn a_draft_is_applied_when_another_section_is_opened() {
    let (mut h, fake) = opened(state(1, 0));
    type_into(&mut h, "Address", 0, "10.0.0.9");
    h.get_by_role_and_label(Role::Button, "MIDI").click();
    h.run_steps(3);
    assert_eq!(fake.state.load().config.remote.http.bind, "10.0.0.9");
}

#[test]
fn a_draft_is_applied_when_settings_closes() {
    let (mut h, fake) = opened(state(1, 0));
    type_into(&mut h, "Address", 0, "10.0.0.9");
    h.get_by_label("Close").click();
    h.run_steps(3);
    assert_eq!(fake.state.load().config.remote.http.bind, "10.0.0.9");
}

#[test]
fn an_invalid_draft_is_dropped_when_settings_closes() {
    let (mut h, fake) = opened(state(1, 0));
    type_into(&mut h, "Address", 0, "10.0.");
    h.get_by_label("Close").click();
    h.run_steps(3);
    assert!(
        sent_configs(&fake)
            .iter()
            .all(|c| c.remote.http.bind != "10.0.")
    );
    assert_eq!(fake.state.load().config.remote.http.bind, "127.0.0.1");
}

#[test]
fn escape_cancels_a_draft_whatever_else_closes() {
    let (mut h, fake) = opened(state(1, 0));
    type_into(&mut h, "Address", 0, "10.0.0.9");
    h.key_press(egui::Key::Escape);
    h.run_steps(3);
    // A second Escape closes Settings, if the first did not.
    h.key_press(egui::Key::Escape);
    h.run_steps(3);
    assert_eq!(fake.state.load().config.remote.http.bind, "127.0.0.1");
    assert!(sent_configs(&fake).is_empty());
}

/// The token buttons are the Settings buttons MIDI uses: 24 px tall and
/// as wide as their 12 px label plus 20 px.
#[test]
fn the_token_buttons_are_settings_buttons() {
    let (h, _) = opened(state(1, 0));
    for label in ["Show", "Copy", "Generate"] {
        let r = h.get_by_label(label).rect();
        let text = h.ctx.fonts_mut(|f| {
            f.layout_no_wrap(
                label.to_owned(),
                fp_app::ui::widgets::font(12.0),
                egui::Color32::WHITE,
            )
            .size()
            .x
        });
        assert!((r.height() - 24.0).abs() < 0.5, "{label}: {r:?}");
        assert!(
            (r.width() - (text + 20.0)).abs() < 0.5,
            "{label}: {r:?} text {text}"
        );
    }
}
