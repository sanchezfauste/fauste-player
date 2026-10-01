//! One handler per route of remote control spec §3.2–3.3.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use fp_model::{
    CartEdit, CartId, CartKind, CartPageId, EntryId, MarkerKind, PlayMode, PlayerId, PlaylistId,
    TrackId,
};
use serde::Deserialize;
use serde_json::json;

use super::Ctx;
use super::json::JsonBody;
use crate::api::{self, ApiError, Edit, Operation};
use crate::dto;

type Reply = Result<Response, ApiError>;

/// A path id; anything that is not a `u64` names nothing.
fn id(raw: &str) -> Result<u64, ApiError> {
    raw.parse().map_err(|_| ApiError::NotFound)
}

fn ok<T: serde::Serialize>(value: T) -> Reply {
    Ok(Json(value).into_response())
}

/// Queues `commands` and answers `202` with the revision.
fn queue(ctx: &Ctx, commands: Vec<fp_model::Command>) -> Reply {
    for command in commands {
        if !ctx.control.send(command) {
            return Err(ApiError::Busy);
        }
    }
    let revision = ctx.control.playback().revision;
    Ok((StatusCode::ACCEPTED, Json(json!({ "revision": revision }))).into_response())
}

fn run(ctx: &Ctx, op: Operation) -> Reply {
    queue(ctx, api::plan(&ctx.control.model(), op)?)
}

fn run_edit(ctx: &Ctx, edit: Edit) -> Reply {
    queue(ctx, api::plan_edit(&ctx.control.model(), edit)?)
}

pub async fn state(State(ctx): State<Ctx>) -> Reply {
    ok(dto::state(&ctx.control.model(), &ctx.control.playback()))
}

pub async fn players(State(ctx): State<Ctx>) -> Reply {
    ok(dto::players(&ctx.control.model(), &ctx.control.playback()))
}

pub async fn player(State(ctx): State<Ctx>, Path(raw): Path<String>) -> Reply {
    let p = dto::player(
        &ctx.control.model(),
        &ctx.control.playback(),
        PlayerId(id(&raw)?),
    );
    ok(p.ok_or(ApiError::NotFound)?)
}

pub async fn playlists(State(ctx): State<Ctx>) -> Reply {
    ok(dto::playlists(&ctx.control.model()))
}

pub async fn playlist(State(ctx): State<Ctx>, Path(raw): Path<String>) -> Reply {
    let list = dto::playlist(&ctx.control.model(), PlaylistId(id(&raw)?));
    ok(list.ok_or(ApiError::NotFound)?)
}

pub async fn track(State(ctx): State<Ctx>, Path(raw): Path<String>) -> Reply {
    let t = dto::track(&ctx.control.model(), TrackId(id(&raw)?));
    ok(t.ok_or(ApiError::NotFound)?)
}

pub async fn cartwall(State(ctx): State<Ctx>) -> Reply {
    ok(dto::cartwall(&ctx.control.model(), &ctx.control.playback()))
}

/// The track, known to be analysed: cover and peaks exist only after that.
fn analysed(ctx: &Ctx, raw: &str) -> Result<TrackId, ApiError> {
    let t = TrackId(id(raw)?);
    let model = ctx.control.model();
    let track = model.library.get(t).ok_or(ApiError::NotFound)?;
    if track.analyzed {
        Ok(t)
    } else {
        Err(ApiError::NotAnalyzed)
    }
}

pub async fn cover(State(ctx): State<Ctx>, Path(raw): Path<String>) -> Reply {
    let t = analysed(&ctx, &raw)?;
    let control = ctx.control.clone();
    let png = tokio::task::spawn_blocking(move || control.cover(t))
        .await
        .ok()
        .flatten()
        .ok_or(ApiError::NotFound)?;
    Ok(([(header::CONTENT_TYPE, "image/png")], png).into_response())
}

pub async fn peaks(State(ctx): State<Ctx>, Path(raw): Path<String>) -> Reply {
    let t = analysed(&ctx, &raw)?;
    let control = ctx.control.clone();
    let w = tokio::task::spawn_blocking(move || control.peaks(t))
        .await
        .ok()
        .flatten()
        .ok_or(ApiError::NotFound)?;
    ok(json!({ "bucket_secs": w.bucket_secs, "full_scale": i16::MAX, "peaks": w.peaks }))
}

pub async fn player_action(
    State(ctx): State<Ctx>,
    Path((raw, action)): Path<(String, String)>,
) -> Reply {
    let p = PlayerId(id(&raw)?);
    let op = match action.as_str() {
        "play" => Operation::Play(p),
        "pause" => Operation::Pause(p),
        "stop" => Operation::Stop(p),
        "fade-stop" => Operation::FadeStop(p),
        "restart" => Operation::Restart(p),
        "previous" => Operation::Previous(p),
        _ => return Err(ApiError::NotFound),
    };
    run(&ctx, op)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct On {
    on: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    entry: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Seek {
    secs: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fader {
    fader: f32,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModeName {
    Single,
    Continuous,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mode {
    mode: ModeName,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlaylistRef {
    playlist: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageRef {
    page: u64,
}

pub async fn set_cue(
    State(ctx): State<Ctx>,
    Path(raw): Path<String>,
    JsonBody(b): JsonBody<On>,
) -> Reply {
    run(&ctx, Operation::SetCue(PlayerId(id(&raw)?), b.on))
}

pub async fn set_next(
    State(ctx): State<Ctx>,
    Path(raw): Path<String>,
    JsonBody(b): JsonBody<Entry>,
) -> Reply {
    run(
        &ctx,
        Operation::SetNext(PlayerId(id(&raw)?), EntryId(b.entry)),
    )
}

pub async fn cue_entry(
    State(ctx): State<Ctx>,
    Path(raw): Path<String>,
    JsonBody(b): JsonBody<Entry>,
) -> Reply {
    run(
        &ctx,
        Operation::CueEntry(PlayerId(id(&raw)?), EntryId(b.entry)),
    )
}

pub async fn seek(
    State(ctx): State<Ctx>,
    Path(raw): Path<String>,
    JsonBody(b): JsonBody<Seek>,
) -> Reply {
    run(&ctx, Operation::Seek(PlayerId(id(&raw)?), b.secs))
}

pub async fn volume(
    State(ctx): State<Ctx>,
    Path(raw): Path<String>,
    JsonBody(b): JsonBody<Fader>,
) -> Reply {
    run(&ctx, Operation::SetFader(PlayerId(id(&raw)?), b.fader))
}

pub async fn mode(
    State(ctx): State<Ctx>,
    Path(raw): Path<String>,
    JsonBody(b): JsonBody<Mode>,
) -> Reply {
    let mode = match b.mode {
        ModeName::Single => PlayMode::Single,
        ModeName::Continuous => PlayMode::Continuous,
    };
    run(&ctx, Operation::SetMode(PlayerId(id(&raw)?), mode))
}

pub async fn stop_after_current(
    State(ctx): State<Ctx>,
    Path(raw): Path<String>,
    JsonBody(b): JsonBody<On>,
) -> Reply {
    run(
        &ctx,
        Operation::SetStopAfterCurrent(PlayerId(id(&raw)?), b.on),
    )
}

pub async fn show_playlist(
    State(ctx): State<Ctx>,
    Path(raw): Path<String>,
    JsonBody(b): JsonBody<PlaylistRef>,
) -> Reply {
    run(
        &ctx,
        Operation::ShowPlaylist(PlayerId(id(&raw)?), PlaylistId(b.playlist)),
    )
}

pub async fn entry_repeat(
    State(ctx): State<Ctx>,
    Path(raw): Path<String>,
    JsonBody(b): JsonBody<On>,
) -> Reply {
    run(&ctx, Operation::SetEntryRepeat(EntryId(id(&raw)?), b.on))
}

pub async fn entry_stop_after(
    State(ctx): State<Ctx>,
    Path(raw): Path<String>,
    JsonBody(b): JsonBody<On>,
) -> Reply {
    run(&ctx, Operation::SetEntryStopAfter(EntryId(id(&raw)?), b.on))
}

pub async fn fire_cart(State(ctx): State<Ctx>, Path(raw): Path<String>) -> Reply {
    run(&ctx, Operation::FireCart(CartId(id(&raw)?)))
}

pub async fn stop_cart(State(ctx): State<Ctx>, Path(raw): Path<String>) -> Reply {
    run(&ctx, Operation::StopCart(CartId(id(&raw)?)))
}

pub async fn cart_cue(
    State(ctx): State<Ctx>,
    Path(raw): Path<String>,
    JsonBody(b): JsonBody<On>,
) -> Reply {
    run(&ctx, Operation::SetCartCue(CartId(id(&raw)?), b.on))
}

pub async fn stop_all_carts(State(ctx): State<Ctx>) -> Reply {
    run(&ctx, Operation::StopAllCarts)
}

pub async fn show_cart_page(State(ctx): State<Ctx>, JsonBody(b): JsonBody<PageRef>) -> Reply {
    run(&ctx, Operation::ShowCartPage(CartPageId(b.page)))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Name {
    name: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InsertBody {
    track: u64,
    index: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MoveBody {
    playlist: u64,
    index: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageEdit {
    name: Option<String>,
    rows: Option<u16>,
    cols: Option<u16>,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CartKindName {
    Jingle,
    Effect,
    Spot,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CartBody {
    name: String,
    kind: CartKindName,
    looped: bool,
    exclusive: bool,
    track: Option<u64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkerBody {
    secs: Option<f64>,
}

pub async fn create_playlist(State(ctx): State<Ctx>, JsonBody(b): JsonBody<Name>) -> Reply {
    run_edit(&ctx, Edit::CreatePlaylist(b.name))
}
pub async fn rename_playlist(
    State(ctx): State<Ctx>,
    Path(raw): Path<String>,
    JsonBody(b): JsonBody<Name>,
) -> Reply {
    run_edit(&ctx, Edit::RenamePlaylist(PlaylistId(id(&raw)?), b.name))
}
pub async fn delete_playlist(State(ctx): State<Ctx>, Path(raw): Path<String>) -> Reply {
    run_edit(&ctx, Edit::DeletePlaylist(PlaylistId(id(&raw)?)))
}
pub async fn insert_entry(
    State(ctx): State<Ctx>,
    Path(raw): Path<String>,
    JsonBody(b): JsonBody<InsertBody>,
) -> Reply {
    run_edit(
        &ctx,
        Edit::InsertTrack {
            playlist: PlaylistId(id(&raw)?),
            index: b.index,
            track: TrackId(b.track),
        },
    )
}
pub async fn remove_entry(State(ctx): State<Ctx>, Path(raw): Path<String>) -> Reply {
    run_edit(&ctx, Edit::RemoveEntry(EntryId(id(&raw)?)))
}
pub async fn move_entry(
    State(ctx): State<Ctx>,
    Path(raw): Path<String>,
    JsonBody(b): JsonBody<MoveBody>,
) -> Reply {
    run_edit(
        &ctx,
        Edit::MoveEntry {
            entry: EntryId(id(&raw)?),
            playlist: PlaylistId(b.playlist),
            index: b.index,
        },
    )
}
pub async fn duplicate_entry(State(ctx): State<Ctx>, Path(raw): Path<String>) -> Reply {
    run_edit(&ctx, Edit::DuplicateEntry(EntryId(id(&raw)?)))
}
pub async fn create_cart_page(State(ctx): State<Ctx>, JsonBody(b): JsonBody<Name>) -> Reply {
    run_edit(&ctx, Edit::CreateCartPage(b.name))
}
pub async fn edit_cart_page(
    State(ctx): State<Ctx>,
    Path(raw): Path<String>,
    JsonBody(b): JsonBody<PageEdit>,
) -> Reply {
    run_edit(
        &ctx,
        Edit::EditCartPage {
            page: CartPageId(id(&raw)?),
            name: b.name,
            rows: b.rows,
            cols: b.cols,
        },
    )
}
pub async fn delete_cart_page(State(ctx): State<Ctx>, Path(raw): Path<String>) -> Reply {
    run_edit(&ctx, Edit::DeleteCartPage(CartPageId(id(&raw)?)))
}
pub async fn set_cart(
    State(ctx): State<Ctx>,
    Path((raw, index)): Path<(String, String)>,
    JsonBody(b): JsonBody<CartBody>,
) -> Reply {
    let index: usize = index.parse().map_err(|_| ApiError::NotFound)?;
    let kind = match b.kind {
        CartKindName::Jingle => CartKind::Jingle,
        CartKindName::Effect => CartKind::Effect,
        CartKindName::Spot => CartKind::Spot,
    };
    let edit = CartEdit {
        name: b.name,
        kind,
        looped: b.looped,
        exclusive: b.exclusive,
    };
    run_edit(
        &ctx,
        Edit::SetCart {
            page: CartPageId(id(&raw)?),
            index,
            edit,
            track: b.track.map(TrackId),
        },
    )
}
pub async fn set_marker(
    State(ctx): State<Ctx>,
    Path((raw, kind)): Path<(String, String)>,
    JsonBody(b): JsonBody<MarkerBody>,
) -> Reply {
    let kind = match kind.as_str() {
        "cue-in" => MarkerKind::CueIn,
        "intro-end" => MarkerKind::IntroEnd,
        "outro-start" => MarkerKind::OutroStart,
        "segue-start" => MarkerKind::SegueStart,
        "cue-out" => MarkerKind::CueOut,
        _ => return Err(ApiError::NotFound),
    };
    run_edit(
        &ctx,
        Edit::SetMarker {
            track: TrackId(id(&raw)?),
            kind,
            secs: b.secs,
        },
    )
}
pub async fn reset_markers(State(ctx): State<Ctx>, Path(raw): Path<String>) -> Reply {
    run_edit(&ctx, Edit::ResetMarkers(TrackId(id(&raw)?)))
}

pub async fn not_found() -> ApiError {
    ApiError::NotFound
}
