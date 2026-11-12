//! Running the schedule (automation spec AU15–AU24, AU40): the schedule's
//! own player, the running state, next, and the per-tick rules. The model
//! never reads the clock: the conductor sets `AppState::local_now` and
//! `wall_now` and calls [`tick`].

use std::collections::{BTreeSet, HashMap};

use chrono::{NaiveDate, NaiveDateTime};

use crate::command::EngineAction;
use crate::ids::{BlockId, EntryId, PlayerId, PlaylistId, RotationId};
use crate::player::{PlayMode, PlayerState};
use crate::playlist::Playlist;
use crate::state::AppState;

/// One item of a date's schedule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemRef {
    pub date: NaiveDate,
    pub block: BlockId,
    pub index: usize,
}

/// The block the schedule's player is playing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockRun {
    pub date: NaiveDate,
    pub block: BlockId,
    /// The block's own start (date and time of day).
    pub start: NaiveDateTime,
    /// It started at its time or was reached in order (AU21): a block whose
    /// start passes while it plays is delayed, not missed.
    pub in_order: bool,
    /// Its rotation item, played by the filler after it (AU20).
    pub rotation: Option<RotationId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AutomationState {
    /// The schedule's player (the Schedule column) and its list.
    pub player: PlayerId,
    pub playlist: PlaylistId,
    /// AU15: stopped after every start; only the operator's Start runs it.
    pub running: bool,
    /// AU16: the next item, from the clock or set by hand.
    pub next: Option<ItemRef>,
    pub next_by_hand: bool,
    pub on_air: Option<BlockRun>,
    /// The item each row of the schedule's list plays.
    pub rows: HashMap<EntryId, ItemRef>,
    /// Blocks that started (yesterday, today, tomorrow): never played again
    /// by the clock.
    pub started: BTreeSet<(NaiveDate, BlockId)>,
    /// AU23: items that did not air (yesterday, today, tomorrow).
    pub missed: BTreeSet<ItemRef>,
    /// The filler is fading out for the coming block (AU24).
    pub handing_over: bool,
}

/// Creates the schedule's player (stopped, hidden, just before the
/// filler's) and its empty list when there is none; returns the engine
/// actions.
pub fn ensure(state: &mut AppState, playlist_name: &str) -> Vec<EngineAction> {
    if state.automation.is_some() {
        return Vec::new();
    }
    let playlist = state.ids.playlist();
    state.playlists.add(Playlist::new(playlist, playlist_name));
    let player = state.ids.player();
    let at = state.first_automation_index();
    state
        .players
        .insert(at, PlayerState::new(player, playlist, PlayMode::Continuous));
    if !state.layout.hidden_players.contains(&player) {
        state.layout.hidden_players.push(player);
    }
    state.automation = Some(AutomationState {
        player,
        playlist,
        running: false,
        next: None,
        next_by_hand: false,
        on_air: None,
        rows: HashMap::new(),
        started: BTreeSet::new(),
        missed: BTreeSet::new(),
        handing_over: false,
    });
    vec![EngineAction::AddPlayer { player }]
}
