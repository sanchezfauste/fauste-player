use crate::cartwall::{CartPage, Cartwall};
use crate::command::SourceRequest;
use crate::config::Config;
use crate::error::ModelError;
use crate::ids::{EntryId, IdGen, PlayerId};
use crate::player::PlayerState;
use crate::playlist::{Playlist, Playlists};
use crate::track::{Library, Track};

/// The complete authoritative state, owned by the conductor.
#[derive(Debug, Clone, PartialEq)]
pub struct AppState {
    pub config: Config,
    pub library: Library,
    pub playlists: Playlists,
    /// Players in display order.
    pub players: Vec<PlayerState>,
    pub cartwall: Cartwall,
    pub ids: IdGen,
}

impl AppState {
    /// A fresh state: one empty playlist and `config.players.count` stopped players.
    pub fn new(config: Config, default_playlist_name: &str) -> Self {
        let mut ids = IdGen::default();
        let mut playlists = Playlists::default();
        let first = ids.playlist();
        playlists.add(Playlist::new(first, default_playlist_name));
        let mode = config.players.default_mode;
        let players = (0..config.players.count)
            .map(|_| PlayerState::new(ids.player(), first, mode))
            .collect();
        let page = CartPage::new(
            &mut ids,
            "",
            config.cartwall.default_rows,
            config.cartwall.default_cols,
        );
        Self {
            config,
            library: Library::default(),
            playlists,
            players,
            cartwall: Cartwall {
                pages: vec![page],
                ..Cartwall::default()
            },
            ids,
        }
    }

    pub fn player_index(&self, id: PlayerId) -> Result<usize, ModelError> {
        self.players
            .iter()
            .position(|p| p.id == id)
            .ok_or(ModelError::UnknownPlayer(id))
    }

    pub fn player(&self, id: PlayerId) -> Result<&PlayerState, ModelError> {
        self.players
            .iter()
            .find(|p| p.id == id)
            .ok_or(ModelError::UnknownPlayer(id))
    }

    pub fn track_for_entry(&self, entry: EntryId) -> Option<&Track> {
        let e = self.playlists.entry(entry)?;
        self.library.get(e.track)
    }

    /// A request that starts the entry at its cue-in point.
    pub fn request_from_cue_in(&self, entry: EntryId) -> Option<SourceRequest> {
        let track = self.track_for_entry(entry)?;
        Some(SourceRequest {
            entry,
            track: track.id,
            path: track.path.clone(),
            from_secs: track.cue_in_secs(),
        })
    }

    /// A request that starts the entry at `secs`, clamped to the file when its
    /// duration is known; a non-finite `secs` falls back to cue-in.
    pub fn request_at(&self, entry: EntryId, secs: f64) -> Option<SourceRequest> {
        let track = self.track_for_entry(entry)?;
        let from_secs = if secs.is_finite() {
            let end = if track.duration_secs > 0.0 {
                track.duration_secs
            } else {
                f64::INFINITY
            };
            secs.clamp(0.0, end)
        } else {
            track.cue_in_secs()
        };
        Some(SourceRequest {
            entry,
            track: track.id,
            path: track.path.clone(),
            from_secs,
        })
    }

    /// True if `entry` is the current entry of any player.
    pub fn is_on_air(&self, entry: EntryId) -> bool {
        self.players.iter().any(|p| p.current == Some(entry))
    }
}
