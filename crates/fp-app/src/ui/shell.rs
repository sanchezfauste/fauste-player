//! Panic isolation for the interface (spec §8.5). Each frame is drawn
//! inside `catch_unwind`; a panic leaves audio untouched and replaces the
//! screen with a banner until the user restarts the interface.

use std::panic::{AssertUnwindSafe, catch_unwind};

use egui::{Align, Layout, RichText, Ui, vec2};

use super::app::AppUi;
use super::theme;
use super::widgets::{self, TileStyle, font, font_medium};

pub struct Shell {
    app: AppUi,
    degraded: bool,
}

impl Shell {
    pub fn new(app: AppUi) -> Self {
        Self {
            app,
            degraded: false,
        }
    }

    pub fn app_mut(&mut self) -> &mut AppUi {
        &mut self.app
    }

    /// True after a frame panicked, until "Restart interface".
    pub fn degraded(&self) -> bool {
        self.degraded
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        if !self.degraded {
            let app = &mut self.app;
            let outcome = catch_unwind(AssertUnwindSafe(|| app.ui(ui)));
            if outcome.is_err() {
                tracing::error!("a frame of the interface panicked; showing the recovery banner");
                self.degraded = true;
                ui.ctx().request_repaint();
            }
            return;
        }
        self.banner(ui);
    }

    /// Runs before every frame, shown or not (see `AppUi::guard_close`).
    pub fn logic(&mut self, ctx: &egui::Context) {
        // In degraded mode the dialog cannot be drawn, and a cancelled
        // close with no dialog would trap the operator: do not guard.
        if !self.degraded {
            self.app.guard_close(ctx);
        }
    }

    fn banner(&mut self, ui: &mut Ui) {
        let rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(rect, 0.0, theme::BG);
        let i18n = self.app.i18n();
        let message = i18n.tr("ui-crashed");
        let restart = i18n.tr("ui-restart");
        ui.allocate_ui_with_layout(
            vec2(rect.width(), 44.0),
            Layout::left_to_right(Align::Center),
            |ui| {
                ui.painter().rect_filled(
                    ui.max_rect(),
                    0.0,
                    theme::ON_AIR_ROW.gamma_multiply(0.35),
                );
                ui.spacing_mut().item_spacing = vec2(12.0, 0.0);
                ui.add_space(16.0);
                ui.add(
                    egui::Label::new(
                        RichText::new(message)
                            .font(font_medium(13.0))
                            .color(theme::TEXT),
                    )
                    .selectable(false),
                );
                let width = ui
                    .painter()
                    .layout_no_wrap(restart.clone(), font(12.0), theme::TEXT)
                    .size()
                    .x
                    + 24.0;
                if widgets::tile(
                    ui,
                    vec2(width, 28.0),
                    &restart,
                    true,
                    TileStyle::plain(),
                    |p, r, c| {
                        p.text(
                            r.center(),
                            egui::Align2::CENTER_CENTER,
                            &restart,
                            font(12.0),
                            c,
                        );
                    },
                )
                .clicked()
                {
                    self.app.reset_view();
                    self.degraded = false;
                    ui.ctx().request_repaint();
                }
            },
        );
    }
}

impl eframe::App for Shell {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        Shell::logic(self, ctx);
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        Shell::ui(self, ui);
    }
}
