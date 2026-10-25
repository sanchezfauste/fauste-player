# Mobile (Android, then iOS) — Design Spec

- **Date:** 2026-10-25.
- **Status:** Approved (2026-10-25).
- **Extends:** [the main spec](2026-09-25-fauste-player-design.md) with
  new target platforms. It changes no player rule.
- **Numbering:** rules are `MB<n>`. Each rule is testable and gets at least
  one test where a test can observe it (host tests for the shared code,
  device checks listed in §9 for the rest).

---

## 1. Intent

Fauste Player on a phone or a tablet, as a **standalone app**: the music
lives on the device, the audio comes out of the device, no computer takes
part. The common use is one or more players and a cartwall, configured as
on the desktop.

The point is the same product, not a lighter one: the same audio engine,
the same rules, the same reliability (no dropouts, nothing goes on air by
itself). Android comes first; iOS is a later phase that should reuse
everything Android needs from Rust.

## 2. Goals, non-goals, anti-goals

**Goals**

- G1. Android app (phone and tablet, portrait and landscape) with the
  desktop behaviour: players (count configured as on the desktop),
  cartwall, playlists, library, markers, analysis, loudness
  normalization, CUE pre-listen with the on-air confirmation (CA rules),
  settings, persistence, backups, locales.
- G2. Maximum reuse: `fp-model`, `fp-store`, `fp-decode`, `fp-dsp`,
  `fp-analysis`, `fp-engine` and `fp-backends` (cpal) build for Android
  unchanged in behaviour; the UI stays egui (`fp-app/src/ui`), with a
  touch layout added.
- G3. Uninterrupted playout with the screen off, the app in the
  background, or another app in front.
- G4. Each player and the cartwall pick their output among the devices
  the system offers (speaker, wired headphones, Bluetooth, USB),
  exactly like the desktop outputs config.
- G5. A later iOS phase reusing the same Rust crates and the same egui UI.

**Non-goals (for now)**

- MIDI control surfaces (`fp-control`) and the remote API (`fp-remote`):
  not built for mobile. They may come later.
- Any link with the desktop app (sync, remote control, shared library).
- Split-channel CUE (left CUE / right programme): rejected by the
  maintainer.
- Exclusive / bit-perfect output and DSD native output: desktop only.
- Writing to music files in the first version (maintainer, 2026-10-25):
  the music is read-only. The tag editor and cover editing are hidden on
  mobile (tags and covers are still shown), and backups stay in the app's
  own storage, with no import/export to user folders yet. Markers, loudness
  and every other edit that lives in the app's state work as on the
  desktop.

**Anti-goals (a failure even if it works)**

- Audio that stops or glitches when the screen locks or the app goes to
  the background.
- Something going on air by itself after the OS kills and restarts the
  app, after a call, or after a device (Bluetooth) reconnects.
- A second implementation of player rules in Kotlin/Swift. Behaviour stays
  in `fp-model`.
- A UI that blocks (file scans, device enumeration on the UI thread).

## 3. Architecture

```
┌──────────────── Android app (APK/AAB) ────────────────┐
│ Kotlin (thin)                     Rust (cdylib)        │
│  MainActivity (GameActivity) ───► fp-mobile ──► fp-app │
│  PlaybackService (foreground,      (JNI glue,    (ui,  │
│   mediaPlayback) + MediaSession     platform     services,
│  audio-focus / device callbacks     trait impl)  bootstrap)
│  media permission                       │              │
│                                         ▼              │
│                         fp-engine / fp-backends(cpal → AAudio)
│                         fp-model, fp-store, fp-decode, fp-analysis
└────────────────────────────────────────────────────────┘
```

- **One Rust library** (`cdylib`) built with `cargo ndk`, loaded by a
  small Kotlin/Gradle project under `mobile/android/`.
- **UI:** eframe on Android through winit's `android-activity`
  (GameActivity backend, `glow`). The same `fp-app` UI code; a touch
  layout chosen by screen size (§6).
- **Audio:** the existing engine and mixer, rendering through cpal's
  AAudio host. Real-time rules (CLAUDE.md rule 5) apply unchanged.
- **Kotlin does only what the OS requires a Java class for:** the
  activity, the foreground service, the media session/notification, the
  audio-focus and audio-device callbacks, and the permission request.
  It contains no player logic; it forwards events to Rust and Rust tells
  it to start/stop the service.
- **New crate `fp-mobile`:** the JNI glue and the Android implementation
  of a small platform trait in `fp-app` (§4). It is the only crate that
  needs `unsafe` (obtaining the `JavaVM`, `#[no_mangle]` JNI exports); see
  §8.
- **Desktop-only pieces are gated off** for `target_os = "android"`:
  `fp-control`, `fp-remote`, `rfd`, `open`, single-instance
  (`instance.rs`), window-system detection, the CLI, exclusive backends.
  Their settings pages are hidden on mobile.

## 4. Platform seam in `fp-app`

`fp-app` gains a `Platform` trait (names are the builder's) for what the
desktop gets from `rfd`, `open`, `directories` and the window system:

- the state directory (`FAUSTE_HOME` equivalent: the app's files dir);
- picking audio files/folders (desktop: `rfd`; Android: the in-app media
  browser of §5);
- opening a URL (About link, guide);
- keep-screen-on, and "playout active" (Android: start/stop the
  foreground service).

The desktop implementation wraps today's code; nothing changes there.
Calls that wait (pickers) answer through channels, never blocking the UI
(rule 8).

## 5. Behaviour on Android

- **MB1 — Foreground service while anything can sound.** The foreground
  service (`foregroundServiceType="mediaPlayback"`, notification with the
  app name and what is on air) runs while any player has a track loaded
  (playing, paused or stopped: between items a player usually sits
  stopped with the next track ready), any cart is playing, or a CUE is
  active. It stops only when no player has a track loaded and nothing
  sounds. *(Builder's call: a show must never fall back to a cold start
  between items.)* While it runs,
  the OS must not kill the process; that is what keeps G3.
- **MB2 — The engine outlives the activity.** Rotation, leaving the app
  and the activity being destroyed never touch the engine, the services
  thread or the analysis pool. They live at process level; a new
  activity attaches to the running session and only rebuilds the UI.
- **MB3 — Process restart is a cold start.** If the OS kills the process,
  the next launch restores state like a desktop restart (spec rules for
  restart): nothing plays (CLAUDE.md rule 10).
- **MB4 — Audio focus never stops playout by itself.** Losing audio focus
  (a call, another app) does not pause or stop anything: a broadcast is
  not the device's music player. The UI shows a warning banner while
  focus is lost, and the event is logged. The app requests focus with
  `USAGE_MEDIA` when playout becomes active. *(Builder's call recorded
  here; flip only by maintainer decision.)*
- **MB4b — The media session shows, it does not control.** The
  notification and lock screen show what is on air; they offer no
  transport controls, and headset/Bluetooth media buttons are ignored
  (a reconnecting device or a pocket press must never start anything).
  The only action in the notification opens the app. *(Builder's call.)*
- **MB5 — Outputs are the system's audio devices.** The outputs list
  shows what `AudioManager.getDevices(OUTPUTS)` reports (through cpal's
  enumeration if sufficient, otherwise through `fp-mobile`), named by
  type and product name. Each player, the cartwall and their Cue routes
  pick from it as on the desktop. A route saved for a device that is
  absent (Bluetooth off) behaves like a missing device on the desktop
  (existing rules): the source is not available, nothing re-routes
  silently to another device and nothing starts when it returns.
  Device identity across reconnects is matched by type + product name,
  not by the volatile Android device id.
- **MB6 — CUE logic is the desktop's.** A Cue route on its own device
  plays at once; a Cue route on an output that carries a Main route is
  an on-air output and follows CA1–CA… (confirmation while something
  sounds). Whether two routes can really sound on two devices at once is
  the OS's business (it usually works for speaker + Bluetooth or USB, and
  usually not for speaker + wired headphones): if the system refuses or
  merges the second stream, the backend reports it like any device
  failure, the UI says so, and nothing else changes.
- **MB7 — Music access.** The app asks for `READ_MEDIA_AUDIO` (Android 13+;
  `READ_EXTERNAL_STORAGE` below) and lists audio files through
  `MediaStore`, giving real file paths that the existing decoders open
  directly. Tracks keep `PathBuf` paths; no copy into app storage. A
  file that disappears degrades to "missing" like on the desktop
  (rule 9). The in-app browser groups by folder and is used wherever the
  desktop opens a file or folder dialog (adding to playlists, assigning
  carts, library folders). Folders outside `MediaStore` (SAF tree URIs)
  are out of scope for the first version.
- **MB8 — Keep screen on** is a `Config` field (default on while playout
  is active), per CLAUDE.md rule 4.
- **MB9 — Buffer and latency** use the existing buffer config; AAudio is
  opened in low-latency performance mode. Defaults may differ per
  platform; they stay `Config` fields with ranges.

## 6. Touch layout

- Same panels as the desktop (players, cartwall, playlists, library,
  settings), same theme (`theme.rs`), same Fluent strings.
- **Phone:** one panel at a time with a bottom tab bar (Players,
  Cartwall, Library/Playlists); on-air status of every player stays
  visible in a compact strip above the tabs. **Tablet** (shortest side
  ≥ 600 dp): the desktop's resizable layout with touch-sized targets.
  The exact breakpoints and arrangement are the builder's: the builder
  draws phone and tablet mockups (in the design project, or as HTML
  mockups if it is unreachable) and the maintainer approves them before
  the UI plan starts.
- Touch targets ≥ 48 dp; no hover-only affordances (hover tooltips get a
  long-press equivalent); drag and drop works by long-press.
- Text input (tag editor, names) uses the soft keyboard through winit's
  IME on Android. If that proves unreliable, it is a blocker to raise
  before the UI plan, not something to work around silently.

## 7. Build, CI, packaging

- `mobile/android/`: Gradle project (Kotlin), `minSdk` 29 (builder may
  raise it with reason), `targetSdk` current Play requirement, ABIs
  `arm64-v8a` and `x86_64` (emulator).
- CI: a job that cross-compiles the Rust library for Android and builds a
  debug APK on Linux. Host tests keep running everywhere; `fp-mobile`
  logic that can be tested without a device is.
- Release: a signed AAB/APK attached to the GitHub release; the Play
  Store listing is a later step (stores.yml). The About window keeps the
  author attribution (NOTICE §7(b)).
- Docs: README platform table, a user-guide page for mobile, a technical
  page (`docs/technical/mobile.md`), CLAUDE.md commands and layout,
  locales for every new string.

## 8. Safety rule exception

CLAUDE.md rule 6 forbids `unsafe_code` workspace-wide. JNI requires a
small amount of it. `fp-mobile` is the single exception (approved by the
maintainer, 2026-10-25, **for the mobile build only**):

- it opts out of the workspace `unsafe_code = "forbid"`, keeping every
  other lint, and keeps each `unsafe` block minimal with a `// SAFETY:`
  comment;
- it is a dependency only under `[target.'cfg(target_os = "android")']`
  (later also iOS), so no desktop build compiles any `unsafe` of ours;
  CI checks that the desktop dependency tree does not contain it;
- every other crate keeps `forbid`; CLAUDE.md rule 6 is amended to name
  this exception and its limit.

## 9. Phases

1. **Spike (throwaway):** eframe + engine + cpal on a real Android phone:
   sound through AAudio, screen off with a foreground service, rotation,
   two outputs at once (speaker + Bluetooth/USB), soft keyboard. Its
   findings may amend this spec; the maintainer approves the amendment
   before planning.
2. **Android plan(s):** platform seam and gating; `fp-mobile` + Kotlin
   shell + service (MB1–MB4); devices and outputs (MB5–MB6); media
   access (MB7); touch layout (§6); CI and packaging (§7); docs.
3. **iOS (later, own spec):** same crates; Swift shell for
   `AVAudioSession` (playback category, background audio mode,
   interruptions), document access, cpal's CoreAudio host. Note: iOS
   sends all output to one route (except USB multi-route), so a separate
   Cue device will rarely exist there; CUE on the on-air output with its
   confirmation still works.

Device checks for each Android plan (manual, on at least one phone and
one tablet): 30 minutes of playout with the screen off and no xrun in
the log; rotation during playout; a call during playout (MB4); Bluetooth
off/on with a route on it (MB5); OS kill and relaunch (MB3).

## 10. Left to the builder

Crate and type names; JNI by hand (`jni` crate) vs a generator (UniFFI)
for the bridge; phone/tablet breakpoints; how the in-app browser looks;
Gradle structure; exact CI job layout; how the "playout active" state
is derived from the snapshot (it must come from `fp-model` state).
