## What and why

<!-- One paragraph: the change and the reason. Link the spec and plan. -->

## How it was verified

<!-- Tests added (watched failing first), manual checks, platforms. -->

## Checklist

- [ ] Conventional Commit title; commits explain *why*
- [ ] `cargo fmt`, `cargo clippy -D warnings` and `cargo test --workspace` green locally
- [ ] New behaviour has tests that failed before the change
- [ ] Docs match the change: README, `docs/user/` (English only; `docs/i18n/` is regenerated before releases), `docs/technical/`, spec/plan, `CLAUDE.md`
- [ ] UI strings in every locale file (`en-US` is the source)
- [ ] No third-party playout product names anywhere
- [ ] CI green on Linux, Windows and macOS
