#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Itr05 UI polish: the BP badge, the settings buttons, the column arrows,
//! the mix marker colour.

use egui::{Color32, Shape};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::status_legend;
use fp_app::ui::theme;
use fp_app::ui::widgets::{
    SETTINGS_BUTTON_HEIGHT, SETTINGS_BUTTON_PAD, TileStyle, bp_tile_style, font, mix_marker_color,
    padded_button_width, text_button,
};

mod support;
use support::{harness, state};

#[test]
fn the_bp_badge_is_green_on_and_grey_off() {
    let on = bp_tile_style(true);
    assert_eq!(on.border, theme::PLAY_BORDER);
    assert_eq!(on.content, theme::PLAY_HOVER);
    assert!(!on.dim_when_disabled, "drawn at full strength");
    let off = bp_tile_style(false);
    assert_eq!(off.border, theme::NEUTRAL_700);
    assert_eq!(off.content, theme::NEUTRAL_600);
}

#[test]
fn a_button_is_its_text_plus_the_padding_on_both_sides() {
    assert_eq!(padded_button_width(50.0), 50.0 + 2.0 * SETTINGS_BUTTON_PAD);
}

#[test]
fn a_settings_button_clears_its_text_by_the_padding_with_and_without_a_glyph() {
    for glyph in [None, Some(egui_phosphor::regular::FOLDER_OPEN)] {
        for label in ["OK", "Import M3U / PLS…", "Re-analyse all tracks"] {
            let seen = std::cell::Cell::new((0.0_f32, 0.0_f32, 0.0_f32));
            let mut h = Harness::new_ui(|ui| {
                let shown = match glyph {
                    Some(g) => format!("{g} {label}"),
                    None => label.to_owned(),
                };
                let text_width = ui
                    .painter()
                    .layout_no_wrap(shown, font(12.0), theme::TEXT)
                    .size()
                    .x;
                let r = text_button(ui, glyph, label, true, TileStyle::plain());
                seen.set((r.rect.width(), r.rect.height(), text_width));
            });
            h.run();
            let _ = h.get_by_label(label);
            let (width, height, text_width) = seen.get();
            assert!(
                width >= text_width + 2.0 * SETTINGS_BUTTON_PAD - 0.01,
                "{label}: {width} vs {text_width}"
            );
            assert_eq!(height, SETTINGS_BUTTON_HEIGHT);
        }
    }
}

#[test]
fn the_active_mix_marker_is_violet_and_not_like_the_outro_marker() {
    assert_eq!(mix_marker_color(true), theme::MIX_LINE);
    assert_eq!(theme::MIX_LINE, theme::ACCENT, "the MIXING badge's colour");
    assert_ne!(theme::MIX_LINE, theme::OUTRO_LINE);
    assert_ne!(mix_marker_color(true), theme::AMBER);
    // Inactive: the dim grey it always had.
    assert_eq!(
        mix_marker_color(false),
        theme::NEUTRAL_600.gamma_multiply(0.6)
    );
}

#[test]
fn the_status_bar_legend_uses_the_mix_marker_colour() {
    let legend = status_legend();
    let (colour, key) = legend[0];
    assert_eq!(key, "legend-mix");
    assert_eq!(colour, mix_marker_color(true));
}

#[test]
fn the_status_bar_legend_names_the_outro() {
    let legend = status_legend();
    assert!(
        legend.contains(&(theme::OUTRO_LINE, "legend-outro")),
        "{legend:?}"
    );
}

/// Every leaf shape of the last frame.
fn leaves(shapes: &[egui::epaint::ClippedShape]) -> Vec<Shape> {
    fn walk(s: &Shape, out: &mut Vec<Shape>) {
        match s {
            Shape::Vec(v) => v.iter().for_each(|s| walk(s, out)),
            other => out.push(other.clone()),
        }
    }
    let mut out = Vec::new();
    shapes.iter().for_each(|c| walk(&c.shape, &mut out));
    out
}

/// The border colour of the rectangle painted exactly at `rect`, and the
/// colour of the "BP" text inside it.
fn badge_paint(h: &Harness<'_, fp_app::ui::app::AppUi>, label: &str) -> (Color32, Color32) {
    let rect = h.get_by_label(label).rect();
    let shapes = leaves(&h.output().shapes);
    let border = shapes
        .iter()
        .find_map(|s| match s {
            Shape::Rect(r)
                if (r.rect.center() - rect.center()).length() < 0.5
                    && r.stroke.width > 0.0
                    && (r.rect.width() - rect.width()).abs() < 0.5 =>
            {
                Some(r.stroke.color)
            }
            _ => None,
        })
        .expect("the badge border");
    let text = shapes
        .iter()
        .find_map(|s| match s {
            Shape::Text(t) if t.galley.text() == "BP" => {
                t.galley.job.sections.first().map(|sec| sec.format.color)
            }
            _ => None,
        })
        .expect("the BP text");
    (border, text)
}

#[test]
fn the_player_paints_the_bp_badge_green_when_on_and_grey_off() {
    let (mut h, fake) = harness(state(1, 1));
    h.run_steps(2);
    let (border, text) = badge_paint(&h, "Bit-perfect: off");
    assert_eq!(border, theme::NEUTRAL_700.gamma_multiply(0.45));
    assert_eq!(text, theme::NEUTRAL_600.gamma_multiply(0.45));
    let player = fake.player(0);
    fake.telemetry
        .store(std::sync::Arc::new(fp_engine::conductor::Telemetry {
            players: vec![(
                player,
                fp_engine::engine::PlayerTelemetry {
                    bit_perfect: true,
                    ..Default::default()
                },
            )],
            ..Default::default()
        }));
    h.run_steps(2);
    let (border, text) = badge_paint(&h, "Bit-perfect: on");
    assert_eq!(border, theme::PLAY_BORDER, "not dimmed");
    assert_eq!(text, theme::PLAY_HOVER, "not dimmed");
}
