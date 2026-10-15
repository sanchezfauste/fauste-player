//! Settings → Playlists → Table columns (feedback 2 spec O24): which
//! columns the track tables show and in which order, for every player.

use egui::{Align, Layout, RichText, Ui, vec2};
use egui_phosphor::regular as icon;
use fp_model::{column_rows, default_columns, move_column, normalize_columns, with_column_shown};

use super::row;
use crate::i18n::Arg;
use crate::ui::app::Scene;
use crate::ui::theme;
use crate::ui::widgets::{self, TileStyle, font};

/// The room a checkbox takes besides its text: the box, its spacing and
/// the button padding on both sides, as egui lays it out, and the gap to
/// the arrows.
fn checkbox_room(ui: &Ui) -> f32 {
    let s = ui.spacing();
    s.icon_width + s.icon_spacing + 2.0 * s.button_padding.x + s.item_spacing.x
}

/// A small arrow button; `label` is its accessible name and tooltip.
fn arrow(ui: &mut Ui, label: &str, glyph: &str, enabled: bool) -> bool {
    widgets::tile(
        ui,
        vec2(28.0, 24.0),
        label,
        enabled,
        TileStyle::plain(),
        |p, r, c| widgets::glyph(p, r, glyph, 14.0, c, false),
    )
    .clicked()
}

pub(super) fn section(ui: &mut Ui, scene: &Scene<'_>) {
    let t = scene.i18n;
    let list = normalize_columns(&scene.state.config.ui.table_columns);
    let rows = column_rows(&list);
    row(
        ui,
        &t.tr("settings-columns"),
        Some(&t.tr("settings-hint-columns")),
        |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
                let names: Vec<(fp_model::TableColumn, bool, String)> = rows
                    .into_iter()
                    .map(|(c, shown)| (c, shown, t.tr(&format!("column-name-{}", c.name()))))
                    .collect();
                // One label column, as wide as the widest name, so the arrows
                // line up in every row.
                let label_width = names
                    .iter()
                    .map(|(_, _, name)| {
                        ui.painter()
                            .layout_no_wrap(name.clone(), font(12.0), theme::TEXT)
                            .size()
                            .x
                    })
                    .fold(0.0, f32::max)
                    + checkbox_room(ui);
                for (column, shown, name) in names {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
                        let mut on = shown;
                        let text = RichText::new(&name).font(font(12.0)).color(theme::TEXT);
                        let changed = ui
                            .allocate_ui_with_layout(
                                vec2(label_width, 24.0),
                                Layout::left_to_right(Align::Center),
                                |ui| {
                                    ui.set_min_width(label_width);
                                    let check = ui.add_enabled(
                                        !column.is_required(),
                                        egui::Checkbox::new(&mut on, text),
                                    );
                                    if column.is_required() {
                                        check
                                            .on_disabled_hover_text(
                                                t.tr("settings-column-required"),
                                            )
                                            .changed()
                                    } else {
                                        check.changed()
                                    }
                                },
                            )
                            .inner;
                        if changed {
                            scene.set_table_columns(with_column_shown(&list, column, on));
                        }
                        // Only the shown columns have a place to move to.
                        let at = list.iter().position(|c| *c == column);
                        let args = [("column", Arg::Text(name.clone()))];
                        let up = t.tr_args("settings-column-up", &args);
                        let down = t.tr_args("settings-column-down", &args);
                        if arrow(ui, &up, icon::CARET_UP, at.is_some_and(|i| i > 0))
                            && let Some(i) = at
                        {
                            scene.set_table_columns(move_column(&list, i, i - 1));
                        }
                        if arrow(
                            ui,
                            &down,
                            icon::CARET_DOWN,
                            at.is_some_and(|i| i + 1 < list.len()),
                        ) && let Some(i) = at
                        {
                            scene.set_table_columns(move_column(&list, i, i + 1));
                        }
                    });
                }
                ui.add_space(6.0);
                let label = t.tr("settings-columns-default");
                let is_default = list == default_columns();
                if widgets::text_button(ui, None, &label, !is_default, TileStyle::plain()).clicked()
                {
                    scene.set_table_columns(default_columns());
                }
            });
        },
    );
}
