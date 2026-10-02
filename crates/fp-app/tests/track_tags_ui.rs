#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O23: the row tooltip and the tag editor.

mod support;

use egui_kittest::kittest::Queryable;
use fp_model::{AppState, AudioFormat, FileState};
use support::{harness, state};

/// `state(1, 3)` whose first track has tags and a format.
fn tagged_state() -> AppState {
    let mut s = state(1, 3);
    let first = s.library.iter().next().unwrap().id;
    let t = s.library.get_mut(first).unwrap();
    t.title = "Song 1".into();
    t.artist = "The Artist".into();
    t.album = "An Album".into();
    t.date = Some("1999-03-07".into());
    t.genre = "Pop".into();
    t.duration_secs = 200.0;
    t.analyzed = true;
    t.tags_read = true;
    t.file_state = FileState::Ok;
    t.format = Some(AudioFormat {
        sample_rate: 44_100,
        bits: Some(16),
        channels: 2,
    });
    s
}

#[test]
fn hovering_a_row_shows_the_tags_format_and_path() {
    let (mut h, _fake) = harness(tagged_state());
    h.get_all_by_label("Song 1").last().unwrap().hover();
    h.run_steps(40);
    // The field labels exist only in the tooltip.
    for label in ["Title", "Format", "Path", "Genre"] {
        assert!(
            h.query_by_label(label).is_some(),
            "the tooltip labels {label}"
        );
    }
    for text in [
        "The Artist",
        "An Album",
        "1999-03-07",
        "Pop",
        "03:20",
        "MP3 · 44.1 kHz · 16 bit",
        "/music/Song 1.mp3",
    ] {
        // The row shows some of these too, so more than one node can match.
        assert!(
            h.query_all_by_label_contains(text).next().is_some(),
            "the tooltip shows {text}"
        );
    }
}

#[test]
fn the_tooltip_waits_for_the_usual_delay() {
    let (mut h, _fake) = harness(tagged_state());
    h.get_all_by_label("Song 1").last().unwrap().hover();
    h.run_steps(2);
    assert!(h.query_by_label_contains("An Album").is_none());
}
