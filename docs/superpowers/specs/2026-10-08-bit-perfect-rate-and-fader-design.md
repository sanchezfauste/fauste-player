# Bit-perfect: rate cuts and the locked fader — Design Spec

- **Date:** 2026-10-08
- **Status:** Draft, for the maintainer's review (§12).
- **Amends:** [the bit-perfect spec](2026-09-26-phase4-bit-perfect-design.md)
  (**B3.3 is replaced** by §4; B1 "unaltered source", B7 out of scope),
  [the live settings spec](2026-10-07-live-settings-design.md) (§8 and
  §13 "the bit-perfect rate change at transitions"), and
  [the second operator feedback spec](2026-10-01-operator-feedback-2-design.md)
  O25 (the fader that leaves unity, §5.4).
- **Numbering:** rate rules are `BR<n>`, fader rules `BF<n>`. Each rule is
  testable and gets at least one test (§8).
- **Line numbers** are those of `master` at the date above (`c31aba4`);
  each reference also names the function.

---

## 1. Goal

Two things a station with a bit-perfect output expects and does not get
today:

1. **The device follows the music, track after track.** Today B3.3 keeps a
   sounding bus at its rate, and in continuous play the bus always sounds
   (the next track starts while the current one ends, or segues into it).
   So the rate set by the first track of the session is, in practice, the
   rate of every track after it: a 96 kHz track after a 44.1 kHz one is
   resampled for its whole length and is never bit-perfect. From now on, a
   change of rate between two tracks of the same player becomes a **rate
   cut**: the current track ends, the device reopens at the new rate, the
   next track starts. A short gap is the documented cost.
2. **No level change reaches a bit-perfect output by accident.** A fader
   below 100 % alters every sample, so the track is not bit-perfect and the
   BP badge stays off, often without the operator noticing. From now on the
   fader of a player whose Main output is bit-perfect is **held at 100 %**
   in the interface, on MIDI surfaces and through the remote API and OSC.

## 2. Maintainer decisions (binding, 2026-10-03 and 2026-10-08)

| # | Decision |
|---|---|
| D1 | **Rate cut.** On a bit-perfect device, when the next track's sample rate differs from the device's running rate and the only thing sounding on that device is this player's outgoing track, the transition is a hard cut: the current track plays to its cue-out (or stops at once on a manual Next), the device reopens at the new rate, then the next track starts. A short gap is accepted. There is no segue or crossfade across a rate change. |
| D2 | If another player or a cart sounds on that device, nothing on air is cut: the next track is resampled to the running rate and is not bit-perfect (today's behaviour, B3.3). |
| D3 | **Fader locked.** For a player whose Main output is on a bit-perfect device the fader is held at 100 % and disabled in the interface, on MIDI and in the remote API and OSC. |
| D4 | Fades and segue crossfades stay. While they run the source is not bit-perfect (as today). |
| D5 | When bit-perfect is turned on for a device (under live settings, when the device is idle), the faders of the players on it go to 100 % at that moment, so there is no level jump on air. When bit-perfect is turned off, the fader is unlocked and stays at 100 %. |
| D6 | Rules live in `fp-model` as pure functions, one test per rule; the engine and the interface only execute and display (CLAUDE.md rule 7). Nothing goes on air by itself (rule 10). |

## 3. Terms

| Term | Meaning |
|---|---|
| **Bit-perfect device** | As B1: a device listed in `outputs.bit_perfect`. For the rules below, the value the device **runs** with counts: `live.devices[d].bit_perfect` (live settings §4.1), not the wanted value of `Config`. A pending "bit-perfect on" or "off" changes nothing until it is applied. |
| **Main device of a player** | `live.placement[Holder::PlayerMain(p)]`: where the engine really put the player's Main (live settings, "Placement"). A player the engine has not placed yet has none. |
| **Stream rate** | The rate a device's stream runs at now (`Bus::sample_rate`). On a bit-perfect device it follows the files (B3) and is not the configured rate. New in the model: `live.stream_rates` (BR1). |
| **Refused rates** | The rates a device refused (`Bus::refused_rates`, `crates/fp-engine/src/bus.rs` ~L167), not asked for again until the device comes back (B3.4). New in the model: `live.refused_rates` (BR1). |
| **Rate cut** | A transition between two entries of one player with no overlap, during which the device reopens at the next entry's rate (BR2). |
| **Fader locked** | The player's volume is held at exactly 1.0 (100 %, 0 dB) and nothing may change it (BF1). |
| **Manual start over the current** | Play while playing (the transport's Next button, "Fade into the next track", `play`, `crates/fp-model/src/reducer.rs` ~L464, `Transport::Playing` arm) and Previous (R24, `previous` ~L498): today both crossfade (`start_or_crossfade` ~L787). |

## 4. Rate cuts (replaces B3.3)

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

### 4.2 The decision

- **BR2 `fp_model::bit_perfect::rate_cut(state, player, next: &SourceRequest)
  -> Option<u32>`** gives the rate to cut to, or `None` for an ordinary
  transition. It is `Some(r)` exactly when all hold:
  1. the player has a Main device `d`, and `d` runs bit-perfect (§3);
  2. the stream rate of `d` is known;
  3. the next entry's format is known, it is PCM (`dsd_rate` is `None`),
     and its `sample_rate` `r` is not zero and differs from the stream rate;
  4. `r` is not among the refused rates of `d`;
  5. nothing else sounds on `d`: every holder placed on `d` other than
     `PlayerMain(player)` has no cause (`fp_model::live::causes`, live
     settings L1). This player's own CUE placed on `d` counts as something
     else.

  When any of these fails the transition is the ordinary one: the next
  track starts at the running rate, resampled if its rate differs (D2).

### 4.3 Scheduled transitions

- **BR3 A scheduled rate cut.** `plan_for` (`crates/fp-model/src/reducer.rs`
  ~L741) applies after the ordinary plan and the O25 hold: when the plan is
  `StartNextAt` and `rate_cut` gives `Some(r)` for the entry the player
  preloads (`preload_target`), the plan becomes `StartNextAt { at_secs: end,
  fade_current_until_secs: None, rate_cut: Some(r) }`, where `end` is the
  current entry's play-range end (`current_end`, ~L751: the cue-out with
  markers on, the end of the file without). The segue point and the
  crossfade are not used. `TransitionPlan::StartNextAt` gains the field
  `rate_cut: Option<u32>`; every other plan has `None`.
- **BR4 The plan follows the state until the transition.** `reconcile`
  already recomputes `plan_for` after every command and event and sends a
  new `Schedule` when it changes (~L1358). So a cart fired, or another
  player started, on the device while the current track plays turns the
  plan back into the ordinary one (segue, resampled); when it ends, the
  plan becomes a rate cut again. No new state is kept for this.
- **BR5 The engine executes a scheduled rate cut.** On a plan with
  `rate_cut: Some(r)`:
  1. the engine prepares the next source at `r` (the preload is opened
     again with `LoadOptions.rate = r`), so the cut does not wait for it to
     buffer; a plan that drops the cut has it opened again at the stream
     rate;
  2. at `at_secs` the current source stops with the usual declick
     (`tuning.declick_ms`);
  3. when the declick has ended the bus reopens at `r` as B3.2 does (the
     busy budget of a quiet bus; waiting sources of other players reopened
     at `r`), and the next source starts at once;
  4. the engine reports `TransitionStarted` as for any scheduled
     transition, and `StreamRate` (BR1).

  If, at step 3, something else sounds on the bus after all (a cart fired
  in the moment between the model's last plan and the cut), the bus keeps
  its rate and the next source, opened again at the stream rate, starts
  as soon as it is ready (resampled). If the reopen fails, B3.4 applies
  (back to the previous rate, the source resampled, the rate remembered as
  refused and reported in `StreamRate`). Either way the gap is longer but
  the next track plays.

### 4.4 Manual starts

- **BR6 A manual start over the current is a hard start when it is a rate
  cut.** In Play while playing and in Previous, when `rate_cut` gives
  `Some(_)` for the request the model is about to start, the model emits
  `EngineAction::StartCurrent` instead of `Crossfade`, as the O25 hold
  already does (`held` in `start_or_crossfade`); the player does not enter
  `fading`.
- **BR7 The engine reopens under a hard start.** In `start_current`
  (`crates/fp-engine/src/engine.rs` ~L2295) without a crossfade, the
  player's current source that the start replaces, and its declick tail,
  do not count as sounding for `prepare_start` (~L1358): the start waits
  for the tail to end, then `prepare_start` reopens the bus at the file's
  rate if nothing else sounds, then the source starts. With a crossfade,
  the outgoing source counts as sounding, as today. Starting from silence
  (Play from stopped or paused, a pre-listen, a cart) keeps B3.2 unchanged.

### 4.5 What does not change

- **BR8 Same rate, unknown rate, DSD.** Two tracks at the same rate segue
  and crossfade as today, and stay bit-perfect outside the overlap. A next
  entry not analysed yet (format unknown) plays at the running rate (B2).
  A DSD next entry, or a DSD stream on air, follows O25 unchanged (the
  `HoldOthers` hold is already a hard transition; the converted PCM of a
  DSD file does not trigger a rate cut). R26 repeats and O37 replays are
  the same file, so they never cut.
- **BR9 A sounding bus keeps its rate (the rest of B3.3).** Outside BR5 and
  BR7 a bus's rate still never changes while anything on it sounds, for the
  reason B3.3 gives (every frame-based timeline on the bus is in its
  frames). A rate cut is not an exception to that: at the reopen, nothing
  on the bus sounds.

### 4.6 Live settings

- **BR10 A rate cut is not a settings change.** It reopens the device with
  the settings it **runs** with (buffer size, exclusive access, DSD mode);
  it never applies a pending item. A pending device item keeps waiting on
  its causes (live settings L5): the player stays `Playing` in the model
  for the whole cut, so its cause never lapses and `live::due` emits
  nothing for that device during the gap.
- **BR11 The engine's quiet guard sees the cut.** A bus whose player is
  between the stop and the start of a rate cut (BR5 steps 2–3, BR7) is not
  quiet for live settings L11: an `ApplyDevice`, `ApplyRoute` or
  `ApplyAudioSystem` kept by the engine does not run in the gap.
- **BR12 A settings change on a bit-perfect device.** As live settings
  L12: a new configured rate is only where the device starts; bit-perfect
  turned off reopens the device shared at `effective_rate`, and the next
  transition is ordinary again (BR2.1 fails). Each reopen is reported
  through `StreamRate` (BR1).

## 5. The locked fader

### 5.1 Rules

- **BF1 `fp_model::bit_perfect::fader_locked(state, player) -> bool`:**
  true when the player's Main device runs bit-perfect (§3). A player not
  placed yet is not locked.
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
- **BF5 Fades are not the fader.** Fade stops, segue fade-outs, crossfades
  and pause ramps run as today (D4). They act on the mixer's gain, not on
  the player's volume, and BF3 does not see them.
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
  Advanced (`settings-bit-perfect-hint`) says that the device follows each
  track's rate with a short gap, and that the faders of the players on it
  are held at 100 %.

### 5.3 MIDI

- **BF10 A fader binding of a locked player is ignored.** In
  `Router::handle` (`crates/fp-control/src/router.rs` ~L110,
  `MidiAction::Volume`) the message gives no command when
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

- **BF12 Refused, with a reason.** `POST /players/{id}/fader` with a value
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
  additive (no breaking change for clients).

### 5.5 O25 (DSD) amended

The O25 rule "the fader leaving unity switches a DSD stream to PCM"
(`Command::SetVolume`, `crates/fp-model/src/reducer.rs` ~L108,
`EngineAction::LeaveDsd`) can no longer be reached from the fader, since
a DSD stream only plays on a bit-perfect device (BF1, BF3). It stays as a
guard and its test stays; the DSD decision's volume check (`DsdFacts`)
stays too.

## 6. Interface strings

New and changed keys. `en-US` and `es-ES` are written by hand; the others
are AI translations, to be checked like every locale. `tip-volume-dsd` is
removed from every locale (BF7). `{ $players }` is a list such as "P1, P2"
built from the existing player labels.

### 6.1 `tip-volume-locked` (new)

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
| en-US | Exclusive access, and the device follows each track's sample rate: when nothing else plays on it, a change of rate between two tracks is a hard cut with a short gap. The volume of the players on it is held at 100 %. With no fade and nothing else on the same outputs, the samples reach it unchanged; the BP badge lights while that is true. |
| es-ES | Acceso exclusivo, y el dispositivo sigue la frecuencia de muestreo de cada pista: cuando no suena nada más en él, un cambio de frecuencia entre dos pistas es un corte seco con una breve pausa. El volumen de los reproductores que salen por él queda fijo al 100 %. Sin fundido y sin nada más en las mismas salidas, las muestras le llegan sin cambios; el distintivo BP se enciende mientras sea así. |
| ca-ES | Accés exclusiu, i el dispositiu segueix la freqüència de mostreig de cada pista: quan no hi sona res més, un canvi de freqüència entre dues pistes és un tall sec amb una breu pausa. El volum dels reproductors que hi surten queda fix al 100 %. Sense fos i sense res més a les mateixes sortides, les mostres hi arriben sense canvis; el distintiu BP s'encén mentre és així. |
| de-DE | Exklusiver Zugriff, und das Gerät folgt der Abtastrate jedes Tracks: Spielt sonst nichts darauf, wird ein Ratenwechsel zwischen zwei Tracks zu einem harten Schnitt mit einer kurzen Pause. Die Lautstärke der Player darauf bleibt fest auf 100 %. Ohne Blende und ohne andere Quellen auf denselben Ausgängen erreichen die Samples es unverändert; solange das gilt, leuchtet das BP-Symbol. |
| eu-ES | Sarbide esklusiboa, eta gailuak pista bakoitzaren lagintze-maiztasuna jarraitzen du: bertan beste ezer entzuten ez denean, bi pisten arteko maiztasun-aldaketa ebaki lehor bat da, eten labur batekin. Bertatik ateratzen diren erreproduzigailuen bolumena % 100ean finkatuta geratzen da. Itzaltzerik gabe eta irteera berberetan beste ezer gabe, laginak aldaketarik gabe iristen zaizkio; hala den bitartean, BP ikurra pizten da. |
| fr-FR | Accès exclusif, et le périphérique suit la fréquence d'échantillonnage de chaque piste : quand rien d'autre n'y est diffusé, un changement de fréquence entre deux pistes devient une coupure franche avec un court blanc. Le volume des lecteurs qui y sortent reste bloqué à 100 %. Sans fondu et sans rien d'autre sur les mêmes sorties, les échantillons lui parviennent intacts ; le badge BP s'allume tant que c'est le cas. |
| gl-ES | Acceso exclusivo, e o dispositivo segue a frecuencia de mostraxe de cada pista: cando non soa nada máis nel, un cambio de frecuencia entre dúas pistas é un corte seco cunha breve pausa. O volume dos reprodutores que saen por el queda fixo ao 100 %. Sen fundido e sen nada máis nas mesmas saídas, as mostras chéganlle sen cambios; o distintivo BP acéndese mentres sexa así. |
| it-IT | Accesso esclusivo, e il dispositivo segue la frequenza di campionamento di ogni brano: quando non vi suona nient'altro, un cambio di frequenza tra due brani diventa uno stacco netto con una breve pausa. Il volume dei lettori che vi escono resta bloccato al 100 %. Senza dissolvenza e senza altro sulle stesse uscite, i campioni gli arrivano inalterati; il badge BP si accende finché è così. |
| nl-NL | Exclusieve toegang, en het apparaat volgt de samplefrequentie van elk nummer: speelt er verder niets op, dan wordt een frequentiewissel tussen twee nummers een harde overgang met een korte pauze. Het volume van de spelers erop staat vast op 100 %. Zonder fade en zonder iets anders op dezelfde uitgangen bereiken de samples het ongewijzigd; de BP-badge brandt zolang dat zo is. |
| pl-PL | Dostęp na wyłączność, a urządzenie dostosowuje się do częstotliwości próbkowania każdego utworu: gdy nic innego na nim nie gra, zmiana częstotliwości między dwoma utworami staje się twardym cięciem z krótką przerwą. Głośność odtwarzaczy na nim jest zablokowana na 100 %. Bez wyciszania i bez niczego innego na tych samych wyjściach próbki docierają do niego bez zmian; wskaźnik BP świeci, dopóki tak jest. |
| pt-PT | Acesso exclusivo, e o dispositivo segue a frequência de amostragem de cada faixa: quando nada mais está a tocar nele, uma mudança de frequência entre duas faixas é um corte seco com uma breve pausa. O volume dos leitores que saem por ele fica fixo a 100 %. Sem fade e sem mais nada nas mesmas saídas, as amostras chegam-lhe sem alterações; o indicador BP acende enquanto assim for. |

The remote API's error message (BF3, BF12) is English, like every API
message, and is not a Fluent string.

## 7. Model and engine changes (summary)

| Crate | Change |
|---|---|
| `fp-model` | New module `bit_perfect` (`rate_cut`, `fader_locked`, `faders_reset_by`); `LiveSettings` gains `stream_rates` and `refused_rates`; `EngineEvent::StreamRate`; `TransitionPlan::StartNextAt::rate_cut`; `plan_for` (BR3); `play` and `previous` (BR6); `reconcile` (BF2); `Command::SetVolume` (BF3); `ModelError::FaderLocked`; `Availability::fader`. |
| `fp-engine` | `StreamRate` reports (BR1); the scheduled rate cut (BR5); `start_current` and `prepare_start` under a hard start (BR7); the quiet guard (BR11). |
| `fp-control` | `Router::handle` for `Volume` (BF10). |
| `fp-remote` | `PlayerDto::fader_locked`, OSC `volume-locked` (BF13); refusals come from the model (BF12). |
| `fp-app` | `widgets::fader` `enabled`; `meter_column` (BF7); the Apply now confirmation (BF8); locales (§6). |

No configuration field is added: a rate cut has nothing to tune (its gap
is the device's reopen time plus `tuning.declick_ms`, both already
configurable or the device's own), and the lock has no setting (Q1).

## 8. Testing

One test per rule, watched failing first. Engine tests use the `Offline`
backend and explicit `now: Instant`; nothing sleeps.

| Rule | Layer, file | Test |
|---|---|---|
| BR1 | `fp-model` `tests/bit_perfect_rules.rs` | `StreamRate` sets `live.stream_rates` and `refused_rates`; `Unplaced` of the last holder forgets them. |
| BR1 | `fp-engine` `tests/bit_perfect.rs` | a bus open, a B3.2 reopen and a refused rate each report `StreamRate` with the right rate and refused set. |
| BR2 | `fp-model` `tests/bit_perfect_rules.rs` | one test per condition 1–5: each one false gives `None`; all true gives `Some(r)`. |
| BR3 | `fp-model` `tests/bit_perfect_rules.rs` | a segue plan into a 96 kHz entry on a 44.1 kHz bit-perfect device becomes `StartNextAt { at: cue-out, fade: None, rate_cut: Some(96000) }`. |
| BR4 | `fp-model` `tests/bit_perfect_rules.rs` | a cart starting on the device sends a `Schedule` with the segue plan; `CartEnded` sends the rate cut again. |
| BR5 | `fp-engine` `tests/bit_perfect.rs` | the preload is opened at `r`; at `at_secs` the current stops, `OfflineDevice::config()` shows `r`, the next starts and is bit-perfect; `TransitionStarted` is reported. A cart fired just before the cut: no reopen, the next plays resampled. A refused rate: B3.4 fallback, the next plays. |
| BR6 | `fp-model` `tests/bit_perfect_rules.rs` | Play while playing and Previous emit `StartCurrent` (not `Crossfade`) when a rate cut applies, and `Crossfade` when not. |
| BR7 | `fp-engine` `tests/bit_perfect.rs` | a hard `StartCurrent` at another rate reopens the bus after the declick; a `Crossfade` at another rate does not (resampled). |
| BR8 | `fp-model` `tests/bit_perfect_rules.rs` | same rate, unknown format and a DSD next entry keep the ordinary plan. |
| BR9 | `fp-engine` `tests/bit_perfect.rs` | the existing B3.3 test (a source started on a sounding bus is resampled) still passes. |
| BR10 | `fp-model` `tests/live.rs` | with a rate cut scheduled, a pending buffer change on the player's device is not due at any point up to `TransitionStarted` (the player stays `Playing`, so it stays a cause). |
| BR11 | `fp-engine` `tests/live_settings.rs` | an `ApplyDevice` kept by the guard does not run in the rate cut's gap; it runs once the device is quiet. |
| BR12 | `fp-engine` `tests/live_settings.rs` | bit-perfect off reports `StreamRate` at `effective_rate`; the next transition is a crossfade. |
| BF1 | `fp-model` `tests/bit_perfect_rules.rs` | locked by the running settings, not the wanted ones; unplaced is not locked. |
| BF2 | `fp-model` `tests/bit_perfect_rules.rs` | `Applied` bit-perfect on, `Placed` on a bit-perfect device and a restored volume of 0.5 each give volume 1.0 and one `SetVolume { 1.0 }`. |
| BF3 | `fp-model` `tests/bit_perfect_rules.rs` | `SetVolume(p, 0.5)` refused with `FaderLocked`, state unchanged; `SetVolume(p, 1.0)` accepted; `availability(..).fader` false. |
| BF4 | `fp-model` `tests/bit_perfect_rules.rs` | `Applied` bit-perfect off leaves volume 1.0 and accepts `SetVolume(p, 0.5)`. |
| BF5 | `fp-model` `tests/bit_perfect_rules.rs` | `FadeStop` and a segue still produce `FadeOutAndStop` and `Crossfade` on a locked player. |
| BF6 | `fp-model` `tests/bit_perfect_rules.rs` | a player whose Cue route is the bit-perfect device and Main is not: not locked, CUE starts. |
| BF7 | `fp-app` `tests/` (kittest, player) | a locked player's fader is disabled, does not send `SetVolume` on drag, and shows `tip-volume-locked`. |
| BF8 | `fp-app` `tests/live_settings.rs` (kittest) | the Apply now confirmation lists `apply-now-fader-reset` with the player; `faders_reset_by` unit test in `fp-model`. |
| BF9 | `fp-app` i18n test | every locale has the new keys and no `tip-volume-dsd` (the existing key-parity test). |
| BF10 | `fp-control` `tests/router.rs` | a fader message for a locked player gives nothing; after unlocking, a control at 50 % gives nothing until it reaches the top. |
| BF11 | `fp-control` `tests/feedback.rs` | feedback for a locked player's buttons is unchanged. |
| BF12 | `fp-remote` `tests/api.rs`, `tests/osc_server.rs` | fader 0.5 on a locked player → 409 `unavailable` with the model's message; fader 1 → 200; the OSC message is refused and noted. |
| BF13 | `fp-remote` `tests/dto.rs`, `tests/osc.rs` | `fader_locked` and `volume-locked` follow the lock. |

The manual loopback check of the bit-perfect docs gains a step: play a
44.1 kHz track followed by a 96 kHz one in continuous mode; the converter
shows 96 kHz on the second track and the BP badge lights on both.

## 9. Documentation to update

- `docs/user/bit-perfect.md`: "The rate follows the file" (rate cuts, the
  gap, when the next track is resampled instead); the BP conditions (the
  volume is held at 100 %); the DSD section (the fader is locked, so the
  "moving the fader switches to PCM" lines go); the loopback procedure
  (§8).
- `docs/user/players.md` (the fader), `docs/user/markers-and-mixing.md`
  (no segue across a rate change on a bit-perfect output),
  `docs/user/settings.md` (Audio outputs, bit-perfect),
  `docs/user/midi.md` (a locked fader binding is ignored and picks up at
  100 %), `docs/user/remote-control.md` (409 on a locked fader,
  `fader_locked`, `volume-locked`).
- `docs/technical/audio-engine.md` ("Rates and bit-perfect buses": rate
  cuts, `StreamRate`, hard starts), `docs/technical/remote-api.md`
  (`fader_locked`, the 409, the OSC address), `docs/technical/ui.md` if
  it describes the fader.
- `README.md`: the bit-perfect feature line (the rate follows each track;
  the volume is held at 100 %).
- Specs: B3.3 marked "Replaced by the bit-perfect rate and fader spec
  (2026-10-08)"; B1 and B7 notes ("Automatic volume bypass" and "Changing
  the rate while a bus is busy" are no longer out of scope in the form of
  §4 and §5); live settings §13 "the bit-perfect rate change at
  transitions" points here; feedback-2 O25 note (§5.5).
- Locales: §6 in all eleven files.

## 10. Out of scope

- Changing the rate while anything else sounds on the device (D2), or in
  the middle of a track.
- A rate cut for the Cue path, carts, or DSD entries (BR8).
- Gapless rate changes: the gap is the device's reopen and cannot be
  hidden without resampling.
- Fader position feedback to MIDI surfaces (motorised faders, LED rings)
  (BF11).
- Locking any gain other than the player's volume (carts, CUE).
- Shortening the gap further than BR5.1 does (for example keeping a second
  stream open at the next rate).

## 11. Decisions taken in this spec (for review)

| # | Decision | Rationale |
|---|---|---|
| S1 | The model decides a rate cut (BR2) from a new `StreamRate` report, not the engine on its own. | Behaviour lives in `fp-model` (D6); the device's running rate on a bit-perfect device follows the files and is not in `DeviceSettings`, so it needs its own report. The engine keeps the final word only when the moment has changed (BR5 fallback). |
| S2 | Known refused rates never cause a cut (BR2.4). | A cut into a rate the device refuses would cost a gap and still resample, on every such transition. |
| S3 | The scheduled cut carries its rate in the plan, and the engine prepares the next source at that rate in advance (BR5.1). | Otherwise the next source is opened again after the reopen and the gap includes its buffering (`tuning.ready_threshold_ms`, 500 ms by default). |
| S4 | The current plays to the end of its play range (the cue-out with markers on), not to its segue point. | D1 says "to its cue-out"; the segue point exists only to overlap. |
| S5 | A manual start over the current (Play while playing and Previous) becomes a hard start when it is a rate cut (BR6). | D1 "stops on a manual Next"; Previous crossfades the same way, so it follows the same rule. |
| S6 | The lock follows the bit-perfect setting the device **runs** with, even when exclusive access was refused (B4). | Predictable for the operator (the checkbox decides), and a later reopen may obtain exclusive access; the BP badge already says when the output is not bit-perfect. |
| S7 | One invariant (BF2) resets the fader in every case: bit-perfect on, a route moved onto the device, an audio system change, a new player, a restored session. | One rule, one test, no path forgotten. Every such change applies only while the player is idle (live settings L5, L6) unless the operator forces it (BF8). |
| S8 | The remote API and OSC **refuse** a locked fader (409 `unavailable`) and accept exactly 100 % (BF12). | It is the existing pattern for operations the model refuses in the current state; silently ignoring would let a remote surface show a level that is not on air. |
| S9 | A locked MIDI fader is ignored and must pick up again at 100 % after unlocking (BF10). | Soft takeover already exists for this; without it the first touch after unlocking would jump the level. |
| S10 | No MIDI fader feedback is added (BF11). | Feedback today covers buttons only; there is no fader position output to set to 100 %. |
| S11 | `tip-volume-dsd` is removed and the O25 fader rule becomes unreachable but stays (§5.5). | A DSD stream needs a bit-perfect device, whose fader is now locked. |
| S12 | The CUE path and carts are untouched (BF6). | The brief's lock is about the on-air player fader; CUE has its own gain and no fader, and the BP badge follows Main only (B6). |
| S13 | `PlayerDto::fader_locked` and OSC `volume-locked` are added (BF13). | Remote surfaces must be able to grey their fader as the interface does; both are additive. |

## 12. Open questions for the maintainer

- **Q1** Should rate cuts be optional per device (for example an
  `outputs.rate_cut` list or a per-device "follow the rate between tracks"
  switch, on by default), for stations that prefer an uninterrupted segue
  resampled over a gap? This spec has no switch: D1 makes the cut the
  behaviour of every bit-perfect device.
