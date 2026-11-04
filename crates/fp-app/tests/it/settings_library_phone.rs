#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Library spec LB46: a phone's Settings → Audio outputs shows the
//! library's pre-listen route, and an empty route reads as the default
//! output when the pre-listen falls back to it.

use crate::support;

use std::sync::Arc;

use egui::accesskit::Role;
use egui::vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_backends::{AudioBackend, OfflineBackend};
use support::{harness_sized, state};

const DEFAULT_OUTPUT: &str = "System default";
const NONE: &str = "None";

fn outputs_on_a_phone(default_output: bool) -> Harness<'static, AppUi> {
    let backend = OfflineBackend::new();
    backend.add_device("speakers", 2);
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let mut s = state(1, 1);
    s.config.outputs.backend = Some("offline".into());
    s.config.outputs.library.cue = None;
    s.config.outputs.library.default_output = default_output;
    let (mut h, _fake) = harness_sized(s, vec2(412.0, 892.0), |ui| {
        ui.with_touch().with_backends(backends)
    });
    h.run_steps(3);
    h.get_by_label("More").click();
    h.run_steps(2);
    h.get_by_label("Settings").click();
    h.run_steps(3);
    h.get_by_label("Audio outputs").click();
    for _ in 0..200 {
        h.run_steps(1);
        if h.query_all_by_label("Test Main").next().is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    h.run_steps(5);
    h
}

fn library_cue_label(h: &Harness<'static, AppUi>) -> Option<usize> {
    h.query_all_by_label(DEFAULT_OUTPUT).count().into()
}

#[test]
fn lb46_the_phone_shows_the_library_outputs_row() {
    let h = outputs_on_a_phone(false);
    assert_eq!(
        h.query_all_by_role_and_label(Role::Label, "Library")
            .count(),
        1
    );
}

#[test]
fn lb46_the_empty_library_route_reads_default_output_when_it_falls_back() {
    // Cartwall and the player's Main read "System default" either way; the
    // library's Cue adds one when it falls back, and loses a "None".
    let off = outputs_on_a_phone(false);
    let on = outputs_on_a_phone(true);
    assert_eq!(
        library_cue_label(&on).unwrap(),
        library_cue_label(&off).unwrap() + 1
    );
    assert_eq!(
        off.query_all_by_label(NONE).count(),
        on.query_all_by_label(NONE).count() + 1
    );
}
