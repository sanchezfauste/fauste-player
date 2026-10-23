#![allow(clippy::unwrap_used, clippy::indexing_slicing, clippy::float_cmp)]
//! The big Play / Next button (itr07 J4): filled green with a white icon, its
//! states defined by theme tokens, the icon at WCAG >= 3:1 on every fill.

mod support;

use egui::{Color32, Rect};
use egui_kittest::kittest::{NodeT, Queryable};
use fp_app::ui::theme;
use fp_model::Command;

type H = egui_kittest::Harness<'static, fp_app::ui::app::AppUi>;

fn luminance(c: Color32) -> f32 {
    let lin = |v: u8| {
        let v = f32::from(v) / 255.0;
        if v <= 0.03928 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * lin(c.r()) + 0.7152 * lin(c.g()) + 0.0722 * lin(c.b())
}

fn contrast(a: Color32, b: Color32) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

fn rect_fills(h: &H) -> Vec<egui::epaint::RectShape> {
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

/// The fill painted exactly over `button` (the tile's own background).
fn fill_of(h: &H, button: Rect) -> Color32 {
    rect_fills(h)
        .iter()
        .rev()
        .find(|r| r.rect == button && r.fill != Color32::TRANSPARENT)
        .map(|r| r.fill)
        .unwrap_or(Color32::TRANSPARENT)
}

#[test]
fn the_icon_is_white_with_enough_contrast_on_every_fill() {
    assert_eq!(theme::PLAY_ICON, Color32::WHITE);
    for (name, fill) in [
        ("idle", theme::PLAY_FILL),
        ("on air", theme::PLAY_FILL_ON_AIR),
        ("hover", theme::PLAY_FILL_HOVER),
        ("pressed", theme::PLAY_FILL_PRESSED),
    ] {
        let ratio = contrast(theme::PLAY_ICON, fill);
        assert!(ratio >= 3.0, "{name}: {ratio}");
    }
    // The countdown band on the on-air fill keeps the icon legible too.
    let band = theme::play_sweep();
    let a = f32::from(band.a()) / 255.0;
    // Premultiplied colour: `band.r()` already carries the alpha.
    let over = Color32::from_rgb(
        (f32::from(band.r()) + f32::from(theme::PLAY_FILL_ON_AIR.r()) * (1.0 - a)).round() as u8,
        (f32::from(band.g()) + f32::from(theme::PLAY_FILL_ON_AIR.g()) * (1.0 - a)).round() as u8,
        (f32::from(band.b()) + f32::from(theme::PLAY_FILL_ON_AIR.b()) * (1.0 - a)).round() as u8,
    );
    assert!(contrast(theme::PLAY_ICON, over) >= 3.0, "{over:?}");
    assert_ne!(over, theme::PLAY_FILL_ON_AIR, "the band is visible");
}

#[test]
fn the_states_have_distinct_fills() {
    let fills = [
        theme::PLAY_FILL,
        theme::PLAY_FILL_ON_AIR,
        theme::PLAY_FILL_HOVER,
        theme::PLAY_FILL_PRESSED,
        theme::PLAY_FILL_DISABLED,
    ];
    for (i, a) in fills.iter().enumerate() {
        for b in &fills[i + 1..] {
            assert_ne!(a, b);
        }
    }
    assert!(luminance(theme::PLAY_FILL_HOVER) > luminance(theme::PLAY_FILL));
    assert!(luminance(theme::PLAY_FILL_PRESSED) < luminance(theme::PLAY_FILL_ON_AIR));
}

#[test]
fn the_button_is_filled_idle_and_deeper_on_air() {
    use fp_app::ui::controller::Controller;
    let (mut h, fake) = support::harness(support::state(1, 3));
    h.run_steps(3);
    let play = h.get_by_label("Play").rect();
    assert_eq!(fill_of(&h, play), theme::PLAY_FILL, "idle");
    fake.send(Command::Play(fake.player(0)));
    h.run_steps(3);
    let next = h.get_by_label("Fade into the next track").rect();
    assert_eq!(fill_of(&h, next), theme::PLAY_FILL_ON_AIR, "on air");
}

#[test]
fn hover_lightens_and_disabled_is_muted() {
    let (mut h, _fake) = support::harness(support::state(1, 3));
    h.run_steps(3);
    let play = h.get_by_label("Play").rect();
    h.hover_at(play.center());
    h.run_steps(2);
    assert_eq!(fill_of(&h, play), theme::PLAY_FILL_HOVER, "hover");
    let (mut h, _fake) = support::harness(support::state(1, 0));
    h.run_steps(3);
    let play = h.get_by_label("Play").rect();
    assert!(h.get_by_label("Play").accesskit_node().is_disabled());
    assert_eq!(fill_of(&h, play), theme::PLAY_FILL_DISABLED, "disabled");
}

#[test]
fn keyboard_focus_draws_a_ring_on_the_button() {
    let (mut h, _fake) = support::harness(support::state(1, 3));
    h.run_steps(3);
    let play = h.get_by_label("Play");
    let rect = play.rect();
    let ring = |h: &H| {
        rect_fills(h)
            .iter()
            .any(|r| r.rect == rect.expand(1.0) && r.stroke.color == theme::FOCUS_RING)
    };
    assert!(!ring(&h));
    play.focus();
    h.run_steps(2);
    assert!(ring(&h), "ring when focused");
}

#[test]
fn a_divider_separates_the_meter_from_the_fader() {
    let (mut h, _fake) = support::harness(support::state(2, 3));
    h.run_steps(3);
    let dividers: Vec<Rect> = rect_fills(&h)
        .iter()
        .filter(|r| r.fill == theme::DIVIDER && r.rect.width() == 1.0 && r.rect.height() > 60.0)
        .map(|r| r.rect)
        .collect();
    assert_eq!(dividers.len(), 2, "one per player: {dividers:?}");
}
