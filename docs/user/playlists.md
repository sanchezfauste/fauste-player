# Playlists

## Tabs

Every player has a row of tabs, one per playlist. All players see the same
playlists; each player chooses which one it shows. A dot on a tab shows where
the player's tracks are: **red** for the track on air, **green** for the next
one.

The tabs share the width of the player. A name that does not fit ends in
"…"; hover over the tab to read it in full. With many playlists the tabs stop
shrinking at a minimum width, arrows appear at the ends of the row and the
mouse wheel over the tabs scrolls them. The tab you choose, and the one a
player shows, is brought into view.

Switching tabs never changes what is on air or what is next. When a track
ends, the player continues in the playlist that contains that track.

Playlists are created, renamed and deleted in [Settings](settings.md). The
last playlist, and a playlist with a track on air, cannot be deleted.

## The track table

![A playlist: played tracks dimmed, the track on air in red, the next track in green, and the footer with the time left](../images/guide/playlist.png)

| Column | Content |
|---|---|
| `#` | Position, zero-padded; an icon replaces it for the current and next tracks |
| Title | From the tags, or the file name (`Artist - Title.mp3` is split). The repeat and stop-after icons of a track sit before it |
| Artist | From the tags; "Unknown artist" when there is none |
| Album | From the tags |
| Date | The recording date as the file stores it (`2019`, `2019-05` or `2019-05-14`, with a time if there is one) |
| Genre | From the tags |
| Dur. | Playing length, from cue-in to cue-out (the whole file with **Use cue-in and cue-out** off) |
| Intro | How long the intro lasts, from where the track starts playing to its intro marker; empty when the track has no intro marker |
| File name | The name of the file, with its extension |

A new installation shows `#`, Title, Artist and Dur. The other columns are
optional; see **Choosing the columns** below. A track that lacks a value
shows an empty cell, except Artist, which shows "Unknown artist".

The columns fill the table and keep their proportions when the window is
resized; the text columns get the most room. Drag the header separators to
change the proportions: the columns to the right of the separator follow the
pointer on every frame (they share what is left in proportion to their
width), the ones to its left stay, and the widths are saved when you let go.
No column gets narrower than its minimum. The widths are remembered per
player; a column you show later starts with its default width and the others
keep their proportions.

### Choosing the columns

Title and Dur. are always shown. Every other column can be shown or hidden,
and any column, those two included, can be moved. The list is the same for
every player and playlist, and it is saved in `config.json` as
`ui.table_columns`. Three ways to change it:

- **Settings → Playlists → Table columns:** tick the columns to show; the
  arrows move a shown column up or down (they read left to right in the
  tables). **Default columns** goes back to `#`, Title, Artist and Dur.
- **Right-click a header:** a menu with a checkbox for every optional column.
  A column you show appears at the right end; drag it from there.
- **Drag a header** onto another: drop it on the left half of a header to put
  the column before it, on the right half to put it after it. Dropping
  anywhere else does nothing.

A name in `ui.table_columns` that this version does not know is ignored, and
a missing Title or Dur. is added back.

When the application opens, each table scrolls so that its player's next
track is in the middle of the table (as near as the ends of the list allow).
That happens once, at start-up, and only when the next track is in the
playlist the table shows.

When a player moves on to another track, its table shows that track's
playlist and scrolls its row to the top, unless you used the table in the
last 10 seconds (scrolled it, dragged a track, opened a track's menu or
clicked a tab): then it waits until you leave it alone for that long. The
time is `ui.follow_current_grace_secs` in `config.json`; 0 turns following
off.

Players are independent: several players can show the same playlist, each
with its own next track, its own played marks and its own times in the
footer. Playing, stopping or skipping on one player never moves another
player's next. The same track can even be on air on two players at once.
Editing the playlist (adding, moving or removing entries) or a file becoming
unreadable can still change the next of any player that shows it.

Row colours:

| Row | Meaning |
|---|---|
| **Red**, with a speaker (or pause) icon | On air on this player. It can also show the green arrow: the track on air is also the next, so it plays once more |
| Red **P2** (or another number) in the number column | On air on that player |
| **Green**, with an arrow | This player's next track |
| Dimmed | Already played on this player |
| File with a cross / warning icon | File missing / unreadable (it is skipped); hover the row: the popup starts with the reason, in amber, then the usual fields. A missing file is looked for again every 30 s (`tuning.missing_recheck_ms`). |
| Reload arrows at the right of the title | Analysed by an earlier version; it still plays with that analysis. **Settings → Analysis → Analyse outdated tracks** brings it up to date (tracks on a player are updated anyway) |
| Hourglass at the right of the title | The track is waiting for its analysis (hover the hourglass: *Analysis pending*). It plays anyway, and the hourglass goes when the analysis finishes |
| Violet | Selected |

**Track tooltip.** Hover a row for a moment to see its title, artist, album, date, genre, length, format (type, sample rate and bit depth when known) and the path of its file. A field the file does not have is left out. The popup is the only hover information on a row, and it never moves while it is shown. For a missing or unreadable file it starts with the reason.

## Mouse

- **Click** selects a track. **Double-click** makes it this player's next
  track. On the track on air it makes it play once more when the current pass
  ends.
- **Right-click** opens the context menu:

![The context menu of a track](../images/guide/track-menu.png)

| Item | Action |
|---|---|
| Play now | Start this track at once (mixing if the player is on air) |
| Set as next | Same as double-click. On the track on air it plays once more, from its start, when the current pass ends (mixing like Repeat, no gap), then the player goes on. It acts once. Stop after, SINGLE mode and a Stop after mark still end the player first. While a CUE is running it moves to the new next |
| Pre-listen on CUE | Play it on the CUE output (it opens the CUE window). Dimmed when the player has no Cue output apart from its Main one |
| Edit tags… | Open the tag editor for this track. **Save** writes the changes into the audio file; **Cancel** (or Esc, when no save is running) closes without writing. The item is dimmed, with the reason when you hover it, while the track is on air, on CUE or on a playing cart, while its tags have not been read yet, when the file is missing, and for formats whose tags cannot be written (for example DSD) |
| Re-analyse | Analyse this track again now, whatever its state. A fixed file that was unreadable is also picked up by itself (see [Troubleshooting](troubleshooting.md)). Manual markers are kept |
| Add tracks below… | Pick files to insert after this track |
| Duplicate | Insert an unplayed copy below (with its repeat and stop-after marks) |
| Repeat this track | Tick to play it again and again, without a gap, until you press Play (next), Previous, Stop or Fade stop, or turn on Stop after. Pause keeps it repeating. A repeat icon shows before the title |
| Stop after this track | Tick to stop the player when this track ends, every time it plays (in any mode). Unlike the player's **Stop after** button, the mark stays with the track and is saved with the playlist. The stop-after icon shows before the title. It wins over Repeat |
| Move to ▸ | Move it to the end of another playlist |
| Remove from playlist | Remove it; not possible while it is on air |

## Editing tags

**Edit tags…** opens a window for one track. While it is open no keyboard
shortcut acts, and files dropped on the application window are ignored.

![The tag editor of a FLAC file, with its cover, title, artist, album, date and genre](../images/guide/tag-editor.png)

- **What you see.** The editor reads the file when it opens (it shows
  "Reading tags…" meanwhile). Always shown: title, artist, album, album
  artist, date, track number and total, disc number and total, genre,
  composer and comment. Shown when the file has them: subtitle, grouping,
  BPM, initial key, mood, ISRC, publisher, catalog number, copyright,
  original artist, original album, original release date, lyricist,
  conductor, remixer, arranger, performer, language, encoded by, lyrics, sort
  title, sort artist, sort album, sort album artist, sort composer and artist
  website.
- **Add field.** The menu below the fields lists the other fields. It offers
  only what the file's tag format can store (a WAV with RIFF INFO, an AIFF or
  an old ID3v1 tag store fewer fields than ID3v2, FLAC or MP4), and it is
  dimmed when there is nothing left to add. One of the always-shown fields
  that the format cannot store is greyed out with a note. Clearing a field
  removes it from the file; an added field left empty is not written.
- **Several values.** Fields that can hold several values (artist, album
  artist, genre, composer, mood and the credits such as lyricist, conductor,
  remixer, arranger and performer, and language) show one value per line;
  **Save** writes one value per line in the format's own way. Comment and
  lyrics are free text over several lines.
- **Checks.** Date and original release date are ISO 8601 (`2019`,
  `2019-05` or `2019-05-14`, optionally with a time); track and disc number,
  their totals and the BPM are whole numbers, and a total needs its number. A
  field with an invalid value is marked and **Save** stays off. A value the
  file already had and you did not touch is kept as it is.
- **Fields that are too long.** A field whose text is longer than
  `limits.max_tag_chars`, or that holds more values than
  `limits.max_tag_values`, is shown read-only with the note "Too long to edit
  here; kept as it is in the file". It is never written back, so a save
  cannot cut it.
- **What is kept.** Everything the editor does not show (other standard
  keys, custom keys, pictures other than the front cover, binary frames)
  stays in the file with the same values. The editor says how many such tags
  are kept (and "more" when the format holds frames that cannot be counted).
  Saving re-encodes the items the editor maps, so a kept item can differ in
  its bytes (text encoding, frame order) but not in its value.
- **The cover.** The editor shows the front cover, or the first picture of
  the file when there is no front cover, as a thumbnail.
  - **Change…** opens a file dialog for a JPEG or PNG image (at most
    `limits.max_cover_bytes`, and it must decode). If it does not, the editor
    says why and nothing changes.
  - **Remove** clears the front cover. It is off when the file has no front
    cover: a picture shown only because there is no front cover is display
    only and is kept as it is.
  - A cover that is in the file but cannot be shown (an image that does not
    decode, or a GIF, BMP or WebP) is announced with "This cover cannot be
    shown; it is kept as it is". **Change…** and **Remove** still work.
  - The change is written by **Save** and discarded by **Cancel**. Back
    covers and every other picture are never touched. A format with no place
    for pictures (WAV with RIFF INFO, AIFF, ID3v1) shows the area disabled.
    After a save the player's cover shows the new cover.
- **How a save works.** The file is copied next to the original, the copy
  gets the tags, is synced and replaces the original, so a failure leaves the
  file as it was. The reason shows in the editor, which stays open to retry,
  and in the status bar. Only the fields you changed are written. After a
  save the table shows the new tags at once, and markers and the waveform are
  kept. If the file did not keep a field you changed, the status bar names it.
- **After an update.** Tracks of an earlier version get their date, genre and
  other tags filled in quietly in the background (no full analysis).

## Drag and drop

- Drag a track within the list to reorder it. A violet line shows where it
  will land: it is at the row boundary nearest to the pointer, and only in
  the list under the pointer. Releasing over the header, a column edge, the
  scroll bar or a window that covers the list (the CUE window) drops nothing. The mouse wheel does not scroll the list while a
  row is being dragged: scroll first, then drag.
- Drag it onto another player's list to move it there.
- Drag it onto a tab to append it to that playlist.
- Drop files or folders from the file manager onto a list to insert them at
  the drop position. If the system does not report the position, they go to
  the end of the list shown.

## Footer

**+ Add** opens a file dialog, starting in the music folder set in
Settings. **Reset played** (the arrow icon next to it) clears the dimmed
"already played" mark of every track of the playlist, for every player,
after asking "Clear the played mark of every track in this playlist?"
(**Cancel**, Esc or a click outside keep the marks). The track that is on
air keeps its state and is marked when the player leaves it. The button is
dimmed when there is nothing to clear. The footer also shows the number of
tracks, the time left in the playlist and its total length.

## Playlist files

- **Import:** Settings → Playlists → **Import M3U / PLS…**, or drop an
  `.m3u`, `.m3u8` or `.pls` file on the window. It becomes a new playlist
  named after the file.
  - Relative paths are resolved against the playlist file's folder.
  - `file://` addresses are understood.
  - Files that cannot be found are still added, marked unavailable.
  - Internet streams are skipped; a message says how many.
- **Export:** the **M3U** button on each playlist in Settings saves it as an
  M3U8 file with titles, lengths and full paths.
