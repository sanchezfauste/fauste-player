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

The columns fill the table and keep their proportions when the window is
resized; Title gets the most room. Drag the header separators to change the
proportions; they are remembered per player.

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
| **Red**, with a speaker (or pause) icon | On air on this player |
| Red **P2** (or another number) in the number column | On air on that player |
| **Green**, with an arrow | This player's next track |
| Dimmed | Already played on this player |
| File with a cross / warning icon | File missing / unreadable (it is skipped); hover it for the reason and the path. A missing file is looked for again every 30 s (`tuning.missing_recheck_ms`). |
| Reload arrows at the right of the title | Analysed by an earlier version; it still plays with that analysis. **Settings → Analysis → Analyse outdated tracks** brings it up to date (tracks on a player are updated anyway) |
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
| Duplicate | Insert an unplayed copy below (with its repeat and stop-after marks) |
| Repeat this track | Tick to play it again and again, without a gap, until you press Play (next), Previous, Stop or Fade stop, or turn on Stop after. Pause keeps it repeating. A repeat icon shows at the right of the title |
| Stop after this track | Tick to stop the player when this track ends, every time it plays (in any mode). Unlike the player's **Stop after** button, the mark stays with the track and is saved with the playlist. The stop-after icon shows at the right of the title. It wins over Repeat |
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
