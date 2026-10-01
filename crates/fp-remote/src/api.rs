//! Requests that act (remote control spec §3.3), turned into the model's
//! commands. They are checked as the console checks its buttons
//! (availability, R28), then applied to a copy of the snapshot, so a refusal
//! is answered at once instead of being queued. The player rules stay in
//! `fp-model`.

use fp_model::volume::gain_from_fader;
use fp_model::{
    AppState, CartId, CartPageId, Command, EntryId, ModelError, PlayMode, PlayerId, PlaylistId,
    Transport, command_available,
};

/// One action of a remote client.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operation {
    Play(PlayerId),
    Pause(PlayerId),
    Stop(PlayerId),
    FadeStop(PlayerId),
    Restart(PlayerId),
    Previous(PlayerId),
    SetCue(PlayerId, bool),
    SetNext(PlayerId, EntryId),
    CueEntry(PlayerId, EntryId),
    Seek(PlayerId, f64),
    /// Fader travel 0–1.
    SetFader(PlayerId, f32),
    SetMode(PlayerId, PlayMode),
    SetStopAfterCurrent(PlayerId, bool),
    ShowPlaylist(PlayerId, PlaylistId),
    SetEntryRepeat(EntryId, bool),
    SetEntryStopAfter(EntryId, bool),
    FireCart(CartId),
    StopCart(CartId),
    SetCartCue(CartId, bool),
    StopAllCarts,
    ShowCartPage(CartPageId),
}

/// Why a request was refused (remote control spec §3.5).
#[derive(Debug, Clone, PartialEq)]
pub enum ApiError {
    BadRequest(String),
    Unauthorized,
    ForbiddenOrigin,
    NotFound,
    NotAnalyzed,
    Unavailable(String),
    PayloadTooLarge,
    UnsupportedMediaType,
    Busy,
}

impl ApiError {
    pub fn status(&self) -> u16 {
        match self {
            Self::BadRequest(_) => 400,
            Self::Unauthorized => 401,
            Self::ForbiddenOrigin => 403,
            Self::NotFound | Self::NotAnalyzed => 404,
            Self::Unavailable(_) => 409,
            Self::PayloadTooLarge => 413,
            Self::UnsupportedMediaType => 415,
            Self::Busy => 503,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "bad_request",
            Self::Unauthorized => "unauthorized",
            Self::ForbiddenOrigin => "forbidden_origin",
            Self::NotFound => "not_found",
            Self::NotAnalyzed => "not_analyzed",
            Self::Unavailable(_) => "unavailable",
            Self::PayloadTooLarge => "payload_too_large",
            Self::UnsupportedMediaType => "unsupported_media_type",
            Self::Busy => "busy",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::BadRequest(m) | Self::Unavailable(m) => m.clone(),
            Self::Unauthorized => "a valid token is required".to_owned(),
            Self::ForbiddenOrigin => "this origin or host is not allowed".to_owned(),
            Self::NotFound => "no such resource".to_owned(),
            Self::NotAnalyzed => "the track has not been analysed yet".to_owned(),
            Self::PayloadTooLarge => "the request body is too large".to_owned(),
            Self::UnsupportedMediaType => "the body must be application/json".to_owned(),
            Self::Busy => "the player is busy; try again".to_owned(),
        }
    }

    /// An unknown id is `NotFound`; any other refusal is a conflict that
    /// carries the model's reason.
    pub fn from_model(e: ModelError) -> Self {
        match e {
            ModelError::UnknownPlayer(_)
            | ModelError::UnknownPlaylist(_)
            | ModelError::UnknownEntry(_)
            | ModelError::UnknownTrack(_)
            | ModelError::UnknownCart(_)
            | ModelError::UnknownCartPage(_)
            | ModelError::UnknownCartPosition(_) => Self::NotFound,
            other => Self::Unavailable(other.to_string()),
        }
    }
}

/// The commands for `op` in `state`, already known to be accepted by it.
pub fn plan(state: &AppState, op: Operation) -> Result<Vec<Command>, ApiError> {
    let commands = commands_for(state, op)?;
    dry_run(state, &commands)?;
    Ok(commands)
}

fn commands_for(state: &AppState, op: Operation) -> Result<Vec<Command>, ApiError> {
    use Operation as O;
    let player = |id: PlayerId| state.player(id).map_err(|_| ApiError::NotFound);
    let entry = |id: EntryId| state.playlists.entry(id).ok_or(ApiError::NotFound);
    let cart = |id: CartId| state.cartwall.cart(id).ok_or(ApiError::NotFound);
    let toggle = |current: bool, wanted: bool, command: Command| {
        if current == wanted {
            Ok(Vec::new())
        } else {
            available(state, command)
        }
    };
    match op {
        O::Play(p) => player(p).and_then(|_| available(state, Command::Play(p))),
        O::Pause(p) => player(p).and_then(|_| available(state, Command::Pause(p))),
        O::Stop(p) => player(p).and_then(|_| available(state, Command::Stop(p))),
        O::FadeStop(p) => player(p).and_then(|_| available(state, Command::FadeStop(p))),
        O::Restart(p) => player(p).and_then(|_| available(state, Command::Restart(p))),
        O::Previous(p) => player(p).and_then(|_| available(state, Command::Previous(p))),
        O::SetCue(p, on) => toggle(player(p)?.cue.is_some(), on, Command::ToggleCue(p)),
        O::SetStopAfterCurrent(p, on) => toggle(
            player(p)?.stop_after_current,
            on,
            Command::ToggleStopAfterCurrent(p),
        ),
        O::SetNext(p, e) => {
            player(p)?;
            entry(e)?;
            Ok(vec![Command::SetNext(p, e)])
        }
        O::CueEntry(p, e) => {
            player(p)?;
            entry(e)?;
            Ok(vec![Command::CueEntry(p, e)])
        }
        O::Seek(p, secs) => {
            let pl = player(p)?;
            let track = pl
                .current
                .filter(|_| pl.transport != Transport::Stopped)
                .and_then(|e| state.track_for_entry(e))
                .ok_or_else(|| ApiError::Unavailable("nothing is playing".to_owned()))?;
            let (from, to) = (track.cue_in_secs(), track.cue_out_secs());
            if !(secs >= from && secs <= to) {
                return Err(ApiError::BadRequest(format!(
                    "secs must be within {from}..={to}"
                )));
            }
            Ok(vec![Command::Seek(p, secs)])
        }
        O::SetFader(p, fader) => {
            player(p)?;
            if !(0.0..=1.0).contains(&fader) {
                return Err(ApiError::BadRequest(
                    "fader must be within 0..=1".to_owned(),
                ));
            }
            Ok(vec![Command::SetVolume(p, gain_from_fader(fader))])
        }
        O::SetMode(p, mode) => {
            player(p)?;
            Ok(vec![Command::SetMode(p, mode)])
        }
        O::ShowPlaylist(p, list) => {
            player(p)?;
            state.playlists.get(list).ok_or(ApiError::NotFound)?;
            Ok(vec![Command::ShowPlaylist(p, list)])
        }
        O::SetEntryRepeat(e, on) => toggle(entry(e)?.repeat, on, Command::ToggleEntryRepeat(e)),
        O::SetEntryStopAfter(e, on) => {
            toggle(entry(e)?.stop_after, on, Command::ToggleEntryStopAfter(e))
        }
        O::FireCart(c) => cart(c).map(|_| vec![Command::FireCart(c)]),
        O::StopCart(c) => cart(c).map(|_| vec![Command::StopCart(c)]),
        O::SetCartCue(c, on) => {
            cart(c)?;
            toggle(state.cartwall.cue == Some(c), on, Command::CueCart(c))
        }
        O::StopAllCarts => Ok(vec![Command::StopAllCarts]),
        O::ShowCartPage(page) => {
            state.cartwall.page(page).ok_or(ApiError::NotFound)?;
            Ok(vec![Command::ShowCartPage(page)])
        }
    }
}

/// `command` alone, unless the console would grey its button out (R28).
fn available(state: &AppState, command: Command) -> Result<Vec<Command>, ApiError> {
    if command_available(state, &command) {
        Ok(vec![command])
    } else {
        Err(ApiError::Unavailable("not available now".to_owned()))
    }
}

/// Applies `commands` to a copy of `state`; the first refusal is the answer.
fn dry_run(state: &AppState, commands: &[Command]) -> Result<(), ApiError> {
    if commands.is_empty() {
        return Ok(());
    }
    let mut copy = state.clone();
    for command in commands {
        fp_model::apply(&mut copy, command.clone()).map_err(ApiError::from_model)?;
    }
    Ok(())
}
