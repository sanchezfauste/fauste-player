//! The as-run log (automation spec AU36): every scheduled item and filler
//! track that aired, and every scheduled item that was missed. The model
//! keeps yesterday's and today's entries; the store keeps the record for
//! `automation.as_run_keep_days` (AU38). Each entry has a sequence number
//! that only grows, so the store appends exactly the entries it has not
//! saved, whatever the clock did.

use std::collections::{BTreeSet, VecDeque};

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

use crate::automation::ItemRef;
use crate::ids::{EntryId, TrackId};
use crate::player::Transport;
use crate::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AsRunSource {
    Schedule,
    Filler,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AsRunOutcome {
    Aired,
    Missed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AsRunEntry {
    pub seq: u64,
    /// The scheduled item's date, or the day a filler track aired.
    pub date: NaiveDate,
    /// The block's start (local time); `None` for the filler.
    pub scheduled: Option<NaiveDateTime>,
    /// When it started (local time); `None` when missed.
    pub started: Option<NaiveDateTime>,
    pub track: Option<TrackId>,
    pub item: Option<ItemRef>,
    pub source: AsRunSource,
    pub outcome: AsRunOutcome,
}

/// The entries of yesterday and today, and what the log saw last.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AsRunLog {
    entries: VecDeque<AsRunEntry>,
    next_seq: u64,
    seen_schedule: Option<EntryId>,
    seen_filler: Option<EntryId>,
    seen_missed: BTreeSet<ItemRef>,
}

impl AsRunLog {
    pub fn entries(&self) -> impl Iterator<Item = &AsRunEntry> {
        self.entries.iter()
    }

    /// The entries from sequence `seq` on.
    pub fn since(&self, seq: u64) -> impl Iterator<Item = &AsRunEntry> {
        self.entries.iter().filter(move |e| e.seq >= seq)
    }

    pub fn next_seq(&self) -> u64 {
        self.next_seq
    }

    /// Numbers new entries from `next_seq` on (the store's next number
    /// after a restart); never goes back.
    pub fn resume_at(&mut self, next_seq: u64) {
        self.next_seq = self.next_seq.max(next_seq);
    }

    fn push(&mut self, mut entry: AsRunEntry) {
        entry.seq = self.next_seq;
        self.next_seq = self.next_seq.saturating_add(1);
        self.entries.push_back(entry);
    }
}

/// The row on air of `player` while it plays or is paused: a pause and
/// its resume are one airing.
fn playing(state: &AppState, player: crate::ids::PlayerId) -> Option<EntryId> {
    state
        .players
        .iter()
        .find(|p| p.id == player)
        .filter(|p| p.transport != Transport::Stopped)
        .and_then(|p| p.current)
}

/// AU36: records what started on the two automation players and what was
/// newly missed since the last look; drops entries before yesterday.
pub(crate) fn record(state: &mut AppState) {
    let now = state.local_now;
    let today = now.date();
    let block_start = |item: ItemRef| crate::automation::block_start(state, item.date, item.block);
    let mut new = Vec::new();
    if let Some(a) = state.automation.as_ref() {
        let current = playing(state, a.player);
        if current != state.as_run.seen_schedule
            && let Some(entry) = current
            && let Some(item) = a.rows.get(&entry).copied()
        {
            new.push(AsRunEntry {
                seq: 0,
                date: item.date,
                scheduled: block_start(item),
                started: Some(now),
                track: state.track_for_entry(entry).map(|t| t.id),
                item: Some(item),
                source: AsRunSource::Schedule,
                outcome: AsRunOutcome::Aired,
            });
        }
        for item in a.missed.difference(&state.as_run.seen_missed) {
            new.push(AsRunEntry {
                seq: 0,
                date: item.date,
                scheduled: block_start(*item),
                started: None,
                track: None,
                item: Some(*item),
                source: AsRunSource::Schedule,
                outcome: AsRunOutcome::Missed,
            });
        }
    }
    let schedule_now = state
        .automation
        .as_ref()
        .and_then(|a| playing(state, a.player));
    let missed_now = state
        .automation
        .as_ref()
        .map(|a| a.missed.clone())
        .unwrap_or_default();
    let filler_now = state.filler.as_ref().and_then(|f| playing(state, f.player));
    if filler_now.is_some() && filler_now != state.as_run.seen_filler {
        new.push(AsRunEntry {
            seq: 0,
            date: today,
            scheduled: None,
            started: Some(now),
            track: filler_now
                .and_then(|e| state.track_for_entry(e))
                .map(|t| t.id),
            item: None,
            source: AsRunSource::Filler,
            outcome: AsRunOutcome::Aired,
        });
    }
    let log = &mut state.as_run;
    for entry in new {
        log.push(entry);
    }
    log.seen_schedule = schedule_now;
    log.seen_filler = filler_now;
    log.seen_missed = missed_now;
    let yesterday = today.pred_opt().unwrap_or(today);
    while log.entries.front().is_some_and(|e| e.date < yesterday) {
        log.entries.pop_front();
    }
}
