# Mobile Plan 2 — Android Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** An installable Android build of Fauste Player that runs the
desktop session and UI, keeps playing in the background, survives the
activity lifecycle and never starts anything by itself. It is a
foundation: output choice, the media browser and the touch layout come
in later plans.

**Architecture:** `fp-app` gains a `Platform` seam (pickers, URL opening,
focus state) and an `AppSession` shared by the desktop binary and the new
Android-only crate `fp-mobile`. `fp-mobile` exports `android_main`, keeps
the session at process level, wraps the existing `Shell` (IME bridge,
safe area) and exposes a few JNI functions that a thin Kotlin shell in
`mobile/android/` polls (foreground service, keep-screen-on) or calls
(insets, audio focus). Behaviour that decides anything (`playout_active`)
is a pure function in `fp-model`.

**Tech Stack:** Rust 1.98 (edition 2024), eframe/egui 0.37
(`android-game-activity`), cpal 0.18 (AAudio), `jni`, `paranoid-android`,
`cargo-ndk` (API 29), Kotlin 2.2.21, AGP 8.13.2, Gradle 9.3.1,
`androidx.games:games-activity:4.4.0`.

**Spec:** `docs/superpowers/specs/2026-10-25-mobile-design.md` (§3–§5, §8,
§11). Reference implementation of the throwaway spike: branch
`spike/android` (`crates/fp-spike-android`, `mobile/android-spike`).

## Global Constraints

- `fp-mobile` is the only crate allowed `unsafe`: its `[lints.rust]`
  sets `unsafe_code = "deny"` (every other workspace lint copied as is),
  each site carries `#[allow(unsafe_code)]` and a `// SAFETY:` comment;
  every other crate keeps `forbid`.
- `fp-mobile` is never a dependency of a desktop build; it is a workspace
  member so its pure modules test on the host, and every Android-only
  module is `#[cfg(target_os = "android")]`.
- No change to player rules; `playout_active` is the only new model rule.
- Real-time rules (CLAUDE.md rule 5) unchanged; nothing new runs on the
  device callback.
- Nothing plays on start, restart or after an OS kill (rule 10, MB3).
- Audio focus never pauses or stops anything (MB4); the media session
  offers no transport controls and ignores media buttons (MB4b).
- Music is read-only on Android: tag editor, cover editing and playlist
  and cart-page import/export are hidden there (spec §2 non-goals).
- MIDI (`fp-control`) and the remote API (`fp-remote`) are not built for
  Android.
- Every new UI string goes into every `crates/fp-app/locales/*/main.ftl`
  (en-US and es-ES written, the rest translated).
- Android: `minSdk` 29, `targetSdk` 36, ABIs `arm64-v8a`, `x86_64`,
  application id `com.sanchezfauste.FaustePlayer`, Rust built with
  `cargo ndk -P 29`.
- Latency and dropouts are judged on release builds only.

## Review Focus

1. The activity destroyed by the system while audio plays (not rotation):
   the audio continues and a new activity shows the running session, not
   an ANR or a second session (Task 6, `android_main` re-entry; checked
   in Task 9 with "Don't keep activities").
2. A player stopped with its next track ready while the app is in the
   background: the foreground service stays, the process is not killed
   (Task 1 `stopped_player_with_a_next_track_keeps_playout_active`).
3. Autocorrect replacing a word in the middle of a field: the field ends
   with exactly the IME's text (Task 6 `replacing_inside_the_text`).
4. A route-less player on Android: it sounds on the default output, not
   on Null (Task 2 `the_host_default_id_resolves_to_the_default_device`).
5. The desktop build pulling `fp-mobile` or `unsafe` by accident: CI
   fails (Task 8 `scripts/check-mobile-isolation.sh`).

---

### Task 1: `playout_active` and the keep-screen-on setting

**Files:**
- Create: `crates/fp-model/src/playout.rs`
- Modify: `crates/fp-model/src/lib.rs` (module + `pub use playout::playout_active`)
- Modify: `crates/fp-model/src/config.rs` (`UiConfig.keep_screen_on`)
- Test: `crates/fp-model/tests/it/playout.rs` (+ `mod playout;` in `tests/it/main.rs`)

**Interfaces:**
- Produces: `pub fn playout_active(state: &AppState) -> bool`;
  `UiConfig.keep_screen_on: bool` (serde default `true`).

- [ ] **Step 1: Write the failing tests** (`tests/it/playout.rs`), one per
  MB1 case, built on `AppState::new(Config::default(), "Main")` like the
  other model tests:
  `a_new_state_is_not_playout_active` (false),
  `stopped_player_with_a_next_track_keeps_playout_active` (player
  `next = Some(entry)`, Stopped → true),
  `playing_or_paused_player_is_playout_active` (true for both),
  `a_playing_cart_is_playout_active`, `a_player_cue_is_playout_active`,
  `a_cart_cue_is_playout_active` (true),
  `a_player_with_nothing_loaded_and_nothing_sounding_is_not` (current and
  next `None`, Stopped, no carts → false); and in `tests/it/config.rs`
  (or the existing config test file) `keep_screen_on_defaults_on_and_old_configs_load`
  (a config JSON without the field loads with `true`).
- [ ] **Step 2: Run** `cargo test -p fp-model --test it playout` — FAIL (no `playout_active`).
- [ ] **Step 3: Implement** `playout_active`: true if any player has
  `current` or `next` set, or `transport != Stopped`, or `cue.is_some()`;
  or the cartwall has a playing cart or `cartwall.cue.is_some()`. Doc
  comment cites MB1. Add `keep_screen_on` with doc "MB8: keep the screen
  on while playout is active (mobile only)".
- [ ] **Step 4: Run** `cargo test -p fp-model --test it` — PASS.
- [ ] **Step 5: Commit** `feat(model): add playout_active and the keep-screen-on setting`.

### Task 2: Default AAudio device

**Files:**
- Modify: `crates/fp-backends/src/cpal_backend.rs:259-264` (`find`)
- Test: same file's `#[cfg(test)]` module

**Interfaces:**
- Produces: `fn resolve<D>(wanted: &str, default: Option<(String, D)>, by_id: impl FnOnce() -> Option<D>) -> Option<D>` (private helper).

- [ ] **Step 1: Write the failing tests**
  `the_host_default_id_resolves_to_the_default_device`
  (`resolve("aaudio:-1", Some(("aaudio:-1".into(), 7)), || None) == Some(7)`)
  and `another_id_goes_through_the_device_list`
  (`resolve("aaudio:5", Some(("aaudio:-1".into(), 7)), || Some(5)) == Some(5)`).
- [ ] **Step 2: Run** `cargo test -p fp-backends --lib resolve` — FAIL.
- [ ] **Step 3: Implement** `resolve` and use it in `find`: the default
  is `host.default_output_device()` with its `id()`; cpal's AAudio host
  names that device `aaudio:-1` and `device_by_id` cannot find it, which
  sent route-less players to Null (spec §11).
- [ ] **Step 4: Run** `cargo test -p fp-backends` — PASS.
- [ ] **Step 5: Commit** `fix(backends): open the host default device by its own id`.

### Task 3: `AppSession`

**Files:**
- Create: `crates/fp-app/src/session.rs`
- Modify: `crates/fp-app/src/lib.rs`, `crates/fp-app/src/main.rs:114-247` (`run` uses it)
- Test: `crates/fp-app/tests/it/app_session.rs` (+ `mod` line)

**Interfaces:**
- Produces:
  ```rust
  pub struct AppSession {
      pub handle: Arc<ConductorHandle>,
      pub requests: Sender<ServiceRequest>,
      pub activity: ActivityCell,
      pub faults: Arc<AtomicU64>,
      pub media: MediaCache,
      pub backends: Vec<Arc<dyn AudioBackend>>,
      pub language: Option<String>,
      /* private: ServicesHandle, analysis cache dir */
  }
  impl AppSession {
      pub fn start(paths: AppPaths, backends: Vec<Arc<dyn AudioBackend>>, report_cap: Option<crash::ReportCap>) -> Result<Self, Box<dyn std::error::Error>>;
      pub fn app_ui(&self) -> AppUi; // AppUi::new(..).with_services(..).with_analysis_activity(..).with_service_faults(..).with_backends(..)
      pub fn analysis_dir(&self) -> PathBuf;
      pub fn shutdown(self); // services, then the engine (main.rs `stop_engine` moves here)
  }
  ```

- [ ] **Step 1: Write the failing test**
  `a_session_starts_and_stops_on_the_null_backend`: temp dir,
  `AppSession::start(AppPaths::under(dir), vec![Arc::new(NullBackend)], None)`,
  `session.handle.model.load().players` is not empty, `shutdown()` returns,
  and `dir/config/config.json` exists afterwards (the final save).
- [ ] **Step 2: Run** `cargo test -p fp-app --test it app_session` — FAIL.
- [ ] **Step 3: Implement** by moving the body of `main.rs::run` up to
  `AppUi::new` (store load, backends + Null last, engine, conductor,
  analyzer, services) into `start`; `main.rs` keeps CLI, instance lock,
  MIDI, remote, inbox and the viewport, and calls `session.shutdown()`
  where it now stops services and engine. Same order as today.
- [ ] **Step 4: Run** `cargo test -p fp-app --test it` and `cargo run -p fp-app` briefly (window opens, closes cleanly) — PASS.
- [ ] **Step 5: Commit** `refactor(app): share session start-up through AppSession`.

### Task 4: The `Platform` seam and the audio-focus banner

**Files:**
- Create: `crates/fp-app/src/platform.rs`
- Modify: the seven `rfd::` call sites (`ui/app.rs` ×2, `ui/settings.rs`,
  `ui/playlist_files.rs` ×2, `ui/settings/carts.rs` ×3) and
  `ui/about.rs` (`Opener` becomes a `Platform` call)
- Modify: `crates/fp-app/src/ui/app.rs` (`with_platform`, banner),
  `crates/fp-app/locales/*/main.ftl`
- Test: `crates/fp-app/tests/it/platform_ui.rs`; `FakePlatform` in `tests/it/support/mod.rs`

**Interfaces:**
- Produces:
  ```rust
  pub trait Platform: Send + Sync {
      fn pick_audio_files(&self, filter_name: &str, start: Option<PathBuf>) -> Vec<PathBuf>;
      fn pick_file(&self, filter_name: &str, extensions: &[&str], start: Option<PathBuf>) -> Option<PathBuf>;
      fn pick_folder(&self, start: Option<PathBuf>) -> Option<PathBuf>;
      fn save_file(&self, filter_name: &str, extensions: &[&str], file_name: String) -> Option<PathBuf>;
      fn open_url(&self, url: &str) -> Result<(), String>;
      fn audio_focus_lost(&self) -> bool;
      fn can_write_user_files(&self) -> bool; // false on Android: hides tag/cover editing and import/export
  }
  pub struct DesktopPlatform; // rfd + open; audio_focus_lost = false; can_write_user_files = true
  impl AppUi { pub fn with_platform(self, platform: Arc<dyn Platform>) -> Self; }
  ```
  Picker methods block; they are only called from the helper threads that
  call `rfd` today (rule 8).

- [ ] **Step 1: Write the failing tests** (kittest, `FakePlatform` records
  calls and returns canned answers):
  `add_asks_the_platform_for_files_and_adds_them` (click the playlist's
  Add; the fake returns two paths; the controller receives the add
  command with those two paths),
  `a_lost_audio_focus_shows_the_banner` (fake returns `true`; the text of
  `alert-audio-focus-lost` is shown; `false` → absent),
  `without_user_file_writes_the_row_menu_has_no_edit_tags` (fake
  `can_write_user_files = false`; the row menu lacks `menu-edit-tags`;
  the playlist tab menu lacks import/export).
- [ ] **Step 2: Run** `cargo test -p fp-app --test it platform_ui` — FAIL.
- [ ] **Step 3: Implement** the trait and `DesktopPlatform` (the current
  `rfd`/`open` code moved verbatim); `AppUi` defaults to `DesktopPlatform`.
  Strings: `alert-audio-focus-lost` = en-US "Another app took the audio
  focus — playout continues", es-ES "Otra app ha tomado el audio — la
  emisión continúa"; other locales translated.
- [ ] **Step 4: Run** `cargo test -p fp-app --test it` — PASS.
- [ ] **Step 5: Commit** `refactor(app): route pickers and links through a Platform seam`.

### Task 5: Build `fp-app` for Android without desktop-only parts

**Files:**
- Modify: `crates/fp-app/Cargo.toml` (`fp-control`, `fp-remote`, `rfd`,
  `open`, `winit` features move under
  `[target.'cfg(not(target_os = "android"))'.dependencies]`; eframe gets
  `android-game-activity` under `cfg(target_os = "android")`)
- Modify: `crates/fp-app/src/lib.rs` (`cli`, `instance`, `midi`, `remote`,
  `window_system` gated), `ui/settings.rs` (MIDI and Remote sections
  gated), `ui/app.rs` (the `midi`/`remote_status` fields gated),
  `platform.rs` (`DesktopPlatform` gated)

- [ ] **Step 1: Verify the failure:** `cargo ndk -P 29 -t arm64-v8a build -p fp-app --lib`
  — FAIL in `rfd` (spec §11).
- [ ] **Step 2: Gate** each module and field with
  `#[cfg(not(target_os = "android"))]`; no stub types.
- [ ] **Step 3: Verify:** the Android build passes;
  `cargo tree -p fp-app --target aarch64-linux-android -e normal | grep -E "rfd|midir|axum|fp-control|fp-remote"`
  prints nothing; `cargo clippy --workspace --all-targets -- -D warnings`
  and `cargo test --workspace` pass on the host.
- [ ] **Step 4: Commit** `build(app): leave desktop-only parts out of the Android build`.

### Task 6: `fp-mobile`

**Files:**
- Create: `crates/fp-mobile/Cargo.toml` (`crate-type = ["cdylib", "rlib"]`,
  lints per Global Constraints, Android deps under `cfg(target_os = "android")`)
- Create: `crates/fp-mobile/src/lib.rs`, `ime.rs`, `insets.rs` (pure,
  every target); `android.rs`, `jni.rs`, `platform.rs` (Android only)
- Test: `crates/fp-mobile/tests/it/{main.rs,ime.rs,insets.rs}`

**Interfaces:**
- Consumes: `AppSession` (Task 3), `Platform` (Task 4), `playout_active`,
  `UiConfig.keep_screen_on` (Task 1).
- Produces:
  ```rust
  // ime.rs
  pub struct ImeBridge { last: String }
  impl ImeBridge {
      pub fn new() -> Self;
      pub fn update(&mut self, now: &str) -> Vec<egui::Event>; // Backspace key presses for removed chars, then one Event::Text
      pub fn reset(&mut self, now: &str);                      // a new field took focus: no events
  }
  // insets.rs
  #[derive(Clone, Copy, Default, PartialEq, Debug)] pub struct Insets { pub left: f32, pub top: f32, pub right: f32, pub bottom: f32 } // physical px
  pub fn safe_area(screen: egui::Rect, insets: Insets, pixels_per_point: f32) -> egui::Rect;
  // jni.rs — called by Kotlin class com.sanchezfauste.fausteplayer.Native
  //   setInsets(left: Int, top: Int, right: Int, bottom: Int)
  //   setAudioFocusLost(lost: Boolean)
  //   state(): Int            // bit 0 playout active, bit 1 keep screen on
  //   onAirText(): String     // "P1 — <title>" lines for the notification, "" when nothing
  //   notificationChannelName(): String
  ```

- [ ] **Step 1: Write the failing tests** (`tests/it/ime.rs`):
  `typing_adds_text` (`"" → "Hola"` gives `[Text("Hola")]`),
  `deleting_sends_backspaces` (`"Hola" → "Ho"` gives two Backspace
  presses), `replacing_inside_the_text` (`"teh cat" → "the cat"`: two
  Backspaces then `Text("he cat")` — common prefix `t`),
  `non_ascii_counts_characters` (`"año" → "añ"`: one Backspace),
  `reset_emits_nothing` (after `reset("abc")`, `update("abc")` is empty);
  (`tests/it/insets.rs`) `safe_area_shrinks_by_the_insets_in_points`
  (screen 0,0–400,800, insets 0/96/0/48 px, ppp 2.0 → 0,48–400,776),
  `zero_insets_keep_the_screen`.
- [ ] **Step 2: Run** `cargo test -p fp-mobile --test it` — FAIL.
- [ ] **Step 3: Implement** `ime.rs`, `insets.rs`; then the Android
  modules from the spike's `crates/fp-spike-android/src/lib.rs`:
  `android_main` starts `AppSession` once (`OnceLock`), paths from
  `app.internal_data_path()`, logging through `paranoid-android` at
  `info`; builds `session.app_ui().with_platform(AndroidPlatform)` per
  activity; `MobileApp` implements `eframe::App`, delegating `logic`/`ui`
  to `Shell`, drawing it in `safe_area` and feeding `ImeBridge` from
  `AndroidApp::text_input_state()` in `raw_input_hook` (reset when egui's
  focused widget changes). `android_main` returns when eframe returns
  (activity destroyed) without touching the session, and the next call
  re-attaches (MB2). `AndroidPlatform`: `pick_audio_files` lists the
  audio files directly in `/sdcard/Music` (sorted; interim until the
  media-browser plan, MB7), `pick_file`/`save_file` → `None`,
  `pick_folder` → `/sdcard/Music`, `open_url` through an intent is left
  out (returns `Err("not available yet")`), `audio_focus_lost` reads the
  atomic set by `setAudioFocusLost`, `can_write_user_files` → false.
  `state()` computes from the session's model snapshot (not the UI), so
  it is right with no activity.
- [ ] **Step 4: Run** `cargo test -p fp-mobile --test it` (host) and
  `cargo ndk -P 29 -t arm64-v8a -t x86_64 build --release -p fp-mobile` — PASS.
- [ ] **Step 5: Commit** `feat(mobile): add the Android crate with session, IME bridge and safe area`.

### Task 7: The Kotlin shell

**Files:**
- Create: `mobile/android/` (Gradle wrapper 9.3.1, `settings.gradle.kts`,
  `build.gradle.kts`, `gradle.properties`, `app/build.gradle.kts`,
  `app/src/main/AndroidManifest.xml`, `res/values/themes.xml`, launcher
  icons from `packaging/icons`)
- Create: `app/src/main/java/com/sanchezfauste/fausteplayer/{MainActivity,PlaybackService,Native}.kt`

- [ ] **Step 1:** Project as the spike's, with the Global Constraints'
  ids and SDKs, AGP 8.13.2, Kotlin 2.2.21 (`jvmTarget` 17), dependencies
  `androidx.appcompat`, `androidx.core`, `androidx.games:games-activity:4.4.0`.
- [ ] **Step 2: `MainActivity : GameActivity`** — manifest:
  `launchMode="singleTask"`, every `configChanges` value of the spike;
  `onBackPressed` → `moveTaskToBack(true)`; asks `READ_MEDIA_AUDIO`
  (33+) / `READ_EXTERNAL_STORAGE` (≤32) and `POST_NOTIFICATIONS`;
  `ViewCompat.setOnApplyWindowInsetsListener` (system bars + display
  cutout) → `Native.setInsets`; a main-thread `Handler` every 500 ms
  reads `Native.state()`: bit 0 starts/stops `PlaybackService`, bit 1
  sets/clears `FLAG_KEEP_SCREEN_ON`.
- [ ] **Step 3: `PlaybackService`** — `foregroundServiceType="mediaPlayback"`;
  notification on channel `Native.notificationChannelName()`, text
  `Native.onAirText()` refreshed every 2 s, tap opens `MainActivity`, no
  actions; a `MediaSession` with no playback actions whose callback
  consumes every media button (MB4b); requests audio focus
  (`AUDIOFOCUS_GAIN`, `USAGE_MEDIA`, `CONTENT_TYPE_MUSIC`) on start and
  abandons it on stop; the focus listener only calls
  `Native.setAudioFocusLost(change < 0)` (MB4); `START_NOT_STICKY` (a
  killed process restarts cold, MB3).
- [ ] **Step 4: Verify:** `./gradlew assembleDebug` (after Task 8's
  script built the libraries) produces the APK.
- [ ] **Step 5: Commit** `feat(mobile): add the Android shell with the playout service`.

### Task 8: Build script and CI

**Files:**
- Create: `scripts/package/android.sh` (`<debug|release>`: `cargo ndk -P 29
  -t arm64-v8a -t x86_64 -o mobile/android/app/src/main/jniLibs build
  [--release] -p fp-mobile`, then `./gradlew assemble<Debug|Release>`)
- Create: `scripts/check-mobile-isolation.sh`
- Modify: `.github/workflows/ci.yml` (job `android`), `.gitignore`
  (`mobile/android/**/build`, `.gradle`, `jniLibs`, `local.properties`)

- [ ] **Step 1: Write** `check-mobile-isolation.sh`: fails if
  `cargo tree -p fp-app -e normal -i fp-mobile` succeeds (desktop pulls
  it), or if any `crates/*/Cargo.toml` other than `fp-mobile`'s lacks
  `workspace = true` under `[lints]`. Run it — PASS; temporarily add
  `fp-mobile` to `fp-app`'s dependencies — FAIL; revert.
- [ ] **Step 2: Job `android`** (ubuntu-latest): JDK 17
  (`actions/setup-java`), the runner's Android SDK/NDK
  (`ANDROID_NDK_HOME` from `$ANDROID_HOME/ndk/27.*` or
  `nttld/setup-ndk` pinned to r27), `rustup target add aarch64-linux-android x86_64-linux-android`,
  `cargo install cargo-ndk --locked` (cached), `scripts/check-mobile-isolation.sh`,
  `cargo test -p fp-mobile`, `scripts/package/android.sh debug`, upload
  the APK as an artifact.
- [ ] **Step 3: Verify** locally: `scripts/package/android.sh debug` — APK built.
- [ ] **Step 4: Commit** `ci: build the Android app and guard the unsafe exception`.

### Task 9: Docs and device checks

**Files:**
- Create: `docs/technical/mobile.md` (architecture, build, the `unsafe`
  exception, lifecycle, IME bridge, what is interim)
- Modify: `CLAUDE.md` (rule 6 names the `fp-mobile` exception and its
  limit; Commands: `scripts/package/android.sh`,
  `scripts/check-mobile-isolation.sh`; "Where things are": `crates/fp-mobile`,
  `mobile/android`), `docs/technical/README.md` (link),
  `README.md` (platform table: Android "in development, not released")
- Modify: spec MB9 — the usage is already media: AAudio maps an unset
  usage to `USAGE_MEDIA`/`CONTENT_TYPE_MUSIC` (AudioFlinger showed
  `Usg 1`, `CT 2` in the spike), so no cpal change is needed.

- [ ] **Step 1:** Write the docs above.
- [ ] **Step 2: Emulator checks** (release APK, `fauste-test` AVD), each
  recorded in the ledger: a route-less player sounds (an AAudio stream
  opens, logcat); rotation, Back, Home and relaunch keep UI and audio;
  "Don't keep activities" on, Home, relaunch: audio continues and the UI
  re-attaches, no ANR (Review Focus 1); 90 s screen off with playback:
  no underrun; force-stop and relaunch: nothing plays; typing in a
  playlist name works; the notification shows what is on air and has no
  buttons; `adb shell cmd media_session dispatch play` starts nothing.
- [ ] **Step 3:** `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` — PASS.
- [ ] **Step 4: Commit** `docs: document the Android foundation`.
