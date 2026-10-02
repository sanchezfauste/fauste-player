# Audit Follow-ups (Feedback 2, Plan 9) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the items the audit of the earlier plans left open (O21): the remote plan 4 minors M1, M4, M5, M6 and M7 (M2 stays a recorded ruling), the missing-file reason tooltip on the title cell, the `docs/user/players.md` peak-hold sentence, decoder and analysis thread priorities through the `thread-priority` crate, and a long-file check of the real application (four hours, seek to 3:59:00 through the remote API).

**Architecture:**
- **Small, independent fixes first.** Tasks 1 to 6 are separate commits, each with its own failing test: the test hygiene of M7 (an honest name for the alignment line thickness, and a `bind_log` test that no longer sleeps 5.5 s because `fp_remote::spawn_with` takes a `Timing`), a number typed into a drag value in Settings > Remote that survives opening another section (M1), carts whose track has no recorded format analysed at once even after "Later" (M4), the remote peaks of tracks an earlier version analysed (M5, worked out on request, cached and one at a time), the outdated-track count worked out once per model snapshot instead of on every frame (M6), and the reason tooltip on the title cell of an unavailable row.
- **Thread priority.** `fp_decode::priority` (one small module, shared by `fp-engine` and `fp-analysis`, which both already depend on `fp-decode`) wraps `thread_priority::set_current_thread_priority`: `Priority::AboveNormal` (decode workers) and `Priority::Low` (the analysis pool). A refusal returns `false` and is logged once per run. Only the device callback stays real time.
- **Verification and documentation.** Task 9 is the long-file check under Xvfb (a measurement, with a defect fixed test-first inside the task if it finds one). Task 10 aligns the docs, the specs, the roadmap and the README.

**Tech Stack:** Rust, egui/eframe 0.36.2, egui_kittest 0.36.2, axum (through `fp-remote`). One new dependency: `thread-priority` 3.1.1 (MIT; `cargo deny check` passes with the existing `deny.toml`, no new entry, see Task 7).

**Spec:** `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md` §10 (item O21) and §13 (global constraints; §12 before the O25–O30 renumbering). Roadmap: `docs/superpowers/plans/2026-10-01-operator-feedback-2-roadmap.md` (plan 9, branch `fix/audit-follow-ups`). The minors M1 to M7 come from the final review of the remote plan 4 (`docs/superpowers/plans/2026-10-01-remote-plan4-followups.md`, ledger `.superpowers/sdd/2026-10-01-remote-plan4-followups/progress.md`); the missing-file minor M2 comes from `.superpowers/sdd/2026-10-01-missing-file-recheck/progress.md`. Main spec §2.2 describes the thread priorities.

**Base.** Plan 8 (`feat/table-columns`) rewrote `ui/table.rs` and touched `ui/app.rs`, `ui/settings.rs`, `README.md` and several docs that this plan changes too, so this plan is written against plan 8's code. Start the branch `fix/audit-follow-ups` from `master` after plan 8 is merged. The plan was replayed task by task on a scratch copy of `feat/table-columns` (`f230760`); line numbers move, so the steps name functions and give the text to find.

## Decisions taken

The spec leaves these open. Each is the most conservative reading.

- **M2 of the remote plan (Matroska Opus pre-skip, ±24 samples) stays a recorded ruling.** After a seek to 0 of a Matroska Opus file the pre-skip is dropped by timestamp, so the start is off by at most 24 samples (0.5 ms). Nothing changes; the ruling goes into the spec's "As built" (Task 10) so it is not found again by the next audit.
- **M1: the drag value updates while it is edited.** `number()` in `ui/settings/remote.rs` used `update_while_editing(false)`, so the value it kept for `flush` was the one from before the typing started. Without that call the typed value reaches `RemoteState::numbers` as it is typed; the function still returns a value only after the drag or the focus ended, so no step restarts a server. Opening another section applies the typed number like Enter does, including the clamp to the range (typing `90` for the port gives 1024); Esc still cancels it.
- **M4: "carts with no analysed format".** A track on a cart that is analysed by an earlier version (`analyzed`, readable file, `format` is `None`) is analysed at once even if the operator chose "Later". A cart track whose analysis only has an older `analysis_version` but has its format keeps waiting for the operator: the format is what bit-perfect needs. A track that is not analysed yet is analysed as before.
- **M5: peaks and cover on demand.** A track the model calls analysed, but for which the analysis cache has no entry of the current version (the cache sweep removed the old ones), used to answer 404 unless a player showed it. Now `Bridge::analysis` (`fp-app/src/remote.rs`) analyses it on the request's own blocking thread, stores the result in the cache and answers. At most one such analysis runs at a time (`Bridge::on_demand`), so a client that asks for many tracks cannot take every core; the model keeps what it had until the operator asks for the new analysis. A track that is not analysed yet is still `404 not_analyzed` and is never analysed for a request; a file that cannot be decoded is `404 not_found`. `docs/technical/remote-api.md` said "neither ever starts an analysis": Task 10 rewrites that sentence.
- **M6: one scan per model snapshot.** `services::OutdatedCount` remembers the `Arc<AppState>` it counted and the count; a new snapshot (any model change) is a new scan, a repeated frame is a pointer comparison. It does not compare libraries. `SettingsDeps::outdated` carries the count to Settings so `ui/settings.rs` does not scan.
- **M7: a thickness, and a test with its own clock.** The constant is the vertical extent of horizontal marks, so it is `ALIGNMENT_LINE_THICKNESS`. `fp_remote::spawn_with(control, Timing)` shortens the supervisor's configuration poll and bind retry for the test; `spawn` keeps the production values (250 ms and 2 s). The test counts distinct published status snapshots: every retry stores a new one.
- **Missing-file M2: the title cell, only for unavailable rows.** The same tooltip text as the icon (`Scene::file_tip`), attached to the title label when the row's status is `Unavailable`. The row's own tooltip (`track_tip`) is unchanged and shows too.
- **Priorities.** Decode workers (`PlayerWorker`: one per player, and the cartwall's) ask for `AboveNormal`, the crate's cross-platform value 60 (nice −5 on Linux, above normal on Windows, a step above the default on macOS). The analysis pool asks for `Low`, the crate's minimum (nice 19, lowest). Nothing else changes: the conductor, services, remote, tag worker, file probe and helpers stay at normal priority, and persistence writes stay on the services thread (main spec §2.2 said "background pool"; Task 10 corrects it). The priority is not an operator value, so there is no `Config` field (CLAUDE.md rule 4 covers what an operator might change; this is an implementation detail of the thread classes). Cost if wrong: an operator who wants other priorities needs a code change.
- **Where the code lives.** `fp_decode::priority`, because `fp-engine` and `fp-analysis` both depend on `fp-decode` and a new crate would need workspace, release and documentation entries for 120 lines. `fp-decode` gains `thread-priority` and `tracing`.
- **A refusal.** On Linux an ordinary user may lower a nice value but not raise it, so the usual outcome of `AboveNormal` there is a refusal (`error code 13`), logged once as a `warn` and ignored; the analysis pool still runs at nice 19. Both outcomes are tested.
- **Long-file check.** The file is generated (pink noise from `ffmpeg`, 4 h, WAV); the app runs as in the CLAUDE.md recipe, with the fader at zero and no reachable sound server, so nothing is audible on the machine.

## Global Constraints

- All code, identifiers, comments, docs, specs, plans and commit messages are in English. Never mention other playout, radio-automation or tag-editor products.
- Spec §13 (binding; the spec's own words):

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

- Spec §10 (binding; the spec's own words):

> - **Remote plan 4 minors.**
>   - M1: a port typed into a drag value in Settings > Remote is kept when
>     another section opens.
>   - M4: carts with no analysed format are analysed even when "Later" is chosen.
>   - M5: `GET /tracks/{id}/peaks` works for outdated tracks.
>   - M6: `outdated_tracks` is recomputed when the library changes, not every
>     frame.
>   - M7: rename `ALIGNMENT_LINE_WIDTH`, and remove the 5.5 s sleep in the
>     `bind_log` test.
>   - M2 (Matroska Opus pre-skip, ±24 samples) stays a recorded ruling.
> - **Missing-file plan minor M2.** The missing-file reason tooltip also shows on
>   the title cell.
> - **Docs.** `docs/user/players.md` says that only digital peak, K-System and
>   custom meters show a peak hold.
> - **Thread priority** (main spec §2.2):
>   - decoder threads run above normal priority (not real time);
>   - the analysis pool runs at low priority;
>   - this uses the `thread-priority` crate, after `cargo deny check`;
>   - when the system refuses the change, the app logs it once and goes on;
>   - `docs/technical/threading-and-realtime.md` is aligned.
> - **Long-file check.**
>   1. In Xvfb, play a four-hour file in the app.
>   2. Seek to 3:59:00 through the remote API.
>   3. Record the result in the ledger.

- CLAUDE.md rule 5: nothing here touches the device callback. The priority calls run first thing on non-real-time threads, never in the mixer or the backend render path.
- CLAUDE.md rule 6: no `unwrap`, `expect` or `panic` outside tests; `fp-decode`, `fp-engine` and `fp-analysis` deny `clippy::indexing_slicing` (use `get`). `unsafe_code` is forbidden in this workspace; the `thread-priority` crate holds the unsafe system calls, this code only calls its safe API.
- CLAUDE.md rules 8 and 9: the UI never blocks (the on-demand analysis of M5 runs on the remote thread's blocking pool, never on the UI thread) and bad data degrades (an undecodable file is a 404, a refused priority is a log line).
- UI strings are Fluent messages in both locales: this plan adds none.
- Commands: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p <crate> --test <file> <name>` (one filter per command). Build only into the repo's `target/`.
- Commit only when fmt, clippy and the whole suite pass: `export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q`. Commits end with the co-author trailer the harness provides (the commit commands below show the subject only). `CHANGELOG.md` is never edited by hand.
- Documentation is part of the change (Task 10).

## Review Focus

Failure modes the spec implies and its tests do not name; each has a test in the task named.

1. **Many peak requests at once, or one for a bad file** (a client that asks for the peaks of every outdated track in parallel, or twice; a file that cannot be decoded): they must neither run several analyses at once nor hang; both answers are the peaks, and the bad file is a 404. (Task 4 `two_requests_at_once_both_get_the_peaks`, `the_peaks_of_an_undecodable_file_are_not_found`.)
2. **A system that refuses the priority change** (the normal case for an unprivileged Linux user raising a priority): the app starts and plays, the thread keeps running, and the refusal is one log line however many threads ask. (Task 7 `only_the_first_refusal_is_reported`, `above_normal_is_accepted_or_leaves_the_thread_alone`; Task 8 `decoder_threads_ask_for_a_higher_priority_and_a_refusal_is_logged_once`.)
3. **A half-typed port when another section is opened or Esc is pressed** (typing `90` where the range starts at 1024; Esc while typing): opening the section applies the number as Enter would, clamped to the range; Esc applies nothing. (Task 2 `a_typed_port_below_the_range_is_clamped_like_enter`, `escape_cancels_a_typed_port`.)
4. **A cart track that must not be analysed** (one that already has its format, one whose file is missing): no analysis, no loop of retries. (Task 3 `a_cart_track_that_has_its_format_is_left_alone`, `a_cart_track_whose_file_is_missing_is_not_analysed`.)
5. **The start-up notice while the library changes under it** (a track becomes unplayable, or is analysed, while the notice is open): the count follows the new snapshot and the notice closes at zero; the cache never shows a stale number. (Task 5 `the_notice_follows_the_library_while_it_is_open`, `the_count_is_worked_out_again_only_when_the_model_changed`.)

## File Structure

- Modify `crates/fp-remote/src/server.rs` (`Timing`, `spawn_with`), `lib.rs` (re-exports), `tests/bind_log.rs`.
- Modify `crates/fp-app/src/ui/widgets.rs` (the constant).
- Modify `crates/fp-app/src/ui/settings/remote.rs` (`number`), `tests/remote_settings.rs`.
- Modify `crates/fp-app/src/services.rs` (`Services::cart_tracks`, `OutdatedCount`), `tests/services.rs`, `tests/outdated_notice.rs`; `crates/fp-app/src/ui/app.rs` and `ui/settings.rs` (`SettingsDeps::outdated`).
- Modify `crates/fp-app/src/remote.rs` (`Bridge::analysis`, `on_demand`), `tests/remote.rs`.
- Modify `crates/fp-app/src/ui/table.rs` (title tooltip), `tests/main_screen.rs`.
- Create `crates/fp-decode/src/priority.rs`; modify `crates/fp-decode/src/lib.rs`, `crates/fp-decode/Cargo.toml`, the root `Cargo.toml` (`[workspace.dependencies]`) and `Cargo.lock`.
- Modify `crates/fp-engine/src/worker.rs`, `crates/fp-engine/Cargo.toml` (dev-dependency), create `crates/fp-engine/tests/decoder_priority.rs`; modify `crates/fp-analysis/src/analyzer.rs`, `crates/fp-analysis/tests/analyzer.rs`.
- Modify docs (Task 10): `docs/user/players.md`, `docs/user/settings.md`, `docs/user/playlists.md`, `docs/user/bit-perfect.md`, `docs/technical/ui.md`, `docs/technical/analysis.md`, `docs/technical/remote-api.md`, `docs/technical/threading-and-realtime.md`, the main spec (§2.2, §12), the feedback 2 spec (status and "As built" under §10), the roadmap (row 9 done), `README.md`.

---

### Task 1: M7, the alignment constant and a `bind_log` test that does not sleep

**Files:**
- Modify: `crates/fp-app/src/ui/widgets.rs` (the constant and its three uses)
- Modify: `crates/fp-remote/src/server.rs`, `crates/fp-remote/src/lib.rs`
- Test: `crates/fp-remote/tests/bind_log.rs`

**Interfaces:**
- Consumes: the private `CONFIG_POLL` (250 ms) and `BIND_RETRY` (2 s) of `server.rs`, `RemoteHandle::status_cell()`.
- Produces (re-exported from `fp_remote`): `struct Timing { pub config_poll: Duration, pub bind_retry: Duration }` (`Default` is the production values), `fn spawn_with(control: Arc<dyn RemoteControl>, timing: Timing) -> std::io::Result<RemoteHandle>`; `spawn(control)` is `spawn_with(control, Timing::default())`.

- [ ] **Step 1: Rewrite the test so that it needs `Timing` (it fails to compile)**

In `crates/fp-remote/tests/bind_log.rs` replace the imports and the body of `a_busy_port_is_logged_once_across_retries` up to `drop(handle);`:

```rust
use std::io::Write;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use fp_remote::{ServerError, ServerStatus, Timing, spawn_with};
use support::{FakeControl, demo_state};
```

```rust
    let timing = Timing {
        config_poll: Duration::from_millis(5),
        bind_retry: Duration::from_millis(10),
    };
    let handle = spawn_with(fake.clone(), timing).unwrap();
    // Every round publishes the status again as a new snapshot, so the
    // number of distinct snapshots with the bind error is the number of
    // times the port was tried. Wait for four of them (no fixed sleep).
    let cell = handle.status_cell();
    let mut seen = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(10);
    while seen.len() < 4 {
        assert!(
            Instant::now() < deadline,
            "only {} attempts seen",
            seen.len()
        );
        let status = cell.load_full();
        let failed = matches!(status.http, ServerStatus::Error(ServerError::Bind(_)));
        if failed && !seen.iter().any(|s| Arc::ptr_eq(s, &status)) {
            seen.push(status);
        }
        std::thread::sleep(Duration::from_millis(1));
    }
```

The rest of the test (`drop(handle);`, the log text and the `assert_eq!(... .count(), 1, ...)`) stays as it is.

- [ ] **Step 2: Run it to see it fail**

Run: `cargo test -p fp-remote --test bind_log`
Expected: does not compile: `unresolved import fp_remote::Timing` (and `spawn_with`).

- [ ] **Step 3: Add `Timing` and `spawn_with`**

In `crates/fp-remote/src/server.rs`, replace `spawn` with:

```rust
/// How often the remote thread looks at things. The defaults are the ones
/// the application runs with; tests shorten them to see several rounds
/// without waiting seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timing {
    /// How often the configuration in the snapshot is looked at.
    pub config_poll: Duration,
    /// How often a server whose address could not be bound tries again.
    pub bind_retry: Duration,
}

impl Default for Timing {
    fn default() -> Self {
        Self {
            config_poll: CONFIG_POLL,
            bind_retry: BIND_RETRY,
        }
    }
}

/// Starts the remote thread. It listens only once `config.remote` asks for it.
pub fn spawn(control: Arc<dyn RemoteControl>) -> std::io::Result<RemoteHandle> {
    spawn_with(control, Timing::default())
}

/// As [`spawn`], with the thread's own timing.
pub fn spawn_with(
    control: Arc<dyn RemoteControl>,
    timing: Timing,
) -> std::io::Result<RemoteHandle> {
    let status = Arc::new(ArcSwap::from_pointee(RemoteStatus::default()));
    let (stop, stopped) = watch::channel(false);
    let shared = status.clone();
    let thread = std::thread::Builder::new()
        .name("fp-remote".to_owned())
        .spawn(move || run(control, shared, stopped, timing))?;
    Ok(RemoteHandle {
        status,
        stop,
        thread: Some(thread),
    })
}
```

Then thread `timing` through: `fn run(control, status, stop, timing: Timing)` calls `rt.block_on(supervise(control, status, stop, timing))`; `async fn supervise(control, status, mut stop, timing: Timing)` uses `if tried.elapsed() >= timing.bind_retry {` and `() = tokio::time::sleep(timing.config_poll) => {}`. The constants `CONFIG_POLL` and `BIND_RETRY` stay (they are the defaults).

In `crates/fp-remote/src/lib.rs`:

```rust
pub use server::{
    RemoteHandle, RemoteStatus, ServerError, ServerStatus, Timing, spawn, spawn_with,
};
```

- [ ] **Step 4: Rename the alignment constant**

In `crates/fp-app/src/ui/widgets.rs` replace every `ALIGNMENT_LINE_WIDTH` with `ALIGNMENT_LINE_THICKNESS` (the definition and the two uses in `meter_layout`) and replace the definition's comment:

```rust
/// Thickness of the alignment line and of the notches: the vertical extent
/// of the horizontal marks (not their length).
const ALIGNMENT_LINE_THICKNESS: f32 = 2.0;
```

Run `cargo fmt --all` (it wraps the two longer lines).

- [ ] **Step 5: Run the test**

Run: `cargo test -p fp-remote --test bind_log`
Expected: PASS in well under a second (it took 5.5 s before).

- [ ] **Step 6: Gate and commit**

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q
git add crates/fp-remote crates/fp-app/src/ui/widgets.rs
git commit -m "test(remote): count bind retries instead of sleeping, and rename the alignment line thickness (M7)"
```

---

### Task 2: M1, a number typed in Settings > Remote survives opening another section

**Files:**
- Modify: `crates/fp-app/src/ui/settings/remote.rs` (`number`)
- Test: `crates/fp-app/tests/remote_settings.rs`

**Interfaces:**
- Consumes: `RemoteState::numbers`, `remote::flush`, `commit_number` (all existing); the `opened(state)` helper and `sent_configs` of the test file.
- Produces: nothing new (a behaviour fix).

- [ ] **Step 1: Write the failing tests**

Append to `crates/fp-app/tests/remote_settings.rs`:

```rust
/// Types `text` into the port field (a drag value), leaving it focused and
/// the Enter key unpressed.
fn type_port(h: &mut egui_kittest::Harness<'static, fp_app::ui::app::AppUi>, text: &str) {
    h.get_all_by_role_and_label(Role::SpinButton, "Port")
        .next()
        .unwrap()
        .focus();
    h.run_steps(2);
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.run_steps(1);
    for c in text.chars() {
        h.get_all_by_role_and_label(Role::SpinButton, "Port")
            .next()
            .unwrap()
            .type_text(&c.to_string());
        h.run_steps(1);
    }
}

#[test]
fn a_typed_port_is_applied_when_another_section_is_opened() {
    let (mut h, fake) = opened(state(1, 0));
    type_port(&mut h, "9000");
    h.get_by_role_and_label(Role::Button, "MIDI").click();
    h.run_steps(3);
    assert_eq!(fake.state.load().config.remote.http.port, 9000);
}

#[test]
fn a_typed_port_is_applied_when_settings_closes() {
    let (mut h, fake) = opened(state(1, 0));
    type_port(&mut h, "9001");
    h.get_by_label("Close").click();
    h.run_steps(3);
    assert_eq!(fake.state.load().config.remote.http.port, 9001);
}

#[test]
fn escape_cancels_a_typed_port() {
    let (mut h, fake) = opened(state(1, 0));
    type_port(&mut h, "9002");
    h.key_press(egui::Key::Escape);
    h.run_steps(3);
    h.key_press(egui::Key::Escape);
    h.run_steps(3);
    assert_ne!(fake.state.load().config.remote.http.port, 9002);
    assert!(sent_configs(&fake).is_empty());
}
```

- [ ] **Step 2: Run them to see the first fail**

Run: `cargo test -p fp-app --test remote_settings typed_port`
Expected: `a_typed_port_is_applied_when_another_section_is_opened` FAILS with `left: 7380, right: 9000` (the other two already pass: closing Settings drops focus while the field is still drawn, and Esc cancels).

- [ ] **Step 3: Let the drag value update while it is edited**

In `crates/fp-app/src/ui/settings/remote.rs` change `number`: drop `update_while_editing(false)` and say why in the doc comment:

```rust
/// A number over a configuration value. Returns the new value once, when a
/// drag is released or typing ends, never the steps in between (each would
/// restart a server). The value being typed is kept in `st` as it changes
/// (the drag value does not wait for Enter), so that `flush` can apply it
/// when another section is opened.
fn number(
    ui: &mut Ui,
    st: &mut RemoteState,
    key: &'static str,
    label: egui::Id,
    current: u32,
    range: std::ops::RangeInclusive<u32>,
    suffix: &str,
) -> Option<u32> {
    let value = st.numbers.entry(key).or_insert(current);
    let response = ui
        .add(egui::DragValue::new(value).range(range).suffix(suffix))
        .labelled_by(label);
    let value = *value;
    if response.dragged() || response.has_focus() {
        return None;
    }
    st.numbers.remove(key);
    (value != current).then_some(value)
}
```

(The rest of the function is unchanged: it was already returning `None` while the field has focus.)

- [ ] **Step 4: Pin the clamp**

Append to `tests/remote_settings.rs`:

```rust
/// The number is applied as Enter would apply it, so a value below the
/// range is the lowest allowed.
#[test]
fn a_typed_port_below_the_range_is_clamped_like_enter() {
    let (mut h, fake) = opened(state(1, 0));
    type_port(&mut h, "90");
    h.get_by_role_and_label(Role::Button, "MIDI").click();
    h.run_steps(3);
    assert_eq!(fake.state.load().config.remote.http.port, 1024);
}
```

- [ ] **Step 5: Run the file**

Run: `cargo test -p fp-app --test remote_settings`
Expected: all pass, including the existing `typing_a_port_applies_only_the_final_value` (one `UpdateConfig`, 9000).

- [ ] **Step 6: Gate and commit**

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q
git add crates/fp-app/src/ui/settings/remote.rs crates/fp-app/tests/remote_settings.rs
git commit -m "fix(ui): keep a number typed in Settings > Remote when another section opens (M1)"
```

---

### Task 3: M4, a cart's track without a format is analysed even after "Later"

**Files:**
- Modify: `crates/fp-app/src/services.rs` (`Services::cart_tracks`, `submit_new`)
- Test: `crates/fp-app/tests/services.rs`

**Interfaces:**
- Consumes: `outdated(&Track)` (analysed, readable, and no format or an older version), `Services::analyse_outdated` (set by `ServiceRequest::AnalyseOutdated`), `Cartwall::pages[].carts[].track`, the test helpers `wav`, `rig_with`, `Rig::run_until`.
- Produces: `fn cart_tracks(state: &AppState) -> HashSet<TrackId>` (private to `Services`).

- [ ] **Step 1: Write the failing test**

In `crates/fp-app/tests/services.rs`, before `a_result_lost_to_a_panic_is_asked_for_again`, add:

```rust
/// A cart bus that plays bit-perfect needs the format of its files, so a
/// cart's track that was analysed before formats were recorded is analysed
/// again at once, even though the operator said "Later" for the library.
#[test]
fn a_cart_track_without_a_format_is_analysed_even_when_the_operator_said_later() {
    let dir = tempfile::tempdir().unwrap();
    let files: Vec<PathBuf> = (0..6)
        .map(|n| wav(dir.path(), &format!("{n}.wav"), 1))
        .collect();
    let jingle = wav(dir.path(), "jingle.wav", 1);
    let mut r = rig_with(&files, dir, Duration::ZERO, |state| {
        let page = state.cartwall.pages.first().unwrap().id;
        fp_model::apply(
            state,
            Command::AssignCartFile {
                page,
                index: 0,
                path: jingle.clone(),
            },
        )
        .unwrap();
        for track in state.library.iter_mut() {
            track.analyzed = true;
            track.duration_secs = 1.0;
            track.format = None;
            track.analysis_version = fp_analysis::cache::ANALYSIS_VERSION;
        }
    });
    let cart_track = r.handle.model.load().cartwall.pages[0].carts[0]
        .track
        .unwrap();
    r.run_until("the cart's track has its format", |r| {
        r.handle
            .model
            .load()
            .library
            .get(cart_track)
            .is_some_and(|t| t.format.is_some())
    });
    // "Later" still holds for the playlist tracks that are not on screen.
    let waiting = r
        .handle
        .model
        .load()
        .library
        .iter()
        .filter(|t| t.id != cart_track && t.format.is_none())
        .count();
    assert!(
        waiting >= 4,
        "only the cart's track jumps the queue; {waiting} wait"
    );
}

/// A cart track that already has its format, or whose file is gone, is not
/// analysed again, and nothing loops.
#[test]
fn a_cart_track_that_has_its_format_is_left_alone() {
    let dir = tempfile::tempdir().unwrap();
    let jingle = wav(dir.path(), "jingle.wav", 1);
    let mut r = rig_with(&[], dir, Duration::ZERO, |state| {
        let page = state.cartwall.pages.first().unwrap().id;
        fp_model::apply(
            state,
            Command::AssignCartFile {
                page,
                index: 0,
                path: jingle.clone(),
            },
        )
        .unwrap();
        for track in state.library.iter_mut() {
            track.analyzed = true;
            track.duration_secs = 1.0;
            // An older version's analysis, but with its format: it waits.
            track.format = Some(fp_model::AudioFormat {
                sample_rate: 48_000,
                bits: Some(16),
                channels: 1,
            });
            track.analysis_version = 0;
        }
    });
    for _ in 0..100 {
        r.conductor.tick(r.now);
        r.services.step(r.now);
        r.now += Duration::from_millis(10);
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(r.analyses.load(Ordering::SeqCst), 0);
}

#[test]
fn a_cart_track_whose_file_is_missing_is_not_analysed() {
    let dir = tempfile::tempdir().unwrap();
    let gone = dir.path().join("gone.wav");
    let mut r = rig_with(&[], dir, Duration::ZERO, |state| {
        let page = state.cartwall.pages.first().unwrap().id;
        fp_model::apply(
            state,
            Command::AssignCartFile {
                page,
                index: 0,
                path: gone.clone(),
            },
        )
        .unwrap();
        for track in state.library.iter_mut() {
            track.analyzed = true;
            track.format = None;
            track.file_state = FileState::Missing;
        }
    });
    for _ in 0..100 {
        r.conductor.tick(r.now);
        r.services.step(r.now);
        r.now += Duration::from_millis(10);
        std::thread::sleep(Duration::from_millis(2));
    }
    assert_eq!(r.analyses.load(Ordering::SeqCst), 0);
}
```

- [ ] **Step 2: Run them to see the first fail**

Run: `cargo test -p fp-app --test services cart_track`
Expected: `a_cart_track_without_a_format_is_analysed_even_when_the_operator_said_later` FAILS after 15 s with `timed out waiting for the cart's track has its format`; the other two pass already (they pin what must not change).

- [ ] **Step 3: Implement**

In `crates/fp-app/src/services.rs`, in `impl Services`, before `tag_pass`:

```rust
    /// Tracks assigned to a cart.
    fn cart_tracks(state: &AppState) -> HashSet<TrackId> {
        state
            .cartwall
            .pages
            .iter()
            .flat_map(|page| page.carts.iter())
            .filter_map(|cart| cart.track)
            .collect()
    }
```

In `submit_new`, after `self.media.retain(|id| wanted.contains(&id));` add `let on_carts = Self::cart_tracks(state);` and replace the `stale` line (keeping the comment above it, extended):

```rust
            // Tracks an earlier version analysed are analysed again, once,
            // when the operator asks; the ones on screen are anyway (`show`).
            // A cart's track without a format is analysed at once: the cart
            // bus opens bit-perfect only for a known format, and a cart must
            // not wait for the operator's answer to play without resampling.
            let stale = outdated(track)
                && (self.analyse_outdated || (track.format.is_none() && on_carts.contains(&id)));
```

- [ ] **Step 4: Run the file**

Run: `cargo test -p fp-app --test services`
Expected: all pass (24 tests, under a second).

- [ ] **Step 5: Gate and commit**

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q
git add crates/fp-app/src/services.rs crates/fp-app/tests/services.rs
git commit -m "fix(app): analyse a cart's track without a format even when the operator chose Later (M4)"
```

---

### Task 4: M5, peaks (and covers) of tracks an earlier version analysed

**Files:**
- Modify: `crates/fp-app/src/remote.rs` (`Bridge::analysis`, `Bridge::on_demand`, `start`)
- Test: `crates/fp-app/tests/remote.rs`

**Interfaces:**
- Consumes: `fp_analysis::analyze_file(&Path, &AnalysisSettings, &Limits)`, `AnalysisCache::{load, store}`, `Config::{analysis, limits}`.
- Produces: no new public name. `RemoteControl::peaks` and `cover` now answer for a track the cache has nothing for (the handlers are unchanged: `analysed()` in `fp-remote/src/http/handlers.rs` still answers `404 not_analyzed` for a track the model does not call analysed).

- [ ] **Step 1: Write the failing tests**

In `crates/fp-app/tests/remote.rs` replace everything from `fn post(` (the old helper) to the end of the file with the text below. It generalises the request helper (`request`, `post`, `get`), moves the set-up of the existing test into `serve`, keeps `a_remote_play_puts_the_player_on_air` as it was, and adds the three new tests:

```rust
fn request(addr: SocketAddr, method: &str, path: &str) -> (u16, String) {
    let mut s = TcpStream::connect_timeout(&addr, Duration::from_secs(2)).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(30))).unwrap();
    write!(
        s,
        "{method} {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\nContent-Length: 0\r\n\r\n"
    )
    .unwrap();
    let mut out = String::new();
    s.read_to_string(&mut out).unwrap();
    let status = out.split(' ').nth(1).unwrap().parse().unwrap();
    (status, out.split_once("\r\n\r\n").unwrap().1.to_owned())
}

fn post(addr: SocketAddr, path: &str) -> (u16, String) {
    request(addr, "POST", path)
}

fn get(addr: SocketAddr, path: &str) -> (u16, String) {
    request(addr, "GET", path)
}

/// A conductor on the Offline backend with the remote API listening on a
/// free loopback port. `prepare` changes the state before it starts.
struct Served {
    conductor: Conductor,
    handle: Arc<fp_engine::conductor::ConductorHandle>,
    remote: fp_remote::RemoteHandle,
    addr: SocketAddr,
    _dir: tempfile::TempDir,
}

fn serve(
    dir: tempfile::TempDir,
    files: Vec<std::path::PathBuf>,
    prepare: impl FnOnce(&mut fp_model::AppState),
) -> Served {
    let paths = AppPaths::under(dir.path());
    let store = Store::new(paths.clone(), Default::default());
    let mut loaded = store.load("Main");
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let state = &mut loaded.state;
    state.config.outputs.backend = Some("offline".into());
    state.config.outputs.buffer_frames = 480;
    state.config.remote.http.enabled = true;
    state.config.remote.http.port = port;
    let playlist = state.playlists.first_id().unwrap();
    fp_model::apply(
        state,
        Command::InsertPaths {
            playlist,
            index: 0,
            paths: files,
        },
    )
    .unwrap();
    prepare(state);
    let limits = state.config.limits.clone();

    let backend = OfflineBackend::new();
    let _device = backend.add_device("main", 2);
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let engine = Engine::new(
        backends,
        EngineSettings::from_config(&loaded.state.config),
        file_opener(),
    );
    let (conductor, handle) = Conductor::new(loaded.state, loaded.actions, engine, Instant::now());
    let handle = Arc::new(handle);
    let remote = fp_app::remote::start(
        handle.clone(),
        MediaCache::default(),
        paths.cache_dir.join("analysis"),
        &limits,
    )
    .unwrap();

    let deadline = Instant::now() + Duration::from_secs(5);
    let addr = loop {
        if let ServerStatus::Listening(addr) = remote.status().http {
            break addr;
        }
        assert!(Instant::now() < deadline, "{:?}", remote.status());
        std::thread::sleep(Duration::from_millis(10));
    };
    Served {
        conductor,
        handle,
        remote,
        addr,
        _dir: dir,
    }
}

#[test]
fn a_remote_play_puts_the_player_on_air() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("tone.wav");
    tone(&file);
    let mut served = serve(dir, vec![file], |_| {});
    let player = served.handle.model.load().players[0].id;
    let (status, body) = post(served.addr, &format!("/api/v1/players/{}/play", player.0));
    assert_eq!(status, 202, "{body}");

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        served.conductor.tick(Instant::now());
        if served.handle.model.load().players[0].transport == Transport::Playing {
            break;
        }
        assert!(Instant::now() < deadline, "the player never started");
        std::thread::sleep(Duration::from_millis(5));
    }
    drop(served.remote);
}

/// A track an earlier version analysed has no cached analysis of this
/// version (older entries are swept), and it is not on a player, so no
/// waveform is in memory either: the peaks are worked out when asked for.
#[test]
fn the_peaks_of_an_outdated_track_are_available() {
    let dir = tempfile::tempdir().unwrap();
    let files: Vec<_> = ["a.wav", "b.wav"]
        .iter()
        .map(|n| {
            let f = dir.path().join(n);
            tone(&f);
            f
        })
        .collect();
    let served = serve(dir, files, |state| {
        for track in state.library.iter_mut() {
            track.analyzed = true;
            track.duration_secs = 3.0;
            track.format = None;
            track.analysis_version = 0;
        }
    });
    let model = served.handle.model.load();
    let playlist = model.playlists.first_id().unwrap();
    let second = model.playlists.get(playlist).unwrap().entries[1].track;
    let (status, body) = get(served.addr, &format!("/api/v1/tracks/{}/peaks", second.0));
    assert_eq!(status, 200, "{body}");
    assert!(body.contains("\"peaks\":[["), "{body}");
    // A second request answers the same.
    let (status, _) = get(served.addr, &format!("/api/v1/tracks/{}/peaks", second.0));
    assert_eq!(status, 200);
    let (status, body) = get(served.addr, &format!("/api/v1/tracks/{}/cover", second.0));
    assert_eq!(status, 404, "a track without a cover: {body}");
    drop(served.remote);
}

/// A file that cannot be decoded answers "not found" for its peaks (rule
/// 9: bad data degrades), and the request ends.
#[test]
fn the_peaks_of_an_undecodable_file_are_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let junk = dir.path().join("junk.wav");
    std::fs::write(&junk, b"this is not audio").unwrap();
    let served = serve(dir, vec![junk], |state| {
        for track in state.library.iter_mut() {
            track.analyzed = true;
            track.format = None;
        }
    });
    let model = served.handle.model.load();
    let track = model.library.iter().next().unwrap().id;
    let (status, body) = get(served.addr, &format!("/api/v1/tracks/{}/peaks", track.0));
    assert_eq!(status, 404, "{body}");
    drop(served.remote);
}

/// Several clients asking for the same outdated track at once all get the
/// peaks; the analyses are taken one at a time.
#[test]
fn two_requests_at_once_both_get_the_peaks() {
    let dir = tempfile::tempdir().unwrap();
    let files: Vec<_> = ["a.wav", "b.wav"]
        .iter()
        .map(|n| {
            let f = dir.path().join(n);
            tone(&f);
            f
        })
        .collect();
    let served = serve(dir, files, |state| {
        for track in state.library.iter_mut() {
            track.analyzed = true;
            track.duration_secs = 3.0;
            track.format = None;
            track.analysis_version = 0;
        }
    });
    let model = served.handle.model.load();
    let playlist = model.playlists.first_id().unwrap();
    let second = model.playlists.get(playlist).unwrap().entries[1].track;
    let (addr, uri) = (served.addr, format!("/api/v1/tracks/{}/peaks", second.0));
    let clients: Vec<_> = (0..2)
        .map(|_| {
            let uri = uri.clone();
            std::thread::spawn(move || get(addr, &uri).0)
        })
        .collect();
    for client in clients {
        assert_eq!(client.join().unwrap(), 200);
    }
    drop(served.remote);
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-app --test remote`
Expected: `the_peaks_of_an_outdated_track_are_available` and `two_requests_at_once_both_get_the_peaks` FAIL with `left: 404, right: 200` (`{"error":"not_found",...}`); the other two pass.

- [ ] **Step 3: Analyse on demand, one at a time**

In `crates/fp-app/src/remote.rs`: `use std::sync::{Arc, Mutex, PoisonError};`, then replace `struct Bridge` and `Bridge::analysis`:

```rust
struct Bridge {
    conductor: Arc<ConductorHandle>,
    media: MediaCache,
    cache: AnalysisCache,
    /// Held while a track is analysed for a request: one at a time, so that
    /// a client asking for many tracks cannot take every core.
    on_demand: Mutex<()>,
}

impl Bridge {
    /// The analysis of `track` (blocking). It is read from the cache; a
    /// track the cache has nothing for (one an earlier version analysed
    /// that no player shows, or an entry that was removed) is analysed now
    /// and cached, so that the next request, and the analysis the services
    /// thread would run, find it.
    fn analysis(&self, track: TrackId) -> Option<fp_analysis::Analysis> {
        let model = self.conductor.model.load_full();
        let path = &model.library.get(track)?.path;
        let settings = &model.config.analysis;
        if let Some(analysis) = self.cache.load(path, settings) {
            return Some(analysis);
        }
        let _one = self
            .on_demand
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        // A request that waited for the lock finds the other one's result.
        if let Some(analysis) = self.cache.load(path, settings) {
            return Some(analysis);
        }
        let analysis = fp_analysis::analyze_file(path, settings, &model.config.limits).ok()?;
        if let Err(error) = self.cache.store(path, settings, &analysis) {
            tracing::warn!(path = %path.display(), %error, "cannot cache the analysis");
        }
        Some(analysis)
    }
}
```

and in `start` add the field to the literal: `cache: AnalysisCache::new(analysis_dir, limits), on_demand: Mutex::new(()),`.

- [ ] **Step 4: Run the file**

Run: `cargo test -p fp-app --test remote`
Expected: 4 passed.

- [ ] **Step 5: Gate and commit**

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q
git add crates/fp-app/src/remote.rs crates/fp-app/tests/remote.rs
git commit -m "fix(remote): work out the peaks of a track an earlier version analysed when asked (M5)"
```

---

### Task 5: M6, the outdated count once per model snapshot

**Files:**
- Modify: `crates/fp-app/src/services.rs` (`OutdatedCount`), `crates/fp-app/src/ui/app.rs` (`AppUi::outdated`, three uses), `crates/fp-app/src/ui/settings.rs` (`SettingsDeps::outdated`, the Analysis section)
- Test: `crates/fp-app/tests/outdated_notice.rs`

**Interfaces:**
- Consumes: `services::outdated_tracks(&AppState) -> usize` (unchanged, still used by the tests of `services`).
- Produces: `#[derive(Default)] pub struct OutdatedCount` with `pub fn get(&mut self, state: &Arc<AppState>) -> usize` and `pub fn scans(&self) -> u32`; `SettingsDeps::outdated: usize`.

- [ ] **Step 1: Write the failing tests**

In `crates/fp-app/tests/outdated_notice.rs` add `use fp_app::ui::controller::Controller;` after the `use fp_app::services::ServiceRequest;` line, and append:

```rust
/// Counting scans the library: it is done again when the model changed, not
/// on every frame while the notice or Settings > Analysis is open.
#[test]
fn the_count_is_worked_out_again_only_when_the_model_changed() {
    use std::sync::Arc;
    let first = Arc::new(library(5, 3));
    let mut count = fp_app::services::OutdatedCount::default();
    assert_eq!(count.get(&first), 3);
    for _ in 0..100 {
        assert_eq!(count.get(&first), 3);
        assert_eq!(count.get(&Arc::clone(&first)), 3, "the same snapshot");
    }
    assert_eq!(count.scans(), 1);
    let second = Arc::new(library(5, 1));
    assert_eq!(count.get(&second), 1);
    assert_eq!(count.get(&second), 1);
    assert_eq!(count.scans(), 2);
    // An equal library in a new snapshot is a change as far as the cache
    // can tell: it never compares libraries.
    let third = Arc::new(library(5, 1));
    assert_eq!(count.get(&third), 1);
    assert_eq!(count.scans(), 3);
}

#[test]
fn the_notice_follows_the_library_while_it_is_open() {
    let (mut h, fake, _rx) = {
        let (tx, rx) = crossbeam_channel::bounded(4);
        let (h, fake) = harness_from(library(5, 3), move |ui| ui.with_services(tx));
        (h, fake, rx)
    };
    h.run_steps(2);
    assert!(
        h.query_by_label_contains("3 tracks were analysed by an earlier version")
            .is_some()
    );
    // One of them is not found any more: it is not counted (and cannot be
    // analysed again).
    let outdated = fake
        .state
        .load()
        .library
        .iter()
        .find(|t| t.analysis_version == 0)
        .unwrap()
        .id;
    fake.send(fp_model::Command::SetFileState {
        track: outdated,
        state: fp_model::FileState::Missing,
    });
    h.run_steps(4);
    assert!(
        h.query_by_label_contains("2 tracks were analysed by an earlier version")
            .is_some()
    );
}
```

- [ ] **Step 2: Run them to see the first fail**

Run: `cargo test -p fp-app --test outdated_notice`
Expected: does not compile: `cannot find OutdatedCount in services`. (The second test is a guard: it passes as soon as the file compiles, and keeps passing once the cache exists, which is its point: the cache must not show a stale number.)

- [ ] **Step 3: Implement**

In `crates/fp-app/src/services.rs`, after `outdated_tracks`:

```rust
/// `outdated_tracks`, remembered for the model snapshot it was worked out
/// on. Counting scans the whole library, which the interface would do on
/// every frame while the start-up notice or Settings > Analysis is open;
/// with this it scans once per new snapshot. It never compares libraries:
/// a new snapshot is a new scan.
#[derive(Default)]
pub struct OutdatedCount {
    /// Kept (not just its address), so that the address cannot be reused
    /// by another snapshot while it is remembered.
    snapshot: Option<Arc<AppState>>,
    count: usize,
    scans: u32,
}

impl OutdatedCount {
    /// How many tracks an earlier version analysed, in `state`.
    pub fn get(&mut self, state: &Arc<AppState>) -> usize {
        if !self
            .snapshot
            .as_ref()
            .is_some_and(|known| Arc::ptr_eq(known, state))
        {
            self.count = outdated_tracks(state);
            self.snapshot = Some(Arc::clone(state));
            self.scans += 1;
        }
        self.count
    }

    /// How many times the library was scanned.
    pub fn scans(&self) -> u32 {
        self.scans
    }
}
```

In `crates/fp-app/src/ui/app.rs`:
- `AppUi`: after the field `outdated_checked: bool,` add `/// How many tracks an earlier version analysed, per model snapshot.` and `outdated: crate::services::OutdatedCount,`; in the constructor after `outdated_checked: false,` add `outdated: crate::services::OutdatedCount::default(),`.
- The start-up check becomes `self.view.outdated_open = self.services.is_some() && self.outdated.get(&state) > 0;`.
- In the notice branch: `let count = self.outdated.get(&state);` (was `crate::services::outdated_tracks(&state)`).
- In the `SettingsDeps { ... }` literal add `outdated: self.outdated.get(&state),` after `restart_pending`.

In `crates/fp-app/src/ui/settings.rs`: add to `SettingsDeps` `/// How many tracks an earlier version analysed.` and `pub outdated: usize,`; in `fn analysis` replace `let outdated = crate::services::outdated_tracks(scene.state);` with `let outdated = deps.outdated;`.

- [ ] **Step 4: Run the file**

Run: `cargo test -p fp-app --test outdated_notice`
Expected: 6 passed.

- [ ] **Step 5: Gate and commit**

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q
git add crates/fp-app
git commit -m "perf(ui): count the outdated tracks once per model snapshot, not every frame (M6)"
```

---

### Task 6: Missing-file M2, the reason tooltip also on the title cell

**Files:**
- Modify: `crates/fp-app/src/ui/table.rs` (the `TableColumn::Title` arm of `track_table`)
- Test: `crates/fp-app/tests/main_screen.rs`

**Interfaces:**
- Consumes: `Scene::file_tip(TrackId) -> Option<String>`, `RowStatus::Unavailable`.
- Produces: nothing new.

- [ ] **Step 1: Write the failing tests**

In `crates/fp-app/tests/main_screen.rs`, before `a_missing_file_and_an_unreadable_one_have_their_own_icons`, add:

```rust
/// The reason shows over the title too, not only over the icon in the first
/// column (that column can be hidden, and the title is where the eye is).
#[test]
fn hovering_the_title_of_an_unavailable_row_says_why() {
    let mut s = state(1, 3);
    let playlist = s.playlists.first_id().unwrap();
    let track = s.playlists.get(playlist).unwrap().entries[1].track;
    fp_model::apply(
        &mut s,
        Command::SetFileState {
            track,
            state: fp_model::FileState::Missing,
        },
    )
    .unwrap();
    let (mut h, _fake) = harness(s);
    h.get_by_label("Song 2").hover();
    h.run_steps(40);
    assert!(
        h.query_by_label_contains("File not found: /music/Song 2.mp3")
            .is_some(),
        "the tooltip gives the reason and the path"
    );
}

#[test]
fn the_title_says_why_even_without_the_number_column() {
    let mut s = state(1, 3);
    s.config.ui.table_columns = vec![
        fp_model::TableColumn::Title,
        fp_model::TableColumn::Duration,
    ];
    let playlist = s.playlists.first_id().unwrap();
    let track = s.playlists.get(playlist).unwrap().entries[1].track;
    fp_model::apply(
        &mut s,
        Command::SetFileState {
            track,
            state: fp_model::FileState::Unreadable,
        },
    )
    .unwrap();
    let (mut h, _fake) = harness(s);
    assert!(
        h.query_by_label_contains(egui_phosphor::regular::WARNING)
            .is_none()
    );
    h.get_by_label("Song 2").hover();
    h.run_steps(40);
    assert!(
        h.query_by_label_contains("Cannot read the file: /music/Song 2.mp3")
            .is_some()
    );
}

#[test]
fn hovering_the_title_of_a_playable_row_gives_no_file_reason() {
    let (mut h, _fake) = harness(state(1, 3));
    h.get_by_label("Song 2").hover();
    h.run_steps(40);
    assert!(h.query_by_label_contains("File not found").is_none());
    assert!(h.query_by_label_contains("Cannot read the file").is_none());
}
```

- [ ] **Step 2: Run them to see the first two fail**

Run: `cargo test -p fp-app --test main_screen title`
Expected: `hovering_the_title_of_an_unavailable_row_says_why` and `the_title_says_why_even_without_the_number_column` FAIL; the playable-row test passes.

- [ ] **Step 3: Attach the tooltip to the title label**

In `track_table`, in the `TableColumn::Title` arm, the last `ui.add(egui::Label::new(RichText::new(&track.title)...))` becomes:

```rust
                                        let title = ui.add(
                                            egui::Label::new(
                                                RichText::new(&track.title)
                                                    .font(row_font.clone())
                                                    .color(text),
                                            )
                                            .selectable(false)
                                            .truncate(),
                                        );
                                        // The same reason as the icon's, where the eye is
                                        // (and the `#` column may be hidden).
                                        if status == RowStatus::Unavailable
                                            && let Some(tip) = scene.file_tip(entry.track)
                                        {
                                            title.on_hover_text(tip);
                                        }
```

- [ ] **Step 4: Run the file**

Run: `cargo test -p fp-app --test main_screen`
Expected: 49 passed.

- [ ] **Step 5: Gate and commit**

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q
git add crates/fp-app/src/ui/table.rs crates/fp-app/tests/main_screen.rs
git commit -m "fix(ui): show the missing-file reason on the title cell too"
```

---

### Task 7: `thread-priority`: licence check and `fp_decode::priority`

**Files:**
- Modify: root `Cargo.toml` (`[workspace.dependencies]`), `crates/fp-decode/Cargo.toml`, `crates/fp-decode/src/lib.rs`, `Cargo.lock` (regenerated by cargo)
- Create: `crates/fp-decode/src/priority.rs` (module and its unit tests)

**Interfaces:**
- Consumes: `thread_priority::{ThreadPriority, ThreadPriorityValue, set_current_thread_priority}` (version 3.1.1: on Linux the normal scheduling policies are tuned through the nice value, `Crossplatform(60)` maps to nice −5, `Min` to nice 19; on Windows 60 is "above normal" and `Min` is "lowest").
- Produces: `fp_decode::priority::{enum Priority { AboveNormal, Low }, fn set_current(priority: Priority, role: &str) -> bool}`. It returns whether the system accepted; a refusal is logged once per run (`warn`, "the system refused a thread priority change; threads keep the normal priority") and ignored.

- [ ] **Step 1: Check the licence and the advisories before using it**

In the root `Cargo.toml`, under `[workspace.dependencies]` after the `tracing` line, add `thread-priority = "3.1.1"`. In `crates/fp-decode/Cargo.toml` add to `[dependencies]` (alphabetical, between `symphonia` and `wavicle`) `thread-priority.workspace = true` and `tracing.workspace = true`. Then:

```bash
export PATH=$HOME/.cargo/bin:$PATH
cargo fetch && cargo deny check 2>&1 | tail -1
```

Expected: `advisories ok, bans ok, licenses ok, sources ok` (the crate is MIT, which `deny.toml` allows; it brings `bitflags`, `cfg-if`, `libc`, `log`, `rustversion` and `windows`, all already allowed; `multiple-versions` only warns, as before). If the check fails, stop: the dependency is not acceptable and the maintainer decides. Release packaging builds `THIRD-PARTY.html` from the lock file, so the crate's MIT notice is picked up there without a change.

- [ ] **Step 2: Write the module with its failing tests**

Create `crates/fp-decode/src/priority.rs`:

```rust
//! Thread priority (main spec §2.2): the decoder threads run above normal
//! priority and the analysis pool below it. Neither is real time: only the
//! device callback is, and a decoder that took more than its share could
//! starve the interface.
//!
//! The system may refuse the change (on Linux, raising a priority needs
//! `RLIMIT_NICE` or `CAP_SYS_NICE`). The thread then keeps the normal
//! priority, and the first refusal of the run is logged; later ones are not,
//! since they would repeat for every thread.

use std::sync::atomic::{AtomicBool, Ordering};

use thread_priority::{ThreadPriority, ThreadPriorityValue, set_current_thread_priority};

/// Above normal on the crate's 0–99 scale (normal is about 50): nice −5 on
/// Linux, "above normal" on Windows, and a step above the default on macOS.
const ABOVE_NORMAL: u8 = 60;

/// What a thread asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    /// Above normal, not real time: decoder threads.
    AboveNormal,
    /// The lowest, short of idle: background analysis.
    Low,
}

impl Priority {
    fn level(self) -> Option<ThreadPriority> {
        match self {
            Priority::Low => Some(ThreadPriority::Min),
            Priority::AboveNormal => ThreadPriorityValue::try_from(ABOVE_NORMAL)
                .ok()
                .map(ThreadPriority::Crossplatform),
        }
    }
}

/// Set once the first refusal was logged.
static REFUSED: AtomicBool = AtomicBool::new(false);

/// True the first time it is called with `flag`, false afterwards.
fn first(flag: &AtomicBool) -> bool {
    !flag.swap(true, Ordering::Relaxed)
}

/// Gives the calling thread `priority`; `role` names it in the log. Returns
/// whether the system accepted. A refusal is logged once per run and
/// otherwise ignored: the thread simply keeps the normal priority.
pub fn set_current(priority: Priority, role: &str) -> bool {
    let Some(level) = priority.level() else {
        return false;
    };
    match set_current_thread_priority(level) {
        Ok(()) => true,
        Err(error) => {
            if first(&REFUSED) {
                tracing::warn!(
                    role,
                    ?priority,
                    %error,
                    "the system refused a thread priority change; threads keep the normal priority"
                );
            }
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_first_refusal_is_reported() {
        let flag = AtomicBool::new(false);
        assert!(first(&flag));
        assert!(!first(&flag));
        assert!(!first(&flag));
    }

    /// The nice value of the calling thread (Linux).
    #[cfg(target_os = "linux")]
    fn nice() -> i32 {
        let stat = std::fs::read_to_string("/proc/thread-self/stat").unwrap();
        // The command name is in parentheses and may hold spaces: count
        // the fields after the last parenthesis (field 19 is the nice).
        let after = stat.rsplit_once(')').unwrap().1;
        after.split_whitespace().nth(16).unwrap().parse().unwrap()
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn low_priority_lowers_the_thread_and_only_that_thread() {
        let before = nice();
        let inside = std::thread::spawn(|| {
            assert!(set_current(Priority::Low, "test"));
            nice()
        })
        .join()
        .unwrap();
        assert_eq!(inside, 19);
        assert_eq!(nice(), before, "the thread that spawned it is untouched");
    }

    /// Whether it is accepted depends on the privileges of whoever runs the
    /// tests; either way the thread ends at the priority the answer says.
    #[cfg(target_os = "linux")]
    #[test]
    fn above_normal_is_accepted_or_leaves_the_thread_alone() {
        let (accepted, inside) = std::thread::spawn(|| {
            let before = nice();
            (set_current(Priority::AboveNormal, "test"), (before, nice()))
        })
        .join()
        .unwrap();
        let (before, after) = inside;
        if accepted {
            assert!(after < before, "{before} -> {after}");
        } else {
            assert_eq!(after, before);
        }
    }
}
```

and in `crates/fp-decode/src/lib.rs` add `pub mod priority;` between `mod opus;` and `mod symph;`.

- [ ] **Step 3: Run the tests**

Run: `cargo test -p fp-decode --lib priority`
Expected: 3 passed on Linux (on other systems only `only_the_first_refusal_is_reported` exists). `above_normal_is_accepted_or_leaves_the_thread_alone` accepts either outcome: an ordinary Linux user gets the refusal, root gets nice −5.

(There is no earlier red step: the module and its tests arrive together because the tests cannot compile without it. Task 8 carries the tests that fail before the threads call it.)

- [ ] **Step 4: Gate and commit**

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q
git add Cargo.toml Cargo.lock crates/fp-decode
git commit -m "feat(decode): a thread priority module over thread-priority (O21)"
```

---

### Task 8: Decode workers above normal, the analysis pool low

**Files:**
- Modify: `crates/fp-engine/src/worker.rs` (`run`), `crates/fp-engine/Cargo.toml` (dev-dependency `tracing-subscriber`), `crates/fp-analysis/src/analyzer.rs` (the pool's thread closure)
- Test: create `crates/fp-engine/tests/decoder_priority.rs`; modify `crates/fp-analysis/tests/analyzer.rs`

**Interfaces:**
- Consumes: `fp_decode::priority::{Priority, set_current}` (Task 7), `PlayerWorker::spawn(name, opener, bus_rate, ready_frames, failures)`, `Analyzer::with_analyze_fn`.
- Produces: nothing new.

- [ ] **Step 1: Write the failing tests**

Add to `crates/fp-engine/Cargo.toml` under `[dev-dependencies]`: `tracing-subscriber.workspace = true`. Create `crates/fp-engine/tests/decoder_priority.rs` (its own test binary: it installs the global log subscriber and reads the threads from `/proc`):

```rust
#![allow(clippy::unwrap_used)]
//! Decoder threads ask for above-normal priority, and a system that refuses
//! is logged once, not once per thread (main spec §2.2). Its own test
//! binary: it installs the global log subscriber, and it reads the threads
//! from `/proc`.
#![cfg(target_os = "linux")]

use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use fp_engine::worker::{PlayerWorker, file_opener};

#[derive(Clone, Default)]
struct Lines(Arc<Mutex<Vec<u8>>>);

impl Write for Lines {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// The nice value of the thread of this process called `name`.
fn nice_of(name: &str) -> Option<i32> {
    for task in std::fs::read_dir("/proc/self/task").unwrap().flatten() {
        let comm = std::fs::read_to_string(task.path().join("comm")).unwrap_or_default();
        if comm.trim_end() != name {
            continue;
        }
        let stat = std::fs::read_to_string(task.path().join("stat")).ok()?;
        let after = stat.rsplit_once(')')?.1;
        return after.split_whitespace().nth(16)?.parse().ok();
    }
    None
}

fn spawn(name: &str) -> PlayerWorker {
    let (failures, _rx) = crossbeam_channel::unbounded();
    PlayerWorker::spawn(name, file_opener(), 48_000, 4_800, failures).unwrap()
}

#[test]
fn decoder_threads_ask_for_a_higher_priority_and_a_refusal_is_logged_once() {
    let lines = Lines::default();
    let writer = lines.clone();
    tracing_subscriber::fmt()
        .with_writer(move || writer.clone())
        .with_ansi(false)
        .init();
    let a = spawn("fp-decode-a");
    let b = spawn("fp-decode-b");
    let refused = |lines: &Lines| {
        String::from_utf8(lines.0.lock().unwrap().clone())
            .unwrap()
            .matches("refused a thread priority change")
            .count()
    };
    let raised = |name: &str| nice_of(name).is_some_and(|n| n < 0);
    // Either the system accepted both (they are above normal), or it refused
    // and said so. Neither happening means the threads never asked.
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let said = refused(&lines) > 0;
        if (said || raised("fp-decode-a")) && (said || raised("fp-decode-b")) {
            break;
        }
        assert!(Instant::now() < deadline, "the decoder threads never asked");
        std::thread::sleep(Duration::from_millis(5));
    }
    // Dropping a worker joins its thread: both have asked by now.
    let both_raised = raised("fp-decode-a") && raised("fp-decode-b");
    drop(a);
    drop(b);
    let said = refused(&lines);
    assert!(said <= 1, "logged {said} times");
    assert!(said == 1 || both_raised);
}
```

Append to `crates/fp-analysis/tests/analyzer.rs`:

```rust
/// Analysis is background work (main spec §2.2): its threads run at the
/// lowest priority, so decoding and the interface keep the processor.
#[cfg(target_os = "linux")]
#[test]
fn the_pool_runs_at_low_priority() {
    use std::sync::Mutex;
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let probe: AnalyzeFn = Arc::new(move |path, settings, limits, _cancelled| {
        // The nice value of the thread running this job (field 19).
        let stat = std::fs::read_to_string("/proc/thread-self/stat").unwrap();
        let nice: i32 = stat
            .rsplit_once(')')
            .unwrap()
            .1
            .split_whitespace()
            .nth(16)
            .unwrap()
            .parse()
            .unwrap();
        record.lock().unwrap().push(nice);
        analyze_file(path, settings, limits)
    });
    let dir = tempfile::tempdir().unwrap();
    let analyzer = Analyzer::with_analyze_fn(
        2,
        AnalysisSettings::default(),
        Limits::default(),
        None,
        probe,
    )
    .unwrap();
    for n in 0..4 {
        analyzer.submit(TrackId(n), wav(dir.path(), &format!("{n}.wav")));
    }
    for _ in 0..4 {
        analyzer.results().recv_timeout(WAIT).unwrap();
    }
    assert_eq!(*seen.lock().unwrap(), vec![19; 4]);
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-engine --test decoder_priority` then `cargo test -p fp-analysis --test analyzer the_pool_runs_at_low_priority`
Expected: the first FAILS after 10 s with `the decoder threads never asked`; the second FAILS with `left: [0, 0, 0, 0]` against `[19, 19, 19, 19]`.

- [ ] **Step 3: Make the threads ask**

`crates/fp-engine/src/worker.rs`: add `use fp_decode::priority::Priority;` before `use crate::decode::FileDecoder;`, and make the first statements of `fn run(...)`:

```rust
    // Above normal, never real time (main spec §2.2): the ring buffers must
    // not run dry because the interface or the analysis pool took the
    // processor. A system that refuses leaves the thread at normal priority.
    fp_decode::priority::set_current(Priority::AboveNormal, "decoder");
```

(before `let mut jobs: Vec<Job> = Vec::new();`). `crates/fp-analysis/src/analyzer.rs`: add `use fp_decode::priority::Priority;` after the `fp_model` import, and at the top of the pool thread's closure (before the cache sweep):

```rust
                        // Background work (main spec §2.2): it must never
                        // take the processor from decoding or the interface.
                        fp_decode::priority::set_current(Priority::Low, "analysis");
```

- [ ] **Step 4: Run them**

Run: `cargo test -p fp-engine --test decoder_priority` and `cargo test -p fp-analysis --test analyzer`
Expected: PASS (13 tests in the second file). On a machine where raising is refused the first passes through the log branch; the log line carries `role="decoder"`.

- [ ] **Step 5: Gate and commit**

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q
git add Cargo.lock crates/fp-engine crates/fp-analysis
git commit -m "feat(engine): decoder threads above normal priority, the analysis pool low (O21)"
```

---

### Task 9: The long-file check (verification; no code unless it finds a defect)

**Files:**
- Create (git-ignored, inside `target/`): `target/long-check/` (the generated file, a scratch `FAUSTE_HOME`, a screenshot)
- Ledger: `.superpowers/sdd/feedback2-plan9/progress.md` (git-ignored)

**Interfaces:** none. This task changes no file that is committed, unless a defect is found (Step 8).

The check plays a four-hour file in the real application, under a virtual X server, and seeks to 3:59:00 through the remote API. Nothing may sound: the fader is at zero and the sound server is made unreachable (the application then falls back to ALSA or the Null system, and the timeline keeps running on a virtual clock if no device opens). Follow the CLAUDE.md recipe "Screenshots for a visual check"; `xvfb`, `xdotool`, ImageMagick's `import`, `ffmpeg`, `curl` and `jq` must be installed.

- [ ] **Step 1: Generate the file and a demo state**

```bash
export PATH=$HOME/.cargo/bin:$PATH
L=$PWD/target/long-check
mkdir -p $L/music $L/run && chmod 700 $L/run
# Four hours of pink noise, mono, 44.1 kHz, 16 bit: 1.27 GB.
ffmpeg -loglevel error -y -f lavfi -i "anoisesrc=color=pink:duration=14400:sample_rate=44100:amplitude=0.3" \
  -ac 1 -c:a pcm_s16le $L/music/long-4h.wav
cargo build --release -p fp-app
FAUSTE_HOME=$L/home cargo run --release -q -p fp-app --example demo_session -- $L/music
jq '.config.remote.http.enabled=true | .config.ui.language="en-US" | .config.outputs.backend="pulseaudio"' \
  $L/home/config/config.json > $L/c.tmp && mv $L/c.tmp $L/home/config/config.json
```

Expected: `$L/home` holds `config/`, `data/` with four players and three playlists of the long file.

- [ ] **Step 2: Start Xvfb and the application, without a reachable sound server**

```bash
Xvfb :77 -screen 0 1920x1080x24 -nolisten tcp > $L/xvfb.log 2>&1 &
env -u WAYLAND_DISPLAY DISPLAY=:77 XDG_RUNTIME_DIR=$L/run PULSE_SERVER=unix:$L/run/none \
  FAUSTE_HOME=$L/home target/release/fauste-player > $L/app.log 2>&1 &
sleep 6
B=http://127.0.0.1:7380/api/v1; H='Content-Type: application/json'
P=$(curl -s $B/players | jq '.[0].id')
curl -s $B/players | jq -c '.[0] | [.id, .transport, (.current // .next).track.analyzed]'
```

Expected: the first player, `"stopped"` or `"paused"`; the log under `$L/home/logs/` shows `audio systems backend=alsa` (or `null`) and, on an ordinary Linux user, one `fp_decode::priority` warning with `role="decoder"` (the refused raise of Task 8).

- [ ] **Step 3: Wait for the analysis, then play silently**

```bash
until [ "$(curl -s $B/players | jq '.[0] | (.current // .next).track.analyzed')" = true ]; do sleep 5; done
curl -s $B/players | jq -c '.[0] | (.current // .next).track | [.duration_secs, .format, .markers.cue_out]'
curl -s -X PUT -H "$H" -d '{"fader":0.0}' $B/players/$P/volume -w "volume %{http_code}\n"
curl -s -X POST $B/players/$P/play -w "play %{http_code}\n"; sleep 2
curl -s $B/players/$P | jq -c '[.transport, .elapsed_secs, .remaining_secs]'
```

Expected: `duration_secs` 14400.0; `volume 202`, `play 202`; two seconds later `["playing", ~2.0, ~14398]`.

- [ ] **Step 4: Seek to 3:59:00 and watch it play**

```bash
time curl -s -X POST -H "$H" -d '{"secs": 14340.0}' $B/players/$P/seek -w "seek %{http_code}\n"
for i in 1 2 3; do sleep 1; curl -s $B/players/$P | jq -c '[.transport, .elapsed_secs, .remaining_secs]'; done
W=$(DISPLAY=:77 xwininfo -name "Fauste Player" | awk '/Window id/{print $4}')
DISPLAY=:77 import -window $W $L/seek.png
```

Expected: `seek 202` within milliseconds; the three readings are about 14341, 14342 and 14343 s elapsed (within 0.1 s of 14340 + the seconds waited) with 59, 58 and 57 s remaining; the screenshot (read it) shows the time row at `3:59:xx / 4:00:00` and the countdown near `-0:59`, with no error banner.

- [ ] **Step 5: Seek back, then to the last second, and let it end**

```bash
curl -s -X POST -H "$H" -d '{"secs": 100.0}' $B/players/$P/seek -w "seek-back %{http_code}\n"; sleep 2
curl -s $B/players/$P | jq -c '[.transport, .elapsed_secs]'
curl -s -X POST -H "$H" -d '{"secs": 14399.5}' $B/players/$P/seek -w "seek-end %{http_code}\n"; sleep 3
curl -s $B/players/$P | jq -c '[.transport, .elapsed_secs]'
grep -E "ERROR|panic" $L/home/logs/*.log | head
```

Expected: after the first seek about 102 s elapsed and still `"playing"`; after the second `["stopped", null]` (the entry ended and, in single-entry playlists, the player stopped); no `ERROR` or panic lines.

- [ ] **Step 6: Optional, a compressed file**

Repeat Steps 1 to 5 with a four-hour MP3 (`-ac 2 -c:a libmp3lame -b:a 128k`, 230 MB, about a minute to encode) in a second music folder and a second `FAUSTE_HOME`. Symphonia may log `mpa: invalid main_data_begin` after a seek into an MP3 (the bit reservoir); that is expected.

- [ ] **Step 7: Clean up, record the result**

```bash
pkill -x fauste-player; pkill -x Xvfb
rm -rf $PWD/target/long-check
```

Write the measurements into `.superpowers/sdd/feedback2-plan9/progress.md` as `Evidence:` lines: the file (format, size, duration), the analysis time, the seek's HTTP code and latency, the elapsed readings one, two and three seconds later, the end of the file, and the log findings. Reference result from writing the plan (Linux, release build, Xvfb, 4 h pink-noise WAV, mono 44.1 kHz): analysis finished within the start-up wait; `seek 202` in 6 ms; elapsed 14341.99 s two seconds after the seek; seek to 100 s and to 14399.5 s accepted, the player stopped at the end; no errors in the log, one `fp_decode::priority` refusal warning (an ordinary user cannot raise a priority). The 4 h 128 kbit/s stereo MP3 gave 14340.98, 14341.99 and 14343.00 s elapsed at one, two and three seconds, with two harmless `mpa: invalid main_data_begin` warnings after the seek. No defect was found.

- [ ] **Step 8: If anything differs, it is a defect: fix it test-first, here**

A seek that errors, lands more than 0.2 s from the target, stalls or jumps, an `ERROR` or panic in the log, or a screenshot with wrong times is a defect of the application, not of the check. In this task: write a failing test at the lowest level that shows it (a decoder or seek test in `fp-decode`, a conductor test in `fp-engine`, or a `fp-analysis/tests/long_files.rs` case for the file kind), watch it fail, fix the code, commit it as `fix(<scope>): ...`, and run Steps 1 to 5 again. Record the defect, the test and the fix in the ledger as a `Ruling:` line. If no defect is found, no commit comes out of this task.

---

### Task 10: Documentation

**Files:**
- Modify: `docs/user/players.md`, `docs/user/settings.md`, `docs/user/playlists.md`, `docs/user/bit-perfect.md`
- Modify: `docs/technical/ui.md`, `docs/technical/analysis.md`, `docs/technical/remote-api.md`, `docs/technical/threading-and-realtime.md`
- Modify: `docs/superpowers/specs/2026-09-25-fauste-player-design.md` (§2.2 and §12), `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md` (status and "As built" under §10), `docs/superpowers/plans/2026-10-01-operator-feedback-2-roadmap.md` (row 9 `done`), `README.md`
- `CLAUDE.md`: no change (no command or layout line changes; `fp_decode::priority` sits in an existing crate).

**Interfaces:** none (prose). Every claim must match the code as built.

- [ ] **Step 1: The peak-hold sentence (verified against the code)**

`MeterBallistics::settings()` in `crates/fp-model/src/config.rs` sets `peak_hold: true` for `DigitalPeak`, `Custom` and `K20`/`K14`/`K12`, and `false` for `EbuPpm`, `DinPpm` and `Vu`; `tests/meter.rs::peak_hold_applies_only_to_meters_that_have_one` pins it, and `docs/user/settings.md` already says so. In `docs/user/players.md` replace the bullet

```
  - The highest level stays lit for a moment as a line (the peak hold).
```

with

```
  - Digital peak, K-System and custom meters keep the highest level lit for a
    moment as a line (the peak hold; its length is **Peak hold** in
    [Settings → Meters](settings.md#meters), and 0 turns it off). EBU PPM,
    DIN PPM and VU meters have no hold.
```

- [ ] **Step 2: The other user-guide lines**

`docs/user/settings.md`, Remote section: replace "Text fields apply when you leave them. An invalid value is corrected, and\nthe field shows what was kept." with:

```
Text fields and numbers apply when you leave them, which includes opening
another section or closing Settings; Esc cancels what you were typing. An
invalid value is corrected, and the field shows what was kept.
```

`docs/user/settings.md`, Analysis section: replace "The tracks on the players are brought up to date anyway, as they are\nshown." with:

```
The tracks on the players are brought up to date anyway, as they are
shown, and so are the tracks of carts that have no recorded format (a cart
plays bit-perfect only when its format is known).
```

`docs/user/playlists.md`: in the status-icon row replace "hover it for the reason and the path." with "hover the icon or the title for the reason and the path.".

`docs/user/bit-perfect.md`: replace "or as soon\n  as a player shows them;" with "or as soon\n  as a player shows them or a cart holds them;".

- [ ] **Step 3: The technical docs**

`docs/technical/ui.md`: after the "Columns." paragraph (it ends "the text of the plain columns is `view::cell_text`.") add:

```
A row whose file is missing or unreadable gives the reason and the path
(`Scene::file_tip`) on its icon in the `#` column and on its title, so the
reason shows without the `#` column too.
```

and at the end of "## Settings window" (after "**Restart now** stays.") add:

```
Settings → Remote keeps what is being typed (`RemoteState::drafts` for text,
`RemoteState::numbers` for the drag values, which do not wait for Enter), so
opening another section applies it through `remote::flush`. The count of
outdated tracks (Settings → Analysis, and the start-up notice) comes from
`AppUi`'s `services::OutdatedCount`: the library is scanned once per model
snapshot, not on every frame (`SettingsDeps::outdated`).
```

`docs/technical/analysis.md`, "How the app uses it": in the bullet about outdated tracks replace "Until then they keep that analysis, which still\n  works; re-analysing a library costs the processor for a while; or" with:

```
Until then they keep that analysis, which still
  works; re-analysing a library costs the processor for a while. The
  exception is a track on a cart that has no format recorded: the cart bus
  opens bit-perfect only for a known format, so it is analysed at once
  (`Services::cart_tracks`); or
```

`docs/technical/remote-api.md`: replace the paragraph that starts "The cover and peaks come from the interface's media cache" with:

```
The cover and peaks come from the interface's media cache, or else from the
analysis cache on disk. A track not analysed yet answers `404 not_analyzed`
and is never analysed for a request. A track the model calls analysed but
whose cache has no entry of the current analysis version (one an earlier
version analysed that no player shows) is analysed when asked, once at a
time, and the result is cached; the model keeps what it had until the
operator asks for the new analysis. An analysed track without a cover, or
whose file cannot be decoded, answers `404 not_found`.
```

and in the `fp-app::remote::Bridge` bullet replace "Covers and peaks are read in\n  `spawn_blocking`." with:

```
Covers and peaks are read in
  `spawn_blocking`; a cache miss analyses the file there, one at a time
  (`Bridge::on_demand`). `spawn_with` and `Timing` shorten the poll and bind
  retry intervals for tests.
```

`docs/technical/threading-and-realtime.md`: in the thread table replace the decode-worker row with `| Decode worker, one per player, and one for the cartwall | `fp-player-<id>`, `fp-cartwall` | above normal, not real time (`fp_decode::priority`) | producer halves of the player's sources; decoders and resamplers | file I/O |` and the analysis-pool row's priority cell `normal` with `low (`fp_decode::priority`)`; then add this section before "## Rules for the real-time thread":

```
## Thread priority

Main spec §2.2: only the device callback is real time. The two other classes
of thread that do heavy work are tuned with the `thread-priority` crate,
through `fp_decode::priority::set_current(Priority, role)` called by the thread
itself as its first action:

| Class | Request | Linux | Windows | macOS |
|---|---|---|---|---|
| Decode workers (`PlayerWorker`) | `Priority::AboveNormal`: the crate's cross-platform value 60 | nice −5 | above normal | a step above the default |
| Analysis pool (`Analyzer`) | `Priority::Low`: the crate's minimum | nice 19 | lowest | lowest |

- **Neither is real time.** A decoder at a real-time class could starve the
  interface and the conductor, and it has a whole ring buffer of slack; the
  device callback is the one thread with a deadline.
- **Lowering always works; raising may not.** On Linux an ordinary user may not
  lower a nice value (`RLIMIT_NICE` or `CAP_SYS_NICE` is needed), so decoders
  often stay at normal priority there, and the analysis pool, whose request is
  always allowed, still yields to them.
- **A refusal is logged once and ignored.** `set_current` returns `false` and
  the thread keeps running at the normal priority. The first refusal of the run
  is a `warn` line (`the system refused a thread priority change`, with the
  role, the request and the operating system's error); later ones are silent,
  since every thread would repeat it.
- The conductor, the services thread, the remote thread and the helpers stay at
  normal priority. The services thread's saves are short and bursty, and the
  spec's "persistence writes on the background pool" is served by it running
  beside the pool, not inside it.
- Tests: `fp-decode` (`priority.rs`) reads the thread's nice value from
  `/proc` on Linux; `fp-engine/tests/decoder_priority.rs` checks that decode
  workers ask and that two refusals are one log line; `fp-analysis`
  (`the_pool_runs_at_low_priority`) checks nice 19 inside a job.
```

- [ ] **Step 4: The main spec, README, roadmap**

Main spec `docs/superpowers/specs/2026-09-25-fauste-player-design.md`: in §2.2 item 2 replace "Elevated (non-RT) priority." with "Above-normal (non-RT) priority, set by the thread itself with `thread-priority`."; replace item 4 "4. **Background worker pool** (low priority, 2 threads). Analysis jobs and persistence writes." with:

```
4. **Background worker pool** (low priority, 2 threads). Analysis jobs. Persistence writes run on the services thread at normal priority.

A priority change the system refuses is logged once and ignored: the thread keeps running at normal priority.
```

and in §12 append `thread-priority` after `audio_thread_priority` in the dependency list.

`README.md` ("In short"): replace

```
- per-player worker threads decode and resample into lock-free rings;
- analysis and saving run in the background;
```

with

```
- per-player worker threads decode and resample into lock-free rings, above
  normal priority but never real time;
- analysis runs on a low-priority pool, and saving on its own thread;
```

Roadmap: in `docs/superpowers/plans/2026-10-01-operator-feedback-2-roadmap.md` change the row `| 9 | Audit follow-ups | O21 | \`fix/audit-follow-ups\` | outline |` to `done`.

- [ ] **Step 5: The feedback 2 spec: status and "As built" under §10**

In `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md` change "Plans 1 to 8 are built" to "Plans 1 to 9 are built", and at the end of §10 (after the "Long-file check" list, before "## 11.") add (correct the Evidence bullet if Task 9 measured something else):

```
- **As built.**
  - M1: `number()` in `ui/settings/remote.rs` no longer calls `update_while_editing(false)`, so the typed value reaches `RemoteState::numbers` and `remote::flush` applies it when another section opens (clamped to the range, as Enter would); Esc cancels.
  - M4: `Services::cart_tracks`; a cart's track that is analysed, readable and has no format is analysed at once even after "Later". A cart track with a format but an older analysis version still waits.
  - M5: `Bridge::analysis` analyses a track the cache has no current entry for, on the request's blocking thread, one at a time (`Bridge::on_demand`), and caches it. A track that is not analysed is still `404 not_analyzed`; an undecodable file is `404 not_found`.
  - M6: `services::OutdatedCount` counts once per model snapshot; Settings gets it through `SettingsDeps::outdated`.
  - M7: `ALIGNMENT_LINE_THICKNESS`; `fp_remote::{Timing, spawn_with}` and a `bind_log` test that waits for four published status snapshots instead of sleeping 5.5 s.
  - M2 of the remote plan (Matroska Opus seek to 0 drops the pre-skip by timestamp, ±24 samples, 0.5 ms) stays a recorded ruling; nothing changed.
  - Missing-file M2: the reason tooltip is also on the title label of an unavailable row.
  - Docs: `players.md` says which meters have a peak hold (checked against `MeterBallistics::settings()`).
  - Thread priority: `fp_decode::priority` over `thread-priority` 3.1.1 (MIT; `cargo deny check` passed with no change to `deny.toml`). Decode workers ask for `AboveNormal` (value 60: nice −5, above normal on Windows), the analysis pool for `Low` (nice 19, lowest). A refusal (the usual answer to a raise for an ordinary Linux user) is one `warn` per run. Persistence stays on the services thread; main spec §2.2 says so now. `threading-and-realtime.md` is aligned.
  - Long-file check: a four-hour mono 44.1 kHz WAV (and a 128 kbit/s stereo MP3) played in the application under Xvfb; the seek to 3:59:00 through `POST /players/{id}/seek` was accepted at once and the position read 14341.99 s two seconds later; seeking back and to 14399.5 s worked and the player stopped at the end; no errors in the log. No defect found.
```

- [ ] **Step 6: Check and commit**

Re-read each edited paragraph against the code, then:

```bash
export PATH=$HOME/.cargo/bin:$PATH; cargo fmt --all --check && cargo clippy --workspace --all-targets -q -- -D warnings && cargo test --workspace -q
git add README.md docs
git commit -m "docs: peak hold, thread priority, cart analysis, remote peaks and the plan 9 notes"
```

---

## Self-Review

**Spec coverage.**
- M1: Task 2. M4: Task 3. M5: Task 4. M6: Task 5. M7: Task 1 (rename and the `bind_log` sleep). M2 (Opus pre-skip): the first Decision and the "As built" bullet in Task 10 (a recorded ruling, no code).
- Missing-file M2 (tooltip on the title cell): Task 6.
- Docs (`players.md` peak hold): Task 10 Step 1, checked against `MeterBallistics::settings()` and the existing model test.
- Thread priority: Task 7 (the crate, `cargo deny check`, the module, the refusal logged once), Task 8 (decoder workers above normal, analysis pool low), Task 10 Step 3 (`threading-and-realtime.md` and the main spec).
- Long-file check: Task 9 (Xvfb, a four-hour file, seek to 3:59:00 through the remote API, the ledger; a defect is fixed test-first there).
- Final documentation, spec status and "As built", roadmap row 9, README: Task 10.

**Placeholder scan.** No TBD. Every code block in Tasks 1 to 8 is text that was compiled: the plan was built by applying its blocks, task by task, to a scratch copy of plan 8's code (`f230760`, the same tree as `master` at `dbcd8b6`), and each task's commit passes `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`. The one text change after that replay is a comment in Task 4's test (the claim "served from the cache" became "answers the same", since nothing asserts it) and the added concurrency test of Task 4, which was run on its own. Task 9 was executed once by hand while the plan was written (reference result in Step 7).

**Type consistency.** `Timing`/`spawn_with` (Task 1) are used only by the `bind_log` test. `OutdatedCount::get(&mut self, &Arc<AppState>) -> usize` and `SettingsDeps::outdated: usize` (Task 5) are what `app.rs` and `settings.rs` use. `fp_decode::priority::{Priority, set_current}` (Task 7) are what `worker.rs` and `analyzer.rs` call (Task 8). `Bridge::on_demand` (Task 4) is named the same in Task 10.

**Review Focus.** Each of the five lines names tests in the task that owns the code.

**Known weak spots to watch in review.**
- Task 4 runs a full analysis on the remote thread's blocking pool. A four-hour file takes a while (the HTTP request may time out at `remote.http.request_timeout_ms`, 10 s by default, while the analysis finishes and fills the cache for the next request). Acceptable for an automation endpoint; a reviewer may prefer a `202` and polling.
- The Linux-only tests read `/proc/thread-self` and `/proc/self/task`; on other systems they are compiled out, so priority behaviour on Windows and macOS is covered only by the crate and by CI compiling.
- A refused raise is the normal case on Linux, so the above-normal decoder priority is rarely active there; documenting `RLIMIT_NICE` for installers is a possible follow-up.
- `thread-priority` brings the `windows` crate on Windows (a second version next to `windows-sys` users); `cargo deny` only warns about duplicates.
