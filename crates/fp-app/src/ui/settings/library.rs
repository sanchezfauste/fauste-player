//! Settings → Library (track library spec §9): the category tree, to
//! create, rename, recolour, change the kind, move and delete categories
//! without opening the library window. It is the window's tree without
//! *All tracks* and *Uncategorised*, and nothing to select.

use egui::{Ui, vec2};

use super::note;
use crate::ui::app::{Scene, ViewState};
use crate::ui::library::tree::{TreeMode, category_tree};
use crate::ui::theme;

/// The tree's height: it scrolls inside the section.
const TREE_HEIGHT: f32 = 360.0;

pub(super) fn section(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState) {
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
