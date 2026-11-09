# Adaptive frame rate — design

Status: approved 2026-11-09. Extends the repaint policy of the main design
spec (§8.1) and the phone layout spec (PL31, PL32).

## 1. Why

At a theatre show the application ran on a Core 2 Duo with 4 GB (Debian 13,
KDE). With a track playing, the interface lagged badly. Reproduced on
Xvfb (Mesa llvmpipe, software rendering) limited to two cores, with one
player playing at the default `ui.max_fps` (30): the process took 147% of
a core, most of it in the llvmpipe threads. On a two-core computer that
leaves little for decoding and mixing, and the interface itself cannot
keep up.

The idle part was fixed separately (one frame a second when nothing moves,
PR #226). This spec covers the frames drawn while something moves.

## 2. Goals, non-goals, anti-goals

Goals:

- While something moves, the interface spends at most a set share of one
  core on drawing. When a frame costs more, the frame rate drops by
  itself, down to a floor, and climbs back when frames get cheaper.
- The log says what draws the interface (GPU or software) at start, and
  when the frame rate is limited and why.

Non-goals:

- Drawing only the parts of the window that changed (partial redraw). It
  may come later as its own spec.
- A Settings control. The new fields are config-file only, like
  `ui.max_fps`.

Anti-goals (a failure even if it works):

- A computer with a GPU drawing fewer frames than today. On a normal
  computer a frame costs a few milliseconds and the rate must stay at
  `ui.max_fps`, whatever the core count.
- Input answered later. Clicks, keys and the pointer still draw at once.
- A rate that oscillates, or one slow frame (a menu opening, a large
  playlist laid out once) that drops the rate.

## 3. Rules

- AF1. **Frame cost.** A frame's cost is the CPU time eframe reports for
  it (`IntegrationInfo::cpu_usage`: the interface, tessellation and the
  GL calls, without waiting for vsync), plus, when the renderer draws in
  software, the time `swap_buffers` took (`IntegrationInfo::swap_usage`,
  §4). Measured with llvmpipe: `cpu_usage` 6.3 ms, `swap_buffers` 24 ms
  per frame (the rasterizing happens in the swap). With a GPU the swap
  only waits (for vsync or the GPU), which costs no CPU, so it is left
  out: a GPU computer cannot look slower than it is.
- AF2. **Software renderer.** Read once at start from `GL_RENDERER`,
  `GL_VENDOR` and `GL_VERSION`, which eframe reports
  (`IntegrationInfo::gl_renderer`, §4; the GL calls are `unsafe`, so they
  stay in eframe). A renderer whose name
  contains a known software rasterizer (`llvmpipe`, `softpipe`,
  `swrast`, `SwiftShader`, `Software Rasterizer`, `Apple Software
  Renderer`, `GDI Generic`, `Microsoft Basic Render`; case-insensitive)
  is software. The log line at start names the renderer, the vendor, the
  GL version and whether it is software. With no GL context (tests,
  another renderer), nothing is known and the swap is never counted.
- AF3. **Smoothing.** The cost is an exponential moving average over the
  frames drawn while something moves (weight 0.1 for each new frame), so
  one slow frame does not change the rate; a frame counts for at most
  three times the average. A frame counts when the frame before it asked
  for the animation rate; idle and blink frames do not, nor do the first
  10 frames after the interface stood still (the first layout of a track
  that starts, its waveform, its cover).
- AF4. **Target rate.** `target = ui.frame_budget_percent / 100 / cost`
  frames a second, then held between `ui.min_fps` and the ceiling. The
  ceiling is `ui.max_fps`; with `max_fps` 0 (the display's rate) there
  is no ceiling, and the rate is limited only when the target falls below
  60 frames a second (the common display rate); above it, every display
  frame is drawn as before.
- AF5. **Hysteresis.** The rate in use falls at once to a lower target,
  and climbs towards a higher one by at most 5 frames a second per
  second. So a sustained high cost limits the rate within a few frames,
  and a short cheap stretch does not bring it back to the top at once.
- AF6. **What it applies to.** Every frame due to something moving
  (`view::next_frame`'s animation period, the cartwall's sweep) uses the
  rate in use. The blink of a paused player (4 frames a second) and the
  idle frame (once a second) are below any sensible floor and do not
  change.
- AF7. **Log.** One info line when the rate becomes limited (below the
  ceiling), with the rate, the smoothed cost and the budget; one when it
  is back at the ceiling; and, while limited, one when it moves by 5
  frames a second or more from the last one logged. Never a line per
  frame.
- AF8. **Configuration** (`UiConfig`, config-file only, lenient):
  `frame_budget_percent` (default 40, 5–100): the share of one core the
  interface may spend on frames while something moves. `min_fps`
  (default 8, 1–60): the lowest rate the adaptation goes down to; it
  never goes above `max_fps` when that is set.

## 4. The eframe change

`vendor/eframe` (already a patched copy) gets one more local change, in
`IntegrationInfo`:

- `swap_usage: Option<f32>`: the seconds `swap_buffers` took on the
  previous frame (glow integration; `None` elsewhere and on the first
  frame), recorded where eframe already pauses its frame timer around the
  swap;
- `gl_renderer: Option<GlRenderer>` (`renderer`, `vendor`, `version`
  strings): read once when the glow painter is created; `None` elsewhere.

`VENDORED.md` lists both.

## 5. Where it lives

- `fp-model::UiConfig`: the two fields, their defaults and ranges.
- `fp-app::ui::view`: the pure rules (AF1 cost, AF3 average, AF4–AF5
  rate) with one test per rule; `next_frame` takes the rate in use.
- `fp-app::ui::shell`: takes `cpu_usage` and `swap_usage` from eframe's
  frame and hands them to `AppUi`.
- `fp-mobile` does not hand the frame costs over yet: on Android the
  rate stays at `ui.max_fps`.
- `fp-app` `main.rs`: takes the renderer from
  `CreationContext::integration_info`, logs it and tells `AppUi` whether
  it is software (AF2).

## 6. Left to the builder

- The names of the internal types and functions.
- How the cartwall's sweep gets the rate in use.
