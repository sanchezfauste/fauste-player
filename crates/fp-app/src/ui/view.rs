//! What each part of the screen shows, derived from the model snapshot and
//! the engine telemetry. Pure functions: everything here is unit-tested.

use fp_model::{
    AppState, EntryId, FileState, PlayMode, PlayerId, PlaylistEntry, PlaylistId, Track, Transport,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerStatus {
    OnAir,
    Paused,
    Stopped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowStatus {
    /// This player's current entry.
    Current,
    /// This player's next.
    Next,
    /// On air on another player (1-based player number); players are
    /// independent, so it is only marked (spec §3 rule 22).
    OnAirElsewhere(usize),
    Unavailable,
    Played,
    Normal,
}

/// Cue points as fractions (0..1) of the track length, for the waveform.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct MarkerFractions {
    pub position: Option<f32>,
    pub cue_in: Option<f32>,
    pub intro_end: Option<f32>,
    pub outro_start: Option<f32>,
    pub segue_start: Option<f32>,
    pub cue_out: Option<f32>,
    /// Players ignore cue-in and cue-out: the marks are drawn dimmed and
    /// the head and tail are not shaded.
    pub ignored: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerView {
    pub status: PlayerStatus,
    pub title: Option<String>,
    pub artist: Option<String>,
    /// "Title – Artist" of the next entry.
    pub next_line: Option<String>,
    pub elapsed: f64,
    /// Track length, when known.
    pub total: Option<f64>,
    /// Seconds to cue-out.
    pub remaining: f64,
    /// The countdown turns red (spec §3 rule 17).
    pub end_warning: bool,
    /// Seconds left of a marked intro (rule 18).
    pub intro: Option<f64>,
    /// `Some(visible)` during the intro's last 3 s (blinking).
    pub intro_blink: Option<bool>,
    /// Seconds from the outro to cue-out (rule 19).
    pub outro: Option<f64>,
    pub markers: MarkerFractions,
    pub mode: PlayMode,
    pub stop_after_current: bool,
    /// The current entry repeats or stops after itself (spec O8).
    pub entry_notice: Option<fp_model::EntryNotice>,
    pub fading: bool,
    pub cueing: bool,
    /// The current source reaches its device unchanged (the BP badge).
    pub bit_perfect: bool,
}

fn fraction(value: Option<f64>, total: f64) -> Option<f32> {
    value
        .filter(|_| total > 0.0)
        .map(|v| (v / total).clamp(0.0, 1.0) as f32)
}

fn line(title: &str, artist: &str) -> String {
    if artist.is_empty() {
        title.to_owned()
    } else {
        format!("{title} – {artist}")
    }
}

/// The entry a player shows: its current one, or, when stopped, its next
/// (the one Play starts), ready at its cue-in.
pub fn shown_entry(state: &AppState, player: PlayerId) -> Option<EntryId> {
    let p = state.player(player).ok()?;
    p.current.or(p.next)
}

/// What a CUE window shows (feedback 2 spec O12). A CUE plays the whole
/// file, so its times run to the end of the file, not to the cue-out.
#[derive(Debug, Clone, PartialEq)]
pub struct CueWindowView {
    pub player: PlayerId,
    pub entry: EntryId,
    pub title: String,
    pub artist: Option<String>,
    pub elapsed: f64,
    /// File length, when known.
    pub total: Option<f64>,
    /// Seconds to the end of the file; 0 while the length is unknown.
    pub remaining: f64,
    pub paused: bool,
    /// The position as a fraction of the file, for the waveform.
    pub position: Option<f32>,
    /// "Load as next" has something to do: the cued entry is not the
    /// current one and not already the explicit next.
    pub can_load_next: bool,
}

/// The CUE window of `player`, or `None` while it has no CUE. `position`
/// is the engine's CUE position; without one (or a broken one) the CUE
/// shows the start of its play range, where it begins.
pub fn cue_window_view(
    state: &AppState,
    player: PlayerId,
    position: Option<f64>,
) -> Option<CueWindowView> {
    let p = state.player(player).ok()?;
    let cue = p.cue?;
    let track = state.track_for_entry(cue.entry)?;
    let total = (track.duration_secs > 0.0).then_some(track.duration_secs);
    let start = track
        .play_range(state.config.players.use_cue_markers)
        .cue_in;
    let mut elapsed = position.filter(|v| v.is_finite()).unwrap_or(start).max(0.0);
    if let Some(total) = total {
        elapsed = elapsed.min(total);
    }
    Some(CueWindowView {
        player,
        entry: cue.entry,
        title: track.title.clone(),
        artist: Some(track.artist.clone()).filter(|a| !a.is_empty()),
        elapsed,
        total,
        remaining: total.map_or(0.0, |t| (t - elapsed).max(0.0)),
        paused: cue.paused,
        position: fraction(Some(elapsed), total.unwrap_or(0.0)),
        can_load_next: p.current != Some(cue.entry)
            && !(p.next == Some(cue.entry) && p.next_explicit),
    })
}

/// Feedback 2 spec O17: while `player`'s CUE runs, a single click on a row
/// moves it to that entry. `Some(entry)` when the CUE is on another entry
/// and `clicked` can be played; `None` for no CUE, the entry already cued,
/// or a file that is missing or unreadable. Selection is UI state, so this
/// decision is here and the move itself is the model's `CueEntry`.
pub fn cue_follow_target(state: &AppState, player: PlayerId, clicked: EntryId) -> Option<EntryId> {
    let cue = state.player(player).ok()?.cue?;
    (cue.entry != clicked && state.playable_request(clicked).is_some()).then_some(clicked)
}

/// Everything a player column shows. `position` comes from the engine;
/// `blink_phase` is a clock in seconds for blinking elements.
pub fn player_view(
    state: &AppState,
    player: PlayerId,
    position: Option<f64>,
    blink_phase: f64,
) -> Option<PlayerView> {
    let p = state.player(player).ok()?;
    let status = match p.transport {
        Transport::Playing => PlayerStatus::OnAir,
        Transport::Paused => PlayerStatus::Paused,
        Transport::Stopped => PlayerStatus::Stopped,
    };
    let next_line = p
        .next
        .and_then(|e| state.track_for_entry(e))
        .map(|t| line(&t.title, &t.artist));
    let current = shown_entry(state, player).and_then(|e| state.track_for_entry(e));
    // A track waiting to be played has no position yet: one reported now
    // is left over from the last track.
    let position = position.filter(|_| p.current.is_some());
    let mut view = PlayerView {
        status,
        title: None,
        artist: None,
        next_line,
        elapsed: 0.0,
        total: None,
        remaining: 0.0,
        end_warning: false,
        intro: None,
        intro_blink: None,
        outro: None,
        markers: MarkerFractions::default(),
        mode: p.mode,
        stop_after_current: p.stop_after_current,
        entry_notice: fp_model::entry_notice(state, player),
        fading: p.fading,
        cueing: p.cue.is_some(),
        bit_perfect: false,
    };
    let Some(track) = current else {
        return Some(view);
    };
    let range = track.play_range(state.config.players.use_cue_markers);
    let pos = position.filter(|v| v.is_finite()).unwrap_or(range.cue_in);
    let cue_out = range.cue_out;
    let total = (track.duration_secs > 0.0).then_some(track.duration_secs);
    view.title = Some(track.title.clone());
    view.artist = Some(track.artist.clone()).filter(|a| !a.is_empty());
    view.elapsed = pos;
    view.total = total;
    view.remaining = (cue_out - pos).max(0.0);
    view.end_warning = status == PlayerStatus::OnAir
        && total.is_some()
        && view.remaining <= state.config.players.end_warning_secs;
    if let Some(intro_end) = track.intro_end_secs()
        && pos < intro_end
    {
        let left = intro_end - pos;
        view.intro = Some((left * 10.0).round() / 10.0);
        // The talk-over warning blinks only for a track on its way.
        view.intro_blink =
            (left <= 3.0 && p.current.is_some()).then(|| blink_phase.rem_euclid(1.0) < 0.5);
    }
    if let Some(outro) = track.outro_start_secs()
        && pos >= outro
    {
        view.outro = Some((cue_out - pos).max(0.0));
    }
    if let Some(total) = total {
        view.markers = MarkerFractions {
            position: fraction(Some(pos), total),
            cue_in: fraction(track.markers.cue_in.map(|m| m.secs), total),
            intro_end: fraction(track.intro_end_secs(), total),
            outro_start: fraction(track.outro_start_secs(), total),
            segue_start: fraction(track.segue_start_secs(), total),
            cue_out: fraction(track.markers.cue_out.map(|m| m.secs), total),
            ignored: !state.config.players.use_cue_markers,
        };
    }
    Some(view)
}

/// Why a track cannot be played, as the Fluent key of its tooltip (which
/// takes the file's `$path`); `None` when it can.
pub fn file_problem(track: &Track) -> Option<&'static str> {
    match track.file_state {
        FileState::Ok => None,
        FileState::Missing => Some("file-missing-tip"),
        FileState::Unreadable => Some("file-unreadable-tip"),
    }
}

/// The icon of a track that cannot be played: a file with a cross when it
/// is not found, a warning sign when it cannot be read.
pub fn file_icon(track: &Track) -> &'static str {
    match track.file_state {
        FileState::Missing => egui_phosphor::regular::FILE_X,
        FileState::Ok | FileState::Unreadable => egui_phosphor::regular::WARNING,
    }
}

/// How a track-table row is drawn (spec §3 rule 1). Takes the row's entry
/// directly: the table already has it, so nothing is searched per row.
pub fn row_status(state: &AppState, player: PlayerId, entry: &PlaylistEntry) -> RowStatus {
    let me = state.player(player).ok();
    if me.is_some_and(|p| p.current == Some(entry.id)) {
        return RowStatus::Current;
    }
    if me.is_some_and(|p| p.next == Some(entry.id)) {
        return RowStatus::Next;
    }
    if let Some(n) = state
        .players
        .iter()
        .position(|p| p.id != player && p.current == Some(entry.id))
    {
        return RowStatus::OnAirElsewhere(n + 1);
    }
    if !state.library.is_playable(entry.track) {
        return RowStatus::Unavailable;
    }
    if entry.is_played_by(player) {
        RowStatus::Played
    } else {
        RowStatus::Normal
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlaylistTimes {
    pub total: f64,
    pub elapsed: f64,
    pub remaining: f64,
}

/// Whole-playlist times (spec §3 rule 20): played entries count in full,
/// on-air entries up to their position.
pub fn playlist_times(
    state: &AppState,
    player: PlayerId,
    playlist: PlaylistId,
    positions: &[(PlayerId, f64)],
) -> PlaylistTimes {
    let use_markers = state.config.players.use_cue_markers;
    let mut total = 0.0;
    let mut elapsed = 0.0;
    for e in state
        .playlists
        .get(playlist)
        .map(|p| p.entries.as_slice())
        .unwrap_or_default()
    {
        let Some(track) = state.library.get(e.track) else {
            continue;
        };
        let range = track.play_range(use_markers);
        let len = range.length();
        total += len;
        // This player's own progress only: players are independent.
        let on_air = state
            .players
            .iter()
            .find(|p| p.id == player && p.current == Some(e.id));
        if let Some(p) = on_air {
            let pos = positions
                .iter()
                .find(|(id, _)| *id == p.id)
                .map_or(range.cue_in, |(_, s)| *s);
            elapsed += (pos - range.cue_in).clamp(0.0, len);
        } else if e.is_played_by(player) {
            elapsed += len;
        }
    }
    PlaylistTimes {
        total,
        elapsed,
        remaining: (total - elapsed).max(0.0),
    }
}

/// The fader curve lives in the model, so MIDI faders share it.
pub use fp_model::volume::{fader_from_gain, gain_from_fader};

/// Gain in dB; `None` for silence (shown as −∞ by the UI).
pub fn volume_db(gain: f32) -> Option<f32> {
    (gain > 0.0 && !gain.is_nan()).then(|| 20.0 * gain.log10())
}

/// Title and Artist are never narrower than this while the table allows.
const TEXT_COLUMN_MIN: f32 = 60.0;

/// Pixel widths of the track table's columns (`#`, Title, Artist, Duration)
/// for a table `width` wide (feedback spec §2.3): the stored fractions, or
/// by default the minimums for `#` and Duration and 60:40 of the rest. The
/// minimums win, and the widths always sum to `width`.
pub fn column_px(
    fractions: Option<[f32; 4]>,
    width: f32,
    number_min: f32,
    duration_min: f32,
) -> [f32; 4] {
    let width = if width.is_finite() {
        width.max(0.0)
    } else {
        0.0
    };
    let wanted = fp_model::ColumnWidths { fractions }
        .normalized()
        .fractions
        .map(|f| f.map(|x| x * width));
    let [n, t, a, d] = wanted.unwrap_or([number_min, 0.6, 0.4, duration_min]);
    let (mut number, mut duration) = (n.max(number_min), d.max(duration_min));
    if number + duration > width {
        // Not even the minimums fit: share what there is.
        let scale = width / (number + duration).max(f32::EPSILON);
        number *= scale;
        duration *= scale;
        return [number, 0.0, 0.0, duration];
    }
    let rest = width - number - duration;
    let share = if t + a > 0.0 { t / (t + a) } else { 0.6 };
    let mut title = rest * share;
    if rest >= 2.0 * TEXT_COLUMN_MIN {
        title = title.clamp(TEXT_COLUMN_MIN, rest - TEXT_COLUMN_MIN);
    }
    [number, title, rest - title, duration]
}

/// A line of the row tooltip (feedback 2 spec O23).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipField {
    Title,
    Artist,
    Album,
    Date,
    Genre,
    Duration,
    Format,
    Path,
}

/// What the row tooltip shows for `track`: the fields it has, in this
/// order. Album artist, composer and comment are left to the editor.
pub fn track_tooltip(track: &Track) -> Vec<(TipField, String)> {
    let mut lines = Vec::new();
    let mut text = |field, value: &str| {
        if !value.is_empty() {
            lines.push((field, value.to_owned()));
        }
    };
    text(TipField::Title, &track.title);
    text(TipField::Artist, &track.artist);
    text(TipField::Album, &track.album);
    text(TipField::Date, track.date.as_deref().unwrap_or_default());
    text(TipField::Genre, &track.genre);
    if track.duration_secs.is_finite() && track.duration_secs > 0.0 {
        text(
            TipField::Duration,
            &super::format::clock(track.duration_secs),
        );
    }
    text(TipField::Format, &format_line(track));
    text(TipField::Path, &track.path.display().to_string());
    lines
}

/// `FLAC · 44.1 kHz · 16 bit`: the codec is the file extension, in capitals.
fn format_line(track: &Track) -> String {
    let mut parts = Vec::new();
    if let Some(ext) = track.path.extension().and_then(|e| e.to_str()) {
        parts.push(ext.to_uppercase());
    }
    if let Some(f) = track.format {
        if f.sample_rate > 0 {
            let khz = f64::from(f.sample_rate) / 1000.0;
            let text = if khz.fract().abs() < 1e-9 {
                format!("{khz:.0}")
            } else {
                format!("{khz:.1}")
            };
            parts.push(format!("{text} kHz"));
        }
        if let Some(bits) = f.bits {
            parts.push(format!("{bits} bit"));
        }
    }
    parts.join(" · ")
}
