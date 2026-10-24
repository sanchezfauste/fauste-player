#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The selected row of a playlist is outlined, whatever its colour: the
//! on-air and next rows paint their own fill, which hides the selection's.

mod support;

use egui::Rect;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_app::ui::theme;
use fp_model::Command;
use support::{harness, state};

type H = Harness<'static, AppUi>;

fn rects(h: &H) -> Vec<egui::epaint::RectShape> {
    fn walk(s: &egui::Shape, out: &mut Vec<egui::epaint::RectShape>) {
        match s {
            egui::Shape::Rect(r) => out.push(r.clone()),
            egui::Shape::Vec(v) => v.iter().for_each(|s| walk(s, out)),
            _ => {}
        }
    }
    let mut all = Vec::new();
    for c in &h.output().shapes {
        walk(&c.shape, &mut all);
    }
    all
}

/// The title cell of the row of "Song n" (the header may name it too).
fn title_of(h: &H, n: usize) -> Rect {
    h.get_all_by_label(&format!("Song {n}"))
        .last()
        .unwrap()
        .rect()
}

fn click_row(h: &mut H, n: usize) {
    h.get_all_by_label(&format!("Song {n}"))
        .last()
        .unwrap()
        .click();
    h.run_steps(2);
}

/// The outline pieces painted on the row whose title is at `title`.
fn outline_on(h: &H, title: Rect) -> Vec<Rect> {
    rects(h)
        .into_iter()
        .filter(|r| r.fill == theme::SELECTION_OUTLINE)
        .map(|r| r.rect)
        .filter(|r| r.top() <= title.center().y + 12.0 && r.bottom() >= title.center().y - 12.0)
        .collect()
}

/// Whether the outline pieces on a row close it: a top and a bottom edge
/// across the row, and a left and a right one.
fn closes_the_row(pieces: &[Rect]) -> bool {
    let horizontal = pieces.iter().filter(|r| r.width() > r.height()).count();
    let vertical = pieces.iter().filter(|r| r.height() > r.width()).count();
    horizontal >= 2 && vertical == 2
}

#[test]
fn the_selected_row_is_outlined_whether_on_air_next_or_plain() {
    use fp_app::ui::controller::Controller;
    let (mut h, fake) = harness(state(1, 3));
    fake.send(Command::Play(fake.player(0)));
    h.run_steps(3);
    // Song 1 is on air, Song 2 is next, Song 3 is a plain row.
    for n in 1..=3 {
        click_row(&mut h, n);
        let pieces = outline_on(&h, title_of(&h, n));
        assert!(closes_the_row(&pieces), "Song {n}: {pieces:?}");
        for other in (1..=3).filter(|o| *o != n) {
            assert!(
                outline_on(&h, title_of(&h, other)).is_empty(),
                "Song {other} is not selected"
            );
        }
    }
}

#[test]
fn no_row_is_outlined_without_a_selection() {
    let (mut h, _fake) = harness(state(1, 3));
    h.run_steps(3);
    assert!(rects(&h).iter().all(|r| r.fill != theme::SELECTION_OUTLINE));
}

#[test]
fn the_outline_stands_out_on_the_on_air_and_next_fills() {
    assert_eq!(theme::SELECTION_OUTLINE, theme::ACCENT_300);
    for fill in [theme::ON_AIR_ROW, theme::NEXT_ROW, theme::ACCENT_900] {
        assert_ne!(fill, theme::SELECTION_OUTLINE);
    }
}
