//! Bit-perfect rate and fader spec (2026-10-08): which players are
//! bit-perfect players, the rate a hard transition reopens their device
//! at, and their locked fader. Pure functions of `AppState`; the device
//! values are the ones the engine reported it **runs** with (`live`).

use crate::command::SourceRequest;
use crate::config::OutputDevice;
use crate::ids::PlayerId;
use crate::live::{Holder, PendingItem, causes, device_settings, holders, pending};
use crate::state::AppState;

/// Where the engine placed `player`'s Main; `None` before it is placed.
fn main_device(state: &AppState, player: PlayerId) -> Option<&OutputDevice> {
    state.live.placement.get(&Holder::PlayerMain(player))
}

/// BR2: `player`'s Main is placed on a device that runs bit-perfect. Its
/// Cue route does not count; a player not placed yet is not one.
pub fn bit_perfect_player(state: &AppState, player: PlayerId) -> bool {
    main_device(state, player)
        .and_then(|d| state.live.devices.get(d))
        .is_some_and(|running| running.bit_perfect)
}

/// BF1: the player's volume is held at exactly 1.0.
pub fn fader_locked(state: &AppState, player: PlayerId) -> bool {
    bit_perfect_player(state, player)
}

/// BR3: the rate `player`'s Main device reopens at during the hard
/// transition into `next`, or `None` when it keeps its rate (the next
/// track then plays at the running rate, resampled if it differs).
pub fn rate_cut(state: &AppState, player: PlayerId, next: &SourceRequest) -> Option<u32> {
    // 1. A bit-perfect player, on Main device `d`.
    if !bit_perfect_player(state, player) {
        return None;
    }
    let device = main_device(state, player)?;
    // 2. The stream rate of `d` is known.
    let stream = *state.live.stream_rates.get(device)?;
    // 3. A known PCM format at a rate other than the stream's.
    let format = next.format.filter(|f| f.dsd_rate.is_none())?;
    let rate = format.sample_rate;
    if rate == 0 || rate == stream {
        return None;
    }
    // 4. Not a rate the device refused.
    if state
        .live
        .refused_rates
        .get(device)
        .is_some_and(|refused| refused.contains(&rate))
    {
        return None;
    }
    // 5. Nothing else sounds on `d` (this player's own CUE counts).
    let others_quiet = holders(state)
        .into_iter()
        .filter(|h| *h != Holder::PlayerMain(player))
        .filter(|h| state.live.placement.get(h) == Some(device))
        .all(|h| causes(state, h).is_empty());
    if !others_quiet {
        return None;
    }
    // 6. The current entry does not go out as DSD (O25 ends that stream).
    if state.player(player).ok()?.dsd.is_some() {
        return None;
    }
    Some(rate)
}

/// BF8: the players whose fader Apply now (live settings L20) would lock
/// while it is below 100 %. Apply now forces every pending output item, so
/// afterwards each device runs its configured settings; a player's Main
/// is then on the `to` of its pending route, or where it is placed now. A
/// pending route to the default output is not resolved here: the engine
/// chooses that device (BF2 still resets the fader once it is placed).
pub fn faders_reset_by(state: &AppState) -> Vec<PlayerId> {
    let items = pending(state);
    state
        .players
        .iter()
        .filter(|p| p.volume < 1.0 && !fader_locked(state, p.id))
        .filter(|p| {
            let holder = Holder::PlayerMain(p.id);
            let moved = items.iter().find_map(|item| match &item.item {
                PendingItem::Route { holder: h, to, .. } if *h == holder => Some(to.clone()),
                _ => None,
            });
            let device = match moved {
                Some(Some(route)) => Some(OutputDevice {
                    backend: route.backend,
                    device: route.device,
                }),
                Some(None) => None,
                None => state.live.placement.get(&holder).cloned(),
            };
            device.is_some_and(|d| device_settings(&state.config.outputs, &d).bit_perfect)
        })
        .map(|p| p.id)
        .collect()
}
