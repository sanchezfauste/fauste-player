#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The conductor wires the model rules to the engine end to end.

mod support;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use fp_backends::{AudioBackend, NullBackend, OfflineBackend, OfflineDevice};
use fp_engine::conductor::{Conductor, ConductorHandle};
use fp_engine::engine::{Engine, EngineSettings};
use fp_model::{
    AppState, Command, Config, EntryId, FileState, ModelError, PlayMode, PlayerId, Transport,
};
use support::tagged_opener;

const BLOCK: usize = 480;
const TRACK_FRAMES: u64 = 48_000;

fn model(players: usize, tracks: u64) -> AppState {
    let mut config = Config::default();
    config.players.count = players;
    config.outputs.backend = Some("offline".into());
    config.outputs.buffer_frames = BLOCK as u32;
    let mut state = AppState::new(config, "Main");
    let playlist = state.playlists.first_id().unwrap();
    let paths = (1..=tracks)
        .map(|n| PathBuf::from(format!("track{n}")))
        .collect();
    fp_model::apply(
        &mut state,
        Command::InsertPaths {
            playlist,
            index: 0,
            paths,
        },
    )
    .unwrap();
    for t in state.library.iter_mut() {
        t.duration_secs = TRACK_FRAMES as f64 / 48_000.0;
    }
    state
}

fn offline_conductor(state: AppState) -> (Conductor, ConductorHandle, OfflineDevice, Instant) {
    let backend = OfflineBackend::new();
    let device = backend.add_device("main", 2);
    let settings = EngineSettings::from_config(&state.config);
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let engine = Engine::new(backends, settings, tagged_opener(TRACK_FRAMES));
    let now = Instant::now();
    let (conductor, handle) = Conductor::new(state, Vec::new(), engine, now);
    (conductor, handle, device, now)
}

fn entries(state: &AppState) -> Vec<EntryId> {
    let p = state.playlists.first_id().unwrap();
    state
        .playlists
        .get(p)
        .unwrap()
        .entries
        .iter()
        .map(|e| e.id)
        .collect()
}

#[test]
fn continuous_playback_chains_tracks_in_the_model_and_on_air() {
    let (mut conductor, handle, device, mut now) = offline_conductor(model(1, 3));
    let e = entries(conductor.state());
    let p = conductor.state().players[0].id;
    assert!(handle.send(Command::Play(p)));
    conductor.tick(now);
    std::thread::sleep(Duration::from_millis(30));
    let mut heard = Vec::new();
    for _ in 0..150 {
        conductor.tick(now);
        heard.extend(device.render(BLOCK).unwrap().chunks(2).map(|f| f[0]));
        now += Duration::from_millis(10);
        std::thread::yield_now();
    }
    let model = handle.model.load();
    let player = model.player(p).unwrap();
    assert_eq!(
        player.current,
        Some(e[1]),
        "the second track is on air after the first ended"
    );
    assert!(model.playlists.entry(e[0]).unwrap().is_played_by(p));
    assert!(heard.iter().any(|v| (*v as u64) / 100_000 == 2));
    assert!(handle.telemetry.load().model_version > 0);
}

#[test]
fn refused_commands_are_reported_to_the_ui() {
    let (mut conductor, handle, _device, now) = offline_conductor(model(1, 2));
    let p = conductor.state().players[0].id;
    handle.send(Command::SetMode(p, PlayMode::Single));
    handle.send(Command::ToggleStopAfterCurrent(p));
    conductor.tick(now);
    assert_eq!(
        handle.rejected.try_recv(),
        Ok(ModelError::StopAfterInSingle)
    );
}

#[test]
fn the_spawned_conductor_runs_in_real_time_on_the_null_backend() {
    let mut state = model(1, 2);
    state.config.outputs.backend = Some("null".into());
    let settings = EngineSettings::from_config(&state.config);
    let engine = Engine::new(
        vec![Arc::new(NullBackend)],
        settings,
        tagged_opener(TRACK_FRAMES),
    );
    let (conductor, handle) = Conductor::new(state, Vec::new(), engine, Instant::now());
    let p = conductor.state().players[0].id;
    let handle = conductor.spawn(handle, Duration::from_millis(5)).unwrap();
    handle.send(Command::Play(p));
    // Poll with a generous deadline instead of a fixed sleep (slow CI runners).
    let deadline = Instant::now() + Duration::from_secs(10);
    let position = loop {
        let t = handle.telemetry.load();
        let position = t
            .players
            .iter()
            .find(|(id, _)| *id == p)
            .and_then(|(_, t)| t.position_secs);
        if position.is_some_and(|pos| pos > 0.1) || Instant::now() > deadline {
            break position;
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    assert!(
        position.is_some_and(|pos| pos > 0.1),
        "position {position:?}"
    );
    drop(handle);
}

#[test]
fn a_flooded_command_queue_refuses_instead_of_blocking() {
    let (conductor, handle, _device, _now) = offline_conductor(model(1, 2));
    let p = conductor.state().players[0].id;
    let started = Instant::now();
    let accepted = (0..5_000)
        .filter(|_| handle.send(Command::SetVolume(p, 0.5)))
        .count();
    assert!(accepted < 5_000, "the queue is bounded");
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "sending never blocks the UI"
    );
}

#[test]
fn test_tones_requested_through_the_handle_play_on_their_route() {
    let (mut conductor, handle, device, now) = offline_conductor(model(1, 1));
    let route = fp_model::Route {
        backend: "offline".into(),
        device: "main".into(),
        first_channel: 0,
    };
    assert!(handle.test_tone(route, 1_000.0));
    conductor.tick(now);
    let out = device.render(BLOCK).unwrap();
    assert!(out.iter().any(|v| *v != 0.0));
}

/// xorshift, deterministic across runs.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn stress(simulated_secs: u64) {
    let (mut conductor, handle, device, mut now) = offline_conductor(model(8, 30));
    let players: Vec<PlayerId> = conductor.state().players.iter().map(|p| p.id).collect();
    let e = entries(conductor.state());
    let mut rng = Rng(0x5eed);
    let mut last_move: HashMap<PlayerId, (f64, u64, Instant)> = HashMap::new();
    let mut history: HashMap<PlayerId, Vec<(u64, String)>> = HashMap::new();
    let mut fading_since: HashMap<PlayerId, (u64, Instant)> = HashMap::new();
    let blocks = simulated_secs * 100;
    for block in 0..blocks {
        if block % 25 == 0 {
            let p = players[rng.below(players.len())];
            let command = match rng.below(10) {
                0..=2 => Command::Play(p),
                3 => Command::Pause(p),
                4 => Command::Stop(p),
                5 => Command::FadeStop(p),
                6 => Command::SetNext(p, e[rng.below(e.len())]),
                7 => Command::SetMode(
                    p,
                    if rng.below(2) == 0 {
                        PlayMode::Single
                    } else {
                        PlayMode::Continuous
                    },
                ),
                8 => Command::Seek(p, rng.below(900) as f64 / 1000.0),
                _ => Command::SetVolume(p, rng.below(100) as f32 / 100.0),
            };
            history
                .entry(p)
                .or_default()
                .push((block, format!("{command:?}")));
            handle.send(command);
        }
        conductor.tick(now);
        device.render(BLOCK).unwrap();
        now += Duration::from_millis(10);
        if block % 4 == 0 {
            std::thread::yield_now();
        }
        // A playing (not paused) player must never stand still for long, a
        // fade must never outlive a whole track, and sources must not pile up.
        let model = handle.model.load();
        let telemetry = handle.telemetry.load();
        for pl in &model.players {
            let since = fading_since.entry(pl.id).or_insert((block, Instant::now()));
            if !pl.fading {
                *since = (block, Instant::now());
            }
            assert!(
                block - since.0 < 1_000 || since.1.elapsed() < Duration::from_secs(2),
                "player {:?} has been fading for 10 simulated seconds",
                pl.id
            );
        }
        assert!(
            conductor.engine().attached_sources() <= players.len() * 8,
            "sources are piling up: {}",
            conductor.engine().attached_sources()
        );
        for (id, t) in &telemetry.players {
            let playing = model
                .player(*id)
                .is_ok_and(|p| p.transport == Transport::Playing);
            match (playing, t.position_secs) {
                (true, Some(pos)) => {
                    let entry = last_move.entry(*id).or_insert((pos, block, Instant::now()));
                    if (pos - entry.0).abs() > f64::EPSILON {
                        *entry = (pos, block, Instant::now());
                    }
                    // Simulated time runs far faster than the (real-time)
                    // worker threads, so "stuck" also requires real time to pass.
                    let stuck =
                        block - entry.1 >= 500 && entry.2.elapsed() > Duration::from_secs(2);
                    if stuck {
                        let h = history
                            .get(id)
                            .map(|v| v[v.len().saturating_sub(8)..].to_vec());
                        let pl = model.player(*id).unwrap();
                        panic!(
                            "ENGINE {} || player {id:?} stuck at {pos} since block {}; now {block}; state {:?} cur {:?} next {:?} fading {}; history {h:#?}",
                            conductor.engine().describe_player(*id),
                            entry.1,
                            pl.transport,
                            pl.current,
                            pl.next,
                            pl.fading
                        );
                    }
                }
                _ => {
                    last_move.remove(id);
                }
            }
        }
    }
    let telemetry = handle.telemetry.load();
    assert_eq!(telemetry.dropped_commands, 0);
    assert_eq!(
        telemetry.slot_exhaustions, 0,
        "mixer slots must never run out"
    );
    let model = handle.model.load();
    assert!(
        model.library.iter().all(|t| t.file_state == FileState::Ok),
        "every test file is valid: none may end up marked unreadable"
    );
}

#[test]
fn eight_players_survive_ten_simulated_minutes_of_random_commands() {
    stress(600);
}

/// The spec's long soak (spec §11): run with `cargo test -- --ignored`.
#[test]
#[ignore = "long soak: 6 simulated hours"]
fn eight_players_survive_six_simulated_hours_of_random_commands() {
    stress(6 * 3600);
}

#[test]
fn firing_an_exclusive_cart_stops_the_others_on_air() {
    let mut state = model(1, 0);
    let page = state.cartwall.pages[0].id;
    for index in 0..2 {
        fp_model::apply(
            &mut state,
            Command::AssignCartFile {
                page,
                index,
                path: PathBuf::from(format!("track{}", index + 1)),
            },
        )
        .unwrap();
    }
    let edit = fp_model::CartEdit {
        name: "Exclusive".into(),
        kind: fp_model::CartKind::Jingle,
        looped: false,
        exclusive: true,
    };
    fp_model::apply(
        &mut state,
        Command::SetCart {
            page,
            index: 1,
            edit,
        },
    )
    .unwrap();
    let (a, b) = (
        state.cartwall.pages[0].carts[0].id,
        state.cartwall.pages[0].carts[1].id,
    );
    let (mut conductor, handle, device, mut now) = offline_conductor(state);
    let mut run = |conductor: &mut Conductor, blocks: usize| {
        for _ in 0..blocks {
            conductor.tick(now);
            device.render(BLOCK).unwrap();
            now += Duration::from_millis(10);
            std::thread::sleep(Duration::from_millis(1));
        }
    };
    handle.send(Command::FireCart(a));
    run(&mut conductor, 20);
    let on_air: Vec<_> = handle
        .telemetry
        .load()
        .carts
        .iter()
        .map(|(c, _)| *c)
        .collect();
    assert_eq!(on_air, vec![a]);
    handle.send(Command::FireCart(b));
    run(&mut conductor, 20);
    let on_air: Vec<_> = handle
        .telemetry
        .load()
        .carts
        .iter()
        .map(|(c, _)| *c)
        .collect();
    assert_eq!(on_air, vec![b]);
    assert_eq!(conductor.state().cartwall.playing.len(), 1);
}

#[test]
fn the_meter_follows_a_playing_source_and_falls_after_stop() {
    let (mut conductor, handle, device, mut now) = offline_conductor(model(1, 3));
    let p = conductor.state().players[0].id;
    let reading = |handle: &ConductorHandle| {
        handle
            .telemetry
            .load()
            .players
            .iter()
            .find(|(id, _)| *id == p)
            .map(|(_, t)| t.meter)
            .unwrap()
    };
    assert!(handle.send(Command::Play(p)));
    conductor.tick(now);
    std::thread::sleep(Duration::from_millis(30));
    for _ in 0..20 {
        conductor.tick(now);
        device.render(BLOCK);
        now += Duration::from_millis(10);
    }
    let playing = reading(&handle);
    assert!(playing.level_db[0] > -60.0, "{playing:?}");
    assert!(handle.send(Command::Stop(p)));
    for _ in 0..30 {
        conductor.tick(now);
        device.render(BLOCK);
        now += Duration::from_millis(10);
    }
    let stopped = reading(&handle);
    assert!(
        stopped.level_db[0] < playing.level_db[0] - 3.0,
        "falls at the ballistics' rate: {stopped:?}"
    );
    assert!(stopped.level_db[0] > -120.0, "not cut to silence at once");
}

#[test]
fn true_peak_can_be_switched_while_playing() {
    let (mut conductor, handle, _device, now) = offline_conductor(model(1, 1));
    let p = conductor.state().players[0].id;
    assert!(handle.send(Command::Play(p)));
    conductor.tick(now);
    assert!(!conductor.engine().true_peak());
    let mut config = conductor.state().config.clone();
    config.meter.true_peak = true;
    assert!(handle.send(Command::UpdateConfig(Box::new(config))));
    conductor.tick(now);
    assert!(conductor.engine().true_peak());
}

#[test]
fn a_programme_meter_preset_reaches_the_buses_while_playing() {
    let (mut conductor, handle, _device, now) = offline_conductor(model(1, 1));
    let p = conductor.state().players[0].id;
    assert!(handle.send(Command::Play(p)));
    conductor.tick(now);
    let mut config = conductor.state().config.clone();
    config.meter.ballistics = fp_model::MeterBallistics::EbuPpm;
    assert!(handle.send(Command::UpdateConfig(Box::new(config.clone()))));
    conductor.tick(now);
    assert_eq!(
        conductor.engine().meter_integration(),
        fp_engine::meter::mixer_integration(&config.meter)
    );
    assert!(conductor.engine().meter_integration().0 > 0.0);
}

/// Plays the first entry of player 0 for 200 ms, stops it and lets the bar
/// fall for 500 ms. Returns the meter while playing and after the fall.
fn play_then_stop(
    conductor: &mut Conductor,
    handle: &ConductorHandle,
    device: &OfflineDevice,
    now: &mut Instant,
) -> (
    fp_engine::meter::MeterReading,
    fp_engine::meter::MeterReading,
) {
    let p = conductor.state().players[0].id;
    let reading = |handle: &ConductorHandle| {
        handle
            .telemetry
            .load()
            .players
            .iter()
            .find(|(id, _)| *id == p)
            .map(|(_, t)| t.meter)
            .unwrap()
    };
    assert!(handle.send(Command::Play(p)));
    conductor.tick(*now);
    std::thread::sleep(Duration::from_millis(30));
    for _ in 0..20 {
        conductor.tick(*now);
        device.render(BLOCK);
        *now += Duration::from_millis(10);
    }
    let playing = reading(handle);
    assert!(handle.send(Command::Stop(p)));
    for _ in 0..50 {
        conductor.tick(*now);
        *now += Duration::from_millis(10);
    }
    (playing, reading(handle))
}

fn meter_of(handle: &ConductorHandle, p: PlayerId) -> fp_engine::meter::MeterReading {
    handle
        .telemetry
        .load()
        .players
        .iter()
        .find(|(id, _)| *id == p)
        .map(|(_, t)| t.meter)
        .unwrap()
}

#[test]
fn the_meter_maximum_outlasts_a_stop_and_restarts_with_the_next_entry() {
    let (mut conductor, handle, device, mut now) = offline_conductor(model(1, 3));
    let p = conductor.state().players[0].id;
    let (playing, stopped) = play_then_stop(&mut conductor, &handle, &device, &mut now);
    assert!(playing.max_db > -60.0, "{playing:?}");
    // The last block still counts after the stop; nothing clears it.
    assert!(stopped.max_db >= playing.max_db, "kept after the stop");
    assert!(stopped.level_db[0] < stopped.max_db - 3.0, "{stopped:?}");
    assert!(handle.send(Command::Play(p)));
    conductor.tick(now);
    let restarted = meter_of(&handle, p);
    assert!(
        restarted.max_db < stopped.max_db - 3.0,
        "a new entry restarts it: {restarted:?}"
    );
}

#[test]
fn the_meter_maximum_restarts_when_the_same_entry_plays_again() {
    let (mut conductor, handle, device, mut now) = offline_conductor(model(1, 1));
    let p = conductor.state().players[0].id;
    let (_, stopped) = play_then_stop(&mut conductor, &handle, &device, &mut now);
    let first = conductor.state().playlists.iter().next().unwrap().entries[0].id;
    assert!(handle.send(Command::Play(p)));
    conductor.tick(now);
    assert_eq!(
        conductor.state().players[0].current,
        Some(first),
        "the same entry"
    );
    let restarted = meter_of(&handle, p);
    assert!(
        restarted.max_db < stopped.max_db - 3.0,
        "playing it again restarts it: {restarted:?}"
    );
}

#[test]
fn the_meter_maximum_restarts_when_the_entry_restarts_within_one_tick() {
    let (mut conductor, handle, device, mut now) = offline_conductor(model(1, 1));
    let p = conductor.state().players[0].id;
    let (_, stopped) = play_then_stop(&mut conductor, &handle, &device, &mut now);
    // Play, then Stop and Play again before the conductor meters once more.
    assert!(handle.send(Command::Play(p)));
    for _ in 0..20 {
        conductor.tick(now);
        device.render(BLOCK);
        now += Duration::from_millis(10);
    }
    conductor.tick(now); // meters the last block
    let playing = meter_of(&handle, p);
    assert!(handle.send(Command::Stop(p)));
    assert!(handle.send(Command::Play(p)));
    conductor.tick(now);
    let restarted = meter_of(&handle, p);
    assert!(playing.max_db > -60.0 && stopped.max_db > -60.0);
    assert!(
        restarted.max_db < playing.max_db - 3.0,
        "a new start restarts it: {restarted:?}"
    );
}

#[test]
fn the_meter_maximum_restarts_on_request() {
    let (mut conductor, handle, device, mut now) = offline_conductor(model(1, 3));
    let p = conductor.state().players[0].id;
    let (_, stopped) = play_then_stop(&mut conductor, &handle, &device, &mut now);
    assert!(handle.reset_meter_max(p));
    conductor.tick(now);
    let restarted = meter_of(&handle, p);
    assert!(restarted.max_db < stopped.max_db - 3.0, "{restarted:?}");
}
