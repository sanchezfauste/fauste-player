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
use fp_app::ui::controller::Controller;
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

/// Counting scans the library: it is done again when the model changed, not
/// on every frame while the notice or Settings > Analysis is open.
#[test]
fn the_count_is_worked_out_again_only_when_the_model_changed() {
    use std::sync::Arc;
    let first = Arc::new(library(5, 3));
    let mut count = fp_app::services::OutdatedCount::default();
    assert_eq!(count.get(&first), 3);
    for _ in 0..100 {
        assert_eq!(count.get(&first), 3);
        assert_eq!(count.get(&Arc::clone(&first)), 3, "the same snapshot");
    }
    assert_eq!(count.scans(), 1);
    let second = Arc::new(library(5, 1));
    assert_eq!(count.get(&second), 1);
    assert_eq!(count.get(&second), 1);
    assert_eq!(count.scans(), 2);
    // An equal library in a new snapshot is a change as far as the cache
    // can tell: it never compares libraries.
    let third = Arc::new(library(5, 1));
    assert_eq!(count.get(&third), 1);
    assert_eq!(count.scans(), 3);
}

#[test]
fn the_notice_follows_the_library_while_it_is_open() {
    let (mut h, fake, _rx) = {
        let (tx, rx) = crossbeam_channel::bounded(4);
        let (h, fake) = harness_from(library(5, 3), move |ui| ui.with_services(tx));
        (h, fake, rx)
    };
    h.run_steps(2);
    assert!(
        h.query_by_label_contains("3 tracks were analysed by an earlier version")
            .is_some()
    );
    // One of them is not found any more: it is not counted (and cannot be
    // analysed again).
    let outdated = fake
        .state
        .load()
        .library
        .iter()
        .find(|t| t.analysis_version == 0)
        .unwrap()
        .id;
    fake.send(fp_model::Command::SetFileState {
        track: outdated,
        state: fp_model::FileState::Missing,
    });
    h.run_steps(4);
    assert!(
        h.query_by_label_contains("2 tracks were analysed by an earlier version")
            .is_some()
    );
}
