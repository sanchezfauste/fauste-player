//! Writes a demo state under `FAUSTE_HOME`: four players and three
//! playlists filled with the audio files of a folder, with the first
//! players paused mid-track some way into their playlist.
//!
//! `FAUSTE_HOME=/tmp/fp-demo cargo run -p fp-app --example demo_session -- <music folder>`

use std::path::PathBuf;
use std::process::ExitCode;

use fp_app::bootstrap;
use fp_app::ui::files::audio_paths;
use fp_model::{AppState, CartEdit, CartKind, Command, Config, Limits, PlaylistId};
use fp_store::Store;

fn main() -> ExitCode {
    // Never write the demo over a real installation.
    if std::env::var_os(bootstrap::HOME_VAR).is_none() {
        eprintln!(
            "demo_session: set {} to a scratch directory first",
            bootstrap::HOME_VAR
        );
        return ExitCode::FAILURE;
    }
    let (Some(paths), Some(folder)) = (bootstrap::paths(), std::env::args().nth(1)) else {
        eprintln!("usage: FAUSTE_HOME=<dir> demo_session <music folder>");
        return ExitCode::FAILURE;
    };
    let files = audio_paths(&[PathBuf::from(folder)]);
    if files.is_empty() {
        eprintln!(
            "demo_session: no audio files directly inside that folder (sub-folders are not read)"
        );
        return ExitCode::FAILURE;
    }
    let mut config = Config::default();
    config.players.count = 4;
    let mut state = AppState::new(config, "Morning");
    let names = ["Afternoon", "Night"];
    for name in names {
        let _ = fp_model::apply(&mut state, Command::CreatePlaylist { name: name.into() });
    }
    let lists: Vec<PlaylistId> = state.playlists.iter().map(|p| p.id).collect();
    for (n, list) in lists.iter().enumerate() {
        let mut chosen = files.clone();
        let len = chosen.len();
        chosen.rotate_left((n * 3) % len);
        let _ = fp_model::apply(
            &mut state,
            Command::InsertPaths {
                playlist: *list,
                index: 0,
                paths: chosen,
            },
        );
    }
    let players: Vec<_> = state.players.iter().map(|p| p.id).collect();
    for (n, player) in players.iter().enumerate() {
        if let Some(list) = lists.get(n % lists.len()) {
            let _ = fp_model::apply(&mut state, Command::ShowPlaylist(*player, *list));
            let first = state
                .playlists
                .get(*list)
                .and_then(|p| p.entries.first())
                .map(|e| e.id);
            if let Some(entry) = first.filter(|_| n < 3) {
                let _ = fp_model::apply(&mut state, Command::SetNext(*player, entry));
            }
        }
        // Some way into the hour: player 1 on its 5th track, player 2 on
        // its 3rd, player 3 on its 2nd (the played rows show dimmed);
        // player 4 waits. Each stop leaves a played track behind.
        let plays = [5, 3, 2].get(n).copied().unwrap_or(0);
        for k in 0..plays {
            if k > 0 {
                let _ = fp_model::apply(&mut state, Command::Stop(*player));
            }
            let _ = fp_model::apply(&mut state, Command::Play(*player));
        }
    }
    // Some carts on the first page, from the shortest files.
    let mut short = files.clone();
    short.sort_by_key(|p| std::fs::metadata(p).map(|m| m.len()).unwrap_or(u64::MAX));
    let page = state.cartwall.pages[0].id;
    let _ = fp_model::apply(
        &mut state,
        Command::RenameCartPage {
            page,
            name: "General".into(),
        },
    );
    let kinds = [CartKind::Jingle, CartKind::Effect, CartKind::Spot];
    for (index, path) in short.iter().take(6).enumerate() {
        let _ = fp_model::apply(
            &mut state,
            Command::AssignCartFile {
                page,
                index,
                path: path.clone(),
            },
        );
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let edit = CartEdit {
            name,
            kind: kinds[index % kinds.len()],
            looped: index == 5,
            exclusive: index == 0,
        };
        let _ = fp_model::apply(&mut state, Command::SetCart { page, index, edit });
    }
    let _ = fp_model::apply(
        &mut state,
        Command::CreateCartPage {
            name: "Effects".into(),
        },
    );
    let store = Store::new(paths, Limits::default());
    let saved = store
        .save_config(&state)
        .and_then(|()| store.save_playlists(&state))
        .and_then(|()| store.save_carts(&state))
        .and_then(|()| store.save_session(&state, |p| 8.0 + f64::from(p.0 as u32 % 4) * 3.0));
    match saved {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("demo_session: {e}");
            ExitCode::FAILURE
        }
    }
}
