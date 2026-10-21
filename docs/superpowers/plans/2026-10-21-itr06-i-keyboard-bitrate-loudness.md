# itr06 plan I: playlist keyboard, bit rate, loudness tooltip, normalization on by default

Goal: four small operator-facing changes (backlog I24-I27).

## I27 Normalization on by default
- `LoudnessConfig::default().normalize = true` (`crates/fp-model/src/config.rs`).
- Tests: default is on; an explicit saved `false` stays off; a missing field takes the default.
- Docs: loudness spec D11/Q1, `docs/user/loudness.md`, `docs/user/settings.md`.

## I25 Bit rate
- `AudioFormat::bitrate: Option<u32>` (bit/s, serde default), read with lofty in `fp-analysis`; none for DSD.
- No `ANALYSIS_VERSION` bump: older cached formats show none until re-read.
- Shown in the row tooltip format line, player track info and cart info; `TrackDto.format.bitrate`.
- Tests: analysis of a generated WAV; tooltip text. Strings in all 11 locales.
- Docs: remote API, user guide, spec.

## I26 Gain tooltip = loudness editor facts
- One shared function builds the gain lines for the badge tooltip and the loudness editor, including the true peak.
- Test: both list the same lines.

## I24 Playlist keyboard
- Up/Down move the active player's selection (scroll into view); Enter sets Next; a second Enter on the same
  entry within egui's `max_double_click_delay` plays. Inactive with a focused text field or a modal; user shortcuts win.
- kittest tests for each rule. Docs: players guide, shortcuts, spec.
