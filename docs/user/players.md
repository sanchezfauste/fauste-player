# Players

Each column is one player. Players are independent: each has its own
playlist tabs, transport, volume and outputs.

![Player 1 on air: its header with the loudness gain, cover, title, next track, transport, countdown, meter, fader and waveform](../images/guide/player.png)

## Header

| Element | Meaning |
|---|---|
| `P1` … `Pn` | Player number (the number key that plays it) |
| Status dot and label | **On air** (red), **Paused** (amber), **Stopped** (grey) |
| Narrow columns | When the column is too narrow for every badge, the header drops the least important first (the state text, then **Others muted**, the entry notice, **Stop after**, **Mixing** / **Fading**, the state dot), so nothing ever overlaps. The player number, **CUE**, the mode, **BP** and the gain badge always stay. Widen the column to see them all |
| **Mixing** / **Fading** badge | A crossfade into the next track, or a fade stop, is running. **Mixing** never lights on a player whose Main output is bit-perfect: it does not crossfade (see [What it costs](bit-perfect.md#what-it-costs)) |
| **Stop after** badge | The player stops when the current track ends |
| **Repeat** / **Stop after track** badge | The current track repeats, or stops the player when it ends, because of its own mark in the playlist menu. Hover for the full sentence. The player's own **Stop after** button wins: while it is on, only its badge shows |
| Gain badge | With [loudness normalization](loudness.md#the-gain-in-the-player-header) on, the gain the track on air plays with (**+4.0 dB**), or an amber **0.0 dB** when the track plays unchanged (no measurement yet, a bit-perfect output, or a track excluded from normalization). Hover for the reason. Shown while the player is playing or paused |
| **BP** / **DSD** | The badge shows only on a player whose Main device is bit-perfect; **BP** is lit while the current track reaches that device unchanged. **DSD** replaces it while a DSD track goes out as DSD, unchanged (see [Bit-perfect output](bit-perfect.md)). **Others muted** shows beside it when that DSD stream keeps other sources off the output |
| **SINGLE** \| **CONT** | Play mode (see below): one joined control, the lit half is the active mode |
| **CUE** | Pre-listen the next track on the CUE output |

Right-click the header's background (not one of its buttons) to maximize
the player for a moment or to hide it (see
[Layout](layout.md#maximize-a-panel)).

## Info row

- **Cover** of the track on air, or a vinyl placeholder.
- **Title and artist** of the track on air. A stopped player shows the
  track Play will start (its next), with its cover, its length and its
  waveform, ready at the cue-in, or where you clicked its waveform.
- A track played or loaded before its analysis has finished already has its
  length when its file's header gives it: the countdown, `elapsed / total`
  and click to jump work at once, over a flat line until the waveform is
  ready. When the header does not store it (raw AAC files, MP3 files with
  no length frame, Matroska and WebM), the total reads "—" and the waveform
  cannot be clicked until the analysis ends.
- **Stereo meter** (in the column at the right of the player, next to the
  fader; it spans the info row and the transport): the level the player
  puts out, after its volume.
  - One continuous bar per channel, on the scale of the meter type's
    standard (the digital peak meter by default: −60 to 0 dBFS, with the
    top 20 dB taking half the height). The scale is a ruler on each side of the
    bars, labelled in the meter's own units on both sides. Its top and its
    bottom (for the digital meter, the scale floor) are always marked; every
    label has a tick on each ruler, and shorter ticks between the labels
    work like those of a measuring ruler: evenly spaced on round values,
    every 1 dB on the EBU meter and every 5 dB below −20 on the digital one
    when the meter is tall enough, and every 2, 2.5, 5 or 10 dB (or none)
    where it is too short for them. On the digital (and custom) meter, a
    tall meter labels more values: every 1 dB from −20 to 0, and every
    5 dB between the 10 dB marks below −20 (−45, −55), always evenly
    spaced and only where they fit without the labels touching; the
    other meter types keep the labels their standard gives. The alignment
    level (−18 dBFS on the digital meter) is a thicker white tick on both
    rulers. Nothing is drawn
    over the bars or between them, so what you see in the bars is only the
    level, the peak hold and the colours.
  - The bar is green, yellow from the warning level (−9 dBFS), and red
    from the danger level (−3 dBFS). The other meter types turn red where
    their scale does (from 0 VU, from the permitted maximum on a PPM).
  - K-System meters show two sections: the solid bar is the average (RMS)
    level and the dimmer part above it reaches the peak. Their colours are
    the K-System's: green below 0, amber from 0 to +4, red above.
  - Digital peak, K-System and custom meters keep the highest level lit for a
    moment as a line (the peak hold; its length is **Peak hold** in
    [Settings → Meters](settings.md#meters), and 0 turns it off). EBU PPM,
    DIN PPM and VU meters have no hold.
  - The number above is the highest level since the entry started, in
    dBFS, red in the danger zone. It stays after a stop and starts again
    when an entry plays (the next one or the same again), or when you
    click it.
  - The number underneath is the loudness in LUFS (EBU R128), green within
    ±1 LU of the target (−23 LUFS).
  - The meter type and every level can be changed in
    [Settings → Meters](settings.md#meters).
  - **Readings above 0 dBFS.** The meter shows what the player puts out, and
    that can exceed full scale. The bar stops at the top of the scale, so
    the same red shows 0 dBFS and anything above it; only the number above
    tells how far, with its sign (for example `+3.5`).
    - A file can carry levels above full scale itself (a float file, or a
      lossy file whose decoded peaks exceed it).
    - Converting the rate can create peaks between the samples: a signal
      that touches 0 dBFS reads about +3 dBFS after 44.1 → 48 kHz. The
      true-peak option reads those peaks too.
    - The meter reads each player alone, not the sum on the device: two
      players on one output can add up above full scale without either
      meter showing it.
    - Nothing in the player adds gain above 100 %. An integer device clips
      at full scale; a float device receives the level as it is and the
      sound system or driver clips it.
- **Volume fader** (right of the meter, as tall as it). It follows a
  mixing-desk law: it is fine near the top and steeper towards the bottom,
  so lowering it a little lowers the level a little (90 % of the travel is
  about −2.5 dB, 50 % about −15.9 dB, 10 % about −48 dB). The top is 0 dB and
  the bottom is silence. Notches across the fader mark 0, −2, −4, −6, −8,
  −10, −12, −15, −20, −25, −30, −40, −50 and −∞ dB, with the level written on their right.
  - Drag it: the fader moves by as much as the pointer does, from where it
    is, so grabbing it never makes it jump; a click without moving changes
    nothing.
  - Use the mouse wheel: one step per notch (about 0.6 dB near 0 dB).
  - Hold **Ctrl** or **Shift** while dragging or using the wheel to move it
    ten times finer.
  - Double-click the knob to return to 0 dB (a double click elsewhere on
    the fader does nothing).

  The level in dB (one decimal, or −∞ for silence) is written under the
  fader, on the same row as the meter's loudness line, and the tooltip shows it too.
  A session saved before this law keeps each player's level; only the
  position of the fader changes.
  On a player whose Main output is bit-perfect the fader is locked at 100 %
  and greyed, it reads a dimmed 0.0, and its tooltip says why (see [What it costs](bit-perfect.md#what-it-costs)).
- **Title, artist** and the **next** line, with a green square. While CUE is
  on, the pre-listen position shows in blue.

## Transport

| Button | Action |
|---|---|
| **Play / Next** (large) | Stopped: start the next track. On air: fade into the next track (the fade time is set in [Settings](settings.md)). Paused: resume. With the track on air as next, Play restarts that track with the usual fade. On a bit-perfect output it switches at once, without a fade, and its tooltip says so (see [What it costs](bit-perfect.md#what-it-costs)). |
| **Stop** | Stop at once (with a short de-click ramp) |
| **Fade stop** | Fade out and stop. While a fade runs, a violet bar drains across the button from full to empty over the fade length, like a playing cart's |
| **Pause** | Pause or resume; blinks amber while paused |
| **Stop after** (a play triangle then a square) | Stop when the current track ends, once. In SINGLE mode it is available only while the current track repeats: it ends the repeat when the pass that is playing ends. To stop after a track every time it plays, or to repeat a track, use its menu in the playlist (see [Playlists](playlists.md)) |
| **Previous** (a bar and two triangles) | While on air: fade back into the track this player played before, as Next does. Press again to keep going back. The track you left becomes the next one. On a bit-perfect output it switches at once, without a fade (see [What it costs](bit-perfect.md#what-it-costs)). |
| **Restart** (a bar and one triangle) | Back to the start of the current track (its cue-in). A paused player stays paused. |

Buttons that cannot act right now are dimmed: Stop and Restart with nothing
loaded, Pause and Fade stop while stopped, Previous with no earlier track or
during a fade. A player remembers the last 50 tracks it played
(`players.history_len` in `config.json`, 0 to 1000).

## Modes

- **CONT (continuous):** at the MIX point the player starts the next track
  and overlaps the end of the current one. See
  [Markers and mixing](markers-and-mixing.md).
- **SINGLE:** every track stops at its end. *Stop after* is not available in
  this mode, because every track already stops, except while the current
  track repeats: then it ends the repeat when the pass that is playing ends.

## Countdown

The large number is the time left until the end of the track (its cue-out),
with tenths. The elapsed time and the total are on the row under the
waveform, on the right. During the last
seconds before the end (10 by default, set in Settings) the countdown blinks
red.

## Waveform

- The part already played is drawn in the waveform colour; the rest is dimmer.
  With a violet waveform colour, the violet MIX line stands out less; pick
  another waveform colour if you need to tell them apart.
- The outline shows the peaks, faint; the solid body inside it is the average
  (RMS) level. On a loud track the peaks fill the height, and the body still
  shows where the track is quieter or louder.
- A blue shaded area at the start marks the **intro**, and a badge counts it
  down. The intro only shows when it has been set.
- An orange shaded area at the end marks the **outro**, with its own countdown.
- A dashed violet line (the colour of the **Mixing** badge, so it is not mistaken
  for the orange outro) with a **MIX** tag marks where the next track starts in
  continuous mode. It is dimmed in single mode, and on a player whose Main
  output is bit-perfect, which does not mix (see [What it costs](bit-perfect.md#what-it-costs)).
- The whole file is drawn. The silent start and end that playback skips (before
  cue-in and after cue-out) are drawn darker, with a thin line where playback
  starts and ends. With **Use cue-in and cue-out** off (Settings → Players)
  nothing is darker and the two lines are dimmed: playback runs from the start
  to the end of the file, and the cue-in and cue-out in this guide mean those
  two ends.
- Hover to see the time under the pointer. **Click to jump** there. On a
  stopped player a click chooses where **Play** starts the next track: the
  playhead and the countdown move there, and nothing plays until you press
  Play. Choosing another next track, moving or removing it, or Stop goes back
  to the cue-in; so does any other way of starting a track, and Restart,
  Previous and the automatic advance always use the cue-in. A click before
  the cue-in (in the darker start) chooses the cue-in. A click at or after
  the cue-out (in the darker tail) cancels an earlier choice: Play starts at
  the cue-in. A click is a press and release without moving the pointer more
  than a few pixels.
- **Press and drag** moves the zoomed view along the track, like grabbing it.
  A drag never jumps, and without zoom it does nothing. Alt-drag still edits
  markers.
- **Mouse wheel** over the waveform: zoom in and out around the pointer, down
  to the finest detail the analysis has. **Shift+wheel** (or a sideways wheel)
  moves along the track. While zoomed, the view follows the playing position,
  except for 10 seconds after you zoom or move it
  (`ui.follow_current_grace_secs`; 0 turns following off). **Full view**, in the top-right corner,
  zooming all the way out or a new track show the whole track again.

## CUE (pre-listen)

Pressing **CUE** (or **Pre-listen on CUE** in a track's menu) plays the track
on the player's CUE output, for example headphones, and opens a small **CUE window** for that player. Several
windows can be open, one per player. See [Settings](settings.md) to choose
the CUE device. A player needs a Cue output: without one, **CUE** and
**Pre-listen on CUE** are dimmed, and hovering them says so.

With one sound card, the Cue output can be the on-air output itself (or
share it with another player or the cartwall). A pre-listen there is heard
on air, so:

- while nothing plays on that output, CUE starts at once;
- while something plays there (a player playing or paused, a cart, another
  pre-listen), CUE asks first: **Pre-listen anyway** or **Cancel** (Cancel
  is the default: Esc and Enter cancel). The same question comes from the
  track menu, the CUE window and the cartwall;
- when something starts on that output while a pre-listen you did not
  confirm plays there, the pre-listen stops and the status bar says why. A
  pre-listen you confirmed keeps playing.

The CUE button's tooltip says when its pre-listen goes out on the on-air
output.

![The CUE window of player 4, pre-listening its next track](../images/guide/cue-window.png)

The window shows:

- the title and artist;
- the waveform of the whole file with the CUE position. It works like the
  player's: click to jump there, zoom with the wheel, drag to move along,
  **Full view**, the intro and outro countdowns, and the intro, outro and MIX
  markers, which you edit here as on the player (see
  [Markers and mixing](markers-and-mixing.md)). A CUE plays the whole file,
  so nothing is drawn darker, the cue-in and cue-out are dimmed lines, and
  the outro counts down to the end of the file. Its zoom is its own: the
  player's waveform does not move. While the CUE plays, a zoomed view
  follows its position as the player's does; a paused CUE keeps the view you
  set, so you can zoom in and place markers. A CUE started after its window
  closed, or on another track, shows the whole file;
- the elapsed time and the time remaining to the end of the file (a CUE plays
  whole files);
- **Pause** / **Resume**, **Stop** and **Set as next**. **Set as next**
  makes the cued track the player's next and keeps the CUE playing. It is dimmed when the track already is the next. If the
  cued track is the one on air, it plays once more when the current pass
  ends.

While the CUE is paused, its **Pause** button (shown as **Resume**) blinks
amber, like the player's own.

A jump on a paused CUE keeps it paused. The close button of the window, or
**Stop**, stops the CUE.

While a CUE runs, setting a next (double-click) or a single click on a row
moves the CUE to that track, from its cue-in; if it was paused, it plays
again. A track whose file is missing or unreadable leaves the CUE where it
is.
