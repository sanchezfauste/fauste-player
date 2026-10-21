#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Settings > Analysis: every value says in one line what it does.

mod support;

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable;
use support::{harness, state};

const HINTS: [(&str, &str); 7] = [
    (
        "Trim threshold",
        "Below this level the start and end of a track count as silence and are skipped.",
    ),
    (
        "Trim margin",
        "Extra time kept around the first and last sound so that soft starts and tails are not cut.",
    ),
    (
        "Segue drop",
        "How far below the track's usual loudness the ending must fall for the MIX point to be placed there.",
    ),
    (
        "Maximum mix length",
        "The longest overlap between two tracks: the MIX point is never more than this before the end.",
    ),
    (
        "Outro level drop",
        "How far the level must fall below the track's median loudness for the outro to start.",
    ),
    (
        "Maximum outro length",
        "The longest outro that is marked, counted back from the end of the track.",
    ),
    (
        "Minimum length for mix and outro markers",
        "Tracks shorter than this, such as jingles and ads, get no MIX and no outro.",
    ),
];

#[test]
fn every_analysis_value_has_a_one_line_hint() {
    let (mut h, _) = harness(state(1, 1));
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Analysis").click();
    h.run_steps(2);
    for (label, hint) in HINTS {
        assert!(h.query_all_by_label(label).next().is_some(), "{label}");
        h.get_by_label(hint);
    }
}

const LOUDNESS_HINTS: [(&str, &str); 4] = [
    ("Target", "The loudness every track is brought to."),
    (
        "True-peak ceiling",
        "A boost never takes a true peak above this.",
    ),
    ("Maximum boost", "The most a quiet track is turned up."),
    ("Maximum trim", "How far a track's trim may go."),
];

#[test]
fn every_loudness_value_has_a_hint() {
    let (mut h, _) = harness(state(1, 1));
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Loudness").click();
    h.run_steps(2);
    for (label, hint) in LOUDNESS_HINTS {
        assert!(h.query_all_by_label(label).next().is_some(), "{label}");
        h.get_by_label(hint);
    }
}
