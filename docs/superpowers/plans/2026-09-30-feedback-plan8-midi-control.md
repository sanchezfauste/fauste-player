# Feedback Plan 8 — MIDI Control and README Screenshot Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:**
- Control every player from MIDI surfaces (F13): transport buttons, volume
  faders with soft takeover, and LED feedback.
- Hot-plug, MIDI learn, and a Settings > MIDI page.
- A realistic README screenshot (F15).

**Architecture:**
- `fp-model` gains `midi.rs`: the configuration, and a pure `bind` helper
  used by learn. The fader curve (`fader_from_gain`, `gain_from_fader`)
  moves from `fp-app::ui::view` to `fp-model::volume`, so MIDI and the UI
  share it. `view` re-exports it.
- New crate `fp-control` (`forbid(unsafe_code)`). Pure parts:
  - message parsing;
  - `Router` (bindings → `Command`, CC edge detection, soft takeover,
    availability);
  - `Feedback` (desired LED states and their diff);
  - `Learn`.

  An I/O part:
  - a `MidiPorts` trait, with a `midir` implementation and a fake for tests;
  - a `MidiService` thread that connects ports and rescans them every
    `rescan_interval_ms`;
  - input callbacks that push raw bytes into a channel;
  - a loop that routes messages to commands through a `MidiControl` trait
    (the conductor handle), sends LED diffs, and answers learn requests;
  - status published through `ArcSwap`.
- `fp-app` starts the service from `main.rs`, adapts `ConductorHandle` to
  `MidiControl`, and adds the Settings > MIDI section. That section talks to
  the service only through its handle (channels and `ArcSwap`), so the UI
  never blocks.

**Tech Stack:** Rust 2024, `midir` 0.11 (MIT), crossbeam-channel, arc-swap, egui/kittest.

**Spec:** [`docs/superpowers/specs/2026-09-30-operator-feedback-design.md`](../specs/2026-09-30-operator-feedback-design.md) §5 and §6. Roadmap: plan 8.

## Global Constraints

- No `unsafe` in our code. `cargo deny check` after adding `midir`.
- Parsing never panics. Malformed or unknown messages are ignored.
- MIDI threads are never audio threads. Commands go through the same
  controller channel as the UI.
- `config.midi`:
  - `enabled` defaults to false;
  - `rescan_interval_ms` defaults to 2000, range 250–60000;
  - `feedback` defaults to true;
  - `devices` lists output overrides;
  - `bindings` is loaded leniently, and invalid bindings are dropped with a
    warning.
- Button actions are only per-player transport actions: Play/Next, Pause,
  Stop, Fade stop, Restart, Previous and Cue. `Volume(n)` is 1-based.
- A button fires on Note On with velocity > 0, or on a CC rising from below
  64 to 64 or above. R28-unavailable actions are ignored.
- Soft takeover:
  - a fader starts "not picked up" (rule 10);
  - it picks up at or across the current fader travel;
  - it re-arms when the volume changes by other means.
- LEDs, when `feedback` is on:
  - velocity or value 127 means lit, 0 means unlit;
  - Play is lit while Playing;
  - Pause blinks at 500 ms while Paused;
  - Cue is lit while cueing;
  - Stop, Restart and Previous are lit while available;
  - only changes are sent.
- Settings > MIDI:
  - it does no I/O on the UI thread;
  - learn binds the next matching message, moves a binding used elsewhere,
    and Esc cancels;
  - its strings are in both locales.
- Branch `feat/midi-control`. Gate before every commit.

## Review Focus

- **Running status, SysEx and real-time bytes in the stream** are ignored,
  and never mis-bound (Task 2 tests).
- **Two devices with the same port name, or a port renamed by the OS on
  replug.** Reconnection by name stays deterministic (Task 5 tests).
- **A binding whose player no longer exists** (the player count was
  lowered) is ignored (Task 3 test).
- **Volume at the pickup edge.** A fader parked exactly on the model
  position picks up at once, and jitter around it never flaps (Task 4
  tests).
- **Learn while the device is unplugged, or a second Learn before the first
  answers.** The last request wins, and nothing is bound twice (Task 7
  tests).

---

### Task 1: Shared fader curve and MIDI config

- Move `fader_from_gain` / `gain_from_fader` / `volume_db`'s curve helpers to `fp-model::volume` (keep `view` re-exports; existing view tests stay green).
- `fp-model/src/midi.rs`: `MidiConfig { enabled, rescan_interval_ms, feedback, devices: Vec<MidiDevice { input: String, output: Option<String> }>, bindings: Vec<MidiBinding> }`, `MidiBinding { device: String, trigger: MidiTrigger, action: MidiAction }`, `MidiTrigger::{Note { channel: u8, note: u8 }, ControlChange { channel: u8, controller: u8 }, PitchBend { channel: u8 }}`, `MidiAction::{Button(ShortcutAction), Volume(u16)}`; `Config.midi` (`#[serde(default)]`); `validate`: clamp `rescan_interval_ms`, drop bindings with a non-transport button action or channel > 15 / note or controller > 127 (warning), and a Volume bound to a Note; `MidiConfig::bind(&mut self, action, device, trigger)` replaces the action's binding and removes the same trigger (same device) from any other action.
- Tests: defaults; JSON round trip; lenient load drops invalid bindings with warnings; `bind` moves a trigger used elsewhere and replaces the action's previous one.

### Task 2: `fp-control` crate and parsing

- `crates/fp-control` (workspace member, lints like the others, `forbid(unsafe_code)`), deps `fp-model`, `midir`, `crossbeam-channel`, `arc-swap`, `tracing`.
- `message::MidiMessage::{NoteOn { channel, note, velocity }, NoteOff { channel, note }, ControlChange { channel, controller, value }, PitchBend { channel, value: u16 }}`; `parse(bytes: &[u8]) -> Option<MidiMessage>` (Note On velocity 0 = Note Off; data bytes must be < 0x80; SysEx, real-time, system common, too short → None).
- Tests: each kind; velocity 0; 14-bit pitch bend (LSB first); every truncated prefix of a valid message; status bytes in data positions; real-time 0xF8; SysEx.

### Task 3: Router (buttons, availability, volume mapping)

- `Router::new(config: &MidiConfig)`; `on_message(&mut self, device: &str, msg: MidiMessage, state: &AppState) -> Option<Command>`: finds the binding (`device` exact, trigger matching channel and number), maps `Button(a)` through the same mapping as keyboard shortcuts (move `shortcut_command`'s player-action part into `fp-model::shortcuts::player_command(state, action)` so both use it) and drops it unless `command_available`; CC edge per (device, channel, controller); `Volume(n)` handled by Task 4.
- Tests: each button action; a Note Off does nothing; a CC 0→127 fires once, 127→127 does not, 30→60 does not, 60→70 fires; an unavailable action (Stop on a stopped player) sends nothing; a binding for player 5 with 4 players does nothing; another device's same note does nothing.

### Task 4: Soft takeover

- `Pickup` per Volume binding: `position = value / max` (CC 7-bit, Pitch Bend 14-bit); not picked up until the input reaches the model's fader travel (within 1/127) or crosses it between two messages; once picked up, sends `SetVolume(player, gain_from_fader(position))` and remembers the travel it set; if the model travel differs from the last one it set by more than 1/127, it is not picked up any more.
- Tests: no change before crossing; crossing from below and from above; exact landing; jitter of ±1 step around the model travel after pickup keeps sending, never flaps; re-arm after the UI moved the volume; Pitch Bend full range.

### Task 5: Ports, service, hot-plug

- `ports::MidiPorts` trait: `inputs() -> Vec<String>`, `outputs() -> Vec<String>`, `connect_input(name, sink: Sender<(String, Vec<u8>)>) -> Result<Box<dyn Send>, String>` (dropping the box disconnects), `connect_output(name) -> Result<Box<dyn MidiOutput>, String>` with `MidiOutput::send(&mut self, bytes) -> Result<(), String>`; `MidirPorts` implementation (client name "Fauste Player").
- `MidiControl` trait: `model() -> Arc<AppState>`, `send(Command) -> bool`.
- `MidiService::spawn(ports, control) -> std::io::Result<MidiHandle>`: one thread named `fp-midi`; loop with `recv_timeout(50 ms)`: route input → commands; every `rescan_interval_ms` (from the current config) reconnect missing inputs/outputs by name (the first of equal names; ledger), drop vanished ones; feedback tick (Task 6); learn (Task 7); `MidiHandle { status: Arc<ArcSwap<MidiStatus>>, requests: Sender<MidiRequest>, learned: Receiver<Learned> }`, `MidiStatus { inputs: Vec<(String, bool)> }`; disabled config → disconnect all, idle.
- Tests (fake ports + fake control, time driven by the loop's own clock through an injectable `now`): a bound Note becomes a command; unplug (fake removes the port) then replug reconnects and refreshes LEDs; disabled sends nothing; a panicking fake port does not kill the thread (caught, logged).

### Task 6: LED feedback

- `feedback::desired(config, state, blink_on) -> Vec<(String /*output*/, Vec<u8>)>`: for each Button binding with a Note or CC trigger, lit or unlit per the rules; output = override for that device or the same name.
- `Feedback` keeps what was last sent per (output, trigger) and returns only changes; `reset()` after a (re)connection sends everything.
- Tests: Play lit/unlit; Pause blinks every 500 ms; Cue; Stop/Restart/Previous by availability; only diffs are sent; override port; `feedback = false` sends nothing.

### Task 7: Learn

- `MidiRequest::Learn(MidiAction)`, `MidiRequest::CancelLearn`; the service answers with `Learned { action, device, trigger }` for the next matching message (buttons: Note On velocity > 0 or CC ≥ 64; Volume: CC or Pitch Bend); a new Learn replaces a pending one.
- Tests: each case; mismatched message kinds ignored; cancel; replacement.

### Task 8: Settings > MIDI

- `Section::Midi` ("MIDI"), `settings/midi.rs`: enable switch (UpdateConfig), feedback switch, port list from `MidiStatus`, per player the rows Play/Next, Pause, Stop, Fade stop, Restart, Previous, Cue, Volume with the binding text (`"<device> · Note 36 ch 1"`), **Learn** (sends `Learn`, shows "Move a control…", Esc cancels) and **Clear**; each frame drains `learned` and applies `MidiConfig::bind` through `UpdateConfig`.
- `AppUi::with_midi(MidiHandle)`; `SettingsDeps.midi: Option<&MidiHandle>`; without a handle the section says MIDI is unavailable.
- Strings in both locales. Tests (kittest with a handle built from channels): the section lists ports; Learn then a `Learned` answer stores the binding; Clear removes it; Esc cancels learning without closing Settings.

### Task 9: Wiring and docs

- `main.rs`: spawn `MidiService` with `MidirPorts` and the conductor handle adapter; a failure to start MIDI logs and continues.
- Docs: `docs/user/midi.md` (new; linked from the user README), `docs/technical/architecture.md` (crate, thread), `README.md` features, main spec §1 non-goal clarified to network remote control, persistence doc (`config.midi`), `CLAUDE.md` crate table.

### Task 10: README screenshot (F15)

- Extend `examples/demo_session` per spec §5 (players on air, paused, further down with played rows; a cartwall page; realistic titles) and capture it with the `CLAUDE.md` procedure (the capture needs Play pressed: document a manual step, or render with the demo's own conductor); replace `docs/images/main-screen.png` only after the maintainer sees it (ask at the end of the run).

### Task 11: Review, PR

Final review (fresh reviewer, most capable model), PR `feat: MIDI control surfaces`, CI on three OSes (midir builds on each), merge.
