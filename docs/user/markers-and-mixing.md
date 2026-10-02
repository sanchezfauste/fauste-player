# Markers and mixing

Every track has up to five **markers**, in seconds:

| Marker | Meaning | How it is set |
|---|---|---|
| Cue in | Where playback starts | Automatic: just before the first sound above the trim threshold |
| Cue out | Where the track ends | Automatic: just after the last sound above the trim threshold |
| MIX (segue start) | Where the next track starts in continuous mode | Automatic (see below) |
| Outro start | Where the ending of the track begins | Automatic (see below) |
| Intro end | End of the spoken-over introduction | By hand, or from an `INTRO` tag in the file |

Markers set by hand always win: a new analysis never replaces them.

## Editing markers

On a player's waveform:

- **Right-click** where you want a marker and choose **Set cue in here**,
  **Set intro end here**, **Set outro start here**, **Set MIX point here** or
  **Set cue out here**. **Reset markers to automatic** removes the markers you
  placed, and the track is analysed again.
- **Hold Alt** (Option on macOS): handles appear on the markers. Drag one to
  move it; the time is shown while you drag. A drag never moves the playhead.

Cue-in must stay before cue-out. The other markers are kept between them.
Changes to the track on air apply to its next transition at once.

## The INTRO tag

A file can carry its intro time in an `INTRO` tag, as seconds (`12.5`) or
`m:ss`. The name may be written in any case (`INTRO`, `Intro`). It can be an
ID3v2 user text frame (MP3, WAV, AIFF, DSF), a Vorbis,
Opus or FLAC comment, an APE item (WavPack, Monkey's Audio) or an MP4
freeform atom. It is read during analysis. A
manual intro end still wins.

## How the automatic markers are found

The analysis measures the track's peaks in 10 ms steps and its loudness in
short windows (50 ms by default).

- **Cue in / cue out:** only near-silence at the start and end is skipped:
  anything whose peak reaches the *trim threshold* (−60 dBFS by default), on
  either channel, is kept, with a *trim margin* (20 ms by default) around it.
  Soft fade-ins, quiet tails and short sounds are never cut.
- **MIX:** the analysis finds the last point where the track is still less
  than the *segue drop* (15 dB by default) below its own typical loudness, so
  loud and quiet masters with the same fade mix the same way. That point is
  never more than the *maximum mix length* (4 s by default) before cue-out,
  so overlaps stay short.
- **Outro:** the analysis scans backwards from cue-out and finds where the
  level drops more than the *outro level drop* (6 dB by default) below the
  track's median loudness. The outro is never longer than 30 s by default.
- Tracks shorter than the *minimum length for mix and outro markers* (60 s by
  default), such as jingles and ads, get no MIX and no outro.

### Long recordings

A whole programme (one, four or more hours) is analysed like a song, while
it plays if need be: a 4-hour FLAC or Opus file takes under a minute on a
current computer, and memory does not grow with the length. Seeking to any
point, even near the end, is immediate. The waveform and markers are kept
in the analysis cache up to about 16 hours of audio; a longer file works
too, but is analysed again each time Fauste Player starts.

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
- **Use cue-in and cue-out off** (Settings → Players): every player plays
  each track from 0 to the end of the file. Cue-in and cue-out, automatic and
  manual, are kept, and the waveform draws them as dim lines. The MIX point,
  intro and outro still work, within the whole file; **Automatic mix** is a
  separate switch. Countdowns, the duration column, the playlist totals in
  Settings and the remote API times follow the same range. Carts always use
  their own cue-in and cue-out. Changing the setting never restarts, seeks or
  stops a track that is playing; the next track is prepared again.

A track can be played before its analysis finishes. Until then it plays from
the start to the end of the file, with no MIX point: the same way a track plays
when cue markers are off.
