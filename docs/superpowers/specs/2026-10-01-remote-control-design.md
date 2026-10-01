# Remote Control — Design Spec

- **Date:** 2026-10-01
- **Status:** Approved in brainstorming, pending written review
- **Extends:** [the main design spec](2026-09-25-fauste-player-design.md)
  (it lifts the "remote control over a network" non-goal) and the
  [operator feedback spec](2026-09-30-operator-feedback-design.md) (R28
  availability, §6 MIDI).
- **Scope:** a network API that external integrations and complete remote
  clients (a web page, an Android app) use to read and operate Fauste Player:
  HTTP/JSON with Server-Sent Events, and OSC over UDP. Delivered in three plans
  (§10).

---

## 1. Goals and non-goals

### Goals

- A remote client can do what the operator does at the console: read players,
  playlists, tracks (with cover and waveform) and the cartwall; run the
  transport, choose the next entry, pre-listen, seek, set volumes and modes,
  fire carts; edit playlists, cart pages and markers.
- Clients learn about changes by push: SSE on HTTP, subscriptions on OSC.
- Cross-platform (Linux, Windows, macOS), standard protocols, testable with
  `curl` and `oscsend`.
- Safe by default: off, loopback only; a token and source allow-lists when it
  listens beyond the machine.

### Non-goals (v1)

- Adding files that are not already loaded: no browsing of the host's disk, no
  uploads. A client reuses tracks already in the library (§3.4).
- Station configuration: `UpdateConfig`, shortcuts, MIDI, player count, column
  widths, opening or closing the cartwall strip.
- WebSocket. It can be added later on the same event stream (§5.1) without
  changing the HTTP API.
- TLS. Beyond the LAN, a reverse proxy terminates TLS.
- Correlating a queued command with the model's later verdict (§6.4).

---

## 2. Principles

- **Same rules as the console.** Every request becomes one or more existing
  `Command`s, sent through the same controller channel as the UI. Availability
  is checked with `fp_model::availability` (R28) against the current snapshot,
  exactly as the UI greys out buttons. `fp-remote` translates protocols; the
  player rules stay in `fp-model` (non-negotiable rule 7).
- **Stable contract.** The JSON shapes are DTOs defined in `fp-remote`, versioned
  under `/api/v1`. The internal `AppState` is never serialised directly.
- **Stable ids over HTTP.** Players, playlists, entries, tracks, carts and cart
  pages are addressed by the model's ids (`u64`, persisted across restarts).
  OSC addresses players and carts by their 1-based position on screen, like
  shortcuts and MIDI, because control surfaces think in positions.
- **Desired values, not toggles.** Every on/off setting is set to a value.
  The model has idempotent commands for them (`SetCue`, `SetStopAfterCurrent`,
  `SetEntryRepeat`, `SetEntryStopAfter`, `SetCartCue`), resolved when the
  conductor applies them, so neither a retried request nor two requests
  planned from the same snapshot can undo each other. A value already in
  place sends nothing.
- **Volume is fader travel** `0.0–1.0`, mapped with `gain_from_fader` like the
  UI fader and MIDI.
- **Nothing goes on air by itself** (rule 10). Starting a server or a client
  connecting never plays anything; only new requests act.

---

## 3. HTTP API

Base path `/api/v1`. Requests and responses are JSON (`application/json`,
UTF-8), except cover images and the event stream.

### 3.1 Resources (DTOs)

- **Player:** `id`, `position` (1-based), `transport` (`stopped` | `playing` |
  `paused`), `fading`, `mode` (`single` | `continuous`), `stop_after_current`,
  `fader`, `playlist` (the one shown), `current` and `next` (each `null` or
  `{entry, track}`), `cue` (`null` or `{entry}`), `elapsed_secs`,
  `remaining_secs` (from the engine's telemetry, computed as the UI does:
  elapsed is the position in track seconds, remaining is cue-out minus it;
  both `null` when there is no current entry).
- **Playlist summary:** `id`, `name`, `entry_count`.
- **Playlist:** the summary plus `entries`: `id`, `track` (Track),
  `repeat`, `stop_after`, `on_air` (player ids playing it), `next_on`
  (player ids that will play it next), `cued_on` (player ids pre-listening it).
- **Track:** `id`, `title`, `artist`, `album`, `duration_secs`, `kind`
  (`music` | `jingle` | `effect` | `ad` | `voice`), `file_state` (`ok` |
  `missing` | `unreadable`), `analyzed`, `format` (`null` or
  `{sample_rate, channels, bits}`), `markers` (`cue_in`, `intro_end`,
  `outro_start`, `segue_start`, `cue_out`, each `null` or `{secs, source}`
  with `source` `auto` | `manual`). Whether a cover exists is only known from
  the cache, so it is learnt from `GET /tracks/{id}/cover`. The file path is
  not exposed.
- **Cartwall:** `shown_page`, `pages` (each `id`, `name`, `rows`, `cols`,
  `carts`), `playing` (in firing order: `cart`, `elapsed_secs`,
  `remaining_secs`), `cue` (`null` or a cart id).
- **Cart:** `id`, `index` (row-major, 0-based within its page), `name`,
  `kind` (`jingle` | `effect` | `spot`), `looped`, `exclusive`, `track`
  (`null` or Track).
- **State:** `revision`, `players`, `playlists` (summaries), `cartwall`.

`revision` is the conductor's model version: it rises with every model
change. It orders events (§5) and is not persisted.

### 3.2 Reading

| Method and path | Returns |
|---|---|
| `GET /state` | State |
| `GET /players` | Players, in display order |
| `GET /players/{id}` | Player |
| `GET /playlists` | Playlist summaries |
| `GET /playlists/{id}` | Playlist |
| `GET /tracks/{id}` | Track |
| `GET /tracks/{id}/cover` | The cover thumbnail (`image/png`) |
| `GET /tracks/{id}/peaks` | `{bucket_secs, full_scale: 32767, peaks: [[min, max, rms], …]}` |
| `GET /cartwall` | Cartwall |
| `GET /events` | Event stream (§5.2) |

Cover and peaks come from the UI's media cache, or else the analysis cache
on disk, read with `spawn_blocking`. They never start an analysis: a track
not analysed yet answers `404 not_analyzed`; an analysed track without a cover
(or whose cache entry is gone) answers `404 not_found`.

### 3.3 Operating

| Method and path | Body | Command(s) |
|---|---|---|
| `POST /players/{id}/play` | — | `Play` (Play/Next) |
| `POST /players/{id}/pause` | — | `Pause` |
| `POST /players/{id}/stop` | — | `Stop` |
| `POST /players/{id}/fade-stop` | — | `FadeStop` |
| `POST /players/{id}/restart` | — | `Restart` (R23) |
| `POST /players/{id}/previous` | — | `Previous` (R24) |
| `PUT /players/{id}/cue` | `{"on": bool}` | `SetCue` if it differs |
| `PUT /players/{id}/next` | `{"entry": id}` | `SetNext` |
| `POST /players/{id}/cue-entry` | `{"entry": id}` | `CueEntry` |
| `POST /players/{id}/seek` | `{"secs": f64}` | `Seek` |
| `PUT /players/{id}/volume` | `{"fader": f32}` | `SetVolume(gain_from_fader(fader))` |
| `PUT /players/{id}/mode` | `{"mode": "single" \| "continuous"}` | `SetMode` |
| `PUT /players/{id}/stop-after-current` | `{"on": bool}` | `SetStopAfterCurrent` if it differs |
| `PUT /players/{id}/playlist` | `{"playlist": id}` | `ShowPlaylist` |
| `PUT /entries/{id}/repeat` | `{"on": bool}` | `SetEntryRepeat` if it differs |
| `PUT /entries/{id}/stop-after` | `{"on": bool}` | `SetEntryStopAfter` if it differs |
| `POST /carts/{id}/fire` | — | `FireCart` (as the cart button: fires, or stops a playing cart) |
| `POST /carts/{id}/stop` | — | `StopCart` |
| `PUT /carts/{id}/cue` | `{"on": bool}` | `SetCartCue` if it differs |
| `POST /cartwall/stop-all` | — | `StopAllCarts` |
| `PUT /cartwall/shown` | `{"page": id}` | `ShowCartPage` |

`fader` outside `0.0–1.0` and `secs` outside the track's cue range are `400`;
the UI's clamping rules are not repeated here.

### 3.4 Editing

| Method and path | Body | Command(s) |
|---|---|---|
| `POST /playlists` | `{"name"}` | `CreatePlaylist` |
| `PATCH /playlists/{id}` | `{"name"}` | `RenamePlaylist` |
| `DELETE /playlists/{id}` | — | `DeletePlaylist` |
| `POST /playlists/{id}/entries` | `{"track": id, "index": n}` | `InsertPaths` with the track's path |
| `DELETE /entries/{id}` | — | `RemoveEntry` |
| `POST /entries/{id}/move` | `{"playlist": id, "index": n}` | `MoveEntry` |
| `POST /entries/{id}/duplicate` | — | `DuplicateEntry` |
| `POST /cartwall/pages` | `{"name"}` | `CreateCartPage` |
| `PATCH /cartwall/pages/{id}` | `{"name"?, "rows"?, "cols"?}` | `RenameCartPage`, `ResizeCartPage` |
| `DELETE /cartwall/pages/{id}` | — | `DeleteCartPage` |
| `PUT /cartwall/pages/{id}/carts/{index}` | `{name, kind, looped, exclusive, track: id \| null}` | `SetCart`, then `AssignCartFile` with the track's path or `ClearCartFile` |
| `PUT /tracks/{id}/markers/{kind}` | `{"secs": f64 \| null}` | `SetMarker` |
| `POST /tracks/{id}/markers/reset` | — | `ResetMarkers` |

Marker kinds in paths are `cue-in`, `intro-end`, `outro-start`,
`segue-start`, `cue-out`. Rows and columns are validated against
`limits.max_cart_rows` and `limits.max_cart_cols`; an index past the playlist's
end is clamped to the end, as a drop in the UI is.

### 3.5 Responses and errors

- `202 Accepted` with `{"revision": n}` (the revision seen when the command was
  queued): every command was queued. Before queuing, the commands are applied
  to a copy of the current snapshot (a dry run): a refusal there is answered
  `404` (an unknown id) or `409` with the model's reason, and nothing is
  queued.
- `200 OK`: reads.
- Errors carry `{"error": code, "message": text}`:

| Status | `error` | When |
|---|---|---|
| 400 | `bad_request` | Malformed JSON, a value out of range |
| 401 | `unauthorized` | Token missing or wrong (§6) |
| 403 | `forbidden_origin` | An `Origin` not in `cors_origins`, a `Host` that is not the server (§6) |
| 404 | `not_found` | Unknown id, path or cover |
| 404 | `not_analyzed` | Peaks or cover of a track not analysed yet |
| 409 | `unavailable` | R28 or a model rule forbids it now (a greyed-out button) |
| 408 | (empty body) | The request took longer than `request_timeout_ms` |
| 413 | `payload_too_large` | Body over `max_body_bytes` |
| 415 | `unsupported_media_type` | A body that is not `application/json` |
| 503 | `busy` | The controller queue is full, or too many event clients |

---

## 4. OSC

OSC 1.0 over UDP, decoded and encoded with `rosc`. All addresses live under
`/fauste`. `n` is a 1-based player position, `c` a 1-based cart position in the
page shown, `p` a 1-based page position.

### 4.1 Input

| Address | Arguments | Effect |
|---|---|---|
| `/fauste/player/{n}/play` (also `pause`, `stop`, `fade-stop`, `restart`, `previous`) | none, or one number | Acts with no argument or a number > 0, so a surface that sends 1 on press and 0 on release fires once |
| `/fauste/player/{n}/cue` | `i`/`f`/`T`/`F` | Cue on (non-zero, true) or off |
| `/fauste/player/{n}/volume` | `f` (0–1) | Fader travel |
| `/fauste/cart/{c}/fire` | as `play` | Fire the cart of the page shown |
| `/fauste/cartwall/page/{p}/cart/{c}/fire` | as `play` | Fire a cart of a given page |
| `/fauste/cartwall/stop-all` | as `play` | Stop every cart |
| `/fauste/cartwall/page/next`, `/fauste/cartwall/page/previous` | as `play` | Change the page shown |
| `/fauste/subscribe` | none, or `i` port | Subscribe (§5.3) |
| `/fauste/unsubscribe` | none, or `i` port | Unsubscribe |

Bundles are accepted and their messages applied in order (time tags are
ignored: they act on arrival). Malformed packets, unknown addresses and
unavailable actions are dropped with a rate-limited log line; OSC has no
replies.

### 4.2 Output (to subscribers)

| Address | Type |
|---|---|
| `/fauste/player/{n}/transport` | `s` (`stopped` / `playing` / `paused`) |
| `/fauste/player/{n}/fading`, `/cueing`, `/stop-after-current` | `i` (0/1) |
| `/fauste/player/{n}/volume` | `f` |
| `/fauste/player/{n}/title`, `/artist` | `s` |
| `/fauste/player/{n}/elapsed`, `/remaining` | `f` (seconds) |
| `/fauste/player/{n}/next/entry` | `h` (int64 entry id, -1 for none: ids are `u64` and may pass `i32`) |
| `/fauste/player/{n}/next/title`, `/next/artist` | `s` |
| `/fauste/cart/{c}/playing` | `i` (0/1), for the page shown |
| `/fauste/cart/{c}/name` | `s`, for the page shown |
| `/fauste/cartwall/page` | `i` (1-based page shown) |

---

## 5. Events

### 5.1 Publisher

- A task in the remote thread checks the model snapshot every 50 ms
  (`Arc::ptr_eq`). On a new snapshot it increments `revision` and diffs the old
  and new snapshots into resource events, each carrying the whole resource
  (not a patch):
  - `player` (one player changed);
  - `playlist` (name or entries changed), `playlist-removed {id}`;
  - `cartwall` (pages, carts, page shown, carts playing, cart cue);
  - `track` (metadata or markers of a track in some playlist or cart changed,
    for example when analysis completes).
- Every `events.position_interval_ms`, and only while a player or cart plays,
  it publishes `position`: elapsed and remaining for every running player and
  cart.
- Events go to a bounded `tokio::sync::broadcast` (256). The diff is a pure
  function, tested without a network.

### 5.2 SSE (`GET /events`)

- On connect the stream starts with a `state` event (the same body as
  `GET /state`), then resource events. Each event has `event: <type>`,
  `id: <revision>` and `data: <json>`.
- **Reconnection.** The server keeps no history: every connection, with or
  without `Last-Event-ID`, starts with a full `state`. This is the contract.
- **Slow clients.** A client that falls behind the broadcast (`Lagged`) gets a
  `resync` event with a full state and carries on. The publisher never waits.
- **Keep-alive.** A `: keepalive` comment every 15 s.
- **Topics.** `?topics=player,position` limits the stream to those types;
  `state` and `resync` are always sent.
- More than `max_event_clients` streams: `503 busy`.

### 5.3 OSC subscriptions

- `/fauste/subscribe [port]` subscribes `source_ip:port` (the packet's source
  port when none is given) and sends it a full dump of §4.2.
- Repeating it before `subscription_ttl_secs` renews it; otherwise it expires.
  `/fauste/unsubscribe` removes it at once. Subscriptions are only accepted from
  `allowed_sources`; past `max_subscribers` one is refused with a log line.
- The publisher keeps the last value sent per subscriber and address, and sends
  a message only when it changes. `elapsed` and `remaining` follow
  `position_interval_ms`. A change in the player count or the page shown
  sends a full dump.
- A failed UDP send is logged (rate-limited) and affects nothing else.

---

## 6. Configuration and security

### 6.1 `config.remote`

Loaded leniently (an invalid field falls back to its default with a
`ConfigWarning`) and validated in `Config::validate`.

| Field | Default | Range / rule |
|---|---|---|
| `http.enabled` | `false` | |
| `http.bind` | `"127.0.0.1"` | An IPv4 or IPv6 literal |
| `http.port` | `7380` | 1024–65535 |
| `http.token` | `""` | Empty, or at least 16 characters |
| `http.cors_origins` | `[]` | Origins (`scheme://host[:port]`); `"*"` only with a token (without one, any web page could drive the station) |
| `http.max_event_clients` | `16` | 1–256 |
| `http.request_timeout_ms` | `10000` | 1000–120000 |
| `http.max_body_bytes` | `65536` | 1024–1048576 |
| `osc.enabled` | `false` | |
| `osc.bind` | `"127.0.0.1"` | An IPv4 or IPv6 literal |
| `osc.port` | `7381` | 1024–65535, not HTTP's port on the same bind |
| `osc.allowed_sources` | `["127.0.0.1/32", "::1/128"]` | IPs or CIDR subnets |
| `osc.max_subscribers` | `16` | 1–256 |
| `osc.subscription_ttl_secs` | `60` | 5–3600 |
| `events.position_interval_ms` | `250` | 50–5000 |

### 6.2 HTTP security

- **Token.** When `bind` is not loopback the token is mandatory: without one the
  HTTP server does not start, logs an error and reports it in Settings. With a
  token, every request needs `Authorization: Bearer <token>`; only
  `GET /events` also accepts `?token=` (browsers' `EventSource` cannot set
  headers). Comparison is constant-time. Query strings are never logged.
  A loopback bind with an empty token needs no authentication.
- **Browser attacks on a local server.** A web page in the operator's browser
  can reach `127.0.0.1`. Therefore, whatever the bind:
  - a request with an `Origin` header not in `cors_origins` is `403`;
  - mutating requests with a body require `Content-Type: application/json`,
    which forces a CORS preflight;
  - with a loopback bind, the `Host` header must name a loopback address or
    `localhost` (DNS rebinding).
- **Limits.** `tower-http` provides the request timeout, the body limit and
  CORS; a concurrency limit bounds in-flight requests.
- **Logging.** Rejected requests log one line per source IP per second at
  most.

### 6.3 OSC security

OSC has no authentication. Packets from sources outside `allowed_sources` are
dropped (rate-limited log). Listening beyond loopback is the operator's
decision; the user guide says that OSC belongs on a trusted studio network.

### 6.4 Known limit

`202` means queued. The dry run (§3.5) catches every refusal the snapshot can
predict; a command can still be refused if the state changes between the
snapshot and the conductor applying it (another operator acted in between).
That refusal reaches the UI notice, and the client sees the outcome in the
next event. Correlating commands with their verdict would change the
conductor and is out of scope for v1.

---

## 7. Architecture

```
fp-app ──(Bridge: impl RemoteControl)──► fp-remote
                                          ├─ thread "remote": tokio current_thread
                                          │   ├─ axum: HTTP routes and SSE
                                          │   ├─ OSC: UdpSocket + rosc
                                          │   └─ publisher: snapshot diff → broadcast
                                          └─ api, events (pure): request + &AppState → Vec<Command> | ApiError
```

- **Crate `fp-remote`.** Depends on `fp-model`, `tokio`, `axum`, `tower`,
  `tower-http`, `rosc`, `serde`, `serde_json`, `getrandom` and `tracing`; all
  MIT or Apache-2.0. `cargo deny check` runs after adding them. It does not
  depend on `fp-engine` or `fp-analysis`; what it needs from them comes
  through a trait implemented in `fp-app`:

  ```rust
  pub trait RemoteControl: Send + Sync + 'static {
      fn model(&self) -> Arc<AppState>;
      /// Elapsed and remaining for players and carts (engine telemetry).
      fn playback(&self) -> Playback;
      /// Queues a command; never blocks.
      fn send(&self, command: Command) -> bool;
      /// Blocking (analysis cache on disk): called in `spawn_blocking`.
      fn cover(&self, track: TrackId) -> Option<Vec<u8>>;
      fn peaks(&self, track: TrackId) -> Option<WaveformData>;
  }
  ```

- **Lifecycle.** `fp_remote::spawn(control, config) -> RemoteHandle` starts one
  thread with a `current_thread` runtime. It is neither an audio nor a UI
  thread (rules 5 and 8 hold). The thread follows `config.remote` in the model
  snapshot (checked every 250 ms) and restarts only the server whose settings
  changed, so Settings needs no extra wiring. `RemoteHandle::status()`
  reads each server's state (`disabled`, `listening(addr)`, `error(text)`)
  without blocking (`ArcSwap`). On exit: cancellation, SSE streams closed,
  join with a timeout. If the thread dies the error is logged and shown, and
  the application carries on, as with MIDI.
- **Bind failures** (port in use, permission) are logged, shown in Settings,
  and the application runs without that server.
- **Token generation.** Settings offers "Generate": 32 random bytes from
  `getrandom`, base64url without padding.

---

## 8. Settings > Remote

- Two groups, HTTP and OSC, each with an enable switch, bind and port, and a
  status line ("Listening on …", "Error: …", "Off").
- HTTP: the token (masked, with show, copy and "Generate"), the CORS origins.
  OSC: the allowed sources.
- Events: the position interval.
- Advanced limits (event clients, timeouts, body size, subscribers, TTL) are
  in `config.json` only, documented in the user guide.
- Saving applies at once (the remote thread follows the model's
  configuration); no restart.
- All strings are in both locales.

---

## 9. Testing

TDD throughout; no test needs a real network interface or audio device.

- **`api` (pure):** one test per route: the commands produced, `404`, `409`
  (R28), desired-value toggles (no command when already in that state), fader
  mapping, index clamping, track-to-path resolution.
- **`events` (pure):** snapshot diff → expected events; `position` only while
  something plays; `track` when analysis completes.
- **HTTP:** the axum router driven with `tower::ServiceExt::oneshot`, no
  sockets: authentication (loopback without token, token required beyond
  loopback, `?token=` only on `/events`), `Origin` and `Host` checks, CORS
  preflight, body limit, content type, every error code, SSE initial `state`,
  events and `resync` on lag.
- **OSC:** UDP sockets on `127.0.0.1:0`: commands, rejected sources, press and
  release, bundles, subscribe and full dump, change-only sending, expiry with
  `tokio::time::pause`.
- **Config:** defaults, ranges, lenient loading, the token rule, port clash.
- **UI:** Settings > Remote with `egui_kittest` and the `Fake` controller.
- **Integration (`fp-app`):** with the `Offline` backend, `POST
  /players/{id}/play` puts the player in `playing`, and an SSE client sees it.

---

## 10. Plans

1. **Foundation, reading and operating.** `config.remote` (HTTP part and
   events), crate `fp-remote`, the bridge in `fp-app`, security (§6.2), §3.1–3.3
   and §3.5 without SSE.
2. **Events and OSC.** Publisher, SSE, OSC input, output and subscriptions,
   `config.remote.osc`.
3. **Editing, Settings and documentation.** §3.4, Settings > Remote, the user
   guide page, the technical API reference, the README (feature, roadmap and a
   new screenshot with a player on air), `CLAUDE.md`.

Each plan reaches `master` through its own pull request.

## 11. Documentation

- `README.md`: the feature, the roadmap and the screenshot.
- `docs/user/remote-control.md`: enabling it, security, examples with `curl`
  and `oscsend`.
- `docs/technical/remote-api.md`: the full reference (routes, DTOs, events, OSC
  addresses, error codes).
- The main spec: the non-goal becomes "Network features, streaming and
  telemetry"; remote control is specified here.
- `CLAUDE.md`: the crate in "Where things are"; in "Testing notes", that
  screenshots of the running app need `xdotool` (to find and click the window)
  and that the remote API can drive it with `curl`.
