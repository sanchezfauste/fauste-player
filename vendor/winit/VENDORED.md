# Vendored `winit` 0.30.13

Source: crates.io `winit` 0.30.13 (Apache-2.0,
<https://github.com/rust-windowing/winit>), used through `[patch.crates-io]`
in the workspace `Cargo.toml`.

## Local changes (Android only)

The Android app keeps its audio session at process level and builds a new
interface for each activity (mobile spec MB2). Two things in winit 0.30.13
stopped that:

- `src/platform_impl/android/mod.rs`: `MainEvent::Destroy` was only logged.
  The activity's `onDestroy` (in android-activity's native glue) waits until
  `android_main` returns, so the main thread hung (an ANR). The event loop
  now exits on it, eframe returns and `android_main` ends.
- `src/event_loop.rs`: a second `EventLoop` in the process was refused
  (`RecreationAttempt`). android-activity runs each activity's
  `android_main` on a new thread, after the previous one ended, so the
  check is skipped on Android.

Desktop targets are unchanged. The examples, tests and dev-dependencies
are left out. Drop this copy once a winit release does both.
