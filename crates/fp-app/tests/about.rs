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
use fp_app::ui::about::{self, Opener, find_notices, notice_candidates};
use support::{harness, harness_from, state};

const ABOUT: &str = "About Fauste Player";

fn recorder() -> (Opener, Arc<Mutex<Vec<PathBuf>>>) {
    let opened = Arc::new(Mutex::new(Vec::new()));
    let seen = opened.clone();
    let opener: Opener = Arc::new(move |p: &Path| seen.lock().unwrap().push(p.to_owned()));
    (opener, opened)
}

#[test]
fn the_top_bar_no_longer_repeats_the_name_or_the_version() {
    let (mut h, _fake) = harness(state(1, 0));
    h.run_steps(2);
    // The native title bar carries them now (feedback 2 spec O32).
    assert!(h.query_by_label_contains(about::VERSION).is_none());
    assert!(h.query_by_label("Fauste Player").is_none());
    // What else the bar shows stays.
    assert!(h.query_by_label_contains("Settings").is_some());
    assert!(h.query_by_label("About Fauste Player").is_some());
}

#[test]
fn the_info_button_sits_next_to_settings_and_opens_about() {
    let (mut h, _) = harness(state(1, 1));
    let button = h.get_by_label(ABOUT);
    let settings = h.get_by_label_contains("Settings");
    let (b, s) = (button.rect(), settings.rect());
    assert!(b.right() <= s.left() && (b.center().y - s.center().y).abs() < 2.0);
    // A tile next to Settings, not the name area at the far left.
    assert!(b.width() <= 30.0 && s.left() - b.right() <= 20.0);
    button.click();
    h.run_steps(2);
    assert!(
        h.query_by_label_contains("Licensed under the GNU General Public License")
            .is_some()
    );
}

#[test]
fn the_info_button_opens_about_and_escape_closes_it() {
    let (mut h, fake) = harness(state(1, 1));
    h.get_by_label(ABOUT).click();
    h.run_steps(2);
    assert!(
        h.query_by_label_contains("Licensed under the GNU General Public License")
            .is_some()
    );
    assert!(
        h.query_by_label_contains(&format!("Version {}", about::VERSION))
            .is_some()
    );
    h.key_press(Key::Escape);
    h.run_steps(2);
    assert!(
        h.query_by_label_contains("Licensed under the GNU General Public License")
            .is_none()
    );
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
    // The scrolling body needs a few frames to settle before a click lands.
    h.run_steps(6);
    h.get_by_label("Inter font — SIL Open Font License 1.1")
        .click();
    h.run_steps(2);
    // The whole window scrolls: bring the heading, and the text under it, up.
    h.get_by_label("Inter font — SIL Open Font License 1.1")
        .scroll_to_me();
    h.run_steps(2);
    assert!(
        h.query_by_label_contains("SIL OPEN FONT LICENSE Version 1.1")
            .is_some()
    );
    // Fold the long text so the next heading is not scrolled out of view.
    h.get_by_label("Inter font — SIL Open Font License 1.1")
        .click();
    h.run_steps(2);
    h.get_by_label("Phosphor Icons — MIT License")
        .scroll_to_me();
    h.run_steps(2);
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

const REPO: &str = "https://github.com/sanchezfauste/fauste-player";
const GPL_TEXT: &str = "https://www.gnu.org/licenses/gpl-3.0.html";

#[test]
fn about_credits_the_author_and_links_the_project() {
    let (opener, opened) = recorder();
    let (mut h, _) = harness_from(state(1, 1), move |ui| ui.with_notice_opener(opener));
    h.get_by_label(ABOUT).click();
    h.run_steps(2);
    assert!(
        h.query_by_label_contains("Fauste Player by Marc Sánchez Fauste")
            .is_some()
    );
    h.get_by_label(REPO).click();
    h.run_steps(2);
    assert_eq!(*opened.lock().unwrap(), vec![PathBuf::from(REPO)]);
}

#[test]
fn about_names_the_licence_and_opens_its_text_online() {
    let (opener, opened) = recorder();
    let (mut h, _) = harness_from(state(1, 1), move |ui| ui.with_notice_opener(opener));
    h.get_by_label(ABOUT).click();
    h.run_steps(2);
    assert!(
        h.query_by_label_contains("GNU General Public License, version 3 or later")
            .is_some()
    );
    h.get_by_label("Read the licence online").click();
    h.run_steps(2);
    assert_eq!(*opened.lock().unwrap(), vec![PathBuf::from(GPL_TEXT)]);
}

/// Opens About in `tag` and says whether it shows the disclaimer.
fn shows_disclaimer(tag: &str) -> bool {
    let t = fp_app::i18n::I18n::new(Some(tag));
    let (mut h, fake) = harness(state(1, 1));
    let mut s = (**fake.state.load()).clone();
    s.config.ui.language = Some(tag.to_owned());
    fake.state.store(Arc::new(s));
    h.run_steps(2);
    h.get_by_label(&t.tr("tip-about")).click();
    h.run_steps(2);
    h.query_by_label(&t.tr("about-disclaimer")).is_some()
}

#[test]
fn the_disclaimer_is_in_english_and_says_as_is_and_own_risk() {
    let en = fp_app::i18n::I18n::new(Some("en-US")).tr("about-disclaimer");
    assert!(
        en.contains("AI") && en.contains("without any warranty"),
        "{en}"
    );
    assert!(en.contains("at your own risk"), "{en}");
}

#[test]
fn every_language_shows_the_disclaimer() {
    for l in fp_app::i18n::LOCALES {
        assert!(shows_disclaimer(l.tag), "{}", l.tag);
    }
}

#[test]
fn every_language_shows_the_untranslated_attribution_and_the_project_link() {
    for l in fp_app::i18n::LOCALES {
        let t = fp_app::i18n::I18n::new(Some(l.tag));
        let (mut h, fake) = harness(state(1, 1));
        let mut st = (**fake.state.load()).clone();
        st.config.ui.language = Some(l.tag.to_owned());
        fake.state.store(Arc::new(st));
        h.run_steps(2);
        h.get_by_label(&t.tr("tip-about")).click();
        h.run_steps(2);
        assert!(h.query_by_label(about::ATTRIBUTION).is_some(), "{}", l.tag);
        assert!(h.query_by_label(about::REPO_URL).is_some(), "{}", l.tag);
    }
}

#[test]
fn every_translation_of_the_disclaimer_differs_from_english() {
    let en = fp_app::i18n::I18n::new(Some("en-US")).tr("about-disclaimer");
    for l in fp_app::i18n::LOCALES.iter().filter(|l| l.tag != "en-US") {
        let text = fp_app::i18n::I18n::new(Some(l.tag)).tr("about-disclaimer");
        assert_ne!(text, en, "{} falls back to English", l.tag);
    }
}

#[test]
fn no_language_carries_a_separate_machine_translation_notice() {
    for l in fp_app::i18n::LOCALES {
        assert!(!l.source.contains("about-machine-translation"), "{}", l.tag);
    }
}
