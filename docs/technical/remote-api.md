# Remote control API

The HTTP/JSON API lets other programs read and operate Fauste Player: a
web page, a phone app, a station's automation. The design is the
[remote control spec](../superpowers/specs/2026-10-01-remote-control-design.md).
This page describes what is implemented. The event stream (SSE), OSC,
editing routes and the Settings page are not implemented yet (spec §10,
plans 2 and 3).

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
  `202`, so a retried request never undoes itself.
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

## Errors

Errors carry `{"error": code, "message": text}`.

| Status | `error` | When |
|---|---|---|
| 400 | `bad_request` | Malformed JSON, a body that is not an object, a value out of range |
| 401 | `unauthorized` | Token missing or wrong |
| 403 | `forbidden_origin` | An `Origin` not in `cors_origins`, or a `Host` that is not loopback on a loopback bind |
| 404 | `not_found` | Unknown id, path or cover |
| 404 | `not_analyzed` | Cover or peaks of a track not analysed yet |
| 405 | (empty) | A route that exists with another method |
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
| `http.cors_origins` | `[]` | `http://` or `https://` origins with no path; `"*"` only with a loopback bind and no token |
| `http.max_event_clients` | `16` | 1–256 (for the event stream, plan 2) |
| `http.request_timeout_ms` | `10000` | 1000–120000 |
| `http.max_body_bytes` | `65536` | 1024–1048576 |
| `events.position_interval_ms` | `250` | 50–5000 (for the event stream, plan 2) |

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
  - `server`: the `fp-remote` thread.
- `fp_remote::spawn` starts one thread with a tokio `current_thread`
  runtime. Every 250 ms the thread compares `config.remote.http` in the
  model snapshot with the configuration it is running, and restarts the
  server when they differ. A change from Settings therefore applies with no
  restart. The status (`Off`, `Listening(addr)`, `Error(…)`) is published
  through `ArcSwap` and read with `RemoteHandle::status`. Dropping the
  handle stops the server (open requests get two seconds) and joins the
  thread.
- `fp-app::remote::Bridge` implements `RemoteControl` over the
  `ConductorHandle` (snapshot, telemetry, non-blocking `send`), the media
  cache and the analysis cache. Covers and peaks are read in
  `spawn_blocking`.
- Tests: the pure modules directly; the router through
  `tower::ServiceExt::oneshot` with the recording `FakeControl`; the server
  on free loopback ports; an `fp-app` integration test that plays through
  the real conductor.

## Examples

```sh
curl -s http://127.0.0.1:7380/api/v1/state
curl -s -X POST http://127.0.0.1:7380/api/v1/players/12/play
curl -s -X PUT -H 'Content-Type: application/json' \
     -d '{"fader": 0.8}' http://127.0.0.1:7380/api/v1/players/12/volume
curl -s -H "Authorization: Bearer $TOKEN" http://studio-pc:7380/api/v1/players
```
