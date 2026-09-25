//! The player behaviour rules of spec §3, as pure state transitions.

use std::path::PathBuf;

use crate::command::{Command, EngineAction, EngineEvent, SourceRequest};
use crate::error::ModelError;
use crate::ids::{EntryId, PlayerId, PlaylistId};
use crate::player::Transport;
use crate::playlist::PlaylistEntry;
use crate::state::AppState;
use crate::track::Track;

/// Applies a user command. On `Err` the state is unchanged.
pub fn apply(state: &mut AppState, command: Command) -> Result<Vec<EngineAction>, ModelError> {
    let mut out = Vec::new();
    match command {
        Command::Play(id) => play(state, id, &mut out)?,
        Command::Pause(id) => pause(state, id, &mut out)?,
        Command::Stop(id) => stop(state, id, &mut out)?,
        Command::FadeStop(id) => fade_stop(state, id, &mut out)?,
        Command::SetNext(id, entry) => set_next(state, id, entry)?,
        Command::InsertPaths {
            playlist,
            index,
            paths,
        } => insert_paths(state, playlist, index, paths)?,
    }
    reconcile(state, &mut out);
    Ok(out)
}

/// Applies an engine observation. Events about unknown players are ignored
/// (the player may have been removed while the event was in flight).
pub fn on_event(state: &mut AppState, event: EngineEvent) -> Vec<EngineAction> {
    let mut out = Vec::new();
    match event {
        EngineEvent::FadeCompleted { player } => {
            if let Ok(i) = state.player_index(player) {
                state.players[i].fading = false;
            }
        }
        EngineEvent::ReachedEnd { player } => {
            if let Ok(i) = state.player_index(player) {
                stop_player(state, i);
            }
        }
    }
    reconcile(state, &mut out);
    out
}

fn play(state: &mut AppState, id: PlayerId, out: &mut Vec<EngineAction>) -> Result<(), ModelError> {
    let i = state.player_index(id)?;
    match state.players[i].transport {
        Transport::Paused if state.players[i].current.is_some() => {
            state.players[i].transport = Transport::Playing;
            out.push(EngineAction::Resume { player: id });
        }
        Transport::Paused | Transport::Stopped => {
            if let Some(request) = advance(state, i) {
                out.push(EngineAction::StartCurrent {
                    player: id,
                    request,
                });
            }
        }
        Transport::Playing => {
            if state.players[i].fading {
                return Ok(());
            }
            let fade_ms = state.config.players.fade_ms;
            if let Some(request) = advance(state, i) {
                state.players[i].fading = true;
                out.push(EngineAction::Crossfade {
                    player: id,
                    request,
                    fade_ms,
                });
            }
        }
    }
    Ok(())
}

fn pause(
    state: &mut AppState,
    id: PlayerId,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let i = state.player_index(id)?;
    let player = &mut state.players[i];
    match player.transport {
        Transport::Playing if !player.fading => {
            player.transport = Transport::Paused;
            out.push(EngineAction::Pause { player: id });
        }
        Transport::Paused => {
            player.transport = Transport::Playing;
            out.push(EngineAction::Resume { player: id });
        }
        _ => {}
    }
    Ok(())
}

fn stop(state: &mut AppState, id: PlayerId, out: &mut Vec<EngineAction>) -> Result<(), ModelError> {
    let i = state.player_index(id)?;
    let player = &state.players[i];
    if player.transport == Transport::Stopped && player.current.is_none() {
        return Ok(());
    }
    stop_player(state, i);
    out.push(EngineAction::StopNow { player: id });
    Ok(())
}

fn fade_stop(
    state: &mut AppState,
    id: PlayerId,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let i = state.player_index(id)?;
    let fade_ms = state.config.players.fade_ms;
    let player = &mut state.players[i];
    if player.transport == Transport::Playing && !player.fading {
        player.fading = true;
        out.push(EngineAction::FadeOutAndStop {
            player: id,
            fade_ms,
        });
    }
    Ok(())
}

fn set_next(state: &mut AppState, id: PlayerId, entry: EntryId) -> Result<(), ModelError> {
    let i = state.player_index(id)?;
    if state.playlists.entry(entry).is_none() {
        return Err(ModelError::UnknownEntry(entry));
    }
    if state.players[i].current == Some(entry) {
        return Err(ModelError::NextIsCurrent);
    }
    state.players[i].next = Some(entry);
    Ok(())
}

fn insert_paths(
    state: &mut AppState,
    playlist: PlaylistId,
    index: usize,
    paths: Vec<PathBuf>,
) -> Result<(), ModelError> {
    if state.playlists.get(playlist).is_none() {
        return Err(ModelError::UnknownPlaylist(playlist));
    }
    let mut entries = Vec::with_capacity(paths.len());
    for path in paths {
        let track = state.ids.track();
        state.library.insert(Track::new(track, path));
        entries.push(PlaylistEntry {
            id: state.ids.entry(),
            track,
            played: false,
        });
    }
    state.playlists.insert(playlist, index, entries)?;
    fill_empty_next(state);
    Ok(())
}

/// Rule 12: the next entry becomes current. Returns the request for the new
/// current source, or `None` (state untouched) when there is nothing to play.
pub(crate) fn advance(state: &mut AppState, i: usize) -> Option<SourceRequest> {
    let next = state.players[i].next?;
    let request = state.request_from_cue_in(next)?;
    if let Some(current) = state.players[i].current {
        state.playlists.mark_played(current);
    }
    let following = state.playlists.next_playable_after(next, &state.library);
    let player = &mut state.players[i];
    player.current = Some(next);
    player.next = following;
    player.transport = Transport::Playing;
    // The preloaded source is now the current one, and the old plan no longer applies.
    player.preloaded = None;
    player.scheduled = None;
    Some(request)
}

/// Rule 7: stop semantics shared by Stop, fade stop, stop-after-current and Single mode.
pub(crate) fn stop_player(state: &mut AppState, i: usize) {
    if let Some(current) = state.players[i].current {
        state.playlists.mark_played(current);
        if state.players[i].next.is_none() {
            state.players[i].next = state.playlists.next_playable_after(current, &state.library);
        }
    }
    let player = &mut state.players[i];
    player.current = None;
    player.transport = Transport::Stopped;
    player.fading = false;
    player.stop_after_current = false;
}

/// An idle player (stopped, nothing current, no next) picks the first playable
/// entry of the playlist it shows. Used after entries appear or disappear.
pub(crate) fn fill_empty_next(state: &mut AppState) {
    let picks: Vec<(usize, Option<EntryId>)> = state
        .players
        .iter()
        .enumerate()
        .filter(|(_, p)| {
            p.transport == Transport::Stopped && p.current.is_none() && p.next.is_none()
        })
        .map(|(i, p)| {
            (
                i,
                state.playlists.first_playable(p.playlist, &state.library),
            )
        })
        .collect();
    for (i, pick) in picks {
        state.players[i].next = pick;
    }
}

/// Derives the engine work implied by the state: preload whatever is next.
pub(crate) fn reconcile(state: &mut AppState, out: &mut Vec<EngineAction>) {
    let preloads: Vec<(usize, Option<EntryId>)> = state
        .players
        .iter()
        .enumerate()
        .filter(|(_, p)| p.preloaded != p.next)
        .map(|(i, p)| (i, p.next))
        .collect();
    for (i, wanted) in preloads {
        let request = wanted.and_then(|e| state.request_from_cue_in(e));
        let player = &mut state.players[i];
        player.preloaded = wanted;
        out.push(EngineAction::Preload {
            player: player.id,
            request,
        });
    }
}
