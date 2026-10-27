//! The touch layout (mobile spec §6): the form factor from the screen size
//! and what a phone shows. A tablet keeps the desktop layout.

use std::collections::HashMap;

use egui::{Align, Layout, Rect, TextureHandle, Theme, Ui, UiBuilder, Vec2, vec2};
use fp_model::{PlayerId, TrackId};

use super::app::{Scene, ViewState};
use super::panel_layout::PANEL_GAP;
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

/// How far a sideways drag must go to be a swipe, in points.
const SWIPE_DISTANCE: f32 = 80.0;

/// The pair a drag of `delta` asks for: the next one (`true`, a drag to
/// the left), the previous one, or none when it is short or not sideways.
pub fn swipe_direction(delta: Vec2) -> Option<bool> {
    (delta.x.abs() > SWIPE_DISTANCE && delta.x.abs() > 2.0 * delta.y.abs()).then_some(delta.x < 0.0)
}

/// A drag across the landscape view, followed until it is let go.
#[derive(Debug, Default)]
pub(crate) struct SwipeTracker {
    origin: Option<egui::Pos2>,
    /// A widget took the drag (a fader, a row): it is not a swipe.
    blocked: bool,
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

/// A phone's middle area: the screen chosen in the top bar.
pub(crate) fn phone_middle(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    covers: &mut HashMap<TrackId, TextureHandle>,
    landscape: bool,
) {
    let players = scene.state.player_ids();
    view_state.phone.settle(&players);
    view_state.player_rects.clear();
    match (view_state.phone.screen, view_state.phone.open) {
        (PhoneScreen::Cartwall, _) => {
            super::cartwall::strip(ui, scene, view_state, true);
        }
        (PhoneScreen::Players, Some(open)) => {
            one_player(ui, scene, view_state, covers, &players, open);
        }
        (PhoneScreen::Players, None) if landscape => {
            pairs(ui, scene, view_state, covers, &players);
        }
        (PhoneScreen::Players, None) => {
            all_players(ui, scene, view_state, covers, &players);
        }
    }
}

/// Landscape: two players side by side, under chips for the pairs and for
/// opening a shown player; a swipe shows the next or previous pair.
fn pairs(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    covers: &mut HashMap<TrackId, TextureHandle>,
    players: &[PlayerId],
) {
    follow_swipe(ui.ctx(), view_state, players.len());
    let shown = view_state.phone.pair_players(players);
    ui.horizontal(|ui| {
        egui::ScrollArea::horizontal()
            .id_salt("phone-pairs")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = vec2(4.0, 0.0);
                    for pair in 0..PhoneView::pairs(players.len()) {
                        let first = pair * 2;
                        let label = if first + 1 < players.len() {
                            format!(
                                "{}–{}",
                                player_name(scene, first),
                                player_name(scene, first + 1)
                            )
                        } else {
                            player_name(scene, first)
                        };
                        if chip(ui, &label, pair == view_state.phone.pair) {
                            view_state.phone.pair = pair;
                        }
                    }
                    ui.add_space(12.0);
                    for id in &shown {
                        let Ok(index) = scene.state.player_index(*id) else {
                            continue;
                        };
                        let label = scene.i18n.tr_args(
                            "touch-open-player",
                            &[("player", player_name(scene, index).into())],
                        );
                        if chip(ui, &label, false) {
                            view_state.phone.open = Some(*id);
                        }
                    }
                });
            });
    });
    ui.add_space(PANEL_GAP);
    let shown: Vec<(PlayerId, usize)> = shown
        .iter()
        .filter_map(|id| scene.state.player_index(*id).ok().map(|i| (*id, i)))
        .collect();
    let rect = ui.available_rect_before_wrap();
    columns(ui, scene, view_state, covers, &shown, rect);
}

/// Turns a sideways drag that no widget took into a change of pair.
fn follow_swipe(ctx: &egui::Context, view_state: &mut ViewState, count: usize) {
    let (pressed, released, position) = ctx.input(|i| {
        (
            i.pointer.primary_pressed(),
            i.pointer.primary_released(),
            i.pointer.interact_pos(),
        )
    });
    let tracker = &mut view_state.phone_swipe;
    if pressed {
        *tracker = SwipeTracker {
            origin: position,
            blocked: false,
        };
    }
    if ctx.dragged_id().is_some() {
        tracker.blocked = true;
    }
    if released {
        let swipe = tracker
            .origin
            .zip(position)
            .filter(|_| !tracker.blocked)
            .and_then(|(from, to)| swipe_direction(to - from));
        *tracker = SwipeTracker::default();
        if let Some(forward) = swipe {
            view_state.phone.swipe(forward, count);
        }
    }
}

fn player_name(scene: &Scene<'_>, index: usize) -> String {
    scene
        .i18n
        .tr_args("player-label", &[("n", (index + 1).into())])
}

/// Every player as a compact panel, one under another, each with a tile
/// that opens it.
fn all_players(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    covers: &mut HashMap<TrackId, TextureHandle>,
    players: &[PlayerId],
) {
    egui::ScrollArea::vertical()
        .id_salt("phone-players")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, PANEL_GAP);
            for (index, id) in players.iter().enumerate() {
                ui.push_id(("phone-player", id.0), |ui| {
                    super::player::compact(ui, scene, view_state, covers, index);
                    let name = player_name(scene, index);
                    let label = scene
                        .i18n
                        .tr_args("touch-open-player", &[("player", name.into())]);
                    if chip(ui, &label, false) {
                        view_state.phone.open = Some(*id);
                    }
                });
            }
        });
}

/// One player with its playlist, under a row of chips: All, then a chip per
/// player.
fn one_player(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    covers: &mut HashMap<TrackId, TextureHandle>,
    players: &[PlayerId],
    open: PlayerId,
) {
    ui.horizontal(|ui| {
        egui::ScrollArea::horizontal()
            .id_salt("phone-chips")
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = vec2(4.0, 0.0);
                    if chip(ui, &scene.i18n.tr("touch-all-players"), false) {
                        view_state.phone.open = None;
                    }
                    for (index, id) in players.iter().enumerate() {
                        if chip(ui, &player_name(scene, index), *id == open) {
                            view_state.phone.open = Some(*id);
                        }
                    }
                });
            });
    });
    ui.add_space(PANEL_GAP);
    let Ok(index) = scene.state.player_index(open) else {
        return;
    };
    let rect = ui.available_rect_before_wrap();
    columns(ui, scene, view_state, covers, &[(open, index)], rect);
}

/// Desktop player columns side by side in `rect`.
fn columns(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    covers: &mut HashMap<TrackId, TextureHandle>,
    shown: &[(PlayerId, usize)],
    rect: Rect,
) {
    let count = shown.len().max(1) as f32;
    let width = (rect.width() - PANEL_GAP * (count - 1.0)) / count;
    for (k, (id, index)) in shown.iter().enumerate() {
        let left = rect.left() + k as f32 * (width + PANEL_GAP);
        let column = Rect::from_min_size(egui::pos2(left, rect.top()), vec2(width, rect.height()));
        view_state.player_rects.push((*id, column));
        let mut column_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(column)
                .id_salt(("player", id.0))
                .layout(Layout::top_down(Align::Min)),
        );
        column_ui.set_clip_rect(column.intersect(ui.clip_rect()));
        super::player::column(&mut column_ui, scene, view_state, covers, *index);
    }
    ui.allocate_rect(rect, egui::Sense::hover());
}
