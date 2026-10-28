# Phone layout for live use — Design Spec

- **Date:** 2026-10-28.
- **Status:** Approved (2026-10-28).
- **Extends:** [the mobile spec](2026-10-25-mobile-design.md), §6 (touch
  layout), for **phones only**. It replaces the phone part of §6; the
  tablet and desktop layouts do not change. It changes no player rule.
- **Mockups** (conceptual, reviewed with the maintainer on 2026-10-28):
  `.superpowers/brainstorm/*/content/main-screen-v3.html`,
  `playlists.html` and `music-settings.html` (git-ignored; their content
  is described here in full). The look comes from the existing egui
  widgets and `theme.rs`, not from the mockups' HTML.
- **Numbering:** rules are `PL<n>`; each gets a test where a host test
  can observe it.

---

## 1. Intent

The maintainer runs **small live gigs straight from the phone**: the phone
is the playout, not a remote. What gets touched most is the **Play button
and the cartwall**; playlists are also changed on the fly (choose the
next track, move, add, remove). The first emulator and phone checks
(2026-10-28) showed that the phone layout of plans 5–6 shrinks the desktop
instead of serving that use:

- the "all players" list has display-only faders, the landscape columns
  too: the volume cannot be changed there;
- a tap on the waveform does not seek (the click needs a hover position a
  finger does not leave), and zoom exists only for the mouse wheel;
- "Open P1" is easy to miss (under each panel; off-screen at the end of
  the landscape chip row);
- the media browser is a window listing folders and file names;
- Settings is a small window with label and control side by side, so the
  combo boxes are cut.

A sound that goes wrong in front of an audience is the failure that
matters: an accidental touch on a seek bar or a fader is heard in the
room.

## 2. Goals, non-goals, anti-goals

**Goals**

- G1. One screen (Live) with every player's full transport and the
  cartwall, so a gig needs no tab change.
- G2. The cartwall large on its own screen.
- G3. Playlists editable on the fly on the phone, with the chosen
  player's full controls and waveform.
- G4. Adding music from a finger-sized, searchable browser that shows
  titles, not file names.
- G5. Settings usable on a phone.
- G6. Faders and seeking that work by touch, and a lock that guards them.

**Non-goals**

- Tablets (shortest side ≥ 600 dp) and the desktop: unchanged. A tablet
  layout may be revisited later.
- iOS.
- New player rules: everything is UI over existing commands.

**Anti-goals**

- A scroll, a swipe or a tab change that moves a level or the position of
  a player.
- A lock that blocks Play, Stop or the carts: they always work.
- Hiding a desktop control the gig needs (the full transport, the
  meters, Single/Cont, CUE).

## 3. Navigation (phone)

- PL1. A top bar with four tabs — **Live · Carts · Playlists · Music** —
  then the **lock** (§7) and a **⋮** menu with **Settings** and
  **About**. The clock stays in the bar while it fits *(builder's call
  where it goes when it does not)*.
- PL2. The phone's Back button goes back inside a screen (a Settings
  section to the section list, Music to its parent folder) before it
  reaches the app's own Back behaviour (mobile spec §5).
- PL3. The current "Players | Cartwall" bar, the All/P1…Pn chips, the
  "Open Pn" buttons and the landscape pairs with swipe are removed on a
  phone.

## 4. Live screen

### Player card

- PL4. Each player is a card, top to bottom:
  1. **its waveform, used as the seek bar** (a tap seeks, a pinch zooms,
     a one-finger drag pans a zoomed view; §8), placed first so it is
     away from the buttons;
  2. the header line: player name, status, Single/Cont, CUE;
  3. title and artist, the **remaining time** large, the next track;
  4. the transport: **Play/Next** large, then Previous, Pause, Stop,
     Restart, Fade stop and Stop after current as smaller buttons (the
     desktop's set and behaviour, including the fade sweeps);
  5. on the right of the transport: the **stereo meter** and the
     **fader** (§8).
- The cover is left out of the card *(builder's call to keep it if it
  fits)*.

### Arrangement

- PL5. **Portrait:** the player cards on top, a reduced cartwall below
  filling the rest. The cards area scrolls on its own when the players
  do not fit; the cartwall stays in place.
- PL6. **Landscape:** the player cards on the left, the reduced cartwall
  on the right.
- The share of the screen between cards and cartwall is the builder's,
  with two cards visible in portrait on a 6" phone (about 400×860 dp) as
  the target.

## 5. Carts screen

- PL7. The cartwall full screen: larger carts, the current page, page
  changes by the existing page controls and by a sideways swipe on empty
  space. Portrait turns a wide page as #173 does (2×8 instead of 8×2,
  reading order kept). Firing, the cart menu (long press) and editing a
  cart work as now.
- The Live screen's reduced cartwall shows the same page and fires the
  same carts; how many rows it shows is the builder's.

## 6. Playlists screen

- PL8. Top: player chips (P1, P2…) and the chosen player's playlist
  tabs (with + for a new playlist).
- PL9. **The track list on top, the chosen player's full card (§4) at the
  bottom**, with a larger waveform for preparing a track (seek, zoom,
  markers: PL11b). In landscape: the list on the left, the
  card on the right *(my call: the card at the bottom leaves too little
  height for the list)*.
- PL10. A row shows on air (red) and next (green) as on the desktop. A
  long press opens the row menu (the existing bottom sheet) with at least
  *Play next*, *Move up*, *Move down*, *Move to another player…*,
  *Remove*, plus the existing entries. Dragging a row by its handle (≡)
  moves it (existing long-press drag).
- PL10b. **Move to another player…** asks for the player and puts the
  entry in the playlist that player shows, right after its next entry
  (at the end when it has none). The entry on air cannot be moved
  (maintainer, 2026-10-28).
- PL11. The footer keeps *+ Add tracks*, which opens the Music screen with
  this player and playlist as the target, and the existing footer
  actions behind a ⋯ button.
- PL11b. **Markers by touch** (maintainer, 2026-10-28): only on the
  Playlists screen's waveform. A long press opens a menu at that point:
  *Set cue-in / intro / outro / mix here* and *Remove* for each marker
  set. Markers are not dragged by touch (a one-finger drag pans, a tap
  seeks). The Live cards' waveforms never edit markers.

## 7. Lock

- PL12. One lock button in the top bar. Locked, **every fader and every
  waveform** (seek, marker edits) ignores touches on every screen; zoom
  and pan of a waveform still work. Play, every transport button, the
  carts, the playlists and navigation are never locked.
- PL13. The button shows its state (closed/open padlock, highlighted when
  locked), and a locked fader or waveform looks muted.
- The lock is remembered across restarts *(my call: a crash or restart
  during a gig must not drop the protection)*.
- *(My calls.)* It is a new flag, not the layout lock (`layout.locked`,
  RL9), and it guards **touches only**: MIDI and the remote API are
  deliberate inputs and are not blocked, so it stays in the UI, not the
  reducer. The button is shown on phones only.

## 8. Touch behaviour (fixes)

- PL14. **Fader:** a drag that *starts on the fader* moves it, on every
  phone screen (and the tablet); a drag that starts elsewhere scrolls.
  This replaces the display-only faders of #173.
- PL15. **Seek:** a tap on a waveform seeks to the tapped point (the
  click position, not the hover position). Same rules as the desktop
  click (rule 3a on a stopped player).
- PL16. **Zoom:** a two-finger pinch on a waveform zooms around the
  fingers' centre, with the same limits as the wheel (`min_span`); the
  existing Full view button resets it. Applies on tablets too.

## 9. Music screen

- PL17. A tab, not a window: a search field (title, artist, album, file
  name), a Folders | Artists | Albums switch, a breadcrumb in Folders.
- PL18. A track row shows **title, artist and duration** (from MediaStore:
  `TITLE`, `ARTIST`, `ALBUM`, `DURATION`; the file name when a tag is
  missing), a check box (a tap on the row toggles it) and a 🎧 button that
  pre-listens on the CUE output with the existing CUE rules (the on-air
  confirmation included).
- PL19. A bar at the bottom while something is selected: the count, the
  target (player · playlist, changeable), and **Add at the end** / **Add
  after the next**. Adding keeps the selection order and clears it.
  *Add after the next* puts the tracks right after the player's next
  entry when the target playlist holds it; otherwise (another playlist,
  or no next) the button reads and acts as *Add at the end*
  (maintainer, 2026-10-28).
- PL19b. *(My call.)* On a phone the Music screen replaces the media
  browser window everywhere it is used today (mobile spec MB7): picking a
  cart's file or a folder opens it in a *pick one* mode (no target bar;
  a tap on a file, or *Choose this folder*, answers the request and
  returns to the screen that asked). Tablets keep the window.
- The browser stays read-only (mobile spec non-goal: no writing to music
  files).

## 10. Settings

- PL20. Full screen from ⋮ → Settings: the list of sections, then the
  chosen section; Back returns to the list, then closes.
- PL21. Inside a section every setting is stacked: the label (and its
  help) on top, the control at full width below. Changes apply at once,
  as now. This replaces #173's chips on a phone; the tablet keeps the
  desktop window.

## 11. Third-party licences

- PL22. The APK includes `THIRD-PARTY.html` (generated by
  `scripts/package/notices.sh` with the Android features) as an asset,
  and About's *Third-party licences* opens it (copied out of the APK to
  the app's cache and shown through the platform's open-URL path, or
  rendered in-app — builder's call). CI's Android job fails if the file
  is missing from a release build.

## 12. Closing the app (Android, phone and tablet)

- PL23. When **nothing sounds** (no player, cart or CUE pre-listen
  playing: `Native.state()`'s `SOUNDING` bit clear; a stopped or paused
  player with a track loaded does not count) and the operator
  closes the app — Back on a screen with nothing left to go back to
  (PL2), or swiping the app away from the app switcher — the app saves its
  state as on a desktop exit and the **process ends**. Today Back only
  moves the task to the background and the process lingers.
- PL24. While something sounds, closing behaves as now: Back and the app
  switcher put the app away, the playout and its notification continue
  (MB1). A swipe from the app switcher while something sounds keeps
  playing (the gig must not stop because of a stray swipe; stopping is
  done in the app; maintainer, 2026-10-28).
- The next launch starts as after any exit: nothing goes on air by itself
  (rule 10).

## 13. Tests and checks

- Host tests (`egui_kittest`, phone sizes in both orientations) for
  PL1–PL22 where observable: tabs, card contents, fader touched vs
  scrolled, tap-to-seek, pinch-to-zoom (multi-touch events), the lock,
  Settings drill-down, Music selection and adding.
- Emulator screenshots of each screen in both orientations at the end of
  each plan, compared with the mockups by the maintainer.
- Docs: `docs/user/android.md`, `docs/technical/mobile.md`, the mobile
  spec's §6 pointer to this spec; all new strings in the 11 locales.

## 14. Left to the builder

- Plan split (suggested: touch fixes + licences first as a small plan;
  then Live + Carts; then Playlists; then Music; then Settings).
- Exact sizes, spacing and where the clock goes.
- Whether the Live cards reuse `player::column` parts or a new card
  function.
- How the 🎧 pre-listen shows that it plays.
