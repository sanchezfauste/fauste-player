//! A control that serves a model held in memory and records commands.
#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};

use arc_swap::ArcSwap;
use fp_model::{AppState, Command, Config, TrackId};
use fp_remote::control::{Playback, RemoteControl, WaveformData};

pub struct FakeControl {
    pub state: ArcSwap<AppState>,
    pub playback: ArcSwap<Playback>,
    pub sent: Mutex<Vec<Command>>,
    /// False simulates a full command queue.
    pub accept: AtomicBool,
    pub covers: Mutex<HashMap<TrackId, Vec<u8>>>,
    pub peaks: Mutex<HashMap<TrackId, WaveformData>>,
    /// False holds every `cover` call until `open_covers`.
    cover_gate: Mutex<bool>,
    cover_open: Condvar,
    waiting: AtomicUsize,
}

impl FakeControl {
    pub fn new(state: AppState) -> Arc<Self> {
        Arc::new(Self {
            state: ArcSwap::from_pointee(state),
            playback: ArcSwap::from_pointee(Playback::default()),
            sent: Mutex::new(Vec::new()),
            accept: AtomicBool::new(true),
            covers: Mutex::new(HashMap::new()),
            peaks: Mutex::new(HashMap::new()),
            cover_gate: Mutex::new(true),
            cover_open: Condvar::new(),
            waiting: AtomicUsize::new(0),
        })
    }

    pub fn close_covers(&self) {
        *self.cover_gate.lock().unwrap() = false;
    }

    pub fn open_covers(&self) {
        *self.cover_gate.lock().unwrap() = true;
        self.cover_open.notify_all();
    }

    /// `cover` calls held by the gate now.
    pub fn covers_waiting(&self) -> usize {
        self.waiting.load(Ordering::SeqCst)
    }

    pub fn take_sent(&self) -> Vec<Command> {
        std::mem::take(&mut *self.sent.lock().unwrap())
    }

    pub fn edit(&self, f: impl FnOnce(&mut AppState)) {
        let mut s = (**self.state.load()).clone();
        f(&mut s);
        self.state.store(Arc::new(s));
    }
}

impl RemoteControl for FakeControl {
    fn model(&self) -> Arc<AppState> {
        self.state.load_full()
    }
    fn playback(&self) -> Playback {
        (**self.playback.load()).clone()
    }
    fn send(&self, command: Command) -> bool {
        if !self.accept.load(Ordering::SeqCst) {
            return false;
        }
        self.sent.lock().unwrap().push(command);
        true
    }
    fn cover(&self, track: TrackId) -> Option<Vec<u8>> {
        self.waiting.fetch_add(1, Ordering::SeqCst);
        let mut open = self.cover_gate.lock().unwrap();
        while !*open {
            open = self.cover_open.wait(open).unwrap();
        }
        drop(open);
        self.waiting.fetch_sub(1, Ordering::SeqCst);
        self.covers.lock().unwrap().get(&track).cloned()
    }
    fn peaks(&self, track: TrackId) -> Option<WaveformData> {
        self.peaks.lock().unwrap().get(&track).cloned()
    }
}

/// Four players over one playlist of three tracks (180 s each); a second,
/// empty playlist "Night"; one cart page with a cart holding the first track.
pub fn demo_state() -> AppState {
    let mut s = AppState::new(Config::default(), "Main");
    let main = s.playlists.first_id().unwrap();
    fp_model::apply(
        &mut s,
        Command::InsertPaths {
            playlist: main,
            index: 0,
            paths: ["a", "b", "c"]
                .map(|n| PathBuf::from(format!("/m/{n}.flac")))
                .to_vec(),
        },
    )
    .unwrap();
    fp_model::apply(
        &mut s,
        Command::CreatePlaylist {
            name: "Night".into(),
        },
    )
    .unwrap();
    let page = s.cartwall.pages.first().unwrap().id;
    fp_model::apply(
        &mut s,
        Command::AssignCartFile {
            page,
            index: 0,
            path: PathBuf::from("/m/a.flac"),
        },
    )
    .unwrap();
    let ids: Vec<TrackId> = s.library.iter().map(|t| t.id).collect();
    for id in ids {
        let t = s.library.get_mut(id).unwrap();
        t.duration_secs = 180.0;
        t.title = format!("Title {}", id.0);
        t.artist = "Artist".into();
    }
    s
}
