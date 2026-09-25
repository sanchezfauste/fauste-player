//! The player behaviour rules of spec §3, as pure state transitions.

use std::collections::HashSet;
use std::path::PathBuf;

use crate::command::{
    Command, EngineAction, EngineEvent, SOURCE_END, SourceRequest, TransitionPlan,
};
use crate::config::Config;
use crate::error::ModelError;
use crate::ids::{EntryId, PlayerId, PlaylistId, TrackId};
use crate::player::{CueState, PlayMode, PlayerState, Transport};
use crate::playlist::{Playlist, PlaylistEntry};
use crate::state::AppState;
use crate::track::{FileState, Track};

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
        Command::SetMode(id, mode) => {
            let i = state.player_index(id)?;
            let player = &mut state.players[i];
            player.mode = mode;
            if mode == PlayMode::Single {
                player.stop_after_current = false;
            }
        }
        Command::ToggleStopAfterCurrent(id) => {
            let i = state.player_index(id)?;
            let player = &mut state.players[i];
            if player.mode == PlayMode::Single {
                return Err(ModelError::StopAfterInSingle);
            }
            player.stop_after_current = !player.stop_after_current;
        }
        Command::ToggleCue(id) => toggle_cue(state, id, &mut out)?,
        Command::CueEntry(id, entry) => cue_entry(state, id, entry, &mut out)?,
        Command::SetVolume(id, volume) => {
            let i = state.player_index(id)?;
            let volume = if volume.is_nan() {
                0.0
            } else {
                volume.clamp(0.0, 1.0)
            };
            state.players[i].volume = volume;
            out.push(EngineAction::SetVolume { player: id, volume });
        }
        Command::Seek(id, secs) => {
            let i = state.player_index(id)?;
            let player = &state.players[i];
            if player.transport != Transport::Stopped
                && let Some(request) = player.current.and_then(|c| state.request_at(c, secs))
            {
                out.push(EngineAction::Seek {
                    player: id,
                    secs: request.from_secs,
                });
            }
        }
        Command::ShowPlaylist(id, playlist) => {
            let i = state.player_index(id)?;
            if state.playlists.get(playlist).is_none() {
                return Err(ModelError::UnknownPlaylist(playlist));
            }
            state.players[i].playlist = playlist;
        }
        Command::SetColumnWidths(id, widths) => {
            let i = state.player_index(id)?;
            state.players[i].columns = widths;
        }
        Command::RemoveEntry(entry) => remove_entry(state, entry, &mut out)?,
        Command::MoveEntry { entry, to, index } => {
            state.playlists.move_entry(entry, to, index)?;
            refresh_next(state);
        }
        Command::DuplicateEntry(entry) => {
            if state.playlists.entry(entry).is_none() {
                return Err(ModelError::UnknownEntry(entry));
            }
            let new_id = state.ids.entry();
            state.playlists.duplicate(entry, new_id)?;
            refresh_next(state);
        }
        Command::CreatePlaylist { name } => {
            let id = state.ids.playlist();
            state.playlists.add(Playlist::new(id, name));
        }
        Command::RenamePlaylist { playlist, name } => state.playlists.rename(playlist, name)?,
        Command::DeletePlaylist(playlist) => delete_playlist(state, playlist, &mut out)?,
        Command::SetPlayerCount(count) => set_player_count(state, count, &mut out)?,
        Command::UpdateConfig(config) => {
            let mut config: Config = *config;
            config.players.count = state.config.players.count;
            state.config = config;
        }
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
        EngineEvent::ReachedEnd { player, entry } => {
            if let Ok(i) = state.player_index(player)
                && state.players[i].current == Some(entry)
            {
                stop_player(state, i);
            }
        }
        EngineEvent::TransitionStarted { player, entry } => {
            // A transition reported after the player was stopped is stale: the
            // engine has already been told to stop everything.
            if let Ok(i) = state.player_index(player)
                && state.players[i].current != Some(entry)
                && state.players[i].transport != Transport::Stopped
            {
                let overlapping = matches!(
                    state.players[i].scheduled,
                    Some(TransitionPlan::StartNextAt {
                        fade_current_until_secs: Some(_),
                        ..
                    })
                );
                if advance_to(state, i, entry).is_some() {
                    state.players[i].fading = overlapping;
                } else {
                    stop_player(state, i);
                    out.push(EngineAction::StopNow { player });
                }
            }
        }
        EngineEvent::SourceFailed { player, entry } => {
            source_failed(state, player, entry, &mut out)
        }
        EngineEvent::CueEnded { player } => {
            if let Ok(i) = state.player_index(player) {
                state.players[i].cue = None;
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
    // Allowed during a crossfade or segue overlap too: the engine fades every
    // Main source of the player. Only a fade stop already in progress blocks it.
    if player.transport == Transport::Playing && !player.fade_stop_pending {
        player.fading = true;
        player.fade_stop_pending = true;
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
    state.players[i].next_explicit = true;
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
    refresh_next(state);
    Ok(())
}

/// Spec §4.5: mark the file unreadable, then skip (Continuous, while on air)
/// or stop. A paused or restored player never starts playing by itself (§7).
fn source_failed(
    state: &mut AppState,
    player: PlayerId,
    entry: EntryId,
    out: &mut Vec<EngineAction>,
) {
    if let Some(t) = state
        .playlists
        .entry(entry)
        .map(|e| e.track)
        .and_then(|track| state.library.get_mut(track))
    {
        t.file_state = FileState::Unreadable;
    }
    if let Ok(i) = state.player_index(player)
        && state.players[i].current == Some(entry)
    {
        let p = &state.players[i];
        let keep_going = p.transport == Transport::Playing
            && p.mode == PlayMode::Continuous
            && !p.stop_after_current;
        let restarted = if keep_going { advance(state, i) } else { None };
        match restarted {
            Some(request) => out.push(EngineAction::StartCurrent { player, request }),
            None => {
                stop_player(state, i);
                out.push(EngineAction::StopNow { player });
            }
        }
    }
    let successors = successors_of(state, entry);
    for p in &mut state.players {
        if p.next == Some(entry) {
            p.next = successors.for_current(p.current);
            p.next_explicit = false;
        }
        if p.cue.is_some_and(|c| c.entry == entry) {
            p.cue = None;
            out.push(EngineAction::StopCue { player: p.id });
        }
    }
    refresh_next(state);
}

/// The two playable entries that follow `entry` in its playlist, so a
/// replacement can skip a player's own current entry (next != current).
struct Successors(Option<EntryId>, Option<EntryId>);

impl Successors {
    fn for_current(&self, current: Option<EntryId>) -> Option<EntryId> {
        if self.0.is_some() && self.0 == current {
            self.1
        } else {
            self.0
        }
    }
}

fn successors_of(state: &AppState, entry: EntryId) -> Successors {
    let first = state.playlists.next_playable_after(entry, &state.library);
    let second = first.and_then(|f| state.playlists.next_playable_after(f, &state.library));
    Successors(first, second)
}

/// Rules 9–11: what the engine must do when the current track ends. Returns
/// `None` while a fade stop is running: the engine stops and reports the end.
pub fn plan_for(state: &AppState, player: &PlayerState) -> Option<TransitionPlan> {
    if player.transport == Transport::Stopped || player.fade_stop_pending {
        return None;
    }
    let track = state.track_for_entry(player.current?)?;
    let end = track.known_cue_out_secs().unwrap_or(SOURCE_END);
    if player.mode == PlayMode::Single || player.stop_after_current || player.next.is_none() {
        return Some(TransitionPlan::StopAt { at_secs: end });
    }
    match track.segue_start_secs() {
        Some(segue) if state.config.players.auto_segue && end.is_finite() => {
            Some(TransitionPlan::StartNextAt {
                at_secs: segue,
                fade_current_until_secs: Some(end),
            })
        }
        _ => Some(TransitionPlan::StartNextAt {
            at_secs: end,
            fade_current_until_secs: None,
        }),
    }
}

fn toggle_cue(
    state: &mut AppState,
    id: PlayerId,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let i = state.player_index(id)?;
    if state.players[i].cue.take().is_some() {
        out.push(EngineAction::StopCue { player: id });
    } else if let Some(entry) = state.players[i].next {
        cue_entry(state, id, entry, out)?;
    }
    Ok(())
}

fn cue_entry(
    state: &mut AppState,
    id: PlayerId,
    entry: EntryId,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let i = state.player_index(id)?;
    let request = state
        .request_from_cue_in(entry)
        .ok_or(ModelError::UnknownEntry(entry))?;
    state.players[i].cue = Some(CueState { entry });
    out.push(EngineAction::StartCue {
        player: id,
        request,
    });
    Ok(())
}

/// Stops cues that point at any of `entries`, and replaces next pointers into
/// them with `replacement(entry, player's current)`.
fn detach_entries(
    state: &mut AppState,
    entries: &HashSet<EntryId>,
    replacement: impl Fn(EntryId, Option<EntryId>) -> Option<EntryId>,
    out: &mut Vec<EngineAction>,
) {
    for p in &mut state.players {
        if let Some(next) = p.next.filter(|n| entries.contains(n)) {
            p.next = replacement(next, p.current);
            p.next_explicit = false;
        }
        if p.cue.is_some_and(|c| entries.contains(&c.entry)) {
            p.cue = None;
            out.push(EngineAction::StopCue { player: p.id });
        }
    }
}

/// Drops library tracks that no playlist entry references any more.
fn forget_unreferenced(state: &mut AppState, tracks: impl IntoIterator<Item = TrackId>) {
    for track in tracks {
        if !state.playlists.references_track(track) {
            state.library.remove(track);
        }
    }
}

fn remove_entry(
    state: &mut AppState,
    entry: EntryId,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    if state.is_on_air(entry) {
        return Err(ModelError::EntryOnAir(entry));
    }
    let successors = successors_of(state, entry);
    let removed = state.playlists.remove_entry(entry)?;
    detach_entries(
        state,
        &HashSet::from([entry]),
        |_, current| successors.for_current(current),
        out,
    );
    forget_unreferenced(state, [removed.track]);
    refresh_next(state);
    Ok(())
}

fn delete_playlist(
    state: &mut AppState,
    playlist: PlaylistId,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let list = state
        .playlists
        .get(playlist)
        .ok_or(ModelError::UnknownPlaylist(playlist))?;
    if list.entries.iter().any(|e| state.is_on_air(e.id)) {
        return Err(ModelError::PlaylistOnAir(playlist));
    }
    let removed = state.playlists.remove(playlist)?;
    let first = state.playlists.first_id().ok_or(ModelError::NoPlaylists)?;
    for p in &mut state.players {
        if p.playlist == playlist {
            p.playlist = first;
        }
    }
    let ids: HashSet<EntryId> = removed.entries.iter().map(|e| e.id).collect();
    detach_entries(state, &ids, |_, _| None, out);
    forget_unreferenced(state, removed.entries.iter().map(|e| e.track));
    refresh_next(state);
    Ok(())
}

fn set_player_count(
    state: &mut AppState,
    count: usize,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let max = state.config.limits.max_players;
    if count == 0 || count > max {
        return Err(ModelError::PlayerCountOutOfRange {
            requested: count,
            max,
        });
    }
    if count < state.players.len() {
        if let Some(busy) = state.players[count..]
            .iter()
            .find(|p| p.transport != Transport::Stopped || p.cue.is_some())
        {
            return Err(ModelError::PlayerBusy(busy.id));
        }
        for p in state.players.drain(count..) {
            out.push(EngineAction::RemovePlayer { player: p.id });
        }
    } else {
        let first = state.playlists.first_id().ok_or(ModelError::NoPlaylists)?;
        let mode = state.config.players.default_mode;
        while state.players.len() < count {
            let id = state.ids.player();
            state.players.push(PlayerState::new(id, first, mode));
            out.push(EngineAction::AddPlayer { player: id });
        }
        fill_empty_next(state);
    }
    state.config.players.count = count;
    Ok(())
}

/// Rule 12: the next entry becomes current. Returns the request for the new
/// current source, or `None` (state untouched) when there is nothing to play.
pub(crate) fn advance(state: &mut AppState, i: usize) -> Option<SourceRequest> {
    let next = state.players[i].next?;
    advance_to(state, i, next)
}

/// Makes `target` the current entry: the normal advance when it is the next,
/// or whatever the engine really started. An explicit next that differs from
/// `target` is kept; otherwise the next is derived from the playlist.
pub(crate) fn advance_to(state: &mut AppState, i: usize, target: EntryId) -> Option<SourceRequest> {
    let request = state.request_from_cue_in(target)?;
    if let Some(current) = state.players[i].current {
        state.playlists.mark_played(current);
    }
    let following = state.playlists.next_playable_after(target, &state.library);
    let player = &mut state.players[i];
    let keep_next = player.next_explicit && player.next.is_some() && player.next != Some(target);
    if !keep_next {
        player.next = following;
        player.next_explicit = false;
    }
    player.current = Some(target);
    player.transport = Transport::Playing;
    player.fade_stop_pending = false;
    // The preloaded source is now the current one, and the old plan no longer applies.
    player.preloaded = None;
    player.scheduled = None;
    Some(request)
}

/// Rule 7: stop semantics shared by Stop, fade stop, stop-after-current and
/// Single mode. An explicit next is kept; a derived one becomes the entry
/// after the stopped one.
pub(crate) fn stop_player(state: &mut AppState, i: usize) {
    if let Some(current) = state.players[i].current {
        state.playlists.mark_played(current);
        if !state.players[i].next_explicit || state.players[i].next.is_none() {
            state.players[i].next = state.playlists.next_playable_after(current, &state.library);
            state.players[i].next_explicit = false;
        }
    }
    let player = &mut state.players[i];
    player.current = None;
    player.transport = Transport::Stopped;
    player.fading = false;
    player.fade_stop_pending = false;
    player.stop_after_current = false;
}

/// After playlist edits: a player with a current entry and a derived next
/// re-derives it (so insertions right after the current track are picked up),
/// and idle players pick up newly available entries.
pub(crate) fn refresh_next(state: &mut AppState) {
    let derived: Vec<(usize, Option<EntryId>)> = state
        .players
        .iter()
        .enumerate()
        .filter(|(_, p)| !p.next_explicit)
        .filter_map(|(i, p)| {
            p.current
                .map(|c| (i, state.playlists.next_playable_after(c, &state.library)))
        })
        .collect();
    for (i, next) in derived {
        state.players[i].next = next;
    }
    fill_empty_next(state);
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
    let plans: Vec<(usize, Option<TransitionPlan>)> = state
        .players
        .iter()
        .enumerate()
        .map(|(i, p)| (i, plan_for(state, p)))
        .filter(|(i, plan)| state.players[*i].scheduled != *plan)
        .collect();
    for (i, plan) in plans {
        let player = &mut state.players[i];
        player.scheduled = plan;
        out.push(EngineAction::Schedule {
            player: player.id,
            plan,
        });
    }
}
