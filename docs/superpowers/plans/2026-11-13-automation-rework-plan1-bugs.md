# Automation rework, plan 1: the two reported bugs — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix the music fill's next and top-up when the operator picks the last row (AR13), and let a block be added after the last block of a day (AR18), on the current UI.

**Architecture:** Both fixes are pure model rules in `fp-model` (rule 7). The filler's `anchor` (where "ahead" starts) follows the player's next; the top-up gives a playing filler with no next its first new row. The editor's free-start search moves from `fp-app` to `fp_model::schedule::free_start` and also tries half, quarter and five-minute slots.

**Tech Stack:** Rust, `fp-model` integration tests (`crates/fp-model/tests/it`).

**Spec:** `docs/superpowers/specs/2026-11-13-automation-rework-design.md` (AR13, AR18).

## Global Constraints

- No `unwrap`/`expect`/`panic` outside tests; prefer `get` to indexing.
- Behaviour lives in `fp-model`; the UI only calls it.
- No new strings, no config fields.

## Review Focus

- A next set **before** the current row (the operator goes back): the look-ahead must not count the current row twice nor skip the top-up — covered by Task 1's third test.
- A stopped filler keeps today's behaviour (its next is the first new row when its list ran out) — covered by the existing `settle_fills_up_to_the_lookahead` and friends; run the whole `filler` module.
- A day with every minute taken: `free_start` returns `None`, the button does nothing — Task 2's last assertion.
- Automation timing (`starts_ahead`, `refit`, `drop_beyond`) uses `anchor`: the full `automation_*` and `filler*` test modules must stay green.

---

### Task 1: The filler's ahead follows next; new rows become next (AR13)

**Files:**
- Modify: `crates/fp-model/src/filler.rs` (`anchor`, end of `top_up`)
- Test: `crates/fp-model/tests/it/filler.rs`

**Interfaces:** none new.

- [ ] **Step 1: Write the failing tests** in `filler.rs` (use the existing `filled`, `settle`, `next`, `filler`, `rows` helpers; lookahead is 30 min by default, tracks of 180 s):

```rust
#[test]
fn choosing_the_last_row_as_next_tops_up_after_it() {
    let mut s = filled(40, 180.0);
    let (player, _) = filler(&s);
    apply(&mut s, Command::Play(player)).unwrap();
    let last = *rows(&s).last().unwrap();
    apply(&mut s, Command::SetNext(player, last)).unwrap();
    let after = rows(&s);
    let at = after.iter().position(|e| *e == last).unwrap();
    assert!(after.len() - at - 1 >= 9, "30 min of 3-min songs after the chosen next");
    assert_eq!(s.player(player).unwrap().next, Some(last));
}

#[test]
fn rows_added_after_the_last_row_on_air_become_next() {
    let mut s = filled(40, 180.0);
    let (player, _) = filler(&s);
    apply(&mut s, Command::Play(player)).unwrap();
    // The last row goes on air with nothing after it yet.
    let last = *rows(&s).last().unwrap();
    apply(&mut s, Command::SetNext(player, last)).unwrap();
    next(&mut s, player);
    let p = s.player(player).unwrap();
    assert_eq!(p.current, Some(last));
    let list = rows(&s);
    let at = list.iter().position(|e| *e == last).unwrap();
    assert_eq!(p.next, list.get(at + 1).copied());
    assert!(p.next.is_some());
}

#[test]
fn a_next_before_the_row_on_air_still_tops_up() {
    let mut s = filled(40, 180.0);
    let (player, _) = filler(&s);
    apply(&mut s, Command::Play(player)).unwrap();
    next(&mut s, player);
    next(&mut s, player);
    let first = rows(&s)[0];
    apply(&mut s, Command::SetNext(player, first)).unwrap();
    let len = rows(&s).len();
    settle(&mut s);
    assert_eq!(rows(&s).len(), len, "no growth: the rows from next on already cover 30 min");
    assert_eq!(s.player(player).unwrap().next, Some(first));
}
```

(Adapt `s.player(id)` to the existing accessor used in this file if it differs.)

- [ ] **Step 2: Run them and watch the first two fail**

Run: `cargo test -p fp-model --test it filler::`
Expected: the first two FAIL (no rows after the chosen last; next `None`); the third may pass.

- [ ] **Step 3: Implement**
  - `anchor(state, f) -> usize`: if the player has a `next` that is in the list, return its position; else `current` position + 1; else the list length. (Next wins over current: rows between the row on air and next will not play.)
  - `top_up`: replace `player_next_empty` (current and next both `None`) with "the filler player's `next` is `None`" so a playing filler whose last row is on air gets the first new row as next (`next_explicit = false`). `reconcile` in `settle_all` preloads it.

- [ ] **Step 4: Run `cargo test -p fp-model --test it filler` and `cargo test -p fp-model --test it automation`** — all PASS.

- [ ] **Step 5: Commit** `fix(model): the music fill tops up after a next chosen at its end`

### Task 2: A block can be added after the last one (AR18)

**Files:**
- Modify: `crates/fp-model/src/schedule.rs` (add `pub fn free_start`)
- Modify: `crates/fp-app/src/ui/schedule_editor.rs` (remove its `free_start`, call the model's at both call sites)
- Test: `crates/fp-model/tests/it/schedule_templates.rs`

**Interfaces:**
- Produces: `fp_model::schedule::free_start(starts: impl IntoIterator<Item = TimeOfDay>) -> Option<TimeOfDay>` — first, for each step of 60, 30, 15, 5 and 1 minutes in that order, the first free multiple of the step strictly after the last start (before midnight); if none, the same steps from 00:00; `None` when every minute is taken. With no blocks: 00:00.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn free_start_finds_a_slot_after_the_last_block() {
    let t = |h, m| TimeOfDay::hms(h, m, 0).unwrap();
    assert_eq!(free_start([t(8, 0)]), Some(t(9, 0)));
    assert_eq!(free_start([t(8, 0), t(23, 0)]), Some(t(23, 30)), "the reported case");
    assert_eq!(free_start([]), Some(t(0, 0)));
    let every_hour: Vec<_> = (0..24).map(|h| t(h, 0)).collect();
    assert_eq!(free_start(every_hour.clone()), Some(t(23, 30)));
    let mut more = every_hour;
    more.push(t(23, 30));
    assert_eq!(free_start(more), Some(t(23, 45)));
    let all: Vec<_> = (0..1440).map(|m| t(m / 60, m % 60)).collect();
    assert_eq!(free_start(all), None);
}
```

- [ ] **Step 2: Run** `cargo test -p fp-model --test it schedule_templates::free_start` — FAIL (no such function).
- [ ] **Step 3: Implement** `free_start` in `schedule.rs`; delete the app's copy and import the model's in `schedule_editor.rs` (both call sites).
- [ ] **Step 4: Run** the test and `cargo test -p fp-app --test it schedule_editor_ui` — PASS.
- [ ] **Step 5: Commit** `fix(model): a block can be added after the last of a day`
