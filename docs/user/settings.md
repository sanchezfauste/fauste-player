# Settings

Open **Settings** in the top bar. Close it with **Close** or `Esc`. Most
changes apply at once and are saved automatically.

## Audio outputs

Changes in this section apply **the next time the application starts**.

| Setting | Meaning |
|---|---|
| Audio system | Linux: PipeWire (in builds that include it), PulseAudio, JACK or ALSA. Windows: WASAPI, ASIO (in builds that include it) or JACK. macOS: Core Audio or JACK. Systems missing on this computer, or with no output device (a JACK server that is not running), are shown as unavailable. "System default" uses the first available one in that order. |
| Sample rate | The rate every output runs at; files are converted to it with high-quality resampling. Bit-perfect devices start at this rate and then follow the files. |
| Buffer size | Frames per audio block; the resulting latency is shown below it |
| Outputs per player | For each player, a **Main** (on-air) device and a **Cue** (pre-listen) device, each with a channel pair. Multichannel interfaces can carry several players on different pairs. |
| Test Main / Test Cue | Plays a short tone (1 kHz on Main, 440 Hz on Cue, 1.5 s, −18 dBFS) on the chosen output, so you can check the wiring before going on air |
| Cartwall | The cartwall's Main and Cue outputs. Main defaults to the system output. Without a Cue there is no cart pre-listen. |
| Bit-perfect devices | One switch per device chosen above. A bit-perfect device is opened with exclusive access and follows each file's sample rate while nothing plays on it. The switch is disabled where the device cannot give exclusive access. See [Bit-perfect output](bit-perfect.md). |

A Cue output never falls back to the output Main uses. A Cue that names a
device on an audio system this computer does not have, or the same output as
Main, means "no cue".

If a device disappears while playing, the players keep their timelines, and
the device is reopened when it comes back (see
[Troubleshooting](troubleshooting.md)).

## Players

| Setting | Default | Meaning |
|---|---|---|
| Language | System | Interface language |
| Number of players | 4 | Columns on the main screen (a player on air cannot be removed) |
| Default mode | CONT | The mode players start in |
| Fade time | 1000 ms | Used by Play while on air and by Fade stop |
| Automatic mix at the MIX point | On | Overlap tracks in continuous mode |
| End-of-track warning | 10 s | When the countdown starts blinking red |

## Analysis

The thresholds described in [Markers and mixing](markers-and-mixing.md).
**Re-analyse all tracks** runs the analysis again for the whole library;
manual markers are kept.

## Playlists

- **Music folder:** where the file dialogs start.
- **New playlist**, **rename** (edit the name and press Enter; Esc cancels)
  and **delete** (trash icon).
- **Import M3U / PLS…** creates a new playlist from a playlist file. **M3U**
  on each row exports it as M3U8. See [Playlists](playlists.md).

**Language:** System, English or Español. The interface switches at once.

## Cartwall

Pages, grid size, the cart editor, and cart page import and export. See
[Cartwall](cartwall.md).

## Keyboard shortcuts

See [Keyboard](keyboard.md).
