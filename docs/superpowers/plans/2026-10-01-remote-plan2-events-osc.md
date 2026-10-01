# Remote Plan 2 — Events (SSE) and OSC Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** remote clients learn about changes by push: Server-Sent Events on
HTTP, and subscriptions on OSC. OSC can also operate players and carts.

**Architecture:**
- `fp-model/src/remote.rs` gains `config.remote.osc` and a `Cidr`
  allow-list type.
- `fp-remote` gains:
  - `events.rs` (pure): the snapshot diff that produces resource events,
    and the position event;
  - `throttle.rs`: the per-source log throttle, moved out of the guard and
    shared with OSC;
  - `http/events.rs`: `GET /events`, which sends a full `state`, then the
    events, a `resync` on lag, and a keep-alive. It honours a client limit,
    `?topics=` and `?token=`;
  - `osc.rs` (pure):
    - address parsing into `Operation`s or subscriptions;
    - the value table of spec §4.2;
    - a `Subscribers` table that tracks what each subscriber was last sent;
  - `osc_server.rs`: the UDP loop (allow-list, decode, plan, send; push
    changes to subscribers);
  - `server.rs`, extended:
    - a publisher task (every 50 ms) that diffs snapshots into a
      `broadcast` channel and emits `position` every
      `position_interval_ms` while something plays;
    - HTTP and OSC servers that each follow their configuration;
    - `RemoteStatus { http, osc }`;
    - when a server stops, its streams are ended and a task still running
      after the grace period is aborted.

**Tech Stack:** axum 0.8 SSE, tokio broadcast/watch, futures-util 0.3 `stream::unfold`, rosc 0.11.4 (MIT/Apache).

**Spec:** [`docs/superpowers/specs/2026-10-01-remote-control-design.md`](../specs/2026-10-01-remote-control-design.md) §4, §5, §6.1 (OSC), §6.2 (`?token=`), §6.3, §10 plan 2.

## Global Constraints

- Branch `feat/remote-events-osc`. Before every commit:
  `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.
- Same rules as plan 1: no `unsafe`, no `unwrap`/`expect`/`panic` outside
  tests, `indexing_slicing` denied in `fp-remote`, the remote thread never
  touches audio or the UI, rule 10 (nothing goes on air by itself).
- Event names: `state`, `player`, `playlist`, `playlist-removed`,
  `cartwall`, `track`, `position`, `resync`. Every SSE event has
  `id: <revision>`.
- `config.remote.osc` defaults: `enabled = false`, `bind = "127.0.0.1"`,
  `port = 7381`, `allowed_sources = ["127.0.0.1/32", "::1/128"]`,
  `max_subscribers = 16` (1–256), `subscription_ttl_secs = 60` (5–3600).
- OSC numbering is 1-based (players in display order, carts of the page
  shown, pages in order). A button acts with no argument or a number > 0.
- OSC packets from sources outside `allowed_sources` are dropped. IPv4
  addresses mapped into IPv6 are compared as IPv4.
- Every refusal logs at most one line per source per second.

## Review Focus

- **A slow or vanished SSE client:** a slow client gets `resync` and the
  publisher never waits. A disconnect frees its client slot (Task 3 tests).
- **Reconfiguring or quitting while event streams are open:** the old
  server ends its streams and stops. Quitting does not hang (Task 3 and
  Task 5 tests).
- **Hostile OSC input:** a malformed packet, an unknown address,
  `/fauste/player/0/play`, `/fauste/player/99/play`, a string where a
  number is expected, deeply nested bundles. Each is dropped with a log
  line and never panics (Task 4 tests).
- **OSC from a source that is not allowed,** including `::ffff:127.0.0.1`
  on a dual-stack socket (Task 1 and Task 5 tests).
- **The player count or the page shown changes** while OSC clients are
  subscribed: they get a full dump (Task 4 tests).

---

### Task 1: `config.remote.osc` and source allow-lists

**Files:**
- Modify: `crates/fp-model/src/remote.rs`, `crates/fp-model/src/lib.rs`
- Test: `crates/fp-model/src/remote.rs` tests

**Interfaces:**
- Produces:
  - `fp_model::OscRemoteConfig { enabled, bind: String, port: u16, allowed_sources: Vec<String>, max_subscribers: u32, subscription_ttl_secs: u32 }`;
  - `OscRemoteConfig::bind_addr() -> Option<IpAddr>` and
    `OscRemoteConfig::allows(IpAddr) -> bool`;
  - `fp_model::remote::Cidr::{parse(&str) -> Option<Cidr>, contains(IpAddr) -> bool}`;
  - `RemoteConfig.osc`.

- [ ] **Step 1: Failing tests** (append to the `tests` module)

```rust
    use super::Cidr;
    use std::net::IpAddr;

    fn ip(s: &str) -> IpAddr {
        s.parse().unwrap()
    }

    #[test]
    fn osc_is_off_and_local_by_default() {
        let c = Config::default();
        assert!(!c.remote.osc.enabled);
        assert_eq!(c.remote.osc.bind, "127.0.0.1");
        assert_eq!(c.remote.osc.port, 7381);
        assert_eq!(c.remote.osc.allowed_sources, vec!["127.0.0.1/32", "::1/128"]);
        assert_eq!(c.remote.osc.max_subscribers, 16);
        assert_eq!(c.remote.osc.subscription_ttl_secs, 60);
        assert!(c.remote.osc.allows(ip("127.0.0.1")));
        assert!(c.remote.osc.allows(ip("::1")));
        assert!(!c.remote.osc.allows(ip("192.168.1.20")));
    }

    #[test]
    fn subnets_match_by_prefix() {
        let net = Cidr::parse("192.168.1.0/24").unwrap();
        assert!(net.contains(ip("192.168.1.200")));
        assert!(!net.contains(ip("192.168.2.1")));
        assert!(Cidr::parse("10.1.2.3").unwrap().contains(ip("10.1.2.3")));
        assert!(Cidr::parse("0.0.0.0/0").unwrap().contains(ip("8.8.8.8")));
        assert!(Cidr::parse("fd00::/8").unwrap().contains(ip("fd12::1")));
        assert!(!Cidr::parse("fd00::/8").unwrap().contains(ip("10.0.0.1")));
        for bad in ["", "x", "10.0.0.0/33", "::/129", "10.0.0.0/-1", "10.0.0.0/"] {
            assert!(Cidr::parse(bad).is_none(), "{bad:?}");
        }
    }

    #[test]
    fn ipv4_mapped_sources_count_as_ipv4() {
        let c = Config::default();
        assert!(c.remote.osc.allows(ip("::ffff:127.0.0.1")));
        assert!(!c.remote.osc.allows(ip("::ffff:192.168.1.1")));
    }

    #[test]
    fn osc_values_are_brought_into_range() {
        let mut c = Config::default();
        c.remote.osc.bind = "nowhere".into();
        c.remote.osc.port = 10;
        c.remote.osc.allowed_sources = vec!["10.0.0.0/8".into(), "bogus".into()];
        c.remote.osc.max_subscribers = 0;
        c.remote.osc.subscription_ttl_secs = 1;
        let w = c.validate();
        assert_eq!(c.remote.osc.bind, "127.0.0.1");
        assert_eq!(c.remote.osc.port, 1024);
        assert_eq!(c.remote.osc.allowed_sources, vec!["10.0.0.0/8"]);
        assert_eq!(c.remote.osc.max_subscribers, 1);
        assert_eq!(c.remote.osc.subscription_ttl_secs, 5);
        assert_eq!(w.len(), 5);
    }

    #[test]
    fn osc_moves_off_the_http_port_on_the_same_bind() {
        let mut c = Config::default();
        c.remote.osc.port = c.remote.http.port;
        let w = c.validate();
        assert_eq!(c.remote.osc.port, 7381);
        assert_eq!(w[0].field, "remote.osc.port");
        c.remote.osc.bind = "0.0.0.0".into();
        c.remote.osc.port = c.remote.http.port;
        assert!(c.validate().is_empty());
    }
```

Run: `cargo test -p fp-model --lib remote::`
Expected: compile errors (`no field osc`, `Cidr` not found).

- [ ] **Step 2: Implement** (in `remote.rs`, below `RemoteEventsConfig`)

```rust
/// OSC over UDP (remote control spec §4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct OscRemoteConfig {
    pub enabled: bool,
    pub bind: String,
    pub port: u16,
    /// IP addresses or CIDR subnets whose packets are accepted.
    pub allowed_sources: Vec<String>,
    pub max_subscribers: u32,
    /// A subscription not renewed within this time ends.
    pub subscription_ttl_secs: u32,
}

impl Default for OscRemoteConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bind: "127.0.0.1".to_owned(),
            port: 7381,
            allowed_sources: vec!["127.0.0.1/32".to_owned(), "::1/128".to_owned()],
            max_subscribers: 16,
            subscription_ttl_secs: 60,
        }
    }
}

impl OscRemoteConfig {
    pub fn bind_addr(&self) -> Option<IpAddr> {
        self.bind.parse().ok()
    }

    /// Whether a packet from `source` is accepted.
    pub fn allows(&self, source: IpAddr) -> bool {
        self.allowed_sources
            .iter()
            .filter_map(|s| Cidr::parse(s))
            .any(|net| net.contains(source))
    }
}

/// An address with a prefix length: `10.0.0.0/8`, `::1/128`, or a bare
/// address (all of it).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cidr {
    addr: IpAddr,
    prefix: u8,
}

/// IPv4 seen through a dual-stack socket arrives as `::ffff:a.b.c.d`.
fn canonical(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(v6) => v6.to_ipv4_mapped().map_or(ip, IpAddr::V4),
        v4 => v4,
    }
}

impl Cidr {
    pub fn parse(text: &str) -> Option<Self> {
        let (addr, prefix) = match text.split_once('/') {
            Some((a, p)) => (a, Some(p)),
            None => (text, None),
        };
        let addr = canonical(addr.trim().parse().ok()?);
        let max = if addr.is_ipv4() { 32 } else { 128 };
        let prefix = match prefix {
            Some(p) => p.trim().parse::<u8>().ok().filter(|p| *p <= max)?,
            None => max,
        };
        Some(Self { addr, prefix })
    }

    pub fn contains(&self, ip: IpAddr) -> bool {
        match (self.addr, canonical(ip)) {
            (IpAddr::V4(net), IpAddr::V4(ip)) => {
                same_prefix(u128::from(u32::from(net)), u128::from(u32::from(ip)), self.prefix, 32)
            }
            (IpAddr::V6(net), IpAddr::V6(ip)) => {
                same_prefix(u128::from(net), u128::from(ip), self.prefix, 128)
            }
            _ => false,
        }
    }
}

/// Whether the first `prefix` of `bits` bits of `a` and `b` agree.
fn same_prefix(a: u128, b: u128, prefix: u8, bits: u32) -> bool {
    let prefix = u32::from(prefix);
    prefix == 0 || ((a ^ b) >> (bits - prefix)) == 0
}
```

Add `pub osc: OscRemoteConfig,` to `RemoteConfig`, between `http` and
`events`. Its `Default` derive covers it. In `RemoteConfig::validate`,
after the HTTP block and before the events clamp:

```rust
        let o = &mut self.osc;
        if o.bind_addr().is_none() {
            w.push(ConfigWarning {
                field: "remote.osc.bind",
                message: format!("{:?} is not an IP address; using 127.0.0.1", o.bind),
            });
            o.bind = "127.0.0.1".to_owned();
        }
        clamp_to(&mut o.port, 1024, u16::MAX, "remote.osc.port", w);
        if o.port == self.http.port && o.bind == self.http.bind {
            let moved = if self.http.port == u16::MAX { self.http.port - 1 } else { self.http.port + 1 };
            w.push(ConfigWarning {
                field: "remote.osc.port",
                message: format!("same as the HTTP port; using {moved}"),
            });
            o.port = moved;
        }
        o.allowed_sources.retain(|s| {
            let ok = Cidr::parse(s).is_some();
            if !ok {
                w.push(ConfigWarning {
                    field: "remote.osc.allowed_sources",
                    message: format!("{s:?} is not an address or subnet; dropped"),
                });
            }
            ok
        });
        clamp_to(&mut o.max_subscribers, 1, 256, "remote.osc.max_subscribers", w);
        clamp_to(&mut o.subscription_ttl_secs, 5, 3600, "remote.osc.subscription_ttl_secs", w);
```

The HTTP block holds `let h = &mut self.http;`. End that borrow before this
block: the clash check reads `self.http`. Export `OscRemoteConfig` from
`lib.rs`.

- [ ] **Step 3: Run** `cargo test -p fp-model --lib remote::` → 12 passed.
- [ ] **Step 4: Gate and commit** `feat(model): OSC remote configuration and source allow-lists`.

---

### Task 2: snapshot diff into events (`events.rs`)

**Files:**
- Create: `crates/fp-remote/src/events.rs`
- Modify: `crates/fp-remote/src/lib.rs` (`pub mod events;`)
- Test: `crates/fp-remote/tests/events.rs`

**Interfaces:**
- Consumes: `dto::{state, player, players, playlist, cartwall, track}`, `Playback`.
- Produces:
  - `events::Event`, with variants `State`, `Player`, `Playlist`,
    `PlaylistRemoved`, `Cartwall`, `Track` and `Position`, plus
    `name() -> &'static str` and `json() -> String`;
  - `events::{diff, position}`;
  - `events::{Envelope { revision: u64, event: Event }, PositionDto, PlayerTimesDto, RemovedDto}`.

- [ ] **Step 1: Failing tests** (`crates/fp-remote/tests/events.rs`)

```rust
#![allow(clippy::unwrap_used)]
mod support;

use fp_model::{Command, MarkerKind, Transport};
use fp_remote::control::Playback;
use fp_remote::events::{Event, diff, position};
use support::demo_state;

fn names(events: &[Event]) -> Vec<&'static str> {
    events.iter().map(Event::name).collect()
}

#[test]
fn an_unchanged_snapshot_has_no_events() {
    let s = demo_state();
    assert!(diff(&s, &s.clone(), &Playback::default()).is_empty());
}

#[test]
fn playing_changes_the_player_and_its_playlist() {
    let old = demo_state();
    let mut new = old.clone();
    let p = new.players[0].id;
    fp_model::apply(&mut new, Command::Play(p)).unwrap();
    let events = diff(&old, &new, &Playback::default());
    assert!(names(&events).contains(&"player"));
    assert!(names(&events).contains(&"playlist"));
    let Some(Event::Player(dto)) = events.iter().find(|e| e.name() == "player") else {
        panic!("{events:?}")
    };
    assert_eq!(dto.id, p);
    assert_eq!(dto.transport, "playing");
}

#[test]
fn a_moving_position_alone_is_not_a_player_change() {
    let mut old = demo_state();
    let p = old.players[0].id;
    fp_model::apply(&mut old, Command::Play(p)).unwrap();
    let new = old.clone();
    let pb = Playback { players: vec![(p, 42.0)], ..Default::default() };
    assert!(diff(&old, &new, &pb).is_empty());
}

#[test]
fn a_deleted_playlist_is_announced() {
    let old = demo_state();
    let mut new = old.clone();
    let night = new.playlists.iter().nth(1).unwrap().id;
    fp_model::apply(&mut new, Command::DeletePlaylist(night)).unwrap();
    let events = diff(&old, &new, &Playback::default());
    assert!(matches!(events.as_slice(), [Event::PlaylistRemoved(r)] if r.id == night), "{events:?}");
}

#[test]
fn a_marker_change_announces_the_track() {
    let old = demo_state();
    let mut new = old.clone();
    let t = new.playlists.iter().next().unwrap().entries[0].track;
    fp_model::apply(
        &mut new,
        Command::SetMarker { track: t, kind: MarkerKind::IntroEnd, secs: Some(9.0) },
    )
    .unwrap();
    assert!(names(&diff(&old, &new, &Playback::default())).contains(&"track"));
}

#[test]
fn firing_a_cart_changes_the_cartwall() {
    let old = demo_state();
    let mut new = old.clone();
    let c = new.cartwall.pages[0].carts[0].id;
    fp_model::apply(&mut new, Command::FireCart(c)).unwrap();
    assert_eq!(names(&diff(&old, &new, &Playback::default())), vec!["cartwall"]);
}

#[test]
fn a_different_player_count_sends_the_whole_state() {
    let old = demo_state();
    let mut new = old.clone();
    fp_model::apply(&mut new, Command::SetPlayerCount(2)).unwrap();
    assert_eq!(names(&diff(&old, &new, &Playback::default())), vec!["state"]);
}

#[test]
fn position_only_while_something_plays() {
    let mut s = demo_state();
    assert!(position(&s, &Playback::default()).is_none());
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(s.players[0].transport, Transport::Playing);
    let pos = position(&s, &Playback { players: vec![(p, 20.0)], ..Default::default() }).unwrap();
    assert_eq!(pos.players[0].elapsed_secs, Some(20.0));
    assert_eq!(pos.players[0].remaining_secs, Some(160.0));
    assert!(pos.carts.is_empty());
}

#[test]
fn events_serialise_to_json() {
    let old = demo_state();
    let mut new = old.clone();
    fp_model::apply(&mut new, Command::FireCart(new.cartwall.pages[0].carts[0].id)).unwrap();
    let json: serde_json::Value =
        serde_json::from_str(&diff(&old, &new, &Playback::default())[0].json()).unwrap();
    assert!(json["playing"].is_array());
}
```

Run: `cargo test -p fp-remote --test events` → compile error (`no events`).

- [ ] **Step 2: Implement `crates/fp-remote/src/events.rs`**

```rust
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
```

Check that `Playlists::references_track` and `Cartwall::references_track`
are `pub`. Both are, in `fp-model`.

- [ ] **Step 3: Run** `cargo test -p fp-remote --test events` → 9 passed.
- [ ] **Step 4: Gate and commit** `feat(remote): resource events from snapshot diffs`.

---

### Task 3: publisher and `GET /events`

**Files:**
- Create: `crates/fp-remote/src/throttle.rs`, `crates/fp-remote/src/http/events.rs`
- Modify:
  - `crates/fp-remote/src/http/mod.rs` (`Ctx` fields, `param`, the route);
  - `crates/fp-remote/src/http/guard.rs` (`?token=` on `/events`, uses
    `Throttle`);
  - `crates/fp-remote/src/server.rs` (publisher; streams ended and the task
    aborted on stop);
  - `crates/fp-remote/src/lib.rs`;
  - `crates/fp-remote/Cargo.toml` (`futures-util`).
- Test: `crates/fp-remote/tests/sse.rs`; update the `check` calls in
  `tests/guard.rs`.

**Interfaces:**
- Produces:
  - `Ctx { …, events: broadcast::Sender<Arc<Envelope>>, event_clients: Arc<AtomicUsize>, stop: Arc<watch::Sender<bool>> }`;
  - `Ctx::with_events(self, broadcast::Sender<Arc<Envelope>>) -> Self`;
  - `http::EVENT_BUFFER: usize = 256`;
  - `guard::check(&HttpRemoteConfig, &HeaderMap, query_token: Option<&str>)`;
  - `throttle::Throttle::note(Option<IpAddr>, &str)`.

- [ ] **Step 1: Dependencies.** Workspace:
  `futures-util = { version = "0.3.34", default-features = false, features = ["std"] }`.
  In `fp-remote`: `futures-util.workspace = true`.

- [ ] **Step 2: Failing tests** (`crates/fp-remote/tests/sse.rs`)

```rust
#![allow(clippy::unwrap_used)]
mod support;

use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use fp_model::HttpRemoteConfig;
use fp_remote::events::{Envelope, Event, RemovedDto};
use fp_remote::http::{Ctx, router};
use http_body_util::BodyExt;
use support::{FakeControl, demo_state};
use tokio::sync::broadcast;
use tower::ServiceExt;

const TOKEN: &str = "0123456789abcdef0123";

fn ctx_with(config: HttpRemoteConfig, capacity: usize) -> (Ctx, broadcast::Sender<Arc<Envelope>>) {
    let (tx, _) = broadcast::channel(capacity);
    let ctx = Ctx::new(FakeControl::new(demo_state()), Arc::new(config)).with_events(tx.clone());
    (ctx, tx)
}

fn get(uri: &str) -> Request<Body> {
    Request::get(uri).header("host", "127.0.0.1:7380").body(Body::empty()).unwrap()
}

/// The next chunk of an event stream as text, or `None` when it ended.
async fn next(body: &mut Body) -> Option<String> {
    let frame = tokio::time::timeout(Duration::from_secs(5), body.frame())
        .await
        .expect("timed out waiting for an event")?;
    let data = frame.unwrap().into_data().ok()?;
    Some(String::from_utf8(data.to_vec()).unwrap())
}

fn removed(id: u64) -> Arc<Envelope> {
    Arc::new(Envelope {
        revision: 5,
        event: Event::PlaylistRemoved(RemovedDto { id: fp_model::PlaylistId(id) }),
    })
}

#[tokio::test]
async fn a_stream_starts_with_the_state_then_carries_events() {
    let (ctx, tx) = ctx_with(HttpRemoteConfig::default(), 16);
    let res = router(ctx).oneshot(get("/api/v1/events")).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.headers()["content-type"], "text/event-stream");
    let mut body = res.into_body();
    let first = next(&mut body).await.unwrap();
    assert!(first.contains("event: state"), "{first}");
    assert!(first.contains("\"players\""), "{first}");
    tx.send(removed(77)).unwrap();
    let second = next(&mut body).await.unwrap();
    assert!(second.contains("event: playlist-removed"), "{second}");
    assert!(second.contains("id: 5"), "{second}");
    assert!(second.contains("{\"id\":77}"), "{second}");
}

#[tokio::test]
async fn topics_filter_the_stream() {
    let (ctx, tx) = ctx_with(HttpRemoteConfig::default(), 16);
    let res = router(ctx).oneshot(get("/api/v1/events?topics=player,position")).await.unwrap();
    let mut body = res.into_body();
    assert!(next(&mut body).await.unwrap().contains("event: state"));
    tx.send(removed(1)).unwrap();
    tx.send(Arc::new(Envelope {
        revision: 6,
        event: Event::Position(fp_remote::events::PositionDto { players: vec![], carts: vec![] }),
    }))
    .unwrap();
    let got = next(&mut body).await.unwrap();
    assert!(got.contains("event: position"), "{got}");
}

#[tokio::test]
async fn a_client_that_falls_behind_gets_a_resync() {
    let (ctx, tx) = ctx_with(HttpRemoteConfig::default(), 1);
    let res = router(ctx).oneshot(get("/api/v1/events")).await.unwrap();
    let mut body = res.into_body();
    assert!(next(&mut body).await.unwrap().contains("event: state"));
    for n in 0..4 {
        tx.send(removed(n)).unwrap();
    }
    let got = next(&mut body).await.unwrap();
    assert!(got.contains("event: resync"), "{got}");
    assert!(got.contains("\"players\""), "{got}");
}

#[tokio::test]
async fn the_client_limit_is_enforced_and_freed_on_disconnect() {
    let config = HttpRemoteConfig { max_event_clients: 1, ..Default::default() };
    let (ctx, _tx) = ctx_with(config, 16);
    let first = router(ctx.clone()).oneshot(get("/api/v1/events")).await.unwrap();
    assert_eq!(first.status(), StatusCode::OK);
    let second = router(ctx.clone()).oneshot(get("/api/v1/events")).await.unwrap();
    assert_eq!(second.status(), StatusCode::SERVICE_UNAVAILABLE);
    drop(first);
    let third = router(ctx).oneshot(get("/api/v1/events")).await.unwrap();
    assert_eq!(third.status(), StatusCode::OK);
}

#[tokio::test]
async fn stopping_ends_open_streams() {
    let (ctx, _tx) = ctx_with(HttpRemoteConfig::default(), 16);
    let stop = ctx.stop.clone();
    let res = router(ctx).oneshot(get("/api/v1/events")).await.unwrap();
    let mut body = res.into_body();
    assert!(next(&mut body).await.unwrap().contains("event: state"));
    stop.send(true).unwrap();
    assert_eq!(next(&mut body).await, None);
}

#[tokio::test]
async fn the_token_may_come_in_the_query_only_for_events() {
    let config = HttpRemoteConfig { token: TOKEN.into(), ..Default::default() };
    let (ctx, _tx) = ctx_with(config, 16);
    let ok = router(ctx.clone())
        .oneshot(get(&format!("/api/v1/events?token={TOKEN}")))
        .await
        .unwrap();
    assert_eq!(ok.status(), StatusCode::OK);
    let refused = router(ctx.clone())
        .oneshot(get(&format!("/api/v1/state?token={TOKEN}")))
        .await
        .unwrap();
    assert_eq!(refused.status(), StatusCode::UNAUTHORIZED);
    let wrong = router(ctx).oneshot(get("/api/v1/events?token=nope")).await.unwrap();
    assert_eq!(wrong.status(), StatusCode::UNAUTHORIZED);
}
```

In `tests/guard.rs`, change every `check(&c, &h)` to `check(&c, &h, None)`.
Run: `cargo test -p fp-remote --test sse` → compile errors.

- [ ] **Step 3: Implement**

`crates/fp-remote/src/throttle.rs`, replacing `RejectLog` in `guard.rs`:

```rust
//! Log lines about refused input, at most one per source per second, so a
//! flood cannot flood the log.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

const LOG_EVERY: Duration = Duration::from_secs(1);
/// Sources remembered before old ones are forgotten.
const LOG_SOURCES: usize = 256;

#[derive(Default)]
pub struct Throttle {
    last: Mutex<HashMap<Option<IpAddr>, Instant>>,
}

impl Throttle {
    pub fn note(&self, source: Option<IpAddr>, what: &str) {
        let now = Instant::now();
        let mut last = self.last.lock().unwrap_or_else(PoisonError::into_inner);
        if last.len() >= LOG_SOURCES {
            last.retain(|_, at| now.duration_since(*at) < LOG_EVERY);
        }
        let due = last.get(&source).is_none_or(|at| now.duration_since(*at) >= LOG_EVERY);
        if due {
            last.insert(source, now);
            tracing::warn!(?source, what, "remote input refused");
        }
    }
}
```

In `guard.rs`:
- drop `RejectLog` with its constants and imports;
- `ctx.rejections` becomes `Arc<Throttle>`, called as
  `ctx.rejections.note(source, error.code())`;
- `check` takes `query_token: Option<&str>`. The token test becomes:

```rust
        let header_ok = given.is_some_and(|t| same(t.as_bytes(), config.token.as_bytes()));
        let query_ok = query_token.is_some_and(|t| same(t.as_bytes(), config.token.as_bytes()));
        if !(header_ok || query_ok) {
            return Err(ApiError::Unauthorized);
        }
```

  and `guard` computes it:

```rust
    let query_token = (req.uri().path() == super::EVENTS_PATH)
        .then(|| req.uri().query().and_then(|q| super::param(q, "token")))
        .flatten();
    match check(&ctx.config, req.headers(), query_token.as_deref()) {
```

In `http/mod.rs`:
- add `mod events;`;
- add the route `.route("/events", get(events::events))`;
- add `pub(crate) const EVENTS_PATH: &str = "/api/v1/events";`;
- add `pub const EVENT_BUFFER: usize = 256;`;
- give `Ctx` these fields and the `with_events` method:

```rust
    pub rejections: Arc<Throttle>,
    pub events: broadcast::Sender<Arc<Envelope>>,
    pub event_clients: Arc<AtomicUsize>,
    /// Set to true to end every open event stream.
    pub stop: Arc<watch::Sender<bool>>,
```

```rust
impl Ctx {
    pub fn new(control: Arc<dyn RemoteControl>, config: Arc<HttpRemoteConfig>) -> Self {
        Self {
            control,
            config,
            rejections: Arc::default(),
            events: broadcast::channel(EVENT_BUFFER).0,
            event_clients: Arc::default(),
            stop: Arc::new(watch::channel(false).0),
        }
    }

    /// Streams events from the server's publisher.
    pub fn with_events(mut self, events: broadcast::Sender<Arc<Envelope>>) -> Self {
        self.events = events;
        self
    }
}

/// The value of `name` in a query string, percent-decoded.
pub(crate) fn param(query: &str, name: &str) -> Option<String> {
    query.split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        (key == name).then(|| percent_decode(value))
    })
}

fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while let Some(&b) = bytes.get(i) {
        let hex = (b == b'%')
            .then(|| bytes.get(i + 1..i + 3))
            .flatten()
            .and_then(|h| std::str::from_utf8(h).ok())
            .and_then(|h| u8::from_str_radix(h, 16).ok());
        match (b, hex) {
            (_, Some(v)) => {
                out.push(v);
                i += 3;
            }
            (b'+', None) => {
                out.push(b' ');
                i += 1;
            }
            (b, None) => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}
```

`crates/fp-remote/src/http/events.rs`:

```rust
//! `GET /events` (remote control spec §5.2): a full `state`, then resource
//! events; `resync` with a full state when the client fell behind. The
//! publisher never waits for a client.

use std::convert::Infallible;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use axum::extract::{RawQuery, State};
use axum::response::sse::{Event as SseEvent, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use futures_util::stream;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{broadcast, watch};

use super::Ctx;
use crate::api::ApiError;
use crate::dto;
use crate::events::Envelope;

/// A comment line this often keeps proxies and NAT from closing the stream.
const KEEP_ALIVE: Duration = Duration::from_secs(15);

/// One of `max_event_clients`; given back when the stream is dropped.
struct Slot(Arc<AtomicUsize>);

impl Drop for Slot {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

fn take_slot(ctx: &Ctx) -> Option<Slot> {
    let taken = ctx.event_clients.fetch_add(1, Ordering::AcqRel);
    let slot = Slot(ctx.event_clients.clone());
    (taken < ctx.config.max_event_clients as usize).then_some(slot)
}

struct Feed {
    ctx: Ctx,
    rx: broadcast::Receiver<Arc<Envelope>>,
    stop: watch::Receiver<bool>,
    topics: Option<Vec<String>>,
    started: bool,
    _slot: Slot,
}

impl Feed {
    fn wants(&self, name: &str) -> bool {
        self.topics.as_ref().is_none_or(|t| t.iter().any(|x| x == name))
    }

    /// The whole state, as `state` or `resync`.
    fn full(&self, name: &'static str) -> SseEvent {
        let playback = self.ctx.control.playback();
        let state = dto::state(&self.ctx.control.model(), &playback);
        SseEvent::default()
            .event(name)
            .id(playback.revision.to_string())
            .data(serde_json::to_string(&state).unwrap_or_default())
    }
}

pub async fn events(State(ctx): State<Ctx>, RawQuery(query): RawQuery) -> Response {
    let Some(slot) = take_slot(&ctx) else {
        return ApiError::Busy.into_response();
    };
    let topics = query
        .as_deref()
        .and_then(|q| super::param(q, "topics"))
        .map(|t| t.split(',').map(str::to_owned).collect());
    let feed = Feed {
        rx: ctx.events.subscribe(),
        stop: ctx.stop.subscribe(),
        topics,
        started: false,
        _slot: slot,
        ctx,
    };
    let stream = stream::unfold(feed, |mut f| async move {
        if !f.started {
            f.started = true;
            let first = f.full("state");
            return Some((Ok::<_, Infallible>(first), f));
        }
        loop {
            tokio::select! {
                _ = f.stop.changed() => return None,
                received = f.rx.recv() => match received {
                    Ok(env) => {
                        if f.wants(env.event.name()) {
                            let e = SseEvent::default()
                                .event(env.event.name())
                                .id(env.revision.to_string())
                                .data(env.event.json());
                            return Some((Ok(e), f));
                        }
                    }
                    Err(RecvError::Lagged(_)) => {
                        let e = f.full("resync");
                        return Some((Ok(e), f));
                    }
                    Err(RecvError::Closed) => return None,
                },
            }
        }
    });
    Sse::new(stream)
        .keep_alive(KeepAlive::new().interval(KEEP_ALIVE).text("keepalive"))
        .into_response()
}
```

`state` and `resync` are always sent, whatever the topics: `full` is not
filtered.

In `server.rs`:
- **Publisher.** In `supervise`, before the loop:

```rust
    let (events, _) = broadcast::channel::<Arc<Envelope>>(EVENT_BUFFER);
    let publisher = tokio::spawn(publish(control.clone(), events.clone()));
```

  At the end of `supervise`, call `publisher.abort();`. The function itself:

```rust
/// How often the publisher looks for a new snapshot.
const PUBLISH_EVERY: Duration = Duration::from_millis(50);

/// Diffs snapshots into events, and reports positions every
/// `position_interval_ms` while something plays. Idle without listeners.
async fn publish(control: Arc<dyn RemoteControl>, events: broadcast::Sender<Arc<Envelope>>) {
    let mut last = control.model();
    let mut last_position = tokio::time::Instant::now();
    let mut tick = tokio::time::interval(PUBLISH_EVERY);
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tick.tick().await;
        let model = control.model();
        if events.receiver_count() == 0 {
            last = model;
            continue;
        }
        let playback = control.playback();
        if !Arc::ptr_eq(&last, &model) {
            for event in events::diff(&last, &model, &playback) {
                let _ = events.send(Arc::new(Envelope { revision: playback.revision, event }));
            }
            last = model.clone();
        }
        let every = Duration::from_millis(model.config.remote.events.position_interval_ms.into());
        if last_position.elapsed() >= every {
            last_position = tokio::time::Instant::now();
            if let Some(p) = events::position(&model, &playback) {
                let _ = events.send(Arc::new(Envelope {
                    revision: playback.revision,
                    event: Event::Position(p),
                }));
            }
        }
    }
}
```

- **Starting HTTP.** `start` takes `events: &broadcast::Sender<Arc<Envelope>>`
  and builds the context with
  `Ctx::new(..).with_events(events.clone())`. It keeps
  `let streams = ctx.stop.clone();` before handing `ctx` to `router`.
- **`Running`.** It gains `streams: Option<Arc<watch::Sender<bool>>>`, and
  `task` becomes `JoinHandle<()>` (the serve result is logged in the task).
  `stop` becomes:

```rust
    async fn stop(self) {
        if let Some(streams) = &self.streams {
            let _ = streams.send(true);
        }
        let _ = self.shutdown.send(());
        let mut task = self.task;
        if tokio::time::timeout(SHUTDOWN_GRACE, &mut task).await.is_err() {
            task.abort();
        }
    }
```

  This also resolves plan 1's deferred minor "old server not aborted after
  the grace".

- [ ] **Step 4: Server-level test** (append to `tests/server.rs`). An event
  stream over TCP sees a model change made after it connected.

```rust
#[test]
fn an_event_stream_sees_a_change_made_after_it_connected() {
    let fake = FakeControl::new(demo_state());
    let port = free_port();
    fake.edit(|s| {
        s.config.remote.http.enabled = true;
        s.config.remote.http.port = port;
    });
    let handle = spawn(fake.clone()).unwrap();
    let ServerStatus::Listening(addr) =
        wait_for(&handle, "listening", |s| matches!(s, ServerStatus::Listening(_)))
    else {
        unreachable!()
    };
    let mut s = TcpStream::connect(addr).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    write!(s, "GET /api/v1/events HTTP/1.1\r\nHost: {addr}\r\n\r\n").unwrap();
    let mut seen = String::new();
    let mut buf = [0u8; 4096];
    while !seen.contains("event: state") {
        let n = s.read(&mut buf).unwrap();
        seen.push_str(&String::from_utf8_lossy(&buf[..n]));
    }
    let night = fake.state.load().playlists.iter().nth(1).unwrap().id;
    fake.edit(|st| {
        fp_model::apply(st, fp_model::Command::DeletePlaylist(night)).unwrap();
    });
    while !seen.contains("event: playlist-removed") {
        let n = s.read(&mut buf).unwrap();
        assert!(n > 0, "stream closed: {seen}");
        seen.push_str(&String::from_utf8_lossy(&buf[..n]));
    }
    drop(handle);
}
```

- [ ] **Step 5: Run** `cargo test -p fp-remote` → all pass. Check that
  `dropping_the_handle_stops_the_server` still finishes in under 5 s with an
  event stream open: add an open `/events` connection to that test before
  `drop(handle)`.
- [ ] **Step 6: Gate, `cargo deny check`, commit** `feat(remote): event stream over SSE`.

---

### Task 4: OSC parsing, values and subscribers (`osc.rs`, pure)

**Files:**
- Create: `crates/fp-remote/src/osc.rs`
- Modify:
  - `Cargo.toml`: workspace `rosc = "0.11.4"`;
  - `crates/fp-remote/Cargo.toml`: `rosc.workspace = true`;
  - `lib.rs`: `pub mod osc;`.
- Test: `crates/fp-remote/tests/osc.rs`

**Interfaces:**
- Produces:
  - `osc::OscRequest::{Op(Operation), Subscribe(Option<u16>), Unsubscribe(Option<u16>)}`;
  - `osc::parse(&OscMessage, &AppState) -> Result<OscRequest, &'static str>`;
  - `osc::messages(&[u8]) -> Result<Vec<OscMessage>, &'static str>`;
  - `osc::values(&AppState, &Playback) -> Vec<(String, OscType)>`;
  - `osc::Subscribers` with `new(max, ttl)`, `subscribe(addr, now) -> bool`,
    `unsubscribe`, `expire(now)`, `reset`, `is_empty`, `len` and
    `changes(&[(String, OscType)]) -> Vec<(SocketAddr, Vec<OscMessage>)>`.

- [ ] **Step 1: Failing tests** (`crates/fp-remote/tests/osc.rs`)

```rust
#![allow(clippy::unwrap_used)]
mod support;

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use fp_model::{Command, PlayMode};
use fp_remote::api::Operation as O;
use fp_remote::control::Playback;
use fp_remote::osc::{OscRequest, Subscribers, messages, parse, values};
use rosc::{OscBundle, OscMessage, OscPacket, OscTime, OscType, encoder};
use support::demo_state;

fn msg(addr: &str, args: Vec<OscType>) -> OscMessage {
    OscMessage { addr: addr.into(), args }
}

#[test]
fn player_buttons_act_on_press_and_ignore_release() {
    let s = demo_state();
    let p = s.players[1].id;
    assert_eq!(parse(&msg("/fauste/player/2/play", vec![]), &s), Ok(OscRequest::Op(O::Play(p))));
    assert_eq!(
        parse(&msg("/fauste/player/2/fade-stop", vec![OscType::Float(1.0)]), &s),
        Ok(OscRequest::Op(O::FadeStop(p)))
    );
    assert!(parse(&msg("/fauste/player/2/play", vec![OscType::Int(0)]), &s).is_err());
    assert!(parse(&msg("/fauste/player/2/play", vec![OscType::String("x".into())]), &s).is_err());
}

#[test]
fn cue_and_volume_take_values() {
    let s = demo_state();
    let p = s.players[0].id;
    assert_eq!(
        parse(&msg("/fauste/player/1/cue", vec![OscType::Bool(true)]), &s),
        Ok(OscRequest::Op(O::SetCue(p, true)))
    );
    assert_eq!(
        parse(&msg("/fauste/player/1/cue", vec![OscType::Int(0)]), &s),
        Ok(OscRequest::Op(O::SetCue(p, false)))
    );
    assert_eq!(
        parse(&msg("/fauste/player/1/volume", vec![OscType::Float(0.5)]), &s),
        Ok(OscRequest::Op(O::SetFader(p, 0.5)))
    );
    assert!(parse(&msg("/fauste/player/1/volume", vec![]), &s).is_err());
}

#[test]
fn positions_outside_the_screen_are_refused() {
    let s = demo_state();
    for addr in [
        "/fauste/player/0/play",
        "/fauste/player/99/play",
        "/fauste/player/x/play",
        "/fauste/player/1/dance",
        "/fauste/cart/999/fire",
        "/other/thing",
        "/fauste/",
    ] {
        assert!(parse(&msg(addr, vec![]), &s).is_err(), "{addr}");
    }
}

#[test]
fn carts_fire_on_the_page_shown_or_a_given_page() {
    let s = demo_state();
    let first = s.cartwall.pages[0].carts[0].id;
    assert_eq!(parse(&msg("/fauste/cart/1/fire", vec![]), &s), Ok(OscRequest::Op(O::FireCart(first))));
    assert_eq!(
        parse(&msg("/fauste/cartwall/page/1/cart/1/fire", vec![]), &s),
        Ok(OscRequest::Op(O::FireCart(first)))
    );
    assert_eq!(parse(&msg("/fauste/cartwall/stop-all", vec![]), &s), Ok(OscRequest::Op(O::StopAllCarts)));
    assert!(parse(&msg("/fauste/cartwall/page/previous", vec![]), &s).is_err());
}

#[test]
fn page_next_moves_to_the_following_page() {
    let mut s = demo_state();
    fp_model::apply(&mut s, Command::CreateCartPage { name: "B".into() }).unwrap();
    let second = s.cartwall.pages[1].id;
    assert_eq!(
        parse(&msg("/fauste/cartwall/page/next", vec![]), &s),
        Ok(OscRequest::Op(O::ShowCartPage(second)))
    );
}

#[test]
fn subscriptions_take_an_optional_port() {
    let s = demo_state();
    assert_eq!(parse(&msg("/fauste/subscribe", vec![]), &s), Ok(OscRequest::Subscribe(None)));
    assert_eq!(
        parse(&msg("/fauste/subscribe", vec![OscType::Int(9000)]), &s),
        Ok(OscRequest::Subscribe(Some(9000)))
    );
    assert!(parse(&msg("/fauste/subscribe", vec![OscType::Int(70000)]), &s).is_err());
    assert_eq!(parse(&msg("/fauste/unsubscribe", vec![]), &s), Ok(OscRequest::Unsubscribe(None)));
}

#[test]
fn bundles_are_flattened_and_garbage_is_refused() {
    let bundle = OscPacket::Bundle(OscBundle {
        timetag: OscTime { seconds: 0, fractional: 1 },
        content: vec![
            OscPacket::Message(msg("/fauste/player/1/play", vec![])),
            OscPacket::Message(msg("/fauste/player/2/play", vec![])),
        ],
    });
    let got = messages(&encoder::encode(&bundle).unwrap()).unwrap();
    assert_eq!(got.len(), 2);
    assert!(messages(b"garbage").is_err());
    assert!(messages(&[]).is_err());
}

#[test]
fn values_describe_players_and_the_cart_page() {
    let mut s = demo_state();
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    fp_model::apply(&mut s, Command::SetMode(s.players[1].id, PlayMode::Single)).unwrap();
    let v = values(&s, &Playback { players: vec![(p, 10.0)], ..Default::default() });
    let get = |a: &str| v.iter().find(|(addr, _)| addr == a).map(|(_, v)| v.clone()).unwrap();
    assert_eq!(get("/fauste/player/1/transport"), OscType::String("playing".into()));
    assert_eq!(get("/fauste/player/1/elapsed"), OscType::Float(10.0));
    assert_eq!(get("/fauste/player/1/remaining"), OscType::Float(170.0));
    assert_eq!(get("/fauste/player/2/transport"), OscType::String("stopped".into()));
    assert_eq!(get("/fauste/player/2/title"), OscType::String(String::new()));
    assert_eq!(get("/fauste/cartwall/page"), OscType::Int(1));
    assert_eq!(get("/fauste/cart/1/playing"), OscType::Int(0));
    assert!(matches!(get("/fauste/player/1/next/entry"), OscType::Long(n) if n > 0));
}

#[test]
fn subscribers_get_everything_first_then_only_changes() {
    let to: SocketAddr = "127.0.0.1:9000".parse().unwrap();
    let mut subs = Subscribers::new(2, Duration::from_secs(60));
    let now = Instant::now();
    assert!(subs.subscribe(to, now));
    let table = vec![
        ("/a".to_owned(), OscType::Int(1)),
        ("/b".to_owned(), OscType::Int(2)),
    ];
    let first = subs.changes(&table);
    assert_eq!(first[0].1.len(), 2);
    assert!(subs.changes(&table).is_empty());
    let changed = vec![
        ("/a".to_owned(), OscType::Int(1)),
        ("/b".to_owned(), OscType::Int(3)),
    ];
    let second = subs.changes(&changed);
    assert_eq!(second[0].1.len(), 1);
    assert_eq!(second[0].1[0].addr, "/b");
    subs.reset();
    assert_eq!(subs.changes(&changed)[0].1.len(), 2);
}

#[test]
fn subscriptions_expire_renew_and_are_capped() {
    let a: SocketAddr = "127.0.0.1:9000".parse().unwrap();
    let b: SocketAddr = "127.0.0.1:9001".parse().unwrap();
    let c: SocketAddr = "127.0.0.1:9002".parse().unwrap();
    let ttl = Duration::from_secs(60);
    let mut subs = Subscribers::new(2, ttl);
    let t0 = Instant::now();
    assert!(subs.subscribe(a, t0));
    assert!(subs.subscribe(b, t0));
    assert!(!subs.subscribe(c, t0), "capped at 2");
    assert!(subs.subscribe(a, t0 + Duration::from_secs(50)), "renewal always works");
    subs.expire(t0 + Duration::from_secs(61));
    assert_eq!(subs.len(), 1, "b expired, a was renewed");
    subs.unsubscribe(a);
    assert!(subs.is_empty());
}
```

Run: `cargo test -p fp-remote --test osc` → compile error.

- [ ] **Step 2: Implement `crates/fp-remote/src/osc.rs`**

```rust
//! OSC (remote control spec §4): addresses under `/fauste` turned into
//! operations, the values sent to subscribers, and what each subscriber was
//! last sent. Pure; `osc_server` does the I/O.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use fp_model::{AppState, CartId, CartPage, PlayerId};
use rosc::{OscMessage, OscPacket, OscType};

use crate::api::Operation;
use crate::control::Playback;
use crate::dto;

/// Bundles nested deeper than this are ignored.
const MAX_BUNDLE_DEPTH: usize = 8;

#[derive(Debug, Clone, PartialEq)]
pub enum OscRequest {
    Op(Operation),
    /// To the sender's address, on this port or the packet's source port.
    Subscribe(Option<u16>),
    Unsubscribe(Option<u16>),
}

/// The messages of a packet, bundles flattened in order (time tags ignored).
pub fn messages(packet: &[u8]) -> Result<Vec<OscMessage>, &'static str> {
    let (_, packet) = rosc::decoder::decode_udp(packet).map_err(|_| "malformed OSC packet")?;
    let mut out = Vec::new();
    flatten(packet, &mut out, 0);
    Ok(out)
}

fn flatten(packet: OscPacket, out: &mut Vec<OscMessage>, depth: usize) {
    match packet {
        OscPacket::Message(m) => out.push(m),
        OscPacket::Bundle(b) if depth < MAX_BUNDLE_DEPTH => {
            for p in b.content {
                flatten(p, out, depth + 1);
            }
        }
        OscPacket::Bundle(_) => {}
    }
}

pub fn parse(msg: &OscMessage, state: &AppState) -> Result<OscRequest, &'static str> {
    let path = msg.addr.strip_prefix("/fauste/").ok_or("not a /fauste address")?;
    let parts: Vec<&str> = path.split('/').collect();
    let args = msg.args.as_slice();
    let op = match parts.as_slice() {
        ["player", n, action] => {
            let p = player_at(state, n)?;
            match *action {
                "cue" => return Ok(OscRequest::Op(Operation::SetCue(p, switch(args)?))),
                "volume" => {
                    let v = number(args).ok_or("volume needs a number")?;
                    return Ok(OscRequest::Op(Operation::SetFader(p, v as f32)));
                }
                "play" => Operation::Play(p),
                "pause" => Operation::Pause(p),
                "stop" => Operation::Stop(p),
                "fade-stop" => Operation::FadeStop(p),
                "restart" => Operation::Restart(p),
                "previous" => Operation::Previous(p),
                _ => return Err("unknown player action"),
            }
        }
        ["cart", c, "fire"] => {
            let page = state.cartwall.shown_page().ok_or("no cart page")?;
            Operation::FireCart(cart_at(page, c)?)
        }
        ["cartwall", "page", p, "cart", c, "fire"] => {
            Operation::FireCart(cart_at(page_at(state, p)?, c)?)
        }
        ["cartwall", "stop-all"] => Operation::StopAllCarts,
        ["cartwall", "page", dir @ ("next" | "previous")] => {
            let pages = &state.cartwall.pages;
            let shown = state.cartwall.shown_page().map(|p| p.id);
            let i = pages.iter().position(|p| Some(p.id) == shown).unwrap_or(0);
            let j = if *dir == "next" {
                i + 1
            } else {
                i.checked_sub(1).ok_or("no previous page")?
            };
            Operation::ShowCartPage(pages.get(j).ok_or("no next page")?.id)
        }
        ["subscribe"] => return Ok(OscRequest::Subscribe(port(args)?)),
        ["unsubscribe"] => return Ok(OscRequest::Unsubscribe(port(args)?)),
        _ => return Err("unknown address"),
    };
    pressed(args)?;
    Ok(OscRequest::Op(op))
}

fn number(args: &[OscType]) -> Option<f64> {
    match args.first()? {
        OscType::Int(i) => Some(f64::from(*i)),
        OscType::Float(f) => Some(f64::from(*f)),
        OscType::Double(d) => Some(*d),
        OscType::Long(l) => Some(*l as f64),
        OscType::Bool(b) => Some(f64::from(u8::from(*b))),
        _ => None,
    }
}

/// A button: no argument, or a number above zero (a release sends zero).
fn pressed(args: &[OscType]) -> Result<(), &'static str> {
    if args.is_empty() {
        return Ok(());
    }
    match number(args) {
        Some(v) if v > 0.0 => Ok(()),
        Some(_) => Err("button released"),
        None => Err("a button takes a number or nothing"),
    }
}

fn switch(args: &[OscType]) -> Result<bool, &'static str> {
    number(args).map(|v| v > 0.0).ok_or("needs on or off")
}

fn port(args: &[OscType]) -> Result<Option<u16>, &'static str> {
    if args.is_empty() {
        return Ok(None);
    }
    number(args)
        .filter(|v| v.fract() == 0.0)
        .and_then(|v| u16::try_from(v as i64).ok())
        .filter(|p| *p != 0)
        .map(Some)
        .ok_or("bad port")
}

/// A 1-based position as an index.
fn index(raw: &str) -> Result<usize, &'static str> {
    raw.parse::<usize>()
        .ok()
        .and_then(|n| n.checked_sub(1))
        .ok_or("positions start at 1")
}

fn player_at(state: &AppState, raw: &str) -> Result<PlayerId, &'static str> {
    state.players.get(index(raw)?).map(|p| p.id).ok_or("no such player")
}

fn page_at<'a>(state: &'a AppState, raw: &str) -> Result<&'a CartPage, &'static str> {
    state.cartwall.pages.get(index(raw)?).ok_or("no such cart page")
}

fn cart_at(page: &CartPage, raw: &str) -> Result<CartId, &'static str> {
    page.carts.get(index(raw)?).map(|c| c.id).ok_or("no such cart")
}

fn flag(on: bool) -> OscType {
    OscType::Int(i32::from(on))
}

fn text(s: Option<&str>) -> OscType {
    OscType::String(s.unwrap_or_default().to_owned())
}

/// Every address subscribers follow, with its current value (spec §4.2).
pub fn values(model: &AppState, playback: &Playback) -> Vec<(String, OscType)> {
    let mut out = Vec::new();
    for p in dto::players(model, playback) {
        let n = p.position;
        let at = |leaf: &str| format!("/fauste/player/{n}/{leaf}");
        let current = p.current.as_ref().map(|c| &c.track);
        let next = p.next.as_ref();
        out.push((at("transport"), OscType::String(p.transport.to_owned())));
        out.push((at("fading"), flag(p.fading)));
        out.push((at("cueing"), flag(p.cue.is_some())));
        out.push((at("stop-after-current"), flag(p.stop_after_current)));
        out.push((at("volume"), OscType::Float(p.fader)));
        out.push((at("title"), text(current.map(|t| t.title.as_str()))));
        out.push((at("artist"), text(current.map(|t| t.artist.as_str()))));
        out.push((at("elapsed"), OscType::Float(p.elapsed_secs.unwrap_or(0.0) as f32)));
        out.push((at("remaining"), OscType::Float(p.remaining_secs.unwrap_or(0.0) as f32)));
        let entry = next.map_or(-1, |e| i64::try_from(e.entry.0).unwrap_or(-1));
        out.push((at("next/entry"), OscType::Long(entry)));
        out.push((at("next/title"), text(next.map(|e| e.track.title.as_str()))));
        out.push((at("next/artist"), text(next.map(|e| e.track.artist.as_str()))));
    }
    let cw = dto::cartwall(model, playback);
    let shown = cw
        .pages
        .iter()
        .enumerate()
        .find(|(_, page)| Some(page.id) == cw.shown_page);
    if let Some((i, page)) = shown {
        out.push((
            "/fauste/cartwall/page".to_owned(),
            OscType::Int(i32::try_from(i + 1).unwrap_or(i32::MAX)),
        ));
        for c in &page.carts {
            let n = c.index + 1;
            let playing = cw.playing.iter().any(|pc| pc.cart == c.id);
            out.push((format!("/fauste/cart/{n}/playing"), flag(playing)));
            out.push((format!("/fauste/cart/{n}/name"), OscType::String(c.name.clone())));
        }
    }
    out
}

struct Subscriber {
    expires: Instant,
    /// The last value sent to it, per address.
    sent: HashMap<String, OscType>,
}

/// OSC subscribers and what each was last sent (spec §5.3).
pub struct Subscribers {
    max: usize,
    ttl: Duration,
    list: HashMap<SocketAddr, Subscriber>,
}

impl Subscribers {
    pub fn new(max: usize, ttl: Duration) -> Self {
        Self { max, ttl, list: HashMap::new() }
    }

    /// Adds or renews `to`; false when the table is full.
    pub fn subscribe(&mut self, to: SocketAddr, now: Instant) -> bool {
        if let Some(s) = self.list.get_mut(&to) {
            s.expires = now + self.ttl;
            return true;
        }
        if self.list.len() >= self.max {
            return false;
        }
        self.list.insert(to, Subscriber { expires: now + self.ttl, sent: HashMap::new() });
        true
    }

    pub fn unsubscribe(&mut self, to: SocketAddr) {
        self.list.remove(&to);
    }

    pub fn expire(&mut self, now: Instant) {
        self.list.retain(|_, s| s.expires > now);
    }

    /// Forgets what was sent, so the next `changes` is a full dump.
    pub fn reset(&mut self) {
        for s in self.list.values_mut() {
            s.sent.clear();
        }
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    /// For each subscriber, the messages whose value it has not been sent.
    pub fn changes(&mut self, values: &[(String, OscType)]) -> Vec<(SocketAddr, Vec<OscMessage>)> {
        let mut out = Vec::new();
        for (to, s) in &mut self.list {
            let mut messages = Vec::new();
            for (addr, value) in values {
                if s.sent.get(addr) != Some(value) {
                    s.sent.insert(addr.clone(), value.clone());
                    messages.push(OscMessage { addr: addr.clone(), args: vec![value.clone()] });
                }
            }
            if !messages.is_empty() {
                out.push((*to, messages));
            }
        }
        out
    }
}
```

`OscRequest` derives `PartialEq` and contains `Operation`, which already
derives it.

- [ ] **Step 3: Run** `cargo test -p fp-remote --test osc` → 10 passed.
- [ ] **Step 4: Gate, `cargo deny check`, commit** `feat(remote): OSC addresses, values and subscriptions`.

---

### Task 5: the OSC server

**Files:**
- Create: `crates/fp-remote/src/osc_server.rs`
- Modify:
  - `server.rs` (OSC follows `config.remote.osc`; `RemoteStatus.osc`);
  - `lib.rs`.
- Test: `crates/fp-remote/tests/osc_server.rs`

**Interfaces:**
- Consumes: `osc::*`, `api::plan`, `Throttle`, the publisher's broadcast.
- Produces: `RemoteStatus { http: ServerStatus, osc: ServerStatus }`.

- [ ] **Step 1: Failing tests** (`crates/fp-remote/tests/osc_server.rs`)

```rust
#![allow(clippy::unwrap_used)]
mod support;

use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

use fp_model::{Command, Transport};
use fp_remote::{RemoteHandle, ServerStatus, spawn};
use rosc::{OscMessage, OscPacket, OscType, decoder, encoder};
use support::{FakeControl, demo_state};

fn free_udp_port() -> u16 {
    UdpSocket::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

fn osc_addr(handle: &RemoteHandle) -> SocketAddr {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let ServerStatus::Listening(a) = handle.status().osc {
            return a;
        }
        assert!(Instant::now() < deadline, "{:?}", handle.status());
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn send(sock: &UdpSocket, to: SocketAddr, addr: &str, args: Vec<OscType>) {
    let bytes = encoder::encode(&OscPacket::Message(OscMessage { addr: addr.into(), args })).unwrap();
    sock.send_to(&bytes, to).unwrap();
}

fn enabled(fake: &FakeControl) -> u16 {
    let port = free_udp_port();
    fake.edit(|s| {
        s.config.remote.osc.enabled = true;
        s.config.remote.osc.port = port;
    });
    port
}

fn wait_sent(fake: &FakeControl, n: usize) -> Vec<Command> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let sent = fake.sent.lock().unwrap().clone();
        if sent.len() >= n {
            return sent;
        }
        assert!(Instant::now() < deadline, "only {sent:?}");
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn a_play_message_queues_play() {
    let fake = FakeControl::new(demo_state());
    enabled(&fake);
    let handle = spawn(fake.clone()).unwrap();
    let to = osc_addr(&handle);
    let sock = UdpSocket::bind("127.0.0.1:0").unwrap();
    send(&sock, to, "/fauste/player/1/play", vec![OscType::Int(0)]);
    send(&sock, to, "/fauste/player/1/play", vec![OscType::Int(1)]);
    let p = fake.state.load().players[0].id;
    assert_eq!(wait_sent(&fake, 1), vec![Command::Play(p)]);
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(fake.take_sent().len(), 1, "the release did nothing");
}

#[test]
fn packets_from_other_sources_are_dropped() {
    let fake = FakeControl::new(demo_state());
    enabled(&fake);
    fake.edit(|s| s.config.remote.osc.allowed_sources = vec!["10.0.0.0/8".into()]);
    let handle = spawn(fake.clone()).unwrap();
    let to = osc_addr(&handle);
    let sock = UdpSocket::bind("127.0.0.1:0").unwrap();
    send(&sock, to, "/fauste/player/1/play", vec![]);
    sock.send_to(b"garbage", to).unwrap();
    std::thread::sleep(Duration::from_millis(200));
    assert!(fake.take_sent().is_empty());
    assert!(matches!(handle.status().osc, ServerStatus::Listening(_)), "still running");
}

#[test]
fn a_subscriber_gets_a_dump_then_changes() {
    let fake = FakeControl::new(demo_state());
    enabled(&fake);
    let handle = spawn(fake.clone()).unwrap();
    let to = osc_addr(&handle);
    let sock = UdpSocket::bind("127.0.0.1:0").unwrap();
    sock.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    send(&sock, to, "/fauste/subscribe", vec![]);
    let mut buf = [0u8; 65536];
    let mut seen: Vec<(String, Vec<OscType>)> = Vec::new();
    let mut read_until = |want: &str, seen: &mut Vec<(String, Vec<OscType>)>| {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !seen.iter().any(|(a, v)| format!("{a} {v:?}").contains(want)) {
            assert!(Instant::now() < deadline, "never saw {want}: {seen:?}");
            let (n, _) = sock.recv_from(&mut buf).unwrap();
            if let Ok((_, OscPacket::Message(m))) = decoder::decode_udp(&buf[..n]) {
                seen.push((m.addr, m.args));
            }
        }
    };
    read_until("/fauste/player/1/transport [String(\"stopped\")]", &mut seen);
    let p = fake.state.load().players[0].id;
    fake.edit(|s| {
        fp_model::apply(s, Command::Play(p)).unwrap();
        assert_eq!(s.players[0].transport, Transport::Playing);
    });
    seen.clear();
    read_until("/fauste/player/1/transport [String(\"playing\")]", &mut seen);
    assert!(
        !seen.iter().any(|(a, _)| a == "/fauste/player/2/transport"),
        "unchanged values are not resent: {seen:?}"
    );
}

#[test]
fn disabling_osc_closes_the_socket_and_dropping_stops_it() {
    let fake = FakeControl::new(demo_state());
    let port = enabled(&fake);
    let handle = spawn(fake.clone()).unwrap();
    osc_addr(&handle);
    fake.edit(|s| s.config.remote.osc.enabled = false);
    let deadline = Instant::now() + Duration::from_secs(5);
    while handle.status().osc != ServerStatus::Off {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(UdpSocket::bind(("127.0.0.1", port)).is_ok(), "port freed");
    drop(handle);
}
```

Run: `cargo test -p fp-remote --test osc_server` → compile error (`no field osc`).

- [ ] **Step 2: Implement `crates/fp-remote/src/osc_server.rs`**

```rust
//! The OSC socket (remote control spec §4, §5.3): checks the source,
//! decodes, plans and queues commands, and pushes changed values to
//! subscribers after every event and subscription.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use fp_model::{CartPageId, OscRemoteConfig};
use rosc::OscPacket;
use tokio::net::UdpSocket;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{broadcast, oneshot};

use crate::api;
use crate::control::RemoteControl;
use crate::events::Envelope;
use crate::osc::{self, OscRequest, Subscribers};
use crate::throttle::Throttle;

/// The largest UDP payload.
const MAX_PACKET: usize = 65_536;
/// How often expired subscriptions are dropped.
const EXPIRY_CHECK: Duration = Duration::from_secs(1);

pub(crate) async fn run(
    socket: UdpSocket,
    control: Arc<dyn RemoteControl>,
    config: OscRemoteConfig,
    mut events: broadcast::Receiver<Arc<Envelope>>,
    mut stop: oneshot::Receiver<()>,
) {
    let mut buf = vec![0u8; MAX_PACKET];
    let ttl = Duration::from_secs(config.subscription_ttl_secs.into());
    let mut subs = Subscribers::new(config.max_subscribers as usize, ttl);
    let mut shape: Option<(usize, Option<CartPageId>)> = None;
    let log = Throttle::default();
    let mut expiry = tokio::time::interval(EXPIRY_CHECK);
    loop {
        tokio::select! {
            _ = &mut stop => break,
            received = socket.recv_from(&mut buf) => {
                let Ok((len, from)) = received else { continue };
                if !config.allows(from.ip()) {
                    log.note(Some(from.ip()), "OSC source not allowed");
                    continue;
                }
                let Some(packet) = buf.get(..len) else { continue };
                if handle(packet, from, &*control, &mut subs, &log) {
                    push(&socket, &*control, &mut subs, &mut shape, &log).await;
                }
            }
            event = events.recv() => {
                if matches!(event, Err(RecvError::Closed)) {
                    break;
                }
                push(&socket, &*control, &mut subs, &mut shape, &log).await;
            }
            _ = expiry.tick() => subs.expire(Instant::now()),
        }
    }
}

/// Acts on one packet; true when a subscriber was added or renewed.
fn handle(
    packet: &[u8],
    from: SocketAddr,
    control: &dyn RemoteControl,
    subs: &mut Subscribers,
    log: &Throttle,
) -> bool {
    let source = Some(from.ip());
    let messages = match osc::messages(packet) {
        Ok(m) => m,
        Err(why) => {
            log.note(source, why);
            return false;
        }
    };
    let mut subscribed = false;
    for message in messages {
        let model = control.model();
        let to = |port: Option<u16>| SocketAddr::new(from.ip(), port.unwrap_or(from.port()));
        match osc::parse(&message, &model) {
            Ok(OscRequest::Op(op)) => match api::plan(&model, op) {
                Ok(commands) => {
                    for c in commands {
                        if !control.send(c) {
                            log.note(source, "command queue full");
                        }
                    }
                }
                Err(e) => log.note(source, e.code()),
            },
            Ok(OscRequest::Subscribe(port)) => {
                if subs.subscribe(to(port), Instant::now()) {
                    subscribed = true;
                } else {
                    log.note(source, "too many OSC subscribers");
                }
            }
            Ok(OscRequest::Unsubscribe(port)) => subs.unsubscribe(to(port)),
            Err(why) => log.note(source, why),
        }
    }
    subscribed
}

/// Sends every subscriber the values that changed since it was last sent
/// them; a new player count or page shown sends everything again.
async fn push(
    socket: &UdpSocket,
    control: &dyn RemoteControl,
    subs: &mut Subscribers,
    shape: &mut Option<(usize, Option<CartPageId>)>,
    log: &Throttle,
) {
    if subs.is_empty() {
        return;
    }
    let model = control.model();
    let now = (model.players.len(), model.cartwall.shown_page().map(|p| p.id));
    if *shape != Some(now) {
        subs.reset();
        *shape = Some(now);
    }
    let values = osc::values(&model, &control.playback());
    for (to, messages) in subs.changes(&values) {
        for m in messages {
            let Ok(bytes) = rosc::encoder::encode(&OscPacket::Message(m)) else { continue };
            if socket.send_to(&bytes, to).await.is_err() {
                log.note(Some(to.ip()), "OSC send failed");
            }
        }
    }
}
```

In `server.rs`:
- `RemoteStatus` gains `pub osc: ServerStatus`;
- `supervise` tracks `applied_osc: Option<OscRemoteConfig>` and
  `osc: Option<Running>` exactly like HTTP, and keeps one
  `status: RemoteStatus` that it stores after either changes;
- on exit, both servers stop.

`start_osc`:

```rust
async fn start_osc(
    control: &Arc<dyn RemoteControl>,
    config: &OscRemoteConfig,
    events: &broadcast::Sender<Arc<Envelope>>,
) -> (ServerStatus, Option<Running>) {
    if !config.enabled {
        return (ServerStatus::Off, None);
    }
    let Some(ip) = config.bind_addr() else {
        return (ServerStatus::Error(ServerError::InvalidBind), None);
    };
    let socket = match UdpSocket::bind((ip, config.port)).await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(bind = %ip, port = config.port, error = %e, "remote OSC could not listen");
            return (ServerStatus::Error(ServerError::Bind(e.to_string())), None);
        }
    };
    let addr = socket.local_addr().unwrap_or_else(|_| SocketAddr::new(ip, config.port));
    let (shutdown, signal) = oneshot::channel();
    let task = tokio::spawn(crate::osc_server::run(
        socket,
        control.clone(),
        config.clone(),
        events.subscribe(),
        signal,
    ));
    tracing::info!(%addr, "remote OSC listening");
    (ServerStatus::Listening(addr), Some(Running { shutdown, task, streams: None }))
}
```

Add `mod osc_server;` and `pub mod throttle;` to `lib.rs` (the latter
already in Task 3).

- [ ] **Step 3: Run** `cargo test -p fp-remote` → all pass. Repeat
  `--test osc_server` five times to check for flakiness.
- [ ] **Step 4: Gate and commit** `feat(remote): OSC server with subscriptions`.

---

### Task 6: documentation and spec

**Files:**
- `docs/technical/remote-api.md`: sections "Events (SSE)" and "OSC". Remove
  "SSE and OSC are not implemented yet". Update the implementation section
  (publisher, `osc`, `osc_server`, `throttle`) and the configuration table
  (OSC rows).
- `docs/user/remote-control.md`:
  - a section "Live updates" (`curl -N`, `EventSource` with `?token=`);
  - a section "OSC" (enabling, `allowed_sources`, the address list, how to
    subscribe, and examples with `oscsend` from liblo-tools, e.g.
    `oscsend localhost 7381 /fauste/player/1/play` and
    `oscsend localhost 7381 /fauste/subscribe i 9000` with
    `oscdump 9000`);
  - the OSC settings added to the table.
- `docs/technical/persistence.md`: the `osc.*` rows under `remote`.
- `docs/technical/threading-and-realtime.md`: in the Remote row, add
  "publisher every 50 ms, OSC socket".
- `README.md`: the remote control bullet mentions live events and OSC.
- The spec, §4.2: `next/entry` is `h` (int64, -1 for none), because entry ids
  are `u64` and may pass `i32`.
- `CLAUDE.md`: testing notes add that the OSC tests use UDP sockets on
  `127.0.0.1:0`.

- [ ] **Step 1: Write the docs.**
- [ ] **Step 2: Check the links:** `grep -rn "remote-control.md\|remote-api.md" docs README.md CLAUDE.md`.
- [ ] **Step 3: Gate and commit** `docs: describe remote events and OSC`.

---

## After the last task

Final review with a fresh reviewer on the most capable model, then fix pass,
PR `feat(remote): live events over SSE and OSC control`, CI, merge.
