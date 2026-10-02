# Window and Layout (Feedback 2, Plan 12) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Five layout fixes from operator feedback: the cartwall fits its rows without scrolling (O31), one title bar that carries the icon, the name and the version (O32), times that keep a fixed width while they change (O33), playlist tabs that shrink, cut long names and scroll sideways (O35), and no console window for the Windows release (O36).

**Architecture:**
- **Pure helpers first, then the drawing.** The sizes are worked out by small pure functions that tests call without a window: `cart_view::button_height` (O31), `ui::tab_strip` (O35), `widgets::paint_tabular_right` and the badge width (O33), `cli::window_title` and `cli::emit` (O32, O36). The drawing code only calls them.
- **O32 uses the native title bar.** The app has no custom window frame: `main.rs` already opens a native window and the top bar (`top_bar` in `ui/app.rs`) paints a brand block (icon, name, version) that repeats what the title bar should say. The plan puts "Fauste Player 1.2.3" in the viewport title (`ViewportBuilder::with_title`), keeps the window icon (`cli::window_icon`, already set), and removes the brand block from the top bar.
- **O36 is a build attribute plus a safe fallback.** `#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]` in `main.rs`; text that used to go to the console (`--version`, `--help`, start-up errors) goes through `cli::emit`, which shows a native message box on that build (`rfd`, already a dependency) and prints as before everywhere else. No `unsafe`.

**Tech Stack:** Rust, egui/eframe 0.36.2, egui_kittest 0.36.2, rfd (existing). No new dependency.

**Spec:** `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md` §13 (plan 12: O31, O32, O33, O35, O36; items table rows O31–O36) and §15 (global constraints). Roadmap: `docs/superpowers/plans/2026-10-01-operator-feedback-2-roadmap.md` (plan 12, branch `fix/window-layout`).

**Base.** Branch `fix/window-layout` from `master` after plan 9 (`6d78e76`). Line numbers below are from that tree and move; steps name functions and give the text to find.

## Decisions taken

The spec leaves these open. Each is the most conservative reading.

- **O32, maintainer decision: one title bar, the native one.** The native bar shows the title `Fauste Player <version>` (spec: "Fauste Player 1.2.3", no `v`) and the icon on Windows and Linux. macOS title bars do not draw an application icon (the icon is the Dock and bundle icon, already set by the bundle); there the bar shows the name and the version. The title is not localised (the name is the same in both locales). The top bar loses the icon, the name, the `v1.2.3` text and the clickable name; it keeps the clock, Settings, the About button (info icon), the Restart pending pill and everything else. The About window is still opened by the info button and still shows the version. The Fluent key `tip-about-name` becomes unused and is removed from both locales; `app-name` stays (About window).
- **CLAUDE.md screenshot recipe.** `xwininfo -name "Fauste Player"` matches the exact title, which now has the version; the recipe changes to `xwininfo -root -tree | grep "Fauste Player"` (Task 1).
- **O31: the area keeps its rule, the buttons adapt.** `cartwall::height` still asks for the nominal height (rows × 40 px) and `app.rs` still caps the strip at 60% of the middle area. New: the buttons take their height from the room the grid really has, `button_height(available, rows)` = (available − gaps) / rows, clamped to `MIN_BUTTON_HEIGHT` (28 px) … `BUTTON_HEIGHT` (40 px, never taller than today). Only when the rows do not fit even at 28 px does the grid scroll. 28 px, not 24: at 24 the 11 px name and the 10 px detail line overlap (Ruling: raised to 28, confirmed in review — cost if wrong: one constant).
- **O33: what "fixed width" means per site.** Digits are drawn in equal cells by the existing `widgets::paint_tabular` (the UI font has no tabular figures through egui). Audit of every live time (found with `grep -n "format::clock\|format::countdown\|Local::now\|time_badge"` in `crates/fp-app/src`):
  - already steady: the big countdown and the tenths, elapsed / total in the player info row, the CUE time in the info row, the CUE window times, the footer total and remaining;
  - changed here: the cart countdown (`view.time`, right aligned, so it needs a right-aligned tabular paint), the intro and outro badges (`time_badge`: the value box is sized for `00.0`, so 9.9 → 10.0 does not move the badge edge), the top-bar clock, the waveform hover time and the marker-drag time;
  - not live, left alone: the playlist table's duration and total columns (they change only when the playlist does), the right-click time label (fixed while the menu is open), the Settings playlist total (a fixed 80 px box).
- **O35: equal widths, a floor, arrows.** Tabs keep today's look (equal widths filling the strip). A tab is `strip / count` wide, never narrower than `MIN_TAB_WIDTH` (72 px: the dot, the "…" and about three letters). Names are cut by egui's one-row layout with the "…" overflow character (today's wrapping code lays out several rows and clips them). The tooltip with the full name already exists. When `count × 72` exceeds the strip, two arrow buttons (22 px) take the ends of the strip, the tabs sit in a clipped view between them, the mouse wheel over the strip scrolls it, an arrow moves one tab, and the shown tab is brought into view when it, the playlist count or the strip width changes (not on every frame, so the operator can scroll away from it). The two constants are layout metrics, not operator values (the same class as `TABS_HEIGHT`), so no `Config` field (CLAUDE.md rule 4 is about operator values).
- **O36: debug builds keep their console.** `cargo run` and `cargo test` on Windows stay console programs (`not(debug_assertions)` in the attribute); only release binaries are GUI programs. Release `--version` and `--help` open an information box with the same text (this is the "useful without unsafe" path the spec asks for); start-up errors that used to go to stderr (no home folder, cannot lock, cannot start) open an error box. Rust's `println!`/`eprintln!` already ignore a missing standard handle, so the rest of the code needs no change. CI and packaging were checked: `scripts/package/windows.sh` and `scripts/package-release.sh` only copy and sign the executable, the Windows jobs of `ci.yml` run debug builds, and the two `--version` smoke tests (`scripts/package/linux.sh`, `scripts/package/macos.sh`) run on Linux and macOS binaries, which keep stdout.

## Global Constraints

- All code, identifiers, comments, docs, specs, plans and commit messages are in English. Never mention other playout, radio-automation or tag-editor products.
- Spec §15 (binding; the spec's own words):

> `CLAUDE.md` rules 1–10 apply to every plan. In particular:
>
> - English everywhere, and UI strings in both locales;
> - no product names;
> - operator values are `Config` fields with defaults, ranges and lenient
>   loading;
> - the real-time path never allocates, locks, logs or panics;
> - behaviour lives in `fp-model`;
> - the UI never blocks (tag writing, restart and file work run on helper
>   threads);
> - bad data never crashes;
> - nothing goes on air by itself.
>
> TDD throughout. Each plan ends with a docs task and a review by a fresh
> reviewer.

- Spec §13 (binding; the spec's own words):

> - **O31 Cartwall without scrolling.** The configured rows always fit the
>   cartwall area: the cart buttons take their height from the space there is,
>   down to a minimum height. The area scrolls only when the rows do not fit
>   even at that minimum.
> - **O32 One title bar.** The native title bar shows the icon, the name and
>   the version ("Fauste Player 1.2.3") on Windows, Linux and macOS. The app's
>   own top bar no longer repeats the icon, the name or the version; it keeps
>   everything else it shows.
> - **O33 Fixed-width times.** Every time that changes while it is shown (the
>   cart countdowns, the intro and outro countdowns, elapsed and remaining
>   times) keeps the same width: digits use tabular (fixed-width) figures, or
>   the text sits in a box sized for its longest value, so nothing beside it
>   moves.
> - **O35 Playlist tabs.** Tabs shrink, down to a minimum width, and cut long
>   names with "…"; the full name shows in a tooltip. When the tabs still do
>   not fit, the tab strip scrolls sideways (wheel and arrow buttons at its
>   ends), and the selected tab always stays in view.
> - **O36 No console on Windows.** The Windows executable is a GUI program
>   (`windows_subsystem = "windows"`), so no console window opens with it. The
>   plan keeps `--version` and `--help` useful without `unsafe` code (for
>   example, debug builds stay console programs), and checks that CI and the
>   packaging scripts do not depend on console output from the release binary.

- CLAUDE.md rule 6: `unsafe_code` is forbidden; no `unwrap`, `expect` or `panic` outside tests (tests carry `#![allow(clippy::unwrap_used, clippy::indexing_slicing)]` as the existing ones do); prefer `get` to indexing.
- CLAUDE.md rule 8: nothing here blocks the UI thread. `cli::emit` shows a modal box only before the window exists (start-up) or when the process is about to exit.
- UI strings are Fluent messages in `crates/fp-app/locales/en-US/main.ftl` and `es-ES/main.ftl`, always both. This plan adds none and removes one (`tip-about-name`).
- UI tests: `egui_kittest` with `Harness::builder().with_step_dt(0.02)` (through `support::harness_sized`) and the recording `Fake` controller in `crates/fp-app/tests/support`.
- Gate for every commit (run from the repo root, builds go only into the repo's `target/`):

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q
scripts/check-commits.sh origin/master   # no output = pass
```

  Commit subjects are Conventional Commits and end with the trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>` (the commit commands below show the subject only). `CHANGELOG.md` is never edited by hand.
- Documentation is part of each task (CLAUDE.md "Documentation is part of every change"); Task 6 does the spec, roadmap and a last consistency sweep.

## Review Focus

Failure modes the spec implies and its tests do not name; each has a test in the task named.

1. **Many playlists, or one with a very long name, in a narrow player column** (30 tabs, a 200-character name, a column narrower than the two arrows): the strip never spills outside its column, nothing panics, the name ends in "…", the shown tab is reachable. (Task 5 `a_name_of_200_characters_is_cut_and_the_strip_stays_in_its_column`, `thirty_tabs_scroll_and_the_shown_one_is_in_view`, `a_strip_narrower_than_its_arrows_does_not_panic`.)
2. **One playlist, or none visible changes**: a single tab fills the strip, no arrows appear, deleting the shown playlist while scrolled keeps the offset valid. (Task 5 `few_tabs_fill_the_strip_without_arrows`, `the_offset_is_clamped_when_the_tabs_get_fewer`.)
3. **A cartwall page with one row, a collapsed cartwall, or a window so small that even 28 px buttons do not fit**: one row keeps the 40 px buttons, collapsing draws nothing, and the small window scrolls with the last cart still reachable. (Task 3 `one_row_keeps_the_nominal_button_height`, `a_collapsed_cartwall_draws_no_buttons`, existing `the_last_cart_is_reachable_in_a_small_window`.)
4. **A time of one hour or more, a NaN or a negative value**: the width stays steady inside each format and never panics. (Task 4 `hour_times_keep_their_width_inside_their_format`, `a_nan_time_still_has_a_width`.)
5. **A Windows release binary started with `--version`, `--help`, a bad option, or when start-up fails**: the operator must get the text in a box instead of nothing, and a missing standard output must never panic. (Task 2 `emit_prints_on_a_console_build`, `console_is_the_default_off_windows`, `check-windows-gui.sh` self-test; Windows-only behaviour is verified by the PE check, see Task 2.)

## File Structure

- Create `crates/fp-app/src/ui/tab_strip.rs` (pure tab layout), `crates/fp-app/tests/tab_strip.rs`, `crates/fp-app/tests/playlist_tabs_ui.rs`, `scripts/check-windows-gui.sh`.
- Modify `crates/fp-app/src/cli.rs` (`window_title`, `emit`, `CONSOLE`), `crates/fp-app/src/main.rs` (title, subsystem attribute, `emit` calls).
- Modify `crates/fp-app/src/ui/app.rs` (`top_bar`, `ViewState`), `crates/fp-app/src/ui/cart_view.rs` and `crates/fp-app/src/ui/cartwall.rs` (button height), `crates/fp-app/src/ui/widgets.rs` (right-aligned tabular paint, badge, hover time), `crates/fp-app/src/ui/player.rs` (badges, marker-drag time, tabs), `crates/fp-app/src/ui.rs` (`pub mod tab_strip;`).
- Modify tests `crates/fp-app/tests/about.rs`, `cli.rs`, `cartwall_ui.rs`, `tabular.rs`; locales (`tip-about-name` removed from both).
- Modify `scripts/package/windows.sh`.
- Modify docs: `README.md`, `CLAUDE.md`, `docs/user/getting-started.md`, `docs/user/cartwall.md`, `docs/user/playlists.md`, `docs/technical/ui.md`, `docs/technical/release-process.md`, the feedback 2 spec (§13 "As built", status) and the roadmap (row 12 done).

---

### Task 1: O32, one title bar

**Files:**
- Modify: `crates/fp-app/src/cli.rs` (add `window_title`), `crates/fp-app/src/main.rs` (the viewport title), `crates/fp-app/src/ui/app.rs` (`top_bar`: remove the brand block), `crates/fp-app/locales/en-US/main.ftl`, `crates/fp-app/locales/es-ES/main.ftl` (remove `tip-about-name`)
- Test: `crates/fp-app/tests/cli.rs`, `crates/fp-app/tests/about.rs`
- Docs: `README.md` (lines mentioning the version "next to the name in the top bar", about 35 and 356), `docs/user/getting-started.md` ("Command line" paragraph), `docs/technical/ui.md`, `CLAUDE.md` (screenshot recipe step 4)

**Interfaces:**
- Produces: `fp_app::cli::window_title() -> String` (`"Fauste Player <CARGO_PKG_VERSION>"`). Tasks 2 and 6 refer to it.

- [ ] **Step 1: Write the failing tests**

In `crates/fp-app/tests/cli.rs` add:

```rust
#[test]
fn the_window_title_is_the_name_and_the_version() {
    assert_eq!(
        fp_app::cli::window_title(),
        format!("Fauste Player {}", fp_app::ui::about::VERSION)
    );
}
```

In `crates/fp-app/tests/about.rs`, read the file first. Replace the test `clicking_the_name_opens_about_and_escape_closes_it` by one that opens About from the info button and closes it with Escape (copy the body of `the_info_button_sits_next_to_settings_and_opens_about` and add the Escape step the old test had). Remove the constant `ABOUT_NAME` if nothing else uses it. Change the assertion near line 32 (`h.query_by_label_contains(about::VERSION).is_some()`) into the new rule, and add a second test:

```rust
#[test]
fn the_top_bar_no_longer_repeats_the_name_or_the_version() {
    let (mut h, _fake) = support::harness(support::state(1, 0));
    h.run_steps(2);
    // The native title bar carries them now (feedback 2 spec O32).
    assert!(h.query_by_label_contains(about::VERSION).is_none());
    assert!(h.query_by_label("Fauste Player").is_none());
    // What else the bar shows stays.
    assert!(h.query_by_label("Settings").is_some());
    assert!(h.query_by_label("About Fauste Player").is_some()); // the info button
}
```

(If the harness needs `mod support;` at the top of `about.rs`, add it. The label of the Settings tile is the `top-settings` text; check `locales/en-US/main.ftl` and use the real text.)

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-app --test cli the_window_title -q` (fails to compile: `window_title` missing) and `cargo test -p fp-app --test about the_top_bar_no_longer -q` (fails: the version is still in the bar).

- [ ] **Step 3: Implement**

`crates/fp-app/src/cli.rs`, next to `window_icon`:

```rust
/// The native window's title: the name and the version (feedback 2 spec
/// O32). The window icon is `window_icon`.
pub fn window_title() -> String {
    format!("Fauste Player {}", env!("CARGO_PKG_VERSION"))
}
```

`crates/fp-app/src/main.rs`: `.with_title("Fauste Player")` becomes `.with_title(cli::window_title())`. Leave the two `set_title("Fauste Player")` of message boxes as they are (a box is titled by the name alone).

`crates/fp-app/src/ui/app.rs`, `top_bar`: delete from `ui.add(egui::Label::new(RichText::new(egui_phosphor::fill::BROADCAST)...` through the `if response.clicked() { view_state.about_open = true; }` that follows the `tip-about-name` interaction (the icon, `name`, `version`, `about_label`, `response`). Keep `ui.add_space(10.0)` and the whole `right_to_left` block (clock, Settings, About button, Restart pending pill). Remove imports that become unused (`about::VERSION` use stays only if still referenced; `font_medium` may still be used elsewhere in the file: let clippy decide). Remove `tip-about-name` from both `main.ftl` files. Check that `theme::ICONS_FILL` and the fill icon font are still used elsewhere (`grep -rn ICONS_FILL crates/fp-app/src`); if not, leave them: removing the font is out of scope.

- [ ] **Step 4: Run the tests and the whole `fp-app` suite**

Run: `cargo test -p fp-app -q`. Expected: PASS, including `tests/i18n.rs` (it checks both locales have the same keys).

- [ ] **Step 5: Documentation**

- `README.md`: the About bullet and the About paragraph say the version is in the top bar next to the name; say instead that the window's title bar shows "Fauste Player <version>" and that About (the info button in the top bar) shows the version and the licences.
- `docs/user/getting-started.md` "Command line": drop "The top bar shows it too, next to the name: click either, or the **About** button..." and write that the title bar shows the name and the version and the **About** button (an info icon, left of **Settings**) opens **About**.
- `docs/technical/ui.md`: a sentence under the files table or `ui/app.rs` row: the title (`cli::window_title`) and icon (`cli::window_icon`) come from the native frame; the top bar has no brand block.
- `CLAUDE.md` step 4 of the screenshot recipe: replace `DISPLAY=:77 xwininfo -name "Fauste Player"` by ``DISPLAY=:77 xwininfo -root -tree | grep "Fauste Player"`` and say the title carries the version. `AGENTS.md` stays a pointer.

- [ ] **Step 6: Gate and commit**

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q
git add crates README.md CLAUDE.md docs
git commit -m "feat(ui): show the name and version in the native title bar only"
scripts/check-commits.sh origin/master
```

---

### Task 2: O36, no console window on Windows

**Files:**
- Modify: `crates/fp-app/src/cli.rs` (`CONSOLE`, `Stream`, `emit`), `crates/fp-app/src/main.rs` (attribute, every `println!`/`print!`/`eprintln!` that tells the operator something)
- Create: `scripts/check-windows-gui.sh`
- Modify: `scripts/package/windows.sh`
- Test: `crates/fp-app/tests/cli.rs`, the self-test inside `scripts/check-windows-gui.sh`
- Docs: `docs/user/getting-started.md` ("Command line"), `docs/technical/release-process.md`, `docs/technical/testing.md` (one line on the PE check)

**Interfaces:**
- Consumes: `fp_app::cli::window_title` is unrelated; this task needs only `cli.rs`.
- Produces: `pub const CONSOLE: bool` (`true` unless this is a Windows release build), `pub enum Stream { Out, Err }`, `pub fn emit(stream: Stream, text: &str)` in `fp_app::cli`.

**How it is verified.** On Linux CI only the compile and the tests below run: the attribute is `cfg`'d out. The Windows release build is checked in two places: (1) `scripts/package/windows.sh` calls `scripts/check-windows-gui.sh` on `fauste-player.exe` and fails the release job when the PE header's Subsystem field is not 2 (GUI); (2) the same script has a `--self-test` that builds a synthetic PE header with `printf` and runs on Linux, so the checker itself is tested here. A manual step (not automated): on a Windows machine, double-click the release executable and see no console, run `fauste-player.exe --version` and see the message box.

- [ ] **Step 1: Write the failing tests**

In `crates/fp-app/tests/cli.rs`:

```rust
#[test]
fn console_is_the_default_off_windows() {
    // Only a Windows release build is a GUI program (feedback 2 spec O36);
    // the test suite itself always runs with a console.
    assert!(fp_app::cli::CONSOLE);
}

#[test]
fn emit_prints_on_a_console_build() {
    // Must not panic or open a box where a console exists.
    fp_app::cli::emit(fp_app::cli::Stream::Out, "");
    fp_app::cli::emit(fp_app::cli::Stream::Err, "");
}
```

Create `scripts/check-windows-gui.sh` with the failing self-test first (the function body empty), see it fail, then fill it:

```bash
#!/usr/bin/env bash
# Checks that a Windows executable is a GUI program (PE Subsystem 2), so
# that it opens no console window (feedback 2 spec O36).
#
#   scripts/check-windows-gui.sh <file.exe>
#   scripts/check-windows-gui.sh --self-test
set -euo pipefail

# Prints the PE Subsystem field: 2 is GUI, 3 is console.
subsystem_of() {
  local exe="$1" pe
  pe="$(od -An -tu4 -j60 -N4 "${exe}" | tr -d ' ')"
  od -An -tu2 -j"$((pe + 92))" -N2 "${exe}" | tr -d ' '
}

# A file with a PE header whose Subsystem is $1.
fake_pe() {
  local file="$1" subsystem="$2"
  : > "${file}"
  printf 'MZ' >> "${file}"
  head -c 58 /dev/zero >> "${file}"
  printf '\x80\x00\x00\x00' >> "${file}"          # e_lfanew = 128 at offset 60
  head -c 64 /dev/zero >> "${file}"               # up to offset 128
  head -c 92 /dev/zero >> "${file}"               # PE signature + headers
  printf "\\x0$((subsystem))\\x00" >> "${file}"   # Subsystem at pe + 92
}

if [ "${1:-}" = "--self-test" ]; then
  tmp="$(mktemp)"
  trap 'rm -f "${tmp}"' EXIT
  fake_pe "${tmp}" 2
  [ "$(subsystem_of "${tmp}")" = "2" ] || { echo "self-test: GUI not recognised" >&2; exit 1; }
  fake_pe "${tmp}" 3
  [ "$(subsystem_of "${tmp}")" = "3" ] || { echo "self-test: console not recognised" >&2; exit 1; }
  echo "ok"
  exit 0
fi

exe="${1:?usage: check-windows-gui.sh <file.exe> | --self-test}"
found="$(subsystem_of "${exe}")"
if [ "${found}" != "2" ]; then
  echo "${exe} is not a GUI program (PE subsystem ${found}); it would open a console window" >&2
  exit 1
fi
```

`chmod +x` it. The `fake_pe` offsets: DOS header 64 bytes (`MZ` + 58 zero + 4-byte e_lfanew at 60), the PE header at 128 (hence the 64 more zero bytes), Subsystem at 128 + 92 = 220. Run `scripts/check-windows-gui.sh --self-test`; expected before the body is right: failure; after: `ok`. (If the byte arithmetic of `fake_pe` is off by a few bytes, fix the padding, not the `+ 92`: it is the offset of `Subsystem` in the optional header for PE32 and PE32+.)

- [ ] **Step 2: Run the Rust tests to see them fail**

Run: `cargo test -p fp-app --test cli -q`. Expected: compile error (`CONSOLE`, `emit`, `Stream` missing).

- [ ] **Step 3: Implement**

`crates/fp-app/src/cli.rs`:

```rust
/// Whether this build has a console to print to. A Windows release build is
/// a GUI program (`windows_subsystem = "windows"` in `main.rs`) and has
/// none; debug builds, and every other platform, do (feedback 2 spec O36).
pub const CONSOLE: bool = cfg!(any(not(windows), debug_assertions));

/// Which standard stream a message is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stream {
    Out,
    Err,
}

/// Tells the operator `text`: printed where there is a console, otherwise in
/// a message box (information for `Out`, error for `Err`). Used only before
/// the window exists or when the process is about to exit.
pub fn emit(stream: Stream, text: &str) {
    if CONSOLE {
        match stream {
            Stream::Out => print!("{text}"),
            Stream::Err => eprint!("{text}"),
        }
        return;
    }
    let level = match stream {
        Stream::Out => rfd::MessageLevel::Info,
        Stream::Err => rfd::MessageLevel::Error,
    };
    let _ = rfd::MessageDialog::new()
        .set_title("Fauste Player")
        .set_description(text)
        .set_level(level)
        .show();
}
```

(`print!`/`eprint!` do not panic when a handle is missing: std ignores an invalid standard handle on Windows.)

`crates/fp-app/src/main.rs`: first line of the file after the doc comment:

```rust
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
```

(An inner attribute must come before items and after the `//!` docs; put it right after them.) Then route the messages:

- `--version`: `cli::emit(cli::Stream::Out, &format!("fauste-player {}\n", env!("CARGO_PKG_VERSION")));`
- `--help`: `cli::emit(cli::Stream::Out, &cli::usage());`
- the "not a playlist, ignored" line, the CLI parse error, "no home directory found", "cannot lock": `cli::emit(cli::Stream::Err, &format!("fauste-player: ...\n"))` (keep each message text; add the `\n`). For "not a playlist, ignored" use `tracing::warn!` only on a GUI build: simplest is to keep `eprintln!` there (an ignored argument is not worth a box) and a log line.
- the "could not start" error at the end of `main` and `run`'s failure: `emit(Stream::Err, ...)`.
- the "could not start again" branch already shows a dialog: leave its `eprintln!` (invisible on a GUI build, harmless).
- `hand_over` (second instance): read the function; if it prints, use `emit` as above.

- [ ] **Step 4: Run tests**

Run: `cargo test -p fp-app --test cli -q` and `scripts/check-windows-gui.sh --self-test`. Expected: PASS, `ok`. Also run `cargo build -p fp-app --release -q` once to see the attribute does not break a non-Windows release build.

- [ ] **Step 5: Wire the check into the Windows packaging and document**

`scripts/package/windows.sh`: after the `package-release.sh` call and before `sign "${exe}"`, add

```bash
"${root}/scripts/check-windows-gui.sh" "${exe}" >&2
```

- `docs/user/getting-started.md` "Command line": on Windows the release program opens no console: `--version`, `--help` and start-up errors appear in a box instead.
- `docs/technical/release-process.md`: the Windows packaging step checks the PE subsystem (`scripts/check-windows-gui.sh`), and why debug builds keep their console.
- `docs/technical/testing.md`: one line that the Windows GUI attribute is verified by the PE check, not by tests.

- [ ] **Step 6: Gate and commit**

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q
git add crates scripts docs
git commit -m "feat(app): build the Windows release as a GUI program without a console"
scripts/check-commits.sh origin/master
```

---

### Task 3: O31, the cartwall fits its rows

**Files:**
- Modify: `crates/fp-app/src/ui/cart_view.rs` (constants and `button_height`), `crates/fp-app/src/ui/cartwall.rs` (`height`, `strip`, `grid`)
- Test: `crates/fp-app/tests/cartwall_ui.rs`
- Docs: `docs/user/cartwall.md`, `docs/technical/ui.md` (the `ui/cartwall.rs` row)

**Interfaces:**
- Produces in `fp_app::ui::cart_view` (public, pure): `pub const BUTTON_HEIGHT: f32 = 40.0; pub const MIN_BUTTON_HEIGHT: f32 = 28.0; pub const GAP: f32 = 6.0; pub fn button_height(available: f32, rows: usize) -> f32`.

- [ ] **Step 1: Write the failing tests**

In `crates/fp-app/tests/cartwall_ui.rs` add (use `use fp_app::ui::cart_view::{BUTTON_HEIGHT, GAP, MIN_BUTTON_HEIGHT, button_height};`):

```rust
#[test]
fn buttons_take_the_height_there_is_within_their_limits() {
    // Room for the nominal height: unchanged.
    assert_eq!(button_height(4.0 * BUTTON_HEIGHT + 3.0 * GAP, 4), BUTTON_HEIGHT);
    assert_eq!(button_height(1000.0, 4), BUTTON_HEIGHT);
    // Less room: they shrink, gaps included.
    let h = button_height(130.0, 4);
    assert!((h - (130.0 - 3.0 * GAP) / 4.0).abs() < 0.01, "{h}");
    // Never below the minimum, whatever the room.
    assert_eq!(button_height(10.0, 4), MIN_BUTTON_HEIGHT);
    assert_eq!(button_height(f32::NAN, 4), BUTTON_HEIGHT);
    assert_eq!(button_height(-5.0, 4), MIN_BUTTON_HEIGHT);
}

#[test]
fn one_row_keeps_the_nominal_button_height() {
    assert_eq!(button_height(BUTTON_HEIGHT, 1), BUTTON_HEIGHT);
    assert_eq!(button_height(500.0, 1), BUTTON_HEIGHT);
    assert_eq!(button_height(100.0, 0), BUTTON_HEIGHT);
}

#[test]
fn four_rows_fit_a_short_window_without_scrolling() {
    // 4 rows at 40 px do not fit the 60% the strip may take of a 360 px
    // window; at the minimum height they do.
    let (mut h, _fake) =
        support::harness_sized(with_cart(), egui::vec2(700.0, 360.0), |ui| ui);
    h.run_steps(3);
    let first = h.get_all_by_label("Station ID").next().unwrap().rect();
    let last = h.get_all_by_label("Cart 16, empty").next().unwrap().rect();
    assert!(first.height() < BUTTON_HEIGHT && first.height() >= MIN_BUTTON_HEIGHT, "{first:?}");
    // The last row is on screen without scrolling (above the 24 px status bar).
    assert!(last.bottom() <= 360.0 - 24.0, "{last:?}");
    assert!(last.top() > first.top());
}

#[test]
fn a_collapsed_cartwall_draws_no_buttons() {
    let mut s = with_cart();
    apply(&mut s, Command::SetCartwallOpen(false)).unwrap();
    let (mut h, _fake) = support::harness(s);
    h.run_steps(3);
    assert!(h.query_by_label("Station ID").is_none());
}
```

(Check the `harness_sized` signature in `tests/support/mod.rs`: `(state, size, build)`. The default cart button's accessible label for an empty cart is `Cart 16, empty` as the existing test shows; "Station ID" is the first cart's name. If `get_all_by_label("Station ID")` finds also the footer or table, use `get_by_role_and_label(Role::Button, ...)`. If the 4-row default needs a different row count, set `rows` through `Command::SetCartPageSize` or whatever the model offers: grep `rows` in `crates/fp-model/src/command.rs`.)

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-app --test cartwall_ui -q`. Expected: compile error (the new items missing).

- [ ] **Step 3: Implement**

`crates/fp-app/src/ui/cart_view.rs`:

```rust
/// Height of a cart button when there is room (feedback 2 spec O31).
pub const BUTTON_HEIGHT: f32 = 40.0;
/// The least a cart button shrinks to: room for the name and the detail line.
pub const MIN_BUTTON_HEIGHT: f32 = 28.0;
/// Gap between cart buttons and below the header.
pub const GAP: f32 = 6.0;

/// The height of a cart button when `rows` rows share `available` pixels
/// (gaps included): what fits, between `MIN_BUTTON_HEIGHT` and
/// `BUTTON_HEIGHT`. The grid scrolls only when even the minimum does not fit.
pub fn button_height(available: f32, rows: usize) -> f32 {
    if rows == 0 || available.is_nan() {
        return BUTTON_HEIGHT;
    }
    let gaps = GAP * (rows - 1) as f32;
    ((available - gaps) / rows as f32).clamp(MIN_BUTTON_HEIGHT, BUTTON_HEIGHT)
}
```

`crates/fp-app/src/ui/cartwall.rs`: delete the private `BUTTON_HEIGHT` and `GAP`, import them from `cart_view`. In `strip`, after `ui.add_space(GAP)` take the room the grid has and pass it down:

```rust
let available = ui.available_height();
egui::ScrollArea::both()
    .id_salt("cart-grid")
    .auto_shrink([false, false])
    .show(ui, |ui| grid(ui, scene, view_state, available));
```

`grid` gains `available: f32`; at its top compute `let button = button_height(available, rows_of_page);` with `rows_of_page = page.carts.chunks(cols).count()` (the rows actually drawn) and replace both uses of `BUTTON_HEIGHT` inside it (`vec2(width, BUTTON_HEIGHT)`) by `button`. `height()` keeps using `BUTTON_HEIGHT` (the nominal request). The detail line is painted at `inner.left_bottom()` with `inner = r.shrink2(vec2(8.0, 4.0))`: at 24 px the name (11 px) and the detail (10 px) share 16 px, so verify visually at the minimum (Step 5) and, if they overlap, raise `MIN_BUTTON_HEIGHT` to 28 and update the tests' expectations through the constant (the tests use the constants).

- [ ] **Step 4: Run tests**

Run: `cargo test -p fp-app --test cartwall_ui -q`. Expected: PASS, including the existing `the_last_cart_is_reachable_in_a_small_window` (still scrolls at 420x480? if it no longer needs to, it must still pass: it only scrolls to the last cart).

- [ ] **Step 5: Look at it, then document**

Optional visual check with the CLAUDE.md Xvfb recipe at a small window (for example `xdotool windowsize` 1000x500, one player). Docs: `docs/user/cartwall.md` one paragraph: the buttons shrink (to a minimum height) to fit the configured rows, and the cartwall scrolls only below that; `docs/technical/ui.md` row for `ui/cartwall.rs`: `cart_view::button_height`, nominal 40, minimum 28, the 60% cap in `AppUi::ui` unchanged.

- [ ] **Step 6: Gate and commit**

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q
git add crates docs
git commit -m "fix(ui): fit the cartwall rows to the room there is before scrolling"
scripts/check-commits.sh origin/master
```

---

### Task 4: O33, fixed-width times

**Files:**
- Modify: `crates/fp-app/src/ui/widgets.rs` (`paint_tabular_right`, `time_badge`, the waveform hover time), `crates/fp-app/src/ui/cartwall.rs` (the cart time), `crates/fp-app/src/ui/player.rs` (the marker-drag time in `edit_markers`), `crates/fp-app/src/ui/app.rs` (the top-bar clock)
- Test: `crates/fp-app/tests/tabular.rs`
- Docs: `docs/technical/ui.md`

**Interfaces:**
- Consumes: `widgets::paint_tabular(painter, left_top, text, font, color) -> Rect`, `widgets::tabular_size(painter, text, font) -> Vec2`, `widgets::tabular_label(ui, text, font, color) -> Response` (all in `ui/widgets.rs`).
- Produces: `pub fn paint_tabular_right(painter: &Painter, right_top: Pos2, text: &str, font: &FontId, color: Color32) -> Rect` and `pub fn time_badge_width(painter: &Painter, caption: &str, value: &str) -> f32` in `ui::widgets`.

- [ ] **Step 1: Write the failing tests**

`crates/fp-app/tests/tabular.rs` already has a `widths` helper that measures with the app fonts installed. Add a second helper for painted rectangles and the tests:

```rust
/// The rectangle `paint_tabular_right` covers for each text, with the app's
/// fonts installed.
fn right_rects(texts: &'static [&'static str], font: egui::FontId) -> Vec<egui::Rect> {
    let mut h = Harness::new_ui_state(
        move |ui, (themed, out): &mut (bool, Vec<egui::Rect>)| {
            if !*themed {
                theme::apply(ui.ctx());
                *themed = true;
                return;
            }
            *out = texts
                .iter()
                .map(|t| {
                    fp_app::ui::widgets::paint_tabular_right(
                        ui.painter(),
                        egui::pos2(200.0, 10.0),
                        t,
                        &font,
                        egui::Color32::WHITE,
                    )
                })
                .collect();
        },
        (false, Vec::new()),
    );
    h.run_steps(3);
    h.state().1.clone()
}

#[test]
fn a_right_aligned_time_keeps_its_left_edge_as_digits_change() {
    let r = right_rects(&["-00:09", "-08:88", "-11:11"], font(10.0));
    assert_eq!(r[0].right(), 200.0);
    assert_eq!(r[0].width(), r[1].width());
    assert_eq!(r[1].width(), r[2].width());
}

#[test]
fn hour_times_keep_their_width_inside_their_format() {
    let r = right_rects(&["1:11:11", "8:08:08", "-1:11:11"], font(10.0));
    assert_eq!(r[0].width(), r[1].width());
    assert!(r[2].width() > r[0].width()); // the sign is its own glyph
}

#[test]
fn a_nan_time_still_has_a_width() {
    // `format::clock` maps NaN and negatives to 00:00.
    let r = right_rects(&[&"00:00"], font(10.0));
    assert!(r[0].width() > 0.0);
}

#[test]
fn the_intro_badge_keeps_its_width_from_9_9_to_10_0() {
    let w = badge_widths(&["9.9", "1.1", "8.8", "10.0", "12.3"]);
    assert_eq!(w[0], w[1]);
    assert_eq!(w[1], w[2]);
    assert_eq!(w[2], w[3]); // the box is sized for 00.0
    assert_eq!(w[3], w[4]);
}
```

with `badge_widths` calling `fp_app::ui::widgets::time_badge_width(ui.painter(), "INTRO", t)` the same way (copy `right_rects`, collect `f32`s). `a_nan_time_still_has_a_width` takes `fp_app::ui::format::clock(f64::NAN)` as its text; make the helper take `Vec<String>` instead of `&'static [&'static str]` if a computed text is needed, and use `format::clock(f64::NAN)`.

For the clock in the top bar add to `tabular.rs`:

```rust
mod support;

#[test]
fn the_top_bar_clock_keeps_its_width() {
    let (mut h, _fake) = support::harness(support::state(1, 0));
    h.run_steps(2);
    let is_clock = |s: &str| {
        s.len() == 8
            && s.char_indices().all(|(i, c)| if i == 2 || i == 5 { c == ':' } else { c.is_ascii_digit() })
    };
    let clock = h
        .get_all_by_role(egui::accesskit::Role::Label)
        .find(|n| n.label().is_some_and(|l| is_clock(&l)))
        .expect("the clock label");
    let text = clock.label().unwrap();
    let expected = widths_of(&text, font(13.0)); // tabular width of this text
    assert!((clock.rect().width() - expected).abs() < 0.5, "{} vs {expected}", clock.rect().width());
}
```

where `widths_of` is `widths(&[text], font).first().1` (the helper takes `&'static`; make a variant taking `Vec<String>`). `tabular_label` sets a `Label` widget_info with the text, so the node exists only when the clock goes through it. (If `n.label()` is not the kittest API name, use `egui_kittest::kittest::NodeT::label`; the existing tests use `h.get_by_label(...)`.)

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-app --test tabular -q`. Expected: compile error (`paint_tabular_right`, `time_badge_width`), then the clock test failing (the clock is a plain label).

- [ ] **Step 3: Implement**

`ui/widgets.rs`:

```rust
/// As [`paint_tabular`], with the text ending at `right_top.x`.
pub fn paint_tabular_right(
    painter: &Painter,
    right_top: Pos2,
    text: &str,
    font: &FontId,
    color: Color32,
) -> Rect {
    let width = tabular_size(painter, text, font).x;
    paint_tabular(painter, pos2(right_top.x - width, right_top.y), text, font, color)
}
```

`time_badge`: the value is laid out with `tabular_size` in a box of at least `tabular_size("00.0")` and painted with `paint_tabular`; factor the width into

```rust
/// The width of an intro / outro badge: the value box is sized for `00.0`
/// (feedback 2 spec O33), so the badge does not change width from 9.9 to 10.0.
pub fn time_badge_width(painter: &Painter, caption: &str, value: &str) -> f32 {
    let cap = painter.layout_no_wrap(caption.to_owned(), font_semibold(9.0), Color32::WHITE);
    let big = font_medium(22.0);
    let value_w = tabular_size(painter, value, &big)
        .x
        .max(tabular_size(painter, "00.0", &big).x);
    cap.size().x + 6.0 + value_w + 16.0
}
```

and have `time_badge` call it for `width`, take `height = tabular_size(...).y + 4.0`, and paint the value with `paint_tabular(painter, pos2(rect.left() + 8.0 + cap.size().x + 6.0, rect.top() + 2.0), value, &big, text)`.

`ui/cartwall.rs`: the `p.text(inner.right_bottom(), Align2::RIGHT_BOTTOM, &view.time, font(10.0), colour)` becomes `widgets::paint_tabular_right(p, pos2(inner.right(), inner.bottom() - tabular_size(p, &view.time, &font(10.0)).y), &view.time, &font(10.0), colour)`.

`ui/widgets.rs` waveform hover time: `let galley = painter.layout_no_wrap(text, font(10.0), theme::TEXT); let tw = galley.size().x + 8.0;` becomes `let tw = tabular_size(painter, &text, &font(10.0)).x + 8.0;` and the `painter.galley(...)` becomes `paint_tabular(painter, pos2(lx + 4.0, bg.top() + 1.0), &text, &font(10.0), theme::TEXT)`.

`ui/player.rs` `edit_markers`, the marker-drag `painter.text(pos2(x + 4.0, inner.bottom() - 4.0), LEFT_BOTTOM, format::clock(secs_at(x)), font(10.0), theme::TEXT)` becomes a `paint_tabular` at `pos2(x + 4.0, inner.bottom() - 4.0 - tabular_size(..).y)`.

`ui/app.rs` `top_bar`: the clock `ui.add(egui::Label::new(RichText::new(clock).font(font(13.0)).color(theme::NEUTRAL_200)).selectable(false))` becomes `widgets::tabular_label(ui, &clock, &font(13.0), theme::NEUTRAL_200);`.

Re-run the audit command from "Decisions" to confirm no other live time is left: `grep -n "format::clock\|format::countdown\|Local::now\|time_badge" crates/fp-app/src -r`.

- [ ] **Step 4: Run tests**

Run: `cargo test -p fp-app -q`. Expected: PASS (the existing `tabular.rs` tests, the cartwall and player tests included).

- [ ] **Step 5: Document**

`docs/technical/ui.md`: a short "Times" paragraph: live times are drawn in equal digit cells (`paint_tabular`, `paint_tabular_right`, `tabular_label`) and the intro/outro badges have a value box sized for `00.0`; list the sites. No user-guide text (not visible as a feature).

- [ ] **Step 6: Gate and commit**

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q
git add crates docs
git commit -m "fix(ui): keep every live time at a fixed width"
scripts/check-commits.sh origin/master
```

---

### Task 5: O35, playlist tabs that shrink, cut and scroll

**Files:**
- Create: `crates/fp-app/src/ui/tab_strip.rs`, `crates/fp-app/tests/tab_strip.rs`, `crates/fp-app/tests/playlist_tabs_ui.rs`
- Modify: `crates/fp-app/src/ui.rs` (`pub mod tab_strip;`), `crates/fp-app/src/ui/app.rs` (`ViewState::tab_scroll`), `crates/fp-app/src/ui/player.rs` (`tabs`)
- Docs: `docs/user/playlists.md` ("Tabs"), `docs/technical/ui.md`

**Interfaces:**
- Produces, in `fp_app::ui::tab_strip` (pure):

```rust
pub const MIN_TAB_WIDTH: f32 = 72.0;
pub const ARROW_WIDTH: f32 = 22.0;

/// How `count` tabs are laid out in a strip `available` pixels wide.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    pub tab_width: f32,
    /// The tabs do not fit: arrows show and the tabs scroll.
    pub overflow: bool,
    /// Width of the clipped view the tabs sit in (the strip, less the arrows).
    pub view_width: f32,
    /// Total width of the tabs.
    pub content_width: f32,
}

pub fn layout(available: f32, count: usize) -> Layout;
/// `offset` limited to what the content can scroll.
pub fn clamp_offset(offset: f32, content: f32, view: f32) -> f32;
/// The least change of `offset` that shows tab `index` whole.
pub fn reveal(offset: f32, index: usize, layout: &Layout) -> f32;
/// One arrow press: a tab to the left (`-1`) or right (`1`).
pub fn step(offset: f32, direction: i32, layout: &Layout) -> f32;
```

  and in `ViewState` (`ui/app.rs`): `pub tab_scroll: HashMap<PlayerId, TabScroll>` with `pub(crate) struct TabScroll { pub offset: f32, pub revealed: Option<(PlaylistId, usize, u32)> }` (shown playlist, tab count, `view_width.to_bits()`; the tab is revealed again when this key changes).

- [ ] **Step 1: Write the failing unit tests** (`crates/fp-app/tests/tab_strip.rs`)

```rust
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
//! The layout of the playlist tabs (feedback 2 spec O35).

use fp_app::ui::tab_strip::{ARROW_WIDTH, MIN_TAB_WIDTH, clamp_offset, layout, reveal, step};

#[test]
fn few_tabs_fill_the_strip_without_arrows() {
    let l = layout(600.0, 3);
    assert_eq!(l.tab_width, 200.0);
    assert!(!l.overflow);
    assert_eq!(l.view_width, 600.0);
    assert_eq!(layout(600.0, 1).tab_width, 600.0);
    assert!(!layout(600.0, 0).overflow);
}

#[test]
fn tabs_shrink_to_the_minimum_then_scroll() {
    let fits = layout(MIN_TAB_WIDTH * 4.0, 4);
    assert!(!fits.overflow);
    assert_eq!(fits.tab_width, MIN_TAB_WIDTH);
    let l = layout(MIN_TAB_WIDTH * 4.0, 5);
    assert!(l.overflow);
    assert_eq!(l.tab_width, MIN_TAB_WIDTH);
    assert_eq!(l.view_width, MIN_TAB_WIDTH * 4.0 - 2.0 * ARROW_WIDTH);
    assert_eq!(l.content_width, MIN_TAB_WIDTH * 5.0);
}

#[test]
fn a_strip_narrower_than_its_arrows_does_not_panic() {
    let l = layout(10.0, 30);
    assert!(l.overflow);
    assert_eq!(l.view_width, 0.0);
    assert_eq!(clamp_offset(50.0, l.content_width, l.view_width), l.content_width);
    assert_eq!(reveal(0.0, 29, &l), l.content_width); // no negative or NaN
    let nan = layout(f32::NAN, 3);
    assert!(nan.tab_width.is_finite() && nan.view_width.is_finite());
}

#[test]
fn the_offset_is_clamped_when_the_tabs_get_fewer() {
    assert_eq!(clamp_offset(900.0, 360.0, 200.0), 160.0);
    assert_eq!(clamp_offset(-5.0, 360.0, 200.0), 0.0);
    assert_eq!(clamp_offset(30.0, 100.0, 200.0), 0.0); // fits: no scroll
}

#[test]
fn reveal_scrolls_the_least_that_shows_the_tab() {
    let l = layout(MIN_TAB_WIDTH * 3.0, 10); // overflow
    let view = l.view_width;
    // Already whole in view: unchanged.
    assert_eq!(reveal(0.0, 0, &l), 0.0);
    // Off to the right: its right edge meets the view's right edge.
    let o = reveal(0.0, 9, &l);
    assert!((o + view - 10.0 * MIN_TAB_WIDTH).abs() < 0.01, "{o}");
    // Off to the left: its left edge meets the view's left edge.
    let o = reveal(o, 2, &l);
    assert!((o - 2.0 * MIN_TAB_WIDTH).abs() < 0.01, "{o}");
}

#[test]
fn an_arrow_moves_one_tab_and_stops_at_the_ends() {
    let l = layout(MIN_TAB_WIDTH * 3.0, 10);
    assert_eq!(step(0.0, 1, &l), MIN_TAB_WIDTH);
    assert_eq!(step(0.0, -1, &l), 0.0);
    let max = l.content_width - l.view_width;
    assert_eq!(step(max - 5.0, 1, &l), max);
}
```

- [ ] **Step 2: Write the failing UI tests** (`crates/fp-app/tests/playlist_tabs_ui.rs`)

Build a state with many playlists through `Command::CreatePlaylist { .. }` and `Command::RenamePlaylist { playlist, name }` (read both variants in `crates/fp-model/src/command.rs` and apply them with `fp_model::apply` as `tests/support/mod.rs` does). Use `support::harness_sized(state, vec2(1000.0, 700.0), |ui| ui)`; every playlist tab is a button whose accessible label is the playlist name (see `response.widget_info` in `tabs`).

```rust
#[test]
fn a_name_of_200_characters_is_cut_and_the_strip_stays_in_its_column() {
    // Two playlists, one named with 200 characters. The tab is a button
    // named by the full name; its rect never leaves the player column.
    let long = "A".repeat(200);
    // ... build the state, run_steps(3)
    let tab = h.get_by_label(&long).rect();
    let column = h.get_by_label("Add").rect(); // any footer widget of the same column
    assert!(tab.width() <= 1000.0 / 2.0, "{tab:?}");
    assert!(tab.right() <= column.right() + 1.0);
}

#[test]
fn thirty_tabs_scroll_and_the_shown_one_is_in_view() {
    // 30 playlists "List 1".."List 30"; show the last one on player 1
    // (Command::ShowPlaylist). After a few frames the "List 30" tab lies
    // inside the column and the right arrow is gone, the left one shown.
}

#[test]
fn the_arrows_move_the_strip_by_one_tab() {
    // 30 tabs, "List 1" shown. Click the right arrow (accessible label
    // "Scroll tabs right"; it needs a Fluent key, see Step 3): the first
    // tab's left edge moves left by one tab width; the left arrow moves it back.
}

#[test]
fn the_wheel_over_the_strip_scrolls_it() {
    // h.hover_at(a point on a tab); h.event(egui::Event::MouseWheel { unit:
    // egui::MouseWheelUnit::Line, delta: egui::vec2(0.0, -3.0), modifiers:
    // egui::Modifiers::NONE }); run_steps(3); the first tab moved left.
}

#[test]
fn a_tab_click_still_shows_the_playlist() {
    // Click "List 3" among 30 tabs: Command::ShowPlaylist(player, id) sent.
}

#[test]
fn few_tabs_show_no_arrows() {
    // The default state (one playlist): no "Scroll tabs left/right" buttons.
}
```

Write every test body fully when implementing (the comments above name each assertion); shared helper `many_playlists(n: usize) -> AppState`.

- [ ] **Step 3: Run them to see them fail**

Run: `cargo test -p fp-app --test tab_strip -q` (compile error: module missing), then the UI tests after the module exists (they fail on behaviour).

- [ ] **Step 4: Implement the pure module**

`crates/fp-app/src/ui/tab_strip.rs`: the items of the Interfaces block, with non-finite `available` treated as 0, `tab_width = available / count` when `count > 0` and it is at least `MIN_TAB_WIDTH`; otherwise `MIN_TAB_WIDTH` with `overflow = true`, `view_width = (available - 2 * ARROW_WIDTH).max(0.0)`, `content_width = count * MIN_TAB_WIDTH`. `clamp_offset(o, content, view) = o.clamp(0.0, (content - view).max(0.0))` (guard `o` NaN to 0). `reveal`: `left = index * tab_width`, `right = left + tab_width`; if `left < offset` then `left`, else if `right > offset + view` then `right - view`, else unchanged; clamp the result. `step(offset, direction, l) = clamp_offset(offset + direction as f32 * l.tab_width, ...)`. Add `pub mod tab_strip;` to `ui.rs` (alphabetical, after `shell`).

- [ ] **Step 5: Implement the drawing** (`ui/player.rs`, function `tabs`)

Keep the head of `tabs` (current and next playlists) and the per-tab drawing (fill, accent line, separator, dot, `dnd` handling, click handling, tooltip, `widget_info`) but change the geometry:

1. `let l = tab_strip::layout(strip.width(), count)`; the tabs live in `view = Rect::from_min_size(pos2(strip.left() + if l.overflow { ARROW_WIDTH } else { 0.0 }, strip.top()), vec2(l.view_width, TABS_HEIGHT))`.
2. State: `let st = view_state.tab_scroll.entry(id).or_default();` Compute the reveal key `(shown, count, l.view_width.to_bits())`; when it differs from `st.revealed`, set `st.offset = reveal(st.offset, index_of_shown, &l)` and store the key. Always `st.offset = clamp_offset(st.offset, l.content_width, l.view_width)`; when `!l.overflow` set it to 0.
3. Wheel: when `l.overflow` and `ui.rect_contains_pointer(strip)`, take `ui.input(|i| i.smooth_scroll_delta)` (and zero it with `ui.input_mut(|i| i.smooth_scroll_delta = Vec2::ZERO)` so the table below does not also scroll; check how `table.rs` reads scroll so the two do not both react to a pointer over the tabs), then `st.offset = clamp_offset(st.offset - (delta.x + delta.y), ...)`. Verify the sign in the wheel test.
4. Arrows (only when `l.overflow`): two `widgets::tile`s of `vec2(ARROW_WIDTH, TABS_HEIGHT)` at the strip's ends with `icon::CARET_LEFT` / `icon::CARET_RIGHT`, labels from new Fluent keys `tip-tabs-scroll-left` / `tip-tabs-scroll-right` ("Scroll tabs left" / "Scroll tabs right"; Spanish "Desplazar las pestañas a la izquierda" / "...a la derecha"), disabled (dimmed) at the ends; a click sets `st.offset = step(st.offset, ±1, &l)`.
5. Each tab's rect is `Rect::from_min_size(pos2(view.left() - st.offset + i as f32 * l.tab_width, strip.top()), vec2(l.tab_width, TABS_HEIGHT))`. Draw and `interact` only the part inside `view`: use `let painter = ui.painter().with_clip_rect(view)` for the fill and text and skip `interact` for tabs whose rect does not intersect `view`; clip the interaction by intersecting: `ui.interact(rect.intersect(view), ...)` (egui ignores empty rects).
6. Name: replace `painter.layout(name, font(11.0), color, wrap_width)` by a one-row job so the cut ends in "…":

```rust
let mut job = egui::text::LayoutJob::simple_singleline(pl.name.clone(), font(11.0), color);
job.wrap = egui::text::TextWrapping {
    max_width: (l.tab_width - 24.0).max(8.0),
    max_rows: 1,
    break_anywhere: true,
    overflow_character: Some('…'),
};
let galley = painter.layout_job(job);
```

   (check `egui::text::TextWrapping` fields in egui 0.36.2: `max_width`, `max_rows`, `break_anywhere`, `overflow_character`).
7. The tooltip with the full name stays (`on_hover_text(&pl.name)`); the shown tab keeps its accent line.

Keep the bottom 1 px rule across the whole strip.

- [ ] **Step 6: Run tests**

Run: `cargo test -p fp-app -q`. Expected: PASS: the new tests and the existing ones that click tabs (`main_screen.rs`, `table_follow.rs`, `track_tags_ui.rs`, the drag-onto-a-tab tests). Fix any test that depended on equal widths over the whole strip.

- [ ] **Step 7: Document**

`docs/user/playlists.md` "Tabs": tabs share the width; long names end in "…" (hover for the full name); with many playlists the tabs stop shrinking, arrows appear at the ends and the mouse wheel scrolls them, and the tab you choose or the one a player shows is brought into view. `docs/technical/ui.md`: `ui/tab_strip.rs` (pure layout), `ViewState::tab_scroll`, when the shown tab is revealed.

- [ ] **Step 8: Gate and commit**

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q
git add crates docs
git commit -m "feat(ui): shrink, cut and scroll the playlist tabs"
scripts/check-commits.sh origin/master
```

---

### Task 6: Spec "As built", roadmap, README and a last check

**Files:**
- Modify: `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md` (header status line; "As built" at the end of §13), `docs/superpowers/plans/2026-10-01-operator-feedback-2-roadmap.md` (row 12 `done`), `README.md`, and any doc a grep finds stale.

- [ ] **Step 1: Sweep for stale statements**

```bash
grep -rn "top bar" README.md docs/user docs/technical | grep -i "name\|version"
grep -rn "tip-about-name\|About Fauste Player (name" crates docs
grep -rn "windows_subsystem\|CONSOLE\|emit(" crates/fp-app/src | head
```

Expected: nothing says the name or version is in the top bar; `tip-about-name` is gone. README: the feature list mentions the title bar, the fitting cartwall, fixed-width times and scrolling tabs in the right places (the features list and the roadmap table, if it lists plans); the platform table or install section notes that the Windows release opens no console.

- [ ] **Step 2: Spec**

Header line `- **Status:** Approved. Plans 1 to 9 are built; ...` becomes `Plans 1 to 9 and 12 are built`. At the end of §13, after the O36 bullet, add an `- **As built.**` bullet list with one line per item, written from what the tasks really did: O31 `cart_view::button_height`, 40 → 28 px, 60% cap unchanged; O32 `cli::window_title`, top-bar brand removed, `tip-about-name` removed, macOS shows no icon in its title bar by design; O33 the sites changed and the sites left alone (with the reason); O35 `ui/tab_strip.rs`, 72 px minimum, equal widths, arrows and wheel, reveal key; O36 the attribute, `cli::emit`, the PE check in `windows.sh`, what was checked in CI and the packaging scripts, and that a Windows run by hand was not done in this plan unless it was.

- [ ] **Step 3: Roadmap and verification**

Roadmap row 12: status `outline` -> `done`. Then run, before saying done (superpowers:verification-before-completion):

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q
scripts/check-windows-gui.sh --self-test
```

- [ ] **Step 4: Commit**

```bash
git add README.md docs
git commit -m "docs: window and layout notes, plan 12 as built, roadmap row 12 done"
scripts/check-commits.sh origin/master
```

Then the plan's end-of-plan review (superpowers:requesting-code-review, a fresh reviewer on the most capable model; Critical and Important findings fixed test-first, Minor ones deferred in the ledger) and the pull request as in CLAUDE.md.

---

## Self-Review

**Spec coverage.**
- O31: Task 3 (`button_height`, the grid, the scroll only below the minimum). 
- O32: Task 1 (native title with icon, name and version; the top bar repeats nothing; the maintainer's one-title-bar decision; macOS icon caveat in Decisions).
- O33: Task 4 (every live time audited; the sites changed and the sites left alone are listed).
- O35: Task 5 (shrink to a minimum, "…", tooltip already there, arrows, wheel, selected tab revealed).
- O36: Task 2 (attribute, `--version` and `--help` through a box, debug builds keep a console, CI and packaging checked and a PE check added).
- Docs for each behaviour change are inside its task; the spec "As built", the roadmap row and the README sweep are Task 6.

**Placeholder scan.** The Task 5 UI test bodies are given as named assertions with the exact accessible labels, the events and the helper; the implementer writes them out in full as Step 2 says. Task 2's `fake_pe` byte layout and Task 3's possible `MIN_BUTTON_HEIGHT` bump are called out with the check that settles them. No TBD or "similar to task N".

**Type consistency.** `cli::window_title`, `cli::{CONSOLE, Stream, emit}` (Tasks 1, 2); `cart_view::{BUTTON_HEIGHT, MIN_BUTTON_HEIGHT, GAP, button_height}` (Task 3, used by `cartwall.rs` and the tests); `widgets::{paint_tabular_right, time_badge_width}` (Task 4, tests and callers); `tab_strip::{MIN_TAB_WIDTH, ARROW_WIDTH, Layout, layout, clamp_offset, reveal, step}` and `ViewState::tab_scroll` (Task 5). Every name is the same where it appears twice.

**Review Focus.** Each of the five lines names the tests in the task that owns the code.

**Known weak spots to watch in review.**
- Task 5 changes a 140-line drawing function that also handles drag and drop onto tabs; the existing drag tests must keep passing and a drop onto a partly hidden tab should work on its visible part.
- The cartwall minimum height (28 px) is a visual judgement; Task 3 Step 5 asks for a look at it.
- The Windows behaviour (no console, message boxes) is verified by the PE check and by compile only; nothing runs a Windows GUI in CI.
