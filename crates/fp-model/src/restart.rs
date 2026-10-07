//! Settings that apply only when the application starts (feedback 2 spec
//! O4). The audio engine, its devices, the resource limits and the engine
//! tuning are built once at start-up; every other setting is read while
//! running.

use std::collections::{BTreeMap, HashSet};

use crate::config::{Config, OutputDevice, OutputsConfig, Route};
use crate::ids::PlayerId;

/// Why a restart is needed, in the order the notice lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RestartReason {
    AudioSystem,
    /// The global sample rate, or a device's own (operator feedback 4, Q12.6).
    SampleRate,
    /// The global buffer size, or a device's own.
    BufferSize,
    /// A player's or the cartwall's Main or Cue output.
    Routes,
    BitPerfect,
    /// A device's DSD mode, the DSD mix or the DSD silence time.
    DsdOutput,
    Limits,
    Tuning,
}

type Pair<'a> = (Option<&'a Route>, Option<&'a Route>);

/// Each player's routes by player, without empty entries.
fn player_routes(outputs: &OutputsConfig) -> BTreeMap<PlayerId, Pair<'_>> {
    outputs
        .routes
        .iter()
        .filter(|r| r.main.is_some() || r.cue.is_some())
        .map(|r| (r.player, (r.main.as_ref(), r.cue.as_ref())))
        .collect()
}

/// Whether a device opens differently under `a` and under `b` for one
/// setting (`effective`: the rate or buffer the engine opens it with). Only
/// devices routed in both are compared: a device routed in only one is a
/// change of routes, and an own value on an unrouted device applies to
/// nothing. An own value equal to the global one changes nothing either.
fn device_values_differ(
    a: &OutputsConfig,
    b: &OutputsConfig,
    effective: fn(&OutputsConfig, &OutputDevice) -> u32,
) -> bool {
    let in_b = b.routed_devices();
    a.routed_devices()
        .iter()
        .filter(|d| in_b.contains(d))
        .any(|d| effective(a, d) != effective(b, d))
}

/// What changed between the configuration the application started with
/// and the current one that only a restart applies.
pub fn restart_pending(started: &Config, current: &Config) -> Vec<RestartReason> {
    let (a, b) = (&started.outputs, &current.outputs);
    let mut reasons = Vec::new();
    if a.backend != b.backend {
        reasons.push(RestartReason::AudioSystem);
    }
    if a.sample_rate != b.sample_rate || device_values_differ(a, b, OutputsConfig::effective_rate) {
        reasons.push(RestartReason::SampleRate);
    }
    if a.buffer_frames != b.buffer_frames
        || device_values_differ(a, b, OutputsConfig::effective_buffer)
    {
        reasons.push(RestartReason::BufferSize);
    }
    if player_routes(a) != player_routes(b) || a.cartwall != b.cartwall {
        reasons.push(RestartReason::Routes);
    }
    let set = |o: &OutputsConfig| o.bit_perfect.iter().cloned().collect::<HashSet<_>>();
    if set(a) != set(b) {
        reasons.push(RestartReason::BitPerfect);
    }
    let dsd = |o: &OutputsConfig| {
        (
            o.dsd_output.iter().cloned().collect::<HashSet<_>>(),
            o.dsd_mix,
            o.dsd_silence_ms.to_bits(),
        )
    };
    if dsd(a) != dsd(b) {
        reasons.push(RestartReason::DsdOutput);
    }
    if started.limits != current.limits {
        reasons.push(RestartReason::Limits);
    }
    if started.tuning != current.tuning {
        reasons.push(RestartReason::Tuning);
    }
    reasons
}
