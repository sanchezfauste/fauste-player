//! SPIKE ONLY (mobile plan 1): the desktop session inside `android_main`.
//! The session lives at process level; each activity only builds a UI on it.
#![cfg(target_os = "android")]

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use cpal::traits::{DeviceTrait, HostTrait};
use crossbeam_channel::Sender;
use fp_analysis::analyzer::Analyzer;
use fp_analysis::cache::AnalysisCache;
use fp_app::activity::ActivityCell;
use fp_app::i18n::I18n;
use fp_app::services::{MediaCache, ServiceRequest, Services, ServicesHandle};
use fp_app::ui::app::AppUi;
use fp_app::ui::shell::Shell;
use fp_app::ui::theme;
use fp_backends::{AudioBackend, NullBackend, system_backends};
use fp_engine::conductor::{Conductor, ConductorHandle};
use fp_engine::engine::{Engine, EngineSettings};
use fp_engine::worker::file_opener;
use fp_store::{AppPaths, Store};
use winit::platform::android::activity::AndroidApp;

struct Session {
    handle: Arc<ConductorHandle>,
    requests: Sender<ServiceRequest>,
    activity: ActivityCell,
    faults: Arc<AtomicU64>,
    media: MediaCache,
    backends: Vec<Arc<dyn AudioBackend>>,
    language: Option<String>,
    _services: ServicesHandle,
}

static SESSION: OnceLock<Session> = OnceLock::new();
static ATTACHED: AtomicUsize = AtomicUsize::new(0);

fn start(paths: AppPaths) -> Result<Session, Box<dyn std::error::Error>> {
    let store = Store::new(paths.clone(), fp_model::Limits::default());
    let loaded = store.load(&I18n::new(None).tr("default-playlist-name"));
    for warning in &loaded.warnings {
        tracing::warn!(%warning, "while loading the saved state");
    }
    let config = loaded.state.config.clone();
    let mut backends = system_backends();
    backends.push(Arc::new(NullBackend) as Arc<dyn AudioBackend>);
    let engine = Engine::new(
        backends.clone(),
        EngineSettings::from_config(&config),
        file_opener(),
    );
    tracing::info!(backend = %engine.backend_in_use(), "audio system");
    let (conductor, handle) = Conductor::new(loaded.state, loaded.actions, engine, Instant::now());
    let handle = Arc::new(conductor.spawn(handle)?);
    let analyzer = Analyzer::spawn(
        2,
        config.analysis.clone(),
        config.limits.clone(),
        Some(AnalysisCache::new(paths.cache_dir.join("analysis"))),
    )?;
    let media = MediaCache::default();
    let services = Services::new(handle.clone(), store, analyzer, media.clone());
    let requests = services.requests();
    let activity = services.activity();
    let faults = services.faults();
    Ok(Session {
        handle,
        requests,
        activity,
        faults,
        media,
        backends,
        language: config.ui.language.clone(),
        _services: services.spawn()?,
    })
}

fn log_devices() {
    for host_id in cpal::available_hosts() {
        let Ok(host) = cpal::host_from_id(host_id) else {
            continue;
        };
        let Ok(devices) = host.output_devices() else {
            tracing::warn!(?host_id, "no output devices");
            continue;
        };
        for device in devices {
            let configs: Vec<String> = device
                .supported_output_configs()
                .map(|c| c.map(|c| format!("{c:?}")).collect())
                .unwrap_or_default();
            tracing::info!(?host_id, device = ?device.description(), ?configs, "output device");
        }
    }
}

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    use tracing_subscriber::prelude::*;
    let _ = tracing_subscriber::registry()
        .with(paranoid_android::layer("fauste"))
        .try_init();
    let session = match SESSION.get() {
        Some(s) => {
            tracing::info!("reusing the running session");
            s
        }
        None => {
            log_devices();
            let Some(root) = app.internal_data_path() else {
                tracing::error!("no internal data path");
                return;
            };
            match start(AppPaths::under(&root)) {
                Ok(s) => SESSION.get_or_init(|| s),
                Err(e) => {
                    tracing::error!(error = %e, "could not start");
                    return;
                }
            }
        }
    };
    let n = ATTACHED.fetch_add(1, Ordering::Relaxed) + 1;
    tracing::info!(n, "activity attached");
    let ui = AppUi::new(
        session.handle.clone(),
        I18n::new(session.language.as_deref()),
        session.media.clone(),
    )
    .with_services(session.requests.clone())
    .with_analysis_activity(session.activity.clone())
    .with_service_faults(session.faults.clone())
    .with_backends(session.backends.clone());
    let options = eframe::NativeOptions {
        android_app: Some(app),
        ..Default::default()
    };
    let result = eframe::run_native(
        "fauste-player",
        options,
        Box::new(move |cc| {
            theme::apply(&cc.egui_ctx);
            Ok(Box::new(Shell::new(ui)))
        }),
    );
    // The engine keeps running: the foreground service holds the process.
    tracing::info!(n, ?result, "activity detached");
}
