# itr07 plan K: visual polish (J2–J5)

Goal: the fade countdown sweeps left to right in the tone of its button, the
Play button becomes a filled green button with clearly defined states, and a
small polish pass makes the player column look more professional. No layout
that operators are used to moves.

## Tasks

1. **Sweep direction (J2).** The countdown band on Next/Play, Previous and
   Fade stop is anchored to the right edge and shrinks towards it, so its
   leading edge moves left to right like the cart sweep. Files:
   `ui/player.rs`. Tests: `tests/fade_countdown_ui.rs` pins the band's right
   edge on the button's right edge and its left edge moving right.
2. **Sweep tone (J3).** The band takes the tone of the button it is drawn
   on: `theme::play_sweep()` on Play/Next, the button's content tone at low
   strength on the neutral buttons (`theme::sweep_tone`). Tests pin the
   colour per button.
3. **Play button (J4).** Filled green with a white icon. Tokens in
   `ui/theme.rs` (`PLAY_FILL*`, `PLAY_ICON`, focus ring); `TileStyle` gains
   `disabled_fill`. States: idle, on air (Next), hover, pressed, disabled,
   keyboard focus. Tests: fill and icon colour per state, WCAG contrast of
   icon on fill >= 3:1 (also under the sweep band).
4. **Polish (J5).** Divider between the VU meter and the fader; keyboard
   focus ring on every tile; other small items listed in the PR and ledger.

## Docs

`docs/technical/ui.md`, the user guide page for the player, the spec
(`docs/superpowers/specs/`) where it describes the countdown colour/direction.
No new strings.
