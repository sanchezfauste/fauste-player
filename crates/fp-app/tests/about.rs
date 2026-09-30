#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The version and the About window (feedback spec F21).

mod support;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use egui::Key;
use egui_kittest::kittest::Queryable;
use fp_app::ui::about::{self, NoticeOpener, find_notices, notice_candidates};
use support::{harness, harness_from, state};

const ABOUT: &str = "About Fauste Player";

fn recorder() -> (NoticeOpener, Arc<Mutex<Vec<PathBuf>>>) {
    let opened = Arc::new(Mutex::new(Vec::new()));
    let seen = opened.clone();
    let opener: NoticeOpener = Arc::new(move |p: &Path| seen.lock().unwrap().push(p.to_owned()));
    (opener, opened)
}

#[test]
fn the_top_bar_shows_the_version() {
    let (h, _) = harness(state(1, 1));
    assert!(h.query_by_label_contains(about::VERSION).is_some());
}

#[test]
fn clicking_the_name_opens_about_and_escape_closes_it() {
    let (mut h, fake) = harness(state(1, 1));
    h.get_by_label(ABOUT).click();
    h.run_steps(2);
    assert!(h.query_by_label_contains("All rights reserved").is_some());
    assert!(
        h.query_by_label_contains(&format!("Version {}", about::VERSION))
            .is_some()
    );
    h.key_press(Key::Escape);
    h.run_steps(2);
    assert!(h.query_by_label_contains("All rights reserved").is_none());
    // Esc only closed the window.
    assert!(fake.take_sent().is_empty());
}

#[test]
fn player_shortcuts_do_nothing_while_about_is_open() {
    let (mut h, fake) = harness(state(1, 1));
    h.get_by_label(ABOUT).click();
    h.run_steps(2);
    h.key_press(Key::Num1);
    h.run_steps(2);
    assert!(fake.take_sent().is_empty());
}

#[test]
fn the_bundled_licences_can_be_read() {
    let (mut h, _) = harness(state(1, 1));
    h.get_by_label(ABOUT).click();
    h.run_steps(2);
    h.get_by_label("Inter font — SIL Open Font License 1.1")
        .click();
    h.run_steps(2);
    assert!(
        h.query_by_label_contains("SIL OPEN FONT LICENSE Version 1.1")
            .is_some()
    );
    h.get_by_label("Phosphor Icons — MIT License").click();
    h.run_steps(2);
    assert!(
        h.query_by_label_contains("Permission is hereby granted")
            .is_some()
    );
}

#[test]
fn third_party_licences_open_the_installed_file() {
    let (opener, opened) = recorder();
    let file = PathBuf::from("/opt/fauste/licenses/THIRD-PARTY.html");
    let path = file.clone();
    let (mut h, _) = harness_from(state(1, 1), move |ui| {
        ui.with_notices(Some(path)).with_notice_opener(opener)
    });
    h.get_by_label(ABOUT).click();
    h.run_steps(2);
    h.get_by_label("Third-party licences").click();
    h.run_steps(2);
    assert_eq!(*opened.lock().unwrap(), vec![file]);
}

#[test]
fn without_installed_notices_the_button_does_nothing_and_says_why() {
    let (opener, opened) = recorder();
    let (mut h, _) = harness_from(state(1, 1), move |ui| {
        ui.with_notices(None).with_notice_opener(opener)
    });
    h.get_by_label(ABOUT).click();
    h.run_steps(2);
    h.get_by_label("Third-party licences").click();
    h.run_steps(2);
    assert!(opened.lock().unwrap().is_empty());
    assert!(
        h.query_by_label_contains("installed with release packages")
            .is_some()
    );
}

#[test]
fn notices_are_looked_up_where_the_packages_install_them() {
    let dir = Path::new("/usr/bin");
    let c = notice_candidates(dir);
    assert!(c.contains(&dir.join("licenses").join("THIRD-PARTY.html")));
    assert!(c.contains(&dir.join("../share/doc/fauste-player/THIRD-PARTY.html")));
    assert!(c.contains(&dir.join("../Resources/licenses/THIRD-PARTY.html")));
}

#[test]
fn find_notices_returns_an_installed_file() {
    let root = tempfile::tempdir().unwrap();
    let bin = root.path().join("bin");
    let doc = root.path().join("share/doc/fauste-player");
    std::fs::create_dir_all(&bin).unwrap();
    std::fs::create_dir_all(&doc).unwrap();
    let exe = bin.join("fauste-player");
    assert_eq!(find_notices(&exe), None);
    std::fs::write(doc.join("THIRD-PARTY.html"), "<p>notices</p>").unwrap();
    let found = find_notices(&exe).unwrap();
    assert_eq!(
        found.canonicalize().unwrap(),
        doc.join("THIRD-PARTY.html").canonicalize().unwrap()
    );
}
