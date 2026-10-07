#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Operator feedback 4, Q10.4: the row menu's Re-analyse.

mod support;

use egui_kittest::kittest::Queryable;
use fp_app::services::ServiceRequest;
use support::{harness, harness_from, state};

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
