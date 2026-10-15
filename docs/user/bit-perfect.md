# Bit-perfect output

A **bit-perfect** device receives each file's samples exactly as they are in
the file: same sample rate, same values, with no resampling, volume change or
mixing.

Bit-perfect output is a **special mode for audiophile playback**: listening
to an archive, feeding a hi-fi or monitoring chain, checking a digital link,
anywhere processing on the computer should be avoided. It is off by default
and it is not meant for a mixed music programme. Read
[What it costs](#what-it-costs) before you turn it on.

## What it costs

To keep every track untouched, a player whose Main output is on a
bit-perfect device plays differently from the others:

- **No crossfades and no segues.** Each track plays to its end (its cue-out,
  or the end of the file with **Use cue-in and cue-out** off), then the next
  one starts. **Automatic mix at the MIX point** and the MIX point are not
  used: the MIX marker is drawn dim and the **Mixing** badge never lights.
  **Play / Next** and **Previous** while a track is on air switch at once,
  with a short click-free stop, instead of fading.
- **A short silence when the sample rate changes.** Between two tracks whose
  sample rates differ, when nothing else sounds on the device, the device
  first plays what it still has buffered (the end of the track), then is
  reopened at the new rate, which leaves a short silence (usually a few tens
  of milliseconds, longer on some devices). Tracks at the same rate follow
  each other with no gap and no overlap. When the rate cannot follow (see
  [What happens on a bit-perfect device](#what-happens-on-a-bit-perfect-device)),
  the next track starts at once, converted to the device's rate.
- **No loudness normalization.** A track on a bit-perfect device always
  plays at unity gain, even with [loudness normalization](loudness.md#bit-perfect-outputs)
  on: the header says **Not normalized**. Other players and carts are
  normalized as usual.
- **The volume is held at 100 %.** The player's volume fader is locked at
  the top: on screen, on [MIDI](midi.md) controllers and over the
  [remote API and OSC](remote-control.md). It goes to 100 % when bit-perfect
  is turned on for the device, and stays there when it is turned off.
- **What still works:** **Fade stop** (it fades out over the fade time),
  **Pause**, **Stop**, **Restart**, the repeat and stop-after marks, Single
  and Continuous modes, Cue (pre-listen, which has no fader and is not affected), carts,
  [DSD output](#dsd), and every player whose Main output is on another
  device, which mixes as usual. A cart can play on the same device: while it
  sounds, the player's next track starts at the device's current rate,
  converted (so it is not bit-perfect), and no silence is inserted.
- **When to use it.** Turn it on when every track must reach the converter
  untouched. Keep it off for an on-air programme that needs mixed
  transitions or level changes. The mode is set per device, so mixed
  players can stay on another device.

## Setting a device bit-perfect

1. In **Settings → Audio outputs**, choose the device explicitly for a
   player's Main output (or the cartwall's). A player left on the system
   default cannot be made bit-perfect. A device that no output uses any
   more loses its bit-perfect switch and its DSD mode the next time the
   application starts.
2. Choose **Advanced** at the top of the section. Under **Per-device
   settings**, turn on **Bit-perfect** next to the device.
3. The device becomes bit-perfect as soon as nothing plays on it (see
   [Settings pending](settings.md#settings-pending)).

The device then starts at its own sample rate when you gave it one in the same
place, else at the global sample rate, and follows each file from there.

The switch is disabled when the device cannot give exclusive access.
- **Linux:** choose an ALSA device whose name starts with `hw:`. It is the
  sound card itself. PulseAudio, PipeWire, JACK and the ALSA `default` or
  `plughw:` devices mix or convert, so they are never bit-perfect.
- **Windows:** choose the device on the **WASAPI** system. It is opened in
  exclusive mode.
  - In the Windows sound settings, the device's **Advanced** properties must
    have *Allow applications to take exclusive control of this device*
    turned on (it is on by default).
  - While it plays, no other program can use the device.
- **macOS:** choose the device on **Core Audio**. It is opened in hog mode.
  - The device's sample rate is set to the track's, and its format to the
    widest integer format it offers at that rate (the settings Audio MIDI
    Setup shows).
  - They are given back when the application stops using the device.
  - Two devices with exactly the same name cannot be made bit-perfect.

## What happens on a bit-perfect device

- **Exclusive access.** Nothing else on the computer can play on the device
  while the application uses it. If exclusive access is refused, the device
  still plays, shared, and the BP badge stays off.
- **The rate follows the file.** The device is reopened at a track's sample
  rate when the track starts:
  - **From silence:** you play a track, resume one loaded paused,
    pre-listen, or fire a cart while nothing plays on the device. Tracks
    that are only waiting (the next track of each player) are prepared again
    at the new rate.
  - **Between two tracks of a player:** the track on air stops (at its end,
    or at once for **Play / Next** and **Previous**), the device reopens and
    the next track starts. This is the short silence of
    [What it costs](#what-it-costs).
  - The device is reopened only once it has played everything it had
    buffered, so the end of the previous track is never cut short. Then the
    reopen takes as long as the device needs to start. Together that is
    usually a few tens of milliseconds; the start is that much later. The
    wait for the buffer is never longer than half a second
    (`tuning.play_out_max_ms` in the configuration file, 0 to 2000 ms),
    whatever the device reports.
- **When the rate cannot follow.** The device keeps its rate, and a track at
  another rate is converted for its whole length (it is not bit-perfect),
  when:
  - something else sounds on the device as the track starts: another
    player (or one paused with a track), a cart, or a pre-listen;
  - the device refused that rate before; it keeps the previous one;
  - the track has not been analysed yet, so its rate is not known.

  The change between the two tracks is still without a crossfade.
- **No processing, when nothing asks for it.** The samples pass unchanged
  while no fade is running (Fade stop, the pause and resume ramps, the short
  click-free stop of a track) and nothing else plays on the same outputs
  (another player, a cart, a test tone). The volume is no concern: a
  bit-perfect output holds it at 100 %.

## DSD

A DSD file normally plays converted to PCM, like any other file. A
bit-perfect device can instead receive the DSD stream unchanged.

**The three modes.** In the Advanced view of Settings → Audio outputs, every
device an output uses has a **DSD** choice under its bit-perfect switch:
- **Convert to PCM** (the default): DSD is converted, as on any other device.
- **DoP** (DSD over PCM): the DSD bits travel inside 24-bit PCM samples, which
  most DSD-capable converters recognise. It works on every system.
- **Native DSD** (Linux only): raw DSD, for ALSA `hw:` devices whose driver
  reports a DSD sample format.

Only the modes the device can take are offered, and a line under the choice
says why the others are not: the device is not connected, bit-perfect is
off, the device cannot be opened exclusively, native DSD needs Linux, or
the device does not take native DSD. A mode saved for a device that cannot
take it now shows as PCM, which is what plays; the saved mode comes back
when the device can take it again. Changing a mode, the mixing setting or
the DSD silence applies once nothing plays on the device, like the other
output settings.

**When DSD goes out unchanged.** All of these must hold when the track
starts:
- the device is bit-perfect, with exclusive access, and its mode is DoP or
  native DSD;
- the track is DSD (DSF or DFF), mono or stereo, and has been analysed (that
  is how its DSD rate is known);
- nothing else plays on the device (another player, a cart, a test tone);
- the track has no [loudness normalization](loudness.md#bit-perfect-outputs)
  gain: one loaded (for example paused) with a gain before the device was
  turned bit-perfect keeps it, so it plays converted, without **DSD** or
  **BP**;
- the device accepts the stream. DoP needs a device rate of the DSD rate
  divided by 16 (176.4 kHz for DSD64, 352.8 kHz for DSD128, 705.6 kHz for
  DSD256) and a 24- or 32-bit format. Native DSD needs a device that takes
  the DSD format at that rate.

When native DSD ends, the device goes back to PCM at the rate it had before
the DSD track, because many converters take native DSD at rates they cannot
play as PCM (no converter plays PCM at the rate DSD512 runs at). A track
that goes on as PCM keeps the DSD stream's rate when the device takes it
as PCM, and otherwise continues at the earlier rate from where it was,
like everything else playing on that device; a fade in progress there
ends at once. The device never stays on a rate it refuses: if no rate opens
(for example, the device was unplugged at that moment), the output is lost
until the automatic retry reopens it at the earlier rate.

Otherwise the track is converted to PCM and the log says why (for example
"something else plays on the device" or "the device refused 705600 Hz").
Pre-listen and carts are always converted.

While DSD goes out unchanged:
- the header badge reads **DSD** instead of **BP**;
- the meters show the level of the PCM conversion of the same track, so they
  work as usual;
- Stop and a fade stop stop the track at once, with no fade, since a DSD
  stream cannot be faded;
- pause and resume also act at once, without a ramp.

**Silence at the edges.** Every start, end and switch to PCM sends DSD
silence first (200 ms by default), so that the converter locks without a
click. The exception is a DSD track that continues a stream of the same kind
and DSD rate whose silence is still running: the converter is still locked, so
it starts with no extra silence. A track therefore starts that much later, and a switch to PCM leaves a
gap of that length (a native DSD device also plays what it has buffered
before it is reopened as PCM, a few milliseconds more). It is **DSD silence** in Settings → Audio outputs, Advanced
(0 to 2000 ms).

**When another source needs the device.** **When another source needs a DSD
output** in Settings → Audio outputs chooses what happens when another
player, a cart or a test tone starts on the same device:
- **Continue the DSD track as PCM** (the default). The stream switches to PCM
  after the DSD silence, and the track goes on, converted, from where it was.
  The same happens to the track that follows by itself (see below).
- **Keep DSD and mute the other sources.** Nothing interrupts the DSD stream.
  Other sources routed to the device are muted until the DSD track ends, and
  the player shows an **Others muted** badge meanwhile. The player's own next
  track starts when the DSD track ends: a PCM track after the DSD silence, a
  DSD track of the same kind and DSD rate at once, continuing the stream.

**An album does not stay DSD under the default setting.** With *Continue the
DSD track as PCM*, only a DSD track that starts on an idle device goes out as
DSD. The tracks the player starts by itself afterwards (at the end of a
track) start from a preload, which is always PCM, so
the device switches to PCM and they play converted. A track you start
yourself (Play, double click) goes out as DSD again when the device is idle
or the previous DSD stream is still in its silence at the same DSD rate. To
keep a whole DSD album as DSD, choose *Keep DSD and mute the other sources*.
Then each track of the player goes out as DSD, and the next one starts when
the previous one ends.

If the device is lost while DSD plays and comes back unable to carry it (for
example without exclusive access), the track goes on as PCM.

## The BP badge

The **BP** badge in the player header lights up (green border and text; dim
grey when off) while the current track reaches its Main device unchanged.
All of these must hold:

- the device is bit-perfect and open with exclusive access;
- the device runs at the track's sample rate;
- the track is lossless integer PCM (WAV, AIFF, FLAC, ALAC, WavPack or
  Monkey's Audio), mono or stereo, and at most 24-bit, and the device
  format holds its sample size (a 24-bit file on a 16-bit device is not
  bit-perfect). DSD is converted, so it never lights BP; when it goes out
  unchanged (see [DSD](#dsd)) the badge reads **DSD** instead;
- the track has been analysed, since that is how its rate and sample size
  are known. Tracks an earlier version analysed get their format once
  analysed again (the start-up notice, or Settings → Analysis), or as soon
  as a player shows them or a cart holds them;
- the track plays at unity gain: a bit-perfect output is never
  normalized, but a track that started with a normalization gain before its
  device was turned bit-perfect keeps that gain until it stops, and BP stays
  off for it;
- the volume is held at 100 % on a bit-perfect output, no fade runs (Fade
  stop, the pause and resume ramps and the short click-free stop of a track
  are the only fades left on such a player), and nothing else plays on the same outputs.

Some files are never shown as bit-perfect:
- **Lossy files** (MP3, AAC, Ogg Vorbis, Opus): their decoded samples are not
  the integer values a device takes.
- **Files above 24 bits:** the mixer works in 32-bit floating point, which
  carries 24 bits exactly.
- **Files with more than two channels:** they are mixed down to stereo.

## Checking it yourself

To verify a chain end to end:

1. Connect the device's digital output (S/PDIF, AES or USB loopback) to a
   recorder that captures bit-exact.
2. Play a lossless test file with nothing else playing.
3. Record it.
4. Compare the recording with the file. For example, with SoX, invert one
   and mix them: `sox -m -v 1 file.wav -v -1 recording.wav diff.wav` after
   aligning their starts. Every sample of the difference must be zero.
5. Check a rate change: in Continuous mode, with **Automatic mix at the MIX
   point** on, play a 44.1 kHz track that has a MIX point followed by a
   96 kHz one. The first plays to its cue-out with no fade; after a short
   silence the converter shows 96 kHz on the second. The BP badge lights on
   both.
6. Then play two tracks at the same rate: the second follows the first with
   no gap and no overlap.

The project's automated tests check the same property inside the
application, on a simulated device.

### DSD on a real converter

The automated tests check DSD on simulated devices only. DoP and native DSD
have not been tried on a real converter by the project. To check one:
1. Set the device to **DoP** (or **Native DSD** on Linux) and play
   a DSD file with nothing else playing. The header must show
   **DSD**, and the converter's own display should show the DSD rate (for
   example DSD64) instead of a PCM rate. A converter that shows a PCM rate
   or plays noise does not recognise the stream: go back to **Convert to
   PCM**.
2. Listen for a click or a burst of noise at the start, at Stop, at the end
   of the track. A click means the converter needs
   a longer **DSD silence** (Settings → Audio outputs, Advanced).
3. Start a cart or another player on the same device, once with each mixing
   setting, and check the behaviour described above.
4. On Linux, to check native DSD without the application, run
   `FAUSTE_NATIVE_DSD_DEVICE=alsa:hw:CARD=<card>,DEV=0 cargo test -p fp-backends --test conformance -- --ignored native_dsd_on_a_real_device`.
   It opens the device in native DSD at DSD64 and plays one second of DSD
   silence. It must pass, and the converter should lock to DSD64.
5. With DSD files in `test-music/`, `cargo test --release -p fp-engine --test dsd_real_music -- --ignored`
   plays them through the engine on a simulated device and compares the
   words with the file's bytes (see [Testing](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/testing.md)).
