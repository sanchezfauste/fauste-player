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
