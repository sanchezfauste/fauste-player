# Resizable Layout, Plan 1: the Model and the Session

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `fp-model` holds and rules the main window's layout (RL1–RL7, RL9, RL10 and the model side of RL3), and `fp-store` saves it in the session and restores it leniently with its repairs (RL19). Nothing on screen changes yet.

**Architecture:** A new pure module `fp_model::layout` holds the types (`Layout`, `CartwallLayout`, `Dock`, `Panel`), the read helpers the interface will call (`shown_players`, `shown_fractions`, `share`, `are_neighbours`, `is_last_shown`, `CartwallLayout::share`), the reducer rules (`pub(crate)` functions the reducer's `match` calls) and `settle_layout`, the one repair that both `SetPlayerCount` (RL3) and `AppState::restore` (RL19) run. `AppState` gains `layout`; `RestoreParts` gains `layout`; `SessionDoc` gains `layout`, read with `fp_model::layout::lenient_layout` (each part, and each list element, falls back to its default). The model does no logging: `fp-store` turns the raw JSON into load notes (`layout_notes`) and probes `settle_layout` on a copy after the restore, as it already does for the cart pages.

**Tech Stack:** Rust 2024, `fp-model` (pure, `serde`), `fp-store` (`serde_json`), Fluent strings in `fp-app`.

**Spec:** `docs/superpowers/specs/2026-10-21-resizable-layout-design.md` (§1–§3, §5, §7 "Model" and "Restore and store"). Read it whole before starting.

This is plan 1 of 3, executed strictly in order on one branch, `feat/resizable-layout`:
1. **this plan** — the model, the session document, the error strings;
2. `2026-10-21-p13-plan2-…` — the pixel pass for the players, the player splitters, the top-bar tiles and layout menu, the lock in the interface, maximise, RL8's player minimums, RL17 and RL18;
3. `2026-10-21-p13-plan3-…` — the cartwall dock in the interface, its splitter, RL13 (buttons scale), RL14 (collapsed on a side), the user guide, `docs/technical/ui.md`, the Phase 2 spec amendment.

Between plan 1 and plan 2 every layout command works in the model and is saved, but nothing sends them: the window looks exactly as today.

## Rulings (planning)

Record each one in `.superpowers/sdd/itr05/progress.md` (section P13, plan 1) when the task that applies it is done.

**Ruling: `SetPlayerShares` keeps the pair's sum itself.** The model scales the two new shares so that their sum is the sum of the two players' current (RL2-resolved) shares, then writes down a share for every current player. — Why: RL4 says "their sum stays as before"; scaling makes that hold whatever rounding the interface's drag maths has, so no other player's width ever moves; writing every share down drops shares of players that are gone. The interface may send any two positive numbers in the ratio it wants. — Cost if wrong: a test and four lines in `set_player_shares`.

**Ruling: "the mean share of the others" (RL2) is the mean of the valid shares of every current player, hidden ones included; shares of ids that are not current players are ignored.** — Why: a hidden player keeps its share (RL2), so it is still one of "the others"; a stale id must not tilt a new player's slice. — Cost if wrong: a filter in `Layout::share`.

**Ruling: `SetCartwallDock` to the dock the cartwall already has changes nothing (the size is kept).** — Why: the layout menu (RL15) shows the current dock checked; clicking it must not throw away a resized cartwall. — Cost if wrong: one `if`.

**Ruling: `SetCartwallSize(None)` on a side dock stores the default side share; `Some` of a non-finite number is `BadLayout`.** — Why: `None` means "fit the content", which only exists at the bottom; RL12's double-click on a side splitter wants the default share. — Cost if wrong: one line.

**Ruling: `SetPanelHidden` for a player that does not exist is `ModelError::UnknownPlayer`, not `BadLayout`.** — Why: every other command refuses an unknown player that way, and the interface's notice for it ("That item no longer exists") reads right. RL4's stranger stays `BadLayout`, as the spec says. — Cost if wrong: one error variant in one test.

**Ruling: the lock is checked first.** A locked layout answers `LayoutLocked` to every layout command, even one that would also be `BadLayout`. — Why: RL9 ("the model rejects every layout command"). — Cost if wrong: none.

**Ruling: one repair, `settle_layout`, serves RL3 and RL19.** It drops shares and hidden flags of players that are not current, a second share or flag of the same player (the first wins), and shares that are not finite and above 0; it shows every panel when none is shown; and it fits the cartwall's `size` to its dock (`CartwallLayout::share`: a side dock gets a share, any share is clamped). `set_player_count` runs it (so `UpdateConfig` lowering `max_players` does too) and `AppState::restore` runs it after it builds the players. — Why: the spec's repairs and RL3's are the same set; the extra ones (duplicates, bad shares, the dock-fitted size) keep a hand-edited session from carrying nonsense forward. — Cost if wrong: drop the extra branches.

**Ruling: the model does not log; the store reports.** `settle_layout` returns one note per repair; `fp-store` runs it again on a copy of the saved layout after the restore (the restore ran it with the same players) and logs the notes, the way it already probes `Cartwall::normalize`. Parts that do not parse are reported by `fp-store::layout_notes` from the raw JSON, the way pending starts and cart colours are. — Why: `fp-model` does no I/O (CLAUDE.md, architecture). — Cost if wrong: none known.

**Ruling: RL17 and RL18 are interface rules.** The table selection and "the last player used" are `ViewState` (`active_player` in `crates/fp-app/src/ui/app.rs`), and file drops are handled in `AppUi::file_drops`; nothing in `AppState` changes. They go to plan 2. RL8's minimums are interface constants (plans 2 and 3).

**Ruling: the three error messages land in this plan, in every locale.** — Why: `fp-app`'s `error_text` matches `ModelError` exhaustively, so the workspace does not build without them; CLAUDE.md wants every new string in every locale (en-US and es-ES written, the other nine translated with AI). — Cost if wrong: none.

**Ruling: `Layout` is re-exported at the crate root as the spec names it.** `fp-app` already imports `egui::Layout` in many files: there, plans 2 and 3 use `fp_model::layout::Layout` (or `use fp_model::Layout as PanelLayout;`), never a bare `Layout` import from both. — Cost if wrong: a rename.

## Global Constraints

- All code, identifiers, comments, docs and commit messages in English; never mention other playout or radio-automation products.
- Behaviour lives in `fp-model` as pure functions, **one test per spec rule** (CLAUDE.md rule 7, spec §7); test names start with the rule (`rl1_…`, `rl19_…`), the §2 terms with `terms_…`.
- Spec §3: "The layout is session state, kept with the players (RL16), as the table column widths are. It belongs to the operator's desk, not to the station's settings, so it is not part of `Config`."
- RL1: "The default is today's window: equal shares, nothing hidden, the cartwall at the bottom fitting its content, unlocked."
- RL6: the default side share is **0.3**; the size is clamped to **`0.05 ..= 0.95`** (constants `DEFAULT_SIDE_SHARE`, `MIN_CARTWALL_SHARE`, `MAX_CARTWALL_SHARE`; they are layout rules, not operator limits).
- RL7: "Hiding never touches playback: a hidden player that is on air keeps playing."
- RL19: "The session document gains `layout`, so `SESSION_SCHEMA` stays 1." "A part that does not parse … loads as its default with a log line. The rest of the session is never rejected because of it." "Each repair writes a log line."
- Never crash on bad data (CLAUDE.md rule 9); `unwrap`, `expect`, `panic` denied outside tests; `unsafe_code` forbidden.
- On `Err` a command leaves the state unchanged (`apply`'s contract): every layout rule validates before it writes.
- Conventional Commits, scopes `model`, `store`, `app`; every commit ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- `cargo` is at `$HOME/.cargo/bin` (`export PATH="$HOME/.cargo/bin:$PATH"`). Gate for every task: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.
- Progress ledger: `.superpowers/sdd/itr05/progress.md` (section P13, plan 1). Record deviations as `Ruling: <decision> — <why> — <cost if wrong>`.

## Review Focus

1. **Absurd but finite shares in a hand-edited session** (`f32::MAX` for every player, or `1e-30`): the fractions must stay finite and add up to 1, not become `inf`/`NaN` widths. Test: Task 1, `rl2_huge_or_tiny_shares_still_give_fractions_that_add_up_to_one`.
2. **A drag whose two shares do not add up to the pair's old sum** (the interface's rounding, or pixel widths sent as shares): the two players split their old space and no other player moves. Test: Task 2, `rl4_the_pair_keeps_its_sum_whatever_the_interface_sends`.
3. **The same player twice in `player_shares` or `hidden_players`** (a hand edit): the first share wins, the flag counts once, and settling again changes nothing. Test: Task 4, `rl19_settle_keeps_the_first_of_a_repeated_player`.
4. **A side dock saved with `size: null`, or a size of 5.0**: the restored cartwall has a usable share (0.3, 0.95) and the load says so. Test: Task 6, `rl19_a_side_dock_gets_a_usable_size_with_a_note`.
5. **A session written by a newer build with an extra field inside `layout`** (named layouts are planned): the layout loads as saved with no warning. Test: Task 6, `rl19_a_field_from_a_newer_build_is_ignored`.

---

## File Structure

| File | Responsibility |
|---|---|
| `crates/fp-model/src/layout.rs` (new) | Types, constants, read helpers (RL1, RL2, RL4 terms, RL6, RL7), the rules (`pub(crate)`), `settle_layout`, lenient deserialisers and `lenient_layout` |
| `crates/fp-model/src/lib.rs` | `pub mod layout;` and re-exports |
| `crates/fp-model/src/state.rs` | `AppState::layout`, `AppState::player_ids` |
| `crates/fp-model/src/command.rs` | the seven layout commands |
| `crates/fp-model/src/error.rs` | `BadLayout`, `LastPanel`, `LayoutLocked` |
| `crates/fp-model/src/reducer.rs` (`apply_command` L30-397, `set_player_count` L1304-1336) | the commands' arms; RL3 in `set_player_count` |
| `crates/fp-model/src/session.rs` (`RestoreParts` L94-103, `AppState::restore` L131-261) | `RestoreParts::layout`; RL19 in `restore` |
| `crates/fp-model/tests/layout.rs` (new) | RL1–RL7, RL9, RL10, RL3, model side of RL19 |
| `crates/fp-model/tests/common/mod.rs`, `tests/session.rs`, `tests/pending_start.rs`, `tests/loudness.rs`, `tests/cue_markers_off.rs`, `tests/review_fixes.rs` | `RestoreParts` literals gain `layout` |
| `crates/fp-store/src/docs.rs` | `SessionDoc::layout` |
| `crates/fp-store/src/layout_notes.rs` (new) | load notes for parts of `layout` that do not parse |
| `crates/fp-store/src/lib.rs`, `crates/fp-store/src/store.rs` | wiring: parse notes, save, load, the settle probe, the fuzz entry |
| `crates/fp-store/tests/session_layout.rs` (new) | RL19 through the files |
| `crates/fp-app/src/ui/app.rs` (`error_text` L1604-1629), `crates/fp-app/locales/*/main.ftl` | the three error messages |
| `docs/technical/architecture.md`, `docs/technical/persistence.md` | the `layout` module; the session's `layout` |

---

### Task 1: Layout types and the share rules (RL1, RL2)

**Files:**
- Create: `crates/fp-model/src/layout.rs`
- Modify: `crates/fp-model/src/lib.rs` (module list L5-36, re-exports L38-98)
- Modify: `crates/fp-model/src/state.rs` (`AppState` L11-27, `AppState::new` L29-58)
- Modify: `crates/fp-model/src/session.rs` (the `AppState { … }` literal in `restore`, L173-181)
- Modify: `docs/technical/architecture.md` (L7)
- Test: `crates/fp-model/tests/layout.rs` (new)

**Interfaces:**
- Consumes: `fp_model::PlayerId` (`pub struct PlayerId(pub u64)`, `Copy + Eq + Hash + Ord + Serialize + Deserialize`, transparent).
- Produces (all in `fp_model::layout`, re-exported at the crate root):
  - `pub const DEFAULT_SIDE_SHARE: f32 = 0.3;` `pub const MIN_CARTWALL_SHARE: f32 = 0.05;` `pub const MAX_CARTWALL_SHARE: f32 = 0.95;`
  - `pub enum Dock { Bottom /* default */, Left, Right }` (`Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize`; JSON `"Bottom"`, `"Left"`, `"Right"`), `impl Dock { pub const ALL: [Dock; 3]; pub fn is_side(self) -> bool }`
  - `pub enum Panel { Player(PlayerId), Cartwall }` (`Debug, Clone, Copy, PartialEq, Eq, Hash`; never saved)
  - `pub struct CartwallLayout { pub dock: Dock, pub size: Option<f32>, pub hidden: bool }` (`Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize`), `impl CartwallLayout { pub fn share(&self) -> Option<f32> }`
  - `pub struct Layout { pub player_shares: Vec<(PlayerId, f32)>, pub hidden_players: Vec<PlayerId>, pub cartwall: CartwallLayout, pub locked: bool }` (`Debug, Clone, PartialEq, Default, Serialize, Deserialize`)
  - `impl Layout { pub fn is_hidden(&self, panel: Panel) -> bool; pub fn shown_players(&self, players: &[PlayerId]) -> Vec<PlayerId>; pub fn shown_panels(&self, players: &[PlayerId]) -> usize; pub fn is_last_shown(&self, players: &[PlayerId], panel: Panel) -> bool; pub fn share(&self, players: &[PlayerId], player: PlayerId) -> f32; pub fn shown_fractions(&self, players: &[PlayerId]) -> Vec<(PlayerId, f32)>; pub fn are_neighbours(&self, players: &[PlayerId], left: PlayerId, right: PlayerId) -> bool }`
  - `AppState::layout: Layout` and `AppState::player_ids(&self) -> Vec<PlayerId>` (display order).
  - Every helper that takes `players: &[PlayerId]` expects `state.player_ids()`.

- [ ] **Step 1: Write the failing tests**

Create `crates/fp-model/tests/layout.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Resizable layout spec RL1–RL10, RL3 and the model side of RL19: one
//! test per rule, named after it.

mod common;

use common::fixture;
use fp_model::{AppState, CartwallLayout, Config, Dock, Layout, Panel, PlayerId};

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-5
}

fn all_close(got: &[f32], want: &[f32]) -> bool {
    got.len() == want.len() && got.iter().zip(want).all(|(a, b)| close(*a, *b))
}

/// The fractions of the shown players, in display order.
fn fractions(s: &AppState) -> Vec<f32> {
    s.layout
        .shown_fractions(&s.player_ids())
        .into_iter()
        .map(|(_, f)| f)
        .collect()
}

#[test]
fn rl1_the_default_layout_is_todays_window() {
    let s = AppState::new(Config::default(), "Main");
    assert_eq!(s.layout, Layout::default());
    assert!(s.layout.player_shares.is_empty());
    assert!(s.layout.hidden_players.is_empty());
    assert_eq!(
        s.layout.cartwall,
        CartwallLayout {
            dock: Dock::Bottom,
            size: None,
            hidden: false
        }
    );
    assert!(!s.layout.locked);
    assert!(all_close(&fractions(&s), &[0.25; 4]));
}

#[test]
fn rl1_panels_are_the_shown_players_and_the_cartwall() {
    let mut s = fixture(0);
    let p = s.player_ids();
    assert_eq!(s.layout.shown_panels(&p), 5);
    s.layout.hidden_players = vec![p[0], p[1], p[2]];
    s.layout.cartwall.hidden = true;
    assert_eq!(s.layout.shown_panels(&p), 1);
    assert!(s.layout.is_hidden(Panel::Cartwall));
    assert!(s.layout.is_hidden(Panel::Player(p[0])));
    assert!(s.layout.is_last_shown(&p, Panel::Player(p[3])));
    assert!(!s.layout.is_last_shown(&p, Panel::Cartwall), "already hidden");
    assert!(!s.layout.is_last_shown(&p, Panel::Player(PlayerId(999_999))));
}

#[test]
fn rl2_with_no_share_every_player_gets_one() {
    let s = fixture(0);
    let p = s.player_ids();
    assert!(p.iter().all(|id| close(s.layout.share(&p, *id), 1.0)));
}

#[test]
fn rl2_a_player_without_a_share_gets_the_mean_of_the_others() {
    let mut s = fixture(0);
    let p = s.player_ids();
    s.layout.player_shares = vec![(p[0], 2.0), (p[1], 1.0)];
    assert!(close(s.layout.share(&p, p[2]), 1.5));
    assert!(all_close(
        &fractions(&s),
        &[2.0 / 6.0, 1.0 / 6.0, 1.5 / 6.0, 1.5 / 6.0]
    ));
}

#[test]
fn rl2_a_bad_share_counts_as_missing() {
    let mut s = fixture(0);
    let p = s.player_ids();
    s.layout.player_shares = vec![(p[0], f32::NAN), (p[1], -1.0), (p[2], 0.0), (p[3], 2.0)];
    assert!(p.iter().all(|id| close(s.layout.share(&p, *id), 2.0)));
    assert!(all_close(&fractions(&s), &[0.25; 4]));
}

#[test]
fn rl2_a_share_of_a_player_that_does_not_exist_is_ignored() {
    let mut s = fixture(0);
    let p = s.player_ids();
    s.layout.player_shares = vec![(PlayerId(999_999), 10.0)];
    assert!(close(s.layout.share(&p, p[0]), 1.0));
}

#[test]
fn rl2_a_hidden_player_keeps_its_share() {
    let mut s = fixture(0);
    let p = s.player_ids();
    s.layout.player_shares = vec![(p[0], 1.0), (p[1], 3.0), (p[2], 1.0), (p[3], 1.0)];
    s.layout.hidden_players = vec![p[1]];
    assert_eq!(s.layout.shown_players(&p), vec![p[0], p[2], p[3]]);
    assert!(close(s.layout.share(&p, p[1]), 3.0));
    assert!(all_close(&fractions(&s), &[1.0 / 3.0; 3]));
    s.layout.hidden_players.clear();
    assert!(close(fractions(&s)[1], 0.5), "shown again, it has its width back");
}

#[test]
fn rl2_huge_or_tiny_shares_still_give_fractions_that_add_up_to_one() {
    let mut s = fixture(0);
    let p = s.player_ids();
    s.layout.player_shares = p.iter().map(|id| (*id, f32::MAX)).collect();
    assert!(all_close(&fractions(&s), &[0.25; 4]));
    s.layout.player_shares = vec![(p[0], 1e-30), (p[1], 1e-30)];
    let f = fractions(&s);
    assert!(f.iter().all(|x| x.is_finite() && *x >= 0.0), "{f:?}");
    assert!(close(f.iter().sum(), 1.0), "{f:?}");
}

#[test]
fn terms_neighbours_skip_hidden_players() {
    let mut s = fixture(0);
    let p = s.player_ids();
    assert!(s.layout.are_neighbours(&p, p[0], p[1]));
    assert!(!s.layout.are_neighbours(&p, p[1], p[0]), "left comes first");
    assert!(!s.layout.are_neighbours(&p, p[0], p[2]));
    s.layout.hidden_players = vec![p[1]];
    assert!(s.layout.are_neighbours(&p, p[0], p[2]));
    assert!(!s.layout.are_neighbours(&p, p[0], p[1]));
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-model --test layout`
Expected: FAIL to compile: no `CartwallLayout`, `Dock`, `Layout`, `Panel` in `fp_model`, no field `layout`, no method `player_ids`.

- [ ] **Step 3: Create `crates/fp-model/src/layout.rs`**

```rust
//! The main window's arrangement (resizable layout spec RL1–RL10, RL19):
//! the players' shares of the width, the hidden panels, where the cartwall
//! sits and how large it is, and the lock. Pure rules: the interface turns
//! them into rectangles (RL11).

use serde::{Deserialize, Serialize};

use crate::ids::PlayerId;

/// RL6: the share of the middle area's width the cartwall gets when it is
/// docked on a side.
pub const DEFAULT_SIDE_SHARE: f32 = 0.3;
/// RL6: the smallest share of the middle area the cartwall's `size` holds.
pub const MIN_CARTWALL_SHARE: f32 = 0.05;
/// RL6: the largest share of the middle area the cartwall's `size` holds.
pub const MAX_CARTWALL_SHARE: f32 = 0.95;

/// Where the cartwall sits (spec §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Dock {
    /// Under the players (today's place).
    #[default]
    Bottom,
    /// The full height of the middle area, left of the players.
    Left,
    /// The full height of the middle area, right of the players.
    Right,
}

impl Dock {
    /// Every dock, in the order the layout menu lists them.
    pub const ALL: [Dock; 3] = [Dock::Bottom, Dock::Left, Dock::Right];

    /// `Left` or `Right`.
    pub fn is_side(self) -> bool {
        !matches!(self, Dock::Bottom)
    }
}

/// One panel of the middle area (spec §2): a player column or the
/// cartwall.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Panel {
    Player(PlayerId),
    Cartwall,
}

/// The cartwall's place and size (RL1).
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CartwallLayout {
    pub dock: Dock,
    /// The cartwall's share of the middle area: of its height at the
    /// bottom, of its width on a side. `None` fits the content (bottom
    /// only).
    pub size: Option<f32>,
    pub hidden: bool,
}

impl CartwallLayout {
    /// RL6, RL11: the share the cartwall takes. At the bottom, `size`
    /// (`None`: fit the content); on a side, `size` or the default side
    /// share. A share is always within `MIN_CARTWALL_SHARE ..=
    /// MAX_CARTWALL_SHARE`; a non-finite `size` counts as `None`.
    pub fn share(&self) -> Option<f32> {
        let size = self.size.filter(|s| s.is_finite());
        let size = if self.dock.is_side() {
            size.or(Some(DEFAULT_SIDE_SHARE))
        } else {
            size
        };
        size.map(|s| s.clamp(MIN_CARTWALL_SHARE, MAX_CARTWALL_SHARE))
    }
}

/// The layout of the main window (RL1). Session state, saved with the
/// players; not part of `Config`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Layout {
    /// Each player's share of the players' width (RL2).
    pub player_shares: Vec<(PlayerId, f32)>,
    pub hidden_players: Vec<PlayerId>,
    pub cartwall: CartwallLayout,
    pub locked: bool,
}

impl Layout {
    /// RL7: whether `panel` is hidden.
    pub fn is_hidden(&self, panel: Panel) -> bool {
        match panel {
            Panel::Player(id) => self.hidden_players.contains(&id),
            Panel::Cartwall => self.cartwall.hidden,
        }
    }

    /// The shown players among `players` (`AppState::player_ids`), in
    /// display order.
    pub fn shown_players(&self, players: &[PlayerId]) -> Vec<PlayerId> {
        players
            .iter()
            .copied()
            .filter(|p| !self.hidden_players.contains(p))
            .collect()
    }

    /// How many panels are shown: the shown players and the cartwall.
    pub fn shown_panels(&self, players: &[PlayerId]) -> usize {
        self.shown_players(players).len() + usize::from(!self.cartwall.hidden)
    }

    /// RL7, RL15: `panel` is the only shown panel, so it cannot be hidden
    /// (its top-bar tile is disabled). False for a player not in
    /// `players`.
    pub fn is_last_shown(&self, players: &[PlayerId], panel: Panel) -> bool {
        let known = match panel {
            Panel::Player(id) => players.contains(&id),
            Panel::Cartwall => true,
        };
        known && !self.is_hidden(panel) && self.shown_panels(players) == 1
    }

    /// RL2: the share of `player`: its saved share when that is finite and
    /// above 0, else the mean of the valid shares of `players` (hidden ones
    /// included), else 1.0.
    pub fn share(&self, players: &[PlayerId], player: PlayerId) -> f32 {
        self.valid_share(player)
            .unwrap_or_else(|| self.mean_share(players))
    }

    /// RL2: the shown players with their fraction of the players' width;
    /// the fractions add up to 1. The pixel pass (RL11) adds the minimums.
    pub fn shown_fractions(&self, players: &[PlayerId]) -> Vec<(PlayerId, f32)> {
        let shown = self.shown_players(players);
        let shares: Vec<f64> = shown
            .iter()
            .map(|p| f64::from(self.share(players, *p)))
            .collect();
        let sum: f64 = shares.iter().sum();
        shown
            .into_iter()
            .zip(shares)
            .map(|(p, s)| (p, (s / sum) as f32))
            .collect()
    }

    /// RL4 (spec §2): `right` is the next shown player after `left`.
    /// Hidden players between them do not count.
    pub fn are_neighbours(&self, players: &[PlayerId], left: PlayerId, right: PlayerId) -> bool {
        self.shown_players(players)
            .windows(2)
            .any(|pair| pair == [left, right])
    }

    fn valid_share(&self, player: PlayerId) -> Option<f32> {
        self.player_shares
            .iter()
            .find(|(id, _)| *id == player)
            .map(|(_, share)| *share)
            .filter(|share| share.is_finite() && *share > 0.0)
    }

    /// Summed in `f64`, so a hand-edited `f32::MAX` cannot overflow.
    fn mean_share(&self, players: &[PlayerId]) -> f32 {
        let valid: Vec<f64> = players
            .iter()
            .filter_map(|p| self.valid_share(*p))
            .map(f64::from)
            .collect();
        if valid.is_empty() {
            1.0
        } else {
            (valid.iter().sum::<f64>() / valid.len() as f64) as f32
        }
    }
}
```

- [ ] **Step 4: Wire it into the crate and the state**

`crates/fp-model/src/lib.rs`: add `pub mod layout;` after `pub mod ids;` and, after the `pub use ids::…;` line:

```rust
pub use layout::{
    CartwallLayout, DEFAULT_SIDE_SHARE, Dock, Layout, MAX_CARTWALL_SHARE, MIN_CARTWALL_SHARE,
    Panel,
};
```

`crates/fp-model/src/state.rs`: add the field after `cue_on_air` in `AppState`:

```rust
    /// The main window's arrangement (resizable layout spec RL1), saved
    /// with the session.
    pub layout: crate::layout::Layout,
```

in `AppState::new`, after `cue_on_air: crate::cue_on_air::CueOnAir::default(),`:

```rust
            layout: crate::layout::Layout::default(),
```

and after `player` (L67-72):

```rust
    /// The players' ids in display order: what the layout helpers take.
    pub fn player_ids(&self) -> Vec<PlayerId> {
        self.players.iter().map(|p| p.id).collect()
    }
```

`crates/fp-model/src/session.rs`, in the `AppState { … }` literal of `restore`, after `cue_on_air: …,`:

```rust
            layout: crate::layout::Layout::default(),
```

(Task 5 replaces it with the saved layout.)

`docs/technical/architecture.md` L7: after "pre-listen on an on-air output in `cue_on_air`: what sounds there, the confirmation request, the notice" insert ", the main window's layout in `layout`: the players' shares, hidden panels, the cartwall's dock and size, the lock (resizable layout spec)".

- [ ] **Step 5: Run the tests**

Run: `cargo test -p fp-model --test layout`
Expected: PASS (10 tests).

- [ ] **Step 6: Gate and commit**

Run the gate. Then:

```bash
git add crates/fp-model/src/layout.rs crates/fp-model/src/lib.rs crates/fp-model/src/state.rs \
  crates/fp-model/src/session.rs crates/fp-model/tests/layout.rs docs/technical/architecture.md
git commit -m "feat(model): the main window's layout and its share rules

RL1 and RL2 of the resizable layout spec: AppState holds a Layout
(player shares, hidden players, the cartwall's dock and size, the lock)
whose default is today's window. A missing or bad share counts as the
mean of the others, a hidden player keeps its share, and fractions are
summed in f64 so a hand-edited share cannot overflow.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: Resizing players and equal widths (RL4, RL5); the layout errors

**Files:**
- Modify: `crates/fp-model/src/layout.rs`
- Modify: `crates/fp-model/src/command.rs` (imports L5-11; variants after `SetColumnWidths`, L67)
- Modify: `crates/fp-model/src/error.rs` (after `FaderLocked`, L46-47)
- Modify: `crates/fp-model/src/reducer.rs` (arm after `Command::SetColumnWidths`, L182-185)
- Modify: `crates/fp-app/src/ui/app.rs` (`error_text`, L1604-1629)
- Modify: `crates/fp-app/locales/{en-US,es-ES,ca-ES,de-DE,eu-ES,fr-FR,gl-ES,it-IT,nl-NL,pl-PL,pt-PT}/main.ftl` (after the `error-invalid-trim` line)
- Test: `crates/fp-model/tests/layout.rs`

**Interfaces:**
- Consumes: Task 1's `Layout::share`, `Layout::are_neighbours`, `AppState::player_ids`.
- Produces:
  - `Command::SetPlayerShares { left: (PlayerId, f32), right: (PlayerId, f32) }` — `right` must be the next shown player after `left`; both shares finite and above 0 (0 is refused: a player dragged to its minimum sends a small positive share). The model scales the pair to keep its old sum, so the interface may send the two widths' ratio in any unit.
  - `Command::EqualPlayerWidths`
  - `ModelError::BadLayout`, `ModelError::LastPanel`, `ModelError::LayoutLocked` (unit variants).
  - Fluent keys `error-bad-layout`, `error-last-panel`, `error-layout-locked`, shown by `error_text`.
  - `pub(crate) fn layout::unlocked(state: &AppState) -> Result<(), ModelError>` (Task 3 uses it).

- [ ] **Step 1: Write the failing tests**

In `crates/fp-model/tests/layout.rs`, make the `use fp_model::…` line read:

```rust
use fp_model::{
    AppState, CartwallLayout, Command, Config, Dock, Layout, ModelError, Panel, PlayerId, apply,
};
```

and append:

```rust
/// Every player's RL2 share, in display order.
fn shares(s: &AppState) -> Vec<f32> {
    let p = s.player_ids();
    p.iter().map(|id| s.layout.share(&p, *id)).collect()
}

#[test]
fn rl4_a_splitter_drag_sets_both_shares() {
    let mut s = fixture(0);
    let p = s.player_ids();
    apply(
        &mut s,
        Command::SetPlayerShares {
            left: (p[0], 1.5),
            right: (p[1], 0.5),
        },
    )
    .unwrap();
    assert!(all_close(&shares(&s), &[1.5, 0.5, 1.0, 1.0]));
}

#[test]
fn rl4_the_pair_keeps_its_sum_whatever_the_interface_sends() {
    let mut s = fixture(0);
    let p = s.player_ids();
    let before = fractions(&s);
    apply(
        &mut s,
        Command::SetPlayerShares {
            left: (p[1], 300.0),
            right: (p[2], 100.0),
        },
    )
    .unwrap();
    assert!(all_close(&shares(&s), &[1.0, 1.5, 0.5, 1.0]));
    let after = fractions(&s);
    assert!(close(after[0], before[0]) && close(after[3], before[3]));
}

#[test]
fn rl4_every_player_gets_its_share_written_down() {
    let mut s = fixture(0);
    let p = s.player_ids();
    s.layout.player_shares = vec![(p[0], 4.0), (PlayerId(999_999), 2.0)];
    apply(
        &mut s,
        Command::SetPlayerShares {
            left: (p[2], 1.0),
            right: (p[3], 7.0),
        },
    )
    .unwrap();
    let ids: Vec<PlayerId> = s.layout.player_shares.iter().map(|(id, _)| *id).collect();
    assert_eq!(ids, p, "one share per current player, the stranger gone");
    assert!(all_close(&shares(&s), &[4.0, 4.0, 1.0, 7.0]));
}

#[test]
fn rl4_neighbours_across_a_hidden_player() {
    let mut s = fixture(0);
    let p = s.player_ids();
    s.layout.hidden_players = vec![p[1]];
    apply(
        &mut s,
        Command::SetPlayerShares {
            left: (p[0], 0.5),
            right: (p[2], 1.5),
        },
    )
    .unwrap();
    assert!(all_close(&shares(&s), &[0.5, 1.0, 1.5, 1.0]));
}

#[test]
fn rl4_a_stranger_a_non_neighbour_and_a_bad_share_are_rejected() {
    let mut s = fixture(0);
    let p = s.player_ids();
    s.layout.hidden_players = vec![p[3]];
    let stranger = PlayerId(999_999);
    let before = s.clone();
    for (left, right) in [
        ((stranger, 1.0), (p[0], 1.0)),
        ((p[0], 1.0), (p[2], 1.0)),
        ((p[1], 1.0), (p[0], 1.0)),
        ((p[2], 1.0), (p[3], 1.0)),
        ((p[0], f32::NAN), (p[1], 1.0)),
        ((p[0], 1.0), (p[1], f32::INFINITY)),
        ((p[0], 0.0), (p[1], 2.0)),
        ((p[0], -1.0), (p[1], 3.0)),
    ] {
        let err = apply(&mut s, Command::SetPlayerShares { left, right }).unwrap_err();
        assert_eq!(err, ModelError::BadLayout, "{left:?} {right:?}");
        assert_eq!(s, before, "a refused command changes nothing");
    }
}

#[test]
fn rl5_equal_widths_gives_every_player_the_same_share() {
    let mut s = fixture(0);
    let p = s.player_ids();
    s.layout.player_shares = vec![(p[0], 3.0), (p[1], 0.2)];
    s.layout.hidden_players = vec![p[2]];
    apply(&mut s, Command::EqualPlayerWidths).unwrap();
    assert!(all_close(&shares(&s), &[1.0; 4]));
    assert!(all_close(&fractions(&s), &[1.0 / 3.0; 3]));
    assert_eq!(s.layout.hidden_players, vec![p[2]], "it shows nothing");
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-model --test layout`
Expected: FAIL to compile: no variant `SetPlayerShares`, `EqualPlayerWidths`, `BadLayout`.

- [ ] **Step 3: Errors, commands, rules**

`crates/fp-model/src/error.rs`, after `FaderLocked`:

```rust
    /// Resizable layout spec RL4, RL6: a resize between players that are not
    /// neighbours, or a share or size that is not a usable number.
    #[error("that layout change is not valid")]
    BadLayout,
    /// RL7: the last shown panel cannot be hidden.
    #[error("the last shown panel cannot be hidden")]
    LastPanel,
    /// RL9: the layout is locked.
    #[error("the layout is locked")]
    LayoutLocked,
```

`crates/fp-model/src/command.rs` (no new import in this task; Task 3 adds `use crate::layout::{Dock, Panel};`), after `SetColumnWidths(PlayerId, ColumnWidths),`:

```rust
    /// Resizable layout spec RL4: the shares of two neighbouring shown
    /// players after a splitter drag (`right` is the next shown player
    /// after `left`). Each share must be finite and above 0; the model
    /// scales the pair to keep its sum, so no other player moves.
    SetPlayerShares {
        left: (PlayerId, f32),
        right: (PlayerId, f32),
    },
    /// RL5: every player gets the same share.
    EqualPlayerWidths,
```

`crates/fp-model/src/layout.rs`: add `use crate::error::ModelError;` and `use crate::state::AppState;` to the imports, and at the end:

```rust
/// RL9: every layout command but `SetLayoutLocked` is refused while the
/// layout is locked. Checked before anything else.
pub(crate) fn unlocked(state: &AppState) -> Result<(), ModelError> {
    if state.layout.locked {
        Err(ModelError::LayoutLocked)
    } else {
        Ok(())
    }
}

fn usable_share(share: f32) -> bool {
    share.is_finite() && share > 0.0
}

/// Every player of `players` with its RL2 share, in display order.
fn resolved_shares(layout: &Layout, players: &[PlayerId]) -> Vec<(PlayerId, f32)> {
    players
        .iter()
        .map(|p| (*p, layout.share(players, *p)))
        .collect()
}

/// RL4: the pair takes the new shares, scaled to keep the pair's sum; every
/// current player's share is written down (shares of players that are gone
/// are dropped).
pub(crate) fn set_player_shares(
    state: &mut AppState,
    left: (PlayerId, f32),
    right: (PlayerId, f32),
) -> Result<(), ModelError> {
    unlocked(state)?;
    let players = state.player_ids();
    let layout = &state.layout;
    if !layout.are_neighbours(&players, left.0, right.0)
        || !usable_share(left.1)
        || !usable_share(right.1)
    {
        return Err(ModelError::BadLayout);
    }
    let before =
        f64::from(layout.share(&players, left.0)) + f64::from(layout.share(&players, right.0));
    let scale = before / (f64::from(left.1) + f64::from(right.1));
    let new_left = (f64::from(left.1) * scale) as f32;
    let new_right = (f64::from(right.1) * scale) as f32;
    if !usable_share(new_left) || !usable_share(new_right) {
        return Err(ModelError::BadLayout);
    }
    let mut shares = resolved_shares(layout, &players);
    for (id, share) in &mut shares {
        if *id == left.0 {
            *share = new_left;
        } else if *id == right.0 {
            *share = new_right;
        }
    }
    state.layout.player_shares = shares;
    Ok(())
}

/// RL5: every current player gets share 1.0.
pub(crate) fn equal_player_widths(state: &mut AppState) -> Result<(), ModelError> {
    unlocked(state)?;
    state.layout.player_shares = state.players.iter().map(|p| (p.id, 1.0)).collect();
    Ok(())
}
```

`crates/fp-model/src/reducer.rs`, after the `Command::SetColumnWidths(id, widths) => { … }` arm:

```rust
        Command::SetPlayerShares { left, right } => {
            crate::layout::set_player_shares(state, left, right)?
        }
        Command::EqualPlayerWidths => crate::layout::equal_player_widths(state)?,
```

- [ ] **Step 4: The interface's messages**

`crates/fp-app/src/ui/app.rs`, in `error_text`, after `ModelError::NoPlaylists => …,`:

```rust
        ModelError::BadLayout => i18n.tr("error-bad-layout"),
        ModelError::LastPanel => i18n.tr("error-last-panel"),
        ModelError::LayoutLocked => i18n.tr("error-layout-locked"),
```

In each locale file, insert these three lines right after the `error-invalid-trim = …` line:

`en-US`:
```
error-bad-layout = That layout change is not possible.
error-last-panel = At least one panel must stay shown.
error-layout-locked = The layout is locked.
```
`es-ES`:
```
error-bad-layout = Ese cambio de distribución no es posible.
error-last-panel = Al menos un panel debe seguir visible.
error-layout-locked = La distribución está bloqueada.
```
`ca-ES`:
```
error-bad-layout = Aquest canvi de disposició no és possible.
error-last-panel = Com a mínim un panell ha de quedar visible.
error-layout-locked = La disposició està bloquejada.
```
`de-DE`:
```
error-bad-layout = Diese Layoutänderung ist nicht möglich.
error-last-panel = Mindestens ein Bereich muss sichtbar bleiben.
error-layout-locked = Das Layout ist gesperrt.
```
`eu-ES`:
```
error-bad-layout = Diseinu-aldaketa hori ezin da egin.
error-last-panel = Gutxienez panel batek ikusgai egon behar du.
error-layout-locked = Diseinua blokeatuta dago.
```
`fr-FR`:
```
error-bad-layout = Cette modification de la disposition n'est pas possible.
error-last-panel = Au moins un panneau doit rester affiché.
error-layout-locked = La disposition est verrouillée.
```
`gl-ES`:
```
error-bad-layout = Ese cambio de disposición non é posible.
error-last-panel = Polo menos un panel debe quedar visible.
error-layout-locked = A disposición está bloqueada.
```
`it-IT`:
```
error-bad-layout = Questa modifica del layout non è possibile.
error-last-panel = Almeno un pannello deve restare visibile.
error-layout-locked = Il layout è bloccato.
```
`nl-NL`:
```
error-bad-layout = Die indelingswijziging is niet mogelijk.
error-last-panel = Er moet minstens één paneel zichtbaar blijven.
error-layout-locked = De indeling is vergrendeld.
```
`pl-PL`:
```
error-bad-layout = Ta zmiana układu nie jest możliwa.
error-last-panel = Co najmniej jeden panel musi pozostać widoczny.
error-layout-locked = Układ jest zablokowany.
```
`pt-PT`:
```
error-bad-layout = Essa alteração da disposição não é possível.
error-last-panel = Pelo menos um painel tem de continuar visível.
error-layout-locked = A disposição está bloqueada.
```

Plan 2 names the layout menu with the same word in each language ("distribución" in es-ES).

- [ ] **Step 5: Run the tests**

Run: `cargo test -p fp-model --test layout` and `cargo test -p fp-app --test i18n`
Expected: PASS (the i18n test checks every locale has the en-US keys).

- [ ] **Step 6: Gate and commit**

Run the gate. Then:

```bash
git add crates/fp-model/src crates/fp-model/tests/layout.rs crates/fp-app/src/ui/app.rs crates/fp-app/locales
git commit -m "feat(model): resize two neighbouring players and equal widths

RL4 and RL5: SetPlayerShares sets the shares of two shown neighbours
(hidden players between them do not count), scaled so the pair keeps
its sum and no other player moves; EqualPlayerWidths gives everyone
1.0. The layout errors (BadLayout, LastPanel, LayoutLocked) get their
messages in every locale.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: Dock, size, hiding, lock and reset (RL6, RL7, RL9, RL10)

**Files:**
- Modify: `crates/fp-model/src/layout.rs`
- Modify: `crates/fp-model/src/command.rs`
- Modify: `crates/fp-model/src/reducer.rs` (arms after Task 2's)
- Test: `crates/fp-model/tests/layout.rs`

**Interfaces:**
- Consumes: Task 1's types and constants; Task 2's `unlocked`, `ModelError::{BadLayout, LastPanel, LayoutLocked}`.
- Produces:
  - `Command::SetCartwallDock(Dock)` — a new dock resets `size` (`None` at the bottom, `Some(DEFAULT_SIDE_SHARE)` on a side); the same dock changes nothing.
  - `Command::SetCartwallSize(Option<f32>)` — clamped to `MIN_CARTWALL_SHARE ..= MAX_CARTWALL_SHARE`; `None` on a side stores `DEFAULT_SIDE_SHARE`; a non-finite `Some` is `BadLayout`.
  - `Command::SetPanelHidden { panel: Panel, hidden: bool }` — idempotent; `LastPanel` for the last shown panel; `UnknownPlayer` for a player that does not exist.
  - `Command::SetLayoutLocked(bool)` — always accepted.
  - `Command::ResetLayout` — `Layout::default()`.
  - While `layout.locked`, every one of these but `SetLayoutLocked` is `LayoutLocked`. `SetCartwallOpen` is not a layout command and works while locked.

- [ ] **Step 1: Write the failing tests**

Make the imports at the top of `crates/fp-model/tests/layout.rs` read:

```rust
use common::{fixture, p0};
use fp_model::{
    AppState, CartwallLayout, Command, Config, DEFAULT_SIDE_SHARE, Dock, Layout,
    MAX_CARTWALL_SHARE, MIN_CARTWALL_SHARE, ModelError, Panel, PlayerId, Transport, apply,
};
```

and append:

```rust
fn hide(s: &mut AppState, panel: Panel, hidden: bool) -> Result<(), ModelError> {
    apply(s, Command::SetPanelHidden { panel, hidden }).map(|_| ())
}

#[test]
fn rl6_a_side_dock_gets_the_default_share_and_the_bottom_fits_the_content() {
    let mut s = fixture(0);
    apply(&mut s, Command::SetCartwallDock(Dock::Left)).unwrap();
    assert_eq!(s.layout.cartwall.dock, Dock::Left);
    assert_eq!(s.layout.cartwall.size, Some(DEFAULT_SIDE_SHARE));
    apply(&mut s, Command::SetCartwallSize(Some(0.5))).unwrap();
    apply(&mut s, Command::SetCartwallDock(Dock::Right)).unwrap();
    assert_eq!(
        s.layout.cartwall.size,
        Some(DEFAULT_SIDE_SHARE),
        "a dock change resets the size"
    );
    apply(&mut s, Command::SetCartwallDock(Dock::Bottom)).unwrap();
    assert_eq!(s.layout.cartwall.size, None);
}

#[test]
fn rl6_docking_where_it_already_is_keeps_the_size() {
    let mut s = fixture(0);
    apply(&mut s, Command::SetCartwallSize(Some(0.4))).unwrap();
    apply(&mut s, Command::SetCartwallDock(Dock::Bottom)).unwrap();
    assert_eq!(s.layout.cartwall.size, Some(0.4));
}

#[test]
fn rl6_the_size_is_clamped() {
    let mut s = fixture(0);
    apply(&mut s, Command::SetCartwallSize(Some(0.0))).unwrap();
    assert_eq!(s.layout.cartwall.size, Some(MIN_CARTWALL_SHARE));
    apply(&mut s, Command::SetCartwallSize(Some(2.0))).unwrap();
    assert_eq!(s.layout.cartwall.size, Some(MAX_CARTWALL_SHARE));
    apply(&mut s, Command::SetCartwallSize(None)).unwrap();
    assert_eq!(s.layout.cartwall.size, None, "fit the content at the bottom");
    apply(&mut s, Command::SetCartwallDock(Dock::Left)).unwrap();
    apply(&mut s, Command::SetCartwallSize(None)).unwrap();
    assert_eq!(
        s.layout.cartwall.size,
        Some(DEFAULT_SIDE_SHARE),
        "a side has no content size"
    );
    let before = s.clone();
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let err = apply(&mut s, Command::SetCartwallSize(Some(bad))).unwrap_err();
        assert_eq!(err, ModelError::BadLayout);
        assert_eq!(s, before);
    }
}

#[test]
fn rl6_the_cartwall_share_is_usable_on_every_dock() {
    let at = |dock, size| {
        CartwallLayout {
            dock,
            size,
            hidden: false,
        }
        .share()
    };
    assert_eq!(at(Dock::Bottom, None), None);
    assert_eq!(at(Dock::Bottom, Some(0.4)), Some(0.4));
    assert_eq!(at(Dock::Bottom, Some(f32::NAN)), None);
    assert_eq!(at(Dock::Left, None), Some(DEFAULT_SIDE_SHARE));
    assert_eq!(at(Dock::Right, Some(5.0)), Some(MAX_CARTWALL_SHARE));
    assert_eq!(at(Dock::Left, Some(-1.0)), Some(MIN_CARTWALL_SHARE));
}

#[test]
fn rl7_hiding_and_showing_a_panel() {
    let mut s = fixture(0);
    let p = s.player_ids();
    hide(&mut s, Panel::Player(p[1]), true).unwrap();
    hide(&mut s, Panel::Player(p[1]), true).unwrap();
    assert_eq!(s.layout.hidden_players, vec![p[1]], "hiding twice is hiding once");
    hide(&mut s, Panel::Cartwall, true).unwrap();
    assert!(s.layout.cartwall.hidden);
    hide(&mut s, Panel::Player(p[1]), false).unwrap();
    hide(&mut s, Panel::Cartwall, false).unwrap();
    assert_eq!(s.layout, Layout::default());
}

#[test]
fn rl7_the_last_shown_panel_cannot_be_hidden() {
    let mut s = fixture(0);
    let p = s.player_ids();
    for panel in [
        Panel::Cartwall,
        Panel::Player(p[0]),
        Panel::Player(p[1]),
        Panel::Player(p[2]),
    ] {
        hide(&mut s, panel, true).unwrap();
    }
    let before = s.clone();
    assert_eq!(
        hide(&mut s, Panel::Player(p[3]), true).unwrap_err(),
        ModelError::LastPanel
    );
    assert_eq!(s, before);
    hide(&mut s, Panel::Cartwall, false).unwrap();
    hide(&mut s, Panel::Player(p[3]), true).unwrap();
}

#[test]
fn rl7_hiding_a_player_that_does_not_exist_is_refused() {
    let mut s = fixture(0);
    let stranger = PlayerId(999_999);
    assert_eq!(
        hide(&mut s, Panel::Player(stranger), true).unwrap_err(),
        ModelError::UnknownPlayer(stranger)
    );
}

#[test]
fn rl7_a_hidden_player_keeps_playing() {
    let mut s = fixture(3);
    let p = p0(&s);
    apply(&mut s, Command::Play(p)).unwrap();
    let actions = apply(
        &mut s,
        Command::SetPanelHidden {
            panel: Panel::Player(p),
            hidden: true,
        },
    )
    .unwrap();
    assert!(actions.is_empty(), "{actions:?}");
    assert_eq!(s.player(p).unwrap().transport, Transport::Playing);
    let actions = apply(&mut s, Command::Stop(p)).unwrap();
    assert!(!actions.is_empty(), "commands still reach a hidden player");
    assert_eq!(s.player(p).unwrap().transport, Transport::Stopped);
}

#[test]
fn rl7_collapsing_the_cartwall_works_while_locked() {
    let mut s = fixture(0);
    apply(&mut s, Command::SetLayoutLocked(true)).unwrap();
    apply(&mut s, Command::SetCartwallOpen(false)).unwrap();
    assert!(!s.cartwall.open);
}

#[test]
fn rl9_a_locked_layout_rejects_every_layout_command() {
    let mut s = fixture(0);
    let p = s.player_ids();
    apply(&mut s, Command::SetLayoutLocked(true)).unwrap();
    assert!(s.layout.locked);
    let before = s.clone();
    for command in [
        Command::SetPlayerShares {
            left: (p[0], 1.5),
            right: (p[1], 0.5),
        },
        Command::SetPlayerShares {
            left: (PlayerId(999_999), 1.0),
            right: (p[1], 1.0),
        },
        Command::EqualPlayerWidths,
        Command::SetCartwallDock(Dock::Left),
        Command::SetCartwallSize(Some(0.4)),
        Command::SetPanelHidden {
            panel: Panel::Cartwall,
            hidden: true,
        },
        Command::ResetLayout,
    ] {
        let err = apply(&mut s, command.clone()).unwrap_err();
        assert_eq!(err, ModelError::LayoutLocked, "{command:?}");
        assert_eq!(s, before);
    }
    apply(&mut s, Command::SetLayoutLocked(true)).unwrap();
    apply(&mut s, Command::SetLayoutLocked(false)).unwrap();
    apply(&mut s, Command::SetCartwallDock(Dock::Left)).unwrap();
}

#[test]
fn rl10_reset_gives_the_default_layout() {
    let mut s = fixture(0);
    let p = s.player_ids();
    apply(
        &mut s,
        Command::SetPlayerShares {
            left: (p[0], 1.5),
            right: (p[1], 0.5),
        },
    )
    .unwrap();
    hide(&mut s, Panel::Player(p[2]), true).unwrap();
    apply(&mut s, Command::SetCartwallDock(Dock::Right)).unwrap();
    hide(&mut s, Panel::Cartwall, true).unwrap();
    apply(&mut s, Command::ResetLayout).unwrap();
    assert_eq!(s.layout, Layout::default());
    assert!(!s.layout.locked);
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-model --test layout`
Expected: FAIL to compile: no variants `SetCartwallDock`, `SetCartwallSize`, `SetPanelHidden`, `SetLayoutLocked`, `ResetLayout`.

- [ ] **Step 3: Commands and rules**

`crates/fp-model/src/command.rs`: add `use crate::layout::{Dock, Panel};` and, after `EqualPlayerWidths,`:

```rust
    /// RL6: moves the cartwall. A new dock resets its size: fit the content
    /// at the bottom, the default side share on a side. The same dock
    /// changes nothing.
    SetCartwallDock(Dock),
    /// RL6: the cartwall's share of the middle area, clamped to
    /// `MIN_CARTWALL_SHARE ..= MAX_CARTWALL_SHARE`; `None` fits the content
    /// at the bottom and is the default share on a side.
    SetCartwallSize(Option<f32>),
    /// RL7: hides or shows a panel. Never touches playback.
    SetPanelHidden { panel: Panel, hidden: bool },
    /// RL9: while locked, every other layout command is refused.
    SetLayoutLocked(bool),
    /// RL10: back to the default layout (unlocked only).
    ResetLayout,
```

`crates/fp-model/src/layout.rs`, at the end:

```rust
/// RL6: a new dock resets the size.
pub(crate) fn set_cartwall_dock(state: &mut AppState, dock: Dock) -> Result<(), ModelError> {
    unlocked(state)?;
    let cartwall = &mut state.layout.cartwall;
    if cartwall.dock != dock {
        cartwall.dock = dock;
        cartwall.size = dock.is_side().then_some(DEFAULT_SIDE_SHARE);
    }
    Ok(())
}

/// RL6: the size, clamped; `None` is the default share on a side.
pub(crate) fn set_cartwall_size(
    state: &mut AppState,
    size: Option<f32>,
) -> Result<(), ModelError> {
    unlocked(state)?;
    if size.is_some_and(|s| !s.is_finite()) {
        return Err(ModelError::BadLayout);
    }
    let cartwall = &mut state.layout.cartwall;
    cartwall.size = CartwallLayout {
        size,
        ..*cartwall
    }
    .share();
    Ok(())
}

/// RL7: hides or shows `panel`; the last shown panel stays.
pub(crate) fn set_panel_hidden(
    state: &mut AppState,
    panel: Panel,
    hidden: bool,
) -> Result<(), ModelError> {
    unlocked(state)?;
    if let Panel::Player(id) = panel {
        state.player(id)?;
    }
    let players = state.player_ids();
    let layout = &mut state.layout;
    if layout.is_hidden(panel) == hidden {
        return Ok(());
    }
    if hidden && layout.is_last_shown(&players, panel) {
        return Err(ModelError::LastPanel);
    }
    match panel {
        Panel::Player(id) if hidden => layout.hidden_players.push(id),
        Panel::Player(id) => layout.hidden_players.retain(|p| *p != id),
        Panel::Cartwall => layout.cartwall.hidden = hidden,
    }
    Ok(())
}

/// RL10: the default layout, unlocked.
pub(crate) fn reset_layout(state: &mut AppState) -> Result<(), ModelError> {
    unlocked(state)?;
    state.layout = Layout::default();
    Ok(())
}
```

(`set_cartwall_size` reuses `CartwallLayout::share`, which already gives the side default and the clamp.)

`crates/fp-model/src/reducer.rs`, after Task 2's arms:

```rust
        Command::SetCartwallDock(dock) => crate::layout::set_cartwall_dock(state, dock)?,
        Command::SetCartwallSize(size) => crate::layout::set_cartwall_size(state, size)?,
        Command::SetPanelHidden { panel, hidden } => {
            crate::layout::set_panel_hidden(state, panel, hidden)?
        }
        Command::SetLayoutLocked(locked) => state.layout.locked = locked,
        Command::ResetLayout => crate::layout::reset_layout(state)?,
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p fp-model --test layout`
Expected: PASS.

- [ ] **Step 5: Gate and commit**

Run the gate. Then:

```bash
git add crates/fp-model/src crates/fp-model/tests/layout.rs
git commit -m "feat(model): cartwall dock and size, hidden panels, layout lock and reset

RL6, RL7, RL9 and RL10. A new dock resets the cartwall's size, the
size is clamped to 0.05..=0.95, hiding never touches playback and the
last shown panel stays, a locked layout refuses every layout command
but the lock itself, and Reset goes back to today's window.
SetCartwallOpen is not a layout command and works while locked.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: Players come and go; the layout repair (RL3)

**Files:**
- Modify: `crates/fp-model/src/layout.rs`
- Modify: `crates/fp-model/src/lib.rs` (the `pub use layout::…` line)
- Modify: `crates/fp-model/src/reducer.rs` (`set_player_count`, L1304-1336)
- Test: `crates/fp-model/tests/layout.rs`

**Interfaces:**
- Consumes: Tasks 1–3.
- Produces: `pub fn fp_model::settle_layout(layout: &mut Layout, players: &[PlayerId]) -> Vec<String>` — the RL3/RL19 repair; returns one note per repair (each starts with `"layout: "`), empty when nothing changed; idempotent. `fp-store` (Task 6) prefixes `"session: "` and logs them.

- [ ] **Step 1: Write the failing tests**

Add `settle_layout` to the `use fp_model::{…}` list in `crates/fp-model/tests/layout.rs` and append:

```rust
#[test]
fn rl3_fewer_players_drop_their_shares_and_hidden_flags() {
    let mut s = fixture(0);
    let p = s.player_ids();
    apply(
        &mut s,
        Command::SetPlayerShares {
            left: (p[2], 1.5),
            right: (p[3], 0.5),
        },
    )
    .unwrap();
    hide(&mut s, Panel::Player(p[3]), true).unwrap();
    apply(&mut s, Command::SetPlayerCount(2)).unwrap();
    assert!(s.layout.player_shares.iter().all(|(id, _)| p[..2].contains(id)));
    assert!(s.layout.hidden_players.is_empty());
}

#[test]
fn rl3_a_lower_max_players_drops_them_too() {
    let mut s = fixture(0);
    let p = s.player_ids();
    apply(&mut s, Command::EqualPlayerWidths).unwrap();
    hide(&mut s, Panel::Player(p[3]), true).unwrap();
    let mut config = s.config.clone();
    config.limits.max_players = 2;
    apply(&mut s, Command::UpdateConfig(Box::new(config))).unwrap();
    assert_eq!(s.player_ids(), p[..2].to_vec());
    assert_eq!(s.layout.player_shares, vec![(p[0], 1.0), (p[1], 1.0)]);
    assert!(s.layout.hidden_players.is_empty());
}

#[test]
fn rl3_new_players_are_shown_with_an_equal_slice() {
    let mut s = fixture(0);
    let p = s.player_ids();
    hide(&mut s, Panel::Player(p[0]), true).unwrap();
    apply(&mut s, Command::SetPlayerCount(6)).unwrap();
    let now = s.player_ids();
    assert_eq!(s.layout.shown_players(&now), now[1..].to_vec());
    assert!(close(s.layout.share(&now, now[5]), 1.0));
}

#[test]
fn rl3_when_no_panel_is_left_every_panel_is_shown() {
    let mut s = fixture(0);
    let p = s.player_ids();
    for panel in [
        Panel::Cartwall,
        Panel::Player(p[0]),
        Panel::Player(p[1]),
        Panel::Player(p[2]),
    ] {
        hide(&mut s, panel, true).unwrap();
    }
    apply(&mut s, Command::SetPlayerCount(3)).unwrap();
    assert!(s.layout.hidden_players.is_empty());
    assert!(!s.layout.cartwall.hidden);
}

#[test]
fn rl19_settle_keeps_the_first_of_a_repeated_player() {
    let p = [PlayerId(1), PlayerId(2)];
    let mut layout = Layout {
        player_shares: vec![(p[0], 2.0), (p[0], 5.0), (p[1], -1.0), (p[1], 3.0)],
        hidden_players: vec![p[1], p[1]],
        ..Layout::default()
    };
    let notes = settle_layout(&mut layout, &p);
    assert_eq!(layout.player_shares, vec![(p[0], 2.0), (p[1], 3.0)]);
    assert_eq!(layout.hidden_players, vec![p[1]]);
    assert_eq!(notes.len(), 3, "{notes:?}");
    assert!(
        settle_layout(&mut layout, &p).is_empty(),
        "settling twice changes nothing"
    );
}

#[test]
fn rl19_settle_fits_the_cartwall_size_to_its_dock() {
    let p = [PlayerId(1)];
    let mut layout = Layout {
        cartwall: CartwallLayout {
            dock: Dock::Left,
            size: None,
            hidden: false,
        },
        ..Layout::default()
    };
    assert_eq!(settle_layout(&mut layout, &p).len(), 1);
    assert_eq!(layout.cartwall.size, Some(DEFAULT_SIDE_SHARE));
    layout.cartwall = CartwallLayout {
        dock: Dock::Bottom,
        size: Some(5.0),
        hidden: false,
    };
    assert_eq!(settle_layout(&mut layout, &p).len(), 1);
    assert_eq!(layout.cartwall.size, Some(MAX_CARTWALL_SHARE));
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-model --test layout`
Expected: FAIL to compile: no `settle_layout` in `fp_model`.

- [ ] **Step 3: The repair**

`crates/fp-model/src/layout.rs`, at the end:

```rust
/// RL3 and RL19's repairs, for `players` (`AppState::player_ids`): drops
/// the shares and hidden flags of players that are not in `players`, a
/// second share or flag of the same player (the first wins) and shares
/// that are not finite and above 0; shows every panel when none is shown;
/// fits the cartwall's size to its dock (`CartwallLayout::share`).
/// Returns one note per repair, for the load warnings. Idempotent.
pub fn settle_layout(layout: &mut Layout, players: &[PlayerId]) -> Vec<String> {
    let mut notes = Vec::new();
    let mut seen = Vec::new();
    layout.player_shares.retain(|(id, share)| {
        let keep = if !players.contains(id) {
            notes.push(format!(
                "layout: the share of player {} was dropped: there is no such player",
                id.0
            ));
            false
        } else if seen.contains(id) {
            notes.push(format!(
                "layout: a second share of player {} was dropped",
                id.0
            ));
            false
        } else if !usable_share(*share) {
            notes.push(format!(
                "layout: the share {share} of player {} is not above 0; it was dropped",
                id.0
            ));
            false
        } else {
            true
        };
        if keep {
            seen.push(*id);
        }
        keep
    });
    let mut seen = Vec::new();
    layout.hidden_players.retain(|id| {
        let keep = if !players.contains(id) {
            notes.push(format!(
                "layout: the hidden flag of player {} was dropped: there is no such player",
                id.0
            ));
            false
        } else if seen.contains(id) {
            notes.push(format!("layout: player {} was hidden twice", id.0));
            false
        } else {
            true
        };
        if keep {
            seen.push(*id);
        }
        keep
    });
    if layout.shown_panels(players) == 0 {
        layout.hidden_players.clear();
        layout.cartwall.hidden = false;
        notes.push("layout: every panel was hidden; all are shown again".to_owned());
    }
    let fitted = layout.cartwall.share();
    if fitted.map(f32::to_bits) != layout.cartwall.size.map(f32::to_bits) {
        notes.push(format!(
            "layout: the cartwall's size {:?} does not fit its dock; it is now {fitted:?}",
            layout.cartwall.size
        ));
        layout.cartwall.size = fitted;
    }
    notes
}
```

(Comparing bit patterns avoids a float comparison; `share` returns `size` unchanged when it is already usable.)

`crates/fp-model/src/lib.rs`: add `settle_layout` to the `pub use layout::{…}` list.

`crates/fp-model/src/reducer.rs`, `set_player_count`: replace the closing

```rust
    state.config.players.count = count;
    Ok(())
}
```

with

```rust
    state.config.players.count = count;
    // RL3: removed players take their shares and hidden flags with them;
    // new players are shown; with no panel left shown, all are.
    let players = state.player_ids();
    let _ = crate::layout::settle_layout(&mut state.layout, &players);
    Ok(())
}
```

(`update_config` lowers the count through `set_player_count`, so a lower `limits.max_players` is covered.)

- [ ] **Step 4: Run the tests**

Run: `cargo test -p fp-model --test layout`
Expected: PASS.

- [ ] **Step 5: Gate and commit**

Run the gate. Then:

```bash
git add crates/fp-model/src crates/fp-model/tests/layout.rs
git commit -m "feat(model): removed players leave the layout, new ones are shown

RL3: SetPlayerCount, and an UpdateConfig that lowers max_players, drop
the removed players' shares and hidden flags, and show every panel when
none is left shown. settle_layout is the one repair; the session restore
uses it too, and it reports each repair for the load warnings.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: The layout in the restore, read leniently (RL19, model side)

**Files:**
- Modify: `crates/fp-model/src/layout.rs` (serde attributes, lenient deserialisers)
- Modify: `crates/fp-model/src/session.rs` (`RestoreParts` L94-103; `restore` L131-261)
- Modify: `crates/fp-model/tests/common/mod.rs` (`roundtrip`, L54-63)
- Modify: `RestoreParts` literals in `crates/fp-model/tests/session.rs` (L13, L89, L153), `tests/pending_start.rs` (L318), `tests/loudness.rs` (L951), `tests/cue_markers_off.rs` (L262), `tests/review_fixes.rs` (L15)
- Modify: `crates/fp-store/src/store.rs` (the two `RestoreParts { … }` literals, L157 and L266: pass `layout: Default::default()` for now; Task 6 passes the saved one)
- Test: `crates/fp-model/tests/layout.rs`

**Interfaces:**
- Consumes: Task 4's `settle_layout`.
- Produces:
  - `RestoreParts::layout: Layout` (the saved layout; `Default` is today's).
  - `AppState::restore` applies it after building the players and runs `settle_layout` on it.
  - `pub fn fp_model::layout::lenient_layout<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Layout, D::Error>` — a `deserialize_with` for a `layout` field: anything that is not an object is `Layout::default()`. Inside, each field, each part of `cartwall`, and each element of `player_shares` and `hidden_players` falls back on its own.

- [ ] **Step 1: Write the failing tests**

Add `RestoreParts` to the `use fp_model::{…}` list in `crates/fp-model/tests/layout.rs` and append:

```rust
fn restore_with(state: &AppState, layout: Layout) -> AppState {
    let parts = RestoreParts {
        config: state.config.clone(),
        library: state.library.clone(),
        playlists: state.playlists.clone(),
        cart_pages: state.cartwall.pages.clone(),
        cartwall_session: state.cartwall.session(),
        layout,
        ids: state.ids.clone(),
    };
    AppState::restore(parts, &state.sessions(|_| 0.0), "Main").0
}

#[test]
fn rl19_the_layout_survives_a_restart() {
    let mut s = fixture(0);
    let p = s.player_ids();
    apply(
        &mut s,
        Command::SetPlayerShares {
            left: (p[0], 1.5),
            right: (p[1], 0.5),
        },
    )
    .unwrap();
    hide(&mut s, Panel::Player(p[2]), true).unwrap();
    apply(&mut s, Command::SetCartwallDock(Dock::Right)).unwrap();
    apply(&mut s, Command::SetLayoutLocked(true)).unwrap();
    assert_eq!(common::roundtrip(&s).layout, s.layout);
}

#[test]
fn rl19_restore_drops_players_that_do_not_exist_and_shows_everything_hidden() {
    let s = fixture(0);
    let p = s.player_ids();
    let stranger = PlayerId(999_999);
    let layout = Layout {
        player_shares: vec![(p[0], 2.0), (stranger, 5.0)],
        hidden_players: vec![p[0], p[1], p[2], p[3], stranger],
        cartwall: CartwallLayout {
            dock: Dock::Left,
            size: None,
            hidden: true,
        },
        locked: true,
    };
    let restored = restore_with(&s, layout);
    assert_eq!(restored.layout.player_shares, vec![(p[0], 2.0)]);
    assert!(restored.layout.hidden_players.is_empty());
    assert!(!restored.layout.cartwall.hidden);
    assert_eq!(restored.layout.cartwall.size, Some(DEFAULT_SIDE_SHARE));
    assert!(restored.layout.locked, "the lock is kept");
}

#[test]
fn rl19_a_session_saved_with_more_players_drops_the_extra_ones() {
    let mut s = fixture(0);
    let p = s.player_ids();
    apply(&mut s, Command::EqualPlayerWidths).unwrap();
    hide(&mut s, Panel::Player(p[3]), true).unwrap();
    let layout = s.layout.clone();
    s.config.players.count = 3;
    let restored = restore_with(&s, layout);
    assert_eq!(restored.player_ids(), p[..3].to_vec());
    assert_eq!(
        restored.layout.player_shares,
        vec![(p[0], 1.0), (p[1], 1.0), (p[2], 1.0)]
    );
    assert!(restored.layout.hidden_players.is_empty());
}

#[test]
fn rl19_each_part_of_the_layout_loads_leniently() {
    let layout: Layout = serde_json::from_str(
        r#"{"player_shares": [[1, 2.0], ["x", 1.0], [2]],
            "hidden_players": "none",
            "cartwall": {"dock": "Top", "size": 0.4, "hidden": true},
            "locked": "yes"}"#,
    )
    .unwrap();
    assert_eq!(layout.player_shares, vec![(PlayerId(1), 2.0)]);
    assert!(layout.hidden_players.is_empty());
    assert_eq!(
        layout.cartwall,
        CartwallLayout {
            dock: Dock::Bottom,
            size: Some(0.4),
            hidden: true
        }
    );
    assert!(!layout.locked);
}

#[test]
fn rl19_a_layout_that_is_not_an_object_loads_as_the_default() {
    #[derive(serde::Deserialize)]
    struct Doc {
        #[serde(default, deserialize_with = "fp_model::layout::lenient_layout")]
        layout: Layout,
    }
    for json in [
        r#"{}"#,
        r#"{"layout": 5}"#,
        r#"{"layout": null}"#,
        r#"{"layout": {"cartwall": "left"}}"#,
    ] {
        let doc: Doc = serde_json::from_str(json).unwrap();
        assert_eq!(doc.layout, Layout::default(), "{json}");
    }
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-model --test layout`
Expected: FAIL to compile: no field `layout` in `RestoreParts`, no `fp_model::layout::lenient_layout`.

- [ ] **Step 3: Lenient deserialisers**

`crates/fp-model/src/layout.rs`: add `use serde::Deserializer;` to the imports and, after the constants:

```rust
/// One value read leniently: anything that does not parse is `Other`.
#[derive(Deserialize)]
#[serde(untagged)]
enum Lenient<T> {
    Valid(T),
    Other(serde::de::IgnoredAny),
}

impl<T> Lenient<T> {
    fn ok(self) -> Option<T> {
        match self {
            Lenient::Valid(v) => Some(v),
            Lenient::Other(_) => None,
        }
    }
}

/// RL19: a part that does not parse loads as its default.
fn lenient<'de, D: Deserializer<'de>, T: Deserialize<'de> + Default>(
    d: D,
) -> Result<T, D::Error> {
    Ok(Lenient::<T>::deserialize(d)?.ok().unwrap_or_default())
}

/// RL19: a list keeps every element that parses; a value that is not a
/// list is empty.
fn lenient_items<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> Result<Vec<T>, D::Error> {
    let items = Lenient::<Vec<Lenient<T>>>::deserialize(d)?
        .ok()
        .unwrap_or_default();
    Ok(items.into_iter().filter_map(Lenient::ok).collect())
}

/// RL19: the `deserialize_with` of a session's `layout`: a value that is
/// not a layout loads as the default, and inside it each part falls back
/// on its own. The store reports what fell back (`fp-store`'s
/// `layout_notes`).
pub fn lenient_layout<'de, D: Deserializer<'de>>(d: D) -> Result<Layout, D::Error> {
    lenient(d)
}
```

Then give the fields their deserialisers. `CartwallLayout`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CartwallLayout {
    #[serde(deserialize_with = "lenient")]
    pub dock: Dock,
    /// The cartwall's share of the middle area: of its height at the
    /// bottom, of its width on a side. `None` fits the content (bottom
    /// only).
    #[serde(deserialize_with = "lenient")]
    pub size: Option<f32>,
    #[serde(deserialize_with = "lenient")]
    pub hidden: bool,
}
```

`Layout`:

```rust
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Layout {
    /// Each player's share of the players' width (RL2).
    #[serde(deserialize_with = "lenient_items")]
    pub player_shares: Vec<(PlayerId, f32)>,
    #[serde(deserialize_with = "lenient_items")]
    pub hidden_players: Vec<PlayerId>,
    #[serde(deserialize_with = "lenient")]
    pub cartwall: CartwallLayout,
    #[serde(deserialize_with = "lenient")]
    pub locked: bool,
}
```

- [ ] **Step 4: The restore**

`crates/fp-model/src/session.rs`: add `use crate::layout::Layout;` and, in `RestoreParts`, after `cartwall_session`:

```rust
    /// The saved layout (resizable layout spec RL19); `restore` repairs it
    /// for the players it builds.
    pub layout: Layout,
```

In `restore`, add `layout,` to the `let RestoreParts { … } = parts;` pattern (after `cartwall_session,`), keep `layout: Layout::default()` in the `AppState { … }` literal, and right after the `for k in 0..state.config.players.count { … }` loop insert:

```rust
        // RL19: the saved layout, repaired for the players this start has
        // (`fp-store` reports the repairs).
        state.layout = layout;
        let players = state.player_ids();
        let _ = crate::layout::settle_layout(&mut state.layout, &players);
```

- [ ] **Step 5: Every `RestoreParts` literal gets a layout**

- `crates/fp-model/tests/common/mod.rs` `roundtrip`: after `cartwall_session: state.cartwall.session(),` add `layout: state.layout.clone(),`.
- `crates/fp-model/tests/session.rs` L13 (`parts`), `tests/review_fixes.rs` L15 (`parts`), `tests/pending_start.rs` L318, `tests/loudness.rs` L951, `tests/cue_markers_off.rs` L262: after the `cartwall_session` line add `layout: state.layout.clone(),` (in `loudness.rs` the state is `s`: `layout: s.layout.clone(),`).
- `crates/fp-model/tests/session.rs` L89 and L153 (no state at hand): add `layout: fp_model::Layout::default(),`.
- `crates/fp-store/src/store.rs` L157 (`fuzz_documents`) and L266 (`Store::load`): after `cartwall_session,` add `layout: Default::default(),`.

Run `grep -rn "RestoreParts {" crates fuzz --include=*.rs` and check every hit has `layout`.

- [ ] **Step 6: Run the tests**

Run: `cargo test -p fp-model`
Expected: PASS (the whole model suite: the session and roundtrip tests now carry the layout).

- [ ] **Step 7: Gate and commit**

Run the gate. Then:

```bash
git add crates/fp-model crates/fp-store/src/store.rs
git commit -m "feat(model): the layout comes back after a restart, repaired

RL19, model side: RestoreParts carries the saved layout and the restore
applies it after it builds the players, dropping players that no longer
exist and showing everything when all was hidden. lenient_layout reads
it part by part: a part that does not parse loads as its default and the
rest is kept.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: The layout in `session.json` (RL19, store side); persistence docs

**Files:**
- Create: `crates/fp-store/src/layout_notes.rs`
- Modify: `crates/fp-store/src/lib.rs` (module list L4-11)
- Modify: `crates/fp-store/src/docs.rs` (imports L6; `SessionDoc` L47-54)
- Modify: `crates/fp-store/src/store.rs` (`parse_session` L47-74, `fuzz_documents` L144-170, `Store::load` L240-286, `Store::save_session` L330-345)
- Modify: `docs/technical/persistence.md` (the `data/session.json` row, L10; "Migrations", L93-97)
- Test: `crates/fp-store/tests/session_layout.rs` (new)

**Interfaces:**
- Consumes: Task 5's `RestoreParts::layout`, `fp_model::layout::lenient_layout`; Task 4's `settle_layout`; `crate::lenient::quoted(&Value) -> String`.
- Produces: `SessionDoc::layout: Layout` (`#[serde(default, deserialize_with = "fp_model::layout::lenient_layout")]`); `session.json` gains `"layout": { "player_shares": [[id, share], …], "hidden_players": [id, …], "cartwall": { "dock": "Bottom"|"Left"|"Right", "size": number|null, "hidden": bool }, "locked": bool }`; `SESSION_SCHEMA` stays 1. Load warnings start with `"session: layout"` (parts that do not parse) or `"session: layout: "` (repairs).

- [ ] **Step 1: Write the failing tests**

Create `crates/fp-store/tests/session_layout.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Resizable layout spec RL19: the layout in `session.json`.

use std::fs;

use fp_model::{
    AppState, Command, Config, DEFAULT_SIDE_SHARE, Dock, Layout, MAX_CARTWALL_SHARE, Panel,
    apply,
};
use fp_store::{AppPaths, Store};
use serde_json::json;

fn store(dir: &tempfile::TempDir) -> Store {
    Store::new(AppPaths::under(dir.path()), Config::default().limits)
}

/// Saves a state whose first player is at half volume and whose layout is
/// not the default: P1 wider than P2, P3 hidden, the cartwall on the
/// left, locked.
fn saved(s: &Store) -> AppState {
    let mut state = AppState::new(Config::default(), "Main");
    let p = state.player_ids();
    apply(&mut state, Command::SetVolume(p[0], 0.5)).unwrap();
    apply(
        &mut state,
        Command::SetPlayerShares {
            left: (p[0], 1.5),
            right: (p[1], 0.5),
        },
    )
    .unwrap();
    apply(
        &mut state,
        Command::SetPanelHidden {
            panel: Panel::Player(p[2]),
            hidden: true,
        },
    )
    .unwrap();
    apply(&mut state, Command::SetCartwallDock(Dock::Left)).unwrap();
    apply(&mut state, Command::SetLayoutLocked(true)).unwrap();
    s.save_config(&state).unwrap();
    s.save_playlists(&state).unwrap();
    s.save_session(&state, |_| 0.0).unwrap();
    state
}

fn edit_session(s: &Store, edit: impl FnOnce(&mut serde_json::Value)) {
    let path = s.paths().session_file();
    let mut doc: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    edit(&mut doc);
    fs::write(&path, serde_json::to_vec(&doc).unwrap()).unwrap();
}

/// The rest of the session was kept: same players, same volume.
fn session_kept(loaded: &AppState, saved: &AppState) {
    assert_eq!(loaded.player_ids(), saved.player_ids());
    assert_eq!(loaded.players[0].volume, 0.5);
}

#[test]
fn rl19_the_layout_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
    let state = saved(&s);
    let loaded = s.load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert_eq!(loaded.state.layout, state.layout);
}

#[test]
fn rl19_an_older_session_without_a_layout_loads_the_default() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
    let state = saved(&s);
    edit_session(&s, |doc| {
        doc.as_object_mut().unwrap().remove("layout");
    });
    let loaded = s.load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert_eq!(loaded.state.layout, Layout::default());
    session_kept(&loaded.state, &state);
}

#[test]
fn rl19_a_malformed_layout_keeps_the_rest_of_the_session() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
    let state = saved(&s);
    edit_session(&s, |doc| doc["layout"] = json!("wide"));
    let loaded = s.load("Main");
    assert_eq!(loaded.state.layout, Layout::default());
    session_kept(&loaded.state, &state);
    assert_eq!(loaded.warnings.len(), 1, "{:?}", loaded.warnings);
    assert!(loaded.warnings[0].contains("layout"), "{:?}", loaded.warnings);
}

#[test]
fn rl19_a_malformed_part_keeps_the_other_parts() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
    let state = saved(&s);
    edit_session(&s, |doc| {
        doc["layout"]["cartwall"]["dock"] = json!("Top");
        doc["layout"]["locked"] = json!("yes");
        doc["layout"]["player_shares"][0] = json!("x");
    });
    let loaded = s.load("Main");
    let layout = &loaded.state.layout;
    assert_eq!(layout.cartwall.dock, Dock::Bottom, "an unknown dock");
    assert!(!layout.locked);
    assert_eq!(layout.player_shares.len(), 3, "the other shares are kept");
    assert_eq!(layout.hidden_players, state.layout.hidden_players);
    session_kept(&loaded.state, &state);
    assert_eq!(loaded.warnings.len(), 3, "{:?}", loaded.warnings);
    assert!(
        loaded.warnings.iter().all(|w| w.starts_with("session: layout")),
        "{:?}",
        loaded.warnings
    );
}

#[test]
fn rl19_players_that_no_longer_exist_are_dropped_with_a_note() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
    let state = saved(&s);
    // The session has four players; the configuration now wants three.
    let mut fewer = state.clone();
    fewer.config.players.count = 3;
    s.save_config(&fewer).unwrap();
    edit_session(&s, |doc| {
        let layout = &mut doc["layout"];
        layout["player_shares"]
            .as_array_mut()
            .unwrap()
            .push(json!([999_999, 2.0]));
        layout["hidden_players"]
            .as_array_mut()
            .unwrap()
            .push(json!(999_999));
    });
    let loaded = s.load("Main");
    let players = loaded.state.player_ids();
    assert_eq!(players.len(), 3);
    let layout = &loaded.state.layout;
    assert!(layout.player_shares.iter().all(|(id, _)| players.contains(id)));
    assert!(layout.hidden_players.iter().all(|id| players.contains(id)));
    let dropped = loaded
        .warnings
        .iter()
        .filter(|w| w.contains("no such player"))
        .count();
    assert_eq!(dropped, 3, "{:?}", loaded.warnings);
    assert_eq!(loaded.warnings.len(), 3, "{:?}", loaded.warnings);
}

#[test]
fn rl19_everything_hidden_is_shown_with_a_note() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
    let state = saved(&s);
    let all: Vec<u64> = state.player_ids().iter().map(|p| p.0).collect();
    edit_session(&s, |doc| {
        doc["layout"]["hidden_players"] = json!(all);
        doc["layout"]["cartwall"]["hidden"] = json!(true);
    });
    let loaded = s.load("Main");
    assert!(loaded.state.layout.hidden_players.is_empty());
    assert!(!loaded.state.layout.cartwall.hidden);
    assert_eq!(loaded.warnings.len(), 1, "{:?}", loaded.warnings);
    assert!(loaded.warnings[0].contains("every panel"));
}

#[test]
fn rl19_a_side_dock_gets_a_usable_size_with_a_note() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
    saved(&s);
    edit_session(&s, |doc| doc["layout"]["cartwall"]["size"] = json!(null));
    let loaded = s.load("Main");
    assert_eq!(loaded.state.layout.cartwall.size, Some(DEFAULT_SIDE_SHARE));
    assert_eq!(loaded.warnings.len(), 1, "{:?}", loaded.warnings);

    saved(&s);
    edit_session(&s, |doc| doc["layout"]["cartwall"]["size"] = json!(5.0));
    let loaded = s.load("Main");
    assert_eq!(loaded.state.layout.cartwall.size, Some(MAX_CARTWALL_SHARE));
    assert_eq!(loaded.warnings.len(), 1, "{:?}", loaded.warnings);
}

#[test]
fn rl19_a_field_from_a_newer_build_is_ignored() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
    let state = saved(&s);
    edit_session(&s, |doc| doc["layout"]["named"] = json!([{"name": "Night"}]));
    let loaded = s.load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert_eq!(loaded.state.layout, state.layout);
}
```

(`saved` is called twice in one test: it writes fresh files over the old ones.)

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-store --test session_layout`
Expected: FAIL: the session file has no `layout` yet, so the round trip loads the default and the tests that edit `doc["layout"]` fail or panic; `rl19_an_older_session_without_a_layout_loads_the_default` may pass already.

- [ ] **Step 3: The document**

`crates/fp-store/src/docs.rs`: import `Layout` (`use fp_model::{CartPage, CartwallSession, Config, IdGen, Layout, Library, PlayerSession, Playlists};`) and add to `SessionDoc`, after `cartwall`:

```rust
    /// The main window's layout (resizable layout spec RL19). An older
    /// session has none (the default layout); each part that does not
    /// parse loads as its default. Older builds ignore the field, so the
    /// schema stays 1.
    #[serde(default, deserialize_with = "fp_model::layout::lenient_layout")]
    pub layout: Layout,
```

- [ ] **Step 4: The load notes**

Create `crates/fp-store/src/layout_notes.rs`:

```rust
//! Load notes for the session's `layout` (resizable layout spec RL19).
//! `fp_model::layout::lenient_layout` loads every part that does not parse
//! as its default; these notes say which, for the log.

use fp_model::{Dock, PlayerId};
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::lenient::quoted;

/// One note per part of `layout` (the raw `"layout"` value, if any) that
/// does not parse, and per element of its lists that does not.
pub(crate) fn layout_notes(layout: Option<&Value>) -> Vec<String> {
    let mut notes = Vec::new();
    let Some(layout) = layout else {
        return notes;
    };
    let Some(parts) = layout.as_object() else {
        notes.push(format!(
            "session: the saved layout {} is not valid; the default layout is used",
            quoted(layout)
        ));
        return notes;
    };
    items::<(PlayerId, f32)>(parts.get("player_shares"), "player_shares", &mut notes);
    items::<PlayerId>(parts.get("hidden_players"), "hidden_players", &mut notes);
    if let Some(cartwall) = parts.get("cartwall") {
        match cartwall.as_object() {
            Some(c) => {
                part::<Dock>(c.get("dock"), "cartwall.dock", &mut notes);
                part::<Option<f32>>(c.get("size"), "cartwall.size", &mut notes);
                part::<bool>(c.get("hidden"), "cartwall.hidden", &mut notes);
            }
            None => bad("cartwall", cartwall, &mut notes),
        }
    }
    part::<bool>(parts.get("locked"), "locked", &mut notes);
    notes
}

fn part<T: DeserializeOwned>(value: Option<&Value>, name: &str, notes: &mut Vec<String>) {
    if let Some(v) = value
        && serde_json::from_value::<T>(v.clone()).is_err()
    {
        bad(name, v, notes);
    }
}

fn items<T: DeserializeOwned>(value: Option<&Value>, name: &str, notes: &mut Vec<String>) {
    let Some(v) = value else {
        return;
    };
    match v.as_array() {
        Some(list) => {
            for item in list {
                if serde_json::from_value::<T>(item.clone()).is_err() {
                    notes.push(format!(
                        "session: layout.{name}: invalid element {} ignored",
                        quoted(item)
                    ));
                }
            }
        }
        None => bad(name, v, notes),
    }
}

fn bad(name: &str, value: &Value, notes: &mut Vec<String>) {
    notes.push(format!(
        "session: layout.{name}: invalid value {}; using the default",
        quoted(value)
    ));
}
```

`crates/fp-store/src/lib.rs`: add `mod layout_notes;` after `pub mod lenient;`.

- [ ] **Step 5: Parse, save, load, probe**

`crates/fp-store/src/store.rs`:

In `parse_session`, before `let doc: SessionDoc = serde_json::from_value(value)…`, add:

```rust
    let layout_notes = crate::layout_notes::layout_notes(value.get("layout"));
```

and replace the tail

```rust
    let notes = raw
        …
        .collect();
    Ok((doc, notes))
```

with

```rust
    let mut notes: Vec<String> = raw
        …
        .collect();
    notes.extend(layout_notes);
    Ok((doc, notes))
```

(keep the `…` chain as it is; only `let notes` becomes `let mut notes: Vec<String>` and the `extend` line is new).

In `fuzz_documents`:

```rust
    let (sessions, cartwall_session, layout) = session
        .map(|d| (d.players, d.cartwall, d.layout))
        .unwrap_or_default();
```

and in its `RestoreParts` replace Task 5's `layout: Default::default(),` with `layout,`.

In `Store::load`, replace

```rust
        let (sessions, cartwall_session) = session
            .value
            .map(|d| (d.players, d.cartwall))
            .unwrap_or_default();
```

with

```rust
        let (sessions, cartwall_session, layout) = session
            .value
            .map(|d| (d.players, d.cartwall, d.layout))
            .unwrap_or_default();
        // RL19: the restore repairs the layout; a copy says what it repaired.
        let saved_layout = layout.clone();
```

in its `RestoreParts` replace `layout: Default::default(),` with `layout,`, and right after the `let (mut state, actions) = AppState::restore(…);` statement add:

```rust
        // The restore ran the same repair for the same players.
        let mut probe = saved_layout;
        warnings.extend(
            fp_model::settle_layout(&mut probe, &state.player_ids())
                .into_iter()
                .map(|note| format!("session: {note}")),
        );
```

In `Store::save_session`, add `layout: state.layout.clone(),` after `cartwall: state.cartwall.session(),`.

- [ ] **Step 6: Run the tests**

Run: `cargo test -p fp-store`
Expected: PASS (the new file and the existing round-trip tests, whose warnings must stay empty).

- [ ] **Step 7: Persistence docs**

`docs/technical/persistence.md`, the `data/session.json` row: replace the closing ", `cartwall` (open, page shown) |" with:

```markdown
, `cartwall` (open, page shown), `layout` (resizable layout spec RL19: `player_shares` `[[player, share], …]`, each player's share of the players' width, a missing or non-positive one counting as the mean of the others; `hidden_players` `[player, …]`; `cartwall` `{ dock: "Bottom" | "Left" | "Right", size, hidden }`, where `size` is the cartwall's share of the middle area's height at the bottom or of its width on a side, clamped to 0.05–0.95, or `null` to fit the content at the bottom; `locked`. An older session has none and gets the default, today's window. It loads leniently, part by part and element by element: what does not parse is its default and the load reports a warning that is logged (`layout_notes`). The restore (`fp_model::settle_layout`) drops shares and hidden flags of players that do not exist, repeated ones and non-positive shares, shows every panel when all are hidden and fits the cartwall's size to its dock, each with a warning) |
```

"Migrations" section: after "with a test that loads a fixture of the old shape." add:

```markdown
A new field that loads as its default when it is missing, and that older
builds ignore, needs no bump: the session's `layout` (resizable layout
spec RL19) was added that way.
```

- [ ] **Step 8: Gate and commit**

Run the gate. Then:

```bash
git add crates/fp-store docs/technical/persistence.md
git commit -m "feat(store): save the layout in the session and report its repairs

RL19: session.json gains layout (SESSION_SCHEMA stays 1: older builds
ignore it, and an older session loads the default). Each part that does
not parse loads as its default with a load warning, and the warnings
also list what the restore repaired: players that no longer exist,
everything hidden, a cartwall size that does not fit its dock.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

## Self-review (against the spec)

- RL1: Task 1 (types, default, `AppState::layout`).
- RL2: Task 1 (missing, bad, stranger, hidden keeps its share, huge and tiny shares); neighbours as a §2 term.
- RL3: Task 4 (fewer players, lower `max_players`, new players shown, nothing left shown); the restore side in Tasks 5 and 6.
- RL4, RL5: Task 2 (neighbours across a hidden player, stranger, non-neighbour, reversed pair, bad shares, sum kept, equal widths).
- RL6: Task 3 (dock resets size, same dock keeps it, clamp, `None` on a side, non-finite refused, `CartwallLayout::share`).
- RL7: Task 3 (hide/show, idempotent, last panel, unknown player, a hidden player keeps playing, `SetCartwallOpen` while locked). The shortcut that does nothing while hidden and the top-bar toggle are plan 2.
- RL8: interface constants, plans 2 and 3.
- RL9, RL10: Task 3.
- `BadLayout`, `LastPanel`, `LayoutLocked` messages in every locale: Task 2.
- RL11–RL16: plans 2 and 3. RL17, RL18: plan 2 (interface state; ruling above).
- RL19: Task 5 (restore, lenient reading in the model), Task 6 (the document, notes, an old session, a malformed layout and part, unknown players, more players than `players.count`, an unknown dock, everything hidden, persistence docs).
- §6: no remote API or MIDI change; `fp-remote`'s `ApiError::from_model` maps the new errors through its `other =>` arm. `docs/technical/ui.md`, the user guide and the Phase 2 spec amendment are plan 3.

## What plans 2 and 3 consume

- Types and constants: `fp_model::layout::{Layout, CartwallLayout, Dock, Panel, DEFAULT_SIDE_SHARE, MIN_CARTWALL_SHARE, MAX_CARTWALL_SHARE}` (also at the crate root; in `fp-app`, which uses `egui::Layout`, write `fp_model::layout::Layout`). `Dock::ALL`, `Dock::is_side`.
- State: `AppState::layout`, `AppState::player_ids() -> Vec<PlayerId>`.
- Read helpers (pass `state.player_ids()`): `Layout::is_hidden(Panel) -> bool`, `Layout::shown_players(&[PlayerId]) -> Vec<PlayerId>`, `Layout::shown_panels(&[PlayerId]) -> usize`, `Layout::is_last_shown(&[PlayerId], Panel) -> bool` (disable that tile), `Layout::share(&[PlayerId], PlayerId) -> f32`, `Layout::shown_fractions(&[PlayerId]) -> Vec<(PlayerId, f32)>` (adds up to 1), `Layout::are_neighbours(&[PlayerId], left, right) -> bool`, `CartwallLayout::share() -> Option<f32>` (`None`: fit the content at the bottom).
- Commands: `SetPlayerShares { left: (PlayerId, f32), right: (PlayerId, f32) }` (any positive pair in the wanted ratio; never 0), `EqualPlayerWidths`, `SetCartwallDock(Dock)`, `SetCartwallSize(Option<f32>)`, `SetPanelHidden { panel: Panel, hidden: bool }`, `SetLayoutLocked(bool)`, `ResetLayout`.
- Errors: `ModelError::{BadLayout, LastPanel, LayoutLocked}`, already shown by `error_text` (`error-bad-layout`, `error-last-panel`, `error-layout-locked`).
