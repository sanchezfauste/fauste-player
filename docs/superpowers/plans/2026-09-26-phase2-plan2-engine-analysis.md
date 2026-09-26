# Phase 2 · Plan 2 — Engine and analysis: the cartwall on air, INTRO tags

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans (or superpowers:subagent-driven-development) to implement this plan task by task.

**Goal:** Play carts for real:
- a cartwall decode worker;
- Main and Cue routes for the cartwall;
- sample-accurate ends and gapless loops;
- cart telemetry for the UI.

Also, read the `INTRO` tag during analysis.

**Architecture:**
- **Carts are sources.** No new real-time code: the mixer already mixes any source.
- **Precise ends and loops in the worker.** A `Load` gains an optional `until_secs` and a `looped` flag. The worker counts the frames it produces:
  - at `until`, the source ends (EOF), so a cart that is not looped finishes exactly at its cue-out;
  - a looped one reopens at `from_secs` and keeps filling the same ring, so the loop point is gapless and sample-accurate;
  - a looped cart whose length is unknown loops at the natural end of the file.
- **The engine keeps separate bookkeeping for the cartwall** (`CartwallRt`): its worker, its routes, and the playing and cue sources keyed by `CartId`.
- **Mixer events map to model events:**
  - `Finished` → `CartEnded` (or `CartCueEnded`);
  - `Failed` → `CartFailed`.

**Spec:** [`docs/superpowers/specs/2026-09-26-phase2-cartwall-settings-design.md`](../specs/2026-09-26-phase2-cartwall-settings-design.md) (P2.4, P2.8 INTRO tag), within the parent spec §4.

## Global Constraints

- Rules carried over: real-time rules (no allocation, lock, log or panic on the mixer path); English; no third-party names; no unsafe; no unwrap, expect or panic outside tests; no hardcoded limits; TDD.
- Engine tests use the `Offline` backend and explicit time. The worker tests use the tagged test opener in `fp-engine/tests/support`.

## Review Focus

1. **Loop boundaries.** A looped cart must produce exactly `n × (until − from)` seconds of frames over n loops: no gap, no repeated or dropped frame at the loop point, and no click beyond what the audio itself contains.
2. **Stopping a cart during its first 500 ms** (still prebuffering). The slot must be released, no `CartEnded` arrives later, and nothing keeps playing.
3. **Many carts at once** (a full page of 16, fired in the same tick). There must be no slot exhaustion (capacity is derived), and all start.
4. **A cart file that disappears between firing and decoding.** `CartFailed` arrives, the model marks it unreadable, and nothing else is affected.
5. **The cartwall Cue route missing** (no route configured). The cue must not play on Main.

---

### Task 1: Worker `until` and `looped`

- `WorkerCommand::Load` gains `until_secs: Option<f64>` and `looped: bool`, and so does `PlayerWorker::load`. The player paths pass `None, false`, so their behaviour is unchanged.
- In `step`, the job tracks `produced_frames`:
  - with `until`, the limit is `(until − from) × bus_rate` frames;
  - frames beyond the limit are cut from the pending block;
  - at the limit, a job that is not looped ends (`eof`), and a looped job reopens the opener at `from_secs` and continues;
  - at the natural EOF, a looped job reopens.
- A zero-length loop (`until ≤ from`) ends at once instead of spinning.
- **Tests** (`fp-engine/tests/worker.rs`, tagged opener):
  - `a_source_with_until_ends_exactly_there`;
  - `a_looped_source_repeats_without_gaps` (the frame values are continuous across the loop point by the tag counter);
  - `a_looped_source_without_until_loops_at_the_end_of_the_file`;
  - `a_zero_length_loop_ends`.

### Task 2: Cartwall routes and configuration

- `OutputsConfig.cartwall: CartwallRoutes { main: Option<Route>, cue: Option<Route> }` (serde default).
- `EngineSettings` carries it.
- Main falls back to the default output. A missing Cue means "no cue".
- **Test:** `cartwall_routes_default_to_the_default_output_and_no_cue` (engine, Offline).

### Task 3: Engine cartwall

- `Engine` gains `cartwall: Option<CartwallRt>`, created lazily on the first cart action, with a `fp-cartwall` worker.
- It handles the actions:
  - `StartCart(request)`: allocate a slot on the Main bus (grow capacity as needed), load it on the worker with `until` and `looped`, and start it when ready. This reuses the existing ready-start path, generalised from players to cart keys.
  - `StopCart`: a de-click ramp, then detach, whether started or not (Review Focus 2).
  - `StartCartCue` / `StopCartCue`: the same on the Cue bus. Without a Cue route there is no cue, and the model gets `CartCueEnded`.
- Mixer events for cart slots:
  - `Finished` → `CartEnded` / `CartCueEnded`;
  - `Failed` → `CartFailed` / `CartCueEnded`.
- **Capacity:** the Main bus capacity includes the carts that are playing, with the existing headroom.
- **Telemetry:** `Telemetry.carts: Vec<(CartId, CartTelemetry { position_secs, peak })>` and `cart_cue: Option<(CartId, f64)>`. A looped cart's position wraps within `[from, until)`.
- **Tests** (`fp-engine/tests/cartwall.rs`, Offline):
  - `a_fired_cart_plays_on_the_cartwall_route_and_ends_with_cart_ended`;
  - `a_looped_cart_keeps_playing_across_the_loop_point`;
  - `stopping_a_cart_before_it_starts_releases_its_slot` (Review Focus 2);
  - `sixteen_carts_fired_together_all_start` (Review Focus 3);
  - `a_missing_cart_file_reports_cart_failed` (Review Focus 4);
  - `the_cart_cue_never_reaches_main` and `without_a_cue_route_the_cart_cue_ends_at_once` (Review Focus 5);
  - `cart_positions_are_reported`.
- **Conductor:** an end-to-end test through the model, `firing_an_exclusive_cart_stops_the_others_on_air`.

- **Known limit (Phase 2 plan 1 review, M3):** a marker change on a cart's track applies from the next fire. The running cart keeps the `until` it started with. This is documented; it is not changed live.

### Task 4: INTRO tag

- `metadata.rs` reads an intro time from:
  - an ID3v2 `TXXX` with description `INTRO`;
  - a Vorbis/FLAC comment `INTRO`;
  - an APE item `INTRO`;
  - an MP4 freeform `----:com.apple.iTunes:INTRO`.

  The value is seconds (`12.5`) or `m:ss(.f)`; anything else is ignored.
- `Analysis` sets `TrackAnalysis.intro_end` from it, clamped to the cue range.
- **Tests:**
  - `intro_times_parse_seconds_and_minutes`;
  - `an_intro_tag_in_a_flac_file_sets_the_intro_end`: write the tag with lofty in a generated file;
  - `an_intro_tag_in_a_wav_file_sets_the_intro_end`: ID3 in WAV, if lofty supports writing it; otherwise log a ruling;
  - `a_malformed_intro_tag_is_ignored`.

### Task 5: Docs and verification

- Update the `docs/technical` audio engine and analysis pages.
- Full checks, a fresh review, fixes, then merge.
