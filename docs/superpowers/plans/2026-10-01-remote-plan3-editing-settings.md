# Remote Plan 3 — Editing, Settings > Remote and the README Screenshot Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:**
- Remote clients can edit playlists, cart pages and markers.
- The operator configures and watches remote control in Settings > Remote.
- The deferred hardening from plans 1 and 2 lands.
- The README shows the main screen with players on air.

**Architecture:**
- `fp-model` gains `Command::InsertTracks` and `Command::AssignCartTrack`.
  They reuse tracks already in the library, so manual markers and analysis
  are kept; a new path would create a new track.
- `fp-remote`:
  - `api::Edit` and `api::plan_edit`, with the same dry run as operations;
  - the §3.4 routes;
  - hardening: JSON 405, bind retry, a case-insensitive `Bearer`, cart
    times like the UI;
  - `RemoteHandle::status_cell()`.
- `fp-app`:
  - `ui/settings/remote.rs`;
  - `AppUi::with_remote_status`;
  - `remote::new_token` (getrandom + base64url);
  - strings in both locales.
- Docs and the screenshot.

**Tech Stack:** as before; `getrandom` 0.3 (MIT/Apache, already in the tree).

**Spec:** [`docs/superpowers/specs/2026-10-01-remote-control-design.md`](../specs/2026-10-01-remote-control-design.md) §3.4, §7 (token generation), §8, §10 plan 3, §11.

## Global Constraints

- Branch `feat/remote-editing-settings`. Gate before every commit (fmt,
  clippy `-D warnings`, the whole suite).
- The rules of plans 1 and 2 still hold.
- Every UI string is in `en-US` and `es-ES`. Settings does no I/O on the UI
  thread. It reads the remote status from an `ArcSwap`, and its changes go
  through `Command::UpdateConfig`.
- Names are trimmed and must not be empty (`400`). Rows and columns are
  within `limits.max_cart_rows` and `limits.max_cart_cols` (`400`). An index
  past the end is clamped to the end.

## Review Focus

- **Editing an entry or a cart that is on air:** deleting the playlist on
  air, or removing the current entry, answers `409` with the model's
  reason, and nothing is queued (Task 2 tests). Moving the current entry is
  allowed: rule 13 forbids only its removal (corrected in plan 4).
- **A cart edit that keeps the same track:** it does not stop the cart on
  air (C8 only applies to a new file) (Task 2 tests).
- **Text fields in Settings:** a half-typed bind address, or a short token,
  is not applied until the field loses focus. An invalid value falls back,
  and the field shows what was kept (Task 4 tests).
- **The port is busy at start and frees later:** the server starts within
  a few seconds, with no restart (Task 3 tests).
- **The token is visible on screen:** it is masked by default (Task 4
  tests).

---

### Task 1: model commands that reuse library tracks

**Files:**
- Modify: `crates/fp-model/src/command.rs`, `crates/fp-model/src/reducer.rs`,
  `crates/fp-model/src/cart_rules.rs`
- Test: `crates/fp-model/tests/reuse_tracks.rs`

**Interfaces:**
- Produces:
  - `Command::InsertTracks { playlist: PlaylistId, index: usize, tracks: Vec<TrackId> }`;
  - `Command::AssignCartTrack { page: CartPageId, index: usize, track: TrackId }`.

- [ ] **Step 1: Failing tests**

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Remote control spec §3.4: a client reuses tracks already loaded, so their
//! markers and analysis come along.

mod common;

use common::{entries, fixture};
use fp_model::{Command, MarkerKind, TrackId, apply};

#[test]
fn inserting_a_track_shares_it_with_its_markers() {
    let mut s = fixture(3);
    let t = s.playlists.entry(entries(&s)[0]).unwrap().track;
    apply(&mut s, Command::SetMarker { track: t, kind: MarkerKind::IntroEnd, secs: Some(5.0) }).unwrap();
    apply(&mut s, Command::CreatePlaylist { name: "B".into() }).unwrap();
    let b = s.playlists.iter().nth(1).unwrap().id;
    apply(&mut s, Command::InsertTracks { playlist: b, index: 0, tracks: vec![t] }).unwrap();
    let copy = &s.playlists.get(b).unwrap().entries[0];
    assert_eq!(copy.track, t, "the same track, not a new one");
    assert_eq!(s.library.get(t).unwrap().markers.intro_end.unwrap().secs, 5.0);
}

#[test]
fn inserting_an_unknown_track_is_refused() {
    let mut s = fixture(1);
    let list = s.playlists.first_id().unwrap();
    assert!(apply(&mut s, Command::InsertTracks { playlist: list, index: 0, tracks: vec![TrackId(999_999)] }).is_err());
    assert_eq!(s.playlists.get(list).unwrap().entries.len(), 1);
}

#[test]
fn a_cart_can_take_a_loaded_track() {
    let mut s = fixture(1);
    let t = s.playlists.entry(entries(&s)[0]).unwrap().track;
    let page = s.cartwall.pages[0].id;
    apply(&mut s, Command::AssignCartTrack { page, index: 2, track: t }).unwrap();
    assert_eq!(s.cartwall.pages[0].carts[2].track, Some(t));
    // Replacing it keeps the track: the playlist still uses it.
    apply(&mut s, Command::ClearCartFile { page, index: 2 }).unwrap();
    assert!(s.library.get(t).is_some());
    assert!(apply(&mut s, Command::AssignCartTrack { page, index: 2, track: TrackId(999_999) }).is_err());
}
```

Run: `cargo test -p fp-model --test reuse_tracks` → compile errors.

- [ ] **Step 2: Implement**

In `command.rs`, after `InsertPaths`:

```rust
    /// Entries for tracks already in the library (remote control spec
    /// §3.4): the same tracks, with their markers and analysis.
    InsertTracks {
        playlist: PlaylistId,
        index: usize,
        tracks: Vec<TrackId>,
    },
```

and after `AssignCartFile`:

```rust
    /// Gives a cart a track already in the library. Stops it if playing (C8).
    AssignCartTrack {
        page: CartPageId,
        index: usize,
        track: TrackId,
    },
```

In `reducer.rs`:

```rust
        Command::InsertTracks { playlist, index, tracks } => {
            if state.playlists.get(playlist).is_none() {
                return Err(ModelError::UnknownPlaylist(playlist));
            }
            if let Some(missing) = tracks.iter().find(|t| state.library.get(**t).is_none()) {
                return Err(ModelError::UnknownTrack(*missing));
            }
            let entries = tracks
                .into_iter()
                .map(|t| PlaylistEntry::new(state.ids.entry(), t))
                .collect();
            state.playlists.insert(playlist, index, entries)?;
            refresh_next(state);
        }
```

```rust
        Command::AssignCartTrack { page, index, track } => {
            cart_rules::assign_track(state, page, index, track, &mut out)?
        }
```

In `cart_rules.rs`, `assign_file` is split so both share it:

```rust
pub(crate) fn assign_file(
    state: &mut AppState,
    page: CartPageId,
    index: usize,
    path: Option<PathBuf>,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    cart_at(state, page, index)?;
    let track = path.map(|path| {
        let track = state.ids.track();
        state.library.insert(Track::new(track, path));
        track
    });
    set_track(state, page, index, track, out)
}

pub(crate) fn assign_track(
    state: &mut AppState,
    page: CartPageId,
    index: usize,
    track: TrackId,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    if state.library.get(track).is_none() {
        return Err(ModelError::UnknownTrack(track));
    }
    set_track(state, page, index, Some(track), out)
}

/// Stops the cart, gives it `track` and forgets the old one if unused (C8).
fn set_track(
    state: &mut AppState,
    page: CartPageId,
    index: usize,
    track: Option<TrackId>,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let (id, old) = {
        let cart = cart_at(state, page, index)?;
        (cart.id, cart.track)
    };
    stop_playing(state, id, out);
    if state.cartwall.cue == Some(id) {
        stop_cue(state, out);
    }
    cart_at_mut(state, page, index)?.track = track;
    forget_tracks(state, old);
    Ok(())
}
```

Check that `forget_tracks` only removes tracks nothing references (the
`a_cart_can_take_a_loaded_track` test pins this). Make the
`fp-app` and `fp-control` matches over `Command` exhaustive, if any are.

- [ ] **Step 3: Run** `cargo test -p fp-model` → all pass.
- [ ] **Step 4: Gate and commit** `feat(model): insert and assign tracks already in the library`.

---

### Task 2: editing routes (`api::Edit`)

**Files:**
- Modify: `crates/fp-remote/src/api.rs`, `crates/fp-remote/src/http/handlers.rs`,
  `crates/fp-remote/src/http/mod.rs`
- Test: `crates/fp-remote/tests/edit.rs`

**Interfaces:**
- Produces:
  - `api::Edit` (variants below);
  - `api::plan_edit(&AppState, Edit) -> Result<Vec<Command>, ApiError>`;
  - the routes of spec §3.4.

- [ ] **Step 1: Failing tests** (`crates/fp-remote/tests/edit.rs`)

```rust
#![allow(clippy::unwrap_used)]
mod support;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use fp_model::{CartEdit, CartKind, Command, HttpRemoteConfig, MarkerKind, TrackId};
use fp_remote::api::{ApiError, Edit, plan_edit};
use fp_remote::http::{Ctx, router};
use serde_json::{Value, json};
use support::{FakeControl, demo_state};
use tower::ServiceExt;

#[test]
fn names_are_trimmed_and_must_not_be_empty() {
    let s = demo_state();
    assert_eq!(
        plan_edit(&s, Edit::CreatePlaylist("  Late ".into())).unwrap(),
        vec![Command::CreatePlaylist { name: "Late".into() }]
    );
    assert_eq!(plan_edit(&s, Edit::CreatePlaylist("  ".into())).unwrap_err().status(), 400);
}

#[test]
fn a_track_is_inserted_by_id_with_the_index_clamped() {
    let s = demo_state();
    let night = s.playlists.iter().nth(1).unwrap().id;
    let t = s.playlists.iter().next().unwrap().entries[0].track;
    assert_eq!(
        plan_edit(&s, Edit::InsertTrack { playlist: night, index: 99, track: t }).unwrap(),
        vec![Command::InsertTracks { playlist: night, index: 0, tracks: vec![t] }]
    );
    assert_eq!(
        plan_edit(&s, Edit::InsertTrack { playlist: night, index: 0, track: TrackId(999_999) }).unwrap_err(),
        ApiError::NotFound
    );
}

#[test]
fn the_playlist_on_air_cannot_be_deleted() {
    let mut s = demo_state();
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    let main = s.playlists.first_id().unwrap();
    let e = plan_edit(&s, Edit::DeletePlaylist(main)).unwrap_err();
    assert_eq!(e.status(), 409, "{e:?}");
    let current = s.players[0].current.unwrap();
    assert_eq!(plan_edit(&s, Edit::RemoveEntry(current)).unwrap_err().status(), 409);
}

#[test]
fn moving_clamps_into_the_target_playlist() {
    let s = demo_state();
    let e = s.playlists.iter().next().unwrap().entries[2].id;
    let night = s.playlists.iter().nth(1).unwrap().id;
    assert_eq!(
        plan_edit(&s, Edit::MoveEntry { entry: e, playlist: night, index: 50 }).unwrap(),
        vec![Command::MoveEntry { entry: e, to: night, index: 0 }]
    );
}

#[test]
fn cart_pages_are_renamed_and_resized_within_limits() {
    let s = demo_state();
    let page = s.cartwall.pages[0].id;
    let (rows, cols) = (s.cartwall.pages[0].rows, s.cartwall.pages[0].cols);
    assert_eq!(
        plan_edit(&s, Edit::EditCartPage { page, name: Some("Jingles".into()), rows: None, cols: None }).unwrap(),
        vec![Command::RenameCartPage { page, name: "Jingles".into() }]
    );
    assert_eq!(
        plan_edit(&s, Edit::EditCartPage { page, name: None, rows: Some(rows + 1), cols: None }).unwrap(),
        vec![Command::ResizeCartPage { page, rows: rows + 1, cols }]
    );
    let too_big = s.config.limits.max_cart_rows + 1;
    assert_eq!(
        plan_edit(&s, Edit::EditCartPage { page, name: None, rows: Some(too_big), cols: None }).unwrap_err().status(),
        400
    );
    assert_eq!(
        plan_edit(&s, Edit::EditCartPage { page, name: None, rows: None, cols: None }).unwrap_err().status(),
        400
    );
}

#[test]
fn a_cart_edit_keeping_its_track_does_not_reassign_it() {
    let s = demo_state();
    let page = s.cartwall.pages[0].id;
    let t = s.cartwall.pages[0].carts[0].track.unwrap();
    let edit = CartEdit { name: "Top".into(), kind: CartKind::Spot, looped: false, exclusive: true };
    assert_eq!(
        plan_edit(&s, Edit::SetCart { page, index: 0, edit: edit.clone(), track: Some(t) }).unwrap(),
        vec![Command::SetCart { page, index: 0, edit: edit.clone() }]
    );
    let other = s.playlists.iter().next().unwrap().entries[1].track;
    assert_eq!(
        plan_edit(&s, Edit::SetCart { page, index: 0, edit: edit.clone(), track: Some(other) }).unwrap(),
        vec![
            Command::SetCart { page, index: 0, edit: edit.clone() },
            Command::AssignCartTrack { page, index: 0, track: other },
        ]
    );
    assert_eq!(
        plan_edit(&s, Edit::SetCart { page, index: 0, edit: edit.clone(), track: None }).unwrap(),
        vec![Command::SetCart { page, index: 0, edit: edit.clone() }, Command::ClearCartFile { page, index: 0 }]
    );
    assert_eq!(
        plan_edit(&s, Edit::SetCart { page, index: 999, edit, track: None }).unwrap_err(),
        ApiError::NotFound
    );
}

#[test]
fn markers_take_finite_seconds() {
    let s = demo_state();
    let t = s.playlists.iter().next().unwrap().entries[0].track;
    assert_eq!(
        plan_edit(&s, Edit::SetMarker { track: t, kind: MarkerKind::CueOut, secs: Some(170.0) }).unwrap(),
        vec![Command::SetMarker { track: t, kind: MarkerKind::CueOut, secs: Some(170.0) }]
    );
    assert_eq!(
        plan_edit(&s, Edit::SetMarker { track: t, kind: MarkerKind::CueOut, secs: Some(f64::NAN) }).unwrap_err().status(),
        400
    );
    assert_eq!(
        plan_edit(&s, Edit::SetMarker { track: t, kind: MarkerKind::CueOut, secs: Some(-1.0) }).unwrap_err().status(),
        400
    );
    assert_eq!(plan_edit(&s, Edit::ResetMarkers(t)).unwrap(), vec![Command::ResetMarkers { track: t }]);
}

async fn call(fake: &Arc<FakeControl>, method: &str, uri: &str, body: Option<Value>) -> StatusCode {
    let ctx = Ctx::new(fake.clone(), Arc::new(HttpRemoteConfig::default()));
    let mut req = Request::builder().method(method).uri(uri).header("host", "127.0.0.1:7380");
    let body = match body {
        Some(v) => {
            req = req.header("content-type", "application/json");
            Body::from(v.to_string())
        }
        None => Body::empty(),
    };
    router(ctx).oneshot(req.body(body).unwrap()).await.unwrap().status()
}

#[tokio::test]
async fn every_editing_route_answers() {
    let s = demo_state();
    let main = s.playlists.first_id().unwrap().0;
    let night = s.playlists.iter().nth(1).unwrap().id.0;
    let e = s.playlists.iter().next().unwrap().entries[2].id.0;
    let t = s.playlists.iter().next().unwrap().entries[0].track.0;
    let page = s.cartwall.pages[0].id.0;
    let fake = FakeControl::new(s);
    let accepted = [
        ("POST", "/api/v1/playlists".to_owned(), Some(json!({"name": "Late"}))),
        ("PATCH", format!("/api/v1/playlists/{night}"), Some(json!({"name": "Overnight"}))),
        ("POST", format!("/api/v1/playlists/{night}/entries"), Some(json!({"track": t, "index": 0}))),
        ("POST", format!("/api/v1/entries/{e}/move"), Some(json!({"playlist": main, "index": 0}))),
        ("POST", format!("/api/v1/entries/{e}/duplicate"), None),
        ("DELETE", format!("/api/v1/entries/{e}"), None),
        ("POST", "/api/v1/cartwall/pages".to_owned(), Some(json!({"name": "B"}))),
        ("PATCH", format!("/api/v1/cartwall/pages/{page}"), Some(json!({"name": "A"}))),
        (
            "PUT",
            format!("/api/v1/cartwall/pages/{page}/carts/1"),
            Some(json!({"name": "Bed", "kind": "effect", "looped": true, "exclusive": false, "track": t})),
        ),
        ("PUT", format!("/api/v1/tracks/{t}/markers/intro-end"), Some(json!({"secs": 4.0}))),
        ("PUT", format!("/api/v1/tracks/{t}/markers/intro-end"), Some(json!({"secs": null}))),
        ("POST", format!("/api/v1/tracks/{t}/markers/reset"), None),
        ("DELETE", format!("/api/v1/playlists/{night}"), None),
    ];
    for (method, uri, body) in accepted {
        assert_eq!(call(&fake, method, &uri, body).await, StatusCode::ACCEPTED, "{method} {uri}");
    }
    assert_eq!(
        call(&fake, "PUT", &format!("/api/v1/tracks/{t}/markers/middle"), Some(json!({"secs": 1.0}))).await,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(&fake, "PUT", &format!("/api/v1/cartwall/pages/{page}/carts/0"), Some(json!({"name": "x", "kind": "song", "looped": false, "exclusive": false, "track": null}))).await,
        StatusCode::BAD_REQUEST
    );
}
```

Run: `cargo test -p fp-remote --test edit` → compile errors.

- [ ] **Step 2: Implement `api::Edit` and `plan_edit`** (append to `api.rs`)

```rust
/// One editing action of a remote client (remote control spec §3.4).
#[derive(Debug, Clone, PartialEq)]
pub enum Edit {
    CreatePlaylist(String),
    RenamePlaylist(PlaylistId, String),
    DeletePlaylist(PlaylistId),
    InsertTrack { playlist: PlaylistId, index: usize, track: TrackId },
    RemoveEntry(EntryId),
    MoveEntry { entry: EntryId, playlist: PlaylistId, index: usize },
    DuplicateEntry(EntryId),
    CreateCartPage(String),
    EditCartPage { page: CartPageId, name: Option<String>, rows: Option<u16>, cols: Option<u16> },
    DeleteCartPage(CartPageId),
    SetCart { page: CartPageId, index: usize, edit: CartEdit, track: Option<TrackId> },
    SetMarker { track: TrackId, kind: MarkerKind, secs: Option<f64> },
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
        Err(ApiError::BadRequest("the name must not be empty".to_owned()))
    } else {
        Ok(name.to_owned())
    }
}

fn edit_commands(state: &AppState, edit: Edit) -> Result<Vec<Command>, ApiError> {
    let list_len = |id: PlaylistId| {
        state.playlists.get(id).map(|l| l.entries.len()).ok_or(ApiError::NotFound)
    };
    let track = |id: TrackId| state.library.get(id).ok_or(ApiError::NotFound);
    let page = |id: CartPageId| state.cartwall.page(id).ok_or(ApiError::NotFound);
    Ok(match edit {
        Edit::CreatePlaylist(n) => vec![Command::CreatePlaylist { name: name(&n)? }],
        Edit::RenamePlaylist(playlist, n) => {
            list_len(playlist)?;
            vec![Command::RenamePlaylist { playlist, name: name(&n)? }]
        }
        Edit::DeletePlaylist(playlist) => {
            list_len(playlist)?;
            vec![Command::DeletePlaylist(playlist)]
        }
        Edit::InsertTrack { playlist, index, track: t } => {
            let len = list_len(playlist)?;
            track(t)?;
            vec![Command::InsertTracks { playlist, index: index.min(len), tracks: vec![t] }]
        }
        Edit::RemoveEntry(entry) => {
            state.playlists.entry(entry).ok_or(ApiError::NotFound)?;
            vec![Command::RemoveEntry(entry)]
        }
        Edit::MoveEntry { entry, playlist, index } => {
            state.playlists.entry(entry).ok_or(ApiError::NotFound)?;
            let len = list_len(playlist)?;
            vec![Command::MoveEntry { entry, to: playlist, index: index.min(len) }]
        }
        Edit::DuplicateEntry(entry) => {
            state.playlists.entry(entry).ok_or(ApiError::NotFound)?;
            vec![Command::DuplicateEntry(entry)]
        }
        Edit::CreateCartPage(n) => vec![Command::CreateCartPage { name: name(&n)? }],
        Edit::EditCartPage { page: id, name: new_name, rows, cols } => {
            let p = page(id)?;
            if new_name.is_none() && rows.is_none() && cols.is_none() {
                return Err(ApiError::BadRequest("nothing to change".to_owned()));
            }
            let mut out = Vec::new();
            if let Some(n) = new_name {
                out.push(Command::RenameCartPage { page: id, name: name(&n)? });
            }
            if rows.is_some() || cols.is_some() {
                let limits = &state.config.limits;
                let rows = rows.unwrap_or(p.rows);
                let cols = cols.unwrap_or(p.cols);
                if !(1..=limits.max_cart_rows).contains(&rows) || !(1..=limits.max_cart_cols).contains(&cols) {
                    return Err(ApiError::BadRequest(format!(
                        "rows must be within 1..={} and cols within 1..={}",
                        limits.max_cart_rows, limits.max_cart_cols
                    )));
                }
                out.push(Command::ResizeCartPage { page: id, rows, cols });
            }
            out
        }
        Edit::DeleteCartPage(id) => {
            page(id)?;
            vec![Command::DeleteCartPage(id)]
        }
        Edit::SetCart { page: id, index, edit, track: wanted } => {
            let cart = page(id)?.carts.get(index).ok_or(ApiError::NotFound)?;
            let mut out = vec![Command::SetCart { page: id, index, edit }];
            match wanted {
                Some(t) if cart.track != Some(t) => {
                    track(t)?;
                    out.push(Command::AssignCartTrack { page: id, index, track: t });
                }
                None if cart.track.is_some() => out.push(Command::ClearCartFile { page: id, index }),
                _ => {}
            }
            out
        }
        Edit::SetMarker { track: t, kind, secs } => {
            track(t)?;
            if let Some(s) = secs
                && !(s.is_finite() && s >= 0.0)
            {
                return Err(ApiError::BadRequest("secs must be a position in the track".to_owned()));
            }
            vec![Command::SetMarker { track: t, kind, secs }]
        }
        Edit::ResetMarkers(t) => {
            track(t)?;
            vec![Command::ResetMarkers { track: t }]
        }
    })
}
```

Add `CartEdit, MarkerKind, TrackId` to the `fp_model` imports.

- [ ] **Step 3: Handlers and routes**

In `handlers.rs`, factor the queueing out of `run`:

```rust
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
```

Bodies:

```rust
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Name { name: String }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InsertBody { track: u64, index: usize }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MoveBody { playlist: u64, index: usize }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageEdit { name: Option<String>, rows: Option<u16>, cols: Option<u16> }

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CartKindName { Jingle, Effect, Spot }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CartBody { name: String, kind: CartKindName, looped: bool, exclusive: bool, track: Option<u64> }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarkerBody { secs: Option<f64> }
```

The handlers. Each one parses its ids with `id()` and calls `run_edit`:

```rust
pub async fn create_playlist(State(ctx): State<Ctx>, JsonBody(b): JsonBody<Name>) -> Reply {
    run_edit(&ctx, Edit::CreatePlaylist(b.name))
}
pub async fn rename_playlist(State(ctx): State<Ctx>, Path(raw): Path<String>, JsonBody(b): JsonBody<Name>) -> Reply {
    run_edit(&ctx, Edit::RenamePlaylist(PlaylistId(id(&raw)?), b.name))
}
pub async fn delete_playlist(State(ctx): State<Ctx>, Path(raw): Path<String>) -> Reply {
    run_edit(&ctx, Edit::DeletePlaylist(PlaylistId(id(&raw)?)))
}
pub async fn insert_entry(State(ctx): State<Ctx>, Path(raw): Path<String>, JsonBody(b): JsonBody<InsertBody>) -> Reply {
    run_edit(&ctx, Edit::InsertTrack { playlist: PlaylistId(id(&raw)?), index: b.index, track: TrackId(b.track) })
}
pub async fn remove_entry(State(ctx): State<Ctx>, Path(raw): Path<String>) -> Reply {
    run_edit(&ctx, Edit::RemoveEntry(EntryId(id(&raw)?)))
}
pub async fn move_entry(State(ctx): State<Ctx>, Path(raw): Path<String>, JsonBody(b): JsonBody<MoveBody>) -> Reply {
    run_edit(&ctx, Edit::MoveEntry { entry: EntryId(id(&raw)?), playlist: PlaylistId(b.playlist), index: b.index })
}
pub async fn duplicate_entry(State(ctx): State<Ctx>, Path(raw): Path<String>) -> Reply {
    run_edit(&ctx, Edit::DuplicateEntry(EntryId(id(&raw)?)))
}
pub async fn create_cart_page(State(ctx): State<Ctx>, JsonBody(b): JsonBody<Name>) -> Reply {
    run_edit(&ctx, Edit::CreateCartPage(b.name))
}
pub async fn edit_cart_page(State(ctx): State<Ctx>, Path(raw): Path<String>, JsonBody(b): JsonBody<PageEdit>) -> Reply {
    run_edit(&ctx, Edit::EditCartPage { page: CartPageId(id(&raw)?), name: b.name, rows: b.rows, cols: b.cols })
}
pub async fn delete_cart_page(State(ctx): State<Ctx>, Path(raw): Path<String>) -> Reply {
    run_edit(&ctx, Edit::DeleteCartPage(CartPageId(id(&raw)?)))
}
pub async fn set_cart(State(ctx): State<Ctx>, Path((raw, index)): Path<(String, String)>, JsonBody(b): JsonBody<CartBody>) -> Reply {
    let index: usize = index.parse().map_err(|_| ApiError::NotFound)?;
    let kind = match b.kind {
        CartKindName::Jingle => CartKind::Jingle,
        CartKindName::Effect => CartKind::Effect,
        CartKindName::Spot => CartKind::Spot,
    };
    let edit = CartEdit { name: b.name, kind, looped: b.looped, exclusive: b.exclusive };
    run_edit(&ctx, Edit::SetCart { page: CartPageId(id(&raw)?), index, edit, track: b.track.map(TrackId) })
}
pub async fn set_marker(State(ctx): State<Ctx>, Path((raw, kind)): Path<(String, String)>, JsonBody(b): JsonBody<MarkerBody>) -> Reply {
    let kind = match kind.as_str() {
        "cue-in" => MarkerKind::CueIn,
        "intro-end" => MarkerKind::IntroEnd,
        "outro-start" => MarkerKind::OutroStart,
        "segue-start" => MarkerKind::SegueStart,
        "cue-out" => MarkerKind::CueOut,
        _ => return Err(ApiError::NotFound),
    };
    run_edit(&ctx, Edit::SetMarker { track: TrackId(id(&raw)?), kind, secs: b.secs })
}
pub async fn reset_markers(State(ctx): State<Ctx>, Path(raw): Path<String>) -> Reply {
    run_edit(&ctx, Edit::ResetMarkers(TrackId(id(&raw)?)))
}
```

`PageEdit` may name fields as `null` (serde treats them as `None`).

The routes in `http/mod.rs` (use `routing::{delete, patch}`):

```rust
        .route("/playlists", get(h::playlists).post(h::create_playlist))
        .route("/playlists/{id}", get(h::playlist).patch(h::rename_playlist).delete(h::delete_playlist))
        .route("/playlists/{id}/entries", post(h::insert_entry))
        .route("/entries/{id}", delete(h::remove_entry))
        .route("/entries/{id}/move", post(h::move_entry))
        .route("/entries/{id}/duplicate", post(h::duplicate_entry))
        .route("/cartwall/pages", post(h::create_cart_page))
        .route("/cartwall/pages/{id}", patch(h::edit_cart_page).delete(h::delete_cart_page))
        .route("/cartwall/pages/{id}/carts/{index}", put(h::set_cart))
        .route("/tracks/{id}/markers/{kind}", put(h::set_marker))
        .route("/tracks/{id}/markers/reset", post(h::reset_markers))
```

These replace the existing `/playlists` and `/playlists/{id}` lines.

- [ ] **Step 4: Run** `cargo test -p fp-remote` → all pass.
- [ ] **Step 5: Gate and commit** `feat(remote): edit playlists, cart pages and markers`.

---

### Task 3: hardening (deferred minors)

**Files:**
- Modify: `crates/fp-remote/src/api.rs` (`MethodNotAllowed`),
  `http/mod.rs`, `http/guard.rs`, `dto.rs`, `server.rs`
- Test: `tests/http.rs`, `tests/guard.rs`, `tests/dto.rs`, `tests/server.rs`

**Interfaces:**
- Produces: `ApiError::MethodNotAllowed` (405, `method_not_allowed`).

- [ ] **Step 1: Failing tests**

`tests/http.rs`:

```rust
#[tokio::test]
async fn a_wrong_method_is_a_json_405() {
    let fake = FakeControl::new(demo_state());
    let (status, body, _) = call(ctx(&fake), "GET", "/api/v1/players/1/play", None).await;
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(body["error"], "method_not_allowed");
}
```

`tests/guard.rs`:

```rust
#[test]
fn the_bearer_scheme_is_case_insensitive() {
    for scheme in ["Bearer", "bearer", "BEARER"] {
        let h = headers(&[("host", "pc:7380"), ("authorization", &format!("{scheme} {TOKEN}"))]);
        assert_eq!(check(&lan(), &h, None), Ok(()), "{scheme}");
    }
}
```

`tests/dto.rs` (cart times like the cart button: clamped into the play
length, and none when the duration is unknown):

```rust
#[test]
fn cart_times_are_clamped_and_unknown_without_a_duration() {
    let mut s = demo_state();
    let cart = s.cartwall.pages[0].carts[0].id;
    fp_model::apply(&mut s, Command::FireCart(cart)).unwrap();
    let late = Playback { carts: vec![(cart, 500.0)], ..Default::default() };
    assert_eq!(dto::cartwall(&s, &late).playing[0].remaining_secs, Some(0.0));
    let t = s.cartwall.pages[0].carts[0].track.unwrap();
    s.library.get_mut(t).unwrap().duration_secs = 0.0;
    let out = dto::cartwall(&s, &Playback { carts: vec![(cart, 3.0)], ..Default::default() });
    assert_eq!(out.playing[0].remaining_secs, None);
}
```

`tests/server.rs` (a port busy at start frees later):

```rust
#[test]
fn a_port_that_frees_later_is_taken_without_a_restart() {
    let taken = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = taken.local_addr().unwrap().port();
    let fake = FakeControl::new(demo_state());
    fake.edit(|s| {
        s.config.remote.http.enabled = true;
        s.config.remote.http.port = port;
    });
    let handle = spawn(fake).unwrap();
    wait_for(&handle, "error", |s| matches!(s, ServerStatus::Error(_)));
    drop(taken);
    let deadline = Instant::now() + Duration::from_secs(10);
    while !matches!(handle.status().http, ServerStatus::Listening(_)) {
        assert!(Instant::now() < deadline, "{:?}", handle.status());
        std::thread::sleep(Duration::from_millis(50));
    }
}
```

Run each, and watch them fail.

- [ ] **Step 2: Implement**

- `ApiError::MethodNotAllowed`: status 405, code `method_not_allowed`,
  message "this resource does not take that method". In `router`, add
  `.method_not_allowed_fallback(h::method_not_allowed)` on the outer router
  and on `api`, with
  `pub async fn method_not_allowed() -> ApiError { ApiError::MethodNotAllowed }`.
  Add the row to `every_error_has_its_status_and_code`.
- Guard: split the scheme off and compare it case-insensitively:

```rust
            .and_then(|v| v.split_once(' '))
            .filter(|(scheme, _)| scheme.eq_ignore_ascii_case("bearer"))
            .map(|(_, token)| token.trim());
```

- `dto::cartwall`: mirror `ui/cart_view.rs`:

```rust
            let length = track.map(Track::play_length_secs).unwrap_or(0.0);
            let known = track.is_some_and(|t| t.duration_secs > 0.0) && length > 0.0;
            PlayingCartDto {
                cart: pc.cart,
                elapsed_secs: elapsed,
                remaining_secs: track
                    .zip(elapsed)
                    .filter(|_| known)
                    .map(|(t, e)| (t.cue_out_secs() - e).clamp(0.0, length)),
            }
```

- `server.rs`: a bind that failed is tried again.
  `const BIND_RETRY: Duration = Duration::from_secs(2);`. In `supervise`,
  keep `last_try` per server (`tokio::time::Instant`). When a server's status
  is `Error(ServerError::Bind(_))` and `last_try.elapsed() >= BIND_RETRY`,
  set `applied = None` (or `applied_osc = None`), so the next loop restarts
  it. Errors other than `Bind` wait for a configuration change.
- `fp-app/src/remote.rs`: `Bridge::analysis` uses
  `self.conductor.model.load_full()` (it is held across disk I/O).

- [ ] **Step 3: Plan 2's deferred minors.** Each one gets a failing test
  first.
  - **A stream opened just after a stop never ends.** In
    `http/events.rs`, replace `f.stop.changed()` with
    `f.stop.wait_for(|stopped| *stopped)`, which checks the current value
    first.

    Test, in `tests/sse.rs`: send `stop` *before* the request. The stream
    gives the `state` and then ends.
  - **Stale events after a resync.** On `RecvError::Lagged`, set
    `f.rx = f.rx.resubscribe()` before sending `resync`.

    Test: with capacity 1, after the resync and one fresh event, the next
    chunk is that fresh event, not an older buffered one.
  - **Position drift.** In `publish`, advance
    `last_position += every` (and reset it to now when it falls more than
    one interval behind) instead of setting it to now.

    No reliable unit test exists for this: record a `Ruling:`. The existing
    SSE and server tests guard behaviour.
  - **The OSC shape** includes the cart count of the page shown, so a grid
    resize sends everything again.

    Test, in `tests/osc_server.rs`: subscribe, drain, resize the shown page
    through `fake.edit`, and the full dump (including
    `/fauste/player/1/transport`) arrives again.
- [ ] **Step 4: Run** `cargo test -p fp-remote` → all pass.
- [ ] **Step 5: Gate and commit** `fix(remote): JSON 405, bind retry, case-insensitive bearer, cart times like the UI`, plus the plan 2 minors.

---

### Task 4: Settings > Remote

**Files:**
- Create: `crates/fp-app/src/ui/settings/remote.rs`, `crates/fp-app/tests/remote_settings.rs`
- Modify:
  - `crates/fp-app/src/ui/settings.rs` (`Section::Remote`, the tab,
    `SettingsDeps.remote`, the `SettingsState` buffers);
  - `crates/fp-app/src/ui/app.rs` (`with_remote_status`, passes a status
    snapshot);
  - `crates/fp-app/src/remote.rs` (`new_token`);
  - `crates/fp-app/src/main.rs`;
  - `crates/fp-remote/src/server.rs` (`RemoteHandle::status_cell`);
  - `crates/fp-app/Cargo.toml` (`getrandom = "0.3"`, via the workspace);
  - both locale files.

**Interfaces:**
- Produces:
  - `RemoteHandle::status_cell(&self) -> Arc<ArcSwap<RemoteStatus>>`;
  - `AppUi::with_remote_status(Arc<ArcSwap<RemoteStatus>>) -> Self`;
  - `fp_app::remote::new_token() -> Option<String>` (43 base64url
    characters, 32 random bytes).

- [ ] **Step 1: Failing tests** (`crates/fp-app/tests/remote_settings.rs`)

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing)]
//! Settings > Remote (remote control spec §8).

mod support;

use std::sync::Arc;

use arc_swap::ArcSwap;
use egui::accesskit::Role;
use egui_kittest::kittest::Queryable;
use fp_model::Command;
use fp_remote::{RemoteStatus, ServerError, ServerStatus};
use support::{harness_from, state};

fn open(status: RemoteStatus) -> (egui_kittest::Harness<'static, fp_app::ui::app::AppUi>, Arc<support::Fake>, Arc<ArcSwap<RemoteStatus>>) {
    let cell = Arc::new(ArcSwap::from_pointee(status));
    let given = cell.clone();
    let (mut h, fake) = harness_from(state(1, 0), move |ui| ui.with_remote_status(given));
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Remote").click();
    h.run_steps(2);
    (h, fake, cell)
}

fn last_config(fake: &support::Fake) -> fp_model::Config {
    fake.take_sent()
        .into_iter()
        .rev()
        .find_map(|c| match c {
            Command::UpdateConfig(c) => Some(*c),
            _ => None,
        })
        .expect("no UpdateConfig sent")
}

#[test]
fn the_switches_turn_the_servers_on() {
    let (mut h, fake, _) = open(RemoteStatus::default());
    h.get_by_label("Allow remote control over HTTP").click();
    h.run_steps(2);
    assert!(last_config(&fake).remote.http.enabled);
    h.get_by_label("Allow OSC control").click();
    h.run_steps(2);
    assert!(last_config(&fake).remote.osc.enabled);
}

#[test]
fn the_status_of_each_server_is_shown() {
    let status = RemoteStatus {
        http: ServerStatus::Listening("127.0.0.1:7380".parse().unwrap()),
        osc: ServerStatus::Error(ServerError::TokenRequired),
    };
    let (h, _, _) = open(status);
    assert!(h.query_by_label_contains("Listening on 127.0.0.1:7380").is_some());
    assert!(h.query_by_label_contains("a token is required").is_some());
}

#[test]
fn generate_makes_a_long_random_token() {
    let (mut h, fake, _) = open(RemoteStatus::default());
    h.get_by_label("Generate").click();
    h.run_steps(2);
    let token = last_config(&fake).remote.http.token;
    assert_eq!(token.len(), 43);
    assert!(token.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
}

#[test]
fn the_token_is_masked_until_shown() {
    let mut s = state(1, 0);
    s.config.remote.http.token = "s3cret-token-value-0123".into();
    let cell = Arc::new(ArcSwap::from_pointee(RemoteStatus::default()));
    let (mut h, _) = harness_from(s, move |ui| ui.with_remote_status(cell));
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Remote").click();
    h.run_steps(2);
    assert!(h.query_by_label_contains("s3cret-token").is_none());
    h.get_by_label("Show").click();
    h.run_steps(2);
    assert!(h.query_by_value("s3cret-token-value-0123").is_some());
}

#[test]
fn a_warning_appears_beyond_this_computer_without_a_token() {
    let mut s = state(1, 0);
    s.config.remote.http.bind = "0.0.0.0".into();
    let cell = Arc::new(ArcSwap::from_pointee(RemoteStatus::default()));
    let (mut h, _) = harness_from(s, move |ui| ui.with_remote_status(cell));
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Remote").click();
    h.run_steps(2);
    assert!(h.query_by_label_contains("A token is required to listen beyond this computer").is_some());
}

#[test]
fn new_tokens_differ() {
    let a = fp_app::remote::new_token().unwrap();
    let b = fp_app::remote::new_token().unwrap();
    assert_ne!(a, b);
    assert_eq!(a.len(), 43);
}
```

Add `fp-remote.workspace = true` to the `fp-app` dev-dependencies if the
normal dependency does not already cover the test. Kittest queries are
matched to the widgets the section draws. If a query name differs (for
example a checkbox label), adjust the test to the label the strings define,
not the other way round.

Run: `cargo test -p fp-app --test remote_settings` → compile errors.

- [ ] **Step 2: Implement**

`RemoteHandle::status_cell` returns `self.status.clone()`.

`fp-app/src/remote.rs`:

```rust
/// A new API token: 32 random bytes in base64url without padding (43
/// characters). `None` if the system has no random source.
pub fn new_token() -> Option<String> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).ok()?;
    Some(base64url(&bytes))
}

fn base64url(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [chunk.first().copied().unwrap_or(0), chunk.get(1).copied().unwrap_or(0), chunk.get(2).copied().unwrap_or(0)];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        let chars = chunk.len() + 1;
        for i in 0..chars {
            let index = ((n >> (18 - 6 * i)) & 63) as usize;
            if let Some(c) = ALPHABET.get(index) {
                out.push(char::from(*c));
            }
        }
    }
    out
}
```

`AppUi`:
- a field `remote_status: Option<Arc<ArcSwap<RemoteStatus>>>`, set by
  `with_remote_status`;
- when it builds `SettingsDeps`, pass
  `remote: self.remote_status.as_ref().map(|c| (**c.load()).clone())`;
- `SettingsDeps` gains `pub remote: Option<fp_remote::RemoteStatus>`.

`main.rs`: after starting remote, call
`if let Some(r) = &remote { app = app.with_remote_status(r.status_cell()); }`.
The start must happen before `AppUi::new`, and `app` must be `mut` at that
point. Move the `let mut app` as needed.

`settings.rs`:
- add `Section::Remote` after `Midi`;
- add the tab `(Section::Remote, icon::BROADCAST, "settings-tab-remote")`;
- add the match arm `Section::Remote => remote::section(ui, scene, st, deps.remote.as_ref())`;
- add `mod remote;`;
- add `pub(super) remote: remote::RemoteState` to `SettingsState`.

`ui/settings/remote.rs`:

```rust
//! Settings > Remote (remote control spec §8): the HTTP and OSC switches,
//! addresses, token, allowed origins and senders, and each server's state.
//! Text fields apply when they lose focus; the servers follow the saved
//! configuration by themselves.

use std::collections::HashMap;

use egui::{RichText, Ui, vec2};
use fp_model::Config;
use fp_remote::{RemoteStatus, ServerError, ServerStatus};

use super::super::app::Scene;
use super::super::theme;
use super::super::widgets::font;
use super::{heading, update};

#[derive(Default)]
pub(crate) struct RemoteState {
    /// Text being typed, per field, until it loses focus.
    drafts: HashMap<&'static str, String>,
    show_token: bool,
}

fn text(ui: &mut Ui, s: impl Into<String>, color: egui::Color32) {
    ui.label(RichText::new(s.into()).font(font(13.0)).color(color));
}

/// A text field over a configuration value. Returns the new text when the
/// field loses focus with a change.
fn field(
    ui: &mut Ui,
    st: &mut RemoteState,
    key: &'static str,
    current: &str,
    multiline: bool,
    password: bool,
) -> Option<String> {
    let draft = st.drafts.entry(key).or_insert_with(|| current.to_owned());
    let edit = if multiline {
        egui::TextEdit::multiline(draft).desired_rows(3)
    } else {
        egui::TextEdit::singleline(draft).password(password)
    };
    let response = ui.add(edit.desired_width(320.0));
    if response.has_focus() {
        return None;
    }
    let changed = (*draft != current).then(|| draft.clone());
    if !response.lost_focus() {
        // Not being edited: show what the configuration holds.
        *draft = current.to_owned();
        return None;
    }
    st.drafts.remove(key);
    changed
}

fn lines(text: &str) -> Vec<String> {
    text.lines().map(str::trim).filter(|l| !l.is_empty()).map(str::to_owned).collect()
}

fn status_line(scene: &Scene<'_>, status: &ServerStatus) -> (String, egui::Color32) {
    let t = scene.i18n;
    match status {
        ServerStatus::Off => (t.tr("remote-status-off"), theme::NEUTRAL_500),
        ServerStatus::Listening(addr) => (
            t.tr_args("remote-status-listening", &[("addr", addr.to_string().into())]),
            theme::NEUTRAL_300,
        ),
        ServerStatus::Error(ServerError::TokenRequired) => (t.tr("remote-error-token"), theme::DANGER),
        ServerStatus::Error(ServerError::InvalidBind) => (t.tr("remote-error-address"), theme::DANGER),
        ServerStatus::Error(ServerError::Bind(reason) | ServerError::Runtime(reason)) => (
            t.tr_args("remote-error-other", &[("reason", reason.clone().into())]),
            theme::DANGER,
        ),
    }
}

pub(super) fn section(ui: &mut Ui, scene: &Scene<'_>, st: &mut super::SettingsState, status: Option<&RemoteStatus>) {
    let t = scene.i18n;
    let st = &mut st.remote;
    heading(ui, &t.tr("settings-tab-remote"));
    let Some(status) = status else {
        text(ui, t.tr("remote-unavailable"), theme::NEUTRAL_400);
        return;
    };
    let config = scene.state.config.remote.clone();

    // HTTP
    text(ui, t.tr("remote-http"), theme::TEXT);
    let mut on = config.http.enabled;
    if ui.checkbox(&mut on, t.tr("remote-http-enabled")).changed() {
        update(scene, |c| c.remote.http.enabled = on);
    }
    let (line, color) = status_line(scene, &status.http);
    text(ui, line, color);
    ui.horizontal(|ui| {
        text(ui, t.tr("remote-bind"), theme::NEUTRAL_300);
        if let Some(v) = field(ui, st, "http.bind", &config.http.bind, false, false) {
            update(scene, |c| c.remote.http.bind = v.trim().to_owned());
        }
        text(ui, t.tr("remote-port"), theme::NEUTRAL_300);
        let mut port = config.http.port;
        if ui.add(egui::DragValue::new(&mut port).range(1024..=65535)).changed() {
            update(scene, |c| c.remote.http.port = port);
        }
    });
    ui.horizontal(|ui| {
        text(ui, t.tr("remote-token"), theme::NEUTRAL_300);
        let masked = !st.show_token;
        if let Some(v) = field(ui, st, "http.token", &config.http.token, false, masked) {
            update(scene, |c| c.remote.http.token = v.trim().to_owned());
        }
        let label = if st.show_token { t.tr("remote-token-hide") } else { t.tr("remote-token-show") };
        if ui.button(label).clicked() {
            st.show_token = !st.show_token;
        }
        if ui.button(t.tr("remote-token-copy")).clicked() {
            ui.ctx().copy_text(config.http.token.clone());
        }
        if ui.button(t.tr("remote-token-generate")).clicked()
            && let Some(token) = crate::remote::new_token()
        {
            st.drafts.remove("http.token");
            update(scene, |c| c.remote.http.token = token);
        }
    });
    let local = config.http.bind_addr().is_some_and(|ip| ip.is_loopback());
    if !local && config.http.token.is_empty() {
        text(ui, t.tr("remote-token-needed"), theme::WARNING);
    }
    text(ui, t.tr("remote-origins"), theme::NEUTRAL_300);
    if let Some(v) = field(ui, st, "http.origins", &config.http.cors_origins.join("\n"), true, false) {
        update(scene, |c| c.remote.http.cors_origins = lines(&v));
    }

    ui.add_space(16.0);
    // OSC
    text(ui, t.tr("remote-osc"), theme::TEXT);
    let mut on = config.osc.enabled;
    if ui.checkbox(&mut on, t.tr("remote-osc-enabled")).changed() {
        update(scene, |c| c.remote.osc.enabled = on);
    }
    let (line, color) = status_line(scene, &status.osc);
    text(ui, line, color);
    ui.horizontal(|ui| {
        text(ui, t.tr("remote-bind"), theme::NEUTRAL_300);
        if let Some(v) = field(ui, st, "osc.bind", &config.osc.bind, false, false) {
            update(scene, |c| c.remote.osc.bind = v.trim().to_owned());
        }
        text(ui, t.tr("remote-port"), theme::NEUTRAL_300);
        let mut port = config.osc.port;
        if ui.add(egui::DragValue::new(&mut port).range(1024..=65535)).changed() {
            update(scene, |c| c.remote.osc.port = port);
        }
    });
    text(ui, t.tr("remote-sources"), theme::NEUTRAL_300);
    if let Some(v) = field(ui, st, "osc.sources", &config.osc.allowed_sources.join("\n"), true, false) {
        update(scene, |c| c.remote.osc.allowed_sources = lines(&v));
    }

    ui.add_space(16.0);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = vec2(8.0, 0.0);
        text(ui, t.tr("remote-position-interval"), theme::NEUTRAL_300);
        let mut ms = config.events.position_interval_ms;
        if ui.add(egui::DragValue::new(&mut ms).range(50..=5000).suffix(" ms")).changed() {
            update(scene, |c| c.remote.events.position_interval_ms = ms);
        }
    });
    // The servers' state changes on their own thread.
    ui.ctx().request_repaint_after(std::time::Duration::from_millis(500));
    let _: Option<&Config> = None;
}
```

Use the existing theme colour names (`theme::DANGER`, `theme::WARNING`, …).
If one does not exist, pick the one the MIDI and Outputs sections use for
errors and warnings. Drop the placeholder `let _: Option<&Config>` line and
the `Config` import if unused.

Strings (`en-US` / `es-ES`):

```ftl
settings-tab-remote = Remote
remote-unavailable = Remote control is not available.
remote-http = HTTP API
remote-http-enabled = Allow remote control over HTTP
remote-osc = OSC
remote-osc-enabled = Allow OSC control
remote-bind = Address
remote-port = Port
remote-token = Token
remote-token-show = Show
remote-token-hide = Hide
remote-token-copy = Copy
remote-token-generate = Generate
remote-token-needed = A token is required to listen beyond this computer.
remote-origins = Web pages allowed to use the API (one origin per line, e.g. https://studio.example)
remote-sources = Senders allowed (one address or subnet per line, e.g. 192.168.1.0/24)
remote-position-interval = Publish times every
remote-status-off = Off
remote-status-listening = Listening on { $addr }
remote-error-token = Not started: a token is required beyond this computer.
remote-error-address = Not started: the address is not valid.
remote-error-other = Not started: { $reason }
```

```ftl
settings-tab-remote = Remoto
remote-unavailable = El control remoto no está disponible.
remote-http = API HTTP
remote-http-enabled = Permitir el control remoto por HTTP
remote-osc = OSC
remote-osc-enabled = Permitir el control por OSC
remote-bind = Dirección
remote-port = Puerto
remote-token = Token
remote-token-show = Mostrar
remote-token-hide = Ocultar
remote-token-copy = Copiar
remote-token-generate = Generar
remote-token-needed = Hace falta un token para escuchar fuera de este ordenador.
remote-origins = Páginas web que pueden usar la API (un origen por línea, p. ej. https://studio.example)
remote-sources = Emisores permitidos (una dirección o subred por línea, p. ej. 192.168.1.0/24)
remote-position-interval = Publicar los tiempos cada
remote-status-off = Desactivado
remote-status-listening = Escuchando en { $addr }
remote-error-token = No iniciado: hace falta un token fuera de este ordenador.
remote-error-address = No iniciado: la dirección no es válida.
remote-error-other = No iniciado: { $reason }
```

The i18n test that compares both locales must still pass.

- [ ] **Step 3: Run** `cargo test -p fp-app` → all pass, including the i18n
  key-parity test.
- [ ] **Step 4: Look at it.** Run the app with a scratch home, open Settings >
  Remote, and take a screenshot (CLAUDE.md, testing notes). Check the layout
  at the dialog's width.
- [ ] **Step 5: Gate, `cargo deny check`, commit** `feat(ui): Settings > Remote`.

---

### Task 5: documentation and the README screenshot

**Files:**
- `docs/user/remote-control.md`: say that an OSC subscriber may name any
  port of its own address (plan 2 review). Also, "Turning it on" uses Settings > Remote,
  and editing `config.json` becomes an alternative. Add an "Editing"
  paragraph to "What a client can do".
- `docs/user/settings.md`: a "Remote" section (switches, address, port,
  token with Generate/Show/Copy, origins, senders, status lines).
- `docs/technical/remote-api.md`:
  - an "Editing" routes table (spec §3.4, with `InsertTracks` and
    `AssignCartTrack`);
  - 405 `method_not_allowed`;
  - bind retry every 2 s;
  - the Settings page.

  Drop "not implemented yet".
- `docs/technical/persistence.md`: `remote` is "(Settings → Remote)".
- `docs/technical/ui.md`: Settings > Remote reads the status cell.
- The spec, §3.4: editing reuses library tracks (`InsertTracks`,
  `AssignCartTrack`), and §6.1 mentions the bind retry.
- `README.md`:
  - the remote control bullet mentions editing and Settings;
  - a **new screenshot** with players on air (see below).
- `CLAUDE.md`: nothing new, unless the layout changed.

**Screenshot**:

```sh
D=<scratch>
cargo build --release -p fp-app --example demo_session
rm -rf $D/home && FAUSTE_HOME=$D/home target/release/examples/demo_session <music dir>
python3 - $D/home/config/config.json <<'EOF'
import json,sys; p=sys.argv[1]; c=json.load(open(p)); c["config"]["remote"]["http"]["enabled"]=True; json.dump(c,open(p,"w"),indent=2)
EOF
env -u WAYLAND_DISPLAY FAUSTE_HOME=$D/home target/release/fauste-player &
# wait for the API, then put two players on air and fire a cart
curl -s http://127.0.0.1:7380/api/v1/players   # ids
curl -s -X POST http://127.0.0.1:7380/api/v1/players/<id1>/play
curl -s -X POST http://127.0.0.1:7380/api/v1/players/<id2>/play
sleep 8   # analysis draws waveforms and the countdowns move
import -window "$(xwininfo -name 'Fauste Player' | awk '/Window id/{print $4}')" docs/images/main-screen.png
```

Check the image (open it). It must show players on air with waveforms and
countdowns. Use real music if `test-music/` has some. Otherwise use
generated tones longer than 2 minutes, named like music, and say so in the
PR. Compare the size with the old image (`file docs/images/main-screen.png`
before overwriting) and keep the same window size.

- [ ] **Step 1: Docs.**
- [ ] **Step 2: Screenshot.**
- [ ] **Step 3: Gate and commit** `docs: remote editing, Settings > Remote, and a main screen on air`.

---

## After the last task

Final review with a fresh reviewer on the most capable model, then fix pass,
PR `feat(remote): editing routes and Settings > Remote`, CI, merge.
