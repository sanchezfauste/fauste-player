#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Phase 2 spec P2.7: playlist import and export from the interface.

mod support;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use egui_kittest::kittest::Queryable;
use fp_app::ui::playlist_files::export_entries;
use fp_model::Command;
use fp_store::playlist_io::write_m3u8;
use support::{harness, state};

#[test]
fn imported_playlists_become_new_playlists() {
    let dir = tempfile::tempdir().unwrap();
    let list = dir.path().join("Morning show.m3u");
    std::fs::write(
        &list,
        "#EXTM3U\n/music/a.mp3\nhttp://radio/stream\nb.flac\n",
    )
    .unwrap();
    let (mut h, fake) = harness(state(1, 0));
    h.state_mut().import_playlist(list);
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut created = None;
    while created.is_none() && Instant::now() < deadline {
        h.run_steps(1);
        created = fake.take_sent().into_iter().find_map(|c| match c {
            Command::CreatePlaylistFromPaths { name, paths } => Some((name, paths)),
            _ => None,
        });
        std::thread::sleep(Duration::from_millis(5));
    }
    let (name, paths) = created.expect("a playlist is created");
    assert_eq!(name, "Morning show");
    assert_eq!(
        paths,
        vec![PathBuf::from("/music/a.mp3"), dir.path().join("b.flac")]
    );
    h.run_steps(2);
    assert!(
        h.query_all_by_label_contains("1 stream").next().is_some(),
        "the skipped stream is reported"
    );
}

#[test]
fn exported_playlists_are_valid_m3u8() {
    let s = state(1, 2);
    let playlist = s.playlists.first_id().unwrap();
    let entries = export_entries(&s, playlist);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].path, PathBuf::from("/music/Song 1.mp3"));
    let text = write_m3u8(&entries);
    assert!(text.starts_with("#EXTM3U\n"));
    assert!(text.contains("/music/Song 2.mp3"));
}

#[test]
fn an_import_with_nothing_playable_creates_no_playlist() {
    let dir = tempfile::tempdir().unwrap();
    let list = dir.path().join("streams.m3u");
    std::fs::write(&list, "http://radio/one\nhttp://radio/two\n").unwrap();
    let (mut h, fake) = harness(state(1, 0));
    h.state_mut().import_playlist(list);
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline
        && h.query_all_by_label_contains("streams skipped")
            .next()
            .is_none()
    {
        h.run_steps(1);
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(
        !fake
            .take_sent()
            .iter()
            .any(|c| matches!(c, Command::CreatePlaylistFromPaths { .. }))
    );
    assert!(
        h.query_all_by_label_contains("streams skipped")
            .next()
            .is_some()
    );
}

#[test]
fn a_playlist_path_argument_is_imported_at_start() {
    let dir = tempfile::tempdir().unwrap();
    let list = dir.path().join("Night.m3u");
    std::fs::write(&list, "/music/a.mp3\n").unwrap();
    // As `main` does: queued before the interface has run a frame.
    let (mut h, fake) = support::harness_with(state(1, 0), |app| app.import_playlist(list));
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut created = None;
    while created.is_none() && Instant::now() < deadline {
        h.run_steps(1);
        created = fake.take_sent().into_iter().find_map(|c| match c {
            Command::CreatePlaylistFromPaths { name, .. } => Some(name),
            _ => None,
        });
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(created.as_deref(), Some("Night"));
}

#[test]
fn playlists_handed_over_by_a_second_start_are_imported() {
    let dir = tempfile::tempdir().unwrap();
    let list = dir.path().join("Late.m3u");
    std::fs::write(&list, "/music/a.mp3\n").unwrap();
    let (tx, rx) = crossbeam_channel::unbounded();
    let (mut h, fake) = support::harness_from(state(1, 0), |app| app.with_inbox(rx));
    tx.send(list).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut created = None;
    while created.is_none() && Instant::now() < deadline {
        h.run_steps(1);
        created = fake.take_sent().into_iter().find_map(|c| match c {
            Command::CreatePlaylistFromPaths { name, .. } => Some(name),
            _ => None,
        });
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(created.as_deref(), Some("Late"));
}
