//! Fauste Player: starts logging, loads the saved state, builds the audio
//! engine, and runs the conductor, analysis, services and interface.

#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::process::ExitCode;
use std::time::Duration;

use fp_app::i18n::I18n;
use fp_app::session::AppSession;
use fp_app::ui::shell::Shell;
use fp_app::ui::theme;
use fp_app::{APP_ID, bootstrap, cli, crash, instance, logging};
use fp_backends::system_backends;

/// How often the running instance looks for playlists handed over.
const INBOX_INTERVAL: Duration = Duration::from_millis(500);

/// Tries at the instance lock, and the pause between them: an instance that
/// is ending may hold it for an instant longer.
const LOCK_ATTEMPTS: usize = 3;
const LOCK_RETRY_PAUSE: Duration = Duration::from_millis(20);

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
    // Read before logging starts; the full load below reports its problems
    // through the log.
    let (log_mb, log_files, repeat_secs) = fp_store::peek_log_limits(&paths);
    let _log = logging::init(&paths.log_dir, log_mb, log_files, repeat_secs);
    let report_cap = crash::install_panic_hook(
        paths.log_dir.clone(),
        fp_model::Limits::default().max_crash_reports,
    );
    tracing::info!(version = env!("CARGO_PKG_VERSION"), "starting");
    // Logged once logging is up, so that a GUI build without a console
    // keeps the warning too.
    for path in &ignored {
        tracing::warn!(path = %path.display(), "not a playlist, ignored");
    }
    let result = run(paths, playlists, report_cap);
    drop(lock);
    match result {
        Ok(()) => {
            tracing::info!("stopped");
            ExitCode::SUCCESS
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
    report_cap: crash::ReportCap,
) -> Result<(), Box<dyn std::error::Error>> {
    let session = AppSession::start(paths.clone(), system_backends(), Some(report_cap))?;
    let window_system = session.handle.model.load().config.ui.window_system;
    let handle = session.handle.clone();
    let remote = fp_app::remote::start(
        handle.clone(),
        session.media.clone(),
        session.analysis_dir(),
    );
    let mut app = session.app_ui().with_notices(
        std::env::current_exe()
            .ok()
            .and_then(|exe| fp_app::ui::about::find_notices(&exe)),
    );
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
        event_loop_builder: event_loop_hook(window_system),
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
    // Everything that can send commands stops first; then the final save
    // with the current positions, then the audio.
    drop(remote);
    if let Some(midi) = midi {
        midi.shutdown();
    }
    drop(handle);
    session.shutdown();
    result.map_err(|e| e.to_string())?;
    Ok(())
}

/// Opens the window on the window system the configuration asks for
/// (Linux; see `fp_app::window_system`).
#[cfg(target_os = "linux")]
fn event_loop_hook(setting: fp_model::WindowSystem) -> Option<eframe::EventLoopBuilderHook> {
    use fp_app::window_system::Choice;
    use winit::platform::wayland::EventLoopBuilderExtWayland;
    use winit::platform::x11::EventLoopBuilderExtX11;
    let choice = fp_app::window_system::from_env(setting);
    tracing::info!(?setting, ?choice, "window system");
    Some(Box::new(move |builder| match choice {
        Some(Choice::X11) => {
            builder.with_x11();
        }
        Some(Choice::Wayland) => {
            builder.with_wayland();
        }
        None => {}
    }))
}

#[cfg(not(target_os = "linux"))]
fn event_loop_hook(_setting: fp_model::WindowSystem) -> Option<eframe::EventLoopBuilderHook> {
    None
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
