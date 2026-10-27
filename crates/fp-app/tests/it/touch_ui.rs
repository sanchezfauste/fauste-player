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

fn drag(h: &mut Harness<'static, AppUi>, from: egui::Pos2, to: egui::Pos2) {
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
    h.get_by_label("Open P3").click();
    h.run_steps(3);
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
