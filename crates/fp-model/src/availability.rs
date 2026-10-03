//! Which transport actions make sense now (feedback spec R28). The UI dims
//! the others, and shortcuts and MIDI ignore them; `apply` keeps its own
//! guards, so this is presentation, not a second source of rules.

use crate::command::Command;
use crate::ids::PlayerId;
use crate::player::{PlayMode, Transport};
use crate::state::AppState;

/// One flag per transport action of a player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Availability {
    pub play: bool,
    pub pause: bool,
    pub stop: bool,
    pub fade_stop: bool,
    pub restart: bool,
    pub previous: bool,
    /// Continuous mode, or Single mode while the current entry repeats (O38).
    pub stop_after_current: bool,
    pub cue: bool,
}

/// The actions `player` can take now; all off for an unknown player.
pub fn availability(state: &AppState, player: PlayerId) -> Availability {
    let Ok(p) = state.player(player) else {
        return Availability::default();
    };
    let playing = p.transport == Transport::Playing;
    let paused = p.transport == Transport::Paused;
    let has_previous = p
        .history
        .iter()
        .any(|e| Some(*e) != p.current && state.playable_request(*e).is_some());
    Availability {
        play: (paused && p.current.is_some()) || (p.next.is_some() && !p.fading),
        pause: (playing && !p.fading) || paused,
        stop: p.current.is_some(),
        fade_stop: playing && !p.fade_stopping(),
        restart: p.current.is_some() && p.transport != Transport::Stopped && !p.fade_stopping(),
        previous: playing && !p.fading && has_previous,
        stop_after_current: p.mode == PlayMode::Continuous
            || crate::reducer::repeat_entry(state, p),
        // A running CUE can always be stopped; a new one needs a Cue output
        // that is not the Main output (spec §4.6).
        cue: (p.next.is_some() && state.config.outputs.player_has_cue(player)) || p.cue.is_some(),
    }
}

/// False for a transport command that `availability` marks unavailable,
/// and for a CUE of an entry or a cart pre-listen that has no Cue output
/// apart from Main (spec §4.6); true for every other command.
pub fn command_available(state: &AppState, command: &Command) -> bool {
    let a = |id: &PlayerId| availability(state, *id);
    match command {
        Command::Play(id) => a(id).play,
        Command::Pause(id) => a(id).pause,
        Command::Stop(id) => a(id).stop,
        Command::FadeStop(id) => a(id).fade_stop,
        Command::Restart(id) => a(id).restart,
        Command::Previous(id) => a(id).previous,
        Command::ToggleStopAfterCurrent(id) => a(id).stop_after_current,
        Command::ToggleCue(id) => a(id).cue,
        Command::CueEntry(id, _) => state.config.outputs.player_has_cue(*id),
        // A running cart pre-listen can always be stopped.
        Command::CueCart(cart) => {
            state.cartwall.cue == Some(*cart) || state.config.outputs.cartwall_has_cue()
        }
        _ => true,
    }
}
