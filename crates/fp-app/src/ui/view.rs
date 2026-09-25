//! What each part of the screen shows, derived from the model snapshot and
//! the engine telemetry. Pure functions: everything here is unit-tested.

use fp_model::{AppState, EntryId, PlayMode, PlayerId, PlaylistId, Transport};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerStatus {
    OnAir,
    Paused,
    Stopped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowStatus {
    Current,
    Next,
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
    pub fading: bool,
    pub cueing: bool,
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
    let current = p.current.and_then(|e| state.track_for_entry(e));
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
        fading: p.fading,
        cueing: p.cue.is_some(),
    };
    let Some(track) = current else {
        return Some(view);
    };
    let pos = position
        .filter(|v| v.is_finite())
        .unwrap_or_else(|| track.cue_in_secs());
    let cue_out = track.cue_out_secs();
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
        view.intro_blink = (left <= 3.0).then(|| blink_phase.rem_euclid(1.0) < 0.5);
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
        };
    }
    Some(view)
}

/// How a track-table row is drawn (spec §3 rule 1).
pub fn row_status(state: &AppState, player: PlayerId, entry: EntryId) -> RowStatus {
    if state.is_on_air(entry) {
        return RowStatus::Current;
    }
    if state.player(player).is_ok_and(|p| p.next == Some(entry)) {
        return RowStatus::Next;
    }
    let Some(e) = state.playlists.entry(entry) else {
        return RowStatus::Normal;
    };
    if !state.library.is_playable(e.track) {
        return RowStatus::Unavailable;
    }
    if e.played {
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
    playlist: PlaylistId,
    positions: &[(PlayerId, f64)],
) -> PlaylistTimes {
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
        let len = track.play_length_secs();
        total += len;
        let on_air = state.players.iter().find(|p| p.current == Some(e.id));
        if let Some(p) = on_air {
            let pos = positions
                .iter()
                .find(|(id, _)| *id == p.id)
                .map_or(track.cue_in_secs(), |(_, s)| *s);
            elapsed += (pos - track.cue_in_secs()).clamp(0.0, len);
        } else if e.played {
            elapsed += len;
        }
    }
    PlaylistTimes {
        total,
        elapsed,
        remaining: (total - elapsed).max(0.0),
    }
}

/// Range of the fader in dB below 0 dB; the bottom of the travel is silence.
const FADER_RANGE_DB: f32 = 60.0;

/// Fader travel (0 bottom … 1 top) to linear gain: linear in dB, 0 dB at the top.
pub fn gain_from_fader(pos: f32) -> f32 {
    if pos <= 0.0 || pos.is_nan() {
        return 0.0;
    }
    let db = FADER_RANGE_DB * (pos.min(1.0) - 1.0);
    10f32.powf(db / 20.0)
}

pub fn fader_from_gain(gain: f32) -> f32 {
    if gain <= 0.0 || gain.is_nan() {
        return 0.0;
    }
    (1.0 + 20.0 * gain.min(1.0).log10() / FADER_RANGE_DB).clamp(0.0, 1.0)
}

pub fn volume_db_text(gain: f32) -> String {
    if gain <= 0.0 || gain.is_nan() {
        "-∞ dB".to_owned()
    } else {
        format!("{:.1} dB", 20.0 * gain.log10())
    }
}
