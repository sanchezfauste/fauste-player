# Mobile Plan 1 — Android Spike Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Answer, on an emulator and on a real Android phone, the questions
the mobile spec leaves to the spike (§9.1), and turn the answers into a
spec amendment the maintainer approves before the Android plans are
written.

**Architecture:** A **throwaway** branch `spike/android`, never merged. A
`cdylib` crate `crates/fp-spike-android` starts the same session as
`crates/fp-app/src/main.rs::run` (store, engine with cpal + Null,
conductor, analysis, services) inside `android_main`, keeps it in a
process-wide `OnceLock`, and runs the existing `Shell(AppUi)` through
eframe's `android-game-activity` backend. A minimal Gradle project
`mobile/android-spike/` holds `MainActivity` (GameActivity) and a
`PlaybackService` (foreground, `mediaPlayback`). Hacks are allowed; each
one is written down as a finding.

**Tech Stack:** Rust 1.98 (edition 2024), eframe/egui 0.37 (`glow`,
`android-game-activity`), cpal 0.18 (AAudio), `cargo-ndk`, NDK
27.0.12077973, Android SDK platforms 34/36, Gradle 9.3.1, Kotlin,
`androidx.games:games-activity`.

**Spec:** `docs/superpowers/specs/2026-10-25-mobile-design.md`

## Global Constraints

- The spike branch is never merged; only the findings reach `master`
  (Task 7), through a docs PR.
- No change to `fp-model` rules. Desktop code may be patched on the spike
  branch only to get it to compile for Android; every such patch is a
  finding ("needs gating").
- `unsafe` (including `#[unsafe(no_mangle)]`) only in
  `crates/fp-spike-android`, which drops the workspace `[lints]` table;
  every site is a finding (spec §8).
- Real-time rules (CLAUDE.md rule 5) are not relaxed: the mixer and the
  backend render path are used unchanged.
- Spikes get no tests (brainstorming skill); evidence is logcat output,
  screenshots and the findings table.
- Nothing plays on start (CLAUDE.md rule 10): the spike loads the saved
  state like the desktop and waits for the operator.
- Tooling present on this machine: `~/Android/Sdk` (set
  `ANDROID_HOME=$HOME/Android/Sdk`,
  `ANDROID_NDK_HOME=$ANDROID_HOME/ndk/27.0.12077973`), targets
  `aarch64-linux-android` and `x86_64-linux-android`, `cargo-ndk`, an
  `android-34 google_apis x86_64` emulator image, JDK 25.

## Review Focus

The spike has no tests; these are the conditions most likely to break the
real app that the findings must cover explicitly (each is a row of the
findings table in Task 7):

1. Activity destroyed while audio plays (rotation, back button, "don't
   keep activities"): audio must not stop, and a new activity must show
   the running session, not a second one.
2. Screen off for 30 minutes with a track playing: no xrun or underrun in
   the log, process not killed.
3. A route on a Bluetooth device that disconnects and reconnects
   mid-playout: nothing re-routes silently, nothing starts by itself.
4. A phone call (or another app taking audio focus) during playout: what
   AAudio does to the stream (keeps going, ducked, disconnected).
5. Two routes on two devices at once (speaker + Bluetooth, speaker + USB,
   speaker + wired): which pairs really sound simultaneously.

---

### Task 1: Cross-compile probe

**Files:**
- Modify (spike branch only, as needed): `crates/*/Cargo.toml`,
  `crates/fp-app/src/lib.rs`, any file that fails to build
- Create: `.superpowers/sdd/mobile-spike/progress.md` (ledger, git-ignored)

- [ ] **Step 1:** `git checkout -b spike/android master`; export
  `ANDROID_HOME` and `ANDROID_NDK_HOME` (Global Constraints).
- [ ] **Step 2:** Run
  `cargo ndk -t arm64-v8a build -p fp-engine -p fp-analysis -p fp-store`.
  Expected: builds. Record any failure (crate, error) in the ledger.
- [ ] **Step 3:** Run `cargo ndk -t arm64-v8a build -p fp-app --lib`.
  Record each failing dependency (`rfd`, `midir`, `fp-remote`,
  `thread-priority`, `directories`, cpal `realtime`…).
- [ ] **Step 4:** Make it build with the smallest patch per failure
  (`cfg(not(target_os = "android"))` on the dependency and on the code
  using it). Each patch is a ledger line:
  `Finding: <crate/module> — needs gating — <how it was gated>`.
- [ ] **Step 5:** Repeat Step 3 until it passes; commit
  `chore(spike): build fp-app for Android`.

### Task 2: The spike crate (`android_main`)

**Files:**
- Create: `crates/fp-spike-android/Cargo.toml` (`crate-type = ["cdylib"]`,
  no `[lints] workspace = true`)
- Create: `crates/fp-spike-android/src/lib.rs`

**Interfaces:**
- Produces: the library `libfp_spike_android.so`, exporting
  `android_main(app: android_activity::AndroidApp)`; Task 3 loads it as
  `fp_spike_android`.

- [ ] **Step 1:** Dependencies: `fp-app`, `fp-engine`, `fp-backends`,
  `fp-store`, `fp-model`, `fp-analysis`, `eframe` with features
  `android-game-activity` and `glow` (no `wayland`/`x11` on Android: use a
  target-specific feature set), and a logcat tracing layer
  (`paranoid-android` or `tracing-android`; builder's pick).
- [ ] **Step 2:** `android_main` starts the session once:
  `static SESSION: OnceLock<Session>`, where `Session` holds what
  `main.rs::run` builds before `run_native` (conductor handle, services,
  analyzer, media cache, backends). `AppPaths` come from
  `app.internal_data_path()` (data) and its `cache` sibling. On a second
  call (new activity) it reuses `SESSION` and only builds a new `AppUi`
  from it.
- [ ] **Step 3:** Runs `eframe::run_native` with
  `NativeOptions { android_app: Some(app), .. }` and
  `Shell::new(app_ui)` after `theme::apply`, exactly like the desktop.
  When `run_native` returns, it does **not** stop the engine (the service
  keeps the process alive). Log `"activity attached"` /
  `"activity detached"` with a counter.
- [ ] **Step 4:** At start, log every output device cpal reports
  (host id, device name, supported configs) at `info`.
- [ ] **Step 5:** `cargo ndk -t arm64-v8a -t x86_64 -o
  mobile/android-spike/app/src/main/jniLibs build -p fp-spike-android`.
  Expected: two `.so` files. Ledger: every `unsafe` site
  (`Finding: unsafe — <file:line> — <why>`). Commit.

### Task 3: Gradle shell with the foreground service

**Files:**
- Create: `mobile/android-spike/` (Gradle wrapper 9.3.1, `settings.gradle.kts`,
  `app/build.gradle.kts`, `AndroidManifest.xml`)
- Create: `app/src/main/java/com/sanchezfauste/faustespike/MainActivity.kt`
- Create: `app/src/main/java/com/sanchezfauste/faustespike/PlaybackService.kt`

- [ ] **Step 1:** `applicationId "com.sanchezfauste.FaustePlayer.spike"`,
  `minSdk 29`, `targetSdk 36`, ABIs `arm64-v8a`, `x86_64`; dependency
  `androidx.games:games-activity` (version matching android-activity's
  GameActivity; builder checks android-activity's docs).
- [ ] **Step 2:** Manifest: permissions `FOREGROUND_SERVICE`,
  `FOREGROUND_SERVICE_MEDIA_PLAYBACK`, `POST_NOTIFICATIONS`,
  `READ_MEDIA_AUDIO`, `READ_EXTERNAL_STORAGE` (`maxSdkVersion 32`);
  `PlaybackService` with `foregroundServiceType="mediaPlayback"`;
  `MainActivity` with `android.app.lib_name = fp_spike_android` and
  `configChanges` **not** set (so rotation recreates the activity, which
  is what the spike must observe).
- [ ] **Step 3:** `MainActivity : GameActivity` requests
  `READ_MEDIA_AUDIO` and `POST_NOTIFICATIONS`, then starts
  `PlaybackService` (spike: always on while the process lives).
  `PlaybackService` posts a plain notification ("Fauste spike") and calls
  `startForeground` with the media-playback type. Nothing else.
- [ ] **Step 4:** `./gradlew assembleDebug`. Expected:
  `app/build/outputs/apk/debug/app-debug.apk`. Commit.

### Task 4: Emulator checks

- [ ] **Step 1:** Start the `android-34 google_apis x86_64` emulator;
  `adb install -r app-debug.apk`; push two test files to
  `/sdcard/Music/` (a WAV tagged as in CLAUDE.md "Tagged test audio" and
  an MP3 if available); `adb shell am broadcast -a
  android.intent.action.MEDIA_SCANNER_SCAN_FILE`… or reboot the emulator so
  MediaStore indexes them.
- [ ] **Step 2 (UI):** The UI appears; record whether text, icons (Phosphor
  font), touch, scrolling and drag work. Screenshot
  (`adb exec-out screencap -p`).
- [ ] **Step 3 (files, MB7):** Adding `/sdcard/Music/<file>` by path (spike:
  type it, or hardcode it in a debug command) loads, analyses and plays.
  Record whether `std::fs` opens the path with only `READ_MEDIA_AUDIO`.
- [ ] **Step 4 (keyboard):** Focus a text field (playlist rename): does the
  soft keyboard appear, and do typed characters arrive? Record verdict.
- [ ] **Step 5 (activity recreate, Review Focus 1):** Play, rotate, press
  back, reopen; also with Developer options → "Don't keep activities".
  Expected from logcat: one session, attach/detach counter increases,
  audio continues. Record what really happens.
- [ ] **Step 6:** Fill the ledger rows for UI, MB7, IME, recreate.

### Task 5: Real-device checks (needs the maintainer's phone)

The maintainer connects a phone with USB debugging (`adb devices` lists
it) and, if possible, a Bluetooth speaker/headphones and a USB audio
interface or USB-C dongle.

- [ ] **Step 1 (devices):** Install the `arm64-v8a` APK; copy the logged
  device list into the ledger.
- [ ] **Step 2 (dual output, Review Focus 5):** In Settings → outputs, set
  Player 1 Main on one device and its Cue (or the cartwall) on another;
  play both. Record, per pair (speaker+BT, speaker+USB, speaker+wired,
  BT+USB): both sound / one only / error in log.
- [ ] **Step 3 (screen off, Review Focus 2):** Play a long playlist with
  the screen off for 30 minutes. Pass: no `alert-underruns` / xrun lines in
  logcat, process alive (`adb shell pidof com.sanchezfauste.FaustePlayer.spike`
  unchanged).
- [ ] **Step 4 (Bluetooth, Review Focus 3):** Route Player 1 to a
  Bluetooth device, play, turn Bluetooth off, then on. Record what the
  engine logs and whether anything moves to the speaker or restarts.
- [ ] **Step 5 (call / focus, Review Focus 4):** Call the phone during
  playout (or start another music app). Record whether the stream keeps
  going, is ducked, or disconnects.
- [ ] **Step 6 (latency):** Record the buffer size AAudio grants with the
  default buffer config, and CPU load (`adb shell top -n 1`) while two
  players and a cart play.

### Task 6: OS kill

- [ ] **Step 1 (MB3):** During playout,
  `adb shell am kill com.sanchezfauste.FaustePlayer.spike` (with the app in
  the background) and `adb shell am force-stop …`; relaunch. Pass: state
  restored, nothing plays.

### Task 7: Findings and spec amendment

**Files:**
- Modify (from `master`, branch `docs/mobile-spike-findings`):
  `docs/superpowers/specs/2026-10-25-mobile-design.md`

- [ ] **Step 1:** Write a "Spike findings" section (§9.1 → new §11) with
  one table row per check of Tasks 1–6 (check, result, consequence for
  the plans), the list of modules needing gating, the `unsafe` sites, the
  dual-output matrix, the IME verdict and the recommended `minSdk`.
- [ ] **Step 2:** Amend any MB rule the spike contradicts (for example
  MB2 if `android_main` cannot be re-entered, MB6 if dual output never
  works, §6 if the soft keyboard is a blocker); mark each amendment
  "from the spike".
- [ ] **Step 3:** Commit `docs: record the Android spike findings in the
  mobile spec`, open the PR, and ask the maintainer to approve it. The
  Android plans (§9.2) are written only after that approval.
- [ ] **Step 4:** Keep `spike/android` pushed as a reference branch (not
  merged); note its last commit in the findings.
