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
  allows a new one on Android (see its `VENDORED.md`). eframe 0.37.0 then
  panicked on the way out, because it drops the window on suspend and still
  expected it when saving; `vendor/eframe` takes it as optional.
- **No recreation.** The activity declares every `configChanges` value and
  `launchMode="singleTask"`; Back (the key, taken in `onKeyDown` before
  GameActivity hands it to the native side, and the gesture, through the
  back dispatcher) moves the task to the background. A
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
- **Outputs (MB5, MB6).** Settings → Audio outputs lists what Android
  reports as outputs, and each player, the cartwall and their Cue routes
  pick from it as on the desktop. Android gives a device a new id each
  time it connects, so the AAudio host lists its outputs under stable ids
  (`fp_backends::aaudio_ids`): the interface and the type as the module's
  own tokens and cpal's name, percent-escaped and joined with `/`
  (`aaudio:bluetooth/unknown/JBL Flip`). A Bluetooth device with an
  address adds it (`…@XX:XX:XX:XX:AB:CD`; Android hides part of it but
  keeps it per device), so two identical speakers keep their routes;
  other outputs that share all three get `#2`, `#3`… in list order (the
  built-in speaker and its "speaker safe" path) and ` 2`, ` 3`… in their
  labels. A test pins whole ids, because saved routes hold them: a cpal
  upgrade that renames a type fails it. Renaming a Bluetooth device in
  Android's settings changes its id, so its routes must be picked again,
  and so must routes saved with cpal's raw `aaudio:<n>` ids before these
  ids existed (no release had them). A route's device
  is found by that id when its stream opens, so a Bluetooth speaker that
  goes away and comes back is the same output; while it is away the route
  behaves like a missing device on the desktop (nothing re-routes,
  nothing starts when it returns). Labels are cpal's names, plus the
  interface when cpal has no type for the device (Bluetooth A2DP: "JBL
  Flip (Bluetooth)"). Cue routes and the on-air confirmation are the
  desktop's; whether two outputs can sound at once is up to Android, and
  a refused stream is reported like any device failure.
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

## Touch layout (§6)

`fp-mobile` turns on touch mode (`AppUi::with_touch`). It widens egui's
interaction radius and the scroll bars, so the desktop's tiles are easier
to hit, and lays the screen out by its size (`fp_app::ui::touch`): a
screen whose shortest side is 600 points (Android's dp) or more is a
tablet and keeps the desktop layout; a smaller one is a phone.

A phone's top bar keeps the clock, Settings and About, and its panel tiles
give way to Players | Cartwall (`PhoneView`, view state only: nothing a
view hides stops or changes playout):

- *Players, portrait:* every player as a compact panel (`player::compact`:
  the header, the titles with the countdown, the transport, the meter and
  the level, and a progress bar), each with an "Open P1" tile. An open player
  is the desktop column with its playlist at the full width, under chips
  (All, P1…Pn).
- *Players, landscape:* two desktop columns side by side, under chips for
  the pairs and for opening a shown player; a sideways swipe of more than
  80 points that no widget took (a fader, a row) shows the next or
  previous pair.
- *Cartwall:* the cartwall filling the area.

The open player survives rotation and closes when its player goes away
(`PhoneView::settle`). The top bar keeps only icons beside the screen
tiles, which shrink to fit. A finger drag scrolls: in touch mode table
rows and column headers do not drag (the row menu moves entries), and a
compact panel's fader only shows the level (it is adjusted in the open
player), so scrolling never moves a level on air. The cartwall screen
never folds. Context menus open by long press (egui reports it as a
secondary click). Still to come: the row menu as a bottom sheet and
long-press tooltips.

## Media access (MB7)

Android has no file dialog that gives back paths, so the pickers go
through the media browser (`fp_app::ui::media_browser`). `MediaFiles`
(Kotlin) lists the device's audio files from MediaStore (the `DATA`
column, every audio file, not only music, since carts are often jingles
and effects) on a background thread after the permission is answered, on
resume and whenever MediaStore changes (refreshes asked while a query
waits to start join it, since a scan notifies once per file), and hands
the paths to `Native.setMediaFiles`. The decoders open those paths as on the desktop;
nothing is copied, and music is only read (spec §2). Android 10 needs
`requestLegacyExternalStorage` for that; from Android 11 the media
permission is enough.

`AndroidPlatform`'s pickers call `MediaBrowser::browse` on the helper
thread they already run on, and it waits for the operator: adding tracks
ticks files (`AudioFiles`), a cart takes one file with the cart
extensions (`OneFile`) and Settings takes the music folder (`Folder`).
`AppUi` draws the browser window while a request is pending. The handle
is process-wide (`fp_mobile::platform::MEDIA`); a request still pending
when its activity is destroyed is answered with nothing when the next one
starts (its answer would reach no one), and a newer request answers the
older one with nothing, so no helper thread hangs. The browser lists only
audio: pickers for anything else (a cover image, a playlist or a cart page
to import) return nothing on Android for now. Folders
start closed and are named without the part they all share; a file that
leaves the list leaves the selection; without permission or music the
window says how to get some.

## The `unsafe` exception

`fp-mobile` is the only crate allowed `unsafe` (spec §8), for
`#[unsafe(no_mangle)]` on `android_main` and the six JNI entry points, and
for the one call into the activity (`jni::open_url`: the Java VM and the
activity reference android-activity hands out, kept in `ACTIVITY` while
the activity runs and locked through the call). Its
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

- the outputs page is the desktop's (the touch layout reworks it);
