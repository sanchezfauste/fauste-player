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
- **Windows and macOS:** exclusive modes (WASAPI exclusive, Core Audio hog
  mode) are coming in a later version.

## What happens on a bit-perfect device

- **Exclusive access.** Nothing else on the computer can play on the device
  while the application uses it. If exclusive access is refused, the device
  still plays, shared, and the BP badge stays off.
- **The rate follows the file.** When nothing is playing on the device and a
  track at another sample rate starts, the device is reopened at that rate.
  - This takes as long as the device needs to start (usually a few tens of
    milliseconds). The start is that much later.
  - While something is playing, the rate never changes. A track at another
    rate that starts during a mix (for example a 48 kHz track mixed into after
    a 44.1 kHz one) is converted for its whole length, and is not bit-perfect.
  - If the device refuses a rate, it keeps the previous one and the track is
    converted.
- **No processing, when nothing asks for it.** The samples pass unchanged
  while all of these hold:
  - the player's volume is at 100 %;
  - no fade is running;
  - nothing else plays on the same outputs (another player, a cart, a test
    tone).

## The BP badge

The **BP** badge in the player header lights while the current track reaches
its Main device unchanged. All of these must hold:

- the device is bit-perfect and open with exclusive access;
- the device runs at the track's sample rate;
- the track is lossless (WAV, FLAC, AIFF, ALAC), and the device format holds
  its sample size (a 24-bit file on a 16-bit device is not bit-perfect);
- the track has been analysed, since that is how its rate and sample size
  are known;
- volume is 100 %, no fade runs, and nothing else plays on the same outputs.

Lossy files (MP3, AAC, Ogg Vorbis, Opus) are never shown as bit-perfect:
their decoded samples are not the integer values a device takes.

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
