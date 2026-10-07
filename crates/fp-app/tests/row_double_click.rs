#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! A double-click on a playlist row sets it as next, also when egui counts
//! it as a triple click (operator feedback 4, Q6 side finding): a
//! double-click less than 0.6 s after a click elsewhere. A real triple
//! click sets it once. Real pointer events, as an operator's mouse.

mod support;

use egui::{Event, Modifiers, PointerButton, Pos2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_model::Command;
use support::{harness, state};

/// Moves the pointer to `at` and presses and releases the primary button.
fn click(h: &mut Harness<'_, AppUi>, at: Pos2) {
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    for pressed in [true, false] {
        h.event(Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        });
    }
    h.step();
}

fn row(h: &Harness<'_, AppUi>, title: &str) -> Pos2 {
    h.get_by_label(title).rect().center()
}

fn set_next_count(sent: &[Command]) -> usize {
    sent.iter()
        .filter(|c| matches!(c, Command::SetNext(..)))
        .count()
}

#[test]
fn a_double_click_soon_after_a_click_on_another_row_sets_the_next() {
    let (mut h, fake) = harness(state(1, 4));
    let (p, e) = (fake.player(0), fake.entries());
    let (song2, song3) = (row(&h, "Song 2"), row(&h, "Song 3"));
    fake.take_sent();
    click(&mut h, song2);
    click(&mut h, song3);
    click(&mut h, song3);
    h.run_steps(2);
    let sent = fake.take_sent();
    assert!(
        sent.contains(&Command::SetNext(p, e[2])),
        "the double-click on Song 3 did not set it as next: {sent:?}"
    );
    assert_eq!(set_next_count(&sent), 1, "{sent:?}");
}

#[test]
fn a_triple_click_on_a_row_sets_the_next_once() {
    let (mut h, fake) = harness(state(1, 4));
    let (p, e) = (fake.player(0), fake.entries());
    let song3 = row(&h, "Song 3");
    fake.take_sent();
    for _ in 0..3 {
        click(&mut h, song3);
    }
    h.run_steps(2);
    let sent = fake.take_sent();
    assert_eq!(
        sent.iter()
            .filter(|c| matches!(c, Command::SetNext(..)))
            .collect::<Vec<_>>(),
        [&Command::SetNext(p, e[2])],
        "{sent:?}"
    );
}

#[test]
fn a_plain_double_click_still_sets_the_next_once() {
    let (mut h, fake) = harness(state(1, 4));
    let (p, e) = (fake.player(0), fake.entries());
    let song3 = row(&h, "Song 3");
    fake.take_sent();
    click(&mut h, song3);
    click(&mut h, song3);
    h.run_steps(2);
    let sent = fake.take_sent();
    assert!(sent.contains(&Command::SetNext(p, e[2])), "{sent:?}");
    assert_eq!(set_next_count(&sent), 1, "{sent:?}");
}
