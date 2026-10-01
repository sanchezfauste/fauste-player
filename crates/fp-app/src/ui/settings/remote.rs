//! Settings > Remote (remote control spec §8): the HTTP and OSC switches,
//! addresses, token, allowed origins and senders, and each server's state.
//! Text fields apply when they lose focus; the servers follow the saved
//! configuration by themselves.

use std::collections::HashMap;

use egui::{RichText, Ui, vec2};
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
}

fn text(ui: &mut Ui, s: impl Into<String>, color: egui::Color32) {
    ui.label(RichText::new(s.into()).font(font(13.0)).color(color));
}

/// A text field over a configuration value. Returns the new text when the
/// field loses focus with a change.
fn field(
    ui: &mut Ui,
    st: &mut RemoteState,
    key: &'static str,
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
    let response = ui.add(edit.desired_width(320.0));
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
    changed
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
        text(ui, t.tr("remote-unavailable"), theme::NEUTRAL_400);
        return;
    };
    let config = scene.state.config.remote.clone();

    // HTTP
    text(ui, t.tr("remote-http"), theme::TEXT);
    let mut on = config.http.enabled;
    if ui.checkbox(&mut on, t.tr("remote-http-enabled")).changed() {
        update(scene, |c| c.remote.http.enabled = on);
    }
    let (line, color) = status_line(scene, &status.http);
    text(ui, line, color);
    ui.horizontal(|ui| {
        text(ui, t.tr("remote-bind"), theme::NEUTRAL_300);
        if let Some(v) = field(ui, st, "http.bind", &config.http.bind, false, false) {
            update(scene, |c| c.remote.http.bind = v.trim().to_owned());
        }
        text(ui, t.tr("remote-port"), theme::NEUTRAL_300);
        let mut port = config.http.port;
        if ui
            .add(egui::DragValue::new(&mut port).range(1024..=65535))
            .changed()
        {
            update(scene, |c| c.remote.http.port = port);
        }
    });
    ui.horizontal(|ui| {
        text(ui, t.tr("remote-token"), theme::NEUTRAL_300);
        let masked = !st.show_token;
        if let Some(v) = field(ui, st, "http.token", &config.http.token, false, masked) {
            update(scene, |c| c.remote.http.token = v.trim().to_owned());
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
        text(ui, t.tr("remote-token-needed"), theme::AMBER);
    }
    text(ui, t.tr("remote-origins"), theme::NEUTRAL_300);
    if let Some(v) = field(
        ui,
        st,
        "http.origins",
        &config.http.cors_origins.join("\n"),
        true,
        false,
    ) {
        update(scene, |c| c.remote.http.cors_origins = lines(&v));
    }

    ui.add_space(16.0);
    // OSC
    text(ui, t.tr("remote-osc"), theme::TEXT);
    let mut on = config.osc.enabled;
    if ui.checkbox(&mut on, t.tr("remote-osc-enabled")).changed() {
        update(scene, |c| c.remote.osc.enabled = on);
    }
    let (line, color) = status_line(scene, &status.osc);
    text(ui, line, color);
    ui.horizontal(|ui| {
        text(ui, t.tr("remote-bind"), theme::NEUTRAL_300);
        if let Some(v) = field(ui, st, "osc.bind", &config.osc.bind, false, false) {
            update(scene, |c| c.remote.osc.bind = v.trim().to_owned());
        }
        text(ui, t.tr("remote-port"), theme::NEUTRAL_300);
        let mut port = config.osc.port;
        if ui
            .add(egui::DragValue::new(&mut port).range(1024..=65535))
            .changed()
        {
            update(scene, |c| c.remote.osc.port = port);
        }
    });
    text(ui, t.tr("remote-sources"), theme::NEUTRAL_300);
    if let Some(v) = field(
        ui,
        st,
        "osc.sources",
        &config.osc.allowed_sources.join("\n"),
        true,
        false,
    ) {
        update(scene, |c| c.remote.osc.allowed_sources = lines(&v));
    }

    ui.add_space(16.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(8.0, 0.0);
        text(ui, t.tr("remote-position-interval"), theme::NEUTRAL_300);
        let mut ms = config.events.position_interval_ms;
        if ui
            .add(egui::DragValue::new(&mut ms).range(50..=5000).suffix(" ms"))
            .changed()
        {
            update(scene, |c| c.remote.events.position_interval_ms = ms);
        }
    });
    // The servers' state changes on their own thread.
    ui.ctx()
        .request_repaint_after(std::time::Duration::from_millis(500));
}
