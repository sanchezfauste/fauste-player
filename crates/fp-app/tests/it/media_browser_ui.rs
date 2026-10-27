#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The media browser window answers a picker's request (mobile spec MB7).

use crate::support;

use std::path::PathBuf;
use std::thread::JoinHandle;

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_app::ui::media_browser::{BrowseMode, MediaBrowser};
use support::{harness_from, state};

const TITLE: &str = "Music on this device";

fn library() -> Vec<PathBuf> {
    [
        "/m/Album/b.mp3",
        "/m/Album/a.mp3",
        "/m/Other/c.flac",
        "/m/Other/cover.jpg",
    ]
    .iter()
    .map(PathBuf::from)
    .collect()
}

/// A harness with a browser holding `files`, and a helper thread waiting on
/// a request in `mode`, once the window shows it.
fn browsing(
    files: Vec<PathBuf>,
    mode: BrowseMode,
) -> (Harness<'static, AppUi>, JoinHandle<Vec<PathBuf>>) {
    let browser = MediaBrowser::default();
    browser.set_files(files);
    let given = browser.clone();
    let (mut h, _fake) = harness_from(state(1, 0), move |ui| ui.with_media_browser(given));
    let thread = std::thread::spawn(move || browser.browse(mode, None));
    for _ in 0..200 {
        h.run_steps(1);
        if h.query_by_label(TITLE).is_some() {
            return (h, thread);
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    panic!("the window did not open");
}

fn answer(h: &mut Harness<'static, AppUi>, thread: JoinHandle<Vec<PathBuf>>) -> Vec<PathBuf> {
    h.run_steps(2);
    for _ in 0..200 {
        if thread.is_finished() {
            return thread.join().unwrap();
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    panic!("no answer");
}

fn paths(list: &[&str]) -> Vec<PathBuf> {
    list.iter().map(PathBuf::from).collect()
}

#[test]
fn no_window_without_a_request() {
    let browser = MediaBrowser::default();
    browser.set_files(library());
    let (mut h, _fake) = harness_from(state(1, 0), move |ui| ui.with_media_browser(browser));
    h.run_steps(3);
    assert!(h.query_by_label(TITLE).is_none());
}

#[test]
fn audio_files_mode_adds_the_ticked_files() {
    let (mut h, thread) = browsing(library(), BrowseMode::AudioFiles);
    assert!(h.query_by_label("a.mp3").is_none(), "folders start closed");
    h.get_by_label("Album, 2 files").click();
    h.run_steps(2);
    h.get_by_label("b.mp3").click();
    h.run_steps(2);
    h.get_by_label("Add 1").click();
    assert_eq!(answer(&mut h, thread), paths(&["/m/Album/b.mp3"]));
}

#[test]
fn a_folder_tick_selects_its_files() {
    let (mut h, thread) = browsing(library(), BrowseMode::AudioFiles);
    h.get_by_label("Album").click();
    h.run_steps(2);
    h.get_by_label("Add 2").click();
    assert_eq!(
        answer(&mut h, thread),
        paths(&["/m/Album/a.mp3", "/m/Album/b.mp3"])
    );
}

#[test]
fn one_file_mode_answers_on_the_first_tap() {
    let mode = BrowseMode::OneFile {
        extensions: vec!["flac".to_owned()],
    };
    let (mut h, thread) = browsing(library(), mode);
    assert!(h.query_by_label("Album, 2 files").is_none());
    h.get_by_label("Other, 1 file").click();
    h.run_steps(2);
    h.get_by_label("c.flac").click();
    assert_eq!(answer(&mut h, thread), paths(&["/m/Other/c.flac"]));
}

#[test]
fn folder_mode_answers_the_folder() {
    let (mut h, thread) = browsing(library(), BrowseMode::Folder);
    let uses = h.get_all_by_label("Use this folder").count();
    assert_eq!(uses, 2);
    h.get_all_by_label("Use this folder")
        .last()
        .unwrap()
        .click();
    assert_eq!(answer(&mut h, thread), paths(&["/m/Other"]));
}

#[test]
fn cancel_answers_nothing() {
    let (mut h, thread) = browsing(library(), BrowseMode::AudioFiles);
    h.get_by_label("Cancel").click();
    assert_eq!(answer(&mut h, thread), Vec::<PathBuf>::new());
}

#[test]
fn an_empty_list_says_where_music_comes_from() {
    let (mut h, thread) = browsing(Vec::new(), BrowseMode::AudioFiles);
    assert!(h.query_by_label_contains("No music found").is_some());
    h.get_by_label("Cancel").click();
    assert_eq!(answer(&mut h, thread), Vec::<PathBuf>::new());
}

#[test]
fn a_selection_keeps_only_files_still_listed() {
    let browser = MediaBrowser::default();
    browser.set_files(library());
    let given = browser.clone();
    let (mut h, _fake) = harness_from(state(1, 0), move |ui| ui.with_media_browser(given));
    let asking = browser.clone();
    let thread = std::thread::spawn(move || asking.browse(BrowseMode::AudioFiles, None));
    for _ in 0..200 {
        h.run_steps(1);
        if h.query_by_label(TITLE).is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    h.get_by_label("Album").click();
    h.run_steps(2);
    browser.set_files(paths(&["/m/Album/a.mp3"]));
    h.run_steps(2);
    h.get_by_label("Add 1").click();
    assert_eq!(answer(&mut h, thread), paths(&["/m/Album/a.mp3"]));
}

#[test]
fn folders_with_the_same_name_read_apart() {
    let files = paths(&["/m/A/CD1/a.mp3", "/m/B/CD1/b.mp3"]);
    let (mut h, thread) = browsing(files, BrowseMode::AudioFiles);
    assert!(h.query_by_label("A/CD1, 1 file").is_some());
    assert!(h.query_by_label("B/CD1, 1 file").is_some());
    h.get_by_label("Cancel").click();
    answer(&mut h, thread);
}
