//! The JSON shapes of `/api/v1` (remote control spec §3.1). They are the
//! API's contract: the model's own serialisation never leaks out, and file
//! paths are never exposed.

use fp_model::volume::fader_from_gain;
use fp_model::{
    AppState, CartId, CartKind, CartPage, CartPageId, EntryId, FileState, GainLimit, Marker,
    MarkerSource, Normalization, NormalizationStatus, PlayMode, PlayerId, PlayerState, Playlist,
    PlaylistId, Track, TrackId, TrackKind, Transport,
};
use serde::Serialize;

use crate::control::Playback;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StateDto {
    pub revision: u64,
    pub players: Vec<PlayerDto>,
    pub playlists: Vec<PlaylistSummaryDto>,
    pub cartwall: CartwallDto,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlayerDto {
    pub id: PlayerId,
    /// 1-based, in display order.
    pub position: usize,
    pub transport: &'static str,
    pub fading: bool,
    pub mode: &'static str,
    pub stop_after_current: bool,
    /// Fader travel 0–1 (the UI fader's curve).
    pub fader: f32,
    /// The fader is held at 100 % because the player's output is
    /// bit-perfect; such a player also starts every track without a
    /// crossfade (bit-perfect rate and fader spec BF13, BR16).
    pub fader_locked: bool,
    /// The playlist shown in the player.
    pub playlist: PlaylistId,
    pub current: Option<EntryRefDto>,
    pub next: Option<EntryRefDto>,
    pub cue: Option<CueDto>,
    pub elapsed_secs: Option<f64>,
    pub remaining_secs: Option<f64>,
    /// The normalization the current entry started with, while the player
    /// is not stopped (loudness normalization spec N23). It is what the
    /// model asked the engine for; it does not change while the entry
    /// plays.
    pub normalization: Option<NormalizationDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EntryRefDto {
    pub entry: EntryId,
    pub track: TrackDto,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CueDto {
    pub entry: EntryId,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlaylistSummaryDto {
    pub id: PlaylistId,
    pub name: String,
    pub entry_count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlaylistDto {
    pub id: PlaylistId,
    pub name: String,
    pub entry_count: usize,
    pub entries: Vec<EntryDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EntryDto {
    pub id: EntryId,
    pub track: TrackDto,
    pub repeat: bool,
    pub stop_after: bool,
    /// Players playing this entry.
    pub on_air: Vec<PlayerId>,
    /// Players that will play it next.
    pub next_on: Vec<PlayerId>,
    /// Players pre-listening it.
    pub cued_on: Vec<PlayerId>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TrackDto {
    pub id: TrackId,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_secs: f64,
    pub kind: &'static str,
    pub file_state: &'static str,
    pub analyzed: bool,
    pub format: Option<FormatDto>,
    pub markers: MarkersDto,
    /// The measurement; `None` until the analysis measured the file
    /// (loudness normalization spec N23).
    pub loudness: Option<LoudnessDto>,
    /// The operator's per-track setting.
    pub loudness_edit: LoudnessEditDto,
}

/// What the analysis measured of a whole file.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LoudnessDto {
    /// Gated integrated loudness, LUFS; `None` for a silent file.
    pub integrated_lufs: Option<f32>,
    pub true_peak_dbtp: f32,
}

/// A track's loudness edit (`PUT /tracks/{id}/loudness`).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LoudnessEditDto {
    pub excluded: bool,
    pub trim_db: f32,
}

/// The fixed gain a source plays with and why (see `normalization_name`).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NormalizationDto {
    pub gain_db: f32,
    pub status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FormatDto {
    pub sample_rate: u32,
    pub channels: u32,
    pub bits: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MarkersDto {
    pub cue_in: Option<MarkerDto>,
    pub intro_end: Option<MarkerDto>,
    pub outro_start: Option<MarkerDto>,
    pub segue_start: Option<MarkerDto>,
    pub cue_out: Option<MarkerDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MarkerDto {
    pub secs: f64,
    pub source: &'static str,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CartwallDto {
    pub shown_page: Option<CartPageId>,
    pub pages: Vec<CartPageDto>,
    pub playing: Vec<PlayingCartDto>,
    pub cue: Option<CartId>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CartPageDto {
    pub id: CartPageId,
    pub name: String,
    pub rows: u16,
    pub cols: u16,
    pub carts: Vec<CartDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CartDto {
    pub id: CartId,
    /// Row-major, 0-based within its page.
    pub index: usize,
    pub name: String,
    pub kind: &'static str,
    pub looped: bool,
    pub exclusive: bool,
    /// Cart colour spec CC9: the colour's lower-case name, or `null`.
    pub color: Option<&'static str>,
    pub track: Option<TrackDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlayingCartDto {
    pub cart: CartId,
    pub elapsed_secs: Option<f64>,
    pub remaining_secs: Option<f64>,
    /// The normalization the cart was fired with (N23).
    pub normalization: NormalizationDto,
}

pub fn state(model: &AppState, playback: &Playback) -> StateDto {
    StateDto {
        revision: playback.revision,
        players: players(model, playback),
        playlists: playlists(model),
        cartwall: cartwall(model, playback),
    }
}

pub fn players(model: &AppState, playback: &Playback) -> Vec<PlayerDto> {
    model
        .players
        .iter()
        .enumerate()
        .map(|(i, p)| player_dto(model, playback, i, p))
        .collect()
}

pub fn player(model: &AppState, playback: &Playback, id: PlayerId) -> Option<PlayerDto> {
    let i = model.player_index(id).ok()?;
    model
        .players
        .get(i)
        .map(|p| player_dto(model, playback, i, p))
}

pub fn playlists(model: &AppState) -> Vec<PlaylistSummaryDto> {
    model.playlists.iter().map(summary).collect()
}

pub fn playlist(model: &AppState, id: PlaylistId) -> Option<PlaylistDto> {
    let list = model.playlists.get(id)?;
    let entries = list
        .entries
        .iter()
        .filter_map(|e| {
            let track = model.library.get(e.track)?;
            let players = |f: &dyn Fn(&PlayerState) -> bool| -> Vec<PlayerId> {
                model
                    .players
                    .iter()
                    .filter(|p| f(p))
                    .map(|p| p.id)
                    .collect()
            };
            Some(EntryDto {
                id: e.id,
                track: track_dto(track),
                repeat: e.repeat,
                stop_after: e.stop_after,
                on_air: players(&|p| p.current == Some(e.id) && p.transport != Transport::Stopped),
                next_on: players(&|p| p.next == Some(e.id)),
                cued_on: players(&|p| p.cue.is_some_and(|c| c.entry == e.id)),
            })
        })
        .collect();
    Some(PlaylistDto {
        id: list.id,
        name: list.name.clone(),
        entry_count: list.entries.len(),
        entries,
    })
}

pub fn track(model: &AppState, id: TrackId) -> Option<TrackDto> {
    model.library.get(id).map(track_dto)
}

pub fn cartwall(model: &AppState, playback: &Playback) -> CartwallDto {
    let cw = &model.cartwall;
    let playing = cw
        .playing
        .iter()
        .map(|pc| {
            let track = cw
                .cart(pc.cart)
                .and_then(|c| c.track)
                .and_then(|t| model.library.get(t));
            let elapsed = playback
                .cart_position(pc.cart)
                .or_else(|| track.map(Track::cue_in_secs));
            // As the cart button shows it: within the play length, and
            // nothing while the duration is unknown.
            let length = track.map_or(0.0, Track::play_length_secs);
            let known = track.is_some_and(|t| t.duration_secs > 0.0) && length > 0.0;
            PlayingCartDto {
                cart: pc.cart,
                elapsed_secs: elapsed,
                remaining_secs: track
                    .zip(elapsed)
                    .filter(|_| known)
                    .map(|(t, e)| (t.cue_out_secs() - e).clamp(0.0, length)),
                normalization: normalization_dto(&pc.normalization),
            }
        })
        .collect();
    CartwallDto {
        shown_page: cw.shown_page().map(|p| p.id),
        pages: cw.pages.iter().map(|p| page_dto(model, p)).collect(),
        playing,
        cue: cw.cue,
    }
}

fn player_dto(model: &AppState, playback: &Playback, index: usize, p: &PlayerState) -> PlayerDto {
    let use_markers = model.config.players.use_cue_markers;
    let current = p.current.and_then(|e| model.track_for_entry(e));
    let elapsed = current.map(|t| {
        playback
            .player_position(p.id)
            .filter(|v| v.is_finite())
            .unwrap_or_else(|| t.play_range(use_markers).cue_in)
    });
    PlayerDto {
        id: p.id,
        position: index + 1,
        transport: transport_name(p.transport),
        fading: p.fading,
        mode: mode_name(p.mode),
        stop_after_current: p.stop_after_current,
        fader: fader_from_gain(p.volume),
        fader_locked: fp_model::fader_locked(model, p.id),
        playlist: p.playlist,
        current: p.current.and_then(|e| entry_ref(model, e)),
        next: p.next.and_then(|e| entry_ref(model, e)),
        cue: p.cue.map(|c| CueDto { entry: c.entry }),
        elapsed_secs: elapsed,
        remaining_secs: current
            .zip(elapsed)
            .map(|(t, e)| (t.play_range(use_markers).cue_out - e).max(0.0)),
        normalization: p
            .on_air
            .filter(|_| p.transport != Transport::Stopped)
            .map(|n| normalization_dto(&n)),
    }
}

/// N23: a normalization status as the API names it.
pub fn normalization_name(status: NormalizationStatus) -> &'static str {
    use NormalizationStatus as S;
    match status {
        S::Applied { limited: None } => "applied",
        S::Applied {
            limited: Some(GainLimit::TruePeak),
        } => "limited_true_peak",
        S::Applied {
            limited: Some(GainLimit::MaxBoost),
        } => "limited_max_boost",
        S::Off => "off",
        S::Excluded => "excluded",
        S::NotMeasured => "not_measured",
        S::Silent => "silent",
        S::BitPerfect => "bit_perfect",
    }
}

fn normalization_dto(n: &Normalization) -> NormalizationDto {
    NormalizationDto {
        gain_db: n.gain_db,
        status: normalization_name(n.status),
    }
}

fn entry_ref(model: &AppState, entry: EntryId) -> Option<EntryRefDto> {
    model.track_for_entry(entry).map(|t| EntryRefDto {
        entry,
        track: track_dto(t),
    })
}

fn summary(list: &Playlist) -> PlaylistSummaryDto {
    PlaylistSummaryDto {
        id: list.id,
        name: list.name.clone(),
        entry_count: list.entries.len(),
    }
}

fn page_dto(model: &AppState, page: &CartPage) -> CartPageDto {
    CartPageDto {
        id: page.id,
        name: page.name.clone(),
        rows: page.rows,
        cols: page.cols,
        carts: page
            .carts
            .iter()
            .enumerate()
            .map(|(index, c)| CartDto {
                id: c.id,
                index,
                name: c.name.clone(),
                kind: cart_kind_name(c.kind),
                looped: c.looped,
                exclusive: c.exclusive,
                color: c.color.map(fp_model::CartColor::name),
                track: c.track.and_then(|t| model.library.get(t)).map(track_dto),
            })
            .collect(),
    }
}

fn track_dto(t: &Track) -> TrackDto {
    let m = &t.markers;
    TrackDto {
        id: t.id,
        title: t.title.clone(),
        artist: t.artist.clone(),
        album: t.album.clone(),
        duration_secs: t.duration_secs,
        kind: track_kind_name(t.kind),
        file_state: file_state_name(t.file_state),
        analyzed: t.analyzed,
        format: t.format.as_ref().map(|f| FormatDto {
            sample_rate: f.sample_rate,
            channels: f.channels,
            bits: f.bits,
        }),
        markers: MarkersDto {
            cue_in: m.cue_in.map(marker_dto),
            intro_end: m.intro_end.map(marker_dto),
            outro_start: m.outro_start.map(marker_dto),
            segue_start: m.segue_start.map(marker_dto),
            cue_out: m.cue_out.map(marker_dto),
        },
        loudness: t.loudness.map(|l| LoudnessDto {
            integrated_lufs: l.integrated_lufs,
            true_peak_dbtp: l.true_peak_dbtp,
        }),
        loudness_edit: LoudnessEditDto {
            excluded: t.loudness_edit.excluded,
            trim_db: t.loudness_edit.trim_db,
        },
    }
}

fn marker_dto(m: Marker) -> MarkerDto {
    MarkerDto {
        secs: m.secs,
        source: match m.source {
            MarkerSource::Auto => "auto",
            MarkerSource::Manual => "manual",
        },
    }
}

fn transport_name(t: Transport) -> &'static str {
    match t {
        Transport::Stopped => "stopped",
        Transport::Playing => "playing",
        Transport::Paused => "paused",
    }
}

pub(crate) fn mode_name(m: PlayMode) -> &'static str {
    match m {
        PlayMode::Single => "single",
        PlayMode::Continuous => "continuous",
    }
}

fn track_kind_name(k: TrackKind) -> &'static str {
    match k {
        TrackKind::Music => "music",
        TrackKind::Jingle => "jingle",
        TrackKind::Effect => "effect",
        TrackKind::Ad => "ad",
        TrackKind::Voice => "voice",
    }
}

fn cart_kind_name(k: CartKind) -> &'static str {
    match k {
        CartKind::Jingle => "jingle",
        CartKind::Effect => "effect",
        CartKind::Spot => "spot",
    }
}

fn file_state_name(s: FileState) -> &'static str {
    match s {
        FileState::Ok => "ok",
        FileState::Missing => "missing",
        FileState::Unreadable => "unreadable",
    }
}
