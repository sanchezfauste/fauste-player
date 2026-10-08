//! Fauste Player: starts logging, loads the saved state, builds the audio
//! engine, and runs the conductor, analysis, services and interface.

#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

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
use fp_backends::{AudioBackend, Availability, NullBackend, display_name, system_backends};
use fp_engine::conductor::{Conductor, ConductorHandle};
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

/// Tries at the instance lock, and the pause between them: a restarting
/// instance probes the lock for an instant while it waits for this one.
const LOCK_ATTEMPTS: usize = 3;
const LOCK_RETRY_PAUSE: Duration = Duration::from_millis(20);

/// How the interface ended.
enum Exit {
    Quit,
    /// The operator asked for a restart (feedback 2 spec O4); inside a
    /// Flatpak, this process waits at most `handoff` for the new one.
    Restart {
        handoff: Duration,
        /// `ui.language`, for a failure message after the shutdown.
        language: Option<String>,
    },
}

fn main() -> ExitCode {
    let (playlists, ignored) = match cli::parse(std::env::args_os().skip(1)) {
        Ok(cli::Invocation::Version) => {
            cli::emit(
                cli::Stream::Out,
                &format!("fauste-player {}\n", env!("CARGO_PKG_VERSION")),
            );
            return ExitCode::SUCCESS;
        }
        Ok(cli::Invocation::Help) => {
            cli::emit(cli::Stream::Out, &cli::usage());
            return ExitCode::SUCCESS;
        }
        Ok(cli::Invocation::Run { playlists, ignored }) => {
            for path in &ignored {
                // An ignored argument is not worth a message box.
                eprintln!("fauste-player: not a playlist, ignored: {}", path.display());
            }
            (playlists, ignored)
        }
        Err(e) => {
            cli::emit(cli::Stream::Err, &format!("fauste-player: {e}\n"));
            return ExitCode::FAILURE;
        }
    };
    let Some(paths) = bootstrap::paths() else {
        cli::emit(
            cli::Stream::Err,
            "fauste-player: no home directory found; set FAUSTE_HOME\n",
        );
        return ExitCode::FAILURE;
    };
    // One instance per data folder: a second start (a playlist opened from
    // the file manager during a show) hands its playlists over and ends.
    let lock = match instance::acquire_with_retry(&paths.data_dir, LOCK_ATTEMPTS, LOCK_RETRY_PAUSE)
    {
        Ok(Some(lock)) => lock,
        Ok(None) => return hand_over(&paths.data_dir, &playlists),
        Err(e) => {
            cli::emit(
                cli::Stream::Err,
                &format!(
                    "fauste-player: cannot lock {}: {e}\n",
                    paths.data_dir.display()
                ),
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
    // Logged once logging is up, so that a GUI build without a console
    // keeps the warning too.
    for path in &ignored {
        tracing::warn!(path = %path.display(), "not a playlist, ignored");
    }
    let data_dir = paths.data_dir.clone();
    let result = run(paths, playlists);
    drop(lock);
    match result {
        Ok(Exit::Quit) => {
            tracing::info!("stopped");
            ExitCode::SUCCESS
        }
        Ok(Exit::Restart { handoff, language }) => {
            // The lock is released above: the new process can take it.
            match fp_app::restart::relaunch(&data_dir, handoff) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    tracing::error!(error = %e, "could not start again");
                    let i18n = I18n::new(language.as_deref());
                    eprintln!("fauste-player: {}", i18n.tr("restart-failed"));
                    let _ = rfd::MessageDialog::new()
                        .set_title("Fauste Player")
                        .set_description(i18n.tr("restart-failed"))
                        .set_level(rfd::MessageLevel::Error)
                        .show();
                    ExitCode::FAILURE
                }
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "could not start");
            cli::emit(cli::Stream::Err, &format!("fauste-player: {e}\n"));
            ExitCode::FAILURE
        }
    }
}

fn run(
    paths: fp_store::AppPaths,
    playlists: Vec<std::path::PathBuf>,
) -> Result<Exit, Box<dyn std::error::Error>> {
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
    let settings = EngineSettings::from_config(&config);
    let engine = Engine::new(backends.clone(), settings, file_opener());
    let in_use = engine.backend_in_use().to_owned();
    tracing::info!(backend = %in_use, ?availability, "audio systems");
    let output = if in_use == "null" {
        i18n.tr("settings-backend-null")
    } else {
        display_name(&in_use).to_owned()
    };
    let platform = format!("{} · {}", os_name(), output);
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

    let remote = fp_app::remote::start(
        handle.clone(),
        media.clone(),
        paths.cache_dir.join("analysis"),
        &config.limits,
    );
    let mut app = AppUi::new(handle.clone(), i18n, media)
        .with_services(requests)
        .with_service_faults(faults)
        .with_backends(backends)
        .with_platform(platform)
        // The engine above was built from this configuration.
        .with_started_config(config.clone())
        .with_notices(
            std::env::current_exe()
                .ok()
                .and_then(|exe| fp_app::ui::about::find_notices(&exe)),
        );
    let restart = app.restart_flag();
    if let Some(r) = &remote {
        app = app.with_remote_status(r.status_cell());
    }
    let midi = fp_app::midi::start(handle.clone());
    if let Some(midi) = &midi {
        app = app.with_midi(midi.handle());
    }
    for playlist in playlists {
        tracing::info!(path = %playlist.display(), "importing a playlist given at start");
        app.import_playlist(playlist);
    }
    let (inbox_tx, inbox_rx) = crossbeam_channel::unbounded();
    instance::watch(paths.data_dir.clone(), inbox_tx, INBOX_INTERVAL)?;
    let app = app.with_inbox(inbox_rx);
    let mut viewport = egui::ViewportBuilder::default()
        .with_title(cli::window_title())
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
    let final_config = handle.model.load_full();
    let handoff =
        Duration::from_secs_f64(final_config.config.tuning.restart_handoff_ms.max(0.0) / 1000.0);
    let language = final_config.config.ui.language.clone();
    drop(final_config);
    // Everything that can send commands stops first; then the final save
    // with the current positions, then the audio.
    drop(remote);
    if let Some(midi) = midi {
        midi.shutdown();
    }
    services.shutdown();
    stop_engine(handle);
    result.map_err(|e| e.to_string())?;
    // Only a confirmed Restart now sets the flag; a plain close quits.
    Ok(if restart.load(std::sync::atomic::Ordering::Acquire) {
        Exit::Restart { handoff, language }
    } else {
        Exit::Quit
    })
}

/// Stops the conductor and the engine, closing the output streams, now:
/// a restarted instance may need the same device. Every other holder of the
/// handle must have stopped already.
fn stop_engine(handle: Arc<ConductorHandle>) {
    match Arc::try_unwrap(handle) {
        Ok(conductor) => {
            drop(conductor);
            tracing::info!("the audio engine is stopped");
        }
        Err(handle) => tracing::warn!(
            holders = Arc::strong_count(&handle) - 1,
            "the audio engine is still held by another part of the application; \
             it stops when the process ends"
        ),
    }
}

/// Gives `playlists` to the instance already running; without any, tells
/// the operator it is already running.
fn hand_over(data_dir: &std::path::Path, playlists: &[std::path::PathBuf]) -> ExitCode {
    if !playlists.is_empty() {
        return match instance::deliver(data_dir, playlists) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                cli::emit(
                    cli::Stream::Err,
                    &format!("fauste-player: cannot hand the playlists over: {e}\n"),
                );
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
