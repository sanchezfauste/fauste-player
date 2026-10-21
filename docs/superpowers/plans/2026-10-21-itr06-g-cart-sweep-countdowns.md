# itr06 G: cart sweep, fade origin, fader overshoot

Goal: four small UI fixes (I19, I20, I22, I23). One commit per item, test first.

## I23 Fader overshoot
- `crates/fp-app/src/ui/widgets.rs`: `FaderDrag.travel` stays unclamped; the
  knob and the sent volume use the clamped value, sent only when it changes.
- Test (`tests/fader_ui.rs`): drag 200 px past an end and 50 px back keeps the
  limit; further back follows without offset. Top and bottom.

## I19 and I22 Cart position line
- Check the sweep direction and what the cartwall draws (`cart_view.rs`).
- Diagnose the stutter: snapshot cadence and who requests repaints.
- The cartwall requests a repaint every frame while a cart plays; the drawn
  position is extrapolated from the last snapshot and the frame clock
  (clamped to the cart length, never backwards except on a restart or seek),
  drawn sub-pixel.
- Tests: cartwall alone and maximised keep repainting; the position advances
  between snapshots.

## I20 Fade origin
- `fp-model`: runtime-only `FadeOrigin { Next, Previous, Segue, FadeStop }` in
  the player state, set by the reducer where a fade starts; one test per rule.
- UI: the countdown is drawn on the button matching the origin (segue stays on
  Next). Spec and `docs/user/players.md` updated.
