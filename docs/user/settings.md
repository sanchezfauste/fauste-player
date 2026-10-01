# Settings

Open **Settings** in the top bar. Close it with **Close** or `Esc`. Most
changes apply at once and are saved automatically.

The window has one size (900 × 640, smaller on a small screen) whatever the
section, and the section scrolls inside it. Every section lines its labels up
in one column.

The Players, Meters, Analysis and Keyboard shortcuts sections have a
**Restore defaults** button in their header. It asks for confirmation and then
resets only that section (Players keeps the number of players and the
language; Shortcuts has no other reset button). Audio outputs, Playlists,
Cartwall, MIDI and Remote have none.

## Restart pending

Some changes only take effect when the application starts again: the audio
system, sample rate, buffer size, the Main and Cue outputs (players and
cartwall) and the bit-perfect devices. The number of players is not one of
them: it applies at once.

The limits and the engine tuning also apply at the next start, but they are
edited in the configuration file with the application closed (see
[Data and backups](data-and-backups.md)), so they never show as pending.

While one of these is waiting, the Settings footer says "Some changes take
effect after a restart." and offers **Restart now**, and the top bar shows a
**Restart pending** pill. Hover the pill to see what is waiting. A short
notice (for example, that a setting was saved) can take the place of the
footer text for a moment; **Restart now** stays. Both do the same thing:

- When nothing is on air, **Restart now** (or the pill) restarts at once.
- When something is on air, the window that lists what is sounding appears,
  with **Stop and restart** or **Cancel**.

The session is saved first and the audio and MIDI control stop, then the
application starts again with the same data folder (`FAUSTE_HOME`), and
nothing goes on air by itself afterwards. If the application cannot start
again (in a Flatpak, also when the new one does not start in time), it says so;
start it from your applications menu.

## Audio outputs

Changes in this section wait for a restart: see
[Restart pending](#restart-pending).

| Setting | Meaning |
|---|---|
| Audio system | The silent test backend is not listed, unless it is the one configured: it then shows as "No output (silent)". Linux: PipeWire (in builds that include it), PulseAudio, JACK or ALSA. Windows: WASAPI, ASIO (in builds that include it) or JACK. macOS: Core Audio or JACK. Systems missing on this computer, or with no output device (a JACK server that is not running), are shown as unavailable. "System default" uses the first available one in that order. |
| Sample rate | The rate every output runs at; files are converted to it with high-quality resampling. Bit-perfect devices start at this rate and then follow the files. |
| Buffer size | Frames per audio block; the resulting latency is shown below it |
| Outputs per player | For each player, a **Main** (on-air) device and a **Cue** (pre-listen) device, each with a channel pair. A sound card that offers several output profiles (ALSA lists front, surround, direct hardware…) shows each as *card — profile*; two entries that would still read the same get their device id in brackets. Multichannel interfaces can carry several players on different pairs. |
| Test Main / Test Cue | Plays a short tone (1 kHz on Main, 440 Hz on Cue, 1.5 s, −18 dBFS) on the chosen output, so you can check the wiring before going on air |
| Cartwall | The cartwall's Main and Cue outputs. Main defaults to the system output. Without a Cue there is no cart pre-listen. |
| Bit-perfect devices | One switch per device chosen above. A bit-perfect device is opened with exclusive access and follows each file's sample rate while nothing plays on it. The switch is disabled where the device cannot give exclusive access. See [Bit-perfect output](bit-perfect.md). |

A Cue output never falls back to the output Main uses. A Cue that names a
device on an audio system this computer does not have, or the same output as
Main, means "no cue".

If a device disappears while playing, the players keep their timelines, and
the device is reopened when it comes back (see
[Troubleshooting](troubleshooting.md)).

## Players

| Setting | Default | Meaning |
|---|---|---|
| Language | System | Interface language |
| Number of players | 4 | Columns on the main screen (a player on air cannot be removed) |
| Default mode | CONT | The mode players start in |
| Fade time | 1000 ms | Used by Play while on air and by Fade stop |
| Automatic mix at the MIX point | On | Overlap tracks in continuous mode |
| End-of-track warning | 10 s | When the countdown starts blinking red |

## Meters

Changes apply at once. Settings shows only what the chosen meter type
uses: the EBU, DIN and VU meters have the scale, red zone and behaviour
their standard fixes (only the alignment level is set), and a K-System
meter's alignment is its own 0. A value you set is kept for when you
choose that type again.

| Setting | Default | Meaning |
|---|---|---|
| Meter type | Digital peak | How the bar rises and falls, and its scale, after a standard (see below) |
| Rise time, Fall rate | 5 ms, 11.8 dB/s | Only for **Custom**. The rise time is an integration time: a tone burst that long reads 2 dB low; 0 shows every peak. |
| True peak | Off | Digital peak, custom and K-System only. Measure between samples, with the 4× oversampling filter ITU-R BS.1770 publishes. It shows peaks that exceed 0 dBFS after conversion, which a sample-peak meter misses. As the standard allows, an isolated one-sample click can read up to about 0.3 dB below its sample value. |
| Scale floor | −60 dBFS | The bottom of the digital scale (digital peak and custom). The other meters show the range their standard gives. |
| Peak hold | 2 s | Digital peak, custom and K-System only: how long the highest level stays lit; 0 turns it off. Programme meters and the VU have no hold. |
| Alignment level | −18 dBFS | All but the K-System. Marked on the scale (EBU R68). It is also where the EBU TEST mark, the DIN −9 mark and 0 VU sit. |
| Warning from | −9 dBFS | Yellow from here (EBU permitted maximum), for the digital peak and custom meters |
| Danger from | −3 dBFS | Red from here, for the digital peak and custom meters. The others turn red where their scale does: VU from 0 VU, EBU and DIN PPM from the permitted maximum (EBU +9, DIN 0), the K-System from +4. |
| Loudness readout | Short-term | The loudness under the meter: off, momentary (last 400 ms) or short-term (last 3 s), EBU R128 |
| Loudness target | −23 LUFS | The readout is green within ±1 LU (EBU R128) |

| Meter type | Standard | Behaviour |
|---|---|---|
| Digital peak | IEC 60268-18 | Shows every peak at once; falls 20 dB in 1.7 s |
| EBU PPM | IEC 60268-10 type IIb | Peaks shorter than about 10 ms read lower (a 10 ms tone burst reads about 1.6 dB low, a 0.5 ms one about 18 dB low), within EBU Tech 3205's tolerances; falls 24 dB in 2.8 s |
| DIN PPM | IEC 60268-10 type I | The same with a 5 ms integration time; falls 20 dB in 1.5 s |
| VU | IEC 60268-17 | The average level, with the needle movement of a VU meter: 99 % in 300 ms, with a slight overshoot; a sine reads its peak level |
| K-20, K-14, K-12 | K-System | Two sections: the average (RMS, 600 ms) as the solid bar and the peak (falls 26 dB in 3 s) dimmed above it. 0 is 20, 14 or 12 dB below full scale; green below 0, amber from 0 to +4, red above. K-12 suits broadcast, K-14 and K-20 more dynamic programme. |
| Custom | — | Your rise time and fall rate |

Each meter uses the scale of its standard, with its marks between the
channels:

| Meter | Scale |
|---|---|
| Digital peak, custom | −60 … 0 dBFS, marks every 10 dB down to −40 and every 5 dB above; the top 20 dB take half the height |
| EBU PPM | −12 … +12 around the alignment level (TEST), every 4 dB; quieter levels rest at the bottom |
| DIN PPM | −50 … +5, where 0 is 9 dB above the alignment level (−9 dBFS by default) |
| VU | −20 … +3 VU, 0 VU at the alignment level; the bar moves in proportion to the voltage, like the needle |
| K-System | from +20, +14 or +12 (0 dBFS) down to −60; even in dB down to −24 |

## Analysis

The thresholds described in [Markers and mixing](markers-and-mixing.md).
**Re-analyse all tracks** runs the analysis again for the whole library;
manual markers are kept.

After an update whose analysis has changed, tracks analysed by the earlier
version keep their markers and waveforms, which still work. At start,
Fauste Player says how many there are and offers **Analyse now** or
**Later**; **Analyse outdated tracks (N)** here does the same at any time.
The tracks on the players are brought up to date anyway, as they are
shown. Tracks whose file is missing are not counted until the file is
back.

## Playlists

- **Music folder:** where the file dialogs start.
- **New playlist**, **rename** (edit the name and press Enter; Esc cancels)
  and **delete** (trash icon).
- **Import M3U / PLS…** creates a new playlist from a playlist file. **M3U**
  on each row exports it as M3U8. See [Playlists](playlists.md).

**Language:** System, English or Español. The interface switches at once.

## Cartwall

Pages, grid size, the cart editor, and cart page import and export. See
[Cartwall](cartwall.md).

## Keyboard shortcuts

See [Keyboard](keyboard.md).

## MIDI

Turn MIDI control surfaces on, see the input ports, and learn a control
for each player action. See [MIDI control surfaces](midi.md).

## Remote

Remote control over the network, for web pages, phone apps, automation and
control surfaces. See [Remote control](remote-control.md).

- **Allow remote control over HTTP**, its **address** and **port**, and a
  line that says whether it is listening.
- **Token**, required beyond this computer. **Generate** makes a random one,
  **Show** reveals it, and **Copy** puts it on the clipboard. A warning
  appears when the address reaches beyond this computer and there is no
  token.
- **Web pages allowed to use the API**: one origin per line.
- **Allow OSC control**, its **address** and **port**, and the **senders
  allowed** (addresses or subnets, one per line).
- **Publish times every**: how often elapsed and remaining times are sent
  while something plays.

Text fields apply when you leave them. An invalid value is corrected, and
the field shows what was kept.
