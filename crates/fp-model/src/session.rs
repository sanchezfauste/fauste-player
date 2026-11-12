//! Per-player session snapshot (for `session.json`) and crash-recovery restore.

use serde::{Deserialize, Serialize};

use crate::cartwall::{CartPage, Cartwall, CartwallSession};
use crate::command::EngineAction;
use crate::config::Config;
use crate::ids::{EntryId, IdGen, PlayerId, PlaylistId};
use crate::layout::Layout;
use crate::player::{ColumnWidths, PlayMode, PlayerState, Transport};
use crate::playlist::{Playlist, Playlists};
use crate::reducer::{fill_empty_next, reconcile};
use crate::state::AppState;
use crate::track::Library;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerSession {
    pub id: PlayerId,
    pub playlist: PlaylistId,
    #[serde(default)]
    pub current: Option<EntryId>,
    #[serde(default)]
    pub next: Option<EntryId>,
    #[serde(default)]
    pub next_explicit: bool,
    #[serde(default)]
    pub mode: PlayMode,
    #[serde(default)]
    pub stop_after_current: bool,
    #[serde(default)]
    pub position_secs: f64,
    #[serde(default = "full_volume")]
    pub volume: f32,
    #[serde(default, deserialize_with = "lenient_columns")]
    pub columns: ColumnWidths,
    /// Entries the player left, oldest first (R25).
    #[serde(default, deserialize_with = "lenient_history")]
    pub history: Vec<EntryId>,
    /// Rule 3a (operator feedback 4, Q8.8): where Play starts the next,
    /// kept only while that entry is still the next.
    #[serde(default, deserialize_with = "lenient_pending_start")]
    pub pending_start: Option<(EntryId, f64)>,
}

/// Column widths that do not parse load as the default layout.
fn lenient_columns<'de, D: serde::Deserializer<'de>>(d: D) -> Result<ColumnWidths, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Lenient {
        Widths(ColumnWidths),
        Other(serde::de::IgnoredAny),
    }
    Ok(match Lenient::deserialize(d)? {
        Lenient::Widths(w) => w.normalized(),
        Lenient::Other(_) => ColumnWidths::default(),
    })
}

/// A history that does not parse loads as empty: it only feeds Previous.
fn lenient_history<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<EntryId>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Lenient {
        Entries(Vec<EntryId>),
        Other(serde::de::IgnoredAny),
    }
    Ok(match Lenient::deserialize(d)? {
        Lenient::Entries(entries) => entries,
        Lenient::Other(_) => Vec::new(),
    })
}

/// A pending start that does not parse loads as none: it only changes
/// where Play starts.
fn lenient_pending_start<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Option<(EntryId, f64)>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Lenient {
        Valid(Option<(EntryId, f64)>),
        Other(serde::de::IgnoredAny),
    }
    Ok(match Lenient::deserialize(d)? {
        Lenient::Valid(v) => v.filter(|(_, secs)| secs.is_finite()),
        Lenient::Other(_) => None,
    })
}

fn full_volume() -> f32 {
    1.0
}

/// Everything loaded from disk except the per-player sessions.
#[derive(Debug, Clone, Default)]
pub struct RestoreParts {
    pub config: Config,
    pub library: Library,
    pub categories: crate::category::Categories,
    pub playlists: Playlists,
    /// Cart pages (their carts reference `library` tracks).
    pub cart_pages: Vec<CartPage>,
    pub cartwall_session: CartwallSession,
    /// The saved layout (resizable layout spec RL19); `restore` repairs it
    /// for the players it builds.
    pub layout: Layout,
    /// The saved library window state; `restore` repairs it for the categories.
    pub library_view: crate::library_view::LibraryView,
    pub ids: IdGen,
    /// The music filler's rotations (automation spec AU1–AU4).
    pub rotations: crate::rotation::Rotations,
    /// When each track last aired (AU35); times of tracks not in `library`
    /// are dropped.
    pub air_times: crate::air_times::AirTimes,
    /// The schedule (automation spec AU9–AU12).
    pub schedule: crate::schedule::Schedule,
    /// The music filler as saved (AU27); restored last, not playing.
    pub filler: Option<crate::filler::FillerSession>,
}

impl AppState {
    /// The auxiliary players' sessions; the music filler is saved apart.
    pub fn sessions(&self, position_of: impl Fn(PlayerId) -> f64) -> Vec<PlayerSession> {
        self.aux_players()
            .map(|p| session_of(p, &position_of))
            .collect()
    }

    /// Rebuilds the state after a start or a crash. Nothing is ever restored
    /// as playing: nothing goes on air by itself (spec §7).
    pub fn restore(
        parts: RestoreParts,
        sessions: &[PlayerSession],
        default_playlist_name: &str,
    ) -> (AppState, Vec<EngineAction>) {
        let RestoreParts {
            config,
            library,
            categories,
            mut playlists,
            cart_pages,
            cartwall_session,
            layout,
            library_view,
            mut ids,
            rotations,
            mut air_times,
            schedule,
            filler,
        } = parts;
        ids.observe(library.max_raw_id());
        ids.observe(schedule.max_raw_id());
        ids.observe(rotations.max_raw_id());
        air_times.retain_tracks(|t| library.get(t).is_some());
        ids.observe(categories.max_raw_id());
        ids.observe(playlists.max_raw_id());
        for s in sessions {
            ids.observe(s.id.0);
        }
        let mut cartwall = Cartwall {
            pages: cart_pages,
            open: cartwall_session.open,
            ..Cartwall::default()
        };
        cartwall.shown = cartwall_session
            .page
            .filter(|p| cartwall.page(*p).is_some());
        ids.observe(cartwall.max_raw_id());
        let _ = cartwall.normalize(&config.limits, &mut ids, &library);
        if cartwall.pages.is_empty() {
            cartwall.pages.push(CartPage::new(
                &mut ids,
                "",
                config.cartwall.default_rows,
                config.cartwall.default_cols,
            ));
        }
        // After every id is observed, so a new one cannot collide.
        let _ = playlists.normalize(&mut ids);
        if playlists.is_empty() {
            playlists.add(Playlist::new(ids.playlist(), default_playlist_name));
        }
        let mut state = AppState {
            config,
            library,
            categories,
            playlists,
            players: Vec::new(),
            cartwall,
            ids,
            live: crate::live::LiveSettings::default(),
            cue_on_air: crate::cue_on_air::CueOnAir::default(),
            library_cue: crate::library_cue::LibraryCue::default(),
            layout: Layout::default(),
            library_view: crate::library_view::LibraryView::default(),
            filler: None,
            rotations,
            air_times,
            schedule,
            wall_now: crate::wall_time::WallTime::default(),
            rng: crate::wall_time::SplitMix64::default(),
        };
        // One track per file (LB14): a library saved before that rule, or
        // by a buggy build, may hold several. Unreferenced tracks stay (LB15).
        let _ = crate::library_rules::merge_duplicate_paths(&mut state);
        let mut out = Vec::new();
        for k in 0..state.config.players.count {
            let player = match sessions.get(k) {
                Some(session) => restore_player(&mut state, session),
                None => new_player(&mut state),
            };
            state.players.push(player);
        }
        restore_filler(&mut state, filler);
        // RL19: the saved layout, repaired for the players this start has
        // (`fp-store` reports the repairs).
        state.layout = layout;
        state.library_view = library_view.repaired(&state.categories);
        let players = state.player_ids();
        let _ = crate::layout::settle_layout(&mut state.layout, &players);
        // N13: the gain of each source loaded paused, computed now from the
        // current configuration and track.
        let mut loaded = Vec::new();
        for p in &state.players {
            // The engine starts every player at full volume; send the restored one.
            out.push(EngineAction::SetVolume {
                player: p.id,
                volume: p.volume,
            });
            if let Some(current) = p.current {
                let saved = sessions
                    .iter()
                    .find(|s| s.id == p.id)
                    .map_or(0.0, |s| s.position_secs);
                // A position at or past the end of the play range would end the
                // track the moment Play resumes it: it comes back at the start
                // of the range.
                let position = match state.track_for_entry(current) {
                    Some(t) => {
                        let range = t.play_range(state.config.players.use_cue_markers);
                        if range.known_end().is_some_and(|out| saved >= out) {
                            range.cue_in
                        } else {
                            saved
                        }
                    }
                    None => saved,
                };
                if let Some(request) =
                    state.request_at(current, position, crate::live::Holder::PlayerMain(p.id))
                {
                    loaded.push((p.id, request.normalization));
                    out.push(EngineAction::LoadPaused {
                        player: p.id,
                        request,
                    });
                }
            }
        }
        for (id, normalization) in loaded {
            if let Ok(i) = state.player_index(id) {
                state.players[i].on_air = Some(normalization);
            }
        }
        state.normalize_played_marks();
        fill_empty_next(&mut state);
        reconcile(&mut state, &mut out);
        (state, out)
    }
}

/// What is saved of `p`.
pub(crate) fn session_of(p: &PlayerState, position_of: &impl Fn(PlayerId) -> f64) -> PlayerSession {
    PlayerSession {
        id: p.id,
        playlist: p.playlist,
        current: p.current,
        next: p.next,
        next_explicit: p.next_explicit,
        mode: p.mode,
        stop_after_current: p.stop_after_current,
        position_secs: if p.current.is_some() {
            position_of(p.id)
        } else {
            0.0
        },
        volume: p.volume,
        columns: p.columns.clone(),
        history: p.history.clone(),
        pending_start: p.pending_start,
    }
}

/// AU27: the saved filler, last of the players. A record whose playlist is
/// gone, or whose player id is already taken, is dropped: `filler::ensure`
/// then makes a new one.
fn restore_filler(state: &mut AppState, saved: Option<crate::filler::FillerSession>) {
    let Some(saved) = saved else {
        return;
    };
    let Some(list) = state.playlists.get(saved.playlist) else {
        return;
    };
    if state.players.iter().any(|p| p.id == saved.player.id) {
        return;
    }
    let rows: std::collections::HashSet<EntryId> = list.entries.iter().map(|e| e.id).collect();
    let mut player = restore_player(state, &saved.player);
    player.playlist = saved.playlist;
    player.mode = PlayMode::Continuous;
    state.ids.observe(player.id.0);
    state.players.push(player.clone());
    state.filler = Some(crate::filler::FillerState {
        player: player.id,
        playlist: saved.playlist,
        rotation: saved.rotation.filter(|r| state.rotations.get(*r).is_some()),
        cursor: saved.cursor,
        generated: saved
            .generated
            .into_iter()
            .filter(|e| rows.contains(e))
            .collect(),
        starved: false,
    });
}

fn first_playlist(state: &AppState) -> PlaylistId {
    // `restore` guarantees at least one playlist.
    state.playlists.first_id().unwrap_or(PlaylistId(0))
}

fn new_player(state: &mut AppState) -> PlayerState {
    let id = state.ids.player();
    PlayerState::new(id, first_playlist(state), state.config.players.default_mode)
}

fn restore_player(state: &mut AppState, s: &PlayerSession) -> PlayerState {
    let id = if state.players.iter().any(|p| p.id == s.id) {
        state.ids.player()
    } else {
        s.id
    };
    let playlist = if state.playlists.get(s.playlist).is_some() {
        s.playlist
    } else {
        first_playlist(state)
    };
    // A current entry must still resolve to a track, or it could never be resumed.
    let current = s.current.filter(|e| state.track_for_entry(*e).is_some());
    // An explicit next may be the current entry (O37: it plays once more).
    let kept_next = s.next.filter(|e| {
        state.playlists.entry(*e).is_some() && (Some(*e) != current || s.next_explicit)
    });
    let next_explicit = s.next_explicit && kept_next.is_some();
    let next = kept_next
        .or_else(|| current.and_then(|c| state.playlists.next_playable_after(c, &state.library)));
    let volume = if s.volume.is_finite() && (0.0..=1.0).contains(&s.volume) {
        s.volume
    } else {
        1.0
    };
    let mut player = PlayerState::new(id, playlist, s.mode);
    player.current = current;
    player.next = next;
    player.next_explicit = next_explicit;
    // Q8.8: only on a stopped player, for its next; `reconcile` clamps it
    // to the play range (`pending_start::settle`).
    player.pending_start = s
        .pending_start
        .filter(|(e, secs)| current.is_none() && Some(*e) == next && secs.is_finite());
    player.transport = if current.is_some() {
        Transport::Paused
    } else {
        Transport::Stopped
    };
    player.stop_after_current = s.stop_after_current
        && (s.mode == PlayMode::Continuous || crate::reducer::repeat_entry(state, &player));
    player.volume = volume;
    player.columns = s.columns.clone().normalized();
    // Only entries that still exist, and no more than the configured depth.
    let kept: Vec<EntryId> = s
        .history
        .iter()
        .copied()
        .filter(|e| state.playlists.entry(*e).is_some())
        .collect();
    let skip = kept.len().saturating_sub(state.config.players.history_len);
    player.history = kept.into_iter().skip(skip).collect();
    player
}
