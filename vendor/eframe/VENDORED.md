# Vendored `eframe` 0.37.0

Source: crates.io `eframe` 0.37.0 (MIT OR Apache-2.0,
<https://github.com/emilk/egui>), used through `[patch.crates-io]` in the
workspace `Cargo.toml`.

## Local changes

### No panic when a destroyed activity has no window (Android only in effect)

- `src/native/glow_integration.rs`, `save_and_destroy`: the window was
  taken with `window(ViewportId::ROOT)`, which panics when it does not
  exist. On Android eframe drops the window when the app is suspended, and
  an activity is always suspended before it is destroyed, so every
  destroyed activity (Recents swipe, "Don't keep activities", the system
  reclaiming it) ended with a panic on the way out (mobile spec MB2). It
  now uses `window_opt`, as `save` already does. On the desktop the window
  always exists there, so nothing changes.

### The swap time and the GL renderer for the app (adaptive frame rate spec AF1, AF2)

- `src/epi.rs`: `IntegrationInfo` gains `swap_usage` (the seconds
  `swap_buffers` took on the previous frame) and `gl_renderer` (a new
  `GlRenderer` with `GL_RENDERER`, `GL_VENDOR` and `GL_VERSION`); both
  `None` outside the glow integration (`epi_integration.rs`,
  `web/app_runner.rs`, the kittest mock).
- `src/native/epi_integration.rs`: `report_swap_time` and
  `set_gl_renderer`.
- `src/native/glow_integration.rs`: the renderer is read once after the
  painter is created (an `unsafe` GL query, like eframe's own), and the
  swap is timed where eframe already pauses its frame timer.

The application needs both to tell how much a frame costs when the
renderer draws in software (the rasterizing happens in the swap), and its
own code may not call GL (`unsafe`).

The crate's own files are copied as published. Drop this copy once an
eframe release makes the same changes (or offers the same information).
