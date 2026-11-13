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
    /// The volume fader can move: false while it is locked at 100 % on a
    /// bit-perfect output (bit-perfect rate and fader spec BF3).
    pub fader: bool,
}

/// The actions `player` can take now; all off for an unknown player.
pub fn availability(state: &AppState, player: PlayerId) -> Availability {
    let Ok(p) = state.player(player) else {
        return Availability::default();
    };
    if state.is_schedule(player) {
        return log_availability(state, p);
    }
    let playing = p.transport == Transport::Playing;
    let paused = p.transport == Transport::Paused;
    let has_previous = p.history.iter().any(|e| {
        Some(*e) != p.current
            && state
                .playable_request(*e, crate::live::Holder::PlayerMain(p.id))
                .is_some()
    });
    Availability {
        play: (paused && p.current.is_some()) || (p.next.is_some() && !p.fading),
        pause: (playing && !p.fading) || paused,
        stop: p.current.is_some(),
        fade_stop: playing && !p.fade_stopping(),
        restart: p.current.is_some() && p.transport != Transport::Stopped && !p.fade_stopping(),
        previous: playing && !p.fading && has_previous,
        stop_after_current: p.mode == PlayMode::Continuous
            || crate::reducer::repeat_entry(state, p),
        // A running CUE can always be stopped; a new one needs a Cue route
        // (spec §4.6, on-air pre-listen spec CA1).
        cue: (p.next.is_some() && state.config.outputs.player_has_cue(player)) || p.cue.is_some(),
        fader: !crate::bit_perfect::fader_locked(state, player),
    }
}

/// False for a transport command that `availability` marks unavailable,
/// for a `SetVolume` that a locked fader refuses (BF3),
/// and for a CUE of an entry or a cart pre-listen without a Cue route
/// (spec §4.6, CA1); true for every other command. For an
/// unknown player every one of these is false, as `availability` is all
/// off: a caller that reports an unknown player checks it first.
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
        // BF3: a locked fader takes only exactly 100 % (a NaN is ignored).
        #[allow(clippy::float_cmp)]
        Command::SetVolume(id, v) => {
            state.player(*id).is_ok() && (a(id).fader || *v == 1.0 || v.is_nan())
        }
        Command::CueEntry(id, _) => state.config.outputs.player_has_cue(*id),
        // A running cart pre-listen can always be stopped.
        Command::CueCart(cart) => {
            state.cartwall.cue == Some(*cart) || state.config.outputs.cartwall_has_cue()
        }
        // LB32: a held preview can always be released; a new one needs the
        // route and a playable track on the bar.
        Command::PlayLibraryCue => {
            state.library_cue.transport == crate::player::Transport::Paused
                || (state.config.outputs.library_has_cue()
                    && state
                        .library_cue
                        .track
                        .is_some_and(|t| state.library.is_playable(t)))
        }
        _ => true,
    }
}

/// The Log's buttons (automation rework AR3–AR6): they act on what is on
/// air for the Log, its row or the music fill's song; Previous never.
fn log_availability(state: &AppState, p: &crate::player::PlayerState) -> Availability {
    let Some(a) = state.automation.as_ref() else {
        return Availability::default();
    };
    let fill = crate::automation::fill_is_current(state);
    let row = p.transport != Transport::Stopped;
    let playing = p.transport == Transport::Playing || fill;
    let paused = p.transport == Transport::Paused || a.paused_fill;
    Availability {
        play: paused || a.next.is_some() || (row && p.next.is_some()),
        pause: playing,
        stop: row || fill || a.paused_fill,
        fade_stop: playing,
        restart: row && !p.fade_stopping(),
        previous: false,
        stop_after_current: a.running
            && (p.mode == PlayMode::Continuous || crate::reducer::repeat_entry(state, p)),
        cue: (p.next.is_some() && state.config.outputs.player_has_cue(p.id)) || p.cue.is_some(),
        fader: !crate::bit_perfect::fader_locked(state, p.id),
    }
}
