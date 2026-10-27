//! The touch layout (mobile spec §6): the form factor from the screen size
//! and what a phone shows. A tablet keeps the desktop layout.

use egui::{Theme, Ui, Vec2, vec2};
use fp_model::PlayerId;

use super::app::Scene;
use super::theme;
use super::widgets::{self, TileStyle, font};

/// The shortest side, in points (Android's dp), from which a screen is a
/// tablet (spec §6).
const TABLET_SIDE: f32 = 600.0;

/// How much of the screen there is to lay out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormFactor {
    Phone { landscape: bool },
    Tablet,
}

pub fn form_factor(size: Vec2) -> FormFactor {
    if size.x.min(size.y) >= TABLET_SIDE {
        FormFactor::Tablet
    } else {
        FormFactor::Phone {
            landscape: size.x > size.y,
        }
    }
}

/// The phone's screens, chosen in the top bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PhoneScreen {
    #[default]
    Players,
    Cartwall,
}

/// What the phone shows: the screen, the player open on it (all players
/// when none) and, in landscape, which pair of players.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PhoneView {
    pub screen: PhoneScreen,
    pub open: Option<PlayerId>,
    pub pair: usize,
}

impl PhoneView {
    /// Follows the players there are: an open player that went away closes,
    /// and the pair stays within the players.
    pub fn settle(&mut self, players: &[PlayerId]) {
        if self.open.is_some_and(|id| !players.contains(&id)) {
            self.open = None;
        }
        self.pair = self.pair.min(Self::pairs(players.len()).saturating_sub(1));
    }

    /// How many pairs `count` players make (the last may hold one).
    pub fn pairs(count: usize) -> usize {
        count.div_ceil(2)
    }

    /// The players of the shown pair.
    pub fn pair_players(&self, players: &[PlayerId]) -> Vec<PlayerId> {
        players
            .iter()
            .skip(self.pair * 2)
            .take(2)
            .copied()
            .collect()
    }

    /// The next (`forward`) or the previous pair, if there is one.
    pub fn swipe(&mut self, forward: bool, count: usize) {
        let last = Self::pairs(count).saturating_sub(1);
        self.pair = if forward {
            (self.pair + 1).min(last)
        } else {
            self.pair.saturating_sub(1)
        };
    }
}

/// How far from a widget a finger still hits it: the desktop's tiles are
/// smaller than a fingertip.
const INTERACT_RADIUS: f32 = 12.0;
/// Scroll bars a finger can drag.
const SCROLL_BAR_WIDTH: f32 = 12.0;
/// The phone's top bar, tall enough for its screen tiles.
pub(crate) const PHONE_TOP_BAR_HEIGHT: f32 = 48.0;
/// The height of a target a finger uses.
pub(crate) const TARGET: f32 = 40.0;

/// The style touch mode adds to the theme's.
pub(crate) fn apply_style(ctx: &egui::Context) {
    ctx.style_mut_of(Theme::Dark, |style| {
        style.interaction.interact_radius = INTERACT_RADIUS;
        style.spacing.scroll.bar_width = SCROLL_BAR_WIDTH;
    });
}

/// The phone's top bar, left part: Players | Cartwall.
pub(crate) fn screen_tiles(ui: &mut Ui, scene: &Scene<'_>, phone: &mut PhoneView) {
    let t = scene.i18n;
    ui.spacing_mut().item_spacing = vec2(4.0, 0.0);
    for (screen, key) in [
        (PhoneScreen::Players, "touch-players"),
        (PhoneScreen::Cartwall, "touch-cartwall"),
    ] {
        if chip(ui, &t.tr(key), phone.screen == screen) {
            phone.screen = screen;
        }
    }
}

/// A tile that is lit while `on`; true when tapped.
pub(crate) fn chip(ui: &mut Ui, label: &str, on: bool) -> bool {
    let width = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font(13.0), theme::TEXT)
        .size()
        .x
        + 28.0;
    let style = if on {
        TileStyle {
            fill: theme::NEUTRAL_800,
            border: theme::NEUTRAL_700,
            content: theme::TEXT,
            hover_fill: theme::NEUTRAL_700,
            ..TileStyle::plain()
        }
    } else {
        TileStyle {
            border: theme::NEUTRAL_800,
            content: theme::NEUTRAL_400,
            hover_fill: theme::NEUTRAL_800,
            ..TileStyle::plain()
        }
    };
    widgets::tile_without_tooltip(ui, vec2(width, TARGET), label, true, style, |p, r, c| {
        p.text(
            r.center(),
            egui::Align2::CENTER_CENTER,
            label,
            font(13.0),
            c,
        );
    })
    .clicked()
}
