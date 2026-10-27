# Android

Fauste Player also runs on Android phones and tablets (Android 10 or
newer), on the same playout core as the desktop: the players, playlists,
the cartwall, markers, loudness normalization and every play rule work
the same way.

## Install

A release that carries the app has `fauste-player-<version>-android.apk`
on its **Releases** page (with its `.sha256`). Open it on the phone and
allow your browser or file manager to install apps when Android asks.
To update, install the newer APK the same way: your playlists and
settings stay. The app is not in Google Play yet.

## First start

- **Music access.** Allow access to music (and notifications, on Android
  13 and newer). Without it the music browser stays empty.
- **Playout keeps going.** While a player or a cart plays, a notification
  stays up and the audio goes on with the screen off or the app in the
  background. Back and the app switcher put the app away without stopping
  anything. As on the desktop, nothing goes on air by itself after a
  start or a crash.
- **Other apps' sound.** When another app takes the audio, Fauste Player
  goes on playing and says so at the top of the screen.

## The screen

On a **phone** the top bar switches between **Players** and
**Cartwall**:

- *Players:* every player as a compact panel with its transport, its
  countdown, its level and a progress bar. **Open P1** shows that player
  with its playlist; the chips above it switch players, and **All** goes
  back. Turned sideways, two players show side by side: the chips or a
  sideways swipe show the next two.
- *Cartwall:* the shown cart page, filling the screen.

The volume fader moves only in an open player, so scrolling the list
never changes a level on air. In a playlist a finger scrolls; to move a
track, use the row menu.

On a **tablet** the layout is the desktop's (see [Layout](layout.md)),
with targets made easier to hit.

A long press opens what a right click opens on the desktop: the row menu,
the header menus.

## Music

**Add** in a playlist footer, a cart's file and Settings → Playlists →
Music folder open the **music browser**: the device's audio files by
folder. Tick files to add them, tap one for a cart, or **Use this folder**.
Copy music to the device (for example to its *Music* folder) to see it
there. Fauste Player only reads your music: tags and covers are not
written, and playlists and cart pages are not imported or exported on
Android yet.

## Outputs

Settings → Audio outputs lists the phone's outputs by kind: the speaker,
wired headphones, a USB interface, a Bluetooth speaker ("JBL Flip
(Bluetooth)"). Each player, the cartwall and their CUE pick one, as on the
desktop. A Bluetooth speaker that is turned off and on again is found
again by its route; while it is away its players behave as with a missing
device on the desktop. Whether two outputs can sound at the same time
depends on the phone.

## Not on Android

MIDI control surfaces, remote control, tag and cover editing, and
playlist and cart page files are desktop features for now. A keyboard
plugged into a tablet works as on the desktop (see [Keyboard](keyboard.md)).
