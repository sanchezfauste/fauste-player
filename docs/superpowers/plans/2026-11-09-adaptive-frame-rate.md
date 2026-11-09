# Adaptive frame rate Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** While something moves, the interface keeps its drawing within a share of one core, lowering the frame rate by itself on slow (software-rendered) computers and leaving it at `ui.max_fps` everywhere else.

**Architecture:** The vendored eframe reports the swap time and the GL renderer; `fp-app::ui::view` holds the pure rules (cost, average, target, hysteresis) and an `AdaptiveRate` state; `Shell` feeds it each frame's cost and `AppUi` uses its period for every animation frame.

**Tech Stack:** Rust, eframe/egui 0.37 (glow), egui_kittest.

**Spec:** `docs/superpowers/specs/2026-11-09-adaptive-frame-rate-design.md`

## Global Constraints

- `ui.frame_budget_percent`: default 40, range 5–100. `ui.min_fps`: default 8, range 1–60. Config-file only, lenient (clamped with a `ConfigWarning`).
- Average weight 0.1 per frame; climb at most 5 fps per second; fall at once.
- The swap time counts only with a software renderer (AF1, AF2 name list).
- Never a log line per frame (AF7).
- No `unsafe` outside `vendor/eframe`; no `unwrap`/`expect`/`panic` outside tests.

## Review Focus

- A GPU computer with vsync: the swap blocks ~16 ms per frame. Expected: rate stays at `max_fps` (swap ignored when not software). Test in Task 3 (`a_gpu_swap_never_counts`).
- `max_fps` 0 (display rate) and a cheap frame: expected unlimited (period zero). Test in Task 3 (`an_unlimited_ceiling_stays_unlimited`).
- `min_fps` above `max_fps` (e.g. 20 and 10): expected the rate never exceeds `max_fps`. Test in Task 3 (`the_floor_never_exceeds_the_ceiling`).
- No frame info (tests, first frame, `cpu_usage` `None`): expected the ceiling. Test in Task 3 (`no_measurement_keeps_the_ceiling`).
- A cost of zero or NaN: expected no division blow-up, the ceiling. Test in Task 3 (`a_degenerate_cost_keeps_the_ceiling`).

---

### Task 1: eframe reports the swap time and the GL renderer

**Files:**
- Modify: `vendor/eframe/src/epi.rs` (`IntegrationInfo`, new `GlRenderer`)
- Modify: `vendor/eframe/src/native/glow_integration.rs` (swap timing; renderer after `egui_glow::Painter::new`)
- Modify: `vendor/eframe/src/native/epi_integration.rs:226`, `vendor/eframe/src/web/app_runner.rs:87` (new fields `None`)
- Modify: `vendor/eframe/VENDORED.md`, `CLAUDE.md` (vendor row)

**Interfaces:**
- Produces: `eframe::IntegrationInfo { swap_usage: Option<f32>, gl_renderer: Option<eframe::GlRenderer>, .. }`; `pub struct GlRenderer { pub renderer: String, pub vendor: String, pub version: String }` (Clone, Debug).

- [ ] **Step 1:** Add the fields and the struct; set `gl_renderer` once from `gl.get_parameter_string(glow::RENDERER | VENDOR | VERSION)` when the painter is created (before the app creator runs, so `CreationContext::integration_info` carries it); time `swap_buffers` with an `Instant` and store it in `integration.frame.info.swap_usage` next to `report_frame_time`.
- [ ] **Step 2:** `cargo build -p fp-app` succeeds; `VENDORED.md` lists both changes under "Local changes".
- [ ] **Step 3:** Commit `build(deps): report the swap time and the GL renderer from the vendored eframe`.

### Task 2: configuration fields

**Files:**
- Modify: `crates/fp-model/src/config.rs` (`UiConfig`, `Default`, `Config::validate`, inline tests next to the `max_fps` test)

**Interfaces:**
- Produces: `UiConfig::frame_budget_percent: u32`, `UiConfig::min_fps: u32`.

- [ ] **Step 1: Failing test** `frame_rate_fields_default_and_clamp`: defaults 40 and 8; `frame_budget_percent` 0 → 5 and 500 → 100 with a warning on `ui.frame_budget_percent`; `min_fps` 0 → 1 and 1000 → 60 with a warning on `ui.min_fps`; a config JSON without the fields loads with the defaults.
- [ ] **Step 2:** Run `cargo test -p fp-model frame_rate_fields` → FAIL (no field).
- [ ] **Step 3:** Add the fields with doc comments (what, default, range, config file only) and two `clamp_to` lines.
- [ ] **Step 4:** Test passes.
- [ ] **Step 5:** Commit `feat(model): frame budget and minimum frame rate settings`.

### Task 3: the rules

**Files:**
- Modify: `crates/fp-app/src/ui/view.rs`
- Create: `crates/fp-app/tests/it/frame_rate.rs` (+ `mod frame_rate;` in `tests/it/main.rs`)

**Interfaces:**
- Consumes: Task 2 fields.
- Produces (all `pub` in `fp_app::ui::view`):
  - `fn is_software_renderer(renderer: &str) -> bool`
  - `fn frame_cost(cpu: Option<f32>, swap: Option<f32>, software: bool) -> Option<f32>`
  - `fn target_fps(cost_secs: f32, ui: &UiConfig) -> f32` (`f32::INFINITY` = unlimited)
  - `struct AdaptiveRate` (Default) with `fn observe(&mut self, cost: Option<f32>, moving: bool, now: f64, ui: &UiConfig) -> Option<RateNote>`, `fn fps(&self, ui: &UiConfig) -> f32`, `fn period(&self, ui: &UiConfig) -> Duration`
  - `enum RateNote { Limited { fps: f32, cost_ms: f32, budget: u32 }, Changed { .. same }, Restored { fps: f32 } }`

- [ ] **Step 1: Failing tests** (one per rule, spec values):
  - `software_renderers_are_recognised`: `"llvmpipe (LLVM 19.1.7, 128 bits)"`, `"softpipe"`, `"Google SwiftShader"`, `"Apple Software Renderer"`, `"GDI Generic"` → true; `"Mesa Intel(R) HD Graphics 620"`, `"NVIDIA GeForce GTX 1060/PCIe/SSE2"` → false; case-insensitive.
  - `a_gpu_swap_never_counts`: `frame_cost(Some(0.003), Some(0.016), false) == Some(0.003)`; with `true` → `Some(0.019)`.
  - `the_target_follows_the_budget`: defaults, cost 0.030 → `40/100/0.030 ≈ 13.3`; cost 0.002 → 30 (ceiling).
  - `the_floor_never_exceeds_the_ceiling`: `min_fps` 20, `max_fps` 10, cost 1.0 → 10.
  - `an_unlimited_ceiling_stays_unlimited`: `max_fps` 0, cost 0.002 → `period` zero after observing.
  - `one_slow_frame_does_not_drop_the_rate`: many 0.002 frames, then one 0.100 → fps still 30.
  - `a_sustained_cost_drops_the_rate_and_it_climbs_back_slowly`: 60 frames of 0.030 at 30 fps (`now` += 1/30) → fps ≤ 14 and a `Limited` note was returned once; then cheap frames: after 1 s fps ≤ previous + 5; eventually 30 and a `Restored` note.
  - `only_moving_frames_count`: costs observed with `moving == false` leave the rate at the ceiling.
  - `no_measurement_keeps_the_ceiling` and `a_degenerate_cost_keeps_the_ceiling` (0.0, NaN).
- [ ] **Step 2:** `cargo test -p fp-app --test it frame_rate::` → FAIL (not defined).
- [ ] **Step 3:** Implement. EMA weight 0.1 (first sample seeds it); fall at once to a lower target; climb by ≤ `5.0 * dt` (dt from the previous observed `now`, capped at 1 s); notes per AF7 (Changed when |fps − last logged| ≥ 5).
- [ ] **Step 4:** Tests pass.
- [ ] **Step 5:** Commit `feat(ui): adaptive frame rate rules`.

### Task 4: wiring and log lines

**Files:**
- Modify: `crates/fp-app/src/ui/shell.rs` (eframe `ui` passes `frame.info()` costs), `crates/fp-app/src/ui/app.rs` (`AppUi` field `rate: AdaptiveRate`, `software_renderer: bool`, `last_moving: bool`; `Scene::frame_period`; `next_frame` call), `crates/fp-app/src/ui/view.rs` (`next_frame` takes `moving_period: Duration` instead of reading `max_fps`), `crates/fp-app/src/ui/cartwall.rs:522` (uses `scene.frame_period`), `crates/fp-app/src/main.rs` (renderer log, `with_software_renderer`)
- Test: `crates/fp-app/tests/it/frame_rate.rs`, `crates/fp-app/tests/it/touch_ui.rs` (calls of `next_frame`)

**Interfaces:**
- Consumes: Tasks 1 and 3.
- Produces: `AppUi::with_software_renderer(self, bool) -> Self`, `AppUi::observe_frame(&mut self, cpu: Option<f32>, swap: Option<f32>)`, `AppUi::frame_period(&self) -> Duration`.

- [ ] **Step 1: Failing test** `a_slow_software_renderer_lowers_the_animation_rate` (kittest, one player playing, `with_software_renderer(true)`): run frames calling `observe_frame(Some(0.006), Some(0.024))` before each step → `frame_period()` ≥ 1/14 s; same with `with_software_renderer(false)` → 1/30 s.
- [ ] **Step 2:** FAIL (no methods).
- [ ] **Step 3:** Implement; log at info: at start `renderer`, `vendor`, `version`, `software` (or "no GL renderer reported"); `RateNote`s as one line each (`frame rate limited` / `frame rate changed` / `frame rate back to the maximum`) with fps, cost in ms and budget.
- [ ] **Step 4:** `cargo test -p fp-app --test it` passes.
- [ ] **Step 5:** Commit `feat(ui): lower the frame rate when frames cost more than the budget`.

### Task 5: docs, measurement, review

**Files:** `docs/superpowers/specs/2026-09-25-fauste-player-design.md` (§8.1 repaint policy points to AF), `docs/technical/ui.md`, `docs/user/troubleshooting.md`.

- [ ] **Step 1:** Update the docs.
- [ ] **Step 2:** Measure with the release build on Xvfb (llvmpipe), `taskset -c 0,1`, one player playing: the log shows `software=true` and a limited rate; process CPU well below the 147% before.
- [ ] **Step 3:** fmt, clippy, whole suite; fresh reviewer on the branch; fix Critical/Important test-first.
- [ ] **Step 4:** PR, CI on three OSes, merge.
