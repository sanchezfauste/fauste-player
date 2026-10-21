# Layout

Arrange the main window for your shift: give the players the widths you
want, hide the panels you do not use and maximize one for a moment. The
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

## Maximize a panel

Right-click the background of a player's header, or of the cartwall's
header (not one of their buttons), for its menu:

- **Maximize** gives that panel all the room: the other panels shrink to
  their smallest. It is only for the moment and is not saved. **Restore**
  in the same menu ends it, and so do hiding or showing a panel, locking
  the layout, moving the cartwall, changing the number of players and
  dragging the gap between two players; the sizes from before come back.
- **Hide** hides the panel, as its tile in the top bar does.

## The layout menu

The layout button, right of the tiles, opens a menu with:

- **Cartwall at the bottom**, **Cartwall on the left**, **Cartwall on the
  right**: where the cartwall sits;
- **Lock layout**: while it is checked, nothing changes the layout by
  accident: the tiles, the other entries of this menu, the gaps between
  the players and the headers' **Maximize** and **Hide** do nothing.
  Playback, and collapsing or expanding the cartwall, work as usual. The
  lock is saved with the layout;
- **Equal player widths**: every player the same width;
- **Reset layout**: back to a new installation's layout.
