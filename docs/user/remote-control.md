# Remote control

Fauste Player can be read and operated over the network through an HTTP
API. A web page, a phone app or a station's automation can use it. It is
**off** until you turn it on, and at first it only answers on this computer.

## Turning it on

Settings has no page for it yet. Close Fauste Player, open `config.json`
(see [Data and backups](data-and-backups.md) for where it is), and inside
the `"config"` object set `remote.http.enabled` to `true`:

    {
      "schema_version": 1,
      "config": {
        …
        "remote": {
          "http": { "enabled": true }
        }
      }
    }

On the next start it listens on `http://127.0.0.1:7380`. Starting it never
plays anything; only requests act.

## Listening on the studio network

To reach it from other computers, set `bind` to `0.0.0.0` (or one of this
computer's addresses) and set a **token** of at least 16 characters.
Without a token the server refuses to start, and the log says why.

    "remote": {
      "http": {
        "enabled": true,
        "bind": "0.0.0.0",
        "port": 7380,
        "token": "a-long-random-secret-of-your-own"
      }
    }

Clients send the token as `Authorization: Bearer <token>`. The API is not
encrypted. Keep it on a trusted studio network, or put it behind a reverse
proxy with HTTPS.

## Web pages

A web page served from another address can use the API only if its origin
(for example `https://studio.example`) is listed in `cors_origins`. Requests
from other pages are refused, even on this computer, so a page you happen
to have open cannot drive the player.

## What a client can do

A client can:

- read the players, playlists, tracks (with cover and waveform) and the
  cartwall;
- play, pause, stop, fade, restart and go back;
- choose the next entry, pre-listen, and seek;
- set volumes, modes, stop-after-current, and the repeat and stop-after
  marks of an entry;
- fire, stop and pre-listen carts, and change the cart page shown.

A button that is greyed out on screen is refused remotely too. The full
reference is in [the technical documentation](../technical/remote-api.md).

Try it from a terminal:

    curl -s http://127.0.0.1:7380/api/v1/players
    curl -s -X POST http://127.0.0.1:7380/api/v1/players/<id>/play

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
| `remote.http.max_event_clients` | `16` | Live event streams at once (coming) |
| `remote.events.position_interval_ms` | `250` | How often times are published while playing (coming) |

Values out of range are corrected when the file is loaded, and the
correction is logged.
