# Vendored `eframe` 0.37.0

Source: crates.io `eframe` 0.37.0 (MIT OR Apache-2.0,
<https://github.com/emilk/egui>), used through `[patch.crates-io]` in the
workspace `Cargo.toml`.

## Local change (Android only in effect)

- `src/native/glow_integration.rs`, `save_and_destroy`: the window was
  taken with `window(ViewportId::ROOT)`, which panics when it does not
  exist. On Android eframe drops the window when the app is suspended, and
  an activity is always suspended before it is destroyed, so every
  destroyed activity (Recents swipe, "Don't keep activities", the system
  reclaiming it) ended with a panic on the way out (mobile spec MB2). It
  now uses `window_opt`, as `save` already does. On the desktop the window
  always exists there, so nothing changes.

The crate's own files are copied as published. Drop this copy once an
eframe release makes the same change.
