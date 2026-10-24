#![allow(clippy::unwrap_used, clippy::float_cmp)]
//! Feedback 2 spec O24: column widths are fractions keyed by column, and the
//! old four-number form is converted when it maps cleanly.

use fp_model::TableColumn::{Album, Artist, Duration, Number, Title};
use fp_model::{ColumnWidths, PlayerSession, TableColumn};

fn widths(json: &str) -> ColumnWidths {
    serde_json::from_str(json).unwrap()
}

#[test]
fn keyed_widths_round_trip_with_stable_names() {
    let w = ColumnWidths::keyed([(Title, 3.0), (Artist, 1.0)]);
    let json = serde_json::to_value(&w).unwrap();
    assert_eq!(json["fractions"]["title"], 0.75);
    assert_eq!(json["fractions"]["artist"], 0.25);
    let back: ColumnWidths = serde_json::from_value(json).unwrap();
    assert_eq!(back, w);
}

#[test]
fn the_old_four_fractions_become_the_four_default_columns() {
    let w = widths(r#"{"fractions":[0.05,0.5,0.35,0.1]}"#);
    assert_eq!(w.fraction(Number), Some(0.05));
    assert_eq!(w.fraction(Title), Some(0.5));
    assert_eq!(w.fraction(Artist), Some(0.35));
    assert_eq!(w.fraction(Duration), Some(0.1));
    assert_eq!(w.fraction(Album), None);
}

#[test]
fn an_old_array_that_does_not_map_cleanly_gives_the_default_layout() {
    for json in [
        r#"{"fractions":[0.5,0.5]}"#,
        r#"{"fractions":[0.1,0.2,0.3,0.4,0.5]}"#,
        r#"{"fractions":[]}"#,
        r#"{"fractions":["a","b","c","d"]}"#,
    ] {
        assert_eq!(widths(json).normalized().fractions, None, "{json}");
    }
}

#[test]
fn an_old_array_with_broken_numbers_is_dropped_when_normalised() {
    let w = widths(r#"{"fractions":[-1,0.5,0.35,0.1]}"#);
    assert_eq!(w.normalized().fractions, None);
    let zeros = widths(r#"{"fractions":[0,0,0,0]}"#);
    assert_eq!(zeros.normalized().fractions, None);
}

#[test]
fn an_old_array_is_scaled_to_sum_to_one() {
    let w = widths(r#"{"fractions":[1,3,2,2]}"#).normalized();
    assert!((w.fraction(Title).unwrap() - 0.375).abs() < 1e-6);
}

#[test]
fn unknown_columns_are_dropped_and_known_ones_kept() {
    let w = widths(r#"{"fractions":{"title":2.0,"bpm":9.0,"artist":2.0}}"#).normalized();
    assert_eq!(w.fraction(Title), Some(0.5));
    assert_eq!(w.fraction(Artist), Some(0.5));
    assert_eq!(w.fractions.unwrap().len(), 2);
}

#[test]
fn only_unknown_columns_give_the_default_layout() {
    let w = widths(r#"{"fractions":{"bpm":1.0}}"#).normalized();
    assert_eq!(w.fractions, None);
}

#[test]
fn a_value_of_the_wrong_type_gives_the_default_layout() {
    for json in [
        r#"{"fractions":"wide"}"#,
        r#"{"fractions":7}"#,
        r#"{"fractions":{"title":"a"}}"#,
        r#"{"fractions":null}"#,
        r#"{}"#,
    ] {
        assert_eq!(widths(json).normalized().fractions, None, "{json}");
    }
}

#[test]
fn a_session_with_the_old_widths_loads_them_keyed() {
    let session: PlayerSession = serde_json::from_value(serde_json::json!({
        "id": 1,
        "playlist": 1,
        "columns": {"fractions": [0.1, 0.5, 0.3, 0.1]}
    }))
    .unwrap();
    assert_eq!(session.columns.fraction(Artist), Some(0.3));
}

#[test]
fn normalising_twice_changes_nothing() {
    let once = ColumnWidths::keyed([(Title, 3.0), (Artist, 2.0), (Duration, 1.0)]);
    assert_eq!(once.clone().normalized(), once);
    assert_eq!(
        TableColumn::from_name("file_name"),
        Some(TableColumn::FileName)
    );
    assert_eq!(TableColumn::from_name("file-name"), None);
}
