//! The two confirmations of a removal from the library (track library spec
//! LB11, LB17): the first gives what goes (subcategories and tracks), the
//! second, only when some of those tracks are used, the playlist entries
//! and carts that lose them. A removal that touches anything on air is
//! refused before any question (LB17b).

use egui::{RichText, vec2};
use fp_model::{
    CategoryId, Command, Library, ModelError, RemovalImpact, TrackId, category_deletion,
    missing_tracks, removal_impact,
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
    Tracks(Vec<TrackId>),
    /// A category with its subcategories and their tracks.
    Category(CategoryId),
}

/// What a removal question shows: the subcategories that go (none for
/// tracks) and what the tracks take with them, counted in `library`.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Asked {
    pub subcategories: usize,
    pub impact: RemovalImpact,
    pub library: Library,
}

impl Asked {
    /// Counts `removal` now; `None` when there is nothing left to remove.
    fn new(scene: &Scene<'_>, removal: &Removal) -> Option<Asked> {
        let (subcategories, impact) = match removal {
            Removal::Tracks(tracks) => {
                let impact = removal_impact(scene.state, tracks);
                (impact.tracks > 0).then_some((0, impact))?
            }
            Removal::Category(category) => category_deletion(scene.state, *category)?,
        };
        Some(Asked {
            subcategories,
            impact,
            library: scene.state.library.clone(),
        })
    }

    /// The same question: the same counts.
    fn same_counts(&self, other: &Asked) -> bool {
        self.subcategories == other.subcategories
            && self.impact.tracks == other.impact.tracks
            && self.impact.entries == other.impact.entries
            && self.impact.carts == other.impact.carts
    }
}

/// Refuses a removal that holds something on air: its error as a notice.
/// True when refused.
fn refused(scene: &Scene<'_>, view_state: &mut ViewState, impact: &RemovalImpact) -> bool {
    if impact.on_air.is_empty() {
        return false;
    }
    let text = error_text(
        scene.i18n,
        &ModelError::LibraryOnAir(impact.on_air.clone()),
        &scene.state.categories,
    );
    view_state.set_notice(text, scene.time);
    true
}

/// Starts a removal: refused at once while any of it is on air, else the
/// first question opens.
pub(crate) fn ask_removal(scene: &Scene<'_>, view_state: &mut ViewState, removal: Removal) {
    let Some(asked) = Asked::new(scene, &removal) else {
        return;
    };
    if refused(scene, view_state, &asked.impact) {
        return;
    }
    view_state.library.dialog = Some(LibraryDialog::Removal {
        removal,
        step: RemovalStep::First,
        asked: Box::new(asked),
    });
}

/// Starts *Remove missing tracks…* (LB21): the tracks it would remove are
/// counted now; with none, a notice says so instead of a question.
pub(crate) fn ask_remove_missing(
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    category: CategoryId,
) {
    let Some(name) = scene.state.categories.get(category).map(|c| c.name.clone()) else {
        return;
    };
    let count = missing_tracks(scene.state, category).len();
    if count == 0 {
        let text = scene
            .i18n
            .tr_args("library-remove-missing-none", &[("name", name.into())]);
        view_state.set_notice(text, scene.time);
        return;
    }
    view_state.library.dialog = Some(LibraryDialog::RemoveMissing { category, count });
}

/// Draws the *Remove missing tracks…* question, if open. The model counts
/// again when the command arrives, so tracks that became used or came back
/// meanwhile are kept.
fn draw_remove_missing(ctx: &egui::Context, scene: &Scene<'_>, view_state: &mut ViewState) {
    let Some(LibraryDialog::RemoveMissing { category, count }) = view_state.library.dialog.clone()
    else {
        return;
    };
    let Some(name) = scene.state.categories.get(category).map(|c| c.name.clone()) else {
        view_state.library.dialog = None;
        return;
    };
    let t = scene.i18n;
    let question = t.tr_args(
        "library-remove-missing-question",
        &[("count", count.into()), ("name", name.into())],
    );
    let title = t.tr("library-remove-missing-title");
    let confirm = t.tr("library-remove-missing-confirm");
    match ask(ctx, scene, title, vec![question], &confirm) {
        Some(true) => {
            view_state.library.dialog = None;
            scene.ctl.send(Command::RemoveMissingTracks(category));
        }
        Some(false) => view_state.library.dialog = None,
        None => {}
    }
}

fn command(removal: Removal) -> Command {
    match removal {
        Removal::Tracks(tracks) => Command::RemoveTracks(tracks),
        Removal::Category(category) => Command::DeleteCategory(category),
    }
}

/// Draws the open removal question, if any, and acts on its answer. The
/// counts shown are taken again only when the library changed; the answer
/// is judged on the state of its frame (what is on air lives outside the
/// library).
pub(crate) fn draw_confirmations(
    ctx: &egui::Context,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
) {
    draw_remove_missing(ctx, scene, view_state);
    if !matches!(
        view_state.library.dialog,
        Some(LibraryDialog::Removal { .. })
    ) {
        return;
    }
    let Some(LibraryDialog::Removal {
        removal,
        mut step,
        mut asked,
    }) = view_state.library.dialog.take()
    else {
        return;
    };
    // Gone or on air once the library changed: closed (refused) at once.
    if !asked.library.shares_storage(&scene.state.library) {
        let Some(now) = Asked::new(scene, &removal) else {
            return;
        };
        if refused(scene, view_state, &now.impact) {
            return;
        }
        *asked = now;
    }
    match question(ctx, scene, &removal, step, &asked) {
        Some(true) => {
            let Some(now) = Asked::new(scene, &removal) else {
                return;
            };
            if refused(scene, view_state, &now.impact) {
                return;
            }
            let changed = !now.same_counts(&asked);
            let used = now.impact.entries > 0 || now.impact.carts > 0;
            if changed {
                // The counts moved under the question: it asks again with
                // the new ones, at the same step.
                *asked = now;
            } else if step == RemovalStep::First && used {
                step = RemovalStep::Second;
            } else {
                scene.ctl.send(command(removal));
                return;
            }
        }
        Some(false) => return,
        None => {}
    }
    view_state.library.dialog = Some(LibraryDialog::Removal {
        removal,
        step,
        asked,
    });
}

/// One question. `Some(true)` goes on, `Some(false)` cancels (the button,
/// Esc or a click on the backdrop), `None` keeps it open.
fn question(
    ctx: &egui::Context,
    scene: &Scene<'_>,
    removal: &Removal,
    step: RemovalStep,
    asked: &Asked,
) -> Option<bool> {
    let t = scene.i18n;
    let (subcategories, impact) = (asked.subcategories, &asked.impact);
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
    ask(ctx, scene, title, lines, &confirm)
}

/// The modal question: a title, lines of text, *Cancel* and a confirm
/// button. `Some(true)` goes on, `Some(false)` cancels, `None` keeps it open.
fn ask(
    ctx: &egui::Context,
    scene: &Scene<'_>,
    title: String,
    lines: Vec<String>,
    confirm: &str,
) -> Option<bool> {
    let t = scene.i18n;
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
                if button(ui, confirm, Some(theme::ON_AIR_ROW)) {
                    answer = Some(true);
                }
            });
        });
    if answer.is_none() && modal.should_close() {
        answer = Some(false);
    }
    answer
}
