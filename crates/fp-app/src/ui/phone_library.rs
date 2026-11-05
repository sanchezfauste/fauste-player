//! The phone's Library tab (track library spec §11.1): the category
//! selector (LB38), a category's track list (LB39) and Device music, the
//! phone's Music screen (phone layout spec §9). The open category is the
//! shared `library_view.selection` and the search the library window's;
//! the tab keeps its page, its ticks (LB42) and the bar's target. A row's
//! 🎧 pre-listens on the library CUE, with a slim strip over the list
//! (LB45); leaving the tab stops it. A long press on a row opens its menu
//! (LB43), and the header's ＋ opens Device music to add to the list's
//! category (LB44).

use std::collections::HashSet;

use egui::{
    Align, Align2, Color32, Layout, Rect, Role, Sense, Stroke, StrokeKind, Ui, UiBuilder,
    WidgetInfo, pos2, vec2,
};
use egui_phosphor::regular as icon;
use fp_model::{
    AppState, CartColor, Categories, CategoryId, Command, LibrarySelection, LibraryView, PlayerId,
    PlaylistId, Track, TrackId, Transport,
};

use super::app::{Scene, ViewState};
use super::glyphs::{self, TransportAction};
use super::library::confirm::Removal;
use super::library::menu::{MenuOutcome, MenuScope};
use super::player::paused_play_style;
use super::widgets::{self, TileStyle, font, font_medium};
use super::{format, library, music, theme, touch, track_cells, view};

/// The height of a selector row, for a finger (as the Music screen's
/// group rows).
const ROW: f32 = 48.0;
/// How far each level of the category tree is indented.
const INDENT: f32 = 16.0;
/// The side of a category's colour swatch.
const SWATCH: f32 = 12.0;

/// What the Library tab shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LibraryPage {
    /// *All tracks*, the categories, *Uncategorised* and Device music.
    #[default]
    Selector,
    /// The open category's tracks.
    Tracks,
    /// The device's music (phone layout spec §9).
    Device,
}

/// What the Library tab remembers while the app runs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PhoneLibraryView {
    pub page: LibraryPage,
    /// Ticked library tracks, in tick order (LB42).
    pub ticked: Vec<TrackId>,
    /// Device music opened from a list's ＋: the category its files join
    /// (`Some(None)`: uncategorised). `None`: opened from the selector.
    pub adding_to: Option<Option<CategoryId>>,
    /// The list opened: its header's name, and when its category goes
    /// away the tab returns to the selector (the snapshot's selection falls
    /// back on *All tracks* then, so it cannot tell).
    pub shown: Option<LibrarySelection>,
    /// Where the ticked tracks go: a player and a playlist (LB42, PL19).
    pub target: Option<(PlayerId, PlaylistId)>,
}

impl PhoneLibraryView {
    /// PL2 Back: Device → Tracks (when `adding_to`) or Selector; Tracks →
    /// Selector (ticks cleared); false on the Selector.
    pub fn back(&mut self) -> bool {
        match self.page {
            LibraryPage::Selector => false,
            LibraryPage::Tracks => {
                self.page = LibraryPage::Selector;
                self.ticked.clear();
                self.shown = None;
                true
            }
            LibraryPage::Device => {
                self.page = if self.adding_to.take().is_some() {
                    LibraryPage::Tracks
                } else {
                    LibraryPage::Selector
                };
                true
            }
        }
    }

    /// Opens `selection`'s list: sends `SetLibraryView` with it, clears ticks.
    pub(crate) fn open(&mut self, scene: &Scene<'_>, selection: LibrarySelection) {
        // A phone has no library window, so nothing of its rectangle or
        // tree width waits to reach the snapshot.
        let mut view = scene.state.library_view.clone();
        view.selection = selection;
        scene.ctl.send(Command::SetLibraryView(Box::new(view)));
        self.page = LibraryPage::Tracks;
        self.ticked.clear();
        self.adding_to = None;
        self.shown = Some(selection);
    }

    /// A list whose category no longer exists returns to the selector,
    /// its ticks cleared (LB39, settled details).
    pub fn settle(&mut self, categories: &Categories) {
        if let Some(LibrarySelection::Category(id)) = self.shown
            && categories.get(id).is_none()
        {
            *self = Self::default();
        }
    }
}

/// One row of the selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Row {
    All,
    Category { id: CategoryId, depth: usize },
    Uncategorised,
    Device,
}

/// The Library tab: the page it is on.
pub(crate) fn screen(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState) {
    // A pick in progress holds the tab on Device music (PL19b); the list
    // it returns to settles afterwards.
    if view_state.music.pick.is_some() {
        view_state.phone_library.page = LibraryPage::Device;
    } else {
        view_state.phone_library.settle(&scene.state.categories);
    }
    match view_state.phone_library.page {
        LibraryPage::Selector => selector(ui, scene, view_state),
        LibraryPage::Tracks => tracks_page(ui, scene, view_state),
        LibraryPage::Device => {
            // LB50: a page of the tab has its header; a pick keeps Cancel.
            if view_state.music.pick.is_none()
                && device_header(ui, scene, &view_state.phone_library)
            {
                view_state.phone_library.back();
                return;
            }
            music::screen(ui, scene, view_state);
        }
    }
}

/// LB50: Device music's header: back to where it was opened from (its
/// label says where) and the page's name. True when back is tapped.
fn device_header(ui: &mut Ui, scene: &Scene<'_>, tab: &PhoneLibraryView) -> bool {
    let t = scene.i18n;
    let back = if tab.adding_to.is_some() {
        t.tr("phone-library-back-to-list")
    } else {
        t.tr("phone-library-back")
    };
    let title = t.tr("phone-library-device-music");
    header_bar(ui, &back, &title, None) == Some(HeaderTap::Back)
}

/// *Add to <category>*, or *Add to Uncategorised* (LB44).
pub(crate) fn add_to_category_label(scene: &Scene<'_>, category: Option<CategoryId>) -> String {
    let t = scene.i18n;
    match category.and_then(|id| scene.state.categories.get(id)) {
        Some(c) => t.tr_args(
            "phone-library-add-to-category",
            &[("category", c.name.clone().into())],
        ),
        None => t.tr("phone-library-add-to-uncategorised"),
    }
}

/// The category selector (LB38).
fn selector(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState) {
    library::tree::refresh_counts(&mut view_state.library, scene.state);
    let categories = &scene.state.categories;
    let mut rows = vec![Row::All];
    rows.extend(
        library::menu::tree_order(categories)
            .into_iter()
            .map(|(id, _, depth)| Row::Category { id, depth }),
    );
    rows.extend([Row::Uncategorised, Row::Device]);
    let counts = view_state.library.counts.as_ref().map(|c| &c.counts);
    let mut chosen = None;
    egui::ScrollArea::vertical()
        .id_salt("phone-library-selector")
        .auto_shrink([false, false])
        .show_rows(ui, ROW, rows.len(), |ui, range| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            for row in rows.get(range).unwrap_or_default() {
                let count = counts.map(|c| match row {
                    Row::All => c.all,
                    Row::Uncategorised => c.uncategorised,
                    Row::Category { id, .. } => c.by_category.get(id).copied().unwrap_or(0),
                    Row::Device => 0,
                });
                if selector_row(ui, scene, *row, count) {
                    chosen = Some(*row);
                }
            }
        });
    let opened = match chosen {
        Some(Row::All) => Some(LibrarySelection::All),
        Some(Row::Uncategorised) => Some(LibrarySelection::Uncategorised),
        Some(Row::Category { id, .. }) => Some(LibrarySelection::Category(id)),
        Some(Row::Device) => {
            let tab = &mut view_state.phone_library;
            tab.page = LibraryPage::Device;
            tab.adding_to = None;
            None
        }
        None => None,
    };
    if let Some(selection) = opened {
        view_state.phone_library.open(scene, selection);
        // LB51: a search left in another list must not hide this one.
        view_state.library.search.clear();
    }
}

/// One selector row: its swatch (or Device music's icon), its name and
/// its count; true when tapped.
fn selector_row(ui: &mut Ui, scene: &Scene<'_>, row: Row, count: Option<usize>) -> bool {
    let t = scene.i18n;
    let category = match row {
        Row::Category { id, .. } => scene.state.categories.get(id),
        _ => None,
    };
    let (name, color, depth) = match row {
        Row::All => (t.tr("library-tree-all"), None, 0),
        Row::Uncategorised => (t.tr("library-tree-uncategorised"), None, 0),
        Row::Device => (t.tr("phone-library-device-music"), None, 0),
        Row::Category { depth, .. } => (
            category.map(|c| c.name.clone()).unwrap_or_default(),
            category.and_then(|c| c.color),
            depth,
        ),
    };
    let count = count.filter(|_| row != Row::Device);
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), ROW), Sense::click());
    let label = name.clone();
    response.widget_info(|| {
        let mut info = WidgetInfo::labeled(Role::Button, true, label.clone());
        info.current_text_value = count.map(|c| c.to_string());
        info
    });
    if ui.is_rect_visible(rect) {
        paint_row(
            ui,
            rect,
            response.hovered(),
            row,
            &name,
            color,
            depth,
            count,
        );
    }
    touch::tapped(ui, &response)
}

#[allow(clippy::too_many_arguments)]
fn paint_row(
    ui: &Ui,
    rect: Rect,
    hovered: bool,
    row: Row,
    name: &str,
    color: Option<CartColor>,
    depth: usize,
    count: Option<usize>,
) {
    let painter = ui.painter();
    if hovered {
        painter.rect_filled(rect, 4.0, theme::NEUTRAL_800);
    }
    let left = rect.left() + 14.0 + depth as f32 * INDENT;
    let mark = Rect::from_center_size(
        pos2(left + SWATCH / 2.0, rect.center().y),
        vec2(SWATCH, SWATCH),
    );
    match (row, color) {
        (Row::Device, _) => {
            painter.text(
                mark.center(),
                Align2::CENTER_CENTER,
                icon::MUSIC_NOTES,
                font(18.0),
                theme::NEUTRAL_300,
            );
        }
        (_, Some(c)) => {
            painter.rect_filled(mark, 2.0, theme::cart_color(c));
        }
        (_, None) => {
            painter.rect_stroke(
                mark,
                2.0,
                Stroke::new(1.0, theme::NEUTRAL_700),
                StrokeKind::Inside,
            );
        }
    }
    let mut name_right = rect.right() - 14.0;
    if let Some(count) = count {
        let galley = painter.layout_no_wrap(count.to_string(), font(13.0), theme::NEUTRAL_500);
        let at = pos2(
            rect.right() - 14.0 - galley.size().x,
            rect.center().y - galley.size().y / 2.0,
        );
        name_right = at.x - 8.0;
        painter.galley(at, galley, theme::NEUTRAL_500);
    }
    let name_rect = Rect::from_min_max(
        pos2(mark.right() + 12.0, rect.top()),
        pos2(name_right, rect.bottom()),
    );
    painter
        .with_clip_rect(name_rect.intersect(ui.clip_rect()))
        .text(
            pos2(name_rect.left(), rect.center().y),
            Align2::LEFT_CENTER,
            name,
            font(15.0),
            theme::TEXT,
        );
}

/// Room at a track row's right for its preview button (LB45).
const PREVIEW_ROOM: f32 = 44.0;
/// The preview strip's remaining time.
const STRIP_COUNTDOWN: f32 = 20.0;

/// What a tap on a track row did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RowTap {
    Tick,
    Prelisten,
}

/// Why `track` cannot be pre-listened (the message key of the window's
/// Play tooltip), or `None` when it can: the library CUE needs its route
/// and a playable file (LB32, LB45).
pub(crate) fn prelisten_blocked(state: &AppState, track: TrackId) -> Option<&'static str> {
    if !state.config.outputs.library_has_cue() {
        Some("library-preview-no-route")
    } else if !state.library.is_playable(track) {
        Some("library-preview-not-available")
    } else {
        None
    }
}

/// LB45: pre-listens `track` on the library CUE from its cue-in. A running
/// or held preview switches to it (LB34); a stopped one starts, and the
/// model asks first on a sounding on-air output (LB47). A track that
/// cannot be pre-listened sends nothing.
pub(crate) fn prelisten(scene: &Scene<'_>, track: TrackId) {
    if prelisten_blocked(scene.state, track).is_some() {
        return;
    }
    scene.ctl.send(Command::SelectLibraryCue(Some(track)));
    if scene.state.library_cue.transport == Transport::Stopped {
        scene.ctl.send(Command::PlayLibraryCue);
    }
}

/// LB45: leaving the Library tab (for any other screen, by a tab or by
/// Back) stops the preview. Called once a phone frame, after the screen
/// has settled.
pub(crate) fn follow_screen(scene: &Scene<'_>, view_state: &mut ViewState) {
    let screen = view_state.phone.screen;
    let left = view_state.phone_screen_seen == Some(touch::PhoneScreen::Library)
        && screen != touch::PhoneScreen::Library;
    view_state.phone_screen_seen = Some(screen);
    if left && scene.state.library_cue.transport != Transport::Stopped {
        scene.ctl.send(Command::StopLibraryCue);
    }
}

/// LB45: a window that stops being a phone (a resize, a foldable opened)
/// leaves the Library tab with it, so the preview stops as on
/// `follow_screen`. Called every frame with whether the frame is a phone's.
pub(crate) fn follow_layout(scene: &Scene<'_>, view_state: &mut ViewState, phone: bool) {
    if phone {
        return;
    }
    let left = view_state.phone_screen_seen.take() == Some(touch::PhoneScreen::Library);
    if left && scene.state.library_cue.transport != Transport::Stopped {
        scene.ctl.send(Command::StopLibraryCue);
    }
}

/// The open list (LB39–LB42): the header, the search and the *Whole
/// library* toggle, the rows, and the selection bar while something is
/// ticked.
fn tracks_page(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState) {
    let t = scene.i18n;
    let selection = view_state
        .phone_library
        .shown
        .unwrap_or(scene.state.library_view.selection);
    match header(ui, scene, selection) {
        Some(HeaderTap::Back) => {
            view_state.phone_library.back();
            return;
        }
        // LB44: Device music adds to the list's category.
        Some(HeaderTap::Add) => {
            let view = LibraryView {
                selection,
                ..scene.state.library_view.clone()
            };
            let tab = &mut view_state.phone_library;
            tab.adding_to = Some(library::added_category(&view));
            tab.page = LibraryPage::Device;
            return;
        }
        None => {}
    }
    ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
    let wide = music::is_wide(ui);
    let mut whole = false;
    if wide {
        ui.horizontal(|ui| {
            whole = whole_toggle(ui, scene);
            search_field(ui, scene, view_state);
        });
    } else {
        search_field(ui, scene, view_state);
        ui.horizontal(|ui| whole = whole_toggle(ui, scene));
    }
    if whole {
        let mut view = scene.state.library_view.clone();
        view.whole_library = !view.whole_library;
        scene.ctl.send(Command::SetLibraryView(Box::new(view)));
    }
    preview_strip(ui, scene);
    let rows = view_state.library.rows(scene.state);
    let tab = &mut view_state.phone_library;
    // Ticks the search or the scope hides, or the library lost, are not
    // added (LB42 acts on what the operator sees); the order stays.
    if !tab.ticked.is_empty() {
        let listed: HashSet<TrackId> = rows.iter().copied().collect();
        tab.ticked.retain(|id| listed.contains(id));
    }
    if rows.is_empty() {
        ui.add(
            egui::Label::new(
                egui::RichText::new(t.tr("phone-library-empty"))
                    .font(font(13.0))
                    .color(theme::TEXT),
            )
            .selectable(false)
            .wrap(),
        );
        return;
    }
    let bar = !tab.ticked.is_empty();
    let list_height = if bar {
        (ui.available_height() - music::bar_height(wide)).max(0.0)
    } else {
        ui.available_height()
    };
    let mut toggled = None;
    let mut heard = None;
    let mut held = None;
    let mut outcome = None;
    egui::ScrollArea::vertical()
        .id_salt("phone-library-tracks")
        .max_height(list_height)
        .auto_shrink([false, false])
        .show_rows(ui, music::TRACK_ROW, rows.len(), |ui, range| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            for &id in rows.get(range).unwrap_or_default() {
                let Some(track) = scene.state.library.get(id) else {
                    continue;
                };
                let ticked = tab.ticked.contains(&id);
                let (response, tap) = track_row(ui, scene, track, ticked);
                match tap {
                    Some(RowTap::Tick) => toggled = Some(id),
                    Some(RowTap::Prelisten) => heard = Some(id),
                    None => {}
                }
                // LB43: a long press (egui's secondary click on a touch
                // screen) ticks an unticked row first while others are.
                let joins = !ticked && !tab.ticked.is_empty();
                if response.secondary_clicked() && joins {
                    held = Some(id);
                }
                response.context_menu(|ui| {
                    let targets = menu_targets(&tab.ticked, id);
                    let scope = MenuScope::Phone {
                        ticked: !tab.ticked.is_empty(),
                    };
                    if let Some(o) = library::menu::row_menu(ui, scene, &targets, scope) {
                        outcome = Some(o);
                    }
                });
            }
        });
    if let Some(id) = heard {
        prelisten(scene, id);
    }
    if let Some(id) = held
        && !tab.ticked.contains(&id)
    {
        tab.ticked.push(id);
    }
    if let Some(id) = toggled {
        match tab.ticked.iter().position(|t| *t == id) {
            Some(at) => {
                tab.ticked.remove(at);
            }
            None => tab.ticked.push(id),
        }
    }
    match outcome {
        Some(MenuOutcome::Remove(tracks)) => {
            library::confirm::ask_removal(scene, view_state, Removal::Tracks(tracks));
        }
        Some(MenuOutcome::Notice(text)) => view_state.set_notice(text, scene.time),
        // Only the window's menu edits tags and loudness.
        Some(MenuOutcome::Row(_)) | None => {}
    }
    let tab = &mut view_state.phone_library;
    if !tab.ticked.is_empty() {
        let shown = view_state.phone.shown_player(&scene.state.player_ids());
        let tab = &mut view_state.phone_library;
        let picked = music::Picked::Tracks(&mut tab.ticked);
        music::selection_bar(ui, scene, picked, &mut tab.target, shown, wide);
    }
}

/// LB45, LB49: the slim strip over the list. Its room is always kept, so
/// the rows never move: while the library CUE runs or is held it holds
/// Pause/Play, Stop, the remaining time and the title (no waveform), and
/// otherwise a dim hint.
fn preview_strip(ui: &mut Ui, scene: &Scene<'_>) {
    // Exactly this room, whatever the strip holds.
    let (rect, _) =
        ui.allocate_exact_size(vec2(ui.available_width(), touch::TARGET), Sense::hover());
    let mut strip = ui.new_child(
        UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::left_to_right(Align::Center)),
    );
    strip.set_clip_rect(rect.intersect(ui.clip_rect()));
    {
        let ui = &mut strip;
        ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
        if !strip_controls(ui, scene) {
            ui.add_space(8.0);
            ui.add(
                egui::Label::new(
                    egui::RichText::new(icon::HEADPHONES)
                        .font(font(16.0))
                        .color(theme::NEUTRAL_500),
                )
                .selectable(false),
            );
            ui.add(
                egui::Label::new(
                    egui::RichText::new(scene.i18n.tr("phone-library-strip-idle"))
                        .font(font(13.0))
                        .color(theme::NEUTRAL_500),
                )
                .selectable(false)
                .truncate(),
            );
        }
    }
}

/// The strip's controls while the library CUE runs or is held; false
/// (nothing drawn) while it is stopped.
fn strip_controls(ui: &mut Ui, scene: &Scene<'_>) -> bool {
    let t = scene.i18n;
    let cue = &scene.state.library_cue;
    if cue.transport == Transport::Stopped {
        return false;
    }
    // The engine's position only while it is the strip's track.
    let position = scene
        .telemetry
        .library_cue
        .filter(|tel| Some(tel.track) == cue.track)
        .map(|tel| tel.position_secs);
    let Some(v) = view::library_cue_view(scene.state, position) else {
        return false;
    };
    let paused = v.transport == Transport::Paused;
    let blink = widgets::blink(scene.time);
    let size = vec2(touch::TARGET, touch::TARGET);
    {
        let (label, action, style) = if paused {
            (
                t.tr("library-preview-play"),
                TransportAction::Play,
                paused_play_style(blink),
            )
        } else {
            (
                t.tr("library-preview-pause"),
                TransportAction::Pause,
                TileStyle::plain(),
            )
        };
        let toggle = widgets::tile(ui, size, &label, true, style, |p, r, c| {
            glyphs::paint(p, r, action, 16.0, c)
        });
        if touch::tapped(ui, &toggle) {
            scene.ctl.send(Command::SetLibraryCuePaused(!paused));
        }
        let stop = widgets::tile(
            ui,
            size,
            &t.tr("library-preview-stop"),
            true,
            TileStyle::plain(),
            |p, r, c| glyphs::paint(p, r, TransportAction::Stop, 16.0, c),
        );
        if touch::tapped(ui, &stop) {
            scene.ctl.send(Command::StopLibraryCue);
        }
        widgets::countdown(
            ui,
            v.remaining.unwrap_or(f64::NAN),
            theme::TEXT,
            STRIP_COUNTDOWN,
        );
        ui.add(
            egui::Label::new(
                egui::RichText::new(v.title.clone())
                    .font(font(15.0))
                    .color(theme::TEXT),
            )
            .selectable(false)
            .truncate(),
        );
    }
    true
}

/// What a tap on the list's header asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HeaderTap {
    Back,
    /// ＋: Device music, to add to the list's category (LB44).
    Add,
}

/// What the menu of the row `held` acts on (LB43): the row alone while
/// nothing is ticked, else every ticked row in tick order, with `held`
/// last when it was not ticked.
fn menu_targets(ticked: &[TrackId], held: TrackId) -> Vec<TrackId> {
    let mut targets = ticked.to_vec();
    if !targets.contains(&held) {
        targets.push(held);
    }
    targets
}

/// The list's header: the back button, the list's name and its ＋ (LB44).
fn header(ui: &mut Ui, scene: &Scene<'_>, selection: LibrarySelection) -> Option<HeaderTap> {
    let t = scene.i18n;
    let name = match selection {
        LibrarySelection::All => t.tr("library-tree-all"),
        LibrarySelection::Uncategorised => t.tr("library-tree-uncategorised"),
        LibrarySelection::Category(id) => scene
            .state
            .categories
            .get(id)
            .map(|c| c.name.clone())
            .unwrap_or_default(),
    };
    header_bar(
        ui,
        &t.tr("phone-library-back"),
        &name,
        Some(&t.tr("phone-library-add-from-device")),
    )
}

/// A page's header: a back button labelled `back`, the `title`, and a ＋
/// labelled `add` when given.
fn header_bar(ui: &mut Ui, back: &str, title: &str, add: Option<&str>) -> Option<HeaderTap> {
    let mut tap = None;
    let style = TileStyle {
        border: Color32::TRANSPARENT,
        ..TileStyle::plain()
    };
    let size = vec2(touch::TARGET, touch::TARGET);
    let button = |ui: &mut Ui, label: &str, glyph: &str| {
        let response = widgets::tile(ui, size, label, true, style, |p, r, c| {
            p.text(r.center(), Align2::CENTER_CENTER, glyph, font(18.0), c);
        });
        touch::tapped(ui, &response)
    };
    ui.horizontal(|ui| {
        if button(ui, back, icon::ARROW_LEFT) {
            tap = Some(HeaderTap::Back);
        }
        let room = if add.is_some() {
            touch::TARGET + ui.spacing().item_spacing.x
        } else {
            0.0
        };
        let width = (ui.available_width() - room).max(0.0);
        ui.allocate_ui(vec2(width, touch::TARGET), |ui| {
            ui.add(
                egui::Label::new(
                    egui::RichText::new(title)
                        .font(font_medium(18.0))
                        .color(theme::TEXT),
                )
                .selectable(false)
                .truncate(),
            );
        });
        if let Some(add) = add
            && button(ui, add, icon::PLUS)
        {
            tap = Some(HeaderTap::Add);
        }
    });
    tap
}

/// The search over the list (LB40), the library window's search text.
fn search_field(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState) {
    let hint = scene.i18n.tr("phone-library-search-hint");
    let search = ui.add(
        egui::TextEdit::singleline(&mut view_state.library.search)
            .hint_text(&hint)
            .background_color(theme::NEUTRAL_800)
            .margin(vec2(10.0, 10.0))
            .font(font(15.0))
            .min_size(vec2(0.0, 40.0))
            .desired_width(f32::INFINITY),
    );
    search.widget_info(|| WidgetInfo::labeled(Role::TextInput, true, hint.clone()));
}

/// *Whole library* (LB40); true when tapped.
fn whole_toggle(ui: &mut Ui, scene: &Scene<'_>) -> bool {
    let on = scene.state.library_view.whole_library;
    touch::chip(ui, &scene.i18n.tr("phone-library-whole"), on)
}

/// A library track: a check box, the title (or the file name) over the
/// artist, its status marks (§7.4), the duration and its 🎧 (LB45). Its
/// label joins title, artist and duration. A tap on the 🎧, enabled or
/// not, never ticks the row.
fn track_row(
    ui: &mut Ui,
    scene: &Scene<'_>,
    track: &Track,
    ticked: bool,
) -> (egui::Response, Option<RowTap>) {
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), music::TRACK_ROW), Sense::click());
    let title = if track.title.is_empty() {
        track
            .path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| track.path.to_string_lossy().into_owned())
    } else {
        track.title.clone()
    };
    let duration = (track.duration_secs > 0.0).then(|| format::clock(track.duration_secs));
    let artist = Some(track.artist.as_str()).filter(|a| !a.is_empty());
    let label = music::row_label(&title, artist, duration.as_deref());
    response.widget_info(|| WidgetInfo::selected(Role::CheckBox, true, ticked, &label));
    if !ui.is_rect_visible(rect) {
        let tap = touch::tapped(ui, &response).then_some(RowTap::Tick);
        return (response, tap);
    }
    music::paint_check(ui, rect, ticked);
    let right = rect.right() - PREVIEW_ROOM;
    let marks_right = match &duration {
        Some(d) => music::paint_duration(ui, rect, right, d) - 8.0,
        None => right,
    };
    // The marks are widgets (their names say why), right to left.
    let marks = Rect::from_min_max(
        pos2(rect.left() + 44.0, rect.top()),
        pos2(marks_right, rect.bottom()),
    );
    let mut marks_ui = ui.new_child(
        UiBuilder::new()
            .max_rect(marks)
            .layout(Layout::right_to_left(Align::Center)),
    );
    marks_ui.spacing_mut().item_spacing.x = 4.0;
    track_cells::analysis_flags(&mut marks_ui, scene, track);
    track_cells::file_flag(&mut marks_ui, scene, track);
    let used = marks_ui.min_rect();
    let text_right = if used.width() > 0.0 {
        used.left() - 8.0
    } else {
        marks_right
    };
    music::paint_lines(ui, rect, text_right, &title, artist);
    // Over the row, so it takes its own taps.
    let button = Rect::from_min_max(pos2(rect.right() - touch::TARGET, rect.top()), rect.max);
    if preview_button(ui, scene, track, &title, button) {
        return (response, Some(RowTap::Prelisten));
    }
    let on_button = response
        .interact_pointer_pos()
        .is_some_and(|p| button.contains(p));
    // The finger that held the row up for its menu lifts as a tap; a tap
    // that closes a menu ticks nothing either.
    let menu_open = egui::Popup::is_any_open(ui.ctx());
    let tap = (touch::tapped(ui, &response) && !on_button && !menu_open).then_some(RowTap::Tick);
    (response, tap)
}

/// A row's 🎧 in `rect`: dimmed, with the window's reason in its tooltip,
/// when the track cannot be pre-listened; true when tapped.
fn preview_button(ui: &mut Ui, scene: &Scene<'_>, track: &Track, title: &str, rect: Rect) -> bool {
    let t = scene.i18n;
    let blocked = prelisten_blocked(scene.state, track.id);
    let label = t.tr_args("phone-library-prelisten", &[("title", title.into())]);
    let tip = blocked.map_or_else(|| label.clone(), |key| t.tr(key));
    let style = TileStyle {
        border: Color32::TRANSPARENT,
        ..TileStyle::plain()
    };
    let mut child = ui.new_child(UiBuilder::new().max_rect(rect));
    let response = widgets::tile_without_tooltip(
        &mut child,
        rect.size(),
        &label,
        blocked.is_none(),
        style,
        |p, r, c| {
            p.text(
                r.center(),
                Align2::CENTER_CENTER,
                icon::HEADPHONES,
                font(20.0),
                c,
            );
        },
    )
    .on_hover_text(tip);
    touch::tapped(&child, &response)
}
