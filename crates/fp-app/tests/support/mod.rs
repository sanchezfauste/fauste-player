//! A controller that applies commands to a model in memory, the way the
//! conductor does, and records them.
#![allow(dead_code, clippy::unwrap_used)]

use std::path::PathBuf;
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
    let fake = Fake::new(state);
    let ui = build(AppUi::new(
        fake.clone(),
        I18n::new(Some("en-US")),
        MediaCache::default(),
    ));
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
