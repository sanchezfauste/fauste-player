#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! A panic that repeats (for example once per step of a background thread)
//! must not fill the disk with crash reports.

use fp_app::crash::install_panic_hook;

#[test]
fn crash_reports_are_capped_per_run() {
    let dir = tempfile::tempdir().unwrap();
    install_panic_hook(dir.path().to_path_buf(), 2);
    for _ in 0..5 {
        let _ = std::panic::catch_unwind(|| panic!("again"));
    }
    let reports = std::fs::read_dir(dir.path())
        .unwrap()
        .filter(|e| {
            e.as_ref()
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with("crash-")
        })
        .count();
    assert_eq!(reports, 2);
}
