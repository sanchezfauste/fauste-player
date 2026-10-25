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
