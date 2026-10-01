//! The cartwall strip under the players (Phase 2 spec P2.9, v3 layout).

use egui::{Align, Color32, Layout, Rect, RichText, Sense, Stroke, StrokeKind, Ui, pos2, vec2};
use egui_phosphor::regular as icon;
use fp_model::{CartKind, CartPageId, Command};

use super::app::{Scene, ViewState};
use super::cart_view::{CartStatus, cart_view, page_on_air};
use super::theme;
use super::widgets::{self, TileStyle, font, font_medium};

const HEADER_HEIGHT: f32 = 24.0;
const BUTTON_HEIGHT: f32 = 40.0;
const GAP: f32 = 6.0;
const MIN_BUTTON_WIDTH: f32 = 56.0;

/// Height the strip needs, collapsed or open.
pub(crate) fn height(scene: &Scene<'_>) -> f32 {
    let wall = &scene.state.cartwall;
    let rows = wall.shown_page().map_or(0, |p| p.rows);
    if wall.open && rows > 0 {
        HEADER_HEIGHT + GAP + f32::from(rows) * BUTTON_HEIGHT + f32::from(rows - 1) * GAP
    } else {
        HEADER_HEIGHT
    }
}

/// The name shown for a page: its own, or "Carts n" while it has none.
pub(crate) fn page_name(scene: &Scene<'_>, index: usize, name: &str) -> String {
    if name.trim().is_empty() {
        scene
            .i18n
            .tr_args("cart-page-default", &[("n", (index + 1).into())])
    } else {
        name.to_owned()
    }
}

pub(crate) fn strip(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState) {
    header(ui, scene);
    if !scene.state.cartwall.open {
        return;
    }
    ui.add_space(GAP);
    // A grid larger than the strip scrolls instead of being cut off.
    egui::ScrollArea::both()
        .id_salt("cart-grid")
        .auto_shrink([false, false])
        .show(ui, |ui| grid(ui, scene, view_state));
}

fn header(ui: &mut Ui, scene: &Scene<'_>) {
    let t = scene.i18n;
    let wall = &scene.state.cartwall;
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), HEADER_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing = vec2(10.0, 0.0);
            let title = t.tr("cartwall-title");
            let caret = if wall.open {
                icon::CARET_DOWN
            } else {
                icon::CARET_RIGHT
            };
            let width = ui
                .painter()
                .layout_no_wrap(title.clone(), font(10.0), theme::NEUTRAL_300)
                .size()
                .x
                + 24.0;
            let style = TileStyle {
                border: Color32::TRANSPARENT,
                hover_fill: Color32::TRANSPARENT,
                active_fill: Color32::TRANSPARENT,
                ..TileStyle::plain()
            };
            if widgets::tile(ui, vec2(width, 20.0), &title, true, style, |p, r, c| {
                p.text(
                    r.left_center(),
                    egui::Align2::LEFT_CENTER,
                    format!("{caret} {title}"),
                    font(10.0),
                    c,
                );
            })
            .clicked()
            {
                scene.ctl.send(Command::SetCartwallOpen(!wall.open));
            }
            tabs(ui, scene);
            ui.add(
                egui::Label::new(
                    RichText::new(t.tr("cartwall-hint"))
                        .font(font(10.0))
                        .color(theme::NEUTRAL_500),
                )
                .selectable(false)
                .truncate(),
            );
        },
    );
}

fn tabs(ui: &mut Ui, scene: &Scene<'_>) {
    let wall = &scene.state.cartwall;
    let shown = wall.shown_page().map(|p| p.id);
    let pages: Vec<(CartPageId, String, bool)> = wall
        .pages
        .iter()
        .enumerate()
        .map(|(i, p)| {
            (
                p.id,
                page_name(scene, i, &p.name),
                page_on_air(scene.state, p.id),
            )
        })
        .collect();
    let frame = ui.available_rect_before_wrap();
    let max_width = (frame.width() * 0.6).max(120.0);
    egui::ScrollArea::horizontal()
        .id_salt("cart-tabs")
        .max_width(max_width)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                for (id, name, on_air) in pages {
                    let on = Some(id) == shown;
                    let width = ui
                        .painter()
                        .layout_no_wrap(name.clone(), font(11.0), theme::TEXT)
                        .size()
                        .x
                        + 32.0;
                    let style = TileStyle {
                        fill: theme::NEUTRAL_900,
                        border: theme::NEUTRAL_800,
                        content: if on { theme::TEXT } else { theme::NEUTRAL_500 },
                        hover_fill: theme::NEUTRAL_800,
                        ..TileStyle::plain()
                    };
                    let response =
                        widgets::tile(ui, vec2(width, 22.0), &name, true, style, |p, r, c| {
                            if on_air {
                                p.rect_filled(
                                    Rect::from_center_size(
                                        pos2(r.left() + 12.0, r.center().y),
                                        vec2(6.0, 6.0),
                                    ),
                                    0.0,
                                    theme::ON_AIR_TEXT,
                                );
                            }
                            p.text(
                                pos2(r.left() + 20.0, r.center().y),
                                egui::Align2::LEFT_CENTER,
                                &name,
                                font(11.0),
                                c,
                            );
                            if on {
                                p.rect_filled(
                                    Rect::from_min_size(
                                        pos2(r.left(), r.bottom() - 2.0),
                                        vec2(r.width(), 2.0),
                                    ),
                                    0.0,
                                    theme::ACCENT,
                                );
                            }
                        });
                    if response.clicked() && !on {
                        scene.ctl.send(Command::ShowCartPage(id));
                    }
                }
            });
        });
}

fn kind_color(kind: CartKind) -> Color32 {
    match kind {
        CartKind::Jingle => theme::ACCENT,
        CartKind::Effect => theme::AMBER,
        CartKind::Spot => theme::NEUTRAL_400,
    }
}

fn kind_label(scene: &Scene<'_>, kind: CartKind) -> String {
    scene.i18n.tr(match kind {
        CartKind::Jingle => "cart-kind-jingle",
        CartKind::Effect => "cart-kind-effect",
        CartKind::Spot => "cart-kind-spot",
    })
}

fn grid(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState) {
    let t = scene.i18n;
    let Some(page) = scene.state.cartwall.shown_page() else {
        return;
    };
    let cols = usize::from(page.cols.max(1));
    let width =
        ((ui.available_width() - GAP * (cols as f32 - 1.0)) / cols as f32).max(MIN_BUTTON_WIDTH);
    let page_id = page.id;
    for (row_index, row) in page.carts.chunks(cols).enumerate() {
        if row_index > 0 {
            ui.add_space(GAP);
        }
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(GAP, 0.0);
            for (col_index, cart) in row.iter().enumerate() {
                let index = row_index * cols + col_index;
                let Some(view) = cart_view(scene.state, scene.telemetry, cart.id) else {
                    continue;
                };
                let empty = view.status == CartStatus::Empty;
                let label = if empty {
                    t.tr_args("cart-empty-n", &[("n", (index + 1).into())])
                } else {
                    view.name.clone()
                };
                let shown = if empty {
                    t.tr("cart-empty")
                } else {
                    label.clone()
                };
                let playing = view.status == CartStatus::Playing;
                let style = TileStyle {
                    fill: theme::NEUTRAL_900,
                    border: if playing {
                        theme::ON_AIR_TEXT
                    } else {
                        theme::NEUTRAL_800
                    },
                    content: theme::TEXT,
                    hover_fill: theme::NEUTRAL_800,
                    hover_content: theme::TEXT,
                    active_fill: theme::NEUTRAL_700,
                    border_width: 1.0,
                };
                let kind = kind_label(scene, view.kind);
                let response = widgets::tile(
                    ui,
                    vec2(width, BUTTON_HEIGHT),
                    &label,
                    !empty,
                    style,
                    |p, r, c| {
                        if playing {
                            let bar = Rect::from_min_size(
                                r.min,
                                vec2(r.width() * view.remaining_fraction, r.height()),
                            );
                            p.rect_filled(bar, 0.0, theme::ON_AIR_ROW.gamma_multiply(0.5));
                        }
                        let inner = r.shrink2(vec2(8.0, 4.0));
                        let dot = match view.status {
                            CartStatus::Empty => theme::NEUTRAL_700,
                            CartStatus::Unavailable => theme::AMBER,
                            _ => kind_color(view.kind),
                        };
                        p.rect_filled(
                            Rect::from_min_size(
                                pos2(inner.left(), inner.top() + 4.0),
                                vec2(6.0, 6.0),
                            ),
                            0.0,
                            dot,
                        );
                        let title = match view.status {
                            CartStatus::Unavailable => format!("{} {shown}", icon::WARNING),
                            _ => shown.clone(),
                        };
                        let name_color = if empty { theme::NEUTRAL_600 } else { c };
                        let galley =
                            p.layout(title, font_medium(11.0), name_color, inner.width() - 12.0);
                        p.with_clip_rect(inner).galley(
                            pos2(inner.left() + 12.0, inner.top()),
                            galley,
                            name_color,
                        );
                        if !empty {
                            let mut left = kind.clone();
                            if view.looped {
                                left.push_str(&format!(" {}", icon::REPEAT));
                            }
                            if view.exclusive {
                                left.push_str(&format!(" {}", icon::HAND));
                            }
                            p.text(
                                inner.left_bottom(),
                                egui::Align2::LEFT_BOTTOM,
                                left,
                                font(10.0),
                                theme::NEUTRAL_500,
                            );
                            p.text(
                                inner.right_bottom(),
                                egui::Align2::RIGHT_BOTTOM,
                                &view.time,
                                font(10.0),
                                if playing {
                                    theme::NEUTRAL_100
                                } else {
                                    theme::NEUTRAL_500
                                },
                            );
                        }
                        if view.cueing {
                            p.rect_stroke(
                                r.shrink(1.0),
                                0.0,
                                Stroke::new(1.0, theme::CUE),
                                StrokeKind::Inside,
                            );
                        }
                    },
                );
                let response = match cart.track.and_then(|t| scene.file_tip(t)) {
                    Some(tip) => response.on_hover_text(tip),
                    None => response,
                };
                if response.clicked() {
                    scene.ctl.send(Command::FireCart(cart.id));
                }
                // An empty cart looks disabled and does not fire, but its
                // menu still edits it (to choose its file).
                let menu_on = if empty {
                    let area = ui.interact(response.rect, response.id.with("menu"), Sense::click());
                    // Its own name: it only opens the menu to choose a file.
                    let name = t.tr_args("cart-empty-edit", &[("n", (index + 1).into())]);
                    area.widget_info(|| {
                        egui::WidgetInfo::labeled(egui::WidgetType::Button, true, name.clone())
                    });
                    area
                } else {
                    response.clone()
                };
                {
                    menu_on.context_menu(|ui| {
                        ui.set_min_width(200.0);
                        let item = |ui: &mut Ui, glyph: &str, key: &str| {
                            let text = t.tr(key);
                            let r = ui.button(format!("{glyph}  {text}"));
                            r.widget_info(|| {
                                egui::WidgetInfo::labeled(
                                    egui::WidgetType::Button,
                                    true,
                                    text.clone(),
                                )
                            });
                            r
                        };
                        if !empty && item(ui, icon::HEADPHONES, "menu-cue").clicked() {
                            scene.ctl.send(Command::CueCart(cart.id));
                            ui.close();
                        }
                        if !empty && item(ui, egui_phosphor::fill::STOP, "menu-cart-stop").clicked()
                        {
                            scene.ctl.send(Command::StopCart(cart.id));
                            ui.close();
                        }
                        if item(ui, icon::PENCIL_SIMPLE, "menu-cart-edit").clicked() {
                            view_state.edit_cart = Some((page_id, index));
                            view_state.settings_open = true;
                            ui.close();
                        }
                    });
                }
            }
        });
    }
}
