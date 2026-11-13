//! The music filler (automation spec AU27–AU29): a player of its own,
//! kept last in `AppState::players`, that fills its own playlist from a
//! rotation and trims what it played. The playlist is never listed with
//! the operator's playlists.

use std::collections::HashSet;

use crate::command::EngineAction;
use crate::ids::{EntryId, PlayerId, PlaylistId, RotationId};
use crate::player::{PlayMode, PlayerState};
use crate::playlist::Playlist;
use crate::rotation_cursor::RotationCursor;
use crate::state::AppState;
use chrono::NaiveDateTime;

#[derive(Debug, Clone, PartialEq)]
pub struct FillerState {
    pub player: PlayerId,
    pub playlist: PlaylistId,
    /// The rotation it plays; `None` plays the default (AU4).
    pub rotation: Option<RotationId>,
    pub cursor: RotationCursor,
    /// Rows the filler added and may replace (AU28); every other row of
    /// the playlist was added or moved by the operator.
    pub generated: HashSet<EntryId>,
    /// The last top-up found nothing to play (AU34 alert); runtime only.
    pub starved: bool,
    /// The row on air and when the model first saw it there (AU25: when
    /// the rows ahead will start); runtime only.
    pub current_since: Option<(EntryId, NaiveDateTime)>,
    /// How many picks before a block found no track that fits and took
    /// the one that cuts least (AU25); runtime only.
    pub unfitted: u64,
    /// The row on air the queue was last fitted for (AU25); runtime only.
    pub fitted_for: Option<EntryId>,
}

/// Creates the filler (a stopped player last in `players` and an empty
/// playlist named `playlist_name`) when there is none; returns the engine
/// actions.
pub fn ensure(state: &mut AppState, playlist_name: &str) -> Vec<EngineAction> {
    if state.filler.is_some() {
        return Vec::new();
    }
    let playlist = state.ids.playlist();
    state.playlists.add(Playlist::new(playlist, playlist_name));
    let player = state.ids.player();
    state
        .players
        .push(PlayerState::new(player, playlist, PlayMode::Continuous));
    let cursor = RotationCursor::new(state.rotations.default_id().unwrap_or(RotationId(0)));
    // A station that never automates never sees the Music column.
    if !state.layout.hidden_players.contains(&player) {
        state.layout.hidden_players.push(player);
    }
    state.filler = Some(FillerState {
        player,
        playlist,
        rotation: None,
        cursor,
        generated: HashSet::new(),
        starved: false,
        current_since: None,
        unfitted: 0,
        fitted_for: None,
    });
    vec![EngineAction::AddPlayer { player }]
}

/// The fallback length of a row whose track has no known duration.
pub const UNKNOWN_LENGTH_SECS: f64 = 180.0;
/// At most this many rows are added by one `settle`, whatever the lengths:
/// a bound on the work of one step, not an operator limit.
pub const MAX_ADDED_PER_SETTLE: usize = 50;

/// How long a row plays, for the look-ahead.
fn row_secs(state: &AppState, entry: EntryId) -> f64 {
    let length = state
        .track_for_entry(entry)
        .map(|t| {
            let range = t.play_range(state.config.players.use_cue_markers);
            range.cue_out - range.cue_in
        })
        .unwrap_or(0.0);
    if length.is_finite() && length > 0.0 {
        length
    } else {
        UNKNOWN_LENGTH_SECS
    }
}

/// Where the filler's list stands: the rows before `anchor` are history,
/// the rows from `anchor` on are ahead. A next after the row on air wins
/// (AR13): the rows between them will not play.
fn anchor(state: &AppState, f: &FillerState) -> usize {
    let Some(list) = state.playlists.get(f.playlist) else {
        return 0;
    };
    let Some(player) = state.players.iter().find(|p| p.id == f.player) else {
        return list.entries.len();
    };
    let current = player.current.and_then(|c| list.position(c));
    let next = player.next.and_then(|n| list.position(n));
    match (current, next) {
        // A next back before the row on air: the rows after the row on air
        // stay ahead, so the row on air is never among them.
        (Some(c), Some(n)) if n <= c => c + 1,
        (_, Some(n)) => n,
        (Some(c), None) => c + 1,
        (None, None) => list.entries.len(),
    }
}

/// The rows before `played_end` have played: those before the row on air,
/// or before `anchor` when nothing is on air. Rows skipped by a next
/// further down are not history.
fn played_end(state: &AppState, f: &FillerState) -> usize {
    let current = state
        .players
        .iter()
        .find(|p| p.id == f.player)
        .and_then(|p| p.current)
        .and_then(|c| state.playlists.get(f.playlist)?.position(c));
    current.unwrap_or_else(|| anchor(state, f))
}

/// AU27, AU28: trims the played rows beyond `filler_played_kept` and tops
/// the list up to `filler_lookahead_mins` from the filler's rotation.
pub(crate) fn settle(state: &mut AppState, out: &mut Vec<EngineAction>) {
    let Some(f) = state.filler.clone() else {
        return;
    };
    note_current(state, &f);
    trim(state, &f, out);
    drop_beyond(state, &f, out);
    refit(state, out);
    top_up(state);
}

/// Keeps `current_since` on the filler's row on air.
fn note_current(state: &mut AppState, f: &FillerState) {
    let current = state
        .players
        .iter()
        .find(|p| p.id == f.player)
        .filter(|p| p.transport != crate::player::Transport::Stopped)
        .and_then(|p| p.current);
    let since = match current {
        Some(entry) if f.current_since.is_some_and(|(e, _)| e == entry) => f.current_since,
        Some(entry) => Some((entry, state.local_now)),
        None => None,
    };
    if let Some(filler) = state.filler.as_mut() {
        filler.current_since = since;
    }
}

/// How long after a row starts the row after it starts (AU25): to the
/// segue point when `players.auto_segue` is on and the track has one
/// strictly inside its play range (as the player's own plan), else to the
/// end; an unknown length counts as `UNKNOWN_LENGTH_SECS`.
pub(crate) fn lead_secs(state: &AppState, entry: EntryId) -> f64 {
    state
        .track_for_entry(entry)
        .and_then(|t| crate::selection::timing(state, t.id))
        .map_or(UNKNOWN_LENGTH_SECS, |(_, lead)| lead)
}

/// When each row of the filler's list from its anchor on is expected to
/// start (AU25): the first when the row on air reaches its lead (never
/// before now), or now when nothing plays; each next one a lead later.
pub fn starts_ahead(state: &AppState) -> Vec<(EntryId, NaiveDateTime)> {
    match state.filler.as_ref() {
        Some(f) => timeline(state, f).0,
        None => Vec::new(),
    }
}

/// `starts_ahead`, and when a row added after the last would start.
fn timeline(state: &AppState, f: &FillerState) -> (Vec<(EntryId, NaiveDateTime)>, NaiveDateTime) {
    let now = state.local_now;
    let mut at = match f.current_since {
        Some((entry, since)) => (since + millis(lead_secs(state, entry))).max(now),
        None => now,
    };
    let mut out = Vec::new();
    if let Some(list) = state.playlists.get(f.playlist) {
        for e in list.entries.iter().skip(anchor(state, f)) {
            out.push((e.id, at));
            at += millis(lead_secs(state, e.id));
        }
    }
    (out, at)
}

fn millis(secs: f64) -> chrono::TimeDelta {
    chrono::TimeDelta::milliseconds((secs * 1000.0).round() as i64)
}

fn secs_between(from: NaiveDateTime, to: NaiveDateTime) -> f64 {
    (to - from).num_milliseconds() as f64 / 1000.0
}

/// AU25: the row starting at `start` is still audible when the fade
/// starts, so it is the last before the block (unknown length: no).
fn covers(state: &AppState, entry: EntryId, start: NaiveDateTime, fade_at: NaiveDateTime) -> bool {
    start < fade_at
        && state
            .track_for_entry(entry)
            .and_then(|t| crate::selection::timing(state, t.id))
            .is_some_and(|(length, _)| start + millis(length) >= fade_at)
}

/// AU24, AU25: when the filler's fade before the coming block starts.
fn fade_start(state: &AppState) -> Option<NaiveDateTime> {
    let start = crate::automation::coming_start(state)?;
    Some(start - chrono::TimeDelta::milliseconds(i64::from(state.config.automation.block_fade_ms)))
}

/// AU25: what a row starting `gap_secs` before the fade must fit.
fn fit_for(state: &AppState, gap_secs: f64) -> crate::selection::Fit {
    let a = &state.config.automation;
    crate::selection::Fit {
        gap_secs,
        max_cut_secs: f64::from(a.max_fade_cut_secs),
        min_play_secs: f64::from(a.min_song_play_secs),
    }
}

/// AU25: once per song on air while a block is coming, the generated rows
/// after the loaded next are checked against the timeline (the queue
/// drifts with segues and edits); the first that no longer fits leaves
/// with every generated row after it, and the top-up chooses again. The
/// current row, the loaded next and the operator's rows stay.
fn refit(state: &mut AppState, out: &mut Vec<EngineAction>) {
    let Some(f) = state.filler.clone() else {
        return;
    };
    let Some(fade_at) = fade_start(state) else {
        return;
    };
    let player = state.players.iter().find(|p| p.id == f.player);
    let current = player
        .filter(|p| p.transport != crate::player::Transport::Stopped)
        .and_then(|p| p.current);
    if current.is_none() || f.fitted_for == current {
        return;
    }
    let keep_next = player.and_then(|p| p.next);
    let mut drop = HashSet::new();
    // The generated row before, which may change too: a row that does not
    // fit often only shows that the one before left a gap nothing closes.
    let mut before: Option<EntryId> = None;
    for (entry, start) in timeline(state, &f).0 {
        if Some(entry) == keep_next || !f.generated.contains(&entry) {
            before = None;
            continue;
        }
        let gap = secs_between(start, fade_at);
        let fits = || {
            state
                .track_for_entry(entry)
                .is_none_or(|t| crate::selection::fits(state, t.id, &fit_for(state, gap)))
        };
        if drop.is_empty() && gap > 0.0 && !fits() {
            drop.extend(before);
        }
        if !drop.is_empty() {
            drop.insert(entry);
        }
        before = Some(entry);
    }
    if let Some(filler) = state.filler.as_mut() {
        filler.fitted_for = current;
        filler.generated.retain(|e| !drop.contains(e));
    }
    if !drop.is_empty() {
        crate::reducer::take_out_entries(state, &drop, out);
    }
}

/// How far ahead the filler queues: `filler_lookahead_mins`, or up to the
/// coming block while automation runs (AU27), whichever comes first.
fn look_ahead_secs(state: &AppState) -> f64 {
    let own = f64::from(state.config.automation.filler_lookahead_mins) * 60.0;
    match crate::automation::coming_start(state) {
        Some(start) => {
            let secs = (start - state.local_now).num_milliseconds() as f64 / 1000.0;
            own.min(secs.max(0.0))
        }
        None => own,
    }
}

/// AU27, AU28: the generated rows that would start at or after the coming
/// block leave the list (the operator's stay; the next of a playing filler
/// is already loaded and stays).
fn drop_beyond(state: &mut AppState, f: &FillerState, out: &mut Vec<EngineAction>) {
    let Some(fade_at) = fade_start(state) else {
        return;
    };
    let player = state.players.iter().find(|p| p.id == f.player);
    // A row on air still audible at the fade is the last: even the loaded
    // next would start just before the fade (AU25).
    let current_covers = f
        .current_since
        .is_some_and(|(e, since)| covers(state, e, since, fade_at));
    let keep_next = player
        .filter(|p| p.transport != crate::player::Transport::Stopped)
        .and_then(|p| p.next)
        .filter(|_| !current_covers);
    let mut covered = current_covers;
    let mut drop = HashSet::new();
    for (e, start) in timeline(state, f).0 {
        if (covered || start >= fade_at) && f.generated.contains(&e) && Some(e) != keep_next {
            drop.insert(e);
        }
        covered |= covers(state, e, start, fade_at);
    }
    if drop.is_empty() {
        return;
    }
    crate::reducer::take_out_entries(state, &drop, out);
    if let Some(filler) = state.filler.as_mut() {
        filler.generated.retain(|e| !drop.contains(e));
    }
}

fn trim(state: &mut AppState, f: &FillerState, out: &mut Vec<EngineAction>) {
    let Some(list) = state.playlists.get(f.playlist) else {
        return;
    };
    let current = state
        .players
        .iter()
        .find(|p| p.id == f.player)
        .and_then(|p| p.current);
    let history = played_end(state, f);
    let kept = state.config.automation.filler_played_kept as usize;
    let drop: HashSet<EntryId> = list
        .entries
        .iter()
        .take(history.saturating_sub(kept))
        .map(|e| e.id)
        .filter(|e| Some(*e) != current)
        .collect();
    if drop.is_empty() {
        return;
    }
    crate::reducer::take_out_entries(state, &drop, out);
    if let Some(filler) = state.filler.as_mut() {
        filler.generated.retain(|e| !drop.contains(e));
    }
}

fn top_up(state: &mut AppState) {
    let Some(f) = state.filler.clone() else {
        return;
    };
    let Some(list) = state.playlists.get(f.playlist) else {
        return;
    };
    let ahead: Vec<EntryId> = list
        .entries
        .iter()
        .skip(anchor(state, &f))
        .map(|e| e.id)
        .collect();
    let now = state.wall_now;
    let mut remaining = 0.0;
    let mut planned = Vec::with_capacity(ahead.len());
    for entry in &ahead {
        if let Some(track) = state.track_for_entry(*entry).map(|t| t.id) {
            planned.push((track, now.plus_secs(remaining)));
        }
        remaining += row_secs(state, *entry);
    }
    let target = look_ahead_secs(state);
    // AU25: with a block coming, the rows are timed on the timeline and
    // each is chosen to fit the time left before the fade.
    let fade_at = fade_start(state);
    let (rows, mut end) = timeline(state, &f);
    // A row still audible at the fade is the last before the block.
    let mut covered = fade_at.is_some_and(|fade| {
        f.current_since
            .is_some_and(|(e, since)| covers(state, e, since, fade))
            || rows
                .iter()
                .any(|(e, start)| covers(state, *e, *start, fade))
    });
    let horizon = fade_at.map(|fade| {
        let own = i64::from(state.config.automation.filler_lookahead_mins) * 60_000;
        fade.min(state.local_now + chrono::TimeDelta::milliseconds(own))
    });
    let mut cursor = f.cursor.clone();
    let mut rng = state.rng.clone();
    let mut added = Vec::new();
    let mut starved = false;
    let mut unfitted = 0;
    while added.len() < MAX_ADDED_PER_SETTLE
        && match horizon {
            Some(h) => end < h && !covered,
            None => remaining < target,
        }
    {
        let at = now.plus_secs(remaining);
        let fit = fade_at.map(|fade| fit_for(state, secs_between(end, fade)));
        let found = crate::rotation_cursor::next_track_fitted(
            state,
            &mut cursor,
            at,
            &planned,
            &mut rng,
            fit.as_ref(),
        );
        let Some(picked) = found.picked else {
            starved = true;
            break;
        };
        unfitted += u64::from(picked.unfitted);
        let entry = state.ids.entry();
        if let Some(list) = state.playlists.get_mut(f.playlist) {
            list.entries
                .push(crate::playlist::PlaylistEntry::new(entry, picked.track));
        }
        planned.push((picked.track, at));
        remaining += row_secs(state, entry);
        if let Some(fade) = fade_at {
            covered = covers(state, entry, end, fade);
        }
        end += millis(lead_secs(state, entry));
        added.push(entry);
    }
    state.rng = rng;
    // AR13: whether stopped or with its last row on air, a filler with no
    // next takes the first new row.
    let player_next_empty = state
        .players
        .iter()
        .find(|p| p.id == f.player)
        .is_some_and(|p| p.next.is_none());
    if let Some(filler) = state.filler.as_mut() {
        filler.cursor = cursor;
        filler.generated.extend(added.iter().copied());
        filler.starved = starved && ahead.is_empty() && added.is_empty();
        filler.unfitted += unfitted;
    }
    if player_next_empty
        && let Some(first) = added.first().copied()
        && let Some(p) = state.players.iter_mut().find(|p| p.id == f.player)
    {
        p.next = Some(first);
        p.next_explicit = false;
    }
}

/// AU28: plays `rotation` (`None`: the default) from now on: the generated
/// rows not yet played are replaced, the operator's stay.
pub(crate) fn set_rotation(
    state: &mut AppState,
    rotation: Option<RotationId>,
    out: &mut Vec<EngineAction>,
) -> Result<(), crate::error::ModelError> {
    if let Some(id) = rotation
        && state.rotations.get(id).is_none()
    {
        return Err(crate::error::ModelError::UnknownRotation(id));
    }
    let Some(f) = state.filler.clone() else {
        return Ok(());
    };
    let player = state.players.iter().find(|p| p.id == f.player);
    // The next of a playing filler is already loaded for the hand-over.
    let keep_next = player
        .filter(|p| p.transport != crate::player::Transport::Stopped)
        .and_then(|p| p.next);
    let current = player.and_then(|p| p.current);
    let from = anchor(state, &f);
    let drop: HashSet<EntryId> = state
        .playlists
        .get(f.playlist)
        .map(|l| {
            l.entries
                .iter()
                .skip(from)
                .map(|e| e.id)
                .filter(|e| f.generated.contains(e))
                .filter(|e| Some(*e) != keep_next && Some(*e) != current)
                .collect()
        })
        .unwrap_or_default();
    crate::reducer::take_out_entries(state, &drop, out);
    let start = rotation
        .or(state.rotations.default_id())
        .unwrap_or(RotationId(0));
    if let Some(filler) = state.filler.as_mut() {
        filler.rotation = rotation;
        filler.cursor = RotationCursor::new(start);
        filler.generated.retain(|e| !drop.contains(e));
    }
    Ok(())
}

/// AU4, AU28: after a rotation is deleted or another becomes the default,
/// a filler on a deleted rotation falls back to the default, and one on
/// the default follows the new default; its queued rows are replaced as
/// when the operator picks a rotation.
pub(crate) fn retarget(
    state: &mut AppState,
    out: &mut Vec<EngineAction>,
) -> Result<(), crate::error::ModelError> {
    let Some(f) = state.filler.as_ref() else {
        return Ok(());
    };
    let gone = f.rotation.is_some_and(|r| state.rotations.get(r).is_none());
    let stale = f.rotation.is_none() && Some(f.cursor.rotation) != state.rotations.default_id();
    if gone || stale {
        set_rotation(state, None, out)
    } else {
        Ok(())
    }
}

/// AU28: a row the operator moved becomes theirs.
pub(crate) fn claim(state: &mut AppState, entry: EntryId) {
    if let Some(f) = state.filler.as_mut() {
        f.generated.remove(&entry);
    }
}

/// What is saved of the filler (`kv` value `filler`).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FillerSession {
    pub player: crate::session::PlayerSession,
    pub playlist: PlaylistId,
    #[serde(default)]
    pub rotation: Option<RotationId>,
    pub cursor: RotationCursor,
    #[serde(default)]
    pub generated: Vec<EntryId>,
}

impl AppState {
    /// The filler as saved; `position_of` gives its playback position.
    pub fn filler_session(&self, position_of: impl Fn(PlayerId) -> f64) -> Option<FillerSession> {
        let f = self.filler.as_ref()?;
        let player = self.players.iter().find(|p| p.id == f.player)?;
        let mut generated: Vec<EntryId> = f.generated.iter().copied().collect();
        generated.sort();
        Some(FillerSession {
            player: crate::session::session_of(player, &position_of),
            playlist: f.playlist,
            rotation: f.rotation,
            cursor: f.cursor.clone(),
            generated,
        })
    }
}
