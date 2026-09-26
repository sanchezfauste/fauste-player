## What and why

<!-- One paragraph: the behaviour that changes and the reason. Link the spec
section or plan task it implements. -->

## How it was verified

- [ ] Tests written first and watched failing (TDD); new behaviour is covered
- [ ] `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` pass
- [ ] Real-time rules kept (no allocation, lock, log or panic on audio threads)
- [ ] UI strings added to both `en-US` and `es-ES`
- [ ] Docs updated (`docs/user`, `docs/technical`, README) where behaviour changed

## Notes for the reviewer

<!-- Rulings made, trade-offs, anything deliberately left out. -->

<!-- The PR title must be a Conventional Commit, e.g. "feat(engine): add ...". -->
