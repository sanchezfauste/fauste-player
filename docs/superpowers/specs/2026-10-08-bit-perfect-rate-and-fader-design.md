# Bit-perfect: hard transitions, rate cuts and the locked fader — Design Spec

- **Date:** 2026-10-08, amended 2026-10-13 (D8: no crossfades on
  bit-perfect devices) and 2026-10-14 (BR16: the device plays out before
  a rate cut closes it).
- **Status:** Approved (2026-10-13); implemented (the plans
  `2026-10-13-bit-perfect-plan1` to `plan4`).
- **Amends:** [the bit-perfect spec](2026-09-26-phase4-bit-perfect-design.md)
  (**B3.3 is replaced** by §4; B1 "unaltered source", B7 out of scope),
  [the main spec](2026-09-25-fauste-player-design.md) §3.2 rules 5 (Play
  while Playing) and 11 (Continuous mode) and R24 (Previous) for a player
  whose Main output is bit-perfect (§4),
  [the live settings spec](2026-10-07-live-settings-design.md) (§8 and
  §13 "the bit-perfect rate change at transitions"), and
  [the second operator feedback spec](2026-10-01-operator-feedback-2-design.md)
  O25 (the fader that leaves unity, §5.5).
- **Numbering:** transition rules are `BR<n>`, fader rules `BF<n>`. Each
  rule is testable and gets at least one test (§8).
- **Line numbers** are those of `master` at the date above (`c31aba4`);
  each reference also names the function.

---

## 1. Goal

Bit-perfect output is a **special mode for audiophile playback**, not the
default: a station turns it on for a device when it wants every track to
reach the converter exactly as it is in the file, and accepts what that
costs. Three things such a station expects and does not get today:

1. **No track is altered by a transition.** A segue or a crossfade mixes
   two tracks, so neither is bit-perfect while they overlap, and the
   incoming one keeps the outgoing one's rate (B3.3) for its whole length.
   From now on a player whose Main output is on a bit-perfect device
   **never crossfades or segues between tracks**: every track change is a
   **hard transition** (the current track ends, the next one starts, with
   no overlap).
2. **The device follows the music, track after track.** Today B3.3 keeps a
   sounding bus at its rate, and in continuous play the bus always sounds,
   so the rate of the first track of the session is in practice the rate
   of every track after it: a 96 kHz track after a 44.1 kHz one is
   resampled for its whole length. From now on, when the next track's rate
   differs and nothing else sounds on the device, the hard transition is a
   **rate cut**: the device reopens at the new rate between the two tracks.
   A short silence is the documented cost. At the same rate the next track
   follows straight after, with no gap.
3. **No level change reaches a bit-perfect output by accident.** A fader
   below 100 % alters every sample, so the track is not bit-perfect and the
   BP badge stays off, often without the operator noticing. From now on the
   fader of a player whose Main output is bit-perfect is **held at 100 %**
   in the interface, on MIDI surfaces and through the remote API and OSC.

## 2. Maintainer decisions (binding, 2026-10-03, 2026-10-08 and 2026-10-13)

| # | Decision |
|---|---|
| D1 | **Rate cut.** On a bit-perfect device, when the next track's sample rate differs from the device's running rate and the only thing sounding on that device is this player's outgoing track, the device reopens at the new rate between the two tracks of the hard transition (D8). A short gap is accepted. *(2026-10-13: the transition itself is now set by D8 for every track change; D1 only adds the reopen.)* |
| D2 | If another player or a cart sounds on that device, nothing on air is cut: the device does not reopen and the next track is resampled to the running rate, so it is not bit-perfect (today's behaviour, B3.3). The transition is still hard (D8): no crossfade. |
| D3 | **Fader locked.** For a player whose Main output is on a bit-perfect device the fader is held at 100 % and disabled in the interface, on MIDI and in the remote API and OSC. |
| D4 | Fades that are not a transition between two tracks stay: Fade stop, the pause and resume ramps and the declicks (BR9). While they run the source is not bit-perfect (as today). *(2026-10-13: segue and Play-while-playing crossfades no longer stay on a bit-perfect Main; superseded by D8.)* |
| D5 | When bit-perfect is turned on for a device (under live settings, when the device is idle), the faders of the players on it go to 100 % at that moment, so there is no level jump on air. When bit-perfect is turned off, the fader is unlocked and stays at 100 %. |
| D6 | Rules live in `fp-model` as pure functions, one test per rule; the engine and the interface only execute and display (CLAUDE.md rule 7). Nothing goes on air by itself (rule 10). |
| D7 | **No switch for the cut** (2026-10-13). The gap is the price of bit-perfect output, which a station turns on only when it wants audiophile playback; such a device does not crossfade across a rate change and accepts a silence there. Every bit-perfect device cuts; there is no per-device option. |
| D8 | **Never a crossfade or a segue on a bit-perfect device** (2026-10-13). A player whose Main output is on a bit-perfect device never crossfades or segues between tracks. Every transition behaves the same way, whatever the rates: the current track plays to the end of its play range (its cue-out with markers on; a manual Next or Previous stops it at once with the usual declick), then the next track starts. When the rate changes and nothing else sounds on the device, the device reopens at the new rate in between (D1), so there is a short silence; at the same rate the next track follows straight after (no overlap). The silence and the missing crossfades are the accepted price of bit-perfect output, which is a special mode for audiophile playback, not the default. **Supersedes** the parts of D1 and D4 (and of the former BR8) that let tracks at the same rate segue and crossfade. |

## 3. Terms

| Term | Meaning |
|---|---|
| **Bit-perfect device** | As B1: a device listed in `outputs.bit_perfect`. For the rules below, the value the device **runs** with counts: `live.devices[d].bit_perfect` (live settings §4.1), not the wanted value of `Config`. A pending "bit-perfect on" or "off" changes nothing until it is applied. |
| **Main device of a player** | `live.placement[Holder::PlayerMain(p)]`: where the engine really put the player's Main (live settings, "Placement"). A player the engine has not placed yet has none. |
| **Bit-perfect player** | A player whose Main device is a bit-perfect device (BR2). Its Cue route does not count. |
| **Stream rate** | The rate a device's stream runs at now (`Bus::sample_rate`). On a bit-perfect device it follows the files (B3) and is not the configured rate. New in the model: `live.stream_rates` (BR1). |
| **Refused rates** | The rates a device refused (`Bus::refused_rates`, `crates/fp-engine/src/bus.rs` ~L167), not asked for again until the device comes back (B3.4). New in the model: `live.refused_rates` (BR1). |
| **Hard transition** | A change from one entry of a player to the next with no overlap and no fade: the current entry stops (at the end of its play range, or at once with the declick for a manual start), then the next starts. Today it is what Continuous mode does without a MIX point or with automatic mix off (main spec rule 11, second bullet), and what O25 `HoldOthers` and the R26/O37 passes do. On a bit-perfect player every transition is hard (D8). |
| **Rate cut** | A hard transition during which the device reopens at the next entry's rate (BR3). |
| **Fader locked** | The player's volume is held at exactly 1.0 (100 %, 0 dB) and nothing may change it (BF1). |
| **Manual start over the current** | Play while playing (the transport's Next button, "Fade into the next track", `play`, `crates/fp-model/src/reducer.rs` ~L464, `Transport::Playing` arm, including Play on the entry on air set as next, O37) and Previous (R24, `previous` ~L498): today both crossfade (`start_or_crossfade` ~L787). |

## 4. Hard transitions and rate cuts (replaces B3.3)

### 4.1 What the model knows

- **BR1 Stream rates reach the model.** The engine reports
  `EngineEvent::StreamRate { device: OutputDevice, rate: u32, refused:
  Vec<u32> }` when a bus opens, whenever its stream rate changes (a B3.2
  reopen, a rate cut, a B3.4 fallback, a live-settings reopen, the
  watchdog reopening a lost device) and whenever a rate is added to or
  cleared from its refused set. The reducer stores `live.stream_rates[d] =
  rate` and `live.refused_rates[d] = refused` (runtime only, never saved);
  `forget_unused_devices` (`crates/fp-model/src/live.rs` ~L198) forgets both
  for a device no holder uses. A device with no reported rate has an
  **unknown** stream rate.

### 4.2 The decisions

- **BR2 `fp_model::bit_perfect::bit_perfect_player(state, player) ->
  bool`:** true when the player has a Main device and that device runs
  bit-perfect (§3). A player not placed yet is not a bit-perfect player.
  It decides every rule of this spec: hard transitions (BR4, BR7) and the
  locked fader (BF1).
- **BR3 `fp_model::bit_perfect::rate_cut(state, player, next:
  &SourceRequest) -> Option<u32>`** gives the rate the device reopens at
  during a hard transition, or `None` when it keeps its rate. It is
  `Some(r)` exactly when all hold:
  1. the player is a bit-perfect player (BR2), on Main device `d`;
  2. the stream rate of `d` is known;
  3. the next entry's format is known, it is PCM (`dsd_rate` is `None`),
     and its `sample_rate` `r` is not zero and differs from the stream rate;
  4. `r` is not among the refused rates of `d`;
  5. nothing else sounds on `d`: every holder placed on `d` other than
     `PlayerMain(player)` has no cause (`fp_model::live::causes`, live
     settings L1) and is not another player's Main paused with a current
     track (the engine keeps that track's timeline on the bus and never
     reopens under it, B3.3). This player's own CUE placed on `d` counts as
     something else;
  6. the player's current entry does not go out as DSD (`player.dsd` is
     `None`): the end of a DSD stream, its silence and the rate the device
     goes back to belong to O25 (BR10).

  When any of these fails the transition is still hard (on a bit-perfect
  player), but the device keeps its rate: the next track starts at the
  running rate, resampled if its rate differs (D2).

### 4.3 Scheduled transitions

- **BR4 A bit-perfect player's plan is always hard.** `plan_for`
  (`crates/fp-model/src/reducer.rs` ~L741) applies after the ordinary plan
  and the O25 hold: when the player is a bit-perfect player (BR2) and the
  plan is `StartNextAt`, the plan becomes `StartNextAt { at_secs: end,
  fade_current_until_secs: None, rate_cut }`, where `end` is the current
  entry's play-range end (`current_end`, ~L751: the cue-out with markers
  on, the end of the file without) and `rate_cut` is BR3 for the request
  the player preloads (`preload_target`). The MIX point (`segue_start`) and
  `players.auto_segue` are not used. `StopAt` plans (Single mode, stop
  after current, an entry's stop-after, no next, the O25 hold) are
  unchanged. `TransitionPlan::StartNextAt` gains the field `rate_cut:
  Option<u32>`; every other player's plan has `None`. An R26 repeat and an
  O37 replay are the same file, so their `rate_cut` is `None` when the
  pass on air plays at its own rate; a pass that started resampled (a cart
  was sounding) gives `Some(file rate)` like any other entry (BR3), so the
  next pass is bit-perfect again.
- **BR5 The plan follows the state until the transition.** `reconcile`
  already recomputes `plan_for` after every command and event and sends a
  new `Schedule` when it changes (~L1358). No new state is kept for this:
  - a cart fired, or another player started, on the device while the
    current track plays turns `rate_cut` into `None` (BR3.5); when it ends,
    `rate_cut` is `Some(r)` again. The plan stays hard at the same point
    throughout: only the reopen comes and goes;
  - the next entry changed, or analysed (its format becomes known), gives
    the plan the new entry's `rate_cut`;
  - the player's Main moved onto a bit-perfect device, or bit-perfect
    turned on for its device (a `Placed` or `Applied` event, live settings
    L8), turns the plan hard; moved off, or bit-perfect turned off, gives
    back the ordinary plan (a segue when automatic mix is on). If the
    position has already passed that plan's point, the engine runs it at
    once, as it does today for any plan that arrives late (`dispatch_plans`
    in the conductor tick, `crates/fp-engine/src/engine.rs` ~L3091). Live
    settings apply such a change only when the player is idle (L5, L6)
    unless the operator forces it (Apply now); a transition already
    started finishes as it began.
- **BR6 The engine executes a scheduled hard transition.**
  - With `rate_cut: None` the engine does what it does today for a hard
    `StartNextAt` (automatic mix off): at `at_secs` the next source starts
    on the frame where the current one stops, with the declick only when
    the cue-out is inside the file and a gapless join at the file's own end
    (`dispatch`, ~L3130). Nothing new.
  - With `rate_cut: Some(r)`:
    1. the engine prepares the next source at `r` (the preload is opened
       again with `LoadOptions.rate = r`), so the cut does not wait for it
       to buffer; a plan that drops the cut has it opened again at the
       stream rate;
    2. at `at_secs` the current source stops with the usual declick
       (`tuning.declick_ms`);
    3. when the declick has ended and the device has played it (BR16) the
       bus reopens at `r` as B3.2 does (the
       busy budget of a quiet bus; waiting sources of other players
       reopened at `r`), and the next source starts at once;
    4. the engine reports `TransitionStarted` as for any scheduled
       transition, and `StreamRate` (BR1).

    If, at step 3, something else sounds on the bus after all (a cart fired
    in the moment between the model's last plan and the cut), the bus keeps
    its rate and the next source, opened again at the stream rate, starts
    as soon as it is ready (resampled). If the reopen fails, B3.4 applies
    (back to the previous rate, the source resampled, the rate remembered
    as refused and reported in `StreamRate`). Either way the gap is longer
    but the next track plays.

### 4.4 Manual starts

- **BR7 A manual start over the current is always hard on a bit-perfect
  player.** In Play while playing (including Play on the entry on air set
  as next, O37) and in Previous, when the player is a bit-perfect player
  (BR2), the model emits `EngineAction::StartCurrent` instead of
  `Crossfade`, whatever the rates, as the O25 hold already does (`held` in
  `start_or_crossfade`); the player does not enter `fading`, so the
  **Mixing** badge never lights and Previous stays available.
  `players.fade_ms` is not used for these starts.
- **BR8 The engine reopens under a hard start.** In `start_current`
  (`crates/fp-engine/src/engine.rs` ~L2295) without a crossfade, on a
  bit-perfect bus, when the file's rate differs from the stream rate: the
  player's current source that the start replaces, and its declick tail,
  do not count as sounding for `prepare_start` (~L1358); the start waits
  for the tail to end and to be played (BR16), then `prepare_start` reopens the bus at the file's
  rate if nothing else sounds, then the source starts. At the same rate the
  start is today's hard start (the current stops with the declick and the
  next starts at once). Starting from silence (Play from stopped or
  paused, a pre-listen, a cart) keeps B3.2 unchanged.

### 4.5 What does not change

- **BR9 Fades that are not transitions stay.** On a bit-perfect player:
  Fade stop still fades out over `players.fade_ms` and stops
  (`FadeOutAndStop`, main spec rule 8); Pause and resume keep their ramp
  (`tuning.pause_ramp_ms`); Stop, a manual start over the current and a cut
  inside the file keep the declick (`tuning.declick_ms`), and a start
  inside the audio keeps its declick ramp in (audio path audit A1).
  Restart is a seek and is unchanged. While any of them runs the source is
  not bit-perfect, as today (D4). The DSD exceptions of O25 (Stop and Fade
  stop end a DSD stream at once) are unchanged.
- **BR10 What else keeps its behaviour.** Players whose Main is not on a
  bit-perfect device keep segues and crossfades, even when their Cue route
  plays on one (BF6). Carts and Cue (PFL) are untouched: they never cause
  a hard transition or a rate cut, and when they sound on the device they
  only turn the reopen off (BR3.5). A next entry not analysed yet (format
  unknown) gets a hard transition with no reopen and plays at the running
  rate (B2). DSD follows O25 unchanged: the `HoldOthers` hold already gives
  a hard transition; a DSD stream on air never takes a rate cut (BR3.6);
  the converted PCM of a DSD next entry does not trigger one (BR3.3).
- **BR11 A sounding bus keeps its rate (the rest of B3.3).** Outside BR6
  and BR8 a bus's rate still never changes while anything on it sounds,
  for the reason B3.3 gives (every frame-based timeline on the bus is in
  its frames). A rate cut is not an exception to that: at the reopen,
  nothing on the bus sounds.
- **BR16 The device plays out before a rate cut closes it** (2026-10-14,
  the open point of the engine review). Closing a stream discards what the
  driver and the device still hold (ALSA drops its buffer, transfers to a
  USB converter included), and the mixer renders that far ahead of what is
  heard: a reopen right after the declick has been rendered would cut the
  declick, or the end of the track, short, and stop the converter in the
  middle of a waveform. So the stream of a rate cut (BR6, BR8, and a start
  from silence at another rate while the end of what last sounded on the
  device has not been played yet) closes only once the bus has rendered,
  after the frame where the last source on it ended, as many frames as the
  stream holds: what the backend reports (`OutputStream::latency_frames`,
  measured from the driver's delay), else two blocks, but never more than
  `tuning.play_out_max_ms` (default 500 ms, 0–2000; 0 never waits) at the
  bus's rate: a bogus measurement (a saturated timestamp after an xrun)
  cannot hold the output silent, and the bound is logged once per output.
  The engine waits without blocking (it looks again every tick). The
  gap of a rate cut grows by about the device's buffer (some 25–35 ms with
  512-frame blocks at 44.1 kHz on a USB converter). The same holds for a
  native DSD stream going back to PCM (O25): it closes once its DSD
  silence has been played, so the whole silence reaches the converter.

### 4.6 Live settings

- **BR12 A rate cut is not a settings change.** It reopens the device with
  the settings it **runs** with (buffer size, exclusive access, DSD mode);
  it never applies a pending item. A pending device item keeps waiting on
  its causes (live settings L5): the player stays `Playing` in the model
  for the whole cut, so its cause never lapses and `live::due` emits
  nothing for that device during the gap.
- **BR13 The engine's quiet guard sees the cut.** A bus whose player is
  between the stop and the start of a rate cut (BR6 steps 2–3, BR8) is not
  quiet for live settings L11: an `ApplyDevice`, `ApplyRoute` or
  `ApplyAudioSystem` kept by the engine does not run in the gap.
- **BR14 A settings change on a bit-perfect device.** As live settings
  L12: a new configured rate is only where the device starts; bit-perfect
  turned off reopens the device shared at `effective_rate`, the player is
  no longer a bit-perfect player (BR2) and its transitions are ordinary
  again (BR5). Each reopen is reported through `StreamRate` (BR1).

### 4.7 Interface, MIDI and remote control

- **BR15 The interface says that a bit-perfect player does not mix.** It
  reads `bit_perfect_player` from the snapshot; nothing else changes:
  - the large Play/Next button's tooltip while the player is on air reads
    `tip-next-bit-perfect` instead of `tip-next` (§6.4);
  - the waveform draws the MIX marker of the current and next tracks dim,
    with the dim style it already uses for the MIX marker in Single mode
    (`crates/fp-app/src/ui/widgets.rs` ~L1626): the marker belongs to the
    track and still applies on other players;
  - Settings → Players shows the note `settings-players-bit-perfect-note`
    (§6.5) under **Fade time** and **Automatic mix at the MIX point** while
    at least one player is a bit-perfect player. Both settings stay
    editable: they are global and still apply to the other players.
- **BR16 MIDI, the remote API and OSC add nothing.** Their Play/Next and
  Previous map to `Command::Play` and `Command::Previous`, so on a
  bit-perfect player they start hard (BR7); they are accepted, never
  refused, since the start itself happens. A client tells a bit-perfect
  player by `PlayerDto::fader_locked` and OSC `volume-locked` (BF13),
  which hold exactly when BR2 does; the documentation says so. No
  "crossfade" operation exists in either protocol, so none is refused.

## 5. The locked fader

### 5.1 Rules

- **BF1 `fp_model::bit_perfect::fader_locked(state, player) -> bool`:**
  `bit_perfect_player` (BR2). A player not placed yet is not locked.
- **BF2 The fader goes to 100 % when it locks.** After every command and
  every event (in `reconcile`, so every path is covered), a locked player
  whose volume is not exactly 1.0 gets `volume = 1.0` and
  `EngineAction::SetVolume { volume: 1.0 }`. This covers, with one rule:
  bit-perfect turned on for its device (`Applied` for that device, live
  settings L8), its Main route moved onto a bit-perfect device and an
  audio system change that places it on one (`Placed`), a player added,
  and a session restored with a lower volume (the engine's first `Placed`
  at start).
- **BF3 A locked fader refuses changes.** `Command::SetVolume(p, v)` on a
  locked player is refused with `ModelError::FaderLocked(p)` ("the volume
  of player {p} is held at 100 % on a bit-perfect output") unless `v` is
  exactly 1.0, which is accepted and changes nothing. A NaN is ignored as
  today. `Availability` gains `fader: bool` (false when locked), and
  `command_available` answers false for a `SetVolume` that BF3 refuses.
- **BF4 Unlocking keeps the level.** When the player stops being locked
  (bit-perfect turned off, the route moved off the device, the device
  replaced by the default output of an audio system without bit-perfect),
  the volume stays at 1.0 and the fader works again. Nothing moves it.
- **BF5 Fades are not the fader.** The fades that remain on a bit-perfect
  player (BR9: Fade stop, pause ramps, declicks) act on the mixer's gain,
  not on the player's volume, and BF3 does not see them.
- **BF6 Cue (PFL) is not affected.** The CUE path has its own gain
  (`cue_volume`, `crates/fp-engine/src/engine.rs` `open_source`) and no
  fader; it is neither locked nor reset, wherever the Cue route plays.
  Carts have no fader and are not affected.

### 5.2 Interface

- **BF7 The on-screen fader.** In the player's meter column
  (`meter_column`, `crates/fp-app/src/ui/player.rs` ~L465), a locked
  player's fader is drawn at the top, disabled (greyed, no drag, no wheel),
  with the tooltip `tip-volume-locked`. `widgets::fader`
  (`crates/fp-app/src/ui/widgets.rs` ~L1252) gains an `enabled` flag. The
  tooltip replaces `tip-volume-dsd`: a DSD stream reaches a device only
  when it is bit-perfect, so its fader is always locked and the old
  "moving the fader continues the track as PCM" can no longer happen.
- **BF8 Apply now says which faders move.** When the items
  `ApplySettingsNow` would force (live settings L22) would lock a player
  whose volume is below 100 % (bit-perfect on for its device, or its route
  moved onto a bit-perfect device), the confirmation adds the line
  `apply-now-fader-reset` naming those players. `fp_model::bit_perfect::
  faders_reset_by(state) -> Vec<PlayerId>` computes them. Applied without
  confirmation (the device idle), BF2 needs no warning: nothing plays.
- **BF9 Settings.** The bit-perfect hint under Settings → Audio outputs →
  Advanced (`settings-bit-perfect-hint`, §6.3) says that it is a special
  mode, that the players on the device never crossfade or mix, that a
  change of rate leaves a short silence, and that their volume is held at
  100 %.

### 5.3 MIDI

- **BF10 A fader binding of a locked player is ignored.** In
  `Router::on_message` (`crates/fp-control/src/router.rs` ~L58, the
  `MidiAction::Volume` arm ~L112) the message gives no command when
  `fader_locked`, and the binding's soft takeover forgets its pickup
  (`picked = false`, `last_set = None`): once the fader unlocks, the
  surface's control must pick up the volume at 100 % (by reaching the top
  or crossing it) before it moves anything, so the level never jumps.
- **BF11 Feedback.** LED feedback (`fp_control::feedback::desired`) covers
  buttons only; no fader or ring position is sent today, so there is
  nothing that could show a level other than 100 %. Buttons are unchanged.
  Sending fader positions (motorised faders, LED rings) is out of scope
  (§10).

### 5.4 Remote API and OSC

- **BF12 Refused, with a reason.** `PUT /players/{id}/volume` with `{"fader": f}`, a value
  below 1 on a locked player is refused with **409 `unavailable`** and the
  model's message, through the existing path (`api::plan` → `dry_run` →
  `ApiError::from_model`, `crates/fp-remote/src/api.rs` ~L116). A value of
  exactly 1 is accepted and changes nothing. This follows every other
  operation the model refuses in the current state (for example stop
  after current in Single mode): a client is told, never silently
  ignored, and a client that sends 1 to restore a level is not punished.
  An OSC `/fauste/player/{n}/volume` message is refused the same way and
  noted in the remote log with the code `unavailable`, as other refused
  OSC operations are (`osc_server`, ~L93).
- **BF13 Clients can see the lock.** `PlayerDto` gains `fader_locked:
  bool`, and OSC publishes `/fauste/player/{n}/volume-locked` (int 0/1)
  beside `/volume`, so remote surfaces can grey their fader. Both are
  additive (no breaking change for clients). They are true exactly for a
  bit-perfect player, which also starts every track hard (BR16).

### 5.5 O25 (DSD) amended

The O25 rule "the fader leaving unity switches a DSD stream to PCM"
(`Command::SetVolume`, `crates/fp-model/src/reducer.rs` ~L108,
`EngineAction::LeaveDsd`) can no longer be reached from the fader, since
a DSD stream only plays on a bit-perfect device (BF1, BF3). It stays as a
guard and its test stays; the DSD decision's volume check (`DsdFacts`)
stays too.

## 6. Interface strings

New and changed keys. `en-US` and `es-ES` are written by hand; the others
are AI translations, written at implementation time for **all eleven
locales** (ca-ES, de-DE, en-US, es-ES, eu-ES, fr-FR, gl-ES, it-IT, nl-NL,
pl-PL, pt-PT) and checked like every locale. Where a table below lists only
en-US and es-ES, the other nine are translated from the en-US text then.
`tip-volume-dsd` is removed from every locale (BF7). `{ $players }` is a
list such as "P1, P2" built from the existing player labels.

### 6.1 `tip-volume-locked` (new)

The fader's tooltip, and also the interface's notice when the model
refuses a `SetVolume` with `ModelError::FaderLocked`.

| Locale | Text |
|---|---|
| en-US | Volume held at 100 %: this player's output is bit-perfect. |
| es-ES | Volumen fijo al 100 %: la salida de este reproductor es bit-perfect. |
| ca-ES | Volum fix al 100 %: la sortida d'aquest reproductor és bit-perfect. |
| de-DE | Lautstärke fest auf 100 %: Der Ausgang dieses Players ist bit-perfect. |
| eu-ES | Bolumena % 100ean finkatuta: erreproduzigailu honen irteera bit-perfect da. |
| fr-FR | Volume bloqué à 100 % : la sortie de ce lecteur est bit-perfect. |
| gl-ES | Volume fixo ao 100 %: a saída deste reprodutor é bit-perfect. |
| it-IT | Volume bloccato al 100 %: l'uscita di questo lettore è bit-perfect. |
| nl-NL | Volume vast op 100 %: de uitgang van deze speler is bit-perfect. |
| pl-PL | Głośność zablokowana na 100 %: wyjście tego odtwarzacza jest bit-perfect. |
| pt-PT | Volume fixo a 100 %: a saída deste leitor é bit-perfect. |

### 6.2 `apply-now-fader-reset` (new)

| Locale | Text |
|---|---|
| en-US | The volume of { $players } goes to 100 % (bit-perfect output). |
| es-ES | El volumen de { $players } pasa al 100 % (salida bit-perfect). |
| ca-ES | El volum de { $players } passa al 100 % (sortida bit-perfect). |
| de-DE | Die Lautstärke von { $players } geht auf 100 % (Bit-perfect-Ausgang). |
| eu-ES | { $players }(r)en bolumena % 100era igotzen da (bit-perfect irteera). |
| fr-FR | Le volume de { $players } passe à 100 % (sortie bit-perfect). |
| gl-ES | O volume de { $players } pasa ao 100 % (saída bit-perfect). |
| it-IT | Il volume di { $players } passa al 100 % (uscita bit-perfect). |
| nl-NL | Het volume van { $players } gaat naar 100 % (bit-perfect uitgang). |
| pl-PL | Głośność { $players } zmienia się na 100 % (wyjście bit-perfect). |
| pt-PT | O volume de { $players } passa para 100 % (saída bit-perfect). |

### 6.3 `settings-bit-perfect-hint` (changed)

| Locale | Text |
|---|---|
| en-US | A special mode for audiophile playback, not for mixed programmes. The device is opened with exclusive access and follows each track's sample rate. The players that play on it never crossfade or mix: each track plays to its end, then the next one starts. When the sample rate changes and nothing else plays on the device, there is a short silence while it reopens. Their volume is held at 100 %. With no fade and nothing else on the same outputs, the samples reach the device unchanged; the BP badge lights while that is true. |
| es-ES | Un modo especial para la escucha audiófila, no para programas con mezclas. El dispositivo se abre en acceso exclusivo y sigue la frecuencia de muestreo de cada pista. Los reproductores que salen por él nunca encadenan ni mezclan pistas: cada pista suena hasta el final y luego empieza la siguiente. Si cambia la frecuencia de muestreo y no suena nada más en el dispositivo, hay un breve silencio mientras se vuelve a abrir. Su volumen queda fijo al 100 %. Sin fundidos y sin nada más en las mismas salidas, las muestras llegan al dispositivo sin cambios; el distintivo BP se enciende mientras sea así. |

### 6.4 `tip-next-bit-perfect` (new, BR15)

| Locale | Text |
|---|---|
| en-US | Next track, at once and without a crossfade (bit-perfect output) |
| es-ES | Siguiente pista, al momento y sin fundido (salida bit-perfect) |

### 6.5 `settings-players-bit-perfect-note` (new, BR15)

| Locale | Text |
|---|---|
| en-US | Players on a bit-perfect output never mix or crossfade: automatic mix does not apply to them, and the fade time only to Fade stop. |
| es-ES | Los reproductores con salida bit-perfect nunca mezclan ni encadenan con fundido: la mezcla automática no se les aplica, y la duración del fundido solo cuenta para Stop con fundido. |

The remote API's error message (BF3, BF12) is English, like every API
message, and is not a Fluent string.

## 7. Model and engine changes (summary)

| Crate | Change |
|---|---|
| `fp-model` | New module `bit_perfect` (`bit_perfect_player`, `rate_cut`, `fader_locked`, `faders_reset_by`); `LiveSettings` gains `stream_rates` and `refused_rates`; `EngineEvent::StreamRate`; `TransitionPlan::StartNextAt::rate_cut`; `plan_for` (BR4); `play` and `previous` through `start_or_crossfade` (BR7); `reconcile` (BF2); `Command::SetVolume` (BF3); `ModelError::FaderLocked`; `Availability::fader`. |
| `fp-engine` | `StreamRate` reports (BR1); the scheduled rate cut (BR6); `start_current` and `prepare_start` under a hard start (BR8); the quiet guard (BR13). A hard `StartNextAt` without a rate cut uses the existing path. |
| `fp-control` | `Router::on_message`, `Volume` arm (BF10). |
| `fp-remote` | `PlayerDto::fader_locked`, OSC `volume-locked` (BF13); refusals come from the model (BF12). |
| `fp-app` | `widgets::fader` `enabled`; `meter_column` (BF7); the Play/Next tooltip, the dim MIX marker and the Settings → Players note (BR15); the Apply now confirmation (BF8); locales (§6). |

No configuration field is added: a hard transition and a rate cut have
nothing to tune (the gap is the device's reopen time plus
`tuning.declick_ms`, both already configurable or the device's own), the
lock has no setting (Q1), and `players.fade_ms` and `players.auto_segue`
keep their meaning for the other players (BR15).

## 8. Testing

One test per rule, watched failing first. Engine tests use the `Offline`
backend and explicit `now: Instant`; nothing sleeps.

| Rule | Layer, file | Test |
|---|---|---|
| BR1 | `fp-model` `tests/bit_perfect_rules.rs` | `StreamRate` sets `live.stream_rates` and `refused_rates`; `Unplaced` of the last holder forgets them. |
| BR1 | `fp-engine` `tests/bit_perfect.rs` | a bus open, a B3.2 reopen and a refused rate each report `StreamRate` with the right rate and refused set. |
| BR2 | `fp-model` `tests/bit_perfect_rules.rs` | true for a Main placed on a device running bit-perfect; false for the wanted-but-pending value, for an unplaced player, and for a player whose only Cue route is on the device. |
| BR3 | `fp-model` `tests/bit_perfect_rules.rs` | one test per condition 1–6: each one false gives `None`; all true gives `Some(r)`. |
| BR4 | `fp-model` `tests/bit_perfect_rules.rs` | on a bit-perfect player with automatic mix on and a MIX point: a next at the same rate gives `StartNextAt { at: cue-out, fade: None, rate_cut: None }`; a 96 kHz next on a 44.1 kHz device gives `rate_cut: Some(96000)`; Single mode still gives `StopAt`; the same player off bit-perfect gives the segue plan. |
| BR5 | `fp-model` `tests/bit_perfect_rules.rs` | a cart starting on the device sends a `Schedule` with `rate_cut: None` and the same `at_secs`; `CartEnded` sends `Some(r)` again; `Placed` off the device sends the segue plan. |
| BR6 | `fp-engine` `tests/bit_perfect.rs` | `rate_cut: None`: the next starts on the current's stop frame with no overlapping ramp and the device keeps its rate. `rate_cut: Some(r)`: the preload is opened at `r`; at `at_secs` the current stops, `OfflineDevice::config()` shows `r`, the next starts and is bit-perfect; `TransitionStarted` is reported. A cart fired just before the cut: no reopen, the next plays resampled. A refused rate: B3.4 fallback, the next plays. |
| BR7 | `fp-model` `tests/bit_perfect_rules.rs` | on a bit-perfect player, Play while playing, Play on the self-next (O37) and Previous emit `StartCurrent` (not `Crossfade`) at the same rate and at another rate, and `fading` stays false; on another player they emit `Crossfade`. |
| BR8 | `fp-engine` `tests/bit_perfect.rs` | a `StartCurrent` at another rate replacing a playing source on a bit-perfect bus reopens the bus after the declick; at the same rate it starts at once with no reopen; a `Crossfade` at another rate does not reopen (resampled). |
| BR9 | `fp-model` `tests/bit_perfect_rules.rs` | on a bit-perfect player `FadeStop` emits `FadeOutAndStop` with `fade_ms`, and `Pause` and `Restart` emit what they emit on any player. |
| BR10 | `fp-model` `tests/bit_perfect_rules.rs` | an unknown-format next gets a hard plan with `rate_cut: None`; a DSD next and a DSD stream on air get no rate cut; the O25 hold still gives `StopAt`; a non-bit-perfect player on the same device set keeps its segue plan. |
| BR11 | `fp-engine` `tests/bit_perfect.rs` | the existing B3.3 test (a source started on a sounding bus is resampled) still passes. |
| BR12 | `fp-model` `tests/live.rs` | with a rate cut scheduled, a pending buffer change on the player's device is not due at any point up to `TransitionStarted` (the player stays `Playing`, so it stays a cause). |
| BR13 | `fp-engine` `tests/bit_perfect.rs` | an `ApplyDevice` kept by the guard does not run in the rate cut's gap; it runs once the device is quiet. |
| BR14 | `fp-engine` `tests/bit_perfect.rs` | bit-perfect off reports `StreamRate` at `effective_rate`; the next manual start is a crossfade again. |
| BR16 | `fp-engine` `tests/bit_perfect.rs`, `tests/dsd_output.rs`, `tests/bus.rs`; `fp-backends` | with the Offline device holding three blocks, a scheduled cut, a hard start and a start right after a stop reopen only after at least that much silence was rendered after the last sound, and not much later; one reporting `u32::MAX` waits `tuning.play_out_max_ms` and no longer, and a bus clamps what its stream reports to it at its rate (a new maximum applies at once); a native DSD stream closes after its silence and the three blocks; a bus counts what its stream reports, two blocks when it cannot tell, nothing without a stream; the cpal and native DSD measurements (`held_frames`, `held_words`). On hardware: `examples/rate_cut_tail.rs`. |
| BR15 | `fp-app` `tests/` (kittest, player and settings) | a bit-perfect player's Play/Next tooltip is `tip-next-bit-perfect` while on air; its MIX marker is drawn dim; Settings → Players shows `settings-players-bit-perfect-note` only while a bit-perfect player exists. |
| BR16 | `fp-remote` `tests/api.rs`, `fp-control` `tests/router.rs` | `POST /players/{id}/play` on air and the MIDI Play/Next binding on a bit-perfect player are accepted and give `Command::Play` (the model then starts hard, BR7). |
| BF1 | `fp-model` `tests/bit_perfect_rules.rs` | locked exactly when `bit_perfect_player`; unplaced is not locked. |
| BF2 | `fp-model` `tests/bit_perfect_rules.rs` | `Applied` bit-perfect on, `Placed` on a bit-perfect device and a restored volume of 0.5 each give volume 1.0 and one `SetVolume { 1.0 }`. |
| BF3 | `fp-model` `tests/bit_perfect_rules.rs` | `SetVolume(p, 0.5)` refused with `FaderLocked`, state unchanged; `SetVolume(p, 1.0)` accepted; `availability(..).fader` false. |
| BF4 | `fp-model` `tests/bit_perfect_rules.rs` | `Applied` bit-perfect off leaves volume 1.0 and accepts `SetVolume(p, 0.5)`. |
| BF5 | `fp-model` `tests/bit_perfect_rules.rs` | `FadeStop` on a locked player produces `FadeOutAndStop` and leaves `volume` at 1.0. |
| BF6 | `fp-model` `tests/bit_perfect_rules.rs` | a player whose Cue route is the bit-perfect device and Main is not: not locked, CUE starts. |
| BF7 | `fp-app` `tests/` (kittest, player) | a locked player's fader is disabled, does not send `SetVolume` on drag, and shows `tip-volume-locked`. |
| BF8 | `fp-app` `tests/live_settings.rs` (kittest) | the Apply now confirmation lists `apply-now-fader-reset` with the player; `faders_reset_by` unit test in `fp-model`. |
| BF9 | `fp-app` i18n test | every locale has the new keys (§6) and no `tip-volume-dsd` (the existing key-parity test). |
| BF10 | `fp-control` `tests/router.rs` | a fader message for a locked player gives nothing; after unlocking, a control at 50 % gives nothing until it reaches the top. |
| BF11 | `fp-control` `tests/feedback.rs` | feedback for a locked player's buttons is unchanged. |
| BF12 | `fp-remote` `tests/api.rs`, `tests/osc_server.rs` | fader 0.5 on a locked player → 409 `unavailable` with the model's message; fader 1 → 202 Accepted; the OSC message is refused and noted. |
| BF13 | `fp-remote` `tests/dto.rs`, `tests/osc.rs` | `fader_locked` and `volume-locked` follow the lock. |

The manual loopback check of the bit-perfect docs gains two steps: play a
44.1 kHz track followed by a 96 kHz one in continuous mode, with automatic
mix on and a MIX point; the first plays to its cue-out with no fade, the
converter shows 96 kHz on the second track after a short silence, and the
BP badge lights on both. Then two tracks at the same rate: the second
follows the first with no gap and no overlap.

## 9. Documentation to update

The maintainer asks for this mode to be explained well: an operator must
learn from the guide, before turning it on, that bit-perfect is a special
mode and what it costs. Every page below says the same thing in its own
place.

### 9.1 User guide (`docs/user/`, English only)

- **`bit-perfect.md`**
  - **Introduction** (the first paragraph): add that it is a special mode
    for audiophile playback (archive listening, a hi-fi or monitoring
    chain, checking a digital link), not the default and not meant for a
    mixed music programme.
  - **New section "What it costs", right after the introduction and
    before "Setting a device bit-perfect"**, a plain list for every player
    whose Main output is on the device:
    - no crossfades and no segues between tracks: each track plays to its
      end (its cue-out), then the next starts; **Automatic mix at the MIX
      point** and the MIX point are not used, and **Play / Next** and
      **Previous** while on air switch at once (with a click-free stop)
      instead of fading;
    - a **short silence** between two tracks whose sample rates differ,
      while the device reopens (usually a few tens of milliseconds, longer
      on some devices); none between tracks at the same rate, which follow
      each other with no gap;
    - the **volume fader is held at 100 %**, on screen, on MIDI and over
      the remote API and OSC, and goes to 100 % when bit-perfect is turned
      on;
    - **what still works:** Fade stop (it fades over the fade time), Pause,
      Stop, Restart, repeat and stop-after marks, Single and Continuous
      modes, Cue (pre-listen, which is not locked), carts (they play, and
      while one plays on the device the next track is not bit-perfect and
      no silence is inserted), DSD output, and every player whose Main
      output is on another device, which mixes as usual;
    - **when to use it**, and when not: use it when every track must reach
      the converter untouched; keep it off for an on-air programme that
      needs mixed transitions or level changes.
  - **"What happens on a bit-perfect device"**: rewrite "The rate follows
    the file" for hard transitions and rate cuts (when the device reopens
    between two tracks, when the next track is resampled instead: another
    player, a cart or a CUE on the device, a rate the device refused, an
    unanalysed track); the example "a 48 kHz track mixed into after a
    44.1 kHz one" goes. In "No processing, when nothing asks for it", the
    volume is held at 100 % rather than something to check.
  - **"DSD"**: the "volume must stay at 100 %: the fader's tooltip says so.
    Moving it switches the track to PCM" line and the two "moving the
    fader" remarks go (the fader is locked); "Pressing Play on another
    track while it plays cuts it the same way instead of crossfading" and
    "The player's own next track does not overlap" become true of every
    track on a bit-perfect player and are said once, in "What it costs";
    "An album does not stay DSD" loses "a segue or a crossfade".
  - **"The BP badge"**: "volume is 100 %" becomes "the volume is held at
    100 % on a bit-perfect output"; "no fade runs" names Fade stop and the
    pause ramp, the only fades left.
  - **"Checking it yourself"**: the two loopback steps of §8.
- **`players.md`**: the **Mixing** badge (Header table: never on a
  bit-perfect player); the **Volume fader** bullet (locked at 100 % on a
  bit-perfect output, with its tooltip); the Transport table rows **Play /
  Next** and **Previous** (on a bit-perfect output they switch at once,
  without a fade); the **Waveform** section (the MIX marker is drawn dim
  on a bit-perfect player).
- **`markers-and-mixing.md`**, "What the player does with them": a bullet
  saying that a player on a bit-perfect output ignores the MIX point and
  the automatic mix, and that Play while on air does not fade there, with
  a link to "What it costs".
- **`settings.md`**: in **Players**, the **Fade time** and **Automatic mix
  at the MIX point** rows say they do not apply to players on a
  bit-perfect output (except Fade stop), and the note under them is
  mentioned; in **Audio outputs**, the **Bit-perfect: *device*** row says
  it is a special mode, the players on it never crossfade, a change of rate
  leaves a short silence and their volume is held at 100 %, linking to
  "What it costs".
- **`midi.md`**, "How the controls behave": a locked volume binding is
  ignored and picks up again at 100 % once unlocked; Play/Next and
  Previous on a bit-perfect player switch at once.
- **`remote-control.md`**, "What a client can do" and "OSC": `play` and
  `previous` on a bit-perfect player switch at once; a fader below 1 is
  refused (409) and exactly 1 is accepted; `fader_locked` and
  `volume-locked` tell a bit-perfect player.
- **`troubleshooting.md`**, "Bit-perfect": "A short silence before a
  track" explains the rate change and that it happens only when the rate
  changes; new entries "Tracks do not mix, and Play does not fade" and
  "The volume fader does not move", both pointing to "What it costs".
- **`README.md`** of the guide (the page table): the Bit-perfect output
  row adds "a special mode: no crossfades, volume at 100 %".

### 9.2 Repository README and technical docs

- **`README.md`**, the **Bit-perfect output** feature line: a special
  mode; the device follows each track's rate, with a short silence when it
  changes; tracks follow each other without crossfades; the volume is held
  at 100 %.
- **`docs/technical/audio-engine.md`**: "Rates and bit-perfect buses"
  (hard transitions and `rate_cut`, `StreamRate`, the reopen under a hard
  start, BR6 and BR8); the engine action table (`Schedule(plan)` lists
  `rate_cut`); "Live settings" (the quiet guard in a rate cut's gap,
  BR13).
- **`docs/technical/remote-api.md`**: `fader_locked`, the 409, the OSC
  address `volume-locked`, and that `play` and `previous` start hard on a
  bit-perfect player.
- **`docs/technical/ui.md`**: the fader's `enabled` flag, the Play/Next
  tooltip, the dim MIX marker and the Settings → Players note (BR15).
- **`docs/technical/audio-path-audit.md`** §5.4 "Rate changes on
  bit-perfect devices": a note pointing to this spec.

### 9.3 Specs

- The bit-perfect spec: B3.3 marked "Replaced by the bit-perfect rate and
  fader spec (2026-10-08)"; B1 and B7 notes ("Automatic volume bypass" and
  "Changing the rate while a bus is busy" are no longer out of scope in
  the form of §4 and §5).
- The main spec §3.2: rules 5 and 11 and R24 note "except on a bit-perfect
  player: a hard transition (bit-perfect rate and fader spec, D8)".
- Live settings §13 "the bit-perfect rate change at transitions" points
  here.
- Feedback-2 O25: the note of §5.5.

### 9.4 Locales

§6 in all eleven files.

## 10. Out of scope

- Any crossfade or segue on a bit-perfect player (D8), including an
  option to allow them at the same rate, per device (D7), per player or
  per entry.
- Changing the rate while anything else sounds on the device (D2), or in
  the middle of a track.
- A rate cut for the Cue path, carts, or DSD streams (BR10).
- Gapless rate changes: the gap is the device's reopen and cannot be
  hidden without resampling.
- Hiding or disabling the fade time and automatic mix settings: they are
  global and apply to the other players (BR15).
- Fader position feedback to MIDI surfaces (motorised faders, LED rings)
  (BF11).
- Locking any gain other than the player's volume (carts, CUE).
- Shortening the gap further than BR6.1 does (for example keeping a second
  stream open at the next rate).

## 11. Decisions taken in this spec (for review)

| # | Decision | Rationale |
|---|---|---|
| S1 | The model decides a rate cut (BR3) from a new `StreamRate` report, not the engine on its own. | Behaviour lives in `fp-model` (D6); the device's running rate on a bit-perfect device follows the files and is not in `DeviceSettings`, so it needs its own report. The engine keeps the final word only when the moment has changed (BR6 fallback). |
| S2 | Known refused rates never cause a reopen (BR3.4). | A reopen into a rate the device refuses would cost a gap and still resample, on every such transition. The transition stays hard. |
| S3 | The scheduled cut carries its rate in the plan, and the engine prepares the next source at that rate in advance (BR6.1). | Otherwise the next source is opened again after the reopen and the gap includes its buffering (`tuning.ready_threshold_ms`, 500 ms by default). |
| S4 | The current plays to the end of its play range (the cue-out with markers on), never to its MIX point (BR4). | D8; the MIX point exists only to overlap. |
| S5 | Play while playing, Play on the self-next (O37) and Previous are all hard on a bit-perfect player (BR7). | D8 names Next and Previous; the O37 replay is the same Play button. |
| S6 | The lock and the hard transitions follow the bit-perfect setting the device **runs** with, even when exclusive access was refused (B4). | Predictable for the operator (the checkbox decides), and a later reopen may obtain exclusive access; the BP badge already says when the output is not bit-perfect. |
| S7 | One invariant (BF2) resets the fader in every case: bit-perfect on, a route moved onto the device, an audio system change, a new player, a restored session. | One rule, one test, no path forgotten. Every such change applies only while the player is idle (live settings L5, L6) unless the operator forces it (BF8). |
| S8 | The remote API and OSC **refuse** a locked fader (409 `unavailable`) and accept exactly 100 % (BF12). | It is the existing pattern for operations the model refuses in the current state; silently ignoring would let a remote surface show a level that is not on air. |
| S9 | A locked MIDI fader is ignored and must pick up again at 100 % after unlocking (BF10). | Soft takeover already exists for this; without it the first touch after unlocking would jump the level. |
| S10 | No MIDI fader feedback is added (BF11). | Feedback today covers buttons only; there is no fader position output to set to 100 %. |
| S11 | `tip-volume-dsd` is removed and the O25 fader rule becomes unreachable but stays (§5.5). | A DSD stream needs a bit-perfect device, whose fader is now locked. |
| S12 | The CUE path and carts are untouched (BF6, BR10); a player's Cue route on a bit-perfect device does not make it a bit-perfect player (BR2). | The lock and D8 are about the on-air player; CUE has its own gain and no fader, and the BP badge follows Main only (B6). |
| S13 | `PlayerDto::fader_locked` and OSC `volume-locked` are added (BF13) and also stand for "this player starts every track hard" (BR16); no second field is added. | Both hold exactly when BR2 does; one flag keeps the API small. A client that needs to tell the two apart has no case where they differ. |
| S14 | One predicate, `bit_perfect_player` (BR2), decides both the hard transitions and the lock. | They are the same condition in D3 and D8; two predicates could drift apart. |
| S15 | At the same rate a scheduled hard transition is the engine's existing hard join (BR6), and a manual one today's hard start (BR8): the current's declick tail (`tuning.declick_ms`, 5 ms by default) may overlap the start of the next, as it does today with automatic mix off. | No new engine path; the tail exists only when the cue-out is inside the file, and it is what keeps the cut click-free. The next track is bit-perfect from the end of that tail. |
| S16 | A DSD stream on air never takes a rate cut (BR3.6); its transition is still hard. | O25 already decides how a DSD stream ends, with its silence and the rate the device goes back to; two reopen mechanisms on one transition would conflict. This keeps the former BR8. |
| S17 | The interface shows the mode with three small cues (BR15): the Play/Next tooltip, a dim MIX marker, and a note in Settings → Players. The settings stay editable. | Minimal and enough for the operator to see why nothing mixes; the settings are global and still apply to the other players, so disabling them would be wrong. |
| S18 | A plan recomputed after the player moves off a bit-perfect device whose segue point has passed runs at once (BR5), as any late plan does today. | No new rule; the move only happens with the player idle unless the operator forces it. |
| S19 | No new operation and no refusal in MIDI, the remote API or OSC for Next and Previous (BR16). | The start itself happens; only its shape changes. Refusing it would leave a remote client unable to go to the next track. |

## 12. Answered questions

- **Q1** Should rate cuts be optional per device, for stations that prefer
  an uninterrupted segue resampled over a gap? **No** (D7): the cut and
  its gap are the price of bit-perfect output, and every bit-perfect
  device cuts.
- **Q2** Should tracks at the same rate still segue and crossfade on a
  bit-perfect device? **No** (D8, 2026-10-13): a bit-perfect player never
  crossfades or segues; every transition is hard.
