//! `android_main`: the session lives at process level and each activity
//! only builds an interface on it (mobile spec MB2). When the activity is
//! destroyed eframe returns and the audio goes on; the next activity calls
//! `android_main` again and re-attaches.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, OnceLock};

use fp_app::session::AppSession;
use fp_app::ui::shell::Shell;
use fp_app::ui::theme;
use fp_store::AppPaths;
use winit::platform::android::activity::AndroidApp;
use winit::platform::android::activity::input::{TextInputState, TextSpan};

use crate::ime::ImeBridge;
use crate::insets::{Insets, safe_area};
use crate::platform::{AndroidPlatform, MEDIA};

/// The running session; started by the first activity, never stopped (the
/// process ends with it).
pub(crate) static SESSION: OnceLock<AppSession> = OnceLock::new();

/// The insets the Kotlin shell last reported (`f32` bits, physical pixels):
/// left, top, right, bottom.
pub(crate) static INSETS: [AtomicU32; 4] = [
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
];

fn insets() -> Insets {
    let side = |i: usize| f32::from_bits(INSETS[i].load(Ordering::Relaxed));
    Insets {
        left: side(0),
        top: side(1),
        right: side(2),
        bottom: side(3),
    }
}

/// The shared interface, drawn inside the safe area, with the keyboard's
/// text bridged to egui.
struct MobileApp {
    shell: Shell,
    app: AndroidApp,
    ime: ImeBridge,
}

impl eframe::App for MobileApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.shell.logic(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let screen = ui.max_rect();
        ui.painter().rect_filled(screen, 0.0, theme::BG);
        let area = safe_area(screen, insets(), ui.ctx().pixels_per_point());
        ui.scope_builder(egui::UiBuilder::new().max_rect(area), |ui| {
            self.shell.ui(ui);
        });
    }

    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        let focused = ctx.memory(|m| m.focused());
        if self.ime.restart_for(focused, &raw.events) {
            // The keyboard starts empty for the field (or after a tap that
            // may have moved the cursor): what it held belongs elsewhere.
            self.app.set_text_input_state(TextInputState {
                text: String::new(),
                selection: TextSpan { start: 0, end: 0 },
                compose_region: None,
            });
            return;
        }
        let text = self.app.text_input_state().text;
        raw.events.extend(self.ime.update(&text));
    }
}

/// The session, started on the first call.
fn session(app: &AndroidApp) -> Option<&'static AppSession> {
    if let Some(session) = SESSION.get() {
        tracing::info!("re-attaching to the running session");
        return Some(session);
    }
    let Some(root) = app.internal_data_path() else {
        tracing::error!("no internal data path; cannot start");
        return None;
    };
    let paths = AppPaths::under(&root);
    let cap = fp_app::crash::install_panic_hook(
        paths.log_dir.clone(),
        fp_model::Limits::default().max_crash_reports,
    );
    match AppSession::start(paths, fp_backends::system_backends(), Some(cap)) {
        Ok(session) => Some(SESSION.get_or_init(|| session)),
        Err(error) => {
            tracing::error!(%error, "could not start");
            None
        }
    }
}

#[allow(unsafe_code)]
// SAFETY: `android_main` is the entry point android-activity looks up by
// name; no other symbol in the process has it.
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    use tracing_subscriber::prelude::*;
    let _ = tracing_subscriber::registry()
        .with(paranoid_android::layer("fauste"))
        .with(tracing_subscriber::EnvFilter::new("info"))
        .try_init();
    tracing::info!(version = env!("CARGO_PKG_VERSION"), "activity starting");
    let Some(session) = session(&app) else {
        return;
    };
    // A picker of an earlier activity waits on channels that went with its
    // interface: answer it, so this one does not show a request no one gets.
    MEDIA.cancel_pending();
    let ui = session
        .app_ui()
        .with_platform(Arc::new(AndroidPlatform))
        .with_media_browser(MEDIA.clone());
    let options = eframe::NativeOptions {
        android_app: Some(app.clone()),
        ..Default::default()
    };
    let result = eframe::run_native(
        "fauste-player",
        options,
        Box::new(move |cc| {
            theme::apply(&cc.egui_ctx);
            Ok(Box::new(MobileApp {
                shell: Shell::new(ui),
                app,
                ime: ImeBridge::new(),
            }))
        }),
    );
    // The session goes on: the foreground service holds the process (MB1).
    match result {
        Ok(()) => tracing::info!("activity ended"),
        Err(error) => tracing::error!(%error, "the activity's interface failed"),
    }
}
