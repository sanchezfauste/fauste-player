# Remote Plan 1 — HTTP Foundation, Reading and Operating Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** an HTTP/JSON API, off by default, that reads players, playlists,
tracks (with cover and waveform) and the cartwall, and operates the
transport, next entry, pre-listen, seek, volume, modes and carts.

**Architecture:**
- `fp-model` gains `remote.rs`: `config.remote.http` and
  `config.remote.events`, validated in `Config::validate`.
- New crate `fp-remote`, whose only workspace dependency is `fp-model`:
  - `control.rs`: the `RemoteControl` trait the application implements, plus
    `Playback` and `WaveformData`;
  - `dto.rs`: the JSON contract of `/api/v1`, built from `&AppState` +
    `&Playback` (pure);
  - `api.rs`: operations → `Vec<Command>`, with availability (R28),
    desired-value toggles, and a dry run on a copy of the snapshot (pure);
  - `http/`: an axum router with handlers, a JSON body extractor, and the
    guard (token, `Origin`, `Host`) plus CORS, body limit and timeout;
  - `server.rs`: one `fp-remote` thread with a tokio `current_thread`
    runtime. It follows `config.remote.http` in the model snapshot, restarts
    the server when it changes, and publishes its status through `ArcSwap`.
- `fp-app` gains `remote.rs`:
  - a `Bridge` from `ConductorHandle` + `MediaCache` + `AnalysisCache` to
    `RemoteControl`;
  - `main.rs` starts it.

**Tech Stack:** Rust 2024, axum 0.8.9 (http1, json, tokio), tokio 1.53.1
(rt, net, time, sync, macros), tower 0.5.3, tower-http 0.7.1 (cors,
timeout, limit), arc-swap, serde_json. Versions checked together in a
scratch build on 2026-10-01.

**Spec:** [`docs/superpowers/specs/2026-10-01-remote-control-design.md`](../specs/2026-10-01-remote-control-design.md) §2, §3.1–3.3, §3.5, §6.1 (HTTP and events), §6.2, §6.4, §7. Plan 1 of 3 (§10).

## Global Constraints

- Branch `feat/remote-control`. Before every commit:
  `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.
- No `unsafe`. No `unwrap`/`expect`/`panic` outside tests.
  `fp-remote` denies `clippy::indexing_slicing`.
- The remote thread is neither an audio nor a UI thread. Commands go through
  `ConductorHandle::send`, which never blocks.
- Off by default: `remote.http.enabled = false`, `bind = "127.0.0.1"`,
  `port = 7380`.
- Rule 10: starting, restarting or reconfiguring the server never sends a
  command.
- Every HTTP error body is `{"error": code, "message": text}`, except
  408 (empty body).
- `config.remote` ranges (spec §6.1), applied in `Config::validate`:

  | Field | Rule |
  |---|---|
  | `http.port` | 1024–65535 |
  | `http.token` | empty, or ≥ 16 characters (a shorter one is dropped, with a warning) |
  | `http.bind` | IP literal, else `127.0.0.1` (with a warning) |
  | `http.cors_origins` | `scheme://host[:port]` with scheme `http` or `https`; `"*"` only with a loopback bind and no token |
  | `http.max_event_clients` | 1–256 (default 16) |
  | `http.request_timeout_ms` | 1000–120000 (default 10000) |
  | `http.max_body_bytes` | 1024–1048576 (default 65536) |
  | `events.position_interval_ms` | 50–5000 (default 250) |
- Beyond loopback without a token, the HTTP server does not start: status
  `Error(TokenRequired)` and an error log line.
- Ids in HTTP are the model's `u64` ids. A non-numeric id is a `404`.
- Desired values: a setting already in the wanted state sends no command and
  still answers `202`.
- Docs and spec stay in sync. `CHANGELOG.md` is not touched.
- Commit trailer: `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

## Review Focus

- **Malformed ids and paths:** `/players/abc`, `/players/99999999999999999999999`
  and `/players/1/dance` answer `404` JSON and never `500` (Task 4 tests).
- **Hostile or broken bodies:** `{"fader":"x"}`, `{"fader":1e999}`, `{}`, an
  empty body, `text/plain`, and an oversized body answer `400`/`415`/`413`
  JSON (Task 4 and Task 5 tests).
- **A browser page attacking the local server:** a foreign `Origin`, a
  `Host: evil.example` (DNS rebinding), a missing `Host`, and `[::1]:7380`
  (which must be accepted) (Task 5 tests).
- **Port already in use, or a configuration changed at run time:** the app
  keeps running with an `Error` status; a port change frees the old port;
  disabling closes it (Task 6 tests).
- **An id that vanished between the snapshot and the request** (a deleted
  playlist, a removed player): the dry run turns it into `404`, never a
  queued command that the model then refuses (Task 3 tests).

---

### Task 1: `config.remote` in `fp-model`

**Files:**
- Create: `crates/fp-model/src/remote.rs`
- Modify: `crates/fp-model/src/config.rs` (field `remote`, `Default`,
  `validate`, `clamp_to` becomes `pub(crate)`)
- Modify: `crates/fp-model/src/lib.rs` (module and re-exports)
- Test: `crates/fp-model/src/remote.rs` (`#[cfg(test)]`)

**Interfaces:**
- Produces:
  - `fp_model::RemoteConfig { pub http: HttpRemoteConfig, pub events: RemoteEventsConfig }`;
  - `HttpRemoteConfig { enabled: bool, bind: String, port: u16, token: String, cors_origins: Vec<String>, max_event_clients: u32, request_timeout_ms: u32, max_body_bytes: u32 }`;
  - `HttpRemoteConfig::bind_addr(&self) -> Option<IpAddr>`;
  - `RemoteEventsConfig { position_interval_ms: u32 }`;
  - `Config::remote`.

- [ ] **Step 1: Write the failing tests** in `crates/fp-model/src/remote.rs`

```rust
//! Network remote control configuration (remote control spec §6.1).

#[cfg(test)]
mod tests {
    use crate::Config;

    #[test]
    fn remote_control_is_off_and_local_by_default() {
        let c = Config::default();
        assert!(!c.remote.http.enabled);
        assert_eq!(c.remote.http.bind, "127.0.0.1");
        assert_eq!(c.remote.http.port, 7380);
        assert!(c.remote.http.token.is_empty());
        assert!(c.remote.http.cors_origins.is_empty());
        assert_eq!(c.remote.http.max_event_clients, 16);
        assert_eq!(c.remote.http.request_timeout_ms, 10_000);
        assert_eq!(c.remote.http.max_body_bytes, 65_536);
        assert_eq!(c.remote.events.position_interval_ms, 250);
        assert!(Config::default().validate().is_empty());
    }

    #[test]
    fn numbers_are_brought_into_range() {
        let mut c = Config::default();
        c.remote.http.port = 80;
        c.remote.http.max_event_clients = 0;
        c.remote.http.request_timeout_ms = 999_999;
        c.remote.http.max_body_bytes = 10;
        c.remote.events.position_interval_ms = 1;
        let w = c.validate();
        assert_eq!(c.remote.http.port, 1024);
        assert_eq!(c.remote.http.max_event_clients, 1);
        assert_eq!(c.remote.http.request_timeout_ms, 120_000);
        assert_eq!(c.remote.http.max_body_bytes, 1024);
        assert_eq!(c.remote.events.position_interval_ms, 50);
        assert_eq!(w.len(), 5);
    }

    #[test]
    fn a_bind_that_is_not_an_ip_address_falls_back_to_loopback() {
        let mut c = Config::default();
        c.remote.http.bind = "localhost".into();
        let w = c.validate();
        assert_eq!(c.remote.http.bind, "127.0.0.1");
        assert_eq!(w[0].field, "remote.http.bind");
        c.remote.http.bind = "::".into();
        assert!(c.validate().is_empty());
        assert!(c.remote.http.bind_addr().is_some());
    }

    #[test]
    fn a_short_token_is_dropped() {
        let mut c = Config::default();
        c.remote.http.token = "short".into();
        let w = c.validate();
        assert!(c.remote.http.token.is_empty());
        assert_eq!(w[0].field, "remote.http.token");
        c.remote.http.token = "0123456789abcdef".into();
        assert!(c.validate().is_empty());
    }

    #[test]
    fn invalid_cors_origins_are_dropped() {
        let mut c = Config::default();
        c.remote.http.cors_origins = vec![
            "https://studio.example".into(),
            "http://10.0.0.5:8080".into(),
            "ftp://x".into(),
            "https://a.example/path".into(),
            "nonsense".into(),
            "https://".into(),
        ];
        let w = c.validate();
        assert_eq!(
            c.remote.http.cors_origins,
            vec!["https://studio.example", "http://10.0.0.5:8080"]
        );
        assert_eq!(w.len(), 4);
    }

    #[test]
    fn a_wildcard_origin_is_only_kept_locally_without_a_token() {
        let mut c = Config::default();
        c.remote.http.cors_origins = vec!["*".into()];
        assert!(c.validate().is_empty());
        c.remote.http.token = "0123456789abcdef".into();
        c.validate();
        assert!(c.remote.http.cors_origins.is_empty());
        c.remote.http.token.clear();
        c.remote.http.bind = "0.0.0.0".into();
        c.remote.http.cors_origins = vec!["*".into()];
        c.validate();
        assert!(c.remote.http.cors_origins.is_empty());
    }

    #[test]
    fn missing_remote_fields_take_their_defaults() {
        let c: Config =
            serde_json::from_str(r#"{"remote":{"http":{"enabled":true}}}"#).unwrap();
        assert!(c.remote.http.enabled);
        assert_eq!(c.remote.http.port, 7380);
        assert_eq!(c.remote.events.position_interval_ms, 250);
    }
}
```

- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p fp-model remote::`
Expected: compile error, `no field remote on type Config`.

- [ ] **Step 3: Implement**

At the top of `crates/fp-model/src/remote.rs`, above the tests:

```rust
use std::net::IpAddr;

use serde::{Deserialize, Serialize};

use crate::config::{ConfigWarning, clamp_to};

/// Remote control over the network: off until the operator turns it on.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RemoteConfig {
    pub http: HttpRemoteConfig,
    pub events: RemoteEventsConfig,
}

/// The HTTP/JSON API (remote control spec §3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HttpRemoteConfig {
    pub enabled: bool,
    /// An IPv4 or IPv6 address literal; "0.0.0.0" or "::" listen everywhere.
    pub bind: String,
    pub port: u16,
    /// Bearer token; mandatory when `bind` is not a loopback address.
    pub token: String,
    /// Web origins allowed to call the API from a browser.
    pub cors_origins: Vec<String>,
    /// Event streams open at once (remote control spec §5.2).
    pub max_event_clients: u32,
    pub request_timeout_ms: u32,
    pub max_body_bytes: u32,
}

impl Default for HttpRemoteConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bind: "127.0.0.1".to_owned(),
            port: 7380,
            token: String::new(),
            cors_origins: Vec::new(),
            max_event_clients: 16,
            request_timeout_ms: 10_000,
            max_body_bytes: 65_536,
        }
    }
}

/// What remote clients are told, and how often.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RemoteEventsConfig {
    /// How often elapsed and remaining times are published while playing.
    pub position_interval_ms: u32,
}

impl Default for RemoteEventsConfig {
    fn default() -> Self {
        Self {
            position_interval_ms: 250,
        }
    }
}

/// Shortest token accepted: guessing 16 random characters is out of reach.
const MIN_TOKEN_CHARS: usize = 16;

impl HttpRemoteConfig {
    /// The address to listen on, if `bind` is an IP literal.
    pub fn bind_addr(&self) -> Option<IpAddr> {
        self.bind.parse().ok()
    }

    fn is_local(&self) -> bool {
        self.bind_addr().is_some_and(|ip| ip.is_loopback())
    }
}

/// `scheme://host[:port]` with an http or https scheme and nothing after.
fn is_origin(origin: &str) -> bool {
    let rest = origin
        .strip_prefix("https://")
        .or_else(|| origin.strip_prefix("http://"));
    rest.is_some_and(|host| {
        !host.is_empty()
            && !host.contains('/')
            && !host.chars().any(char::is_whitespace)
    })
}

impl RemoteConfig {
    pub(crate) fn validate(&mut self, w: &mut Vec<ConfigWarning>) {
        let h = &mut self.http;
        if h.bind_addr().is_none() {
            w.push(ConfigWarning {
                field: "remote.http.bind",
                message: format!("{:?} is not an IP address; using 127.0.0.1", h.bind),
            });
            h.bind = "127.0.0.1".to_owned();
        }
        clamp_to(&mut h.port, 1024, u16::MAX, "remote.http.port", w);
        if !h.token.is_empty() && h.token.chars().count() < MIN_TOKEN_CHARS {
            h.token.clear();
            w.push(ConfigWarning {
                field: "remote.http.token",
                message: format!("shorter than {MIN_TOKEN_CHARS} characters; ignored"),
            });
        }
        let wildcard_ok = h.is_local() && h.token.is_empty();
        h.cors_origins.retain(|o| {
            let ok = if o == "*" { wildcard_ok } else { is_origin(o) };
            if !ok {
                w.push(ConfigWarning {
                    field: "remote.http.cors_origins",
                    message: format!("{o:?} cannot be used; dropped"),
                });
            }
            ok
        });
        clamp_to(&mut h.max_event_clients, 1, 256, "remote.http.max_event_clients", w);
        clamp_to(
            &mut h.request_timeout_ms,
            1000,
            120_000,
            "remote.http.request_timeout_ms",
            w,
        );
        clamp_to(&mut h.max_body_bytes, 1024, 1_048_576, "remote.http.max_body_bytes", w);
        clamp_to(
            &mut self.events.position_interval_ms,
            50,
            5000,
            "remote.events.position_interval_ms",
            w,
        );
    }
}
```

In `crates/fp-model/src/config.rs`:
- add `/// Remote control over the network (remote control spec).` and
  `pub remote: crate::remote::RemoteConfig,` after `midi` in `Config`;
- add `remote: crate::remote::RemoteConfig::default(),` to `Default`;
- change `fn clamp_to` to `pub(crate) fn clamp_to`;
- in `validate`, after `self.midi.validate(&mut w);`, add
  `self.remote.validate(&mut w);`.

In `crates/fp-model/src/lib.rs`, add `pub mod remote;` (alphabetical, after
`reducer`) and
`pub use remote::{HttpRemoteConfig, RemoteConfig, RemoteEventsConfig};`.

- [ ] **Step 4: Run the tests to see them pass**

Run: `cargo test -p fp-model`
Expected: all pass, including `defaults_are_valid`.

- [ ] **Step 5: Lenient loading check**

Add to `crates/fp-store/src/lenient.rs` tests (next to the existing ones):

```rust
#[test]
fn a_bad_remote_port_keeps_the_other_remote_fields() {
    let user: serde_json::Value =
        serde_json::from_str(r#"{"remote":{"http":{"enabled":true,"port":"x"}}}"#).unwrap();
    let mut warnings = Vec::new();
    let c = config_from_value(&user, &mut warnings);
    assert!(c.remote.http.enabled);
    assert_eq!(c.remote.http.port, 7380);
    assert_eq!(warnings.len(), 1);
}
```

Run: `cargo test -p fp-store lenient`
Expected: PASS. The lenient loader is generic, so no code change is needed.

- [ ] **Step 6: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace \
 && git add crates/fp-model crates/fp-store && git commit -m "feat(model): remote control configuration

Off and loopback-only by default; a short token, a bad bind or an
unsafe wildcard origin falls back with a warning, as every other
configuration value does.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: crate `fp-remote`, control trait and DTOs

**Files:**
- Modify: `Cargo.toml` (workspace dependencies)
- Modify: `release-please-config.json` (bump `fp-remote` in `Cargo.lock`)
- Create: `crates/fp-remote/Cargo.toml`, `crates/fp-remote/src/lib.rs`,
  `crates/fp-remote/src/control.rs`, `crates/fp-remote/src/dto.rs`
- Create: `crates/fp-remote/tests/support/mod.rs`, `crates/fp-remote/tests/dto.rs`

**Interfaces:**
- Consumes: `fp_model::*`, `fp_model::volume::fader_from_gain`.
- Produces:
  - `fp_remote::control::{RemoteControl, Playback, WaveformData}`;
  - `fp_remote::dto::{state, players, player, playlists, playlist, track, cartwall}`,
    and the `*Dto` types listed below;
  - test support `support::{FakeControl, demo_state}`.

- [ ] **Step 1: Workspace and crate manifest**

Add to `[workspace.dependencies]` in `Cargo.toml`:

```toml
fp-remote = { path = "crates/fp-remote" }
axum = { version = "0.8.9", default-features = false, features = ["http1", "json", "tokio"] }
tokio = { version = "1.53.1", features = ["rt", "net", "time", "sync", "macros"] }
tower = { version = "0.5.3", features = ["util", "limit"] }
tower-http = { version = "0.7.1", features = ["cors", "timeout", "limit"] }
http-body-util = "0.1.5"
```

`crates/fp-remote/Cargo.toml`:

```toml
[package]
name = "fp-remote"
description = "Remote control over the network: HTTP/JSON API"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
publish.workspace = true

[dependencies]
fp-model.workspace = true
arc-swap.workspace = true
axum.workspace = true
tokio.workspace = true
tower.workspace = true
tower-http.workspace = true
serde.workspace = true
serde_json.workspace = true
tracing.workspace = true

[dev-dependencies]
http-body-util.workspace = true

[lints]
workspace = true
```

In `release-please-config.json`, add after the `fp-control` entry:

```json
        {
          "type": "toml",
          "path": "Cargo.lock",
          "jsonpath": "$.package[?(@.name.value==='fp-remote')].version"
        },
```

- [ ] **Step 2: Control trait** (`crates/fp-remote/src/control.rs`)

```rust
//! What the remote server needs from the application (remote control spec
//! §7). `fp-app` implements it over the conductor and the caches, so this
//! crate depends on nothing but the model.

use std::sync::Arc;

use fp_model::{AppState, CartId, Command, PlayerId, TrackId};

/// Engine telemetry the API reports.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Playback {
    /// The conductor's model version: it rises with every model change.
    pub revision: u64,
    /// Position of each player's current source, in track seconds.
    pub players: Vec<(PlayerId, f64)>,
    /// Carts on air and their position in the file, in firing order.
    pub carts: Vec<(CartId, f64)>,
}

impl Playback {
    pub fn player_position(&self, player: PlayerId) -> Option<f64> {
        self.players.iter().find(|(id, _)| *id == player).map(|(_, s)| *s)
    }

    pub fn cart_position(&self, cart: CartId) -> Option<f64> {
        self.carts.iter().find(|(id, _)| *id == cart).map(|(_, s)| *s)
    }
}

/// A track's waveform: min, max and RMS per bucket, full scale `i16::MAX`.
#[derive(Debug, Clone, PartialEq)]
pub struct WaveformData {
    pub bucket_secs: f64,
    pub peaks: Vec<[i16; 3]>,
}

pub trait RemoteControl: Send + Sync + 'static {
    /// The current model snapshot.
    fn model(&self) -> Arc<AppState>;
    fn playback(&self) -> Playback;
    /// Queues a command; never blocks. False when the queue is full.
    fn send(&self, command: Command) -> bool;
    /// The cover thumbnail (PNG). May read the disk: call it off the
    /// runtime's thread (`spawn_blocking`).
    fn cover(&self, track: TrackId) -> Option<Vec<u8>>;
    /// The waveform. May read the disk, like `cover`.
    fn peaks(&self, track: TrackId) -> Option<WaveformData>;
}
```

`crates/fp-remote/src/lib.rs`:

```rust
//! Remote control over the network (remote control spec): an HTTP/JSON API
//! on its own thread. Requests become the same commands the interface
//! sends; the remote thread is never an audio or UI thread.

#![deny(clippy::indexing_slicing)]

pub mod control;
pub mod dto;
```

- [ ] **Step 3: Test support** (`crates/fp-remote/tests/support/mod.rs`)

```rust
//! A control that serves a model held in memory and records commands.
#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use arc_swap::ArcSwap;
use fp_model::{AppState, Command, Config, TrackId};
use fp_remote::control::{Playback, RemoteControl, WaveformData};

pub struct FakeControl {
    pub state: ArcSwap<AppState>,
    pub playback: ArcSwap<Playback>,
    pub sent: Mutex<Vec<Command>>,
    /// False simulates a full command queue.
    pub accept: AtomicBool,
    pub covers: Mutex<HashMap<TrackId, Vec<u8>>>,
    pub peaks: Mutex<HashMap<TrackId, WaveformData>>,
}

impl FakeControl {
    pub fn new(state: AppState) -> Arc<Self> {
        Arc::new(Self {
            state: ArcSwap::from_pointee(state),
            playback: ArcSwap::from_pointee(Playback::default()),
            sent: Mutex::new(Vec::new()),
            accept: AtomicBool::new(true),
            covers: Mutex::new(HashMap::new()),
            peaks: Mutex::new(HashMap::new()),
        })
    }

    pub fn take_sent(&self) -> Vec<Command> {
        std::mem::take(&mut *self.sent.lock().unwrap())
    }

    pub fn edit(&self, f: impl FnOnce(&mut AppState)) {
        let mut s = (**self.state.load()).clone();
        f(&mut s);
        self.state.store(Arc::new(s));
    }
}

impl RemoteControl for FakeControl {
    fn model(&self) -> Arc<AppState> {
        self.state.load_full()
    }
    fn playback(&self) -> Playback {
        (**self.playback.load()).clone()
    }
    fn send(&self, command: Command) -> bool {
        if !self.accept.load(Ordering::SeqCst) {
            return false;
        }
        self.sent.lock().unwrap().push(command);
        true
    }
    fn cover(&self, track: TrackId) -> Option<Vec<u8>> {
        self.covers.lock().unwrap().get(&track).cloned()
    }
    fn peaks(&self, track: TrackId) -> Option<WaveformData> {
        self.peaks.lock().unwrap().get(&track).cloned()
    }
}

/// Four players over one playlist of three tracks (180 s each); a second,
/// empty playlist "Night"; one cart page with a cart holding the first track.
pub fn demo_state() -> AppState {
    let mut s = AppState::new(Config::default(), "Main");
    let main = s.playlists.first_id().unwrap();
    fp_model::apply(
        &mut s,
        Command::InsertPaths {
            playlist: main,
            index: 0,
            paths: ["a", "b", "c"].map(|n| PathBuf::from(format!("/m/{n}.flac"))).to_vec(),
        },
    )
    .unwrap();
    fp_model::apply(&mut s, Command::CreatePlaylist { name: "Night".into() }).unwrap();
    let page = s.cartwall.pages.first().unwrap().id;
    fp_model::apply(
        &mut s,
        Command::AssignCartFile {
            page,
            index: 0,
            path: PathBuf::from("/m/a.flac"),
        },
    )
    .unwrap();
    let ids: Vec<TrackId> = s.library.iter().map(|t| t.id).collect();
    for id in ids {
        let t = s.library.get_mut(id).unwrap();
        t.duration_secs = 180.0;
        t.title = format!("Title {}", id.0);
        t.artist = "Artist".into();
    }
    s
}
```

- [ ] **Step 4: Write the failing DTO tests** (`crates/fp-remote/tests/dto.rs`)

```rust
#![allow(clippy::unwrap_used)]
mod support;

use fp_model::{Command, MarkerKind, PlayerId};
use fp_remote::control::Playback;
use fp_remote::dto;
use support::demo_state;

#[test]
fn the_state_lists_players_in_display_order_with_one_based_positions() {
    let s = demo_state();
    let out = dto::state(&s, &Playback { revision: 7, ..Default::default() });
    assert_eq!(out.revision, 7);
    assert_eq!(out.players.len(), 4);
    assert_eq!(out.players[0].position, 1);
    assert_eq!(out.players[3].position, 4);
    assert_eq!(out.players[0].transport, "stopped");
    assert_eq!(out.players[0].mode, "continuous");
    assert_eq!(out.playlists.len(), 2);
    assert_eq!(out.playlists[0].entry_count, 3);
}

#[test]
fn a_player_reports_next_fader_and_times_like_the_ui() {
    let mut s = demo_state();
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    s.players[0].volume = 1.0;
    let playback = Playback { players: vec![(p, 30.0)], ..Default::default() };
    let out = dto::player(&s, &playback, p).unwrap();
    assert_eq!(out.transport, "playing");
    assert_eq!(out.fader, 1.0);
    let current = out.current.unwrap();
    assert_eq!(current.track.title, format!("Title {}", current.track.id.0));
    assert_eq!(out.elapsed_secs, Some(30.0));
    assert_eq!(out.remaining_secs, Some(150.0));
    assert!(out.next.is_some());
    assert!(dto::player(&s, &playback, PlayerId(999_999)).is_none());
}

#[test]
fn a_stopped_player_without_current_has_no_times() {
    let s = demo_state();
    let out = dto::player(&s, &Playback::default(), s.players[1].id).unwrap();
    assert!(out.current.is_none());
    assert_eq!(out.elapsed_secs, None);
    assert_eq!(out.remaining_secs, None);
}

#[test]
fn playlist_entries_say_who_plays_them() {
    let mut s = demo_state();
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    let list = s.playlists.first_id().unwrap();
    let out = dto::playlist(&s, list).unwrap();
    assert_eq!(out.entries.len(), 3);
    assert_eq!(out.entries[0].on_air, vec![p]);
    assert!(out.entries[1].next_on.contains(&p));
    assert!(!out.entries[0].repeat);
}

#[test]
fn a_track_carries_markers_and_hides_its_path() {
    let mut s = demo_state();
    let id = s.playlists.iter().next().unwrap().entries[0].track;
    fp_model::apply(
        &mut s,
        Command::SetMarker { track: id, kind: MarkerKind::IntroEnd, secs: Some(12.0) },
    )
    .unwrap();
    let out = dto::track(&s, id).unwrap();
    assert_eq!(out.kind, "music");
    assert_eq!(out.file_state, "ok");
    let intro = out.markers.intro_end.unwrap();
    assert_eq!(intro.secs, 12.0);
    assert_eq!(intro.source, "manual");
    let json = serde_json::to_string(&out).unwrap();
    assert!(!json.contains("/m/"), "{json}");
}

#[test]
fn the_cartwall_shows_its_page_and_carts() {
    let s = demo_state();
    let out = dto::cartwall(&s, &Playback::default());
    let page = &out.pages[0];
    assert_eq!(out.shown_page, Some(page.id));
    assert_eq!(page.carts[0].index, 0);
    assert!(page.carts[0].track.is_some());
    assert!(page.carts[1].track.is_none());
    assert_eq!(page.carts[0].kind, "jingle");
    assert!(out.playing.is_empty());
    assert_eq!(out.cue, None);
}

#[test]
fn a_playing_cart_reports_its_times() {
    let mut s = demo_state();
    let cart = s.cartwall.pages[0].carts[0].id;
    fp_model::apply(&mut s, Command::FireCart(cart)).unwrap();
    let playback = Playback { carts: vec![(cart, 10.0)], ..Default::default() };
    let out = dto::cartwall(&s, &playback);
    assert_eq!(out.playing[0].cart, cart);
    assert_eq!(out.playing[0].elapsed_secs, Some(10.0));
    assert_eq!(out.playing[0].remaining_secs, Some(170.0));
}
```

Run: `cargo test -p fp-remote --test dto`
Expected: compile error, `could not find dto`.

If `Command::Play` on a stopped player with `next` does not set `current`
in the pure model (check `reducer.rs` `fn play`), set the expected values from
what `apply` produces and record a `Ruling:` in the ledger.

- [ ] **Step 5: Implement `crates/fp-remote/src/dto.rs`**

```rust
//! The JSON shapes of `/api/v1` (remote control spec §3.1). They are the
//! API's contract: the model's own serialisation never leaks out, and file
//! paths are never exposed.

use fp_model::volume::fader_from_gain;
use fp_model::{
    AppState, CartId, CartKind, CartPage, CartPageId, EntryId, FileState, Marker, MarkerSource,
    PlayMode, PlayerId, PlayerState, Playlist, PlaylistId, Track, TrackId, TrackKind, Transport,
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
    /// The playlist shown in the player.
    pub playlist: PlaylistId,
    pub current: Option<EntryRefDto>,
    pub next: Option<EntryRefDto>,
    pub cue: Option<CueDto>,
    pub elapsed_secs: Option<f64>,
    pub remaining_secs: Option<f64>,
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
    pub track: Option<TrackDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlayingCartDto {
    pub cart: CartId,
    pub elapsed_secs: Option<f64>,
    pub remaining_secs: Option<f64>,
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
    model.players.get(i).map(|p| player_dto(model, playback, i, p))
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
                model.players.iter().filter(|p| f(p)).map(|p| p.id).collect()
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
            PlayingCartDto {
                cart: pc.cart,
                elapsed_secs: elapsed,
                remaining_secs: track
                    .zip(elapsed)
                    .map(|(t, e)| (t.cue_out_secs() - e).max(0.0)),
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
    let current = p.current.and_then(|e| model.track_for_entry(e));
    let elapsed = current.map(|t| {
        playback
            .player_position(p.id)
            .filter(|v| v.is_finite())
            .unwrap_or_else(|| t.cue_in_secs())
    });
    PlayerDto {
        id: p.id,
        position: index + 1,
        transport: transport_name(p.transport),
        fading: p.fading,
        mode: mode_name(p.mode),
        stop_after_current: p.stop_after_current,
        fader: fader_from_gain(p.volume),
        playlist: p.playlist,
        current: p.current.and_then(|e| entry_ref(model, e)),
        next: p.next.and_then(|e| entry_ref(model, e)),
        cue: p.cue.map(|c| CueDto { entry: c.entry }),
        elapsed_secs: elapsed,
        remaining_secs: current
            .zip(elapsed)
            .map(|(t, e)| (t.cue_out_secs() - e).max(0.0)),
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
```

- [ ] **Step 6: Run the tests to see them pass**

Run: `cargo test -p fp-remote --test dto`
Expected: 7 passed.

- [ ] **Step 7: Licences**

Run: `cargo deny check`
Expected: `advisories ok, bans ok, licenses ok, sources ok`. If a new
transitive licence is refused, stop and record it in the ledger. Do not
widen `deny.toml` without the maintainer.

- [ ] **Step 8: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace \
 && git add Cargo.toml Cargo.lock release-please-config.json crates/fp-remote \
 && git commit -m "feat(remote): crate with the control trait and the API's JSON contract

The DTOs are the stable /api/v1 contract, built from a model snapshot
and the engine's telemetry; file paths never leave the machine.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: operations → commands (`api.rs`)

**Files:**
- Create: `crates/fp-remote/src/api.rs`
- Modify: `crates/fp-remote/src/lib.rs` (`pub mod api;`)
- Test: `crates/fp-remote/tests/api.rs`

**Interfaces:**
- Consumes: `fp_model::{apply, command_available, AppState, Command, ModelError, …}`.
- Produces:
  - `fp_remote::api::Operation` (the variants listed below);
  - `fp_remote::api::plan(&AppState, Operation) -> Result<Vec<Command>, ApiError>`;
  - `fp_remote::api::ApiError` with `status() -> u16`, `code() -> &'static str`
    and `message() -> String`;
  - `ApiError::from_model(ModelError) -> ApiError`.

- [ ] **Step 1: Write the failing tests** (`crates/fp-remote/tests/api.rs`)

```rust
#![allow(clippy::unwrap_used)]
mod support;

use fp_model::volume::gain_from_fader;
use fp_model::{CartId, Command, EntryId, PlayMode, PlayerId, PlaylistId};
use fp_remote::api::{ApiError, Operation as O, plan};
use support::demo_state;

#[test]
fn play_on_a_player_with_a_next_entry_is_one_play_command() {
    let s = demo_state();
    let p = s.players[0].id;
    assert_eq!(plan(&s, O::Play(p)).unwrap(), vec![Command::Play(p)]);
}

#[test]
fn an_unavailable_action_is_a_conflict() {
    let s = demo_state();
    let p = s.players[0].id;
    // Nothing is on air: Pause, Stop, Fade stop, Restart and Previous are off (R28).
    for op in [O::Pause(p), O::Stop(p), O::FadeStop(p), O::Restart(p), O::Previous(p)] {
        let e = plan(&s, op).unwrap_err();
        assert_eq!(e.status(), 409, "{op:?}");
        assert_eq!(e.code(), "unavailable");
    }
}

#[test]
fn an_unknown_player_is_not_found_even_for_transport() {
    let s = demo_state();
    assert_eq!(plan(&s, O::Play(PlayerId(999_999))).unwrap_err(), ApiError::NotFound);
    assert_eq!(
        plan(&s, O::SetFader(PlayerId(999_999), 0.5)).unwrap_err(),
        ApiError::NotFound
    );
}

#[test]
fn toggles_take_the_desired_value() {
    let mut s = demo_state();
    let p = s.players[0].id;
    assert_eq!(plan(&s, O::SetStopAfterCurrent(p, false)).unwrap(), vec![]);
    assert_eq!(
        plan(&s, O::SetStopAfterCurrent(p, true)).unwrap(),
        vec![Command::ToggleStopAfterCurrent(p)]
    );
    assert_eq!(plan(&s, O::SetCue(p, false)).unwrap(), vec![]);
    assert_eq!(plan(&s, O::SetCue(p, true)).unwrap(), vec![Command::ToggleCue(p)]);
    let e = s.playlists.iter().next().unwrap().entries[0].id;
    assert_eq!(plan(&s, O::SetEntryRepeat(e, false)).unwrap(), vec![]);
    fp_model::apply(&mut s, Command::ToggleEntryRepeat(e)).unwrap();
    assert_eq!(plan(&s, O::SetEntryRepeat(e, true)).unwrap(), vec![]);
    assert_eq!(
        plan(&s, O::SetEntryRepeat(e, false)).unwrap(),
        vec![Command::ToggleEntryRepeat(e)]
    );
}

#[test]
fn stop_after_current_in_single_mode_is_a_conflict() {
    let mut s = demo_state();
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::SetMode(p, PlayMode::Single)).unwrap();
    assert_eq!(plan(&s, O::SetStopAfterCurrent(p, true)).unwrap_err().status(), 409);
}

#[test]
fn the_fader_maps_through_the_ui_curve_and_must_be_in_range() {
    let s = demo_state();
    let p = s.players[0].id;
    assert_eq!(
        plan(&s, O::SetFader(p, 0.8)).unwrap(),
        vec![Command::SetVolume(p, gain_from_fader(0.8))]
    );
    for bad in [-0.1, 1.1, f32::NAN, f32::INFINITY] {
        assert_eq!(plan(&s, O::SetFader(p, bad)).unwrap_err().status(), 400, "{bad}");
    }
}

#[test]
fn seek_needs_a_running_entry_and_a_position_inside_its_cue_range() {
    let mut s = demo_state();
    let p = s.players[0].id;
    assert_eq!(plan(&s, O::Seek(p, 10.0)).unwrap_err().status(), 409);
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(plan(&s, O::Seek(p, 10.0)).unwrap(), vec![Command::Seek(p, 10.0)]);
    assert_eq!(plan(&s, O::Seek(p, 500.0)).unwrap_err().status(), 400);
    assert_eq!(plan(&s, O::Seek(p, f64::NAN)).unwrap_err().status(), 400);
}

#[test]
fn next_and_cue_entry_need_known_ids() {
    let s = demo_state();
    let p = s.players[0].id;
    let e = s.playlists.iter().next().unwrap().entries[2].id;
    assert_eq!(plan(&s, O::SetNext(p, e)).unwrap(), vec![Command::SetNext(p, e)]);
    assert_eq!(plan(&s, O::CueEntry(p, e)).unwrap(), vec![Command::CueEntry(p, e)]);
    assert_eq!(plan(&s, O::SetNext(p, EntryId(999_999))).unwrap_err(), ApiError::NotFound);
}

#[test]
fn the_dry_run_turns_a_model_refusal_into_a_conflict() {
    let mut s = demo_state();
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    let current = s.players[0].current.unwrap();
    let e = plan(&s, O::SetNext(p, current)).unwrap_err();
    assert_eq!(e.status(), 409);
    assert!(e.message().contains("current"), "{}", e.message());
}

#[test]
fn an_id_that_vanished_is_not_found() {
    let mut s = demo_state();
    let night = s.playlists.iter().nth(1).unwrap().id;
    let p = s.players[0].id;
    assert_eq!(
        plan(&s, O::ShowPlaylist(p, night)).unwrap(),
        vec![Command::ShowPlaylist(p, night)]
    );
    fp_model::apply(&mut s, Command::DeletePlaylist(night)).unwrap();
    assert_eq!(plan(&s, O::ShowPlaylist(p, night)).unwrap_err(), ApiError::NotFound);
    assert_eq!(
        plan(&s, O::ShowPlaylist(p, PlaylistId(424_242))).unwrap_err(),
        ApiError::NotFound
    );
}

#[test]
fn carts_fire_stop_and_cue_by_id() {
    let s = demo_state();
    let c = s.cartwall.pages[0].carts[0].id;
    assert_eq!(plan(&s, O::FireCart(c)).unwrap(), vec![Command::FireCart(c)]);
    assert_eq!(plan(&s, O::StopCart(c)).unwrap(), vec![Command::StopCart(c)]);
    assert_eq!(plan(&s, O::SetCartCue(c, false)).unwrap(), vec![]);
    assert_eq!(plan(&s, O::SetCartCue(c, true)).unwrap(), vec![Command::CueCart(c)]);
    assert_eq!(plan(&s, O::StopAllCarts).unwrap(), vec![Command::StopAllCarts]);
    assert_eq!(plan(&s, O::FireCart(CartId(999_999))).unwrap_err(), ApiError::NotFound);
    let page = s.cartwall.pages[0].id;
    assert_eq!(plan(&s, O::ShowCartPage(page)).unwrap(), vec![Command::ShowCartPage(page)]);
}

#[test]
fn every_error_has_its_status_and_code() {
    let cases = [
        (ApiError::BadRequest("x".into()), 400, "bad_request"),
        (ApiError::Unauthorized, 401, "unauthorized"),
        (ApiError::ForbiddenOrigin, 403, "forbidden_origin"),
        (ApiError::NotFound, 404, "not_found"),
        (ApiError::NotAnalyzed, 404, "not_analyzed"),
        (ApiError::Unavailable("x".into()), 409, "unavailable"),
        (ApiError::PayloadTooLarge, 413, "payload_too_large"),
        (ApiError::UnsupportedMediaType, 415, "unsupported_media_type"),
        (ApiError::Busy, 503, "busy"),
    ];
    for (e, status, code) in cases {
        assert_eq!((e.status(), e.code()), (status, code));
        assert!(!e.message().is_empty());
    }
}
```

Run: `cargo test -p fp-remote --test api`
Expected: compile error, `could not find api`.

- [ ] **Step 2: Implement `crates/fp-remote/src/api.rs`**

```rust
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
                return Err(ApiError::BadRequest("fader must be within 0..=1".to_owned()));
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
```

Add `pub mod api;` to `lib.rs`.

- [ ] **Step 3: Run the tests**

Run: `cargo test -p fp-remote --test api`
Expected: 12 passed. If a test fails because the pure model behaves
differently from what is assumed here (for example `ToggleCue` without a
`next`), fix the test to match the model, not the model. Record a `Ruling:`
in the ledger.

- [ ] **Step 4: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace \
 && git add crates/fp-remote && git commit -m "feat(remote): turn operations into checked model commands

Availability (R28), desired values instead of toggles, and a dry run on
a copy of the snapshot, so a refused request is answered at once and
nothing that the model would refuse is queued.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: HTTP router, reads and operations

**Files:**
- Create: `crates/fp-remote/src/http/mod.rs`, `crates/fp-remote/src/http/json.rs`,
  `crates/fp-remote/src/http/handlers.rs`
- Modify: `crates/fp-remote/src/lib.rs` (`pub mod http;`)
- Test: `crates/fp-remote/tests/http.rs`

**Interfaces:**
- Consumes: `api::{plan, Operation, ApiError}`, `dto::*`, `control::RemoteControl`,
  `fp_model::HttpRemoteConfig`.
- Produces:
  - `fp_remote::http::Ctx { control: Arc<dyn RemoteControl>, config: Arc<HttpRemoteConfig> }`
    and `Ctx::new(control, config)`;
  - `fp_remote::http::router(ctx: Ctx) -> axum::Router`. Task 5 adds the
    guard and limits inside it, and Task 6 serves it.

- [ ] **Step 1: Write the failing tests** (`crates/fp-remote/tests/http.rs`)

```rust
#![allow(clippy::unwrap_used)]
mod support;

use std::sync::Arc;
use std::sync::atomic::Ordering;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use fp_model::{Command, HttpRemoteConfig};
use fp_remote::control::{Playback, WaveformData};
use fp_remote::http::{Ctx, router};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use support::{FakeControl, demo_state};
use tower::ServiceExt;

pub fn ctx(fake: &Arc<FakeControl>) -> Ctx {
    Ctx::new(fake.clone(), Arc::new(HttpRemoteConfig::default()))
}

/// Sends a request with `Host: 127.0.0.1:7380` (the guard of Task 5 needs it).
pub async fn call(
    ctx: Ctx,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value, Vec<u8>) {
    let mut req = Request::builder()
        .method(method)
        .uri(uri)
        .header("host", "127.0.0.1:7380");
    let body = match body {
        Some(v) => {
            req = req.header("content-type", "application/json");
            Body::from(v.to_string())
        }
        None => Body::empty(),
    };
    let res = router(ctx).oneshot(req.body(body).unwrap()).await.unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes().to_vec();
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json, bytes)
}

#[tokio::test]
async fn get_state_returns_players_playlists_and_cartwall() {
    let fake = FakeControl::new(demo_state());
    fake.playback.store(Arc::new(Playback { revision: 3, ..Default::default() }));
    let (status, body, _) = call(ctx(&fake), "GET", "/api/v1/state", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["revision"], 3);
    assert_eq!(body["players"].as_array().unwrap().len(), 4);
    assert_eq!(body["playlists"][0]["entry_count"], 3);
    assert!(body["cartwall"]["pages"].is_array());
}

#[tokio::test]
async fn single_resources_are_found_by_id() {
    let s = demo_state();
    let p = s.players[0].id.0;
    let list = s.playlists.first_id().unwrap().0;
    let track = s.playlists.iter().next().unwrap().entries[0].track.0;
    let fake = FakeControl::new(s);
    for uri in [
        format!("/api/v1/players/{p}"),
        "/api/v1/players".to_owned(),
        "/api/v1/playlists".to_owned(),
        format!("/api/v1/playlists/{list}"),
        format!("/api/v1/tracks/{track}"),
        "/api/v1/cartwall".to_owned(),
    ] {
        let (status, _, _) = call(ctx(&fake), "GET", &uri, None).await;
        assert_eq!(status, StatusCode::OK, "{uri}");
    }
}

#[tokio::test]
async fn malformed_or_unknown_ids_and_paths_are_json_404s() {
    let fake = FakeControl::new(demo_state());
    for (method, uri) in [
        ("GET", "/api/v1/players/abc"),
        ("GET", "/api/v1/players/99999999999999999999999"),
        ("GET", "/api/v1/players/424242"),
        ("POST", "/api/v1/players/1/dance"),
        ("GET", "/api/v1/nothing"),
        ("GET", "/"),
    ] {
        let (status, body, _) = call(ctx(&fake), method, uri, None).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{uri}");
        assert_eq!(body["error"], "not_found", "{uri}");
    }
}

#[tokio::test]
async fn play_is_accepted_and_queued() {
    let s = demo_state();
    let p = s.players[0].id;
    let fake = FakeControl::new(s);
    fake.playback.store(Arc::new(Playback { revision: 9, ..Default::default() }));
    let (status, body, _) =
        call(ctx(&fake), "POST", &format!("/api/v1/players/{}/play", p.0), None).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(body["revision"], 9);
    assert_eq!(fake.take_sent(), vec![Command::Play(p)]);
}

#[tokio::test]
async fn every_transport_action_has_its_route() {
    let s = demo_state();
    let p = s.players[0].id.0;
    let fake = FakeControl::new(s);
    for action in ["pause", "stop", "fade-stop", "restart", "previous"] {
        let (status, body, _) =
            call(ctx(&fake), "POST", &format!("/api/v1/players/{p}/{action}"), None).await;
        assert_eq!(status, StatusCode::CONFLICT, "{action}");
        assert_eq!(body["error"], "unavailable");
    }
    assert!(fake.take_sent().is_empty());
}

#[tokio::test]
async fn bodies_are_read_and_checked() {
    let s = demo_state();
    let p = s.players[0].id.0;
    let e = s.playlists.iter().next().unwrap().entries[2].id.0;
    let night = s.playlists.iter().nth(1).unwrap().id.0;
    let c = s.cartwall.pages[0].carts[0].id.0;
    let page = s.cartwall.pages[0].id.0;
    let fake = FakeControl::new(s);
    let ok = [
        ("PUT", format!("/api/v1/players/{p}/volume"), json!({"fader": 0.5})),
        ("PUT", format!("/api/v1/players/{p}/next"), json!({"entry": e})),
        ("POST", format!("/api/v1/players/{p}/cue-entry"), json!({"entry": e})),
        ("PUT", format!("/api/v1/players/{p}/mode"), json!({"mode": "single"})),
        ("PUT", format!("/api/v1/players/{p}/stop-after-current"), json!({"on": false})),
        ("PUT", format!("/api/v1/players/{p}/cue"), json!({"on": true})),
        ("PUT", format!("/api/v1/players/{p}/playlist"), json!({"playlist": night})),
        ("PUT", format!("/api/v1/entries/{e}/repeat"), json!({"on": true})),
        ("PUT", format!("/api/v1/entries/{e}/stop-after"), json!({"on": true})),
        ("PUT", format!("/api/v1/carts/{c}/cue"), json!({"on": true})),
        ("PUT", "/api/v1/cartwall/shown".to_owned(), json!({"page": page})),
    ];
    for (method, uri, body) in ok {
        let (status, out, _) = call(ctx(&fake), method, &uri, Some(body)).await;
        assert_eq!(status, StatusCode::ACCEPTED, "{uri}: {out}");
    }
    for uri in [
        format!("/api/v1/carts/{c}/fire"),
        format!("/api/v1/carts/{c}/stop"),
        "/api/v1/cartwall/stop-all".to_owned(),
    ] {
        let (status, _, _) = call(ctx(&fake), "POST", &uri, None).await;
        assert_eq!(status, StatusCode::ACCEPTED, "{uri}");
    }
}

#[tokio::test]
async fn broken_bodies_are_400s() {
    let s = demo_state();
    let p = s.players[0].id.0;
    let fake = FakeControl::new(s);
    let uri = format!("/api/v1/players/{p}/volume");
    for body in [json!({"fader": "x"}), json!({}), json!({"fader": 2.0}), json!([1])] {
        let (status, out, _) = call(ctx(&fake), "PUT", &uri, Some(body.clone())).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
        assert_eq!(out["error"], "bad_request");
    }
    let (status, _, _) =
        call(ctx(&fake), "PUT", &format!("/api/v1/players/{p}/mode"), Some(json!({"mode": "x"})))
            .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(fake.take_sent().is_empty());
}

#[tokio::test]
async fn a_body_that_is_not_json_is_415() {
    let s = demo_state();
    let p = s.players[0].id.0;
    let fake = FakeControl::new(s);
    let req = Request::put(format!("/api/v1/players/{p}/volume"))
        .header("host", "127.0.0.1:7380")
        .header("content-type", "text/plain")
        .body(Body::from(r#"{"fader":0.5}"#))
        .unwrap();
    let res = router(ctx(&fake)).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[tokio::test]
async fn a_full_queue_is_503() {
    let s = demo_state();
    let p = s.players[0].id.0;
    let fake = FakeControl::new(s);
    fake.accept.store(false, Ordering::SeqCst);
    let (status, body, _) =
        call(ctx(&fake), "POST", &format!("/api/v1/players/{p}/play"), None).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["error"], "busy");
}

#[tokio::test]
async fn cover_and_peaks_come_from_the_control_once_analysed() {
    let mut s = demo_state();
    let t = s.playlists.iter().next().unwrap().entries[0].track;
    let fake = FakeControl::new(s.clone());
    let uri = format!("/api/v1/tracks/{}/peaks", t.0);
    let (status, body, _) = call(ctx(&fake), "GET", &uri, None).await;
    assert_eq!((status, body["error"].clone()), (StatusCode::NOT_FOUND, json!("not_analyzed")));

    s.library.get_mut(t).unwrap().analyzed = true;
    fake.state.store(Arc::new(s));
    let (status, body, _) = call(ctx(&fake), "GET", &uri, None).await;
    assert_eq!((status, body["error"].clone()), (StatusCode::NOT_FOUND, json!("not_found")));

    fake.peaks.lock().unwrap().insert(
        t,
        WaveformData { bucket_secs: 0.05, peaks: vec![[-10, 20, 5]] },
    );
    fake.covers.lock().unwrap().insert(t, vec![0x89, b'P', b'N', b'G']);
    let (status, body, _) = call(ctx(&fake), "GET", &uri, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!({"bucket_secs": 0.05, "full_scale": 32767, "peaks": [[-10, 20, 5]]}));
    let (status, _, bytes) =
        call(ctx(&fake), "GET", &format!("/api/v1/tracks/{}/cover", t.0), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(bytes, vec![0x89, b'P', b'N', b'G']);
}
```

Run: `cargo test -p fp-remote --test http`
Expected: compile error, `could not find http`.

- [ ] **Step 2: JSON body extractor** (`crates/fp-remote/src/http/json.rs`)

```rust
//! Request bodies: JSON only, within the body limit, with errors in the
//! API's format.

use axum::body::Bytes;
use axum::extract::{FromRequest, Request};
use axum::http::{StatusCode, header};
use serde::de::DeserializeOwned;

use crate::api::ApiError;

pub struct JsonBody<T>(pub T);

impl<S: Send + Sync, T: DeserializeOwned> FromRequest<S> for JsonBody<T> {
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, ApiError> {
        let is_json = req
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(';').next())
            .is_some_and(|v| v.trim().eq_ignore_ascii_case("application/json"));
        if !is_json {
            return Err(ApiError::UnsupportedMediaType);
        }
        let bytes = Bytes::from_request(req, state).await.map_err(|r| {
            if r.status() == StatusCode::PAYLOAD_TOO_LARGE {
                ApiError::PayloadTooLarge
            } else {
                ApiError::BadRequest(r.body_text())
            }
        })?;
        serde_json::from_slice(&bytes)
            .map(JsonBody)
            .map_err(|e| ApiError::BadRequest(e.to_string()))
    }
}
```

- [ ] **Step 3: Handlers** (`crates/fp-remote/src/http/handlers.rs`)

```rust
//! One handler per route of remote control spec §3.2–3.3.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use fp_model::{CartId, CartPageId, EntryId, PlayMode, PlayerId, PlaylistId, TrackId};
use serde::Deserialize;
use serde_json::json;

use super::Ctx;
use super::json::JsonBody;
use crate::api::{self, ApiError, Operation};
use crate::dto;

type Reply = Result<Response, ApiError>;

/// A path id; anything that is not a `u64` names nothing.
fn id(raw: &str) -> Result<u64, ApiError> {
    raw.parse().map_err(|_| ApiError::NotFound)
}

fn ok<T: serde::Serialize>(value: T) -> Reply {
    Ok(Json(value).into_response())
}

/// Plans `op` against the current snapshot and queues its commands.
fn run(ctx: &Ctx, op: Operation) -> Reply {
    let model = ctx.control.model();
    for command in api::plan(&model, op)? {
        if !ctx.control.send(command) {
            return Err(ApiError::Busy);
        }
    }
    let revision = ctx.control.playback().revision;
    Ok((StatusCode::ACCEPTED, Json(json!({ "revision": revision }))).into_response())
}

pub async fn state(State(ctx): State<Ctx>) -> Reply {
    ok(dto::state(&ctx.control.model(), &ctx.control.playback()))
}

pub async fn players(State(ctx): State<Ctx>) -> Reply {
    ok(dto::players(&ctx.control.model(), &ctx.control.playback()))
}

pub async fn player(State(ctx): State<Ctx>, Path(raw): Path<String>) -> Reply {
    let p = dto::player(&ctx.control.model(), &ctx.control.playback(), PlayerId(id(&raw)?));
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
    if track.analyzed { Ok(t) } else { Err(ApiError::NotAnalyzed) }
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

pub async fn set_cue(State(ctx): State<Ctx>, Path(raw): Path<String>, JsonBody(b): JsonBody<On>) -> Reply {
    run(&ctx, Operation::SetCue(PlayerId(id(&raw)?), b.on))
}

pub async fn set_next(State(ctx): State<Ctx>, Path(raw): Path<String>, JsonBody(b): JsonBody<Entry>) -> Reply {
    run(&ctx, Operation::SetNext(PlayerId(id(&raw)?), EntryId(b.entry)))
}

pub async fn cue_entry(State(ctx): State<Ctx>, Path(raw): Path<String>, JsonBody(b): JsonBody<Entry>) -> Reply {
    run(&ctx, Operation::CueEntry(PlayerId(id(&raw)?), EntryId(b.entry)))
}

pub async fn seek(State(ctx): State<Ctx>, Path(raw): Path<String>, JsonBody(b): JsonBody<Seek>) -> Reply {
    run(&ctx, Operation::Seek(PlayerId(id(&raw)?), b.secs))
}

pub async fn volume(State(ctx): State<Ctx>, Path(raw): Path<String>, JsonBody(b): JsonBody<Fader>) -> Reply {
    run(&ctx, Operation::SetFader(PlayerId(id(&raw)?), b.fader))
}

pub async fn mode(State(ctx): State<Ctx>, Path(raw): Path<String>, JsonBody(b): JsonBody<Mode>) -> Reply {
    let mode = match b.mode {
        ModeName::Single => PlayMode::Single,
        ModeName::Continuous => PlayMode::Continuous,
    };
    run(&ctx, Operation::SetMode(PlayerId(id(&raw)?), mode))
}

pub async fn stop_after_current(State(ctx): State<Ctx>, Path(raw): Path<String>, JsonBody(b): JsonBody<On>) -> Reply {
    run(&ctx, Operation::SetStopAfterCurrent(PlayerId(id(&raw)?), b.on))
}

pub async fn show_playlist(State(ctx): State<Ctx>, Path(raw): Path<String>, JsonBody(b): JsonBody<PlaylistRef>) -> Reply {
    run(&ctx, Operation::ShowPlaylist(PlayerId(id(&raw)?), PlaylistId(b.playlist)))
}

pub async fn entry_repeat(State(ctx): State<Ctx>, Path(raw): Path<String>, JsonBody(b): JsonBody<On>) -> Reply {
    run(&ctx, Operation::SetEntryRepeat(EntryId(id(&raw)?), b.on))
}

pub async fn entry_stop_after(State(ctx): State<Ctx>, Path(raw): Path<String>, JsonBody(b): JsonBody<On>) -> Reply {
    run(&ctx, Operation::SetEntryStopAfter(EntryId(id(&raw)?), b.on))
}

pub async fn fire_cart(State(ctx): State<Ctx>, Path(raw): Path<String>) -> Reply {
    run(&ctx, Operation::FireCart(CartId(id(&raw)?)))
}

pub async fn stop_cart(State(ctx): State<Ctx>, Path(raw): Path<String>) -> Reply {
    run(&ctx, Operation::StopCart(CartId(id(&raw)?)))
}

pub async fn cart_cue(State(ctx): State<Ctx>, Path(raw): Path<String>, JsonBody(b): JsonBody<On>) -> Reply {
    run(&ctx, Operation::SetCartCue(CartId(id(&raw)?), b.on))
}

pub async fn stop_all_carts(State(ctx): State<Ctx>) -> Reply {
    run(&ctx, Operation::StopAllCarts)
}

pub async fn show_cart_page(State(ctx): State<Ctx>, JsonBody(b): JsonBody<PageRef>) -> Reply {
    run(&ctx, Operation::ShowCartPage(CartPageId(b.page)))
}

pub async fn not_found() -> ApiError {
    ApiError::NotFound
}
```

(`cargo fmt` reflows the long signatures.)

- [ ] **Step 4: Router** (`crates/fp-remote/src/http/mod.rs`)

```rust
//! The HTTP/JSON API (remote control spec §3).

mod handlers;
mod json;

use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use fp_model::HttpRemoteConfig;
use serde_json::json;

use crate::api::ApiError;
use crate::control::RemoteControl;

/// What every handler sees.
#[derive(Clone)]
pub struct Ctx {
    pub control: Arc<dyn RemoteControl>,
    pub config: Arc<HttpRemoteConfig>,
}

impl Ctx {
    pub fn new(control: Arc<dyn RemoteControl>, config: Arc<HttpRemoteConfig>) -> Self {
        Self { control, config }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status()).unwrap_or(StatusCode::BAD_REQUEST);
        let body = json!({ "error": self.code(), "message": self.message() });
        (status, Json(body)).into_response()
    }
}

pub fn router(ctx: Ctx) -> Router {
    use handlers as h;
    let api = Router::new()
        .route("/state", get(h::state))
        .route("/players", get(h::players))
        .route("/players/{id}", get(h::player))
        .route("/players/{id}/{action}", post(h::player_action))
        .route("/players/{id}/cue", put(h::set_cue))
        .route("/players/{id}/next", put(h::set_next))
        .route("/players/{id}/cue-entry", post(h::cue_entry))
        .route("/players/{id}/seek", post(h::seek))
        .route("/players/{id}/volume", put(h::volume))
        .route("/players/{id}/mode", put(h::mode))
        .route("/players/{id}/stop-after-current", put(h::stop_after_current))
        .route("/players/{id}/playlist", put(h::show_playlist))
        .route("/playlists", get(h::playlists))
        .route("/playlists/{id}", get(h::playlist))
        .route("/entries/{id}/repeat", put(h::entry_repeat))
        .route("/entries/{id}/stop-after", put(h::entry_stop_after))
        .route("/tracks/{id}", get(h::track))
        .route("/tracks/{id}/cover", get(h::cover))
        .route("/tracks/{id}/peaks", get(h::peaks))
        .route("/cartwall", get(h::cartwall))
        .route("/cartwall/stop-all", post(h::stop_all_carts))
        .route("/cartwall/shown", put(h::show_cart_page))
        .route("/carts/{id}/fire", post(h::fire_cart))
        .route("/carts/{id}/stop", post(h::stop_cart))
        .route("/carts/{id}/cue", put(h::cart_cue));
    Router::new()
        .nest("/api/v1", api)
        .fallback(h::not_found)
        .with_state(ctx)
}
```

Add `pub mod http;` to `lib.rs`.

Note: `POST /players/{id}/cue` and `POST /players/{id}/volume` match a
static route that has only `PUT`, so axum answers `405 Method Not Allowed`.
That is correct HTTP. The `{action}` route only catches names that have no
route of their own.

- [ ] **Step 5: Run the tests**

Run: `cargo test -p fp-remote --test http`
Expected: 11 passed. If the nested router's fallback does not apply to
unmatched paths under `/api/v1`, add `.fallback(h::not_found)` to `api` as
well.

- [ ] **Step 6: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace \
 && git add crates/fp-remote && git commit -m "feat(remote): HTTP routes to read and operate players and carts

Every route of /api/v1 for reading and operating, with JSON errors for
unknown ids, bad bodies and a full command queue.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: security guard, CORS and limits

**Files:**
- Create: `crates/fp-remote/src/http/guard.rs`
- Modify: `crates/fp-remote/src/http/mod.rs` (layers in `router`, `Ctx`
  gains `rejections`)
- Test: `crates/fp-remote/tests/guard.rs`

**Interfaces:**
- Consumes: `Ctx`, `HttpRemoteConfig`.
- Produces:
  - `fp_remote::http::guard::check(&HttpRemoteConfig, &HeaderMap) -> Result<(), ApiError>`
    (pure);
  - the router enforces it, plus CORS, `max_body_bytes`, `request_timeout_ms`
    and an in-flight limit.

- [ ] **Step 1: Write the failing tests** (`crates/fp-remote/tests/guard.rs`)

```rust
#![allow(clippy::unwrap_used)]
mod support;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{HeaderMap, HeaderValue, Request, StatusCode};
use fp_model::HttpRemoteConfig;
use fp_remote::api::ApiError;
use fp_remote::http::guard::check;
use fp_remote::http::{Ctx, router};
use support::{FakeControl, demo_state};
use tower::ServiceExt;

const TOKEN: &str = "0123456789abcdef0123";

fn headers(pairs: &[(&'static str, &str)]) -> HeaderMap {
    let mut h = HeaderMap::new();
    for (k, v) in pairs {
        h.insert(*k, HeaderValue::from_str(v).unwrap());
    }
    h
}

fn local() -> HttpRemoteConfig {
    HttpRemoteConfig::default()
}

fn lan() -> HttpRemoteConfig {
    HttpRemoteConfig { bind: "0.0.0.0".into(), token: TOKEN.into(), ..Default::default() }
}

#[test]
fn locally_without_a_token_any_loopback_host_passes() {
    for host in ["127.0.0.1:7380", "localhost:7380", "LOCALHOST", "[::1]:7380", "127.1.2.3"] {
        assert_eq!(check(&local(), &headers(&[("host", host)])), Ok(()), "{host}");
    }
}

#[test]
fn locally_a_foreign_host_is_refused() {
    for host in ["evil.example", "evil.example:7380", "192.168.1.10:7380", ""] {
        assert_eq!(
            check(&local(), &headers(&[("host", host)])),
            Err(ApiError::ForbiddenOrigin),
            "{host:?}"
        );
    }
    assert_eq!(check(&local(), &HeaderMap::new()), Err(ApiError::ForbiddenOrigin));
}

#[test]
fn a_foreign_origin_is_refused_whatever_the_bind() {
    let h = headers(&[("host", "127.0.0.1:7380"), ("origin", "https://evil.example")]);
    assert_eq!(check(&local(), &h), Err(ApiError::ForbiddenOrigin));
    let h = headers(&[
        ("host", "10.0.0.2:7380"),
        ("origin", "https://evil.example"),
        ("authorization", &format!("Bearer {TOKEN}")),
    ]);
    assert_eq!(check(&lan(), &h), Err(ApiError::ForbiddenOrigin));
}

#[test]
fn a_listed_origin_and_the_wildcard_pass() {
    let mut c = local();
    c.cors_origins = vec!["https://studio.example".into()];
    let h = headers(&[("host", "127.0.0.1:7380"), ("origin", "https://studio.example")]);
    assert_eq!(check(&c, &h), Ok(()));
    c.cors_origins = vec!["*".into()];
    let h = headers(&[("host", "127.0.0.1:7380"), ("origin", "https://any.example")]);
    assert_eq!(check(&c, &h), Ok(()));
}

#[test]
fn with_a_token_the_bearer_must_match_and_any_host_is_fine() {
    let ok = headers(&[("host", "studio-pc:7380"), ("authorization", &format!("Bearer {TOKEN}"))]);
    assert_eq!(check(&lan(), &ok), Ok(()));
    for auth in ["", "Bearer", "Bearer wrong", "Basic abc", &format!("bearer {TOKEN}x")] {
        let h = headers(&[("host", "studio-pc:7380"), ("authorization", auth)]);
        assert_eq!(check(&lan(), &h), Err(ApiError::Unauthorized), "{auth:?}");
    }
    let h = headers(&[("host", "studio-pc:7380")]);
    assert_eq!(check(&lan(), &h), Err(ApiError::Unauthorized));
}

#[test]
fn a_local_server_with_a_token_also_checks_host_and_token() {
    let mut c = local();
    c.token = TOKEN.into();
    let h = headers(&[("host", "127.0.0.1:7380")]);
    assert_eq!(check(&c, &h), Err(ApiError::Unauthorized));
    let h = headers(&[("host", "evil.example"), ("authorization", &format!("Bearer {TOKEN}"))]);
    assert_eq!(check(&c, &h), Err(ApiError::ForbiddenOrigin));
}

fn app(config: HttpRemoteConfig) -> axum::Router {
    router(Ctx::new(FakeControl::new(demo_state()), Arc::new(config)))
}

#[tokio::test]
async fn the_router_enforces_the_guard() {
    let req = Request::get("/api/v1/state").header("host", "evil.example").body(Body::empty());
    let res = app(local()).oneshot(req.unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    let req = Request::get("/api/v1/state").header("host", "pc:7380").body(Body::empty());
    let res = app(lan()).oneshot(req.unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn an_oversized_body_is_413() {
    let mut c = local();
    c.max_body_bytes = 1024;
    let body = format!(r#"{{"fader":0.5,"pad":"{}"}}"#, "x".repeat(4096));
    let req = Request::put("/api/v1/players/1/volume")
        .header("host", "127.0.0.1:7380")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();
    let res = app(c).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn a_preflight_from_a_listed_origin_is_answered() {
    let mut c = local();
    c.cors_origins = vec!["https://studio.example".into()];
    let req = Request::options("/api/v1/players/1/volume")
        .header("host", "127.0.0.1:7380")
        .header("origin", "https://studio.example")
        .header("access-control-request-method", "PUT")
        .header("access-control-request-headers", "content-type")
        .body(Body::empty())
        .unwrap();
    let res = app(c).oneshot(req).await.unwrap();
    assert!(res.status().is_success());
    assert_eq!(
        res.headers().get("access-control-allow-origin").unwrap(),
        "https://studio.example"
    );
}
```

Run: `cargo test -p fp-remote --test guard`
Expected: compile error, `could not find guard`.

- [ ] **Step 2: Implement `crates/fp-remote/src/http/guard.rs`**

```rust
//! Who may call the API (remote control spec §6.2): a bearer token when one
//! is set, no foreign `Origin`, and on a loopback bind a loopback `Host`, so
//! a web page in the operator's browser cannot drive the local server.

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use axum::extract::{ConnectInfo, Request, State};
use axum::http::{HeaderMap, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use fp_model::HttpRemoteConfig;

use super::Ctx;
use crate::api::ApiError;

/// One log line per source and per this long, so an attack cannot flood the log.
const LOG_EVERY: Duration = Duration::from_secs(1);
/// Sources remembered for log throttling before old ones are forgotten.
const LOG_SOURCES: usize = 256;

pub fn check(config: &HttpRemoteConfig, headers: &HeaderMap) -> Result<(), ApiError> {
    if let Some(origin) = headers.get(header::ORIGIN) {
        let allowed = origin.to_str().is_ok_and(|o| {
            config.cors_origins.iter().any(|c| c == "*" || c == o)
        });
        if !allowed {
            return Err(ApiError::ForbiddenOrigin);
        }
    }
    if config.bind_addr().is_some_and(|ip| ip.is_loopback()) {
        let host = headers.get(header::HOST).and_then(|h| h.to_str().ok());
        if !host.is_some_and(host_is_loopback) {
            return Err(ApiError::ForbiddenOrigin);
        }
    }
    if !config.token.is_empty() {
        let given = headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));
        if !given.is_some_and(|t| same(t.as_bytes(), config.token.as_bytes())) {
            return Err(ApiError::Unauthorized);
        }
    }
    Ok(())
}

/// `localhost` or a loopback address, with or without a port.
fn host_is_loopback(host: &str) -> bool {
    let name = match host.strip_prefix('[') {
        Some(rest) => rest.split(']').next().unwrap_or_default(),
        None => host.rsplit_once(':').map_or(host, |(name, _)| name),
    };
    name.eq_ignore_ascii_case("localhost")
        || name.parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback())
}

/// Compares without stopping at the first difference.
fn same(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// Throttles the log lines of refused requests.
#[derive(Default)]
pub struct RejectLog {
    last: Mutex<HashMap<Option<IpAddr>, Instant>>,
}

impl RejectLog {
    fn note(&self, source: Option<IpAddr>, error: &ApiError) {
        let now = Instant::now();
        let mut last = self.last.lock().unwrap_or_else(PoisonError::into_inner);
        if last.len() >= LOG_SOURCES {
            last.retain(|_, at| now.duration_since(*at) < LOG_EVERY);
        }
        let due = last
            .get(&source)
            .is_none_or(|at| now.duration_since(*at) >= LOG_EVERY);
        if due {
            last.insert(source, now);
            tracing::warn!(?source, code = error.code(), "remote request refused");
        }
    }
}

pub async fn guard(State(ctx): State<Ctx>, req: Request, next: Next) -> Response {
    match check(&ctx.config, req.headers()) {
        Ok(()) => next.run(req).await,
        Err(error) => {
            let source = req
                .extensions()
                .get::<ConnectInfo<SocketAddr>>()
                .map(|c| c.0.ip());
            ctx.rejections.note(source, &error);
            error.into_response()
        }
    }
}
```

- [ ] **Step 3: Wire it into the router** (`crates/fp-remote/src/http/mod.rs`)

Add `pub mod guard;`. Give `Ctx` a `pub rejections: Arc<guard::RejectLog>`
field, initialised in `Ctx::new` with `Arc::default()`. Replace the end of
`router`:

```rust
    let config = ctx.config.clone();
    Router::new()
        .nest("/api/v1", api)
        .fallback(h::not_found)
        .layer(middleware::from_fn_with_state(ctx.clone(), guard::guard))
        .layer(DefaultBodyLimit::max(config.max_body_bytes as usize))
        .layer(cors(&config))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_millis(config.request_timeout_ms.into()),
        ))
        .layer(ConcurrencyLimitLayer::new(IN_FLIGHT_REQUESTS))
        .with_state(ctx)
```

with these additions to the module:

```rust
use std::time::Duration;

use axum::extract::DefaultBodyLimit;
use axum::http::{HeaderValue, Method, header};
use axum::middleware;
use tower::limit::ConcurrencyLimitLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::timeout::TimeoutLayer;

/// Requests served at once; more wait their turn. An engineering bound
/// that keeps a flood from growing memory, not an operator setting.
const IN_FLIGHT_REQUESTS: usize = 64;

/// How long a browser may cache a preflight answer.
const PREFLIGHT_MAX_AGE: Duration = Duration::from_secs(600);

fn cors(config: &HttpRemoteConfig) -> CorsLayer {
    let layer = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .max_age(PREFLIGHT_MAX_AGE);
    if config.cors_origins.iter().any(|o| o == "*") {
        layer.allow_origin(AllowOrigin::any())
    } else {
        layer.allow_origin(AllowOrigin::list(
            config
                .cors_origins
                .iter()
                .filter_map(|o| HeaderValue::from_str(o).ok()),
        ))
    }
}
```

The CORS layer is outside the guard, so a preflight (no token, as browsers
send it) is answered by CORS. The guard sees every real request.

- [ ] **Step 4: Run all fp-remote tests**

Run: `cargo test -p fp-remote`
Expected: every test passes, including Task 4's (they send a loopback
`Host`).

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace \
 && git add crates/fp-remote && git commit -m "feat(remote): token, origin and host checks with CORS and limits

A page open in the operator's browser can reach 127.0.0.1, so foreign
origins and non-loopback Host headers are refused even without a token.
Refusals log once per source per second.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: the server thread

**Files:**
- Create: `crates/fp-remote/src/server.rs`
- Modify: `crates/fp-remote/src/lib.rs` (`pub mod server;` and
  `pub use server::{spawn, RemoteHandle, RemoteStatus, ServerStatus, ServerError};`)
- Test: `crates/fp-remote/tests/server.rs`

**Interfaces:**
- Consumes: `http::{router, Ctx}`, `RemoteControl`.
- Produces:
  - `fp_remote::spawn(control: Arc<dyn RemoteControl>) -> std::io::Result<RemoteHandle>`;
  - `RemoteHandle::status(&self) -> RemoteStatus`; dropping the handle stops
    the thread;
  - `RemoteStatus { pub http: ServerStatus }`;
  - `ServerStatus::{Off, Listening(SocketAddr), Error(ServerError)}`;
  - `ServerError::{InvalidBind, TokenRequired, Bind(String), Runtime(String)}`.
    Plan 3 localises them in Settings.

- [ ] **Step 1: Write the failing tests** (`crates/fp-remote/tests/server.rs`)

```rust
#![allow(clippy::unwrap_used)]
mod support;

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::{Duration, Instant};

use fp_remote::{RemoteHandle, ServerError, ServerStatus, spawn};
use support::{FakeControl, demo_state};

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

fn wait_for(handle: &RemoteHandle, what: &str, ok: impl Fn(&ServerStatus) -> bool) -> ServerStatus {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let s = handle.status().http;
        if ok(&s) {
            return s;
        }
        assert!(Instant::now() < deadline, "waiting for {what}; last {s:?}");
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn get(addr: SocketAddr, path: &str) -> u16 {
    let mut s = TcpStream::connect_timeout(&addr, Duration::from_secs(2)).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    write!(s, "GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n").unwrap();
    let mut out = String::new();
    s.read_to_string(&mut out).unwrap();
    out.split(' ').nth(1).unwrap().parse().unwrap()
}

#[test]
fn disabled_by_default_and_nothing_listens() {
    let fake = FakeControl::new(demo_state());
    let handle = spawn(fake).unwrap();
    std::thread::sleep(Duration::from_millis(300));
    assert_eq!(handle.status().http, ServerStatus::Off);
}

#[test]
fn enabling_it_at_run_time_starts_listening_and_answers() {
    let fake = FakeControl::new(demo_state());
    let handle = spawn(fake.clone()).unwrap();
    let port = free_port();
    fake.edit(|s| {
        s.config.remote.http.enabled = true;
        s.config.remote.http.port = port;
    });
    let s = wait_for(&handle, "listening", |s| matches!(s, ServerStatus::Listening(_)));
    let ServerStatus::Listening(addr) = s else { unreachable!() };
    assert_eq!(addr.port(), port);
    assert_eq!(get(addr, "/api/v1/state"), 200);
    assert!(fake.take_sent().is_empty(), "rule 10: starting sends nothing");
}

#[test]
fn a_port_change_moves_the_server_and_disabling_closes_it() {
    let fake = FakeControl::new(demo_state());
    let first = free_port();
    fake.edit(|s| {
        s.config.remote.http.enabled = true;
        s.config.remote.http.port = first;
    });
    let handle = spawn(fake.clone()).unwrap();
    wait_for(&handle, "first port", |s| matches!(s, ServerStatus::Listening(a) if a.port() == first));
    let second = free_port();
    fake.edit(|s| s.config.remote.http.port = second);
    wait_for(&handle, "second port", |s| matches!(s, ServerStatus::Listening(a) if a.port() == second));
    assert!(TcpStream::connect(("127.0.0.1", first)).is_err(), "old port still open");
    fake.edit(|s| s.config.remote.http.enabled = false);
    wait_for(&handle, "off", |s| *s == ServerStatus::Off);
    assert!(TcpStream::connect(("127.0.0.1", second)).is_err());
}

#[test]
fn a_port_in_use_is_an_error_status_not_a_crash() {
    let taken = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = taken.local_addr().unwrap().port();
    let fake = FakeControl::new(demo_state());
    fake.edit(|s| {
        s.config.remote.http.enabled = true;
        s.config.remote.http.port = port;
    });
    let handle = spawn(fake).unwrap();
    let s = wait_for(&handle, "error", |s| matches!(s, ServerStatus::Error(_)));
    assert!(matches!(s, ServerStatus::Error(ServerError::Bind(_))), "{s:?}");
}

#[test]
fn beyond_loopback_without_a_token_it_refuses_to_start() {
    let fake = FakeControl::new(demo_state());
    fake.edit(|s| {
        s.config.remote.http.enabled = true;
        s.config.remote.http.bind = "0.0.0.0".into();
        s.config.remote.http.port = free_port();
    });
    let handle = spawn(fake).unwrap();
    let s = wait_for(&handle, "error", |s| matches!(s, ServerStatus::Error(_)));
    assert_eq!(s, ServerStatus::Error(ServerError::TokenRequired));
}

#[test]
fn dropping_the_handle_stops_the_server() {
    let fake = FakeControl::new(demo_state());
    let port = free_port();
    fake.edit(|s| {
        s.config.remote.http.enabled = true;
        s.config.remote.http.port = port;
    });
    let handle = spawn(fake).unwrap();
    wait_for(&handle, "listening", |s| matches!(s, ServerStatus::Listening(_)));
    let started = Instant::now();
    drop(handle);
    assert!(started.elapsed() < Duration::from_secs(5));
    assert!(TcpStream::connect(("127.0.0.1", port)).is_err());
}
```

Run: `cargo test -p fp-remote --test server`
Expected: compile error, `could not find spawn`.

- [ ] **Step 2: Implement `crates/fp-remote/src/server.rs`**

```rust
//! The remote thread (remote control spec §7): a tokio `current_thread`
//! runtime that follows `config.remote` in the model snapshot, (re)starts
//! the HTTP server when it changes, and publishes its status. It never
//! sends a command by itself (rule 10).

use std::net::SocketAddr;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use arc_swap::ArcSwap;
use fp_model::HttpRemoteConfig;
use tokio::net::TcpListener;
use tokio::sync::{oneshot, watch};

use crate::control::RemoteControl;
use crate::http::{Ctx, router};

/// How often the configuration in the snapshot is looked at.
const CONFIG_POLL: Duration = Duration::from_millis(250);
/// How long open requests may take to finish when the server stops.
const SHUTDOWN_GRACE: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerError {
    /// `bind` is not an IP address.
    InvalidBind,
    /// Listening beyond loopback needs a token.
    TokenRequired,
    /// The address could not be bound (in use, no permission).
    Bind(String),
    /// The runtime could not start.
    Runtime(String),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ServerStatus {
    #[default]
    Off,
    Listening(SocketAddr),
    Error(ServerError),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RemoteStatus {
    pub http: ServerStatus,
}

/// Owns the remote thread; dropping it stops the servers and joins.
pub struct RemoteHandle {
    status: Arc<ArcSwap<RemoteStatus>>,
    stop: watch::Sender<bool>,
    thread: Option<JoinHandle<()>>,
}

impl RemoteHandle {
    pub fn status(&self) -> RemoteStatus {
        (**self.status.load()).clone()
    }
}

impl Drop for RemoteHandle {
    fn drop(&mut self) {
        let _ = self.stop.send(true);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Starts the remote thread. It listens only once `config.remote` asks for it.
pub fn spawn(control: Arc<dyn RemoteControl>) -> std::io::Result<RemoteHandle> {
    let status = Arc::new(ArcSwap::from_pointee(RemoteStatus::default()));
    let (stop, stopped) = watch::channel(false);
    let shared = status.clone();
    let thread = std::thread::Builder::new()
        .name("fp-remote".to_owned())
        .spawn(move || run(control, shared, stopped))?;
    Ok(RemoteHandle {
        status,
        stop,
        thread: Some(thread),
    })
}

fn run(control: Arc<dyn RemoteControl>, status: Arc<ArcSwap<RemoteStatus>>, stop: watch::Receiver<bool>) {
    match tokio::runtime::Builder::new_current_thread().enable_all().build() {
        Ok(rt) => rt.block_on(supervise(control, status, stop)),
        Err(e) => {
            tracing::error!(error = %e, "remote control could not start");
            status.store(Arc::new(RemoteStatus {
                http: ServerStatus::Error(ServerError::Runtime(e.to_string())),
            }));
        }
    }
}

/// A server that is running, and how to stop it.
struct Running {
    shutdown: oneshot::Sender<()>,
    task: tokio::task::JoinHandle<std::io::Result<()>>,
}

impl Running {
    async fn stop(self) {
        let _ = self.shutdown.send(());
        let _ = tokio::time::timeout(SHUTDOWN_GRACE, self.task).await;
    }
}

async fn supervise(
    control: Arc<dyn RemoteControl>,
    status: Arc<ArcSwap<RemoteStatus>>,
    mut stop: watch::Receiver<bool>,
) {
    let mut applied: Option<HttpRemoteConfig> = None;
    let mut running: Option<Running> = None;
    loop {
        let wanted = control.model().config.remote.http.clone();
        if applied.as_ref() != Some(&wanted) {
            if let Some(server) = running.take() {
                server.stop().await;
            }
            let (http, server) = start(&control, &wanted).await;
            status.store(Arc::new(RemoteStatus { http }));
            running = server;
            applied = Some(wanted);
        }
        tokio::select! {
            _ = stop.changed() => break,
            () = tokio::time::sleep(CONFIG_POLL) => {}
        }
    }
    if let Some(server) = running.take() {
        server.stop().await;
    }
    status.store(Arc::new(RemoteStatus::default()));
}

async fn start(
    control: &Arc<dyn RemoteControl>,
    config: &HttpRemoteConfig,
) -> (ServerStatus, Option<Running>) {
    if !config.enabled {
        return (ServerStatus::Off, None);
    }
    let Some(ip) = config.bind_addr() else {
        return (ServerStatus::Error(ServerError::InvalidBind), None);
    };
    if !ip.is_loopback() && config.token.is_empty() {
        tracing::error!(bind = %ip, "remote HTTP not started: a token is required beyond this computer");
        return (ServerStatus::Error(ServerError::TokenRequired), None);
    }
    let listener = match TcpListener::bind((ip, config.port)).await {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!(bind = %ip, port = config.port, error = %e, "remote HTTP could not listen");
            return (ServerStatus::Error(ServerError::Bind(e.to_string())), None);
        }
    };
    let addr = listener
        .local_addr()
        .unwrap_or_else(|_| SocketAddr::new(ip, config.port));
    let app = router(Ctx::new(control.clone(), Arc::new(config.clone())));
    let (shutdown, signal) = oneshot::channel::<()>();
    let task = tokio::spawn(async move {
        axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
            .with_graceful_shutdown(async {
                let _ = signal.await;
            })
            .await
    });
    tracing::info!(%addr, "remote HTTP listening");
    (ServerStatus::Listening(addr), Some(Running { shutdown, task }))
}
```

- [ ] **Step 3: Run the tests**

Run: `cargo test -p fp-remote --test server`
Expected: 6 passed. The tests wait for the remote thread with a deadline.
They never sleep to wait for audio.

- [ ] **Step 4: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace \
 && git add crates/fp-remote && git commit -m "feat(remote): server thread that follows the configuration

The remote thread watches config.remote in the model snapshot and
restarts the HTTP server when it changes, so Settings needs no extra
wiring; a busy port or a missing token is a status, never a crash.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 7: the application bridge

**Files:**
- Create: `crates/fp-app/src/remote.rs`
- Modify: `crates/fp-app/src/lib.rs` (`pub mod remote;`)
- Modify: `crates/fp-app/Cargo.toml` (`fp-remote.workspace = true`)
- Modify: `crates/fp-app/src/main.rs` (start it, drop it before services shutdown)
- Test: `crates/fp-app/tests/remote.rs`

**Interfaces:**
- Consumes:
  - `fp_remote::{spawn, RemoteHandle, control::*}`;
  - `ConductorHandle` (`model`, `telemetry`, `send`);
  - `MediaCache::get`;
  - `AnalysisCache::{new, load}`.
- Produces: `fp_app::remote::start(conductor: Arc<ConductorHandle>, media: MediaCache, analysis_dir: PathBuf, limits: &Limits) -> Option<RemoteHandle>`.

- [ ] **Step 1: Write the failing integration test** (`crates/fp-app/tests/remote.rs`)

```rust
//! The remote API drives the real conductor (Offline backend).
#![allow(clippy::unwrap_used)]

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::time::{Duration, Instant};

use fp_app::services::MediaCache;
use fp_backends::{AudioBackend, OfflineBackend};
use fp_engine::conductor::Conductor;
use fp_engine::engine::{Engine, EngineSettings};
use fp_engine::worker::file_opener;
use fp_model::{Command, Transport};
use fp_remote::ServerStatus;
use fp_store::{AppPaths, Store};

fn tone(path: &std::path::Path) {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 48_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(path, spec).unwrap();
    for n in 0..48_000 * 3 {
        let v = ((n as f32 * 440.0 * std::f32::consts::TAU / 48_000.0).sin() * 8000.0) as i16;
        w.write_sample(v).unwrap();
        w.write_sample(v).unwrap();
    }
    w.finalize().unwrap();
}

fn post(addr: SocketAddr, path: &str) -> (u16, String) {
    let mut s = TcpStream::connect_timeout(&addr, Duration::from_secs(2)).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    write!(
        s,
        "POST {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\nContent-Length: 0\r\n\r\n"
    )
    .unwrap();
    let mut out = String::new();
    s.read_to_string(&mut out).unwrap();
    let status = out.split(' ').nth(1).unwrap().parse().unwrap();
    (status, out.split_once("\r\n\r\n").unwrap().1.to_owned())
}

#[test]
fn a_remote_play_puts_the_player_on_air() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("tone.wav");
    tone(&file);
    let paths = AppPaths::under(dir.path());
    let store = Store::new(paths.clone(), Default::default());
    let mut loaded = store.load("Main");
    let port = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let state = &mut loaded.state;
    state.config.outputs.backend = Some("offline".into());
    state.config.outputs.buffer_frames = 480;
    state.config.remote.http.enabled = true;
    state.config.remote.http.port = port;
    let playlist = state.playlists.first_id().unwrap();
    fp_model::apply(state, Command::InsertPaths { playlist, index: 0, paths: vec![file] })
        .unwrap();
    let player = state.players[0].id;
    let limits = state.config.limits.clone();

    let backend = OfflineBackend::new();
    let _device = backend.add_device("main", 2);
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let engine = Engine::new(backends, EngineSettings::from_config(&loaded.state.config), file_opener());
    let (mut conductor, handle) =
        Conductor::new(loaded.state, loaded.actions, engine, Instant::now());
    let handle = Arc::new(handle);
    let remote = fp_app::remote::start(
        handle.clone(),
        MediaCache::default(),
        paths.cache_dir.join("analysis"),
        &limits,
    )
    .unwrap();

    let deadline = Instant::now() + Duration::from_secs(5);
    let addr = loop {
        if let ServerStatus::Listening(addr) = remote.status().http {
            break addr;
        }
        assert!(Instant::now() < deadline, "{:?}", remote.status());
        std::thread::sleep(Duration::from_millis(10));
    };
    let (status, body) = post(addr, &format!("/api/v1/players/{}/play", player.0));
    assert_eq!(status, 202, "{body}");

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        conductor.tick(Instant::now());
        if handle.model.load().players[0].transport == Transport::Playing {
            break;
        }
        assert!(Instant::now() < deadline, "the player never started");
        std::thread::sleep(Duration::from_millis(5));
    }
    drop(remote);
}
```

Add to `crates/fp-app/Cargo.toml` `[dependencies]`: `fp-remote.workspace = true`.

Run: `cargo test -p fp-app --test remote`
Expected: compile error, `could not find remote in fp_app`.

- [ ] **Step 2: Implement `crates/fp-app/src/remote.rs`**

```rust
//! Connects the remote server (`fp-remote`) to the conductor and to the
//! caches that hold covers and waveforms.

use std::path::PathBuf;
use std::sync::Arc;

use fp_analysis::cache::AnalysisCache;
use fp_engine::conductor::ConductorHandle;
use fp_model::{AppState, Command, Limits, TrackId};
use fp_remote::RemoteHandle;
use fp_remote::control::{Playback, RemoteControl, WaveformData};

use crate::services::MediaCache;

struct Bridge {
    conductor: Arc<ConductorHandle>,
    media: MediaCache,
    cache: AnalysisCache,
}

impl Bridge {
    /// The cached analysis of `track`, read from disk (blocking).
    fn analysis(&self, track: TrackId) -> Option<fp_analysis::Analysis> {
        let model = self.conductor.model.load();
        let path = &model.library.get(track)?.path;
        self.cache.load(path, &model.config.analysis)
    }
}

impl RemoteControl for Bridge {
    fn model(&self) -> Arc<AppState> {
        self.conductor.model.load_full()
    }

    fn playback(&self) -> Playback {
        let t = self.conductor.telemetry.load();
        Playback {
            revision: t.model_version,
            players: t
                .players
                .iter()
                .filter_map(|(id, p)| p.position_secs.map(|s| (*id, s)))
                .collect(),
            carts: t.carts.iter().map(|(id, c)| (*id, c.position_secs)).collect(),
        }
    }

    fn send(&self, command: Command) -> bool {
        self.conductor.send(command)
    }

    fn cover(&self, track: TrackId) -> Option<Vec<u8>> {
        match self.media.get(track) {
            Some(media) => media.cover_png.as_deref().map(<[u8]>::to_vec),
            None => self.analysis(track)?.cover_png,
        }
    }

    fn peaks(&self, track: TrackId) -> Option<WaveformData> {
        let (peaks, bucket_secs) = match self.media.get(track) {
            Some(media) => (media.peaks.clone(), media.peak_bucket_secs),
            None => {
                let a = self.analysis(track)?;
                (a.peaks, a.peak_bucket_secs)
            }
        };
        Some(WaveformData {
            bucket_secs,
            peaks: peaks.iter().map(|p| [p.min, p.max, p.rms]).collect(),
        })
    }
}

/// Starts the remote thread. It listens only when `config.remote` asks for
/// it. A failure is logged and the application runs without it.
pub fn start(
    conductor: Arc<ConductorHandle>,
    media: MediaCache,
    analysis_dir: PathBuf,
    limits: &Limits,
) -> Option<RemoteHandle> {
    let bridge = Bridge {
        conductor,
        media,
        cache: AnalysisCache::new(analysis_dir, limits),
    };
    match fp_remote::spawn(Arc::new(bridge)) {
        Ok(handle) => Some(handle),
        Err(error) => {
            tracing::warn!(%error, "remote control could not start");
            None
        }
    }
}
```

`fp_analysis::Analysis` is re-exported by `fp-analysis` (`pub use analyze::Analysis`).

In `crates/fp-app/src/lib.rs` add `pub mod remote;` next to `pub mod midi;`.

- [ ] **Step 3: Start it in `main.rs`**

In `run`, before `let mut app = AppUi::new(handle.clone(), i18n, media)`,
add:

```rust
    let remote = fp_app::remote::start(
        handle.clone(),
        media.clone(),
        paths.cache_dir.join("analysis"),
        &config.limits,
    );
```

and before `services.shutdown();` add `drop(remote);`, so the remote thread
stops before the final save and the conductor shutdown.

- [ ] **Step 4: Run the integration test**

Run: `cargo test -p fp-app --test remote`
Expected: PASS.

- [ ] **Step 5: Run the app by hand**

```bash
mkdir -p /tmp/fp-remote && FAUSTE_HOME=/tmp/fp-remote cargo run -p fp-app --example demo_session -- test-music
```

Add `"remote": {"http": {"enabled": true}}` to
`/tmp/fp-remote/config/config.json`. The path is printed by `bootstrap`;
check `docs/user/data-and-backups.md`. Then:

```bash
FAUSTE_HOME=/tmp/fp-remote cargo run -p fp-app &
curl -s http://127.0.0.1:7380/api/v1/players | head -c 400; echo
P=$(curl -s http://127.0.0.1:7380/api/v1/players | python3 -c 'import json,sys; print(json.load(sys.stdin)[0]["id"])')
curl -si -X POST http://127.0.0.1:7380/api/v1/players/$P/play | head -1
```

Expected: JSON players, then `HTTP/1.1 202 Accepted`, and the first player
goes on air. If `test-music/` is empty, use any folder with audio files.

- [ ] **Step 6: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace \
 && git add crates/fp-app Cargo.lock && git commit -m "feat(app): start the remote control server

The bridge serves covers and waveforms from the UI's media cache, or
from the analysis cache on disk, and reports positions from the
conductor's telemetry.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 8: documentation

**Files:**
- Create: `docs/technical/remote-api.md`, `docs/user/remote-control.md`
- Modify:
  - `docs/technical/architecture.md` (crate table, graph);
  - `docs/technical/threading-and-realtime.md` (thread table);
  - `docs/technical/README.md` (index);
  - `docs/user/README.md` (index);
  - `README.md` (feature bullet);
  - `docs/superpowers/specs/2026-09-25-fauste-player-design.md` (non-goal);
  - `CLAUDE.md` (crate table, testing notes).

- [ ] **Step 1: `docs/technical/remote-api.md`**

Write the reference for what plan 1 delivers, in present tense:
- **Overview.** Base path `/api/v1`, JSON, ids, desired values, `202` with
  `revision`, and the dry run.
- **Resources:** the DTO fields of spec §3.1, as tables.
- **Routes:** the tables of spec §3.2 and §3.3, without `GET /events`, which
  says "plan 2" in the spec.
- **Errors:** the table of spec §3.5, including 408.
- **Security:** spec §6.2.
- **Configuration:** the `remote.http` and `remote.events` rows of spec §6.1.
- **Implementation:**
  - `fp-remote` modules (`control`, `dto`, `api`, `http`, `server`);
  - the `RemoteControl` trait;
  - the `fp-remote` thread;
  - the 250 ms configuration poll;
  - `spawn_blocking` for covers and peaks.
- **Examples:**

```sh
curl -s http://127.0.0.1:7380/api/v1/state
curl -s -X POST http://127.0.0.1:7380/api/v1/players/12/play
curl -s -X PUT -H 'Content-Type: application/json' \
     -d '{"fader": 0.8}' http://127.0.0.1:7380/api/v1/players/12/volume
curl -s -H "Authorization: Bearer $TOKEN" http://studio-pc:7380/api/v1/players
```

- [ ] **Step 2: `docs/user/remote-control.md`**

```markdown
# Remote control

Fauste Player can be read and operated over the network through an HTTP API,
for example from a web page, a phone app or a station's automation. It is
**off** until you turn it on, and at first it only answers on this computer.

## Turning it on

Settings has no page for it yet. Edit `config.json` in the data folder (see
[Data and backups](data-and-backups.md)) while Fauste Player is closed:

    "remote": {
      "http": { "enabled": true }
    }

It listens on `http://127.0.0.1:7380`. Changes made while the application
runs (for example from a future Settings page) apply within a quarter of a
second, with no restart.

## Listening on the studio network

To reach it from other computers, set `bind` to `0.0.0.0` (or one of this
computer's addresses) and set a **token** of at least 16 characters. Without
a token the server refuses to start and the log says why.

    "remote": {
      "http": {
        "enabled": true,
        "bind": "0.0.0.0",
        "port": 7380,
        "token": "a-long-random-secret-of-your-own"
      }
    }

Clients send it as `Authorization: Bearer <token>`. The API has no
encryption: keep it on a trusted studio network, or put it behind a reverse
proxy with HTTPS.

## Web pages

A web page served from another address can use the API only if its origin
(for example `https://studio.example`) is listed in `cors_origins`. Requests
from other pages are refused, even on this computer.

## What a client can do

Read the players, playlists, tracks (with cover and waveform) and the
cartwall; play, pause, stop, fade, restart and go back; choose the next
entry; pre-listen; seek; set volumes, modes and stop-after-current; fire,
stop and pre-listen carts. A button that is greyed out on screen is refused
remotely too. The full reference is in
[the technical documentation](../technical/remote-api.md).

## All settings

| Setting | Default | Meaning |
|---|---|---|
| `remote.http.enabled` | `false` | Turn the API on |
| `remote.http.bind` | `127.0.0.1` | Address to listen on (an IP address) |
| `remote.http.port` | `7380` | Port (1024–65535) |
| `remote.http.token` | empty | Required beyond this computer; at least 16 characters |
| `remote.http.cors_origins` | none | Web origins allowed to call the API |
| `remote.http.request_timeout_ms` | `10000` | Longest a request may take |
| `remote.http.max_body_bytes` | `65536` | Largest request body |
| `remote.http.max_event_clients` | `16` | Live event streams at once |
| `remote.events.position_interval_ms` | `250` | How often times are published while playing |
```

- [ ] **Step 3: Indexes and the other docs**

- `docs/user/README.md`: after the MIDI row, add
  `| [Remote control](remote-control.md) | Operating Fauste Player over the network: HTTP API, security |`.
- `docs/technical/README.md`: add `remote-api.md` to its list, in the
  same style as the other entries.
- `docs/technical/architecture.md`:
  - after the `fp-control` row, add
    `| \`fp-remote\` | Remote control over the network: the HTTP/JSON API (axum on a tokio current-thread runtime), its security guard and the remote thread; reaches the application through the \`RemoteControl\` trait | \`fp-model\` |`;
  - in the graph, add `   ├──► fp-remote ─────► fp-model` (and, while
    there, the missing `fp-control` line `   ├──► fp-control ────► fp-model`).
- `docs/technical/threading-and-realtime.md`: after the Services row, add
  `| Remote | \`fp-remote\` | normal | the tokio runtime of the HTTP API; follows \`config.remote\` every 250 ms | network I/O (async); covers and peaks in \`spawn_blocking\` |`.
- `README.md`: after the MIDI bullet, add:

```markdown
- **Remote control API:** an HTTP/JSON API, off by default, lets a web page,
  a phone app or automation read the players, playlists and cartwall and
  operate them, with a token and origin checks when it listens on the
  network.
```

- Main spec, Non-goals: replace the line with
  `- Network features, streaming, telemetry. (Local MIDI control surfaces are in scope: feedback spec §6. Remote control over the network is in scope: remote control spec.)`
- `CLAUDE.md`:
  - after the `fp-control` row, add
    `| \`crates/fp-remote\` | Remote control over the network: HTTP/JSON API, security guard, the remote thread |`;
  - add to "Testing notes":

```markdown
- Remote API tests drive the axum router with `tower::ServiceExt::oneshot`
  (no sockets) and the recording `FakeControl` in
  `crates/fp-remote/tests/support`; server tests bind `127.0.0.1` on free
  ports.
- Driving the running app for a screenshot: enable `remote.http` in the
  scratch `config.json` and use `curl` (see `docs/user/remote-control.md`),
  or click with `xdotool` (`xdotool search --name "Fauste Player"`, then
  `xdotool mousemove --window <id> <x> <y> click 1`). `xdotool` is needed
  for clicks; install it with the system's package manager.
```

- [ ] **Step 4: Check the links**

Run: `grep -rn "remote-control.md\|remote-api.md" docs README.md CLAUDE.md`
Expected: every link points to a file that exists.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace \
 && git add docs README.md CLAUDE.md && git commit -m "docs: describe the remote control API

User guide page, technical reference, architecture and threads, and the
main spec's non-goal now that network remote control is in scope.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

## After the last task

- Run `superpowers:requesting-code-review` on the branch with a fresh reviewer
  on the most capable model. Fix Critical and Important findings test-first,
  and record Minor ones in the ledger.
- Push `feat/remote-control` and open the PR (title
  `feat(remote): HTTP API to read and operate the player`, following the
  template). Wait for CI on all three OSes, then merge. Plans 2 and 3
  continue from an updated `master` on a new branch.
