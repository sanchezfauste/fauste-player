# Contributing

Thank you for helping. This page is the short version; [`CLAUDE.md`](CLAUDE.md)
has the full rules, which apply to people and agents alike.

## Workflow

1. **Discuss first** for anything beyond a small fix: open an issue with the
   *Feature request* template. Behaviour changes update the
   [design spec](docs/superpowers/specs/2026-09-25-fauste-player-design.md).
2. **Branch** from `master`: `feat/<topic>`, `fix/<topic>`, `docs/<topic>`.
3. **Test first.** Write the failing test, watch it fail for the right
   reason, then make it pass. A bug fix starts with a test that reproduces it.
4. **Check locally:**
   ```sh
   cargo fmt --all
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   cargo deny check        # when dependencies change
   ```
5. **Update the docs** in the same PR: README, the user guide
   (`docs/user/`), the technical docs (`docs/technical/`), the spec when
   behaviour changes, and `CLAUDE.md` when the workflow changes.
6. **Commit** with [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).
   Enable the hook with `git config core.hooksPath .githooks`.
7. **Open a PR.** Its title is a Conventional Commit too, and the template
   lists what reviewers check. It is merged once CI is green on Linux,
   Windows and macOS.

## Code rules

- English everywhere in code, comments, docs and commits. Use established,
  widely accepted names for concepts; do not invent terms.
- No `unsafe`, and no `unwrap`, `expect` or `panic` outside tests.
- Nothing on the real-time path allocates, frees, locks, logs or panics.
- No hardcoded product limits: add a `Config` field with a default and a
  validated range instead.
- Every user-visible string goes into `locales/en-US/main.ftl` (the source)
  and every other `locales/<tag>/main.ftl`.
- Never mention other playout or radio-automation products in code, docs or
  commits.
- Update `docs/user` and `docs/technical` when behaviour changes.

## Releases

Maintainers merge the release PR maintained by release-please; see
[Release process](docs/technical/release-process.md).
