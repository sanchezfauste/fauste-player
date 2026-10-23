# itr07 plan L: a MIDI controller plugged in before start does not respond (J6)

## Goal

On Windows 10 (WinMM) a controller plugged in before the program starts is
reported "MIDI input connected" about 0.2 s after start, yet no control acts
and no LED changes. Unplugging and plugging it back makes it work. Find the
cause or, failing that, make the next field log decide it and give the
operator a way out that needs no cable.

## Findings (from the code; no Windows machine here)

- The start-up path and the hot-plug path are the same code: `MidiCore::scan`
  connects the input, then the output, from the `fp-midi` thread, through the
  same channel. Each port is opened once; nothing else in the program
  enumerates or opens MIDI ports (the settings only read `MidiStatus`).
- midir's WinMM backend keeps no client state (`MidiInput::new` is empty),
  finds the port again by name and device interface id at connect time, and
  starts the input (`midiInStart`) before returning. Rescans only query
  device capabilities. No stale index, no double open, no replaced callback.
- The router keeps no per-connection state that a replug resets.
- So the difference is outside the service: the first open in a fresh
  process, or the device or driver left in a bad state (for example by the
  previous run's close), which a USB replug re-initialises. The log cannot
  tell them apart today.

## Tasks

1. **Diagnostics** (`crates/fp-control/src/service.rs`, `router.rs`,
   `ports.rs`). Not on any real-time path: everything is logged from the
   `fp-midi` thread, never from midir's callback.
   - The port lists when they change (info), so a second port with the same
     name, or a missing one, shows.
   - The system id of each opened port and how many ports share its name
     (info; a warning when several do).
   - Input and output disconnected, with the reason (info).
   - Output connected (info); output not connected (warn).
   - The first message received on each input connection (info, once).
   - Ignored messages with the reason (debug): unreadable, learning, and
     each router refusal (`Router::route` returns `Result<Command,
     Ignored>`; `on_message` stays as its `ok()`).
   - Tests: router reasons; the service logs the first message once per
     connection and the drop (captured subscriber).
2. **Reconnect** (`MidiRequest::Reconnect`, Settings › MIDI button
   "Reconnect ports"). Closes every port and opens them again on the next
   pass, LEDs refreshed. It is the replug without the cable, and the field
   test that tells the two causes apart: if it revives the controller, the
   first connection was dead in the program; if not, the device or driver
   needs re-initialising.
   - Tests: service reopens the ports; the button sends the request.
   - Strings `midi-reconnect`, `midi-reconnect-tip` in all 11 locales.
3. **Docs**: spec §6.3/6.4, `docs/user/midi.md`, `docs/user/settings.md`,
   `docs/user/troubleshooting.md`.
