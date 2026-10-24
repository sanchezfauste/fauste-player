#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Operator feedback 4, Q10.4: the row menu's Re-analyse.

use crate::support;

use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT, Queryable};
use fp_app::activity::{AnalysisActivity, PassProgress};
use fp_app::services::ServiceRequest;
use support::{activity_cell, harness, harness_from, harness_sized, nth_track, state};

#[test]
fn the_row_menu_re_analyses_that_track() {
    let (tx, rx) = crossbeam_channel::bounded(4);
    let (mut h, fake) = harness_from(state(1, 3), move |ui| ui.with_services(tx));
    let track = fake.state.load().playlists.iter().next().unwrap().entries[1].track;
    h.get_all_by_label("Song 2")
        .last()
        .unwrap()
        .click_secondary();
    h.run_steps(2);
    h.get_by_label("Re-analyse").click();
    h.run_steps(2);
    assert_eq!(rx.try_recv(), Ok(ServiceRequest::ReanalyseTrack(track)));
    assert!(rx.try_recv().is_err(), "one request");
}

#[test]
fn the_item_is_in_the_menu_even_without_services() {
    let (mut h, _) = harness(state(1, 3));
    h.get_all_by_label("Song 2")
        .last()
        .unwrap()
        .click_secondary();
    h.run_steps(2);
    h.get_by_label("Re-analyse").click();
    h.run_steps(2);
    assert!(h.query_by_label("Re-analyse").is_none(), "the menu closed");
}

/// Opens the row menu of "Song 2".
fn row_menu(h: &mut egui_kittest::Harness<'static, fp_app::ui::app::AppUi>) {
    h.get_all_by_label("Song 2")
        .last()
        .unwrap()
        .click_secondary();
    h.run_steps(2);
}

/// Main spec §6.1: Re-analyse of a track already queued or running is
/// dimmed and says why.
fn the_item_is_dimmed_while_the_track_is(running: bool, reason: &str) {
    let s = state(1, 3);
    let track = nth_track(&s, 1);
    let mut activity = AnalysisActivity::default();
    if running {
        activity.running.push(track);
    } else {
        activity.queued.insert(track);
    }
    let cell = activity_cell(activity);
    let (tx, rx) = crossbeam_channel::bounded(4);
    let (mut h, _) = harness_from(s, move |ui| {
        ui.with_services(tx).with_analysis_activity(cell)
    });
    row_menu(&mut h);
    let item = h.get_by_label("Re-analyse");
    assert!(item.accesskit_node().is_disabled());
    item.hover();
    h.run_steps(40);
    assert!(h.query_by_label_contains(reason).is_some(), "{reason}");
    h.get_by_label("Re-analyse").click();
    h.run_steps(2);
    assert!(rx.try_recv().is_err(), "no request");
}

#[test]
fn the_item_is_dimmed_for_a_queued_track() {
    the_item_is_dimmed_while_the_track_is(false, "Already waiting for analysis");
}

#[test]
fn the_item_is_dimmed_for_a_running_track() {
    the_item_is_dimmed_while_the_track_is(true, "Being analysed now");
}

/// Opens Settings → Analysis in a tall window.
fn analysis_settings(
    activity: AnalysisActivity,
) -> (
    egui_kittest::Harness<'static, fp_app::ui::app::AppUi>,
    crossbeam_channel::Receiver<ServiceRequest>,
    fp_app::activity::ActivityCell,
) {
    let cell = activity_cell(activity);
    let shared = cell.clone();
    let (tx, rx) = crossbeam_channel::bounded(4);
    let (mut h, _) = harness_sized(state(1, 3), egui::vec2(1000.0, 1600.0), move |ui| {
        ui.with_services(tx).with_analysis_activity(shared)
    });
    h.run_steps(2);
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Analysis").click();
    h.run_steps(2);
    (h, rx, cell)
}

/// Main spec §6.1: while the library pass runs, the button is dimmed and
/// shows its progress; once it ends the button works again.
#[test]
fn the_reanalyse_all_button_shows_the_pass_and_waits_for_it() {
    let (mut h, rx, cell) = analysis_settings(AnalysisActivity {
        pass: Some(PassProgress {
            done: 120,
            total: 340,
        }),
        ..Default::default()
    });
    let label = "Analysing… 120 of 340";
    h.get_by_role_and_label(Role::Button, label).scroll_to_me();
    h.run_steps(3);
    let button = h.get_by_role_and_label(Role::Button, label);
    assert!(button.accesskit_node().is_disabled());
    button.click();
    h.run_steps(2);
    assert!(rx.try_recv().is_err(), "no request while the pass runs");
    cell.store(std::sync::Arc::new(AnalysisActivity::default()));
    h.run_steps(2);
    let button = h.get_by_role_and_label(Role::Button, "Re-analyse all tracks");
    assert!(!button.accesskit_node().is_disabled());
    button.click();
    h.run_steps(2);
    assert_eq!(rx.try_recv(), Ok(ServiceRequest::ReanalyseAll));
}
