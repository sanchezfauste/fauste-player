# CLAUDE.md — guide for AI agents working on Fauste Player

This is the **canonical** guide for agents (Claude Code and any other).
[`AGENTS.md`](AGENTS.md) points here, and the two must never diverge: edit
this file and keep `AGENTS.md` a pointer.

Fauste Player is a desktop radio playout application in Rust (Linux,
Windows, macOS). Read [`README.md`](README.md) for the overview and
[`docs/technical/README.md`](docs/technical/README.md) for how it works.

## Non-negotiable rules

1. **Language.**
   - All code, identifiers, comments, docs, specs, plans and commit messages
     are in **English**.
   - Answer the maintainer in the language they write in (usually Spanish).
   - UI strings are Fluent messages in `crates/fp-app/locales/en-US/main.ftl`
     (the source) and `es-ES/main.ftl`, always both.
2. **Confidentiality.** Never mention other playout or radio-automation
   products (commercial or open source) anywhere: code, comments, docs,
   specs, plans, commit messages, issues, PRs. Describe behaviour in its own
   terms.
3. **Conventions over invention.** Use established, widely accepted names:
   audio (source, bus, mixer, cue/PFL, segue, cue-in/cue-out, intro/outro,
   xrun), Rust API guidelines, Conventional Commits, SemVer, Keep a Changelog.
   Do not invent terms for concepts that already have names.
4. **No hardcoded product limits.** Anything an operator might change is a
   `Config` field (`crates/fp-model/src/config.rs`) with a documented
   default, a range in `Config::validate`, and lenient loading. Engine
   capacities are derived from configuration.
5. **Real-time safety.** Code that runs on the device callback (the mixer and
   the backend render path) never allocates, frees, locks (only `try_lock` on
   the bus mixer), logs, does I/O or panics. Memory enters through
   `BusCommand` and leaves through `Retired`.
6. **Safety lints.**
   - `unsafe_code` is forbidden.
   - `unwrap`, `expect` and `panic` are denied outside tests.
   - `fp-engine`, `fp-backends`, `fp-decode` and `fp-analysis` deny
     `clippy::indexing_slicing` (use `get`); prefer `get` everywhere.
   - Test-only hooks go behind the `test-hooks` feature of `fp-app`.
7. **Behaviour lives in `fp-model`.** Player rules are pure functions
   (`apply`, `on_event`, `plan_for`, `reconcile`), one test per rule in spec
   §3. The engine and the UI only execute and display.
8. **The UI never blocks.** No file-system scans, dialogs, device
   enumeration or waits on the UI thread; use helper threads and channels.
   The UI only reads snapshots and sends commands.
9. **Never crash on bad data.** Untrusted inputs (audio, tags, covers,
   state files) degrade to "not available" with a log line.
10. **Nothing goes on air by itself** after a start, restart or crash.

## Commands

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace                        # ~1 min; no audio device or display needed
cargo test -p <crate> --test <file> <name>    # focused run
cargo deny check                              # after dependency changes
cargo run -p fp-app                           # run the app (FAUSTE_HOME=<dir> for a scratch state)
FAUSTE_HOME=/tmp/fp-demo cargo run -p fp-app --example demo_session -- <music dir>
scripts/package-release.sh <target>           # release archive for one target
scripts/package/linux.sh <target>             # .deb, .rpm and AppImage (Windows: windows.sh, macOS: macos.sh)
scripts/check-commits.sh origin/master        # commit subjects vs Conventional Commits
```

Only commit when fmt, clippy and the whole test suite pass, for example:

```sh
if cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings \
   && cargo test --workspace; then git commit ...; fi
```

## Workflow and skills

The project uses the **superpowers** skills in Claude Code. Invoke a skill
whenever it applies:

| When | Skill |
|---|---|
| New feature, phase or behaviour change | `superpowers:brainstorming` → update the spec in `docs/superpowers/specs/` |
| Turning a spec into work | `superpowers:writing-plans` → `docs/superpowers/plans/YYYY-MM-DD-phaseN-planM-<topic>.md` (English) |
| Implementing a plan | `superpowers:executing-plans` (inline, the default here) or `superpowers:subagent-driven-development` |
| Any code | `superpowers:test-driven-development`: watch every new test fail first |
| Any bug or unexpected output | `superpowers:systematic-debugging` |
| Before saying "done" | `superpowers:verification-before-completion` |
| End of a plan | `superpowers:requesting-code-review` with a fresh reviewer on the most capable model; fix Critical and Important findings test-first, and defer the Minor ones in the ledger |
| Merging | `superpowers:finishing-a-development-branch` |

Progress ledgers live in `.superpowers/sdd/<plan>/progress.md` (git-ignored).
Record every deviation from a plan as
`Ruling: <decision> — <why> — <cost if wrong>`.

Branches: `feat/…`, `fix/…`, `docs/…` from `master`. The maintainer has
authorised merging finished, reviewed plans into `master` locally. **Never
push** unless asked.

## Commits and releases

- Conventional Commits: `type(scope)!: summary`. Types: `feat`, `fix`,
  `perf`, `refactor`, `docs`, `test`, `build`, `ci`, `chore`, `style`,
  `revert`. Scopes are crate or area names (`model`, `store`, `engine`,
  `backends`, `decode`, `analysis`, `app`, `ui`, `ci`, `docs`).
- The body explains *why*. Agent commits end with the co-author trailer the
  harness provides.
- SemVer. release-please owns `CHANGELOG.md`, the version in `Cargo.toml` and
  `Cargo.lock`, and `.release-please-manifest.json`. Never edit them by hand
  outside a release PR. See `docs/technical/release-process.md`.

## Where things are

| Path | Content |
|---|---|
| `crates/fp-model` | State, commands, reducer (rules), config, sessions |
| `crates/fp-store` | Persistence, backups, migrations, lenient config |
| `crates/fp-decode` | symphonia decoding and downmix |
| `crates/fp-backends` | `AudioBackend` trait, cpal, Null, Offline |
| `crates/fp-engine` | Mixer, sources, buses, workers, engine, conductor |
| `crates/fp-analysis` | Metadata, signal analysis, cache, pool |
| `crates/fp-app` | UI (`src/ui/*`), services thread, bootstrap, logging, crash reports, locales, fonts |
| `docs/superpowers/specs` | The binding design spec |
| `docs/superpowers/plans` | Implementation plans (one per step of a phase) |
| `docs/user`, `docs/technical` | User and technical documentation. Keep them in sync with behaviour. |
| `packaging/`, `scripts/package/` | Icons, desktop entry, AppStream, Flatpak, WiX and Info.plist; the per-format package scripts |
| `.github/workflows` | CI, release-please, release builds and packages, commit checks |

## Design source

The UI follows the "Reproductor v3" design (the Nocturne design system). It
lives in the Claude Design project `ab44062f-ab67-4424-ae98-ed3e39f38ff8`, file
`Reproductor v3.dc.html`. Read it through the `claude_design` MCP server
(`get_project`, `read_file`). Colours given there in `oklch` are converted
once to sRGB constants in `crates/fp-app/src/ui/theme.rs`.

## Testing notes

- Engine tests use the `Offline` backend and drive time explicitly
  (`now: Instant`). Never sleep to wait for audio.
- UI tests use `egui_kittest`, with `Harness::builder().with_step_dt(0.02)`
  (double clicks) and the recording `Fake` controller in
  `crates/fp-app/tests/support`.
- Screenshots for a visual check (Linux, X11):
  1. run `env -u WAYLAND_DISPLAY FAUSTE_HOME=<dir> target/release/fauste-player`;
  2. find the window with `xwininfo -name "Fauste Player"`;
  3. capture it with `import -window <id> shot.png`.
