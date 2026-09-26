# Players

Each column is one player. Players are independent: each has its own
playlist tabs, transport, volume and outputs.

## Header

| Element | Meaning |
|---|---|
| `P1` … `Pn` | Player number (the number key that plays it) |
| Status dot and label | **On air** (red), **Paused** (amber), **Stopped** (grey) |
| **Mixing** / **Fading** badge | A crossfade into the next track, or a fade stop, is running |
| **Stop after** badge | The player stops when the current track ends |
| **BP** | Bit-perfect output indicator (inactive until a later version) |
| **SINGLE** / **CONT** | Play mode (see below) |
| **CUE** | Pre-listen the next track on the CUE output |

## Info row

- **Cover** of the track on air, or a vinyl placeholder.
- **Stereo meter:** 20 segments, green, yellow and red, with peak hold.
- **Volume fader:** drag it or use the mouse wheel, one step per notch. The
  tooltip shows the level in dB; the top is 0 dB and the bottom is silence.
- **Title, artist** and the **next** line, with a green square. While CUE is
  on, the pre-listen position shows in blue.

## Transport

| Button | Action |
|---|---|
| **Play / Next** (large) | Stopped: start the next track. On air: fade into the next track (the fade time is set in [Settings](settings.md)). Paused: resume. |
| **Stop** | Stop at once (with a short de-click ramp) |
| **Fade stop** | Fade out and stop |
| **Pause** | Pause or resume; blinks amber while paused |
| **Stop after** | Stop when the current track ends (continuous mode only) |

## Modes

- **CONT (continuous):** at the MIX point the player starts the next track
  and overlaps the end of the current one. See
  [Markers and mixing](markers-and-mixing.md).
- **SINGLE:** every track stops at its end. *Stop after* is not available in
  this mode, because every track already stops.

## Countdown

The large number is the time left until the end of the track (its cue-out),
with tenths. Next to it are the elapsed time and the total. During the last
seconds before the end (10 by default, set in Settings) the countdown blinks
red.

## Waveform

- The part already played is drawn in the waveform colour; the rest is dimmer.
- A blue shaded area at the start marks the **intro**, and a badge counts it
  down. The intro only shows when it has been set.
- An orange shaded area at the end marks the **outro**, with its own countdown.
- A dashed amber line with a **MIX** tag marks where the next track starts in
  continuous mode. It is dimmed in single mode.
- Hover to see the time under the pointer. **Click to jump** there.

## CUE (pre-listen)

**CUE** plays the next track on the player's CUE output, for example
headphones, without touching the on-air output. The context menu of any
track also has **Pre-listen on CUE**. See [Settings](settings.md) to choose
the CUE device.
