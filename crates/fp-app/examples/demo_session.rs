//! Writes a demo state under `FAUSTE_HOME`: four players and three
//! playlists filled with the audio files of a folder, with the first
//! players paused mid-track.
//!
//! `FAUSTE_HOME=/tmp/fp-demo cargo run -p fp-app --example demo_session -- <music folder>`

use std::path::PathBuf;
use std::process::ExitCode;

use fp_app::bootstrap;
use fp_app::ui::files::audio_paths;
use fp_model::{AppState, Command, Config, Limits, PlaylistId};
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
        chosen.rotate_left(n * 3);
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
        if n < 3 {
            let _ = fp_model::apply(&mut state, Command::Play(*player));
        }
    }
    let store = Store::new(paths, Limits::default());
    let saved = store
        .save_config(&state)
        .and_then(|()| store.save_playlists(&state))
        .and_then(|()| store.save_session(&state, |p| 40.0 + f64::from(p.0 as u32 % 7) * 19.0));
    match saved {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("demo_session: {e}");
            ExitCode::FAILURE
        }
    }
}
