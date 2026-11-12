//! Running the schedule (automation spec AU15–AU24, AU40): the schedule's
//! own player, the running state, next, and the per-tick rules. The model
//! never reads the clock: the conductor sets `AppState::local_now` and
//! `wall_now` and calls [`tick`].

use std::collections::{BTreeSet, HashMap, HashSet};

use chrono::{Days, NaiveDate, NaiveDateTime, TimeDelta};

use crate::command::EngineAction;
use crate::error::ModelError;
use crate::ids::{BlockId, EntryId, PlayerId, PlaylistId, RotationId};
use crate::player::{PlayMode, PlayerState, Transport};
use crate::playlist::{Playlist, PlaylistEntry};
use crate::rotation::StepSource;
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

pub(crate) fn set_next(
    state: &mut AppState,
    item: ItemRef,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let items =
        block_items(state, item.date, item.block).ok_or(ModelError::UnknownBlock(item.block))?;
    if item.index >= items.len() {
        return Err(ModelError::UnknownItem);
    }
    if let Some(a) = state.automation.as_mut() {
        a.next = Some(item);
        a.next_by_hand = true;
    }
    // AU18: while a block plays, a next set by hand follows the item on air.
    if state
        .automation
        .as_ref()
        .is_some_and(|a| a.running && a.on_air.is_some())
    {
        load_block(state, item, false, out);
    }
    Ok(())
}

pub(crate) fn clear_next(state: &mut AppState) -> Result<(), ModelError> {
    if let Some(a) = state.automation.as_mut() {
        a.next_by_hand = false;
    }
    Ok(())
}

/// Where a block of the window starts, in local time.
pub fn block_start(state: &AppState, date: NaiveDate, block: BlockId) -> Option<NaiveDateTime> {
    day_starts(state, date)
        .into_iter()
        .find(|(_, b)| b.id == block)
        .map(|(start, _)| start)
}

fn player_index(state: &AppState, player: PlayerId) -> Option<usize> {
    state.players.iter().position(|p| p.id == player)
}

fn is_playing(state: &AppState, player: Option<PlayerId>) -> bool {
    player
        .and_then(|p| player_index(state, p))
        .and_then(|i| state.players.get(i))
        .is_some_and(|p| p.transport != Transport::Stopped)
}

fn filler_player(state: &AppState) -> Option<PlayerId> {
    state.filler.as_ref().map(|f| f.player)
}

/// AU17, AU18: loads `from`'s block into the schedule's list, from that
/// item on. The rows not yet started are replaced (never the one on air);
/// a track item becomes its track, a category item a track chosen now (§6),
/// and a rotation item ends the rows and is kept for the filler (AU20). An
/// item with nothing playable is missed. The schedule's next is the first
/// new row.
pub fn load_block(
    state: &mut AppState,
    from: ItemRef,
    in_order: bool,
    out: &mut Vec<EngineAction>,
) {
    let Some(a) = state.automation.clone() else {
        return;
    };
    let Some(items) = block_items(state, from.date, from.block) else {
        return;
    };
    let Some(start) = block_start(state, from.date, from.block) else {
        return;
    };
    let Some(i) = player_index(state, a.player) else {
        return;
    };
    let current = state.players.get(i).and_then(|p| p.current);
    let drop: HashSet<EntryId> = state
        .playlists
        .get(a.playlist)
        .map(|l| {
            l.entries
                .iter()
                .map(|e| e.id)
                .filter(|e| Some(*e) != current)
                .collect()
        })
        .unwrap_or_default();
    crate::reducer::take_out_entries(state, &drop, out);
    let mut rows: HashMap<EntryId, ItemRef> = a
        .rows
        .iter()
        .filter(|(e, _)| Some(**e) == current)
        .map(|(e, r)| (*e, *r))
        .collect();
    let mut missed = Vec::new();
    let mut rotation = None;
    let mut first = None;
    let mut planned = Vec::new();
    let mut rng = state.rng.clone();
    for (index, item) in items.iter().enumerate().skip(from.index) {
        let item_ref = ItemRef {
            date: from.date,
            block: from.block,
            index,
        };
        let track = match item {
            Item::Rotation { rotation: r } => {
                rotation = Some(*r);
                break;
            }
            Item::Track { track, .. } => Some(*track).filter(|t| state.library.is_playable(*t)),
            Item::Category { category, .. } => crate::selection::pick(
                state,
                &StepSource::Category(*category),
                state.wall_now,
                &planned,
                &mut rng,
            )
            .map(|p| p.track),
        };
        let Some(track) = track else {
            missed.push(item_ref);
            continue;
        };
        let entry = state.ids.entry();
        if let Some(list) = state.playlists.get_mut(a.playlist) {
            list.entries.push(PlaylistEntry::new(entry, track));
        }
        rows.insert(entry, item_ref);
        planned.push((track, state.wall_now));
        first.get_or_insert(entry);
    }
    state.rng = rng;
    if let Some(p) = state.players.get_mut(i) {
        p.next = first;
        p.next_explicit = first.is_some();
    }
    if let Some(a) = state.automation.as_mut() {
        a.rows = rows;
        a.missed.extend(missed);
        a.started.insert((from.date, from.block));
        a.on_air = Some(BlockRun {
            date: from.date,
            block: from.block,
            start,
            in_order,
            rotation,
        });
        a.next_by_hand = false;
        a.handing_over = false;
    }
}

/// Plays the schedule's next row when its player is stopped; a filler song
/// on air fades out under it.
fn play_schedule(state: &mut AppState, fade_ms: u32, out: &mut Vec<EngineAction>) {
    let Some(player) = state.automation.as_ref().map(|a| a.player) else {
        return;
    };
    if is_playing(state, Some(player)) {
        return;
    }
    let filler = filler_player(state);
    if is_playing(state, filler)
        && let Some(f) = filler
        && let Some(i) = player_index(state, f)
        && let Some(p) = state.players.get_mut(i)
        && !p.fade_stopping()
    {
        p.begin_fade(fade_ms, crate::player::FadeOrigin::FadeStop);
        p.fade_stop_pending = true;
        out.push(EngineAction::FadeOutAndStop { player: f, fade_ms });
    }
    let _ = crate::reducer::play(state, player, out);
}

/// Plays the filler when it is stopped (music until the coming block).
fn play_filler(state: &mut AppState, out: &mut Vec<EngineAction>) {
    let filler = filler_player(state);
    if let Some(f) = filler
        && !is_playing(state, filler)
    {
        let _ = crate::reducer::play(state, f, out);
    }
}

/// The rules of a running schedule, after a start, a command or a tick.
fn drive(state: &mut AppState, out: &mut Vec<EngineAction>) {
    let Some(a) = state.automation.clone() else {
        return;
    };
    let schedule_idle = player_index(state, a.player)
        .and_then(|i| state.players.get(i))
        .is_some_and(|p| p.transport == Transport::Stopped && p.current.is_none());
    // A block ends when its player is idle again.
    if a.on_air.is_some() && schedule_idle {
        if let Some(a) = state.automation.as_mut() {
            a.on_air = None;
            a.rows.clear();
        }
        settle_next(state);
    }
    if !a.running {
        return;
    }
    let Some(a) = state.automation.clone() else {
        return;
    };
    if a.on_air.is_some() {
        return;
    }
    let Some(next) = a.next else {
        play_filler(state, out);
        return;
    };
    if a.next_by_hand {
        load_block(state, next, false, out);
        play_schedule(state, 0, out);
        return;
    }
    let Some(start) = block_start(state, next.date, next.block) else {
        return;
    };
    if start <= state.local_now {
        load_block(state, next, true, out);
        play_schedule(state, 0, out);
    } else {
        play_filler(state, out);
    }
}

pub(crate) fn start(state: &mut AppState, out: &mut Vec<EngineAction>) -> Result<(), ModelError> {
    if let Some(a) = state.automation.as_mut() {
        a.running = true;
    }
    drive(state, out);
    Ok(())
}

pub(crate) fn stop_after_current(state: &mut AppState) {
    let Some(a) = state.automation.as_mut() else {
        return;
    };
    a.running = false;
    a.next_by_hand = false;
    a.handing_over = false;
    let players = [Some(a.player), filler_player(state)];
    for player in players.into_iter().flatten() {
        if let Some(i) = player_index(state, player)
            && let Some(p) = state.players.get_mut(i)
            && p.transport != Transport::Stopped
        {
            p.stop_after_current = true;
        }
    }
}

/// One conductor tick: the running schedule's rules at `local_now`.
/// Returns the engine actions and whether the state changed.
pub fn tick(state: &mut AppState) -> (Vec<EngineAction>, bool) {
    let before = state.automation.clone();
    let mut out = Vec::new();
    if before.is_none() {
        return (out, false);
    }
    drive(state, &mut out);
    let second_changed = before
        .as_ref()
        .and_then(|a| a.seen)
        .is_none_or(|seen| seen.and_utc().timestamp() != state.local_now.and_utc().timestamp());
    if !out.is_empty() || second_changed || state.automation != before {
        crate::reducer::settle_all(state, &mut out);
    }
    let changed = !out.is_empty() || state.automation != before;
    (out, changed)
}
