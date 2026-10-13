# Loudness Normalization, Plan 4: the Per-Source Gain in the Engine

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The engine plays every source with the fixed normalization gain its request carries: a static per-slot factor in the bus mixer, multiplied with the fader, sent with `BusCommand::Attach`, kept when a source is reopened, metered after the gain, and exactly 1.0 (bit-exact) when there is no gain.

**Architecture:** `BusCommand::Attach` gains `gain: f32`; the mixer's `Slot` keeps it as a plain field and the per-sample gain becomes `fade × pause × volume_now × gain`. The engine thread computes the value with `Normalization::linear_gain()` (plan 3) in the two places that attach a decoded source (`Engine::open_source` for players and CUE, `Engine::cart_source` for carts); the test tone attaches at 1.0. Every reopen path already clones the source's own request (`Playing::request`, `CartSource::request`), so a seek, a forced rate change, a route move or a reconnect keeps the gain with no new code (N15). No new command, no allocation, no lock on the device callback.

**Tech Stack:** Rust 2024, `fp-engine` (real-time mixer, `rtrb` queues), `fp-model` types; tests with the `Offline` backend, `assert_no_alloc`.

**Spec:** `docs/superpowers/specs/2026-10-08-loudness-normalization-design.md` (§7 N14–N17, §10 row `fp-engine`, §11 `audio-engine.md`, §13 main spec §4.3). Also `docs/technical/audio-path-audit.md` (the real-time rules the mixer is audited against).

## Global Constraints

- All code, identifiers, comments, docs and commit messages in English; never mention other playout or radio-automation products.
- Real-time safety (CLAUDE.md rule 5): the mixer and the render path never allocate, free, lock (only `try_lock` on the bus mixer), log, do I/O or panic. "The value is copied with the rest of `Attach`, which is already sent off the callback" (N14).
- N14: "the per-sample gain becomes `fade × pause × volume_now × gain` … The fader stays clamped to 0–1; the normalization gain is not (a boost is above 1)."
- N15: "a source opened again (a seek, a rate change, a route move …, a device reconnect) is attached with the same gain. Only a new request brings a new gain."
- N16: "Meters measure after gain … The unaltered-source check (bit-perfect spec B1) sees `gain ≠ 1` as any other gain."
- DSD: "`BusCommand::DsdMode`: no gain touches the words" — unchanged (N8); a test pins that the new gain does not either.
- `fp-engine` denies `clippy::indexing_slicing`; `unwrap`, `expect`, `panic` denied outside tests.
- Engine tests use the `Offline` backend and explicit time; never sleep to wait for audio (the existing `settle` helpers poll with a deadline).
- Conventional Commits; every commit ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Gate for every task: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.
- Progress ledger: `.superpowers/sdd/2026-10-08-loudness-plan4-engine-gain/progress.md`.

## Review Focus

1. **A boost on a source that already peaks near full scale** (a quiet master with one loud transient, true peak limited by the model to the ceiling): the mixer must apply it as asked — no hidden clamp of the gain to 1 — and the device conversion still clips only what is above full scale, as before. Test: Task 1, `n14_a_gain_above_unity_is_not_clamped`.
2. **A fader below unity together with a gain above unity** (−6 dB fader, +6 dB gain): the product is unity, but the source is still "altered" for the bit-perfect badge only if the factors are not exactly 1; the badge follows the product per frame (`g == 1.0`), which is the existing rule. Test: Task 1, `n16_the_unaltered_check_sees_the_product_of_the_gains`.
3. **A CUE seek or a seek on a held (paused) source**: the replacement source is attached with the original request's gain, not 1.0. Test: Task 2, `n15_a_cue_seek_keeps_the_gain`.
4. **A preloaded next started by a scheduled transition**: it plays with the gain of its preload request, not the current entry's. Test: Task 2, `n14_a_scheduled_next_plays_with_its_own_gain`.
5. **A non-finite or zero gain reaching the engine** (a model bug): `linear_gain` returns 1.0 for anything unusable (plan 3), and the mixer multiplies whatever it gets without panicking; the engine never computes `powf` on the callback. Test: Task 2, `a_nan_gain_db_plays_at_unity`.

---

## File Structure

| File | Responsibility |
|---|---|
| `crates/fp-engine/src/mixer.rs` | `BusCommand::Attach { gain }`, `Slot::gain`, the per-sample product |
| `crates/fp-engine/src/engine.rs` (`open_source` ~L1281-1350, `play_test_tone` ~L508) | The gain from the request; 1.0 for the test tone |
| `crates/fp-engine/src/engine/carts.rs` (`cart_source` ~L127-160) | The gain from the cart request |
| `crates/fp-engine/tests/mixer.rs`, `tests/dsd_mixer.rs`, `tests/audio_path_audit.rs`, `tests/bus.rs` | Mixer tests; `Attach` literals gain `gain: 1.0` |
| `crates/fp-engine/tests/engine.rs`, `tests/cartwall.rs`, `tests/bit_perfect.rs` | Engine tests |
| `docs/technical/audio-engine.md`, `docs/technical/audio-path-audit.md`, main spec §4.3 | Task 3 |

---

### Task 1: The mixer applies a per-slot gain (N14, N16)

**Files:**
- Modify: `crates/fp-engine/src/mixer.rs` (`BusCommand::Attach` L29-34 and its doc, `Slot` L282-312, `apply` Attach arm L445-487, `render_slot` L983-990)
- Modify (compile fixes, `gain: 1.0`): every `BusCommand::Attach { … }` in `crates/fp-engine/tests/{bus,mixer,dsd_mixer,audio_path_audit}.rs`, and in `src/engine.rs:508` (test tone), `src/engine.rs:1315`, `src/engine/carts.rs:154` (temporarily `gain: 1.0`; Task 2 wires the request)
- Test: `crates/fp-engine/tests/mixer.rs`, `crates/fp-engine/tests/dsd_mixer.rs`, `crates/fp-engine/tests/audio_path_audit.rs`

**Interfaces:**
- Produces: `BusCommand::Attach { slot: usize, source: SourceConsumer, volume: Arc<AtomicF32>, first_channel: u16, gain: f32 }`.

- [ ] **Step 1: Write the failing tests**

`crates/fp-engine/tests/mixer.rs` — add a helper after `attach` and the tests:

```rust
/// As `attach`, with a normalization gain (loudness normalization N14).
fn attach_with_gain(
    h: &mut MixerHandle,
    slot: usize,
    source: fp_engine::source::SourceConsumer,
    volume: Arc<AtomicF32>,
    gain: f32,
) {
    assert!(
        h.commands
            .push(BusCommand::Attach {
                slot,
                source,
                volume,
                first_channel: 0,
                gain,
            })
            .is_ok()
    );
}

#[test]
fn n14_a_slot_plays_at_its_gain_and_is_metered_after_it() {
    let (mut m, mut h) = mixer(2);
    let (_p, c) = counting_source(16, false);
    let shared = c.shared.clone();
    attach_with_gain(&mut h, 0, c, full_volume(), 2.0);
    start(&mut h, 0);
    let out = render(&mut m, 4, 2);
    assert_eq!(left(&out), vec![2.0, 4.0, 6.0, 8.0]);
    assert_eq!(out[1], -2.0);
    assert_eq!(shared.peak_l.take(), 8.0, "the meter reads the gained level");
}

#[test]
fn n14_a_gain_above_unity_is_not_clamped() {
    let (mut m, mut h) = mixer(2);
    let (_p, c) = counting_source(16, false);
    attach_with_gain(&mut h, 0, c, full_volume(), 4.0);
    start(&mut h, 0);
    assert_eq!(left(&render(&mut m, 2, 2)), vec![4.0, 8.0]);
}

#[test]
fn n14_a_fader_move_keeps_the_gain() {
    let (mut m, mut h) = mixer(2);
    let (_p, c) = counting_source(16, false);
    let volume = full_volume();
    attach_with_gain(&mut h, 0, c, volume.clone(), 2.0);
    start(&mut h, 0);
    render(&mut m, 2, 2);
    volume.store(0.5);
    // `volume_smoothing_frames` is 1 in `CONFIG`: the next frame is there.
    let out = render(&mut m, 2, 2);
    assert_eq!(left(&out), vec![3.0, 4.0], "frames 3 and 4 at 0.5 × 2");
}

#[test]
fn n16_a_gain_other_than_one_alters_the_source() {
    let (mut m, mut h) = mixer(2);
    let (_p, c) = counting_source(64, false);
    let shared = c.shared.clone();
    attach_with_gain(&mut h, 0, c, full_volume(), 1.000_1);
    start(&mut h, 0);
    render(&mut m, 4, 2);
    assert!(!shared.unaltered.load(Ordering::Acquire));
}

#[test]
fn n16_the_unaltered_check_sees_the_product_of_the_gains() {
    let (mut m, mut h) = mixer(2);
    let (_p, c) = counting_source(64, false);
    let shared = c.shared.clone();
    attach_with_gain(&mut h, 0, c, Arc::new(AtomicF32::new(0.5)), 2.0);
    start(&mut h, 0);
    render(&mut m, 4, 2);
    assert!(shared.unaltered.load(Ordering::Acquire), "0.5 × 2 is exactly 1");
}
```

`crates/fp-engine/tests/dsd_mixer.rs`:

```rust
#[test]
fn in_dsd_mode_the_normalization_gain_does_not_touch_the_words() {
    let (mut m, mut h) = Mixer::new(4, CONFIG);
    let (_p, c, v) = dsd_source(8, 1.0);
    send(
        &mut h,
        BusCommand::Attach {
            slot: 0,
            source: c,
            volume: v,
            first_channel: 0,
            gain: 2.0,
        },
    );
    send(&mut h, BusCommand::DsdMode { on: true, at_frame: 0 });
    send(&mut h, BusCommand::Start { slot: 0, at_frame: 0 });
    let out = render(&mut m, 4);
    assert_eq!(out[0], word_to_sample(0, 1));
    assert_eq!(out[1], -word_to_sample(0, 1));
}
```

`crates/fp-engine/tests/audio_path_audit.rs`, after `no_gain_in_the_mixer_exceeds_unity` (whose name now reads "no gain but the normalization gain"; rename it to `no_fader_gain_exceeds_unity` and keep its body with `gain: 1.0`):

```rust
#[test]
fn the_normalization_gain_is_applied_without_allocating() {
    let (mut m, mut h) = meter_mixer(1);
    assert!(
        h.commands
            .push(BusCommand::Attach {
                slot: 0,
                source: dc_source(0.25, 4_096),
                volume: Arc::new(AtomicF32::new(1.0)),
                first_channel: 0,
                gain: 2.0,
            })
            .is_ok()
    );
    assert!(h.commands.push(BusCommand::Start { slot: 0, at_frame: 0 }).is_ok());
    let mut out = vec![0.0f32; 512 * 2];
    assert_no_alloc::assert_no_alloc(|| m.render(&mut out, 2));
    assert!((out[100] - 0.5).abs() < 1e-6, "the only gain above unity is the one asked for");
}
```

(If `audio_path_audit.rs` does not install the `AllocDisabler` global allocator, copy the `#[cfg(debug_assertions)] #[global_allocator]` lines from `tests/mixer.rs` — check first; it likely does, since the file is the audit.)

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p fp-engine --test mixer n14 2>&1 | tail -5`
Expected: FAIL to compile ("struct variant `BusCommand::Attach` has no field named `gain`").

- [ ] **Step 3: Implement**

`mixer.rs`:

```rust
    /// Puts a source in `slot`, silent until `Start`. `gain` is the
    /// source's fixed normalization gain (linear; loudness normalization
    /// spec N14), multiplied with the fader for its whole life; 1.0 leaves
    /// it bit-exact.
    Attach {
        slot: usize,
        source: SourceConsumer,
        volume: Arc<AtomicF32>,
        first_channel: u16,
        gain: f32,
    },
```

`Slot` gains, after `volume_now`:

```rust
    /// The fixed normalization gain (N14); not clamped (a boost is above 1).
    gain: f32,
```

The `Attach` arm destructures `gain` and stores it in the new `Slot` (`gain,`). In `render_slot`:

```rust
                let g = slot.fade.next_gain() * slot.pause.next_gain() * slot.volume_now * slot.gain;
```

(The words branch above it is untouched: DSD words get no gain.) Update the module doc's first paragraph if it lists the gains ("applying gain ramps at exact frames" stays true).

Add `gain: 1.0,` to every other `Attach` literal (the list under **Files**).

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p fp-engine`
Expected: PASS.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-engine
git commit -m "feat(engine): give each mixer slot a fixed gain

A source's normalization gain arrives with its Attach and multiplies the
fader for its whole life, with no new command on the real-time thread
(loudness normalization spec N14, N16).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: The engine attaches each source with its request's gain (N14, N15, N17)

**Files:**
- Modify: `crates/fp-engine/src/engine.rs` (`open_source`: the `Attach` at ~L1315), `crates/fp-engine/src/engine/carts.rs` (`cart_source`: the `Attach` at ~L154)
- Test: `crates/fp-engine/tests/engine.rs`, `crates/fp-engine/tests/cartwall.rs`, `crates/fp-engine/tests/bit_perfect.rs`

**Interfaces:**
- Consumes: `SourceRequest::normalization`, `CartRequest::normalization`, `Normalization::linear_gain()` (plan 3); `BusCommand::Attach { gain }` (Task 1).

- [ ] **Step 1: Write the failing tests**

`crates/fp-engine/tests/engine.rs` (add `Normalization, NormalizationStatus` to the `fp_model` imports):

```rust
/// `request` with a normalization gain of `gain_db`.
fn gained(track: u64, from_secs: f64, gain_db: f32) -> SourceRequest {
    SourceRequest {
        normalization: Normalization {
            gain_db,
            status: NormalizationStatus::Applied { limited: None },
        },
        ..request(track, from_secs)
    }
}

/// +6.02 dB is a factor of 2 (to 1e-3).
const DOUBLE_DB: f32 = 6.0206;

/// `heard` divided by 2 is the tagged sample `(track, index)`.
fn halved(v: f32) -> (u64, u64) {
    tag((v / 2.0).round())
}

#[test]
fn n14_a_source_plays_with_its_normalization_gain() {
    let mut r = rig(96_000, false);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: gained(1, 0.0, DOUBLE_DB),
    });
    r.settle();
    r.run(2);
    let first = r.heard.iter().position(|v| *v != 0.0).unwrap();
    assert_eq!(halved(r.heard[first + 100]), (1, 100));
}

#[test]
fn a_nan_gain_db_plays_at_unity() {
    let mut r = rig(96_000, false);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: gained(1, 0.0, f32::NAN),
    });
    r.settle();
    r.run(2);
    let first = r.heard.iter().position(|v| *v != 0.0).unwrap();
    assert_eq!(tag(r.heard[first + 100]), (1, 100));
}

#[test]
fn n15_a_seek_keeps_the_gain() {
    let mut r = rig(96_000, false);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: gained(1, 0.0, DOUBLE_DB),
    });
    r.settle();
    r.run(2);
    r.act(EngineAction::Seek { player: P, secs: 1.5 });
    r.settle();
    r.run(4);
    let (track, index) = halved(*r.heard.last().unwrap());
    assert_eq!(track, 1);
    assert!(index as f64 / RATE >= 1.5, "{index}");
}

#[test]
fn n14_a_scheduled_next_plays_with_its_own_gain() {
    let mut r = rig(96_000, false);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, 0.0),
    });
    r.act(EngineAction::Preload {
        player: P,
        request: Some(gained(2, 0.0, DOUBLE_DB)),
    });
    r.act(EngineAction::Schedule {
        player: P,
        plan: Some(TransitionPlan::StartNextAt {
            at_secs: 0.05,
            fade_current_until_secs: None,
        }),
    });
    r.settle();
    r.run(20);
    let v = *r.heard.last().unwrap();
    assert_eq!(halved(v).0, 2, "track 2 at twice its level: {v}");
}

#[test]
fn n15_a_cue_seek_keeps_the_gain() {
    let mut r = rig(480_000, true);
    r.act(EngineAction::StartCue {
        player: P,
        request: gained(3, 0.0, DOUBLE_DB),
    });
    r.settle();
    r.run_hearing_cue(4);
    r.act(EngineAction::SeekCue { player: P, secs: 2.0 });
    r.settle();
    let heard = r.run_hearing_cue(6);
    let (track, index) = halved(*heard.last().unwrap());
    assert_eq!(track, 3);
    assert!(index as f64 / RATE >= 2.0, "{index}");
}
```

`crates/fp-engine/tests/cartwall.rs` (import `Normalization, NormalizationStatus`):

```rust
#[test]
fn n14_a_cart_plays_with_its_normalization_gain() {
    let mut r = rig(2, Some(route(0)), None, 48_000);
    let request = CartRequest {
        normalization: Normalization {
            gain_db: -6.0206,
            status: NormalizationStatus::Applied { limited: None },
        },
        ..cart(1, 0.0, 2_400.0 / RATE, false)
    };
    r.act(EngineAction::StartCart(request));
    r.settle();
    r.run(10);
    let left = r.channel(0);
    let first = left.iter().position(|v| *v != 0.0).unwrap();
    // Track 1, sample 100 is 100 100; at −6.02 dB it is half of that.
    assert!((left[first + 100] - 50_050.0).abs() < 50.0, "{}", left[first + 100]);
}
```

`crates/fp-engine/tests/bit_perfect.rs` (import `Normalization, NormalizationStatus`):

```rust
#[test]
fn n16_a_normalization_gain_is_not_bit_perfect() {
    let mut r = rig(true, true);
    let path = wav(r.dir.path(), "a.wav", 48_000);
    let mut req = request(1, path, pcm16(48_000));
    req.normalization = Normalization {
        gain_db: -3.0,
        status: NormalizationStatus::Applied { limited: None },
    };
    r.start(req);
    assert!(!bit_perfect_after(&mut r, 2_400));
}

#[test]
fn n16_no_gain_stays_bit_perfect() {
    // `request` carries `Normalization::default()`: 0 dB, a factor of 1.0.
    let mut r = rig(true, true);
    let path = wav(r.dir.path(), "a.wav", 48_000);
    r.start(request(1, path, pcm16(48_000)));
    assert!(bit_perfect_after(&mut r, 2_400));
}
```

(`bit_perfect_playback_is_bit_exact` already pins that a source with no gain reaches the device bit for bit; it must keep passing.)

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p fp-engine --test engine n14 2>&1 | tail -15`
Expected: FAIL — `n14_a_source_plays_with_its_normalization_gain` asserts `(1, 50)`-like halved tags (the source plays at unity, so halving gives the wrong index).

- [ ] **Step 3: Implement**

`engine.rs`, `open_source`: compute the gain before taking the bus borrow and attach with it:

```rust
        // The source's fixed normalization gain (loudness normalization
        // N14), computed here, off the device callback. Every reopen
        // passes the same request, so it keeps the gain (N15).
        let gain = request.normalization.linear_gain();
```

and `gain,` in its `BusCommand::Attach { … }`. `carts.rs`, `cart_source`: the same with `request.normalization.linear_gain()`. The test tone keeps `gain: 1.0` (spec §12: the test tone is not normalized).

Check that no other code path builds an `Attach` for a decoded source: `grep -n "BusCommand::Attach" crates/fp-engine/src` must list only `open_source`, `cart_source` and `play_test_tone`.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test -p fp-engine`
Expected: PASS.

- [ ] **Step 5: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/fp-engine
git commit -m "feat(engine): play each source at its normalization gain

Players, CUE and carts attach their source with the gain the model
decided; a seek or any other reopen keeps it, and no gain keeps a
bit-perfect source bit-exact (loudness normalization spec N14-N17).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: Documentation

**Files:**
- Modify: `docs/technical/audio-engine.md` (the mixer section: the per-slot gain `fade × pause × volume × gain`, where it comes from, kept on reopen; the meters section: "after its gain" now includes the normalization gain; the bit-perfect section: a gain ≠ 1 alters the source like the fader)
- Modify: `docs/technical/audio-path-audit.md` (the mixer row lists the gain field: copied with `Attach`, no allocation; the statement that no mixer gain exceeds unity becomes "no fader or ramp gain exceeds unity; the normalization gain may, by design (loudness normalization spec N14)", citing the renamed test `no_fader_gain_exceeds_unity` and `the_normalization_gain_is_applied_without_allocating`)
- Modify: `docs/superpowers/specs/2026-09-25-fauste-player-design.md` §4.3, after the volume bullet: "Each source also has a fixed normalization gain from its request (loudness normalization spec N14), multiplied with the volume; it never changes while the source exists."

- [ ] **Step 1: Edit the three documents** as listed.
- [ ] **Step 2: Gate and commit**

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add docs
git commit -m "docs: describe the per-source normalization gain

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

## Self-review notes

- Spec coverage: N14 → Tasks 1–2; N15 (seek, CUE seek; rate change, route move and reconnect share the cloned-request path, pinned by the seek tests and by `grep` in Task 2 Step 3) → Task 2; N16 → Tasks 1–2; N17 (a paused source keeps its gain when its device becomes bit-perfect) is the model's `on_air` record (plan 3, `n12_a_placement_on_a_bit_perfect_device…`) plus the engine never touching a slot's gain after `Attach` (Task 1); §11 audio-engine.md, §13 main spec §4.3 → Task 3.
- A forced rate change (`follow_forced_rate`) reopens with `old.request.clone()` (engine.rs ~L742-752) and carts with `source.request.clone()` (carts.rs ~L220): no code change; Task 2's grep confirms nothing else attaches.
