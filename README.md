# Fauste Player

Desktop radio playout for Linux, Windows and macOS, written in Rust:
independent players with their own playlists, sample-accurate mixes,
pre-listen (CUE) output, automatic cue markers, and an interface that never
blocks the audio.

## Building

Rust 1.98 or newer (`rustup` recommended).

System packages:

| OS | Packages |
|---|---|
| Debian / Ubuntu | `build-essential pkg-config libasound2-dev libdbus-1-dev` |
| Fedora | `gcc pkgconf-pkg-config alsa-lib-devel dbus-devel` |
| Windows | Visual Studio Build Tools (MSVC) |
| macOS | Xcode command-line tools |

```sh
cargo build --release
cargo test --workspace
```

## Running

```sh
cargo run --release -p fp-app
```

Settings, playlists and the session are stored in the OS directories. Set
`FAUSTE_HOME=<dir>` to keep everything in one folder instead (portable
installs, tests).

To try it with a ready-made state (four players, three playlists) built from
a folder of audio files:

```sh
FAUSTE_HOME=/tmp/fp-demo cargo run -p fp-app --example demo_session -- ~/Music
FAUSTE_HOME=/tmp/fp-demo cargo run --release -p fp-app
```

Keys: `1`–`9` play each player, `Del` removes the selected track, `Esc`
closes dialogs. Double-click a track to set it as next; right-click for more.

Logs are written to the log directory (daily files, 14 kept); a crash report
is written there if the application panics.

## Workspace

| Crate | Role |
|---|---|
| `fp-model` | Pure domain state and rules (commands in, engine actions out) |
| `fp-store` | Atomic JSON persistence, backups, migrations |
| `fp-backends` | Audio backends (system via cpal, Null, Offline for tests) |
| `fp-decode` | File decoding (symphonia) |
| `fp-engine` | Real-time mixer, buses, decode workers, conductor thread |
| `fp-analysis` | Tags, cover art, waveform peaks and automatic markers |
| `fp-app` | The egui application |

## Licences

The Inter font (`crates/fp-app/assets/fonts`) is distributed under the SIL
Open Font License 1.1; see `crates/fp-app/assets/fonts/OFL.txt`. Icons come
from Phosphor (MIT) via `egui-phosphor`.
