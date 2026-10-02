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

    /// As `request_from_cue_in`, only for an entry whose file is not known
    /// to be missing or unreadable.
    pub fn playable_request(&self, entry: EntryId) -> Option<SourceRequest> {
        let track = self.track_for_entry(entry)?;
        if !track.file_state.is_playable() {
            return None;
        }
        self.request_from_cue_in(entry)
    }

    /// A request that starts the entry at the start of its play range: its
    /// cue-in, or 0 when players ignore cue markers.
    pub fn request_from_cue_in(&self, entry: EntryId) -> Option<SourceRequest> {
        let track = self.track_for_entry(entry)?;
        let range = track.play_range(self.config.players.use_cue_markers);
        Some(SourceRequest {
            entry,
            track: track.id,
            path: track.path.clone(),
            from_secs: range.cue_in,
            format: track.format,
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
            track.play_range(self.config.players.use_cue_markers).cue_in
        };
        Some(SourceRequest {
            entry,
            track: track.id,
            path: track.path.clone(),
            from_secs,
            format: track.format,
        })
    }

    /// Turns the shared `played` flags of files written before players were
    /// independent into marks for every player (spec §3 rule 22).
    pub fn normalize_played_marks(&mut self) {
        let players: Vec<PlayerId> = self.players.iter().map(|p| p.id).collect();
        self.playlists.convert_legacy_played(&players);
    }

    /// True if `entry` is the current entry of any player.
    pub fn is_on_air(&self, entry: EntryId) -> bool {
        self.players.iter().any(|p| p.current == Some(entry))
    }
}
