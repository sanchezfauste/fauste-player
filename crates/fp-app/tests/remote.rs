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

#[derive(Clone, Default)]
struct Lines(Arc<std::sync::Mutex<Vec<u8>>>);

impl Write for Lines {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn request(addr: SocketAddr, method: &str, path: &str) -> (u16, String) {
    let mut s = TcpStream::connect_timeout(&addr, Duration::from_secs(2)).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(30))).unwrap();
    write!(
        s,
        "{method} {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\nContent-Length: 0\r\n\r\n"
    )
    .unwrap();
    let mut out = String::new();
    s.read_to_string(&mut out).unwrap();
    let status = out.split(' ').nth(1).unwrap().parse().unwrap();
    (status, out.split_once("\r\n\r\n").unwrap().1.to_owned())
}

fn post(addr: SocketAddr, path: &str) -> (u16, String) {
    request(addr, "POST", path)
}

fn get(addr: SocketAddr, path: &str) -> (u16, String) {
    request(addr, "GET", path)
}

/// A conductor on the Offline backend with the remote API listening on a
/// free loopback port. `prepare` changes the state before it starts.
struct Served {
    conductor: Conductor,
    handle: Arc<fp_engine::conductor::ConductorHandle>,
    remote: fp_remote::RemoteHandle,
    addr: SocketAddr,
    _dir: tempfile::TempDir,
}

fn serve(
    dir: tempfile::TempDir,
    files: Vec<std::path::PathBuf>,
    prepare: impl FnOnce(&mut fp_model::AppState),
) -> Served {
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
            paths: files,
        },
    )
    .unwrap();
    prepare(state);
    let limits = state.config.limits.clone();

    let backend = OfflineBackend::new();
    let _device = backend.add_device("main", 2);
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let engine = Engine::new(
        backends,
        EngineSettings::from_config(&loaded.state.config),
        file_opener(),
    );
    let (conductor, handle) = Conductor::new(loaded.state, loaded.actions, engine, Instant::now());
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
    Served {
        conductor,
        handle,
        remote,
        addr,
        _dir: dir,
    }
}

#[test]
fn a_remote_play_puts_the_player_on_air() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("tone.wav");
    tone(&file);
    let mut served = serve(dir, vec![file], |_| {});
    let player = served.handle.model.load().players[0].id;
    let (status, body) = post(served.addr, &format!("/api/v1/players/{}/play", player.0));
    assert_eq!(status, 202, "{body}");

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        served.conductor.tick(Instant::now());
        if served.handle.model.load().players[0].transport == Transport::Playing {
            break;
        }
        assert!(Instant::now() < deadline, "the player never started");
        std::thread::sleep(Duration::from_millis(5));
    }
    drop(served.remote);
}

/// A track an earlier version analysed has no cached analysis of this
/// version (older entries are swept), and it is not on a player, so no
/// waveform is in memory either: the peaks are worked out when asked for.
#[test]
fn the_peaks_of_an_outdated_track_are_available() {
    let dir = tempfile::tempdir().unwrap();
    let files: Vec<_> = ["a.wav", "b.wav"]
        .iter()
        .map(|n| {
            let f = dir.path().join(n);
            tone(&f);
            f
        })
        .collect();
    let served = serve(dir, files, |state| {
        for track in state.library.iter_mut() {
            track.analyzed = true;
            track.duration_secs = 3.0;
            track.format = None;
            track.analysis_version = 0;
        }
    });
    let model = served.handle.model.load();
    let playlist = model.playlists.first_id().unwrap();
    let second = model.playlists.get(playlist).unwrap().entries[1].track;
    let (status, body) = get(served.addr, &format!("/api/v1/tracks/{}/peaks", second.0));
    assert_eq!(status, 200, "{body}");
    assert!(body.contains("\"peaks\":[["), "{body}");
    // A second request answers the same.
    let (status, _) = get(served.addr, &format!("/api/v1/tracks/{}/peaks", second.0));
    assert_eq!(status, 200);
    let (status, body) = get(served.addr, &format!("/api/v1/tracks/{}/cover", second.0));
    assert_eq!(status, 404, "a track without a cover: {body}");
    drop(served.remote);
}

/// A file that cannot be decoded answers "not found" for its peaks (rule
/// 9: bad data degrades), and the request ends.
#[test]
fn the_peaks_of_an_undecodable_file_are_not_found() {
    let lines = Lines::default();
    let writer = lines.clone();
    tracing_subscriber::fmt()
        .with_writer(move || writer.clone())
        .with_ansi(false)
        .init();
    let dir = tempfile::tempdir().unwrap();
    let junk = dir.path().join("junk.wav");
    std::fs::write(&junk, b"this is not audio").unwrap();
    let served = serve(dir, vec![junk], |state| {
        for track in state.library.iter_mut() {
            track.analyzed = true;
            track.format = None;
        }
    });
    let model = served.handle.model.load();
    let track = model.library.iter().next().unwrap().id;
    let (status, body) = get(served.addr, &format!("/api/v1/tracks/{}/peaks", track.0));
    assert_eq!(status, 404, "{body}");
    let text = String::from_utf8(lines.0.lock().unwrap().clone()).unwrap();
    assert!(
        text.contains("cannot analyse the track") && text.contains("junk.wav"),
        "{text}"
    );
    drop(served.remote);
}

/// Several clients asking for the same outdated track at once all get the
/// peaks; the analyses are taken one at a time.
#[test]
fn two_requests_at_once_both_get_the_peaks() {
    let dir = tempfile::tempdir().unwrap();
    let files: Vec<_> = ["a.wav", "b.wav"]
        .iter()
        .map(|n| {
            let f = dir.path().join(n);
            tone(&f);
            f
        })
        .collect();
    let served = serve(dir, files, |state| {
        for track in state.library.iter_mut() {
            track.analyzed = true;
            track.duration_secs = 3.0;
            track.format = None;
            track.analysis_version = 0;
        }
    });
    let model = served.handle.model.load();
    let playlist = model.playlists.first_id().unwrap();
    let second = model.playlists.get(playlist).unwrap().entries[1].track;
    let (addr, uri) = (served.addr, format!("/api/v1/tracks/{}/peaks", second.0));
    let clients: Vec<_> = (0..2)
        .map(|_| {
            let uri = uri.clone();
            std::thread::spawn(move || get(addr, &uri).0)
        })
        .collect();
    for client in clients {
        assert_eq!(client.join().unwrap(), 200);
    }
    drop(served.remote);
}
