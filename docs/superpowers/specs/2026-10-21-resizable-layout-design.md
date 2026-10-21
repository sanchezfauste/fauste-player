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
- hide panels that the shift does not use, and maximise one for a moment.

A lock keeps a finished layout from being changed by a stray drag on air.

Out of scope:

- players in more than one row;
- more than one cartwall panel;
- named layouts (a later step, built on RL15's document).

## 2. Terms

- **Panel**: one player column, or the cartwall.
- **Middle area**: everything between the top bar and the status bar.
- **Splitter**: the gap between two neighbouring panels. Dragging it moves
  space from one neighbour to the other.
- **Dock**: where the cartwall sits. The dock is `Bottom` (today's place,
  under the players), `Left` or `Right`. A side dock is the full height of
  the middle area.

## 3. Model (`fp-model`)

The layout is session state, kept with the players (RL15). It belongs to
the operator's desk, not to the station's settings, so it is not part of
`Config`. Its rules are pure functions in a new module,
`fp-model/src/layout.rs`. The interface only executes and displays them.

- **RL1. Layout state.** `AppState` gains `layout: Layout`, with:
  - `player_widths: Vec<(PlayerId, f32)>`: each player's share of the
    players' width;
  - `hidden_players: Vec<PlayerId>`;
  - `cartwall: CartwallLayout { dock: Dock, size: Option<f32>, hidden: bool }`.
    `size` is the cartwall's share of the middle area: of its height when
    docked at the bottom, of its width on a side. `None` means "fit the
    content", which is today's height at the bottom;
  - `locked: bool`.

  The default is today's window: equal widths, nothing hidden, the
  cartwall at the bottom fitting its content, unlocked.
- **RL2. Shares.** The width of a shown player is its share divided by the
  sum of the shown players' shares. A player without a share counts as the
  mean share of the others, so a new player gets an equal slice. Removing a
  player drops its share. Hiding a player keeps its share, so showing it
  again restores its width. A share that is not finite or not above 0 is
  treated as missing.
- **RL3. Resizing players.** `Command::ResizePlayers { left, right, fraction }`
  moves `fraction` of the players' width from `right` to `left`. A negative
  fraction moves it the other way. `left` and `right` are neighbours among
  the shown players. Only those two change. The UI clamps the drag so that
  neither falls below its minimum width (RL7), and the model clamps both
  results to stay above 0. Unknown or non-neighbouring players are
  rejected (`ModelError`).
- **RL4. Equal widths.** `Command::EqualPlayerWidths` gives every player
  the same share.
- **RL5. Cartwall dock and size.** `Command::SetCartwallDock(Dock)` moves
  the cartwall. Its `size` goes back to `None` at the bottom; on a side it
  becomes the default side share, `0.3` of the middle area's width.
  `Command::SetCartwallSize(Option<f32>)` sets the share, clamped to
  `0.05 ..= 0.95`.
- **RL6. Hiding.** `Command::SetPanelHidden { panel: Panel, hidden: bool }`
  hides or shows a panel, where `Panel` is `Player(PlayerId)` or
  `Cartwall`. The last shown panel cannot be hidden; that command is
  rejected. Hiding never touches playback: a hidden player that is on
  air keeps playing. Keys, MIDI, the remote API, the CUE windows and
  automation act on it as before.
- **RL7. Lock.** `Command::SetLayoutLocked(bool)`. While the layout is
  locked, the model rejects every layout command except
  `SetLayoutLocked` and `ResetLayout`. Each rejection is a
  `ModelError::LayoutLocked`, which the UI never causes, because it
  disables those controls.
- **RL8. Reset.** `Command::ResetLayout` restores RL1's default and keeps
  `locked`. The UI offers it only while the layout is unlocked.

Minimum sizes come from the panels' content, so they are interface
constants, not operator limits:

- a player is at least as wide as its fixed parts need (the meter, the
  fader and the transport), which is today's `MIN_COLUMN_WIDTH`;
- the cartwall is at least `cols` × the smallest cart button wide and `rows`
  × its smallest height tall, plus the header and the gaps.

The model only keeps the shares. The interface turns them into pixels and
enforces the minimums.

## 4. Interface (`fp-app`)

- **RL9. Splitters.**
  - **Dragging.** The pointer over a splitter shows the resize cursor.
  - **Players.** Dragging the splitter between two players resizes those
    two, clamped at both minimums. While dragging, the new widths are
    drawn at once; the command is sent once, when the drag ends, as the
    sliders already do. Double-clicking a player splitter sends
    `EqualPlayerWidths`.
  - **Cartwall.** The splitter between the players and the cartwall
    resizes the cartwall (`SetCartwallSize`): horizontal at the bottom,
    vertical on a side. Double-clicking it goes back to "fit the
    content" at the bottom, or to the default share on a side.
  - **Too narrow a window.** When the window is narrower than the shown
    panels' minimums, the players' row scrolls horizontally, as today.
    The cartwall keeps its minimum.
  - **Lock.** While the layout is locked, splitters do not react, and the
    pointer over them shows the normal cursor.
- **RL10. Cart buttons scale.**
  - **Size.** The shown page's grid (`rows` × `cols`) fills the cartwall
    panel. A button is the panel's width less the gaps, divided by `cols`;
    its height is the free height, divided by `rows`.
  - **Too small a panel.** Buttons never shrink below the smallest button
    (`MIN_BUTTON_WIDTH` × `MIN_BUTTON_HEIGHT`). When the panel is smaller
    than the grid needs, the grid scrolls.
  - **Fit the content.** At the bottom with `size: None`, the panel is
    exactly as tall as the rows at today's `BUTTON_HEIGHT`, never more
    than 60 % of the middle area. This is today's layout, unchanged.
  - **Text.** The name and the detail line stay at their sizes, centred.
    The waveform behind them grows with the button.
  - The grid limits stay `limits.max_cart_rows` and `limits.max_cart_cols`.
- **RL11. Collapsed cartwall.** Collapsing (`SetCartwallOpen(false)`)
  works as today at the bottom: only the header is shown. On a side, the
  cartwall becomes a narrow vertical strip, the width of the header's
  height. It shows the expand caret, and an on-air dot while a cart
  plays. Its splitter does not react while it is collapsed.
- **RL12. Top bar.**
  - **Panel toggles.** On the left of the top bar, one tile per player
    (`P1`, `P2`, …) and one for the cartwall (`Carts`). A lit tile is a
    shown panel; a click hides or shows it (RL6). The last lit tile is
    disabled.
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
- **RL13. Maximise.**
  - **Menu.** A right click on a player's header or on the cartwall's
    header opens a menu with `Maximise` (or `Restore`) and `Hide`.
  - **Maximised.** The panel takes all the free space; every other shown
    panel shrinks to its minimum.
  - **Ending it.** `Restore`, a second `Maximise` or any splitter drag ends
    it, and the sizes from before come back.
  - **Not saved.** Maximising is a moment's view (`ViewState`, never
    saved), and it sends no command. It is still disabled while the
    layout is locked.
- **RL14. File drops.** A file dropped on the window away from a table goes
  to the last player used, as today. When that player is hidden, it goes to
  the first shown player instead.

## 5. Persistence (`fp-store`)

- **RL15. Session document.** The session document gains `layout`, written
  with `#[serde(default)]`, so `SESSION_SCHEMA` stays 1.
  - An older session loads with RL1's default, which is today's layout.
  - Loading is lenient: shares of unknown players are dropped, unknown
    hidden players are dropped, and an unknown dock loads as `Bottom`.
    Each repair writes a log line.
  - A layout that hides every panel loads with every panel shown.
  - Reset (RL8) and every layout command mark the session for saving, as
    the cartwall's `open` does.

## 6. Remote API, MIDI and documentation

- The remote API and MIDI do not change. The layout is the local
  window's arrangement, and hiding a panel changes nothing they control
  (RL6).
- **User guide:**
  - `docs/user/` gains a "Layout" section: splitters, docking, hiding,
    the on-air dot, maximising, the lock and the reset;
  - the cartwall page mentions the dock and that buttons scale.
- **Technical docs:**
  - `docs/technical/ui.md` describes the layout pass;
  - `docs/technical/persistence.md` describes the session's `layout`.
- New strings go into every locale.
- The README and guide screenshots use the default layout, which does not
  change.

## 7. Tests

- **Model**, one test per rule:
  - RL1 and RL2: the default layout; shares of new, removed and hidden
    players; bad shares.
  - RL3 and RL4: neighbours only; a resize is rejected for a stranger or
    a non-neighbour; equal widths.
  - RL5: a dock change resets the size; the clamp.
  - RL6: hiding the last panel is rejected; hiding a player that plays
    keeps it playing.
  - RL7: locked rejects every layout command except the lock and the
    reset. RL8: reset keeps the lock.
- **Store** (RL15): an old session without `layout`; unknown players;
  an unknown dock; everything hidden.
- **Interface** (`egui_kittest`):
  - RL9: dragging a player splitter changes both widths and sends one
    command at the end; a double-click sends equal widths; nothing reacts
    while locked; the minimums hold.
  - RL10: buttons grow with the panel; the grid scrolls at the minimum;
    at the bottom with `None`, the rects are today's.
  - RL11: a collapsed side strip.
  - RL12: the toggles, the on-air dot, the menu disabled while locked.
  - RL13: maximise and restore; a drag ends it.
  - RL14: a drop with a hidden last player.
