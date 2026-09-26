# Troubleshooting

## No sound

1. Open **Settings → Audio outputs** and press **Test Main** for the player.
   If you hear the tone, check the player's volume fader.
2. If you hear nothing, pick another device or channel pair. Changes to
   outputs take effect after a restart.
3. On Linux, check that no other program holds the ALSA device exclusively.
   A PipeWire or PulseAudio desktop provides the ALSA `default` and `pipewire`
   / `pulse` devices; native PipeWire, PulseAudio and JACK support arrives in a
   later version.

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
