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
use fp_app::{bootstrap, cli, crash, instance, logging};
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

/// Reverse-DNS application id, shared by the desktop entry, AppStream,
/// Flatpak and the macOS bundle.
const APP_ID: &str = "org.fauste.FaustePlayer";

/// How often the running instance looks for playlists handed over.
const INBOX_INTERVAL: Duration = Duration::from_millis(500);

fn main() -> ExitCode {
    let playlists = match cli::parse(std::env::args_os().skip(1)) {
        Ok(cli::Invocation::Version) => {
            println!("fauste-player {}", env!("CARGO_PKG_VERSION"));
            return ExitCode::SUCCESS;
        }
        Ok(cli::Invocation::Help) => {
            print!("{}", cli::usage());
            return ExitCode::SUCCESS;
        }
        Ok(cli::Invocation::Run { playlists, ignored }) => {
            for path in ignored {
                eprintln!("fauste-player: not a playlist, ignored: {}", path.display());
            }
            playlists
        }
        Err(e) => {
            eprintln!("fauste-player: {e}");
            return ExitCode::FAILURE;
        }
    };
    let Some(paths) = bootstrap::paths() else {
        eprintln!("fauste-player: no home directory found; set FAUSTE_HOME");
        return ExitCode::FAILURE;
    };
    // One instance per data folder: a second start (a playlist opened from
    // the file manager during a show) hands its playlists over and ends.
    let lock = match instance::acquire(&paths.data_dir) {
        Ok(Some(lock)) => lock,
        Ok(None) => return hand_over(&paths.data_dir, &playlists),
        Err(e) => {
            eprintln!(
                "fauste-player: cannot lock {}: {e}",
                paths.data_dir.display()
            );
            return ExitCode::FAILURE;
        }
    };
    let _log = logging::init(&paths.log_dir);
    crash::install_panic_hook(
        paths.log_dir.clone(),
        fp_model::Limits::default().max_crash_reports,
    );
    tracing::info!(version = env!("CARGO_PKG_VERSION"), "starting");
    let result = run(paths, playlists);
    drop(lock);
    match result {
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

fn run(
    paths: fp_store::AppPaths,
    playlists: Vec<std::path::PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
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

    let mut app = AppUi::new(handle.clone(), i18n, media)
        .with_services(requests)
        .with_service_faults(faults)
        .with_backends(backends)
        .with_platform(platform)
        .with_notices(
            std::env::current_exe()
                .ok()
                .and_then(|exe| fp_app::ui::about::find_notices(&exe)),
        );
    for playlist in playlists {
        tracing::info!(path = %playlist.display(), "importing a playlist given at start");
        app.import_playlist(playlist);
    }
    let (inbox_tx, inbox_rx) = crossbeam_channel::unbounded();
    instance::watch(paths.data_dir.clone(), inbox_tx, INBOX_INTERVAL)?;
    let app = app.with_inbox(inbox_rx);
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Fauste Player")
        // Matches the desktop entry, so the window gets its icon and name.
        .with_app_id(APP_ID)
        .with_inner_size([1600.0, 940.0])
        .with_min_inner_size([420.0, 480.0]);
    if let Some(icon) = cli::window_icon() {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions {
        viewport,
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

/// Gives `playlists` to the instance already running; without any, tells
/// the operator it is already running.
fn hand_over(data_dir: &std::path::Path, playlists: &[std::path::PathBuf]) -> ExitCode {
    if !playlists.is_empty() {
        return match instance::deliver(data_dir, playlists) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("fauste-player: cannot hand the playlists over: {e}");
                ExitCode::FAILURE
            }
        };
    }
    let i18n = I18n::new(None);
    eprintln!("fauste-player: {}", i18n.tr("already-running"));
    let _ = rfd::MessageDialog::new()
        .set_title("Fauste Player")
        .set_description(i18n.tr("already-running"))
        .set_level(rfd::MessageLevel::Info)
        .show();
    ExitCode::SUCCESS
}

fn os_name() -> &'static str {
    match std::env::consts::OS {
        "linux" => "Linux",
        "windows" => "Windows",
        "macos" => "macOS",
        other => other,
    }
}
