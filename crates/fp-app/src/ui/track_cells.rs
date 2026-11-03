//! The cells a track row shows, shared by the playlists' table and the
//! library's (track library spec §7.4: the same columns and the same
//! status): header labels, the tag columns, the title with its flags, and
//! the selected row's outline.

use egui::{Align, Color32, FontId, Layout, Rect, RichText, Sense, Ui, pos2, vec2};
use egui_phosphor::regular as icon;
use fp_model::{TableColumn, Track};

use super::app::Scene;
use super::theme;
use super::view;
use super::widgets::{self, font};

/// The colours of a row's text: the title and times, and the other tags.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RowColours {
    pub text: Color32,
    pub secondary: Color32,
}

/// Whether a column's values sit at the right of their cell (times).
pub(crate) fn right_aligned(column: TableColumn) -> bool {
    matches!(column, TableColumn::Duration | TableColumn::Intro)
}

/// A column header's text, in capitals.
pub(crate) fn header_label(ui: &mut Ui, text: &str, right: bool) {
    let label = egui::Label::new(
        RichText::new(text.to_uppercase())
            .font(font(10.0))
            .color(theme::NEUTRAL_500),
    )
    .selectable(false);
    if right {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.add_space(8.0);
            ui.add(label);
        });
    } else {
        ui.add_space(8.0);
        ui.add(label.truncate());
    }
}

/// A small flag icon with `label` as its accessible name and tooltip.
pub(crate) fn flag(ui: &mut Ui, label: &str, paint: impl FnOnce(&egui::Painter, Rect)) {
    let (rect, response) = ui.allocate_exact_size(vec2(16.0, 12.0), Sense::hover());
    let owned = label.to_owned();
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, owned.clone()));
    if ui.is_rect_visible(rect) {
        paint(ui.painter(), rect);
    }
    response.on_hover_text(label);
}

/// The flag of a track whose file cannot be played (missing or
/// unreadable), its reason and path as its name; nothing when it can.
pub(crate) fn file_flag(ui: &mut Ui, scene: &Scene<'_>, track: &Track) {
    let Some(tip) = scene.file_tip(track.id) else {
        return;
    };
    flag(ui, &tip, |p, r| {
        widgets::glyph(p, r, view::file_icon(track), 13.0, theme::AMBER, false);
    });
}

/// The title cell: the title (after what `leading` draws before it, such
/// as an entry's repeat and stop flags) and, at the right, the analysis
/// flags (main spec §6.1).
pub(crate) fn title_cell(
    ui: &mut Ui,
    scene: &Scene<'_>,
    track: &Track,
    row_font: &FontId,
    text: Color32,
    leading: impl FnOnce(&mut Ui),
) {
    let t = scene.i18n;
    ui.add_space(8.0);
    // The entry's flags sit before the title, in the row's text colour
    // (feedback 2 spec O9); the "analysed by an earlier version" flag stays
    // at the right.
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        ui.add_space(4.0);
        // Shown tracks are brought up to date anyway, so the flag only stays
        // on the ones waiting.
        if crate::services::outdated(track) {
            flag(ui, &t.tr("flag-outdated"), |p, r| {
                widgets::glyph(
                    p,
                    r,
                    icon::ARROWS_CLOCKWISE,
                    13.0,
                    theme::NEUTRAL_500,
                    false,
                );
            });
        }
        // Main spec §6.1: running, queued, or not analysed yet and not
        // queued.
        if let Some((glyph, key)) = view::analysis_flag(track, scene.analysis) {
            flag(ui, &t.tr(key), |p, r| {
                widgets::glyph(p, r, glyph, 13.0, theme::NEUTRAL_400, false);
            });
        }
        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            leading(ui);
            ui.add(
                egui::Label::new(
                    RichText::new(&track.title)
                        .font(row_font.clone())
                        .color(text),
                )
                .selectable(false)
                .truncate(),
            );
        });
    });
}

/// A tag column's cell: every column but `#` (which draws an entry's
/// position) and Title ([`title_cell`]). Times sit at the right in the
/// text colour; the other tags at the left in the secondary one.
pub(crate) fn tag_cell(
    ui: &mut Ui,
    scene: &Scene<'_>,
    track: &Track,
    column: TableColumn,
    colours: RowColours,
) {
    let use_markers = scene.state.config.players.use_cue_markers;
    match column {
        TableColumn::Number => {}
        TableColumn::Title => {
            title_cell(ui, scene, track, &font(12.0), colours.text, |_| {});
        }
        TableColumn::Duration | TableColumn::Intro => {
            let d = view::cell_text(track, column, use_markers);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(10.0);
                ui.add(
                    egui::Label::new(RichText::new(d).font(font(12.0)).color(colours.text))
                        .selectable(false),
                );
            });
        }
        TableColumn::Artist
        | TableColumn::Album
        | TableColumn::Date
        | TableColumn::Genre
        | TableColumn::FileName
        | TableColumn::Comment => {
            let text = if column == TableColumn::Artist && track.artist.is_empty() {
                scene.i18n.tr("unknown-artist")
            } else {
                view::cell_text(track, column, use_markers)
            };
            ui.add_space(8.0);
            ui.add(
                egui::Label::new(
                    RichText::new(text)
                        .font(font(12.0))
                        .color(colours.secondary),
                )
                .selectable(false)
                .truncate(),
            );
        }
    }
}

/// Width of the selected row's outline.
const SELECTION_OUTLINE_WIDTH: f32 = 2.0;

/// The selected row's outline, on the part of the row that `cell` covers:
/// its top and bottom edges, and the left or right edge when the cell is the
/// row's first or last. Drawn on any row colour, so a selected on-air or
/// next row still shows it is selected.
pub(crate) fn selection_outline(ui: &Ui, cell: Rect, first: bool, last: bool) {
    let w = SELECTION_OUTLINE_WIDTH;
    let painter = ui.painter();
    let colour = theme::SELECTION_OUTLINE;
    painter.rect_filled(
        Rect::from_min_size(cell.min, vec2(cell.width(), w)),
        0.0,
        colour,
    );
    painter.rect_filled(
        Rect::from_min_size(pos2(cell.left(), cell.bottom() - w), vec2(cell.width(), w)),
        0.0,
        colour,
    );
    if first {
        painter.rect_filled(
            Rect::from_min_size(cell.min, vec2(w, cell.height())),
            0.0,
            colour,
        );
    }
    if last {
        painter.rect_filled(
            Rect::from_min_size(pos2(cell.right() - w, cell.top()), vec2(w, cell.height())),
            0.0,
            colour,
        );
    }
}

/// A row's background and the line under it, over `full`.
pub(crate) fn row_background(ui: &Ui, full: Rect, fill: Color32) {
    ui.painter().rect_filled(full, 0.0, fill);
    ui.painter().rect_filled(
        Rect::from_min_size(
            pos2(full.left(), full.bottom() - 1.0),
            vec2(full.width(), 1.0),
        ),
        0.0,
        theme::TEXT.gamma_multiply(0.06),
    );
}
