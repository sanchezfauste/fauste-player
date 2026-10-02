#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O9: the per-entry repeat and stop icons are drawn before
//! the title.

mod support;

use egui::Rect;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_model::{AppState, Command, EntryId};
use support::{harness, harness_sized, state};

fn entries(s: &AppState) -> Vec<EntryId> {
    s.playlists
        .iter()
        .next()
        .unwrap()
        .entries
        .iter()
        .map(|x| x.id)
        .collect()
}

/// The title label of a table row (the one in the row of `icon`, when
/// given; the table's rows are below the header).
fn title_in_row(h: &Harness<'_, AppUi>, title: &str, row_y: f32) -> Rect {
    h.query_all_by_label(title)
        .map(|n| n.rect())
        .find(|r| (r.center().y - row_y).abs() < 4.0)
        .unwrap_or_else(|| panic!("no {title} label in the row at {row_y}"))
}

fn flagged(repeat: &[usize], stop: &[usize]) -> AppState {
    let mut s = state(1, 4);
    let e = entries(&s);
    for i in repeat {
        fp_model::apply(&mut s, Command::ToggleEntryRepeat(e[*i])).unwrap();
    }
    for i in stop {
        fp_model::apply(&mut s, Command::ToggleEntryStopAfter(e[*i])).unwrap();
    }
    s
}

#[test]
fn the_repeat_icon_comes_before_the_title_in_its_row() {
    let (h, _) = harness(flagged(&[1], &[]));
    let icon = h.get_by_label("Repeats").rect();
    let title = title_in_row(&h, "Song 2", icon.center().y);
    assert!(icon.right() <= title.left() + 0.5, "{icon:?} {title:?}");
}

#[test]
fn the_stop_icon_comes_before_the_title_in_its_row() {
    let (h, _) = harness(flagged(&[], &[2]));
    let icon = h.get_by_label("Stops after").rect();
    let title = title_in_row(&h, "Song 3", icon.center().y);
    assert!(icon.right() <= title.left() + 0.5, "{icon:?} {title:?}");
}

#[test]
fn both_icons_keep_their_order_before_the_title() {
    let (h, _) = harness(flagged(&[1], &[1]));
    let repeat = h.get_by_label("Repeats").rect();
    let stop = h.get_by_label("Stops after").rect();
    let title = title_in_row(&h, "Song 2", repeat.center().y);
    assert!(repeat.right() <= stop.left() + 0.5, "{repeat:?} {stop:?}");
    assert!(stop.right() <= title.left() + 0.5, "{stop:?} {title:?}");
}

#[test]
fn a_flagged_title_makes_room_and_an_unflagged_one_does_not() {
    let (h, _) = harness(flagged(&[1], &[]));
    let icon = h.get_by_label("Repeats").rect();
    let flagged_title = title_in_row(&h, "Song 2", icon.center().y);
    let plain: Vec<Rect> = ["Song 3", "Song 4"]
        .iter()
        .map(|t| {
            h.query_all_by_label(t)
                .map(|n| n.rect())
                .max_by(|a, b| a.top().total_cmp(&b.top()))
                .unwrap()
        })
        .collect();
    assert!((plain[0].left() - plain[1].left()).abs() < 0.5);
    assert!(
        flagged_title.left() > plain[0].left() + 10.0,
        "{flagged_title:?} {plain:?}"
    );
}

#[test]
fn the_icons_stay_in_a_narrow_table_and_the_title_gives_way() {
    let (h, _) = harness_sized(flagged(&[0], &[0]), egui::vec2(380.0, 700.0), |ui| ui);
    let repeat = h.get_by_label("Repeats").rect();
    let stop = h.get_by_label("Stops after").rect();
    assert!(
        repeat.left() >= 0.0 && stop.right() <= 380.0,
        "{repeat:?} {stop:?}"
    );
    assert!(repeat.right() <= stop.left() + 0.5);
}

#[test]
fn the_outdated_flag_stays_after_the_title() {
    let mut s = flagged(&[1], &[]);
    let track = s.playlists.entry(entries(&s)[1]).unwrap().track;
    let t = s.library.get_mut(track).unwrap();
    t.analyzed = true;
    t.analysis_version = 0;
    let (h, _) = harness(s);
    let outdated = h
        .get_by_label_contains("Analysed by an earlier version")
        .rect();
    let repeat = h.get_by_label("Repeats").rect();
    let title = title_in_row(&h, "Song 2", repeat.center().y);
    assert!(
        outdated.left() >= title.right() - 0.5,
        "{outdated:?} {title:?}"
    );
}
