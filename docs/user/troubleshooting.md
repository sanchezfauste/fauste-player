# Troubleshooting

## No sound

1. Open **Settings → Audio outputs** and press **Test Main** for the player.
   If you hear the tone, check the player's volume fader.
2. If you hear nothing, pick another device or channel pair. Changes to
   outputs take effect after a restart.
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
- **A `hw:` device cannot be opened (Linux).** A sound server may be holding
  the card. Stop it, or set the server to leave that card alone, and restart
  the application.

## "Output lost" alert

The status bar shows **Output lost: <device>** when a device stops responding.
The players keep counting and mixing on an internal clock, so the automation
does not stall. The device is retried every 2 seconds and takes over again
when it returns. Reconnect the cable or power the interface back on.

## A track shows a warning icon

The file is missing (moved, deleted, unmounted) or cannot be decoded. The
players skip it. Put the file back, then use **Settings → Analysis →
Re-analyse all tracks** to check it again.

## Audio dropouts

- Increase the **buffer size** in Settings (and restart).
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
