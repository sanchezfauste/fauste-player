#![allow(dead_code, clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::PathBuf;

use fp_model::{
    AppState, AudioFormat, Command, Config, EngineAction, EngineEvent, EntryId, Holder,
    OutputDevice, PlayerId, PlayerRoutes, Route, apply, device_settings, on_event,
};

/// Default config (4 players), one playlist named "Main" with `tracks`
/// entries of 180 s each. Every player shows that playlist. Every player
/// and the cartwall have a Cue output off the on-air output (see
/// `with_cue_routes`).
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
        layout: state.layout.clone(),
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

/// The device the bit-perfect tests play on.
pub fn bp_device() -> OutputDevice {
    OutputDevice {
        backend: "null".to_owned(),
        device: "dac".to_owned(),
    }
}

/// A route to `bp_device`, on its first channel pair.
pub fn bp_route() -> Route {
    Route {
        backend: "null".to_owned(),
        device: "dac".to_owned(),
        first_channel: 0,
    }
}

/// Routes `holder` to `dac`, lists `dac` as bit-perfect, and reports the
/// engine placing `holder` there with `dac` running bit-perfect (live
/// settings L3): nothing is pending afterwards.
pub fn place_on_dac(state: &mut AppState, holder: Holder) -> Vec<EngineAction> {
    let route = bp_route();
    let outputs = &mut state.config.outputs;
    match holder {
        Holder::PlayerMain(p) | Holder::PlayerCue(p) => {
            let routes = outputs.routes.iter_mut().find(|r| r.player == p).unwrap();
            if matches!(holder, Holder::PlayerMain(_)) {
                routes.main = Some(route.clone());
            } else {
                routes.cue = Some(route.clone());
            }
        }
        Holder::CartwallMain => outputs.cartwall.main = Some(route.clone()),
        Holder::CartwallCue => outputs.cartwall.cue = Some(route.clone()),
    }
    if !outputs.bit_perfect.contains(&bp_device()) {
        outputs.bit_perfect.push(bp_device());
    }
    let running = device_settings(&state.config.outputs, &bp_device());
    on_event(
        state,
        EngineEvent::Placed {
            holder,
            route: Some(route),
            device: bp_device(),
            running,
        },
    )
}

/// The engine reports `dac` running at `rate`, having refused `refused` (BR1).
pub fn report_stream_rate(state: &mut AppState, rate: u32, refused: Vec<u32>) -> Vec<EngineAction> {
    on_event(
        state,
        EngineEvent::StreamRate {
            device: bp_device(),
            rate,
            refused,
        },
    )
}

/// Gives the tracks of the first playlist, in order, 16-bit stereo PCM at `rates`.
pub fn set_track_rates(state: &mut AppState, rates: &[u32]) {
    for (entry, rate) in entries(state).into_iter().zip(rates) {
        let track = state.playlists.entry(entry).unwrap().track;
        state.library.get_mut(track).unwrap().format = Some(AudioFormat {
            sample_rate: *rate,
            bits: Some(16),
            channels: 2,
            dsd_rate: None,
        });
    }
}
