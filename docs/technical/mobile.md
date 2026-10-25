# Mobile (Android)

The Android app is in development and not released. The binding design is
the mobile spec,
[`2026-10-25-mobile-design.md`](../superpowers/specs/2026-10-25-mobile-design.md);
this page describes what is built. iOS comes later.

## Architecture

The phone runs the same Rust session as the desktop: the store, the
engine on cpal's AAudio host, the conductor, the analysis and the
services thread, started by `fp_app::session::AppSession`, and the same
egui interface (`fp_app::ui::shell::Shell`). Three pieces are mobile-only:

| Piece | Where | What it does |
|---|---|---|
| `fp-mobile` | `crates/fp-mobile` | The Android library (`libfp_mobile.so`): `android_main`, the activity glue, the IME bridge, the safe area, the Android `Platform` and the JNI calls |
| The Kotlin shell | `mobile/android` | A Gradle project: `MainActivity` (a `GameActivity`), `PlaybackService` (the foreground service) and `Native` (the JNI declarations) |
| The platform seam | `crates/fp-app/src/platform.rs` | The `Platform` trait the interface uses for pickers, links, the audio focus and whether user files may be written; `DesktopPlatform` on the desktop, `AndroidPlatform` in `fp-mobile` |

`fp-app` builds for Android without its desktop-only parts: MIDI
(`fp-control`), the remote API (`fp-remote`), the native dialogs (`rfd`),
the system opener (`open`), the command line, the instance lock and the
window-system choice are `cfg(not(target_os = "android"))` dependencies and
modules, and Settings has no MIDI or Remote section there. Until the shell
gives its platform, `AppUi` uses `NoPlatform` on Android.

`fp_model::playout_active` (MB1) is the one model rule added for mobile:
a player with a track loaded (current or next, even stopped), playing,
paused or pre-listening, or a cart playing or pre-listening.

## The activity and the session

- **One session per process (MB2).** The first `android_main` starts the
  `AppSession` and keeps it in a `OnceLock`; every later activity builds a
  new `AppUi` on it. When the activity is destroyed, eframe returns and
  `android_main` ends without touching the session: the audio goes on.
  winit 0.30.13 did neither on its own: it ignored the activity's
  destruction (android-activity's `onDestroy` waits for `android_main` to
  return, so the main thread hung) and refused the next activity's event
  loop. The vendored copy in `vendor/winit` ends the loop on destroy and
  allows a new one on Android (see its `VENDORED.md`).
- **No recreation.** The activity declares every `configChanges` value and
  `launchMode="singleTask"`; Back moves the task to the background. A
  recreated activity used to stall the interface while the audio played
  (spec §11).
- **The foreground service (MB1).** `MainActivity` polls `Native.state()`
  every 500 ms while it is resumed: bit 0 (`PLAYOUT_ACTIVE`) starts
  `PlaybackService`, bit 1 (`KEEP_SCREEN_ON`, MB8, `ui.keep_screen_on`)
  sets `FLAG_KEEP_SCREEN_ON`. The service refreshes its notification every
  2 s with `Native.onAirText()` and stops itself when playout ends, so it
  stops with no activity too. It is `START_NOT_STICKY`: a killed process
  starts cold and plays nothing (MB3, rule 10).
- **Audio focus (MB4).** The service holds `AUDIOFOCUS_GAIN` with media
  attributes and "pause when ducked" only while something sounds
  (`SOUNDING`, bit 2 of `Native.state()`: a player playing, a cart, a
  CUE), so a loaded track alone does not pause the operator's other apps
  and the system never ducks the playout. A loss only calls `Native.setAudioFocusLost`, which the
  interface shows as a status-bar alert (`alert-audio-focus-lost`).
- **No media controls (MB4b).** The media session has no playback actions
  and its callback swallows every media button; the notification has no
  buttons.
- **The default output (MB5).** cpal's AAudio host names its default
  device `aaudio:-1`, which `device_by_id` cannot find; `fp-backends`
  opens the default device by that id, so a player without a route sounds
  on the phone's output instead of the Null device.
- **Stream usage (MB9).** AAudio maps cpal's unset usage to
  `USAGE_MEDIA` / `CONTENT_TYPE_MUSIC` (AudioFlinger reports `Usg 1`,
  `CT 2`), so no cpal change is needed.

## Keyboard and safe area

winit 0.30 drops GameActivity's text events. `fp_mobile::ime::ImeBridge`
reads the keyboard's text (`AndroidApp::text_input_state`) in
`raw_input_hook` and turns the difference with the last text into
Backspace presses and one text event. The keyboard knows neither the
field's text nor egui's cursor, so when a field takes the focus, or a tap
may have moved the cursor, the keyboard's text is cleared
(`set_text_input_state`) and only what is typed from there is bridged.

winit does not report insets either. `MainActivity` listens to the window
insets (system bars and display cutout) and calls `Native.setInsets` in
physical pixels; `fp_mobile::insets::safe_area` turns them into the
rectangle the interface draws in (MB10).

## The `unsafe` exception

`fp-mobile` is the only crate allowed `unsafe` (spec §8), for
`#[unsafe(no_mangle)]` on `android_main` and the five JNI entry points. Its
`Cargo.toml` sets `unsafe_code = "deny"` and copies the other workspace
lints; each site opts in with `#[allow(unsafe_code)]` and a `// SAFETY:`
comment. Its Android modules are `#[cfg(target_os = "android")]`; its pure
modules (`ime`, `insets`, `shell_state`) build and test on the host.
`scripts/check-mobile-isolation.sh` (run by CI) fails when another crate
declares a dependency on `fp-mobile`, when another crate leaves the
workspace lints, or when `fp-mobile` stops denying `unsafe_code`.

## Build

```sh
rustup target add aarch64-linux-android x86_64-linux-android
cargo install cargo-ndk
scripts/package/android.sh debug     # or release
```

The script finds the SDK (`ANDROID_HOME`, `mobile/android/local.properties`
or `~/Android/Sdk`) and an NDK r27, builds `libfp_mobile.so` for
`arm64-v8a` and `x86_64` with `cargo ndk -P 29` (API 29: `midir`'s
`libamidi` and AAudio) into `mobile/android/app/src/main/jniLibs`, and
assembles the APK with Gradle (AGP 8.13.2, Kotlin 2.2.21, Gradle 9.3.1, a
JDK 17 or newer). The APK is in
`mobile/android/app/build/outputs/apk/<profile>/`. `minSdk` is 29,
`targetSdk` 36, the application id `com.sanchezfauste.FaustePlayer`.

Judge dropouts and latency on release builds only: a debug build
underruns at start. CI's `android` job runs the isolation check, the
`fp-mobile` tests and the debug build, and uploads the APK.

## Interim parts

These hold until the plans that replace them:

- the pickers list the audio files directly in `/sdcard/Music`
  (`AndroidPlatform`), until the media browser (MB7);
- links in the About window are not opened (`open_url` returns an error);
- the outputs keep the desktop Settings; output choice and device checks
  (MB5, MB6) come with their own plan;
- the layout is the desktop one; the touch layout (spec §6) has its plan;
- release builds are signed with the debug key, `versionCode` is 1 and the
  notification uses a system icon, until a release plan.
