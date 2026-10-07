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
        icons::restart(rect, theme::TEXT),
        icons::previous(rect, theme::TEXT),
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

#[test]
fn restart_is_a_bar_then_one_triangle_and_previous_a_bar_then_two() {
    use fp_app::ui::icons;
    let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(18.0, 13.0));
    let restart = icons::restart(rect, theme::TEXT);
    let previous = icons::previous(rect, theme::TEXT);
    assert_eq!(restart.len(), 2);
    assert_eq!(previous.len(), 3);
    for shapes in [restart, previous] {
        assert!(matches!(shapes[0], egui::Shape::Rect(_)), "{:?}", shapes[0]);
        let bar = shapes[0].visual_bounding_rect();
        let mut left = bar.right();
        for tri in &shapes[1..] {
            let r = tri.visual_bounding_rect();
            assert!(r.left() >= left - 0.01, "{r:?} after {left}");
            assert!((r.center().y - bar.center().y).abs() < 0.01);
            assert!(rect.contains_rect(r), "{r:?}");
            left = r.right();
        }
    }
}

#[test]
fn the_meter_ticks_are_pinned() {
    // Operator feedback 4, Q11: the alignment tick is the only white mark
    // on the meter; minor ticks are the major tick's colour, fainter.
    assert_eq!(theme::METER_TICK, theme::NEUTRAL_400);
    assert_eq!(theme::METER_TICK_MINOR_ALPHA, 0.60);
    assert_eq!(theme::METER_ALIGNMENT_TICK, Color32::WHITE);
}

/// The characters each Inter weight lacks, among `text`. Each weight is
/// checked alone (its own character map), without the fallback fonts, so
/// a label never mixes typefaces.
fn missing_from_inter(text: &str) -> Vec<(String, char)> {
    let fonts = theme::fonts();
    let chars: std::collections::BTreeSet<char> =
        text.chars().filter(|c| !c.is_control()).collect();
    let mut missing = Vec::new();
    for w in ["inter-400", theme::MEDIUM, theme::SEMIBOLD] {
        let face = ttf_parser::Face::parse(&fonts.font_data[w].font, 0).unwrap();
        for &c in &chars {
            if face.glyph_index(c).is_none() {
                missing.push((w.to_owned(), c));
            }
        }
    }
    missing
}

#[test]
fn inter_covers_the_letters_of_every_planned_language() {
    // Polish, Catalan, Portuguese, German, French, Spanish, Basque,
    // Galician, Italian and Dutch, upper and lower case, and punctuation.
    let letters = "ąćęłńóśźż ĄĆĘŁŃÓŚŹŻ l·l L·L ŀ Ŀ çàèéíïòóúü ÇÀÈÉÍÏÒÓÚÜ ãõâêô ÃÕÂÊÔ \
                   ßäöü ÄÖÜẞ œæâêîôûëÿ ŒÆÂÊÎÔÛËŸ «» ‹› ‘’‚ “”„ … – — ñÑ ¿¡ ìù ĳĲ ª º €";
    assert_eq!(missing_from_inter(letters), Vec::new());
}

#[test]
fn inter_covers_every_character_of_every_locale_file() {
    for l in fp_app::i18n::LOCALES {
        // Comments are never shown.
        let shown: String = l
            .source
            .lines()
            .filter(|line| !line.trim_start().starts_with('#'))
            .collect();
        assert_eq!(missing_from_inter(&shown), Vec::new(), "{}", l.tag);
    }
}
