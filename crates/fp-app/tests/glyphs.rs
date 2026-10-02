#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Operator feedback 2, O18: one icon per transport action.

use egui::{Rect, pos2};
use fp_app::ui::glyphs::{self, TransportAction};
use fp_app::ui::theme;

#[test]
fn every_action_has_exactly_one_kind_of_icon_and_no_two_share_one() {
    let mut seen: Vec<String> = Vec::new();
    for action in TransportAction::ALL {
        let font = glyphs::font_glyph(action);
        let drawn = glyphs::drawn(action);
        assert!(
            font.is_some() != drawn.is_some(),
            "{action:?} must be a font glyph or a drawn icon, not both or neither"
        );
        let key = match (font, drawn) {
            (Some((g, fill)), _) => format!("font:{g}:{fill}"),
            (_, Some(d)) => {
                // Drawn icons are told apart by their shapes.
                let rect = Rect::from_min_size(pos2(0.0, 0.0), d.size);
                format!("drawn:{:?}", (d.draw)(rect, theme::TEXT))
            }
            _ => unreachable!(),
        };
        assert!(!seen.contains(&key), "{action:?} repeats another icon");
        seen.push(key);
    }
    assert_eq!(seen.len(), 9);
}

#[test]
fn stop_is_the_filled_stop_square_and_cue_the_headphones() {
    assert_eq!(
        glyphs::font_glyph(TransportAction::Stop),
        Some((egui_phosphor::fill::STOP, true))
    );
    assert_eq!(
        glyphs::font_glyph(TransportAction::Cue),
        Some((egui_phosphor::regular::HEADPHONES, false))
    );
    assert_eq!(
        glyphs::glyph_text(TransportAction::Stop),
        egui_phosphor::fill::STOP
    );
    assert_eq!(
        glyphs::glyph_text(TransportAction::Play),
        egui_phosphor::fill::PLAY
    );
    // A drawn icon has no text form.
    assert_eq!(glyphs::glyph_text(TransportAction::FadeStop), "");
}

#[test]
fn drawn_actions_stay_inside_their_natural_size() {
    for action in TransportAction::ALL {
        let Some(d) = glyphs::drawn(action) else {
            continue;
        };
        let rect = Rect::from_min_size(pos2(10.0, 20.0), d.size);
        let shapes = (d.draw)(rect, theme::TEXT);
        assert!(!shapes.is_empty(), "{action:?}");
        for shape in shapes {
            assert!(
                rect.expand(1.0).contains_rect(shape.visual_bounding_rect()),
                "{action:?} {shape:?}"
            );
        }
        assert!(d.size.x > 0.0 && d.size.y > 0.0 && d.size.x <= 24.0 && d.size.y <= 16.0);
    }
}

/// The player, the cartwall and the track table draw transport icons only
/// through `glyphs` (a status marker such as the table's pause icon is not
/// a transport action and may stay).
#[test]
fn the_screens_do_not_name_transport_icons_themselves() {
    let sources = [
        ("player.rs", include_str!("../src/ui/player.rs")),
        ("cartwall.rs", include_str!("../src/ui/cartwall.rs")),
        ("table.rs", include_str!("../src/ui/table.rs")),
        ("cue_window.rs", include_str!("../src/ui/cue_window.rs")),
    ];
    for (name, source) in sources {
        for forbidden in [
            "fill::STOP",
            "fill::PLAY",
            "fill::FAST_FORWARD",
            "HEADPHONES",
            "icons::",
        ] {
            assert!(
                !source.contains(forbidden),
                "{name} still names {forbidden}; use ui::glyphs"
            );
        }
    }
}
