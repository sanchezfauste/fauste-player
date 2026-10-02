# Fauste Player

[![CI](https://github.com/sanchezfauste/fauste-player/actions/workflows/ci.yml/badge.svg)](https://github.com/sanchezfauste/fauste-player/actions/workflows/ci.yml)
[![Conventional Commits](https://img.shields.io/badge/Conventional%20Commits-1.0.0-fe5196.svg)](https://www.conventionalcommits.org/en/v1.0.0/)
[![SemVer](https://img.shields.io/badge/versioning-SemVer%202.0.0-blue.svg)](https://semver.org/spec/v2.0.0.html)

**Fauste Player** is a desktop radio playout application for Linux, Windows
and macOS, written in Rust. It runs any number of independent players side
by side, each with its own playlists, transport and outputs, with
sample-accurate overlapping mixes, a separate pre-listen (CUE) output, and an
audio engine that the interface can never block.

![Main screen](docs/images/main-screen.png)

## Contents

- [Features](#features)
- [Platform support](#platform-support)
- [Install](#install)
- [Build from source](#build-from-source)
- [Develop](#develop)
- [How it is built](#how-it-is-built)
- [Working with AI agents](#working-with-ai-agents)
- [Versioning and releases](#versioning-and-releases)
- [Roadmap](#roadmap)
- [Documentation](#documentation)
- [Contributing](#contributing) · [Security](#security) · [Licence](#licence)

## Features

- **Independent players:** four by default, any number configurable. Each
  has Play/Next, Stop, Fade stop, Pause and Stop-after-current, SINGLE and
  CONT modes, a countdown with tenths, a stereo meter, a volume fader and
  its own playlist tabs. Times keep a fixed width as they count.
- **About window:** the version sits next to the name in the top bar; a
  click on it, or on the info button next to Settings, opens the copyright
  and the licence notices of the bundled components.
- **Restore defaults and Restart now:** Players, Meters, Analysis and
  Shortcuts can each be reset to their defaults; settings that need a
  restart show a "Restart pending" pill, and Restart now applies them.
- **Close guard:** closing the window while audio is on air asks first,
  lists what is sounding, and offers Stop and close or Cancel.
- **Standard level meters:** digital peak (IEC 60268-18), EBU and DIN PPM
  (IEC 60268-10), VU (IEC 60268-17) and K-System (K-20, K-14, K-12), each
  with its own ballistics and scale. Optional true peak and EBU R128
  loudness, and a maximum readout.
- **Sample-accurate mixing:** in continuous mode the next track starts
  exactly at the MIX point and overlaps the fading end of the current one.
  How far below the track's own level the MIX point sits, and the maximum
  overlap, are configurable.
- **Automatic markers:** background analysis finds cue-in, cue-out, the MIX
  point and the outro. It also reads tags and cover art, and draws the
  waveform. Markers set by hand are never overwritten.
- **CUE pre-listen** on a separate device or channel pair, with a window per
  CUE to seek, pause, stop and load the track as next.
- **Cartwall:** pages of instant carts (jingles, effects, spots), each with a
  configurable grid. Carts overlap by default; they can loop gaplessly or stop
  the others, and a **Stop all** button stops every playing cart. They have
  their own Main and Cue outputs. Pages can be imported and exported.
- **Tag editor and track tooltip**: hover a track for its tags, format and path; edit the common tag fields (title, artist, album, date, track and disc numbers, genre, BPM, key, lyrics and more) and the front cover (view, change, remove) of MP3, FLAC, MP4, WAV and other files, written safely into the file.
- **Marker editing on the waveform:** cue in and out, intro, outro and MIX,
  set from a menu or with Alt-drag. An `INTRO` tag can also set the intro.
  A setting makes players ignore cue-in and cue-out and play whole files.
- **Remappable keyboard shortcuts** for players and carts.
- **MIDI control surfaces:** every player's transport buttons and volume
  fader can be learned onto a MIDI controller, with soft takeover for
  faders, LED feedback and hot-plug.
- **Remote control:** an HTTP/JSON API with live events (Server-Sent
  Events) and OSC, off by default and set up in Settings → Remote. A web
  page, a phone app, automation or a control surface can follow the
  players, playlists and cartwall, operate them, and edit playlists, cart
  pages and markers. A token, origin checks and source allow-lists protect
  it when it listens on the network.
- **M3U / M3U8 / PLS import and M3U8 export.**
- **Bit-perfect output:** a device can be played with exclusive access,
  following each file's sample rate while idle; at 100 % volume with no fade
  or overlap the samples reach it unchanged, and a BP badge says so (ALSA
  `hw:` devices, WASAPI exclusive mode, Core Audio hog mode).
- **Routing per player:** Main and Cue outputs on any device and channel
  pair. Multichannel interfaces carry several players at once.
- **Resilient:**
  - if an output device is lost, the timeline keeps running on a virtual
    clock and the device reconnects on its own;
  - decoder, analysis, services and interface panics are contained;
  - saves are atomic with rotating backups;
  - after a crash, players come back paused at their positions, and nothing
    goes on air by itself.
- **Fast, dense interface** (egui). Virtualised track tables handle
  thousands of entries, with drag and drop within and across players and
  onto tabs, drops from the file manager, a context menu and keyboard
  shortcuts.
- **English and Spanish** interface (Fluent). It follows the OS language or
  the language chosen in Settings.

## Platform support

| OS | Audio systems | Bit-perfect |
|---|---|---|
| Linux (x86-64, ARM64) | PulseAudio (also on PipeWire desktops), JACK, ALSA; native PipeWire in builds with the `pipewire` feature | ALSA `hw:` devices |
| Windows 10/11 (x86-64) | WASAPI, JACK, ASIO (when built with the SDK) | WASAPI exclusive mode |
| macOS (Intel, Apple silicon) | Core Audio, JACK | Core Audio hog mode |

With nothing configured, the first available system is used, in this order:

- Linux: PipeWire, PulseAudio, JACK, ALSA;
- Windows: WASAPI, ASIO, JACK;
- macOS: Core Audio, JACK.

Systems that are missing on a machine show as unavailable in Settings; the
application still starts.

Formats: WAV, AIFF, CAF, FLAC, MP3 (and MP1/MP2), AAC/M4A, ALAC, Ogg Vorbis, Opus, WavPack, Monkey's Audio (APE), DSD (DSF and DFF) and Matroska audio (MKA). DSD is converted to PCM. Whole programme recordings play like songs: a 4-hour file is analysed in under a minute, with memory that does not grow with its length, and seeks at once.

## Install

Download a package for your platform from
[Releases](https://github.com/sanchezfauste/fauste-player/releases). Each
file has a `.sha256` next to it.

| Platform | Packages |
|---|---|
| Debian, Ubuntu | `fauste-player_<version>_amd64.deb` / `_arm64.deb`: `sudo apt install ./fauste-player_*.deb` |
| Fedora, openSUSE | `fauste-player-<version>-1.x86_64.rpm` / `.aarch64.rpm`: `sudo dnf install ./fauste-player-*.rpm` |
| Any Linux | `fauste-player-<version>-x86_64.AppImage` / `-aarch64.AppImage`, or the Flatpak bundle `fauste-player-<version>-x86_64.flatpak` |
| Windows | `fauste-player-<version>-x86_64-pc-windows-msvc.msi` |
| macOS | `fauste-player-<version>-macos-universal.dmg` (Apple silicon and Intel) |
| All | Portable archives (`.tar.gz`, `.zip`) per target |

See the [getting started guide](docs/user/getting-started.md) for installing,
updating and removing each one, and for first starts of unsigned builds on
macOS and Windows.

## Build from source

The toolchain is pinned in `rust-toolchain.toml` (Rust 1.98.1, edition 2024).
Install [rustup](https://rustup.rs); it picks the right version by itself.

### Linux

| Distribution | Packages |
|---|---|
| Debian, Ubuntu | `sudo apt install build-essential pkg-config libasound2-dev libdbus-1-dev` |
| Fedora | `sudo dnf install gcc pkgconf-pkg-config alsa-lib-devel dbus-devel` |
| Arch | `sudo pacman -S base-devel pkgconf alsa-lib dbus` |

PulseAudio support needs no extra package. To build with the optional
audio systems:

```sh
sudo apt install libpipewire-0.3-dev libspa-0.2-dev libjack-jackd2-dev clang   # Debian, Ubuntu
cargo build --release -p fp-app --features pipewire,jack
```

JACK is loaded at run time on every OS, so a `jack` build still starts
where JACK is not installed; on Windows and macOS it needs no SDK to build.
`pipewire` links libpipewire, so that build needs PipeWire installed to
start. ASIO needs the Steinberg ASIO SDK: set `CPAL_ASIO_DIR` to it and
build with `--features asio`.

The release archives are built with `jack` only. PipeWire desktops are
served through PulseAudio (pipewire-pulse), and the archives start on any
Linux with ALSA and D-Bus.

The window system libraries (X11 or Wayland, `libxkbcommon`, OpenGL) are
loaded at run time. Any desktop has them. Minimal systems may need
`libxkbcommon-x11-0` and `libgl1` (or their equivalents).

### Windows

Install the **Visual Studio Build Tools** with the *Desktop development with
C++* workload (MSVC and the Windows SDK), then rustup. Use the default
`x86_64-pc-windows-msvc` toolchain.

### macOS

```sh
xcode-select --install
```

### Build and run

```sh
git clone https://github.com/sanchezfauste/fauste-player.git
cd fauste-player
cargo run --release -p fp-app          # builds and starts target/release/fauste-player
```

To try it with a ready-made state (four players and three playlists built
from a folder of audio files):

```sh
FAUSTE_HOME=/tmp/fp-demo cargo run -p fp-app --example demo_session -- ~/Music
FAUSTE_HOME=/tmp/fp-demo cargo run --release -p fp-app
```

`FAUSTE_HOME` keeps every file of the application in one folder (portable
mode). Without it, the OS locations are used; see
[Data and backups](docs/user/data-and-backups.md).

## Develop

### Workspace

| Crate | Role |
|---|---|
| [`fp-model`](crates/fp-model) | Pure domain state and player rules: commands in, engine actions out |
| [`fp-store`](crates/fp-store) | Atomic JSON persistence, backups, migrations |
| [`fp-decode`](crates/fp-decode) | File decoding (symphonia) to stereo `f32` |
| [`fp-backends`](crates/fp-backends) | Audio backends: system through cpal, Null, Offline for tests |
| [`fp-engine`](crates/fp-engine) | Real-time mixer, buses, decode workers, the conductor thread |
| [`fp-analysis`](crates/fp-analysis) | Tags, covers, peaks, automatic markers, cache, background pool |
| [`fp-app`](crates/fp-app) | The egui application and the `fauste-player` binary |

### Everyday commands

```sh
cargo fmt --all                                        # format
cargo clippy --workspace --all-targets -- -D warnings  # lint (CI fails on any warning)
cargo test --workspace                                 # all tests: no sound card or display needed
cargo test -p fp-engine --test conductor               # one test file
cargo test --release -p fp-engine --test conductor -- --ignored six_simulated_hours   # soak
cargo deny check                                       # licences, advisories, sources
RUST_LOG=debug cargo run -p fp-app                     # verbose logs
git config core.hooksPath .githooks                    # enable the Conventional Commits hook
```

### Rules the code follows

- **No `unsafe`** (`forbid` workspace-wide), and no `unwrap`, `expect` or
  `panic` outside tests. The audio and analysis crates deny
  `clippy::indexing_slicing`.
- **The real-time thread never allocates, frees, locks, logs or panics.**
  `assert_no_alloc` tests enforce it. See
  [Threading and real time](docs/technical/threading-and-realtime.md).
- **Behaviour lives in the pure `fp-model` reducer** and is tested rule by
  rule, without audio.
- **No hardcoded product limits:** everything tunable is in `Config`, with
  defaults, ranges and lenient loading.
- **TDD:** write the failing test first. UI interactions are tested
  headlessly with `egui_kittest`.
- **English** for all code, identifiers, comments, commits and docs. Every
  UI string goes through Fluent, in both `en-US` and `es-ES`.

## How it is built

The design started as an interactive prototype (the "v3" main screen with
the Nocturne design system). It became a binding
[design spec](docs/superpowers/specs/2026-09-25-fauste-player-design.md):
normative player rules, engine, backends, analysis, persistence, UI,
security, testing and phases. Each phase is split into implementation plans
in [`docs/superpowers/plans`](docs/superpowers/plans). Each plan was
executed task by task with test-driven development, then reviewed by a fresh
reviewer, and reaches `master` through a pull request once CI is green on
every OS. Deviations and decisions are recorded as
rulings in the plans.

In short:

- the UI sends commands to a **conductor** thread;
- the conductor applies them through the **pure model reducer** and turns
  the resulting actions into **sample-accurate commands** for one real-time
  mixer per output device;
- per-player worker threads decode and resample into lock-free rings;
- analysis and saving run in the background;
- the UI reads immutable snapshots and never waits for the audio path.

Details are in the [technical documentation](docs/technical/README.md).

## Working with AI agents

The project is developed with Claude Code and the **superpowers** skills.
[`CLAUDE.md`](CLAUDE.md) is the canonical guide for agents (rules, commands,
invariants, workflow). [`AGENTS.md`](AGENTS.md) points other agents to it.

Install the skills from the official plugin marketplace inside Claude Code
(`/plugin` → *superpowers*). Use them as follows:

| Situation | Skill | How |
|---|---|---|
| A new feature or phase | `superpowers:brainstorming` | Explore the idea, agree the design, update the spec |
| A spec is agreed | `superpowers:writing-plans` | Write `docs/superpowers/plans/<date>-<phase>-<plan>.md` with tasks, interfaces and a review focus |
| Implementing a plan | `superpowers:executing-plans` (inline) or `superpowers:subagent-driven-development` | Task by task, with a ledger in `.superpowers/sdd/` |
| Writing any code | `superpowers:test-driven-development` | Red, green, refactor; never skip the failing run |
| A bug or a failing test | `superpowers:systematic-debugging` | Find the cause before changing code |
| Before claiming done | `superpowers:verification-before-completion` | Run the checks and read the output |
| After a plan | `superpowers:requesting-code-review` | A fresh reviewer on the most capable model reviews the whole branch |
| Handling review findings | `superpowers:receiving-code-review` | Verify each finding, then fix it test-first |
| Finishing | `superpowers:finishing-a-development-branch` | Green suite, then merge or PR |
| Isolated work | `superpowers:using-git-worktrees` | One worktree per branch |

The UI design lives in a Claude Design project, which agents can read through
the `claude_design` MCP server (see `CLAUDE.md`).

## Versioning and releases

- [Semantic Versioning](https://semver.org/spec/v2.0.0.html) and
  [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/)
  (`feat(engine): …`, `fix(ui): …`, `docs: …`).
- [release-please](https://github.com/googleapis/release-please) keeps a
  release PR that bumps the version and writes the
  [changelog](CHANGELOG.md).
- Merging the release PR tags `vX.Y.Z`, publishes the GitHub release, and
  attaches binaries for Linux (x86-64, ARM64), Windows (x86-64) and macOS
  (Intel, Apple silicon), with checksums and third-party licence notices.

See [Release process](docs/technical/release-process.md).

## Roadmap

| Phase | Content | Status |
|---|---|---|
| 1. Usable core | Players, mixing, CUE, analysis, persistence, main screen, Settings subset, CI and releases | done |
| 2. Cartwall and full Settings | Cart pages, remappable shortcuts, language selector, M3U/M3U8/PLS import and export, manual marker editing | done |
| 3. Native backends | PipeWire, PulseAudio, JACK, ASIO | done |
| 4. Bit-perfect | Output at the file's rate and format with no processing, BP badge; ALSA `hw:`, WASAPI exclusive mode, Core Audio hog mode | done (Windows and macOS need an on-device loopback check) |
| 5. Packaging | deb, rpm, Flatpak, AppImage, MSI, universal dmg; signing and notarisation when certificates are configured | done |

## Documentation

- [User guide](docs/user/README.md)
- [Technical documentation](docs/technical/README.md)
- [Design spec](docs/superpowers/specs/2026-09-25-fauste-player-design.md) and [implementation plans](docs/superpowers/plans)
- [Changelog](CHANGELOG.md)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Security

See [SECURITY.md](SECURITY.md). The application makes no network
connections and sends no telemetry.

## Licence

No licence has been chosen for Fauste Player yet: until one is added, all
rights are reserved by the author. Bundled third-party components keep
their own licences:

- the Inter font: SIL Open Font License 1.1,
  [`OFL.txt`](crates/fp-app/assets/fonts/OFL.txt);
- the Phosphor icons: MIT,
  [`Phosphor-MIT.txt`](crates/fp-app/assets/licenses/Phosphor-MIT.txt);
- every Rust dependency: listed in `licenses/THIRD-PARTY.html` inside each
  release archive.

The About window (click the name or the info button in the top bar) shows the copyright, both
bundled notices and a button that opens the installed `THIRD-PARTY.html`.
