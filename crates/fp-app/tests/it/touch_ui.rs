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
