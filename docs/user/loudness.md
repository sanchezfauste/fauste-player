# Loudness normalization

Tracks come from many sources and are mastered at very different levels: a
recent pop single can sound twice as loud as a 1970s recording or a spoken
promo. **Loudness normalization** plays every track and cart at the same
perceived loudness, so the operator does not ride the fader from one track
to the next.

It is **off by default**. Turn it on in
[Settings → Loudness](settings.md#loudness).

![Settings, Loudness: normalization on, with the target, the true-peak ceiling, the maximum boost and the maximum trim at their defaults](../images/guide/settings-loudness.png)

## How it works

**A fixed gain per track.** Each track gets one gain, in dB, decided when
the track is loaded and kept until it stops. Nothing is compressed or
limited: the whole track is made louder or quieter by the same amount, so
its own dynamics stay as they were mastered. The player's fader works on
top of it, as before.

**The measurement.** The background analysis measures each file once:

- its **integrated loudness**, in LUFS: how loud the whole file sounds, as
  ITU-R BS.1770 and EBU R128 define it (quiet passages and silence are
  gated out, so a long fade-out or a pause does not lower it);
- its **true peak**, in dBTP: the highest level the signal reaches between
  samples once it is converted to analogue.

New files are measured with their first analysis. Files added before
loudness was measured are measured by a **background pass** that runs while
the normal analysis has nothing to do, without touching their markers. It
runs whether normalization is on or off, so the measurements are ready when
you turn it on. **Re-analyse** (in the playlist menu) measures a file again.
A file shorter than
0.4 s is never measured, and a silent file has no loudness: both play
unchanged.

**The gain.** The gain brings the measured loudness to the **Target**, plus
the track's own **Trim** (see [Per track](#per-track-exclude-and-trim)):

> gain = target − measured loudness + trim

A track louder than the target is turned down by as much as it needs. A
quieter track is turned up, but never by more than the **Maximum boost**,
and never so much that its true peak would go above the **True-peak
ceiling**: there is no limiter, so the ceiling is what keeps a boosted
track from clipping. A track whose true peak is already above the ceiling
is not boosted at all.

Examples, with the defaults (target −16 LUFS, ceiling −1 dBTP, maximum
boost 12 dB):

| Track | Measured | True peak | Gain |
|---|---|---|---|
| A loud modern master | −9.0 LUFS | +0.5 dBTP | −7.0 dB |
| A quieter older master | −20.0 LUFS | −10.0 dBTP | +4.0 dB |
| A quiet track with high peaks | −22.0 LUFS | −3.0 dBTP | +2.0 dB (limited by the ceiling) |
| A very quiet recording | −35.0 LUFS | −20.0 dBTP | +12.0 dB (limited by the maximum boost) |

**When the gain is decided.** When a track is loaded: prepared as a
player's next, started, loaded paused, pre-listened on CUE, or fired as a
cart. A track that is playing or paused **keeps its gain** until it stops,
through seeks and pauses. Changing the settings, a track's measurement or
its trim applies the next time the track is loaded; the next track already
waiting on a player is prepared again with the new gain, so it starts with
it.

**Pre-listen.** CUE and cart pre-listen play a track with the same gain it
gets on air, so what you hear in the headphones matches the programme.

**Meters.** The meters and their loudness readout measure after the gain,
so they show the normalized level. The meters' own **Loudness target**
(Settings → Meters) only colours the readout; it does not change any gain.

## Bit-perfect outputs

A player or cart whose output is [bit-perfect](bit-perfect.md) is **never
normalized**: it always plays at unity gain (0 dB), whatever the settings,
because any gain would change the samples the device receives. With
normalization on, its header shows a grey **Not normalized** badge whose
tooltip says "Bit-perfect output: tracks play unaltered." The same holds
for pre-listen on a bit-perfect Cue device, while the player's Main output,
if it is not bit-perfect, still gets the gain.

A track that is already playing (or paused) when its device is turned
bit-perfect keeps the gain it started with, and the **BP** badge stays off
for it. The next track plays at 0 dB and bit-perfect. A DSD track in that
situation plays converted to PCM with its gain, never as an unchanged DSD
stream.

## The gain in the player header

While a player is playing or paused, a badge left of **BP** shows the gain
of the track on air. Hover it for the reason.

| Badge | Meaning |
|---|---|
| **+4.0 dB**, **−7.0 dB** (blue) | The track is normalized with this gain. The tooltip gives the target, the measured loudness and the track's trim, and says when the true-peak ceiling or the maximum boost held a boost back. When the settings, the measurement or the trim changed after the track was loaded, it says only that the track keeps the gain set when it was loaded. |
| **Not normalized** (amber) | Normalization is on, but the track has no measurement yet (or is too short to measure): it plays unchanged. |
| **Not normalized** (grey) | The output is bit-perfect, or the track is excluded from normalization. |
| No badge | Normalization was off when the track started, the track is silent, or the player is stopped. A bit-perfect player shows none either while normalization is off. |

The badge shows the gain the player asked for when the track started.

## Per track: exclude and trim

Some tracks should not follow the target: a station ID mastered loud on
purpose, a test tone, a track whose measurement does not match how it
sounds (speech often sounds louder than music at the same LUFS). Each track
has two settings for that:

- **Exclude from normalization:** the track always plays unchanged.
- **Trim:** dB added to the track's gain, from − to + the **Maximum trim**
  (Settings → Loudness), in 0.5 dB steps. Double-click the slider to put it
  back to 0. The trim is dimmed while the track is excluded.

Where to change them:

- **On a waveform:** right-click a player's waveform (or its CUE window's)
  and choose **Loudness…**. The popup names the track and shows its
  measurement. It keeps editing that track even when the player moves on to
  another one; close it with its **×** button. It closes by itself if the
  track is removed.
- **In the cart editor:** Settings → Cartwall, click a cart: the same two
  controls are under **Stop other carts when fired**, with the measurement
  under the label.
- **Remotely:** `PUT /api/v1/tracks/<id>/loudness` (see
  [Remote control](remote-control.md)).

The settings belong to the track and are saved with it. A change applies
the next time the track is loaded, like any other change.

**The track tooltip** of the playlist table shows the measurement
("Loudness: −14.2 LUFS, true peak −0.8 dBTP", or "Loudness: not measured")
and, when it is not the default, the track's setting ("Excluded from
normalization" or "Trim +1.5 dB"). The trim shown is the one in use: when a
lowered **Maximum trim** limits a trim saved earlier, the line also gives
the saved value ("Trim +6.0 dB (saved +10.0 dB, limited by the maximum
trim)"). The gain badge's tooltip and the **Trim** slider show the trim in
use too.

## Choosing the settings

- **Target.** −16 LUFS suits most music programmes on air and online.
  −23 LUFS (EBU R128) or −24 LUFS (ATSC A/85) leave more headroom, but quiet
  tracks then need larger boosts, which the ceiling and the maximum boost
  may hold back. Choose a target close to how loud your material already is.
- **True-peak ceiling.** −1 dBTP leaves room for the conversion and for
  lossy encoders downstream. Lower it when a stream encoder clips.
- **Maximum boost.** Keeps a very quiet recording (or a mostly silent one)
  from being raised into noise. 0 dB only turns loud tracks down.
- **Maximum trim.** How far the per-track trim may go. Lowering it limits
  trims already set without changing them, until you next edit that
  track's loudness: the edit then saves the limited trim.
