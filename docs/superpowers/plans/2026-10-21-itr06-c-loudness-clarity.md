# itr06 plan C: loudness clarity

Goal: make the per-track loudness editors say what the trim does, and document the basis of the loudness defaults.

## I4 gain lines
- `view::gain_lines` (pure, from `fp_model::normalization`, `trim_in_use`, `wanted_gain_db`); `loudness_edit::gain_text` draws it in the popup and as the Trim row hint of the cart editor.
- Strings: `loudness-auto-gain`, `loudness-trimmed-gain`, `loudness-no-gain-off/-excluded/-unmeasured/-silent`, `loudness-off-note` in all 11 locales.
- Tests in `crates/fp-app/tests/loudness_ui.rs`.
- Docs: user guide "Per track", spec N20/N21.

## I6 standards
- Guide section "Standards behind the defaults", spec D5 rationale, config table in `docs/technical/persistence.md`.
- No Settings link: the app has no pattern for opening the guide.
