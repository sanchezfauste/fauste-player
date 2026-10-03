# Troubleshooting

## No sound

1. Open **Settings → Audio outputs** and press **Test Main** for the player.
   If you hear the tone, check the player's volume fader.
2. If you hear nothing, pick another device or channel pair. Changes to
   outputs take effect after a restart: press **Restart now** in Settings.
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
  [Bit-perfect output](bit-perfect.md#the-bp-badge): volume at 100 %, no
  fade, nothing else on the same outputs, a lossless file that has been
  analysed, and a device running at the file's rate.
- **A short silence before a track.** The bit-perfect device reopened at the
  track's sample rate. Keep the library at one rate to avoid it.
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
  needs more DSD silence: raise `outputs.dsd_silence_ms` (200 by default) in
  the configuration file.
- **Other players or carts are silent on the device.** A DSD track is playing
  with **Keep DSD and mute the other sources**; the **Others muted** badge
  shows. They sound again when the track ends.

## "Output lost" alert

The status bar shows **Output lost: <device>** when a device stops responding.
The players keep counting and mixing on an internal clock, so the automation
does not stall. The device is retried every 2 seconds and takes over again
when it returns. Reconnect the cable or power the interface back on.

## A track shows a warning icon or a file with a cross

The file is missing (moved, deleted, unmounted: a file with a cross) or
cannot be decoded (a warning sign). The players skip it. Hover the icon
to see which, and the file's path.

A missing file is looked for again every 30 seconds
(`tuning.missing_recheck_ms` in the configuration file): when the drive
is mounted or the file is put back, the track becomes playable by itself. A
file that cannot be decoded is checked again only with **Settings →
Analysis → Re-analyse all tracks**.

## Audio dropouts

The status bar warns for 5 seconds after each dropout the application
detects: **P1: audio dropouts (3)** when a player's decoding did not keep up
with the disk (the count is for the track now playing), and
**<device>: audio device dropouts (2)** when the output device missed a
deadline (an xrun). The log records each one too, at most one line every 10
seconds per kind, with how many happened. Not every audio system reports
xruns (PulseAudio does not; Windows exclusive mode does not).


- Increase the **buffer size** in Settings (and press **Restart now**).
- On Linux, allow real-time scheduling. The application asks the system for
  it through rtkit (D-Bus). Membership of the `audio` group with an `rtprio`
  limit also works.
- Avoid network drives for music that plays on air.

## "The interface hit an error"

A drawing error was caught. Audio is not affected. Press **Restart
interface**. Please report it with the logs.

## Logs and crash reports

See [Data and backups](data-and-backups.md) for the log folder. There is one
log file per day, and the last 14 are kept. Crash reports are saved as
`crash-<time>.txt`. Set `RUST_LOG=debug` in the environment for more detail.
Attach both files when reporting a bug.
