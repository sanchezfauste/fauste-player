//! The touch layout: the form factor from the screen size and what a phone
//! shows (phone layout spec). A tablet keeps the desktop layout (mobile
//! spec §6).

use std::collections::HashMap;

use egui::{Align, Layout, Rect, TextureHandle, Theme, Ui, UiBuilder, Vec2, vec2};
use fp_model::{Command, PlayerId, TrackId};

use super::app::{Scene, ViewState};
use super::panel_layout::PANEL_GAP;
use super::theme;
use super::widgets::{self, TileStyle, font};
use egui_phosphor::regular as icon;

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

/// The phone's screens, chosen in the top bar (PL1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PhoneScreen {
    /// Every player's card and the cartwall.
    #[default]
    Live,
    /// The cartwall alone.
    Carts,
    /// One player with its playlists.
    Playlists,
    /// The device's music (§9).
    Music,
}

/// What the phone shows: the screen and the player the Playlists screen
/// shows (the first when none).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PhoneView {
    pub screen: PhoneScreen,
    pub playlists: Option<PlayerId>,
}

impl PhoneView {
    /// Follows the players there are: a chosen player that went away
    /// leaves the choice to the first.
    pub fn settle(&mut self, players: &[PlayerId]) {
        if self.playlists.is_some_and(|id| !players.contains(&id)) {
            self.playlists = None;
        }
    }

    /// The player the Playlists screen shows.
    pub fn shown_player(&self, players: &[PlayerId]) -> Option<PlayerId> {
        self.playlists
            .filter(|id| players.contains(id))
            .or_else(|| players.first().copied())
    }

    /// Back from another screen goes to Live (PL2); false on Live.
    pub fn back(&mut self) -> bool {
        let away = self.screen != PhoneScreen::Live;
        self.screen = PhoneScreen::Live;
        away
    }
}

/// How far a sideways drag must go to be a swipe, in points.
const SWIPE_DISTANCE: f32 = 80.0;

/// The page a drag of `delta` asks for: the next one (`true`, a drag to
/// the left), the previous one, or none when it is short or not sideways.
pub fn swipe_direction(delta: Vec2) -> Option<bool> {
    (delta.x.abs() > SWIPE_DISTANCE && delta.x.abs() > 2.0 * delta.y.abs()).then_some(delta.x < 0.0)
}

/// How far from a widget a finger still hits it: the desktop's tiles are
/// smaller than a fingertip.
const INTERACT_RADIUS: f32 = 12.0;
/// The Live screen's share for the cards: of the height in portrait (less
/// when the cards need less), of the width in landscape (PL5, PL6).
const LIVE_CARDS_SHARE: f32 = 0.66;
/// Scroll bars a finger can drag.
const SCROLL_BAR_WIDTH: f32 = 12.0;
/// The phone's top bar, tall enough for its screen tiles.
pub(crate) const PHONE_TOP_BAR_HEIGHT: f32 = 56.0;
/// Below this width of the top bar a phone leaves the clock out.
pub(crate) const CLOCK_MIN_WIDTH: f32 = 400.0;
/// The size of a target a finger uses (spec §6: 48 dp).
pub(crate) const TARGET: f32 = 48.0;

/// The style touch mode adds to the theme's.
pub(crate) fn apply_style(ctx: &egui::Context) {
    ctx.style_mut_of(Theme::Dark, |style| {
        style.interaction.interact_radius = INTERACT_RADIUS;
        style.spacing.scroll.bar_width = SCROLL_BAR_WIDTH;
    });
}

/// The phone's tabs: the screen, its name's message and its icon (PL26).
const TABS: [(PhoneScreen, &str, &str); 4] = [
    (PhoneScreen::Live, "touch-live", icon::BROADCAST),
    (PhoneScreen::Carts, "touch-carts", icon::SQUARES_FOUR),
    (PhoneScreen::Playlists, "touch-playlists", icon::PLAYLIST),
    (PhoneScreen::Music, "touch-music", icon::MUSIC_NOTES),
];

/// The gap between the top bar's tiles.
const TAB_GAP: f32 = 4.0;

/// The phone's top bar, left part, stopping before `right` (where the lock
/// and More begin): the four tabs as icons, the chosen one with its name
/// when there is room (PL26), and in landscape on the Playlists screen the
/// player chips after them (PL27).
pub(crate) fn screen_tiles(
    ui: &mut Ui,
    scene: &Scene<'_>,
    phone: &mut PhoneView,
    right: f32,
    landscape: bool,
) {
    let t = scene.i18n;
    ui.spacing_mut().item_spacing = vec2(TAB_GAP, 0.0);
    let players = scene.state.player_ids();
    let chips = landscape && phone.screen == PhoneScreen::Playlists && !players.is_empty();
    let icons = TABS.len() as f32 * (TARGET + TAB_GAP);
    // The chips keep room for two of them before the name takes any.
    let kept = if chips {
        2.0 * (TARGET + TAB_GAP) + 12.0
    } else {
        0.0
    };
    let room = right - ui.cursor().left() - icons - kept;
    for (screen, key, glyph) in TABS {
        let name = t.tr(key);
        let on = phone.screen == screen;
        let name_width = text_width(ui, &name);
        let named = on && name_width + 8.0 <= room;
        if tab(ui, glyph, &name, on, named.then_some(name_width)) {
            phone.screen = screen;
        }
    }
    if chips {
        let bar = ui.max_rect();
        let left = ui.cursor().left() + 12.0;
        let rect = Rect::from_min_max(
            egui::pos2(left, bar.center().y - TARGET / 2.0),
            egui::pos2(right.max(left), bar.center().y + TARGET / 2.0),
        );
        let mut chips_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(rect)
                .layout(Layout::top_down(Align::Min)),
        );
        player_chips(&mut chips_ui, scene, phone, &players);
    }
}

/// The width of `text` in a tab's font.
fn text_width(ui: &Ui, text: &str) -> f32 {
    ui.painter()
        .layout_no_wrap(text.to_owned(), font(13.0), theme::TEXT)
        .size()
        .x
}

/// One tab: its icon, and its name beside it when `named` (the name's
/// width); the name is its accessible label either way.
fn tab(ui: &mut Ui, glyph: &str, name: &str, on: bool, named: Option<f32>) -> bool {
    let width = named.map_or(TARGET, |w| TARGET + w + 4.0);
    let glyph = glyph.to_owned();
    let text = named.map(|_| name.to_owned());
    widgets::tile_without_tooltip(
        ui,
        vec2(width, TARGET),
        name,
        true,
        chip_style(on),
        move |p, r, c| {
            let icon = p.layout_no_wrap(glyph, egui::FontId::proportional(20.0), c);
            let icon_x = r.left() + (TARGET - icon.size().x) / 2.0;
            let icon_y = r.center().y - icon.size().y / 2.0;
            p.galley(egui::pos2(icon_x, icon_y), icon, c);
            if let Some(text) = text {
                let galley = p.layout_no_wrap(text, font(13.0), c);
                let y = r.center().y - galley.size().y / 2.0;
                p.galley(egui::pos2(r.left() + TARGET - 6.0, y), galley, c);
            }
        },
    )
    .clicked()
}

/// The player chips (P1, P2…) of the Playlists screen, scrolling sideways
/// when they do not fit (PL8, PL27).
fn player_chips(ui: &mut Ui, scene: &Scene<'_>, phone: &mut PhoneView, players: &[PlayerId]) {
    let Some(shown) = phone.shown_player(players) else {
        return;
    };
    egui::ScrollArea::horizontal()
        .id_salt("phone-chips")
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = vec2(TAB_GAP, 0.0);
                for (index, id) in players.iter().enumerate() {
                    if chip(ui, &player_name(scene, index), *id == shown) {
                        phone.playlists = Some(*id);
                    }
                }
            });
        });
}

/// Whether the phone's lock holds faders and waveforms (PL12). It guards
/// touches on a phone only: a tablet and the desktop have no lock button.
pub(crate) fn locked(scene: &Scene<'_>, view_state: &ViewState) -> bool {
    view_state.phone_active && scene.state.layout.touch_locked
}

/// The top bar's lock tile (PL12, PL13): a closed padlock, lit, while
/// locked; an open one otherwise. A tap toggles it.
pub(crate) fn lock_tile(ui: &mut Ui, scene: &Scene<'_>) {
    let on = scene.state.layout.touch_locked;
    let (label, glyph) = if on {
        ("tip-touch-unlock", egui_phosphor::regular::LOCK_SIMPLE)
    } else {
        ("tip-touch-lock", egui_phosphor::regular::LOCK_SIMPLE_OPEN)
    };
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
    let label = scene.i18n.tr(label);
    if widgets::tile(ui, vec2(TARGET, TARGET), &label, true, style, |p, r, c| {
        p.text(
            r.center(),
            egui::Align2::CENTER_CENTER,
            glyph,
            font(20.0),
            c,
        );
    })
    .clicked()
    {
        scene.ctl.send(Command::SetTouchLocked(!on));
    }
}

/// The top bar's More tile (⋮) and its menu: Settings and About (PL1).
pub(crate) fn more_menu(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState) {
    let t = scene.i18n;
    let style = TileStyle {
        border: theme::NEUTRAL_800,
        hover_fill: theme::NEUTRAL_800,
        ..TileStyle::plain()
    };
    let response = widgets::tile(
        ui,
        vec2(TARGET, TARGET),
        &t.tr("touch-more"),
        true,
        style,
        |p, r, c| {
            p.text(
                r.center(),
                egui::Align2::CENTER_CENTER,
                egui_phosphor::regular::DOTS_THREE_VERTICAL,
                font(20.0),
                c,
            );
        },
    );
    egui::Popup::menu(&response).show(|ui| {
        ui.set_min_width(180.0);
        for (key, open) in [
            ("top-settings", &mut view_state.settings_open),
            ("touch-about", &mut view_state.about_open),
        ] {
            let item = egui::Button::new(egui::RichText::new(t.tr(key)).font(font(15.0)))
                .min_size(vec2(180.0, TARGET));
            if ui.add(item).clicked() {
                *open = true;
                ui.close();
            }
        }
    });
}

/// A tile that is lit while `on`; true when tapped.
pub(crate) fn chip(ui: &mut Ui, label: &str, on: bool) -> bool {
    chip_at_most(ui, label, on, f32::INFINITY)
}

/// [`chip`] at most `max_width` wide, its text cut short to fit.
fn chip_at_most(ui: &mut Ui, label: &str, on: bool, max_width: f32) -> bool {
    let natural = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font(13.0), theme::TEXT)
        .size()
        .x
        + 28.0;
    let width = natural.min(max_width);
    let style = chip_style(on);
    let owned = label.to_owned();
    widgets::tile_without_tooltip(
        ui,
        vec2(width, TARGET),
        label,
        true,
        style,
        move |p, r, c| {
            let mut job = egui::text::LayoutJob::simple_singleline(owned, font(13.0), c);
            job.wrap = egui::text::TextWrapping::truncate_at_width((r.width() - 12.0).max(0.0));
            let galley = p.layout_job(job);
            p.galley(r.center() - galley.size() / 2.0, galley, c);
        },
    )
    .clicked()
}

/// A chip's look: highlighted when chosen.
fn chip_style(on: bool) -> TileStyle {
    if on {
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
    }
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
    super::music::follow_request(view_state);
    view_state.player_rects.clear();
    match view_state.phone.screen {
        PhoneScreen::Live => live(ui, scene, view_state, covers, &players, landscape),
        PhoneScreen::Carts => carts(ui, scene, view_state),
        PhoneScreen::Playlists => playlists(ui, scene, view_state, covers, &players, landscape),
        PhoneScreen::Music => super::music::screen(ui, scene, view_state),
    }
}

fn player_name(scene: &Scene<'_>, index: usize) -> String {
    scene
        .i18n
        .tr_args("player-label", &[("n", (index + 1).into())])
}

/// Every player's card and the cartwall (PL5, PL6): in portrait the cards
/// on top, scrolling on their own, and the cartwall below; in landscape the
/// cards on the left and the cartwall on the right. A folded cartwall is
/// its header across the bottom (PL35).
fn live(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    covers: &mut HashMap<TrackId, TextureHandle>,
    players: &[PlayerId],
    landscape: bool,
) {
    let area = ui.available_rect_before_wrap();
    let used_id = ui.scope_id().with("live-cards-height");
    let (cards, carts) = if !scene.state.cartwall.open {
        // PL35: folded, the cartwall's header at the bottom, the players
        // over the rest.
        let header = super::cartwall::height(scene);
        let top = area.bottom() - header;
        (
            Rect::from_min_max(area.min, egui::pos2(area.right(), top - PANEL_GAP)),
            Rect::from_min_max(egui::pos2(area.left(), top), area.max),
        )
    } else if landscape {
        let width = area.width() * LIVE_CARDS_SHARE;
        (
            Rect::from_min_size(area.min, vec2(width, area.height())),
            Rect::from_min_max(
                egui::pos2(area.left() + width + PANEL_GAP, area.top()),
                area.max,
            ),
        )
    } else {
        // As tall as the cards were in the last frame, up to the share.
        let used = ui
            .data(|d| d.get_temp::<f32>(used_id))
            .unwrap_or(f32::INFINITY);
        let height = (area.height() * LIVE_CARDS_SHARE).min(used);
        (
            Rect::from_min_size(area.min, vec2(area.width(), height)),
            Rect::from_min_max(
                egui::pos2(area.left(), area.top() + height + PANEL_GAP),
                area.max,
            ),
        )
    };
    let mut cards_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(cards)
            .layout(Layout::top_down(Align::Min)),
    );
    let output = egui::ScrollArea::vertical()
        .id_salt("live-cards")
        .auto_shrink([false, false])
        .show(&mut cards_ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, PANEL_GAP);
            for (index, id) in players.iter().enumerate() {
                ui.push_id(("live-card", id.0), |ui| {
                    super::player::card(ui, scene, view_state, covers, index);
                });
            }
        });
    ui.data_mut(|d| d.insert_temp(used_id, output.content_size.y));
    let mut carts_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(carts)
            .layout(Layout::top_down(Align::Min)),
    );
    carts_ui.set_clip_rect(carts.intersect(ui.clip_rect()));
    super::cartwall::strip(&mut carts_ui, scene, view_state, true);
    ui.allocate_rect(area, egui::Sense::hover());
}

/// The cartwall full screen (PL7); a sideways swipe shows the next or the
/// previous page, stopping at the ends.
fn carts(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState) {
    let area = ui.available_rect_before_wrap();
    if let Some(forward) = page_swipe(ui, area) {
        let wall = &scene.state.cartwall;
        let shown = wall.shown_page().map(|p| p.id);
        let at = wall.pages.iter().position(|p| Some(p.id) == shown);
        let to = at.and_then(|i| {
            if forward {
                wall.pages.get(i + 1)
            } else {
                i.checked_sub(1).and_then(|i| wall.pages.get(i))
            }
        });
        if let Some(page) = to {
            scene.ctl.send(Command::ShowCartPage(page.id));
        }
    }
    super::cartwall::strip(ui, scene, view_state, true);
}

/// A sideways swipe that started in `area` on this layer (not in a menu or
/// a window over it), when it is let go: the next page (`true`) or the
/// previous one. Carts fire on a tap, so a swipe may start on a cart; a
/// drag some widget took (the page tabs scrolling) is not a swipe.
fn page_swipe(ui: &Ui, area: Rect) -> Option<bool> {
    let ctx = ui.ctx();
    let origin_id = ui.scope_id().with("page-swipe");
    if ctx.dragged_id().is_some() || ctx.drag_stopped_id().is_some() {
        ctx.data_mut(|d| d.remove_temp::<egui::Pos2>(origin_id));
    }
    let (pressed, released, position) = ctx.input(|i| {
        (
            i.pointer.primary_pressed(),
            i.pointer.primary_released(),
            i.pointer.interact_pos(),
        )
    });
    if pressed {
        let here =
            position.filter(|p| area.contains(*p) && ctx.layer_id_at(*p) == Some(ui.layer_id()));
        ctx.data_mut(|d| {
            if let Some(p) = here {
                d.insert_temp(origin_id, p);
            } else {
                d.remove_temp::<egui::Pos2>(origin_id);
            }
        });
    }
    if !released {
        return None;
    }
    let origin = ctx.data_mut(|d| d.remove_temp::<egui::Pos2>(origin_id));
    origin
        .zip(position)
        .and_then(|(from, to)| swipe_direction(to - from))
}

/// The Playlists screen (PL8, PL9): a chip per player on top, then the
/// chosen player's playlists and card.
fn playlists(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    covers: &mut HashMap<TrackId, TextureHandle>,
    players: &[PlayerId],
    landscape: bool,
) {
    let Some(shown) = view_state.phone.shown_player(players) else {
        return;
    };
    // In landscape the chips are in the top bar (PL27).
    if !landscape {
        ui.horizontal(|ui| player_chips(ui, scene, &mut view_state.phone, players));
        ui.add_space(PANEL_GAP);
    }
    let Ok(index) = scene.state.player_index(shown) else {
        return;
    };
    super::player::playlists_screen(ui, scene, view_state, covers, index, landscape);
}
