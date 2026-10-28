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
  background. As on the desktop, nothing goes on air by itself after a
  start or a crash.
- **Back and closing.** Back first closes what is open (the music
  browser, Settings, About, an open player). With nothing left to close,
  Back puts the app away while something sounds (a player, a cart or a
  CUE pre-listen) and keeps playing; when nothing sounds, it saves and
  closes the app. Swiping the app away in the app switcher does the same:
  a swipe never stops what is playing, and the app closes once it falls
  silent.
- **Other apps' sound.** When another app takes the audio, Fauste Player
  goes on playing and says so in the status bar at the bottom.

## The screen

On a **phone** the top bar has four tabs, **Live**, **Carts**,
**Playlists** and **Music**, then the **padlock** and **⋮** (More: Settings and About).
The clock shows when there is room for it.

- *Live:* every player as a card, and the cartwall. Each card has the
  player's waveform on top (a tap seeks), its name, status, Single/Cont
  and CUE, the title with the time left and the next track, the transport
  (Play/Next, Previous, Pause, Stop, Restart, Fade stop, Stop after
  current), and the meter with the volume fader. Held upright the cards
  are on top, two at a time (scroll for more), and the cartwall below;
  turned sideways the cards are on the left and the cartwall on the
  right. A cart page too wide for the space shows in fewer columns, in
  the same order.
- *Carts:* the shown cart page, filling the screen. Swipe sideways to
  show the next or the previous page. Held upright, a page wider than
  tall is turned (8 × 2 shows as 2 × 8), in the same order.
- *Playlists:* the chips at the top (P1, P2…) choose the player; under
  them are its playlist tabs (+ makes a new one), the track list and,
  at the bottom (on the right when the phone is on its side), the
  player's card with a larger waveform for preparing a track. A long
  press on a row opens its menu, which also has **Move up**, **Move
  down** and **Move to another player** (the track goes right after that
  player's next track, in the playlist it shows; the track on air
  stays). Drag a row by its ≡ handle to move it. The footer has **Add**
  and, behind **⋯**, **Reset played**. A long press on the waveform sets
  a marker at that point (cue in, intro end, outro start, MIX, cue out)
  or removes one you set; markers are edited here only, not on Live.
- *Music:* the device's music (see [Music](#music)).

Back from another screen goes to Live; on Music it first goes up a
folder.

**The padlock** guards against stray touches during a show. Locked (the
padlock closed and lit), the volume faders only show the level and the
waveforms neither seek nor edit markers; they look dimmed. A pinch and a
drag still zoom and move a waveform, and Play, the transport, the carts
and the playlists always work. The lock stays on after a restart and is
saved with the session, so loading another session takes that session's
lock; the remote API and MIDI are not locked.

A drag that starts on a volume fader moves it; one that starts anywhere
else scrolls, so scrolling never changes a level on air. A tap on a
waveform seeks there, a two-finger pinch on it zooms around your fingers,
a one-finger drag moves a zoomed view and **Full view** shows the whole
track again. In a playlist a finger scrolls; to move a track, use its ≡
handle or the row menu.

On a **tablet** the layout is the desktop's (see [Layout](layout.md)),
with targets made easier to hit.

A long press opens what a right click opens on the desktop: the row menu,
the header menus, a cart's menu (**Edit…** chooses an empty cart's
file). On a phone, Settings shows its sections as a row of chips above
the section.

About → **Third-party licences** opens the notices of the libraries the
app is built with in a browser or HTML viewer.

## Music

On a **phone** the **Music** tab shows the device's audio files by
**Folders** (with the path above the list; tap a part of it to go back
up), **Artists** or **Albums**, and a search over title, artist, album
and file name. Each row shows the title, the artist and the duration; tap
rows to tick them. While something is ticked, a bar at the bottom shows
how many, the player and playlist they go to (tap it to choose another),
**Add at the end** and, when that player's next track is in that
playlist, **Add after the next**. The tracks go in the order you ticked
them.

**Add** in a playlist footer, a cart's file and Settings → Playlists →
Music folder open the Music tab on a phone (on a tablet, the **music
browser** window, by folder). Tick files and tap **Add**, tap one file for
a cart, or **Use this folder**; **Cancel** or Back goes back to where you
were. Copy music to the device (for example to its *Music* folder) to see it
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
