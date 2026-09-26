# Settings

Open **Settings** in the top bar. Close it with **Close** or `Esc`. Most
changes apply at once and are saved automatically.

## Audio outputs

Changes in this section apply **the next time the application starts**.

| Setting | Meaning |
|---|---|
| Audio system | The system audio interface: ALSA on Linux, WASAPI (shared) on Windows, Core Audio on macOS. "System default" follows the OS choice. |
| Sample rate | The rate every output runs at; files are converted to it with high-quality resampling |
| Buffer size | Frames per audio block; the resulting latency is shown below it |
| Outputs per player | For each player, a **Main** (on-air) device and a **Cue** (pre-listen) device, each with a channel pair. Multichannel interfaces can carry several players on different pairs. |
| Test Main / Test Cue | Plays a short tone (1 kHz on Main, 440 Hz on Cue, 1.5 s, −18 dBFS) on the chosen output, so you can check the wiring before going on air |

If a device disappears while playing, the players keep their timelines, and
the device is reopened when it comes back (see
[Troubleshooting](troubleshooting.md)).

## Players

| Setting | Default | Meaning |
|---|---|---|
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

The interface language follows the operating system (English or Spanish).
