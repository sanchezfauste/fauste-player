//! Pre-listen on an on-air output (on-air pre-listen spec, 2026-10-15):
//! where a holder plays, whether a Cue holder is on air and what sounds
//! there (§2), the confirmation request (CA3) and the notice (CA5). Pure
//! functions of `AppState`; the outputs are the ones the engine reported
//! (`live.placement`, `live.routes`).

use crate::command::{Command, CueStart, EngineAction};
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
    /// Numbers the requests and notices.
    seq: u64,
}

impl CueOnAir {
    /// The id of the next request or notice.
    pub(crate) fn next_id(&mut self) -> u64 {
        self.seq += 1;
        self.seq
    }
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

/// CA6: the pre-listen `command` would start from nothing: the next of a
/// player without a running CUE (`ToggleCue`, `SetCue(_, true)`), an entry
/// (`CueEntry`), a cart not pre-listened (`CueCart`,
/// `SetCartCue(_, true)`). `None` for anything else, `ForceCue` included.
pub fn cue_start(state: &AppState, command: &Command) -> Option<CueStart> {
    match command {
        Command::ToggleCue(p) | Command::SetCue(p, true) => {
            let player = state.player(*p).ok()?;
            if player.cue.is_some() {
                return None;
            }
            player
                .next
                .map(|entry| CueStart::Entry { player: *p, entry })
        }
        Command::CueEntry(p, entry) => Some(CueStart::Entry {
            player: *p,
            entry: *entry,
        }),
        Command::CueCart(cart) | Command::SetCartCue(cart, true) => {
            (state.cartwall.cue != Some(*cart)).then_some(CueStart::Cart(*cart))
        }
        _ => None,
    }
}

/// `start` begins a pre-listen from nothing: its holder has no running
/// one, a Cue route, and something it can play.
fn starts_fresh(state: &AppState, start: CueStart) -> bool {
    match start {
        CueStart::Entry { player, entry } => {
            state.player(player).is_ok_and(|p| p.cue.is_none())
                && state.config.outputs.player_has_cue(player)
                && state
                    .request_from_cue_in(entry, Holder::PlayerCue(player))
                    .is_some()
        }
        CueStart::Cart(cart) => {
            state.cartwall.cue.is_none()
                && state.config.outputs.cartwall_has_cue()
                && crate::cart_rules::cue_request(state, cart).is_some()
        }
    }
}

/// CA6: `apply(command)` would record a confirmation request instead of
/// starting a pre-listen (its on-air output sounds). The remote API and
/// MIDI cannot answer one, so they refuse such a command.
pub fn needs_confirmation(state: &AppState, command: &Command) -> bool {
    cue_start(state, command).is_some_and(|start| {
        starts_fresh(state, start) && !sounding_on(state, start.holder()).is_empty()
    })
}

/// CA3: records the request for `start`, replacing any other, and returns
/// true when its on-air output sounds; false (nothing recorded) otherwise.
pub(crate) fn ask(state: &mut AppState, start: CueStart) -> bool {
    let holder = start.holder();
    let sounding = sounding_on(state, holder);
    if sounding.is_empty() {
        return false;
    }
    let id = state.cue_on_air.next_id();
    state.cue_on_air.request = Some(CueConfirmation {
        id,
        holder,
        start,
        sounding,
    });
    true
}

/// CA4: a forced pre-listen on `holder` answers its request and, once it
/// runs, is confirmed.
pub(crate) fn forced(state: &mut AppState, holder: Holder) {
    let on_air = &mut state.cue_on_air;
    if on_air.request.as_ref().is_some_and(|r| r.holder == holder) {
        on_air.request = None;
    }
    if audible(state, holder) && !state.cue_on_air.confirmed.contains(&holder) {
        state.cue_on_air.confirmed.push(holder);
    }
}

/// CA3: the operator cancelled request `id`.
pub(crate) fn cancel(state: &mut AppState, id: u64) {
    if state
        .cue_on_air
        .request
        .as_ref()
        .is_some_and(|r| r.id == id)
    {
        state.cue_on_air.request = None;
    }
}

/// What CA5 compares a step against: each running pre-listen that is not
/// confirmed, with the Main holders audible on its output before the step.
pub(crate) struct Before(Vec<(Holder, Vec<Holder>)>);

/// The Main holders other than `cue` audible on `cue`'s output.
fn audible_mains(state: &AppState, cue: Holder) -> Vec<Holder> {
    sharing(state, cue)
        .into_iter()
        .filter(|h| is_main(*h) && audible(state, *h))
        .collect()
}

/// Opens every `apply` and `on_event` (CA5).
pub(crate) fn before_step(state: &AppState) -> Before {
    Before(
        holders(state)
            .into_iter()
            .filter(|h| matches!(h, Holder::PlayerCue(_) | Holder::CartwallCue))
            .filter(|h| audible(state, *h) && !state.cue_on_air.confirmed.contains(h))
            .map(|h| (h, audible_mains(state, h)))
            .collect(),
    )
}

/// Stops the pre-listen on `cue`.
fn stop(state: &mut AppState, cue: Holder, out: &mut Vec<EngineAction>) {
    match cue {
        Holder::PlayerCue(player) => {
            if let Ok(i) = state.player_index(player)
                && state.players[i].cue.take().is_some()
            {
                out.push(EngineAction::StopCue { player });
            }
        }
        Holder::CartwallCue => {
            if state.cartwall.cue.take().is_some() {
                out.push(EngineAction::StopCartCue);
            }
        }
        Holder::PlayerMain(_) | Holder::CartwallMain => {}
    }
}

/// Closes every `apply` and `on_event`. CA5: a pre-listen that was running
/// unconfirmed before the step stops when a Main holder became audible on
/// its output during it; the stop goes first in `out`, and a notice names
/// that holder. Then the confirmed list keeps only running pre-listens, and
/// the request goes once its output no longer sounds (or its holder already
/// runs a pre-listen, or what it asks for can no longer start) or else
/// names what sounds now (CA3).
pub(crate) fn after_step(state: &mut AppState, before: &Before, out: &mut Vec<EngineAction>) {
    let mut stops = Vec::new();
    for (cue, was) in &before.0 {
        if !audible(state, *cue) || state.cue_on_air.confirmed.contains(cue) {
            continue;
        }
        let Some(by) = audible_mains(state, *cue)
            .into_iter()
            .find(|h| !was.contains(h))
        else {
            continue;
        };
        stop(state, *cue, &mut stops);
        let id = state.cue_on_air.next_id();
        state.cue_on_air.notice = Some(CueStopped { id, cue: *cue, by });
    }
    let confirmed: Vec<Holder> = state
        .cue_on_air
        .confirmed
        .iter()
        .copied()
        .filter(|h| audible(state, *h))
        .collect();
    state.cue_on_air.confirmed = confirmed;
    if let Some((holder, start)) = state
        .cue_on_air
        .request
        .as_ref()
        .map(|r| (r.holder, r.start))
    {
        // What it asks for can no longer start (its entry or cart went, or
        // its holder lost the Cue output): nothing is left to confirm.
        let sounding = if audible(state, holder) || !starts_fresh(state, start) {
            Vec::new()
        } else {
            sounding_on(state, holder)
        };
        match state.cue_on_air.request.as_mut() {
            Some(request) if !sounding.is_empty() => request.sounding = sounding,
            _ => state.cue_on_air.request = None,
        }
    }
    if !stops.is_empty() {
        out.splice(0..0, stops);
        // A stopped pre-listen may free an output change (live settings L8).
        crate::live::dispatch_due(state, out);
    }
}
