#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Spec §4.6 and rule 26: a player whose Cue output is its Main output (or
//! that has none) has no CUE, and the interface says why.

mod support;

use egui_kittest::kittest::{NodeT, Queryable};
use fp_model::AppState;
use support::{harness, state};

const NO_OUTPUT: &str = "This player has no Cue output";
const CUE_TIP: &str = "Pre-listen (PFL) the next track";

/// One player whose Cue route is the same as its Main route.
fn cue_on_main() -> AppState {
    let mut s = state(1, 3);
    for r in &mut s.config.outputs.routes {
        r.main = r.cue.clone();
    }
    s
}

#[test]
fn the_cue_button_is_dimmed_and_says_why() {
    let (h, _) = harness(cue_on_main());
    let button = h.get_by_label_contains(NO_OUTPUT);
    assert!(button.accesskit_node().is_disabled());
    assert!(h.query_by_label(CUE_TIP).is_none());
}

#[test]
fn with_a_cue_output_the_cue_button_keeps_its_tooltip() {
    let (h, _) = harness(state(1, 3));
    assert!(!h.get_by_label(CUE_TIP).accesskit_node().is_disabled());
    assert!(h.query_by_label_contains(NO_OUTPUT).is_none());
}

#[test]
fn the_row_menu_pre_listen_is_dimmed_and_says_why() {
    let (mut h, fake) = harness(cue_on_main());
    h.get_all_by_label("Song 2")
        .max_by(|a, b| a.rect().min.y.total_cmp(&b.rect().min.y))
        .unwrap()
        .click_secondary();
    h.run_steps(3);
    // The header's CUE button already carries the reason as its name.
    let before = h.query_all_by_label_contains(NO_OUTPUT).count();
    let item = h.get_by_label("Pre-listen on CUE");
    assert!(item.accesskit_node().is_disabled());
    item.hover();
    h.run_steps(40);
    assert!(h.query_all_by_label_contains(NO_OUTPUT).count() > before);
    assert!(fake.take_sent().is_empty());
}
