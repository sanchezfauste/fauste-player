//! The phone's Library tab (track library spec §11.1): the category
//! selector (LB38), a category's track list (LB39) and Device music, the
//! phone's Music screen (phone layout spec §9). The open category is the
//! shared `library_view.selection`; the tab keeps only its page.

use egui::{Align2, Color32, Rect, Role, Sense, Stroke, StrokeKind, Ui, WidgetInfo, pos2, vec2};
use egui_phosphor::regular as icon;
use fp_model::{CartColor, Categories, CategoryId, Command, LibrarySelection, TrackId};

use super::app::{Scene, ViewState};
use super::widgets::{self, TileStyle, font, font_medium};
use super::{library, music, theme, touch};

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
    /// The category whose list was opened: when it goes away the tab
    /// returns to the selector (the snapshot's selection falls back on
    /// *All tracks* then, so it cannot tell).
    pub shown: Option<CategoryId>,
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
        self.shown = match selection {
            LibrarySelection::Category(id) => Some(id),
            _ => None,
        };
    }

    /// A list whose category no longer exists returns to the selector,
    /// its ticks cleared (LB39, settled details).
    pub fn settle(&mut self, categories: &Categories) {
        if self.shown.is_some_and(|id| categories.get(id).is_none()) {
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

/// The open category's list (LB39): its header with the back button and
/// the category's name. Its rows come with the track list (plan 6, task 3).
fn tracks_page(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState) {
    let t = scene.i18n;
    let name = match scene.state.library_view.selection {
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
    if back {
        view_state.phone_library.back();
    }
}
