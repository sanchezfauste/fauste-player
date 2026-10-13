# Bit-perfect Hard Transitions, Plan 2: the Engine Follows the Music

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The engine reports every bus's stream rate (BR1), executes a scheduled hard transition that carries a rate cut — the next track prepared at the new rate, the current stopped with its declick, the device reopened, the next started at once (BR6) — and reopens the device under a manual hard start at another rate (BR8), with the live-settings quiet guard covering the gap (BR13).

**Architecture:** A `Playing` source remembers the rate its worker decodes to (`Playing::rate`), so a preload can be opened at the rate of a planned cut while the bus still runs at the old one, and the engine never starts a source whose rate is not its bus's. `TransitionPlan::StartNextAt { rate_cut: Some(r) }` (plan 1) is dispatched as a stop of the current track at `at_secs` (the existing `StopAt` + `RampOutBeforeCut`); when the mixer reports the current finished, `current_finished` reopens the bus at `r` through `prepare_start` (B3.2: only when nothing else sounds) and starts the preload as it does at any end of stream. A manual `StartCurrent` at another rate on a bit-perfect bus where only this player sounds keeps the new current waiting (`PlayerRuntime::rate_cut`) until the old one's declick tail has ended, then reopens and starts it (`run_rate_cuts`, once per tick). `report_stream_rates` compares each bus's rate and refused rates with what it last reported, at the end of every tick, and emits `EngineEvent::StreamRate` for the differences. Nothing here runs on the device callback: no new mixer command.

**Tech Stack:** Rust 2024, `fp-engine` (conductor-thread engine, `Bus`, the real-time mixer untouched), `fp-backends` Offline backend for tests.

**Spec:** `docs/superpowers/specs/2026-10-08-bit-perfect-rate-and-fader-design.md` (§4.1 BR1, §4.3 BR6, §4.4 BR8, §4.5 BR10, BR11, §4.6 BR13, BR14, §7 `fp-engine` row, §9.2 `audio-engine.md` and `audio-path-audit.md`). Also `docs/superpowers/specs/2026-09-26-phase4-bit-perfect-design.md` B3 (the rate rules this extends) and `docs/technical/audio-engine.md` "Rates and bit-perfect buses".

This is plan 2 of 4 (after `2026-10-13-bit-perfect-plan1-model-rules.md`, which must be merged first: it adds `EngineEvent::StreamRate` and `TransitionPlan::StartNextAt::rate_cut`).

## Rulings (planning)

**Ruling: one reporting point for stream rates.** `report_stream_rates` runs at the end of every `tick` and reports each bus whose `(sample_rate, refused_rates)` differs from what it last reported (a new bus has nothing reported). — Why: BR1 lists six causes (open, B3.2 reopen, rate cut, B3.4 fallback, live-settings reopen, watchdog), each in a different function; one comparison covers them all, and any future cause. — Cost if wrong: a rate change made inside `execute` reaches the model one tick (≤ 10 ms) later; the model only uses it for the next plan.

**Ruling: the scheduled cut reopens on the current's `Finished`.** The dispatch of a cut plan sends the existing `StopAt` + `RampOutBeforeCut` for the current (the declick ends on `at_frame`, as at any hard cut inside the file), and leaves the preload idle; the reopen and the start happen in `current_finished`, the path every end of stream already takes. — Why: "when the declick has ended" (BR6.3) is exactly the current slot's `Finished`; no new state is needed for a scheduled cut. — Cost if wrong: the gap includes the tick that sees `Finished` (≤ one block).

**Ruling: a source never starts at a rate other than its bus's.** `fit_preload` (on `Schedule`, before a non-cut dispatch, and in the cut's fallback) and the rate check in `take_or_open` reopen a preload whose `rate` is not the one it must play at. — Why: a preload opened at the rate of a cut that is then dropped (a cart fired, BR5) would otherwise play at the wrong speed. — Cost if wrong: an extra open of the next track, which BR6 accepts ("a plan that drops the cut has it opened again at the stream rate").

**Ruling: a pause, stop, seek or new start drops a waiting manual cut.** `run_rate_cuts` drops the cut when the player is paused or its current is no longer the waiting source; a resume then follows B3.2 like a track loaded paused (`resume` already calls `prepare_start`). `stop_now` and `start_current` clear it explicitly. — Cost if wrong: in that rare case the track starts at the running rate, resampled.

**Ruling: DSD files never cause a manual cut.** BR8 reopens only for a PCM file (`dsd_rate` is `None`), as BR3.3 does for scheduled cuts; a DSD file keeps the O25 path (`try_start_dsd`). — Cost if wrong: one condition.

**Ruling: BR13 and BR14 engine tests live in `tests/bit_perfect.rs`.** §8 puts them in `tests/live_settings.rs`, whose rig decodes with `tagged_opener` and requests carry no format; a rate cut needs real files at two rates and a bit-perfect bus, which `tests/bit_perfect.rs` already builds. — Cost if wrong: two tests in another file.

## Global Constraints

- All code, identifiers, comments, docs and commit messages in English; never mention other playout or radio-automation products.
- Real-time safety (CLAUDE.md rule 5): nothing new runs on the device callback; every reopen happens on the conductor thread, as B3.2 already does.
- BR6.3: "when the declick has ended the bus reopens at `r` as B3.2 does (the busy budget of a quiet bus; waiting sources of other players reopened at `r`), and the next source starts at once."
- BR6 fallback: "If, at step 3, something else sounds on the bus after all … the bus keeps its rate and the next source, opened again at the stream rate, starts as soon as it is ready (resampled). If the reopen fails, B3.4 applies … Either way the gap is longer but the next track plays."
- BR8: "the player's current source that the start replaces, and its declick tail, do not count as sounding for `prepare_start`; the start waits for the tail to end, then `prepare_start` reopens the bus at the file's rate if nothing else sounds, then the source starts. At the same rate the start is today's hard start … Starting from silence … keeps B3.2 unchanged."
- BR11: outside BR6 and BR8 a sounding bus never changes rate; a `Crossfade` never reopens.
- BR12: a rate cut "never applies a pending item".
- BR13: "A bus whose player is between the stop and the start of a rate cut … is not quiet for live settings L11."
- `fp-engine` denies `clippy::indexing_slicing`; `unwrap`, `expect`, `panic` denied outside tests.
- Engine tests use the `Offline` backend and explicit time (the rig's `settle` polls the workers with a deadline; never sleep to wait for audio).
- Conventional Commits; every commit ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Gate for every task: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.
- Progress ledger: `.superpowers/sdd/2026-10-13-bit-perfect-plan2-engine/progress.md`.

## Review Focus

1. **A scheduled cut whose plan is dropped after the preload was opened at the new rate** (a cart fired after the `Schedule`, BR5): the next `Schedule` reopens the preload at the stream rate; it must never start at the cut's rate on a bus at the old one (it would play at the wrong speed). Test: Task 2, `br6_a_dropped_cut_opens_the_next_again_at_the_stream_rate`.
2. **The operator presses Next during a scheduled cut's last seconds** (`StartCurrent` of the preloaded entry, opened at the cut's rate): `take_or_open` must not take a preload at a rate the start will not run at. Test: Task 4, `br8_next_takes_a_preload_opened_for_a_cut_only_at_its_rate`.
3. **Stop pressed in the gap of a manual cut** (after the old track's declick, before the new one starts): nothing starts afterwards, and the device does not reopen for a track that will not play. Test: Task 4, `br8_stop_in_the_gap_starts_nothing`.
4. **A pending device change waiting for the player while the cut's gap runs** (live settings L11): it must not run in the gap. Test: Task 4, `br13_a_kept_device_change_waits_through_the_gap`.
5. **The device refuses the cut's rate** (B3.4): the previous rate stays, the rate is reported refused, and the next track plays (resampled). Test: Task 3, `br6_a_refused_cut_falls_back_and_the_next_plays`.

---

## File Structure

| File | Responsibility |
|---|---|
| `crates/fp-engine/src/bus.rs` (`refused_rates` field L165-167, accessors ~L360) | `Bus::refused_rates()` |
| `crates/fp-engine/src/engine.rs` | `Playing::rate`, `PlayerRuntime::rate_cut`, `Engine::reported_rates`; `open_source_at`, `preload_rate`, `fit_preload`, `reopen_preload_at`, `cut_to`, `hard_start_cut`, `sounding_besides`, `run_rate_cuts`, `report_stream_rates`; changes in `preload`, `take_or_open`, `start_current`, `reopen_waiting`, `current_finished`, `dispatch`, `stop_now`, `pause`, `player_quiet`, `tick`, `execute` (`Schedule`) |
| `crates/fp-engine/tests/bit_perfect.rs` | the rig collects events; BR1, BR6, BR8, BR11, BR13, BR14 tests |
| `docs/technical/audio-engine.md`, `docs/technical/audio-path-audit.md` | Task 5 |

---

### Task 1: The engine reports stream rates (BR1)

**Files:**
- Modify: `crates/fp-engine/src/bus.rs` (after `sample_rate` ~L360-362)
- Modify: `crates/fp-engine/src/engine.rs` (`Engine` struct L333-380: a field after `orphans`; `Engine::new` ~L471-530: its initialiser; `tick` L2678-2709: a call before `std::mem::take(&mut self.events)`; a new method after `report_placement` ~L748-759)
- Test: `crates/fp-engine/tests/bit_perfect.rs` (the `Rig` L29-34 and its `settle` / `run` L114-136; new tests)

**Interfaces:**
- Consumes: `EngineEvent::StreamRate { device, rate, refused }` (plan 1).
- Produces: `pub fn refused_rates(&self) -> Vec<u32>` on `Bus` (ascending); `Rig::events: Vec<EngineEvent>` and `fn stream_rates(&self) -> Vec<(u32, Vec<u32>)>` in the test rig (used by every later task).

- [ ] **Step 1: The rig keeps the events**

In `tests/bit_perfect.rs`, `Rig` gains `events: Vec<EngineEvent>` (initialised `Vec::new()` in `rig`), and every `self.engine.tick(self.clock);` in `settle` and `run` becomes `self.tick();` with

```rust
    fn tick(&mut self) {
        let events = self.engine.tick(self.clock);
        self.events.extend(events);
    }

    /// The `StreamRate` reports so far, in order.
    fn stream_rates(&self) -> Vec<(u32, Vec<u32>)> {
        self.events
            .iter()
            .filter_map(|e| match e {
                EngineEvent::StreamRate { rate, refused, .. } => Some((*rate, refused.clone())),
                _ => None,
            })
            .collect()
    }

    fn transition_started(&self, entry: u64) -> bool {
        self.events.iter().any(|e| {
            matches!(e, EngineEvent::TransitionStarted { entry: EntryId(n), .. } if *n == entry)
        })
    }
```

(Add `EngineEvent` and `TransitionPlan` to the `fp_model` imports.)

- [ ] **Step 2: Write the failing tests**

```rust
#[test]
fn br1_a_bus_reports_its_rate_when_it_opens_and_when_it_follows_a_file() {
    let mut r = rig(true, true);
    r.settle();
    assert_eq!(r.stream_rates(), vec![(48_000, vec![])], "opened at the global rate");
    let path = wav(r.dir.path(), "a.wav", 44_100);
    r.start(request(1, path, pcm16(44_100)));
    assert_eq!(r.stream_rates().last(), Some(&(44_100, vec![])), "a B3.2 reopen");
    let before = r.stream_rates().len();
    r.run(4_410);
    assert_eq!(r.stream_rates().len(), before, "nothing changed, nothing reported");
}

#[test]
fn br1_a_refused_rate_is_reported() {
    let mut r = rig(true, true);
    r.dac.refuse_rate(44_100);
    let path = wav(r.dir.path(), "a.wav", 44_100);
    r.start(request(1, path, pcm16(44_100)));
    assert_eq!(r.stream_rates().last(), Some(&(48_000, vec![44_100])));
}
```

- [ ] **Step 3: Run to verify it fails**

Run: `cargo test -p fp-engine --test bit_perfect br1 2>&1 | tail -8`
Expected: FAIL — `stream_rates()` is empty.

- [ ] **Step 4: Implement**

`bus.rs`, after `sample_rate`:

```rust
    /// The rates this device refused (Phase 4 spec B3.4), ascending.
    pub fn refused_rates(&self) -> Vec<u32> {
        let mut rates: Vec<u32> = self.refused_rates.iter().copied().collect();
        rates.sort_unstable();
        rates
    }
```

`engine.rs`, `Engine` gains after `orphans`:

```rust
    /// The stream rate and refused rates last reported for each bus
    /// (bit-perfect rate and fader spec BR1).
    reported_rates: HashMap<BusKey, (u32, Vec<u32>)>,
```

initialised with `reported_rates: HashMap::new(),` in `Engine::new`. After `report_placement`:

```rust
    /// BR1: reports each bus whose stream rate or refused rates changed
    /// since it was last reported (a bus just opened has nothing reported),
    /// whatever changed them: a B3.2 reopen, a rate cut, a refusal, a
    /// live-settings reopen or the watchdog. Called at the end of `tick`.
    fn report_stream_rates(&mut self) {
        let buses = &self.buses;
        self.reported_rates.retain(|key, _| buses.contains_key(key));
        for (key, bus) in &self.buses {
            let now = (bus.sample_rate(), bus.refused_rates());
            if self.reported_rates.get(key) != Some(&now) {
                self.events.push(EngineEvent::StreamRate {
                    device: key.output_device(),
                    rate: now.0,
                    refused: now.1.clone(),
                });
                self.reported_rates.insert(key.clone(), now);
            }
        }
    }
```

`tick`: call `self.report_stream_rates();` after `self.send_tunes();`.

- [ ] **Step 5: Run to verify it passes**

Run: `cargo test -p fp-engine`
Expected: PASS (other engine tests ignore the new events; check `tests/live_settings.rs`, which asserts on whole event lists in a few places — e.g. `l3_the_engine_reports_its_audio_system_and_where_each_holder_plays` — and filter `StreamRate` out there if one compares a full list).

- [ ] **Step 6: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-engine
git commit -m "feat(engine): report each bus's stream rate and refused rates

At the end of every tick the engine tells the model the rate each bus
runs at and the rates its device refused, whenever they changed, so the
model can plan rate cuts (bit-perfect rate and fader spec BR1).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: Sources know their rate; a planned cut's next is prepared at it (BR6.1)

**Files:**
- Modify: `crates/fp-engine/src/engine.rs`: `Playing` (L255-282) gains `rate`; `open_source` (L1968-2048) becomes `open_source_at` with a `rate: Option<u32>` parameter (`open_source` and `new_source` keep their signatures and pass `None`); `preload` (L2254-2276); `take_or_open` (L2279-2293) and its caller in `start_current` (~L2343); `reopen_waiting` (L1385-1432); `execute`'s `Schedule` arm (~L1823-1828); new `preload_rate`, `reopen_preload_at`, `fit_preload` after `take_or_open`
- Test: `crates/fp-engine/tests/bit_perfect.rs`

**Interfaces:**
- Consumes: `TransitionPlan::StartNextAt::rate_cut` (plan 1).
- Produces (private to `Engine`, used by Tasks 3–4): `fn open_source_at(&mut self, player: PlayerId, cue: bool, request: &SourceRequest, dsd: bool, rate: Option<u32>) -> Result<Playing, AttachError>`; `fn preload_rate(&self, player: PlayerId) -> Option<u32>`; `fn reopen_preload_at(&mut self, player: PlayerId, rate: u32)`; `fn fit_preload(&mut self, player: PlayerId)`; `fn take_or_open(&mut self, player: PlayerId, request: &SourceRequest, rate: Option<u32>) -> Result<Playing, AttachError>`; `Playing::rate: u32`.

- [ ] **Step 1: Write the failing tests**

```rust
fn cut_plan(at_secs: f64, rate_cut: Option<u32>) -> EngineAction {
    EngineAction::Schedule {
        player: P,
        plan: Some(TransitionPlan::StartNextAt {
            at_secs,
            fade_current_until_secs: None,
            rate_cut,
        }),
    }
}

/// A 44.1 kHz track on air on the bit-perfect bus, and a 48 kHz next preloaded.
fn on_air_with_next(r: &mut Rig) {
    let a = wav(r.dir.path(), "a.wav", 44_100);
    let b = wav(r.dir.path(), "b.wav", 48_000);
    r.start(request(1, a, pcm16(44_100)));
    assert_eq!(r.rate(), 44_100);
    r.act(EngineAction::Preload {
        player: P,
        request: Some(request(2, b, pcm16(48_000))),
    });
}

/// The decode rate of P's source in `part` (`"cur ["` or `"pre ["`), read
/// from the engine's one-line description.
fn decode_rate(r: &Rig, part: &str) -> Option<u32> {
    let d = r.engine.describe_player(P);
    let source = d.split(part).nth(1)?.split(']').next()?;
    source.split("rate ").nth(1)?.split(' ').next()?.parse().ok()
}

#[test]
fn br6_the_next_of_a_planned_cut_is_prepared_at_the_new_rate() {
    let mut r = rig(true, true);
    on_air_with_next(&mut r);
    assert_eq!(decode_rate(&r, "pre ["), Some(44_100), "no cut planned yet");
    r.act(cut_plan(0.5, Some(48_000)));
    r.settle();
    assert_eq!(r.rate(), 44_100, "nothing reopens before the transition");
    assert_eq!(decode_rate(&r, "pre ["), Some(48_000));
}

#[test]
fn br6_a_dropped_cut_opens_the_next_again_at_the_stream_rate() {
    let mut r = rig(true, true);
    on_air_with_next(&mut r);
    r.act(cut_plan(0.5, Some(48_000)));
    r.settle();
    r.act(cut_plan(0.5, None));
    r.settle();
    assert_eq!(decode_rate(&r, "pre ["), Some(44_100));
}
```

`describe_player` (L3255) is the engine's one-line description for logs and test failures; this task makes it print each source's `rate` (Step 3), which is how a test sees the rate a source decodes to without a new public API. (The plan's `Debug` in the same line may also show a rate; `decode_rate` reads only inside the source's brackets.)

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p fp-engine --test bit_perfect br6_the_next 2>&1 | tail -8`
Expected: FAIL — the description has no rate.

- [ ] **Step 3: Implement**

`Playing` gains, after `start_secs`:

```rust
    /// The rate its worker decodes to: its bus's rate when it was opened,
    /// or the rate of the cut its player's plan carries (bit-perfect rate
    /// and fader spec BR6.1). It starts only when its bus runs at it.
    rate: u32,
```

`open_source` keeps its signature and becomes `self.open_source_at(player, cue, request, dsd, None)`; `open_source_at` is the old body with:

```rust
        // A bus running faster than configured needs more frames for the
        // same seconds of buffer.
        let decode_rate = rate.unwrap_or_else(|| self.rate_of(&bus_key));
        let ring = ((self.settings.tuning.prebuffer_secs
            * f64::from(decode_rate.max(self.settings.sample_rate))) as usize)
            .max(1024);
```

(replacing the first `let rate = …` and the `let ring = …`), `let rate = decode_rate;` instead of the second `let rate = self.rate_of(&bus_key);`, and `rate,` in the `Playing { … }` literal.

After `take_or_open`:

```rust
    /// The rate `player`'s preload must decode at now: the rate of the cut
    /// its plan carries, on a bit-perfect Main bus (BR6.1), else its Main
    /// bus's rate.
    fn preload_rate(&self, player: PlayerId) -> Option<u32> {
        let rt = self.players.get(&player)?;
        let cut = match rt.plan {
            Plan::Waiting(TransitionPlan::StartNextAt { rate_cut, .. })
            | Plan::Dispatched(TransitionPlan::StartNextAt { rate_cut, .. }, _) => rate_cut,
            _ => None,
        };
        let bus = &rt.main.0;
        Some(
            cut.filter(|_| self.device_of(bus).bit_perfect)
                .unwrap_or_else(|| self.rate_of(bus)),
        )
    }

    /// Opens `player`'s waiting preload again at `rate` when it decodes at
    /// another one.
    fn reopen_preload_at(&mut self, player: PlayerId, rate: u32) {
        let stale = self.players.get(&player).is_some_and(|rt| {
            rt.preload
                .as_ref()
                .is_some_and(|p| p.start == StartState::Idle && p.rate != rate)
        });
        if !stale {
            return;
        }
        let Some(old) = self.players.get_mut(&player).and_then(|rt| rt.preload.take()) else {
            return;
        };
        let request = old.request.clone();
        self.release(old);
        let fresh = self
            .open_source_at(player, false, &request, false, Some(rate))
            .ok();
        if let Some(rt) = self.players.get_mut(&player) {
            rt.preload = fresh;
        }
    }

    /// Opens `player`'s preload again if it does not decode at the rate it
    /// must play at now (`preload_rate`): a plan that gains a cut prepares
    /// the next at the new rate, one that drops it at the stream rate.
    fn fit_preload(&mut self, player: PlayerId) {
        if let Some(rate) = self.preload_rate(player) {
            self.reopen_preload_at(player, rate);
        }
    }
```

`preload`: the new source is opened with `self.open_source_at(player, false, &req, false, self.preload_rate(player)).ok()` instead of `self.new_source(player, false, &req).ok()`.

`take_or_open` gains `rate: Option<u32>` (the rate the start will run at; `None`: the bus's):

```rust
    /// Takes the preload if it matches `request` and decodes at `rate`
    /// (the bus's rate when `None`), otherwise opens a new source at it.
    fn take_or_open(
        &mut self,
        player: PlayerId,
        request: &SourceRequest,
        rate: Option<u32>,
    ) -> Result<Playing, AttachError> {
        let rt = self.players.get(&player).ok_or(AttachError::NoPlayer)?;
        let wanted = rate.unwrap_or_else(|| self.rate_of(&rt.main.0));
        let matches = rt.preload.as_ref().is_some_and(|p| {
            p.entry == request.entry && p.start_secs == request.from_secs && p.rate == wanted
        });
        if matches && let Some(p) = self.players.get_mut(&player).and_then(|rt| rt.preload.take()) {
            return Ok(p);
        }
        self.open_source_at(player, false, request, false, Some(wanted))
    }
```

and its caller in `start_current` passes `None` for now (Task 4 passes the cut's rate).

`reopen_waiting`: a waiting source that already decodes at the rate it must have is kept. Replace the two `waiting(...)` takes with

```rust
            let preload_rate = self.preload_rate(player);
            let bus_rate = self.rate_of(bus);
            let Some(rt) = self.players.get_mut(&player) else {
                continue;
            };
            let old_preload = if waiting(&rt.preload)
                && rt.preload.as_ref().map(|p| p.rate) != preload_rate
            {
                rt.preload.take()
            } else {
                None
            };
            let old_current = if waiting(&rt.current)
                && rt.current.as_ref().map(|p| p.rate) != Some(bus_rate)
            {
                rt.current.take()
            } else {
                None
            };
```

and open the fresh preload with `self.open_source_at(player, false, &request, false, preload_rate)`.

`execute`, `Schedule`:

```rust
            EngineAction::Schedule { player, plan } => {
                self.undispatch(player);
                if let Some(rt) = self.players.get_mut(&player) {
                    rt.plan = plan.map_or(Plan::None, Plan::Waiting);
                }
                // BR6.1: the next is prepared at the rate it will play at.
                self.fit_preload(player);
            }
```

`describe_player`: its per-source closure `d` prints `rate {}` (`p.rate`) right after `key{}`, so a source reads `key7 rate 48000 slot1 start Idle …`; keep the rest of the line as it is.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p fp-engine`
Expected: PASS (every existing test: without a cut, every source still decodes at its bus's rate).

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-engine
git commit -m "feat(engine): prepare the next track at the rate of a planned cut

A source remembers the rate it decodes to; the preload of a plan that
carries a rate cut is opened at the new rate while the bus still runs at
the old one, opened again when the plan drops the cut, and never taken
for a start at another rate (bit-perfect rate and fader spec BR6.1).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: The scheduled rate cut (BR6)

**Files:**
- Modify: `crates/fp-engine/src/engine.rs`: `dispatch` (L3130-3251), `current_finished` (L2890-2963), a new `cut_to` after `prepare_start` (~L1383)
- Test: `crates/fp-engine/tests/bit_perfect.rs`

**Interfaces:**
- Consumes: `preload_rate`, `reopen_preload_at`, `fit_preload` (Task 2); `prepare_start` (existing, L1358).
- Produces: `fn cut_to(&mut self, player: PlayerId, rate: u32)`.

- [ ] **Step 1: Write the failing tests**

```rust
/// Renders block by block until the dac runs at `rate`, at most `frames`.
fn run_until_rate(r: &mut Rig, rate: u32, frames: usize) -> bool {
    for _ in 0..frames.div_ceil(BLOCK) {
        if r.rate() == rate {
            return true;
        }
        r.run(BLOCK);
    }
    r.rate() == rate
}

#[test]
fn br6_a_scheduled_cut_reopens_the_device_between_the_tracks() {
    let mut r = rig(true, true);
    on_air_with_next(&mut r);
    r.act(cut_plan(0.5, Some(48_000)));
    r.settle();
    assert!(run_until_rate(&mut r, 48_000, 44_100), "past 0.5 s the device runs at 48 kHz");
    assert_eq!(
        r.engine.unsettled_sources(),
        0,
        "the next was ready at the new rate: nothing to buffer after the reopen"
    );
    r.run(4_800);
    assert!(r.transition_started(2));
    assert!(bit_perfect_after(&mut r, 4_800), "the next track is bit-perfect");
    assert_eq!(r.stream_rates().last(), Some(&(48_000, vec![])));
}

#[test]
fn br6_without_a_cut_the_hard_join_keeps_the_rate() {
    let mut r = rig(true, true);
    let a = wav(r.dir.path(), "a.wav", 44_100);
    let b = wav(r.dir.path(), "b.wav", 44_100);
    r.start(request(1, a, pcm16(44_100)));
    r.act(EngineAction::Preload {
        player: P,
        request: Some(request(2, b, pcm16(44_100))),
    });
    r.act(cut_plan(0.5, None));
    r.settle();
    let attempts = r.dac.open_attempts();
    r.run(44_100);
    assert!(r.transition_started(2));
    assert_eq!(r.rate(), 44_100);
    assert_eq!(r.dac.open_attempts(), attempts, "no reopen");
    assert!(bit_perfect_after(&mut r, 4_410));
}

#[test]
fn br6_a_cart_fired_just_before_the_cut_keeps_the_rate_and_the_next_plays() {
    let mut r = rig(true, true);
    on_air_with_next(&mut r);
    r.act(cut_plan(0.5, Some(48_000)));
    r.settle();
    let jingle = wav(r.dir.path(), "j.wav", 44_100);
    r.act(EngineAction::StartCart(CartRequest {
        cart: CartId(1),
        track: TrackId(9),
        path: jingle,
        from_secs: 0.0,
        until_secs: f64::INFINITY,
        looped: false,
        format: pcm16(44_100),
    }));
    r.settle();
    r.run(44_100);
    assert!(r.transition_started(2), "the next plays");
    assert_eq!(r.rate(), 44_100, "the cart sounds: no reopen");
    assert!(!r.engine.telemetry(P).bit_perfect, "resampled");
}

#[test]
fn br6_a_refused_cut_falls_back_and_the_next_plays() {
    let mut r = rig(true, true);
    r.dac.refuse_rate(48_000);
    on_air_with_next(&mut r);
    r.act(cut_plan(0.5, Some(48_000)));
    r.settle();
    r.run(44_100);
    r.settle();
    r.run(4_410);
    assert!(r.transition_started(2));
    assert_eq!(r.rate(), 44_100, "B3.4: back to the previous rate");
    assert_eq!(r.stream_rates().last(), Some(&(44_100, vec![48_000])));
    let heard = r.run(4_410);
    assert!(heard.iter().any(|v| *v != 0.0), "it plays, resampled");
}
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p fp-engine --test bit_perfect br6_a_scheduled 2>&1 | tail -8`
Expected: FAIL — the rate stays 44 100 (the cut is ignored: the next starts at the running rate).

- [ ] **Step 3: Implement**

`cut_to`, after `prepare_start`:

```rust
    /// BR6 steps 3–4: `player`'s current track has stopped at the cut. Its
    /// Main bus reopens at the preload's rate as B3.2 does, when nothing
    /// else sounds there (the current is gone and the next only waits); if
    /// something sounds or the device refuses (B3.4), the bus keeps its
    /// rate and the next is opened again at it (resampled).
    fn cut_to(&mut self, player: PlayerId, rate: u32) {
        let Some((bus, format)) = self.players.get(&player).map(|rt| {
            let format = rt.preload.as_ref().and_then(|p| p.request.format);
            (rt.main.0.clone(), format)
        }) else {
            return;
        };
        self.prepare_start(&bus, format);
        let stream = self.rate_of(&bus);
        if stream != rate {
            tracing::info!(?player, rate, stream, "rate cut not taken; the next track is resampled");
        }
        self.reopen_preload_at(player, stream);
    }
```

`current_finished`: right after `self.release(finished);` and the `failed` early return, before `let next = match plan { … }`:

```rust
        // BR6: a planned rate cut reopens the device now, between the two
        // tracks; the next then starts as at any end of stream.
        if let Some(TransitionPlan::StartNextAt {
            rate_cut: Some(rate),
            ..
        }) = plan
        {
            self.cut_to(player, rate);
        }
```

`dispatch`: a cut plan stops the current and leaves the preload waiting for `current_finished`:

```rust
    fn dispatch(&mut self, player: PlayerId, plan: TransitionPlan, at_frame: u64, declick: u64) {
        let cut = matches!(
            plan,
            TransitionPlan::StartNextAt {
                rate_cut: Some(_),
                ..
            }
        );
        if !cut {
            // A next prepared for a cut the plan no longer has plays at the
            // stream rate (BR6 fallback).
            self.fit_preload(player);
        }
        // A next that needs a bus carrying DSD may have to wait for the
        // switch to PCM: the whole transition moves with it.
        let next_bus = match plan {
            TransitionPlan::StartNextAt { rate_cut: None, .. } => self
                .players
                .get(&player)
                .and_then(|rt| rt.preload.as_ref())
                .map(|p| p.bus.clone()),
            _ => None,
        };
```

and in the computation of `next`, `TransitionPlan::StartNextAt { .. } => rt.preload.as_mut().map(…)` becomes `TransitionPlan::StartNextAt { rate_cut: None, .. } => …` with `_ => None` for the rest, so a cut plan falls into the final `_` arm (the stop with its declick at `at_frame`). `rt.plan` stays `Plan::Dispatched(plan, at_frame)`, which `current_finished` reads.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p fp-engine`
Expected: PASS.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-engine
git commit -m "feat(engine): reopen a bit-perfect device between two tracks

A scheduled hard transition that carries a rate cut stops the current
track with its declick, reopens the device at the next track's rate when
nothing else sounds there, and starts the next, prepared at that rate;
a cart in the way or a refused rate keeps the rate and the next plays
resampled (bit-perfect rate and fader spec BR6).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: The reopen under a manual hard start, and the quiet guard (BR8, BR13)

**Files:**
- Modify: `crates/fp-engine/src/engine.rs`: `PlayerRuntime` (L299-331) gains `rate_cut`, initialised `None` in `add_player` (~L1894-1911); `bus_sounding` (L1340-1352) delegates to a new `sounding_besides`; new `hard_start_cut` and `run_rate_cuts` after `cut_to`; `start_current` (L2295-2389); `stop_now` (L2514-2534); `pause` (L2417-2451); `player_quiet` (L776-792); `tick` (call `run_rate_cuts` after `end_dsd_streams`, before `run_kept`)
- Test: `crates/fp-engine/tests/bit_perfect.rs`

**Interfaces:**
- Consumes: `take_or_open(…, rate)`, `prepare_start`, `reopen_preload_at` (Tasks 2–3).
- Produces: `PlayerRuntime::rate_cut: Option<u32>`; `fn hard_start_cut(&self, player: PlayerId, bus: &BusKey, format: Option<fp_model::AudioFormat>) -> Option<u32>`; `fn sounding_besides(&self, bus: &BusKey, player: Option<PlayerId>) -> bool`; `fn run_rate_cuts(&mut self)`.

- [ ] **Step 1: Write the failing tests**

```rust
/// P plays a 44.1 kHz track; `b` (48 kHz) is started over it with `action`.
fn started_over(action: fn(SourceRequest) -> EngineAction) -> Rig {
    let mut r = rig(true, true);
    let a = wav(r.dir.path(), "a.wav", 44_100);
    let b = wav(r.dir.path(), "b.wav", 48_000);
    r.start(request(1, a, pcm16(44_100)));
    r.run(4_410);
    r.act(action(request(2, b, pcm16(48_000))));
    r
}

fn hard_start(request: SourceRequest) -> EngineAction {
    EngineAction::StartCurrent { player: P, request }
}

fn crossfade(request: SourceRequest) -> EngineAction {
    EngineAction::Crossfade {
        player: P,
        request,
        fade_ms: 200,
    }
}

#[test]
fn br8_a_hard_start_at_another_rate_reopens_after_the_declick() {
    let mut r = started_over(hard_start);
    assert_eq!(r.rate(), 44_100, "not while the old track's declick sounds");
    r.settle();
    r.run(BLOCK);
    r.settle();
    assert_eq!(r.rate(), 48_000);
    assert!(bit_perfect_after(&mut r, 4_800));
}

#[test]
fn br8_at_the_same_rate_the_hard_start_does_not_reopen() {
    let mut r = rig(true, true);
    let a = wav(r.dir.path(), "a.wav", 44_100);
    let b = wav(r.dir.path(), "b.wav", 44_100);
    r.start(request(1, a, pcm16(44_100)));
    r.run(4_410);
    let attempts = r.dac.open_attempts();
    r.start(request(2, b, pcm16(44_100)));
    r.run(4_410);
    assert_eq!(r.dac.open_attempts(), attempts);
    assert!(bit_perfect_after(&mut r, 4_410));
}

#[test]
fn br8_a_crossfade_at_another_rate_does_not_reopen() {
    let mut r = started_over(crossfade);
    r.settle();
    r.run(9_600);
    assert_eq!(r.rate(), 44_100, "BR11: a sounding bus keeps its rate");
    assert!(!r.engine.telemetry(P).bit_perfect, "resampled");
}

#[test]
fn br8_stop_in_the_gap_starts_nothing() {
    let mut r = started_over(hard_start);
    r.act(EngineAction::StopNow { player: P });
    r.settle();
    let heard = r.run(9_600);
    assert_eq!(r.rate(), 44_100, "no reopen for a track that will not play");
    assert!(heard.iter().skip(BLOCK * 2).all(|v| *v == 0.0), "silence");
}

#[test]
fn br8_next_takes_a_preload_opened_for_a_cut_only_at_its_rate() {
    let mut r = rig(true, true);
    on_air_with_next(&mut r);
    r.act(cut_plan(2.5, Some(48_000)));
    r.settle();
    // The operator presses Next before the planned cut: the same entry.
    let b = r.dir.path().join("b.wav");
    r.act(hard_start(request(2, b, pcm16(48_000))));
    r.settle();
    r.run(BLOCK);
    r.settle();
    assert_eq!(r.rate(), 48_000);
    assert!(bit_perfect_after(&mut r, 4_800), "played at its own rate, at the right speed");
}

#[test]
fn br8_without_a_cut_next_does_not_take_a_preload_at_another_rate() {
    let mut r = rig(true, true);
    on_air_with_next(&mut r);
    r.act(cut_plan(2.5, Some(48_000)));
    r.settle();
    assert_eq!(decode_rate(&r, "pre ["), Some(48_000));
    // A cart sounds on the device: the hard start keeps the rate (D2).
    let jingle = wav(r.dir.path(), "j.wav", 44_100);
    r.act(EngineAction::StartCart(CartRequest {
        cart: CartId(1),
        track: TrackId(9),
        path: jingle,
        from_secs: 0.0,
        until_secs: f64::INFINITY,
        looped: false,
        format: pcm16(44_100),
    }));
    r.settle();
    let b = r.dir.path().join("b.wav");
    r.act(hard_start(request(2, b, pcm16(48_000))));
    assert_eq!(
        decode_rate(&r, "cur ["),
        Some(44_100),
        "opened again at the stream rate, never played at the wrong speed"
    );
    r.settle();
    r.run(4_410);
    assert_eq!(r.rate(), 44_100);
}

#[test]
fn br13_a_kept_device_change_waits_through_the_gap() {
    let mut r = started_over(hard_start);
    let dac = OutputDevice {
        backend: "offline".into(),
        device: "dac".into(),
    };
    let running = r.engine.running_settings(&dac).unwrap();
    r.act(EngineAction::ApplyDevice {
        device: dac.clone(),
        settings: fp_model::DeviceSettings {
            buffer_frames: 960,
            ..running
        },
        force: false,
    });
    r.settle();
    r.run(BLOCK);
    let applied = |r: &Rig| {
        r.events
            .iter()
            .any(|e| matches!(e, EngineEvent::Applied { .. }))
    };
    assert!(!applied(&r), "not in the gap, nor while the new track plays");
    r.act(EngineAction::StopNow { player: P });
    r.run(BLOCK * 4);
    assert!(applied(&r), "once the device is quiet");
}
```

(`OutputDevice` is already imported in this file.)

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p fp-engine --test bit_perfect br8 2>&1 | tail -8`
Expected: FAIL — `br8_a_hard_start_at_another_rate…` keeps 44 100 (`prepare_start` sees the old track sounding), and `br8_next_takes_a_preload…` plays a preload at the wrong rate or resampled.

- [ ] **Step 3: Implement**

`PlayerRuntime` gains, after `notify_fade`:

```rust
    /// BR8: a hard start waits to reopen the Main bus at this rate once the
    /// declick of the track it replaced has ended (`run_rate_cuts`).
    rate_cut: Option<u32>,
```

`bus_sounding` becomes `self.sounding_besides(bus, None)`, and

```rust
    /// As `bus_sounding`, not counting `player`'s current and outgoing
    /// sources (the track a hard start replaces and its declick tail, BR8).
    /// Its own CUE and preload still count.
    fn sounding_besides(&self, bus: &BusKey, player: Option<PlayerId>) -> bool {
        let sounding = |start: StartState| start != StartState::Idle;
        let players = self.players.iter().any(|(id, rt)| {
            let own = Some(*id) == player;
            rt.current
                .iter()
                .chain(rt.outgoing.iter())
                .filter(|_| !own)
                .chain(rt.preload.iter())
                .chain(rt.cue_src.iter())
                .chain(rt.cue_outgoing.iter())
                .any(|p| &p.bus == bus && sounding(p.start))
        });
        players || self.carts_sounding(bus) || self.tones.iter().any(|(b, _)| b == bus)
    }

    /// BR8: the rate `player`'s hard start of a file with `format` reopens
    /// its Main bus at, once the replaced track's declick has ended: a
    /// bit-perfect bus not carrying DSD, a PCM file whose known rate is not
    /// the bus's and was not refused, and nothing sounding there but this
    /// player's current and outgoing sources. `None`: today's hard start.
    fn hard_start_cut(
        &self,
        player: PlayerId,
        bus: &BusKey,
        format: Option<fp_model::AudioFormat>,
    ) -> Option<u32> {
        if !self.device_of(bus).bit_perfect || self.dsd_buses.contains_key(bus) {
            return None;
        }
        let rate = format
            .filter(|f| f.dsd_rate.is_none())
            .map(|f| f.sample_rate)
            .filter(|r| *r > 0)?;
        let running = self.buses.get(bus)?;
        if running.sample_rate() == rate || running.refused_rates().contains(&rate) {
            return None;
        }
        (!self.sounding_besides(bus, Some(player))).then_some(rate)
    }

    /// BR8: hard starts waiting for their bus to change rate. Once the
    /// player's outgoing sources on its Main bus have ended, the bus reopens
    /// as B3.2 does when nothing else sounds, and the new current starts;
    /// if it keeps its rate, the current is opened again at it (resampled).
    /// A cut whose player paused, or whose current is no longer the waiting
    /// source (a stop, a seek, another start), is dropped: a resume follows
    /// B3.2 itself.
    fn run_rate_cuts(&mut self) {
        let waiting: Vec<(PlayerId, u32)> = self
            .players
            .iter()
            .filter_map(|(id, rt)| rt.rate_cut.map(|r| (*id, r)))
            .collect();
        for (player, rate) in waiting {
            let Some(rt) = self.players.get_mut(&player) else {
                continue;
            };
            let idle = rt
                .current
                .as_ref()
                .is_some_and(|c| c.start == StartState::Idle);
            if rt.paused || !idle {
                rt.rate_cut = None;
                continue;
            }
            let bus = rt.main.0.clone();
            if rt.outgoing.iter().any(|p| p.bus == bus) {
                continue;
            }
            rt.rate_cut = None;
            let format = rt.current.as_ref().and_then(|c| c.request.format);
            self.prepare_start(&bus, format);
            let stream = self.rate_of(&bus);
            if stream != rate {
                tracing::info!(?player, rate, stream, "rate change not taken; the track is resampled");
            }
            // `prepare_start` reopened every waiting source not at the new
            // rate; when the bus kept its rate, the current is fitted to it.
            let stale = self
                .players
                .get(&player)
                .and_then(|rt| rt.current.as_ref())
                .is_some_and(|c| c.rate != stream);
            if stale {
                let old = self.players.get_mut(&player).and_then(|rt| rt.current.take());
                if let Some(old) = old {
                    let request = old.request.clone();
                    self.release(old);
                    match self.open_source_at(player, false, &request, false, Some(stream)) {
                        Ok(fresh) => {
                            if let Some(rt) = self.players.get_mut(&player) {
                                rt.current = Some(fresh);
                            }
                        }
                        Err(e) => {
                            tracing::error!(?player, error = ?e, "cannot reopen a source");
                            self.events.push(EngineEvent::ReachedEnd {
                                player,
                                entry: request.entry,
                            });
                            continue;
                        }
                    }
                }
            }
            if let Some(c) = self.players.get_mut(&player).and_then(|rt| rt.current.as_mut()) {
                c.start = StartState::WhenReady { fade_in: false };
            }
        }
    }
```

`start_current`, from `let main = …` to the end of the `opened` match:

```rust
        let main = self.players.get(&player).map(|rt| rt.main.0.clone());
        let direct = match &main {
            Some(main) => self.try_start_dsd(player, main, request),
            None => None,
        };
        // BR8: a hard start at another rate waits for the old track's
        // declick, then the bus reopens (`run_rate_cuts`).
        let cut = match (&main, &direct, crossfade_ms) {
            (Some(main), None, None) => self.hard_start_cut(player, main, request.format),
            _ => None,
        };
        let opened = match direct {
            Some(p) => Ok(p),
            None => {
                if let (Some(main), None) = (&main, cut) {
                    self.prepare_start(main, request.format);
                }
                self.take_or_open(player, request, cut)
            }
        };
```

and, where the new source's start is set:

```rust
        if next.start == StartState::Requested {
            self.send(&next.bus, BusCommand::Cancel { slot: next.slot });
        }
        // A cut keeps it waiting until the bus has reopened.
        next.start = if cut.is_some() {
            StartState::Idle
        } else {
            StartState::WhenReady { fade_in: false }
        };
        let Some(rt) = self.players.get_mut(&player) else {
            return;
        };
        rt.paused = false;
        rt.plan = Plan::None;
        rt.rate_cut = cut;
```

`stop_now` and `pause`: set `rt.rate_cut = None;` where they set `rt.plan` / `rt.paused` (a stop drops the cut; a paused waiting current is resumed by `resume`, which follows B3.2).

`player_quiet`, first lines:

```rust
        // BR13: a player between the stop and the start of a rate cut is
        // not quiet on its Main bus.
        if rt.rate_cut.is_some() && rt.current.as_ref().is_some_and(&on) {
            return false;
        }
```

`tick`: `self.run_rate_cuts();` after `self.end_dsd_streams();` and before `self.run_kept();`.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p fp-engine`
Expected: PASS, including the existing `a_busy_bit_perfect_bus_keeps_its_rate` (BR11), `after_a_stop_the_next_track_follows_its_rate` and `tests/dsd_output.rs` (DSD starts never take a cut).

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-engine
git commit -m "feat(engine): follow the rate under a hard start on a bit-perfect bus

Next or Previous on a bit-perfect player, at another rate, waits for the
old track's declick, reopens the device at the file's rate when nothing
else sounds there and then starts; output changes kept for the device
wait through the gap (bit-perfect rate and fader spec BR8, BR13).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: Settings changes report their rate; documentation (BR11, BR14, §9.2)

**Files:**
- Test: `crates/fp-engine/tests/bit_perfect.rs`
- Modify: `docs/technical/audio-engine.md` — "Rates and bit-perfect buses" (L177 onwards): a paragraph on hard transitions and rate cuts (`TransitionPlan::StartNextAt::rate_cut`, the preload opened at the cut's rate with `Playing::rate`, the dispatch as a stop, the reopen in `current_finished` through `prepare_start`, the fallbacks), the reopen under a hard start (`PlayerRuntime::rate_cut`, `run_rate_cuts`, `hard_start_cut`), and `StreamRate` (`report_stream_rates`, one comparison per tick); the engine action table row `Schedule(plan)` (L400) lists `rate_cut`; "Live settings" (L487 onwards) says a bus whose player is in a rate cut's gap is not quiet (BR13)
- Modify: `docs/technical/audio-path-audit.md` §5.4 "Rate changes on bit-perfect devices" (L403): a closing note "Since the bit-perfect rate and fader spec (2026-10-08), a bit-perfect player's transitions are hard and the device reopens between two tracks at different rates when nothing else sounds (BR6, BR8); B3.3 still holds for a sounding bus (BR11)."; row A21 (L558) adds `tests/bit_perfect.rs` `br6_…`, `br8_…` to its evidence

- [ ] **Step 1: Write the test**

```rust
#[test]
fn br14_bit_perfect_turned_off_reopens_shared_at_the_configured_rate_and_reports_it() {
    let mut r = rig(true, true);
    let a = wav(r.dir.path(), "a.wav", 44_100);
    r.start(request(1, a, pcm16(44_100)));
    r.run(4_410);
    r.act(EngineAction::StopNow { player: P });
    r.run(BLOCK * 4);
    let dac = OutputDevice {
        backend: "offline".into(),
        device: "dac".into(),
    };
    let running = r.engine.running_settings(&dac).unwrap();
    r.act(EngineAction::ApplyDevice {
        device: dac,
        settings: fp_model::DeviceSettings {
            bit_perfect: false,
            ..running
        },
        force: false,
    });
    r.run(BLOCK);
    assert_eq!(r.rate(), 48_000, "effective_rate");
    assert!(!r.dac.config().unwrap().exclusive);
    assert_eq!(r.stream_rates().last(), Some(&(48_000, vec![])));
}
```

Run: `cargo test -p fp-engine --test bit_perfect br14` — expected PASS at once (Task 1's reporting and the existing L12 reopen do it); record in the ledger that it pins BR14's engine half. BR11 is pinned by the existing `a_busy_bit_perfect_bus_keeps_its_rate` and Task 4's `br8_a_crossfade_at_another_rate_does_not_reopen`.

- [ ] **Step 2: The docs** as listed above.

- [ ] **Step 3: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-engine/tests/bit_perfect.rs docs
git commit -m "docs: describe the rate cuts of bit-perfect buses

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

## Self-review notes

- Spec coverage (this plan): BR1 engine half → Task 1; BR6.1 → Task 2; BR6.2–6.4 and its fallbacks → Task 3; BR8, BR13 → Task 4; BR10 (no manual cut for DSD; DSD streams keep O25) → Task 4 `hard_start_cut` and the existing `tests/dsd_output.rs`; BR11 → Tasks 4–5; BR12 (a cut applies nothing: `cut_to` and `run_rate_cuts` only call `prepare_start`, never `apply_device`) → Tasks 3–4; BR14 engine half → Task 5; §9.2 `audio-engine.md`, `audio-path-audit.md` → Task 5.
- `describe_player` gains the rate of each source (Task 2): it is a debug line, and the only way a test sees a preload's decode rate without widening the public API.
- After this plan a bit-perfect player follows each track's rate in continuous play; the fader is still enabled on screen until plan 4 (the model refuses it, plan 1).
