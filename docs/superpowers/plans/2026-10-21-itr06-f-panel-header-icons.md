# itr06 plan F: panel header icons and top-bar alignment

**Goal.** Each player header and the cartwall header get two icon buttons at
their right end, Hide (minus) and Maximize or Restore (corners out or in), with
the actions and the disabled rules of the header menu (RL16). Every top-bar
tile and button shares one height and one vertical centre line (I21).

**Spec.** `docs/superpowers/specs/2026-10-21-resizable-layout-design.md`
RL15 (tiles) and RL16 (header buttons).

## I16. Header buttons

- `ui/panels.rs`: `can_hide`, `can_maximise` shared by the menu and a new
  `header_buttons` (drawn right to left: Maximize, then Hide), `HEADER_BUTTONS_WIDTH`.
  Phosphor `MINUS`, `CORNERS_OUT`, `CORNERS_IN`. Tooltip and accessible name
  are the existing `layout-hide`, `layout-maximise`, `layout-restore`; a
  disabled button's tooltip gives the reason (`error-layout-locked`,
  `error-last-panel`) when there is one.
- `ui/player.rs`: the buttons join the never-collapsed right group and its
  measured width, so `header_layout::fit` leaves room for them.
- `ui/cartwall.rs`: the buttons at the right end, left of nothing; "Stop all"
  moves left of them; the caret and the right-click menu stay. The cartwall's
  least width counts them.
- The status text's least width in a player header drops from 40 to 30 points
  so that, at `MIN_PLAYER_WIDTH`, the state text still stays beside the buttons.
- Tests: `layout_ui.rs` (click, disabled states, restore, cartwall),
  `header_overlap_ui.rs` (the buttons never overlap at `MIN_PLAYER_WIDTH`
  in en-US, es-ES and de-DE).

## I17. Dropped

The maintainer dropped it: the top-bar tiles keep "P1", "P2"… (long labels
do not fit with many players).

## I21. Top-bar alignment

The panel tiles and the layout button share the height and centre line of
the buttons on the right: `panels::layout_controls` draws in a centred layout
(a `ui.horizontal` row put the tiles 3 points low). Test in `layout_ui.rs`
compares the rects with 3 and 30 players.

## Docs

Spec RL15/RL16, `docs/user/layout.md`, `docs/technical/ui.md` if it describes
the headers.
