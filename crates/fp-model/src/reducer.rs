//! The player behaviour rules of spec §3, as pure state transitions.

use std::collections::HashSet;
use std::path::PathBuf;

use crate::cart_rules;
use crate::command::{
    Command, EngineAction, EngineEvent, SOURCE_END, SourceRequest, TransitionPlan,
};
use crate::config::Config;
use crate::error::ModelError;
use crate::ids::{EntryId, PlayerId, PlaylistId, TrackId};
use crate::player::{CueState, PlayMode, PlayerState, Transport};
use crate::playlist::{Playlist, PlaylistEntry};
use crate::shortcuts::{Shortcut, default_shortcuts};
use crate::state::AppState;
use crate::track::{FileState, MarkerKind, Track};

/// Applies a user command. On `Err` the state is unchanged.
pub fn apply(state: &mut AppState, command: Command) -> Result<Vec<EngineAction>, ModelError> {
    let mut out = Vec::new();
    match command {
        Command::Play(id) => play(state, id, &mut out)?,
        Command::Pause(id) => pause(state, id, &mut out)?,
        Command::Stop(id) => stop(state, id, &mut out)?,
        Command::FadeStop(id) => fade_stop(state, id, &mut out)?,
        Command::Restart(id) => {
            let i = state.player_index(id)?;
            let player = &state.players[i];
            // A fade stop has already taken the source: nothing to seek.
            if player.transport != Transport::Stopped
                && !player.fade_stopping()
                && let Some(request) = player.current.and_then(|c| state.request_from_cue_in(c))
            {
                out.push(EngineAction::Seek {
                    player: id,
                    secs: request.from_secs,
                });
            }
        }
        Command::Previous(id) => previous(state, id, &mut out)?,
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
        Command::SetCue(id, on) => {
            if state.player(id)?.cue.is_some() != on {
                toggle_cue(state, id, &mut out)?;
            }
        }
        Command::SetStopAfterCurrent(id, on) => {
            if state.player(id)?.stop_after_current != on {
                out.extend(apply(state, Command::ToggleStopAfterCurrent(id))?);
            }
        }
        Command::CueEntry(id, entry) => cue_entry(state, id, entry, &mut out)?,
        Command::SetVolume(id, volume) => {
            let i = state.player_index(id)?;
            // A broken value never silences what is on air: it is ignored.
            if volume.is_nan() {
                return Ok(out);
            }
            let volume = volume.clamp(0.0, 1.0);
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
            state.players[i].columns = widths.normalized();
        }
        Command::ApplyAnalysis { track, analysis } => {
            // Analysis may finish after the track was removed: nothing to do.
            if let Some(t) = state.library.get_mut(track) {
                t.apply_analysis(&analysis);
                refresh_next(state);
            }
        }
        Command::SetFileState {
            track,
            state: file_state,
        } => {
            if let Some(t) = state.library.get_mut(track) {
                t.file_state = file_state;
                refresh_next(state);
            }
        }
        Command::RemoveEntry(entry) => remove_entry(state, entry, &mut out)?,
        Command::MoveEntry { entry, to, index } => {
            state.playlists.move_entry(entry, to, index)?;
            refresh_next(state);
        }
        Command::ToggleEntryRepeat(entry) => {
            let e = state
                .playlists
                .entry_mut(entry)
                .ok_or(ModelError::UnknownEntry(entry))?;
            e.repeat = !e.repeat;
        }
        Command::SetEntryRepeat(entry, on) => {
            state
                .playlists
                .entry_mut(entry)
                .ok_or(ModelError::UnknownEntry(entry))?
                .repeat = on;
        }
        Command::SetEntryStopAfter(entry, on) => {
            state
                .playlists
                .entry_mut(entry)
                .ok_or(ModelError::UnknownEntry(entry))?
                .stop_after = on;
        }
        Command::ToggleEntryStopAfter(entry) => {
            let e = state
                .playlists
                .entry_mut(entry)
                .ok_or(ModelError::UnknownEntry(entry))?;
            e.stop_after = !e.stop_after;
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
        Command::CreatePlaylistFromPaths { name, paths } => {
            let id = state.ids.playlist();
            state.playlists.add(Playlist::new(id, name));
            insert_paths(state, id, 0, paths)?;
        }
        Command::RenamePlaylist { playlist, name } => state.playlists.rename(playlist, name)?,
        Command::DeletePlaylist(playlist) => delete_playlist(state, playlist, &mut out)?,
        Command::SetPlayerCount(count) => set_player_count(state, count, &mut out)?,
        Command::UpdateConfig(config) => update_config(state, *config, &mut out)?,
        Command::FireCart(cart) => cart_rules::fire(state, cart, &mut out)?,
        Command::StopCart(cart) => cart_rules::stop(state, cart, &mut out)?,
        Command::StopAllCarts => cart_rules::stop_all(state, &mut out),
        Command::CueCart(cart) => cart_rules::cue(state, cart, &mut out)?,
        Command::SetCartCue(cart, on) => {
            state
                .cartwall
                .cart(cart)
                .ok_or(ModelError::UnknownCart(cart))?;
            if (state.cartwall.cue == Some(cart)) != on {
                cart_rules::cue(state, cart, &mut out)?;
            }
        }
        Command::CreateCartPage { name } => cart_rules::create_page(state, name),
        Command::RenameCartPage { page, name } => cart_rules::rename_page(state, page, name)?,
        Command::DeleteCartPage(page) => cart_rules::delete_page(state, page, &mut out)?,
        Command::ResizeCartPage { page, rows, cols } => {
            cart_rules::resize_page(state, page, rows, cols)?
        }
        Command::SetCart { page, index, edit } => cart_rules::set_cart(state, page, index, edit)?,
        Command::AssignCartFile { page, index, path } => {
            cart_rules::assign_file(state, page, index, Some(path), &mut out)?
        }
        Command::ClearCartFile { page, index } => {
            cart_rules::assign_file(state, page, index, None, &mut out)?
        }
        Command::ImportCartPage(import) => cart_rules::import_page(state, *import)?,
        Command::ShowCartPage(page) => {
            if state.cartwall.page(page).is_none() {
                return Err(ModelError::UnknownCartPage(page));
            }
            state.cartwall.shown = Some(page);
        }
        Command::SetCartwallOpen(open) => state.cartwall.open = open,
        Command::SetMarker { track, kind, secs } => set_marker(state, track, kind, secs)?,
        Command::SetShortcut { action, chord } => {
            let shortcuts = &mut state.config.shortcuts;
            shortcuts.retain(|s| s.action != action && Some(&s.chord) != chord.as_ref());
            if let Some(chord) = chord {
                shortcuts.push(Shortcut { action, chord });
            }
        }
        Command::ResetShortcuts => state.config.shortcuts = default_shortcuts(),
        Command::ResetMarkers { track } => {
            let t = state
                .library
                .get_mut(track)
                .ok_or(ModelError::UnknownTrack(track))?;
            t.markers.clear_manual();
            t.analyzed = false;
            refresh_next(state);
        }
    }
    fill_empty_next(state);
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
            // The engine started the entry that is already current: a
            // repeating entry's next pass (R26), perhaps committed just before
            // a command changed the model. It stays current; the engine used
            // up its preload and plan, so `reconcile` sends new ones, and the
            // audio is playing whatever the model asked meanwhile (a pause
            // that arrived after the restart does not stop it).
            if let Ok(i) = state.player_index(player)
                && state.players[i].current == Some(entry)
                && state.players[i].transport != Transport::Stopped
            {
                let p = &mut state.players[i];
                p.preloaded = None;
                p.scheduled = None;
                p.transport = Transport::Playing;
            }
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
                if advance_to(state, i, entry, true).is_some() {
                    state.players[i].fading = overlapping;
                } else {
                    stop_player(state, i);
                    out.push(EngineAction::StopNow { player });
                }
            }
        }
        EngineEvent::PreloadFailed { player, entry } => {
            preload_failed(state, player, entry, &mut out)
        }
        EngineEvent::SourceFailed { player, entry } => {
            source_failed(state, player, entry, &mut out)
        }
        EngineEvent::CartEnded { cart } => cart_rules::ended(state, cart),
        EngineEvent::CartFailed { cart } => cart_rules::failed(state, cart),
        EngineEvent::CartCueEnded { cart } => cart_rules::cue_ended(state, cart),
        EngineEvent::CueEnded { player, entry } => {
            if let Ok(i) = state.player_index(player)
                && state.players[i].cue.is_some_and(|c| c.entry == entry)
            {
                state.players[i].cue = None;
            }
        }
    }
    fill_empty_next(state);
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

/// R24: pops the history until an entry that still exists and can play,
/// crossfades into it like Play-while-Playing, and queues the entry left as
/// the explicit next without recording it (so Previous keeps going back).
fn previous(
    state: &mut AppState,
    id: PlayerId,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let i = state.player_index(id)?;
    let player = &state.players[i];
    if player.transport != Transport::Playing || player.fading {
        return Ok(());
    }
    let left = player.current;
    let target = loop {
        let Some(entry) = state.players[i].history.pop() else {
            return Ok(());
        };
        if Some(entry) != left && state.playable_request(entry).is_some() {
            break entry;
        }
    };
    let fade_ms = state.config.players.fade_ms;
    if let Some(request) = advance_to(state, i, target, false) {
        let player = &mut state.players[i];
        player.fading = true;
        if let Some(left) = left {
            player.next = Some(left);
            player.next_explicit = true;
        }
        out.push(EngineAction::Crossfade {
            player: id,
            request,
            fade_ms,
        });
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
        entries.push(PlaylistEntry::new(state.ids.entry(), track));
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

/// The source prepared to play next could not be opened: its file is
/// unreadable, and whoever was going to play it gets another next. What is
/// on air plays on, even when it is the same entry (a repeat, R26): its
/// pass ends normally and the player moves on.
fn preload_failed(
    state: &mut AppState,
    player: PlayerId,
    entry: EntryId,
    out: &mut Vec<EngineAction>,
) {
    let on_air = state
        .player_index(player)
        .is_ok_and(|i| state.players[i].current == Some(entry));
    if !on_air {
        source_failed(state, player, entry, out);
        return;
    }
    if let Some(t) = state
        .playlists
        .entry(entry)
        .map(|e| e.track)
        .and_then(|track| state.library.get_mut(track))
    {
        t.file_state = FileState::Unreadable;
    }
    if let Ok(i) = state.player_index(player) {
        // The failed preload is gone.
        state.players[i].preloaded = None;
    }
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
    let current = player.current?;
    let track = state.track_for_entry(current)?;
    let end = track.known_cue_out_secs().unwrap_or(SOURCE_END);
    // R27: an entry marked "stop after" stops the player, in any mode.
    if state.playlists.entry(current).is_some_and(|e| e.stop_after) {
        return Some(TransitionPlan::StopAt { at_secs: end });
    }
    // R26: a repeating entry starts again at its own cue-in, gaplessly, as a
    // hard transition into itself (its preload is the entry, see
    // `preload_target`).
    if repeating(state, player) {
        return Some(TransitionPlan::StartNextAt {
            at_secs: end,
            fade_current_until_secs: None,
        });
    }
    if player.mode == PlayMode::Single || player.stop_after_current || player.next.is_none() {
        return Some(TransitionPlan::StopAt { at_secs: end });
    }
    // A segue only makes sense strictly inside the effective cue range (an
    // automatic one may be stale against a manual cue-out).
    let segue = track
        .segue_start_secs()
        .filter(|s| *s >= track.cue_in_secs() && *s < end);
    match segue {
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

/// Phase 2 spec P2.8: a manual marker, validated against the effective cue
/// range. Scheduling follows through `reconcile`.
fn set_marker(
    state: &mut AppState,
    track: TrackId,
    kind: MarkerKind,
    secs: Option<f64>,
) -> Result<(), ModelError> {
    let t = state
        .library
        .get(track)
        .ok_or(ModelError::UnknownTrack(track))?;
    let cue_in = t.cue_in_secs();
    let cue_out = t.known_cue_out_secs();
    let duration = (t.duration_secs > 0.0).then_some(t.duration_secs);
    let value = match secs {
        None => None,
        Some(v) if !v.is_finite() => return Err(ModelError::InvalidMarker),
        Some(v) => Some(match kind {
            // Cue points only make sense once the length is known.
            MarkerKind::CueIn | MarkerKind::CueOut if duration.is_none() => {
                return Err(ModelError::InvalidMarker);
            }
            MarkerKind::CueIn => {
                let v = v.max(0.0);
                if cue_out.is_some_and(|out| v >= out) {
                    return Err(ModelError::InvalidMarker);
                }
                v
            }
            MarkerKind::CueOut => {
                let v = duration.map_or(v, |d| v.min(d));
                if v <= cue_in {
                    return Err(ModelError::InvalidMarker);
                }
                v
            }
            _ => v.max(cue_in).min(cue_out.unwrap_or(f64::INFINITY)),
        }),
    };
    if let Some(t) = state.library.get_mut(track) {
        t.markers.set_manual(kind, value);
        if matches!(kind, MarkerKind::CueIn | MarkerKind::CueOut) {
            // Keep the inner markers within the new range.
            let (cue_in, cue_out) = (t.cue_in_secs(), t.known_cue_out_secs());
            t.markers
                .clamp_manual_inner(cue_in, cue_out.unwrap_or(f64::INFINITY));
        }
        if value.is_none() {
            // Let analysis put the automatic value back.
            t.analyzed = false;
        }
    }
    refresh_next(state);
    Ok(())
}

/// Drops library tracks that no playlist entry or cart references any more.
fn forget_unreferenced(state: &mut AppState, tracks: impl IntoIterator<Item = TrackId>) {
    cart_rules::forget_tracks(state, tracks);
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

/// Takes a new configuration, validated. The player count only changes
/// through `SetPlayerCount`, except that a lower `limits.max_players`
/// removes the players above it, refused while one of them is busy.
fn update_config(
    state: &mut AppState,
    mut config: Config,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let _ = config.validate();
    let count = state.players.len().min(config.limits.max_players);
    if let Some(busy) = state.players[count..]
        .iter()
        .find(|p| p.transport != Transport::Stopped || p.cue.is_some())
    {
        return Err(ModelError::PlayerBusy(busy.id));
    }
    config.players.count = state.config.players.count;
    state.config = config;
    if count < state.players.len() {
        set_player_count(state, count, out)?;
    }
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
    advance_to(state, i, next, true)
}

/// R25: records that the player left `entry`, keeping at most
/// `players.history_len` entries.
pub(crate) fn push_history(state: &mut AppState, i: usize, entry: EntryId) {
    let cap = state.config.players.history_len;
    let history = &mut state.players[i].history;
    history.push(entry);
    let excess = history.len().saturating_sub(cap);
    history.drain(..excess);
}

/// Makes `target` the current entry: the normal advance when it is the next,
/// or whatever the engine really started. An explicit next that differs from
/// `target` is kept; otherwise the next is derived from the playlist. The
/// entry left is recorded in the history when `record` (not by Previous).
pub(crate) fn advance_to(
    state: &mut AppState,
    i: usize,
    target: EntryId,
    record: bool,
) -> Option<SourceRequest> {
    let request = state.request_from_cue_in(target)?;
    if let Some(current) = state.players[i].current {
        let player = state.players[i].id;
        state.playlists.mark_played(current, player);
        if record {
            push_history(state, i, current);
        }
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
        let player = state.players[i].id;
        state.playlists.mark_played(current, player);
        push_history(state, i, current);
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

/// R26: the player's current entry repeats (and R27, stop-after-current or
/// a fade stop do not end it first).
fn repeating(state: &AppState, player: &PlayerState) -> bool {
    player.transport != Transport::Stopped
        && !player.fade_stop_pending
        && !player.stop_after_current
        && player
            .current
            .and_then(|c| state.playlists.entry(c))
            .is_some_and(|e| e.repeat && !e.stop_after)
        // A file that can no longer be opened plays its pass out, once.
        && player
            .current
            .and_then(|c| state.track_for_entry(c))
            .is_some_and(|t| t.file_state.is_playable())
}

/// The entry to preload: the current one while it repeats, else the next.
fn preload_target(state: &AppState, player: &PlayerState) -> Option<EntryId> {
    if repeating(state, player) {
        player.current
    } else {
        player.next
    }
}

/// Derives the engine work implied by the state: preload whatever is next.
pub(crate) fn reconcile(state: &mut AppState, out: &mut Vec<EngineAction>) {
    let preloads: Vec<(usize, Option<SourceRequest>)> = state
        .players
        .iter()
        .enumerate()
        .map(|(i, p)| {
            (
                i,
                preload_target(state, p).and_then(|e| state.request_from_cue_in(e)),
            )
        })
        .filter(|(i, request)| {
            let wanted = request.as_ref().map(|r| (r.entry, r.from_secs));
            state.players[*i].preloaded != wanted
        })
        .collect();
    for (i, request) in preloads {
        let player = &mut state.players[i];
        player.preloaded = request.as_ref().map(|r| (r.entry, r.from_secs));
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
