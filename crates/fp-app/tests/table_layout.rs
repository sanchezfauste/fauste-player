#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
//! The widths of the track table's columns (feedback 2 spec O16 and O24).

use fp_app::ui::table_layout::{column_min, column_px, fit, fractions_of, resize_px};
use fp_model::TableColumn::{Album, Artist, Date, Duration, FileName, Genre, Intro, Number, Title};
use fp_model::{ColumnWidths, TableColumn, default_columns};

fn sum(px: &[f32]) -> f32 {
    px.iter().sum()
}

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.01
}

fn mins(columns: &[TableColumn]) -> Vec<f32> {
    columns.iter().map(|c| column_min(*c, 2)).collect()
}

// --- column_px

#[test]
fn the_default_layout_is_what_the_table_always_had() {
    let px = column_px(&default_columns(), &ColumnWidths::default(), 1000.0, 2);
    let number_min = column_min(Number, 2);
    assert!(near(px[0], number_min), "{px:?}");
    assert!(near(px[3], column_min(Duration, 2)));
    let rest = 1000.0 - number_min - column_min(Duration, 2);
    assert!(near(px[1], rest * 0.6) && near(px[2], rest * 0.4), "{px:?}");
    assert!(near(sum(&px), 1000.0));
}

#[test]
fn narrow_columns_stay_at_their_minimum_and_text_columns_share_the_rest() {
    let columns = [
        Number, Title, Artist, Album, Date, Genre, Duration, Intro, FileName,
    ];
    let px = column_px(&columns, &ColumnWidths::default(), 1600.0, 3);
    assert!(near(px[0], column_min(Number, 3)));
    assert!(
        near(px[4], 84.0) && near(px[6], 52.0) && near(px[7], 52.0),
        "{px:?}"
    );
    // Title 3 : Artist 2 : Album 2 : Genre 1.5 : File name 2.
    assert!(
        near(px[1] / px[2], 1.5) && near(px[2], px[3]) && near(px[8], px[2]),
        "{px:?}"
    );
    assert!(near(px[5] / px[2], 0.75));
    assert!(near(sum(&px), 1600.0));
}

#[test]
fn stored_fractions_scale_with_the_width() {
    let w = ColumnWidths::keyed([
        (Number, 0.05),
        (Title, 0.5),
        (Artist, 0.35),
        (Duration, 0.1),
    ]);
    let a = column_px(&default_columns(), &w, 1000.0, 2);
    let b = column_px(&default_columns(), &w, 1500.0, 2);
    assert!(near(sum(&b), 1500.0));
    assert!(
        near(b[1] / a[1], 1.5) && near(b[2] / a[2], 1.5),
        "{a:?} {b:?}"
    );
}

#[test]
fn a_shown_column_without_a_stored_width_takes_its_default_and_the_rest_keep_their_shares() {
    let w = ColumnWidths::keyed([
        (Number, 0.05),
        (Title, 0.5),
        (Artist, 0.35),
        (Duration, 0.1),
    ]);
    let columns = [Number, Title, Artist, Date, Duration];
    let px = column_px(&columns, &w, 1000.0, 2);
    assert!(near(px[3], 84.0), "Date at its default: {px:?}");
    assert!(near(sum(&px), 1000.0));
    // The stored ones keep their proportions: Title:Artist is still 0.5:0.35.
    assert!(near(px[1] / px[2], 0.5 / 0.35), "{px:?}");
}

#[test]
fn a_hidden_column_gives_its_room_to_the_others_in_proportion() {
    let w = ColumnWidths::keyed([
        (Number, 0.05),
        (Title, 0.5),
        (Artist, 0.35),
        (Duration, 0.1),
    ]);
    let columns = [Title, Duration];
    let px = column_px(&columns, &w, 1000.0, 2);
    assert!(near(sum(&px), 1000.0));
    assert!(near(px[0] / px[1], 5.0), "0.5 : 0.1 -> {px:?}");
}

#[test]
fn a_reordered_list_keeps_each_columns_width() {
    let w = ColumnWidths::keyed([(Title, 0.5), (Artist, 0.3), (Duration, 0.2)]);
    let a = column_px(&[Title, Artist, Duration], &w, 1000.0, 2);
    let b = column_px(&[Duration, Artist, Title], &w, 1000.0, 2);
    assert!(
        near(a[0], b[2]) && near(a[1], b[1]) && near(a[2], b[0]),
        "{a:?} {b:?}"
    );
}

#[test]
fn the_minimums_win_in_a_narrow_table() {
    let w = ColumnWidths::keyed([
        (Number, 0.01),
        (Title, 0.6),
        (Artist, 0.38),
        (Duration, 0.01),
    ]);
    let px = column_px(&default_columns(), &w, 360.0, 2);
    assert!(
        px[0] >= column_min(Number, 2) - 0.01 && px[3] >= 52.0 - 0.01,
        "{px:?}"
    );
    assert!(near(sum(&px), 360.0));
    let tiny = column_px(&default_columns(), &ColumnWidths::default(), 50.0, 2);
    assert!(tiny.iter().all(|w| *w >= 0.0 && w.is_finite()), "{tiny:?}");
    assert!(near(sum(&tiny), 50.0));
}

#[test]
fn nonsense_widths_never_give_nonsense_pixels() {
    for width in [f32::NAN, f32::INFINITY, -5.0, 0.0] {
        let px = column_px(&default_columns(), &ColumnWidths::default(), width, 2);
        assert!(
            px.iter().all(|w| w.is_finite() && *w >= 0.0),
            "{width}: {px:?}"
        );
    }
    let broken = ColumnWidths {
        fractions: Some([(Title, f32::NAN)].into()),
    };
    let px = column_px(&default_columns(), &broken, 800.0, 2);
    assert!(near(sum(&px), 800.0), "{px:?}");
}

// --- fit

#[test]
fn fit_pins_what_is_too_narrow_and_shares_the_rest() {
    let px = fit(&[10.0, 300.0, 300.0], &[50.0, 60.0, 60.0], 600.0);
    assert!(
        near(px[0], 50.0) && near(px[1], 275.0) && near(px[2], 275.0),
        "{px:?}"
    );
}

#[test]
fn fit_scales_the_minimums_when_nothing_fits() {
    let px = fit(&[1.0, 1.0], &[60.0, 60.0], 60.0);
    assert!(near(px[0], 30.0) && near(px[1], 30.0), "{px:?}");
    assert!(fit(&[], &[], 100.0).is_empty());
}

// --- resize_px

#[test]
fn dragging_an_edge_moves_all_the_columns_on_its_right() {
    let columns = [Number, Title, Artist, Album, Duration];
    let m = mins(&columns);
    let start = [44.0, 300.0, 200.0, 200.0, 100.0];
    let out = resize_px(&start, &m, 1, 400.0);
    assert!(near(out[0], 44.0), "left of the edge: unchanged");
    assert!(near(out[1], 400.0));
    assert!(near(sum(&out), sum(&start)));
    // Artist, Album and Duration share what is left (400) as 200:200:100.
    assert!(
        near(out[2], 160.0) && near(out[3], 160.0) && near(out[4], 80.0),
        "{out:?}"
    );
}

#[test]
fn every_step_of_a_drag_keeps_the_total_and_the_minimums() {
    let columns = [Number, Title, Artist, Album, Duration];
    let m = mins(&columns);
    let start = [44.0, 300.0, 200.0, 200.0, 100.0];
    for i in 0..=100 {
        let wanted = -50.0 + i as f32 * 12.0;
        for edge in 0..5 {
            let out = resize_px(&start, &m, edge, wanted);
            assert!(
                near(sum(&out), sum(&start)),
                "edge {edge} wanted {wanted}: {out:?}"
            );
            for (w, min) in out.iter().zip(&m) {
                assert!(
                    *w >= min - 0.01 && w.is_finite(),
                    "edge {edge} wanted {wanted}: {out:?}"
                );
            }
        }
    }
}

#[test]
fn an_edge_cannot_be_pushed_past_the_minimums_of_the_columns_after_it() {
    let columns = [Title, Artist, Duration];
    let m = mins(&columns);
    let start = [400.0, 300.0, 100.0];
    let out = resize_px(&start, &m, 0, 10_000.0);
    assert!(near(out[1], m[1]) && near(out[2], m[2]), "{out:?}");
    assert!(near(sum(&out), 800.0));
}

#[test]
fn an_edge_cannot_shrink_a_column_below_its_minimum() {
    let columns = [Title, Artist, Duration];
    let m = mins(&columns);
    let out = resize_px(&[400.0, 300.0, 100.0], &m, 0, -50.0);
    assert!(near(out[0], m[0]), "{out:?}");
}

#[test]
fn the_last_column_has_no_edge_and_bad_input_changes_nothing() {
    let start = [300.0, 300.0, 100.0];
    let m = mins(&[Title, Artist, Duration]);
    assert_eq!(resize_px(&start, &m, 2, 500.0), start);
    assert_eq!(resize_px(&start, &m, 9, 500.0), start);
    assert_eq!(resize_px(&start, &m, 0, f32::NAN), start);
    assert!(resize_px(&[], &[], 0, 5.0).is_empty());
}

#[test]
fn a_drag_back_to_where_it_began_restores_the_widths() {
    let m = mins(&[Title, Artist, Album, Duration]);
    let start = [300.0, 200.0, 200.0, 100.0];
    let out = resize_px(&start, &m, 1, 200.0);
    for (a, b) in out.iter().zip(&start) {
        assert!(near(*a, *b), "{out:?}");
    }
}

// --- fractions_of

#[test]
fn the_stored_widths_are_each_columns_share() {
    let columns = [Title, Genre, Duration];
    let w = fractions_of(&columns, &[600.0, 300.0, 100.0]);
    assert!(near(w.fraction(Title).unwrap(), 0.6));
    assert!(near(w.fraction(Genre).unwrap(), 0.3));
    assert!(near(w.fraction(Duration).unwrap(), 0.1));
    assert_eq!(w.fraction(Artist), None);
}
