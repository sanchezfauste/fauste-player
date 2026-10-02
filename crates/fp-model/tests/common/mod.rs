#![allow(dead_code, clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use fp_model::{AppState, Command, Config, EntryId, PlayerId, apply};

/// Default config (4 players), one playlist named "Main" with `tracks`
/// entries of 180 s each. Every player shows that playlist.
pub fn fixture(tracks: usize) -> AppState {
    let mut state = AppState::new(Config::default(), "Main");
    let playlist = state.playlists.first_id().unwrap();
    let paths = (0..tracks)
        .map(|i| PathBuf::from(format!("/music/track{i}.flac")))
        .collect();
    apply(
        &mut state,
        Command::InsertPaths {
            playlist,
            index: 0,
            paths,
        },
    )
    .unwrap();
    for track in state.library.iter_mut() {
        track.duration_secs = 180.0;
    }
    state
}

pub fn entries(state: &AppState) -> Vec<EntryId> {
    let playlist = state.playlists.first_id().unwrap();
    state
        .playlists
        .get(playlist)
        .unwrap()
        .entries
        .iter()
        .map(|e| e.id)
        .collect()
}

pub fn p0(state: &AppState) -> PlayerId {
    state.players[0].id
}

/// Snapshots the players' sessions and restores them over the same library
/// and playlists, as after a restart.
pub fn roundtrip(state: &AppState) -> AppState {
    let parts = fp_model::RestoreParts {
        config: state.config.clone(),
        library: state.library.clone(),
        playlists: state.playlists.clone(),
        cart_pages: state.cartwall.pages.clone(),
        cartwall_session: state.cartwall.session(),
        ids: state.ids.clone(),
    };
    AppState::restore(parts, &state.sessions(|_| 10.0), "Main").0
}
