# Website and Guide Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Publish a project website on GitHub Pages (a landing page with
per-platform downloads plus the user guide built from `docs/user/`), add
screenshots to the guide, refresh the README screenshot and set the
repository's description and homepage (O1, O28, O29, O30).

**Architecture:** mdBook renders `docs/user/` as the guide; a static
landing page in `site/` uses the Nocturne colours. One script,
`scripts/site/build.sh`, assembles both into one output folder and fills
the download links from the latest release at build time. A Pages workflow
builds and deploys it on doc changes, on every release (called by
release-please) and by hand. A screenshot script drives the real app in
Xvfb, on the silent backend, from a scripted demo scene.

**Tech Stack:** mdBook (prebuilt binary, pinned), POSIX shell, plain
HTML/CSS/JS (no framework, no build step), GitHub Actions
(`actions/upload-pages-artifact`, `actions/deploy-pages`), Xvfb, xdotool,
ImageMagick `import`, ffmpeg, the remote HTTP API.

**Spec:** `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md`
§12 (Plan 11 — Website and guide).

## Global Constraints

- All code, docs, comments and commits in English; Conventional Commits
  with the harness co-author trailer.
- Never mention other playout, radio-automation or tag-editor products
  anywhere (CLAUDE.md rule 2).
- Nothing is copied from `docs/user/` into the repository; the only new
  file there is `docs/user/SUMMARY.md`. `docs/user/README.md` stays the
  first page.
- Visitors' browsers never call the GitHub API: links are written at
  build time.
- Without JavaScript all three download tabs show.
- The Linux tab lists .deb, .rpm, AppImage, Flatpak and tar.gz for x86_64
  and aarch64 (Flatpak exists for x86_64 only: list what the release has);
  every tab also links the releases page.
- Screenshots are taken in English, in Xvfb (never on the real desktop),
  and the app is configured with the silent backend
  (`outputs.backend = "null"`): **nothing may open or play to a real audio
  device**. Builds go only to the repository's `target/`; downloaded tools
  go to `target/tools/`.
- Colours come from `crates/fp-app/src/ui/theme.rs` (Nocturne), copied as
  CSS custom properties.
- The landing page works at phone width (no horizontal scroll) and has no
  external requests except the release download links themselves.
- `CHANGELOG.md`, versions and the release manifest are never edited.

## Review Focus

1. **No release reachable at build time** (gh missing, no token, offline):
   the build still succeeds and every tab links the releases page.
2. **A release missing an asset** (no aarch64 rpm, no Flatpak): that row
   is left out, never a dead link.
3. **Guide links that leave `docs/user/`** (`../technical/…`,
   `../images/…`) resolve on the site and on GitHub.
4. **Screenshot run on a machine with sound hardware**: the app must never
   open a real device (silent backend checked before any capture).
5. **The visitor's OS is unknown** (a bot, an old browser): no tab is
   hidden; all show, as without JavaScript.

---

### Task 1: The guide as an mdBook

**Files:**
- Create: `docs/user/SUMMARY.md`, `docs/book.toml`,
  `scripts/site/build.sh`, `scripts/site/mdbook.sh`
- Modify: `docs/user/bit-perfect.md` and `docs/user/remote-control.md`
  (links into `docs/technical/`), `.gitignore` if needed

**Interfaces:**
- Produces: `scripts/site/build.sh [out-dir]` (default `target/site`):
  writes the guide to `<out>/guide/` and copies `docs/images/` to
  `<out>/images/`, so that a guide page's `../images/x.png` resolves
  inside the site. `scripts/site/mdbook.sh` prints the path of a pinned
  mdBook binary, downloading the release archive for the host into
  `target/tools/mdbook-<version>/` on first use (`MDBOOK_VERSION` defined
  once, in that script). Task 2 extends `build.sh`; Task 3 calls it in CI.

- [ ] **Step 1:** Write `docs/user/SUMMARY.md`: the README as the
  introduction, then every guide page in the order of the README's table.
- [ ] **Step 2:** Write `docs/book.toml`: `src = "user"`, title "Fauste
  Player — User guide", language `en`, the repository URL as
  `git-repository-url`, an edit link to `docs/user/{path}` on master, and
  the build directory left to the script (`mdbook build -d`).
- [ ] **Step 3:** Point the two links into `docs/technical/` at their
  GitHub URLs on master (`https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/<file>`),
  so that they work both on GitHub and in the book.
- [ ] **Step 4:** Write `mdbook.sh` (pick the latest stable mdBook release
  with `gh release view -R rust-lang/mdBook` once, pin it; Linux x86_64 and
  aarch64, macOS; fail with a clear message elsewhere) and `build.sh`
  (`set -eu`, POSIX sh, run from any directory).
- [ ] **Step 5:** Verify: `scripts/site/build.sh` succeeds; mdBook prints
  no warnings; `target/site/guide/index.html` exists; every `<img src>` and
  every relative `href` in `target/site/guide/*.html` resolves to a file in
  `target/site` (check with a small shell or Python loop and keep it as
  `scripts/site/check-links.sh <out-dir>`, exit 1 on a dead link).
- [ ] **Step 6:** Commit `docs(site): build the user guide with mdBook`.

### Task 2: The landing page and download links

**Files:**
- Create: `site/index.html`, `site/style.css`, `site/download.js`,
  `site/favicon.svg` (or a copy step for an existing icon in `packaging/`)
- Modify: `scripts/site/build.sh`, `scripts/site/check-links.sh`

**Interfaces:**
- Consumes: Task 1's `build.sh` and `check-links.sh`.
- Produces: `<out>/index.html` with the download links filled in.
  `build.sh` reads the release with
  `gh release view --json tagName,url,assets` (env `FAUSTE_RELEASE_JSON`
  may point at a saved JSON file instead, for tests and offline builds).

- [ ] **Step 1:** Content of `index.html`: the name and one-line pitch, the
  main screenshot (`images/main-screen.png`), six to eight main features
  taken from the README (players, mixing, CUE, cartwall, bit-perfect and
  DSD, MIDI and remote control, crash safety, English and Spanish), the
  download section, and links to the guide (`guide/`), the repository and
  the releases page. Plain semantic HTML; no external fonts or scripts.
- [ ] **Step 2:** The download section has three tabs (Windows, macOS,
  Linux), each a `<section>` visible by default. Windows: .msi and .zip.
  macOS: the universal .dmg and the two .tar.gz. Linux: a table by
  architecture (x86_64, aarch64) with .deb, .rpm, AppImage, Flatpak and
  .tar.gz. Each tab ends with "All files and checksums" linking the
  release page. Every link is a placeholder in the source
  (`data-asset="<pattern>"`, `href="https://github.com/sanchezfauste/fauste-player/releases/latest"`)
  that `build.sh` rewrites; a pattern with no matching asset removes its
  row; the version (`tagName`) fills `data-version` spots.
- [ ] **Step 3:** `download.js`: adds the tab buttons, picks the tab from
  `navigator.userAgentData.platform`, else `navigator.userAgent`
  (Windows / Mac / Linux, Android excluded); unknown OS leaves all tabs
  visible. Keyboard accessible tabs (`role="tablist"`, arrow keys).
- [ ] **Step 4:** `style.css`: Nocturne tokens from `theme.rs` as custom
  properties (background, panels, text, accent, on-air), a dark page,
  responsive down to 360 px, visible focus rings.
- [ ] **Step 5:** `build.sh` copies `site/` to `<out>/`, fills the links
  (Python 3 or `jq`, whichever the CI image and a Linux desktop both have;
  prefer Python 3), and still succeeds without a release (Review Focus 1).
- [ ] **Step 6:** Verify with a saved release JSON
  (`gh release view v0.4.0 --json tagName,url,assets > <scratch>/rel.json`)
  and with a JSON missing the aarch64 rpm and the Flatpak: rows drop, no
  placeholder link survives (`check-links.sh` fails on any remaining
  `data-asset` link pointing at `/releases/latest` inside a download row).
  Open the page in a headless browser at 1280 px and 375 px if one is
  available and check there is no horizontal scroll.
- [ ] **Step 7:** Commit `feat(site): landing page with per-platform downloads`.

### Task 3: The Pages workflow and the links to the site

**Files:**
- Create: `.github/workflows/pages.yml`
- Modify: `.github/workflows/release-please.yml`, `README.md`,
  `CLAUDE.md`, `docs/technical/release-process.md`, `docs/technical/README.md`
  (if it indexes the workflows)

**Interfaces:**
- Consumes: `scripts/site/build.sh`, `scripts/site/check-links.sh`.

- [ ] **Step 1:** `pages.yml`: triggers `push` to `master` on
  `docs/user/**`, `docs/images/**`, `docs/book.toml`, `site/**`,
  `scripts/site/**`, `.github/workflows/pages.yml`; `workflow_dispatch`;
  and `workflow_call`. Releases made by release-please use `GITHUB_TOKEN`
  and trigger no other workflow, so release-please calls `pages.yml` after
  its build job (`needs: build`). Job `build`: checkout, `build.sh
  _site` with `GH_TOKEN: ${{ github.token }}`, `check-links.sh _site`,
  `actions/upload-pages-artifact`. Job `deploy`: `actions/deploy-pages`,
  environment `github-pages`, permissions `pages: write`,
  `id-token: write`; concurrency group `pages`, no cancel in progress.
  Pin actions by major version as the other workflows do.
- [ ] **Step 2:** Pull requests touching those paths run the build and the
  link check only (no deploy): add a `pull_request` trigger with the same
  paths and guard the deploy job with `if: github.event_name != 'pull_request'`.
- [ ] **Step 3:** README: a "Website" link near the top
  (`https://sanchezfauste.com/fauste-player/`) and the guide link;
  CLAUDE.md Commands: `scripts/site/build.sh [out]` and
  `scripts/site/check-links.sh <out>`; release-process.md: the site is
  rebuilt on every release and the maintainer's one-time step (Pages
  source "GitHub Actions").
- [ ] **Step 4:** Verify with `actionlint` if installed (else a YAML parse
  with Python) and a local `build.sh` + `check-links.sh` run.
- [ ] **Step 5:** Commit `ci(site): build and deploy the website on GitHub Pages`.

### Task 4: The screenshot scene and the README screenshot (O29)

**Files:**
- Create: `scripts/site/screenshots.sh`, `scripts/site/tones.sh` (or one
  Python helper beside them)
- Modify: `crates/fp-app/examples/demo_session.rs` (only if the scene
  needs it), `docs/images/main-screen.png`, `CLAUDE.md` (Testing notes:
  point at the script instead of the manual steps)

**Interfaces:**
- Produces: `scripts/site/screenshots.sh [--only main]` that builds the
  release binary, generates the tones into a scratch folder, writes the
  scratch `FAUSTE_HOME` (demo session + `config.json` with
  `ui.language = "en-US"`, `remote.http.enabled = true`,
  `outputs.backend = "null"`), starts Xvfb on a free display, starts the
  app, waits for the remote API, builds the scene and captures. Helpers
  for Task 5: `start_app`, `api <method> <path> [json]`, `window_id`,
  `capture <file> [geometry]`, `click <x> <y>`, `key <keys>`, `stop_app`.

- [ ] **Step 1:** Tones: at least 16 files longer than 3 minutes, named like
  music ("Artist – Title"), each with its own envelope (different
  modulation rate, depth, shape, fades and quiet passages) so that no two
  waveforms look alike; ffmpeg pink noise and sines through a limiter, as
  CLAUDE.md describes; tagged as WAV `LIST/INFO` (title, artist) if the
  table shows tags.
- [ ] **Step 2:** Silent backend first: after the app starts, read
  `GET` status/devices through the API (or the log) and abort before any
  play command unless the output backend is `null`.
- [ ] **Step 3:** The scene (spec O29): players 1 and 2 playing some way
  into their playlists, a seek to mid-track, one cart fired; in one of the
  two playing players the entry marked next is 3 or 4 entries after the
  one on air (use the API's "set next" or the documented command).
  Window at 1920×1080, English.
- [ ] **Step 4:** Capture `docs/images/main-screen.png` (1920×1080). Look
  at it (Read the PNG) and check: two players on air with distinct
  waveforms, the far-ahead next entry, a cart lit, nothing clipped.
- [ ] **Step 5:** Commit `docs: a truer README screenshot from a scripted scene (O29)`.

### Task 5: Screenshots in the guide (O28)

**Files:**
- Modify: `scripts/site/screenshots.sh`; the guide pages in `docs/user/`
- Create: `docs/images/guide/*.png`

**Interfaces:**
- Consumes: Task 4's helpers and scene.

- [ ] **Step 1:** Extend the script with one capture per screen or window
  the guide describes: the main screen, a player column, the playlist
  table, the cartwall, the CUE window, the tag editor, Settings and each
  of its sections, the About window. Open windows with documented
  shortcuts (`docs/user/keyboard.md`), the API, or clicks at positions
  found from a capture. Crop sub-views with `capture <file> WxH+X+Y`.
  Keep the PNGs small (`-strip`, 8-bit palette when it looks the same).
- [ ] **Step 2:** Embed each screenshot in the page whose subject it is,
  beside the text it illustrates, with alt text that says what it shows
  (`![The cartwall with one cart playing](../images/guide/cartwall.png)`).
- [ ] **Step 3:** Look at every PNG (Read it) and check that it shows what
  its page says, in English, unclipped.
- [ ] **Step 4:** `build.sh` + `check-links.sh` pass.
- [ ] **Step 5:** Commit `docs(user): screenshots of every screen in the guide (O28)`.

### Task 6: As built

**Files:**
- Modify: the spec (status line, §12 "As built"), the README roadmap row,
  this plan ("Deviations as built")

- [ ] **Step 1:** Spec status: all thirteen plans built; §12 "As built"
  notes (paths, the release-please call, the maintainer step, the
  repository About done after the site is live).
- [ ] **Step 2:** README roadmap row for plan 11 done.
- [ ] **Step 3:** Commit `docs: plan 11 as built`.

**After merge (controller, O30):** once the Pages deploy succeeds and the
site answers, run
`gh repo edit --description "<one sentence from the README pitch>" --homepage https://sanchezfauste.com/fauste-player/`.
Pages already uses "GitHub Actions" as its source (checked 2026-10-03);
the account's custom domain serves project sites at
`https://sanchezfauste.com/fauste-player/`.

### Deviations as built

Recorded from the ledger; the spec's section 12 "As built" has the same notes in context.

- Ruling: the site URL is `https://sanchezfauste.com/fauste-player/` (the account's custom domain) — the Pages API reports it — one URL edit if wrong.
- Ruling: release-please calls `pages.yml` through `workflow_call`, instead of `on: release` — a release made with `GITHUB_TOKEN` starts no other workflow — none.
- Ruling: links from the guide into `docs/technical/` are absolute GitHub URLs — they work on GitHub and in the book — they point at `master`.
- Fix after review: pull request checks get a concurrency group of their own; `build.sh` refuses dangerous output folders.
- Tasks 4–5: the songs are tagged FLAC files with covers (WAV showed "cannot store" notes and no covers); `build.sh` copies `docs/images` recursively; the media lives in `target/screenshots` (the /tmp quota) behind the `SHOTS_MEDIA` link, so no home path shows.
- Ruling: the Settings crops are kept at the window's fixed size; the rows cut at the bottom are the section's own scroll edge.
- Parked: a transient `gh` failure in CI deploys a site without download links; JSON of an unexpected shape stops the build; iOS preselects the macOS tab; a dead placeholder check in `check-links.sh`; no checksum on the mdBook download; a check-then-use race in `free_display`.
