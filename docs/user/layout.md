# Layout

Arrange the main window for your shift: give the players and the cartwall
the sizes you want, put the cartwall under the players or beside them,
hide the panels you do not use and maximize one for a moment. The
layout is saved with the session and comes back at the next start; a new
installation starts with every player the same width and the cartwall
under them.

## Show and hide panels

The tiles on the left of the top bar, **P1** … **Pn** and **Cartwall**, show
which panels are on screen: a lit tile is shown. Click a tile to hide or
show its panel. With many players, the tiles scroll sideways (scroll the
mouse wheel over them) and the layout button stays in view. The other panels take the room it leaves, and a player
shown again comes back at its width.

Hiding only takes a panel off the screen. A hidden player keeps playing,
and its number key, MIDI, the remote API, its CUE window and automation
still control it. While a hidden panel is on air its tile shows a red dot:
a player that is playing or paused, or the cartwall while a cart plays.

Hiding a player clears its selected track, so **Delete** cannot remove a
track you cannot see. Files dropped on the window away from a list go to
the last player you used, or to the first shown player; with every player
hidden they are refused, and the status bar says so.

At least one panel stays shown: the last lit tile cannot be clicked.
While the cartwall is hidden, the shortcut that collapses or expands it
does nothing; click **Cartwall** to show it.

## Resize the players

Drag the gap between two players to move width from one to the other. The
other players do not move, and no player gets narrower than its controls
need. Double-click the gap to give every player the same width again.

When the window is too narrow for every shown player at its smallest, the
players' row scrolls sideways, as it always has.

## Move and resize the cartwall

The cartwall sits under the players, or on the left or the right of the
window at its full height: choose **Cartwall at the bottom**, **Cartwall on
the left** or **Cartwall on the right** in [the layout menu](#the-layout-menu).
Under the players it is as tall as its rows need, at most six tenths of the
window; beside them it takes three tenths of the width at first.

The cartwall is never narrower than its header (**Stop all**, its title and
a page tab) or than its columns of buttons at their smallest. When the
window is too narrow for it and every shown player at their smallest, the
players' row scrolls sideways.

Drag the gap between the cartwall and the players to make it larger or
smaller: up or down when it is under the players, sideways beside them. The
players keep the room their controls need. Double-click the gap to go back
to the starting size: as tall as its rows need under the players, three
tenths of the width beside them. The size is saved with the layout; moving
the cartwall starts it again at its starting size.

Once you resize the cartwall, or while it sits beside the players, its
buttons grow to fill it; the names and times keep their size, in the middle
of each button. When the cartwall is smaller than its rows and columns need
at the smallest buttons, it scrolls.

Click **CARTWALL** to collapse the cartwall. Under the players only its
header stays; beside them it becomes a narrow strip the height of the
window, with an arrow to expand it again and, while a cart plays, a red
dot. Expanded, it has its size back. While it is collapsed, the gap beside
it does nothing.

## Maximize a panel

Right-click the background of a player's header, or of the cartwall's
header (not one of their buttons), for its menu:

- **Maximize** gives that panel all the room: the other panels shrink to
  their smallest. It is only for the moment and is not saved. **Restore**
  in the same menu ends it, and so do hiding or showing a panel, locking
  the layout, moving the cartwall, changing the number of players and
  dragging or double-clicking a gap between panels; the sizes from before
  come back. A collapsed cartwall cannot be maximized, and collapsing a
  maximized cartwall ends it.
- **Hide** hides the panel, as its tile in the top bar does.

## The layout menu

The layout button, right of the tiles, opens a menu with:

- **Cartwall at the bottom**, **Cartwall on the left**, **Cartwall on the
  right**: where the cartwall sits;
- **Lock layout**: while it is checked, nothing changes the layout by
  accident: the tiles, the other entries of this menu, the gaps between
  the panels and the headers' **Maximize** and **Hide** do nothing.
  Playback, and collapsing or expanding the cartwall, work as usual. The
  lock is saved with the layout;
- **Equal player widths**: every player the same width;
- **Reset layout**: back to a new installation's layout.
