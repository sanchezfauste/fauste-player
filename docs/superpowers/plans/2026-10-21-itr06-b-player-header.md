# itr06 plan B: player header

Goal: the player header never overlaps, a not-normalized track reads "0.0 dB"
in amber, the BP badge appears only on a bit-perfect device, and the fade
countdown sits on the button that started the fade.

## Tasks

1. **I13, BP badge.** `ui/player.rs` draws the BP/DSD badge only when
   `fp_model::bit_perfect_player(state, id) || pv.bit_perfect || pv.dsd`.
   Test: `tests/bit_perfect_ui.rs`. Docs: `bit-perfect.md`, `players.md`,
   bit-perfect spec B5, main spec 8.3.
2. **I12, header fit.**
   - `view::gain_badge`: every not-normalized status is `badge-gain` "0.0 dB",
     amber, reason in the tooltip; `badge-not-normalized` is removed from all
     locales. Test: `tests/view.rs`.
   - The gain badge has a fixed width and a border in its tone.
   - New pure `ui/header_layout.rs::fit` (measured widths and priorities in,
     which items stay out): the lowest-priority items collapse first. The
     header measures, fits, and draws left to right with the right group
     reserved. Unit tests for `fit`; kittest at the minimum width in es-ES and
     de-DE with every badge on, no two header rects intersect.
   - Docs: `loudness.md`, `players.md`, loudness spec N18.
3. **I1, fade countdown.** The bar is drawn on the Next/Play tile when
   `pv.fading && !fade_stopping`, on Fade stop for a fade stop (a fade stop
   during a crossfade moves it). Tests: `tests/fade_countdown_ui.rs`.
   Docs: main spec 8.3, `players.md`.

No new strings; `badge-not-normalized` is deleted.
