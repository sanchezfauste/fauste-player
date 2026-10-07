# AGENTS.md

The canonical guide for AI agents working on this repository is
**[`CLAUDE.md`](CLAUDE.md)**. Read it before changing anything; this file only
repeats the essentials for agents that do not follow links.

- English for all code, comments, docs, specs, plans and commits; answer the
  maintainer in their language. UI strings go through Fluent; `en-US` is
  the source and every string goes into every locale file.
- Never mention other playout or radio-automation products anywhere.
- Use established names; do not invent terms.
- No `unsafe`. No `unwrap`, `expect` or `panic` outside tests. No hardcoded
  product limits (use `Config`).
- The real-time mixer path never allocates, frees, locks, logs or panics.
  The UI thread never blocks.
- Test first (TDD). Before committing, run `cargo fmt --all`,
  `cargo clippy --workspace --all-targets -- -D warnings` and
  `cargo test --workspace`.
- Conventional Commits and SemVer; release-please owns versions and the
  changelog.
- Keep `docs/user` and `docs/technical` in sync with behaviour. Change the
  guide in English only; its translations in `docs/i18n` are regenerated
  before each release.

If this file and `CLAUDE.md` ever disagree, `CLAUDE.md` wins; fix this file.
