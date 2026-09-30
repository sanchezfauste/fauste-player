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
| **BP** | Lit while the current track reaches its Main device unchanged (see [Bit-perfect output](bit-perfect.md)) |
| **SINGLE** \| **CONT** | Play mode (see below): one joined control, the lit half is the active mode |
| **CUE** | Pre-listen the next track on the CUE output |

## Info row

- **Cover** of the track on air, or a vinyl placeholder.
- **Stereo meter:** the level the player puts out, after its volume.
  - One continuous bar per channel, on the scale of the meter type's
    standard (the digital peak meter by default: −60 to 0 dBFS, with the
    top 20 dB taking half the height). The scale's marks sit between the
    channels; the brighter one is the alignment level (−18 dBFS).
  - The bar is green, yellow from the warning level (−9 dBFS), and red
    from the danger level (−3 dBFS). The other meter types turn red where
    their scale does (from 0 VU, from the permitted maximum on a PPM).
  - K-System meters show two sections: the solid bar is the average (RMS)
    level and the dimmer part above it reaches the peak. Their colours are
    the K-System's: green below 0, amber from 0 to +4, red above.
  - The highest level stays lit for a moment as a line (the peak hold).
  - The number above is the highest level since the entry started, in
    dBFS, red in the danger zone. It stays after a stop and starts again
    when an entry plays (the next one or the same again), or when you
    click it.
  - The number underneath is the loudness in LUFS (EBU R128), green within
    ±1 LU of the target (−23 LUFS).
  - The meter type and every level can be changed in
    [Settings → Meters](settings.md#meters).
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
| **Stop after** (a play triangle then a square) | Stop when the current track ends (continuous mode only) |

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
- The outline shows the peaks, faint; the solid body inside it is the average
  (RMS) level. On a loud track the peaks fill the height, and the body still
  shows where the track is quieter or louder.
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
