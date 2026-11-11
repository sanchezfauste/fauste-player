//! Settings → Library (track library spec §9): the external editor
//! (external editor spec EE1), then the category tree, to create, rename,
//! recolour, change the kind, move and delete categories without opening
//! the library window. It is the window's tree without *All tracks* and
//! *Uncategorised*, and nothing to select.

use std::path::PathBuf;

use crossbeam_channel::Receiver;
use egui::{RichText, Ui, vec2};
use egui_phosphor::regular as icon;

use super::{SettingsState, note, row, update};
use crate::ui::app::{Scene, ViewState};
use crate::ui::library::tree::{TreeMode, category_tree};
use crate::ui::theme;
use crate::ui::widgets::{self, TileStyle, font};

/// The tree's height: it scrolls inside the section.
const TREE_HEIGHT: f32 = 360.0;

pub(super) fn section(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    st: &mut SettingsState,
) {
    // EE1, §6: only where programs can be started (not on Android, nor in
    // the Flatpak and snap builds).
    if scene.platform.can_launch_programs() {
        external_editor(ui, scene, st);
        ui.add_space(12.0);
    }
    // On a touch screen the tree only takes taps: no drag to point to.
    let hint = if view_state.touch {
        "settings-library-hint-touch"
    } else {
        "settings-library-hint"
    };
    note(ui, &scene.i18n.tr(hint), theme::NEUTRAL_400);
    ui.add_space(6.0);
    ui.allocate_ui(vec2(ui.available_width(), TREE_HEIGHT), |ui| {
        category_tree(ui, scene, view_state, TreeMode::Settings);
    });
}

/// The editor's program (EE1): its path, Choose… (the native file dialog,
/// off the interface thread) and Clear.
fn external_editor(ui: &mut Ui, scene: &Scene<'_>, st: &mut SettingsState) {
    let t = scene.i18n;
    let current = scene.state.config.library.external_editor.clone();
    // No editor set: a dimmed placeholder rather than an empty space.
    let (shown, colour) = match &current {
        Some(p) => (p.display().to_string(), theme::NEUTRAL_300),
        None => (t.tr("settings-external-editor-none"), theme::NEUTRAL_500),
    };
    row(
        ui,
        &t.tr("settings-external-editor"),
        Some(&t.tr("settings-hint-external-editor")),
        |ui| {
            ui.add(
                egui::Label::new(RichText::new(shown).font(font(12.0)).color(colour))
                    .selectable(false)
                    .truncate(),
            );
            let choose = t.tr("settings-choose");
            if widgets::text_button(
                ui,
                Some(icon::FOLDER_OPEN),
                &choose,
                st.editor.is_none(),
                TileStyle::plain(),
            )
            .clicked()
            {
                st.editor = pick_program(scene, current.clone());
            }
            let clear = t.tr("settings-clear");
            if widgets::text_button(ui, None, &clear, current.is_some(), TileStyle::plain())
                .clicked()
            {
                update(scene, |c| c.library.external_editor = None);
            }
        },
    );
}

/// Opens the file dialog for the editor's program on a helper thread, in
/// the folder of the current one; no extension filter (programs have none
/// on Linux and macOS).
fn pick_program(scene: &Scene<'_>, current: Option<PathBuf>) -> Option<Receiver<Option<PathBuf>>> {
    let (tx, rx) = crossbeam_channel::bounded(1);
    let ctx = scene.ctx.clone();
    let title = scene.i18n.tr("settings-external-editor");
    let start = current.and_then(|p| p.parent().map(PathBuf::from));
    let platform = scene.platform.clone();
    let spawned = std::thread::Builder::new()
        .name("fp-editor-dialog".to_owned())
        .spawn(move || {
            let _ = tx.send(platform.pick_file(&title, &[], start));
            ctx.request_repaint();
        });
    match spawned {
        Ok(_) => Some(rx),
        Err(e) => {
            tracing::error!(error = %e, "could not open the file dialog");
            None
        }
    }
}
