#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! External editor spec EE3: launching the editor through the platform.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use fp_app::i18n::I18n;
use fp_app::platform::Platform;
use fp_app::ui::playlist_files::{FileOutcome, open_in_editor};

use crate::support::FakePlatform;

fn launch(platform: &Arc<FakePlatform>) -> crossbeam_channel::Receiver<FileOutcome> {
    let (tx, rx) = crossbeam_channel::unbounded();
    let dynamic: Arc<dyn Platform> = platform.clone();
    open_in_editor(
        &egui::Context::default(),
        &dynamic,
        PathBuf::from("/usr/bin/editor"),
        Path::new("/music/a.flac"),
        tx,
    );
    rx
}

#[test]
fn a_failed_launch_shows_a_notice() {
    let platform = Arc::new(FakePlatform {
        launch_result: Some("no such file".into()),
        ..FakePlatform::default()
    });
    let rx = launch(&platform);
    let Ok(FileOutcome::EditorFailed { program, error }) = rx.try_recv() else {
        panic!("a failure is reported");
    };
    assert_eq!(program, PathBuf::from("/usr/bin/editor"));
    assert_eq!(error, "no such file");
    let t = I18n::new(Some("en-US"));
    let text = t.tr_args(
        "notice-editor-failed",
        &[
            ("program", program.display().to_string().into()),
            ("error", error.into()),
        ],
    );
    assert!(text.contains("/usr/bin/editor") && text.contains("no such file"));
}

#[test]
fn a_started_editor_says_nothing() {
    let platform = Arc::new(FakePlatform::default());
    let rx = launch(&platform);
    assert!(rx.try_recv().is_err());
    assert_eq!(
        platform.launches.lock().unwrap().as_slice(),
        [(
            PathBuf::from("/usr/bin/editor"),
            PathBuf::from("/music/a.flac")
        )]
    );
}

#[cfg(unix)]
#[test]
fn the_desktop_reports_a_program_that_cannot_start() {
    let (tx, rx) = std::sync::mpsc::channel();
    fp_app::platform::DesktopPlatform.launch_editor(
        Path::new("/nonexistent/editor"),
        Path::new("/music/a.flac"),
        Box::new(move |r| tx.send(r).unwrap()),
    );
    let result = rx.recv_timeout(std::time::Duration::from_secs(10)).unwrap();
    assert!(result.is_err());
}

#[cfg(unix)]
#[test]
fn the_desktop_reports_a_started_program() {
    let (tx, rx) = std::sync::mpsc::channel();
    fp_app::platform::DesktopPlatform.launch_editor(
        Path::new("/bin/true"),
        Path::new("/music/a.flac"),
        Box::new(move |r| tx.send(r).unwrap()),
    );
    let result = rx.recv_timeout(std::time::Duration::from_secs(10)).unwrap();
    assert_eq!(result, Ok(()));
}
