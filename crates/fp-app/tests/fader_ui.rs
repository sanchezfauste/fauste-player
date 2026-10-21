#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The player's volume fader: relative drag, fine adjust, wheel, double
//! click and the scale marks.

mod support;

use egui::{Event, Modifiers, MouseWheelUnit, PointerButton, TouchPhase, pos2};
use egui_kittest::kittest::Queryable;
use fp_app::ui::controller::Controller;
use fp_app::ui::widgets::{FADER_MARKS_DB, fader_mark_travels};
use fp_model::Command;
use fp_model::volume::{fader_from_gain, gain_from_fader};
use support::{Fake, harness, lock_player, state};

type H = egui_kittest::Harness<'static, fp_app::ui::app::AppUi>;

/// A one-player harness whose volume sits at fader travel `travel`.
fn at(travel: f32, locked: bool) -> (H, std::sync::Arc<Fake>) {
    let mut s = state(1, 1);
    s.players[0].volume = gain_from_fader(travel);
    if locked {
        lock_player(&mut s, 0);
    }
    let (mut h, fake) = harness(s);
    h.run_steps(2);
    (h, fake)
}

fn fader_rect(h: &H) -> egui::Rect {
    h.query_all_by_label_contains("Volume")
        .next()
        .expect("a fader")
        .rect()
}

/// The pixels the knob travels over the whole fader.
fn travel_px(h: &H) -> f32 {
    fader_rect(h).height() - 6.0
}

fn travel(fake: &Fake) -> f32 {
    fader_from_gain(fake.state.load().players[0].volume)
}

fn button(h: &H, at: egui::Pos2, pressed: bool, modifiers: Modifiers) {
    h.event(Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed,
        modifiers,
    });
}

/// Presses at `from`, moves in small steps to `to` and releases.
fn drag(h: &mut H, from: egui::Pos2, to: egui::Pos2, modifiers: Modifiers) {
    h.event(Event::ModifiersChanged(modifiers));
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    button(h, from, true, modifiers);
    h.run_steps(1);
    for i in 1..=10 {
        let f = i as f32 / 10.0;
        h.event(Event::PointerMoved(from + (to - from) * f));
        h.run_steps(1);
    }
    button(h, to, false, modifiers);
    h.run_steps(2);
    h.event(Event::ModifiersChanged(Modifiers::NONE));
    h.run_steps(1);
}

fn click(h: &mut H, at: egui::Pos2) {
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    button(h, at, true, Modifiers::NONE);
    h.run_steps(1);
    button(h, at, false, Modifiers::NONE);
    h.run_steps(1);
}

fn wheel(h: &mut H, delta: f32, modifiers: Modifiers) {
    wheel_events(
        h,
        &[egui::vec2(0.0, delta)],
        MouseWheelUnit::Line,
        modifiers,
    );
}

/// Wheel events over the first fader, one per frame.
fn wheel_events(h: &mut H, deltas: &[egui::Vec2], unit: MouseWheelUnit, modifiers: Modifiers) {
    let at = fader_rect(h).center();
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    h.event(Event::ModifiersChanged(modifiers));
    for &delta in deltas {
        h.event(Event::MouseWheel {
            unit,
            delta,
            phase: TouchPhase::Move,
            modifiers,
        });
        h.run_steps(1);
    }
    h.run_steps(20);
    h.event(Event::ModifiersChanged(Modifiers::NONE));
    h.run_steps(1);
}

fn set_volumes(fake: &Fake) -> Vec<f32> {
    fake.take_sent()
        .into_iter()
        .filter_map(|c| match c {
            Command::SetVolume(_, v) => Some(v),
            _ => None,
        })
        .collect()
}

#[test]
fn a_click_alone_changes_nothing() {
    let (mut h, fake) = at(0.5, false);
    let r = fader_rect(&h);
    click(&mut h, pos2(r.center().x, r.top() + 4.0));
    assert!(set_volumes(&fake).is_empty());
    assert!((travel(&fake) - 0.5).abs() < 1e-4);
}

#[test]
fn a_drag_moves_the_fader_by_the_pointer_delta_not_to_the_pointer() {
    let (mut h, fake) = at(0.5, false);
    let r = fader_rect(&h);
    // Grab near the top, far from the knob, and pull 20 px down.
    let from = pos2(r.center().x, r.top() + 4.0);
    drag(&mut h, from, from + egui::vec2(0.0, 20.0), Modifiers::NONE);
    let expected = 0.5 - 20.0 / travel_px(&h);
    assert!((travel(&fake) - expected).abs() < 0.01, "{}", travel(&fake));
}

#[test]
fn holding_still_does_not_resend_the_volume() {
    let (mut h, fake) = at(0.5, false);
    let r = fader_rect(&h);
    let from = pos2(r.center().x, r.top() + 4.0);
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    button(&h, from, true, Modifiers::NONE);
    h.run_steps(1);
    h.event(Event::PointerMoved(from + egui::vec2(0.0, 20.0)));
    h.run_steps(1);
    fake.take_sent();
    h.run_steps(10);
    assert!(set_volumes(&fake).is_empty());
}

#[test]
fn ctrl_and_shift_drags_are_ten_times_finer() {
    for modifiers in [Modifiers::CTRL, Modifiers::SHIFT] {
        let (mut h, fake) = at(0.5, false);
        let r = fader_rect(&h);
        let from = pos2(r.center().x, r.top() + 4.0);
        drag(&mut h, from, from + egui::vec2(0.0, 30.0), modifiers);
        let expected = 0.5 - 3.0 / travel_px(&h);
        assert!((travel(&fake) - expected).abs() < 0.003, "{modifiers:?}");
    }
}

#[test]
fn a_wheel_notch_moves_the_fader_a_fixed_fine_step() {
    let (mut h, fake) = at(0.5, false);
    wheel(&mut h, 1.0, Modifiers::NONE);
    assert!((travel(&fake) - 0.525).abs() < 1e-3, "{}", travel(&fake));
    wheel(&mut h, -1.0, Modifiers::NONE);
    assert!((travel(&fake) - 0.5).abs() < 1e-3, "{}", travel(&fake));
}

#[test]
fn the_wheel_step_is_about_six_tenths_of_a_db_near_zero_db() {
    let (mut h, fake) = at(1.0, false);
    wheel(&mut h, -1.0, Modifiers::NONE);
    let db = 20.0 * fake.state.load().players[0].volume.log10();
    assert!((-0.7..-0.5).contains(&db), "{db}");
}

#[test]
fn ctrl_makes_the_wheel_ten_times_finer() {
    let (mut h, fake) = at(0.5, false);
    wheel(&mut h, 1.0, Modifiers::CTRL);
    assert!((travel(&fake) - 0.5025).abs() < 2e-4, "{}", travel(&fake));
}

#[test]
fn a_double_click_returns_to_zero_db() {
    let (mut h, fake) = at(0.5, false);
    let r = fader_rect(&h);
    let p = pos2(r.center().x, r.center().y);
    click(&mut h, p);
    click(&mut h, p);
    assert_eq!(set_volumes(&fake), vec![1.0]);
}

#[test]
fn a_locked_fader_ignores_drag_wheel_and_double_click() {
    let (mut h, fake) = at(0.5, true);
    let r = h.get_by_label_contains("Volume held").rect();
    let from = pos2(r.center().x, r.top() + 4.0);
    drag(&mut h, from, from + egui::vec2(0.0, 20.0), Modifiers::NONE);
    wheel(&mut h, 1.0, Modifiers::NONE);
    click(&mut h, r.center());
    click(&mut h, r.center());
    assert!(set_volumes(&fake).is_empty());
}

#[test]
fn the_marks_follow_the_curve() {
    assert_eq!(
        FADER_MARKS_DB,
        [
            0.0,
            -2.0,
            -4.0,
            -6.0,
            -8.0,
            -10.0,
            -12.0,
            -15.0,
            -20.0,
            -25.0,
            -30.0,
            -40.0,
            -50.0,
            f32::NEG_INFINITY
        ]
    );
    let t = fader_mark_travels();
    assert_eq!(t[0], 1.0);
    assert_eq!(t[13], 0.0);
    assert!((20.0 * gain_from_fader(t[5]).log10() + 10.0).abs() < 0.01);
    assert!(t.windows(2).all(|w| w[0] > w[1]));
}

/// The texts painted on the last frame in the marks' size (the meter's
/// scale has marks of its own), with their colours.
fn painted(h: &H) -> Vec<(String, egui::Color32)> {
    fn walk(shape: &egui::Shape, out: &mut Vec<(String, egui::Color32)>) {
        match shape {
            egui::Shape::Text(t) if t.galley.job.sections[0].format.font_id.size == 7.0 => out
                .push((
                    t.galley.text().to_owned(),
                    t.galley.job.sections[0].format.color,
                )),
            egui::Shape::Vec(v) => v.iter().for_each(|s| walk(s, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    for c in &h.output().shapes {
        walk(&c.shape, &mut out);
    }
    out
}

#[test]
fn the_scale_marks_are_painted_and_dimmed_when_locked() {
    for (locked, colour) in [
        (false, fp_app::ui::theme::NEUTRAL_400),
        (true, fp_app::ui::theme::NEUTRAL_600),
    ] {
        let (h, _) = at(0.5, locked);
        let texts = painted(&h);
        for label in [
            "0",
            "-2",
            "-4",
            "-6",
            "-8",
            "-10",
            "-12",
            "-15",
            "-20",
            "-25",
            "-30",
            "-40",
            "-50",
            "\u{2212}\u{221e}",
        ] {
            let found: Vec<_> = texts.iter().filter(|(t, _)| t == label).collect();
            assert_eq!(found.len(), 1, "{label} in {texts:?}");
            assert_eq!(found[0].1, colour, "{label}");
        }
    }
}

#[test]
fn a_double_click_away_from_the_knob_changes_nothing() {
    let (mut h, fake) = at(0.5, false);
    let r = fader_rect(&h);
    let p = pos2(r.center().x, r.top() + 4.0);
    click(&mut h, p);
    click(&mut h, p);
    assert!(set_volumes(&fake).is_empty());
}

#[test]
fn a_second_drag_starts_from_a_volume_set_elsewhere() {
    let (mut h, fake) = at(0.5, false);
    let r = fader_rect(&h);
    let from = pos2(r.center().x, r.top() + 4.0);
    drag(&mut h, from, from + egui::vec2(0.0, 20.0), Modifiers::NONE);
    // MIDI or the remote API moves the volume between the two drags.
    fake.send(Command::SetVolume(fake.player(0), gain_from_fader(0.8)));
    h.run_steps(2);
    drag(&mut h, from, from + egui::vec2(0.0, -20.0), Modifiers::NONE);
    let expected = 0.8 + 20.0 / travel_px(&h);
    assert!((travel(&fake) - expected).abs() < 0.01, "{}", travel(&fake));
}

#[test]
fn a_sideways_scroll_without_shift_leaves_the_volume_alone() {
    let (mut h, fake) = at(0.5, false);
    wheel_events(
        &mut h,
        &[egui::vec2(1.0, 0.0)],
        MouseWheelUnit::Line,
        Modifiers::NONE,
    );
    assert!(set_volumes(&fake).is_empty());
}

#[test]
fn a_shift_wheel_that_arrives_sideways_is_a_fine_step() {
    let (mut h, fake) = at(0.5, false);
    wheel_events(
        &mut h,
        &[egui::vec2(1.0, 0.0)],
        MouseWheelUnit::Line,
        Modifiers::SHIFT,
    );
    assert!((travel(&fake) - 0.5).abs() > 1e-4, "it moved");
    assert!((travel(&fake) - 0.5).abs() < 0.003, "{}", travel(&fake));
}

#[test]
fn trackpad_scrolling_adds_up_to_whole_steps() {
    let (mut h, fake) = at(0.5, false);
    // Ten small trackpad deltas, 50 points in all: one line's worth
    // (40 points) and a remainder.
    wheel_events(
        &mut h,
        &[egui::vec2(0.0, 5.0); 10],
        MouseWheelUnit::Point,
        Modifiers::NONE,
    );
    assert!((travel(&fake) - 0.525).abs() < 1e-3, "{}", travel(&fake));
}

#[test]
fn a_shift_wheel_on_a_fader_does_not_scroll_the_players_row() {
    let (mut h, fake) = harness(state(8, 1));
    h.run_steps(2);
    let before = fader_rect(&h);
    wheel_events(
        &mut h,
        &[egui::vec2(0.0, -1.0); 6],
        MouseWheelUnit::Line,
        Modifiers::SHIFT,
    );
    assert_eq!(fader_rect(&h), before, "the row stayed put");
    let p1 = fake.player(0);
    for c in fake.take_sent() {
        if let Command::SetVolume(p, _) = c {
            assert_eq!(p, p1, "only player 1's fader moved");
        }
    }
}

#[test]
fn the_scale_labels_do_not_overlap() {
    let (h, _) = at(0.5, false);
    let mut rects: Vec<egui::Rect> = Vec::new();
    fn walk(shape: &egui::Shape, out: &mut Vec<egui::Rect>) {
        match shape {
            egui::Shape::Text(t) if t.galley.job.sections[0].format.font_id.size == 7.0 => {
                out.push(t.visual_bounding_rect())
            }
            egui::Shape::Vec(v) => v.iter().for_each(|s| walk(s, out)),
            _ => {}
        }
    }
    for c in &h.output().shapes {
        walk(&c.shape, &mut rects);
    }
    assert_eq!(rects.len(), FADER_MARKS_DB.len());
    rects.sort_by(|a, b| a.top().total_cmp(&b.top()));
    for w in rects.windows(2) {
        // The glyphs' ink, with a clear pixel between two labels.
        assert!(w[0].bottom() + 1.0 <= w[1].top(), "{:?} {:?}", w[0], w[1]);
    }
}

/// Presses at `from` and moves through `stops` without releasing; returns the
/// travel after each stop.
fn drag_through(h: &mut H, fake: &Fake, from: egui::Pos2, stops: &[egui::Pos2]) -> Vec<f32> {
    drag_through_with(h, fake, from, stops, Modifiers::NONE)
}

fn drag_through_with(
    h: &mut H,
    fake: &Fake,
    from: egui::Pos2,
    stops: &[egui::Pos2],
    modifiers: Modifiers,
) -> Vec<f32> {
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    button(h, from, true, modifiers);
    h.run_steps(1);
    let mut at = from;
    let mut out = Vec::new();
    for &to in stops {
        for i in 1..=10 {
            h.event(Event::PointerMoved(at + (to - at) * (i as f32 / 10.0)));
            h.run_steps(1);
        }
        at = to;
        out.push(travel(fake));
    }
    button(h, at, false, modifiers);
    h.run_steps(2);
    h.event(Event::ModifiersChanged(Modifiers::NONE));
    h.run_steps(1);
    out
}

#[test]
fn a_drag_past_the_top_and_back_keeps_the_knob_at_the_limit_until_the_pointer_returns() {
    let (mut h, fake) = at(0.9, false);
    let r = fader_rect(&h);
    let start = pos2(r.center().x, r.center().y);
    let up = (0.1 * travel_px(&h)) + 200.0;
    let top = start + egui::vec2(0.0, -up);
    let back50 = top + egui::vec2(0.0, 50.0);
    let back_more = top + egui::vec2(0.0, 200.0 + 30.0);
    let t = drag_through(&mut h, &fake, start, &[top, back50, back_more]);
    assert!((t[0] - 1.0).abs() < 1e-4, "{t:?}");
    assert!((t[1] - 1.0).abs() < 1e-4, "{t:?}");
    // 30 px beyond the 200 px of overshoot: follows without offset.
    assert!((t[2] - (1.0 - 30.0 / travel_px(&h))).abs() < 0.01, "{t:?}");
}

#[test]
fn a_drag_past_the_bottom_and_back_keeps_the_knob_at_the_limit_until_the_pointer_returns() {
    let (mut h, fake) = at(0.1, false);
    let r = fader_rect(&h);
    let start = pos2(r.center().x, r.center().y);
    let down = (0.1 * travel_px(&h)) + 200.0;
    let bottom = start + egui::vec2(0.0, down);
    let back50 = bottom - egui::vec2(0.0, 50.0);
    let back_more = bottom - egui::vec2(0.0, 230.0);
    let t = drag_through(&mut h, &fake, start, &[bottom, back50, back_more]);
    assert!(t[0].abs() < 1e-4, "{t:?}");
    assert!(t[1].abs() < 1e-4, "{t:?}");
    assert!((t[2] - 30.0 / travel_px(&h)).abs() < 0.01, "{t:?}");
}

#[test]
fn no_volume_is_sent_while_the_pointer_goes_further_beyond_the_top() {
    let (mut h, fake) = at(0.9, false);
    let r = fader_rect(&h);
    let start = pos2(r.center().x, r.center().y);
    let top = start + egui::vec2(0.0, -(0.1 * travel_px(&h) + 50.0));
    let far = top + egui::vec2(0.0, -100.0);
    h.event(Event::PointerMoved(start));
    h.run_steps(1);
    button(&h, start, true, Modifiers::NONE);
    h.run_steps(1);
    h.event(Event::PointerMoved(top));
    h.run_steps(2);
    let _ = set_volumes(&fake);
    for i in 1..=5 {
        h.event(Event::PointerMoved(top + (far - top) * (i as f32 / 5.0)));
        h.run_steps(1);
    }
    assert!(set_volumes(&fake).is_empty(), "already at the top");
    button(&h, far, false, Modifiers::NONE);
    h.run_steps(2);
}

#[test]
fn a_fine_drag_keeps_its_overshoot_too() {
    let (mut h, fake) = at(0.95, false);
    let r = fader_rect(&h);
    let start = pos2(r.center().x, r.center().y);
    // Fine mode: a tenth of the travel per pixel. 0.05 of travel takes
    // 0.05 * travel_px * 10 px; pull 100 px further, then 40 px back.
    let to_top = 0.05 * travel_px(&h) * 10.0;
    let top = start + egui::vec2(0.0, -(to_top + 100.0));
    let back = top + egui::vec2(0.0, 40.0);
    let back_more = top + egui::vec2(0.0, 100.0 + 20.0);
    h.event(Event::ModifiersChanged(Modifiers::CTRL));
    let t = drag_through_with(
        &mut h,
        &fake,
        start,
        &[top, back, back_more],
        Modifiers::CTRL,
    );
    assert!(
        (t[0] - 1.0).abs() < 1e-4 && (t[1] - 1.0).abs() < 1e-4,
        "{t:?}"
    );
    assert!(
        (t[2] - (1.0 - 20.0 / 10.0 / travel_px(&h))).abs() < 0.01,
        "{t:?}"
    );
}
