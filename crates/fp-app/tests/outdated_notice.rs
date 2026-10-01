#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! After an update, tracks an earlier version analysed are analysed again
//! only when the operator asks (plan 4, item 16).

mod support;

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable;
use fp_app::services::ServiceRequest;
use support::{harness_from, state};

/// `tracks` tracks, `outdated` of them analysed by an earlier version.
fn library(tracks: usize, outdated: usize) -> fp_model::AppState {
    let mut s = state(1, tracks);
    for (n, t) in s.library.iter_mut().enumerate() {
        t.analyzed = true;
        t.format = Some(fp_model::AudioFormat {
            sample_rate: 44_100,
            bits: Some(16),
            channels: 2,
        });
        t.analysis_version = if n < outdated {
            0
        } else {
            fp_analysis::cache::ANALYSIS_VERSION
        };
    }
    s
}

fn open(
    s: fp_model::AppState,
) -> (
    egui_kittest::Harness<'static, fp_app::ui::app::AppUi>,
    crossbeam_channel::Receiver<ServiceRequest>,
) {
    let (tx, rx) = crossbeam_channel::bounded(4);
    let (mut h, _) = harness_from(s, move |ui| ui.with_services(tx));
    h.run_steps(2);
    (h, rx)
}

#[test]
fn the_notice_counts_the_tracks_and_analyse_now_asks_for_them() {
    let (mut h, rx) = open(library(5, 3));
    assert!(
        h.query_by_label_contains("3 tracks were analysed by an earlier version")
            .is_some()
    );
    h.get_by_role_and_label(Role::Button, "Analyse now").click();
    h.run_steps(2);
    assert_eq!(rx.try_recv(), Ok(ServiceRequest::AnalyseOutdated));
    assert!(h.query_by_label("Analyse now").is_none(), "closed");
}

#[test]
fn later_closes_the_notice_without_asking() {
    let (mut h, rx) = open(library(5, 1));
    assert!(
        h.query_by_label_contains("One track was analysed by an earlier version")
            .is_some()
    );
    h.get_by_role_and_label(Role::Button, "Later").click();
    h.run_steps(2);
    assert!(rx.try_recv().is_err());
    assert!(h.query_by_label("Later").is_none(), "closed");
}

#[test]
fn an_up_to_date_library_shows_no_notice() {
    let (h, _) = open(library(5, 0));
    assert!(h.query_by_label("Analyse now").is_none());
}

#[test]
fn settings_offers_the_outdated_tracks_too() {
    let (tx, rx) = crossbeam_channel::bounded(4);
    let (mut h, _) = support::harness_sized(library(5, 2), egui::vec2(1000.0, 1600.0), move |ui| {
        ui.with_services(tx)
    });
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Later").click();
    h.run_steps(2);
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Analysis").click();
    h.run_steps(2);
    // The window has a fixed size: the section body scrolls.
    h.get_by_role_and_label(Role::Button, "Analyse outdated tracks (2)")
        .scroll_to_me();
    h.run_steps(3);
    h.get_by_role_and_label(Role::Button, "Analyse outdated tracks (2)")
        .click();
    h.run_steps(2);
    assert_eq!(rx.try_recv(), Ok(ServiceRequest::AnalyseOutdated));
}
