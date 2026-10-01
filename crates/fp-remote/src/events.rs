//! What changed between two snapshots, as resource events (remote control
//! spec §5.1). Each event carries the whole resource, not a patch. Pure: the
//! publisher in `server` runs it.

use fp_model::{AppState, PlayerId, PlaylistId, Transport};
use serde::Serialize;

use crate::control::Playback;
use crate::dto::{self, CartwallDto, PlayerDto, PlayingCartDto, PlaylistDto, StateDto, TrackDto};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlayerTimesDto {
    pub id: PlayerId,
    pub elapsed_secs: Option<f64>,
    pub remaining_secs: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PositionDto {
    pub players: Vec<PlayerTimesDto>,
    pub carts: Vec<PlayingCartDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RemovedDto {
    pub id: PlaylistId,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    State(Box<StateDto>),
    Player(Box<PlayerDto>),
    Playlist(Box<PlaylistDto>),
    PlaylistRemoved(RemovedDto),
    Cartwall(Box<CartwallDto>),
    Track(Box<TrackDto>),
    Position(PositionDto),
}

/// An event with the revision it belongs to, as the publisher sends it.
#[derive(Debug, Clone, PartialEq)]
pub struct Envelope {
    pub revision: u64,
    pub event: Event,
}

impl Event {
    pub fn name(&self) -> &'static str {
        match self {
            Self::State(_) => "state",
            Self::Player(_) => "player",
            Self::Playlist(_) => "playlist",
            Self::PlaylistRemoved(_) => "playlist-removed",
            Self::Cartwall(_) => "cartwall",
            Self::Track(_) => "track",
            Self::Position(_) => "position",
        }
    }

    pub fn json(&self) -> String {
        let value = match self {
            Self::State(d) => serde_json::to_string(d),
            Self::Player(d) => serde_json::to_string(d),
            Self::Playlist(d) => serde_json::to_string(d),
            Self::PlaylistRemoved(d) => serde_json::to_string(d),
            Self::Cartwall(d) => serde_json::to_string(d),
            Self::Track(d) => serde_json::to_string(d),
            Self::Position(d) => serde_json::to_string(d),
        };
        value.unwrap_or_default()
    }
}

/// The events that take a client from `old` to `new`. Positions are left
/// out (they move all the time; `position` reports them).
pub fn diff(old: &AppState, new: &AppState, playback: &Playback) -> Vec<Event> {
    let still = Playback::default();
    let ids = |s: &AppState| s.players.iter().map(|p| p.id).collect::<Vec<_>>();
    if ids(old) != ids(new) {
        return vec![Event::State(Box::new(dto::state(new, playback)))];
    }
    let mut out = Vec::new();
    for p in &new.players {
        if dto::player(old, &still, p.id) != dto::player(new, &still, p.id)
            && let Some(d) = dto::player(new, playback, p.id)
        {
            out.push(Event::Player(Box::new(d)));
        }
    }
    for list in new.playlists.iter() {
        let after = dto::playlist(new, list.id);
        if dto::playlist(old, list.id) != after
            && let Some(d) = after
        {
            out.push(Event::Playlist(Box::new(d)));
        }
    }
    for list in old.playlists.iter() {
        if new.playlists.get(list.id).is_none() {
            out.push(Event::PlaylistRemoved(RemovedDto { id: list.id }));
        }
    }
    if dto::cartwall(old, &still) != dto::cartwall(new, &still) {
        out.push(Event::Cartwall(Box::new(dto::cartwall(new, playback))));
    }
    for t in new.library.iter() {
        let used = new.playlists.references_track(t.id) || new.cartwall.references_track(t.id);
        let after = dto::track(new, t.id);
        if used
            && dto::track(old, t.id) != after
            && let Some(d) = after
        {
            out.push(Event::Track(Box::new(d)));
        }
    }
    out
}

/// Elapsed and remaining times of every playing player and cart; `None`
/// when nothing plays.
pub fn position(model: &AppState, playback: &Playback) -> Option<PositionDto> {
    let players: Vec<PlayerTimesDto> = model
        .players
        .iter()
        .filter(|p| p.transport == Transport::Playing)
        .filter_map(|p| dto::player(model, playback, p.id))
        .map(|d| PlayerTimesDto {
            id: d.id,
            elapsed_secs: d.elapsed_secs,
            remaining_secs: d.remaining_secs,
        })
        .collect();
    let carts = dto::cartwall(model, playback).playing;
    (!players.is_empty() || !carts.is_empty()).then_some(PositionDto { players, carts })
}
