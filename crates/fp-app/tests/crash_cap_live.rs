#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Live settings spec §6: the crash report cap follows the configuration
//! while the application runs. Its own test binary: a panic hook is
//! global to the process.

use std::path::Path;

use fp_app::crash::install_panic_hook;

fn reports(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .unwrap()
        .filter(|e| {
            e.as_ref()
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with("crash-")
        })
        .count()
}

#[test]
fn the_report_cap_follows_the_configuration() {
    let dir = tempfile::tempdir().unwrap();
    let cap = install_panic_hook(dir.path().to_path_buf(), 1);
    cap.set(3);
    for _ in 0..5 {
        let _ = std::panic::catch_unwind(|| panic!("again"));
    }
    assert_eq!(reports(dir.path()), 3);
    cap.set(1);
    let _ = std::panic::catch_unwind(|| panic!("once more"));
    assert_eq!(reports(dir.path()), 3, "none written, none deleted");
}
