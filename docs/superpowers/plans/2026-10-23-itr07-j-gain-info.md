# itr07 plan J: one gain breakdown everywhere (J1)

Goal: the maintainer wants the gain tooltip to be a clear multi-line popup
with all the relevant facts and the resulting gain at the end, looking the
same everywhere the gain is explained.

## Tasks

1. **Pure builder** (`crates/fp-app/src/ui/view.rs`): `gain_info(track,
   config, bit_perfect, actual) -> Vec<InfoLine>` replaces `loudness_facts`,
   `gain_lines`, `loudness_lines` and `limit_note`. Order: integrated
   loudness, true peak, target, automatic gain (or why none), trim, gain
   wanted and limit, notes, resulting gain (always last). `GainBadge::tip`
   becomes `Vec<InfoLine>`.
2. **One renderer** (`ui/loudness_edit.rs`: `info`, `info_block`): two
   columns, tabular digits, notes across, the result under a rule in
   semibold, wrapping for long locales and narrow windows. Used by the gain
   badge tooltip (`player.rs`, `on_hover_ui`; the tile is named by
   `widgets::info_text`), the Loudness popup (above its controls), the cart
   editor (`settings/carts.rs`) and the playlist row tooltip (`table.rs`).
3. **Strings** in all 11 locales: `info-*` added; `loudness-auto-gain`,
   `loudness-no-gain-*`, `loudness-trimmed-gain`, `tip-gain-limited-*` and
   `track-loudness*` removed.
4. **Tests**: pure tests in `tests/view.rs` (order, result last, limits,
   reasons, badge equals builder); kittest in `tests/loudness_ui.rs` (badge,
   editor and row tooltip show the same lines in order, de-DE at 420 px).
5. **Docs**: user loudness guide, `docs/technical/ui.md`, the loudness spec
   (N18-N21 amendment and string table).
