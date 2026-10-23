//! The About window (feedback spec §3.1, F21): the version, the author
//! attribution the licence's section 7(b) term requires, the licence, the
//! disclaimer and the notices the licences of the bundled components require.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use egui::{RichText, ScrollArea, Ui, vec2};

use super::app::Scene;
use super::theme;
use super::widgets::{self, TileStyle, font, font_medium};

/// The version shown in the top bar and the About window.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The Inter font's copyright line and licence, shipped with the binary.
const INTER_OFL: &str = include_str!("../../assets/fonts/OFL.txt");
/// Phosphor Icons' copyright and permission notice.
const PHOSPHOR_MIT: &str = include_str!("../../assets/licenses/Phosphor-MIT.txt");

/// The name of the third-party notices file release builds install.
const NOTICES_FILE: &str = "THIRD-PARTY.html";

/// The author attribution the licence requires (see `NOTICE`); never translated.
pub const ATTRIBUTION: &str = "Fauste Player by Marc Sánchez Fauste";
/// The project page, the target of the attribution link.
pub const REPO_URL: &str = "https://github.com/sanchezfauste/fauste-player";
/// The text of the licence, read online.
pub const LICENSE_URL: &str = "https://www.gnu.org/licenses/gpl-3.0.html";

/// Opens a file or a web address (the notices file, the links). The default
/// hands it to the system on a helper thread; tests record the call.
pub type Opener = Arc<dyn Fn(&Path) + Send + Sync>;

/// Where the release packages install the notices, relative to the
/// directory of the executable.
pub fn notice_candidates(exe_dir: &Path) -> Vec<PathBuf> {
    vec![
        // Release archives and the Windows installer.
        exe_dir.join("licenses").join(NOTICES_FILE),
        // The macOS app bundle.
        exe_dir.join("../Resources/licenses").join(NOTICES_FILE),
        // deb, rpm and AppImage.
        exe_dir
            .join("../share/doc/fauste-player")
            .join(NOTICES_FILE),
        // Flatpak.
        exe_dir
            .join("../share/licenses/org.fauste.FaustePlayer")
            .join(NOTICES_FILE),
    ]
}

/// The installed notices file, if any. It reads the file system, so it is
/// called once at start-up, never from a frame.
pub fn find_notices(exe: &Path) -> Option<PathBuf> {
    let dir = exe.parent()?;
    notice_candidates(dir).into_iter().find(|p| p.is_file())
}

/// Opens a file with the system's default application on a helper thread,
/// so the interface never waits for it.
pub fn system_opener() -> Opener {
    Arc::new(|path: &Path| {
        let path = path.to_owned();
        let spawned = std::thread::Builder::new()
            .name("fp-open".to_owned())
            .spawn(move || {
                if let Err(error) = open::that_detached(&path) {
                    tracing::warn!(%error, path = %path.display(), "could not open the file or link");
                }
            });
        if let Err(error) = spawned {
            tracing::warn!(%error, "could not start the opener");
        }
    })
}

fn text(ui: &mut Ui, value: String, size: f32, color: egui::Color32) {
    ui.add(
        egui::Label::new(RichText::new(value).font(font(size)).color(color))
            .selectable(false)
            .wrap(),
    );
}

/// A plain text tile; true when it was clicked.
fn button(ui: &mut Ui, label: &str, enabled: bool) -> bool {
    let w = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font(12.0), theme::NEUTRAL_300)
        .size()
        .x
        + 24.0;
    widgets::tile(
        ui,
        vec2(w, 24.0),
        label,
        enabled,
        TileStyle::plain(),
        |p, r, c| {
            p.text(
                r.center(),
                egui::Align2::CENTER_CENTER,
                label,
                font(12.0),
                c,
            );
        },
    )
    .clicked()
}

fn notice(ui: &mut Ui, title: &str, body: &str) {
    egui::CollapsingHeader::new(
        RichText::new(title)
            .font(font_medium(12.0))
            .color(theme::TEXT),
    )
    .id_salt(title)
    .show(ui, |ui| {
        ui.add(
            egui::Label::new(
                RichText::new(body)
                    .font(egui::FontId::monospace(10.0))
                    .color(theme::NEUTRAL_400),
            )
            .wrap(),
        );
    });
}

/// Draws the About window. Returns whether it stays open.
pub(crate) fn show(
    ctx: &egui::Context,
    scene: &Scene<'_>,
    notices: Option<&Path>,
    opener: &Opener,
) -> bool {
    let t = scene.i18n;
    let mut open = true;
    let screen = ctx.content_rect();
    let width = (screen.width() - 48.0).clamp(320.0, 560.0);
    let height = (screen.height() - 82.0).clamp(300.0, 560.0);
    egui::Modal::new(egui::Id::unique("about"))
        .frame(egui::Frame::new().fill(theme::SURFACE).inner_margin(20.0))
        .backdrop_color(theme::NEUTRAL_900.gamma_multiply(0.7))
        .show(ctx, |ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            // The whole body scrolls, so the window fits the smallest height;
            // Close stays below it.
            ScrollArea::vertical()
                .max_height((height - 60.0).max(120.0))
                .show(ui, |ui| {
                    ui.add(
                        egui::Label::new(
                            RichText::new(t.tr("app-name"))
                                .font(font_medium(18.0))
                                .color(theme::TEXT),
                        )
                        .selectable(false),
                    );
                    text(
                        ui,
                        t.tr_args("about-version", &[("version", VERSION.into())]),
                        12.0,
                        theme::NEUTRAL_400,
                    );
                    text(ui, ATTRIBUTION.to_owned(), 12.0, theme::NEUTRAL_300);
                    if button(ui, REPO_URL, true) {
                        opener(Path::new(REPO_URL));
                    }
                    text(ui, t.tr("about-copyright"), 12.0, theme::NEUTRAL_300);
                    text(ui, t.tr("about-license"), 12.0, theme::NEUTRAL_300);
                    if button(ui, &t.tr("about-license-link"), true) {
                        opener(Path::new(LICENSE_URL));
                    }
                    text(ui, t.tr("about-disclaimer"), 12.0, theme::NEUTRAL_400);
                    ui.separator();
                    text(ui, t.tr("about-bundled"), 12.0, theme::NEUTRAL_300);
                    notice(ui, &t.tr("about-inter"), INTER_OFL);
                    notice(ui, &t.tr("about-phosphor"), PHOSPHOR_MIT);
                    text(ui, t.tr("about-crates"), 12.0, theme::NEUTRAL_300);
                    ui.horizontal(|ui| {
                        let clicked = button(ui, &t.tr("about-third-party"), notices.is_some());
                        match notices {
                            Some(path) if clicked => opener(path),
                            Some(_) => {}
                            None => text(
                                ui,
                                t.tr("about-third-party-missing"),
                                11.0,
                                theme::NEUTRAL_500,
                            ),
                        }
                    });
                });
            ui.add_space(4.0);
            if button(ui, &t.tr("about-close"), true) {
                open = false;
            }
        });
    open
}
