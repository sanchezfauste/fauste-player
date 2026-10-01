//! Settings > Remote (remote control spec §8): the HTTP and OSC switches,
//! addresses, token, allowed origins and senders, and each server's state.
//! Text fields apply when they lose focus or another section is opened;
//! the servers follow the saved configuration by themselves.

use std::collections::HashMap;

use egui::{RichText, Ui};
use fp_model::HttpRemoteConfig;
use fp_remote::{RemoteStatus, ServerError, ServerStatus};

use super::super::app::Scene;
use super::super::theme;
use super::super::widgets::font;
use super::{button, labelled_row, update};

#[derive(Default)]
pub(crate) struct RemoteState {
    /// Text being typed, per field, until it loses focus.
    drafts: HashMap<&'static str, String>,
    show_token: bool,
    /// Numbers being dragged or typed, per field, until released.
    numbers: HashMap<&'static str, u32>,
}

fn text(ui: &mut Ui, s: impl Into<String>, color: egui::Color32) -> egui::Response {
    ui.label(RichText::new(s.into()).font(font(13.0)).color(color))
}

/// A text field over a configuration value. Returns the new text when the
/// field loses focus with a change.
#[allow(clippy::too_many_arguments)]
fn field(
    ui: &mut Ui,
    st: &mut RemoteState,
    key: &'static str,
    label: egui::Id,
    current: &str,
    width: f32,
    multiline: bool,
    password: bool,
) -> Option<String> {
    let draft = st.drafts.entry(key).or_insert_with(|| current.to_owned());
    let edit = if multiline {
        egui::TextEdit::multiline(draft).desired_rows(3)
    } else {
        egui::TextEdit::singleline(draft).password(password)
    };
    let response = ui.add(edit.desired_width(width)).labelled_by(label);
    if response.has_focus() {
        return None;
    }
    let changed = (*draft != current).then(|| draft.clone());
    if !response.lost_focus() {
        // Not being edited: show what the configuration holds.
        *draft = current.to_owned();
        return None;
    }
    st.drafts.remove(key);
    // Escape cancels the edit; it does not apply it.
    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        return None;
    }
    changed
}

/// A number over a configuration value. Returns the new value once, when a
/// drag is released or typing ends, never the steps in between (each would
/// restart a server).
fn number(
    ui: &mut Ui,
    st: &mut RemoteState,
    key: &'static str,
    label: egui::Id,
    current: u32,
    range: std::ops::RangeInclusive<u32>,
    suffix: &str,
) -> Option<u32> {
    let value = st.numbers.entry(key).or_insert(current);
    let response = ui
        .add(
            egui::DragValue::new(value)
                .range(range)
                .suffix(suffix)
                .update_while_editing(false),
        )
        .labelled_by(label);
    let value = *value;
    if response.dragged() || response.has_focus() {
        return None;
    }
    st.numbers.remove(key);
    (value != current).then_some(value)
}

/// An IP address literal, as `bind` takes it.
fn address(text: &str) -> Option<String> {
    let text = text.trim();
    text.parse::<std::net::IpAddr>()
        .ok()
        .map(|_| text.to_owned())
}

fn lines(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Applies the text of field `key` to `config`; a value that is not
/// valid (a half-typed address, a token too short to use) keeps the one in
/// use.
fn commit(config: &mut fp_model::Config, key: &str, text: &str) {
    let remote = &mut config.remote;
    match key {
        "http.bind" => {
            if let Some(v) = address(text) {
                remote.http.bind = v;
            }
        }
        "osc.bind" => {
            if let Some(v) = address(text) {
                remote.osc.bind = v;
            }
        }
        "http.token" => {
            let v = text.trim();
            if HttpRemoteConfig::token_acceptable(v) {
                remote.http.token = v.to_owned();
            }
        }
        "http.origins" => remote.http.cors_origins = lines(text),
        "osc.sources" => remote.osc.allowed_sources = lines(text),
        _ => {}
    }
}

/// Applies the value of number `key` to `config`.
fn commit_number(config: &mut fp_model::Config, key: &str, value: u32) {
    let remote = &mut config.remote;
    match key {
        "http.port" => {
            if let Ok(port) = u16::try_from(value) {
                remote.http.port = port;
            }
        }
        "osc.port" => {
            if let Ok(port) = u16::try_from(value) {
                remote.osc.port = port;
            }
        }
        "events.position" => remote.events.position_interval_ms = value,
        _ => {}
    }
}

/// Applies what is still being edited when another section is opened, as
/// if its field had lost focus; an invalid draft is dropped.
pub(super) fn flush(scene: &Scene<'_>, st: &mut RemoteState) {
    let drafts = std::mem::take(&mut st.drafts);
    let numbers = std::mem::take(&mut st.numbers);
    update(scene, |c| {
        for (key, text) in &drafts {
            commit(c, key, text);
        }
        for (key, value) in &numbers {
            commit_number(c, key, *value);
        }
    });
}

fn status_line(scene: &Scene<'_>, status: &ServerStatus) -> (String, egui::Color32) {
    let t = scene.i18n;
    match status {
        ServerStatus::Off => (t.tr("remote-status-off"), theme::NEUTRAL_500),
        ServerStatus::Listening(addr) => (
            t.tr_args(
                "remote-status-listening",
                &[("addr", addr.to_string().into())],
            ),
            theme::NEUTRAL_300,
        ),
        ServerStatus::Error(ServerError::TokenRequired) => {
            (t.tr("remote-error-token"), theme::ON_AIR_TEXT)
        }
        ServerStatus::Error(ServerError::InvalidBind) => {
            (t.tr("remote-error-address"), theme::ON_AIR_TEXT)
        }
        ServerStatus::Error(ServerError::Bind(reason) | ServerError::Runtime(reason)) => (
            t.tr_args("remote-error-other", &[("reason", reason.clone().into())]),
            theme::ON_AIR_TEXT,
        ),
    }
}

pub(super) fn section(
    ui: &mut Ui,
    scene: &Scene<'_>,
    st: &mut super::SettingsState,
    status: Option<&RemoteStatus>,
) {
    let t = scene.i18n;
    let st = &mut st.remote;
    let Some(status) = status else {
        let _ = text(ui, t.tr("remote-unavailable"), theme::NEUTRAL_400);
        return;
    };
    let config = scene.state.config.remote.clone();

    // HTTP
    let _ = text(ui, t.tr("remote-http"), theme::TEXT);
    let mut on = config.http.enabled;
    if ui.checkbox(&mut on, t.tr("remote-http-enabled")).changed() {
        update(scene, |c| c.remote.http.enabled = on);
    }
    let (line, color) = status_line(scene, &status.http);
    let _ = text(ui, line, color);
    labelled_row(ui, &t.tr("remote-bind"), None, |ui, label| {
        let w = ui.available_width();
        if let Some(v) = field(
            ui,
            st,
            "http.bind",
            label,
            &config.http.bind,
            w,
            false,
            false,
        ) {
            update(scene, |c| commit(c, "http.bind", &v));
        }
    });
    labelled_row(ui, &t.tr("remote-port"), None, |ui, label| {
        let port = number(
            ui,
            st,
            "http.port",
            label,
            config.http.port.into(),
            1024..=65535,
            "",
        );
        if let Some(port) = port {
            update(scene, |c| commit_number(c, "http.port", port));
        }
    });
    labelled_row(ui, &t.tr("remote-token"), None, |ui, label| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Right to left: Generate, Copy, Show, then the field fills.
            if button(ui, &t.tr("remote-token-generate"))
                && let Some(token) = crate::remote::new_token()
            {
                st.drafts.remove("http.token");
                update(scene, |c| c.remote.http.token = token);
            }
            if button(ui, &t.tr("remote-token-copy")) {
                ui.ctx().copy_text(config.http.token.clone());
            }
            let shown = if st.show_token {
                t.tr("remote-token-hide")
            } else {
                t.tr("remote-token-show")
            };
            if button(ui, &shown) {
                st.show_token = !st.show_token;
            }
            let masked = !st.show_token;
            let w = ui.available_width();
            if let Some(v) = field(
                ui,
                st,
                "http.token",
                label,
                &config.http.token,
                w,
                false,
                masked,
            ) {
                update(scene, |c| commit(c, "http.token", &v));
            }
        });
    });
    let local = config.http.bind_addr().is_some_and(|ip| ip.is_loopback());
    if !local && config.http.token.is_empty() {
        let _ = text(ui, t.tr("remote-token-needed"), theme::AMBER);
    }
    labelled_row(ui, &t.tr("remote-origins"), None, |ui, label| {
        let w = ui.available_width();
        let current = config.http.cors_origins.join("\n");
        if let Some(v) = field(ui, st, "http.origins", label, &current, w, true, false) {
            update(scene, |c| commit(c, "http.origins", &v));
        }
    });

    ui.add_space(16.0);
    // OSC
    let _ = text(ui, t.tr("remote-osc"), theme::TEXT);
    let mut on = config.osc.enabled;
    if ui.checkbox(&mut on, t.tr("remote-osc-enabled")).changed() {
        update(scene, |c| c.remote.osc.enabled = on);
    }
    let (line, color) = status_line(scene, &status.osc);
    let _ = text(ui, line, color);
    labelled_row(ui, &t.tr("remote-bind"), None, |ui, label| {
        let w = ui.available_width();
        if let Some(v) = field(ui, st, "osc.bind", label, &config.osc.bind, w, false, false) {
            update(scene, |c| commit(c, "osc.bind", &v));
        }
    });
    labelled_row(ui, &t.tr("remote-port"), None, |ui, label| {
        let port = number(
            ui,
            st,
            "osc.port",
            label,
            config.osc.port.into(),
            1024..=65535,
            "",
        );
        if let Some(port) = port {
            update(scene, |c| commit_number(c, "osc.port", port));
        }
    });
    labelled_row(ui, &t.tr("remote-sources"), None, |ui, label| {
        let w = ui.available_width();
        let current = config.osc.allowed_sources.join("\n");
        if let Some(v) = field(ui, st, "osc.sources", label, &current, w, true, false) {
            update(scene, |c| commit(c, "osc.sources", &v));
        }
    });

    ui.add_space(16.0);
    labelled_row(ui, &t.tr("remote-position-interval"), None, |ui, label| {
        let every = config.events.position_interval_ms;
        if let Some(ms) = number(ui, st, "events.position", label, every, 50..=5000, " ms") {
            update(scene, |c| commit_number(c, "events.position", ms));
        }
    });
    // The servers' state changes on their own thread.
    ui.ctx()
        .request_repaint_after(std::time::Duration::from_millis(500));
}
