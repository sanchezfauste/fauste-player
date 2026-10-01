# Remote control API

The HTTP/JSON API lets other programs read and operate Fauste Player: a
web page, a phone app, a station's automation. The design is the
[remote control spec](../superpowers/specs/2026-10-01-remote-control-design.md).
This page describes what is implemented: reading, operating and editing
over HTTP, live events over Server-Sent Events, OSC, and Settings → Remote.

## Overview

- Base path `/api/v1`. Bodies and answers are JSON (`application/json`,
  UTF-8). Request bodies must be JSON **objects**.
- Resources are addressed by the model's ids, `u64` numbers that persist
  across restarts. A path id that is not a number names nothing: `404`.
- Requests that act answer `202 Accepted` with `{"revision": n}`.
  `revision` is the conductor's model version, which rises with every model
  change. The command is queued, and its effect shows in the next reads.
- **Same rules as the console.** Every request becomes the commands the
  interface would send. A transport action whose button is greyed out
  (availability, R28) is refused with `409`. The commands are then applied to
  a copy of the current snapshot (a dry run): a refusal there is answered
  `404` (an unknown id) or `409` with the model's reason, and nothing is
  queued. A refusal is still possible if another operator changes the state
  between the snapshot and the conductor applying the command. The interface
  then shows its usual notice.
- **Desired values.** On/off settings take the value wanted (`{"on": true}`).
  If it is already in that state, nothing is sent and the answer is still
  `202`. Otherwise an idempotent model command (`SetCue`, …) is sent. It is
  resolved when the conductor applies it, so a retried request, or two
  clients asking for the same value at once, never undo each other.
- **Volume** is fader travel `0.0`–`1.0`, with the on-screen fader's curve.

## Resources

**Player**

| Field | Meaning |
|---|---|
| `id` | Player id |
| `position` | 1-based, in display order |
| `transport` | `stopped`, `playing` or `paused` |
| `fading`, `stop_after_current` | Booleans |
| `mode` | `single` or `continuous` |
| `fader` | Fader travel 0–1 |
| `playlist` | Id of the playlist shown in the player |
| `current`, `next` | `null` or `{entry, track}` |
| `cue` | `null` or `{entry}` (pre-listening) |
| `elapsed_secs`, `remaining_secs` | As the countdown computes them: the position in track seconds, and cue-out minus it. `null` without a current entry |

**Playlist summary:** `id`, `name`, `entry_count`. **Playlist:** the
summary plus `entries`, each with `id`, `track`, `repeat`, `stop_after`,
`on_air` (ids of players playing it), `next_on` (players that will play it
next) and `cued_on` (players pre-listening it).

**Track**

| Field | Meaning |
|---|---|
| `id`, `title`, `artist`, `album`, `duration_secs` | |
| `kind` | `music`, `jingle`, `effect`, `ad` or `voice` |
| `file_state` | `ok`, `missing` or `unreadable` |
| `analyzed` | Whether background analysis has finished |
| `format` | `null` or `{sample_rate, channels, bits}` |
| `markers` | `cue_in`, `intro_end`, `outro_start`, `segue_start`, `cue_out`, each `null` or `{secs, source}` with `source` `auto` or `manual` |

The file path is never exposed.

**Cartwall:** `shown_page`, `pages` (each `id`, `name`, `rows`, `cols`,
`carts`), `playing` (in firing order: `cart`, `elapsed_secs`,
`remaining_secs`) and `cue` (`null` or a cart id). **Cart:** `id`, `index`
(row-major, 0-based in its page), `name`, `kind` (`jingle`, `effect` or
`spot`), `looped`, `exclusive`, `track` (`null` or a Track).

**State:** `revision`, `players`, `playlists` (summaries) and `cartwall`.

## Routes

### Reading

| Method and path | Returns |
|---|---|
| `GET /state` | State |
| `GET /players` | Players, in display order |
| `GET /players/{id}` | Player |
| `GET /playlists` | Playlist summaries |
| `GET /playlists/{id}` | Playlist |
| `GET /tracks/{id}` | Track |
| `GET /tracks/{id}/cover` | The cover thumbnail, `image/png` |
| `GET /tracks/{id}/peaks` | `{bucket_secs, full_scale: 32767, peaks: [[min, max, rms], …]}` |
| `GET /cartwall` | Cartwall |
| `GET /events` | The event stream (see [Events](#events-sse)) |

The cover and peaks come from the interface's media cache, or else from the
analysis cache on disk. Neither ever starts an analysis. A track not
analysed yet answers `404 not_analyzed`. An analysed track without a cover,
or whose cache entry is gone, answers `404 not_found`.

### Operating

| Method and path | Body | Effect |
|---|---|---|
| `POST /players/{id}/play` | — | Play/Next |
| `POST /players/{id}/pause` | — | Pause / resume |
| `POST /players/{id}/stop` | — | Stop |
| `POST /players/{id}/fade-stop` | — | Fade stop |
| `POST /players/{id}/restart` | — | Back to the current entry's cue-in (R23) |
| `POST /players/{id}/previous` | — | Back to the previous entry (R24) |
| `PUT /players/{id}/cue` | `{"on": bool}` | Pre-listen the next entry, or stop pre-listening |
| `PUT /players/{id}/next` | `{"entry": id}` | Choose the next entry |
| `POST /players/{id}/cue-entry` | `{"entry": id}` | Pre-listen an entry |
| `POST /players/{id}/seek` | `{"secs": f64}` | Seek within the cue range of what is playing |
| `PUT /players/{id}/volume` | `{"fader": f32}` | Fader travel 0–1 |
| `PUT /players/{id}/mode` | `{"mode": "single" \| "continuous"}` | |
| `PUT /players/{id}/stop-after-current` | `{"on": bool}` | |
| `PUT /players/{id}/playlist` | `{"playlist": id}` | Show that playlist in the player |
| `PUT /entries/{id}/repeat` | `{"on": bool}` | R26 |
| `PUT /entries/{id}/stop-after` | `{"on": bool}` | R27 |
| `POST /carts/{id}/fire` | — | As the cart button: fires it, or stops it if it plays |
| `POST /carts/{id}/stop` | — | |
| `PUT /carts/{id}/cue` | `{"on": bool}` | Pre-listen a cart |
| `POST /cartwall/stop-all` | — | |
| `PUT /cartwall/shown` | `{"page": id}` | Show a cart page |

### Editing

Editing reuses tracks already loaded: an inserted entry or a cart refers
to the same library track, with its markers and analysis
(`InsertTracks`, `AssignCartTrack`). Files cannot be added remotely.

| Method and path | Body | Effect |
|---|---|---|
| `POST /playlists` | `{"name"}` | Create a playlist |
| `PATCH /playlists/{id}` | `{"name"}` | Rename it |
| `DELETE /playlists/{id}` | — | Delete it (refused while it is on air) |
| `POST /playlists/{id}/entries` | `{"track": id, "index": n}` | Insert a loaded track; an index past the end is the end |
| `DELETE /entries/{id}` | — | Remove an entry (refused while on air) |
| `POST /entries/{id}/move` | `{"playlist": id, "index": n}` | Move it, also to another playlist (allowed while on air: only removal is refused, rule 13) |
| `POST /entries/{id}/duplicate` | — | Duplicate it |
| `POST /cartwall/pages` | `{"name"}` | Create a cart page |
| `PATCH /cartwall/pages/{id}` | `{"name"?, "rows"?, "cols"?}` | Rename or resize it, within `limits.max_cart_rows` and `limits.max_cart_cols` |
| `DELETE /cartwall/pages/{id}` | — | Delete it |
| `PUT /cartwall/pages/{id}/carts/{index}` | `{name, kind, looped, exclusive, track: id \| null}` | Set up a cart; `kind` is `jingle`, `effect` or `spot`. Keeping its track does not stop it; a new track or `null` does (C8) |
| `PUT /tracks/{id}/markers/{kind}` | `{"secs": f64 \| null}` | Set a marker by hand, or with `null` let analysis fill it again; `kind` is `cue-in`, `intro-end`, `outro-start`, `segue-start` or `cue-out` |
| `POST /tracks/{id}/markers/reset` | — | Drop every manual marker and analyse again |

Names are trimmed and must not be empty.

## Events (SSE)

`GET /api/v1/events` is a Server-Sent Events stream
(`text/event-stream`). Each event has `event: <type>`, `id: <revision>` and
one line of JSON `data`.

| Event | Data |
|---|---|
| `state` | The whole State, always first |
| `player` | A Player whose state, entries or settings changed |
| `playlist` | A Playlist whose name or entries changed (or a new one) |
| `playlist-removed` | `{id}` |
| `cartwall` | The Cartwall, when pages, carts, the page shown, carts playing or the cart cue changed |
| `track` | A Track of some playlist or cart whose metadata or markers changed (for example when analysis finishes) |
| `position` | `{players: [{id, elapsed_secs, remaining_secs}], carts: [{cart, elapsed_secs, remaining_secs}]}`, every `events.position_interval_ms` while a player or cart plays |
| `resync` | The whole State again, after the client fell behind |

- Events carry whole resources, not patches. A moving position alone is not
  a `player` change; `position` reports it.
- A change in the number of players sends a `state`.
- **Reconnection.** The server keeps no history: every connection, with or
  without `Last-Event-ID`, starts with a full `state`.
- **Slow clients.** The publisher never waits. A client more than 256
  events behind gets `resync` and carries on.
- **Keep-alive.** A `:keepalive` comment every 15 s.
- **Topics.** `?topics=player,position` limits the stream to those types;
  `state` and `resync` are always sent.
- **Token.** Browsers' `EventSource` cannot set headers, so this route also
  accepts `?token=<token>`. No other route does.
- More than `http.max_event_clients` open streams answer `503 busy`.

```sh
curl -sN 'http://127.0.0.1:7380/api/v1/events?topics=player,position'
```

```js
const events = new EventSource('http://studio-pc:7380/api/v1/events?token=' + token);
events.addEventListener('player', (e) => update(JSON.parse(e.data)));
```

## OSC

OSC 1.0 over UDP, on `osc.bind`:`osc.port` (default `127.0.0.1:7381`).
Packets are accepted only from `osc.allowed_sources`. OSC has no
authentication, so keep it on a trusted network. Players and carts are
numbered from 1 in screen order: `n` is a player's position, `c` a cart of
the page shown, `p` a page.

**Input**

| Address | Arguments | Effect |
|---|---|---|
| `/fauste/player/{n}/play` (also `pause`, `stop`, `fade-stop`, `restart`, `previous`) | none, or a number | Acts with no argument or a number above 0, so a surface that sends 1 on press and 0 on release acts once |
| `/fauste/player/{n}/cue` | a number or boolean | Cue on (above 0, true) or off |
| `/fauste/player/{n}/volume` | `f` 0–1 | Fader travel |
| `/fauste/cart/{c}/fire` | as `play` | Fire a cart of the page shown |
| `/fauste/cartwall/page/{p}/cart/{c}/fire` | as `play` | Fire a cart of a given page |
| `/fauste/cartwall/stop-all` | as `play` | Stop every cart |
| `/fauste/cartwall/page/next`, `/fauste/cartwall/page/previous` | as `play` | Change the page shown |
| `/fauste/subscribe` | none, or `i` port | Subscribe this address (on that port, or the packet's source port) |
| `/fauste/unsubscribe` | none, or `i` port | Unsubscribe |

Bundles are accepted. Their messages act in order, on arrival (time tags
are ignored). An unavailable action, an unknown address or a malformed
packet is dropped with a log line, at most one per source per second.

**Output to subscribers**

| Address | Type |
|---|---|
| `/fauste/player/{n}/transport` | `s`: `stopped`, `playing` or `paused` |
| `/fauste/player/{n}/fading`, `/cueing`, `/stop-after-current` | `i` 0/1 |
| `/fauste/player/{n}/volume` | `f` fader travel |
| `/fauste/player/{n}/title`, `/artist` | `s` (empty without a current entry) |
| `/fauste/player/{n}/elapsed`, `/remaining` | `f` seconds |
| `/fauste/player/{n}/next/entry` | `h` (int64) entry id, -1 for none |
| `/fauste/player/{n}/next/title`, `/next/artist` | `s` |
| `/fauste/cart/{c}/playing` | `i` 0/1, for the page shown |
| `/fauste/cart/{c}/name` | `s`, for the page shown |
| `/fauste/cartwall/page` | `i` 1-based page shown |

A new subscriber gets every address once, then only values that change.
Times follow `events.position_interval_ms`. A change in the player count or
the page shown sends everything again. A subscription ends after
`osc.subscription_ttl_secs` unless the client subscribes again, and at most
`osc.max_subscribers` are kept.

## Errors

Errors carry `{"error": code, "message": text}`.

| Status | `error` | When |
|---|---|---|
| 400 | `bad_request` | Malformed JSON, a body that is not an object, a value out of range |
| 401 | `unauthorized` | Token missing or wrong |
| 403 | `forbidden_origin` | An `Origin` not in `cors_origins`, or a `Host` that is not loopback on a loopback bind |
| 404 | `not_found` | Unknown id, path or cover |
| 404 | `not_analyzed` | Cover or peaks of a track not analysed yet |
| 405 | `method_not_allowed` | A route that exists with another method |
| 408 | (empty) | The request took longer than `request_timeout_ms` |
| 409 | `unavailable` | The action is not available now, or the model refuses it |
| 413 | `payload_too_large` | Body over `max_body_bytes` |
| 415 | `unsupported_media_type` | A body that is not `application/json` |
| 503 | `busy` | The command queue is full |

## Security

- **Token.** When `bind` is not a loopback address, a token is mandatory.
  Without one the server does not start, and the error is logged and shown
  in the status. With a token, every request needs
  `Authorization: Bearer <token>`. The comparison does not stop at the first
  differing byte. With a loopback bind and no token, no authentication is
  asked for.
- **Browsers.** A page open in the operator's browser can reach
  `127.0.0.1`. Therefore, whatever the bind:
  - a request whose `Origin` is not in `cors_origins` is refused with `403`;
  - bodies must be `application/json`, which makes browsers send a CORS
    preflight first;
  - on a loopback bind, the `Host` header must be `localhost` or a
    loopback address (against DNS rebinding).
- **CORS.** Preflights from listed origins are answered for `GET`, `POST`,
  `PUT`, `PATCH` and `DELETE`, with the `Authorization` and `Content-Type`
  headers.
- **Limits.** Request timeout, body size, and 64 requests in flight (the
  rest wait their turn).
- **Logging.** A refused request is logged at most once per second per
  source address.
- There is no TLS. Beyond a trusted network, a reverse proxy terminates
  HTTPS.

## Configuration

`config.remote`, loaded leniently and validated with the rest of the
configuration (see [Persistence and configuration](persistence.md)):

| Field | Default | Range / rule |
|---|---|---|
| `http.enabled` | `false` | |
| `http.bind` | `"127.0.0.1"` | An IPv4 or IPv6 literal, else `127.0.0.1` with a warning |
| `http.port` | `7380` | 1024–65535 |
| `http.token` | `""` | Empty, or at least 16 characters (a shorter one is dropped with a warning) |
| `http.cors_origins` | `[]` | `http://` or `https://` origins with no path; `"*"` only with a token |
| `http.max_event_clients` | `16` | 1–256 |
| `http.request_timeout_ms` | `10000` | 1000–120000 |
| `http.max_body_bytes` | `65536` | 1024–1048576 |
| `osc.enabled` | `false` | |
| `osc.bind` | `"127.0.0.1"` | An IPv4 or IPv6 literal, else `127.0.0.1` with a warning |
| `osc.port` | `7381` | 1024–65535; moved off the HTTP port on the same bind |
| `osc.allowed_sources` | `["127.0.0.1/32", "::1/128"]` | IP addresses or CIDR subnets; invalid ones dropped with a warning. IPv4 seen through a dual-stack socket counts as IPv4 |
| `osc.max_subscribers` | `16` | 1–256 |
| `osc.subscription_ttl_secs` | `60` | 5–3600 |
| `events.position_interval_ms` | `250` | 50–5000 |

## Implementation

- Crate `fp-remote`, whose only workspace dependency is `fp-model`:
  - `control`: the `RemoteControl` trait (`model`, `playback`, `send`,
    `cover`, `peaks`), `Playback` and `WaveformData`;
  - `dto`: the JSON contract, built from a snapshot and the playback
    telemetry (pure);
  - `api`: `Operation` → `Vec<Command>`, with availability, desired values
    and the dry run (pure);
  - `http`: the axum router, its handlers, the JSON body extractor, the
    guard (token, `Origin`, `Host`), and the CORS, body-limit, timeout and
    concurrency layers;
  - `events`: the snapshot diff into resource events, and the position
    event (pure);
  - `http/events`: the SSE stream;
  - `osc`: OSC addresses into operations, the values table, and the
    subscribers with what each was last sent (pure);
  - `osc_server`: the UDP loop;
  - `throttle`: one log line per source per second;
  - `server`: the `fp-remote` thread.
- `fp_remote::spawn` starts one thread with a tokio `current_thread`
  runtime. Every 250 ms the thread compares `config.remote.http` and
  `config.remote.osc` in the model snapshot with what it is running, and
  restarts the server whose settings changed. A change from Settings
  therefore applies with no restart. The status of each server (`Off`,
  `Listening(addr)`, `Error(…)`) is published through `ArcSwap` and read
  with `RemoteHandle::status`.
- A publisher task looks for a new snapshot every 50 ms. It diffs it into
  events on a `broadcast` channel of 256, and every `position_interval_ms`
  it adds a `position` while something plays. It does no work while nobody
  listens. SSE streams and the OSC socket read that channel.
- A server whose address could not be bound (in use, no permission) is
  tried again every 2 s, so a port freed later is taken without a restart.
- Settings → Remote (`ui/settings/remote.rs`) edits `config.remote` through
  `UpdateConfig`, and reads each server's state from
  `RemoteHandle::status_cell`, an `ArcSwap` the remote thread publishes.
  It generates tokens with `getrandom` (32 bytes, base64url).
- Stopping a server ends its event streams, gives open requests two
  seconds, then aborts it. Dropping the handle stops both servers and joins
  the thread.
- `fp-app::remote::Bridge` implements `RemoteControl` over the
  `ConductorHandle` (snapshot, telemetry, non-blocking `send`), the media
  cache and the analysis cache. Covers and peaks are read in
  `spawn_blocking`.
- Tests: the pure modules directly; the router through
  `tower::ServiceExt::oneshot` with the recording `FakeControl`; the server
  on free loopback ports; SSE through `oneshot` and over TCP; OSC with UDP
  sockets on `127.0.0.1:0`; an `fp-app` integration test that plays through
  the real conductor.

## Examples

```sh
curl -s http://127.0.0.1:7380/api/v1/state
curl -s -X POST http://127.0.0.1:7380/api/v1/players/12/play
curl -s -X PUT -H 'Content-Type: application/json' \
     -d '{"fader": 0.8}' http://127.0.0.1:7380/api/v1/players/12/volume
curl -s -H "Authorization: Bearer $TOKEN" http://studio-pc:7380/api/v1/players
```
