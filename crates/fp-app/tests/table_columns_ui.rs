#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O16 and O24: the table draws the configured columns and
//! resizes them live.

mod support;

use egui::{Event, Modifiers, PointerButton, Pos2, pos2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_app::ui::table_layout::column_min;
use fp_model::TableColumn::{Album, Artist, Date, Duration, FileName, Genre, Intro, Number, Title};
use fp_model::{AppState, Command, MarkerKind, TableColumn};
use support::{Fake, harness, harness_sized, state};

fn with_columns(mut s: AppState, columns: &[TableColumn]) -> AppState {
    s.config.ui.table_columns = columns.to_vec();
    s
}

fn tagged(mut s: AppState) -> AppState {
    for (i, t) in s.library.iter_mut().enumerate() {
        t.duration_secs = 200.0;
        t.album = format!("Album {}", i + 1);
        t.date = Some(format!("{}-05-14", 2000 + i));
        t.genre = format!("Genre {}", i + 1);
        t.markers
            .set_manual(MarkerKind::IntroEnd, Some(12.0 + i as f64));
    }
    s
}

fn header_left(h: &Harness<'_, AppUi>, label: &str) -> f32 {
    h.get_by_label(label).rect().left()
}

#[test]
fn the_default_columns_are_number_title_artist_and_duration() {
    let (h, _) = harness(state(1, 3));
    for label in ["#", "TITLE", "ARTIST", "DUR."] {
        assert!(h.query_by_label(label).is_some(), "{label}");
    }
    for label in ["ALBUM", "DATE", "GENRE", "INTRO", "FILE NAME"] {
        assert!(h.query_by_label(label).is_none(), "{label}");
    }
}

#[test]
fn the_optional_columns_show_their_headers_and_cells() {
    let columns = [Title, Album, Date, Genre, FileName, Intro, Duration];
    let (h, _) = harness(with_columns(tagged(state(1, 3)), &columns));
    for label in ["ALBUM", "DATE", "GENRE", "FILE NAME", "INTRO"] {
        assert!(h.query_by_label(label).is_some(), "{label}");
    }
    assert!(h.query_by_label("#").is_none() && h.query_by_label("ARTIST").is_none());
    assert!(h.query_by_label("Album 2").is_some());
    assert!(h.query_by_label("2001-05-14").is_some());
    assert!(h.query_by_label("Genre 3").is_some());
    assert!(h.query_by_label("Song 2.mp3").is_some(), "the file name");
    assert!(h.query_by_label("00:13").is_some(), "the intro of track 2");
}

#[test]
fn the_columns_follow_the_order_of_the_list() {
    let (h, _) = harness(with_columns(
        tagged(state(1, 3)),
        &[Duration, Genre, Title, Album],
    ));
    let x: Vec<f32> = ["DUR.", "GENRE", "TITLE", "ALBUM"]
        .iter()
        .map(|l| header_left(&h, l))
        .collect();
    assert!(x.windows(2).all(|w| w[0] < w[1]), "{x:?}");
}

#[test]
fn a_list_without_the_required_columns_still_shows_them() {
    // Set directly, as a test or a bug could: the table repairs the list.
    let (h, _) = harness(with_columns(state(1, 3), &[Artist]));
    assert!(h.query_by_label("TITLE").is_some());
    assert!(h.query_by_label("DUR.").is_some());
    assert!(h.query_by_label("ARTIST").is_some());
}

#[test]
fn a_track_without_a_value_shows_an_empty_cell() {
    let (h, _) = harness(with_columns(state(1, 2), &[Title, Date, Intro, Duration]));
    assert!(h.query_by_label("DATE").is_some());
    assert!(h.query_by_label_contains("-05-14").is_none());
}

#[test]
fn every_column_fits_the_table_width() {
    let all = [
        Number, Title, Artist, Album, Date, Genre, Duration, Intro, FileName,
    ];
    let (h, fake) = harness(with_columns(state(1, 3), &all));
    let px = h.state().column_widths(fake.player(0)).unwrap();
    assert_eq!(px.len(), 9);
    let digits = 1;
    for (w, c) in px.iter().zip(all) {
        assert!(*w >= column_min(c, digits) - 0.5, "{c:?} {w}");
    }
}

// --- live resizing (O16)

fn press(h: &mut Harness<'_, AppUi>, at: Pos2, pressed: bool) {
    h.event(Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    });
}

/// The point on the edge between the columns whose headers are `left_of`
/// and `right_of` (the right one starts at the edge; its label sits 8
/// points inside).
fn edge(h: &Harness<'_, AppUi>, right_of: &str) -> Pos2 {
    let r = h.get_by_label(right_of).rect();
    pos2(r.left() - 8.0, r.center().y)
}

fn widths(h: &Harness<'_, AppUi>, fake: &Fake) -> Vec<f32> {
    h.state().column_widths(fake.player(0)).unwrap()
}

fn resize_commands(fake: &Fake) -> Vec<Command> {
    fake.take_sent()
        .into_iter()
        .filter(|c| matches!(c, Command::SetColumnWidths(..)))
        .collect()
}

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 1.0
}

#[test]
fn dragging_an_edge_recomputes_the_other_columns_every_frame() {
    let columns = [Number, Title, Artist, Album, Duration];
    let (mut h, fake) = harness(with_columns(state(1, 3), &columns));
    let before = widths(&h, &fake);
    let from = edge(&h, "ARTIST");
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    press(&mut h, from, true);
    h.run_steps(2);
    fake.take_sent();
    let mut seen = vec![before.clone()];
    for step in 1..=6 {
        h.event(Event::PointerMoved(pos2(
            from.x + step as f32 * 15.0,
            from.y,
        )));
        h.run_steps(1);
        seen.push(widths(&h, &fake));
    }
    // Title grew with the pointer, and on every frame.
    for pair in seen.windows(2) {
        assert!(pair[1][1] > pair[0][1] + 5.0, "{pair:?}");
    }
    let last = seen.last().unwrap();
    assert!(near(last[1], before[1] + 90.0), "{before:?} -> {last:?}");
    // The columns left of the edge did not move; those on its right shrank
    // together and still fill the table.
    assert!(near(last[0], before[0]));
    assert!(last[2] < before[2] && last[3] < before[3] && last[4] <= before[4]);
    assert!(near(last.iter().sum::<f32>(), before.iter().sum::<f32>()));
    assert!(
        resize_commands(&fake).is_empty(),
        "nothing is stored while the button is down"
    );
}

#[test]
fn the_widths_are_stored_once_on_release() {
    let columns = [Number, Title, Artist, Album, Duration];
    let (mut h, fake) = harness(with_columns(state(1, 3), &columns));
    let from = edge(&h, "ARTIST");
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    press(&mut h, from, true);
    h.run_steps(2);
    for step in 1..=6 {
        h.event(Event::PointerMoved(pos2(
            from.x + step as f32 * 15.0,
            from.y,
        )));
        h.run_steps(1);
    }
    let live = widths(&h, &fake);
    press(&mut h, pos2(from.x + 90.0, from.y), false);
    h.run_steps(3);
    let sent = resize_commands(&fake);
    assert_eq!(sent.len(), 1, "{sent:?}");
    let Command::SetColumnWidths(p, stored) = &sent[0] else {
        panic!()
    };
    assert_eq!(*p, fake.player(0));
    let total: f32 = live.iter().sum();
    for (c, w) in columns.iter().zip(&live) {
        let f = stored.fraction(*c).expect("every shown column is stored");
        assert!((f - w / total).abs() < 0.01, "{c:?}: {f} vs {}", w / total);
    }
    // The model has them now, and the table keeps drawing them.
    h.run_steps(30);
    let after = widths(&h, &fake);
    for (a, b) in after.iter().zip(&live) {
        assert!(near(*a, *b), "{after:?} vs {live:?}");
    }
}

#[test]
fn a_plain_click_on_an_edge_stores_nothing() {
    let (mut h, fake) = harness(state(1, 3));
    let from = edge(&h, "ARTIST");
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    press(&mut h, from, true);
    h.run_steps(1);
    press(&mut h, from, false);
    h.run_steps(3);
    assert!(resize_commands(&fake).is_empty());
}

#[test]
fn a_drag_cannot_squeeze_a_column_below_its_minimum() {
    let columns = [Title, Artist, Duration];
    let (mut h, fake) = harness(with_columns(state(1, 3), &columns));
    let from = edge(&h, "ARTIST");
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    press(&mut h, from, true);
    h.run_steps(2);
    for step in 1..=20 {
        h.event(Event::PointerMoved(pos2(
            from.x + step as f32 * 50.0,
            from.y,
        )));
        h.run_steps(1);
    }
    let w = widths(&h, &fake);
    assert!(
        w[1] >= column_min(Artist, 1) - 0.5 && w[2] >= column_min(Duration, 1) - 0.5,
        "{w:?}"
    );
    press(&mut h, pos2(from.x + 1000.0, from.y), false);
    h.run_steps(2);
}

#[test]
fn the_window_growing_does_not_store_widths_and_keeps_the_shares() {
    let columns = [Title, Artist, Album, Duration];
    let (mut h, fake) = harness(with_columns(state(1, 3), &columns));
    let before = widths(&h, &fake);
    fake.take_sent();
    h.set_size(egui::vec2(1400.0, 700.0));
    h.run_steps(3);
    let after = widths(&h, &fake);
    assert!(after.iter().sum::<f32>() > before.iter().sum::<f32>() + 300.0);
    assert!(resize_commands(&fake).is_empty());
    assert!(
        near(after[1] / after[2], before[1] / before[2])
            || (after[1] / after[2] - before[1] / before[2]).abs() < 0.05
    );
}

#[test]
fn a_column_shown_later_takes_its_default_width_and_keeps_the_others_shares() {
    let (mut h, fake) = harness(state(1, 3));
    let p = fake.player(0);
    let mut s = (*fake.state.load_full()).clone();
    s.config.ui.table_columns = vec![Number, Title, Artist, Date, Duration];
    fake.state.store(std::sync::Arc::new(s));
    h.run_steps(3);
    let after = h.state().column_widths(p).unwrap();
    assert_eq!(after.len(), 5);
    assert!(near(after[3], 84.0), "{after:?}");
    assert!(near(
        after.iter().sum::<f32>(),
        widths(&h, &fake).iter().sum::<f32>()
    ));
}

#[test]
fn nine_columns_in_the_narrowest_player_never_overflow_or_go_negative() {
    let all = [
        Number, Title, Artist, Album, Date, Genre, Duration, Intro, FileName,
    ];
    let (h, fake) = harness_sized(
        with_columns(state(1, 3), &all),
        egui::vec2(380.0, 700.0),
        |ui| ui,
    );
    let px = h.state().column_widths(fake.player(0)).unwrap();
    assert_eq!(px.len(), 9);
    assert!(px.iter().all(|w| w.is_finite() && *w >= 0.0), "{px:?}");
    assert!(px.iter().sum::<f32>() <= 380.0, "{px:?}");
}

#[test]
fn changing_the_columns_during_a_drag_ends_it_without_storing_anything() {
    let columns = [Number, Title, Artist, Album, Duration];
    let (mut h, fake) = harness(with_columns(state(1, 3), &columns));
    let from = edge(&h, "ARTIST");
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    press(&mut h, from, true);
    h.run_steps(2);
    h.event(Event::PointerMoved(pos2(from.x + 40.0, from.y)));
    h.run_steps(1);
    fake.take_sent();
    let mut s = (*fake.state.load_full()).clone();
    s.config.ui.table_columns = vec![Title, Artist, Duration];
    fake.state.store(std::sync::Arc::new(s));
    h.event(Event::PointerMoved(pos2(from.x + 80.0, from.y)));
    h.run_steps(2);
    press(&mut h, pos2(from.x + 80.0, from.y), false);
    h.run_steps(3);
    assert!(resize_commands(&fake).is_empty());
    assert_eq!(widths(&h, &fake).len(), 3);
}

#[test]
fn a_track_dragged_over_the_header_reorders_no_column() {
    let (mut h, fake) = harness(state(1, 3));
    fake.take_sent();
    let row = centre(&h, "Song 2");
    let header = left_of(&h, "#");
    drag(&mut h, row, header);
    assert!(
        !fake
            .take_sent()
            .iter()
            .any(|c| matches!(c, Command::UpdateConfig(_)))
    );
    assert_eq!(
        header_order(&h, &["#", "TITLE", "ARTIST", "DUR."]),
        ["#", "TITLE", "ARTIST", "DUR."]
    );
}

fn centre(h: &Harness<'_, AppUi>, label: &str) -> Pos2 {
    h.get_by_label(label).rect().center()
}

fn left_of(h: &Harness<'_, AppUi>, label: &str) -> Pos2 {
    let r = h.get_by_label(label).rect();
    pos2(r.left() - 4.0, r.center().y)
}

fn header_order(h: &Harness<'_, AppUi>, labels: &[&str]) -> Vec<String> {
    let mut found: Vec<(f32, String)> = labels
        .iter()
        .filter_map(|l| {
            h.query_by_label(l)
                .map(|n| (n.rect().left(), (*l).to_owned()))
        })
        .collect();
    found.sort_by(|a, b| a.0.total_cmp(&b.0));
    found.into_iter().map(|(_, l)| l).collect()
}

/// Presses at `from`, moves to `to` in steps and releases there.
fn drag(h: &mut Harness<'_, AppUi>, from: Pos2, to: Pos2) {
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    press(h, from, true);
    h.run_steps(1);
    for step in 1..=10 {
        let t = step as f32 / 10.0;
        h.event(Event::PointerMoved(pos2(
            from.x + (to.x - from.x) * t,
            from.y + (to.y - from.y) * t,
        )));
        h.run_steps(1);
    }
    press(h, to, false);
    h.run_steps(3);
}
