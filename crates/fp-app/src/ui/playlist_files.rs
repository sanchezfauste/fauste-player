//! Playlist files from the interface (Phase 2 spec P2.7): import M3U, M3U8
//! and PLS as new playlists, export a playlist as M3U8. File dialogs and
//! file I/O run on helper threads; results come back as `FileOutcome`s.

use std::path::{Path, PathBuf};

use crossbeam_channel::Sender;
use fp_model::{AppState, Limits, PlaylistId};
use fp_store::playlist_io::{ExportEntry, ImportedPlaylist, parse_playlist, write_m3u8};

/// Extensions read as playlist files.
pub const PLAYLIST_EXTENSIONS: &[&str] = &["m3u", "m3u8", "pls"];

pub fn is_playlist_file(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()).is_some_and(|e| {
        PLAYLIST_EXTENSIONS
            .iter()
            .any(|p| p.eq_ignore_ascii_case(e))
    })
}

/// What a background playlist-file job produced.
pub enum FileOutcome {
    Imported {
        name: String,
        result: Result<ImportedPlaylist, String>,
    },
    Exported(Result<PathBuf, String>),
}

/// The entries of `playlist` as they are written to an M3U8 file.
pub fn export_entries(state: &AppState, playlist: PlaylistId) -> Vec<ExportEntry> {
    state
        .playlists
        .get(playlist)
        .map(|p| {
            p.entries
                .iter()
                .filter_map(|e| state.library.get(e.track))
                .map(|t| ExportEntry {
                    path: t.path.clone(),
                    title: Some(if t.artist.is_empty() {
                        t.title.clone()
                    } else {
                        format!("{} - {}", t.artist, t.title)
                    }),
                    duration_secs: (t.duration_secs > 0.0).then(|| t.play_length_secs()),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn spawn(ctx: &egui::Context, job: impl FnOnce() + Send + 'static) {
    let ctx = ctx.clone();
    let spawned = std::thread::Builder::new()
        .name("fp-playlist-file".to_owned())
        .spawn(move || {
            job();
            ctx.request_repaint();
        });
    if let Err(e) = spawned {
        tracing::error!(error = %e, "could not start a helper thread");
    }
}

fn read(path: &Path, limits: &Limits) -> FileOutcome {
    let name = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let result = std::fs::read(path)
        .map_err(|e| e.to_string())
        .and_then(|bytes| parse_playlist(&bytes, path, limits).map_err(|e| e.to_string()));
    FileOutcome::Imported { name, result }
}

/// Reads and parses `path` in the background.
pub(crate) fn import(ctx: &egui::Context, path: PathBuf, limits: Limits, tx: Sender<FileOutcome>) {
    spawn(ctx, move || {
        let _ = tx.send(read(&path, &limits));
    });
}

/// Asks for a playlist file, then imports it.
pub(crate) fn import_with_dialog(ctx: &egui::Context, limits: Limits, tx: Sender<FileOutcome>) {
    spawn(ctx, move || {
        let picked = pollster::block_on(
            rfd::AsyncFileDialog::new()
                .add_filter("M3U / PLS", PLAYLIST_EXTENSIONS)
                .pick_file(),
        );
        if let Some(file) = picked {
            let _ = tx.send(read(file.path(), &limits));
        }
    });
}

/// Asks where to save, then writes the entries as M3U8.
pub(crate) fn export_with_dialog(
    ctx: &egui::Context,
    file_name: String,
    entries: Vec<ExportEntry>,
    tx: Sender<FileOutcome>,
) {
    spawn(ctx, move || {
        let picked = pollster::block_on(
            rfd::AsyncFileDialog::new()
                .add_filter("M3U8", &["m3u8"])
                .set_file_name(file_name)
                .save_file(),
        );
        if let Some(file) = picked {
            let path = file.path().to_path_buf();
            let result = std::fs::write(&path, write_m3u8(&entries))
                .map(|()| path)
                .map_err(|e| e.to_string());
            let _ = tx.send(FileOutcome::Exported(result));
        }
    });
}
