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
    assert_eq!(WAVE_PALETTE.len(), 8);
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
    assert_eq!(wave_colors("no such colour"), wave_colors("sand"));
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
