#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O24: reordering, showing and hiding the table columns
//! from the table header and from Settings.

mod support;

use egui::accesskit::Role;
use egui::{Event, Modifiers, PointerButton, Pos2, pos2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use fp_app::ui::app::AppUi;
use fp_model::TableColumn::{Album, Artist, Date, Duration, Genre, Number, Title};
use fp_model::{Command, TableColumn};
use support::{Fake, harness, state};

fn press(h: &mut Harness<'_, AppUi>, at: Pos2, pressed: bool) {
    h.event(Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    });
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

fn centre(h: &Harness<'_, AppUi>, label: &str) -> Pos2 {
    h.get_by_label(label).rect().center()
}

/// A point in the left part of the header cell labelled `label`.
fn left_of(h: &Harness<'_, AppUi>, label: &str) -> Pos2 {
    let r = h.get_by_label(label).rect();
    pos2(r.left() - 4.0, r.center().y)
}

/// A point in the right part of the cell whose header is `label` (the
/// right-aligned time columns have their text there; any other cell is
/// reached from the label's right).
fn right_of(h: &Harness<'_, AppUi>, label: &str, cell_width: f32) -> Pos2 {
    let r = h.get_by_label(label).rect();
    pos2(r.left() - 8.0 + cell_width - 4.0, r.center().y)
}

fn column_updates(fake: &Fake) -> Vec<Vec<TableColumn>> {
    fake.take_sent()
        .into_iter()
        .filter_map(|c| match c {
            Command::UpdateConfig(config) => Some(config.ui.table_columns),
            _ => None,
        })
        .collect()
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

const HEADERS: [&str; 4] = ["#", "TITLE", "ARTIST", "DUR."];

// --- drag a header

#[test]
fn dragging_a_header_onto_another_moves_the_column_there() {
    let (mut h, fake) = harness(state(1, 3));
    fake.take_sent();
    let from = centre(&h, "ARTIST");
    let to = left_of(&h, "#");
    drag(&mut h, from, to);
    assert_eq!(
        column_updates(&fake),
        vec![vec![Artist, Number, Title, Duration]]
    );
    assert_eq!(header_order(&h, &HEADERS), ["ARTIST", "#", "TITLE", "DUR."]);
}

#[test]
fn a_header_dropped_on_the_right_half_goes_after_that_column() {
    let (mut h, fake) = harness(state(1, 3));
    fake.take_sent();
    let from = centre(&h, "#");
    let width = h.state().column_widths(fake.player(0)).unwrap()[1];
    let to = right_of(&h, "TITLE", width);
    drag(&mut h, from, to);
    assert_eq!(
        column_updates(&fake),
        vec![vec![Title, Number, Artist, Duration]]
    );
}

#[test]
fn a_required_column_can_be_moved_too() {
    let (mut h, fake) = harness(state(1, 3));
    fake.take_sent();
    let from = centre(&h, "TITLE");
    let to = left_of(&h, "#");
    drag(&mut h, from, to);
    assert_eq!(
        column_updates(&fake),
        vec![vec![Title, Number, Artist, Duration]]
    );
}

#[test]
fn dropping_a_header_on_itself_or_outside_the_header_changes_nothing() {
    let (mut h, fake) = harness(state(1, 3));
    fake.take_sent();
    let from = centre(&h, "ARTIST");
    drag(&mut h, from, pos2(from.x + 2.0, from.y));
    let row = centre(&h, "Song 2");
    let artist = centre(&h, "ARTIST");
    drag(&mut h, artist, row);
    let sent = fake.take_sent();
    assert!(
        !sent.iter().any(|c| matches!(c, Command::UpdateConfig(_))),
        "{sent:?}"
    );
    assert!(
        !sent.iter().any(|c| matches!(c, Command::MoveEntry { .. })),
        "a column dropped on the rows moves no entry"
    );
    assert_eq!(header_order(&h, &HEADERS), HEADERS);
}

#[test]
fn dragging_from_a_column_edge_resizes_and_never_reorders() {
    let (mut h, fake) = harness(state(1, 3));
    fake.take_sent();
    let r = h.get_by_label("ARTIST").rect();
    let edge = pos2(r.left() - 8.0, r.center().y);
    drag(&mut h, edge, pos2(edge.x + 60.0, edge.y));
    assert!(column_updates(&fake).is_empty(), "no column move");
    assert_eq!(header_order(&h, &HEADERS), HEADERS);
}

// --- the header menu

fn open_menu(h: &mut Harness<'_, AppUi>) {
    h.get_by_label("TITLE").click_secondary();
    h.run_steps(2);
}

#[test]
fn the_header_menu_offers_the_optional_columns_only() {
    let (mut h, _) = harness(state(1, 3));
    open_menu(&mut h);
    for name in [
        "Number (#)",
        "Artist",
        "Album",
        "Date",
        "Genre",
        "Intro",
        "File name",
    ] {
        assert!(
            h.query_by_role_and_label(Role::CheckBox, name).is_some(),
            "{name}"
        );
    }
    assert!(h.query_by_role_and_label(Role::CheckBox, "Title").is_none());
    assert!(
        h.query_by_role_and_label(Role::CheckBox, "Duration")
            .is_none()
    );
}

#[test]
fn the_header_menu_shows_a_hidden_column() {
    let (mut h, fake) = harness(state(1, 3));
    fake.take_sent();
    open_menu(&mut h);
    h.get_by_role_and_label(Role::CheckBox, "Album").click();
    h.run_steps(3);
    assert_eq!(
        column_updates(&fake),
        vec![vec![Number, Title, Artist, Duration, Album]]
    );
    assert!(h.query_by_label("ALBUM").is_some());
}

#[test]
fn the_header_menu_hides_a_shown_column() {
    let (mut h, fake) = harness(state(1, 3));
    fake.take_sent();
    open_menu(&mut h);
    h.get_by_role_and_label(Role::CheckBox, "Artist").click();
    h.run_steps(3);
    assert_eq!(column_updates(&fake), vec![vec![Number, Title, Duration]]);
    assert!(h.query_by_label("ARTIST").is_none());
}

#[test]
fn the_header_menu_marks_the_shown_columns() {
    let (mut h, _) = harness(state(1, 3));
    open_menu(&mut h);
    assert_eq!(
        h.get_by_role_and_label(Role::CheckBox, "Artist")
            .accesskit_node()
            .toggled(),
        Some(egui::accesskit::Toggled::True)
    );
    assert_eq!(
        h.get_by_role_and_label(Role::CheckBox, "Album")
            .accesskit_node()
            .toggled(),
        Some(egui::accesskit::Toggled::False)
    );
}

#[test]
fn a_hidden_column_comes_back_at_the_end_and_a_shown_one_leaves() {
    let mut s = state(1, 3);
    s.config.ui.table_columns = vec![Duration, Title, Date];
    let (mut h, fake) = harness(s);
    fake.take_sent();
    open_menu(&mut h);
    h.get_by_role_and_label(Role::CheckBox, "Genre").click();
    h.run_steps(3);
    assert_eq!(
        column_updates(&fake),
        vec![vec![Duration, Title, Date, Genre]]
    );
}

// --- Settings

fn open_settings(h: &mut Harness<'_, AppUi>) {
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Playlists").click();
    h.run_steps(3);
}

#[test]
fn settings_lists_every_column_and_marks_the_shown_ones() {
    let (mut h, _) = harness(state(1, 3));
    open_settings(&mut h);
    for name in [
        "Number (#)",
        "Title",
        "Artist",
        "Album",
        "Date",
        "Genre",
        "Duration",
        "Intro",
        "File name",
    ] {
        assert!(
            h.query_by_role_and_label(Role::CheckBox, name).is_some(),
            "{name}"
        );
    }
    assert_eq!(
        h.get_by_role_and_label(Role::CheckBox, "Artist")
            .accesskit_node()
            .toggled(),
        Some(egui::accesskit::Toggled::True)
    );
    assert_eq!(
        h.get_by_role_and_label(Role::CheckBox, "Album")
            .accesskit_node()
            .toggled(),
        Some(egui::accesskit::Toggled::False)
    );
}

#[test]
fn the_required_columns_cannot_be_unticked() {
    let (mut h, fake) = harness(state(1, 3));
    open_settings(&mut h);
    assert!(
        h.get_by_role_and_label(Role::CheckBox, "Title")
            .accesskit_node()
            .is_disabled()
    );
    assert!(
        h.get_by_role_and_label(Role::CheckBox, "Duration")
            .accesskit_node()
            .is_disabled()
    );
    assert!(
        !h.get_by_role_and_label(Role::CheckBox, "Artist")
            .accesskit_node()
            .is_disabled()
    );
    fake.take_sent();
    h.get_by_role_and_label(Role::CheckBox, "Title").click();
    h.run_steps(2);
    assert!(column_updates(&fake).is_empty());
}

#[test]
fn ticking_a_column_in_settings_shows_it() {
    let (mut h, fake) = harness(state(1, 3));
    open_settings(&mut h);
    fake.take_sent();
    h.get_by_role_and_label(Role::CheckBox, "Genre").click();
    h.run_steps(3);
    assert_eq!(
        column_updates(&fake),
        vec![vec![Number, Title, Artist, Duration, Genre]]
    );
}

#[test]
fn unticking_an_optional_column_hides_it() {
    let (mut h, fake) = harness(state(1, 3));
    open_settings(&mut h);
    fake.take_sent();
    h.get_by_role_and_label(Role::CheckBox, "Number (#)")
        .click();
    h.run_steps(3);
    assert_eq!(column_updates(&fake), vec![vec![Title, Artist, Duration]]);
}

#[test]
fn the_arrows_move_a_column_up_and_down() {
    let (mut h, fake) = harness(state(1, 3));
    open_settings(&mut h);
    fake.take_sent();
    h.get_by_label("Move Artist up").click();
    h.run_steps(3);
    assert_eq!(
        column_updates(&fake),
        vec![vec![Number, Artist, Title, Duration]]
    );
    h.get_by_label("Move Number (#) down").click();
    h.run_steps(3);
    assert_eq!(
        column_updates(&fake),
        vec![vec![Artist, Number, Title, Duration]]
    );
}

#[test]
fn the_arrows_are_off_at_the_ends_and_for_hidden_columns() {
    let (mut h, _) = harness(state(1, 3));
    open_settings(&mut h);
    let off =
        |h: &Harness<'_, AppUi>, label: &str| h.get_by_label(label).accesskit_node().is_disabled();
    assert!(off(&h, "Move Number (#) up"));
    assert!(off(&h, "Move Duration down"));
    assert!(!off(&h, "Move Duration up"));
    assert!(!off(&h, "Move Number (#) down"));
    assert!(off(&h, "Move Album up") && off(&h, "Move Album down"));
}

#[test]
fn a_required_column_can_move_in_settings_too() {
    let (mut h, fake) = harness(state(1, 3));
    open_settings(&mut h);
    fake.take_sent();
    h.get_by_label("Move Duration up").click();
    h.run_steps(3);
    assert_eq!(
        column_updates(&fake),
        vec![vec![Number, Title, Duration, Artist]]
    );
}

#[test]
fn default_columns_restores_the_list_and_is_off_when_it_is_the_default() {
    let mut s = state(1, 3);
    s.config.ui.table_columns = vec![Duration, Title, Album];
    let (mut h, fake) = harness(s);
    open_settings(&mut h);
    fake.take_sent();
    h.get_by_label("Default columns").click();
    h.run_steps(3);
    assert_eq!(
        column_updates(&fake),
        vec![vec![Number, Title, Artist, Duration]]
    );
    assert!(
        h.get_by_label("Default columns")
            .accesskit_node()
            .is_disabled()
    );
}

#[test]
fn the_table_follows_a_change_made_in_settings() {
    let (mut h, _) = harness(state(1, 3));
    open_settings(&mut h);
    h.get_by_role_and_label(Role::CheckBox, "Date").click();
    h.run_steps(3);
    h.get_by_label("Close").click();
    h.run_steps(3);
    assert!(h.query_by_label("DATE").is_some());
}
