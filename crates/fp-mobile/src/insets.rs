//! The safe area: what the system bars and the display cutout leave of the
//! screen (mobile spec MB10). winit does not report it; the Kotlin shell
//! does, in physical pixels.

use egui::{Rect, pos2};

/// Insets in physical pixels.
#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct Insets {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

/// `screen` (in points) less `insets` (in pixels, `pixels_per_point` to a
/// point).
pub fn safe_area(screen: Rect, insets: Insets, pixels_per_point: f32) -> Rect {
    let ppp = if pixels_per_point > 0.0 {
        pixels_per_point
    } else {
        1.0
    };
    let area = Rect::from_min_max(
        pos2(
            screen.min.x + insets.left / ppp,
            screen.min.y + insets.top / ppp,
        ),
        pos2(
            screen.max.x - insets.right / ppp,
            screen.max.y - insets.bottom / ppp,
        ),
    );
    if area.is_positive() { area } else { screen }
}
