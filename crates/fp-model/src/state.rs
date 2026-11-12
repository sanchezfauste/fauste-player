use crate::cartwall::{CartPage, Cartwall};
use crate::command::SourceRequest;
use crate::config::Config;
use crate::error::ModelError;
use crate::ids::{EntryId, IdGen, PlayerId};
use crate::live::Holder;
use crate::player::PlayerState;
use crate::playlist::{Playlist, Playlists};
use crate::track::{Library, Track};

/// The complete authoritative state, owned by the conductor.
#[derive(Debug, Clone, PartialEq)]
pub struct AppState {
    pub config: Config,
    pub library: Library,
    /// The tree of track categories.
    pub categories: crate::category::Categories,
    pub playlists: Playlists,
    /// Players in display order.
    pub players: Vec<PlayerState>,
    pub cartwall: Cartwall,
    pub ids: IdGen,
    /// What the engine runs with (live settings spec); runtime only.
    pub live: crate::live::LiveSettings,
    /// Pre-listen on an on-air output: the confirmation request, the
    /// notice and the confirmed pre-listens (on-air pre-listen spec CA3-CA5);
    /// runtime only.
    pub cue_on_air: crate::cue_on_air::CueOnAir,
    /// The main window's arrangement (resizable layout spec RL1), saved
    /// with the session.
    pub layout: crate::layout::Layout,
    /// The library window's state, saved with the session.
    pub library_view: crate::library_view::LibraryView,
    /// The library window pre-listen (library plan 4); runtime only.
    pub library_cue: crate::library_cue::LibraryCue,
    /// The music filler (automation spec AU27–AU29): its player is the
    /// last of `players`. `None` until `filler::ensure`.
    pub filler: Option<crate::filler::FillerState>,
    /// Running the schedule (automation spec AU15–AU24): its player is just
    /// before the filler's. `None` until `automation::ensure`.
    pub automation: Option<crate::automation::AutomationState>,
    /// The music filler's rotations (automation spec AU1–AU4), saved.
    pub rotations: crate::rotation::Rotations,
    /// When each track last aired (automation spec AU35), saved.
    pub air_times: crate::air_times::AirTimes,
    /// The schedule's templates, week plan and date plans (automation spec
    /// AU9–AU12), saved.
    pub schedule: crate::schedule::Schedule,
    /// The wall clock, set by the conductor every tick (by tests directly);
    /// runtime only.
    pub wall_now: crate::wall_time::WallTime,
    /// The local date and time, set with `wall_now`; the schedule compares
    /// block starts with it. Runtime only.
    pub local_now: chrono::NaiveDateTime,
    /// The random generator of automation's choices, seeded by the
    /// conductor; runtime only.
    pub rng: crate::wall_time::SplitMix64,
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
            categories: crate::category::Categories::default(),
            playlists,
            players,
            cartwall: Cartwall {
                pages: vec![page],
                ..Cartwall::default()
            },
            ids,
            live: crate::live::LiveSettings::default(),
            cue_on_air: crate::cue_on_air::CueOnAir::default(),
            layout: crate::layout::Layout::default(),
            library_view: crate::library_view::LibraryView::default(),
            library_cue: crate::library_cue::LibraryCue::default(),
            filler: None,
            automation: None,
            rotations: crate::rotation::Rotations::default(),
            air_times: crate::air_times::AirTimes::default(),
            schedule: crate::schedule::Schedule::default(),
            wall_now: crate::wall_time::WallTime::default(),
            local_now: chrono::NaiveDateTime::default(),
            rng: crate::wall_time::SplitMix64::default(),
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

    /// The players the operator numbers P1…Pn: every player but the
    /// automation players (the schedule's and the music filler's).
    pub fn aux_players(&self) -> impl Iterator<Item = &PlayerState> {
        self.players
            .iter()
            .filter(move |p| !self.is_automation(p.id))
    }

    /// The schedule's player (the Schedule column), not the filler.
    pub fn is_schedule(&self, player: PlayerId) -> bool {
        self.automation.as_ref().is_some_and(|a| a.player == player)
    }

    /// The schedule's player or the music filler's: not an operator player.
    pub fn is_automation(&self, player: PlayerId) -> bool {
        self.is_filler(player) || self.automation.as_ref().is_some_and(|a| a.player == player)
    }

    /// The list of an automation player: not an operator playlist.
    pub fn is_automation_playlist(&self, playlist: crate::ids::PlaylistId) -> bool {
        self.is_filler_playlist(playlist)
            || self
                .automation
                .as_ref()
                .is_some_and(|a| a.playlist == playlist)
    }

    /// The list an automation player shows, the only one it may show.
    pub fn automation_list_of(&self, player: PlayerId) -> Option<crate::ids::PlaylistId> {
        if let Some(f) = self.filler.as_ref().filter(|f| f.player == player) {
            return Some(f.playlist);
        }
        self.automation
            .as_ref()
            .filter(|a| a.player == player)
            .map(|a| a.playlist)
    }

    /// Where the automation players start in `players` (they are last).
    pub(crate) fn first_automation_index(&self) -> usize {
        self.players
            .iter()
            .position(|p| self.is_automation(p.id))
            .unwrap_or(self.players.len())
    }

    pub fn is_filler(&self, player: PlayerId) -> bool {
        self.filler.as_ref().is_some_and(|f| f.player == player)
    }

    pub fn is_filler_playlist(&self, playlist: crate::ids::PlaylistId) -> bool {
        self.filler.as_ref().is_some_and(|f| f.playlist == playlist)
    }

    /// The players' ids in display order: what the layout helpers take.
    /// The Schedule column comes first, then the Music column (the filler),
    /// left of P1 (automation spec §8.1); in `players` they are last, so
    /// positional code is unchanged.
    pub fn player_ids(&self) -> Vec<PlayerId> {
        let schedule = self.automation.as_ref().map(|a| a.player);
        let filler = self.filler.as_ref().map(|f| f.player);
        schedule
            .into_iter()
            .chain(filler)
            .chain(self.aux_player_ids())
            .collect()
    }

    /// The auxiliary players' ids, P1 first.
    pub fn aux_player_ids(&self) -> Vec<PlayerId> {
        self.aux_players().map(|p| p.id).collect()
    }

    pub fn track_for_entry(&self, entry: EntryId) -> Option<&Track> {
        let e = self.playlists.entry(entry)?;
        self.library.get(e.track)
    }

    /// As `request_from_cue_in`, only for an entry whose file is not known
    /// to be missing or unreadable.
    pub fn playable_request(&self, entry: EntryId, holder: Holder) -> Option<SourceRequest> {
        let track = self.track_for_entry(entry)?;
        if !track.file_state.is_playable() {
            return None;
        }
        self.request_from_cue_in(entry, holder)
    }

    /// A request that starts the entry at the start of its play range: its
    /// cue-in, or 0 when players ignore cue markers. `holder` is who plays
    /// it: the request carries its normalization (N9).
    pub fn request_from_cue_in(&self, entry: EntryId, holder: Holder) -> Option<SourceRequest> {
        let track = self.track_for_entry(entry)?;
        let range = track.play_range(self.config.players.use_cue_markers);
        Some(SourceRequest {
            entry,
            track: track.id,
            path: track.path.clone(),
            from_secs: range.cue_in,
            format: track.format,
            normalization: crate::loudness::normalization_for(self, holder, track),
        })
    }

    /// A request that starts the entry at `secs`, clamped to the file when its
    /// duration is known; a non-finite `secs` falls back to cue-in.
    /// `holder` is who plays it: the request carries its normalization (N9).
    pub fn request_at(&self, entry: EntryId, secs: f64, holder: Holder) -> Option<SourceRequest> {
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
            normalization: crate::loudness::normalization_for(self, holder, track),
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
