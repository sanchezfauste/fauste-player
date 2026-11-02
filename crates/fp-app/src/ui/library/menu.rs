//! The library table's row menu (track library spec §7.4): it acts on the
//! selected rows (a right-click on a row outside the selection selects
//! that row alone first). Add to ▸ a playlist, Add to cart ▸ the shown
//! page's empty carts, Move to category ▸, Kind ▸, Edit tags… and
//! Loudness… (one row), Re-analyse, Reset markers to automatic, Remove
//! from library… (LB17) and Show file location (one row).

use std::path::Path;

use egui::Ui;
use egui_phosphor::regular as icon;
use fp_model::{AppState, CartPage, Categories, CategoryId, Command, TrackId};

use super::super::app::Scene;
use super::super::table::RowMenuOutcome;
use super::super::view;
use super::category_dialog::{KINDS, kind_key};
use crate::activity::TrackActivity;
use crate::services::ServiceRequest;

/// What the menu asks the window to do after it closes.
pub(super) enum MenuOutcome {
    /// Edit tags… or Loudness… of one track, as on a playlist row.
    Row(RowMenuOutcome),
    /// Remove from library… asks first (LB17).
    Remove(Vec<TrackId>),
    /// A notice for the status bar.
    Notice(String),
}

/// The carts of `page` that `count` tracks go to when the operator picks
/// the cart at `chosen`: that one and the next empty carts of the page, in
/// grid order (plan 3 rulings). Fewer when the page runs out of empty
/// carts.
pub(crate) fn cart_targets(page: &CartPage, chosen: usize, count: usize) -> Vec<usize> {
    page.carts
        .iter()
        .enumerate()
        .skip(chosen)
        .filter(|(_, c)| c.track.is_none())
        .map(|(i, _)| i)
        .take(count)
        .collect()
}

/// The categories in the tree's order, each with its depth.
fn tree_order(categories: &Categories) -> Vec<(CategoryId, String, usize)> {
    fn walk(
        categories: &Categories,
        parent: Option<CategoryId>,
        depth: usize,
        out: &mut Vec<(CategoryId, String, usize)>,
    ) {
        for c in categories.children(parent) {
            out.push((c.id, c.name.clone(), depth));
            walk(categories, Some(c.id), depth + 1, out);
        }
    }
    let mut out = Vec::new();
    walk(categories, None, 0, &mut out);
    out
}

/// The `file://` address of the folder that holds `path`, with the bytes
/// that cannot appear in a URL escaped (a Windows drive becomes
/// `file:///C:/…`).
pub(crate) fn folder_url(path: &Path) -> Option<String> {
    let folder = path.parent()?;
    let mut text = folder.to_string_lossy().replace('\\', "/");
    if !text.starts_with('/') {
        text.insert(0, '/');
    }
    if !text.ends_with('/') {
        text.push('/');
    }
    let mut url = String::from("file://");
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' | b':' => {
                url.push(char::from(byte));
            }
            other => url.push_str(&format!("%{other:02X}")),
        }
    }
    Some(url)
}

/// A menu entry: its text is its accessible label (tests find it by it).
fn entry(ui: &mut Ui, glyph: &str, text: &str, enabled: bool) -> egui::Response {
    let shown = if glyph.is_empty() {
        text.to_owned()
    } else {
        format!("{glyph}  {text}")
    };
    let response = ui.add_enabled(enabled, egui::Button::new(shown));
    let owned = text.to_owned();
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, owned.clone()));
    response
}

/// A submenu whose button is named `text`.
fn submenu(ui: &mut Ui, glyph: &str, text: &str, enabled: bool, add: impl FnOnce(&mut Ui)) {
    let owned = text.to_owned();
    ui.add_enabled_ui(enabled, |ui| {
        ui.menu_button(format!("{glyph}  {text}"), add)
            .response
            .widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, owned.clone()));
    });
}

/// The row menu over `tracks` (the selected rows, in the table's order).
pub(super) fn row_menu(ui: &mut Ui, scene: &Scene<'_>, tracks: &[TrackId]) -> Option<MenuOutcome> {
    let t = scene.i18n;
    let state: &AppState = scene.state;
    let mut outcome = None;
    let one = match tracks {
        [one] => state.library.get(*one),
        _ => None,
    };
    ui.set_min_width(240.0);
    let heading = match one {
        Some(track) => track.title.clone(),
        None => t.tr_args("library-menu-selected", &[("count", tracks.len().into())]),
    };
    ui.add(
        egui::Label::new(
            egui::RichText::new(heading)
                .font(super::super::widgets::font(11.0))
                .color(super::super::theme::NEUTRAL_400),
        )
        .selectable(false)
        .truncate(),
    );
    ui.separator();

    // Add to ▸ every playlist, at its end.
    let playlists: Vec<_> = state
        .playlists
        .iter()
        .map(|pl| (pl.id, pl.name.clone(), pl.entries.len()))
        .collect();
    submenu(
        ui,
        icon::PLAYLIST,
        &t.tr("library-menu-add-to"),
        !playlists.is_empty(),
        |ui| {
            for (id, name, len) in playlists {
                if entry(ui, "", &name, true).clicked() {
                    scene.ctl.send(Command::InsertTracks {
                        playlist: id,
                        index: len,
                        tracks: tracks.to_vec(),
                    });
                    ui.close();
                }
            }
        },
    );

    // Add to cart ▸ the shown page's empty carts.
    let page = state.cartwall.shown_page();
    let empty: Vec<usize> = page.map_or_else(Vec::new, |p| {
        p.carts
            .iter()
            .enumerate()
            .filter(|(_, c)| c.track.is_none())
            .map(|(i, _)| i)
            .collect()
    });
    submenu(
        ui,
        icon::SQUARES_FOUR,
        &t.tr("library-menu-add-to-cart"),
        !empty.is_empty(),
        |ui| {
            let Some(page) = page else { return };
            for index in empty {
                let name = t.tr_args("library-menu-cart", &[("n", (index + 1).into())]);
                if entry(ui, "", &name, true).clicked() {
                    let targets = cart_targets(page, index, tracks.len());
                    for (cart, track) in targets.iter().zip(tracks) {
                        scene.ctl.send(Command::AssignCartTrack {
                            page: page.id,
                            index: *cart,
                            track: *track,
                        });
                    }
                    let left = tracks.len().saturating_sub(targets.len());
                    if left > 0 {
                        outcome = Some(MenuOutcome::Notice(
                            t.tr_args("library-cart-left-out", &[("count", left.into())]),
                        ));
                    }
                    ui.close();
                }
            }
        },
    );

    // Move to category ▸ the categories in the tree's order, then
    // Uncategorised.
    let categories = tree_order(&state.categories);
    let current: Option<Option<CategoryId>> = {
        let mut of = tracks
            .iter()
            .filter_map(|id| state.library.get(*id))
            .map(|tr| tr.category);
        let first = of.next();
        first.filter(|f| of.all(|c| c == *f))
    };
    submenu(
        ui,
        icon::FOLDER,
        &t.tr("library-menu-move-to-category"),
        true,
        |ui| {
            let choice = |ui: &mut Ui, label: String, category: Option<CategoryId>| {
                let on = current == Some(category);
                let response = ui.add(egui::Button::selectable(on, &label));
                let name = label.trim_start().to_owned();
                response.widget_info(|| {
                    egui::WidgetInfo::selected(egui::Role::Button, true, on, name.clone())
                });
                if response.clicked() {
                    scene.ctl.send(Command::MoveTracksToCategory {
                        tracks: tracks.to_vec(),
                        category,
                    });
                    ui.close();
                }
            };
            for (id, name, depth) in categories {
                choice(ui, format!("{}{name}", "    ".repeat(depth)), Some(id));
            }
            ui.separator();
            choice(ui, t.tr("library-tree-uncategorised"), None);
        },
    );

    // Kind ▸
    let current_kind = {
        let mut of = tracks
            .iter()
            .filter_map(|id| state.library.get(*id))
            .map(|tr| tr.kind);
        let first = of.next();
        first.filter(|f| of.all(|k| k == *f))
    };
    submenu(ui, icon::TAG, &t.tr("library-menu-kind"), true, |ui| {
        for kind in KINDS {
            let text = t.tr(kind_key(kind));
            let on = current_kind == Some(kind);
            let response = ui.add(egui::Button::selectable(on, &text));
            response.widget_info(|| {
                egui::WidgetInfo::selected(egui::Role::Button, true, on, text.clone())
            });
            if response.clicked() {
                scene.ctl.send(Command::SetTracksKind {
                    tracks: tracks.to_vec(),
                    kind,
                });
                ui.close();
            }
        }
    });
    ui.separator();

    // Edit tags… and Loudness…: one row, as on a playlist.
    if let Some(track) = one {
        // Read-only music on mobile (spec §2): no tag editing.
        if scene.platform.can_write_user_files() {
            let block = view::tag_edit_availability(state, track.id);
            let edit = entry(
                ui,
                icon::PENCIL_SIMPLE,
                &t.tr("menu-edit-tags"),
                block.is_none(),
            );
            let edit = match block {
                Some(reason) => {
                    edit.on_disabled_hover_text(t.tr(super::super::tag_editor::block_key(reason)))
                }
                None => edit,
            };
            if edit.clicked() {
                outcome = Some(MenuOutcome::Row(RowMenuOutcome::EditTags(track.id)));
                ui.close();
            }
        }
        if entry(ui, icon::SPEAKER_HIGH, &t.tr("menu-loudness"), true).clicked() {
            outcome = Some(MenuOutcome::Row(RowMenuOutcome::Loudness(track.id)));
            ui.close();
        }
    }

    // Re-analyse: a track already queued or running is left alone (main
    // spec §6.1).
    let idle: Vec<TrackId> = tracks
        .iter()
        .copied()
        .filter(|id| scene.analysis.of(*id).is_none())
        .collect();
    let reanalyse = entry(
        ui,
        icon::ARROWS_CLOCKWISE,
        &t.tr("menu-reanalyse"),
        !idle.is_empty(),
    );
    let reanalyse = match (one.map(|tr| scene.analysis.of(tr.id)), idle.is_empty()) {
        (Some(Some(TrackActivity::Queued)), _) => {
            reanalyse.on_disabled_hover_text(t.tr("menu-reanalyse-queued"))
        }
        (Some(Some(TrackActivity::Running)), _) => {
            reanalyse.on_disabled_hover_text(t.tr("menu-reanalyse-running"))
        }
        _ => reanalyse,
    };
    if reanalyse.clicked() {
        for id in idle {
            scene.request(ServiceRequest::ReanalyseTrack(id));
        }
        ui.close();
    }
    if entry(ui, icon::ARROW_COUNTER_CLOCKWISE, &t.tr("wave-reset"), true).clicked() {
        for id in tracks {
            scene.ctl.send(Command::ResetMarkers { track: *id });
        }
        ui.close();
    }
    if let Some(track) = one
        && let Some(url) = folder_url(&track.path)
        && entry(ui, icon::FOLDER_OPEN, &t.tr("library-menu-show-file"), true).clicked()
    {
        // The platform opens it off this thread (the UI never blocks).
        if let Err(error) = scene.platform.open_url(&url) {
            tracing::warn!(%error, %url, "could not show the file's folder");
        }
        ui.close();
    }
    ui.separator();
    if entry(ui, icon::TRASH, &t.tr("library-menu-remove"), true).clicked() {
        outcome = Some(MenuOutcome::Remove(tracks.to_vec()));
        ui.close();
    }
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn the_folder_url_escapes_what_a_url_cannot_hold() {
        assert_eq!(
            folder_url(&PathBuf::from("/music/My Songs/a.mp3")).as_deref(),
            Some("file:///music/My%20Songs/")
        );
        assert_eq!(
            folder_url(&PathBuf::from("/música/a.mp3")).as_deref(),
            Some("file:///m%C3%BAsica/")
        );
    }
}
