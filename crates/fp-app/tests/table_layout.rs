#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
//! The widths of the track table's columns (feedback 2 spec O16 and O24).

use egui::{Rect, pos2, vec2};
use fp_app::ui::table_layout::{
    DRAG_SCROLL_EDGE, DRAG_SCROLL_MAX_SPEED, boundary_y, column_min, column_px, drag_scroll_speed,
    drop_index, fit, fractions_of, on_column_edge, resize_px,
};
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

// --- drop_index, boundary_y, on_column_edge (operator feedback 4, Q4)

/// A body 500 x 280 points (ten rows of 28) starting at (100, 50).
fn body() -> Rect {
    Rect::from_min_size(pos2(100.0, 50.0), vec2(500.0, 280.0))
}

#[test]
fn the_top_of_the_body_is_boundary_zero() {
    assert_eq!(drop_index(body(), 0.0, 28.0, 5, pos2(200.0, 50.0)), Some(0));
    assert_eq!(drop_index(body(), 0.0, 28.0, 5, pos2(200.0, 60.0)), Some(0));
}

#[test]
fn the_nearest_boundary_wins_in_the_middle_of_the_list() {
    // Row 3 spans y = 134..162: its upper part is boundary 3, its lower 4.
    assert_eq!(
        drop_index(body(), 0.0, 28.0, 9, pos2(200.0, 139.0)),
        Some(3)
    );
    assert_eq!(
        drop_index(body(), 0.0, 28.0, 9, pos2(200.0, 154.0)),
        Some(4)
    );
}

#[test]
fn below_the_last_row_the_index_is_the_length() {
    assert_eq!(
        drop_index(body(), 0.0, 28.0, 5, pos2(200.0, 320.0)),
        Some(5)
    );
}

#[test]
fn a_scrolled_body_shifts_the_boundaries() {
    // Scrolled by two rows: the top of the view is boundary 2.
    assert_eq!(
        drop_index(body(), 56.0, 28.0, 100, pos2(200.0, 51.0)),
        Some(2)
    );
    // Scrolled by ten and a half rows: the top of the view is half way
    // between boundaries 10 and 11; the tie goes down.
    assert_eq!(
        drop_index(body(), 294.0, 28.0, 100, pos2(200.0, 50.0)),
        Some(11)
    );
}

#[test]
fn an_empty_list_has_the_one_boundary_zero() {
    assert_eq!(
        drop_index(body(), 0.0, 28.0, 0, pos2(200.0, 200.0)),
        Some(0)
    );
}

#[test]
fn outside_the_body_there_is_no_target() {
    for pointer in [
        pos2(200.0, 40.0),  // the header
        pos2(90.0, 100.0),  // left of the table
        pos2(601.0, 100.0), // the scroll bar and beyond
        pos2(200.0, 331.0), // below the table
        pos2(f32::NAN, 100.0),
    ] {
        assert_eq!(
            drop_index(body(), 0.0, 28.0, 5, pointer),
            None,
            "{pointer:?}"
        );
    }
}

#[test]
fn a_broken_row_height_gives_no_target() {
    for height in [0.0, -28.0, f32::NAN, f32::INFINITY] {
        assert_eq!(drop_index(body(), 0.0, height, 5, pos2(200.0, 100.0)), None);
    }
}

#[test]
fn a_boundary_is_drawn_at_the_top_of_its_row() {
    assert_eq!(boundary_y(body(), 0.0, 28.0, 0), Some(50.0));
    assert_eq!(boundary_y(body(), 0.0, 28.0, 3), Some(134.0));
    assert_eq!(boundary_y(body(), 56.0, 28.0, 2), Some(50.0));
}

#[test]
fn a_boundary_outside_the_view_has_no_y() {
    assert_eq!(boundary_y(body(), 56.0, 28.0, 1), None, "above the view");
    assert_eq!(
        boundary_y(body(), 56.0, 28.0, 12),
        Some(330.0),
        "the bottom edge"
    );
    assert_eq!(boundary_y(body(), 56.0, 28.0, 13), None, "below the view");
}

#[test]
fn the_interior_column_edges_are_grab_zones_and_the_last_edge_is_not() {
    let px = [60.0, 200.0, 100.0, 140.0];
    // Edges at 160, 360 and 460 for a table starting at x = 100.
    assert!(on_column_edge(&px, 100.0, 4.0, 160.0));
    assert!(on_column_edge(&px, 100.0, 4.0, 163.9));
    assert!(on_column_edge(&px, 100.0, 4.0, 456.5));
    assert!(!on_column_edge(&px, 100.0, 4.0, 165.0));
    assert!(
        !on_column_edge(&px, 100.0, 4.0, 100.0),
        "the table's own edge"
    );
    assert!(
        !on_column_edge(&px, 100.0, 4.0, 600.0),
        "the last column's right edge"
    );
    assert!(!on_column_edge(&[], 100.0, 4.0, 100.0));
}

fn tall_body() -> Rect {
    Rect::from_min_size(pos2(0.0, 100.0), vec2(400.0, 400.0))
}

#[test]
fn drag_scroll_is_still_away_from_the_edges() {
    let body = tall_body();
    assert_eq!(drag_scroll_speed(body, pos2(50.0, 300.0)), 0.0);
    assert_eq!(
        drag_scroll_speed(body, pos2(50.0, 100.0 + DRAG_SCROLL_EDGE + 1.0)),
        0.0
    );
    assert_eq!(
        drag_scroll_speed(body, pos2(50.0, 500.0 - DRAG_SCROLL_EDGE - 1.0)),
        0.0
    );
}

#[test]
fn drag_scroll_ramps_up_towards_each_edge() {
    let body = tall_body();
    let half = DRAG_SCROLL_EDGE / 2.0;
    assert!(near(
        drag_scroll_speed(body, pos2(50.0, 100.0)),
        -DRAG_SCROLL_MAX_SPEED
    ));
    assert!(near(
        drag_scroll_speed(body, pos2(50.0, 500.0)),
        DRAG_SCROLL_MAX_SPEED
    ));
    assert!(near(
        drag_scroll_speed(body, pos2(50.0, 100.0 + half)),
        -DRAG_SCROLL_MAX_SPEED / 2.0
    ));
    assert!(near(
        drag_scroll_speed(body, pos2(50.0, 500.0 - half)),
        DRAG_SCROLL_MAX_SPEED / 2.0
    ));
}

#[test]
fn drag_scroll_needs_the_pointer_inside_the_body() {
    let body = tall_body();
    assert_eq!(drag_scroll_speed(body, pos2(50.0, 99.0)), 0.0);
    assert_eq!(drag_scroll_speed(body, pos2(50.0, 501.0)), 0.0);
    assert_eq!(drag_scroll_speed(body, pos2(401.0, 490.0)), 0.0);
    assert_eq!(drag_scroll_speed(body, pos2(f32::NAN, 490.0)), 0.0);
}

#[test]
fn a_short_body_keeps_a_still_middle() {
    // 60 points high: each edge zone is a third of it at most.
    let body = Rect::from_min_size(pos2(0.0, 100.0), vec2(400.0, 60.0));
    assert_eq!(drag_scroll_speed(body, pos2(50.0, 130.0)), 0.0);
    assert!(drag_scroll_speed(body, pos2(50.0, 105.0)) < 0.0);
    assert!(drag_scroll_speed(body, pos2(50.0, 155.0)) > 0.0);
}
