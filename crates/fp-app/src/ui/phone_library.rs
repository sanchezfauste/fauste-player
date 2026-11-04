//! The phone's Library tab (track library spec §11.1): the category
//! selector (LB38), a category's track list (LB39) and Device music, the
//! phone's Music screen (phone layout spec §9). The open category is the
//! shared `library_view.selection` and the search the library window's;
//! the tab keeps its page, its ticks (LB42) and the bar's target.

use std::collections::HashSet;

use egui::{
    Align, Align2, Color32, Layout, Rect, Role, Sense, Stroke, StrokeKind, Ui, UiBuilder,
    WidgetInfo, pos2, vec2,
};
use egui_phosphor::regular as icon;
use fp_model::{
    CartColor, Categories, CategoryId, Command, LibrarySelection, PlayerId, PlaylistId, Track,
    TrackId,
};

use super::app::{Scene, ViewState};
use super::widgets::{self, TileStyle, font, font_medium};
use super::{format, library, music, theme, touch, track_cells};

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
        LibraryPage::Device => music::screen(ui, scene, view_state),
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
    let tab = &mut view_state.phone_library;
    match chosen {
        Some(Row::All) => tab.open(scene, LibrarySelection::All),
        Some(Row::Uncategorised) => tab.open(scene, LibrarySelection::Uncategorised),
        Some(Row::Category { id, .. }) => tab.open(scene, LibrarySelection::Category(id)),
        Some(Row::Device) => {
            tab.page = LibraryPage::Device;
            tab.adding_to = None;
        }
        None => {}
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

/// The open list (LB39–LB42): the header, the search and the *Whole
/// library* toggle, the rows, and the selection bar while something is
/// ticked.
fn tracks_page(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState) {
    let t = scene.i18n;
    let selection = view_state
        .phone_library
        .shown
        .unwrap_or(scene.state.library_view.selection);
    if header(ui, scene, selection) {
        view_state.phone_library.back();
        return;
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
                if track_row(ui, scene, track, tab.ticked.contains(&id)) {
                    toggled = Some(id);
                }
            }
        });
    if let Some(id) = toggled {
        match tab.ticked.iter().position(|t| *t == id) {
            Some(at) => {
                tab.ticked.remove(at);
            }
            None => tab.ticked.push(id),
        }
    }
    if !tab.ticked.is_empty() {
        let shown = view_state.phone.shown_player(&scene.state.player_ids());
        let tab = &mut view_state.phone_library;
        let picked = music::Picked::Tracks(&mut tab.ticked);
        music::selection_bar(ui, scene, picked, &mut tab.target, shown, wide);
    }
}

/// The list's header: the back button, the list's name and room for its
/// ＋ (LB44); true when the back button is tapped.
fn header(ui: &mut Ui, scene: &Scene<'_>, selection: LibrarySelection) -> bool {
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
    let mut back = false;
    ui.horizontal(|ui| {
        let label = t.tr("phone-library-back");
        let style = TileStyle {
            border: Color32::TRANSPARENT,
            ..TileStyle::plain()
        };
        let size = vec2(touch::TARGET, touch::TARGET);
        let response = widgets::tile(ui, size, &label, true, style, |p, r, c| {
            p.text(
                r.center(),
                Align2::CENTER_CENTER,
                icon::ARROW_LEFT,
                font(18.0),
                c,
            );
        });
        back = touch::tapped(ui, &response);
        let width = (ui.available_width() - touch::TARGET).max(0.0);
        ui.allocate_ui(vec2(width, touch::TARGET), |ui| {
            ui.add(
                egui::Label::new(
                    egui::RichText::new(name)
                        .font(font_medium(18.0))
                        .color(theme::TEXT),
                )
                .selectable(false)
                .truncate(),
            );
        });
    });
    back
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
/// artist, its status marks (§7.4), the duration and room for its preview
/// button. Its label joins title, artist and duration; true when tapped.
fn track_row(ui: &mut Ui, scene: &Scene<'_>, track: &Track, ticked: bool) -> bool {
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
        return touch::tapped(ui, &response);
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
    touch::tapped(ui, &response)
}
