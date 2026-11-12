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
    /// The schedule's row on air and when it started (local time): when the
    /// next row would start, for late time-critical items (AU22).
    pub row_since: Option<(EntryId, NaiveDateTime)>,
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
        row_since: None,
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
    // AU21: a block whose start passes while automation is stopped, or
    // while a block chosen away from its time plays, is missed; one that
    // passes while a block in order plays waits for it.
    // Between blocks while running, a start passes over only when the
    // clock jumped (AU40).
    let marking = !a.running || a.on_air.is_none_or(|b| !b.in_order);
    let by_hand = a.next_by_hand;
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
        // AU22: a time-critical item that would start beyond the late
        // tolerance is missed.
        if item.is_time_critical(state) && state.local_now > start + tolerance(state) {
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
    fade_filler(state, fade_ms, out);
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
    let Some(run) = a.on_air else {
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
                && !started(state, *date, b.id)
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
    // A block ends when its player is idle again.
    let mut ended = None;
    if let Some(run) = a.on_air
        && schedule_idle
    {
        if let Some(a) = state.automation.as_mut() {
            a.on_air = None;
            a.rows.clear();
            a.row_since = None;
        }
        ended = Some(run);
        if a.running {
            // AU20: the filler plays the block's rotation, or the default.
            let current = state.filler.as_ref().map(|f| f.rotation);
            if current.is_some_and(|r| r != run.rotation) {
                let _ = crate::filler::set_rotation(state, run.rotation, out);
            }
        }
        settle_next(state);
    }
    if !a.running {
        return;
    }
    skip_late_critical(state, out);
    let Some(a) = state.automation.clone() else {
        return;
    };
    if a.on_air.is_some() {
        return;
    }
    let fade = state.config.automation.block_fade_ms;
    if !a.next_by_hand
        && let Some(run) = ended.filter(|r| r.in_order)
        && let Some(item) = overdue_after(state, &run)
    {
        load_block(state, item, true, out);
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
            load_block(state, next, false, out);
            play_schedule(state, fade, out);
        } else if let Some(f) = filler_player(state)
            && let Some(i) = player_index(state, f)
            && let Some(p) = state.players.get_mut(i)
        {
            p.stop_after_current = true;
        }
        return;
    }
    let Some(start) = block_start(state, next.date, next.block) else {
        return;
    };
    if start <= state.local_now {
        load_block(state, next, true, out);
        play_schedule(state, 0, out);
        return;
    }
    // AU24: the filler fades out so that the block starts on time.
    let lead_ms = (start - state.local_now).num_milliseconds();
    if filler_on {
        if !a.handing_over && lead_ms <= i64::from(fade) {
            let fade_ms = u32::try_from(lead_ms.max(0)).unwrap_or(fade).min(fade);
            fade_filler(state, fade_ms, out);
            if let Some(a) = state.automation.as_mut() {
                a.handing_over = true;
            }
        }
    } else if !a.handing_over {
        play_filler(state, out);
    }
}

pub(crate) fn start(state: &mut AppState, out: &mut Vec<EngineAction>) -> Result<(), ModelError> {
    if let Some(a) = state.automation.as_mut() {
        a.running = true;
    }
    drive(state, out, true);
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
        settle_next(state);
    }
    drive(state, &mut out, false);
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
    player.mode = PlayMode::Continuous;
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
        handing_over: false,
        seen: None,
        row_since: None,
    });
}
