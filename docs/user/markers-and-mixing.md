# Markers and mixing

Every track has up to five **markers**, in seconds:

| Marker | Meaning | How it is set |
|---|---|---|
| Cue in | Where playback starts | Automatic: the first sound above the silence threshold |
| Cue out | Where the track ends | Automatic: the end of the last sound above the silence threshold |
| MIX (segue start) | Where the next track starts in continuous mode | Automatic (see below) |
| Outro start | Where the ending of the track begins | Automatic (see below) |
| Intro end | End of the spoken-over introduction | Manual only (marker editing arrives in a later version) |

Markers set by hand always win: a new analysis never replaces them.

## How the automatic markers are found

The analysis measures the loudness of the track in short windows (50 ms by
default).

- **Cue in / cue out:** silence at the start and end is skipped. Silence
  means below the *silence threshold* (−40 dBFS by default).
- **MIX:** the analysis scans backwards from cue-out and finds the last point
  where the track is still louder than the *mix level* (−18 dBFS by default).
  That point is never more than the *maximum mix length* (8 s by default)
  before cue-out.
- **Outro:** the analysis scans backwards from cue-out and finds where the
  level drops more than the *outro level drop* (6 dB by default) below the
  track's median loudness. The outro is never longer than 30 s by default.
- Tracks shorter than the *minimum length for mix and outro markers* (60 s by
  default), such as jingles and ads, get no MIX and no outro.

All these values are in **Settings → Analysis**. After changing them, the
tracks are analysed again automatically.

## What the player does with them

- **Continuous mode with automatic mix on:** at the MIX point the next track
  starts at full level while the current one fades out until its cue-out.
  The overlap is sample-accurate.
- **Continuous mode without a MIX point,** or with automatic mix off: the next
  track starts exactly at cue-out, with no gap.
- **Single mode**, or **Stop after**: the player stops at cue-out.
- **Pressing Play while on air:** the next track starts at once and the
  current one fades out over the *fade time* (1 s by default).

A track can be played before its analysis finishes. Until then it plays from
the start to the end of the file, with no MIX point.
