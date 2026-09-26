//! Fauste Player: starts logging, loads the saved state, builds the audio
//! engine, and runs the conductor, analysis, services and interface.

use std::process::ExitCode;
use std::sync::Arc;
use std::time::{Duration, Instant};

use fp_analysis::analyzer::Analyzer;
use fp_analysis::cache::AnalysisCache;
use fp_app::i18n::I18n;
use fp_app::services::{MediaCache, Services};
use fp_app::ui::app::AppUi;
use fp_app::ui::shell::Shell;
use fp_app::ui::theme;
use fp_app::{bootstrap, crash, logging};
use fp_backends::{
    AudioBackend, Availability, NullBackend, choose_default_backend, display_name, system_backends,
};
use fp_engine::conductor::Conductor;
use fp_engine::engine::{Engine, EngineSettings};
use fp_engine::worker::file_opener;
use fp_store::Store;

/// Background analysis threads: enough to keep up with imports without
/// competing with the audio threads.
const ANALYSIS_THREADS: usize = 2;

fn main() -> ExitCode {
    let Some(paths) = bootstrap::paths() else {
        eprintln!("fauste-player: no home directory found; set FAUSTE_HOME");
        return ExitCode::FAILURE;
    };
    let _log = logging::init(&paths.log_dir);
    crash::install_panic_hook(
        paths.log_dir.clone(),
        fp_model::Limits::default().max_crash_reports,
    );
    tracing::info!(version = env!("CARGO_PKG_VERSION"), "starting");
    match run(paths) {
        Ok(()) => {
            tracing::info!("stopped");
            ExitCode::SUCCESS
        }
        Err(e) => {
            tracing::error!(error = %e, "could not start");
            eprintln!("fauste-player: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(paths: fp_store::AppPaths) -> Result<(), Box<dyn std::error::Error>> {
    let store = Store::new(paths.clone(), fp_model::Limits::default());
    let default_name = I18n::new(None).tr("default-playlist-name");
    let loaded = store.load(&default_name);
    for warning in &loaded.warnings {
        tracing::warn!(%warning, "while loading the saved state");
    }
    let config = loaded.state.config.clone();
    let i18n = I18n::new(config.ui.language.as_deref());

    // Every audio system this build supports, plus Null as the last resort.
    let mut backends = system_backends();
    backends.push(Arc::new(NullBackend) as Arc<dyn AudioBackend>);
    let availability: Vec<(String, bool)> = backends
        .iter()
        .map(|b| (b.id().0, b.availability() == Availability::Available))
        .collect();
    let listed: Vec<(&str, bool)> = availability
        .iter()
        .filter(|(id, _)| id != "null")
        .map(|(id, ok)| (id.as_str(), *ok))
        .collect();
    let mut settings = EngineSettings::from_config(&config);
    let chosen = choose_default_backend(
        settings.default_backend.as_deref(),
        &listed,
        std::env::consts::OS,
    )
    .map(str::to_owned);
    if settings.default_backend.is_some() && settings.default_backend != chosen {
        tracing::warn!(
            backend = ?settings.default_backend,
            fallback = ?chosen,
            "configured audio system unavailable"
        );
    }
    settings.default_backend = chosen;
    let in_use = settings
        .default_backend
        .clone()
        .unwrap_or_else(|| "null".to_owned());
    tracing::info!(backend = %in_use, ?availability, "audio systems");
    let platform = format!("{} · {}", os_name(), display_name(&in_use));
    let engine = Engine::new(backends.clone(), settings, file_opener());
    let (conductor, handle) = Conductor::new(loaded.state, loaded.actions, engine, Instant::now());
    let tick = Duration::from_secs_f64(config.tuning.conductor_tick_ms.max(1.0) / 1000.0);
    let handle = Arc::new(conductor.spawn(handle, tick)?);

    let cache = AnalysisCache::new(paths.cache_dir.join("analysis"), &config.limits);
    let analyzer = Analyzer::spawn(
        ANALYSIS_THREADS,
        config.analysis.clone(),
        config.limits.clone(),
        Some(cache),
    )?;
    let media = MediaCache::default();
    let services = Services::new(handle.clone(), store, analyzer, media.clone());
    let requests = services.requests();
    let faults = services.faults();
    let services = services.spawn()?;

    let app = AppUi::new(handle.clone(), i18n, media)
        .with_services(requests)
        .with_service_faults(faults)
        .with_backends(backends)
        .with_platform(platform);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Fauste Player")
            .with_app_id("fauste-player")
            .with_inner_size([1600.0, 940.0])
            .with_min_inner_size([420.0, 480.0]),
        ..Default::default()
    };
    let result = eframe::run_native(
        "fauste-player",
        options,
        Box::new(move |cc| {
            theme::apply(&cc.egui_ctx);
            Ok(Box::new(Shell::new(app)))
        }),
    );
    // Final save with the current positions, then stop the audio.
    services.shutdown();
    drop(handle);
    result.map_err(|e| e.to_string().into())
}

fn os_name() -> &'static str {
    match std::env::consts::OS {
        "linux" => "Linux",
        "windows" => "Windows",
        "macos" => "macOS",
        other => other,
    }
}
