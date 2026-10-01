//! What changed between two snapshots, as resource events (remote control
//! spec §5.1). Each event carries the whole resource, not a patch. Pure: the
//! publisher in `server` runs it.

use std::collections::HashSet;

use fp_model::{AppState, EntryId, PlayerId, PlaylistId, TrackId, Transport};
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
/// The JSON is made once here, not once per client.
#[derive(Debug, Clone, PartialEq)]
pub struct Envelope {
    pub revision: u64,
    pub event: Event,
    pub json: String,
}

impl Envelope {
    pub fn new(revision: u64, event: Event) -> Self {
        let json = event.json();
        Self {
            revision,
            event,
            json,
        }
    }
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
/// out (they move all the time; `position` reports them). Only what changed
/// in the model is turned into DTOs, so the cost follows the change, not the
/// size of the playlists.
pub fn diff(old: &AppState, new: &AppState, playback: &Playback) -> Vec<Event> {
    let still = Playback::default();
    let ids = |s: &AppState| s.players.iter().map(|p| p.id).collect::<Vec<_>>();
    if ids(old) != ids(new) {
        return vec![Event::State(Box::new(dto::state(new, playback)))];
    }
    let mut out = Vec::new();
    // Entries whose on-air, next or cue marks may have moved.
    let mut marked: HashSet<EntryId> = HashSet::new();
    for (before, after) in old.players.iter().zip(&new.players) {
        let marks = |p: &fp_model::PlayerState| (p.current, p.next, p.cue, p.transport);
        if marks(before) != marks(after) {
            for p in [before, after] {
                marked.extend(
                    p.current
                        .into_iter()
                        .chain(p.next)
                        .chain(p.cue.map(|c| c.entry)),
                );
            }
        }
        if dto::player(old, &still, after.id) != dto::player(new, &still, after.id)
            && let Some(d) = dto::player(new, playback, after.id)
        {
            out.push(Event::Player(Box::new(d)));
        }
    }
    let changed_tracks: HashSet<TrackId> = new
        .library
        .iter()
        .filter(|t| old.library.get(t.id) != Some(*t))
        .map(|t| t.id)
        .collect();
    for list in new.playlists.iter() {
        let touched = old.playlists.get(list.id) != Some(list)
            || list
                .entries
                .iter()
                .any(|e| marked.contains(&e.id) || changed_tracks.contains(&e.track));
        if touched && let Some(d) = dto::playlist(new, list.id) {
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
    if !changed_tracks.is_empty() {
        let used: HashSet<TrackId> = new
            .playlists
            .iter()
            .flat_map(|l| l.entries.iter().map(|e| e.track))
            .chain(
                new.cartwall
                    .pages
                    .iter()
                    .flat_map(|p| p.carts.iter().filter_map(|c| c.track)),
            )
            .collect();
        for id in changed_tracks.intersection(&used) {
            if let Some(d) = dto::track(new, *id) {
                out.push(Event::Track(Box::new(d)));
            }
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
