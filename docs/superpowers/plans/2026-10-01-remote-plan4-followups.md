# Remote Control Follow-ups Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the Minor findings left by the reviews of remote plans 1–3,
explain the "Play stops at once after a restart" observation, and refresh
the main screenshot (meter alignment mark, time row, a realistic scene at
1920×1080).

**Architecture:** Each item is a bounded fix in the crate that owns it.
- Remote behaviour lives in `fp-remote` (`server.rs`, `http/mod.rs`,
  `osc.rs`, `api.rs`).
- Two new atomic model commands live in `fp-model`: `EditCartPage` and
  `EditCart`, each with its rule tests.
- UI changes live in `fp-app/src/ui` (Settings > Remote, `player.rs`,
  `widgets.rs`).
- Item 10 is a debugging task. Its fix is decided only once the cause is
  confirmed.

**Tech Stack:** Rust, tokio, axum 0.8, tower (`GlobalConcurrencyLimitLayer`),
rosc, egui + egui_kittest, tracing / tracing-subscriber.

**Spec:**
- `docs/superpowers/specs/2026-10-01-remote-control-design.md` (remote);
- `docs/superpowers/specs/2026-09-25-fauste-player-design.md` (main, rules
  13, 17 and 22, crash recovery);
- `docs/superpowers/specs/2026-09-26-phase2-cartwall-settings-design.md`
  (C rules);
- `docs/superpowers/specs/2026-09-27-meters-design.md` (alignment level).

The design of this plan was agreed in chat on 2026-10-01 (bounded fixes,
no new spec).

## Global Constraints

- All code, comments, docs and commits are in English. UI strings go in both
  `en-US` and `es-ES` `main.ftl`.
- Never mention other playout products anywhere.
- No `unsafe`. No `unwrap`, `expect` or `panic` outside tests. Prefer `get`
  over indexing.
- Player and cart rules are pure functions in `fp-model`, with one test per
  rule.
- The UI never blocks.
- Nothing goes on air by itself (rule 10).
- Commit only with fmt, clippy (`-D warnings`) and the whole suite green.
- Every fix starts with a test that is seen failing first. A pinning test
  that passes at once is recorded as such in the ledger.
- Ledger: `.superpowers/sdd/remote-plan4/progress.md`. Every deviation is
  recorded as `Ruling: <decision> — <why> — <cost if wrong>`.

## Review Focus

- **An open SSE stream holds no request permit.** With the limit made global,
  an open event stream must still let the other requests through. Pinned in
  Task 7.
- **Escape while a Settings > Remote field is focused, then the modal
  closes.** The draft must not be applied, because Escape cancels. Pinned in
  Task 3.
- **OSC types of a cleared address.** A vanished string address gets `""`, a
  float `0.0`, an int `0` and a bool `false`, never a type change. Pinned in
  Task 8.
- **`EditCart` that keeps the track while the cart is playing.** It must not
  stop the cart (C8 applies only to a new file). Pinned in Task 4.
- **A restored position past the cue-out** (whatever Task 10 finds): the
  player must not go silent and stop at once on Play. Pinned in Task 10.

---

### Task 1: Moving the entry on air is allowed (pin it, fix the docs)

Rule 13 forbids only *removing* an entry that is current on any player.
Rule 22 says a move re-derives every player's next. `MoveEntry` has no
on-air guard, so the 202 answer is correct and the plan 3 text is wrong.

**Files:**
- Test: `crates/fp-model/tests/editing.rs`
- Test: `crates/fp-remote/tests/edit.rs`
- Modify: `docs/superpowers/plans/2026-10-01-remote-plan3-editing-settings.md`
  (Review Focus, line ~46)
- Modify: `docs/technical/remote-api.md` (the `POST /entries/{id}/move` row)

- [ ] **Step 1: Pinning tests.** In `crates/fp-model/tests/editing.rs`, use
  that file's own fixture helpers (read its top first):

```rust
/// Rule 13 forbids removing an entry on air, not moving it (rule 22).
#[test]
fn rule13_the_entry_on_air_can_be_moved() {
    let mut s = /* fixture with one playlist of 3 entries, as in this file */;
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    let current = s.players[0].current.unwrap();
    let list = s.playlists.first_id().unwrap();
    fp_model::apply(&mut s, Command::MoveEntry { entry: current, to: list, index: 3 }).unwrap();
    assert_eq!(s.players[0].current, Some(current), "still on air");
    let order: Vec<_> = s.playlists.get(list).unwrap().entries.iter().map(|e| e.id).collect();
    assert_eq!(order.last(), Some(&current));
    assert_eq!(s.players[0].next, None, "nothing after it any more");
}
```

  In `crates/fp-remote/tests/edit.rs`:

```rust
#[tokio::test]
async fn moving_the_entry_on_air_is_accepted() {
    let mut s = demo_state();
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    let current = s.players[0].current.unwrap();
    let main = s.playlists.first_id().unwrap();
    let fake = FakeControl::new(s);
    let status = call(&fake, "POST", &format!("/api/v1/entries/{}/move", current.0),
        Some(json!({"playlist": main.0, "index": 2}))).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert!(matches!(fake.take_sent().as_slice(), [Command::MoveEntry { .. }]));
}
```

- [ ] **Step 2: Run them.**
  `cargo test -p fp-model --test editing rule13_the_entry_on_air_can_be_moved`
  and `cargo test -p fp-remote --test edit moving_the_entry_on_air`. They are
  expected to PASS: they pin behaviour that is already correct. Record
  `Ruling: item 1 pinned, no fix — rule 13 forbids removal only — none` in
  the ledger. If the `next` assertion fails, read rule 12 before changing
  the test.

- [ ] **Step 3: Docs.**
  - Plan 3, Review Focus: change "deleting the playlist on air, or moving or
    removing the current entry, answers `409`" to "deleting the playlist on
    air, or removing the current entry, answers `409` (moving it is allowed,
    rule 13)".
  - `remote-api.md`: change the move row to "Move it, also to another
    playlist (allowed while on air; only removal is refused, rule 13)".

- [ ] **Step 4: Gate and commit.**
  `test(remote): pin that the entry on air can be moved`

---

### Task 2: A busy port is logged once, not every retry

**Files:**
- Create: `crates/fp-remote/src/repeat_log.rs`, declared as
  `pub mod repeat_log;` in `lib.rs`
- Modify: `crates/fp-remote/src/server.rs`
- Modify: `crates/fp-remote/Cargo.toml` (dev-dep `tracing-subscriber.workspace = true`)
- Test: `crates/fp-remote/tests/repeat_log.rs` (unit, explicit time)
- Test: `crates/fp-remote/tests/bind_log.rs` (its own binary, global subscriber)
- Modify: `docs/technical/remote-api.md` (around line 333),
  `docs/user/remote-control.md` (troubleshooting, if it mentions the port)

**Interfaces:**
- Produces: `pub struct RepeatLog`, `RepeatLog::new(every: Duration)`,
  `fn should_log(&mut self, error: &str, now: Instant) -> bool`,
  `fn clear(&mut self)`; `pub const BIND_LOG_EVERY: Duration` (5 min) in
  `server.rs`.

- [ ] **Step 1: Failing unit test** `tests/repeat_log.rs`:

```rust
use std::time::{Duration, Instant};
use fp_remote::repeat_log::RepeatLog;

#[test]
fn the_same_error_is_logged_once_then_after_the_interval() {
    let every = Duration::from_secs(300);
    let mut log = RepeatLog::new(every);
    let t0 = Instant::now();
    assert!(log.should_log("in use", t0));
    for k in 1..10 {
        assert!(!log.should_log("in use", t0 + Duration::from_secs(2 * k)));
    }
    assert!(log.should_log("in use", t0 + every));
    assert!(log.should_log("permission denied", t0 + every + Duration::from_secs(2)));
    log.clear();
    assert!(log.should_log("permission denied", t0 + every + Duration::from_secs(4)));
}
```

- [ ] **Step 2: Run it.** `cargo test -p fp-remote --test repeat_log`.
  Expected: FAIL (`repeat_log` is not found).

- [ ] **Step 3: Implement** `src/repeat_log.rs`:

```rust
//! Decides when a failure that repeats (a busy port, retried every
//! `BIND_RETRY`) is worth a log line: the first time, when the error
//! changes, and every `every` while it lasts.

use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct RepeatLog {
    every: Duration,
    last: Option<(String, Instant)>,
}

impl RepeatLog {
    pub fn new(every: Duration) -> Self {
        Self { every, last: None }
    }

    pub fn should_log(&mut self, error: &str, now: Instant) -> bool {
        let due = match &self.last {
            Some((e, at)) => e != error || now.saturating_duration_since(*at) >= self.every,
            None => true,
        };
        if due {
            self.last = Some((error.to_owned(), now));
        }
        due
    }

    /// The failure is over: the next one is logged at once.
    pub fn clear(&mut self) {
        self.last = None;
    }
}
```

- [ ] **Step 4: Run it.** Expected: PASS.

- [ ] **Step 5: Failing integration test** `tests/bind_log.rs`. It is its own
  test binary, so the global subscriber is safe.

```rust
#![allow(clippy::unwrap_used)]
mod support;

use std::io::Write;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use fp_remote::{ServerError, ServerStatus, spawn};
use support::{FakeControl, demo_state};

#[derive(Clone, Default)]
struct Lines(Arc<Mutex<Vec<u8>>>);
impl Write for Lines {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}

#[test]
fn a_busy_port_is_logged_once_across_retries() {
    let lines = Lines::default();
    let writer = lines.clone();
    tracing_subscriber::fmt()
        .with_writer(move || writer.clone())
        .with_ansi(false)
        .init();
    let busy = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = busy.local_addr().unwrap().port();
    let fake = FakeControl::new(demo_state());
    fake.edit(|s| { s.config.remote.http.enabled = true; s.config.remote.http.port = port; });
    let handle = spawn(fake.clone()).unwrap();
    // BIND_RETRY is 2 s: wait for three attempts.
    let deadline = Instant::now() + Duration::from_millis(5_500);
    while Instant::now() < deadline { std::thread::sleep(Duration::from_millis(100)); }
    assert!(matches!(handle.status().http, ServerStatus::Error(ServerError::Bind(_))));
    let text = String::from_utf8(lines.0.lock().unwrap().clone()).unwrap();
    assert_eq!(text.matches("remote HTTP could not listen").count(), 1, "{text}");
    drop(handle);
}
```

- [ ] **Step 6: Run it.** `cargo test -p fp-remote --test bind_log`.
  Expected: FAIL with a count of 3.

- [ ] **Step 7: Implement in `server.rs`.**
  - Add `pub const BIND_LOG_EVERY: Duration = Duration::from_secs(300);`
    with a doc comment ("how often a bind failure that persists is logged
    again").
  - `supervise` owns `let mut http_log = RepeatLog::new(BIND_LOG_EVERY);`
    and `osc_log`, and passes `&mut` to `start` and `start_osc`.
  - In both bind `Err(e)` arms, wrap the `tracing::warn!` in
    `if log.should_log(&e.to_string(), std::time::Instant::now())`.
  - On a successful bind, and when the server is off, call `log.clear()`.
  - Add the module doc line to `lib.rs`.

- [ ] **Step 8: Run** both tests and `cargo test -p fp-remote --test server`.
  Expected: PASS. Note that the test takes about 6 s.

- [ ] **Step 9: Docs.** `remote-api.md`: "...tried again every 2 s, so a port
  freed later is taken without a restart. The failure is logged once, again
  when its reason changes, and every 5 minutes while it lasts."

- [ ] **Step 10: Gate and commit.**
  `fix(remote): log a busy port once, not on every retry`

---

### Task 3: Settings > Remote applies valid drafts when left or closed

**Files:**
- Modify: `crates/fp-app/src/ui/settings/remote.rs`
- Modify: `crates/fp-app/src/ui/settings.rs` (`nav`, `show`)
- Test: `crates/fp-app/tests/remote_settings.rs`
- Modify: `docs/superpowers/specs/2026-10-01-remote-control-design.md` §8,
  `docs/user/remote-control.md` (Settings part)

**Interfaces:**
- Produces: `pub(super) fn flush(scene: &Scene<'_>, st: &mut RemoteState)`
  in `remote.rs`; a private `fn commit(scene, key: &'static str, text: &str)`
  shared by the focus-lost path and `flush`; a private
  `fn commit_number(scene, key, value: u32)`.

- [ ] **Step 1: Failing tests.** Append to `remote_settings.rs` (it uses the
  existing `opened`, `sent_configs` and `state` helpers). Type without
  pressing a key that ends the edit:

```rust
fn type_into(h: &mut egui_kittest::Harness<'static, fp_app::ui::app::AppUi>, label: &str, nth: usize, text: &str) {
    h.get_all_by_role_and_label(Role::TextInput, label).nth(nth).unwrap().focus();
    h.run_steps(2);
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.run_steps(1);
    for c in text.chars() {
        h.get_all_by_role_and_label(Role::TextInput, label).nth(nth).unwrap().type_text(&c.to_string());
        h.run_steps(1);
    }
}

#[test]
fn a_draft_is_applied_when_another_section_is_opened() {
    let (mut h, fake) = opened(state(1, 0));
    type_into(&mut h, "Address", 0, "10.0.0.9");
    h.get_by_role_and_label(Role::Button, "MIDI").click();
    h.run_steps(3);
    assert_eq!(fake.state.load().config.remote.http.bind, "10.0.0.9");
}

#[test]
fn a_draft_is_applied_when_settings_closes() {
    let (mut h, fake) = opened(state(1, 0));
    type_into(&mut h, "Address", 0, "10.0.0.9");
    h.get_by_label("Close").click();
    h.run_steps(3);
    assert_eq!(fake.state.load().config.remote.http.bind, "10.0.0.9");
}

#[test]
fn an_invalid_draft_is_dropped_when_settings_closes() {
    let (mut h, fake) = opened(state(1, 0));
    type_into(&mut h, "Address", 0, "10.0.");
    h.get_by_label("Close").click();
    h.run_steps(3);
    assert!(sent_configs(&fake).iter().all(|c| c.remote.http.bind != "10.0."));
    assert_eq!(fake.state.load().config.remote.http.bind, "127.0.0.1");
}

#[test]
fn escape_cancels_a_draft_even_if_settings_closes() {
    let (mut h, fake) = opened(state(1, 0));
    type_into(&mut h, "Address", 0, "10.0.0.9");
    h.key_press(egui::Key::Escape);
    h.run_steps(3);
    assert_eq!(fake.state.load().config.remote.http.bind, "127.0.0.1");
}
```

  Check the real labels first: the MIDI tab label (`settings-tab-midi`), the
  close button (`settings-close`) and the default `bind`. Adjust to the
  `en-US` strings.

- [ ] **Step 2: Run them.** `cargo test -p fp-app --test remote_settings draft`.
  Expected: the first two FAIL, because the bind stays at the default. The
  Escape test may already pass; it pins behaviour.

- [ ] **Step 3: Implement.**
  - In `remote.rs`, move each field's validation and `update` call into
    `commit(scene, key, text)` with a `match key`:
    - `"http.bind" | "osc.bind"`: `address`;
    - `"http.token"`: trim, then `token_acceptable`;
    - `"http.origins"`: `lines`;
    - `"osc.sources"`: `lines`.
    The `field` call sites then call `commit` on `Some(text)`.
  - Move number applies into `commit_number(scene, key, value)`, with a
    `match` for `"http.port" | "osc.port" | "events.position"`. Ports go
    through `u16::try_from`.
  - Add `flush`:

```rust
/// Applies what is still being typed when the section is left or Settings
/// closes; an invalid draft is dropped, as when its field loses focus.
pub(super) fn flush(scene: &Scene<'_>, st: &mut RemoteState) {
    for (key, text) in std::mem::take(&mut st.drafts) {
        commit(scene, key, &text);
    }
    for (key, value) in std::mem::take(&mut st.numbers) {
        commit_number(scene, key, value);
    }
}
```

  - A draft equal to the current value is harmless, because `update` only
    sends a changed configuration. Two commits in one frame must not
    overwrite each other: `update` clones `scene.state.config`, which does
    not change within the frame. So `flush` builds one config by applying
    all edits to a single clone. Make `commit` take
    `&mut fp_model::Config`, and let both callers wrap it in `update`:
    `update(scene, |c| for (k, t) in drafts { commit(c, k, &t) })`.
  - In `settings.rs` `nav`, when a click changes the section and the old
    one is `Section::Remote`, call `remote::flush(scene, &mut st.remote)`
    first.
  - In `show`, just before `open` is returned false:
    `if !open && !ctx.input(|i| i.key_pressed(egui::Key::Escape)) { remote::flush(scene, &mut st.remote); }`.
    A field focused at Escape has already removed its own draft in `field`,
    but the modal may close in the same frame, so the Escape guard keeps the
    cancel.

- [ ] **Step 4: Run** the four tests and the whole `remote_settings` file.
  Expected: PASS.

- [ ] **Step 5: Docs.**
  - Spec §8: add "Text and number fields apply when they lose focus, when
    another section is opened and when Settings closes; an invalid value is
    dropped and the one in use kept; Escape cancels the edit."
  - Make the same sentence in the user guide.

- [ ] **Step 6: Gate and commit.**
  `fix(ui): Settings > Remote applies a pending edit when left or closed`

---

### Task 4: Atomic cart page and cart edits (`EditCartPage`, `EditCart`)

**Files:**
- Modify: `crates/fp-model/src/command.rs`, `reducer.rs`, `cart_rules.rs`,
  `lib.rs` (export `CartFileChange`)
- Modify: `crates/fp-remote/src/api.rs` (`Edit::EditCartPage`, `Edit::SetCart`)
- Test: `crates/fp-model/tests/cartwall.rs`
- Test: `crates/fp-remote/tests/edit.rs` (update the two existing tests)
- Modify: phase 2 spec (command table line ~95, new rule C11), remote spec
  table lines 167/169, `docs/technical/remote-api.md` if it names commands

**Interfaces:**
- Produces:

```rust
/// What an `EditCart` does to the cart's file.
#[derive(Debug, Clone, PartialEq)]
pub enum CartFileChange { Keep, Track(TrackId), Clear }

Command::EditCartPage { page: CartPageId, name: Option<String>, grid: Option<(u16, u16)> }
Command::EditCart { page: CartPageId, index: usize, edit: CartEdit, file: CartFileChange }
```

- [ ] **Step 1: Failing rule tests** in `fp-model/tests/cartwall.rs`, with
  the file's helpers `page`, `load`, `started` and `stopped`:

```rust
/// C11: a page edit is checked whole, then applied whole.
#[test]
fn c11_a_page_edit_that_cannot_resize_does_not_rename() {
    let mut s = fixture();
    let p = page(&s);
    load(&mut s, 5, 30.0); // a cart with a file past a 1×1 grid
    let before = s.cartwall.page(p).unwrap().clone();
    let e = fp_model::apply(&mut s, Command::EditCartPage { page: p, name: Some("New".into()), grid: Some((1, 1)) });
    assert!(e.is_err());
    assert_eq!(s.cartwall.page(p).unwrap(), &before);
}

#[test]
fn c11_a_page_edit_renames_and_resizes_together() {
    let mut s = fixture();
    let p = page(&s);
    fp_model::apply(&mut s, Command::EditCartPage { page: p, name: Some("New".into()), grid: Some((2, 3)) }).unwrap();
    let pg = s.cartwall.page(p).unwrap();
    assert_eq!((pg.name.as_str(), pg.rows, pg.cols, pg.carts.len()), ("New", 2, 3, 6));
}

#[test]
fn c11_a_cart_edit_with_an_unknown_track_changes_nothing() {
    let mut s = fixture();
    let p = page(&s);
    let before = s.cartwall.page(p).unwrap().carts[0].clone();
    let edit = CartEdit { name: "X".into(), kind: CartKind::Spot, looped: true, exclusive: true };
    let e = fp_model::apply(&mut s, Command::EditCart { page: p, index: 0, edit, file: CartFileChange::Track(TrackId(9_999)) });
    assert!(e.is_err());
    assert_eq!(s.cartwall.page(p).unwrap().carts[0], before);
}

#[test]
fn c11_a_cart_edit_keeping_its_file_does_not_stop_it() {
    let mut s = fixture();
    let cart = load(&mut s, 0, 30.0);
    fp_model::apply(&mut s, Command::FireCart(cart)).unwrap();
    let edit = CartEdit { name: "Kept".into(), kind: CartKind::Jingle, looped: false, exclusive: false };
    let out = fp_model::apply(&mut s, Command::EditCart { page: page(&s), index: 0, edit, file: CartFileChange::Keep }).unwrap();
    assert!(stopped(&out).is_empty());
    assert_eq!(s.cartwall.page(page(&s)).unwrap().carts[0].name, "Kept");
}

#[test]
fn c11_a_cart_edit_with_a_new_track_stops_it_first() {
    let mut s = fixture();
    let cart = load(&mut s, 0, 30.0);
    let other = load(&mut s, 1, 30.0);
    let other_track = s.cartwall.cart(other).unwrap().track.unwrap();
    fp_model::apply(&mut s, Command::FireCart(cart)).unwrap();
    let edit = CartEdit { name: "New".into(), kind: CartKind::Jingle, looped: false, exclusive: false };
    let out = fp_model::apply(&mut s, Command::EditCart { page: page(&s), index: 0, edit, file: CartFileChange::Track(other_track) }).unwrap();
    assert_eq!(stopped(&out), vec![cart]);
    assert_eq!(s.cartwall.cart(cart).unwrap().track, Some(other_track));
}
```

  Adapt to the real signatures of `load`, `fixture` and `apply`'s return
  value (read the top of `cartwall.rs`). Tests whose carts must be on the
  page use the default grid size.

- [ ] **Step 2: Run them.** `cargo test -p fp-model --test cartwall c11`.
  Expected: FAIL to compile (no `EditCartPage`).

- [ ] **Step 3: Implement.**
  - `command.rs`: add the enum and the two variants, with doc comments
    ("Renames and/or resizes a page as one change: refused whole, C11", and
    "Edits a cart and its file as one change, C8 and C11").
  - `cart_rules.rs`:

```rust
/// C11: every check first, then every change.
pub(crate) fn edit_page(state: &mut AppState, page: CartPageId, name: Option<String>, grid: Option<(u16, u16)>) -> Result<(), ModelError> {
    state.cartwall.page(page).ok_or(ModelError::UnknownCartPage(page))?;
    if let Some((rows, cols)) = grid {
        resize_page(state, page, rows, cols)?; // checks before it mutates
    }
    if let Some(name) = name {
        rename_page(state, page, name)?;
    }
    Ok(())
}

/// C8, C11.
pub(crate) fn edit_cart(state: &mut AppState, page: CartPageId, index: usize, edit: CartEdit, file: CartFileChange, out: &mut Vec<EngineAction>) -> Result<(), ModelError> {
    cart_at(state, page, index)?;
    if let CartFileChange::Track(t) = &file && state.library.get(*t).is_none() {
        return Err(ModelError::UnknownTrack(*t));
    }
    set_cart(state, page, index, edit)?;
    match file {
        CartFileChange::Keep => Ok(()),
        CartFileChange::Track(t) => set_track(state, page, index, Some(t), out),
        CartFileChange::Clear => set_track(state, page, index, None, out),
    }
}
```

  Verify that `resize_page` returns before it mutates on every error (it
  does: `check_grid` and `CartsWouldBeLost` come first). Keep a
  `Track(t)` equal to the current track as `Keep`: if `cart.track ==
  Some(t)`, skip `set_track`. This keeps C8 ("only a new file stops it")
  true even when a caller sends the same track.
  - `reducer.rs`: wire the two variants. Then fix any exhaustive `match` on
    `Command` that no longer compiles (for example `availability.rs`, the
    conductor or MIDI). Use `cargo build --workspace` to find them.
  - `fp-remote/src/api.rs`:
    - `Edit::EditCartPage` emits one `Command::EditCartPage`, with the name
      trimmed and `grid` from rows/cols defaulting to the page's.
    - `Edit::SetCart` emits one `Command::EditCart` with
      `file = match wanted { Some(t) if cart.track != Some(t) => Track(t),
      None if cart.track.is_some() => Clear, _ => Keep }`.
    - The range check stays (`400` before the model's 409).

- [ ] **Step 4: Update the two remote tests**
  (`cart_pages_are_renamed_and_resized_within_limits`,
  `a_cart_edit_keeping_its_track_does_not_reassign_it`) to expect the single
  new command. Add an HTTP test: with `fake.accept` false, `PATCH` with name
  and rows gives `503` and `take_sent()` is empty.

- [ ] **Step 5: Run.** `cargo test -p fp-model --test cartwall` and
  `cargo test -p fp-remote`. Expected: PASS.

- [ ] **Step 6: Docs.**
  - Phase 2 spec: add **C11. Atomic edits.** "`EditCartPage` and `EditCart`
    check everything first and apply all or nothing; a remote client edits
    pages and carts through them." Also add both commands to the table.
  - Remote spec table: `EditCartPage` and `EditCart` replace the pairs.
  - `remote-api.md`: the same, if it names commands.

- [ ] **Step 7: Gate and commit.**
  `fix(remote): edit a cart page and a cart as one atomic command`

---

### Task 5: Pin `forget_tracks` on removing an entry whose track a cart uses

**Files:**
- Test: `crates/fp-model/tests/reuse_tracks.rs`

- [ ] **Step 1: Test.**

```rust
/// A track shared by a cart survives removing its playlist entry.
#[test]
fn removing_an_entry_keeps_a_track_a_cart_still_uses() {
    let mut s = /* fixture of this file */;
    let page = s.cartwall.pages[0].id;
    fp_model::apply(&mut s, Command::AssignCartFile { page, index: 0, path: "/m/jingle.wav".into() }).unwrap();
    let t = s.cartwall.pages[0].carts[0].track.unwrap();
    let list = s.playlists.first_id().unwrap();
    fp_model::apply(&mut s, Command::InsertTracks { playlist: list, index: 0, tracks: vec![t] }).unwrap();
    let entry = s.playlists.get(list).unwrap().entries[0].id;
    fp_model::apply(&mut s, Command::RemoveEntry(entry)).unwrap();
    assert!(s.library.get(t).is_some(), "the cart still uses it");
    fp_model::apply(&mut s, Command::ClearCartFile { page, index: 0 }).unwrap();
    assert!(s.library.get(t).is_none(), "forgotten once nothing uses it");
}
```

- [ ] **Step 2: Run it.** `cargo test -p fp-model --test reuse_tracks removing_an_entry_keeps`.
  Expected: PASS (it pins behaviour). Record
  `Ruling: item 5 pinned, no fix — forget_tracks checks carts — none`.

- [ ] **Step 3: Gate and commit.**
  `test(model): pin that a track a cart uses survives removing its entry`

---

### Task 6: Settings > Remote buttons use the Settings button

**Files:**
- Modify: `crates/fp-app/src/ui/settings/midi.rs` (remove its local `button`)
- Modify: `crates/fp-app/src/ui/settings.rs` (add `pub(super) fn button`)
- Modify: `crates/fp-app/src/ui/settings/remote.rs`
- Test: `crates/fp-app/tests/remote_settings.rs`

- [ ] **Step 1: Failing test.** The tile button is as tall as the MIDI ones
  (24 px); egui's default button is not:

```rust
#[test]
fn the_token_buttons_are_settings_buttons() {
    let (h, _) = opened(state(1, 0));
    for label in ["Show", "Copy", "Generate"] {
        let r = h.get_by_label(label).rect();
        assert!((r.height() - 24.0).abs() < 0.5, "{label}: {r:?}");
    }
}
```

- [ ] **Step 2: Run it.** Expected: FAIL (the egui default height is about
  18–20).
- [ ] **Step 3: Implement.** Move `button` from `midi.rs` to `settings.rs` as
  `pub(super) fn button(ui: &mut Ui, text: &str) -> bool`, unchanged. Use it
  in `midi.rs` as `super::button`, and in `remote.rs` for Show/Hide, Copy
  and Generate. Keep the `widget_info` label: check that `widgets::tile`
  sets it from `text` (the MIDI tests find buttons by label, so it does).
- [ ] **Step 4: Run** `remote_settings` and `midi_settings`. Expected: PASS.
- [ ] **Step 5: Visual check.** Build with `cargo build --release -p fp-app`.
  Open Settings > Remote in Xvfb, following the `CLAUDE.md` recipe. Without
  the API, click the gear and the tab with `xdotool`, find coordinates with
  a first capture, and save to the scratchpad. Look at the image: the three
  buttons must match the MIDI section's buttons.
- [ ] **Step 6: Gate and commit.**
  `fix(ui): Settings > Remote buttons look like the other sections'`

---

### Task 7: The in-flight request limit covers the whole server

**Files:**
- Modify: `crates/fp-remote/src/http/mod.rs`
- Modify: `crates/fp-remote/tests/support/mod.rs` (a cover gate)
- Test: `crates/fp-remote/tests/http.rs`
- Modify: `docs/technical/remote-api.md` line ~271

**Interfaces:**
- Produces: `pub const IN_FLIGHT_REQUESTS: usize` (now `pub`);
  `FakeControl::close_covers()`, `open_covers()`, `covers_waiting() -> usize`.

- [ ] **Step 1: Fake support.** Add these fields and methods to
  `FakeControl`:
  - `cover_gate: Mutex<bool>` (true means open, the default);
  - `cover_open: Condvar`;
  - `waiting: AtomicUsize`.

  `cover()` increments `waiting`, waits while the gate is closed, then
  decrements it. `open_covers` sets the gate and calls `notify_all`.

- [ ] **Step 2: Failing tests** in `http.rs`:

```rust
fn get(uri: &str) -> Request<Body> {
    Request::builder().uri(uri).header("host", "127.0.0.1:7380").body(Body::empty()).unwrap()
}

/// Analysed track with a cover, and the covers held.
fn held_covers() -> (Arc<FakeControl>, String) {
    let mut s = demo_state();
    let t = s.playlists.iter().next().unwrap().entries[0].track;
    s.library.get_mut(t).unwrap().analyzed = true;
    let fake = FakeControl::new(s);
    fake.covers.lock().unwrap().insert(t, vec![0x89, b'P']);
    fake.close_covers();
    (fake, format!("/api/v1/tracks/{}/cover", t.0))
}

async fn wait_waiting(fake: &FakeControl, n: usize) {
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
    while fake.covers_waiting() < n {
        assert!(tokio::time::Instant::now() < deadline, "{} waiting", fake.covers_waiting());
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
}

#[tokio::test]
async fn the_request_limit_is_shared_by_every_route() {
    use fp_remote::http::IN_FLIGHT_REQUESTS;
    let (fake, cover) = held_covers();
    let app = router(ctx(&fake));
    let held: Vec<_> = (0..IN_FLIGHT_REQUESTS)
        .map(|_| tokio::spawn(app.clone().oneshot(get(&cover))))
        .collect();
    wait_waiting(&fake, IN_FLIGHT_REQUESTS).await;
    let mut state = tokio::spawn(app.clone().oneshot(get("/api/v1/state")));
    let early = tokio::time::timeout(std::time::Duration::from_millis(300), &mut state).await;
    assert!(early.is_err(), "request N+1 waits for a free slot");
    fake.open_covers();
    let res = tokio::time::timeout(std::time::Duration::from_secs(5), state).await.unwrap().unwrap().unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    for h in held { assert_eq!(h.await.unwrap().unwrap().status(), StatusCode::OK); }
}

#[tokio::test]
async fn an_open_event_stream_holds_no_slot() {
    use fp_remote::http::IN_FLIGHT_REQUESTS;
    let (fake, cover) = held_covers();
    let app = router(ctx(&fake));
    let stream = app.clone().oneshot(get("/api/v1/events")).await.unwrap();
    assert_eq!(stream.status(), StatusCode::OK); // headers back, body open
    let held: Vec<_> = (0..IN_FLIGHT_REQUESTS - 1)
        .map(|_| tokio::spawn(app.clone().oneshot(get(&cover))))
        .collect();
    wait_waiting(&fake, IN_FLIGHT_REQUESTS - 1).await;
    let res = tokio::time::timeout(std::time::Duration::from_secs(2), app.clone().oneshot(get("/api/v1/state")))
        .await.expect("the stream must not hold a slot").unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    fake.open_covers();
    for h in held { h.await.unwrap().unwrap(); }
    drop(stream);
}
```

  If the events route needs an `Accept` header or a non-zero
  `max_event_clients`, copy what `tests/sse.rs` does.

- [ ] **Step 3: Run them.** `cargo test -p fp-remote --test http limit slot`.
  Expected: the first FAILS (`/state` answers at once, because the limit is
  per route). The second may pass; it pins behaviour.
- [ ] **Step 4: Implement.** In `router`, replace
  `.layer(ConcurrencyLimitLayer::new(IN_FLIGHT_REQUESTS))` with
  `.layer(GlobalConcurrencyLimitLayer::new(IN_FLIGHT_REQUESTS))` (from
  `tower::limit`). Its one semaphore is shared by every route's clone of the
  layer. Make the constant `pub` and keep its comment. Check that the
  `limit` feature of `tower` is enabled in the workspace; it already is for
  `ConcurrencyLimitLayer`.
- [ ] **Step 5: Run** `cargo test -p fp-remote`. Expected: PASS.
- [ ] **Step 6: Docs.** `remote-api.md`: "64 requests in flight across the
  whole server (the rest wait their turn; an open event stream does not
  count)".
- [ ] **Step 7: Gate and commit.**
  `fix(remote): one in-flight request limit for the whole server`

---

### Task 8: OSC clears addresses that disappear

**Files:**
- Modify: `crates/fp-remote/src/osc.rs` (`Subscribers::changes`, a `cleared` helper)
- Test: `crates/fp-remote/tests/osc.rs` (unit), `crates/fp-remote/tests/osc_server.rs`
- Modify: remote spec §5.3, `docs/technical/remote-api.md` (OSC part),
  `docs/user/remote-control.md` (OSC part)

**Interfaces:**
- Produces: `pub fn cleared(value: &OscType) -> OscType` in `osc.rs`.

- [ ] **Step 1: Failing unit test** in `tests/osc.rs`:

```rust
#[test]
fn an_address_that_disappears_is_sent_an_empty_value_once() {
    let mut subs = Subscribers::new(4, Duration::from_secs(10));
    let to: SocketAddr = "127.0.0.1:9000".parse().unwrap();
    subs.subscribe(to, Instant::now());
    let both = vec![
        ("/a".to_owned(), OscType::String("x".into())),
        ("/b".to_owned(), OscType::Float(0.5)),
        ("/c".to_owned(), OscType::Int(3)),
        ("/d".to_owned(), OscType::Bool(true)),
    ];
    subs.changes(&both);
    let only_a = vec![("/a".to_owned(), OscType::String("x".into()))];
    let out = subs.changes(&only_a);
    let mut got: Vec<(String, Vec<OscType>)> = out[0].1.iter().map(|m| (m.addr.clone(), m.args.clone())).collect();
    got.sort_by(|x, y| x.0.cmp(&y.0));
    assert_eq!(got, vec![
        ("/b".into(), vec![OscType::Float(0.0)]),
        ("/c".into(), vec![OscType::Int(0)]),
        ("/d".into(), vec![OscType::Bool(false)]),
    ]);
    assert!(subs.changes(&only_a).is_empty(), "cleared once, then forgotten");
}
```

- [ ] **Step 2: Run it.** Expected: FAIL (`out` is empty).
- [ ] **Step 3: Implement.** In `changes`, after the loop over `values`, for
  each `addr` in `s.sent` that is not in `values` (build a `HashSet<&str>`
  of the value addresses once per call), push
  `OscMessage { addr, args: vec![cleared(&old)] }` and remove it from
  `s.sent`.

```rust
/// The empty value of `value`'s type, sent to an address that no longer
/// exists so a surface does not keep showing its last value.
pub fn cleared(value: &OscType) -> OscType {
    match value {
        OscType::String(_) => OscType::String(String::new()),
        OscType::Float(_) => OscType::Float(0.0),
        OscType::Double(_) => OscType::Double(0.0),
        OscType::Int(_) => OscType::Int(0),
        OscType::Long(_) => OscType::Long(0),
        OscType::Bool(_) => OscType::Bool(false),
        _ => OscType::Nil,
    }
}
```

  `reset()` currently clears `sent`, which would lose what must be cleared.
  Change it to mark a full resend without forgetting: add
  `resend: bool` to `Subscriber`. `changes` sends every value when it is
  set, then clears it. The clearing pass still sees the old addresses.
- [ ] **Step 4: Failing server test** in `tests/osc_server.rs`:

```rust
#[test]
fn fewer_players_clear_the_addresses_of_the_removed_ones() {
    let fake = FakeControl::new(demo_state());
    enabled(&fake);
    let handle = spawn(fake.clone()).unwrap();
    let to = osc_addr(&handle);
    let sock = UdpSocket::bind("127.0.0.1:0").unwrap();
    send(&sock, to, "/fauste/subscribe", vec![]);
    let mut buf = [0u8; 65536];
    sock.set_read_timeout(Some(Duration::from_millis(300))).unwrap();
    while sock.recv_from(&mut buf).is_ok() {}
    fake.edit(|s| {
        let mut c = s.config.clone();
        c.players.count = 2;
        fp_model::apply(s, Command::UpdateConfig(Box::new(c))).unwrap();
    });
    sock.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        assert!(Instant::now() < deadline, "player 4 was never cleared");
        let (n, _) = sock.recv_from(&mut buf).unwrap();
        if let Ok((_, OscPacket::Message(m))) = decoder::decode_udp(&buf[..n])
            && m.addr == "/fauste/player/4/title"
        {
            assert_eq!(m.args, vec![OscType::String(String::new())]);
            break;
        }
    }
}
```

  Use an address that `osc::values` really publishes for each player (read
  `values` in `osc.rs`; `title` or `transport`). `transport` clears to `""`.
- [ ] **Step 5: Run** `cargo test -p fp-remote --test osc --test osc_server`.
  Expected: PASS, including `a_grid_resize_sends_everything_again`.
- [ ] **Step 6: Docs.** Spec §5.3, after "sends a full dump": "and an address
  that no longer exists (a removed player, a cart beyond a smaller grid) is
  sent once its empty value (`""`, `0`, `0.0`, false)". Put the same text in
  the technical and user docs.
- [ ] **Step 7: Gate and commit.**
  `fix(remote): OSC clears the addresses that disappear`

---

### Task 9: Measure the dry-run clone

**Files:**
- Create: `crates/fp-remote/tests/plan_cost.rs` (one `#[ignore]` test, run
  by hand)

- [ ] **Step 1: Write the measurement:**

```rust
#![allow(clippy::unwrap_used)]
//! The cost of the dry run on a large state (plan 4, item 9). Local only:
//! `cargo test --release -p fp-remote --test plan_cost -- --ignored --nocapture`.

use std::path::PathBuf;
use std::time::Instant;

use fp_model::{AppState, Command, Config};
use fp_remote::api::{Edit, Operation, plan, plan_edit};

#[test]
#[ignore]
fn plan_on_5000_entries_and_16_players() {
    let mut c = Config::default();
    c.players.count = 16;
    let mut s = AppState::new(c, "Main");
    let list = s.playlists.first_id().unwrap();
    let paths: Vec<PathBuf> = (0..5_000).map(|i| PathBuf::from(format!("/music/track-{i:05}.flac"))).collect();
    fp_model::apply(&mut s, Command::InsertPaths { playlist: list, index: 0, paths }).unwrap();
    let ids: Vec<_> = s.library.iter().map(|t| t.id).collect();
    for id in ids { s.library.get_mut(id).unwrap().duration_secs = 200.0; }
    let p = s.players[0].id;
    let e = s.playlists.get(list).unwrap().entries[10].id;
    let n = 200;
    let t0 = Instant::now();
    for _ in 0..n { std::hint::black_box(s.clone()); }
    let clone = t0.elapsed() / n;
    let t0 = Instant::now();
    for _ in 0..n { std::hint::black_box(plan(&s, Operation::Play(p)).unwrap()); }
    let play = t0.elapsed() / n;
    let t0 = Instant::now();
    for _ in 0..n { std::hint::black_box(plan_edit(&s, Edit::MoveEntry { entry: e, playlist: list, index: 4_000 }).unwrap()); }
    let mv = t0.elapsed() / n;
    println!("clone {clone:?}  plan(Play) {play:?}  plan_edit(Move) {mv:?}");
}
```

- [ ] **Step 2: Run it** in release, three times, and note the figures.
- [ ] **Step 3: Decide.**
  - If `plan` is under 1 ms per request (an HTTP round trip on a LAN is
    already about 1 ms), write
    `Ruling: keep the dry-run clone — <figures> — a request costs <x> on <machine>; a busy OSC surface at ~50 msg/s would spend <y>%`.
  - Otherwise, skip `dry_run` for the commands that need no proof:
    - those already checked by `command_available` (the transport ones);
    - the idempotent `Set…` commands.
    Do it by having `plan` return early when `op` is in that set. Add a test
    that `plan` still refuses what it refused before (the existing `api.rs`
    tests cover this).
- [ ] **Step 4: Commit** the measurement test
  (`test(remote): measure the dry-run cost on a large state`), plus any code
  change with its gate.

---

### Task 10: A player stops at once after Play following a restart

**REQUIRED SUB-SKILL:** superpowers:systematic-debugging. No fix before the
cause is confirmed.

**Files (investigation first; the fix files depend on the cause):**
- Likely: `crates/fp-model/src/session.rs` (`restore`),
  `crates/fp-model/src/state.rs` (`request_at`)
- Test: `crates/fp-model/tests/session.rs`
- Maybe: the main spec "Crash recovery", `docs/user/` (restart behaviour)

- [ ] **Step 1: Read first.**
  - `AppState::restore` restores each player Paused at the saved position
    via `request_at`.
  - `request_at` clamps to `duration_secs` only when it is known (>0), and
    otherwise not at all.
  - `examples/demo_session.rs` saves positions `40 + (id % 7) * 19` s. With
    30 s tones, every restored position is beyond the end of the file. That
    alone may explain the observation: Play resumes at the end, and the
    source ends at once.
- [ ] **Step 2: Reproduce.**
  - Generate 30 s tagged WAV tones (Python, `LIST/INFO` with `INAM`/`IART`;
    see `CLAUDE.md`) into the scratchpad.
  - Run `demo_session`, then the release app under Xvfb with
    `remote.http.enabled`, and `RUST_LOG=debug`.
  - `POST /api/v1/players/<id>/play`, then poll `GET /api/v1/players/<id>`
    every 100 ms for 2 s. Record the transport, position and the log lines
    (engine `SourceEnded`, transition plans).
  - Repeat with a scratch home where one player played to the end before a
    kill (`kill -9` near 00:29). This separates "position beyond the file"
    from "position at the cue-out".
- [ ] **Step 3: Confirm the hypothesis** with one change at a time (for
  example a session position edited to 10 s plays normally). Write the
  evidence in the ledger.
- [ ] **Step 4: Decide.** The expected outcome, if the hypothesis holds: a
  restored position at or past the track's cue-out (or past its length,
  when unknown) is useless, because Play would end at once.
  - Restore it at the cue-in instead. The model can only decide this where
    the length is known, so cover both cases:
    - in `restore`, when the track is analysed, `position >= cue_out_secs()`
      becomes `cue_in_secs()`;
    - when the length is not known, the engine reports the source's real
      length on load. Check what the conductor does with a `LoadPaused`
      past the end, and whether `ApplyAnalysis` arrives before Play.
  - Pick the smallest fix that covers the reproduction, and record the
    choice as a `Ruling:`.
  - Write the failing test first, in `fp-model/tests/session.rs`:

```rust
#[test]
fn a_position_restored_at_the_cue_out_starts_at_the_cue_in() {
    // state with one analysed track (duration 30, cue_in 0.5, cue_out 29.5),
    // a session with the player current on it at position 29.8
    let (state, actions) = AppState::restore(/* … as the other tests here */);
    let load = actions.iter().find_map(|a| match a {
        EngineAction::LoadPaused { request, .. } => Some(request.from_secs),
        _ => None,
    }).unwrap();
    assert_eq!(load, 0.5);
    assert_eq!(state.players[0].transport, Transport::Paused);
}
```

  - Also fix `demo_session` so that it saves positions inside the files:
    use a fraction of the file length when known, or a small fixed time.
  - If the behaviour turns out correct (for example a real end of track
    restored Stopped), document why in the ledger and in
    `docs/user/troubleshooting.md`.
- [ ] **Step 5: Spec.** Update the main spec "Crash recovery" bullet: "A
  position at or past the cue-out is restored at the cue-in." Update the
  user docs on restarts.
- [ ] **Step 6: Gate and commit.**
  `fix(model): a position restored at the end starts at the cue-in`, or
  `docs: …` when there is no code fix.

---

### Task 11: The meter's alignment mark is two notches, not a bar across the signal

**Files:**
- Modify: `crates/fp-app/src/ui/widgets.rs` (`MeterLayout`, `meter_layout`, `vu`)
- Test: `crates/fp-app/tests/meter_view.rs`
- Modify: meters spec line ~100, `docs/user/players.md` line ~28

**Interfaces:**
- Produces: `MeterLayout::alignment_notches: [Rect; 2]`;
  `const ALIGNMENT_NOTCH: f32 = 3.0` (width).

- [ ] **Step 1: Failing test** in `meter_view.rs`:

```rust
#[test]
fn the_alignment_level_is_two_notches_at_the_outer_edges() {
    let c = MeterConfig::default();
    let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(METER_WIDTH, 120.0));
    let l = meter_layout(rect, &c, false);
    let a = l.lines.iter().find(|m| m.alignment).unwrap();
    let [left, right] = l.alignment_notches;
    for n in [left, right] {
        assert!(n.width() <= 3.0 + f32::EPSILON, "{n:?}");
        assert!((n.center().y - a.y).abs() < 0.5);
    }
    assert!((left.left() - l.bars[0].left()).abs() < 0.5);
    assert!((right.right() - l.bars[1].right()).abs() < 0.5);
}
```

- [ ] **Step 2: Run it.** Expected: FAIL to compile (no
  `alignment_notches`).
- [ ] **Step 3: Implement.**
  - `meter_layout` computes the notches at the alignment line's `y`,
    `ALIGNMENT_LINE_WIDTH` tall and `ALIGNMENT_NOTCH` wide, flush with bar 0's
    left edge and bar 1's right edge.
  - In `vu`, the alignment line is painted like the other reference lines
    (1 px, `NEUTRAL_400` at `REFERENCE_LINE_ALPHA`).
  - The two notches are painted in `NEUTRAL_400` (full alpha), above the
    bars.
  - Its label is unchanged.
- [ ] **Step 4: Run** `cargo test -p fp-app --test meter_view`. Expected:
  PASS (the existing `labels_never_overlap_and_the_alignment_line_stays`
  still holds).
- [ ] **Step 5: Docs.**
  - Meters spec: "**Alignment level:** two short 2 px notches at the outer
    edges of the bars, over a 1 px reference line, at `reference_dbfs`...".
  - `players.md`: "the short notches mark the alignment level (−18 dBFS)".
- [ ] **Step 6: Gate and commit.**
  `fix(ui): a discreet alignment mark on the meter`

---

### Task 12: Elapsed / total under the waveform

**Files:**
- Modify: `crates/fp-app/src/ui/player.rs` (`column`, `time_row`)
- Test: `crates/fp-app/tests/main_screen.rs`
  (`the_meter_and_fader_form_a_column_right_of_the_transport`)
- Modify: main spec rule 17 and §8.3 (the player layout),
  `docs/user/players.md`

- [ ] **Step 1: Change the test first.** In the existing test, replace the
  two time assertions:

```rust
    // Elapsed / total sits under the waveform, right-aligned.
    let time = h.get_by_label("00:00 / 00:00").rect();
    assert!(time.top() >= wave.bottom(), "{time:?} {wave:?}");
    assert!((time.right() - wave.right()).abs() <= 1.0, "{time:?} {wave:?}");
```

- [ ] **Step 2: Run it.** Expected: FAIL.
- [ ] **Step 3: Implement.**
  - In `column`, call `wave(...)` before `time_row(...)`.
  - In `time_row`, drop the `add_space(-4.0)` that pulled it up under the
    transport. Add a small negative space instead, so the row hugs the
    waveform (start at `-4.0`, judge by eye in Task 13), and update the doc
    comment ("under the waveform, right-aligned").
  - Check that `at_the_minimum_player_width_the_countdown_still_fits` and the
    other `main_screen` tests still pass.
- [ ] **Step 4: Run** `cargo test -p fp-app --test main_screen`. Expected:
  PASS.
- [ ] **Step 5: Docs.** Rule 17: "...and `elapsed / total` on the row under
  the waveform". Make the same change in spec §8.3 if it places the row,
  and in `players.md`.
- [ ] **Step 6: Gate and commit.**
  `feat(ui): elapsed / total under the waveform`

---

### Task 13: A realistic main screenshot at 1920×1080, made in Xvfb

**Files:**
- Modify: `crates/fp-app/examples/demo_session.rs`
- Modify: `docs/images/main-screen.png`
- Modify: `CLAUDE.md` (testing notes: resizing the window to 1920×1080)

- [ ] **Step 1: Demo scene.** In `demo_session`, after filling the lists:
  - player 1 is advanced 4 entries;
  - player 2 is advanced 2 entries;
  - player 3 is paused on its 3rd entry;
  - player 4 is stopped with a next.

  "Advanced" means `Command::Play` applied k+1 times (Play while playing
  starts the next, and marks the previous one played). The saved positions
  fall inside the files (Task 10). The session stays Paused on restore
  (rule 10); the capture starts P1 and P2 and fires one cart through the
  API.
- [ ] **Step 2: Music.** Use `test-music/` if it has files. Otherwise
  generate tagged WAV tones longer than 3 minutes, named like music, with
  the same titles and artists as the current image (`Papel mojado` / `Alba
  Serra`, …), into the scratchpad. Write the script there, not in the repo.
- [ ] **Step 3: Capture.**

```sh
D=<scratchpad>/shot
cargo build --release -p fp-app --example demo_session
rm -rf $D/home && FAUSTE_HOME=$D/home target/release/examples/demo_session $D/music
python3 - $D/home/config/config.json <<'EOF'
import json,sys; p=sys.argv[1]; c=json.load(open(p)); c["config"]["remote"]["http"]["enabled"]=True; json.dump(c,open(p,"w"),indent=2)
EOF
Xvfb :77 -screen 0 1920x1080x24 -nolisten tcp &
env -u WAYLAND_DISPLAY DISPLAY=:77 FAUSTE_HOME=$D/home target/release/fauste-player &
W=$(DISPLAY=:77 xwininfo -name "Fauste Player" | awk '/Window id/{print $4}')
DISPLAY=:77 xdotool windowmove $W 0 0 windowsize $W 1920 1080
curl -s http://127.0.0.1:7380/api/v1/players          # ids
curl -s -X POST http://127.0.0.1:7380/api/v1/players/<p1>/play
curl -s -X POST http://127.0.0.1:7380/api/v1/players/<p2>/play
curl -s -X POST http://127.0.0.1:7380/api/v1/carts/<cart>/fire
sleep 8
DISPLAY=:77 import -window $W docs/images/main-screen.png
file docs/images/main-screen.png    # 1920 x 1080
```

- [ ] **Step 4: Look at it.** Check each of these:
  - P1 and P2 on air, with played rows dimmed above the current one;
  - P3 paused and P4 stopped;
  - a cart counting down;
  - the notches on the meter (Task 11);
  - elapsed / total under the waveform (Task 12);
  - no white bar.

  Show the image to the maintainer before committing it.
- [ ] **Step 5: CLAUDE.md.** In the screenshot recipe, add the
  `xdotool windowmove … windowsize … 1920 1080` step and "the README image
  is 1920×1080".
- [ ] **Step 6: Gate and commit.**
  `docs: a 1920×1080 main screenshot made in Xvfb`

---

### Task 14: Each meter type shows the settings its standard defines

Settings > Meters shows floor, peak hold, alignment level, warning and
danger for every meter type, but most types ignore most of them:
- the scales of EBU, DIN, VU and K are fixed by their standards (M4);
- their red zones come from the standard, not from `warning_dbfs` and
  `danger_dbfs`;
- a K meter's alignment level is its own 0.

The doc comment of `MeterBallistics::DinPpm` also says "20 dB fall in
1.7 s", while the engine (`fp-engine/src/meter.rs:169`) uses 1.5 s, as in
DIN 45406.

**Files:**
- Modify: `crates/fp-model/src/config.rs` (`MeterBallistics` docs, a new
  `MeterSettings` applicability struct)
- Modify: `crates/fp-app/src/ui/settings/meters.rs` (show only what applies)
- Modify: `crates/fp-app/src/ui/widgets.rs` (`vu`: no peak hold where it does
  not apply)
- Test: `crates/fp-model/tests/meter.rs`, `crates/fp-app/tests/settings.rs`
  (or the file that tests Settings > Meters)
- Modify: meters spec M2/M3/M4, `docs/user/settings.md` (Meters table)

**Interfaces:**
- Produces:

```rust
/// Which `config.meter` fields a meter type uses (meters spec M3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeterSettings {
    pub custom_ballistics: bool, // attack_ms, release_db_per_sec
    pub floor: bool,             // floor_db
    pub peak_hold: bool,         // peak_hold_secs
    pub alignment: bool,         // reference_dbfs
    pub zones: bool,             // warning_dbfs, danger_dbfs
    pub true_peak: bool,
}
impl MeterBallistics { pub fn settings(self) -> MeterSettings }
```

- [ ] **Step 1: Check the standards** before writing the table. Record each
  source as a ledger line:
  - IEC 60268-18 (digital peak): instant rise, a 20 dB fall in 1.7 s, and an
    optional peak hold indicator;
  - IEC 60268-10 type IIb (EBU): 10 ms integration, 24 dB in 2.8 s; no hold;
    scale −12 … +12 around TEST;
  - IEC 60268-10 type I (DIN 45406): 5 ms integration, 20 dB in 1.5 s; no
    hold; scale −50 … +5;
  - IEC 60268-17 (VU): RMS, 300 ms rise and fall, no hold, 0 VU at the
    alignment level;
  - K-System (Katz): peak plus an RMS average section, 0 = −20/−14/−12 dBFS
    fixed by the type, a peak hold is common, and the peak section reads
    true peak.

  The proposed table (adjust to what the check finds, with a `Ruling:`
  for each change):

  | Type | custom | floor | hold | alignment | zones | true peak |
  |---|---|---|---|---|---|---|
  | DigitalPeak | – | ✓ | ✓ | ✓ | ✓ | ✓ |
  | Custom | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
  | EbuPpm | – | – | – | ✓ (TEST) | – | – |
  | DinPpm | – | – | – | ✓ (−9) | – | – |
  | Vu | – | – | – | ✓ (0 VU) | – | – |
  | K20/K14/K12 | – | – | ✓ | – (its own 0) | – | ✓ |

  The loudness readout and target are independent of the type (EBU R128),
  so they are always shown.

- [ ] **Step 2: Failing model test**, one assertion per row:

```rust
#[test]
fn each_meter_type_uses_the_settings_its_standard_defines() {
    use fp_model::{MeterBallistics as B, MeterSettings as S};
    let all = S { custom_ballistics: false, floor: true, peak_hold: true, alignment: true, zones: true, true_peak: true };
    assert_eq!(B::DigitalPeak.settings(), all);
    assert_eq!(B::Custom.settings(), S { custom_ballistics: true, ..all });
    let ppm = S { custom_ballistics: false, floor: false, peak_hold: false, alignment: true, zones: false, true_peak: false };
    for b in [B::EbuPpm, B::DinPpm, B::Vu] { assert_eq!(b.settings(), ppm, "{b:?}"); }
    let k = S { custom_ballistics: false, floor: false, peak_hold: true, alignment: false, zones: false, true_peak: true };
    for b in [B::K20, B::K14, B::K12] { assert_eq!(b.settings(), k, "{b:?}"); }
}
```

- [ ] **Step 3: Run it.** `cargo test -p fp-model --test meter each_meter_type`.
  Expected: FAIL to compile.
- [ ] **Step 4: Implement** `MeterSettings` and `settings()` in
  `config.rs`, export them from `lib.rs`, and correct the `DinPpm` doc
  comment to "20 dB fall in 1.5 s (DIN 45406)".
  - The stored values stay as they are: lenient loading, and a value is
    kept for when the operator switches back.
  - `validate` still checks every field.
- [ ] **Step 5: Failing UI test.** In the Settings tests, select "VU" (check
  the `meter-vu` string) and assert that the "Warning", "Danger", "Floor"
  and "Peak hold" rows are absent (`query_by_label(..).is_none()`) and that
  "Alignment level" is present. Select "K-20": "Alignment level" is absent
  and "Peak hold" is present. Select "Digital peak": all of them are
  present.
- [ ] **Step 6: Run it.** Expected: FAIL (every row is shown).
- [ ] **Step 7: Implement.**
  - In `meters.rs`, read `let uses = m.ballistics.settings();` and wrap each
    row in its flag. `custom_ballistics` replaces the `== Custom` check.
  - In `widgets::vu`, draw the peak hold only when
    `c.ballistics.settings().peak_hold`.
  - The engine already ignores the fields its type does not use. Check that
    `true_peak` on a PPM or VU does not change the reading; if it does,
    gate it in the engine's meter setup and add a test in
    `fp-engine/tests`.
- [ ] **Step 8: Run** `cargo test -p fp-app` and
  `cargo test -p fp-engine meter`. Expected: PASS.
- [ ] **Step 9: Docs.**
  - Meters spec M3: add a "Used by" column (the table above). M2 gets the
    DIN fall time.
  - `docs/user/settings.md`: "Settings shows only what the chosen meter
    uses".
- [ ] **Step 10: Gate and commit.**
  `fix(ui): each meter type shows only the settings its standard defines`

The screenshot (Task 13) is taken after this task, so move Task 13 last
when executing.

---

### Task 15: Long recordings (1 h, 4 h and more) play, seek and analyse correctly

A whole programme recording must behave like a song: decode, analyse, show
its waveform, seek, count down and save its position. Nothing may overflow,
lose precision or hold the whole file in memory.

**Part A, audit (read, then measure).** Record each finding in the ledger
as `fine` (with the reason), or as a bug that gets a failing test. Check:
- **Time arithmetic:**
  - frame counters and positions stored as `u32`, `f32` or `usize` on 32-bit
    targets;
  - `as f32` on seconds or frames in `fp-engine`, `fp-decode`, `fp-analysis`
    and the UI (`grep -n "as f32\|as u32\|: f32" …`). Note that an `f32`
    holds whole frames exactly only up to 2^24 (6.3 min at 44.1 kHz);
  - `u32` frames overflow at 4h 8m at 288 kHz, or at 6h 12m at 192 kHz;
- **Analysis memory:** does any step keep all the decoded samples (marker
  detection, loudness, RMS waveform)? At 4 h and 44.1 kHz stereo `f32`
  that is about 5 GB. It must stream, with bounded buffers;
- **Waveform:** the number of buckets at `peak_bucket_secs`. For 4 h that
  is 288 000 buckets. Check the cache entry size against `Limits`, the
  `GET /tracks/{id}/peaks` body, and the drawing cost of `wave_view` per
  frame (it must reduce to the pixels shown);
- **Analysis cache:** any size cap in `Limits` that rejects a long track's
  entry;
- **Seeking:** a seek near the end of a 4 h file in each decoder (FLAC, MP3,
  WAV/RF64, Opus, WavPack): it must be accurate and must not decode from
  the start;
- **Display:** countdowns, `elapsed / total`, playlist totals and the
  remote DTOs past 1 h, 10 h and 100 h (there is
  `an_hour_long_countdown_fits_…`; add 10 h);
- **Session:** a position at 3 h 59 m is saved and restored exactly.

**Part B, measure on real files** (local only, never in CI). Generate the
files in the scratchpad with `ffmpeg`:
- `-f lavfi -i "sine=f=440:d=14400" -ac 2` to FLAC;
- MP3, if `libmp3lame` is there;
- Opus;
- a 1 h WAV;
- an RF64 WAV past 4 GB, if the decoder claims to support it.

Then:
- run the analysis and record its time and peak RSS (`/usr/bin/time -v`);
- load the files in the app under Xvfb, play, seek to 3:59:00 through the
  API, and capture the screen.

Add an `#[ignore]` test in `fp-analysis/tests/real_music.rs`'s style
(`long_files`) that analyses whatever long files a folder holds, and
asserts the duration within 0.1 s and the cue-out near the end.

**Part C, fix.** Each bug found gets:
- a failing test that runs in CI. Use synthetic sources and direct
  arithmetic (for example the frame ↔ seconds conversion at 4 h at
  192 kHz, or a stub decoder that reports a 10 h length), never a real
  long file;
- then the fix;
- then one commit per fix: `fix(<crate>): …`.

What cannot be tested in CI gets a `Ruling:` with the measured figures.

**Docs.** `README.md` features and `docs/user/` mention that programme-length
recordings are supported, and give the limits found (for example the
analysis time per hour).

---

### Task 16: After an update, re-analysing old tracks is the operator's choice

**Known:** tracks analysed by an older `ANALYSIS_VERSION` are re-analysed
automatically and silently (`services.rs`, `outdated`). The maintainer saw
errors on tracks after an update.

**Step 1: debug first** (superpowers:systematic-debugging). Reproduce:
- build the previous release tag in a worktree;
- analyse a folder with it under a scratch `FAUSTE_HOME`;
- start this branch's build on the same home.

Then record what "error" shows (Unreadable or Missing state, waveform,
markers, log lines) and why. If it is a bug (for example a cache key
mismatch marking tracks Unreadable, or old analysis data that the new code
misreads), fix it test-first in its own commit.

**Step 2: the notice.** The design:
- Outdated tracks keep their old analysis, which is still usable, and are
  **no longer re-analysed automatically**. Re-analysing a whole library
  takes CPU and disk on an on-air machine. Tracks never analysed are still
  analysed at once, as now.
- At start, when there are outdated tracks, the UI shows a modal:
  - title: "Some tracks need a new analysis";
  - body: "<N> tracks were analysed by an earlier version of Fauste Player.
    Analysing them again updates their markers and waveforms. It uses the
    processor for a while; playback is not affected. You can also do it
    later in Settings > Analysis.";
  - buttons: **Analyse now** and **Later**.

  "Later" asks again at the next start.
- Settings > Analysis gains "Analyse outdated tracks (N)", disabled at 0.
- Nothing about it blocks the UI. The count is computed from the snapshot
  (`library` and `analysis_version`). The request goes to the services
  thread as `ServiceRequest::AnalyseOutdated`.

**Files:**
- `crates/fp-app/src/services.rs`: `outdated` no longer triggers analysis
  unless an `analyse_outdated` flag is set by the request;
- the UI: a startup modal in `ui/app.rs` (or its own `ui/notice.rs`), and
  the button in Settings > Analysis;
- both locales;
- the main spec (analysis section);
- `docs/user/` (updating, analysis);
- `docs/technical/` (the services thread).

**Tests:**
- `services` (`tests/services.rs`): an outdated track is not submitted
  until `AnalyseOutdated` is received, and then it is submitted once. A
  track that was never analysed is still submitted at once;
- UI (`egui_kittest`): with an outdated track in the state, the modal shows
  the count. "Analyse now" sends the request and closes it; "Later" closes
  it without sending. With no outdated tracks there is no modal;
- the pure count: `outdated_tracks(&AppState) -> usize`, in `fp-app` next to
  the services, or in `fp-model` if `ANALYSIS_VERSION` can be passed in.

Commit: `feat(app): ask before analysing tracks of an older version`.

---

## After the last task

1. Run the whole gate.
2. Get a final review with a fresh reviewer on the most capable model
   (`superpowers:requesting-code-review`). Fix Critical and Important
   findings test-first; Minor ones go in the ledger.
3. Open PR `fix(remote): review follow-ups, restore at the end, meter and
   screenshot` with the template checklist.
4. Run `gh pr checks --watch` on the three OSes, then
   `gh pr merge --merge --delete-branch`, and update `master`.
