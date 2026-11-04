//! The library CUE (library plan 4, LB32-LB37): the state of the pre-listen
//! the library window runs on its own holder. Runtime only, never saved
//! (rule 10: nothing goes on air by itself after a start).

use crate::command::{CueStart, EngineAction, LibraryCueRequest};
use crate::ids::TrackId;
use crate::live::Holder;
use crate::player::Transport;
use crate::state::AppState;

/// The library window's pre-listen.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LibraryCue {
    /// The bar's track (LB34).
    pub track: Option<TrackId>,
    /// Stopped, Playing or Paused; a held preview carries audio (Q4).
    pub transport: Transport,
    /// Where Play starts while stopped, in seconds.
    pub start_at: Option<f64>,
}

/// LB36: the engine reports that the preview of `track` ended (the end of
/// its file, a file that cannot play, its output gone): the bar stops and
/// keeps its track. Ignored unless `track` is still the bar's.
pub(crate) fn ended(state: &mut AppState, track: TrackId) {
    let cue = &mut state.library_cue;
    if cue.track == Some(track) {
        cue.transport = Transport::Stopped;
        cue.start_at = None;
    }
}

/// What the engine needs to pre-listen `track` from `at` (LB33): `None`
/// for an unknown track or one whose file cannot play. A missing or
/// non-finite `at` is the start of the play range (the cue-in); a finite
/// one is clamped to the file when its duration is known. The gain is the
/// one the library CUE gets (LB36).
pub fn request(state: &AppState, track: TrackId, at: Option<f64>) -> Option<LibraryCueRequest> {
    let t = state.library.get(track)?;
    if !t.file_state.is_playable() {
        return None;
    }
    let from_secs = match at.filter(|s| s.is_finite()) {
        Some(secs) => secs.clamp(0.0, known_end(t.duration_secs)),
        None => t.play_range(state.config.players.use_cue_markers).cue_in,
    };
    Some(LibraryCueRequest {
        track,
        path: t.path.clone(),
        from_secs,
        format: t.format,
        normalization: crate::loudness::normalization_for(state, Holder::LibraryCue, t),
    })
}

fn known_end(duration_secs: f64) -> f64 {
    if duration_secs > 0.0 {
        duration_secs
    } else {
        f64::INFINITY
    }
}

/// LB34: the library table's selected row changed. A running preview
/// switches to the new track from its cue-in, held when it was held; one
/// that cannot play stops. A stopped preview only follows the bar. `None`
/// leaves a running preview alone.
pub(crate) fn select(state: &mut AppState, track: Option<TrackId>, out: &mut Vec<EngineAction>) {
    let running = state.library_cue.transport != Transport::Stopped;
    let Some(track) = track else {
        if !running {
            state.library_cue.track = None;
            state.library_cue.start_at = None;
        }
        return;
    };
    if state.library_cue.track == Some(track) {
        return;
    }
    state.library_cue.track = Some(track);
    state.library_cue.start_at = None;
    if !running {
        return;
    }
    match request(state, track, None) {
        Some(request) => out.push(EngineAction::StartLibraryCue {
            request,
            paused: state.library_cue.transport == Transport::Paused,
        }),
        None => stop(state, out),
    }
}

/// LB33: plays the bar's track, or releases a held preview. `force` is the
/// answer to a confirmation (CA4); a fresh start on a sounding on-air
/// output asks first otherwise (CA3).
pub(crate) fn play(state: &mut AppState, force: bool, out: &mut Vec<EngineAction>) {
    match state.library_cue.transport {
        Transport::Playing => {}
        Transport::Paused => set_paused(state, false, out),
        Transport::Stopped => {
            if !state.config.outputs.library_has_cue() {
                return;
            }
            let Some(track) = state.library_cue.track else {
                return;
            };
            let Some(request) = request(state, track, state.library_cue.start_at) else {
                return;
            };
            if !force && crate::cue_on_air::ask(state, CueStart::Library(track)) {
                return;
            }
            state.library_cue.start_at = None;
            state.library_cue.transport = Transport::Playing;
            out.push(EngineAction::StartLibraryCue {
                request,
                paused: false,
            });
        }
    }
}

/// LB33: holds or releases the running preview; only a change is sent.
pub(crate) fn set_paused(state: &mut AppState, paused: bool, out: &mut Vec<EngineAction>) {
    let to = match (state.library_cue.transport, paused) {
        (Transport::Playing, true) => Transport::Paused,
        (Transport::Paused, false) => Transport::Playing,
        _ => return,
    };
    state.library_cue.transport = to;
    out.push(EngineAction::SetLibraryCuePaused { paused });
}

/// LB33: ends the preview; nothing is sent when it is stopped.
pub(crate) fn stop(state: &mut AppState, out: &mut Vec<EngineAction>) {
    let cue = &mut state.library_cue;
    cue.start_at = None;
    if cue.transport != Transport::Stopped {
        cue.transport = Transport::Stopped;
        out.push(EngineAction::StopLibraryCue);
    }
}

/// LB33: moves a running preview, or sets where Play starts while stopped.
/// The time is clamped to the file; a non-finite one does nothing.
pub(crate) fn seek(state: &mut AppState, secs: f64, out: &mut Vec<EngineAction>) {
    if !secs.is_finite() {
        return;
    }
    let Some(track) = state.library_cue.track else {
        return;
    };
    if state.library_cue.transport == Transport::Stopped {
        if let Some(t) = state.library.get(track) {
            state.library_cue.start_at = Some(secs.clamp(0.0, known_end(t.duration_secs)));
        }
    } else if let Some(request) = request(state, track, Some(secs)) {
        out.push(EngineAction::SeekLibraryCue {
            secs: request.from_secs,
        });
    }
}

/// LB35: the preview lives only while its track is in the library; when it
/// left, the preview stops and the bar empties.
pub(crate) fn settle(state: &mut AppState, out: &mut Vec<EngineAction>) {
    if let Some(track) = state.library_cue.track
        && state.library.get(track).is_none()
    {
        stop(state, out);
        state.library_cue.track = None;
    }
}
