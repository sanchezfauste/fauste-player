//! One icon per transport action (operator feedback 2, O18).
//!
//! The player, the cartwall and the track table draw their transport icons
//! only through this module (the CUE window will too), so the same action
//! looks the same everywhere.
//! An icon is either a Phosphor font glyph or a shape drawn by
//! [`super::icons`] (the actions Phosphor has no icon for).

use egui::{Color32, Painter, Rect, Shape, Vec2, vec2};

use super::{icons, widgets};

/// What a transport button does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportAction {
    Play,
    /// The play button while a player is on air: fade to the next entry.
    Next,
    Pause,
    Stop,
    FadeStop,
    StopAfter,
    Restart,
    Previous,
    Cue,
}

impl TransportAction {
    pub const ALL: [TransportAction; 9] = [
        Self::Play,
        Self::Next,
        Self::Pause,
        Self::Stop,
        Self::FadeStop,
        Self::StopAfter,
        Self::Restart,
        Self::Previous,
        Self::Cue,
    ];
}

/// A drawing function of [`icons`].
pub type DrawFn = fn(Rect, Color32) -> Vec<Shape>;

/// A drawn icon and the size the transport grid draws it at.
#[derive(Clone, Copy)]
pub struct Drawn {
    pub draw: DrawFn,
    pub size: Vec2,
}

/// The font glyph of an action and whether it is the filled variant, or
/// `None` when the action is a drawn icon.
pub fn font_glyph(action: TransportAction) -> Option<(&'static str, bool)> {
    use egui_phosphor::{fill, regular};
    match action {
        TransportAction::Play => Some((fill::PLAY, true)),
        TransportAction::Next => Some((fill::FAST_FORWARD, true)),
        TransportAction::Pause => Some((fill::PAUSE, true)),
        TransportAction::Stop => Some((fill::STOP, true)),
        TransportAction::Cue => Some((regular::HEADPHONES, false)),
        TransportAction::FadeStop
        | TransportAction::StopAfter
        | TransportAction::Restart
        | TransportAction::Previous => None,
    }
}

/// The drawn icon of an action, or `None` when it is a font glyph.
pub fn drawn(action: TransportAction) -> Option<Drawn> {
    let (draw, w, h): (DrawFn, f32, f32) = match action {
        TransportAction::FadeStop => (icons::fade_stop, 16.0, 12.0),
        TransportAction::StopAfter => (icons::stop_after, 18.0, 13.0),
        TransportAction::Restart => (icons::restart, 14.0, 12.0),
        TransportAction::Previous => (icons::previous, 16.0, 12.0),
        TransportAction::Play
        | TransportAction::Next
        | TransportAction::Pause
        | TransportAction::Stop
        | TransportAction::Cue => return None,
    };
    Some(Drawn {
        draw,
        size: vec2(w, h),
    })
}

/// The glyph as text, for menu items and labels; empty for a drawn icon.
pub fn glyph_text(action: TransportAction) -> &'static str {
    font_glyph(action).map_or("", |(glyph, _)| glyph)
}

/// Paints the icon of `action` centred in `rect`: a font glyph at `size`
/// points, or a drawn icon at its natural size.
pub fn paint(painter: &Painter, rect: Rect, action: TransportAction, size: f32, color: Color32) {
    if let Some((glyph, fill)) = font_glyph(action) {
        widgets::glyph(painter, rect, glyph, size, color, fill);
    } else if let Some(d) = drawn(action) {
        painter.extend((d.draw)(
            Rect::from_center_size(rect.center(), d.size),
            color,
        ));
    }
}
