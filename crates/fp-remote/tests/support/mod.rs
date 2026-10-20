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
    give_cue_routes(&mut s);
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

/// A Cue output (headphones) for every player and the cartwall, with Main on
/// the default output: a CUE needs a Cue route (spec §4.6), and one off the
/// on-air output never asks first (on-air pre-listen spec CA3).
pub fn give_cue_routes(state: &mut AppState) {
    let phones = fp_model::Route {
        backend: "null".to_owned(),
        device: "phones".to_owned(),
        first_channel: 0,
    };
    state.config.outputs.routes = state
        .players
        .iter()
        .map(|p| fp_model::PlayerRoutes {
            player: p.id,
            main: None,
            cue: Some(phones.clone()),
        })
        .collect();
    state.config.outputs.cartwall.cue = Some(phones);
}

/// Places player 1's Main on a `dac` that runs bit-perfect (live settings
/// L3), which locks its fader (bit-perfect rate and fader spec BF1).
pub fn lock_first_player(s: &mut AppState) {
    let p = s.players[0].id;
    let dac = fp_model::OutputDevice {
        backend: "null".into(),
        device: "dac".into(),
    };
    let route = fp_model::Route {
        backend: "null".into(),
        device: "dac".into(),
        first_channel: 0,
    };
    if let Some(r) = s.config.outputs.routes.iter_mut().find(|r| r.player == p) {
        r.main = Some(route.clone());
    }
    s.config.outputs.bit_perfect = vec![dac.clone()];
    let running = fp_model::device_settings(&s.config.outputs, &dac);
    fp_model::on_event(
        s,
        fp_model::EngineEvent::Placed {
            holder: fp_model::Holder::PlayerMain(p),
            route: Some(route),
            device: dac,
            running,
        },
    );
    assert!(fp_model::fader_locked(s, p));
}

/// `demo_state`, with player 1's Cue, player 2's Main and the cartwall's
/// Cue placed on one `desk` output (on-air pre-listen spec §2) and player 2
/// playing: a pre-listen of player 1 or of a cart would be heard on air.
pub fn on_air_cue_state() -> AppState {
    let mut s = demo_state();
    let (p1, p2) = (s.players[0].id, s.players[1].id);
    let desk = fp_model::Route {
        backend: "null".into(),
        device: "desk".into(),
        first_channel: 0,
    };
    let device = fp_model::OutputDevice {
        backend: "null".into(),
        device: "desk".into(),
    };
    for r in &mut s.config.outputs.routes {
        if r.player == p1 {
            r.cue = Some(desk.clone());
        }
        if r.player == p2 {
            r.main = Some(desk.clone());
        }
    }
    s.config.outputs.cartwall.cue = Some(desk.clone());
    let running = fp_model::device_settings(&s.config.outputs, &device);
    for holder in [
        fp_model::Holder::PlayerCue(p1),
        fp_model::Holder::PlayerMain(p2),
        fp_model::Holder::CartwallCue,
    ] {
        fp_model::on_event(
            &mut s,
            fp_model::EngineEvent::Placed {
                holder,
                route: Some(desk.clone()),
                device: device.clone(),
                running,
            },
        );
    }
    fp_model::apply(&mut s, Command::Play(p2)).unwrap();
    s
}
