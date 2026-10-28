#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! `AppSession`: the start-up shared by the desktop binary and the mobile
//! shell (mobile spec §4).

use std::sync::Arc;

use fp_app::session::AppSession;
use fp_backends::{AudioBackend, NullBackend};
use fp_store::AppPaths;

#[test]
fn a_session_starts_and_stops_on_the_null_backend() {
    let dir = tempfile::tempdir().unwrap();
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(NullBackend)];
    let session = AppSession::start(AppPaths::under(dir.path()), backends, None).unwrap();
    assert!(!session.handle.model.load().players.is_empty());
    assert_eq!(
        session.analysis_dir(),
        dir.path().join("cache").join("analysis")
    );
    let _ui = session.app_ui();
    session.shutdown();
    assert!(dir.path().join("config").join("config.json").exists());
}

/// Closing the Android app with nothing on air stops the services after
/// their final save, from a shared session (phone layout spec PL23); a
/// second call finds them stopped and returns at once.
#[test]
fn stop_services_saves_and_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(NullBackend)];
    let session = AppSession::start(AppPaths::under(dir.path()), backends, None).unwrap();
    let mut config = session.handle.model.load().config.clone();
    config.ui.language = Some("es-ES".to_owned());
    assert!(
        session
            .handle
            .send(fp_model::Command::UpdateConfig(Box::new(config)))
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while session.handle.model.load().config.ui.language.is_none() {
        assert!(std::time::Instant::now() < deadline, "the change applies");
        std::thread::yield_now();
    }
    session.stop_services();
    let saved = std::fs::read_to_string(dir.path().join("config").join("config.json")).unwrap();
    assert!(saved.contains("es-ES"), "{saved}");
    let again = std::time::Instant::now();
    session.stop_services();
    assert!(again.elapsed() < std::time::Duration::from_millis(100));
    session.shutdown();
}

/// The shell may ask from two threads at once (the activity closing and
/// the service seeing silence): neither returns before the final save.
#[test]
fn stop_services_from_two_threads_both_wait_for_the_save() {
    let dir = tempfile::tempdir().unwrap();
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(NullBackend)];
    let session = AppSession::start(AppPaths::under(dir.path()), backends, None).unwrap();
    let config_file = dir.path().join("config").join("config.json");
    let _ = std::fs::remove_file(&config_file);
    std::thread::scope(|s| {
        let first = s.spawn(|| session.stop_services());
        let second = s.spawn(|| {
            session.stop_services();
            config_file.exists()
        });
        first.join().unwrap();
        assert!(second.join().unwrap(), "the save was done when it returned");
    });
    session.shutdown();
}
