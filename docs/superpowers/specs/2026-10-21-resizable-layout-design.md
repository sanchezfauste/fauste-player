# Resizable layout — Design Spec

- **Date:** 2026-10-21.
- **Status:** Draft; the design was approved by the maintainer on
  2026-10-21; awaiting the review of this written spec.
- **Amends:** [the Phase 2 spec](2026-09-26-phase2-cartwall-settings-design.md)
  (P2.9, the cartwall strip under the players) and the session document
  ([`docs/technical/persistence.md`](../../technical/persistence.md)).
- **Numbering:** rules are `RL<n>`. Each rule is testable and gets at least
  one test (§7).

---

## 1. Goal

Each station arranges the main window its own way. A two-player desk wants
wide players. A station that plays many jingles wants a large cartwall
beside the players, not under them. The operator can:

- resize the players and the cartwall;
- dock the cartwall at the bottom, on the left or on the right;
- hide panels that the shift does not use, and maximize one for a moment.

A lock keeps a finished layout from being changed by a stray drag on air.

Out of scope:

- players in more than one row;
- more than one cartwall panel;
- named layouts (a later step, built on RL16's document).

## 2. Terms

- **Panel**: one player column, or the cartwall.
- **Middle area**: everything between the top bar and the status bar.
- **Splitter**: the gap between two neighbouring panels. Dragging it moves
  space from one neighbour to the other.
- **Dock**: where the cartwall sits. The dock is `Bottom` (today's place,
  under the players), `Left` or `Right`. A side dock is the full height of
  the middle area.
- **Neighbours**: two players next to each other among the *shown* players.
  Hidden players between them do not count.

## 3. Model (`fp-model`)

The layout is session state, kept with the players (RL16), as the table
column widths are. It belongs to the operator's desk, not to the station's
settings, so it is not part of `Config`. Its rules are pure functions in a
new module, `fp-model/src/layout.rs`. The interface only executes and
displays them.

- **RL1. Layout state.** `AppState` gains `layout: Layout`, with:
  - `player_shares: Vec<(PlayerId, f32)>`: each player's share of the
    players' width;
  - `hidden_players: Vec<PlayerId>`;
  - `cartwall: CartwallLayout { dock: Dock, size: Option<f32>, hidden: bool }`.
    `size` is the cartwall's share of the middle area: of its height when
    docked at the bottom, of its width on a side. `None` means "fit the
    content", which is today's height at the bottom;
  - `locked: bool`.

  The default is today's window: equal shares, nothing hidden, the cartwall
  at the bottom fitting its content, unlocked.
- **RL2. Shares.** A shown player's width is its share divided by the sum of
  the shown players' shares (the pixel pass, RL9, adds the minimums).
  - **Missing shares.** A player without a share counts as the mean share
    of the others, or 1.0 when no player has one, so a new player gets an
    equal slice.
  - **Bad shares.** A share that is not finite or not above 0 counts as
    missing.
  - **Hidden players.** Hiding a player keeps its share, so showing it
    again restores its width.
- **RL3. Players come and go.** When players are removed (`SetPlayerCount`,
  or an `UpdateConfig` that lowers `limits.max_players`), their shares and
  hidden flags are dropped. New players are shown. If no panel is left
  shown, every panel is shown.
- **RL4. Resizing players.** `Command::SetPlayerShares { left: (PlayerId, f32), right: (PlayerId, f32) }`
  sets the shares of two neighbours. Their sum stays as before; the
  interface computes both from the drag. Each new share must be finite and
  above 0. A stranger, two players that are not neighbours, or a bad share
  is rejected (`ModelError::BadLayout`).
- **RL5. Equal widths.** `Command::EqualPlayerWidths` gives every player
  the same share.
- **RL6. Cartwall dock and size.**
  - **Dock.** `Command::SetCartwallDock(Dock)` moves the cartwall. Its
    `size` goes back to `None` at the bottom; on a side it becomes the
    default side share, 0.3 of the middle area's width.
  - **Size.** `Command::SetCartwallSize(Option<f32>)` sets the share,
    clamped to `0.05 ..= 0.95`. The interface also keeps the players'
    minimums (RL8).
- **RL7. Hiding.** `Command::SetPanelHidden { panel: Panel, hidden: bool }`
  hides or shows a panel; `Panel` is `Player(PlayerId)` or `Cartwall`.
  - **The last shown panel** cannot be hidden: that command is rejected
    (`ModelError::LastPanel`).
  - **Playback.** Hiding never touches playback: a hidden player that is
    on air keeps playing. Keys, MIDI, the remote API, the CUE windows and
    automation act on it as before.
  - **Collapsing is separate.** `SetCartwallOpen` (collapse or expand)
    is not a layout command and works while the layout is locked. The
    shortcut that toggles the cartwall does nothing while it is hidden;
    the top-bar toggle shows it.
- **RL8. Minimum sizes.** Minimum sizes come from the panels' content, so
  they are interface constants, not operator limits:
  - a player's width is at least what its fixed parts need (the meter,
    the fader and the transport): today's `MIN_COLUMN_WIDTH`;
  - a player's height is at least its header, waveform and transport,
    plus three table rows (`MIN_PLAYER_HEIGHT`);
  - the cartwall's width is at least the larger of its header's fixed
    parts ("Stop all", the caret, the smallest tab) and `cols` × the
    smallest cart button, plus the gaps. Stop all is measured with a
    two-digit count, so the least width does not change while carts start
    and stop;
  - the cartwall's height is at least its header plus `rows` × the
    smallest cart button, plus the gaps.
- **RL9. Lock.** `Command::SetLayoutLocked(bool)`. While the layout is
  locked, the model rejects every layout command except `SetLayoutLocked`
  itself: `SetPlayerShares`, `EqualPlayerWidths`, `SetCartwallDock`,
  `SetCartwallSize`, `SetPanelHidden` and `ResetLayout` all get
  `ModelError::LayoutLocked`. The interface disables those controls, so
  it never causes that error.
- **RL10. Reset.** `Command::ResetLayout` restores RL1's default and keeps
  the lock off. It is only possible while the layout is unlocked (RL9).

`BadLayout`, `LastPanel` and `LayoutLocked` get a message in every locale
for the interface's notice, if one ever reaches it.

## 4. Interface (`fp-app`)

- **RL11. The pixel pass.** Each frame turns the layout into rectangles:
  1. **The cartwall.** It gets its size: at the bottom, a share of the
     middle area's height, or its content height for `None` (RL13); on a
     side, a share of the width. That size is clamped between the
     cartwall's minimum and what leaves the players their minimum
     (RL8).
     At the bottom with `None`, the size is RL13's "fit the content",
     exactly today's, and is not clamped further.
  2. **The players.** The shown players split the rest. Each first gets
     its minimum width; what is left over is split by share.
  3. **Too small a window.** Only when the minimums alone do not fit
     does the players' row scroll horizontally, as today. The cartwall
     keeps its minimum.

  A column's egui id comes from its `PlayerId`, not from its position
  among the shown players. Its label stays `P<n>`, from its place among
  all players.
- **RL12. Splitters.**
  - **Pointer.** Over a splitter, the pointer shows the resize cursor.
  - **Players.** Dragging the splitter between two neighbours resizes
    those two, clamped at both minimums. The new widths are drawn at once.
    When the drag ends, `SetPlayerShares` is sent once, and only if the
    drag moved something. Until the model's echo arrives, the dragged
    widths are held. This is how the table's column resize already works
    with `SetColumnWidths` and its hold.
  - **Equal widths.** Double-clicking a player splitter sends
    `EqualPlayerWidths`.
  - **Cartwall.** The splitter between the players and the cartwall
    resizes the cartwall the same way (`SetCartwallSize`): horizontal at
    the bottom, vertical on a side. Double-clicking it goes back to "fit
    the content" at the bottom, or to the default share on a side.
  - **Lock.** While the layout is locked, or the cartwall is collapsed on
    a side, its splitter does not react, and the pointer over it shows the
    normal cursor. At the bottom a collapsed cartwall is only its header,
    so its splitter does not react either.
- **RL13. Cart buttons scale.**
  - **Size.** The shown page's grid (`rows` × `cols`) fills the cartwall
    panel. Buttons already fill the width; their height, now the free
    height divided by `rows`, has no 40 px upper clamp when `size` is
    `Some`. The same holds whenever the panel does not fit its content:
    maximized, or alone in the middle area.
  - **Too small a panel.** Buttons never shrink below the smallest button
    (`MIN_BUTTON_WIDTH` × `MIN_BUTTON_HEIGHT`). When the panel is smaller
    than the grid needs, the grid scrolls.
  - **Fit the content.** At the bottom with `size: None`, the panel is
    exactly today's: the rows at `BUTTON_HEIGHT`, never more than 60 % of
    the middle area.
  - **Text.** The name and the detail line stay at their sizes, centred.
    The waveform behind them grows with the button. Centred is vertical:
    the text block of a 40 px button sits in the middle of a taller one.
  - The grid limits stay `limits.max_cart_rows` and
    `limits.max_cart_cols`.
- **RL14. Collapsed cartwall.** At the bottom, collapsing works as today:
  only the header is shown. On a side, the cartwall becomes a narrow
  vertical strip, the width of the header's height, with the expand caret
  and, while a cart plays, an on-air dot. Expanding brings back the share
  it had. At the bottom a collapsed cartwall is its header even when it
  has a share. A collapsed cartwall cannot be maximized, and collapsing a
  maximized cartwall ends the maximize.
- **RL15. Top bar.**
  - **Panel toggles.** On the left of the top bar, one tile per player
    (`P1`, `P2`, …) and one for the cartwall, named as the cartwall is in
    each language (`Cartwall`). A lit tile is a shown panel; a click hides
    or shows it (RL7). The last lit tile is disabled. The tiles never run
    into the clock and the buttons on the right: when they do not fit,
    they scroll sideways, and the layout button stays in view. Every
    tile and button of the top bar has the same height and centre line.
  - **On-air dot.** The tile of a hidden panel that is on air shows a red
    dot. For a player, that is a current entry that is not stopped. For
    the cartwall, any cart playing.
  - **Layout menu.** A layout button opens a menu with:
    - the cartwall's dock: `Bottom`, `Left` or `Right`, with the current
      one checked;
    - `Lock layout`, a check;
    - `Equal player widths`;
    - `Reset layout`.

    While the layout is locked, every entry except `Lock layout` is
    disabled, and so are the panel toggles.
- **RL16. Maximize.**
  - **Menu.** A right click on the background of a player's header or of
    the cartwall's header opens a menu with `Maximize` (or `Restore`) and
    `Hide`. The tiles in the headers keep their own clicks.
  - **Header buttons.** Each player's header and the cartwall's header end
    with two small icon buttons, in this order: `Hide` (a minus; the same
    command as the menu's `Hide`) and `Maximize` (corners out), which is
    `Restore` (corners in) on the maximized panel. Each has a tooltip and
    an accessible name. They follow the menu's rules: both are disabled
    while the layout is locked, `Hide` for the last shown panel, `Maximize`
    with one panel shown and for a collapsed cartwall. In a player's
    header they belong to the right group that never collapses; the
    cartwall's collapse caret stays, and so does the menu.
  - **Maximized.** The panel takes all the free space, and every other
    shown panel shrinks to its minimum (RL8). That includes the cartwall
    on a side, and the players' height at the bottom.
  - **Not saved.** Maximizing is a moment's view: it lives in `ViewState`,
    is never saved and sends no command. It is disabled while the layout
    is locked.
  - **Ending it.** `Restore` or a second `Maximize` ends it and brings
    back the sizes from before. Locking, hiding or showing any panel, a
    dock change and a change in the number of players end it too. A
    splitter drag first restores the earlier sizes, then applies itself.
- **RL17. Hidden players and the selection.** Hiding a player clears its
  table selection. If it was the last player used, there is no last player
  used until another one is clicked. So Delete cannot remove an entry the
  operator cannot see.
- **RL18. File drops.** A file dropped on the window away from a table goes
  to the last player used, as today, or else to the first shown player.
  With no player shown, the drop is refused with a notice.

## 5. Persistence

- **RL19. Session document.**
  - **The field.** The session document gains `layout`, so
    `SESSION_SCHEMA` stays 1. An older session has none and loads with
    RL1's default, which is today's layout.
  - **Lenient loading.** `layout`, and each part of it, loads leniently,
    like the session's column widths and history (a `deserialize_with`
    with a fallback). A part that does not parse, such as a wrong type
    or an unknown dock, loads as its default with a log line. The rest
    of the session is never rejected because of it.
  - **Repairs.** `AppState::restore` applies the layout after it builds
    the players. It drops shares and hidden flags of players that no
    longer exist (a session saved with more players). If every panel is
    hidden, it shows them all. Each repair writes a log line.
  - **Saving.** A layout command changes the model and so is saved like
    `SetColumnWidths`.

## 6. Remote API, MIDI and documentation

- The remote API and MIDI do not change. The layout is the local window's
  arrangement, and hiding a panel changes nothing they control (RL7).
- **User guide:**
  - `docs/user/` gains a "Layout" section: splitters, docking, hiding,
    the on-air dot, maximizing, the lock and the reset;
  - the cartwall page mentions the dock and that buttons scale.
- **Technical docs:**
  - `docs/technical/ui.md` describes the pixel pass;
  - `docs/technical/persistence.md` describes the session's `layout`.
- New strings go into every locale.
- The README and guide screenshots use the default layout, which does not
  change.

## 7. Tests

- **Model** (one test per rule):
  - RL1, RL2: the default layout; missing and bad shares; a hidden
    player keeps its share.
  - RL3: lowering the player count, and lowering `max_players`, drop the
    shares and hidden flags; the "all hidden" repair; new players are
    shown.
  - RL4, RL5: neighbours across a hidden player; a stranger, a
    non-neighbour and a bad share are rejected; equal widths.
  - RL6: a dock change resets the size; the clamp.
  - RL7: hiding the last panel is rejected; a hidden player keeps
    playing; `SetCartwallOpen` works while locked.
  - RL9, RL10: locked rejects every layout command, `ResetLayout`
    included; a reset gives the default.
- **Restore and store** (RL19):
  - an old session without `layout`;
  - a malformed `layout`, and a malformed part of it, keep the rest of
    the session;
  - unknown players, a session saved with more players than
    `players.count`, an unknown dock, everything hidden.
- **Interface** (`egui_kittest`):
  - RL11: shrinking the window below the minimums, on all three docks;
    a hidden player's neighbour keeps its id.
  - RL12: a player-splitter drag changes both widths and sends one
    command at the end, none for a click; a double-click sends equal
    widths; nothing reacts while locked; the minimums hold.
  - RL13: buttons grow with the panel; the grid scrolls at the minimum;
    at the bottom with `None`, the rectangles are today's.
  - RL14: a side dock collapsed, then expanded.
  - RL15: the toggles; the on-air dot; the menu and the toggles disabled
    while locked; the cartwall shortcut while hidden.
  - RL16: maximize and restore; hiding, locking and a dock change end
    it; a drag restores the earlier sizes first.
  - RL17: hiding clears the selection; Delete then does nothing.
  - RL18: a drop with a hidden last player; a drop with no player shown.
