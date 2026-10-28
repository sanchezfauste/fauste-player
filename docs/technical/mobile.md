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
  `launchMode="singleTask"`. A
  recreated activity used to stall the interface while the audio played
  (spec §11).
- **Back and closing (phone layout spec PL2, PL23, PL24).** Back (the
  key, taken in `onKeyDown` before GameActivity hands it to the native
  side, and the gesture, through the back dispatcher) calls `Native.back`,
  which sets a flag and wakes the interface through the stored
  `egui::Context`. The next frame's `MobileApp::logic` calls
  `Shell::back` → `AppUi::back` (closes an open menu, then cancels the
  media browser's request, then closes the topmost window or question as
  Esc or Cancel would, then a phone's open player); with
  nothing to go back to, `Platform::leave(sounding)` calls
  `MainActivity.leave(close)` on a helper thread. While something sounds
  (`shell_state::sounding`, the `SOUNDING` bit: `playout_active` is also
  true for a stopped player with a track loaded) the task moves to the
  background; otherwise `finishAndRemoveTask()`. `onDestroy`, when
  finishing with nothing sounding (Back, or a swipe from the app
  switcher), runs `Native.quitAndEnd` (once per process): on a helper
  thread `Native.quit`
  (`AppSession::stop_services`, the services' final save; the engine is
  left running) and then `Process.killProcess`. A swipe while something
  sounds leaves the service holding the process; its refresh ends the app
  the same way once nothing sounds and no activity exists
  (`MainActivity.alive`).
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

A phone's top bar (phone layout spec PL1) has the tabs Live | Carts |
Playlists | Music (`touch::screen_tiles`, shrinking to fit), the lock tile
(`touch::lock_tile`), a More tile with Settings and About
(`touch::more_menu`, an `egui::Popup::menu`) and the clock while the bar
is at least `CLOCK_MIN_WIDTH` (400 points) wide. `PhoneView` is view state
only (the screen, the Playlists screen's player): nothing a view hides
stops or changes playout.

- *Live* (`touch::live`, PL4–PL6): a card per player (`player::card`: the
  waveform `CARD_WAVE_HEIGHT` tall with no marker editing, the header and
  `top_block`) in a vertical scroll area, and `cartwall::strip` filling
  the rest. Portrait puts the cards on top, as tall as they were in the
  last frame up to `LIVE_CARDS_SHARE` (0.66) of the height; landscape puts
  them on the left, `LIVE_CARDS_SHARE` of the width. On a phone,
  `cartwall::grid` lays a page out in fewer columns, in the same reading
  order, when its columns would be narrower than `PHONE_MIN_CART_WIDTH`.
- *Carts* (`touch::carts`, PL7): the cartwall filling the area. A
  sideways swipe of more than 80 points that started on this layer
  (`page_swipe`; nothing there takes a drag) sends `ShowCartPage` for the
  next or the previous page, stopping at the ends. On an upright phone, a
  page with more columns than rows is laid out turned (columns and rows
  swapped, carts in the same reading order).
- *Playlists* (`touch::playlists`, `player::playlists_screen`, PL8–PL11b):
  chips per player, then that player's playlist tabs (with +), track
  table and footer, and its card under them (on the right when the phone
  is on its side) with a 72-point waveform where markers are edited. The
  card's height is remembered from the last frame, up to 55% of an
  upright screen; a card taller than a phone on its side scrolls. On a
  phone (`view_state.phone_active`) the row menu adds *Move up*, *Move
  down* and *Move to another player* (`table::phone_moves`; the target is
  `fp_model::after_next`: the playlist that player shows, right after its
  next entry there, else its end; never the entry on air), each row ends
  in a ≡ handle (`table::drag_handle`) that starts the desktop's
  `DragEntry` drag and drop, the footer keeps *Add* and puts *Reset
  played* behind a ⋯ menu (`footer_more`), and the waveform's menu (a long
  press) offers *Remove* for each marker set by hand
  (`SetMarker { secs: None }`, which gives it back to the analysis).
- *Music* (`music::screen`, PL17–PL19b): the pure part
  (`music::listing`, `MusicView`) lists the device's files by folder
  (from `top_folder`, the deepest folder holding every file), by artist or
  by album (untagged files in a group of their own), or every file whose
  title, artist, album or file name holds the search. The selection is a
  list in tap order; the bar adds it with `InsertPaths` at the end or at
  `fp_model::after_next`'s index when the target playlist is the one the
  player shows and its next is in it. Back (`AppUi::back`) goes up a
  folder or out of an artist or album before leaving the screen. The 🎧
  pre-listen of PL18 is not built: a file outside a playlist has no CUE
  source yet.

**The lock** (PL12, PL13) is `Layout::touch_locked`, saved with the
session and set by `Command::SetTouchLocked` (the layout lock does not
refuse it, and a layout reset keeps it). `touch::locked` is true only on a
phone: the card's and the column's faders are then disabled, and the
waveforms (`WaveInput::locked`, `WavePanelInput::locked`, the CUE
window's too) do not seek or edit markers and are drawn dimmed, while
zoom, pinch and pan go on. It is UI only: the remote API and MIDI are not
locked.

The Playlists screen's player survives rotation and falls back to the
first when it goes away (`PhoneView::settle`). Back from another screen
goes to Live (`PhoneView::back`). A finger drag scrolls: in touch mode table
rows and column headers do not drag (the row menu and, on a phone, the
row's handle move entries). A drag
that starts on a fader moves it, on every screen (PL14); one that starts beside it scrolls, so scrolling never
moves a level on air. A waveform seeks on a tap at the press position
(`interact_pointer_pos`: a finger leaves no hover position on the frame
it lifts, PL15) and zooms on a two-finger pinch around the fingers'
centre (`multi_touch`, with the wheel's `min_span`; the one-finger pan is
ignored meanwhile, and a pinch never seeks, PL16). The phone's cartwall
never folds. The status bar leaves out the desktop's shortcut hints
(keys, double and right clicks), and in touch mode a Settings window
narrower than 600 points shows its sections as a row of chips above the section instead of
the side list. Context menus open by long press (egui reports it as a
secondary click). Still to come: the row menu as a bottom sheet and
long-press tooltips.

## Media access (MB7)

Android has no file dialog that gives back paths, so the pickers go
through the media browser (`fp_app::ui::media_browser`). `MediaFiles`
(Kotlin) lists the device's audio files from MediaStore (the `DATA`
column with `TITLE`, `ARTIST`, `ALBUM` and `DURATION`, joined with NUL
into one record per file and parsed by `MediaFile::from_record`; every audio file, not only music, since carts are often jingles
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
`AppUi` draws the browser window while a request is pending; on a phone
`music::follow_request` turns to the Music screen instead, which answers
it (a tap for one file, *Use this folder*, *Add* for the ticked files)
and returns to the earlier screen once the request is gone (PL19b). The handle
is process-wide (`fp_mobile::platform::MEDIA`); a request still pending
when its activity is destroyed is answered with nothing when the next one
starts (its answer would reach no one), and a newer request answers the
older one with nothing, so no helper thread hangs. A list that arrives
after an empty one means the permission was just granted:
`Native.setMediaFiles` then asks the services thread to analyse every
unreadable track again (`ServiceRequest::RecheckUnreadable`), since
tracks loaded before it failed and their files did not change, so the
missing-file recheck would never look at them again. A list that came before the session
started asks for it when the session starts. The browser lists only
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

A release build also makes the AAB a store needs. It is signed with the
key the four `FAUSTE_ANDROID_KEYSTORE`, `FAUSTE_ANDROID_KEYSTORE_PASSWORD`,
`FAUSTE_ANDROID_KEY_ALIAS` and `FAUSTE_ANDROID_KEY_PASSWORD` variables
name, and with the debug key without them; some of them without the
others fail the build. The `versionCode` comes from
the workspace version (`android.sh --self-test` checks the formula).
`release-build.yml`'s `android` job publishes the APK and AAB with a
release once the key is a secret (see the release process).

A release build ships the third-party notices (PL22):
`NOTICES_MANIFEST=crates/fp-mobile/Cargo.toml scripts/package/notices.sh`
writes them to `app/src/main/assets/licenses/THIRD-PARTY.html`
(git-ignored; a debug build removes it, so its About says the file is
missing), and the script fails when the APK lacks the asset. It needs
`cargo-about` (or downloads the pinned one, as `notices.sh` does);
`release-build.yml` installs it. `MainActivity.onCreate` copies the asset
to `filesDir/licenses/` before the native side starts; `fp-mobile` passes
that path to `AppUi::with_notices` when the file is there, and
`MainActivity.openUrl` hands a path (a value starting with `/`) to a
viewer through `androidx.core.content.FileProvider` (authority
`<applicationId>.files`, `res/xml/file_paths.xml`), with read permission.

Judge dropouts and latency on release builds only: a debug build
underruns at start. A debug build also aborted at start on the emulator
(2026-10-28) in `android-activity` 0.6.1: a `slice::from_raw_parts`
precondition check in its text-input callback, which only debug builds
make. Check on a device or emulator with a release build. CI's `android` job runs the isolation check, the
`fp-mobile` tests and the debug build, and uploads the APK.

## Interim parts

These hold until the plans that replace them:

- the outputs page is the desktop's (the touch layout reworks it);
