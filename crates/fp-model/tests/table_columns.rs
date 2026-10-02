#![allow(clippy::unwrap_used)]
//! The columns of the track table (feedback 2 spec O24).

use fp_model::TableColumn::{Album, Artist, Date, Duration, FileName, Genre, Intro, Number, Title};
use fp_model::{
    Config, TableColumn, column_rows, default_columns, move_column, move_column_before,
    normalize_columns, with_column_shown,
};

#[test]
fn the_default_is_what_the_table_always_showed() {
    assert_eq!(default_columns(), vec![Number, Title, Artist, Duration]);
    assert_eq!(Config::default().ui.table_columns, default_columns());
}

#[test]
fn title_and_duration_are_the_required_columns() {
    let required: Vec<_> = TableColumn::ALL
        .into_iter()
        .filter(|c| c.is_required())
        .collect();
    assert_eq!(required, vec![Title, Duration]);
    assert_eq!(TableColumn::ALL.len(), 9);
}

#[test]
fn a_repeated_column_keeps_its_first_place() {
    assert_eq!(
        normalize_columns(&[Title, Artist, Title, Duration, Artist]),
        vec![Title, Artist, Duration]
    );
}

#[test]
fn a_missing_title_goes_first_or_after_the_number() {
    assert_eq!(
        normalize_columns(&[Artist, Duration]),
        vec![Title, Artist, Duration]
    );
    assert_eq!(
        normalize_columns(&[Number, Artist, Duration]),
        vec![Number, Title, Artist, Duration]
    );
}

#[test]
fn a_missing_duration_goes_last() {
    assert_eq!(
        normalize_columns(&[Title, Album]),
        vec![Title, Album, Duration]
    );
}

#[test]
fn an_empty_list_keeps_only_the_required_columns() {
    assert_eq!(normalize_columns(&[]), vec![Title, Duration]);
}

#[test]
fn a_good_list_is_not_changed_and_normalising_is_idempotent() {
    let list = vec![Duration, Genre, Title, Number, Intro, FileName, Date, Album];
    assert_eq!(normalize_columns(&list), list);
    let bad = [Artist, Artist];
    let once = normalize_columns(&bad);
    assert_eq!(normalize_columns(&once), once);
}

#[test]
fn showing_a_column_appends_it_and_showing_it_twice_changes_nothing() {
    let list = default_columns();
    let shown = with_column_shown(&list, Album, true);
    assert_eq!(shown, vec![Number, Title, Artist, Duration, Album]);
    assert_eq!(with_column_shown(&shown, Album, true), shown);
}

#[test]
fn hiding_a_column_removes_it_but_never_a_required_one() {
    let list = vec![Number, Title, Artist, Duration];
    assert_eq!(
        with_column_shown(&list, Artist, false),
        vec![Number, Title, Duration]
    );
    assert_eq!(with_column_shown(&list, Title, false), list);
    assert_eq!(with_column_shown(&list, Duration, false), list);
    assert_eq!(with_column_shown(&list, Genre, false), list);
}

#[test]
fn any_column_can_move_even_a_required_one() {
    let list = vec![Number, Title, Artist, Duration];
    assert_eq!(
        move_column(&list, 3, 0),
        vec![Duration, Number, Title, Artist]
    );
    assert_eq!(
        move_column(&list, 1, 2),
        vec![Number, Artist, Title, Duration]
    );
    assert_eq!(move_column(&list, 2, 2), list);
}

#[test]
fn a_move_outside_the_list_is_clamped() {
    let list = vec![Number, Title, Artist, Duration];
    assert_eq!(
        move_column(&list, 0, 99),
        vec![Title, Artist, Duration, Number]
    );
    assert_eq!(
        move_column(&list, 99, 0),
        vec![Duration, Number, Title, Artist]
    );
}

#[test]
fn the_settings_rows_list_the_shown_columns_then_the_hidden_ones() {
    let rows = column_rows(&[Duration, Title]);
    let shown: Vec<_> = rows.iter().filter(|(_, on)| *on).map(|(c, _)| *c).collect();
    let hidden: Vec<_> = rows
        .iter()
        .filter(|(_, on)| !*on)
        .map(|(c, _)| *c)
        .collect();
    assert_eq!(shown, vec![Duration, Title]);
    assert_eq!(
        hidden,
        vec![Number, Artist, Album, Date, Genre, Intro, FileName]
    );
    assert_eq!(rows.len(), 9);
    assert!(rows[..2].iter().all(|(_, on)| *on));
}

#[test]
fn validate_repairs_the_list_and_says_so() {
    let mut c = Config::default();
    c.ui.table_columns = vec![Artist, Artist];
    let warnings = c.validate();
    assert_eq!(c.ui.table_columns, vec![Title, Artist, Duration]);
    assert!(warnings.iter().any(|w| w.field == "ui.table_columns"));
    assert!(
        Config::default()
            .validate()
            .iter()
            .all(|w| w.field != "ui.table_columns")
    );
}

#[test]
fn the_list_is_written_with_stable_names() {
    let json = serde_json::to_string(&vec![Number, FileName, Intro]).unwrap();
    assert_eq!(json, r#"["number","file_name","intro"]"#);
}

#[test]
fn a_header_dropped_before_a_slot_lands_just_before_that_column() {
    let list = vec![Number, Title, Artist, Duration];
    assert_eq!(
        move_column_before(&list, 0, 3),
        vec![Title, Artist, Number, Duration]
    );
    assert_eq!(
        move_column_before(&list, 3, 0),
        vec![Duration, Number, Title, Artist]
    );
    assert_eq!(
        move_column_before(&list, 3, 1),
        vec![Number, Duration, Title, Artist]
    );
}

#[test]
fn a_header_dropped_at_the_end_goes_last() {
    let list = vec![Number, Title, Artist, Duration];
    assert_eq!(
        move_column_before(&list, 0, 4),
        vec![Title, Artist, Duration, Number]
    );
    assert_eq!(
        move_column_before(&list, 0, 99),
        vec![Title, Artist, Duration, Number]
    );
}

#[test]
fn a_header_dropped_beside_itself_changes_nothing() {
    let list = vec![Number, Title, Artist, Duration];
    assert_eq!(move_column_before(&list, 2, 2), list);
    assert_eq!(move_column_before(&list, 2, 3), list);
}

#[test]
fn a_column_is_found_by_the_name_it_has_in_files() {
    for column in TableColumn::ALL {
        assert_eq!(TableColumn::from_name(column.name()), Some(column));
    }
    assert_eq!(TableColumn::from_name("file_name"), Some(FileName));
    assert_eq!(TableColumn::from_name("file-name"), None);
    assert_eq!(TableColumn::from_name("bpm"), None);
}
