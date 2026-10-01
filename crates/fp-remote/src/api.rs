//! Requests that act (remote control spec §3.3), turned into the model's
//! commands. They are checked as the console checks its buttons
//! (availability, R28), then applied to a copy of the snapshot, so a refusal
//! is answered at once instead of being queued. The player rules stay in
//! `fp-model`.

use fp_model::volume::gain_from_fader;
use fp_model::{
    AppState, CartEdit, CartId, CartPageId, Command, EntryId, MarkerKind, ModelError, PlayMode,
    PlayerId, PlaylistId, TrackId, Transport, command_available,
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
    // A value already in place sends nothing. Otherwise the toggle's
    // availability decides, and the idempotent `Set…` command is sent, so
    // two requests planned from one snapshot cannot cancel each other.
    let toggle = |current: bool, wanted: bool, toggle: Command, set: Command| {
        if current == wanted {
            Ok(Vec::new())
        } else {
            available(state, toggle).map(|_| vec![set])
        }
    };
    match op {
        O::Play(p) => player(p).and_then(|_| available(state, Command::Play(p))),
        O::Pause(p) => player(p).and_then(|_| available(state, Command::Pause(p))),
        O::Stop(p) => player(p).and_then(|_| available(state, Command::Stop(p))),
        O::FadeStop(p) => player(p).and_then(|_| available(state, Command::FadeStop(p))),
        O::Restart(p) => player(p).and_then(|_| available(state, Command::Restart(p))),
        O::Previous(p) => player(p).and_then(|_| available(state, Command::Previous(p))),
        O::SetCue(p, on) => toggle(
            player(p)?.cue.is_some(),
            on,
            Command::ToggleCue(p),
            Command::SetCue(p, on),
        ),
        O::SetStopAfterCurrent(p, on) => toggle(
            player(p)?.stop_after_current,
            on,
            Command::ToggleStopAfterCurrent(p),
            Command::SetStopAfterCurrent(p, on),
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
        O::SetEntryRepeat(e, on) => toggle(
            entry(e)?.repeat,
            on,
            Command::ToggleEntryRepeat(e),
            Command::SetEntryRepeat(e, on),
        ),
        O::SetEntryStopAfter(e, on) => toggle(
            entry(e)?.stop_after,
            on,
            Command::ToggleEntryStopAfter(e),
            Command::SetEntryStopAfter(e, on),
        ),
        O::FireCart(c) => cart(c).map(|_| vec![Command::FireCart(c)]),
        O::StopCart(c) => cart(c).map(|_| vec![Command::StopCart(c)]),
        O::SetCartCue(c, on) => {
            cart(c)?;
            toggle(
                state.cartwall.cue == Some(c),
                on,
                Command::CueCart(c),
                Command::SetCartCue(c, on),
            )
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

/// One editing action of a remote client (remote control spec §3.4).
#[derive(Debug, Clone, PartialEq)]
pub enum Edit {
    CreatePlaylist(String),
    RenamePlaylist(PlaylistId, String),
    DeletePlaylist(PlaylistId),
    InsertTrack {
        playlist: PlaylistId,
        index: usize,
        track: TrackId,
    },
    RemoveEntry(EntryId),
    MoveEntry {
        entry: EntryId,
        playlist: PlaylistId,
        index: usize,
    },
    DuplicateEntry(EntryId),
    CreateCartPage(String),
    EditCartPage {
        page: CartPageId,
        name: Option<String>,
        rows: Option<u16>,
        cols: Option<u16>,
    },
    DeleteCartPage(CartPageId),
    SetCart {
        page: CartPageId,
        index: usize,
        edit: CartEdit,
        track: Option<TrackId>,
    },
    SetMarker {
        track: TrackId,
        kind: MarkerKind,
        secs: Option<f64>,
    },
    ResetMarkers(TrackId),
}

/// The commands for `edit` in `state`, already known to be accepted by it.
pub fn plan_edit(state: &AppState, edit: Edit) -> Result<Vec<Command>, ApiError> {
    let commands = edit_commands(state, edit)?;
    dry_run(state, &commands)?;
    Ok(commands)
}

fn name(raw: &str) -> Result<String, ApiError> {
    let name = raw.trim();
    if name.is_empty() {
        Err(ApiError::BadRequest(
            "the name must not be empty".to_owned(),
        ))
    } else {
        Ok(name.to_owned())
    }
}

fn edit_commands(state: &AppState, edit: Edit) -> Result<Vec<Command>, ApiError> {
    let list_len = |id: PlaylistId| {
        state
            .playlists
            .get(id)
            .map(|l| l.entries.len())
            .ok_or(ApiError::NotFound)
    };
    let track = |id: TrackId| state.library.get(id).ok_or(ApiError::NotFound);
    let page = |id: CartPageId| state.cartwall.page(id).ok_or(ApiError::NotFound);
    Ok(match edit {
        Edit::CreatePlaylist(n) => vec![Command::CreatePlaylist { name: name(&n)? }],
        Edit::RenamePlaylist(playlist, n) => {
            list_len(playlist)?;
            vec![Command::RenamePlaylist {
                playlist,
                name: name(&n)?,
            }]
        }
        Edit::DeletePlaylist(playlist) => {
            list_len(playlist)?;
            vec![Command::DeletePlaylist(playlist)]
        }
        Edit::InsertTrack {
            playlist,
            index,
            track: t,
        } => {
            let len = list_len(playlist)?;
            track(t)?;
            vec![Command::InsertTracks {
                playlist,
                index: index.min(len),
                tracks: vec![t],
            }]
        }
        Edit::RemoveEntry(entry) => {
            state.playlists.entry(entry).ok_or(ApiError::NotFound)?;
            vec![Command::RemoveEntry(entry)]
        }
        Edit::MoveEntry {
            entry,
            playlist,
            index,
        } => {
            state.playlists.entry(entry).ok_or(ApiError::NotFound)?;
            let len = list_len(playlist)?;
            vec![Command::MoveEntry {
                entry,
                to: playlist,
                index: index.min(len),
            }]
        }
        Edit::DuplicateEntry(entry) => {
            state.playlists.entry(entry).ok_or(ApiError::NotFound)?;
            vec![Command::DuplicateEntry(entry)]
        }
        Edit::CreateCartPage(n) => vec![Command::CreateCartPage { name: name(&n)? }],
        Edit::EditCartPage {
            page: id,
            name: new_name,
            rows,
            cols,
        } => {
            let p = page(id)?;
            if new_name.is_none() && rows.is_none() && cols.is_none() {
                return Err(ApiError::BadRequest("nothing to change".to_owned()));
            }
            let mut out = Vec::new();
            if let Some(n) = new_name {
                out.push(Command::RenameCartPage {
                    page: id,
                    name: name(&n)?,
                });
            }
            if rows.is_some() || cols.is_some() {
                let limits = &state.config.limits;
                let rows = rows.unwrap_or(p.rows);
                let cols = cols.unwrap_or(p.cols);
                if !(1..=limits.max_cart_rows).contains(&rows)
                    || !(1..=limits.max_cart_cols).contains(&cols)
                {
                    return Err(ApiError::BadRequest(format!(
                        "rows must be within 1..={} and cols within 1..={}",
                        limits.max_cart_rows, limits.max_cart_cols
                    )));
                }
                out.push(Command::ResizeCartPage {
                    page: id,
                    rows,
                    cols,
                });
            }
            out
        }
        Edit::DeleteCartPage(id) => {
            page(id)?;
            vec![Command::DeleteCartPage(id)]
        }
        Edit::SetCart {
            page: id,
            index,
            edit,
            track: wanted,
        } => {
            let cart = page(id)?.carts.get(index).ok_or(ApiError::NotFound)?;
            let mut out = vec![Command::SetCart {
                page: id,
                index,
                edit,
            }];
            match wanted {
                Some(t) if cart.track != Some(t) => {
                    track(t)?;
                    out.push(Command::AssignCartTrack {
                        page: id,
                        index,
                        track: t,
                    });
                }
                None if cart.track.is_some() => {
                    out.push(Command::ClearCartFile { page: id, index })
                }
                _ => {}
            }
            out
        }
        Edit::SetMarker {
            track: t,
            kind,
            secs,
        } => {
            track(t)?;
            if let Some(s) = secs
                && !(s.is_finite() && s >= 0.0)
            {
                return Err(ApiError::BadRequest(
                    "secs must be a position in the track".to_owned(),
                ));
            }
            vec![Command::SetMarker {
                track: t,
                kind,
                secs,
            }]
        }
        Edit::ResetMarkers(t) => {
            track(t)?;
            vec![Command::ResetMarkers { track: t }]
        }
    })
}
