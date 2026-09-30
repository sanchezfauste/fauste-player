//! Per-player session snapshot (for `session.json`) and crash-recovery restore.

use serde::{Deserialize, Serialize};

use crate::cartwall::{CartPage, Cartwall, CartwallSession};
use crate::command::EngineAction;
use crate::config::Config;
use crate::ids::{EntryId, IdGen, PlayerId, PlaylistId};
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
    #[serde(default)]
    pub columns: ColumnWidths,
    /// Entries the player left, oldest first (R25).
    #[serde(default, deserialize_with = "lenient_history")]
    pub history: Vec<EntryId>,
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

fn full_volume() -> f32 {
    1.0
}

/// Everything loaded from disk except the per-player sessions.
#[derive(Debug, Clone, Default)]
pub struct RestoreParts {
    pub config: Config,
    pub library: Library,
    pub playlists: Playlists,
    /// Cart pages (their carts reference `library` tracks).
    pub cart_pages: Vec<CartPage>,
    pub cartwall_session: CartwallSession,
    pub ids: IdGen,
}

impl AppState {
    pub fn sessions(&self, position_of: impl Fn(PlayerId) -> f64) -> Vec<PlayerSession> {
        self.players
            .iter()
            .map(|p| PlayerSession {
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
                columns: p.columns,
                history: p.history.clone(),
            })
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
            mut playlists,
            cart_pages,
            cartwall_session,
            mut ids,
        } = parts;
        ids.observe(library.max_raw_id());
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
            playlists,
            players: Vec::new(),
            cartwall,
            ids,
        };
        // Tracks nothing references (left by an interrupted save) are dropped.
        let orphans: Vec<_> = state
            .library
            .iter()
            .map(|t| t.id)
            .filter(|t| {
                !state.playlists.references_track(*t) && !state.cartwall.references_track(*t)
            })
            .collect();
        for track in orphans {
            state.library.remove(track);
        }
        let mut out = Vec::new();
        for k in 0..state.config.players.count {
            let player = match sessions.get(k) {
                Some(session) => restore_player(&mut state, session),
                None => new_player(&mut state),
            };
            state.players.push(player);
        }
        for p in &state.players {
            // The engine starts every player at full volume; send the restored one.
            out.push(EngineAction::SetVolume {
                player: p.id,
                volume: p.volume,
            });
            if let Some(current) = p.current {
                let position = sessions
                    .iter()
                    .find(|s| s.id == p.id)
                    .map_or(0.0, |s| s.position_secs);
                if let Some(request) = state.request_at(current, position) {
                    out.push(EngineAction::LoadPaused {
                        player: p.id,
                        request,
                    });
                }
            }
        }
        state.normalize_played_marks();
        fill_empty_next(&mut state);
        reconcile(&mut state, &mut out);
        (state, out)
    }
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
    let kept_next = s
        .next
        .filter(|e| state.playlists.entry(*e).is_some() && Some(*e) != current);
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
    player.transport = if current.is_some() {
        Transport::Paused
    } else {
        Transport::Stopped
    };
    player.stop_after_current = s.mode == PlayMode::Continuous && s.stop_after_current;
    player.volume = volume;
    player.columns = s.columns;
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
