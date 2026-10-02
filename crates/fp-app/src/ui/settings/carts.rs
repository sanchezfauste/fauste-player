//! Settings → Cartwall (Phase 2 spec P2.9): pages, grid and the cart editor.

use std::path::PathBuf;

use super::super::playlist_files::FileError;
use crossbeam_channel::Receiver;
use egui::{Color32, RichText, Ui, vec2};
use egui_phosphor::regular as icon;
use fp_model::{CartEdit, CartKind, CartPageId, CartPageImport, Command};
use fp_store::playlist_io::{parse_cart_page, read_bounded, write_cart_page};

use super::super::app::Scene;
use super::super::cartwall::page_name;
use super::super::files::AUDIO_EXTENSIONS;
use super::super::theme;
use super::super::widgets::{self, TileStyle, font};
use super::{SettingsState, labelled_row, row, toggle};

#[derive(Default)]
pub(crate) struct CartsState {
    page: Option<CartPageId>,
    index: usize,
    /// Name being edited, and for which cart.
    name: String,
    name_for: Option<(CartPageId, usize)>,
    page_name: String,
    page_name_for: Option<CartPageId>,
    file: Option<Receiver<Option<PathBuf>>>,
    import: Option<Receiver<Option<Result<CartPageImport, FileError>>>>,
    export: Option<Receiver<Option<Result<PathBuf, FileError>>>>,
    message: Option<String>,
    /// Grid size being dragged, applied on release.
    grid_draft: Option<(CartPageId, u16, u16)>,
}

impl CartsState {
    pub(crate) fn select(&mut self, page: CartPageId, index: usize) {
        self.page = Some(page);
        self.index = index;
    }
}

/// Runs `job` on a helper thread (dialogs and file I/O never block the UI).
fn background<T: Send + 'static>(
    scene: &Scene<'_>,
    job: impl FnOnce() -> T + Send + 'static,
) -> Option<Receiver<T>> {
    let (tx, rx) = crossbeam_channel::bounded(1);
    let ctx = scene.ctx.clone();
    let spawned = std::thread::Builder::new()
        .name("fp-cartwall-io".to_owned())
        .spawn(move || {
            let _ = tx.send(job());
            ctx.request_repaint();
        });
    match spawned {
        Ok(_) => Some(rx),
        Err(e) => {
            tracing::error!(error = %e, "could not start a helper thread");
            None
        }
    }
}

fn button(ui: &mut Ui, label: &str, glyph: &str, enabled: bool) -> bool {
    let width = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font(12.0), theme::TEXT)
        .size()
        .x
        + 36.0;
    widgets::tile(
        ui,
        vec2(width, 28.0),
        label,
        enabled,
        TileStyle::plain(),
        |p, r, c| {
            p.text(
                r.center(),
                egui::Align2::CENTER_CENTER,
                format!("{glyph} {label}"),
                font(12.0),
                c,
            );
        },
    )
    .clicked()
}

/// A labelled single-line text field; the label names the field for
/// accessibility (and tests).
fn text_row(ui: &mut Ui, label: &str, text: &mut String) -> egui::Response {
    labelled_row(ui, label, None, |ui, label_id| {
        ui.add(egui::TextEdit::singleline(text).desired_width(ui.available_width()))
            .labelled_by(label_id)
    })
}

fn poll(scene: &Scene<'_>, c: &mut CartsState) {
    let t = scene.i18n;
    if let Some(rx) = &c.file
        && let Ok(picked) = rx.try_recv()
    {
        if let (Some(path), Some(page)) = (picked, c.page) {
            scene.ctl.send(Command::AssignCartFile {
                page,
                index: c.index,
                path,
            });
        }
        c.file = None;
    }
    if let Some(rx) = &c.import
        && let Ok(result) = rx.try_recv()
    {
        match result {
            Some(Ok(import)) => {
                c.message = Some(t.tr_args(
                    "settings-cart-imported",
                    &[("name", import.name.clone().into())],
                ));
                scene.ctl.send(Command::ImportCartPage(Box::new(import)));
            }
            Some(Err(e)) => {
                c.message = Some(t.tr_args(
                    "settings-cart-import-failed",
                    &[("error", e.text(t).into())],
                ))
            }
            None => {}
        }
        c.import = None;
    }
    if let Some(rx) = &c.export
        && let Ok(result) = rx.try_recv()
    {
        c.message = match result {
            Some(Ok(path)) => Some(t.tr_args(
                "settings-cart-exported",
                &[("path", path.display().to_string().into())],
            )),
            Some(Err(e)) => Some(t.tr_args(
                "settings-cart-export-failed",
                &[("error", e.text(t).into())],
            )),
            None => None,
        };
        c.export = None;
    }
}

pub(super) fn section(ui: &mut Ui, scene: &Scene<'_>, st: &mut SettingsState) {
    let t = scene.i18n;
    let c = &mut st.carts;
    poll(scene, c);
    let wall = &scene.state.cartwall;
    let page = c
        .page
        .and_then(|id| wall.page(id))
        .or_else(|| wall.shown_page())
        .or_else(|| wall.pages.first());
    let Some(page) = page else {
        return;
    };
    let page_id = page.id;
    c.page = Some(page_id);
    // Page tabs and page actions.
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
        for (i, p) in wall.pages.iter().enumerate() {
            let name = page_name(scene, i, &p.name);
            let on = p.id == page_id;
            let style = TileStyle {
                fill: if on {
                    theme::NEUTRAL_700
                } else {
                    Color32::TRANSPARENT
                },
                content: if on { theme::TEXT } else { theme::NEUTRAL_400 },
                ..TileStyle::plain()
            };
            let width = ui
                .painter()
                .layout_no_wrap(name.clone(), font(12.0), theme::TEXT)
                .size()
                .x
                + 24.0;
            if widgets::tile(ui, vec2(width, 28.0), &name, true, style, |pa, r, col| {
                pa.text(
                    r.center(),
                    egui::Align2::CENTER_CENTER,
                    &name,
                    font(12.0),
                    col,
                );
            })
            .clicked()
            {
                c.select(p.id, 0);
            }
        }
        if button(ui, &t.tr("settings-cart-new-page"), icon::PLUS, true) {
            scene.ctl.send(Command::CreateCartPage {
                name: String::new(),
            });
        }
        if button(
            ui,
            &t.tr("settings-cart-import"),
            icon::DOWNLOAD_SIMPLE,
            c.import.is_none(),
        ) {
            let limits = scene.state.config.limits.clone();
            c.import = background(scene, move || {
                let file = pollster::block_on(
                    rfd::AsyncFileDialog::new()
                        .add_filter("JSON", &["json"])
                        .pick_file(),
                )?;
                let path = file.path().to_path_buf();
                Some(
                    read_bounded(&path, limits.max_playlist_file_bytes)
                        .map_err(FileError::from)
                        .and_then(|bytes| {
                            parse_cart_page(&bytes, &path, &limits).map_err(FileError::from)
                        }),
                )
            });
        }
        if button(
            ui,
            &t.tr("settings-cart-export"),
            icon::EXPORT,
            c.export.is_none(),
        ) {
            let text = write_cart_page(page, &scene.state.library);
            let index = wall.pages.iter().position(|p| p.id == page_id).unwrap_or(0);
            let file_name = format!(
                "{}.cartpage.json",
                super::super::playlist_files::safe_file_name(&page_name(scene, index, &page.name))
            );
            c.export = background(scene, move || {
                let file = pollster::block_on(
                    rfd::AsyncFileDialog::new()
                        .set_file_name(file_name)
                        .save_file(),
                )?;
                let path = file.path().to_path_buf();
                Some(
                    std::fs::write(&path, text)
                        .map(|()| path)
                        .map_err(|e| FileError::from_io(&e)),
                )
            });
        }
        if button(ui, &t.tr("settings-cart-delete-page"), icon::TRASH, true) {
            scene.ctl.send(Command::DeleteCartPage(page_id));
        }
    });
    if let Some(message) = &c.message {
        ui.add(
            egui::Label::new(RichText::new(message).font(font(11.0)).color(theme::AMBER))
                .selectable(false)
                .wrap(),
        );
    }
    ui.add_space(8.0);
    // Page name and grid.
    if c.page_name_for != Some(page_id) {
        if let Some(old_id) = c.page_name_for
            && let Some(old) = wall.page(old_id)
            && old.name != c.page_name
        {
            scene.ctl.send(Command::RenameCartPage {
                page: old_id,
                name: c.page_name.trim().to_owned(),
            });
        }
        c.page_name = page.name.clone();
        c.page_name_for = Some(page_id);
    }
    let page_label = t.tr("settings-cart-page-name");
    let response = text_row(ui, &page_label, &mut c.page_name);
    let cancelled = ui.input(|i| i.key_pressed(egui::Key::Escape));
    if response.lost_focus() && !cancelled && c.page_name != page.name {
        scene.ctl.send(Command::RenameCartPage {
            page: page_id,
            name: c.page_name.trim().to_owned(),
        });
    }
    let limits = &scene.state.config.limits;
    // The grid changes when the slider is released: resizing through
    // smaller sizes on the way would drop the settings of trailing carts.
    let (mut rows, mut cols) = match c.grid_draft {
        Some((id, r, k)) if id == page_id => (r, k),
        _ => (page.rows, page.cols),
    };
    let rows_label = t.tr("settings-cart-rows");
    let cols_label = t.tr("settings-cart-cols");
    let mut commit = false;
    row(ui, &t.tr("settings-cart-grid"), None, |ui| {
        for (value, max, label) in [
            (&mut rows, limits.max_cart_rows, &rows_label),
            (&mut cols, limits.max_cart_cols, &cols_label),
        ] {
            let response = ui.add(egui::Slider::new(value, 1..=max.max(1)).step_by(1.0));
            let v = f64::from(*value);
            let owned = label.clone();
            response.widget_info(|| egui::WidgetInfo::slider(true, v, owned.clone()));
            if response.drag_stopped() || (response.changed() && !response.dragged()) {
                commit = true;
            }
        }
    });
    c.grid_draft = Some((page_id, rows, cols));
    if commit && (rows, cols) != (page.rows, page.cols) {
        scene.ctl.send(Command::ResizeCartPage {
            page: page_id,
            rows,
            cols,
        });
        c.grid_draft = None;
    } else if (rows, cols) == (page.rows, page.cols) {
        c.grid_draft = None;
    }
    ui.add_space(8.0);
    let cell_w = ((ui.available_width() - 6.0 * f32::from(page.cols.max(1) - 1))
        / f32::from(page.cols.max(1)))
    .max(48.0);
    for (r, chunk) in page.carts.chunks(usize::from(page.cols.max(1))).enumerate() {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
            for (k, cart) in chunk.iter().enumerate() {
                let index = r * usize::from(page.cols.max(1)) + k;
                let selected = index == c.index;
                let label = t.tr_args("settings-cart-n", &[("n", (index + 1).into())]);
                let style = TileStyle {
                    fill: theme::NEUTRAL_900,
                    border: if selected {
                        theme::ACCENT
                    } else {
                        theme::NEUTRAL_800
                    },
                    border_width: if selected { 2.0 } else { 1.0 },
                    ..TileStyle::plain()
                };
                let name = if cart.track.is_none() && cart.name.is_empty() {
                    t.tr("cart-empty")
                } else {
                    cart.name.clone()
                };
                if widgets::tile(
                    ui,
                    vec2(cell_w, 36.0),
                    &label,
                    true,
                    style,
                    |p, rect, col| {
                        p.text(
                            rect.left_top() + vec2(6.0, 4.0),
                            egui::Align2::LEFT_TOP,
                            &name,
                            font(11.0),
                            col,
                        );
                        p.text(
                            rect.left_bottom() + vec2(6.0, -4.0),
                            egui::Align2::LEFT_BOTTOM,
                            (index + 1).to_string(),
                            font(10.0),
                            theme::NEUTRAL_500,
                        );
                    },
                )
                .clicked()
                {
                    c.index = index;
                }
            }
        });
        ui.add_space(6.0);
    }
    // The cart editor.
    c.index = c.index.min(page.carts.len().saturating_sub(1));
    let Some(cart) = page.carts.get(c.index) else {
        return;
    };
    ui.add_space(8.0);
    ui.add(
        egui::Label::new(
            RichText::new(
                t.tr_args("settings-cart-n", &[("n", (c.index + 1).into())])
                    .to_uppercase(),
            )
            .font(font(10.0))
            .color(theme::ACCENT_300),
        )
        .selectable(false),
    );
    let edit_base = CartEdit {
        name: cart.name.clone(),
        kind: cart.kind,
        looped: cart.looped,
        exclusive: cart.exclusive,
    };
    let index = c.index;
    let send_edit = |edit: CartEdit| {
        scene.ctl.send(Command::SetCart {
            page: page_id,
            index,
            edit,
        });
    };
    if c.name_for != Some((page_id, c.index)) {
        // A name typed for the previous cart is kept, not dropped.
        if let Some((old_page, old_index)) = c.name_for
            && let Some(old) = wall.page(old_page).and_then(|p| p.carts.get(old_index))
            && old.name != c.name
        {
            scene.ctl.send(Command::SetCart {
                page: old_page,
                index: old_index,
                edit: CartEdit {
                    name: c.name.trim().to_owned(),
                    kind: old.kind,
                    looped: old.looped,
                    exclusive: old.exclusive,
                },
            });
        }
        c.name = cart.name.clone();
        c.name_for = Some((page_id, c.index));
    }
    let name_label = t.tr("settings-cart-name");
    let mut rename = None;
    let response = text_row(ui, &name_label, &mut c.name);
    let cancelled = ui.input(|i| i.key_pressed(egui::Key::Escape));
    if response.lost_focus() && !cancelled && c.name != cart.name {
        rename = Some(c.name.clone());
    }
    if let Some(name) = rename {
        send_edit(CartEdit {
            name,
            ..edit_base.clone()
        });
    }
    let file_label = t.tr("settings-cart-file");
    let current_file = cart
        .track
        .and_then(|id| scene.state.library.get(id))
        .map(|tr| tr.path.display().to_string())
        .unwrap_or_else(|| t.tr("settings-cart-no-file"));
    let mut choose = false;
    let mut clear = false;
    row(ui, &file_label, None, |ui| {
        ui.add(
            egui::Label::new(
                RichText::new(&current_file)
                    .font(font(12.0))
                    .color(theme::NEUTRAL_300),
            )
            .selectable(false)
            .truncate(),
        );
        choose = button(
            ui,
            &t.tr("settings-cart-choose"),
            icon::FOLDER_OPEN,
            c.file.is_none(),
        );
        clear = cart.track.is_some() && button(ui, &t.tr("settings-cart-clear"), icon::X, true);
    });
    if choose {
        let filter = t.tr("dialog-audio-files");
        let dir = scene.state.config.ui.music_dir.clone();
        c.file = background(scene, move || {
            let mut dialog = rfd::AsyncFileDialog::new().add_filter(filter, AUDIO_EXTENSIONS);
            if let Some(dir) = dir {
                dialog = dialog.set_directory(dir);
            }
            pollster::block_on(dialog.pick_file()).map(|f| f.path().to_path_buf())
        });
    }
    if clear {
        scene.ctl.send(Command::ClearCartFile {
            page: page_id,
            index,
        });
    }
    row(ui, &t.tr("settings-cart-kind"), None, |ui| {
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
        for (kind, key) in [
            (CartKind::Jingle, "cart-kind-jingle"),
            (CartKind::Effect, "cart-kind-effect"),
            (CartKind::Spot, "cart-kind-spot"),
        ] {
            let text = t.tr(key);
            let on = cart.kind == kind;
            let style = TileStyle {
                fill: if on {
                    theme::NEUTRAL_700
                } else {
                    Color32::TRANSPARENT
                },
                content: if on { theme::TEXT } else { theme::NEUTRAL_400 },
                ..TileStyle::plain()
            };
            if widgets::tile(ui, vec2(80.0, 28.0), &text, true, style, |p, r, col| {
                p.text(
                    r.center(),
                    egui::Align2::CENTER_CENTER,
                    &text,
                    font(12.0),
                    col,
                );
            })
            .clicked()
                && !on
            {
                send_edit(CartEdit {
                    kind,
                    ..edit_base.clone()
                });
            }
        }
    });
    let mut looped = cart.looped;
    let loop_label = t.tr("settings-cart-loop");
    row(ui, &loop_label, None, |ui| {
        if toggle(ui, &mut looped, &loop_label) {
            send_edit(CartEdit {
                looped,
                ..edit_base.clone()
            });
        }
    });
    let mut exclusive = cart.exclusive;
    let excl_label = t.tr("settings-cart-exclusive");
    row(
        ui,
        &excl_label,
        Some(&t.tr("settings-cart-exclusive-hint")),
        |ui| {
            if toggle(ui, &mut exclusive, &excl_label) {
                send_edit(CartEdit {
                    exclusive,
                    ..edit_base.clone()
                });
            }
        },
    );
}
