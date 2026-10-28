//! The media browser's handle: the device's music by folder and one request
//! at a time (mobile spec MB7).

use std::path::PathBuf;
use std::time::{Duration, Instant};

use fp_app::ui::media_browser::{BrowseMode, MediaBrowser, MediaFile, folders};

fn paths(list: &[&str]) -> Vec<PathBuf> {
    list.iter().map(PathBuf::from).collect()
}

/// The id of the pending request, once a helper thread has asked.
fn wait_for_request(browser: &MediaBrowser) -> (u64, BrowseMode) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some((id, mode, _)) = browser.pending() {
            return (id, mode);
        }
        assert!(Instant::now() < deadline, "no request arrived");
        std::thread::yield_now();
    }
}

#[test]
fn folders_group_and_sort_case_insensitively() {
    let files = paths(&[
        "/m/b/Zed.mp3",
        "/m/A/song.flac",
        "/m/b/alpha.mp3",
        "/m/a2/x.ogg",
    ]);
    let grouped = folders(&files, &BrowseMode::AudioFiles);
    assert_eq!(
        grouped,
        vec![
            (PathBuf::from("/m/A"), paths(&["/m/A/song.flac"])),
            (PathBuf::from("/m/a2"), paths(&["/m/a2/x.ogg"])),
            (
                PathBuf::from("/m/b"),
                paths(&["/m/b/alpha.mp3", "/m/b/Zed.mp3"])
            ),
        ]
    );
}

#[test]
fn one_file_mode_keeps_only_its_extensions() {
    let files = paths(&["/m/a.MP3", "/m/b.flac", "/m/c.wav"]);
    let mode = BrowseMode::OneFile {
        extensions: vec!["mp3".to_owned(), "wav".to_owned()],
    };
    assert_eq!(
        folders(&files, &mode),
        vec![(PathBuf::from("/m"), paths(&["/m/a.MP3", "/m/c.wav"]))]
    );
}

fn files(list: &[&str]) -> Vec<MediaFile> {
    paths(list).into_iter().map(MediaFile::from).collect()
}

#[test]
fn files_are_shared() {
    let browser = MediaBrowser::default();
    browser.set_files(files(&["/m/a.mp3"]));
    assert_eq!(*browser.clone().files(), files(&["/m/a.mp3"]));
}

/// PL18: MediaStore's tags travel with the path, NUL-separated.
#[test]
fn a_record_carries_the_tags() {
    let file = MediaFile::from_record("/m/a.mp3\0Song\0Band\0Disc\x00185500").unwrap();
    assert_eq!(file.path, PathBuf::from("/m/a.mp3"));
    assert_eq!(file.title.as_deref(), Some("Song"));
    assert_eq!(file.artist.as_deref(), Some("Band"));
    assert_eq!(file.album.as_deref(), Some("Disc"));
    assert_eq!(file.duration_secs, Some(185.5));
    assert_eq!(file.title_or_name(), "Song");
}

#[test]
fn a_bare_path_is_a_file_without_tags() {
    let file = MediaFile::from_record("/m/Some song.mp3").unwrap();
    assert_eq!(file, MediaFile::from(PathBuf::from("/m/Some song.mp3")));
    assert_eq!(file.title, None);
    assert_eq!(file.duration_secs, None);
    assert_eq!(file.title_or_name(), "Some song");
}

#[test]
fn empty_or_bad_fields_are_missing() {
    let file = MediaFile::from_record("/m/a.mp3\0\0 \0\0soon").unwrap();
    assert_eq!(file.title, None);
    assert_eq!(file.artist, None);
    assert_eq!(file.album, None);
    assert_eq!(file.duration_secs, None);
    assert_eq!(MediaFile::from_record(""), None);
    assert_eq!(MediaFile::from_record("\0Song"), None);
}

#[test]
fn browse_returns_the_answer() {
    let browser = MediaBrowser::default();
    let asking = browser.clone();
    let thread =
        std::thread::spawn(move || asking.browse(BrowseMode::Folder, Some(PathBuf::from("/m"))));
    let (id, mode) = wait_for_request(&browser);
    assert_eq!(mode, BrowseMode::Folder);
    browser.answer(id, paths(&["/m/a"]));
    assert_eq!(thread.join().ok(), Some(paths(&["/m/a"])));
    assert!(browser.pending().is_none());
}

#[test]
fn a_second_request_answers_the_first_with_nothing() {
    let browser = MediaBrowser::default();
    let first = browser.clone();
    let first = std::thread::spawn(move || first.browse(BrowseMode::AudioFiles, None));
    let (first_id, _) = wait_for_request(&browser);
    let second = browser.clone();
    let second = std::thread::spawn(move || second.browse(BrowseMode::Folder, None));
    assert_eq!(first.join().ok(), Some(Vec::new()));
    let deadline = Instant::now() + Duration::from_secs(5);
    let second_id = loop {
        match browser.pending() {
            Some((id, _, _)) if id != first_id => break id,
            _ => assert!(Instant::now() < deadline, "no second request"),
        }
        std::thread::yield_now();
    };
    browser.answer(second_id, paths(&["/m"]));
    assert_eq!(second.join().ok(), Some(paths(&["/m"])));
}

#[test]
fn a_stale_answer_is_ignored() {
    let browser = MediaBrowser::default();
    let asking = browser.clone();
    let thread = std::thread::spawn(move || asking.browse(BrowseMode::AudioFiles, None));
    let (id, _) = wait_for_request(&browser);
    browser.answer(id + 1, paths(&["/m/wrong.mp3"]));
    assert!(browser.pending().is_some());
    browser.answer(id, paths(&["/m/right.mp3"]));
    assert_eq!(thread.join().ok(), Some(paths(&["/m/right.mp3"])));
}

#[test]
fn cancelling_the_pending_request_answers_nothing() {
    let browser = MediaBrowser::default();
    let asking = browser.clone();
    let thread = std::thread::spawn(move || asking.browse(BrowseMode::AudioFiles, None));
    wait_for_request(&browser);
    browser.cancel_pending();
    assert_eq!(thread.join().ok(), Some(Vec::new()));
    assert!(browser.pending().is_none());
    // Nothing pending: nothing happens.
    browser.cancel_pending();
}

#[test]
fn only_audio_extensions_open_the_browser() {
    assert_eq!(
        BrowseMode::one_file(&["MP3", "jpg"]),
        Some(BrowseMode::OneFile {
            extensions: vec!["MP3".to_owned(), "jpg".to_owned()]
        })
    );
    assert_eq!(BrowseMode::one_file(&["jpg", "png"]), None);
    assert_eq!(BrowseMode::one_file(&["m3u", "pls"]), None);
    assert_eq!(BrowseMode::one_file(&[]), None);
}

/// A list after an empty one means the app can read the device's music
/// now (the media permission was granted): the shell rechecks the files
/// that failed before.
#[test]
fn set_files_tells_when_the_files_become_readable() {
    let browser = MediaBrowser::default();
    assert!(!browser.set_files(Vec::new()), "still nothing");
    assert!(browser.set_files(files(&["/m/a.mp3"])), "access gained");
    assert!(
        !browser.set_files(files(&["/m/a.mp3", "/m/b.mp3"])),
        "a new file"
    );
    assert!(!browser.set_files(Vec::new()), "access lost");
    assert!(browser.set_files(files(&["/m/a.mp3"])), "gained again");
}
