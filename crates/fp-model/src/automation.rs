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
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
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
    /// The filler is fading out for this coming item (AU24); it counts only
    /// while that item is still next and its block is within the fade.
    pub handing_over: Option<ItemRef>,
    /// Automation set the filler's stop-after-current so that a next set
    /// by hand follows its song; it is released if that next goes away.
    pub filler_held: bool,
    /// The latest `local_now` the rules have seen: only blocks whose start
    /// passes after it, while the application runs, are marked missed.
    /// `None` before the first look.
    pub seen: Option<NaiveDateTime>,
    /// The schedule's row on air and when it started (local time): when the
    /// next row would start, for late time-critical items (AU22).
    pub row_since: Option<(EntryId, NaiveDateTime)>,
    /// The item of the Log's row last seen on air (runtime only): where
    /// the row below is when automation stops (AR10).
    pub last_on_air: Option<ItemRef>,
    /// AR6: the Log's Stop at end (runtime only). What is on air for the
    /// Log ends, then automation stops; a block that comes first takes it
    /// over (its first row stops automation).
    pub stop_at_end: bool,
    /// The Log's Pause paused the filler's song playing for the Log
    /// (runtime only): the Log's Play resumes it (AR5).
    pub paused_fill: bool,
    /// The filler's song on air for the Log carried the operator's own
    /// Stop at end at the last look (runtime only): when the filler then
    /// stops with no hand-over or hold pending, automation stops too (AR7).
    pub fill_on_air: bool,
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
        handing_over: None,
        filler_held: false,
        seen: None,
        row_since: None,
        last_on_air: None,
        stop_at_end: false,
        paused_fill: false,
        fill_on_air: false,
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

/// A block the clock no longer plays: it started, or its first item was
/// missed (AU23: the operator may still choose it by hand).
fn passed_over(state: &AppState, date: NaiveDate, block: BlockId) -> bool {
    started(state, date, block)
        || state.automation.as_ref().is_some_and(|a| {
            a.missed.contains(&ItemRef {
                date,
                block,
                index: 0,
            })
        })
}

/// AU16: the first item of the first block that has not started, whose
/// start is ahead of `local_now` or less than the late tolerance behind.
pub fn clock_next(state: &AppState) -> Option<ItemRef> {
    let earliest = state.local_now - tolerance(state);
    window_blocks(state)
        .into_iter()
        .find(|(date, start, b)| {
            *start > earliest && !b.items.is_empty() && !passed_over(state, *date, b.id)
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
    // AU21: a block whose start passes while automation is stopped, or
    // while a block chosen away from its time plays, is missed; one that
    // passes while a block in order plays waits for it.
    // Between blocks while running, a start passes over only when the
    // clock jumped (AU40).
    let marking = !a.running || a.on_air.is_none_or(|b| !b.in_order);
    // A next set by hand whose block or item is gone (an edit, or its date
    // left the window at midnight) gives next back to the clock.
    let by_hand = a.next_by_hand
        && a.next.is_some_and(|n| {
            block_items(state, n.date, n.block).is_some_and(|items| n.index < items.len())
        });
    let next = if by_hand { a.next } else { clock_next(state) };
    // A look from before the window (a first look, or a clock that jumped
    // days) marks nothing.
    let [yesterday, ..] = window(state);
    let seen = a.seen.filter(|s| s.date() >= yesterday);
    let earliest = state.local_now - tolerance(state);
    let passed: Vec<ItemRef> = match seen {
        Some(seen) if marking => window_blocks(state)
            .into_iter()
            .filter(|(date, start, b)| {
                // The block's last chance (start + tolerance) passed since
                // the last look.
                *start + tolerance(state) > seen
                    && *start <= earliest
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
    let on_air_item = player_index(state, a.player)
        .and_then(|i| state.players.get(i))
        .filter(|p| p.transport != Transport::Stopped)
        .and_then(|p| p.current)
        .and_then(|e| a.rows.get(&e).copied());
    let Some(a) = state.automation.as_mut() else {
        return;
    };
    a.next = next;
    a.next_by_hand = by_hand;
    if let Some(item) = on_air_item {
        a.last_on_air = Some(item);
    }
    a.seen = Some(state.local_now);
    a.missed.extend(passed);
    a.missed.retain(|i| i.date >= yesterday);
    a.started.retain(|(d, _)| *d >= yesterday);
}

/// The row below `item` in the Log (AR9, AR10): the next item of its
/// block (a rotation item included), else the first item of the next
/// block of the window in time order.
pub fn row_below(state: &AppState, item: ItemRef) -> Option<ItemRef> {
    let len = block_items(state, item.date, item.block)?.len();
    if item.index + 1 < len {
        return Some(ItemRef {
            index: item.index + 1,
            ..item
        });
    }
    let blocks = window_blocks(state);
    let at = blocks
        .iter()
        .position(|(d, _, b)| *d == item.date && b.id == item.block)?;
    blocks
        .iter()
        .skip(at + 1)
        .find(|(_, _, b)| !b.items.is_empty())
        .map(|(date, _, b)| ItemRef {
            date: *date,
            block: b.id,
            index: 0,
        })
}

/// AR10: automation stopped; next stays on the row below what was on air
/// (the Log's loaded next row, else the row below the last item on air,
/// else the next it had), pinned until the operator chooses another.
fn pin_next_after_stop(state: &mut AppState) {
    let Some(a) = state.automation.as_ref() else {
        return;
    };
    let loaded = player_index(state, a.player)
        .and_then(|i| state.players.get(i))
        .and_then(|p| p.next)
        .and_then(|e| a.rows.get(&e).copied());
    let below = loaded.or_else(|| a.last_on_air.and_then(|i| row_below(state, i)));
    if let Some(a) = state.automation.as_mut() {
        if let Some(item) = below {
            a.next = Some(item);
        }
        a.next_by_hand = a.next.is_some();
    }
}

fn transport_of(state: &AppState, player: Option<PlayerId>) -> Transport {
    player
        .and_then(|p| player_index(state, p))
        .and_then(|i| state.players.get(i))
        .map_or(Transport::Stopped, |p| p.transport)
}

/// The music fill is what is on air for the Log (AR16): automation runs,
/// no block is on air, and the filler plays.
pub fn fill_is_current(state: &AppState) -> bool {
    state
        .automation
        .as_ref()
        .is_some_and(|a| a.running && a.on_air.is_none())
        && transport_of(state, filler_player(state)) != Transport::Stopped
}

/// AR4: Play on the Log. A paused Log row, or a filler song the Log
/// paused, resumes; a row on air goes to next as on a player; else
/// automation starts from next.
pub(crate) fn log_play(
    state: &mut AppState,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let Some(a) = state.automation.clone() else {
        return Ok(());
    };
    let log = transport_of(state, Some(a.player));
    let filler = filler_player(state);
    if log == Transport::Paused || (a.running && log == Transport::Playing) {
        crate::reducer::play(state, a.player, out)?;
    } else if a.paused_fill && transport_of(state, filler) == Transport::Paused {
        if let Some(f) = filler {
            crate::reducer::play(state, f, out)?;
        }
    } else if log_is_single(state) {
        return single_play(state, out);
    } else {
        return start(state, out);
    }
    if let Some(a) = state.automation.as_mut() {
        a.running = true;
        a.paused_fill = false;
    }
    Ok(())
}

/// AR6b: the Log in SINGLE mode is manual.
fn log_is_single(state: &AppState) -> bool {
    state
        .automation
        .as_ref()
        .and_then(|a| player_index(state, a.player))
        .and_then(|i| state.players.get(i))
        .is_some_and(|p| p.mode == PlayMode::Single)
}

/// AR6b: Play on a Log in SINGLE plays next at once; a music fill row
/// starts the filler with its rotation and automation stays stopped.
fn single_play(state: &mut AppState, out: &mut Vec<EngineAction>) -> Result<(), ModelError> {
    let Some(next) = state.automation.as_ref().and_then(|a| a.next) else {
        return Ok(());
    };
    let item = block_items(state, next.date, next.block).and_then(|i| i.get(next.index).cloned());
    if let Some(Item::Rotation { rotation }) = item {
        crate::filler::set_rotation(state, Some(rotation), out)?;
        crate::filler::settle(state, out);
        play_filler(state, out);
        let below = row_below(state, next);
        if let Some(a) = state.automation.as_mut() {
            a.next = below.or(a.next);
            a.next_by_hand = a.next.is_some();
        }
        return Ok(());
    }
    if let Some(a) = state.automation.as_mut() {
        a.next_by_hand = true;
    }
    start(state, out)
}

/// AR5: Pause on the Log pauses what is on air for it and stops
/// automation; the row stays current.
pub(crate) fn log_pause(
    state: &mut AppState,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let Some(a) = state.automation.clone() else {
        return Ok(());
    };
    let fill = fill_is_current(state);
    if transport_of(state, Some(a.player)) == Transport::Playing {
        crate::reducer::apply_pause(state, a.player, out)?;
    } else if fill && let Some(f) = filler_player(state) {
        crate::reducer::apply_pause(state, f, out)?;
    }
    if let Some(a) = state.automation.as_mut() {
        a.running = false;
        a.stop_at_end = false;
        a.handing_over = None;
        a.paused_fill = fill;
    }
    Ok(())
}

/// AR7: Pause on the filler while its song is on air for the Log stops
/// automation too; the Log's Play resumes it.
pub(crate) fn filler_pause(
    state: &mut AppState,
    filler: PlayerId,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let fill = fill_is_current(state);
    crate::reducer::apply_pause(state, filler, out)?;
    if fill && let Some(a) = state.automation.as_mut() {
        a.running = false;
        a.stop_at_end = false;
        a.handing_over = None;
        a.fill_on_air = false;
        a.paused_fill = true;
    }
    Ok(())
}

/// AR5: Stop or Fade stop on the Log stops what is on air for it, and
/// automation; next is pinned on the row below (AR10).
pub(crate) fn log_stop(
    state: &mut AppState,
    fade: bool,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let Some(a) = state.automation.clone() else {
        return Ok(());
    };
    let target = if transport_of(state, Some(a.player)) != Transport::Stopped {
        Some(a.player)
    } else if fill_is_current(state) || a.paused_fill {
        filler_player(state)
    } else {
        None
    };
    if a.running || a.paused_fill || target == Some(a.player) {
        halt(state);
    }
    if let Some(a) = state.automation.as_mut() {
        a.paused_fill = false;
    }
    match target {
        Some(p) if fade => crate::reducer::apply_fade_stop(state, p, out),
        Some(p) => crate::reducer::apply_stop(state, p, out),
        None => Ok(()),
    }
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
        let _ = load_block(state, item, false, out);
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
/// Returns false (and changes nothing) when `from` is not an item of the
/// window any more.
pub fn load_block(
    state: &mut AppState,
    from: ItemRef,
    in_order: bool,
    out: &mut Vec<EngineAction>,
) -> bool {
    let Some(a) = state.automation.clone() else {
        return false;
    };
    let Some(items) = block_items(state, from.date, from.block) else {
        return false;
    };
    let Some(start) = block_start(state, from.date, from.block) else {
        return false;
    };
    let Some(i) = player_index(state, a.player) else {
        return false;
    };
    let current = state.players.get(i).and_then(|p| p.current);
    let current_block = current
        .and_then(|c| a.rows.get(&c))
        .map(|r| (r.date, r.block));
    // AU23: the rows of the block on air that will not play now were
    // skipped, and so were the items before `from` of a block that never
    // started.
    let mut missed: Vec<ItemRef> = a
        .rows
        .iter()
        .filter(|(e, r)| Some(**e) != current && current_block == Some((r.date, r.block)))
        .map(|(_, r)| *r)
        .collect();
    if !a.started.contains(&(from.date, from.block)) {
        missed.extend((0..from.index).map(|index| ItemRef {
            date: from.date,
            block: from.block,
            index,
        }));
    }
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
        // AU22: a time-critical item that would start beyond the late
        // tolerance is missed.
        // (AU18: a block played by hand plays in full.)
        if in_order && item.is_time_critical(state) && state.local_now > start + tolerance(state) {
            missed.push(item_ref);
            continue;
        }
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
    // The rows are replaced: a block loaded before but never on air (no
    // row of it is the current one) has not started after all.
    if let Some(a) = state.automation.as_mut() {
        if let Some(prev) = a.on_air
            && (prev.date, prev.block) != (from.date, from.block)
            && current_block != Some((prev.date, prev.block))
        {
            a.started.remove(&(prev.date, prev.block));
        }
        // An item queued now airs after all (AU23: a missed item played
        // by hand).
        for queued in rows.values() {
            a.missed.remove(queued);
        }
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
        a.handing_over = None;
    }
    true
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
    fade_filler(state, fade_ms, out);
    let _ = crate::reducer::play(state, player, out);
    // AR6: a Stop at end set during the music moves to the block's row.
    if state.automation.as_ref().is_some_and(|a| a.stop_at_end) {
        set_stop_mark(state, Some(player), true);
    }
}

/// Sets or clears `stop_after_current` on `player` while it plays.
fn set_stop_mark(state: &mut AppState, player: Option<PlayerId>, on: bool) {
    if let Some(i) = player.and_then(|p| player_index(state, p))
        && let Some(p) = state.players.get_mut(i)
        && p.transport != Transport::Stopped
    {
        p.stop_after_current = on;
    }
}

/// AR6: the Log's Stop at end, on or off; nothing while stopped. The mark
/// goes on the Log's row on air, else on the filler's song playing for the
/// Log.
pub(crate) fn toggle_stop_at_end(state: &mut AppState) {
    let Some(a) = state.automation.as_ref() else {
        return;
    };
    if !a.running {
        return;
    }
    let on = !a.stop_at_end;
    let log = a.player;
    let block_on_air = a.on_air.is_some();
    if let Some(a) = state.automation.as_mut() {
        a.stop_at_end = on;
    }
    if block_on_air {
        set_stop_mark(state, Some(log), on);
    } else {
        let hold = state.automation.as_ref().is_some_and(|a| a.filler_held);
        set_stop_mark(state, filler_player(state), on || hold);
    }
}

/// Automation stops (AR3): not running, no Stop at end, next pinned
/// (AR10).
fn halt(state: &mut AppState) {
    pin_next_after_stop(state);
    if let Some(a) = state.automation.as_mut() {
        a.running = false;
        a.stop_at_end = false;
        a.handing_over = None;
        a.filler_held = false;
        a.fill_on_air = false;
    }
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

/// A filler song on air fades out over `fade_ms` and stops.
fn fade_filler(state: &mut AppState, fade_ms: u32, out: &mut Vec<EngineAction>) {
    let filler = filler_player(state);
    if let Some(f) = filler
        && is_playing(state, filler)
        && !player_index(state, f)
            .and_then(|i| state.players.get(i))
            .is_some_and(|p| p.fade_stop_pending)
    {
        let _ = crate::reducer::fade_stop_over(state, f, fade_ms, out);
    }
}

/// The track length of a schedule row, from its play range.
fn row_secs(state: &AppState, entry: EntryId) -> f64 {
    state
        .track_for_entry(entry)
        .and_then(|t| {
            t.play_range(state.config.players.use_cue_markers)
                .known_end()
                .map(|end| end - t.play_range(state.config.players.use_cue_markers).cue_in)
        })
        .unwrap_or(0.0)
}

/// AU22: before the schedule's player moves on, a next row that is
/// time-critical and would start beyond its block's start plus the late
/// tolerance is taken out and marked missed. Nothing on air is cut.
fn skip_late_critical(state: &mut AppState, out: &mut Vec<EngineAction>) {
    let Some(a) = state.automation.clone() else {
        return;
    };
    let Some(run) = a.on_air.filter(|r| r.in_order) else {
        return;
    };
    let Some(p) = player_index(state, a.player).and_then(|i| state.players.get(i)) else {
        return;
    };
    let (Some(current), Some(next)) = (p.current, p.next) else {
        return;
    };
    // When the row on air started: the first tick that saw it.
    let since = match a.row_since {
        Some((e, at)) if e == current => at,
        _ => {
            if let Some(a) = state.automation.as_mut() {
                a.row_since = Some((current, state.local_now));
            }
            state.local_now
        }
    };
    let Some(item) = a.rows.get(&next).copied() else {
        return;
    };
    let critical = block_items(state, item.date, item.block)
        .and_then(|items| items.get(item.index).cloned())
        .is_some_and(|i| i.is_time_critical(state));
    if !critical {
        return;
    }
    let ends = since + TimeDelta::milliseconds((row_secs(state, current) * 1000.0) as i64);
    let starts = ends.max(state.local_now);
    if starts > run.start + tolerance(state) {
        crate::reducer::take_out_entries(state, &HashSet::from([next]), out);
        if let Some(a) = state.automation.as_mut() {
            a.rows.remove(&next);
            a.missed.insert(item);
        }
    }
}

/// AU21: after a block in order, the earliest block that started while it
/// played (and has not started) plays now, in order.
fn overdue_after(state: &AppState, run: &BlockRun) -> Option<ItemRef> {
    window_blocks(state)
        .into_iter()
        .find(|(date, start, b)| {
            *start > run.start
                && *start <= state.local_now
                && !b.items.is_empty()
                && !passed_over(state, *date, b.id)
        })
        .map(|(date, _, b)| ItemRef {
            date,
            block: b.id,
            index: 0,
        })
}

/// The rules of a running schedule, after a start, a command or a tick.
/// `starting`: the operator pressed Start, and a next set by hand plays at
/// once (AU17); otherwise it follows the filler song on air.
fn drive(state: &mut AppState, out: &mut Vec<EngineAction>, starting: bool) {
    let Some(a) = state.automation.clone() else {
        return;
    };
    let schedule_idle = player_index(state, a.player)
        .and_then(|i| state.players.get(i))
        .is_some_and(|p| p.transport == Transport::Stopped && p.current.is_none());
    // A block ends when its player is idle again; its rows leave the list,
    // so nothing can pick an aired row up again.
    let mut ended = None;
    if let Some(run) = a.on_air
        && schedule_idle
    {
        if !a.running && !a.next_by_hand {
            pin_next_after_stop(state);
        } else if a.running && (a.stop_at_end || log_is_single(state)) {
            halt(state);
        }
        let rows: HashSet<EntryId> = state
            .playlists
            .get(a.playlist)
            .map(|l| l.entries.iter().map(|e| e.id).collect())
            .unwrap_or_default();
        crate::reducer::take_out_entries(state, &rows, out);
        if let Some(a) = state.automation.as_mut() {
            a.on_air = None;
            a.rows.clear();
            a.row_since = None;
            a.last_on_air = None;
        }
        ended = Some(run);
        if state.automation.as_ref().is_some_and(|a| a.running) {
            // AU20: the filler plays the block's rotation, or the default.
            let current = state.filler.as_ref().map(|f| f.rotation);
            if current.is_some_and(|r| r != run.rotation) {
                let _ = crate::filler::set_rotation(state, run.rotation, out);
            }
        }
        settle_next(state);
    }
    if !state.automation.as_ref().is_some_and(|a| a.running) {
        return;
    }
    skip_late_critical(state, out);
    let Some(a) = state.automation.clone() else {
        return;
    };
    // The filler held for a next set by hand that went away goes on.
    if a.filler_held && !a.next_by_hand {
        set_filler_hold(state, false);
    }
    if a.on_air.is_some() {
        return;
    }
    // AR6, AR7: the filler's song for the Log ended under Stop at end, or
    // the operator stopped the filler, before any hand-over or hold:
    // automation stops.
    let filler_stopped = !is_playing(state, filler_player(state));
    if filler_stopped
        && a.handing_over.is_none()
        && !a.filler_held
        && (a.stop_at_end || a.fill_on_air)
    {
        halt(state);
        return;
    }
    let fade = state.config.automation.block_fade_ms;
    if !a.next_by_hand
        && let Some(run) = ended.filter(|r| r.in_order)
        && let Some(item) = overdue_after(state, &run)
        && load_block(state, item, true, out)
    {
        play_schedule(state, 0, out);
        return;
    }
    let Some(next) = a.next else {
        play_filler(state, out);
        return;
    };
    let filler_on = is_playing(state, filler_player(state));
    if a.next_by_hand {
        if starting || !filler_on {
            if load_block(state, next, false, out) {
                set_filler_hold(state, false);
                play_schedule(state, fade, out);
            }
        } else {
            set_filler_hold(state, true);
        }
        return;
    }
    let Some(start) = block_start(state, next.date, next.block) else {
        play_filler(state, out);
        return;
    };
    if start <= state.local_now {
        if load_block(state, next, true, out) {
            play_schedule(state, 0, out);
        }
        return;
    }
    // AU24: the filler fades out so that the block starts on time. A
    // hand-over counts only for the item still next, within the fade: after
    // an edit or a jump the music comes back.
    let lead_ms = (start - state.local_now).num_milliseconds();
    let handing = a.handing_over == Some(next) && lead_ms <= i64::from(fade);
    if a.handing_over.is_some()
        && !handing
        && let Some(a) = state.automation.as_mut()
    {
        a.handing_over = None;
    }
    if filler_on {
        if !handing && lead_ms <= i64::from(fade) {
            let fade_ms = u32::try_from(lead_ms.max(0)).unwrap_or(fade).min(fade);
            fade_filler(state, fade_ms, out);
            if let Some(a) = state.automation.as_mut() {
                a.handing_over = Some(next);
            }
        }
    } else if !handing {
        play_filler(state, out);
    }
}

/// Holds the filler (stop after its song) for a next set by hand, or
/// releases a hold automation set.
fn set_filler_hold(state: &mut AppState, hold: bool) {
    let held = state.automation.as_ref().is_some_and(|a| a.filler_held);
    if hold == held {
        return;
    }
    if let Some(f) = filler_player(state)
        && let Some(i) = player_index(state, f)
        && let Some(p) = state.players.get_mut(i)
        && p.transport != Transport::Stopped
    {
        p.stop_after_current = hold;
    }
    if let Some(a) = state.automation.as_mut() {
        a.filler_held = hold;
    }
}

/// Remembers whether the filler's song on air for the Log carries the
/// operator's own Stop at end (AR7); automation's marks and fades do not
/// count.
pub(crate) fn note_fill(state: &mut AppState) {
    let fill = fill_is_current(state);
    let marked = filler_player(state)
        .and_then(|f| player_index(state, f))
        .and_then(|i| state.players.get(i))
        .is_some_and(|p| p.stop_after_current && !p.fade_stopping());
    if let Some(a) = state.automation.as_mut() {
        a.fill_on_air = fill && marked && !a.stop_at_end && !a.filler_held;
    }
}

/// AR7: Stop or Fade stop on the filler while its song is on air for the
/// Log stops automation too.
pub(crate) fn filler_stop(
    state: &mut AppState,
    filler: PlayerId,
    fade: bool,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    if fill_is_current(state) {
        halt(state);
    }
    if fade {
        crate::reducer::apply_fade_stop(state, filler, out)
    } else {
        crate::reducer::apply_stop(state, filler, out)
    }
}

pub(crate) fn start(state: &mut AppState, out: &mut Vec<EngineAction>) -> Result<(), ModelError> {
    let Some(a) = state.automation.as_mut() else {
        return Ok(());
    };
    a.running = true;
    a.stop_at_end = false;
    a.paused_fill = false;
    a.last_on_air = None;
    // Start after Stop after current: what was to stop goes on.
    let players = [Some(a.player), filler_player(state)];
    for player in players.into_iter().flatten() {
        if let Some(i) = player_index(state, player)
            && let Some(p) = state.players.get_mut(i)
        {
            p.stop_after_current = false;
        }
    }
    drive(state, out, true);
    note_fill(state);
    Ok(())
}

/// `StopAutomationAfterCurrent`: the Log's Stop at end on (AR6).
pub(crate) fn stop_after_current(state: &mut AppState) {
    if state
        .automation
        .as_ref()
        .is_some_and(|a| a.running && !a.stop_at_end)
    {
        toggle_stop_at_end(state);
    }
}

/// A step of the local clock larger than this, or any step back, is a
/// jump (AU40). It tells a jump from a late tick; it is not an operator
/// value.
pub const CLOCK_JUMP_SECS: i64 = 60;

/// One conductor tick: the running schedule's rules at `local_now`.
/// Returns the engine actions and whether the state changed.
pub fn tick(state: &mut AppState) -> (Vec<EngineAction>, bool) {
    let before = state.automation.clone();
    let mut out = Vec::new();
    if before.is_none() {
        return (out, false);
    }
    // AU40: after a jump, next is recomputed (and the blocks jumped over
    // are missed) before anything plays.
    let jumped = before.as_ref().and_then(|a| a.seen).is_some_and(|seen| {
        let step = (state.local_now - seen).num_seconds();
        !(0..=CLOCK_JUMP_SECS).contains(&step)
    });
    if jumped {
        if let Some(seen) = before.as_ref().and_then(|a| a.seen) {
            miss_jumped_over(state, seen);
        }
        settle_next(state);
    }
    drive(state, &mut out, false);
    note_fill(state);
    let second_changed = before
        .as_ref()
        .and_then(|a| a.seen)
        .is_none_or(|seen| seen.and_utc().timestamp() != state.local_now.and_utc().timestamp());
    if !out.is_empty() || second_changed || state.automation != before {
        crate::reducer::settle_all(state, &mut out);
    }
    // The clock bookkeeping (`seen`, `row_since`) is not a change anyone
    // shows or saves.
    let changed = !out.is_empty() || !same_but_clock(state.automation.as_ref(), before.as_ref());
    (out, changed)
}

fn same_but_clock(a: Option<&AutomationState>, b: Option<&AutomationState>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => {
            let mut a = a.clone();
            a.seen = b.seen;
            a.row_since = b.row_since;
            a == *b
        }
        (a, b) => a.is_none() && b.is_none(),
    }
}

/// AU40: after a forward jump, the blocks whose start the jump passed
/// over (after `seen`, up to now) are missed, whatever plays: they never
/// play, not even in order after a block that overran.
fn miss_jumped_over(state: &mut AppState, seen: NaiveDateTime) {
    // A look from before the window (a first look) marks nothing.
    let [yesterday, ..] = window(state);
    if state.local_now <= seen || seen.date() < yesterday {
        return;
    }
    let skipped: Vec<ItemRef> = window_blocks(state)
        .into_iter()
        .filter(|(date, start, b)| {
            *start > seen && *start <= state.local_now && !started(state, *date, b.id)
        })
        .flat_map(|(date, _, b)| {
            (0..b.items.len()).map(move |index| ItemRef {
                date,
                block: b.id,
                index,
            })
        })
        .collect();
    if let Some(a) = state.automation.as_mut() {
        a.missed.extend(skipped);
    }
}

/// AU27: while automation runs and no block plays, the start of the coming
/// block: the filler queues music only up to it.
pub fn coming_start(state: &AppState) -> Option<NaiveDateTime> {
    let a = state.automation.as_ref()?;
    if !a.running || a.on_air.is_some() {
        return None;
    }
    let next = a.next?;
    block_start(state, next.date, next.block)
}

/// What is saved of the schedule's player (`kv` value `automation`): its
/// ids, so its outputs and place in the layout survive a restart. Its rows
/// are rebuilt, and running, next and the missed marks are not saved
/// (AU15, AU16).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AutomationSession {
    pub player: crate::session::PlayerSession,
    pub playlist: PlaylistId,
}

impl AppState {
    /// The schedule's player as saved; `position_of` gives its position.
    pub fn automation_session(
        &self,
        position_of: impl Fn(PlayerId) -> f64,
    ) -> Option<AutomationSession> {
        let a = self.automation.as_ref()?;
        let player = self.players.iter().find(|p| p.id == a.player)?;
        Some(AutomationSession {
            player: crate::session::session_of(player, &position_of),
            playlist: a.playlist,
        })
    }
}

/// The saved schedule's player, stopped, with an empty list, appended to
/// the players (before the filler's, restored after it). A record whose
/// list is gone, or whose player id is taken, is dropped: `ensure` then
/// makes a new one.
pub(crate) fn restore(state: &mut AppState, saved: Option<AutomationSession>) {
    let Some(saved) = saved else {
        return;
    };
    if state.players.iter().any(|p| p.id == saved.player.id) {
        return;
    }
    let Some(list) = state.playlists.get_mut(saved.playlist) else {
        return;
    };
    list.entries.clear();
    let mut player = crate::session::restore_player(state, &saved.player);
    player.playlist = saved.playlist;
    player.current = None;
    player.next = None;
    player.pending_start = None;
    state.ids.observe(player.id.0);
    let id = player.id;
    state.players.push(player);
    state.automation = Some(AutomationState {
        player: id,
        playlist: saved.playlist,
        running: false,
        next: None,
        next_by_hand: false,
        on_air: None,
        rows: HashMap::new(),
        started: BTreeSet::new(),
        missed: BTreeSet::new(),
        handing_over: None,
        filler_held: false,
        seen: None,
        row_since: None,
        last_on_air: None,
        stop_at_end: false,
        paused_fill: false,
        fill_on_air: false,
    });
}

/// How a row of the Schedule column stands (§8.1, AU23).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowStatus {
    Aired,
    OnAir,
    Next,
    Missed,
    Coming,
}

/// One row of the Schedule column: a block's header, or one of its items.
#[derive(Debug, Clone, PartialEq)]
pub enum ScheduleRow {
    Block {
        date: NaiveDate,
        block: BlockId,
        start: NaiveDateTime,
        origin: crate::schedule::BlockOrigin,
        status: RowStatus,
    },
    Item {
        item: ItemRef,
        what: crate::schedule::Item,
        /// A track's play range when known; `None` for a category, a
        /// rotation or an unknown length.
        length_secs: Option<f64>,
        time_critical: bool,
        status: RowStatus,
    },
}

/// The Schedule column's rows (§8.1): yesterday's last block, today's
/// blocks and tomorrow's first, each followed by its items, in time order.
pub fn schedule_view(state: &AppState) -> Vec<ScheduleRow> {
    let Some(a) = state.automation.as_ref() else {
        return Vec::new();
    };
    let [yesterday, today, tomorrow] = window(state);
    let all = window_blocks(state);
    let last_of_yesterday = all
        .iter()
        .rposition(|(d, _, _)| *d == yesterday && yesterday != today);
    let first_of_tomorrow = all
        .iter()
        .position(|(d, _, _)| *d == tomorrow && tomorrow != today);
    // AU21: while a block plays in order, the block that came due meanwhile
    // plays after it, whatever the clock says next is.
    let next = match a.on_air {
        Some(run) if run.in_order && !a.next_by_hand => overdue_after(state, &run).or(a.next),
        _ => a.next,
    };
    // A block on air from yesterday (an overrun past midnight) and a next
    // beyond tomorrow's first block stay listed, with every block between.
    let index_of = |date: NaiveDate, block: BlockId| {
        all.iter().position(|(d, _, b)| *d == date && b.id == block)
    };
    let on_air_index = a.on_air.and_then(|run| index_of(run.date, run.block));
    let next_index = next.and_then(|n| index_of(n.date, n.block));
    let today_first = all.iter().position(|(d, _, _)| *d == today);
    let today_last = all.iter().rposition(|(d, _, _)| *d == today);
    let first = [last_of_yesterday, on_air_index, next_index, today_first]
        .into_iter()
        .flatten()
        .min();
    let last = [first_of_tomorrow, on_air_index, next_index, today_last]
        .into_iter()
        .flatten()
        .max();
    let on_air_item = player_index(state, a.player)
        .and_then(|i| state.players.get(i))
        .and_then(|p| p.current)
        .and_then(|e| a.rows.get(&e).copied());
    let mut out = Vec::new();
    for (i, (date, start, block)) in all.iter().enumerate() {
        let shown = first.is_some_and(|f| i >= f) && last.is_some_and(|l| i <= l);
        if !shown {
            continue;
        }
        let on_air = a
            .on_air
            .is_some_and(|run| run.date == *date && run.block == block.id);
        let started = a.started.contains(&(*date, block.id));
        let item_status = |index: usize| {
            let item = ItemRef {
                date: *date,
                block: block.id,
                index,
            };
            if on_air_item == Some(item) {
                RowStatus::OnAir
            } else if next == Some(item) {
                RowStatus::Next
            } else if a.missed.contains(&item) {
                RowStatus::Missed
            } else if on_air {
                match on_air_item {
                    Some(o) if o.block == block.id && o.date == *date && index < o.index => {
                        RowStatus::Aired
                    }
                    _ => RowStatus::Coming,
                }
            } else if started {
                RowStatus::Aired
            } else {
                RowStatus::Coming
            }
        };
        let statuses: Vec<RowStatus> = (0..block.items.len()).map(item_status).collect();
        let status = if on_air {
            RowStatus::OnAir
        } else if next.is_some_and(|n| n.date == *date && n.block == block.id) {
            RowStatus::Next
        } else if !statuses.is_empty() && statuses.iter().all(|s| *s == RowStatus::Missed) {
            RowStatus::Missed
        } else if started {
            RowStatus::Aired
        } else {
            RowStatus::Coming
        };
        out.push(ScheduleRow::Block {
            date: *date,
            block: block.id,
            start: *start,
            origin: block.origin,
            status,
        });
        for (index, what) in block.items.iter().enumerate() {
            let length_secs = match what {
                crate::schedule::Item::Track { track, .. } => {
                    state.library.get(*track).and_then(|t| {
                        let range = t.play_range(state.config.players.use_cue_markers);
                        range.known_end().map(|_| range.length())
                    })
                }
                _ => None,
            };
            out.push(ScheduleRow::Item {
                item: ItemRef {
                    date: *date,
                    block: block.id,
                    index,
                },
                what: what.clone(),
                length_secs,
                time_critical: what.is_time_critical(state),
                status: statuses.get(index).copied().unwrap_or(RowStatus::Coming),
            });
        }
    }
    out
}
