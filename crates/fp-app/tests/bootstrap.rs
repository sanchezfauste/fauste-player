#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::ffi::OsString;
use std::path::Path;

use fp_app::bootstrap::paths_from;
use fp_app::crash::install_panic_hook;

#[test]
fn fauste_home_overrides_the_os_directories() {
    let paths = paths_from(Some(OsString::from("/portable/fauste"))).unwrap();
    assert_eq!(
        paths.config_file(),
        Path::new("/portable/fauste")
            .join("config")
            .join("config.json")
    );
    assert!(
        paths_from(None).is_some(),
        "the OS directories are used otherwise"
    );
}

#[test]
fn the_panic_hook_writes_a_crash_report() {
    let dir = tempfile::tempdir().unwrap();
    install_panic_hook(dir.path().to_path_buf(), 20);
    let _ = std::panic::catch_unwind(|| panic!("deliberate test panic"));
    let reports: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("crash-")
        })
        .collect();
    assert_eq!(reports.len(), 1);
    let text = std::fs::read_to_string(&reports[0]).unwrap();
    assert!(text.contains("deliberate test panic"), "{text}");
    assert!(text.contains("version"), "{text}");
}
