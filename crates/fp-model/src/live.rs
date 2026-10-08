//! Live settings (live settings spec, 2026-10-07): the output settings the
//! engine still runs with an older value, what keeps each change waiting,
//! and the engine work that applies a change once it is safe. Everything
//! here is a pure function of `AppState`; the engine reports what it runs
//! with through `EngineEvent`s, and `AppState::live` keeps it (runtime
//! only, never saved: L24).

use std::collections::BTreeMap;

use crate::config::{Config, OutputDevice, OutputsConfig, Route};
use crate::dsd::{DsdMix, DsdOutput};
use crate::ids::{CartId, PlayerId};
use crate::player::Transport;
use crate::state::AppState;

/// One audio path a route places on a device (spec §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Holder {
    PlayerMain(PlayerId),
    PlayerCue(PlayerId),
    CartwallMain,
    CartwallCue,
}

/// Everything a device's stream depends on (L4).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeviceSettings {
    pub sample_rate: u32,
    pub buffer_frames: u32,
    pub bit_perfect: bool,
    /// `Pcm` unless the device is bit-perfect.
    pub dsd: DsdOutput,
    /// Only on a device whose `dsd` is not `Pcm`: elsewhere it changes
    /// nothing, so it never makes a device pending.
    pub dsd_mix: Option<DsdMix>,
    pub dsd_silence_ms: Option<f64>,
}

/// What one engine action changes. The variant order is the order of L9.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Target {
    AudioSystem,
    Route(Holder),
    Device(OutputDevice),
}

/// The value an action asks for.
#[derive(Debug, Clone, PartialEq)]
pub enum Wanted {
    AudioSystem(Option<String>),
    Route(Option<Route>),
    Device(DeviceSettings),
}

/// The last value a target refused, and why (L14).
#[derive(Debug, Clone, PartialEq)]
pub struct Failure {
    pub wanted: Wanted,
    pub reason: String,
}

/// Why an item waits (spec §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusyCause {
    PlayerPlaying(PlayerId),
    PlayerFading(PlayerId),
    PlayerCue(PlayerId),
    CartPlaying(CartId),
    CartCue(CartId),
}

/// What the engine runs with, as it reported it (spec §4.1).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LiveSettings {
    /// The `outputs.backend` the engine runs with; `None` until the engine
    /// reports it (`AudioSystemInUse`), and then no audio system change is
    /// pending.
    pub audio_system: Option<Option<String>>,
    /// The backend the engine really chose (the status bar's label).
    pub audio_system_in_use: Option<String>,
    /// Where each holder with a bus plays.
    pub placement: BTreeMap<Holder, OutputDevice>,
    /// The route the engine holds for each holder it knows.
    pub routes: BTreeMap<Holder, Option<Route>>,
    /// The running settings of each device in use.
    pub devices: BTreeMap<OutputDevice, DeviceSettings>,
    /// Actions sent and not answered yet, with the value asked for.
    pub in_flight: BTreeMap<Target, Wanted>,
    /// The last refused value per target.
    pub failures: BTreeMap<Target, Failure>,
}

/// L4: the settings `device` opens with under `outputs`.
pub fn device_settings(outputs: &OutputsConfig, device: &OutputDevice) -> DeviceSettings {
    let dsd = outputs.dsd_output_for(&device.backend, &device.device);
    let carries_dsd = dsd != DsdOutput::Pcm;
    DeviceSettings {
        sample_rate: outputs.effective_rate(device),
        buffer_frames: outputs.effective_buffer(device),
        bit_perfect: outputs.bit_perfect.contains(device),
        dsd,
        dsd_mix: carries_dsd.then_some(outputs.dsd_mix),
        dsd_silence_ms: carries_dsd.then_some(outputs.dsd_silence_ms),
    }
}

/// Every holder, in display order: each player's Main then Cue, then the
/// cartwall's Main and Cue (L9).
pub fn holders(state: &AppState) -> Vec<Holder> {
    state
        .players
        .iter()
        .flat_map(|p| [Holder::PlayerMain(p.id), Holder::PlayerCue(p.id)])
        .chain([Holder::CartwallMain, Holder::CartwallCue])
        .collect()
}

/// The route `config` gives `holder`.
pub fn configured_route(config: &Config, holder: Holder) -> Option<Route> {
    let outputs = &config.outputs;
    match holder {
        Holder::PlayerMain(p) => outputs.player_routes(p).and_then(|r| r.main.clone()),
        Holder::PlayerCue(p) => outputs.player_routes(p).and_then(|r| r.cue.clone()),
        Holder::CartwallMain => outputs.cartwall.main.clone(),
        Holder::CartwallCue => outputs.cartwall.cue.clone(),
    }
}

/// L1: why `holder` carries audio now; empty when it does not. A paused
/// player, a stopped one with a track loaded and a preload carry nothing
/// (D2); a held CUE does (Q4).
pub fn causes(state: &AppState, holder: Holder) -> Vec<BusyCause> {
    match holder {
        Holder::PlayerMain(id) => state
            .player(id)
            .ok()
            .and_then(|p| {
                if p.fading {
                    Some(BusyCause::PlayerFading(id))
                } else if p.transport == Transport::Playing {
                    Some(BusyCause::PlayerPlaying(id))
                } else {
                    None
                }
            })
            .into_iter()
            .collect(),
        Holder::PlayerCue(id) => state
            .player(id)
            .ok()
            .filter(|p| p.cue.is_some())
            .map(|_| BusyCause::PlayerCue(id))
            .into_iter()
            .collect(),
        Holder::CartwallMain => state
            .cartwall
            .playing
            .iter()
            .map(|c| BusyCause::CartPlaying(c.cart))
            .collect(),
        Holder::CartwallCue => state
            .cartwall
            .cue
            .map(BusyCause::CartCue)
            .into_iter()
            .collect(),
    }
}

/// Appends the causes of `holders` to `out`, each once.
fn collect_causes(
    state: &AppState,
    holders: impl Iterator<Item = Holder>,
    out: &mut Vec<BusyCause>,
) {
    for holder in holders {
        for cause in causes(state, holder) {
            if !out.contains(&cause) {
                out.push(cause);
            }
        }
    }
}

/// L2: the causes of every holder placed on `device`. A device is idle
/// when this is empty.
pub fn device_causes(state: &AppState, device: &OutputDevice) -> Vec<BusyCause> {
    let mut out = Vec::new();
    let on_device = holders(state)
        .into_iter()
        .filter(|h| state.live.placement.get(h) == Some(device));
    collect_causes(state, on_device, &mut out);
    out
}
