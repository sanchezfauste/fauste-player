# Library

The library is every track Fauste Player knows: each file you have added to a
playlist, a cart or the library itself is one track, with its tags, markers
and loudness settings. The library window lists them, groups them in
*categories*, finds them by text and sends them to the playlists and the
cartwall. It is a window on the desktop and on a tablet; a phone shows the
library in its **Library** tab instead (see [On a phone](#on-a-phone)).

![The library window: the pre-listen strip with the selected track's waveform, meter and listening fader, the category tree with Music (its subcategories Pop and Evening) selected and Markers to review at the end, and the table of its tracks with their category and kind; Coastline carries the amber markers-to-review flag](../images/guide/library.png)

## Opening the window

Click the **Library** tile in the top bar. It is lit while the window is open;
click it again, or the window's close button, to close it. The window floats
over the players and the cartwall and does not block them: nothing in it stops
the audio or the shortcuts of the main window.

Move it by its title bar and resize it from its edges and corners. The title
bar has three buttons: **Minimize** (only the title bar is left, where the
window was, or at the top of the main area when the window is maximized;
click it again to expand), **Maximize** (the window fills the
main area; **Restore** brings it back) and **Close**. A window that would
not fit, after the main window shrinks or moves to a smaller screen, is
brought back inside the main area.

The window's position and size, minimized or maximized state, the width of the
tree, the columns, the sort order and the selected category are saved with the
session and come back at the next start. Whether the window is open is saved
too, but nothing plays by itself because of it. The text in the search field
is not saved.

## Categories

The tree on the left groups the tracks. At the top, **All tracks**; then your
categories, which can hold subcategories; then **Uncategorised**, the
tracks that belong to no category; and, only while some track needs it,
**Markers to review** (see [When a file changes](#when-a-file-changes)). Each item shows its colour and a count: a
category counts its own tracks and those of its subcategories. A track
belongs to at most one category.

- **Click** an item to list its tracks in the table. A category lists its
  subcategories' tracks too.
- The **arrow** before a category opens or closes its subcategories. Which
  ones are open is saved.
- **New category** (＋ above the tree) asks for a name, a colour and a *kind*.
  A new category takes its colour and kind from the category it is created
  in; the kind is what a track takes when it joins the category, and changing
  it later leaves the tracks already in it as they are. A name cannot be empty
  and cannot repeat a sibling's name; categories nest only up to a limit.
- **Right-click** a category (or long-press on a touch screen) for **New
  subcategory…**, **Rename…**, **Colour**, **Kind** and **Delete…**. On the
  desktop the menu also has **Watch folder…** (**Change watched folder…** and
  **Stop watching folder** once it watches one) and **Remove missing
  tracks…**; see [Watched folders](#watched-folders) and
  [Removing tracks](#removing-tracks).
- **Drag** a category to move it: onto another category to nest it, or into
  the gap between two categories to place it there. A category cannot go into
  itself. On a touch screen the tree only takes taps and scrolls under a
  finger; a category cannot be dragged there.

**Delete…** names what goes with the category: its subcategories and every
track in them are removed from the library. The files are never deleted. If any
of those tracks is in use, a second question says how many playlist entries and
carts use them: **Remove anyway** removes the entries from their playlists and
empties the carts. Nothing is removed while any of them is on air; a notice
says what plays.

The same tree is in **Settings → Library**, to create, rename, recolour,
change the kind of, move and delete categories without opening the window (it
has no *All tracks*, *Uncategorised* or selection).

![Settings, Library: the category tree with Music, its subcategories Pop and Evening, and Jingles, each with its count](../images/guide/settings-library.png)

## Watched folders

On the desktop a category can watch one folder. Right-click it and choose
**Watch folder…**, then pick the folder. The category shows a ⟳ mark before
its count; hover it to read the folder.

- Every audio file in the folder and its subfolders joins the category.
  Subfolders do not create subcategories.
- A track that is already in another category stays there. A track that is
  in no category joins this one. Setting, changing or stopping a watch never
  moves a track: the tracks of a folder that is no longer watched stay in their
  category.
- While the application runs, the folder is followed: a new file joins once it
  has been quiet for a moment (`library.watch_debounce_ms`, 1 second by
  default, so a file that is still being copied is not read half-written). The
  folder is also scanned every time the application starts, to catch what
  changed while it was closed.
- A file that vanishes from the folder (deleted, moved or renamed) leaves its
  track in the library, marked missing, with its markers. If the file comes
  back, the track is normal again by itself. A renamed file joins as a new
  track under its new name.
- Watched folders can be nested. A file under two of them goes to the category
  of the innermost one; stopping one of them leaves the other followed. The
  same folder cannot be watched by two categories: the application says
  which category already has it.
- Nothing a watched folder does loads or plays a track.

If the folder cannot be followed, the ⟳ mark changes colour and its
tooltip says why: *Watched folder not found: …. It is looked for again.* (for
example a drive that is not mounted), *Watched folder cannot be read: …. It is
tried again.*, or *Changes in … are not followed; it is scanned at start and
tried again.* (the system refused to watch it). A folder stored as a path that
is not complete (edited by hand) is not watched at all, and the mark says so.

Every 30 seconds (`tuning.missing_recheck_ms`) the application checks that
each watched folder is still there, so a drive unplugged while it runs is
reported too. A folder in trouble is tried again at that pace and scanned as
soon as it is back. A folder that is not there, cannot be read, or is empty
while the library still has tracks from it (a drive or share that is not
mounted often looks like an empty folder) counts as not found. The watch then
marks none of its tracks missing and removes nothing, also when the folder
sits inside another watched folder.

If you empty a watched folder on purpose, the warning stays until files are
back in it or you stop watching it, and its tracks stay in the library
without being marked missing. To clear them, select them in the table and
choose **Remove from library…**, or choose **Re-analyse** on them: the
analysis marks the tracks whose file is gone missing, and **Remove missing
tracks…** then removes them (if the folder is still watched, its warning
clears at the next check).

One exception to the rule above: a track that has never been analysed is
still marked missing when the analysis cannot find its file, wherever it is.

A subfolder that cannot be read is skipped and the log says so; folder links
(symbolic links to folders) are not followed.

New files go to the analysis behind anything a player or a cart needs, so a
folder with thousands of files takes a while; the analysis progress shows it.

The same menu and mark are in **Settings → Library**. They are not on Android,
which does not watch folders: there a category can take its files from a
device folder that you re-read when you choose (see [Android →
Library](android.md#library)).

## The track table

The table lists the tracks of the selected item, in the same columns as the
playlist tables (the ones chosen in **Settings → Playlists → Table columns**,
without the position `#`) followed by **Category** and **Kind**. A narrow cell
at the left flags a track whose file is missing or cannot be read. A track
waiting for its analysis is flagged at the right of its **Title** cell (so
only when the Title column is shown).

- Drag a header separator to change a column's width; click a header to sort
  by it and again to reverse. Widths and sort order are saved.
- **Click** selects a row, **Ctrl** (**Cmd** on macOS) **click** adds or
  removes one, **Shift click** selects the rows between the last click and
  this one. The selection keeps only rows that are listed: a row that leaves
  the table, because of a search, another category or a removal, leaves the
  selection. Every action applies to the selected rows in table order.
- **Add files** (above the table) picks audio files and adds them to the
  library, in the selected category (uncategorised when *All tracks* or
  *Uncategorised* is selected).

On a touch screen the rows only take taps and scroll under a finger; a
long-press opens the menu, and tracks are sent with the menus (below), not by
dragging.

## Search

The field above the table finds tracks by text. It looks in the title, artist,
album, album artist, composer, genre, comment and file name, ignoring case
and accents, and a track is listed when it contains every word you typed (in
any order and in any of those fields). It searches the selected item;
switch on **Whole library** to search every track whatever the selection. The
scope is saved; the text is not. Clear the field to see the whole item again.

## The row menu

Right-click a row. If the row is not selected, it becomes the selection;
otherwise the menu acts on every selected row. The first line shows the
track's title when the menu is for one track, and how many tracks are
selected when it is for several.

| Item | Action |
|---|---|
| Add to ▸ | A playlist: the tracks are added at its end |
| Add to cart ▸ | A cart of the cartwall's shown page: the tracks fill the chosen cart and the next empty carts of the page, in grid order. The tracks that do not fit are left out and a notice counts them |
| Move to category ▸ | A category, or Uncategorised |
| Kind ▸ | Music, Jingle, Effect, Ad, Voice or Hourly ID |
| Edit tags… | The tag editor (one track) |
| Open in external editor | Start your audio editor with the track's file (one track; see [When a file changes](#when-a-file-changes)) |
| Mark as reviewed | Clear the *markers need review* flag (shown when a selected track has it) |
| Loudness… | The loudness popup (one track) |
| Re-analyse | Analyse the tracks again |
| Reset markers to automatic | Drop the manual markers |
| Show file location | Open the file's folder (one track) |
| Remove from library… | Remove the tracks from the library, after asking (below) |

## When a file changes

Preparing material often means trimming or fixing a file in an audio editor.
Fauste Player helps with both halves.

**Open in external editor** (the row menu of the library and of playlist rows)
starts the program you chose in [Settings → Library](settings.md#library) with
the track's file. It acts on one track. The entry is dimmed, with the reason
as its tooltip, when no editor is set, when the file is missing or unreadable,
or when the track is *sounding*: the current track of a player (playing or
paused), a track waiting to start or on CUE, the outgoing track of a fade, a
playing cart or the cartwall's CUE cart, or the library pre-listen while it
plays or is paused. The editor is not offered on Android, nor in the Flatpak
and snap builds, which cannot start other programs; the rest of this section
still applies there.

Fauste Player notices when a file it knows has changed (its size or
modification time differ from the last analysis), whoever changed it, and
analyses it again by itself: duration, markers, loudness, waveform, format and
tags. It looks:

- once at start, at every track in the library;
- when a watched folder reports a change to a known track;
- in the folder of any file you opened in the editor, until you quit, so a
  save is noticed at once;
- when a track is loaded into a player, a cart or the pre-listen, and when its
  markers are opened for editing or dragged on the waveform;
- when you press **Look for changed files** in
  [Settings → Analysis](settings.md#analysis), which checks the whole library
  and says in a notice how many files changed.

There is no periodic polling. A file outside the watched folders, changed by
another program while Fauste Player runs and not used since, is found at its
next use or by **Look for changed files**.

A changed track is never analysed again while it sounds: it waits until it
stops, and its markers and loudness never move under a playing source.
(**Re-analyse all tracks** is your explicit request and does not wait.) Editors
often save by deleting and recreating the file, so a file that vanishes is
looked at twice before it is marked missing. A file that changes while it is
being analysed (an editor still saving it) is not used: it is analysed again
shortly after, or at its next change, use or look. A missing track whose file comes
back in a folder that is followed only because of the editor is not restored
by itself.

**Markers to review.** If the *audio* of a file changed and the track has
markers you set by hand, the track shows an amber warning icon on its row (its
tooltip names any markers that were dropped). Manual markers that now lie past
the end of the audio are dropped and the automatic value takes their place; the
others stay as they are (when the cue-in or cue-out was dropped, the intro,
outro and MIX markers are moved inside the new cue range). The flag never stops the track from loading, cueing or
playing. It clears when you choose **Mark as reviewed** in the track's menu (on
several selected rows, it clears all of them) or change any marker of the
track. The **Markers to review** item of the tree, after *Uncategorised*, lists
the flagged tracks while there are any (not on a phone). A change to the tags
or the cover alone refreshes the tags and raises no flag, which is also what
happens after you edit tags in Fauste Player. The first analysis after an
update raises the flag only if it had to drop a marker past the end of the
audio.

## Pre-listening

The bar under the title bar pre-listens the selected track on the library's
own **Library** output (chosen in [Settings → Audio
outputs](settings.md#audio-outputs)), for example headphones. A phone
pre-listens from its track list instead (see [On a phone](#on-a-phone)).

The bar shows:

- **Play**, **Pause** and **Stop**;
- the big remaining time of the track, its title, artist and category;
- at the right, a meter and a volume fader drawn as on a player. The meter
  reads only this pre-listen, never a player's or the cartwall's meter;
  click it to restart its maximum. The fader is your **listening level** in
  the headphones or monitors: it sets how loud the pre-listen sounds and
  never changes a track (its file, its loudness gain or its markers), a
  player or the cartwall. It stays as you left it when the preview moves to
  another track and is saved for the next start; its tooltip and the
  readout under it show the level in dB. If the Library output shares
  channels with an on-air output, the fader also sets how loud the
  pre-listen is there. A phone's pre-listen card has no fader;
- the whole file's waveform, as tall as a player's, with its markers, and
  an `elapsed / total` line under it.

Play starts the selected track from its cue-in and plays it to the end of the
file. Play on a paused preview resumes it, and Play on a playing one does
nothing: Stop, then Play, starts again from the cue-in. The loudness
normalization applies as on a player's CUE.

The waveform works as on a player's CUE window: click it to seek (a click or
the release of the mouse seeks; dragging does not), zoom and pan it with the
wheel, and edit the markers and open **Loudness…** from its menu (see
[Editing markers](markers-and-mixing.md#editing-markers)). While the preview
is stopped, a click sets where the next Play starts instead.

**The preview follows the selection.** With several rows selected, the last
one you clicked is the one it uses.

- While it plays, selecting another row switches to that track from its
  cue-in and keeps playing, without asking.
- While it is paused, the new track is loaded paused.
- While it is stopped, it stays stopped and the bar shows the new track.
- Selecting nothing (no row selected, or the selected row filtered out) leaves
  a running preview as it is.
- Selecting a track that cannot be played (its file is missing or unreadable)
  stops the preview, and the bar says that the file cannot be played.

**What stops it.** Stop; removing its track from the library; closing the
window (minimizing it does not). It never plays after a restart.

**Without a Library output** (on the desktop; Android plays on the default
output instead, see [Android → Library](android.md#library)) Play is dimmed
(hover it for the reason) and a line next to them says that pre-listening needs a library CUE
output, with a link that opens Settings → Audio outputs on the **Library**
row. The bar still shows the track, its waveform and its markers.

**On an on-air output.** The Library output may be an on-air output (the same
device and channels as a player's or the cartwall's Main). It then works like
the players' and the cartwall's CUE (see [CUE
(pre-listen)](players.md#cue-pre-listen)): while something plays there, Play
asks first (**Pre-listen anyway** or **Cancel**), and a pre-listen you did not
confirm stops when something starts on that output.

## Dragging to playlists and carts

Drag the selected rows (or the row you start dragging from, when it is not
selected) with the mouse:

- onto a **playlist tab** of a player: the tracks go to the end of that
  playlist;
- onto a **playlist table**: the tracks go where the violet line shows;
- onto a **cart**: the cart gets the first track (a cart holds one file),
  replacing its file when it is stopped. A cart that is playing or on CUE
  refuses the drop, because a new file would stop it on air.

## Removing tracks

**Remove from library…** asks first: the files are not deleted. When any of the
tracks is used, a second question counts the playlist entries and carts that
lose them. A track that is on air (playing in a
player or on the cartwall) cannot be removed: nothing is removed and a notice
says what plays.

**Remove missing tracks…** (in the category's menu, on any category) removes
the tracks of that category and its subcategories whose file is missing and
that no playlist entry or cart uses. It asks first, with their number; no file
is touched. Tracks in use are kept, and a category with none says so in a
notice. Use it to clean up after files left a watched folder: the application
never removes them by itself.

## From a playlist

On the desktop, the track menu of a playlist row has **Show in library** and
**Move to category ▸**, after **Loudness…**. *Show in library* opens the
window on the track's category and selects its row; *Move to category* files
the track without opening the window. For a track that is not in the library,
*Show in library* is shown disabled and *Move to category* is not shown.

## On a phone

A phone has no library window: its **Library** tab opens on the categories
(*All tracks*, your categories, *Uncategorised* and **Device music**), and a
tap on one lists its tracks with a search and **Whole library**. Tick tracks
to add them to a playlist, touch and hold a row for its menu, pre-listen a
track with its 🎧, and add the device's music to a category with the **＋**
of its list. On Android, without a Library output, the pre-listen plays
on the default output. See [Android → Library](android.md#library).
