# itr06 plan F: panel header icons and full player names in the top bar

**Goal.** Each player header and the cartwall header get two icon buttons at
their right end, Hide (minus) and Maximize or Restore (corners out or in), with
the actions and the disabled rules of the header menu (RL16). The top-bar
panel tiles read "Player 1", "Player 2"… instead of "P1".

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
- Tests: `layout_ui.rs` (click, disabled states, restore, cartwall),
  `header_overlap_ui.rs` (the buttons never overlap at `MIN_PLAYER_WIDTH`
  in en-US, es-ES and de-DE).

## I17. Full names in the top bar

- New message `panel-tile-player = Player { $n }` in the 11 locales, used by
  the top-bar tile only (the column header keeps `P{ $n }`).
- Tests that look tiles up by "P1"/"Hide P1"/"Show P1 (on air)" follow.
- `scripts/site/screenshots.sh` checked for tile text.

## Docs

Spec RL15/RL16, `docs/user/layout.md`, `docs/technical/ui.md` if it describes
the headers.
