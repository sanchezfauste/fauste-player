//! The category dialogs of the library (track library spec §7.3, LB9): a
//! new category (name, colour and kind) and a new name. The model judges
//! the name; a rejected one comes back as a notice, as other model errors.

use egui::text::{CCursor, CCursorRange};
use egui::{RichText, Ui, vec2};
use fp_model::{CartColor, CategoryId, Command, LibraryView, TrackKind};

use super::super::app::{Scene, ViewState};
use super::super::exit_guard::button;
use super::super::settings::carts::{color_key, swatch};
use super::super::theme;
use super::super::widgets::{self, TileStyle, font, font_medium};
use super::state::{LibraryDialog, LibraryUi};

/// Every track kind, in the menus' order.
pub(crate) const KINDS: [TrackKind; 6] = [
    TrackKind::Music,
    TrackKind::Jingle,
    TrackKind::Effect,
    TrackKind::Ad,
    TrackKind::Voice,
    TrackKind::HourlyId,
];

/// The locale key of a track kind's name.
pub(crate) fn kind_key(kind: TrackKind) -> &'static str {
    match kind {
        TrackKind::Music => "kind-music",
        TrackKind::Jingle => "kind-jingle",
        TrackKind::Effect => "kind-effect",
        TrackKind::Ad => "kind-ad",
        TrackKind::Voice => "kind-voice",
        TrackKind::HourlyId => "kind-hourly-id",
    }
}

/// Opens the new-category dialog under `parent`. A subcategory starts with
/// its parent's colour and kind.
pub(crate) fn new_category(
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    parent: Option<CategoryId>,
) {
    let from = parent.and_then(|p| scene.state.categories.get(p));
    view_state.library.dialog = Some(LibraryDialog::NewCategory {
        parent,
        name: String::new(),
        color: from.and_then(|c| c.color),
        kind: from.map_or(TrackKind::default(), |c| c.kind),
        fresh: true,
    });
}

/// Opens the rename dialog of `category`, its name selected.
pub(crate) fn rename(scene: &Scene<'_>, view_state: &mut ViewState, category: CategoryId) {
    if let Some(c) = scene.state.categories.get(category) {
        view_state.library.dialog = Some(LibraryDialog::RenameCategory {
            category,
            name: c.name.clone(),
            fresh: true,
        });
    }
}

/// What a dialog's buttons answered this frame.
enum Answer {
    Cancel,
    Confirm,
}

/// Draws the open category dialog, if any, and sends what it confirms.
pub(crate) fn draw(ctx: &egui::Context, scene: &Scene<'_>, view_state: &mut ViewState) {
    let t = scene.i18n;
    let Some(dialog) = view_state.library.dialog.as_mut() else {
        return;
    };
    let (answer, send) = match dialog {
        LibraryDialog::NewCategory {
            parent,
            name,
            color,
            kind,
            fresh,
        } => {
            // A parent deleted meanwhile: nothing to create under.
            let parent_name = match parent {
                Some(p) => match scene.state.categories.get(*p) {
                    Some(c) => Some(c.name.clone()),
                    None => {
                        view_state.library.dialog = None;
                        return;
                    }
                },
                None => None,
            };
            let title = match parent_name {
                Some(parent) => {
                    t.tr_args("library-dialog-new-sub-title", &[("parent", parent.into())])
                }
                None => t.tr("library-dialog-new-title"),
            };
            let confirm = t.tr("library-dialog-create");
            let answer = modal(ctx, scene, &title, name, fresh, &confirm, |ui| {
                field(ui, &t.tr("library-dialog-colour"), |ui| {
                    ui.spacing_mut().item_spacing = vec2(4.0, 0.0);
                    let choices = std::iter::once(None).chain(CartColor::ALL.into_iter().map(Some));
                    for choice in choices {
                        let label = t.tr(color_key(choice));
                        if swatch(ui, choice, *color == choice, &label) {
                            *color = choice;
                        }
                    }
                });
                field(ui, &t.tr("library-dialog-kind"), |ui| {
                    ui.spacing_mut().item_spacing = vec2(4.0, 0.0);
                    for choice in KINDS {
                        let text = t.tr(kind_key(choice));
                        let on = *kind == choice;
                        let style = TileStyle {
                            fill: if on {
                                theme::NEUTRAL_700
                            } else {
                                egui::Color32::TRANSPARENT
                            },
                            content: if on { theme::TEXT } else { theme::NEUTRAL_400 },
                            ..TileStyle::plain()
                        };
                        if widgets::text_button(ui, None, &text, true, style).clicked() {
                            *kind = choice;
                        }
                    }
                });
                ui.add(
                    egui::Label::new(
                        RichText::new(t.tr("library-dialog-kind-hint"))
                            .font(font(11.0))
                            .color(theme::NEUTRAL_500),
                    )
                    .selectable(false)
                    .wrap(),
                );
            });
            let send = Command::CreateCategory {
                parent: *parent,
                name: name.trim().to_owned(),
                color: *color,
                kind: *kind,
            };
            (answer, (send, *parent))
        }
        LibraryDialog::RenameCategory {
            category,
            name,
            fresh,
        } => {
            let Some(old) = scene
                .state
                .categories
                .get(*category)
                .map(|c| c.name.clone())
            else {
                view_state.library.dialog = None;
                return;
            };
            let title = t.tr_args("library-dialog-rename-title", &[("name", old.into())]);
            let confirm = t.tr("library-dialog-rename");
            let answer = modal(ctx, scene, &title, name, fresh, &confirm, |_| {});
            let send = Command::RenameCategory {
                category: *category,
                name: name.trim().to_owned(),
            };
            (answer, (send, None))
        }
        LibraryDialog::Removal { .. } | LibraryDialog::RemoveMissing { .. } => return,
    };
    match answer {
        Some(Answer::Cancel) => view_state.library.dialog = None,
        Some(Answer::Confirm) => {
            view_state.library.dialog = None;
            let (command, parent) = send;
            scene.ctl.send(command);
            // A new subcategory shows: its parent opens.
            if let Some(parent) = parent {
                expand(scene, &view_state.library, parent);
            }
        }
        None => {}
    }
}

/// Opens `category` in the tree, if it is closed.
pub(super) fn expand(scene: &Scene<'_>, library: &LibraryUi, category: CategoryId) {
    let view = super::current_view(&scene.state.library_view, library);
    if view.expanded.contains(&category) {
        return;
    }
    let mut expanded = view.expanded.clone();
    expanded.push(category);
    scene
        .ctl
        .send(Command::SetLibraryView(Box::new(LibraryView {
            expanded,
            ..view
        })));
}

/// One labelled row of a dialog.
fn field(ui: &mut Ui, label: &str, content: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.add_sized(
            vec2(64.0, 22.0),
            egui::Label::new(
                RichText::new(label)
                    .font(font(12.0))
                    .color(theme::NEUTRAL_400),
            )
            .selectable(false),
        );
        content(ui);
    });
}

/// A dialog with a title, the name field, `extra` below it, and Cancel and
/// `confirm`. Enter in the name confirms; Esc or the backdrop cancels.
fn modal(
    ctx: &egui::Context,
    scene: &Scene<'_>,
    title: &str,
    name: &mut String,
    fresh: &mut bool,
    confirm: &str,
    extra: impl FnOnce(&mut Ui),
) -> Option<Answer> {
    let t = scene.i18n;
    let width = (ctx.content_rect().width() - 48.0).clamp(280.0, 420.0);
    let mut answer = None;
    let modal = egui::Modal::new(egui::Id::unique("library-category-dialog"))
        .frame(egui::Frame::new().fill(theme::SURFACE).inner_margin(20.0))
        .backdrop_color(theme::NEUTRAL_900.gamma_multiply(0.7))
        .show(ctx, |ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing = vec2(8.0, 10.0);
            ui.add(
                egui::Label::new(
                    RichText::new(title)
                        .font(font_medium(18.0))
                        .color(theme::TEXT),
                )
                .selectable(false)
                .truncate(),
            );
            let mut enter = false;
            ui.horizontal(|ui| {
                let label = ui.add_sized(
                    vec2(64.0, 22.0),
                    egui::Label::new(
                        RichText::new(t.tr("library-dialog-name"))
                            .font(font(12.0))
                            .color(theme::NEUTRAL_400),
                    )
                    .selectable(false),
                );
                let edit = ui
                    .add(egui::TextEdit::singleline(name).desired_width(f32::INFINITY))
                    .labelled_by(label.id);
                if std::mem::take(fresh) {
                    // The whole name selected, so typing replaces it.
                    let mut state =
                        egui::TextEdit::load_state(ui.ctx(), edit.id).unwrap_or_default();
                    let end = CCursor::new(name.chars().count());
                    state
                        .cursor
                        .set_char_range(Some(CCursorRange::two(CCursor::new(0), end)));
                    state.store(ui.ctx(), edit.id);
                    edit.request_focus();
                }
                enter = edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            });
            extra(ui);
            let ready = !name.trim().is_empty();
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if button(ui, &t.tr("library-dialog-cancel"), None) {
                    answer = Some(Answer::Cancel);
                }
                ui.add_enabled_ui(ready, |ui| {
                    if button(ui, confirm, Some(theme::ACCENT)) {
                        answer = Some(Answer::Confirm);
                    }
                });
            });
            if enter && ready {
                answer = Some(Answer::Confirm);
            }
        });
    if answer.is_none() && modal.should_close() {
        answer = Some(Answer::Cancel);
    }
    answer
}
