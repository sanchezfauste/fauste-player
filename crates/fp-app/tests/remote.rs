//! The remote API drives the real conductor (Offline backend).
#![allow(clippy::unwrap_used)]

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::time::{Duration, Instant};

use fp_app::services::MediaCache;
use fp_backends::{AudioBackend, OfflineBackend};
use fp_engine::conductor::Conductor;
use fp_engine::engine::{Engine, EngineSettings};
use fp_engine::worker::file_opener;
use fp_model::{Command, Transport};
use fp_remote::ServerStatus;
use fp_store::{AppPaths, Store};

fn tone(path: &std::path::Path) {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 48_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(path, spec).unwrap();
    for n in 0..48_000 * 3 {
        let v = ((n as f32 * 440.0 * std::f32::consts::TAU / 48_000.0).sin() * 8000.0) as i16;
        w.write_sample(v).unwrap();
        w.write_sample(v).unwrap();
    }
    w.finalize().unwrap();
}

fn post(addr: SocketAddr, path: &str) -> (u16, String) {
    let mut s = TcpStream::connect_timeout(&addr, Duration::from_secs(2)).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    write!(
        s,
        "POST {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\nContent-Length: 0\r\n\r\n"
    )
    .unwrap();
    let mut out = String::new();
    s.read_to_string(&mut out).unwrap();
    let status = out.split(' ').nth(1).unwrap().parse().unwrap();
    (status, out.split_once("\r\n\r\n").unwrap().1.to_owned())
}

#[test]
fn a_remote_play_puts_the_player_on_air() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("tone.wav");
    tone(&file);
    let paths = AppPaths::under(dir.path());
    let store = Store::new(paths.clone(), Default::default());
    let mut loaded = store.load("Main");
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let state = &mut loaded.state;
    state.config.outputs.backend = Some("offline".into());
    state.config.outputs.buffer_frames = 480;
    state.config.remote.http.enabled = true;
    state.config.remote.http.port = port;
    let playlist = state.playlists.first_id().unwrap();
    fp_model::apply(
        state,
        Command::InsertPaths {
            playlist,
            index: 0,
            paths: vec![file],
        },
    )
    .unwrap();
    let player = state.players[0].id;
    let limits = state.config.limits.clone();

    let backend = OfflineBackend::new();
    let _device = backend.add_device("main", 2);
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let engine = Engine::new(
        backends,
        EngineSettings::from_config(&loaded.state.config),
        file_opener(),
    );
    let (mut conductor, handle) =
        Conductor::new(loaded.state, loaded.actions, engine, Instant::now());
    let handle = Arc::new(handle);
    let remote = fp_app::remote::start(
        handle.clone(),
        MediaCache::default(),
        paths.cache_dir.join("analysis"),
        &limits,
    )
    .unwrap();

    let deadline = Instant::now() + Duration::from_secs(5);
    let addr = loop {
        if let ServerStatus::Listening(addr) = remote.status().http {
            break addr;
        }
        assert!(Instant::now() < deadline, "{:?}", remote.status());
        std::thread::sleep(Duration::from_millis(10));
    };
    let (status, body) = post(addr, &format!("/api/v1/players/{}/play", player.0));
    assert_eq!(status, 202, "{body}");

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        conductor.tick(Instant::now());
        if handle.model.load().players[0].transport == Transport::Playing {
            break;
        }
        assert!(Instant::now() < deadline, "the player never started");
        std::thread::sleep(Duration::from_millis(5));
    }
    drop(remote);
}
