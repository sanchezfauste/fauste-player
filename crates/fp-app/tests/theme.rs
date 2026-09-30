#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The Nocturne theme (spec §8.1).

use egui::{Color32, FontFamily};
use fp_app::ui::theme::{self, WAVE_PALETTE, wave_colors};

#[test]
fn every_waveform_colour_name_resolves() {
    assert_eq!(WAVE_PALETTE.len(), 9);
    for (name, colours) in WAVE_PALETTE {
        assert_eq!(wave_colors(name), *colours, "{name}");
        assert_eq!(
            wave_colors(&name.to_uppercase()),
            *colours,
            "{name} ignores case"
        );
    }
    assert_eq!(
        wave_colors("sand").played,
        Color32::from_rgb(0xe0, 0xcf, 0xac)
    );
    let slate = wave_colors("slate");
    assert_eq!(slate.played, theme::NEUTRAL_300);
    assert_eq!(slate.unplayed, Color32::from_rgb(0x4a, 0x4e, 0x5c));
    assert_eq!(wave_colors("no such colour"), slate);
}

#[test]
fn fonts_provide_inter_weights_and_both_icon_variants() {
    let fonts = theme::fonts();
    for family in [theme::MEDIUM, theme::SEMIBOLD, theme::ICONS_FILL] {
        assert!(
            fonts
                .families
                .contains_key(&FontFamily::Name(family.into())),
            "{family}"
        );
    }
    let proportional = &fonts.families[&FontFamily::Proportional];
    assert_eq!(proportional[0], "inter-400");
    assert!(proportional.iter().any(|f| f == "phosphor"));
}

#[test]
fn applying_the_theme_uses_square_corners_and_the_nocturne_background() {
    let ctx = egui::Context::default();
    theme::apply(&ctx);
    let style = ctx.global_style();
    assert_eq!(style.visuals.panel_fill, theme::BG);
    assert_eq!(
        style.visuals.widgets.inactive.corner_radius,
        egui::CornerRadius::ZERO
    );
    assert!(style.visuals.dark_mode);
}

#[test]
fn drawn_icons_stay_inside_their_rectangle() {
    use fp_app::ui::icons;
    let rect = egui::Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(16.0, 16.0));
    for shapes in [
        icons::fade_stop(rect, theme::TEXT),
        icons::stop_after(rect, theme::TEXT),
    ] {
        assert!(!shapes.is_empty());
        for shape in shapes {
            assert!(
                rect.expand(1.0).contains_rect(shape.visual_bounding_rect()),
                "{shape:?}"
            );
        }
    }
}

#[test]
fn stop_after_is_a_play_triangle_then_a_stop_square() {
    use fp_app::ui::icons;
    // The player draws it in an 18 × 13 rectangle.
    let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(18.0, 13.0));
    let shapes = icons::stop_after(rect, theme::TEXT);
    assert_eq!(shapes.len(), 2);
    let triangle = shapes[0].visual_bounding_rect();
    let square = shapes[1].visual_bounding_rect();
    assert!(matches!(shapes[1], egui::Shape::Rect(_)), "{:?}", shapes[1]);
    assert!(triangle.right() < square.left(), "{triangle:?} {square:?}");
    assert!(
        (square.width() - square.height()).abs() < 0.01,
        "{square:?}"
    );
    assert!((triangle.center().y - square.center().y).abs() < 0.01);
    for r in [triangle, square] {
        assert!(rect.contains_rect(r), "{r:?}");
    }
}

#[test]
fn meter_colours_are_the_muted_traffic_light() {
    assert_eq!(theme::METER_NORMAL, Color32::from_rgb(0x7f, 0xb0, 0x8a));
    assert_eq!(theme::METER_WARNING, Color32::from_rgb(0xd9, 0xb4, 0x5a));
    assert_eq!(theme::METER_DANGER, Color32::from_rgb(0xd8, 0x64, 0x6a));
}
