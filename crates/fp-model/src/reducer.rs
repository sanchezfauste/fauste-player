//! The player behaviour rules of spec §3, as pure state transitions.

use std::collections::HashSet;
use std::path::PathBuf;

use crate::cart_rules;
use crate::command::{
    Command, CueStart, EngineAction, EngineEvent, SOURCE_END, SourceRequest, TransitionPlan,
};
use crate::config::Config;
use crate::dsd::DsdOnAir;
use crate::error::ModelError;
use crate::ids::{EntryId, PlayerId, PlaylistId, TrackId};
use crate::live::Holder;
use crate::player::{CueState, FadeOrigin, PlayMode, PlayerState, Transport};
use crate::playlist::{Playlist, PlaylistEntry};
use crate::shortcuts::{Shortcut, default_shortcuts};
use crate::state::AppState;
use crate::track::{FileState, MarkerKind, Track};

/// Applies a user command. On `Err` the state is unchanged.
pub fn apply(state: &mut AppState, command: Command) -> Result<Vec<EngineAction>, ModelError> {
    let before = crate::cue_on_air::before_step(state);
    let mut out = apply_command(state, command)?;
    crate::cue_on_air::after_step(state, &before, &mut out);
    Ok(out)
}

/// `apply`, without the on-air pre-listen upkeep that closes every step.
fn apply_command(state: &mut AppState, command: Command) -> Result<Vec<EngineAction>, ModelError> {
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
                && let Some(request) = player
                    .current
                    .and_then(|c| state.request_from_cue_in(c, Holder::PlayerMain(id)))
            {
                out.push(EngineAction::Seek {
                    player: id,
                    secs: request.from_secs,
                });
            }
        }
        Command::Previous(id) => previous(state, id, &mut out)?,
        Command::SetNext(id, entry) => {
            set_next(state, id, entry)?;
            follow_cue(state, id, entry, &mut out)?;
        }
        Command::InsertPaths {
            playlist,
            index,
            paths,
        } => insert_paths(state, playlist, index, paths)?,
        Command::InsertTracks {
            playlist,
            index,
            tracks,
        } => {
            if state.playlists.get(playlist).is_none() {
                return Err(ModelError::UnknownPlaylist(playlist));
            }
            if let Some(missing) = tracks.iter().find(|t| state.library.get(**t).is_none()) {
                return Err(ModelError::UnknownTrack(*missing));
            }
            let entries = tracks
                .into_iter()
                .map(|t| PlaylistEntry::new(state.ids.entry(), t))
                .collect();
            state.playlists.insert(playlist, index, entries)?;
            refresh_next(state);
        }
        Command::SetMode(id, mode) => {
            let i = state.player_index(id)?;
            state.players[i].mode = mode;
            // O38: in Single mode the flag only makes sense while the current
            // entry repeats.
            if mode == PlayMode::Single && !repeat_entry(state, &state.players[i]) {
                state.players[i].stop_after_current = false;
            }
        }
        Command::ToggleStopAfterCurrent(id) => {
            let i = state.player_index(id)?;
            if state.players[i].mode == PlayMode::Single && !repeat_entry(state, &state.players[i])
            {
                return Err(ModelError::StopAfterInSingle);
            }
            let player = &mut state.players[i];
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
        Command::CueEntry(id, entry) => cue_entry(state, id, entry, false, &mut out)?,
        Command::SeekCue(id, secs) => seek_cue(state, id, secs, &mut out)?,
        Command::SetCuePaused(id, paused) => set_cue_paused(state, id, paused, &mut out)?,
        Command::CueToNext(id) => {
            let i = state.player_index(id)?;
            if let Some(cue) = state.players[i].cue {
                set_next(state, id, cue.entry)?;
            }
        }
        Command::ForceCue(start) => force_cue(state, start, &mut out)?,
        Command::CancelCueOnAir(id) => crate::cue_on_air::cancel(state, id),
        Command::SetVolume(id, volume) => {
            let i = state.player_index(id)?;
            // A broken value never silences what is on air: it is ignored.
            if volume.is_nan() {
                return Ok(out);
            }
            // BF3 (bit-perfect rate and fader spec): a bit-perfect player's
            // fader is held at exactly 100 %; only that level is accepted,
            // and it changes nothing.
            if crate::bit_perfect::fader_locked(state, id) {
                #[allow(clippy::float_cmp)] // exactly unity is the only level allowed
                if volume != 1.0 {
                    return Err(ModelError::FaderLocked(id));
                }
                return Ok(out);
            }
            let volume = volume.clamp(0.0, 1.0);
            state.players[i].volume = volume;
            // O25. Since the bit-perfect rate and fader spec (§5.5) a DSD
            // stream only plays where the fader is locked, so the fader no
            // longer reaches this rule; it stays as a guard.
            #[allow(clippy::float_cmp)] // only exactly unity keeps DSD unchanged
            if volume != 1.0 && state.players[i].dsd.take().is_some() {
                out.push(EngineAction::LeaveDsd { player: id });
            }
            out.push(EngineAction::SetVolume { player: id, volume });
        }
        Command::Seek(id, secs) => {
            let i = state.player_index(id)?;
            let player = &state.players[i];
            if player.transport == Transport::Stopped {
                // Rule 3a (Q8.2): where Play will start the next. Nothing
                // goes to the engine but the preload (`reconcile`).
                if let Some(next) = player.next
                    && secs.is_finite()
                {
                    let at = crate::pending_start::pending_start_at(
                        state,
                        next,
                        secs,
                        Holder::PlayerMain(id),
                    );
                    state.players[i].pending_start = at;
                }
            } else if let Some(request) = player
                .current
                .and_then(|c| state.request_at(c, secs, Holder::PlayerMain(id)))
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
        Command::SetPlayerShares { left, right } => {
            crate::layout::set_player_shares(state, left, right)?
        }
        Command::EqualPlayerWidths => crate::layout::equal_player_widths(state)?,
        Command::SetCartwallDock(dock) => crate::layout::set_cartwall_dock(state, dock)?,
        Command::SetCartwallSize(size) => crate::layout::set_cartwall_size(state, size)?,
        Command::SetPanelHidden { panel, hidden } => {
            crate::layout::set_panel_hidden(state, panel, hidden)?
        }
        Command::SetLayoutLocked(locked) => state.layout.locked = locked,
        Command::ResetLayout => crate::layout::reset_layout(state)?,
        Command::ApplyAnalysis { track, analysis } => {
            // Analysis may finish after the track was removed: nothing to do.
            if let Some(t) = state.library.get_mut(track) {
                t.apply_analysis(&analysis);
                refresh_next(state);
            }
        }
        Command::ApplyLoudness {
            track,
            loudness,
            version,
        } => {
            if let Some(t) = state.library.get_mut(track) {
                t.apply_loudness(loudness, version);
            }
        }
        Command::ApplyTags { track, tags } => {
            if let Some(t) = state.library.get_mut(track) {
                t.apply_tags(&tags);
            }
        }
        Command::SetDuration { track, secs } => {
            // Q1.2: the header's length stands in until the analysis.
            if secs.is_finite()
                && secs > 0.0
                && let Some(t) = state.library.get_mut(track)
                && !t.analyzed
            {
                t.duration_secs = secs;
            }
        }
        Command::SetFileState {
            track,
            state: file_state,
        } => {
            if let Some(t) = state.library.get_mut(track) {
                t.file_state = file_state;
                // A self-next (O37) whose file can no longer be opened gives
                // way to the entry after it: the player goes on at cue-out
                // instead of stopping there.
                if !file_state.is_playable() {
                    for i in 0..state.players.len() {
                        let p = &state.players[i];
                        if let Some(entry) = p.current
                            && p.next == Some(entry)
                            && state.track_for_entry(entry).is_some_and(|t| t.id == track)
                        {
                            replace_self_next(state, i, entry);
                        }
                    }
                }
                refresh_next(state);
            }
        }
        Command::RemoveEntry(entry) => remove_entry(state, entry, &mut out)?,
        Command::MoveEntry { entry, to, index } => {
            state.playlists.move_entry(entry, to, index)?;
            // Q8.4: a moved entry loses the start chosen for it.
            for p in &mut state.players {
                if p.pending_start.is_some_and(|(e, _)| e == entry) {
                    p.pending_start = None;
                }
            }
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
        Command::ResetPlayed(playlist) => crate::reset_played::reset_played(state, playlist)?,
        Command::CreatePlaylist { name } => {
            let id = state.ids.playlist();
            state.playlists.add(Playlist::new(id, name));
        }
        Command::CreatePlaylistIn { name, player } => {
            let i = state.player_index(player)?;
            let id = state.ids.playlist();
            state.playlists.add(Playlist::new(id, name));
            state.players[i].playlist = id;
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
        Command::CueCart(cart) => cart_rules::cue(state, cart, false, &mut out)?,
        Command::SetCartCue(cart, on) => {
            state
                .cartwall
                .cart(cart)
                .ok_or(ModelError::UnknownCart(cart))?;
            if (state.cartwall.cue == Some(cart)) != on {
                cart_rules::cue(state, cart, false, &mut out)?;
            }
        }
        Command::CreateCartPage { name } => cart_rules::create_page(state, name),
        Command::RenameCartPage { page, name } => cart_rules::rename_page(state, page, name)?,
        Command::DeleteCartPage(page) => cart_rules::delete_page(state, page, &mut out)?,
        Command::ResizeCartPage { page, rows, cols } => {
            cart_rules::resize_page(state, page, rows, cols)?
        }
        Command::SetCart { page, index, edit } => cart_rules::set_cart(state, page, index, edit)?,
        Command::EditCartPage { page, name, grid } => {
            cart_rules::edit_page(state, page, name, grid)?
        }
        Command::EditCart {
            page,
            index,
            edit,
            file,
        } => cart_rules::edit_cart(state, page, index, edit, file, &mut out)?,
        Command::AssignCartFile { page, index, path } => {
            cart_rules::assign_file(state, page, index, Some(path), &mut out)?
        }
        Command::AssignCartTrack { page, index, track } => {
            cart_rules::assign_track(state, page, index, track, &mut out)?
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
        Command::RestoreDefaults(section) => {
            let mut config = state.config.clone();
            crate::restore::restore_defaults(&mut config, section);
            update_config(state, config, &mut out)?;
        }
        Command::ApplySettingsNow => crate::live::apply_now(state, &mut out),
        Command::ResetMarkers { track } => {
            let t = state
                .library
                .get_mut(track)
                .ok_or(ModelError::UnknownTrack(track))?;
            t.markers.clear_manual();
            t.analyzed = false;
            refresh_next(state);
        }
        Command::SetTrackLoudness {
            track,
            excluded,
            trim_db,
        } => {
            // The same maximum the gain rule uses (N7).
            let max = crate::loudness::max_trim_in_use(&state.config.loudness);
            let t = state
                .library
                .get_mut(track)
                .ok_or(ModelError::UnknownTrack(track))?;
            if !trim_db.is_finite() || trim_db.abs() > max {
                return Err(ModelError::InvalidTrim);
            }
            t.loudness_edit = crate::loudness::LoudnessEdit { excluded, trim_db };
        }
    }
    fill_empty_next(state);
    reconcile(state, &mut out);
    crate::live::dispatch_due(state, &mut out);
    Ok(out)
}

/// Applies an engine observation. Events about unknown players are ignored
/// (the player may have been removed while the event was in flight).
pub fn on_event(state: &mut AppState, event: EngineEvent) -> Vec<EngineAction> {
    let before = crate::cue_on_air::before_step(state);
    let mut out = on_engine_event(state, event);
    crate::cue_on_air::after_step(state, &before, &mut out);
    out
}

/// `on_event`, without the on-air pre-listen upkeep that closes every step.
fn on_engine_event(state: &mut AppState, event: EngineEvent) -> Vec<EngineAction> {
    let mut out = Vec::new();
    match event {
        EngineEvent::FadeCompleted { player } => {
            if let Ok(i) = state.player_index(player) {
                state.players[i].fading = false;
                state.players[i].fade_ms = 0;
            }
        }
        EngineEvent::ReachedEnd { player, entry } => {
            if let Ok(i) = state.player_index(player)
                && state.players[i].current == Some(entry)
            {
                let held = dsd_holds(&state.players[i]);
                match held.then(|| dsd_hold_continue(state, i)).flatten() {
                    Some(request) => out.push(EngineAction::StartCurrent { player, request }),
                    None => stop_player(state, i),
                }
            }
        }
        EngineEvent::DsdStarted {
            player,
            entry,
            hold_others,
        } => {
            if let Ok(i) = state.player_index(player)
                && state.players[i].current == Some(entry)
                && state.players[i].transport != Transport::Stopped
            {
                state.players[i].dsd = Some(DsdOnAir { entry, hold_others });
            }
        }
        EngineEvent::DsdEnded { player, entry } => {
            if let Ok(i) = state.player_index(player)
                && state.players[i].dsd.is_some_and(|d| d.entry == entry)
            {
                state.players[i].dsd = None;
            }
        }
        EngineEvent::TransitionStarted {
            player,
            entry,
            normalization,
        } => {
            // N11: the source keeps the gain the engine started it with,
            // whatever the model preloaded since (a re-preload may race a
            // transition the engine had already committed).
            // The engine started the entry that is already current: a
            // repeating entry's next pass (R26) or the replay of an entry set
            // as its own next (O37), perhaps committed just before a command
            // changed the model. The engine used up its preload and plan, so
            // `reconcile` sends new ones, and the audio is playing whatever
            // the model asked meanwhile (a pause that arrived after the
            // restart does not stop it).
            if let Ok(i) = state.player_index(player)
                && state.players[i].current == Some(entry)
                && state.players[i].transport != Transport::Stopped
            {
                // A repeat pass keeps the entry as it is. Otherwise a
                // self-next is the replay: an ordinary play of the same
                // entry (played mark, history, next derived from the playlist).
                let replay =
                    !repeating(state, &state.players[i]) && state.players[i].next == Some(entry);
                if replay && advance_to(state, i, entry, true).is_none() {
                    // The file cannot be read any more: the pass already
                    // started plays out; the next is the entry after it.
                    let following = state.playlists.next_playable_after(entry, &state.library);
                    state.players[i].next = following;
                    state.players[i].next_explicit = false;
                }
                let p = &mut state.players[i];
                p.preloaded = None;
                p.scheduled = None;
                p.transport = Transport::Playing;
                p.on_air = Some(normalization);
            }
            // A transition reported after the player was stopped is stale: the
            // engine has already been told to stop everything.
            if let Ok(i) = state.player_index(player)
                && state.players[i].current != Some(entry)
                && state.players[i].transport != Transport::Stopped
            {
                let overlap_ms = match state.players[i].scheduled {
                    Some(TransitionPlan::StartNextAt {
                        at_secs,
                        fade_current_until_secs: Some(end),
                        ..
                    }) => Some(((end - at_secs).max(0.0) * 1000.0).round() as u32),
                    _ => None,
                };
                if advance_to(state, i, entry, true).is_some() {
                    match overlap_ms {
                        Some(ms) => state.players[i].begin_fade(ms, FadeOrigin::Segue),
                        None => {
                            state.players[i].fading = false;
                            state.players[i].fade_ms = 0;
                        }
                    }
                    state.players[i].on_air = Some(normalization);
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
        EngineEvent::Placed {
            holder,
            route,
            device,
            running,
        } => crate::live::placed(state, holder, route, device, running),
        EngineEvent::Unplaced { holder, route } => crate::live::unplaced(state, holder, route),
        EngineEvent::Gone { holder } => crate::live::gone(state, holder),
        EngineEvent::AudioSystemInUse { configured, in_use } => {
            crate::live::audio_system_in_use(state, configured, in_use)
        }
        EngineEvent::Applied {
            target,
            wanted,
            outcome,
        } => crate::live::applied(state, target, wanted, outcome),
        EngineEvent::StreamRate {
            device,
            rate,
            refused,
        } => crate::live::stream_rate(state, device, rate, refused),
    }
    fill_empty_next(state);
    reconcile(state, &mut out);
    crate::live::dispatch_due(state, &mut out);
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
            // Q8.3: the pending start is used by this start, once.
            let pending = state.players[i].pending_start.take();
            if let Some(request) = advance(state, i) {
                let request = crate::pending_start::start_request(
                    state,
                    pending,
                    request,
                    Holder::PlayerMain(id),
                );
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
            let held = dsd_holds(&state.players[i]);
            if let Some(request) = advance(state, i) {
                start_or_crossfade(state, i, held, request, FadeOrigin::Next, out);
            }
        }
    }
    Ok(())
}

/// R24: pops the history until an entry that still exists and can play,
/// starts it like Play-while-Playing (a crossfade, or a hard start on a
/// bit-perfect player), and queues the entry left as
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
        if Some(entry) != left
            && state
                .playable_request(entry, Holder::PlayerMain(id))
                .is_some()
        {
            break entry;
        }
    };
    let held = dsd_holds(&state.players[i]);
    if let Some(request) = advance_to(state, i, target, false) {
        if let Some(left) = left {
            let player = &mut state.players[i];
            player.next = Some(left);
            player.next_explicit = true;
        }
        start_or_crossfade(state, i, held, request, FadeOrigin::Previous, out);
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
    // Q8.4: Stop forgets a pending start, even on a stopped player.
    state.players[i].pending_start = None;
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
    if state.players[i].dsd.is_some() && state.players[i].transport != Transport::Stopped {
        // O25: a DSD stream cannot be faded; it stops at once.
        return stop(state, id, out);
    }
    let fade_ms = state.config.players.fade_ms;
    let player = &mut state.players[i];
    // Allowed during a crossfade or segue overlap too: the engine fades every
    // Main source of the player. Only a fade stop already in progress blocks it.
    if player.transport == Transport::Playing && !player.fade_stop_pending {
        player.begin_fade(fade_ms, FadeOrigin::FadeStop);
        player.fade_stop_pending = true;
        out.push(EngineAction::FadeOutAndStop {
            player: id,
            fade_ms,
        });
    }
    Ok(())
}

/// O37: the entry on air is accepted; it plays once more.
fn set_next(state: &mut AppState, id: PlayerId, entry: EntryId) -> Result<(), ModelError> {
    let i = state.player_index(id)?;
    if state.playlists.entry(entry).is_none() {
        return Err(ModelError::UnknownEntry(entry));
    }
    state.players[i].next = Some(entry);
    state.players[i].next_explicit = true;
    // Q8.4: a new next, or the same one chosen again (Play now), starts at
    // its cue-in.
    state.players[i].pending_start = None;
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
    // A self-next (O37) on a file that cannot be read: the entry after it is
    // the next, or `advance` would go into the entry that just failed.
    for i in 0..state.players.len() {
        if state.players[i].current == Some(entry) && state.players[i].next == Some(entry) {
            replace_self_next(state, i, entry);
        }
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
        // A self-next (O37) on a file that cannot be read again: the entry
        // after it is the next.
        if state.players[i].next == Some(entry) {
            replace_self_next(state, i, entry);
        }
    }
}

/// The player's next is its own current `entry`, which cannot be played: the
/// next playable entry after it takes its place, by default rules.
fn replace_self_next(state: &mut AppState, i: usize, entry: EntryId) {
    state.players[i].next = state.playlists.next_playable_after(entry, &state.library);
    state.players[i].next_explicit = false;
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

/// Rules 9–11 (see `ordinary_plan`), then O25: a DSD stream that holds its
/// output is never overlapped; it plays to its end and the next entry
/// starts from `ReachedEnd` (`dsd_hold_continue`). Then BR4 (bit-perfect
/// rate and fader spec): a bit-perfect player never segues; it plays to
/// the end of its play range, and the device reopens at the next entry's
/// rate when BR3 allows.
pub fn plan_for(state: &AppState, player: &PlayerState) -> Option<TransitionPlan> {
    let plan = ordinary_plan(state, player)?;
    if !matches!(plan, TransitionPlan::StartNextAt { .. }) {
        return Some(plan);
    }
    if dsd_holds(player) {
        let end = current_end(state, player)?;
        return Some(TransitionPlan::StopAt { at_secs: end });
    }
    if crate::bit_perfect::bit_perfect_player(state, player.id) {
        let end = current_end(state, player)?;
        let rate_cut = preload_target(state, player)
            .and_then(|e| crate::pending_start::preload_request(state, player, e))
            .and_then(|next| crate::bit_perfect::rate_cut(state, player.id, &next));
        return Some(TransitionPlan::StartNextAt {
            at_secs: end,
            fade_current_until_secs: None,
            rate_cut,
        });
    }
    Some(plan)
}

/// The end of the play range of `player`'s current entry.
fn current_end(state: &AppState, player: &PlayerState) -> Option<f64> {
    let track = state.track_for_entry(player.current?)?;
    let range = track.play_range(state.config.players.use_cue_markers);
    Some(range.known_end().unwrap_or(SOURCE_END))
}

/// The player's DSD stream holds its output (`DsdMix::HoldOthers`).
pub(crate) fn dsd_holds(player: &PlayerState) -> bool {
    player
        .dsd
        .is_some_and(|d| d.hold_others && player.current == Some(d.entry))
}

/// O25 HoldOthers: at the end of a held DSD track, what the ordinary plan
/// would have started at its end, started now with no overlap. `None` when
/// the ordinary plan stops the player.
fn dsd_hold_continue(state: &mut AppState, i: usize) -> Option<SourceRequest> {
    let plan = ordinary_plan(state, &state.players[i])?;
    if !matches!(plan, TransitionPlan::StartNextAt { .. }) {
        return None;
    }
    state.players[i].dsd = None;
    let current = state.players[i].current?;
    if repeating(state, &state.players[i]) {
        // R26: another pass of the same entry, not a new play.
        let holder = Holder::PlayerMain(state.players[i].id);
        let request = state.request_from_cue_in(current, holder)?;
        let p = &mut state.players[i];
        p.preloaded = None;
        p.scheduled = None;
        p.on_air = Some(request.normalization);
        return Some(request);
    }
    let next = state.players[i].next?;
    advance_to(state, i, next, true)
}

/// Play-while-playing starts: a crossfade, or a hard start that cuts the
/// current track with the declick: a DSD stream holding its output (O25
/// HoldOthers, `held`) or any bit-perfect player (bit-perfect rate and
/// fader spec BR7), which never enters `fading`.
fn start_or_crossfade(
    state: &mut AppState,
    i: usize,
    held: bool,
    request: SourceRequest,
    origin: FadeOrigin,
    out: &mut Vec<EngineAction>,
) {
    let player = state.players[i].id;
    if held || crate::bit_perfect::bit_perfect_player(state, player) {
        out.push(EngineAction::StartCurrent { player, request });
        return;
    }
    let fade_ms = state.config.players.fade_ms;
    out.push(EngineAction::Crossfade {
        player,
        request,
        fade_ms,
    });
    state.players[i].begin_fade(fade_ms, origin);
}

/// Rules 9–11: what the engine must do when the current track ends. Returns
/// `None` while a fade stop is running: the engine stops and reports the end.
fn ordinary_plan(state: &AppState, player: &PlayerState) -> Option<TransitionPlan> {
    if player.transport == Transport::Stopped || player.fade_stop_pending {
        return None;
    }
    let current = player.current?;
    let track = state.track_for_entry(current)?;
    let range = track.play_range(state.config.players.use_cue_markers);
    let end = range.known_end().unwrap_or(SOURCE_END);
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
            rate_cut: None,
        });
    }
    if player.mode == PlayMode::Single || player.stop_after_current || player.next.is_none() {
        return Some(TransitionPlan::StopAt { at_secs: end });
    }
    // O37: the entry on air was set as next. It plays once more from its
    // cue-in, as a hard transition into itself (like a repeat, so no segue:
    // it would overlap the file with itself). A file that can no longer be
    // opened plays its pass out and stops.
    if player.next == Some(current) {
        return Some(if track.file_state.is_playable() {
            TransitionPlan::StartNextAt {
                at_secs: end,
                fade_current_until_secs: None,
                rate_cut: None,
            }
        } else {
            TransitionPlan::StopAt { at_secs: end }
        });
    }
    // A segue only makes sense strictly inside the effective cue range (an
    // automatic one may be stale against a manual cue-out).
    let segue = track
        .segue_start_secs()
        .filter(|s| *s >= range.cue_in && *s < end);
    match segue {
        Some(segue) if state.config.players.auto_segue && end.is_finite() => {
            Some(TransitionPlan::StartNextAt {
                at_secs: segue,
                fade_current_until_secs: Some(end),
                rate_cut: None,
            })
        }
        _ => Some(TransitionPlan::StartNextAt {
            at_secs: end,
            fade_current_until_secs: None,
            rate_cut: None,
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
        cue_entry(state, id, entry, false, out)?;
    }
    Ok(())
}

/// Starts (or moves) `id`'s CUE on `entry`. A CUE that starts from nothing
/// on a sounding on-air output records a confirmation request instead,
/// unless it is `force`d (on-air pre-listen spec CA3, CA4); a running one
/// moves without asking.
fn cue_entry(
    state: &mut AppState,
    id: PlayerId,
    entry: EntryId,
    force: bool,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let i = state.player_index(id)?;
    // Spec §4.6, CA1: without a Cue route nothing could be heard.
    if !state.config.outputs.player_has_cue(id) {
        return Ok(());
    }
    let request = state
        .request_from_cue_in(entry, Holder::PlayerCue(id))
        .ok_or(ModelError::UnknownEntry(entry))?;
    if state.players[i].cue.is_none()
        && !force
        && crate::cue_on_air::ask(state, CueStart::Entry { player: id, entry })
    {
        return Ok(());
    }
    state.players[i].cue = Some(CueState {
        entry,
        paused: false,
    });
    state.players[i].cue_normalization = Some(request.normalization);
    out.push(EngineAction::StartCue {
        player: id,
        request,
    });
    Ok(())
}

/// On-air pre-listen spec CA4: starts `start` even on a sounding on-air
/// output, or only confirms it when it already runs.
fn force_cue(
    state: &mut AppState,
    start: CueStart,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    match start {
        CueStart::Entry { player, entry } => {
            let i = state.player_index(player)?;
            if state.players[i].cue.is_none_or(|c| c.entry != entry) {
                cue_entry(state, player, entry, true, out)?;
            }
        }
        CueStart::Cart(cart) => {
            state
                .cartwall
                .cart(cart)
                .ok_or(ModelError::UnknownCart(cart))?;
            if state.cartwall.cue != Some(cart) {
                cart_rules::cue(state, cart, true, out)?;
            }
        }
    }
    crate::cue_on_air::forced(state, start.holder());
    Ok(())
}

/// Spec O17: a running CUE follows the player's new next, from its cue-in.
/// The entry already cued is left running (no restart), and an entry that
/// cannot be played leaves the CUE where it is.
fn follow_cue(
    state: &mut AppState,
    id: PlayerId,
    entry: EntryId,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let i = state.player_index(id)?;
    let moves = state.players[i].cue.is_some_and(|c| c.entry != entry);
    if moves
        && state
            .playable_request(entry, Holder::PlayerCue(id))
            .is_some()
    {
        cue_entry(state, id, entry, false, out)?;
    }
    Ok(())
}

/// Spec O12: seeks the running CUE inside its file. The model does not
/// touch `paused`: a held CUE stays held at the new position.
fn seek_cue(
    state: &mut AppState,
    id: PlayerId,
    secs: f64,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let i = state.player_index(id)?;
    let Some(cue) = state.players[i].cue else {
        return Ok(());
    };
    if !secs.is_finite() {
        return Ok(());
    }
    if let Some(request) = state.request_at(cue.entry, secs, Holder::PlayerCue(id)) {
        out.push(EngineAction::SeekCue {
            player: id,
            secs: request.from_secs,
        });
    }
    Ok(())
}

/// Spec O12: holds or releases the running CUE; only a change is sent.
fn set_cue_paused(
    state: &mut AppState,
    id: PlayerId,
    paused: bool,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let i = state.player_index(id)?;
    if let Some(cue) = state.players[i].cue.as_mut()
        && cue.paused != paused
    {
        cue.paused = paused;
        out.push(EngineAction::SetCuePaused { player: id, paused });
    }
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
    let use_markers = state.config.players.use_cue_markers;
    let t = state
        .library
        .get(track)
        .ok_or(ModelError::UnknownTrack(track))?;
    // Cue points are validated against the kept markers; intro, outro and
    // MIX against the effective play range.
    let cue_in = t.cue_in_secs();
    let cue_out = t.known_cue_out_secs();
    let range = t.play_range(use_markers);
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
            _ => v
                .max(range.cue_in)
                .min(range.known_end().unwrap_or(f64::INFINITY)),
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

/// Takes a new configuration, validated. The settings of a device no route
/// names stay until the next start (`fp-store` drops them on load):
/// Settings applies every route click at once, so swapping two devices
/// unroutes one for a moment. The player count only changes
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
    if config.outputs != state.config.outputs || config.tuning != state.config.tuning {
        out.push(EngineAction::UpdateSettings(Box::new(config.clone())));
    }
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
    // RL3: removed players take their shares and hidden flags with them;
    // new players are shown; with no panel left shown, all are.
    let players = state.player_ids();
    let _ = crate::layout::settle_layout(&mut state.layout, &players);
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
    let holder = Holder::PlayerMain(state.players[i].id);
    let request = state.request_from_cue_in(target, holder)?;
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
    player.dsd = None;
    player.on_air = Some(request.normalization);
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
    player.fade_ms = 0;
    player.fade_stop_pending = false;
    player.stop_after_current = false;
    player.dsd = None;
    player.on_air = None;
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

/// R26 without the stop-after-current test: the current entry would repeat
/// (O38 uses it to allow stop-after-current in Single mode).
pub(crate) fn repeat_entry(state: &AppState, player: &PlayerState) -> bool {
    player.transport != Transport::Stopped
        && !player.fade_stop_pending
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

/// R26: the player's current entry repeats (and R27, stop-after-current or
/// a fade stop do not end it first).
fn repeating(state: &AppState, player: &PlayerState) -> bool {
    !player.stop_after_current && repeat_entry(state, player)
}

/// The entry to preload: the current one while it repeats, else the next
/// (with a self-next, O37, that is the current entry too).
fn preload_target(state: &AppState, player: &PlayerState) -> Option<EntryId> {
    if repeating(state, player) {
        player.current
    } else {
        player.next
    }
}

/// Derives the engine work implied by the state: preload whatever is next.
pub(crate) fn reconcile(state: &mut AppState, out: &mut Vec<EngineAction>) {
    // BF2 (bit-perfect rate and fader spec): a locked fader is at exactly
    // 100 % after every command and event.
    #[allow(clippy::float_cmp)] // exactly unity is the locked level
    let locked: Vec<usize> = state
        .players
        .iter()
        .enumerate()
        .filter(|(_, p)| p.volume != 1.0 && crate::bit_perfect::fader_locked(state, p.id))
        .map(|(i, _)| i)
        .collect();
    for i in locked {
        let player = &mut state.players[i];
        player.volume = 1.0;
        out.push(EngineAction::SetVolume {
            player: player.id,
            volume: 1.0,
        });
    }
    // Rule 3a (Q8.4): a pending start lives only while its player is
    // stopped with that entry next, inside the entry's play range.
    crate::pending_start::settle(state);
    // N11: a gain record lives as long as its source.
    for p in &mut state.players {
        if p.current.is_none() {
            p.on_air = None;
        }
        if p.cue.is_none() {
            p.cue_normalization = None;
        }
    }
    if state.cartwall.cue.is_none() {
        state.cartwall.cue_normalization = None;
    }
    // O38: in Single mode the flag lives only while the current entry
    // repeats (it may have stopped applying: repeat off, a stop-after mark,
    // Next into another entry, a file that cannot be read).
    let stale: Vec<usize> = state
        .players
        .iter()
        .enumerate()
        .filter(|(_, p)| {
            p.mode == PlayMode::Single && p.stop_after_current && !repeat_entry(state, p)
        })
        .map(|(i, _)| i)
        .collect();
    for i in stale {
        state.players[i].stop_after_current = false;
    }
    let preloads: Vec<(usize, Option<SourceRequest>)> = state
        .players
        .iter()
        .enumerate()
        .map(|(i, p)| {
            (
                i,
                preload_target(state, p)
                    .and_then(|e| crate::pending_start::preload_request(state, p, e)),
            )
        })
        .filter(|(i, request)| {
            let wanted = request
                .as_ref()
                .map(|r| (r.entry, r.from_secs, r.normalization));
            state.players[*i].preloaded != wanted
        })
        .collect();
    for (i, request) in preloads {
        let player = &mut state.players[i];
        player.preloaded = request
            .as_ref()
            .map(|r| (r.entry, r.from_secs, r.normalization));
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
