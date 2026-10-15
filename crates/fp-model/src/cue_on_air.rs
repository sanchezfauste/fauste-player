//! Pre-listen on an on-air output (on-air pre-listen spec, 2026-10-15):
//! where a holder plays, whether a Cue holder is on air and what sounds
//! there (§2), the confirmation request (CA3) and the notice (CA5). Pure
//! functions of `AppState`; the outputs are the ones the engine reported
//! (`live.placement`, `live.routes`).

use crate::command::CueStart;
use crate::config::OutputDevice;
use crate::live::{Holder, causes, configured_route, holders};
use crate::player::Transport;
use crate::state::AppState;

/// Where a holder plays: the device and the first channel of its pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderOutput {
    pub device: OutputDevice,
    pub first_channel: u16,
}

/// CA3: a pre-listen that did not start because its on-air output sounds.
/// Runtime only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CueConfirmation {
    /// Numbers the requests and notices, so an answer to an older request
    /// is ignored and the interface tells a new notice from a shown one.
    pub id: u64,
    pub holder: Holder,
    /// What was asked: *Pre-listen anyway* sends it as `Command::ForceCue`.
    pub start: CueStart,
    /// The holders that sound on that output, in display order.
    pub sounding: Vec<Holder>,
}

/// CA5: an unconfirmed pre-listen on `cue` stopped because `by` started
/// on its output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CueStopped {
    pub id: u64,
    pub cue: Holder,
    pub by: Holder,
}

/// The runtime state of CA3–CA5 (`AppState::cue_on_air`); never saved.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CueOnAir {
    /// The pending confirmation request, if any.
    pub request: Option<CueConfirmation>,
    /// The last pre-listen CA5 stopped.
    pub notice: Option<CueStopped>,
    /// Cue holders whose running pre-listen was forced (CA4).
    pub confirmed: Vec<Holder>,
}

/// §2: the device the engine placed `holder` on and the first channel of
/// the route it holds there (the configured one when the engine has not
/// reported one). A route to another device (a missing backend plays on
/// the default output) and a missing route play on channels 1–2. `None`
/// while `holder` is not placed.
pub fn holder_output(state: &AppState, holder: Holder) -> Option<HolderOutput> {
    let device = state.live.placement.get(&holder)?.clone();
    let route = match state.live.routes.get(&holder) {
        Some(route) => route.clone(),
        None => configured_route(&state.config, holder),
    };
    let first_channel = route
        .filter(|r| r.backend == device.backend && r.device == device.device)
        .map_or(0, |r| r.first_channel);
    Some(HolderOutput {
        device,
        first_channel,
    })
}

/// §2: on the same device, with channel pairs that share a channel.
pub fn outputs_overlap(a: &HolderOutput, b: &HolderOutput) -> bool {
    a.device == b.device && a.first_channel.abs_diff(b.first_channel) < 2
}

/// A Main holder: what goes on air.
pub(crate) fn is_main(holder: Holder) -> bool {
    matches!(holder, Holder::PlayerMain(_) | Holder::CartwallMain)
}

/// The holders other than `holder` whose output overlaps its own, in
/// display order.
fn sharing(state: &AppState, holder: Holder) -> Vec<Holder> {
    let Some(own) = holder_output(state, holder) else {
        return Vec::new();
    };
    holders(state)
        .into_iter()
        .filter(|h| *h != holder)
        .filter(|h| holder_output(state, *h).is_some_and(|o| outputs_overlap(&o, &own)))
        .collect()
}

/// §2: `cue`'s output overlaps the output of a Main holder.
pub fn cue_is_on_air(state: &AppState, cue: Holder) -> bool {
    sharing(state, cue).into_iter().any(is_main)
}

/// `holder` carries audio now (`live::causes`): a playing or fading
/// player, a running CUE, a playing cart, the cart pre-listen.
pub(crate) fn audible(state: &AppState, holder: Holder) -> bool {
    !causes(state, holder).is_empty()
}

/// A player's Main paused with a current track (BR3.5).
fn paused_with_track(state: &AppState, holder: Holder) -> bool {
    match holder {
        Holder::PlayerMain(id) => state
            .player(id)
            .is_ok_and(|p| p.transport == Transport::Paused && p.current.is_some()),
        _ => false,
    }
}

/// §2, CA3: the holders that sound on `cue`'s on-air output, in display
/// order, `cue` left out. Empty when `cue` is not on air, its player is
/// gone, or nothing sounds there.
pub fn sounding_on(state: &AppState, cue: Holder) -> Vec<Holder> {
    if let Holder::PlayerCue(p) = cue
        && state.player(p).is_err()
    {
        return Vec::new();
    }
    let shared = sharing(state, cue);
    if !shared.iter().copied().any(is_main) {
        return Vec::new();
    }
    shared
        .into_iter()
        .filter(|h| audible(state, *h) || paused_with_track(state, *h))
        .collect()
}
