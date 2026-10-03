#![allow(dead_code, clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use fp_model::{AppState, Command, Config, EntryId, PlayerId, PlayerRoutes, Route, apply};

/// Default config (4 players), one playlist named "Main" with `tracks`
/// entries of 180 s each. Every player shows that playlist. Every player
/// and the cartwall have a Cue output apart from Main (see `with_cue_routes`).
pub fn fixture(tracks: usize) -> AppState {
    let mut state = AppState::new(Config::default(), "Main");
    with_cue_routes(&mut state);
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

/// A Cue route on headphones for every player and the cartwall, with Main
/// on the default output: a CUE needs a Cue output (spec §4.6).
pub fn with_cue_routes(state: &mut AppState) {
    let phones = Route {
        backend: "null".to_owned(),
        device: "phones".to_owned(),
        first_channel: 0,
    };
    state.config.outputs.routes = state
        .players
        .iter()
        .map(|p| PlayerRoutes {
            player: p.id,
            main: None,
            cue: Some(phones.clone()),
        })
        .collect();
    state.config.outputs.cartwall.cue = Some(phones);
}
