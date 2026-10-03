# Remote control

Fauste Player can be read and operated over the network through an HTTP
API, with live updates, and through OSC. A web page, a phone app, a
station's automation or a control surface can use them. It is
**off** until you turn it on, and at first it only answers on this computer.

## Turning it on

![Settings, Remote: the HTTP API on and listening on this computer, and OSC off](../images/guide/settings-remote.png)

Open **Settings → Remote** and tick **Allow remote control over HTTP**
(or **Allow OSC control**). The line under each switch says whether the
server is listening, and where, or why it did not start. Changes apply at
once; there is no need to restart. A text field (an address, the token, a
list) applies when you leave it, open another section or close Settings;
a value that is not valid yet keeps the one in use, and Esc cancels what
you typed.

You can also edit `config.json` while Fauste Player is closed (see
[Data and backups](data-and-backups.md) for where it is). Inside the
`"config"` object, set `remote.http.enabled` to `true`:

    {
      "schema_version": 1,
      "config": {
        …
        "remote": {
          "http": { "enabled": true }
        }
      }
    }

It listens on `http://127.0.0.1:7380`. Starting it never plays anything;
only requests act.

## Listening on the studio network

To reach it from other computers, set `bind` to `0.0.0.0` (or one of this
computer's addresses) and set a **token** of at least 16 characters.
In Settings → Remote, **Generate** makes a long random token. It is hidden
until you press **Show**, and **Copy** puts it on the clipboard for the
client.
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
to have open cannot drive the player. `"*"` (any origin) is only accepted
together with a token.

## What a client can do

A client can:

- read the players, playlists, tracks (with cover and waveform) and the
  cartwall;
- play, pause, stop, fade, restart and go back;
- choose the next entry (the entry on air too: it plays once more), pre-listen, and seek;
- set volumes, modes, stop-after-current, and the repeat and stop-after
  marks of an entry;
- fire, stop and pre-listen carts, and change the cart page shown;
- edit: create, rename and delete playlists; add a track that is already
  loaded, and remove, move or duplicate entries; create, rename, resize and
  delete cart pages, and set up a cart with a loaded track; set or reset
  markers.

Removing what is on air is refused, as it is on screen. Files that are not
loaded yet cannot be added remotely: they live on this computer, so add
them here first.

A button that is greyed out on screen is refused remotely too. The full
reference is in [the technical documentation](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/remote-api.md).

Try it from a terminal:

    curl -s http://127.0.0.1:7380/api/v1/players
    curl -s -X POST http://127.0.0.1:7380/api/v1/players/<id>/play

## Live updates

A client can follow changes as they happen instead of asking again and
again. `GET /api/v1/events` is a stream of events: the whole state first,
then each player, playlist, track or cartwall change, and the times of what
is playing a few times a second.

    curl -sN http://127.0.0.1:7380/api/v1/events

A web page uses `EventSource`. Browsers cannot send the token as a header
there, so it goes in the address:
`/api/v1/events?token=<token>`.

## OSC

OSC is the usual protocol of control surfaces, lighting desks and show
control software. Turn it on with `remote.osc.enabled`. It listens on UDP
port 7381 of this computer. To accept packets from other computers, set
`remote.osc.bind` to `0.0.0.0` and list their addresses or subnets in
`remote.osc.allowed_sources` (for example `"192.168.1.0/24"`). OSC has no
password, so keep it on a trusted studio network.

Players are numbered 1, 2, 3… as they appear on screen. Carts are numbered
in the page shown.

    oscsend localhost 7381 /fauste/player/1/play
    oscsend localhost 7381 /fauste/player/2/volume f 0.8
    oscsend localhost 7381 /fauste/cart/3/fire

A surface that wants to show the state (lights, names, countdowns)
subscribes, and then receives every value once and afterwards only what
changes. When a player or a cart button goes away (fewer players, a
smaller page), its addresses receive an empty value once, so the surface
clears them. It must subscribe again within a minute (`subscription_ttl_secs`)
to keep receiving:

    oscdump 9000 &
    oscsend localhost 7381 /fauste/subscribe i 9000

A subscriber can name any port of its own address, and up to
`max_subscribers` are kept. Anyone allowed to send can therefore also
subscribe. This is one more reason to keep OSC on a trusted network.

`oscsend` and `oscdump` come with liblo (`liblo-tools` on Debian and
Ubuntu). The full list of addresses is in
[the technical documentation](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/remote-api.md#osc).

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
| `remote.osc.enabled` | `false` | Turn OSC on |
| `remote.osc.bind` | `127.0.0.1` | Address to listen on |
| `remote.osc.port` | `7381` | UDP port (1024–65535) |
| `remote.osc.allowed_sources` | this computer | Addresses or subnets whose packets are accepted |
| `remote.osc.max_subscribers` | `16` | Subscribers at once |
| `remote.osc.subscription_ttl_secs` | `60` | A subscription not renewed within this time ends |
| `remote.events.position_interval_ms` | `250` | How often times are published while playing |

Values out of range are corrected when the file is loaded, and the
correction is logged.
