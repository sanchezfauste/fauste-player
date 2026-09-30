# Feedback Plan 3 — Restart, Previous and Button Availability Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Restart and Previous to every player (F3), backed by a per-player
history, and dim every transport button whose action makes no sense now.

**Architecture:**
- `fp-model`:
  - new data: `PlayerState.history` and `players.history_len`;
  - new commands: `Restart` and `Previous`;
  - new rules R23–R25 in `reducer.rs`;
  - a pure `availability` function (R28);
  - `command_available` for shortcuts and, later, MIDI;
  - two new `ShortcutAction`s.
- `fp-store` persists the history through the existing `PlayerSession`.
- `fp-app`:
  - draws two new icons and a 3×2 transport grid;
  - enables every transport button from `availability`;
  - filters shortcuts through `command_available`.
- The engine needs no change: Previous reuses `EngineAction::Crossfade` and
  Restart reuses `EngineAction::Seek`.

**Tech Stack:** Rust 2024, egui 0.36, egui_kittest, serde.

**Spec:** [`docs/superpowers/specs/2026-09-30-operator-feedback-design.md`](../specs/2026-09-30-operator-feedback-design.md) §2.1, §2.2 (R23–R25, R28), §3.2 (transport grid).
Roadmap: plan 3.

## Global Constraints

- English code and docs; UI strings in `en-US` and `es-ES`.
- Every operator value is a `Config` field with a default, a range in
  `Config::validate` and lenient loading: `players.history_len`, default
  **50**, range **0–1000**.
- Behaviour lives in `fp-model` (pure functions, one test per rule). The UI
  only displays and sends commands.
- No `unwrap`/`expect`/`panic` outside tests.
- Commit only with fmt, clippy `-D warnings` and `cargo test --workspace` green.
- Conventional Commits (`model`, `store`, `ui`, `docs`), trailer
  `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Branch `feat/restart-previous` from an up-to-date `master`.
- Shortcuts `RestartPlayer(n)` and `PreviousPlayer(n)` have **no default key**.

## Review Focus

- **History with removed or unplayable entries.** Previous skips them and
  discards them; with nothing playable left it does nothing (Task 3 test).
- **`history_len = 0`.** Nothing is recorded, and Previous is unavailable
  (Task 1 and Task 4 tests).
- **A `session.json` with a broken `history`** (a string, bad ids) loads the
  rest of the session and an empty history (Task 1 test).
- **Previous during a crossfade or a fade stop, and while paused.** Nothing
  happens and the button is dimmed (Task 3 and Task 4 tests).
- **The minimum player width (380 px) with the wider grid.** The countdown
  shrinks to fit instead of overlapping the meter (Task 6 test).

---

### Task 1: History (R25)

**Files:**
- Modify: `crates/fp-model/src/player.rs` (`PlayerState.history`)
- Modify: `crates/fp-model/src/config.rs` (`PlayersConfig.history_len`, validation, tests)
- Modify: `crates/fp-model/src/reducer.rs` (`advance_to`, `stop_player`, new `push_history`)
- Modify: `crates/fp-model/src/session.rs` (`PlayerSession.history`, lenient; restore filters it)
- Test: `crates/fp-model/tests/transport.rs`, `crates/fp-model/tests/session.rs`

**Interfaces:**
- Produces:
  - `pub history: Vec<EntryId>` on `PlayerState` (oldest first; the last element is the most recent);
  - `PlayersConfig.history_len: usize`;
  - `PlayerSession.history: Vec<EntryId>`;
  - `pub(crate) fn push_history(state: &mut AppState, i: usize, entry: EntryId)`.

- [ ] **Step 1: Write the failing tests.** Look at the helpers at the top of
  `crates/fp-model/tests/transport.rs` (the `common` module builds a state with
  tracks; reuse whatever `setup`/`state_with` helper the file already uses to
  get a player and three analysed entries A, B, C). Add:

```rust
#[test]
fn r25_every_advance_records_the_entry_that_was_left() {
    // Play A, Next into B (crossfade), then B's transition into C.
    let (mut s, p, [a, b, c]) = three_entries();
    apply(&mut s, Command::Play(p)).unwrap();
    assert!(s.player(p).unwrap().history.is_empty());
    apply(&mut s, Command::Play(p)).unwrap();
    on_event(&mut s, EngineEvent::FadeCompleted { player: p });
    assert_eq!(s.player(p).unwrap().history, vec![a]);
    on_event(&mut s, EngineEvent::TransitionStarted { player: p, entry: c });
    assert_eq!(s.player(p).unwrap().history, vec![a, b]);
}

#[test]
fn r25_a_stop_records_the_entry_that_stopped() {
    let (mut s, p, [a, _, _]) = three_entries();
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::Stop(p)).unwrap();
    assert_eq!(s.player(p).unwrap().history, vec![a]);
}

#[test]
fn r25_the_history_keeps_only_the_configured_length() {
    let (mut s, p, [a, b, c]) = three_entries();
    s.config.players.history_len = 2;
    for _ in 0..3 {
        apply(&mut s, Command::Play(p)).unwrap();
        on_event(&mut s, EngineEvent::FadeCompleted { player: p });
    }
    // A, B and C were left in turn (C by the third Play, into the wrap-around
    // or a stop): only the last two remain.
    let h = &s.player(p).unwrap().history;
    assert_eq!(h.len(), 2, "{h:?}");
    assert_eq!(h.first(), Some(&b));
    let _ = (a, c);
}

#[test]
fn r25_a_zero_length_history_records_nothing() {
    let (mut s, p, _) = three_entries();
    s.config.players.history_len = 0;
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    assert!(s.player(p).unwrap().history.is_empty());
}
```

  Write `three_entries()` in the test file if no helper fits: a state from
  `common` with one player and three playable entries, returning
  `(AppState, PlayerId, [EntryId; 3])`.

  In `crates/fp-model/tests/session.rs` add:

```rust
#[test]
fn the_history_survives_a_session_round_trip_and_drops_gone_entries() {
    // Build a state whose player has history [A, B], save sessions, delete B
    // from its playlist, restore: the history is [A].
}

#[test]
fn a_broken_history_loads_as_empty() {
    let json = r#"{"id":1,"playlist":1,"history":"oops"}"#;
    let s: fp_model::PlayerSession = serde_json::from_str(json).unwrap();
    assert!(s.history.is_empty());
    let json = r#"{"id":1,"playlist":1,"history":[3,"x",4]}"#;
    let s: fp_model::PlayerSession = serde_json::from_str(json).unwrap();
    assert!(s.history.is_empty());
}
```

  Fill in the round-trip test body with the same building blocks the file's
  existing round-trip tests use (`sessions()`, `AppState::restore`), asserting
  `restored.players[0].history == vec![a]`. Check the id JSON shape against an
  existing `PlayerSession` fixture in the file and adjust the literals (ids may
  serialise as plain numbers or as newtypes).

  In `config.rs` tests add:

```rust
    #[test]
    fn the_history_length_is_validated() {
        let mut c = Config::default();
        assert_eq!(c.players.history_len, 50);
        c.players.history_len = 5000;
        assert!(!c.validate().is_empty());
        assert_eq!(c.players.history_len, 1000);
    }
```

  (Match the shape of the neighbouring validation tests: `validate` clamps and
  returns warnings.)

- [ ] **Step 2: Run** `cargo test -p fp-model`. Expected: compile errors (`history`, `history_len`).

- [ ] **Step 3: Implement.**
  - `PlayerState`: `pub history: Vec<EntryId>`, documented "Entries this player left, oldest first (R25); Previous pops from the end." Initialise empty in `new`.
  - `PlayersConfig`: `pub history_len: usize` (doc: "How many entries Previous can go back (R25); 0 disables Previous."), default 50; in `validate`, `clamp_to(&mut p.history_len, 0, 1000, "players.history_len", &mut w)` next to the other player fields (follow the existing helper's signature).
  - `reducer.rs`:

```rust
/// R25: records that the player left `entry`, keeping at most
/// `players.history_len` entries.
pub(crate) fn push_history(state: &mut AppState, i: usize, entry: EntryId) {
    let cap = state.config.players.history_len;
    let history = &mut state.players[i].history;
    history.push(entry);
    let excess = history.len().saturating_sub(cap);
    history.drain(..excess);
}
```

    Call it in `advance_to` right after `mark_played(current, …)`, and in
    `stop_player` right after its `mark_played`. `advance_to` gains a
    `record: bool` parameter (true everywhere today); Previous (Task 3) passes
    false. Keep `advance` passing true.
  - `session.rs`: `PlayerSession` gets

```rust
    #[serde(default, deserialize_with = "lenient_history")]
    pub history: Vec<EntryId>,
```

    with

```rust
/// A history that does not parse loads as empty: it only feeds Previous.
fn lenient_history<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<EntryId>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Lenient {
        Entries(Vec<EntryId>),
        Other(serde::de::IgnoredAny),
    }
    Ok(match Lenient::deserialize(d)? {
        Lenient::Entries(e) => e,
        Lenient::Other(_) => Vec::new(),
    })
}
```

    `sessions()` writes `history: p.history.clone()`; `restore_player` keeps
    the entries that still exist (`state.playlists.entry(e).is_some()`), and
    only the last `history_len` of them.

- [ ] **Step 4: Run** `cargo test -p fp-model && cargo test -p fp-store`. Expected: PASS. Fix any struct literal of `PlayerSession` or `PlayersConfig` in other crates' tests (`history: Vec::new()`, `..Default::default()`).

- [ ] **Step 5: Commit** — `feat(model): record each player's history` (body: Previous needs the entries a player left, R25).

---

### Task 2: Restart (R23)

**Files:** `crates/fp-model/src/command.rs`, `crates/fp-model/src/reducer.rs`; test `crates/fp-model/tests/transport.rs`.

**Interfaces:** Produces `Command::Restart(PlayerId)`.

- [ ] **Step 1: Failing tests:**

```rust
#[test]
fn r23_restart_seeks_to_cue_in_while_playing() {
    let (mut s, p, [a, _, _]) = three_entries();
    apply(&mut s, Command::Play(p)).unwrap();
    let cue_in = s.request_from_cue_in(a).unwrap().from_secs;
    let out = apply(&mut s, Command::Restart(p)).unwrap();
    assert_eq!(out, vec![EngineAction::Seek { player: p, secs: cue_in }]);
    assert_eq!(s.player(p).unwrap().transport, Transport::Playing);
}

#[test]
fn r23_a_paused_player_restarts_paused() {
    let (mut s, p, _) = three_entries();
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::Pause(p)).unwrap();
    let out = apply(&mut s, Command::Restart(p)).unwrap();
    assert!(matches!(out.as_slice(), [EngineAction::Seek { .. }]));
    assert_eq!(s.player(p).unwrap().transport, Transport::Paused);
}

#[test]
fn r23_restart_does_nothing_when_stopped() {
    let (mut s, p, _) = three_entries();
    assert!(apply(&mut s, Command::Restart(p)).unwrap().is_empty());
}
```

  (If `request_from_cue_in` is not public, compute `cue_in` from the entry's
  markers the way other tests in the file do.)

- [ ] **Step 2: Run** `cargo test -p fp-model --test transport r23`. Expected: compile error.

- [ ] **Step 3: Implement** — add the variant (doc: "R23: back to the current entry's cue-in; a paused player stays paused.") and in `apply`:

```rust
        Command::Restart(id) => {
            let i = state.player_index(id)?;
            let player = &state.players[i];
            if player.transport != Transport::Stopped
                && let Some(request) = player.current.and_then(|c| state.request_from_cue_in(c))
            {
                out.push(EngineAction::Seek {
                    player: id,
                    secs: request.from_secs,
                });
            }
        }
```

- [ ] **Step 4: Run** `cargo test -p fp-model`. Expected: PASS (add `Restart` to any exhaustive `match` on `Command` elsewhere; `cargo build --workspace` finds them).

- [ ] **Step 5: Commit** — `feat(model): restart the current entry`.

---

### Task 3: Previous (R24)

**Files:** `crates/fp-model/src/command.rs`, `crates/fp-model/src/reducer.rs`; test `crates/fp-model/tests/transport.rs`.

**Interfaces:** Consumes `push_history`/`advance_to(…, record)` (Task 1). Produces `Command::Previous(PlayerId)`.

- [ ] **Step 1: Failing tests:**

```rust
/// A plays, then B: history [A], current B.
fn playing_b() -> (AppState, PlayerId, [EntryId; 3]) {
    let (mut s, p, e) = three_entries();
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    on_event(&mut s, EngineEvent::FadeCompleted { player: p });
    (s, p, e)
}

#[test]
fn r24_previous_crossfades_back_and_queues_the_left_entry() {
    let (mut s, p, [a, b, _]) = playing_b();
    let out = apply(&mut s, Command::Previous(p)).unwrap();
    let fade_ms = s.config.players.fade_ms;
    assert!(out.iter().any(|x| matches!(x,
        EngineAction::Crossfade { player, request, fade_ms: f }
            if *player == p && request.entry == a && *f == fade_ms)), "{out:?}");
    let player = s.player(p).unwrap();
    assert_eq!(player.current, Some(a));
    assert_eq!(player.next, Some(b));
    assert!(player.next_explicit);
    assert!(player.fading);
    assert!(player.history.is_empty(), "B is not pushed");
}

#[test]
fn r24_repeated_previous_keeps_going_back() {
    let (mut s, p, [a, b, c]) = three_entries();
    for _ in 0..2 {
        apply(&mut s, Command::Play(p)).unwrap();
        on_event(&mut s, EngineEvent::FadeCompleted { player: p });
    }
    apply(&mut s, Command::Play(p)).unwrap(); // C current? (A, B left)
    on_event(&mut s, EngineEvent::FadeCompleted { player: p });
    assert_eq!(s.player(p).unwrap().current, Some(c));
    apply(&mut s, Command::Previous(p)).unwrap();
    on_event(&mut s, EngineEvent::FadeCompleted { player: p });
    assert_eq!(s.player(p).unwrap().current, Some(b));
    apply(&mut s, Command::Previous(p)).unwrap();
    on_event(&mut s, EngineEvent::FadeCompleted { player: p });
    assert_eq!(s.player(p).unwrap().current, Some(a));
}

#[test]
fn r24_gone_entries_are_skipped_and_discarded() {
    let (mut s, p, [a, b, _]) = three_entries();
    // History [A, B] with C playing; then A stays and B is removed.
    // (Build it as in the previous test.)
    // Previous goes to A, and B is gone from the history.
}

#[test]
fn r24_previous_does_nothing_without_history_or_while_fading_paused_or_stopped() {
    let (mut s, p, _) = three_entries();
    assert!(apply(&mut s, Command::Previous(p)).unwrap().is_empty(), "stopped");
    apply(&mut s, Command::Play(p)).unwrap();
    assert!(apply(&mut s, Command::Previous(p)).unwrap().is_empty(), "no history");
    let (mut s, p, _) = playing_b();
    apply(&mut s, Command::Pause(p)).unwrap();
    assert!(apply(&mut s, Command::Previous(p)).unwrap().is_empty(), "paused");
    let (mut s, p, _) = playing_b();
    apply(&mut s, Command::FadeStop(p)).unwrap();
    assert!(apply(&mut s, Command::Previous(p)).unwrap().is_empty(), "fading");
}
```

  Complete the skipped-entries test with `Command::RemoveEntry(b)` and the
  assertions `current == Some(a)` and `history.is_empty()`. Fix the
  `repeated_previous` setup against the real advance behaviour (three
  entries A→B→C: two Plays after the first leave A and B).

- [ ] **Step 2: Run** `cargo test -p fp-model --test transport r24`. Expected: compile error.

- [ ] **Step 3: Implement** — the variant (doc: "R24: crossfade back to the last entry this player left.") and:

```rust
fn previous(
    state: &mut AppState,
    id: PlayerId,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let i = state.player_index(id)?;
    let player = &state.players[i];
    if player.transport != Transport::Playing || player.fading {
        return Ok(());
    }
    let left = player.current;
    // Pop until an entry that still exists and can play.
    let target = loop {
        let Some(entry) = state.players[i].history.pop() else {
            return Ok(());
        };
        if Some(entry) != left && state.request_from_cue_in(entry).is_some() {
            break entry;
        }
    };
    let fade_ms = state.config.players.fade_ms;
    if let Some(request) = advance_to(state, i, target, false) {
        let player = &mut state.players[i];
        player.fading = true;
        if let Some(left) = left {
            player.next = Some(left);
            player.next_explicit = true;
        }
        out.push(EngineAction::Crossfade {
            player: id,
            request,
            fade_ms,
        });
    }
    Ok(())
}
```

  (`apply` then runs `reconcile`, which preloads the new next.)

- [ ] **Step 4: Run** `cargo test -p fp-model`. Expected: PASS.

- [ ] **Step 5: Commit** — `feat(model): go back to the previous entry`.

---

### Task 4: Availability (R28)

**Files:** Create `crates/fp-model/src/availability.rs`; modify `lib.rs` (re-export); test `crates/fp-model/tests/availability.rs` (new).

**Interfaces:** Produces

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Availability {
    pub play: bool,
    pub pause: bool,
    pub stop: bool,
    pub fade_stop: bool,
    pub restart: bool,
    pub previous: bool,
    pub stop_after_current: bool,
    pub cue: bool,
}
pub fn availability(state: &AppState, player: PlayerId) -> Availability;
/// False for a transport command R28 marks unavailable; true otherwise.
pub fn command_available(state: &AppState, command: &Command) -> bool;
```

- [ ] **Step 1: Failing tests** — one per row, true and false:

```rust
#[test]
fn a_stopped_player_with_a_next_can_only_play_cue_and_arm_stop_after() {
    let (s, p, _) = three_entries();
    let a = availability(&s, p);
    assert!(a.play && a.cue && a.stop_after_current);
    assert!(!a.pause && !a.stop && !a.fade_stop && !a.restart && !a.previous);
}

#[test]
fn a_playing_player_can_do_everything_but_previous_without_history() { /* … */ }

#[test]
fn previous_needs_a_playable_history_entry() { /* playing_b(): true; after RemoveEntry(a): false */ }

#[test]
fn during_a_crossfade_play_pause_and_previous_are_unavailable() { /* Play twice, no FadeCompleted */ }

#[test]
fn during_a_fade_stop_fade_stop_is_unavailable_but_stop_is_not() { /* … */ }

#[test]
fn a_paused_player_can_resume_restart_and_stop_but_not_fade_or_go_back() { /* … */ }

#[test]
fn stop_after_current_needs_continuous_mode() { /* SetMode Single → false */ }

#[test]
fn an_empty_playlist_offers_nothing() { /* state with no entries: all false */ }

#[test]
fn command_available_filters_transport_commands_only() {
    let (s, p, _) = three_entries();
    assert!(!command_available(&s, &Command::Stop(p)));
    assert!(command_available(&s, &Command::Play(p)));
    assert!(command_available(&s, &Command::SetVolume(p, 0.5)));
}
```

  Write each body with explicit asserts on every field involved (the rows of
  the spec table).

- [ ] **Step 2: Run** `cargo test -p fp-model --test availability`. Expected: compile error.

- [ ] **Step 3: Implement** — straight from the spec table:

```rust
pub fn availability(state: &AppState, player: PlayerId) -> Availability {
    let Ok(p) = state.player(player) else {
        return Availability::default();
    };
    let playing = p.transport == Transport::Playing;
    let paused = p.transport == Transport::Paused;
    let has_previous = p
        .history
        .iter()
        .any(|e| Some(*e) != p.current && state.request_from_cue_in(*e).is_some());
    Availability {
        play: (paused && p.current.is_some()) || (p.next.is_some() && !p.fading),
        pause: (playing && !p.fading) || paused,
        stop: p.current.is_some(),
        fade_stop: playing && !p.fade_stopping(),
        restart: p.current.is_some() && p.transport != Transport::Stopped,
        previous: playing && !p.fading && has_previous,
        stop_after_current: p.mode == PlayMode::Continuous,
        cue: p.next.is_some() || p.cue.is_some(),
    }
}

pub fn command_available(state: &AppState, command: &Command) -> bool {
    let a = |id: &PlayerId| availability(state, *id);
    match command {
        Command::Play(id) => a(id).play,
        Command::Pause(id) => a(id).pause,
        Command::Stop(id) => a(id).stop,
        Command::FadeStop(id) => a(id).fade_stop,
        Command::Restart(id) => a(id).restart,
        Command::Previous(id) => a(id).previous,
        Command::ToggleStopAfterCurrent(id) => a(id).stop_after_current,
        Command::ToggleCue(id) => a(id).cue,
        _ => true,
    }
}
```

  (`request_from_cue_in` must be callable from this module: `pub(crate)` is
  enough.)

- [ ] **Step 4: Run** `cargo test -p fp-model`. Expected: PASS.

- [ ] **Step 5: Commit** — `feat(model): say which transport actions are available`.

---

### Task 5: Shortcuts

**Files:** `crates/fp-model/src/shortcuts.rs`, `crates/fp-app/src/ui/settings/keys.rs`, `crates/fp-app/src/ui/app.rs` (`shortcut_command`, the caller at line ~537), both locales; tests `crates/fp-model/tests/shortcuts.rs`, `crates/fp-app/tests/main_screen.rs`.

**Interfaces:** Produces `ShortcutAction::RestartPlayer(u16)`, `ShortcutAction::PreviousPlayer(u16)`.

- [ ] **Step 1: Failing tests.**
  - `shortcuts.rs`: the new actions report `position() == Some(n)`, round-trip through serde, and `default_shortcuts()` binds neither.
  - `main_screen.rs`: with a config whose shortcuts bind `RestartPlayer(1)` to `R`, a playing player gets `Command::Restart` on `R`; a stopped player gets nothing (unavailable). And the bare `Stop` shortcut on a stopped player sends nothing any more.

- [ ] **Step 2: Run** them. Expected: compile errors / FAIL.

- [ ] **Step 3: Implement.**
  - The two variants in `ShortcutAction`, and in `position()`.
  - `keys.rs`: names `shortcut-restart` = "Restart player { $n }" / "Reiniciar reproductor { $n }", `shortcut-previous` = "Previous on player { $n }" / "Anterior en el reproductor { $n }"; list both in `actions` after `PlayPlayer`.
  - `app.rs`: map them to `Command::Restart` / `Command::Previous`; where the command is sent (line ~537), send it only if `fp_model::command_available(state, &command)`.

- [ ] **Step 4: Run** `cargo test --workspace`. Expected: PASS (update any existing test that relied on a shortcut acting while unavailable; ledger each).

- [ ] **Step 5: Commit** — `feat(ui): restart and previous shortcuts, ignored when unavailable`.

---

### Task 6: Transport grid, icons and dimming

**Files:** `crates/fp-app/src/ui/icons.rs` (`restart`, `previous`), `crates/fp-app/src/ui/player.rs` (`transport`, the header CUE button), both locales (`tip-restart`, `tip-previous`); tests `crates/fp-app/tests/theme.rs`, `crates/fp-app/tests/main_screen.rs`.

**Interfaces:** Consumes `availability` (Task 4). Produces `icons::restart(rect, color) -> Vec<Shape>` (a bar then one left triangle) and `icons::previous(rect, color) -> Vec<Shape>` (a bar then two left triangles).

- [ ] **Step 1: Failing tests.**
  - `theme.rs`: `restart` has 2 shapes, `previous` has 3; in both the bar is left of the triangles and everything stays inside the rect (extend `drawn_icons_stay_inside_their_rectangle` too).
  - `main_screen.rs`:

```rust
#[test]
fn the_grid_has_previous_and_restart_in_its_first_column() {
    let (h, _) = harness(state(1, 3));
    let previous = h.get_by_label("Previous track").rect();
    let restart = h.get_by_label("Restart the track").rect();
    let stop = h.get_by_label("Stop").rect();
    let fade = h.get_by_label("Fade out and stop").rect();
    assert!(previous.right() < stop.left() && (previous.top() - stop.top()).abs() < 0.5);
    assert!(restart.right() < fade.left() && (restart.top() - fade.top()).abs() < 0.5);
}

#[test]
fn unavailable_buttons_are_dimmed_and_inert() {
    let (mut h, fake) = harness(state(1, 3));
    for label in ["Previous track", "Restart the track", "Stop", "Pause", "Fade out and stop"] {
        let node = h.get_by_label(label);
        assert!(node.is_disabled(), "{label}");
        node.click();
    }
    h.run_steps(2);
    assert!(sent(&fake).is_empty());
}

#[test]
fn restart_and_previous_send_their_commands_when_available() {
    // A state whose player is Playing with a history (build it through the
    // Fake: apply Play twice and FadeCompleted on its model, as the other
    // main_screen tests that need a playing player do).
}
```

  Check the existing tooltip texts in `main.ftl` (`tip-stop`, `tip-pause`,
  `tip-fade-stop`) and use them verbatim in the test; the new ones are
  `tip-previous = Previous track` / `Pista anterior` and `tip-restart =
  Restart the track` / `Reiniciar la pista`.
  - A test at 380 px (`support::harness_sized`) that the countdown still ends
    left of the meter.

- [ ] **Step 2: Run** them. Expected: FAIL / compile errors.

- [ ] **Step 3: Implement.**
  - Icons (same style as `stop_after`):

```rust
/// Restart: a bar and one triangle pointing left (back to the start).
pub fn restart(rect: Rect, color: Color32) -> Vec<Shape> {
    let r = rect.shrink(rect.height() * 0.1);
    let side = (r.height() * 0.8).min(r.width() / 1.25);
    let bar = side * 0.2;
    let left = r.center().x - (bar + side * 0.8) / 2.0;
    let top = r.center().y - side / 2.0;
    let tri = |x: f32| vec![pos2(x + side * 0.8, top), pos2(x, top + side / 2.0), pos2(x + side * 0.8, top + side)];
    vec![
        Shape::rect_filled(Rect::from_min_size(pos2(left, top), egui::vec2(bar, side)), 0.0, color),
        Shape::convex_polygon(tri(left + bar), color, Stroke::NONE),
    ]
}

/// Previous: a bar and two triangles pointing left, as on a CD player.
pub fn previous(rect: Rect, color: Color32) -> Vec<Shape> { /* bar + two tri, each 0.6 side wide, fitting r.width() */ }
```

  - Transport: a 3×2 grid — row 1 Previous, Stop, Pause; row 2 Restart, Fade stop, Stop after current. Every tile's `enabled` comes from `let a = fp_model::availability(scene.state, id);` (`a.play` for the Play tile, `a.cue` for the header CUE, `a.stop_after_current` for Stop after — which replaces `!single`).
  - Countdown: measure the big text; when `main_size.x + tenths_size.x` exceeds `ui.available_width()`, lay it out again at `38.0 * available / needed` (never below 24.0), scaling the tenths the same way.

- [ ] **Step 4: Run** `cargo test -p fp-app`. Expected: PASS. Release build, screenshot by eye.

- [ ] **Step 5: Commit** — `feat(ui): restart and previous buttons, dimmed when unavailable`.

---

### Task 7: Docs, review, pull request

- Main spec §3: rules R23, R24, R25, R28 (text from the feedback spec §2.2), §8.3 transport (3×2 grid, dimming), §5 config table (`players.history_len`).
- `docs/user/players.md` (buttons table: Previous, Restart; dimmed buttons), `docs/user/keyboard.md` (the two new actions, no default key), `docs/user/settings.md` if the players section lists config values, `docs/technical/persistence.md` (`history` in `session.json`, `players.history_len` row), `docs/technical/ui.md` (availability).
- Full verification, final review (fresh reviewer, most capable model), PR `feat: restart and previous with button availability`, CI, merge.
