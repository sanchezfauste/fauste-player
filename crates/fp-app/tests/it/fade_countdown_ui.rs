#![allow(clippy::unwrap_used, clippy::indexing_slicing, clippy::float_cmp)]
//! The Fade stop button drains while a fade runs (rule 30 in the UI: the
//! remaining time, as the carts show theirs).

use crate::support;

use egui_kittest::kittest::Queryable;
use fp_app::ui::view::fade_remaining_fraction;
use fp_model::Command;

#[test]
fn the_fraction_drains_from_one_to_zero_over_the_fade() {
    assert_eq!(fade_remaining_fraction(10.0, 10.0, 1000), 1.0);
    assert!((fade_remaining_fraction(10.0, 10.25, 1000) - 0.75).abs() < 1e-6);
    assert!((fade_remaining_fraction(10.0, 10.5, 1000) - 0.5).abs() < 1e-6);
    assert_eq!(fade_remaining_fraction(10.0, 11.0, 1000), 0.0);
}

#[test]
fn the_fraction_stays_within_zero_and_one() {
    assert_eq!(fade_remaining_fraction(10.0, 99.0, 1000), 0.0);
    assert_eq!(
        fade_remaining_fraction(10.0, 9.0, 1000),
        1.0,
        "clock went back"
    );
    assert_eq!(fade_remaining_fraction(10.0, 10.5, 0), 0.0, "no length");
    assert_eq!(fade_remaining_fraction(f64::NAN, 10.0, 1000), 0.0);
}

type H = egui_kittest::Harness<'static, fp_app::ui::app::AppUi>;

/// The countdown band painted inside `button`, when there is one. The band
/// takes the tone of its button: green on Play / Next, the button's own
/// content tone on the neutral ones.
fn bar(h: &H, button: egui::Rect) -> Option<egui::Rect> {
    use fp_app::ui::theme;
    fn rects(shape: &egui::Shape, out: &mut Vec<egui::epaint::RectShape>) {
        match shape {
            egui::Shape::Rect(r) => out.push(r.clone()),
            egui::Shape::Vec(v) => v.iter().for_each(|s| rects(s, out)),
            _ => {}
        }
    }
    let mut all = Vec::new();
    for clipped in &h.output().shapes {
        rects(&clipped.shape, &mut all);
    }
    let colours = [
        theme::play_sweep(),
        theme::sweep_tone(theme::NEUTRAL_300),
        theme::sweep_tone(theme::TEXT),
    ];
    all.iter()
        .filter(|r| colours.contains(&r.fill) && button.expand(0.5).contains_rect(r.rect))
        .map(|r| r.rect)
        .max_by(|a, b| a.width().total_cmp(&b.width()))
}

/// Width of the countdown band inside `button` (0 when none).
fn bar_width(h: &H, button: egui::Rect) -> f32 {
    bar(h, button).map_or(0.0, |r| r.width())
}

const NEXT: &str = "Fade into the next track";

#[test]
fn a_fade_stop_drains_on_the_fade_stop_button_and_never_on_next() {
    use fp_app::ui::controller::Controller;
    let (mut h, fake) = support::harness(support::state(1, 3));
    let p = fake.player(0);
    fake.send(Command::Play(p));
    h.run_steps(3);
    let button = h.get_by_label("Fade stop").rect();
    let next = h.get_by_label(NEXT).rect();
    assert_eq!(bar_width(&h, button), 0.0, "no fade, no line");
    assert_eq!(bar_width(&h, next), 0.0, "no fade, no line");
    fake.send(Command::FadeStop(p));
    h.run_steps(2);
    let first = bar_width(&h, button);
    assert!(first > button.width() * 0.9, "full at the start: {first}");
    assert_eq!(bar_width(&h, next), 0.0, "a fade stop never draws on Next");
    h.run_steps(25);
    let later = bar_width(&h, button);
    assert!(
        later < first * 0.8 && later > 0.0,
        "draining: {first} {later}"
    );
    assert_eq!(bar_width(&h, next), 0.0, "a fade stop never draws on Next");
}

#[test]
fn a_crossfade_drains_on_next_and_not_on_fade_stop() {
    use fp_app::ui::controller::Controller;
    let (mut h, fake) = support::harness(support::state(1, 3));
    let p = fake.player(0);
    fake.send(Command::Play(p));
    h.run_steps(3);
    let next = h.get_by_label(NEXT).rect();
    let stop = h.get_by_label("Fade stop").rect();
    fake.send(Command::Play(p)); // crossfade
    h.run_steps(2);
    let first = bar_width(&h, next);
    assert!(first > next.width() * 0.9, "full at the start: {first}");
    h.run_steps(25);
    let later = bar_width(&h, next);
    assert!(
        later < first * 0.8 && later > 0.0,
        "draining: {first} {later}"
    );
    assert_eq!(bar_width(&h, stop), 0.0, "Fade stop did not start it");
}

#[test]
fn a_segue_overlap_drains_on_next_over_its_own_length() {
    use fp_app::ui::controller::Controller;
    let (mut h, fake) = support::harness(support::state(1, 3));
    let p = fake.player(0);
    fake.send(Command::Play(p));
    h.run_steps(3);
    let next = h.get_by_label(NEXT).rect();
    let stop = h.get_by_label("Fade stop").rect();
    // The engine started the next track with a 10 s overlap (the configured
    // fade is 1 s).
    let mut s = (**fake.state.load()).clone();
    s.players[0].fading = true;
    s.players[0].fade_ms = 10_000;
    s.players[0].fade_origin = fp_model::FadeOrigin::Segue;
    s.players[0].fade_seq += 1;
    fake.state.store(std::sync::Arc::new(s));
    h.run_steps(2);
    h.run_steps(25);
    let left = bar_width(&h, next);
    assert!(
        left > next.width() * 0.9,
        "half a second into 10 s is still nearly full: {left}"
    );
    assert_eq!(bar_width(&h, stop), 0.0);
}

#[test]
fn a_fade_stop_during_a_crossfade_moves_the_bar_to_fade_stop() {
    use fp_app::ui::controller::Controller;
    let (mut h, fake) = support::harness(support::state(1, 3));
    let p = fake.player(0);
    fake.send(Command::Play(p));
    h.run_steps(3);
    let next = h.get_by_label(NEXT).rect();
    let stop = h.get_by_label("Fade stop").rect();
    fake.send(Command::Play(p)); // crossfade
    h.run_steps(2);
    h.run_steps(30);
    let draining = bar_width(&h, next);
    assert!(
        draining > 0.0 && draining < next.width() * 0.7,
        "draining: {draining}"
    );
    fake.send(Command::FadeStop(p));
    h.run_steps(2);
    assert_eq!(bar_width(&h, next), 0.0, "Next lets go of it");
    let restarted = bar_width(&h, stop);
    assert!(
        restarted > stop.width() * 0.9,
        "full again on Fade stop for the new fade: {restarted}"
    );
}

#[test]
fn a_crossfade_started_by_previous_drains_on_previous_not_on_next() {
    use fp_app::ui::controller::Controller;
    let (mut h, fake) = support::harness(support::state(1, 3));
    let p = fake.player(0);
    fake.send(Command::Play(p));
    fake.send(Command::Play(p)); // crossfade into the second track
    // The engine reports it done; Previous goes back.
    let mut s = (**fake.state.load()).clone();
    s.players[0].fading = false;
    s.players[0].fade_ms = 0;
    fake.state.store(std::sync::Arc::new(s));
    h.run_steps(3);
    let previous = h.get_by_label("Previous track").rect();
    let next = h.get_by_label(NEXT).rect();
    let stop = h.get_by_label("Fade stop").rect();
    fake.send(Command::Previous(p));
    h.run_steps(2);
    let first = bar_width(&h, previous);
    assert!(first > previous.width() * 0.9, "full at the start: {first}");
    h.run_steps(25);
    let later = bar_width(&h, previous);
    assert!(
        later < first * 0.8 && later > 0.0,
        "draining: {first} {later}"
    );
    assert_eq!(bar_width(&h, next), 0.0, "Next did not start it");
    assert_eq!(bar_width(&h, stop), 0.0, "Fade stop did not start it");
    // A fade stop pressed meanwhile takes the countdown over.
    fake.send(Command::FadeStop(p));
    h.run_steps(2);
    assert_eq!(bar_width(&h, previous), 0.0);
    assert!(bar_width(&h, stop) > stop.width() * 0.9);
}

#[test]
fn the_sweep_runs_left_to_right_on_every_button() {
    use fp_app::ui::controller::Controller;
    let (mut h, fake) = support::harness(support::state(1, 3));
    let p = fake.player(0);
    fake.send(Command::Play(p));
    h.run_steps(3);
    let next = h.get_by_label(NEXT).rect();
    let stop = h.get_by_label("Fade stop").rect();
    fake.send(Command::Play(p)); // crossfade: the band is on Next
    h.run_steps(2);
    let first = bar(&h, next).expect("band on Next");
    h.run_steps(25);
    let later = bar(&h, next).expect("band on Next");
    assert!(
        (first.right() - next.right()).abs() < 0.5 && (later.right() - next.right()).abs() < 0.5,
        "the band stays on the right edge: {first:?} {later:?}"
    );
    assert!(
        later.left() > first.left() + 5.0,
        "its left edge moves right: {first:?} {later:?}"
    );
    fake.send(Command::FadeStop(p));
    h.run_steps(2);
    let first = bar(&h, stop).expect("band on Fade stop");
    h.run_steps(25);
    let later = bar(&h, stop).expect("band on Fade stop");
    assert!((later.right() - stop.right()).abs() < 0.5);
    assert!(later.left() > first.left() + 3.0, "{first:?} {later:?}");
}

#[test]
fn the_sweep_takes_the_tone_of_its_button() {
    use fp_app::ui::{controller::Controller, theme};
    let (mut h, fake) = support::harness(support::state(1, 3));
    let p = fake.player(0);
    fake.send(Command::Play(p));
    h.run_steps(3);
    let next = h.get_by_label(NEXT).rect();
    fake.send(Command::Play(p));
    h.run_steps(2);
    let fill_on = |h: &H, button: egui::Rect| -> Vec<egui::Color32> {
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
        all.iter()
            .filter(|r| button.expand(0.5).contains_rect(r.rect))
            .map(|r| r.fill)
            .collect()
    };
    let on_next = fill_on(&h, next);
    assert!(on_next.contains(&theme::play_sweep()), "green on Play/Next");
    assert!(
        !on_next.contains(&theme::ACCENT.gamma_multiply(0.5)),
        "never the violet"
    );
    // Previous (a crossfade it started) takes its own neutral tone.
    let (mut h, fake) = support::harness(support::state(1, 3));
    let p = fake.player(0);
    fake.send(Command::Play(p));
    fake.send(Command::Play(p));
    let mut s = (**fake.state.load()).clone();
    s.players[0].fading = false;
    s.players[0].fade_ms = 0;
    fake.state.store(std::sync::Arc::new(s));
    h.run_steps(3);
    let previous = h.get_by_label("Previous track").rect();
    fake.send(Command::Previous(p));
    h.run_steps(2);
    let on_prev = fill_on(&h, previous);
    assert!(on_prev.contains(&theme::sweep_tone(theme::NEUTRAL_300)));
    assert!(!on_prev.contains(&theme::play_sweep()));
}
