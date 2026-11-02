//! The two confirmations of a removal from the library (track library spec
//! LB11, LB17): the first gives what goes (subcategories and tracks), the
//! second, only when some of those tracks are used, the playlist entries
//! and carts that lose them. A removal that touches anything on air is
//! refused before any question (LB17b).

use egui::{RichText, vec2};
use fp_model::{
    CategoryId, Command, ModelError, RemovalImpact, TrackId, category_deletion, removal_impact,
};

use super::super::app::{Scene, ViewState, error_text};
use super::super::exit_guard::button;
use super::super::theme;
use super::super::widgets::{font, font_medium};
use super::state::{LibraryDialog, RemovalStep};

/// What the operator asked to remove.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Removal {
    /// Tracks, from the table.
    #[expect(
        dead_code,
        reason = "the table's row menu (plan 3, task 6) removes tracks"
    )]
    Tracks(Vec<TrackId>),
    /// A category with its subcategories and their tracks.
    Category(CategoryId),
}

/// The subcategories that go with `removal` (none for tracks) and what its
/// tracks take with them; `None` when there is nothing left to remove.
fn impact(scene: &Scene<'_>, removal: &Removal) -> Option<(usize, RemovalImpact)> {
    match removal {
        Removal::Tracks(tracks) => {
            let impact = removal_impact(scene.state, tracks);
            (impact.tracks > 0).then_some((0, impact))
        }
        Removal::Category(category) => category_deletion(scene.state, *category),
    }
}

/// Refuses a removal that holds something on air: its error as a notice.
/// True when refused.
fn refused(scene: &Scene<'_>, view_state: &mut ViewState, impact: &RemovalImpact) -> bool {
    if impact.on_air.is_empty() {
        return false;
    }
    let text = error_text(scene.i18n, &ModelError::LibraryOnAir(impact.on_air.clone()));
    view_state.set_notice(text, scene.time);
    true
}

/// Starts a removal: refused at once while any of it is on air, else the
/// first question opens.
pub(crate) fn ask_removal(scene: &Scene<'_>, view_state: &mut ViewState, removal: Removal) {
    let Some((_, impact)) = impact(scene, &removal) else {
        return;
    };
    if refused(scene, view_state, &impact) {
        return;
    }
    view_state.library.dialog = Some(LibraryDialog::Removal {
        removal,
        step: RemovalStep::First,
    });
}

fn command(removal: Removal) -> Command {
    match removal {
        Removal::Tracks(tracks) => Command::RemoveTracks(tracks),
        Removal::Category(category) => Command::DeleteCategory(category),
    }
}

/// Draws the open removal question, if any, and acts on its answer.
pub(crate) fn draw_confirmations(
    ctx: &egui::Context,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
) {
    let Some(LibraryDialog::Removal { removal, step }) = view_state.library.dialog.clone() else {
        return;
    };
    // What is asked about is judged again each frame: it may have changed.
    let Some((subcategories, impact)) = impact(scene, &removal) else {
        view_state.library.dialog = None;
        return;
    };
    if refused(scene, view_state, &impact) {
        view_state.library.dialog = None;
        return;
    }
    let used = impact.entries > 0 || impact.carts > 0;
    match question(ctx, scene, &removal, step, subcategories, &impact) {
        Some(true) if step == RemovalStep::First && used => {
            view_state.library.dialog = Some(LibraryDialog::Removal {
                removal,
                step: RemovalStep::Second,
            });
        }
        Some(true) => {
            view_state.library.dialog = None;
            scene.ctl.send(command(removal));
        }
        Some(false) => view_state.library.dialog = None,
        None => {}
    }
}

/// One question. `Some(true)` goes on, `Some(false)` cancels (the button,
/// Esc or a click on the backdrop), `None` keeps it open.
fn question(
    ctx: &egui::Context,
    scene: &Scene<'_>,
    removal: &Removal,
    step: RemovalStep,
    subcategories: usize,
    impact: &RemovalImpact,
) -> Option<bool> {
    let t = scene.i18n;
    let (title, lines, confirm) = match (step, removal) {
        (RemovalStep::First, Removal::Category(category)) => {
            let name = scene
                .state
                .categories
                .get(*category)
                .map(|c| c.name.clone())
                .unwrap_or_default();
            let mut lines = Vec::new();
            if subcategories > 0 {
                lines.push(t.tr_args(
                    "library-delete-subcategories",
                    &[("count", subcategories.into())],
                ));
            }
            lines.push(if impact.tracks == 0 {
                t.tr("library-delete-no-tracks")
            } else {
                t.tr_args("library-delete-tracks", &[("count", impact.tracks.into())])
            });
            lines.push(t.tr("library-delete-files-kept"));
            (
                t.tr_args("library-delete-title", &[("name", name.into())]),
                lines,
                t.tr("library-delete-confirm"),
            )
        }
        (RemovalStep::First, Removal::Tracks(_)) => (
            t.tr_args("library-remove-title", &[("count", impact.tracks.into())]),
            vec![t.tr("library-delete-files-kept")],
            t.tr("library-remove-confirm"),
        ),
        (RemovalStep::Second, _) => (
            t.tr("library-used-title"),
            vec![t.tr_args(
                "library-used-body",
                &[
                    ("entries", impact.entries.into()),
                    ("carts", impact.carts.into()),
                ],
            )],
            t.tr("library-used-confirm"),
        ),
    };
    let width = (ctx.content_rect().width() - 48.0).clamp(280.0, 420.0);
    let mut answer = None;
    let modal = egui::Modal::new(egui::Id::unique("library-removal"))
        .frame(egui::Frame::new().fill(theme::SURFACE).inner_margin(20.0))
        .backdrop_color(theme::NEUTRAL_900.gamma_multiply(0.7))
        .show(ctx, |ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            ui.add(
                egui::Label::new(
                    RichText::new(title)
                        .font(font_medium(18.0))
                        .color(theme::TEXT),
                )
                .selectable(false)
                .wrap(),
            );
            for line in lines {
                ui.add(
                    egui::Label::new(
                        RichText::new(line)
                            .font(font(12.0))
                            .color(theme::NEUTRAL_300),
                    )
                    .selectable(false)
                    .wrap(),
                );
            }
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if button(ui, &t.tr("library-dialog-cancel"), None) {
                    answer = Some(false);
                }
                if button(ui, &confirm, Some(theme::ON_AIR_ROW)) {
                    answer = Some(true);
                }
            });
        });
    if answer.is_none() && modal.should_close() {
        answer = Some(false);
    }
    answer
}
