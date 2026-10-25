#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The interface asks the platform for files and links, shows when another
//! app took the audio focus and hides file writes where they are not
//! allowed (mobile spec §4, MB4, §2 non-goals).

use crate::support;

use std::sync::Arc;
use std::sync::atomic::Ordering;

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable;
use fp_model::Command;
use support::{FakePlatform, harness_from, state};

const FOCUS_LOST: &str = "Another app took the audio focus — playout continues";

#[test]
fn add_asks_the_platform_for_files_and_adds_them() {
    let dir = tempfile::tempdir().unwrap();
    let files = vec![dir.path().join("a.flac"), dir.path().join("b.mp3")];
    for f in &files {
        std::fs::write(f, b"").unwrap();
    }
    let platform = Arc::new(FakePlatform {
        audio_files: files.clone(),
        ..FakePlatform::default()
    });
    let given = platform.clone();
    let (mut h, fake) = harness_from(state(1, 0), move |ui| ui.with_platform(given));
    h.get_by_label("Add tracks to this playlist").click();
    let mut sent = Vec::new();
    for _ in 0..200 {
        h.run_steps(1);
        sent.extend(fake.take_sent());
        if !sent.is_empty() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(
        *platform.asked.lock().unwrap(),
        vec!["audio files".to_owned()]
    );
    assert!(
        sent.iter()
            .any(|c| matches!(c, Command::InsertPaths { paths, .. } if *paths == files)),
        "{sent:?}"
    );
}

#[test]
fn a_lost_audio_focus_shows_the_banner() {
    let platform = Arc::new(FakePlatform::default());
    let given = platform.clone();
    let (mut h, _fake) = harness_from(state(1, 1), move |ui| ui.with_platform(given));
    assert!(h.query_by_label_contains(FOCUS_LOST).is_none());
    platform.focus_lost.store(true, Ordering::Relaxed);
    h.run_steps(2);
    assert!(h.query_by_label_contains(FOCUS_LOST).is_some());
    platform.focus_lost.store(false, Ordering::Relaxed);
    h.run_steps(2);
    assert!(h.query_by_label_contains(FOCUS_LOST).is_none());
}

fn open_row_menu(h: &mut egui_kittest::Harness<'static, fp_app::ui::app::AppUi>) {
    h.get_all_by_label("Song 1")
        .max_by(|a, b| a.rect().min.y.total_cmp(&b.rect().min.y))
        .unwrap()
        .click_secondary();
    h.run_steps(3);
}

fn open_section(h: &mut egui_kittest::Harness<'static, fp_app::ui::app::AppUi>, name: &str) {
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, name).click();
    h.run_steps(2);
}

#[test]
fn without_user_file_writes_the_row_menu_has_no_edit_tags() {
    for read_only in [false, true] {
        let platform = Arc::new(FakePlatform {
            read_only,
            ..FakePlatform::default()
        });
        let (mut h, _fake) = harness_from(state(1, 1), move |ui| ui.with_platform(platform));
        open_row_menu(&mut h);
        assert!(h.query_by_label("Loudness…").is_some());
        assert_eq!(h.query_by_label("Edit tags…").is_some(), !read_only);
    }
}

#[test]
fn without_user_file_writes_settings_offer_no_import_or_export() {
    for read_only in [false, true] {
        let make = || {
            Arc::new(FakePlatform {
                read_only,
                ..FakePlatform::default()
            })
        };
        let platform = make();
        let (mut h, _fake) = harness_from(state(1, 1), move |ui| ui.with_platform(platform));
        open_section(&mut h, "Playlists");
        assert_eq!(
            h.query_by_label_contains("Import M3U / PLS").is_some(),
            !read_only
        );
        let platform = make();
        let (mut h, _fake) = harness_from(state(1, 1), move |ui| ui.with_platform(platform));
        open_section(&mut h, "Cartwall");
        assert_eq!(h.query_by_label_contains("Import…").is_some(), !read_only);
        assert_eq!(h.query_by_label_contains("Export…").is_some(), !read_only);
    }
}
