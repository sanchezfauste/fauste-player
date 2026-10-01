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
