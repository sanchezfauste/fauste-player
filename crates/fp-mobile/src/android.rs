//! `android_main`: the session lives at process level and each activity
//! only builds an interface on it (mobile spec MB2). When the activity is
//! destroyed eframe returns and the audio goes on; the next activity calls
//! `android_main` again and re-attaches.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

use fp_app::platform::Platform;
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

/// The Java VM and the current activity (android-activity's global
/// reference, valid while its `AndroidApp` lives), for calls into the
/// activity (`jni::open_url`). Set while an activity runs; the lock is held
/// through a call, so the activity cannot end in the middle of one.
pub(crate) static ACTIVITY: Mutex<Option<(usize, usize)>> = Mutex::new(None);

/// Set by `Native.back`: the next frame goes back one step (PL2).
pub(crate) static BACK: AtomicBool = AtomicBool::new(false);

/// The running interface's context, so `Native.back` can wake it.
pub(crate) static REPAINT: Mutex<Option<egui::Context>> = Mutex::new(None);

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
        // Back goes back inside the interface first; with nothing left the
        // app leaves: to the background while something sounds, otherwise
        // for good (PL23, PL24). A loaded but stopped player does not hold
        // it.
        if BACK.swap(false, Ordering::AcqRel) && !self.shell.back(ctx) {
            let playing = SESSION
                .get()
                .is_some_and(|s| crate::shell_state::sounding(&s.handle.model.load()));
            AndroidPlatform.leave(playing);
        }
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
        Ok(session) => {
            let session = SESSION.get_or_init(|| session);
            // The media list may have come before the session: its
            // recheck (`jni::setMediaFiles`) found no one to ask.
            if !crate::platform::MEDIA.files().is_empty() {
                let _ = session
                    .requests
                    .try_send(fp_app::services::ServiceRequest::RecheckUnreadable);
            }
            Some(session)
        }
        Err(error) => {
            tracing::error!(%error, "could not start");
            None
        }
    }
}

/// The third-party notices the activity copies out of the APK into its
/// files folder (phone layout spec PL22); a debug APK has none. It reads
/// the file system, so it is called once as the activity starts.
fn notices(app: &AndroidApp) -> Option<PathBuf> {
    let path = app
        .internal_data_path()?
        .join("licenses")
        .join("THIRD-PARTY.html");
    path.is_file().then_some(path)
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
        .with_touch()
        .with_media_browser(MEDIA.clone())
        .with_notices(notices(&app));
    let this = Some((app.vm_as_ptr() as usize, app.activity_as_ptr() as usize));
    *ACTIVITY.lock().unwrap_or_else(PoisonError::into_inner) = this;
    let options = eframe::NativeOptions {
        android_app: Some(app.clone()),
        ..Default::default()
    };
    let result = eframe::run_native(
        "fauste-player",
        options,
        Box::new(move |cc| {
            theme::apply(&cc.egui_ctx);
            *REPAINT.lock().unwrap_or_else(PoisonError::into_inner) = Some(cc.egui_ctx.clone());
            // A Back that came with no interface to take it is not this one's.
            BACK.store(false, Ordering::Release);
            Ok(Box::new(MobileApp {
                shell: Shell::new(ui),
                app,
                ime: ImeBridge::new(),
            }))
        }),
    );
    // The activity reference dies with this `AndroidApp`; a newer
    // activity's entry stays.
    {
        let mut current = ACTIVITY.lock().unwrap_or_else(PoisonError::into_inner);
        if *current == this {
            *current = None;
            *REPAINT.lock().unwrap_or_else(PoisonError::into_inner) = None;
        }
    }
    // The session goes on: the foreground service holds the process (MB1).
    match result {
        Ok(()) => tracing::info!("activity ended"),
        Err(error) => tracing::error!(%error, "the activity's interface failed"),
    }
}
