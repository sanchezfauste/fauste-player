# Troubleshooting

## No sound

1. Open **Settings → Audio outputs** and press **Test Main** for the player.
   If you hear the tone, check the player's volume fader.
2. If you hear nothing, pick another device or channel pair. A change to
   the outputs applies once nothing plays on them; **Apply now** in Settings
   applies it at once.
3. On Linux, prefer **PipeWire** or **PulseAudio** in Settings → Audio
   outputs → Audio system. They share the sound card with other programs.
   **ALSA** talks to the card directly and may find it busy.
4. **JACK** shows as unavailable ("no output device") when no JACK server is
   running. Start the server (for example with QjackCtl) and restart the
   application. Set the JACK server to the sample rate in Settings (48 kHz
   by default): JACK runs at one rate for every program.
5. **PipeWire** is not offered by the downloadable archives; they reach
   PipeWire through its PulseAudio service, which works the same way. It is
   available in builds made with the `pipewire` feature.

## Bit-perfect

- **The BP badge stays off.** Check each condition in
  [Bit-perfect output](bit-perfect.md#the-bp-badge): no fade, nothing else
  on the same outputs, a lossless file that has been
  analysed, and a device running at the file's rate.
- **A short silence before a track.** The bit-perfect device reopened at the
  track's sample rate. It happens only when the rate changes: tracks at the
  same rate follow each other with no gap. Keep the library at one rate to
  avoid it. See [What it costs](bit-perfect.md#what-it-costs).
- **Tracks do not mix, and Play does not fade.** On a player whose Main
  output is bit-perfect, each track plays to its end and then the next one
  starts, and Play / Next and Previous switch at once: there are no
  crossfades or segues on such a player. Turn bit-perfect off for the
  device, or move the player to another output, to mix again. See [What it costs](bit-perfect.md#what-it-costs).
- **The volume fader does not move.** The player's Main output is
  bit-perfect, which holds the volume at 100 %; the fader's tooltip says so.
  MIDI faders and the remote API cannot move it either. See [What it costs](bit-perfect.md#what-it-costs).
- **A track plays resampled, and the log says the device is busy (Linux).**
  To change rate, the application closes the device and opens it again. In
  that moment the sound server (PipeWire) can take the card. The application
  tries again a few times; if the card is still busy, the track plays at the
  device's current rate, and the next track asks for its rate again. To give
  the application the card to itself, open the system's sound settings and
  set that card's profile to **Off** (or **Pro Audio**), so the sound server
  leaves its `hw:` device alone. The number of tries and the wait between
  them are `tuning.device_busy_retries` and `tuning.device_busy_retry_ms` in
  the configuration file.
- **The device plays, but the BP badge stays off (Windows or macOS).**
  Exclusive access was refused, and the device plays shared.
  - Windows: another program may hold the device exclusively, or exclusive
    control is turned off in the device's Advanced properties.
  - macOS: another program may hold the device in hog mode, or the device
    offers its rates only as a continuous range (most interfaces list fixed
    rates).
- **A `hw:` device cannot be opened (Linux).**
  - A sound server may be holding the card. Stop it, or set the server to
    leave that card alone, and restart the application.
  - Some USB DACs accept only packed 24-bit samples (`S24_3LE`), which the
    audio library does not support. Use that card through `plughw:` (not
    bit-perfect) instead.

### DSD

- **A DSD track plays converted although the device is set to DoP or native
  DSD.** The log says why ("DSD converted to PCM" and the reason) for these
  causes: the player's volume not at 100 %, something else playing on the
  device, more than two channels, or a device that refuses the rate (DoP needs
  the DSD rate divided by 16, for example 176.4 kHz for DSD64) or has no 24-
  or 32-bit format. A track that has not been analysed yet converts silently,
  with no log line: analyse it (Settings → Analysis) and play it again.
- **Only the first track of a DSD album goes out as DSD.** That is the
  default mixing setting: the tracks the player starts by itself play
  converted. Choose **Keep DSD and mute the other sources** in Settings →
  Audio outputs to keep them DSD. See [DSD](bit-perfect.md#dsd).
- **The header shows DSD but the converter plays noise or does not lock.**
  The converter does not recognise DoP (or the native format). Set the device
  back to **Convert to PCM**.
- **A click when a DSD track starts, stops or leaves DSD.** The converter
  needs more DSD silence: raise **DSD silence** (200 ms by default) in
  Settings → Audio outputs, Advanced.
- **Other players or carts are silent on the device.** A DSD track is playing
  with **Keep DSD and mute the other sources**; the **Others muted** badge
  shows. They sound again when the track ends.

## "Output lost" alert

The status bar shows **Output lost: &lt;device&gt;** when a device stops responding.
The players keep counting and mixing on an internal clock, so the automation
does not stall. The device is retried every 2 seconds and takes over again
when it returns. Reconnect the cable or power the interface back on.

### "Output lost" that never clears, with a direct `hw:` output

A sound card used through a direct ALSA `hw:` output (for example a
bit-perfect output) is held by Fauste Player alone: the sound server
(PipeWire or PulseAudio) cannot use it at the same time. If another output
goes through the sound server's default device and that default device is
the same card, that output never starts and stays **Output lost**. The log
says "output device opened but never started" once.

Use one path per card: route every output of that card through the same
`hw:` device (with different channels if needed), or choose another card
as the sound server's default output in your system's sound settings.

## A track shows a warning icon or a file with a cross

The file is missing (moved, deleted, unmounted: a file with a cross) or
cannot be decoded (a warning sign). The players skip it. Hover the row
to see which, and the file's path.

A missing file is looked for again every 30 seconds
(`tuning.missing_recheck_ms` in the configuration file): when the drive
is mounted or the file is put back, the track becomes playable by itself. A
file that cannot be decoded is checked again by itself on the same timer, by
size and modification time: it is not decoded again unless one of them
changed, for example when a copy finishes. To check it at once use
**Re-analyse** in its row menu, or **Settings → Analysis → Re-analyse all
tracks** for the whole library.

A WAV or FLAC that shows as unreadable, or would play as loud noise, may
be a DTS-CD rip: the file holds a DTS bitstream instead of audio. The
application refuses it (the log says "encoded bitstream stored as PCM") and
does not decode DTS. Convert it to ordinary PCM with another tool first.

## Audio dropouts

The status bar warns for 5 seconds after each dropout the application
detects: **P1: audio dropouts (3)** when a player's decoding did not keep up
with the disk (the count is for the track now playing), and
**&lt;device&gt;: audio device dropouts (2)** when the output device missed a
deadline (an xrun). The log records each one too, at most one line every 10
seconds per kind, with how many happened. Not every audio system reports
xruns (PulseAudio does not; Windows exclusive mode does not).


- Increase the **buffer size** in Settings (it applies once nothing plays on that device, or at once with **Apply now**).
- Through a sound server (PulseAudio, PipeWire, the ALSA `default`
  device), every output already asks for at least 50 ms whatever the
  buffer size, so that a busy computer is not heard. On an old or heavily
  loaded computer, raise `shared_buffer_ms` in the `outputs` section of the
  configuration file (for example to 100); cart and play buttons then
  sound that much later.
- On Linux, allow real-time scheduling. The application asks the system for
  it through rtkit (D-Bus), and the log records a refusal. Many desktops
  refuse it to applications; membership of the `audio` group with an
  `rtprio` limit also works.
- Avoid network drives for music that plays on air.

## The interface is slow on an old computer

While something plays, the interface draws at most 30 frames a second
(`max_fps` in the `ui` section of the configuration file), and clicks and
keys are answered at once. On an old or busy computer lower it (for
example to 20) to leave more of the processor to the audio; 0 draws every
frame the display shows. The meters measure the same at any rate; they are
only drawn less often.

## Files cannot be dropped on Linux

The window takes files dragged from a file manager only when it runs on X11.
By default it does, also on a Wayland desktop (through XWayland, which every
desktop has). If `ui.window_system` in `config.json` is set to `"Wayland"`,
the window is a native Wayland one, which can look sharper with fractional
display scaling, but files cannot be dropped on it: use **+ Add** instead.
Set it back to `"Auto"` (the default) or `"X11"` and restart Fauste Player.

## A MIDI controller does not respond

If **Settings → MIDI** shows the controller as connected but its buttons and
faders do nothing, press **Reconnect ports** there: the program closes the
MIDI ports and opens them again, as unplugging and plugging in the
controller would. If that does not help, unplug the controller and plug it
back in.

Either way, the log says what happened. It lists the MIDI ports whenever
they change, each port opened and closed (with why), a port that cannot be
opened (once, until it opens or goes away), the first message received on
each connection, the first message ignored on each connection with the
reason (no binding, a fader that has not reached the volume yet, an action
that cannot act now) and the first LED message that failed. A connected
controller that never logs a first message is sending nothing the program
receives. For every ignored message and every failed retry, start the
program with `RUST_LOG=info,fp_control=debug`.

## "The interface hit an error"

A drawing error was caught. Audio is not affected. Press **Restart
interface**. Please report it with the logs.

## Logs and crash reports

See [Data and backups](data-and-backups.md) for the log folder. There is one
log file per day. Identical lines that repeat one after another are written
once, followed by a line such as `previous message repeated 12 times`
(during a long run of repeats that line is written every minute). Lines
that differ, for example because they carry a counter, are not collapsed.
The
PulseAudio client's routine connection messages are left out by default.
Set `RUST_LOG=debug` in the environment for more detail (it replaces the
default filter, `info,pulseaudio=warn`).

The log is bounded: a day's file stops growing at 10 MiB (its last line
says so; the rest of that day is not logged, which is what a flood of
differing lines leads to) and the last 14 files are kept. Change them in the
config file with `limits.max_log_file_mb` (1 to 1024), `limits.max_log_files`
(2 to 365) and `limits.log_repeat_summary_secs` (1 to 3600, default 60, how
often a long run of repeats is reported); they apply at the next start.
Crash reports are saved as `crash-<time>.txt` in the same folder and are
limited separately (`limits.max_crash_reports` per run).
Attach both files when reporting a bug.
