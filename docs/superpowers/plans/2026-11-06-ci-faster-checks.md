# Faster CI and CD Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans
> to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax
> for tracking.

**Goal:** Cut the wall time of the CI checks and of the workflows that take
screenshots and build the website, without dropping any check, platform or
coverage report.

**Architecture:** Five independent changes to `.github/`: a composite action
that keeps the downloaded `.deb` archives in the Actions cache, cargo-nextest
for the test job, a docs-only filter on the Rust jobs of pull requests, a
narrower trigger for the release dry run, and a timeout on every job.

**Tech stack:** GitHub Actions, cargo-nextest with cargo-llvm-cov,
`dorny/paths-filter`, `actions/cache`.

**Spec:** none (no behaviour of the application changes). The measurements
below are the design input.

## Measurements (2026-11-05)

| Where | Time | Cause |
|---|---|---|
| `pages` build, "Install the screenshot tools" | 42.8 min | `Fetched 49.5 MB in 42min 28s (19.4 kB/s)`: the runner's Ubuntu mirror. `apt-get update` took 4 s |
| `screenshots` shots, same step | 39 min, and a second run cancelled after 38 min | same mirror |
| CI `test (macos-latest)` | 12 min, 9.5 of them running tests | test binaries run one after another: `fp-app` 5.4 min, `fp-engine` 2.4 min, one 23-test binary 70 s mostly waiting |
| CI `test (ubuntu-latest)` / `(windows-latest)` | 8.7 / 7.8 min | same, 7.7 / 6.2 min of tests |
| PR #212 (PNG files of the Play listing only) | full CI (12 min) and a full release dry run (dmg, msi, Linux packages) | no path filter |

Compilation is about 1.5 min with the Rust cache, so it is not the target.

## Decisions

1. **Cache the `.deb` archives** (`.github/actions/apt-install`). apt
   downloads into `~/.cache/apt-archives` (`-o Dir::Cache::Archives=`), and
   `actions/cache` keeps that folder. The key is
   `apt-<ImageOS>-<hash of the sorted package list>-<ImageVersion>`, with
   the key minus the image version as restore key: a new runner image
   restores the previous archives and downloads only what changed. apt still
   runs `update` and `install`, so maintainer scripts run and the versions
   are the mirror's current ones; the cache only saves the download. After
   the install the folder's `lock` and `partial/` are removed and it is
   handed to the runner user, so the cache can save it. Packages come in
   through an environment variable, never interpolated into the script.
   Every workflow that runs `apt-get install` uses it.
2. **cargo-nextest in the test job**: `cargo llvm-cov nextest --no-report`
   with the `ci` profile of `.config/nextest.toml`. All tests of all
   binaries run at once, each in its own process; the LCOV reports, the
   SonarQube job and the three systems stay as they are. There are no
   doctests in the workspace (`cargo llvm-cov` never ran doctests anyway).
   The profile:
   - `fail-fast = false` (as `--no-fail-fast` today);
   - `slow-timeout = { period = "60s", terminate-after = 10 }`: a hung test
     is killed after 10 minutes and named, instead of hanging the job;
   - no retries: a flaky test must fail visibly.
   Locally, `cargo test --workspace` stays the documented command;
   `cargo nextest run --workspace` is mentioned as the faster option.
3. **Docs-only pull requests skip the Rust jobs.** A `changes` job in
   `ci.yml` runs `dorny/paths-filter@v4` with `predicate-quantifier: every`.
   A pull request is docs-only when every changed file matches one of:
   `docs/**`, `**/*.md`, `site/**`, `scripts/site/**`,
   `packaging/android/play/**`. No Rust code reads those paths
   (`include_str!`/`include_bytes!` read only `crates/fp-app/assets` and
   `packaging/icons`), and `pages.yml` still checks the site and the guide
   on those pull requests. The jobs skipped are test, android, sonarqube,
   release-baseline, release-config and fuzz-build; `deny` (under a minute)
   and the commit checks always run. Pushes to `master` always run
   everything. The filter fails open: when it errors, `code` is `true`.
   Skipped jobs count as passed for the required checks of the
   `protect-master` ruleset, which is the point; a job-level `if` is used
   because a workflow-level `paths` filter would leave the required checks
   pending forever.
4. **The release dry run ignores the Play listing**: `release-build.yml`
   adds `'!packaging/android/play/**'` to its `pull_request` paths. No
   package is built from that folder (the stores workflow uploads it).
5. **Every job gets `timeout-minutes`**, about three times its usual
   length, so a stalled mirror, emulator or server fails the job in minutes
   rather than after GitHub's six hours.

| Workflow | Job | Usual | Timeout |
|---|---|---|---|
| ci | changes | <1 min | 5 |
| ci | test | 8–12 min | 45 |
| ci | android | 3.5 min | 30 |
| ci | sonarqube | 1 min | 15 |
| ci | release-baseline | 3 min | 30 |
| ci | deny | 1 min | 10 |
| ci | release-config | <1 min | 15 |
| ci | fuzz-build | <1 min | 20 |
| commits | both | <1 min | 5 |
| fuzz | fuzz | 7.5 min | 30 |
| pages | build | 15 min without the mirror | 60 |
| pages | deploy | <1 min | 10 |
| play-screenshots | shots | 24 min | 75 |
| play-screenshots | publish | 1 min | 15 |
| release-build | every job | 2–8 min | 60 |
| release-please | release-please | 3 min | 15 |
| screenshots | shots | 4 min without the mirror | 45 |
| screenshots | publish | <1 min | 15 |
| stores | snap | 26 min | 45 (already set) |
| stores | every other job | 0–6 min | 30 |

Jobs that only call a reusable workflow (`uses:`) cannot take a timeout;
the called jobs carry theirs.

**Not done:** dropping the coverage on Windows or macOS (it counts the code
built only there), larger paid runners, and splitting fmt and clippy out of
the test job (more runners, same wall time).

## Review focus

- A pull request that changes Rust code and a Markdown file still runs every
  job (`every` quantifier: one non-docs file is enough).
- A test that only passed because an earlier test in the same process set
  something up now fails under nextest: fix the test, never the profile.
- The apt cache on a cache miss still installs (an empty folder is fine) and
  the post step saves it (no root-only files left behind).
- The `sonarqube` job never runs without the `test` job's reports.
- Every required check of `protect-master` still reports on every pull
  request (skipped or run, never missing).

---

### Task 1: apt archive cache

**Files:** create `.github/actions/apt-install/action.yml`; modify every
`apt-get install` step in `ci.yml`, `pages.yml`, `play-screenshots.yml`,
`release-build.yml`, `screenshots.yml`, `stores.yml`.

- [ ] Write the composite action (input `packages`, decisions §1).
- [ ] Replace each `apt-get update && apt-get install …` with
  `uses: ./.github/actions/apt-install` (after `actions/checkout`, which the
  local action needs) and the same package list.
- [ ] `actionlint` (if available) or a YAML parse of every workflow.
- [ ] Commit `ci: keep the downloaded apt archives in the Actions cache`.

### Task 2: nextest in the test job

**Files:** create `.config/nextest.toml`; modify `ci.yml` (install
`cargo-nextest` with `taiki-e/install-action`, run
`cargo llvm-cov nextest --no-report --workspace --locked --profile ci
--features "$FEATURES"`); `docs/technical/testing.md` (Checks run in CI,
local command); `CLAUDE.md` Commands (the nextest line).

- [ ] Commit `ci: run the tests with cargo-nextest`.

### Task 3: docs-only filter and release-build paths

**Files:** modify `ci.yml` (`changes` job, `needs`/`if` on the six jobs),
`release-build.yml` (paths); `docs/technical/testing.md` and
`docs/technical/release-process.md` where they describe what runs on a pull
request.

- [ ] Commit `ci: skip the Rust jobs on docs-only pull requests`.

### Task 4: timeouts

**Files:** every workflow, per the table.

- [ ] Commit `ci: give every job a timeout`.

### Task 5: measure and merge

- [ ] Open the PR; compare the test jobs' times with the table above.
- [ ] Prove the docs-only path: a throwaway branch with only a Markdown
  change shows the six jobs skipped and the required checks green; close it
  without merging.
- [ ] Review the whole branch with a fresh reviewer on the most capable
  model; fix Critical and Important findings.
- [ ] Merge with CI green.
