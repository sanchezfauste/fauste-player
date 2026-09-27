# Playlists

## Tabs

Every player has a row of tabs, one per playlist. All players see the same
playlists; each player chooses which one it shows. A dot on a tab shows where
the player's tracks are: **red** for the track on air, **green** for the next
one.

Switching tabs never changes what is on air or what is next. When a track
ends, the player continues in the playlist that contains that track.

Playlists are created, renamed and deleted in [Settings](settings.md). The
last playlist, and a playlist with a track on air, cannot be deleted.

## The track table

| Column | Content |
|---|---|
| `#` | Position, zero-padded; an icon replaces it for the current and next tracks |
| Title | From the tags, or the file name (`Artist - Title.mp3` is split) |
| Artist | From the tags; "Unknown artist" when there is none |
| Dur. | Playing length, from cue-in to cue-out |

Columns can be resized by dragging the header separators. The widths are
remembered per player.

Players are independent: several players can show the same playlist, each
with its own next track, its own played marks and its own times in the
footer. Playing, stopping or skipping on one player never moves another
player's next. The same track can even be on air on two players at once.
Editing the playlist (adding, moving or removing entries) or a file becoming
unreadable can still change the next of any player that shows it.

Row colours:

| Row | Meaning |
|---|---|
| **Red**, with a speaker (or pause) icon | On air on this player |
| Red **P2** (or another number) in the number column | On air on that player |
| **Green**, with an arrow | This player's next track |
| Dimmed | Already played on this player |
| Warning icon | File missing or unreadable (it is skipped) |
| Violet | Selected |

## Mouse

- **Click** selects a track. **Double-click** makes it this player's next
  track (not the one on air).
- **Right-click** opens the context menu:

| Item | Action |
|---|---|
| Play now | Start this track at once (mixing if the player is on air) |
| Set as next | Same as double-click |
| Pre-listen on CUE | Play it on the CUE output |
| Add tracks below… | Pick files to insert after this track |
| Duplicate | Insert an unplayed copy below |
| Move to ▸ | Move it to the end of another playlist |
| Remove from playlist | Remove it; not possible while it is on air |

## Drag and drop

- Drag a track within the list to reorder it. A violet line shows where it
  will land.
- Drag it onto another player's list to move it there.
- Drag it onto a tab to append it to that playlist.
- Drop files or folders from the file manager onto a list to insert them at
  the drop position. If the system does not report the position, they go to
  the end of the list shown.

## Footer

**+ Add** opens a file dialog, starting in the music folder set in
Settings. The footer also shows the number of tracks, the time left in the
playlist and its total length.

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
