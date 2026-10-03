# Bit-perfect output

A **bit-perfect** device receives each file's samples exactly as they are in
the file: same sample rate, same values, with no resampling, volume change or
mixing. It is useful for monitoring chains and digital links, where any
processing on the computer should be avoided.

## Setting a device bit-perfect

1. In **Settings → Audio outputs**, choose the device explicitly for a
   player's Main output (or the cartwall's). A player left on the system
   default cannot be made bit-perfect.
2. Under **Bit-perfect devices**, turn on the switch next to the device.
3. Restart the application.

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
- **The rate follows the file.** When nothing is playing on the device and a
  track at another sample rate starts, the device is reopened at that rate.
  - This happens when you play a track, resume one loaded paused, pre-listen,
    or fire a cart. Tracks that are only waiting (the next track of each
    player) are prepared again at the new rate.
  - The reopen takes as long as the device needs to start (usually a few tens
    of milliseconds). The start is that much later.
  - While something is playing on the device, the rate never changes. A track
    at another rate that starts then (for example a 48 kHz track mixed into
    after a 44.1 kHz one, or a track started while another player or a cart
    plays on the same device) is converted for its whole length, and is not
    bit-perfect.
  - If the device refuses a rate, it keeps the previous one and the track is
    converted.
- **No processing, when nothing asks for it.** The samples pass unchanged
  while all of these hold:
  - the player's volume is at 100 %;
  - no fade is running;
  - nothing else plays on the same outputs (another player, a cart, a test
    tone).

## DSD

A DSD file normally plays converted to PCM, like any other file. A
bit-perfect device can instead receive the DSD stream unchanged.

**The three modes.** Under each bit-perfect device, Settings → Audio outputs
has a **DSD** choice, next to the device's bit-perfect switch:
- **Convert to PCM** (the default): DSD is converted, as on any other device.
- **DoP** (DSD over PCM): the DSD bits travel inside 24-bit PCM samples, which
  most DSD-capable converters recognise. It works on every system.
- **Native DSD** (Linux only): raw DSD, for ALSA `hw:` devices whose driver
  reports a DSD sample format.

Only the modes the device can take are offered. Changing a mode, the mixing
setting or the DSD silence needs a restart, like the other output settings.

**When DSD goes out unchanged.** All of these must hold when the track
starts:
- the device is bit-perfect, with exclusive access, and its mode is DoP or
  native DSD;
- the track is DSD (DSF or DFF), mono or stereo, and has been analysed (that
  is how its DSD rate is known);
- the player's volume is at 100 %;
- nothing else plays on the device (another player, a cart, a test tone);
- the device accepts the stream. DoP needs a device rate of the DSD rate
  divided by 16 (176.4 kHz for DSD64, 352.8 kHz for DSD128, 705.6 kHz for
  DSD256) and a 24- or 32-bit format. Native DSD needs a device that takes
  the DSD format at that rate.

Otherwise the track is converted to PCM and the log says why (for example
"something else plays on the device" or "the device refused 705600 Hz").
Pre-listen and carts are always converted.

While DSD goes out unchanged:
- the header badge reads **DSD** instead of **BP**;
- the meters show the level of the PCM conversion of the same track, so they
  work as usual;
- the volume must stay at 100 %: the fader's tooltip says so. Moving it
  switches the track to PCM (see below);
- Stop and a fade stop stop the track at once, with no fade, since a DSD
  stream cannot be faded. Pressing Play on another track while it plays cuts
  it the same way instead of crossfading;
- pause and resume also act at once, without a ramp.

**Silence at the edges.** Every start, end and switch to PCM sends DSD
silence first (200 ms by default), so that the converter locks without a
click. The exception is a DSD track that continues a stream of the same kind
and DSD rate whose silence is still running: the converter is still locked, so
it starts with no extra silence. A track therefore starts that much later, and a switch to PCM leaves a
gap of that length. It is `outputs.dsd_silence_ms` in the configuration file
(0 to 2000).

**When another source needs the device.** **When another source needs a DSD
output** in Settings → Audio outputs chooses what happens when another
player, a cart or a test tone starts on the same device (moving the player's
own fader is the exception: it always switches the track to PCM):
- **Continue the DSD track as PCM** (the default). The stream switches to PCM
  after the DSD silence, and the track goes on, converted, from where it was.
  The same happens to the track that follows by itself (see below).
- **Keep DSD and mute the other sources.** Nothing interrupts the DSD stream.
  Other sources routed to the device are muted until the DSD track ends, and
  the player shows an **Others muted** badge meanwhile. The player's own next
  track does not overlap: it starts when the DSD track ends, after the DSD
  silence, with no crossfade or segue. Moving the fader still switches the
  track to PCM.

**An album does not stay DSD under the default setting.** With *Continue the
DSD track as PCM*, only a DSD track that starts on an idle device goes out as
DSD. The tracks the player starts by itself afterwards (at the end of a
track, a segue or a crossfade) start from a preload, which is always PCM, so
the device switches to PCM and they play converted. A track you start
yourself (Play, double click) goes out as DSD again when the device is idle
or the previous DSD stream is still in its silence at the same DSD rate. To
keep a whole DSD album as DSD, choose *Keep DSD and mute the other sources*.
Then each track of the player goes out as DSD, and the next one starts when
the previous one ends.

If the device is lost while DSD plays and comes back unable to carry it (for
example without exclusive access), the track goes on as PCM.

## The BP badge

The **BP** badge in the player header lights while the current track reaches
its Main device unchanged. All of these must hold:

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
- volume is 100 %, no fade runs, and nothing else plays on the same outputs.

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
2. Play a lossless test file at 100 % with nothing else playing.
3. Record it.
4. Compare the recording with the file. For example, with SoX, invert one
   and mix them: `sox -m -v 1 file.wav -v -1 recording.wav diff.wav` after
   aligning their starts. Every sample of the difference must be zero.

The project's automated tests check the same property inside the
application, on a simulated device.

### DSD on a real converter

The automated tests check DSD on simulated devices only. DoP and native DSD
have not been tried on a real converter by the project. To check one:
1. Set the device to **DoP** (or **Native DSD** on Linux), restart, and play
   a DSD file at 100 % with nothing else playing. The header must show
   **DSD**, and the converter's own display should show the DSD rate (for
   example DSD64) instead of a PCM rate. A converter that shows a PCM rate
   or plays noise does not recognise the stream: go back to **Convert to
   PCM**.
2. Listen for a click or a burst of noise at the start, at Stop, at the end
   of the track and when moving the fader. A click means the converter needs
   a longer `outputs.dsd_silence_ms`.
3. Start a cart or another player on the same device, once with each mixing
   setting, and check the behaviour described above.
4. On Linux, to check native DSD without the application, run
   `FAUSTE_NATIVE_DSD_DEVICE=alsa:hw:CARD=<card>,DEV=0 cargo test -p fp-backends --test conformance -- --ignored native_dsd_on_a_real_device`.
   It opens the device in native DSD at DSD64 and plays one second of DSD
   silence. It must pass, and the converter should lock to DSD64.
5. With DSD files in `test-music/`, `cargo test --release -p fp-engine --test dsd_real_music -- --ignored`
   plays them through the engine on a simulated device and compares the
   words with the file's bytes (see [Testing](../technical/testing.md)).
