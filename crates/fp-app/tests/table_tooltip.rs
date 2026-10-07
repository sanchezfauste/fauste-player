#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Operator feedback 4, Q9: the row's track info popup.

mod support;

use egui_kittest::kittest::Queryable;
use fp_model::{AppState, Command, FileState};
use support::{harness, harness_sized, state};

/// Three tracks; "Song 2" cannot be read.
fn unreadable() -> AppState {
    let mut s = state(1, 3);
    let playlist = s.playlists.first_id().unwrap();
    let track = s.playlists.get(playlist).unwrap().entries[1].track;
    fp_model::apply(
        &mut s,
        Command::SetFileState {
            track,
            state: FileState::Unreadable,
        },
    )
    .unwrap();
    s
}

#[test]
fn q9_1_the_popup_starts_with_the_reason_above_the_fields() {
    let (mut h, _) = harness(unreadable());
    h.get_by_label("Song 2").hover();
    h.run_steps(40);
    let reason = h
        .get_by_label_contains("Cannot read the file: /music/Song 2.mp3")
        .rect();
    let title_field = h.get_by_label("Title").rect();
    assert!(
        reason.bottom() <= title_field.top() + 0.5,
        "{reason:?} {title_field:?}"
    );
}

#[test]
fn q9_1_a_readable_track_has_no_reason_line() {
    let (mut h, _) = harness(state(1, 3));
    h.get_by_label("Song 2").hover();
    h.run_steps(40);
    assert!(h.query_by_label("Title").is_some(), "the popup is there");
    assert!(h.query_by_label_contains("Cannot read").is_none());
    assert!(h.query_by_label_contains("File not found").is_none());
}

#[test]
fn q9_2_the_number_and_the_title_have_no_tooltip_of_their_own() {
    // With one tooltip per layer the row's popup is the only one: hovering
    // the icon of the number and the title give the same popup content.
    let (mut h, _) = harness(unreadable());
    h.get_by_label_contains(egui_phosphor::regular::WARNING)
        .hover();
    h.run_steps(40);
    assert_eq!(
        h.query_all_by_label_contains("Cannot read the file")
            .count(),
        1
    );
}

/// Whether a text shape with exactly `text` was painted in the last frame.
fn painted(h: &egui_kittest::Harness<'_, fp_app::ui::app::AppUi>, text: &str) -> bool {
    h.output().shapes.iter().any(|s| match &s.shape {
        egui::Shape::Text(t) => t.galley.text() == text,
        _ => false,
    })
}

#[test]
fn q9_3_the_popup_never_shows_at_another_place_first() {
    let (mut h, _) = harness(unreadable());
    h.hover_at(h.get_by_label("Song 2").rect().center());
    let mut first = None;
    for _ in 0..80 {
        h.step();
        // egui lays a new popup out once invisibly (its sizing pass paints
        // nothing); the first frame that paints the label is the first the
        // operator sees.
        if painted(&h, "Path")
            && let Some(n) = h.query_by_label("Path")
        {
            first = Some(n.rect());
            break;
        }
    }
    let first = first.expect("the popup appears");
    h.run_steps(5);
    let later = h.get_by_label("Path").rect();
    assert!(
        (first.min - later.min).length() < 0.5 && (first.size() - later.size()).length() < 0.5,
        "{first:?} then {later:?}"
    );
}

#[test]
fn the_popup_stays_inside_a_narrow_window() {
    let (mut h, _) = harness_sized(unreadable(), egui::vec2(380.0, 700.0), |ui| ui);
    h.get_by_label("Song 2").hover();
    h.run_steps(40);
    let reason = h.get_by_label_contains("Cannot read the file").rect();
    assert!(
        reason.left() >= 0.0 && reason.right() <= 380.5,
        "{reason:?}"
    );
}
