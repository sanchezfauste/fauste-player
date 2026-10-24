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
    // Idle and on air share a fill (the caption tells Play from Next).
    let fills = [
        theme::PLAY_FILL,
        theme::PLAY_FILL_HOVER,
        theme::PLAY_FILL_PRESSED,
        theme::PLAY_FILL_DISABLED,
        theme::PLAY_FILL_PAUSED_DIM,
    ];
    for (i, a) in fills.iter().enumerate() {
        for b in &fills[i + 1..] {
            assert_ne!(a, b);
        }
    }
    assert!(luminance(theme::PLAY_FILL_HOVER) > luminance(theme::PLAY_FILL));
    assert!(luminance(theme::PLAY_FILL_PRESSED) < luminance(theme::PLAY_FILL));
    assert!(luminance(theme::PLAY_FILL_PAUSED_DIM) < luminance(theme::PLAY_FILL));
}

#[test]
fn the_button_has_the_next_rows_green_idle_and_on_air() {
    use fp_app::ui::controller::Controller;
    assert_eq!(theme::PLAY_FILL, theme::NEXT_ROW);
    assert_eq!(theme::PLAY_FILL_ON_AIR, theme::PLAY_FILL);
    let (mut h, fake) = support::harness(support::state(1, 3));
    h.run_steps(3);
    let play = h.get_by_label("Play").rect();
    assert_eq!(fill_of(&h, play), theme::NEXT_ROW, "idle");
    fake.send(Command::Play(fake.player(0)));
    h.run_steps(3);
    let next = h.get_by_label("Fade into the next track").rect();
    assert_eq!(fill_of(&h, next), theme::NEXT_ROW, "on air");
}

/// Whether a visible border is stroked exactly around `button`.
fn has_border(h: &H, button: Rect) -> bool {
    rect_fills(h)
        .iter()
        .any(|r| r.rect == button && r.stroke.width > 0.0 && r.stroke.color != Color32::TRANSPARENT)
}

#[test]
fn play_and_next_have_no_outline() {
    use fp_app::ui::controller::Controller;
    let (mut h, fake) = support::harness(support::state(1, 3));
    h.run_steps(3);
    let play = h.get_by_label("Play").rect();
    assert!(!has_border(&h, play), "idle");
    fake.send(Command::Play(fake.player(0)));
    h.run_steps(3);
    let next = h.get_by_label("Fade into the next track").rect();
    assert!(!has_border(&h, next), "on air");
}

#[test]
fn a_paused_player_blinks_play_like_pause() {
    use fp_app::ui::controller::Controller;
    let (mut h, fake) = support::harness(support::state(1, 3));
    let p = fake.player(0);
    fake.send(Command::Play(p));
    fake.send(Command::Pause(p));
    h.run_steps(2);
    let mut seen = std::collections::HashSet::new();
    for _ in 0..60 {
        h.step();
        let play = h.get_by_label("Play").rect();
        let pause = h.get_by_label("Pause").rect();
        seen.insert((fill_of(&h, play), fill_of(&h, pause)));
    }
    // In phase: Play is lit while Pause is lit, dim while Pause is dark.
    assert_eq!(
        seen,
        [
            (theme::PLAY_FILL, theme::AMBER_BG),
            (theme::PLAY_FILL_PAUSED_DIM, Color32::TRANSPARENT),
        ]
        .into_iter()
        .collect(),
    );
    assert!(contrast(theme::PLAY_ICON, theme::PLAY_FILL_PAUSED_DIM) >= 3.0);
}

#[test]
fn a_playing_player_does_not_blink_play() {
    let (mut h, _fake) = support::harness(support::state(1, 3));
    h.run_steps(2);
    let mut seen = std::collections::HashSet::new();
    for _ in 0..60 {
        h.step();
        seen.insert(fill_of(&h, h.get_by_label("Play").rect()));
    }
    assert_eq!(seen.len(), 1, "{seen:?}");
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
            .any(|r| r.rect == rect.shrink(1.0) && r.stroke.color == theme::FOCUS_RING)
    };
    assert!(!ring(&h));
    play.focus();
    h.run_steps(2);
    assert!(ring(&h), "ring when focused");
    // Inside the tile: it never spills over a neighbour.
    let outside = rect_fills(&h).iter().any(|r| {
        r.stroke.color == theme::FOCUS_RING
            && !rect
                .expand(0.5)
                .contains_rect(r.rect.expand(r.stroke.width / 2.0))
    });
    assert!(!outside, "the ring stays within the tile");
}

#[test]
fn a_divider_separates_the_meter_from_the_fader() {
    let (mut h, _fake) = support::harness(support::state(2, 3));
    h.run_steps(3);
    let fills = rect_fills(&h);
    let dividers: Vec<Rect> = fills
        .iter()
        .filter(|r| r.fill == theme::DIVIDER && r.rect.width() == 1.0 && r.rect.height() > 60.0)
        .map(|r| r.rect)
        .collect();
    assert_eq!(dividers.len(), 2, "one per player: {dividers:?}");
    // The same hairline as the waveform's (seek bar's) border, so the
    // column reads as one piece.
    let wave_borders: std::collections::HashSet<Color32> = fills
        .iter()
        .filter(|r| r.stroke.width == 1.0 && r.rect.height() > 40.0 && r.rect.width() > 200.0)
        .map(|r| r.stroke.color)
        .collect();
    assert!(
        wave_borders.contains(&theme::DIVIDER),
        "{:?} not in {wave_borders:?}",
        theme::DIVIDER
    );
    // Found through the fader's track (4 px wide, NEUTRAL_800) on its
    // right.
    for d in &dividers {
        let track = fills
            .iter()
            .find(|r| {
                r.fill == theme::NEUTRAL_800
                    && r.rect.width() == 4.0
                    && r.rect.height() > 60.0
                    && (r.rect.top() - d.top()).abs() < 2.0
                    && r.rect.left() > d.right()
                    && r.rect.left() - d.right() < 20.0
            })
            .expect("a fader track right of the divider")
            .rect;
        // Clear of the fader's knob (12 px wide, centred on the track).
        let to_knob = track.center().x - 6.0 - d.right();
        assert!(to_knob >= 5.0, "{to_knob}: {d:?} {track:?}");
        // It stops above the fader's level readout, which may overhang the
        // gap ("-60.0" is wider than the knob).
        assert!(
            d.bottom() <= track.bottom() + 1.0,
            "{d:?} reaches the readout under {track:?}"
        );
    }
}

#[test]
fn the_divider_keeps_clear_of_the_meter_and_the_fader_readout() {
    let (mut h, _fake) = support::harness(support::state(1, 3));
    h.run_steps(3);
    let fills = rect_fills(&h);
    let d = fills
        .iter()
        .find(|r| r.fill == theme::DIVIDER && r.rect.width() == 1.0 && r.rect.height() > 60.0)
        .expect("a divider")
        .rect;
    let readout = h.get_by_label("0.0").rect();
    assert!(
        d.bottom() <= readout.top(),
        "{d:?} overlaps the readout {readout:?}"
    );
    assert!(
        readout.left() - d.right() >= 3.0,
        "{d:?} sticks to the readout {readout:?}"
    );
}

/// The colours the glyph and caption texts inside `button` are painted in.
fn text_colours_in(h: &H, button: Rect) -> Vec<Color32> {
    fn walk(s: &egui::Shape, button: Rect, out: &mut Vec<Color32>) {
        match s {
            egui::Shape::Text(t) if button.contains(t.pos) => {
                out.push(t.fallback_color);
                out.extend(t.galley.job.sections.iter().map(|s| s.format.color));
            }
            egui::Shape::Vec(v) => v.iter().for_each(|s| walk(s, button, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    for c in &h.output().shapes {
        walk(&c.shape, button, &mut out);
    }
    out
}

#[test]
fn during_a_crossfade_the_button_keeps_the_on_air_look_with_a_white_icon() {
    use fp_app::ui::controller::Controller;
    let (mut h, fake) = support::harness(support::state(1, 3));
    let p = fake.player(0);
    fake.send(Command::Play(p));
    h.run_steps(3);
    fake.send(Command::Play(p)); // crossfade
    h.run_steps(8);
    let next = h.get_by_label("Fade into the next track");
    assert!(next.accesskit_node().is_disabled(), "still not clickable");
    let rect = next.rect();
    assert_eq!(fill_of(&h, rect), theme::PLAY_FILL_ON_AIR, "no muted green");
    let colours = text_colours_in(&h, rect);
    assert!(colours.contains(&theme::PLAY_ICON), "{colours:?}");
    assert!(
        colours
            .iter()
            .all(|c| *c == theme::PLAY_ICON || *c == Color32::PLACEHOLDER),
        "an undimmed icon and caption: {colours:?}"
    );
    let band = theme::play_sweep();
    assert!(
        rect_fills(&h)
            .iter()
            .any(|r| r.fill == band && rect.expand(0.5).contains_rect(r.rect))
    );
    let a = f32::from(band.a()) / 255.0;
    let under = |f: u8, b: u8| (f32::from(b) + f32::from(f) * (1.0 - a)).round() as u8;
    let fill = theme::PLAY_FILL_ON_AIR;
    let over = Color32::from_rgb(
        under(fill.r(), band.r()),
        under(fill.g(), band.g()),
        under(fill.b(), band.b()),
    );
    assert!(contrast(theme::PLAY_ICON, over) >= 3.0);
}
