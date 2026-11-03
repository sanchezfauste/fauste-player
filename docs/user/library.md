# Library

The library is every track Fauste Player knows: each file you have added to a
playlist, a cart or the library itself is one track, with its tags, markers
and loudness settings. The library window lists them, groups them in
*categories*, finds them by text and sends them to the playlists and the
cartwall. It is a desktop window; the phone has no library window.

## Opening the window

Click the **Library** tile in the top bar. It is lit while the window is open;
click it again, or the window's close button, to close it. The window floats
over the players and the cartwall and does not block them: nothing in it stops
the audio or the shortcuts of the main window.

Move it by its title bar and resize it from its edges and corners. The title
bar has three buttons: **Minimize** (only the title bar is left at the top of
the main area; click it again to expand), **Maximize** (the window fills the
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
categories, which can hold subcategories; at the end, **Uncategorised**, the
tracks that belong to no category. Each item shows its colour and a count: a
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
  subcategory…**, **Rename…**, **Colour**, **Kind** and **Delete…**.
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

## The track table

The table lists the tracks of the selected item, in the same columns as the
playlist tables (the ones chosen in **Settings → Playlists → Table columns**,
without the position `#`) followed by **Category** and **Kind**. A narrow cell
at the left shows a track's problem: its file is missing or cannot be read, or
it is waiting for its analysis.

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
otherwise the menu acts on every selected row. The first line says how many
tracks are selected.

| Item | Action |
|---|---|
| Add to ▸ | A playlist: the tracks are added at its end |
| Add to cart ▸ | A cart of the cartwall's shown page: the tracks fill the chosen cart and the next empty carts of the page, in grid order. The tracks that do not fit are left out and a notice counts them |
| Move to category ▸ | A category, or Uncategorised |
| Kind ▸ | Music, Jingle, Effect, Ad or Voice |
| Edit tags… | The tag editor (one track) |
| Loudness… | The loudness popup (one track) |
| Re-analyse | Analyse the tracks again |
| Reset markers to automatic | Drop the manual markers |
| Show file location | Open the file's folder (one track) |
| Remove from library… | Remove the tracks from the library, after asking (below) |

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

## From a playlist

On the desktop, the track menu of a playlist row has **Show in library** and
**Move to category ▸**, after **Loudness…**. *Show in library* opens the
window on the track's category and selects its row; *Move to category* files
the track without opening the window. A track that is not in the library has
neither.
