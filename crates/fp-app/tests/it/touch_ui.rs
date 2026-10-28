#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The touch layout on a phone and a tablet (mobile spec §6).

use crate::support;

use egui::{Vec2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use std::sync::Arc;
use support::{Fake, harness_sized, state};

const PORTRAIT: Vec2 = vec2(412.0, 892.0);
const LANDSCAPE: Vec2 = vec2(892.0, 412.0);
const TABLET: Vec2 = vec2(800.0, 1280.0);

fn touch(players: usize, size: Vec2) -> (Harness<'static, AppUi>, Arc<Fake>) {
    let (mut h, fake) = harness_sized(state(players, 3), size, |ui| ui.with_touch());
    h.run_steps(3);
    (h, fake)
}

#[test]
fn the_phone_top_bar_has_the_screen_tiles() {
    let (h, _fake) = touch(2, PORTRAIT);
    assert!(h.query_by_label("Players").is_some());
    assert!(h.query_by_label("Cartwall").is_some());
    assert!(h.query_by_label("Hide P1").is_none());
    assert!(h.query_by_label("Settings").is_some());
}

#[test]
fn a_tablet_keeps_the_panel_tiles() {
    let (h, _fake) = touch(2, TABLET);
    assert!(h.query_by_label("Hide P1").is_some());
    assert!(h.query_by_label("Players").is_none());
}

#[test]
fn without_touch_nothing_changes() {
    let (mut h, _fake) = harness_sized(state(2, 3), PORTRAIT, |ui| ui);
    h.run_steps(3);
    assert!(h.query_by_label("Hide P1").is_some());
    assert!(h.query_by_label("Players").is_none());
}

#[test]
fn landscape_is_a_phone_too() {
    let (h, _fake) = touch(2, LANDSCAPE);
    assert!(h.query_by_label("Players").is_some());
}

const ADD: &str = "Add tracks to this playlist";

#[test]
fn all_players_show_as_compact_panels() {
    let (h, _fake) = touch(2, PORTRAIT);
    assert!(h.query_by_label("Open P1").is_some());
    assert!(h.query_by_label("Open P2").is_some());
    assert_eq!(h.query_all_by_label("Play").count(), 2);
    assert!(
        h.query_by_label(ADD).is_none(),
        "no playlist in a compact panel"
    );
}

#[test]
fn open_shows_one_player_with_its_playlist() {
    let (mut h, _fake) = touch(2, PORTRAIT);
    h.get_by_label("Open P2").click();
    h.run_steps(3);
    assert_eq!(h.query_all_by_label(ADD).count(), 1);
    assert_eq!(h.query_all_by_label("Play").count(), 1);
    assert!(h.query_by_label("Open P1").is_none());
    assert!(h.query_by_label("All").is_some());
}

#[test]
fn chips_switch_the_open_player_and_all_goes_back() {
    let (mut h, _fake) = touch(2, PORTRAIT);
    h.get_by_label("Open P1").click();
    h.run_steps(3);
    h.get_by_label("P2").click();
    h.run_steps(3);
    assert_eq!(h.query_all_by_label(ADD).count(), 1);
    h.get_by_label("All").click();
    h.run_steps(3);
    assert!(h.query_by_label("Open P1").is_some());
    assert!(h.query_by_label(ADD).is_none());
}

#[test]
fn play_in_a_compact_panel_sends_play() {
    let (mut h, fake) = touch(1, PORTRAIT);
    fake.take_sent();
    h.get_by_label("Play").click();
    h.run_steps(2);
    assert!(
        fake.take_sent()
            .iter()
            .any(|c| matches!(c, fp_model::Command::Play(_))),
        "no play command"
    );
}

#[test]
fn landscape_shows_two_players_side_by_side() {
    let (h, _fake) = touch(3, LANDSCAPE);
    assert_eq!(h.query_all_by_label(ADD).count(), 2);
    assert!(h.query_by_label("P1–P2").is_some());
    assert!(h.query_by_label("P3").is_some());
}

#[test]
fn a_pair_chip_shows_that_pair() {
    let (mut h, _fake) = touch(3, LANDSCAPE);
    h.get_by_label("P3").click();
    h.run_steps(3);
    assert_eq!(h.query_all_by_label(ADD).count(), 1);
}

/// A finger drag. The touch event makes egui see a touch screen, as on a
/// phone: only then does a drag scroll a scroll area.
fn drag(h: &mut Harness<'static, AppUi>, from: egui::Pos2, to: egui::Pos2) {
    h.event(egui::Event::Touch {
        device_id: egui::TouchDeviceId(0),
        id: egui::TouchId(0),
        phase: egui::TouchPhase::Cancel,
        pos: from,
        force: None,
    });
    h.event(egui::Event::PointerMoved(from));
    h.event(egui::Event::PointerButton {
        pos: from,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::NONE,
    });
    h.step();
    for k in 1..=5 {
        h.event(egui::Event::PointerMoved(
            from + (to - from) * (k as f32 / 5.0),
        ));
        h.step();
    }
    h.event(egui::Event::PointerButton {
        pos: to,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
    h.run_steps(3);
}

#[test]
fn a_swipe_shows_the_next_pair_and_back() {
    let (mut h, _fake) = touch(3, LANDSCAPE);
    // On the chip row, right of the chips: no widget takes the drag.
    let row = h.get_by_label("P3").rect();
    let from = egui::pos2(row.right() + 400.0, row.center().y);
    drag(&mut h, from, from - vec2(200.0, 0.0));
    assert_eq!(h.query_all_by_label(ADD).count(), 1, "the pair with P3");
    drag(&mut h, from - vec2(200.0, 0.0), from);
    assert_eq!(h.query_all_by_label(ADD).count(), 2);
}

/// A phone in landscape is shorter than a player column: the columns
/// scroll, so the playlist and its footer can be reached.
#[test]
fn landscape_columns_scroll_down_to_the_playlist() {
    let (mut h, _fake) = touch(2, LANDSCAPE);
    let below = |h: &Harness<'static, AppUi>| {
        h.query_all_by_label(ADD)
            .all(|n| n.rect().bottom() > LANDSCAPE.y)
    };
    assert!(below(&h), "the footers start below the screen");
    let play = h.query_all_by_label("Play").next().unwrap().rect().center();
    drag(&mut h, play, play - vec2(0.0, 300.0));
    assert!(!below(&h), "a footer scrolled into view");
    assert_eq!(h.query_all_by_label(ADD).count(), 2, "still the same pair");
}

/// A sideways swipe over the scrolling columns still changes the pair.
#[test]
fn a_swipe_over_the_columns_shows_the_next_pair() {
    let (mut h, _fake) = touch(3, LANDSCAPE);
    let play = h.query_all_by_label("Play").next().unwrap().rect().center();
    drag(&mut h, play, play - vec2(200.0, 0.0));
    assert_eq!(h.query_all_by_label(ADD).count(), 1, "the pair with P3");
}

#[test]
fn an_open_player_in_landscape_scrolls_down_to_its_playlist() {
    let (mut h, _fake) = touch(2, LANDSCAPE);
    h.get_by_label("Open P1").click();
    h.run_steps(3);
    assert!(h.get_by_label(ADD).rect().bottom() > LANDSCAPE.y);
    let play = h.get_by_label("Play").rect().center();
    drag(&mut h, play, play - vec2(0.0, 300.0));
    assert!(h.get_by_label(ADD).rect().bottom() <= LANDSCAPE.y);
}

#[test]
fn landscape_opens_a_player_as_in_portrait() {
    let (mut h, _fake) = touch(3, LANDSCAPE);
    h.get_by_label("Open P2").click();
    h.run_steps(3);
    assert_eq!(h.query_all_by_label(ADD).count(), 1);
    assert!(h.query_by_label("All").is_some());
}

#[test]
fn the_cartwall_screen_shows_the_carts() {
    let (mut h, _fake) = touch(2, PORTRAIT);
    h.get_by_label("Cartwall").click();
    h.run_steps(3);
    assert!(h.query_by_label("Open P1").is_none());
    assert!(h.query_by_label_contains("Stop all").is_some());
    h.get_by_label("Players").click();
    h.run_steps(3);
    assert!(h.query_by_label("Open P1").is_some());
}

#[test]
fn rotation_keeps_the_open_player() {
    let (mut h, _fake) = touch(3, PORTRAIT);
    h.get_by_label("Open P2").click();
    h.run_steps(3);
    assert_eq!(h.query_all_by_label(ADD).count(), 1, "open in portrait");
    h.set_size(LANDSCAPE);
    h.run_steps(3);
    assert_eq!(h.query_all_by_label(ADD).count(), 1);
    assert!(h.query_by_label("All").is_some());
}

#[test]
fn a_removed_open_player_shows_all_players() {
    let (mut h, fake) = touch(2, PORTRAIT);
    h.get_by_label("Open P2").click();
    h.run_steps(3);
    let mut fewer = (**fake.state.load()).clone();
    fewer.players.truncate(1);
    fake.state.store(std::sync::Arc::new(fewer));
    h.run_steps(3);
    assert!(h.query_by_label("Open P1").is_some());
    assert!(h.query_by_label(ADD).is_none());
}

#[test]
fn a_phone_has_no_desktop_panel_buttons() {
    let (mut h, _fake) = touch(2, PORTRAIT);
    assert!(h.query_by_label("Maximize").is_none());
    h.get_by_label("Open P1").click();
    h.run_steps(3);
    assert!(h.query_by_label("Maximize").is_none());
    h.get_by_label("Cartwall").click();
    h.run_steps(3);
    assert!(h.query_by_label("Maximize").is_none());
    let (h, _fake) = touch(2, TABLET);
    assert!(h.query_all_by_label("Maximize").count() > 0);
}

#[test]
fn scrolling_over_a_compact_fader_leaves_the_volume() {
    let (mut h, fake) = touch(3, PORTRAIT);
    fake.take_sent();
    let fader = h.get_all_by_label_contains("Volume").next().unwrap().rect();
    drag(&mut h, fader.center(), fader.center() + vec2(0.0, 60.0));
    assert!(
        !fake
            .take_sent()
            .iter()
            .any(|c| matches!(c, fp_model::Command::SetVolume(..))),
        "a scroll moved the fader"
    );
}

#[test]
fn a_finger_drag_on_the_playlist_moves_no_row() {
    let (mut h, fake) = touch(1, PORTRAIT);
    h.get_by_label("Open P1").click();
    h.run_steps(3);
    fake.take_sent();
    let from = h.get_all_by_label("Song 3").last().unwrap().rect().center();
    let to = h.get_all_by_label("Song 1").last().unwrap().rect().center();
    drag(&mut h, from, to);
    assert!(
        !fake
            .take_sent()
            .iter()
            .any(|c| matches!(c, fp_model::Command::MoveEntry { .. })),
        "a scroll moved a row"
    );
}

#[test]
fn the_top_bar_fits_a_narrow_phone() {
    let (h, _fake) = touch(2, vec2(360.0, 780.0));
    let cartwall = h.get_by_label("Cartwall").rect();
    for label in ["Settings", "About"] {
        let other = h.get_by_label_contains(label).rect();
        assert!(
            !cartwall.intersects(other),
            "{label} overlaps the screen tiles"
        );
    }
}

#[test]
fn the_phone_cartwall_never_folds() {
    let mut closed = state(2, 3);
    closed.cartwall.open = false;
    let (mut h, _fake) = harness_sized(closed, PORTRAIT, |ui| ui.with_touch());
    h.run_steps(3);
    h.get_by_label("Cartwall").click();
    h.run_steps(3);
    assert!(
        h.query_all_by_label_contains("empty").count() > 0,
        "the carts show"
    );
    assert!(h.query_by_label("CARTWALL").is_none(), "no folding caret");
}

/// The status bar's shortcut hints name keys, double clicks and right
/// clicks: a touch screen leaves them out.
#[test]
fn touch_mode_shows_no_keyboard_and_mouse_hints() {
    for size in [PORTRAIT, TABLET] {
        let (h, _fake) = touch(2, size);
        assert!(h.query_by_label_contains("Keys 1–9").is_none(), "{size:?}");
    }
    let (mut h, _fake) = harness_sized(state(2, 3), TABLET, |ui| ui);
    h.run_steps(3);
    assert!(h.query_by_label_contains("Keys 1–9").is_some());
}

/// A page wider than tall (8 × 2 by default) is laid out the other way
/// round on a portrait screen, in the same reading order: wide buttons,
/// all inside the screen.
#[test]
fn a_portrait_cartwall_turns_a_wide_page() {
    let (mut h, _fake) = touch(2, PORTRAIT);
    h.get_by_label("Cartwall").click();
    h.run_steps(3);
    let carts: Vec<egui::Rect> = h
        .query_all_by_label_contains(", empty")
        .map(|n| n.rect())
        .collect();
    assert!(carts.len() > 2);
    for cart in &carts {
        assert!(cart.right() <= PORTRAIT.x, "{cart:?} leaves the screen");
        assert!(cart.width() > 120.0, "{cart:?} is narrow");
    }
    let one = h.get_by_label("Cart 1, empty").rect();
    let two = h.get_by_label("Cart 2, empty").rect();
    let three = h.get_by_label("Cart 3, empty").rect();
    assert!(two.left() > one.right(), "2 right of 1");
    assert!(three.top() > one.bottom(), "3 under 1");
}

/// Landscape keeps the page as it is.
#[test]
fn a_landscape_cartwall_keeps_the_page() {
    let (mut h, _fake) = touch(2, LANDSCAPE);
    h.get_by_label("Cartwall").click();
    h.run_steps(3);
    let one = h.get_by_label("Cart 1, empty").rect();
    let three = h.get_by_label("Cart 3, empty").rect();
    assert!((three.top() - one.top()).abs() < 1.0, "3 on the row of 1");
}

/// On a phone the Settings sections are chips above the section, which
/// takes the whole width (a side list would leave it no room).
#[test]
fn phone_settings_put_the_sections_above_the_content() {
    let (mut h, _fake) = touch(2, PORTRAIT);
    h.get_by_label("Settings").click();
    h.run_steps(3);
    let buffer = h.get_by_label("Buffer size").rect();
    assert!(buffer.left() < 80.0, "{buffer:?} sits right of a side list");
    let meters = h.get_by_label("Meters").rect();
    assert!(meters.bottom() < buffer.top(), "the chips are above");
    h.get_by_label("Meters").click();
    h.run_steps(3);
    assert!(h.query_by_label("Buffer size").is_none(), "another section");
}

/// The landscape columns scroll by finger, so their faders only show the
/// level: a scroll that starts on one never changes a level on air.
#[test]
fn scrolling_the_landscape_columns_over_a_fader_leaves_the_volume() {
    for open in [false, true] {
        let (mut h, fake) = touch(2, LANDSCAPE);
        if open {
            h.get_by_label("Open P1").click();
            h.run_steps(3);
        }
        fake.take_sent();
        let fader = h.get_all_by_label_contains("Volume").next().unwrap().rect();
        // Down: at 0 dB the fader is at its top, where up changes nothing.
        drag(&mut h, fader.center(), fader.center() + vec2(0.0, 60.0));
        assert!(
            !fake
                .take_sent()
                .iter()
                .any(|c| matches!(c, fp_model::Command::SetVolume(..))),
            "open {open}: a scroll moved the fader"
        );
    }
}

/// A touch tablet keeps the desktop's cartwall grid and Settings side list.
#[test]
fn a_touch_tablet_keeps_the_desktop_cartwall_and_settings() {
    let (mut h, _fake) = touch(2, TABLET);
    let one = h.get_by_label("Cart 1, empty").rect();
    let three = h.get_by_label("Cart 3, empty").rect();
    assert!((three.top() - one.top()).abs() < 1.0, "3 on the row of 1");
    h.get_by_label("Settings").click();
    h.run_steps(3);
    let buffer = h.get_by_label("Buffer size").rect();
    let meters = h.get_by_label("Meters").rect();
    assert!(
        meters.right() < buffer.left(),
        "the side list is left of the body"
    );
}
