//! Running the schedule (automation spec AU15–AU24, AU40): the schedule's
//! own player, the running state, next, and the per-tick rules. The model
//! never reads the clock: the conductor sets `AppState::local_now` and
//! `wall_now` and calls [`tick`].

use std::collections::{BTreeSet, HashMap};

use chrono::{Days, NaiveDate, NaiveDateTime, TimeDelta};

use crate::command::EngineAction;
use crate::error::ModelError;
use crate::ids::{BlockId, EntryId, PlayerId, PlaylistId, RotationId};
use crate::player::{PlayMode, PlayerState};
use crate::playlist::Playlist;
use crate::schedule::{DayBlock, Item};
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
    /// The latest `local_now` the rules have seen: only blocks whose start
    /// passes after it, while the application runs, are marked missed.
    /// `None` before the first look.
    pub seen: Option<NaiveDateTime>,
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
        seen: None,
    });
    vec![EngineAction::AddPlayer { player }]
}

/// The dates the schedule runs over: yesterday, today and tomorrow of
/// `local_now`.
fn window(state: &AppState) -> [NaiveDate; 3] {
    let today = state.local_now.date();
    [
        today.checked_sub_days(Days::new(1)).unwrap_or(today),
        today,
        today.checked_add_days(Days::new(1)).unwrap_or(today),
    ]
}

/// A date's blocks with their local start, in time order.
fn day_starts(state: &AppState, date: NaiveDate) -> Vec<(NaiveDateTime, DayBlock)> {
    state
        .schedule
        .day(date)
        .into_iter()
        .filter_map(|b| {
            let time = chrono::NaiveTime::from_num_seconds_from_midnight_opt(b.start.secs(), 0)?;
            Some((date.and_time(time), b))
        })
        .collect()
}

/// Every block of the window, in time order (yesterday's blocks before
/// today's, across midnight).
fn window_blocks(state: &AppState) -> Vec<(NaiveDate, NaiveDateTime, DayBlock)> {
    window(state)
        .into_iter()
        .flat_map(|d| {
            day_starts(state, d)
                .into_iter()
                .map(move |(start, b)| (d, start, b))
        })
        .collect()
}

fn tolerance(state: &AppState) -> TimeDelta {
    TimeDelta::seconds(i64::from(state.config.automation.late_tolerance_secs))
}

fn started(state: &AppState, date: NaiveDate, block: BlockId) -> bool {
    state
        .automation
        .as_ref()
        .is_some_and(|a| a.started.contains(&(date, block)))
}

/// AU16: the first item of the first block that has not started, whose
/// start is ahead of `local_now` or less than the late tolerance behind.
pub fn clock_next(state: &AppState) -> Option<ItemRef> {
    let earliest = state.local_now - tolerance(state);
    window_blocks(state)
        .into_iter()
        .find(|(date, start, b)| {
            *start >= earliest && !b.items.is_empty() && !started(state, *date, b.id)
        })
        .map(|(date, _, b)| ItemRef {
            date,
            block: b.id,
            index: 0,
        })
}

/// The items of a block of the window, `None` for a block not in it.
pub fn block_items(state: &AppState, date: NaiveDate, block: BlockId) -> Option<Vec<Item>> {
    if !window(state).contains(&date) {
        return None;
    }
    state
        .schedule
        .day(date)
        .into_iter()
        .find(|b| b.id == block)
        .map(|b| b.items)
}

/// After every command and event: next follows the clock unless it was set
/// by hand (AU16), and while stopped the blocks the clock passed over are
/// missed (AU21, AU23). Marks of dates before yesterday are dropped.
pub fn settle_next(state: &mut AppState) {
    let Some(a) = state.automation.as_ref() else {
        return;
    };
    let running = a.running;
    let by_hand = a.next_by_hand;
    let next = if by_hand { a.next } else { clock_next(state) };
    // A look from before the window (a first look, or a clock that jumped
    // days) marks nothing.
    let [yesterday, ..] = window(state);
    let seen = a.seen.filter(|s| s.date() >= yesterday);
    let earliest = state.local_now - tolerance(state);
    let passed: Vec<ItemRef> = match seen {
        Some(seen) if !running => window_blocks(state)
            .into_iter()
            .filter(|(date, start, b)| {
                *start + tolerance(state) > seen
                    && *start < earliest
                    && !started(state, *date, b.id)
            })
            .flat_map(|(date, _, b)| {
                (0..b.items.len()).map(move |index| ItemRef {
                    date,
                    block: b.id,
                    index,
                })
            })
            .collect(),
        _ => Vec::new(),
    };
    let Some(a) = state.automation.as_mut() else {
        return;
    };
    a.next = next;
    a.seen = Some(state.local_now);
    a.missed.extend(passed);
    a.missed.retain(|i| i.date >= yesterday);
    a.started.retain(|(d, _)| *d >= yesterday);
}

pub(crate) fn set_next(state: &mut AppState, item: ItemRef) -> Result<(), ModelError> {
    let items =
        block_items(state, item.date, item.block).ok_or(ModelError::UnknownBlock(item.block))?;
    if item.index >= items.len() {
        return Err(ModelError::UnknownItem);
    }
    if let Some(a) = state.automation.as_mut() {
        a.next = Some(item);
        a.next_by_hand = true;
    }
    Ok(())
}

pub(crate) fn clear_next(state: &mut AppState) -> Result<(), ModelError> {
    if let Some(a) = state.automation.as_mut() {
        a.next_by_hand = false;
    }
    Ok(())
}
