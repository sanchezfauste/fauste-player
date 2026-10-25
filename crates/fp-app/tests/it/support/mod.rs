//! A controller that applies commands to a model in memory, the way the
//! conductor does, and records them.
#![allow(dead_code, clippy::unwrap_used)]

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use arc_swap::ArcSwap;
use egui_kittest::Harness;
use fp_app::i18n::I18n;
use fp_app::services::MediaCache;
use fp_app::ui::app::AppUi;
use fp_app::ui::controller::Controller;
use fp_engine::conductor::Telemetry;
use fp_model::{AppState, Command, Config, EntryId, ModelError, PlayerId, Route};

pub struct Fake {
    pub state: ArcSwap<AppState>,
    pub telemetry: ArcSwap<Telemetry>,
    pub sent: Mutex<Vec<Command>>,
    pub rejected: Mutex<Vec<ModelError>>,
    pub tones: Mutex<Vec<(Route, f32)>>,
    pub meter_resets: Mutex<Vec<PlayerId>>,
    /// While set, commands are recorded but not applied: the snapshot lags
    /// behind, as it can for a frame in the app.
    pub hold: AtomicBool,
}

impl Fake {
    pub fn new(state: AppState) -> Arc<Self> {
        Arc::new(Self {
            state: ArcSwap::from_pointee(state),
            telemetry: ArcSwap::from_pointee(Telemetry::default()),
            sent: Mutex::new(Vec::new()),
            rejected: Mutex::new(Vec::new()),
            tones: Mutex::new(Vec::new()),
            meter_resets: Mutex::new(Vec::new()),
            hold: AtomicBool::new(false),
        })
    }

    pub fn take_sent(&self) -> Vec<Command> {
        std::mem::take(&mut *self.sent.lock().unwrap())
    }

    pub fn entries(&self) -> Vec<EntryId> {
        let s = self.state.load();
        s.playlists
            .iter()
            .next()
            .unwrap()
            .entries
            .iter()
            .map(|e| e.id)
            .collect()
    }

    pub fn player(&self, n: usize) -> PlayerId {
        self.state.load().players[n].id
    }
}

impl Controller for Fake {
    fn model(&self) -> Arc<AppState> {
        self.state.load_full()
    }
    fn telemetry(&self) -> Arc<Telemetry> {
        self.telemetry.load_full()
    }
    fn send(&self, command: Command) -> bool {
        self.sent.lock().unwrap().push(command.clone());
        if self.hold.load(Ordering::SeqCst) {
            return true;
        }
        let mut next = (**self.state.load()).clone();
        match fp_model::apply(&mut next, command) {
            Ok(_) => self.state.store(Arc::new(next)),
            Err(e) => self.rejected.lock().unwrap().push(e),
        }
        true
    }
    fn test_tone(&self, route: Route, frequency_hz: f32) -> bool {
        self.tones.lock().unwrap().push((route, frequency_hz));
        true
    }
    fn reset_meter_max(&self, player: PlayerId) -> bool {
        self.meter_resets.lock().unwrap().push(player);
        true
    }
    fn take_rejection(&self) -> Option<ModelError> {
        let mut r = self.rejected.lock().unwrap();
        (!r.is_empty()).then(|| r.remove(0))
    }
}

pub fn state(players: usize, tracks: usize) -> AppState {
    let mut config = Config::default();
    config.players.count = players;
    let mut state = AppState::new(config, "Main");
    give_cue_routes(&mut state);
    let playlist = state.playlists.first_id().unwrap();
    let paths = (1..=tracks)
        .map(|n| PathBuf::from(format!("/music/Song {n}.mp3")))
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
    state
}

pub fn harness(state: AppState) -> (Harness<'static, AppUi>, Arc<Fake>) {
    harness_from(state, |ui| ui)
}

/// As `harness`, with `configure` applied to the interface before its
/// first frame (as `main` does).
pub fn harness_with(
    state: AppState,
    configure: impl FnOnce(&mut AppUi),
) -> (Harness<'static, AppUi>, Arc<Fake>) {
    harness_from(state, |mut ui| {
        configure(&mut ui);
        ui
    })
}

/// As `harness`, with audio systems for Settings → Audio outputs.
pub fn harness_with_backends(
    state: AppState,
    backends: Vec<Arc<dyn fp_backends::AudioBackend>>,
) -> (Harness<'static, AppUi>, Arc<Fake>) {
    harness_from(state, |ui| ui.with_backends(backends))
}

pub fn harness_from(
    state: AppState,
    build: impl FnOnce(AppUi) -> AppUi,
) -> (Harness<'static, AppUi>, Arc<Fake>) {
    harness_sized(state, egui::vec2(1000.0, 700.0), build)
}

/// As `harness_from`, in a window of `size`.
pub fn harness_sized(
    state: AppState,
    size: egui::Vec2,
    build: impl FnOnce(AppUi) -> AppUi,
) -> (Harness<'static, AppUi>, Arc<Fake>) {
    harness_built(state, size, MediaCache::default(), build)
}

/// As `harness`, with `media` as the peaks and covers the interface draws.
pub fn harness_with_media(
    state: AppState,
    media: MediaCache,
) -> (Harness<'static, AppUi>, Arc<Fake>) {
    harness_built(state, egui::vec2(1000.0, 700.0), media, |ui| ui)
}

fn harness_built(
    state: AppState,
    size: egui::Vec2,
    media: MediaCache,
    build: impl FnOnce(AppUi) -> AppUi,
) -> (Harness<'static, AppUi>, Arc<Fake>) {
    let fake = Fake::new(state);
    let ui = build(AppUi::new(fake.clone(), I18n::new(Some("en-US")), media));
    // Short frames, so that two clicks fall within the double-click delay.
    let mut harness = Harness::builder()
        .with_size(size)
        .with_step_dt(0.02)
        .build_ui_state(
            |ui, app: &mut AppUi| {
                // As eframe: `logic`, then `ui`.
                app.guard_close(ui.ctx());
                app.ui(ui);
            },
            ui,
        );
    harness.run_steps(2);
    (harness, fake)
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

/// Places player `n`'s (0-based) Main on a `dac` that runs bit-perfect (live
/// settings L3): its fader locks and it starts every track hard
/// (bit-perfect rate and fader spec BR2, BF1).
pub fn lock_player(state: &mut AppState, n: usize) {
    let p = state.players[n].id;
    let route = Route {
        backend: "null".to_owned(),
        device: "dac".to_owned(),
        first_channel: 0,
    };
    let dac = fp_model::OutputDevice {
        backend: "null".to_owned(),
        device: "dac".to_owned(),
    };
    if let Some(r) = state
        .config
        .outputs
        .routes
        .iter_mut()
        .find(|r| r.player == p)
    {
        r.main = Some(route.clone());
    }
    if !state.config.outputs.bit_perfect.contains(&dac) {
        state.config.outputs.bit_perfect.push(dac.clone());
    }
    let running = fp_model::device_settings(&state.config.outputs, &dac);
    fp_model::on_event(
        state,
        fp_model::EngineEvent::Placed {
            holder: fp_model::Holder::PlayerMain(p),
            route: Some(route),
            device: dac,
            running,
        },
    );
}

/// Routes `holders` to one `desk` output (channels 1–2) and reports the
/// engine placing them there (live settings L3): a Cue among them is on
/// air next to a Main among them (on-air pre-listen spec §2).
pub fn on_desk(state: &mut AppState, holders: &[fp_model::Holder]) {
    use fp_model::Holder;
    let desk = Route {
        backend: "null".to_owned(),
        device: "desk".to_owned(),
        first_channel: 0,
    };
    let device = fp_model::OutputDevice {
        backend: "null".to_owned(),
        device: "desk".to_owned(),
    };
    for holder in holders {
        let outputs = &mut state.config.outputs;
        match *holder {
            Holder::PlayerMain(p) | Holder::PlayerCue(p) => {
                if let Some(r) = outputs.routes.iter_mut().find(|r| r.player == p) {
                    if matches!(holder, Holder::PlayerMain(_)) {
                        r.main = Some(desk.clone());
                    } else {
                        r.cue = Some(desk.clone());
                    }
                }
            }
            Holder::CartwallMain => outputs.cartwall.main = Some(desk.clone()),
            Holder::CartwallCue => outputs.cartwall.cue = Some(desk.clone()),
        }
    }
    let running = fp_model::device_settings(&state.config.outputs, &device);
    for holder in holders {
        fp_model::on_event(
            state,
            fp_model::EngineEvent::Placed {
                holder: *holder,
                route: Some(desk.clone()),
                device: device.clone(),
                running,
            },
        );
    }
}

/// A cell holding `activity`, for `AppUi::with_analysis_activity`.
pub fn activity_cell(
    activity: fp_app::activity::AnalysisActivity,
) -> fp_app::activity::ActivityCell {
    let cell = fp_app::activity::idle_cell();
    cell.store(Arc::new(activity));
    cell
}

/// The track of the `n`th entry (from 0) of the first playlist.
pub fn nth_track(state: &AppState, n: usize) -> fp_model::TrackId {
    state.playlists.iter().next().unwrap().entries[n].track
}

/// A platform that records what the interface asks for and answers with
/// canned values (mobile spec §4).
#[derive(Default)]
pub struct FakePlatform {
    pub audio_files: Vec<PathBuf>,
    pub focus_lost: AtomicBool,
    pub read_only: bool,
    pub asked: Mutex<Vec<String>>,
}

impl fp_app::platform::Platform for FakePlatform {
    fn pick_audio_files(&self, _filter_name: &str, _start: Option<PathBuf>) -> Vec<PathBuf> {
        self.asked.lock().unwrap().push("audio files".into());
        self.audio_files.clone()
    }

    fn pick_file(
        &self,
        _filter_name: &str,
        _extensions: &[&str],
        _start: Option<PathBuf>,
    ) -> Option<PathBuf> {
        self.asked.lock().unwrap().push("file".into());
        None
    }

    fn pick_folder(&self, _start: Option<PathBuf>) -> Option<PathBuf> {
        self.asked.lock().unwrap().push("folder".into());
        None
    }

    fn save_file(
        &self,
        _filter_name: &str,
        _extensions: &[&str],
        _file_name: String,
    ) -> Option<PathBuf> {
        self.asked.lock().unwrap().push("save".into());
        None
    }

    fn open_url(&self, url: &str) -> Result<(), String> {
        self.asked.lock().unwrap().push(format!("open {url}"));
        Ok(())
    }

    fn audio_focus_lost(&self) -> bool {
        self.focus_lost.load(Ordering::Relaxed)
    }

    fn can_write_user_files(&self) -> bool {
        !self.read_only
    }
}
