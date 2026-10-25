# CLAUDE.md — guide for AI agents working on Fauste Player

This is the **canonical** guide for agents (Claude Code and any other).
[`AGENTS.md`](AGENTS.md) points here, and the two must never diverge: edit
this file and keep `AGENTS.md` a pointer.

Fauste Player is a desktop radio playout application in Rust (Linux,
Windows, macOS), with an Android app in development on the same core
(`docs/technical/mobile.md`). Read [`README.md`](README.md) for the overview and
[`docs/technical/README.md`](docs/technical/README.md) for how it works.

## Non-negotiable rules

1. **Language.**
   - All code, identifiers, comments, docs, specs, plans and commit messages
     are in **English**.
   - Answer the maintainer in the language they write in (usually Spanish).
   - UI strings are Fluent messages; `crates/fp-app/locales/en-US/main.ftl`
     is the source. Every new or changed string goes into every locale file
     (en-US and es-ES written with each change, the others translated in
     bulk with AI).
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
   - `unsafe_code` is forbidden. The one exception is `fp-mobile`
     (Android only, mobile spec §8): it denies `unsafe_code` and allows it
     only at its Android entry points (`android_main`, the JNI calls),
     each with `#[allow(unsafe_code)]` and a `// SAFETY:` comment. No other
     crate may depend on it; `scripts/check-mobile-isolation.sh` checks.
   - `unwrap`, `expect` and `panic` are denied outside tests.
   - `fp-engine`, `fp-backends`, `fp-decode`, `fp-analysis` and `fp-dsp` deny
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
11. **Licence.** The project is `GPL-3.0-or-later` plus the attribution term
    in `NOTICE` (section 7(b)): the About window must keep showing the author
    attribution with its link. New crates inherit `license.workspace = true`.

## Commands

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace                        # ~1 min; no audio device or display needed
cargo test -p <crate> --test it <file>::<name>  # focused run (one binary per crate; see docs/technical/testing.md)
cargo deny check                              # after dependency changes
cargo llvm-cov --workspace --html             # test coverage (CI sends it to SonarQube Cloud; see docs/technical/testing.md)
cargo run -p fp-app                           # run the app (FAUSTE_HOME=<dir> for a scratch state)
FAUSTE_HOME=/tmp/fp-demo cargo run -p fp-app --example demo_session -- <music dir> [cart dir]
scripts/package-release.sh <target>           # release archive for one target
scripts/package/linux.sh <target>             # .deb, .rpm and AppImage (Windows: windows.sh, macOS: macos.sh)
scripts/package/msix.sh <target>              # Microsoft Store MSIX (Windows, Git Bash, Windows SDK)
scripts/package/flathub-manifest.py <tag> <commit> <out>  # the Flathub manifest of a release (PyYAML)
scripts/package/aur.sh <version> <out>        # the AUR PKGBUILD of a published release
scripts/package/notices.sh <out> [features]   # THIRD-PARTY.html with cargo-about (the snap, Flatpak and Flathub builds)
scripts/package/icons.sh                      # regenerate the PNG and .ico icons from packaging/icons/fauste-player.svg (inkscape, magick)
scripts/check-commits.sh origin/master        # commit subjects vs Conventional Commits
scripts/site/build.sh [out]                   # website: landing page + mdBook guide (default target/site; needs gh or FAUSTE_RELEASE_JSON for the download links)
scripts/site/check-links.sh <out>             # dead links in the built site
scripts/site/check-translations.sh [out]      # translated guides (docs/i18n) mirror docs/user and carry the AI notice; landing translations (site/locales) match en.toml
python3 scripts/site/landing.py hash          # the [source-hash] of the landing page's English texts, for its translations
python3 -m unittest discover -s scripts/site/tests -p 'test_*.py' && node --test 'scripts/site/tests/*.test.js'  # site script tests
python3 scripts/site/guide.py changed [lang]  # English guide pages changed since each translation
scripts/site/screenshots.sh [--only main] [--lang es-ES] [--out <dir>]  # README and guide screenshots from a scripted scene (Xvfb; see Testing notes)
scripts/site/localized-screenshots.sh <dir> [lang…]  # the same in English and every guide language (CI does it on publish)
scripts/site/compare-screenshots.sh [--fuzz 5] [--threshold 50] [--masks f] [--apply] [--prune] <new-dir> [docs/images]  # which screenshots really changed (the screenshots workflow uses it)
scripts/package/android.sh <debug|release>     # Android APK (cargo-ndk, NDK r27, JDK 17+; see docs/technical/mobile.md)
scripts/check-mobile-isolation.sh             # the fp-mobile unsafe exception stays isolated (CI runs it)
scripts/prune-target.sh [minutes]             # free disk: drop test binaries and caches unused for 60 min
cargo test --release -p fp-analysis --test it real_music:: -- --ignored   # real-music corpus (local only)
cargo run --release -p fp-analysis --example marker_report -- [--set key=value]… [dir]
cargo run -p fp-engine --example rate_cut_tail -- [card] [buffer]   # rate-cut play-out on a real ALSA device (local only; quiet tones)
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

### Finding code

When a local CodeGraph index exists (`.codegraph/` at the repo root,
git-ignored; build it with `codegraph init`), use it before grep or reading
files: the `codegraph_explore` MCP tool (pass this repo as `projectPath`) or
`codegraph explore "<symbols or question>"` returns the relevant source and
the call paths between symbols. `vendor/` and `target/` are not indexed.

### Pull requests

Every change reaches `master` through a pull request on GitHub; nothing is
merged locally.

1. Branch from an up-to-date `master`: `feat/…`, `fix/…`, `docs/…`,
   `ci/…`, `chore/…`.
2. Commit with fmt, clippy and the whole suite green (see Commands).
3. Push the branch and open a PR with `gh pr create`, following
   `.github/pull_request_template.md`. The title is a Conventional Commit
   summary.
4. Wait for CI on all three OSes (`gh pr checks --watch`). A red check is
   fixed on the branch, never bypassed.
5. When the plan's review is done and CI is green, merge with
   `gh pr merge --merge --delete-branch`, then update the local `master`.

The maintainer has authorised pushing branches and merging green,
reviewed PRs.

### Documentation is part of every change

A change is not done until everything that describes it says the same:

- `README.md` (features, platform table, install, roadmap);
- the user guide in `docs/user/` (English only: the translations in
  `docs/i18n/` are regenerated with AI before each release, never in a
  feature PR; the one exception is a PR that moves, renames or deletes an
  image in `docs/images`, which also updates the image links in
  `docs/i18n/`) and the technical docs in `docs/technical/`;
- the spec in `docs/superpowers/specs/` when behaviour changes, and the plan;
- this file (and `AGENTS.md` if the pointer changes) when the workflow,
  commands or layout change;
- the landing page and privacy policy texts in every `site/locales` file:
  `en.toml` is the source, and a change to it updates every translation
  in the same PR (with AI) and its `[source-hash]`
  (`python3 scripts/site/landing.py hash`); `check-translations.sh` fails
  otherwise. A new guide language also needs its `site/locales/<lang>.toml`;
- UI strings in every locale file (en-US and es-ES written with each
  change, the rest translated in bulk with AI); a new locale is one
  `locales/<tag>/main.ftl` plus one entry in `i18n::LOCALES` (see `docs/technical/ui.md`).

The PR template has a checklist for it. `CHANGELOG.md` is never edited by
hand: release-please writes it from the commits.

## Commits and releases

- Conventional Commits: `type(scope)!: summary`. Types: `feat`, `fix`,
  `perf`, `refactor`, `docs`, `test`, `build`, `ci`, `chore`, `style`,
  `revert`. Scopes are crate or area names (`deps`, `model`, `store`, `engine`,
  `backends`, `decode`, `dsp`, `analysis`, `app`, `ui`, `ci`, `docs`).
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
| `crates/fp-decode` | File decoding (symphonia, DSD, WavPack, Monkey's Audio, Opus) and downmix |
| `crates/fp-dsp` | K-weighting, true peak, BS.1770 gating: pure signal processing shared by the engine and the analysis |
| `crates/fp-backends` | `AudioBackend` trait, cpal, Null, Offline |
| `crates/fp-engine` | Mixer, sources, buses, workers, engine, conductor |
| `crates/fp-analysis` | Metadata, signal analysis, cache, pool |
| `crates/fp-control` | MIDI control surfaces: parsing, bindings, soft takeover, LED feedback, learn, the MIDI thread |
| `crates/fp-remote` | Remote control over the network: HTTP/JSON API, SSE events, OSC, security guard, the remote thread |
| `crates/fp-app` | UI (`src/ui/*`), services thread, session start-up (`session.rs`), the platform seam (`platform.rs`), bootstrap, logging, crash reports, locales, fonts |
| `crates/fp-mobile` | The Android library: `android_main`, IME bridge, safe area, Android platform, JNI calls (the only crate with the `unsafe` exception) |
| `mobile/android` | The Android shell (Gradle, Kotlin): the activity, the playout service, the JNI declarations |
| `docs/superpowers/specs` | The binding design spec |
| `docs/superpowers/plans` | Implementation plans (one per step of a phase) |
| `docs/user`, `docs/technical` | User and technical documentation. Keep them in sync with behaviour. |
| `docs/i18n/<lang>`, `docs/book-assets` | The guide's AI translations (regenerated before each release, never in feature PRs; see `docs/technical/release-process.md`) and the language menu and notice script |
| `vendor/opus-decoder` | A patched copy of `opus-decoder` (a real FFT), used through `[patch.crates-io]`; excluded from the workspace. See its `VENDORED.md` |
| `vendor/symphonia-format-mkv` | A patched copy of symphonia's Matroska reader (seeking through cues, blocks without duration), used the same way. See its `VENDORED.md` |
| `vendor/egui-phosphor` | A copy of `egui-phosphor` 0.14.0 with only its egui requirement raised to 0.37 (no upstream release yet), used the same way. See its `VENDORED.md` |
| `vendor/winit` | A patched copy of `winit` 0.30.13 (Android: the event loop ends when the activity is destroyed and a new one may start), used the same way. See its `VENDORED.md` |
| `packaging/`, `scripts/package/` | Icons, desktop entry, AppStream, Flatpak, WiX, MSIX and Info.plist, the AUR PKGBUILD template; the per-format package scripts |
| `snap/` | The snap recipe (`snapcraft.yaml`, where snapcraft looks for it) |
| `site/`, `scripts/site/` | The landing page and privacy policy templates with their texts in `site/locales/<lang>.toml` (rendered by `scripts/site/landing.py`), and the scripts that build the website with the user guide (mdBook, pinned in `scripts/site/mdbook.sh`) |
| `.github/workflows` | CI (with the SonarQube Cloud analysis and coverage; `sonar-project.properties`), Dependabot (`dependabot.yml`), release-please, release builds and packages, store publishing (`stores.yml`: Microsoft Store, Snap Store, Flathub, WinGet, AUR), GitHub Pages, commit checks |

## Design source

The UI follows the "Reproductor v3" design (the Nocturne design system). It
lives in the Claude Design project `ab44062f-ab67-4424-ae98-ed3e39f38ff8`, file
`Reproductor v3.dc.html`. Read it through the `claude_design` MCP server
(`get_project`, `read_file`). Colours given there in `oklch` are converted
once to sRGB constants in `crates/fp-app/src/ui/theme.rs`.

## Testing notes

- Integration tests: one binary per crate. A new test file goes into
  `crates/<crate>/tests/it/` with a `mod` line in `tests/it/main.rs`; only a
  test that installs process-wide state (a panic hook, a global tracing
  subscriber) gets its own file in `tests/`. Every test file is its own
  link otherwise, and `target/` grows by gigabytes per test run.
- Engine tests use the `Offline` backend and drive time explicitly
  (`now: Instant`). Never sleep to wait for audio.
- Real music: `test-music/` (git-ignored except its README; or
  `FAUSTE_TEST_MUSIC=<dir>`) feeds the `#[ignore]` tests in
  `fp-analysis/tests/it/real_music.rs` and the `marker_report` example. They
  never run in CI and pass with a note when the folder is empty.
- UI tests use `egui_kittest`, with `Harness::builder().with_step_dt(0.02)`
  (double clicks) and the recording `Fake` controller in
  `crates/fp-app/tests/it/support`.
- Screenshots (Linux): `scripts/site/screenshots.sh` takes the README
  image and every guide screenshot (`--only main` for the README image
  alone). It builds the release binary and `examples/demo_session`,
  generates stand-in songs and carts with `scripts/site/tones.sh` (pink
  noise and a sine, each with its own envelope, tagged and with a cover),
  writes a scratch `FAUSTE_HOME` (`SHOTS_HOME`; English, the remote API on,
  loudness normalization on, every output on the silent `null` backend),
  runs the app in Xvfb on a free display, waits for the loudness of the
  players' tracks and starts the app again (so the headers show each
  track's gain), checks in the log that the null backend is in use before
  anything plays, builds the scene through the remote API and captures
  with ImageMagick. Needs `xvfb`, `xdotool`, ImageMagick, `ffmpeg`,
  `python3` and `curl`; the songs are kept in `target/screenshots` and
  reached through the link `/tmp/fauste-demo` (`SHOTS_MEDIA`), so no
  screenshot shows a home path.
  `--hold` keeps the app running for a look (`DISPLAY=:<n> import -window
  <id> shot.png`). `--lang <locale>` sets the interface language (a tag of
  `crates/fp-app/locales`; the scene's own text stays) and `--out <dir>`
  writes `main-screen.png` and `guide/*.png` there instead of `docs/images`.
  The crops follow the default 1920×1080 layout: after a
  layout change, check every PNG and adjust the positions in the script.
  Windows and menus whose size depends on the language (About, the tag
  editor, the CUE window, the track menu) and the top bar's buttons are
  found on the screen (`scripts/site/window_box.py`), so check a long
  language too (`--lang de-DE`).

  Nobody retakes `docs/images` by hand: the `screenshots` workflow
  (`.github/workflows/screenshots.yml`) runs on a push to `master` that
  touches `crates/fp-app/src/ui`, the locales, `demo_session.rs` or the
  screenshot scripts, and by hand (`workflow_dispatch`). It takes the
  English set with `screenshots.sh --out`, keeps only the images that
  differ (`compare-screenshots.sh`: ImageMagick `compare -metric AE -fuzz`,
  so antialiasing noise is ignored) and opens or updates one PR from
  `docs/retake-screenshots` (it closes that PR when nothing differs any
  more). The clock, elapsed times, meters and the cart sweep change on
  every run: `scripts/site/screenshot-masks.txt` lists the rectangles
  painted on both images before comparing (update it after a layout
  change; two runs of `screenshots.sh --out` must compare equal). With
  `GITHUB_TOKEN` the repository setting "Allow GitHub Actions to create
  and approve pull requests" must be on, and the PR starts no CI (close
  and reopen it); the optional `SCREENSHOTS_TOKEN` secret avoids both.
  Review the images before merging. The translated guides' screenshots are
  taken at publish time and never committed.

  On a GNOME Wayland desktop, `xdotool` clicks into XWayland windows need
  the "remote interaction" permission every session; Xvfb avoids that.
- Tagged test audio without encoders: WAV files with a RIFF `LIST/INFO`
  chunk (`INAM` title, `IART` artist) written from Python are read by the
  analysis like any tagged file.
- Remote API tests drive the axum router with `tower::ServiceExt::oneshot`
  (no sockets) and the recording `FakeControl` in
  `crates/fp-remote/tests/it/support`; server tests bind `127.0.0.1` on free
  ports, and OSC tests use UDP sockets on `127.0.0.1:0`.
