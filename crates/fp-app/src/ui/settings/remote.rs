//! Settings > Remote (remote control spec §8): the HTTP and OSC switches,
//! addresses, token, allowed origins and senders, and each server's state.
//! Text fields apply when they lose focus; the servers follow the saved
//! configuration by themselves.

use std::collections::HashMap;

use egui::{RichText, Ui, vec2};
use fp_model::HttpRemoteConfig;
use fp_remote::{RemoteStatus, ServerError, ServerStatus};

use super::super::app::Scene;
use super::super::theme;
use super::super::widgets::font;
use super::{heading, update};

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
fn field(
    ui: &mut Ui,
    st: &mut RemoteState,
    key: &'static str,
    label: egui::Id,
    current: &str,
    multiline: bool,
    password: bool,
) -> Option<String> {
    let draft = st.drafts.entry(key).or_insert_with(|| current.to_owned());
    let edit = if multiline {
        egui::TextEdit::multiline(draft).desired_rows(3)
    } else {
        egui::TextEdit::singleline(draft).password(password)
    };
    let response = ui.add(edit.desired_width(320.0)).labelled_by(label);
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
    heading(ui, &t.tr("settings-tab-remote"));
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
    ui.horizontal(|ui| {
        let label = text(ui, t.tr("remote-bind"), theme::NEUTRAL_300).id;
        // An unfinished address keeps the one in use.
        if let Some(v) = field(ui, st, "http.bind", label, &config.http.bind, false, false)
            .and_then(|v| address(&v))
        {
            update(scene, |c| c.remote.http.bind = v);
        }
        let label = text(ui, t.tr("remote-port"), theme::NEUTRAL_300).id;
        let port = number(
            ui,
            st,
            "http.port",
            label,
            config.http.port.into(),
            1024..=65535,
            "",
        );
        if let Some(port) = port.and_then(|p| u16::try_from(p).ok()) {
            update(scene, |c| c.remote.http.port = port);
        }
    });
    ui.horizontal(|ui| {
        let label = text(ui, t.tr("remote-token"), theme::NEUTRAL_300).id;
        let masked = !st.show_token;
        // A token too short to use keeps the one in use.
        if let Some(v) = field(
            ui,
            st,
            "http.token",
            label,
            &config.http.token,
            false,
            masked,
        )
        .map(|v| v.trim().to_owned())
        .filter(|v| HttpRemoteConfig::token_acceptable(v))
        {
            update(scene, |c| c.remote.http.token = v);
        }
        let label = if st.show_token {
            t.tr("remote-token-hide")
        } else {
            t.tr("remote-token-show")
        };
        if ui.button(label).clicked() {
            st.show_token = !st.show_token;
        }
        if ui.button(t.tr("remote-token-copy")).clicked() {
            ui.ctx().copy_text(config.http.token.clone());
        }
        if ui.button(t.tr("remote-token-generate")).clicked()
            && let Some(token) = crate::remote::new_token()
        {
            st.drafts.remove("http.token");
            update(scene, |c| c.remote.http.token = token);
        }
    });
    let local = config.http.bind_addr().is_some_and(|ip| ip.is_loopback());
    if !local && config.http.token.is_empty() {
        let _ = text(ui, t.tr("remote-token-needed"), theme::AMBER);
    }
    let label = text(ui, t.tr("remote-origins"), theme::NEUTRAL_300).id;
    if let Some(v) = field(
        ui,
        st,
        "http.origins",
        label,
        &config.http.cors_origins.join("\n"),
        true,
        false,
    ) {
        update(scene, |c| c.remote.http.cors_origins = lines(&v));
    }

    ui.add_space(16.0);
    // OSC
    let _ = text(ui, t.tr("remote-osc"), theme::TEXT);
    let mut on = config.osc.enabled;
    if ui.checkbox(&mut on, t.tr("remote-osc-enabled")).changed() {
        update(scene, |c| c.remote.osc.enabled = on);
    }
    let (line, color) = status_line(scene, &status.osc);
    let _ = text(ui, line, color);
    ui.horizontal(|ui| {
        let label = text(ui, t.tr("remote-bind"), theme::NEUTRAL_300).id;
        if let Some(v) = field(ui, st, "osc.bind", label, &config.osc.bind, false, false)
            .and_then(|v| address(&v))
        {
            update(scene, |c| c.remote.osc.bind = v);
        }
        let label = text(ui, t.tr("remote-port"), theme::NEUTRAL_300).id;
        let port = number(
            ui,
            st,
            "osc.port",
            label,
            config.osc.port.into(),
            1024..=65535,
            "",
        );
        if let Some(port) = port.and_then(|p| u16::try_from(p).ok()) {
            update(scene, |c| c.remote.osc.port = port);
        }
    });
    let label = text(ui, t.tr("remote-sources"), theme::NEUTRAL_300).id;
    if let Some(v) = field(
        ui,
        st,
        "osc.sources",
        label,
        &config.osc.allowed_sources.join("\n"),
        true,
        false,
    ) {
        update(scene, |c| c.remote.osc.allowed_sources = lines(&v));
    }

    ui.add_space(16.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(8.0, 0.0);
        let label = text(ui, t.tr("remote-position-interval"), theme::NEUTRAL_300).id;
        let every = config.events.position_interval_ms;
        if let Some(ms) = number(ui, st, "events.position", label, every, 50..=5000, " ms") {
            update(scene, |c| c.remote.events.position_interval_ms = ms);
        }
    });
    // The servers' state changes on their own thread.
    ui.ctx()
        .request_repaint_after(std::time::Duration::from_millis(500));
}
