# Resizable Layout, Plan 3: the Cartwall's Dock, Splitter and Scaling

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The cartwall is drawn where the model docks it (RL6: at the bottom, or the full height on the left or the right) and never narrower than its header and its columns (RL8); the splitter between it and the players resizes it on every dock (RL12); its buttons fill a panel with a size of its own (RL13); collapsed on a side it is a narrow strip with its caret and an on-air dot (RL14). The guide, the technical docs, the Phase 2 spec, the README and the screenshots say so, and the whole feature (plans 1–3) is verified, reviewed and merged.

**Architecture:** Plan 2's pure pass `fp_app::ui::panel_layout::arrange` gains the side docks (`on_a_side`) behind its extension point, one range function (`cartwall_range`: the cartwall's least and most size along its axis) that the pass and the drag share, and two outputs: the splitter's gap (`Arrangement::splitter`) and whether the cartwall fits its content (`Arrangement::fit_content`). `CartwallSizes` gains the least width (`cartwall::min_width`, measured from the header's parts and the page's columns) and the collapsed thickness. The cartwall splitter reuses plan 2's `panels::splitter`; its drag (`panel_layout::CartwallDrag`, held in `ViewState::cartwall_drag`) follows the pointer before the pass, which draws a copy of the layout with the drag's share, and sends one `SetCartwallSize` on release. `cartwall::strip` sizes its buttons from `fit_content` and draws the collapsed side strip. Nothing in `fp-model` changes.

**Tech Stack:** Rust 2024, egui 0.36 (`Sense::click_and_drag`, `UiBuilder`, `ScrollArea::both`), `egui_kittest` 0.36, `egui-phosphor` 0.14, Bash and ImageMagick for the screenshots.

**Spec:** `docs/superpowers/specs/2026-10-21-resizable-layout-design.md` (§2, RL6, RL8, RL11–RL14, RL16, §6, §7 "Interface"). Read it whole before starting, then plan 1's "What plans 2 and 3 consume" and plan 2's "What plan 3 consumes" and its rulings: this plan uses their names and code as they leave them.

This is plan 3 of 3, executed strictly in order on one branch, `feat/resizable-layout`, after plans 1 and 2:
1. `2026-10-21-p13-plan1-layout-model.md` — the model, the session document, the error strings (done);
2. `2026-10-21-p13-plan2-layout-players-ui.md` — the pixel pass for the players and the bottom cartwall, the player splitters, the top-bar tiles and layout menu, the lock, maximise, RL17, RL18 (done);
3. **this plan** — the side docks on screen, the cartwall's least width, the cartwall splitter on every dock, RL13, RL14, the remaining docs, the screenshots, and the whole feature's verification, review and pull request.

Before this plan, a cartwall docked on a side is drawn at the bottom (plan 2's ruling), no splitter sets its size, and its buttons are at most 40 points tall.

## Rulings (planning)

Record each one in `.superpowers/sdd/itr05/progress.md` (section P13, plan 3) when the task that applies it is done.

**Ruling: one pure function, `panel_layout::cartwall_range`, gives the open cartwall's least and most size along its axis (height at the bottom, width on a side), and both the pass and the cartwall drag clamp to it.** The drag sends the share of the size it draws (`size / extent`), so after the model's echo the pass draws the dragged size again. — Why: RL11 and RL12 describe the same clamp; one function means the live drag and the next frame cannot disagree (the reason plan 2's `pair_shares` exists for the players). — Cost if wrong: one function inlined twice.

**Ruling: the cartwall splitter is inert while the cartwall is collapsed on any dock, not only on a side** (`cartwall_range` is `None` while collapsed). — Why: RL14 keeps a collapsed bottom cartwall "as today", only its header; a drag there would set a size nothing shows until it is expanded. The spec gets a clarifying sentence (Task 5). — Cost if wrong: the bottom arm returns a range while collapsed, and one test.

**Ruling: a collapsed cartwall at the bottom is its header even when it has a share.** Plan 2's `at_the_bottom` only met `size: None` (nothing set a share yet). — Why: RL14 "At the bottom, collapsing works as today: only the header is shown." — Cost if wrong: one match arm.

**Ruling: a collapsed cartwall is never maximised.** The pass ignores a maximised collapsed cartwall, the headers' menu disables **Maximise** on it, and `settle_view` ends a cartwall maximise when it collapses; expanding does not bring it back. Collapsing is not added to `ArrangementKey`: it does not end a player's maximise. — Why: a maximised collapsed cartwall would be an empty block taking the players' room; RL16's list of what ends a maximise stays as the spec gives it. — Cost if wrong: one filter, one condition, one branch.

**Ruling: the buttons fill the panel (no 40-point upper clamp) whenever the cartwall does not fit its content**, that is whenever `Arrangement::fit_content` is false: a share at the bottom, any side dock, a maximised cartwall, or the cartwall alone in the middle area. RL13's "None keeps today" holds exactly where the cartwall's height is its content height. — Why: RL13 says "no 40 px upper clamp when `size` is `Some`"; a maximised or lone cartwall at `None` would otherwise show two 40-point rows at the top of a large empty panel, which is what RL13 sets out to remove. The spec gets a clarifying sentence (Task 5). — Cost if wrong: `fit_content` drops two conditions.

**Ruling: RL13's "centred" is vertical.** In a button taller than `BUTTON_HEIGHT`, the text block of a 40-point button (the dot and name at its top, the kind and time at its bottom) is centred vertically; horizontally everything keeps its place. The waveform, the remaining-time fill and the CUE outline take the whole button. — Why: the names are left-aligned with their dot today, and the kind and time are at opposite ends; centring them horizontally would break the button's reading order. — Cost if wrong: one rectangle.

**Ruling: the cartwall's least width measures Stop all with a two-digit count** (`cartwall-stop-all-count`, `count` 99), not with the label of the moment. — Why: the label grows from "Stop all" to "Stop all (2)" as carts start; measured live, the least width (and a cartwall at its least width) would jump as carts start and stop, and a live drag would end on the range change. Three-digit counts can cut the hint, which is truncated anyway. — Cost if wrong: one argument.

**Ruling: `MIN_BUTTON_WIDTH` moves from `cartwall.rs` (private) to `cart_view.rs` (`pub`), beside `MIN_BUTTON_HEIGHT`.** — Why: RL13 names them together ("`MIN_BUTTON_WIDTH` × `MIN_BUTTON_HEIGHT`"), and the UI tests need it. — Cost if wrong: none.

**Ruling: a double-click on a side splitter sends `SetCartwallSize(Some(DEFAULT_SIDE_SHARE))`; at the bottom `SetCartwallSize(None)`.** Plan 1 also stores the default share for `None` on a side; the explicit share says in the command what the operator sees. — Cost if wrong: none.

**Ruling: the collapsed side strip adds no string.** Its caret is a tile labelled `cartwall-title` ("CARTWALL"), as the header's caret is, and points at the players (`CARET_RIGHT` on the left dock, `CARET_LEFT` on the right). — Why: the same control has the same name for a screen reader and for the existing tests; nothing new needs translating. — Cost if wrong: one string in eleven locales.

**Ruling: a side cartwall in a window narrower than its own least width takes the whole width of the middle area; the players' row then has no room and scrolls.** — Why: RL11 "The cartwall keeps its minimum", and a rectangle wider than the window would overlap the status bar's edges. — Cost if wrong: one `min`.

**Ruling: no new guide screenshot.** Every PNG in `docs/images` is retaken once, at the end (Task 6), because the top bar gained the tiles and the layout button in plan 2; the scene keeps the default layout (spec §6). — Cost if wrong: one more capture in `shoot_guide`.

**Ruling: the README gets a "Resizable layout" bullet and its Cartwall bullet changes; the website's landing page does not.** — Why: the landing page never said where the cartwall sits, so nothing on it becomes wrong. — Cost if wrong: one `<li>`.

## Global Constraints

- All code, identifiers, comments, docs and commit messages in English; never mention other playout or radio-automation products.
- Behaviour lives in `fp-model` (CLAUDE.md rule 7). Plan 3 adds no model rule: the dock and size rules are plan 1's (`SetCartwallDock`, `SetCartwallSize`, `CartwallLayout::share`); the interface computes rectangles and view state only.
- RL6: "Its `size` goes back to `None` at the bottom; on a side it becomes the default side share, 0.3 of the middle area's width." "Size. `Command::SetCartwallSize(Option<f32>)` sets the share, clamped to `0.05 ..= 0.95`."
- RL8: "the cartwall's width is at least the larger of its header's fixed parts ("Stop all", the caret, the smallest tab) and `cols` × the smallest cart button, plus the gaps"; "Minimum sizes come from the panels' content, so they are interface constants, not operator limits" — no `Config` field.
- RL11: "That size is clamped between the cartwall's minimum and what leaves the players their minimum (RL8)." "Only when the minimums alone do not fit does the players' row scroll horizontally, as today. The cartwall keeps its minimum."
- RL12: "The splitter between the players and the cartwall resizes the cartwall the same way (`SetCartwallSize`): horizontal at the bottom, vertical on a side. Double-clicking it goes back to "fit the content" at the bottom, or to the default share on a side." "While the layout is locked, or the cartwall is collapsed on a side, its splitter does not react, and the pointer over it shows the normal cursor." "When the drag ends, … is sent once, and only if the drag moved something. Until the model's echo arrives, the dragged widths are held."
- RL13: "Buttons never shrink below the smallest button (`MIN_BUTTON_WIDTH` × `MIN_BUTTON_HEIGHT`). When the panel is smaller than the grid needs, the grid scrolls." "At the bottom with `size: None`, the panel is exactly today's: the rows at `BUTTON_HEIGHT`, never more than 60 % of the middle area." "The name and the detail line stay at their sizes, centred. The waveform behind them grows with the button." "The grid limits stay `limits.max_cart_rows` and `limits.max_cart_cols`."
- RL14: "On a side, the cartwall becomes a narrow vertical strip, the width of the header's height, with the expand caret and, while a cart plays, an on-air dot. Expanding brings back the share it had."
- RL16: "A splitter drag first restores the earlier sizes, then applies itself."
- §6: "The README and guide screenshots use the default layout, which does not change."
- The UI never blocks (rule 8); never crash on bad data (rule 9); `unwrap`, `expect`, `panic` denied outside tests; prefer `get` over indexing; no `f32::clamp` on values that could be NaN (it panics): use `.min(…).max(…)`.
- In `fp-app`, write `fp_model::layout::Layout` (never a bare `Layout` import: `egui::Layout` is everywhere); `Panel`, `Dock` and `DEFAULT_SIDE_SHARE` come from `fp_model::layout`.
- Plan 3 adds no UI string (ruling). If an executor finds one is needed, it goes into all eleven locale files (`crates/fp-app/locales/{en-US,es-ES,ca-ES,de-DE,eu-ES,fr-FR,gl-ES,it-IT,nl-NL,pl-PL,pt-PT}/main.ftl`): en-US and es-ES written, the other nine AI-translated in the same change. `docs/i18n/` is never touched (no image moves in this plan).
- Test names start with the rule (`rl6_…`, `rl14_…`); UI tests use `egui_kittest` with the `support` harness (`with_step_dt(0.02)`), whose window has an 8-point outer margin: assert relations between rectangles, not absolute coordinates. The middle area of a harness window `W × H` is `(W − 32) × (H − 90)`.
- Conventional Commits, scope `ui` (docs-only commits `docs`); every commit ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- `cargo` is at `$HOME/.cargo/bin` (`export PATH="$HOME/.cargo/bin:$PATH"`). Gate for every task: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`. Unused imports in a test file fail clippy: each task gives the test file's whole `use` block.
- Progress ledger: `.superpowers/sdd/itr05/progress.md` (section P13, plan 3). Record deviations as `Ruling: <decision> — <why> — <cost if wrong>`.

## Review Focus

1. **A side dock in a window too narrow for it and the players** (four players and the default 2 × 8 page at 1920 × 1080 need 2042 points; or a window narrower than the cartwall's own least width): the cartwall keeps its least width (or the whole width), the players scroll at their minimum, nothing overlaps, nothing is NaN. Tests: Task 1, `rl11_too_narrow_a_window_on_a_side_keeps_the_cartwall_minimum`, `rl11_a_window_narrower_than_the_cartwall_minimum_gives_it_the_whole_width`, `rl11_a_side_dock_in_too_narrow_a_window_scrolls_the_players`.
2. **A cart starting or stopping while the side cartwall is at its least width**: the "Stop all (n)" label grows, but the cartwall does not jump. Test: Task 1, `rl8_the_least_width_does_not_change_while_carts_play`.
3. **The layout is locked, the window resized or the dock changed in the middle of a cartwall drag**: the drag ends and no `SetCartwallSize` goes out. Tests: Task 2, `rl12_a_dock_change_a_resized_window_or_a_new_range_ends_a_cartwall_drag`, `rl12_locking_during_a_cartwall_drag_ends_it_without_a_command`.
4. **A session saved with the cartwall collapsed on a side, with a share**: from the first frame it is the strip, its splitter is inert, and the caret brings back that share. Tests: Task 2, `rl12_a_collapsed_cartwall_has_an_inert_splitter`; Task 4, `rl14_the_caret_expands_it_to_its_share` (both build the state collapsed before the first frame).
5. **The default window must keep today's cartwall** (40-point buttons, the rectangles the screenshot crops use) even though every bottom/`None` path now goes through `cartwall_range`. Tests: plan 2's `rl11_the_default_layout_draws_todays_window` (Task 1 must keep it green), Task 3, `rl13_fitting_the_content_keeps_todays_buttons`.

---

## File Structure

| File | Responsibility |
|---|---|
| `crates/fp-app/src/ui/panel_layout.rs` | `CartwallSizes::{min_width, collapsed}`; `Arrangement::{splitter, fit_content}`; `cartwall_range`; `arrange` with `on_a_side`, `at_the_bottom` on the range, `gap_between`; `CartwallDrag` |
| `crates/fp-app/src/ui/panels.rs` | `cartwall_drag_layout`, `cartwall_splitter`; `header_menu` (collapsed cartwall); `settle_view` (collapsing ends a cartwall maximise) |
| `crates/fp-app/src/ui/app.rs` | The middle-area block of `AppUi::ui` (the model's pass input, the live layout, the splitter, `strip`'s `fill`); `ViewState::cartwall_drag` |
| `crates/fp-app/src/ui/cartwall.rs` | `min_width`, `sizes(ui, scene)`, the header's width helpers, `strip(…, fill)`, `collapsed_strip`, `grid`'s filled buttons and centred text |
| `crates/fp-app/src/ui/cart_view.rs` | `MIN_BUTTON_WIDTH` (moved here), `filled_button_height` |
| `crates/fp-app/tests/panel_layout.rs` | the pure pass, the range, the drag |
| `crates/fp-app/tests/layout_ui.rs` | the window, driven headlessly |
| `crates/fp-app/tests/cartwall_ui.rs` | `filled_button_height` |
| `docs/user/layout.md`, `docs/user/cartwall.md` | the guide |
| `docs/technical/ui.md` | "Main window layout", the Structure rows |
| `docs/superpowers/specs/2026-10-21-resizable-layout-design.md`, `docs/superpowers/specs/2026-09-26-phase2-cartwall-settings-design.md` | clarifying sentences (RL8, RL12–RL14); the P2.9 amendment |
| `README.md` | the feature list |
| `docs/images/main-screen.png`, `docs/images/guide/*.png` (and `scripts/site/screenshots.sh` only if a crop moved) | the screenshots |

---

### Task 1: Side docks and the cartwall's least width (RL6, RL8, RL11, RL14's and RL16's sizes)

**Files:**
- Modify: `crates/fp-app/src/ui/panel_layout.rs` (imports; `CartwallSizes`; `Arrangement`; `arrange`; `at_the_bottom`; new `on_a_side`, `gap_between`, `cartwall_range`)
- Modify: `crates/fp-app/src/ui/cart_view.rs` (constants)
- Modify: `crates/fp-app/src/ui/cartwall.rs` (module doc, imports, constants, `sizes`, new `min_width` and width helpers, `header`, `stop_all`, `tabs`)
- Modify: `crates/fp-app/src/ui/app.rs` (`AppUi::ui`: the `cartwall::sizes` call)
- Modify: `docs/technical/ui.md` ("Main window layout"), `docs/user/layout.md`
- Test: `crates/fp-app/tests/panel_layout.rs`, `crates/fp-app/tests/layout_ui.rs`

**Interfaces:**
- Consumes (plan 2): `PassInput { players, layout, area, cartwall, maximised }` (`Copy`), `Arrangement { players_area, widths, overflow, cartwall }`, `arrange`, `player_widths` (private, unchanged), `PANEL_GAP`, `MIN_PLAYER_WIDTH`, `MIN_PLAYER_HEIGHT`, `CONTENT_HEIGHT_CAP`; `cartwall::{height, min_height, sizes}`; the `layout_ui.rs` helpers `column`, `columns`, `near`, `open_layout_menu`, `maximise`, `header_point`; plan 1's `CartwallLayout::share() -> Option<f32>`, `DEFAULT_SIDE_SHARE`, `Dock::is_side`.
- Produces:
  - `pub struct CartwallSizes { pub content_height: f32, pub min_height: f32, pub min_width: f32, pub collapsed: Option<f32> }` (`collapsed`: the header's height while collapsed, `None` while open).
  - `pub struct Arrangement { …, pub splitter: Option<Rect>, pub fit_content: bool }`.
  - `pub fn panel_layout::cartwall_range(input: &PassInput<'_>) -> Option<(f32, f32)>` (`None` while hidden or collapsed, or with no player shown).
  - `pub const cart_view::MIN_BUTTON_WIDTH: f32 = 56.0`.
  - `pub(crate) fn cartwall::min_width(ui: &Ui, scene: &Scene<'_>) -> f32`; `pub(crate) fn cartwall::sizes(ui: &Ui, scene: &Scene<'_>) -> CartwallSizes` (was `sizes(scene)`).
  - Test helpers in `layout_ui.rs`: `docked(AppState, Dock, Option<f32>) -> AppState`, `wide(AppState)`, `with_station_id(AppState) -> AppState`.

- [ ] **Step 1: Write the failing pure tests**

In `crates/fp-app/tests/panel_layout.rs`, make the `use` block read:

```rust
use std::path::PathBuf;

use egui::{Rect, pos2, vec2};
use fp_app::ui::panel_layout::{
    Arrangement, CONTENT_HEIGHT_CAP, CartwallSizes, LiveSplit, MIN_PLAYER_HEIGHT,
    MIN_PLAYER_WIDTH, Maximised, PANEL_GAP, PassInput, SHARE_FLOOR, arrange, cartwall_range,
    drag_pair, pair_shares, panel_on_air,
};
use fp_model::layout::{Dock, Panel};
use fp_model::{AppState, Command, Config, PlayerId, TrackAnalysis, apply};
```

Replace the `SIZES` constant and its comment with:

```rust
/// A page of two rows of eight: today's content height (24 + 6 + 2 × 40 +
/// 6), the least height (the rows at 28) and the least width (eight columns
/// of 56 with seven gaps of 6); open.
const SIZES: CartwallSizes = CartwallSizes {
    content_height: 116.0,
    min_height: 92.0,
    min_width: 490.0,
    collapsed: None,
};

/// The same cartwall collapsed: only its 24-point header.
const COLLAPSED: CartwallSizes = CartwallSizes {
    content_height: 24.0,
    min_height: 24.0,
    min_width: 490.0,
    collapsed: Some(24.0),
};
```

In `rl11_fitting_the_content_is_capped_at_sixty_percent_as_before`, make the `tall` literal read:

```rust
    let tall = CartwallSizes {
        content_height: 900.0,
        min_height: 300.0,
        ..SIZES
    };
```

Append:

```rust
// --- side docks (RL6, RL8, RL11, RL14, RL16)

/// Two players: room for both beside a side cartwall at 1904 points.
fn two() -> AppState {
    let mut config = Config::default();
    config.players.count = 2;
    AppState::new(config, "Main")
}

fn docked(mut s: AppState, dock: Dock, size: Option<f32>) -> AppState {
    s.layout.cartwall.dock = dock;
    s.layout.cartwall.size = size;
    s
}

fn range(s: &AppState, area: Rect, sizes: CartwallSizes) -> Option<(f32, f32)> {
    let players = s.player_ids();
    cartwall_range(&PassInput {
        players: &players,
        layout: &s.layout,
        area,
        cartwall: sizes,
        maximised: None,
    })
}

#[test]
fn rl6_a_left_dock_is_the_full_height_on_the_left() {
    let s = docked(two(), Dock::Left, Some(0.3));
    let a = pass(&s, full_hd(), None);
    let cart = a.cartwall.unwrap();
    // 0.3 × 1904, between the least width (490) and what leaves two players
    // theirs (1904 − 8 − 2 × 380 − 8).
    assert!(near(cart.width(), 571.2), "{a:?}");
    assert_eq!((cart.left(), cart.top(), cart.bottom()), (8.0, 42.0, 1048.0));
    assert!(near(a.players_area.left(), cart.right() + PANEL_GAP), "{a:?}");
    assert_eq!(a.players_area.right(), 1912.0);
    assert_eq!((a.players_area.top(), a.players_area.bottom()), (42.0, 1048.0));
    // The row is 1324.8 wide: 1324.8 − 8 − 2 × 380 = 556.8 spare, half each.
    assert!(all_near(&widths(&a), &[658.4, 658.4]), "{a:?}");
    assert!(!a.overflow);
}

#[test]
fn rl6_a_right_dock_is_the_full_height_on_the_right() {
    let s = docked(two(), Dock::Right, Some(0.3));
    let a = pass(&s, full_hd(), None);
    let cart = a.cartwall.unwrap();
    assert!(near(cart.width(), 571.2), "{a:?}");
    assert_eq!((cart.right(), cart.top(), cart.bottom()), (1912.0, 42.0, 1048.0));
    assert_eq!(a.players_area.left(), 8.0);
    assert!(near(a.players_area.right() + PANEL_GAP, cart.left()), "{a:?}");
    assert!(all_near(&widths(&a), &[658.4, 658.4]), "{a:?}");
}

#[test]
fn rl6_a_side_dock_without_a_size_takes_the_default_share() {
    let s = docked(two(), Dock::Left, None);
    assert!(near(pass(&s, full_hd(), None).cartwall.unwrap().width(), 571.2));
}

#[test]
fn rl8_a_side_cartwall_keeps_its_least_width() {
    let s = docked(two(), Dock::Left, Some(0.05));
    let a = pass(&s, full_hd(), None);
    assert!(near(a.cartwall.unwrap().width(), SIZES.min_width), "{a:?}");
}

#[test]
fn rl11_a_side_share_leaves_the_players_their_minimum() {
    let s = docked(two(), Dock::Left, Some(0.95));
    let a = pass(&s, full_hd(), None);
    // 1904 − 8 − (2 × 380 + 8)
    assert!(near(a.cartwall.unwrap().width(), 1128.0), "{a:?}");
    assert!(all_near(&widths(&a), &[MIN_PLAYER_WIDTH; 2]), "{a:?}");
    assert!(!a.overflow);
}

#[test]
fn rl11_too_narrow_a_window_on_a_side_keeps_the_cartwall_minimum() {
    // Four players need 4 × 380 + 3 × 8 = 1544; with the cartwall's 490 and
    // its gap that is 2042, more than 1904.
    let s = docked(four(), Dock::Right, Some(0.3));
    let a = pass(&s, full_hd(), None);
    assert!(near(a.cartwall.unwrap().width(), SIZES.min_width), "{a:?}");
    assert!(a.overflow);
    assert!(all_near(&widths(&a), &[MIN_PLAYER_WIDTH; 4]), "{a:?}");
    assert!(a.players_area.right() <= a.cartwall.unwrap().left() - PANEL_GAP + 0.01);
}

#[test]
fn rl11_a_window_narrower_than_the_cartwall_minimum_gives_it_the_whole_width() {
    let s = docked(two(), Dock::Left, Some(0.3));
    let narrow = Rect::from_min_size(pos2(8.0, 42.0), vec2(400.0, 1006.0));
    let a = pass(&s, narrow, None);
    assert_eq!(a.cartwall, Some(narrow));
    assert_eq!(a.players_area.width(), 0.0);
    assert!(a.overflow);
    assert!(widths(&a).iter().all(|w| w.is_finite()));
}

#[test]
fn rl11_with_no_player_shown_a_side_cartwall_takes_the_middle_area() {
    let mut s = docked(two(), Dock::Left, Some(0.3));
    s.layout.hidden_players = s.player_ids();
    let a = pass(&s, full_hd(), None);
    assert_eq!(a.cartwall, Some(full_hd()));
    assert!(a.widths.is_empty());
    assert_eq!(a.splitter, None);
}

#[test]
fn rl16_a_maximised_player_shrinks_a_side_cartwall_to_its_least_width() {
    let s = docked(two(), Dock::Left, Some(0.3));
    let p = s.player_ids();
    let a = pass(&s, full_hd(), Some(Panel::Player(p[0])));
    assert!(near(a.cartwall.unwrap().width(), SIZES.min_width), "{a:?}");
    // The row is 1904 − 490 − 8 = 1406: 1406 − 8 − 760 = 638 spare, all P1's.
    assert!(all_near(&widths(&a), &[1018.0, MIN_PLAYER_WIDTH]), "{a:?}");
}

#[test]
fn rl16_a_maximised_side_cartwall_leaves_the_players_their_minimum() {
    let s = docked(two(), Dock::Right, Some(0.3));
    let a = pass(&s, full_hd(), Some(Panel::Cartwall));
    assert!(near(a.cartwall.unwrap().width(), 1128.0), "{a:?}");
    assert!(all_near(&widths(&a), &[MIN_PLAYER_WIDTH; 2]), "{a:?}");
}

#[test]
fn rl12_the_cartwall_range_is_its_minimum_up_to_the_players_minimum() {
    assert_eq!(
        range(&four(), full_hd(), SIZES),
        Some((92.0, 1006.0 - PANEL_GAP - MIN_PLAYER_HEIGHT))
    );
    assert_eq!(
        range(&docked(two(), Dock::Left, None), full_hd(), SIZES),
        Some((490.0, 1128.0))
    );
    let low = Rect::from_min_size(pos2(8.0, 42.0), vec2(1904.0, 400.0));
    assert_eq!(
        range(&four(), low, SIZES),
        Some((92.0, 92.0)),
        "never below its own minimum"
    );
    let mut s = four();
    s.layout.cartwall.hidden = true;
    assert_eq!(range(&s, full_hd(), SIZES), None, "hidden");
    let mut s = four();
    s.layout.hidden_players = s.player_ids();
    assert_eq!(range(&s, full_hd(), SIZES), None, "no player shown");
    assert_eq!(range(&four(), full_hd(), COLLAPSED), None, "collapsed");
}

#[test]
fn rl12_the_cartwall_splitter_is_the_gap_beside_it() {
    let a = pass(&four(), full_hd(), None);
    assert_eq!(
        a.splitter,
        Some(Rect::from_min_max(pos2(8.0, 924.0), pos2(1912.0, 932.0)))
    );
    let a = pass(&docked(two(), Dock::Left, Some(0.3)), full_hd(), None);
    let (cart, gap) = (a.cartwall.unwrap(), a.splitter.unwrap());
    assert!(near(gap.left(), cart.right()) && near(gap.right(), a.players_area.left()));
    assert_eq!((gap.top(), gap.bottom()), (42.0, 1048.0));
    let a = pass(&docked(two(), Dock::Right, Some(0.3)), full_hd(), None);
    let (cart, gap) = (a.cartwall.unwrap(), a.splitter.unwrap());
    assert!(near(gap.left(), a.players_area.right()) && near(gap.right(), cart.left()));
    let mut s = four();
    s.layout.cartwall.hidden = true;
    assert_eq!(pass(&s, full_hd(), None).splitter, None);
}

#[test]
fn rl13_only_a_bottom_cartwall_without_a_share_fits_its_content() {
    assert!(pass(&four(), full_hd(), None).fit_content);
    let mut s = four();
    s.layout.cartwall.size = Some(0.3);
    assert!(!pass(&s, full_hd(), None).fit_content, "a share");
    assert!(
        !pass(&docked(two(), Dock::Left, None), full_hd(), None).fit_content,
        "a side"
    );
    assert!(
        !pass(&four(), full_hd(), Some(Panel::Cartwall)).fit_content,
        "maximised"
    );
    let mut s = four();
    s.layout.hidden_players = s.player_ids();
    assert!(!pass(&s, full_hd(), None).fit_content, "alone, it fills the area");
}

#[test]
fn rl14_collapsed_on_a_side_is_a_strip_as_wide_as_the_header_is_high() {
    let s = docked(two(), Dock::Left, Some(0.4));
    let a = pass_with(&s, full_hd(), COLLAPSED, None);
    let cart = a.cartwall.unwrap();
    assert!(near(cart.width(), 24.0), "{a:?}");
    assert_eq!((cart.top(), cart.bottom()), (42.0, 1048.0));
    assert!(near(a.players_area.left(), 8.0 + 24.0 + PANEL_GAP), "{a:?}");
    assert!(a.splitter.is_some(), "the gap is still there");
    assert_eq!(range(&s, full_hd(), COLLAPSED), None, "but nothing to resize");
    // Expanded, it has its share back: the model kept it.
    let a = pass(&s, full_hd(), None);
    assert!(near(a.cartwall.unwrap().width(), 0.4 * 1904.0), "{a:?}");
}

#[test]
fn rl14_collapsed_at_the_bottom_is_its_header_whatever_its_share() {
    let mut s = four();
    s.layout.cartwall.size = Some(0.5);
    let a = pass_with(&s, full_hd(), COLLAPSED, None);
    assert!(near(a.cartwall.unwrap().height(), 24.0), "{a:?}");
}

#[test]
fn rl14_a_collapsed_cartwall_is_never_maximised() {
    for s in [four(), docked(two(), Dock::Right, None)] {
        assert_eq!(
            pass_with(&s, full_hd(), COLLAPSED, Some(Panel::Cartwall)),
            pass_with(&s, full_hd(), COLLAPSED, None)
        );
    }
}
```

- [ ] **Step 2: Write the failing UI tests**

In `crates/fp-app/tests/layout_ui.rs`, make the `use` block read:

```rust
use std::path::PathBuf;

use egui::accesskit::Role;
use egui::{CursorIcon, Event, Key, Modifiers, PointerButton, Pos2, Rect, pos2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use fp_app::ui::app::AppUi;
use fp_app::ui::cart_view::{GAP, MIN_BUTTON_WIDTH};
use fp_app::ui::controller::Controller;
use fp_app::ui::panel_layout::{MIN_PLAYER_HEIGHT, MIN_PLAYER_WIDTH, PANEL_GAP};
use fp_app::ui::theme;
use fp_model::layout::{CartwallLayout, Dock, Panel};
use fp_model::{
    AppState, CartEdit, CartKind, Command, KeyChord, PlayerId, Shortcut, ShortcutAction,
    TrackAnalysis, apply,
};
use support::{Fake, harness, harness_sized, state};
```

and append:

```rust
// --- the cartwall's dock (RL6, RL8, RL11)

fn docked(mut s: AppState, dock: Dock, size: Option<f32>) -> AppState {
    s.layout.cartwall = CartwallLayout {
        dock,
        size,
        hidden: false,
    };
    s
}

/// A window wide enough for two players beside a side cartwall at its
/// default share (the middle area is 1968 × 810).
fn wide(s: AppState) -> (Harness<'static, AppUi>, std::sync::Arc<Fake>) {
    harness_sized(s, vec2(2000.0, 900.0), |ui| ui)
}

/// `s` with "Station ID" (10 s, a Jingle) in the first cart of the first
/// page.
fn with_station_id(mut s: AppState) -> AppState {
    let page = s.cartwall.pages[0].id;
    apply(
        &mut s,
        Command::AssignCartFile {
            page,
            index: 0,
            path: PathBuf::from("/carts/id.wav"),
        },
    )
    .unwrap();
    let track = s.cartwall.pages[0].carts[0].track.unwrap();
    apply(
        &mut s,
        Command::ApplyAnalysis {
            track,
            analysis: Box::new(TrackAnalysis {
                duration_secs: 10.0,
                ..TrackAnalysis::default()
            }),
        },
    )
    .unwrap();
    let edit = CartEdit {
        name: "Station ID".into(),
        kind: CartKind::Jingle,
        looped: false,
        exclusive: false,
        color: None,
    };
    apply(&mut s, Command::SetCart { page, index: 0, edit }).unwrap();
    s
}

/// `s` with a first page of `rows` × `cols`.
fn with_grid(mut s: AppState, rows: u16, cols: u16) -> AppState {
    let page = s.cartwall.pages[0].id;
    apply(&mut s, Command::ResizeCartPage { page, rows, cols }).unwrap();
    s
}

/// Eight columns of the smallest button with their gaps.
fn eight_columns() -> f32 {
    8.0 * MIN_BUTTON_WIDTH + 7.0 * GAP
}

#[test]
fn rl6_a_left_dock_draws_the_cartwall_beside_the_players_at_full_height() {
    let (h, fake) = wide(docked(state(2, 3), Dock::Left, Some(0.3)));
    let cart = h.state().cartwall_rect().unwrap();
    let (p1, p2) = (column(&h, fake.player(0)), column(&h, fake.player(1)));
    assert!(near(p1.left(), cart.right() + PANEL_GAP), "{cart:?} {p1:?}");
    assert!(near(cart.top(), p1.top()), "{cart:?} {p1:?}");
    // A column is 2 points shorter than its row.
    assert!(near(cart.bottom(), p1.bottom() + 2.0), "{cart:?} {p1:?}");
    let area = p2.right() - cart.left();
    assert!(near(cart.width(), 0.3 * area), "{cart:?} in {area}");
}

#[test]
fn rl6_a_right_dock_draws_the_cartwall_right_of_the_players() {
    let (h, fake) = wide(docked(state(2, 3), Dock::Right, Some(0.3)));
    let cart = h.state().cartwall_rect().unwrap();
    let (p1, p2) = (column(&h, fake.player(0)), column(&h, fake.player(1)));
    assert!(near(cart.left(), p2.right() + PANEL_GAP), "{cart:?} {p2:?}");
    assert!(near(cart.top(), p1.top()), "{cart:?} {p1:?}");
    let area = cart.right() - p1.left();
    assert!(near(cart.width(), 0.3 * area), "{cart:?} in {area}");
}

#[test]
fn rl6_the_layout_menu_moves_the_cartwall() {
    let (mut h, fake) = wide(state(2, 3));
    let (p1, p2) = (fake.player(0), fake.player(1));
    for (label, beside) in [
        ("Cartwall on the left", Dock::Left),
        ("Cartwall on the right", Dock::Right),
        ("Cartwall at the bottom", Dock::Bottom),
    ] {
        open_layout_menu(&mut h);
        h.get_by_role_and_label(Role::RadioButton, label).click();
        h.run_steps(2);
        let cart = h.state().cartwall_rect().unwrap();
        let placed = match beside {
            Dock::Left => cart.right() < column(&h, p1).left(),
            Dock::Right => cart.left() > column(&h, p2).right(),
            Dock::Bottom => cart.top() > column(&h, p1).bottom(),
        };
        assert!(placed, "{label}: {cart:?}");
    }
}

#[test]
fn rl11_a_side_dock_in_too_narrow_a_window_scrolls_the_players() {
    for dock in [Dock::Left, Dock::Right] {
        let (h, _) = harness(docked(state(2, 3), dock, Some(0.3)));
        let cart = h.state().cartwall_rect().unwrap();
        assert!(near(cart.width(), eight_columns()), "{dock:?} {cart:?}");
        let c = columns(&h);
        assert!(
            c.iter().all(|(_, r)| near(r.width(), MIN_PLAYER_WIDTH)),
            "{dock:?} {c:?}"
        );
        if dock == Dock::Left {
            assert!(c[0].1.left() >= cart.right(), "nothing overlaps: {c:?} {cart:?}");
        }
    }
}

#[test]
fn rl8_a_side_cartwall_is_never_narrower_than_its_grid() {
    let (h, _) = wide(docked(state(2, 3), Dock::Left, Some(0.05)));
    let cart = h.state().cartwall_rect().unwrap();
    assert!(near(cart.width(), eight_columns()), "{cart:?}");
    let eighth = h.get_all_by_label("Cart 8, empty").next().unwrap().rect();
    assert!(near(eighth.width(), MIN_BUTTON_WIDTH), "{eighth:?}");
    assert!(eighth.right() <= cart.right() + 0.5, "{eighth:?} in {cart:?}");
}

#[test]
fn rl8_a_one_column_page_keeps_the_header_whole() {
    let s = docked(with_grid(state(2, 3), 2, 1), Dock::Left, Some(0.05));
    let (h, _) = wide(s);
    let cart = h.state().cartwall_rect().unwrap();
    assert!(cart.width() < eight_columns(), "the header sets it: {cart:?}");
    let title = h.get_by_label("CARTWALL").rect();
    let tab = h.get_by_label("Carts 1").rect();
    let stop = h.get_by_label("Stop all").rect();
    assert!(title.left() >= cart.left() - 0.5, "{title:?} in {cart:?}");
    assert!(title.right() <= tab.left() + 0.5, "{title:?} {tab:?}");
    assert!(tab.right() <= stop.left() + 0.5, "{tab:?} {stop:?}");
    assert!(stop.right() <= cart.right() + 0.5, "{stop:?} in {cart:?}");
}

#[test]
fn rl8_the_least_width_does_not_change_while_carts_play() {
    let s = docked(with_grid(with_station_id(state(2, 3)), 2, 1), Dock::Left, Some(0.05));
    let (mut h, fake) = wide(s);
    let before = h.state().cartwall_rect().unwrap();
    let cart = fake.state.load().cartwall.pages[0].carts[0].id;
    Controller::send(fake.as_ref(), Command::FireCart(cart));
    h.run_steps(2);
    assert!(h.query_by_label("Stop all (1)").is_some(), "a cart plays");
    assert_eq!(h.state().cartwall_rect(), Some(before));
}

#[test]
fn rl16_a_maximised_player_shrinks_a_side_cartwall_to_its_least_width() {
    let (mut h, fake) = wide(docked(state(2, 3), Dock::Right, Some(0.3)));
    maximise(&mut h, header_point(&h, fake.player(0)));
    let cart = h.state().cartwall_rect().unwrap();
    assert!(near(cart.width(), eight_columns()), "{cart:?}");
    assert!(near(column(&h, fake.player(1)).width(), MIN_PLAYER_WIDTH));
}
```

(`with_grid` keeps cart 0, so "Station ID" stays the first cart of a one-column page.)

- [ ] **Step 3: Run them to see them fail**

Run: `cargo test -p fp-app --test panel_layout` and `cargo test -p fp-app --test layout_ui rl6 rl8 rl11 rl16`
Expected: `panel_layout` fails to compile (no `cartwall_range`, no fields `min_width`, `collapsed`, `splitter`, `fit_content`); `layout_ui` fails to compile (no `MIN_BUTTON_WIDTH` in `cart_view`).

- [ ] **Step 4: The smallest cart button, in `cart_view`**

`crates/fp-app/src/ui/cart_view.rs`, after `MIN_BUTTON_HEIGHT`:

```rust
/// RL8, RL13: the least width of a cart button; a grid narrower than its
/// columns at this width scrolls.
pub const MIN_BUTTON_WIDTH: f32 = 56.0;
```

- [ ] **Step 5: The pass on every dock**

`crates/fp-app/src/ui/panel_layout.rs`: make the `fp_model::layout` import read `use fp_model::layout::{DEFAULT_SIDE_SHARE, Dock, Panel};`.

Replace `CartwallSizes` with:

```rust
/// The cartwall's sizes from its content (`cartwall::sizes`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CartwallSizes {
    /// What its content wants at the bottom: the header, or the header and
    /// the shown page's rows at their nominal height.
    pub content_height: f32,
    /// RL8: its least height: the header, or the header and the rows at the
    /// smallest button.
    pub min_height: f32,
    /// RL8: its least width on a side: the larger of its header's fixed
    /// parts and the shown page's columns at the smallest button.
    pub min_width: f32,
    /// RL14: while it is collapsed, its thickness (the header's height):
    /// its height at the bottom, its width on a side. `None` while open.
    pub collapsed: Option<f32>,
}
```

In `Arrangement`, after `cartwall`:

```rust
    /// RL12: the gap between the players and the cartwall, its splitter;
    /// `None` while the cartwall is hidden or no player is shown.
    pub splitter: Option<Rect>,
    /// RL13: the cartwall fits its content (at the bottom, without a share,
    /// nothing maximised, a player shown): its buttons keep `BUTTON_HEIGHT`
    /// at most. Otherwise they fill it.
    pub fit_content: bool,
```

Replace `arrange` and `at_the_bottom` with:

```rust
/// RL11: the pass. The cartwall first, then the players split the rest:
/// each gets its minimum width and the spare room is split by share.
pub fn arrange(input: &PassInput<'_>) -> Arrangement {
    let layout = input.layout;
    let shown = layout.shown_players(input.players);
    // A hidden panel is never maximised (RL16), nor a collapsed cartwall:
    // it has nothing to fill the room with (RL14).
    let maximised = input.maximised.filter(|panel| match panel {
        Panel::Player(id) => shown.contains(id),
        Panel::Cartwall => !layout.cartwall.hidden && input.cartwall.collapsed.is_none(),
    });
    let area = input.area;
    let dock = layout.cartwall.dock;
    let (players_area, cartwall) = if layout.cartwall.hidden {
        (area, None)
    } else {
        match dock {
            Dock::Bottom if shown.is_empty() => {
                (Rect::from_min_size(area.min, Vec2::ZERO), Some(area))
            }
            Dock::Bottom => at_the_bottom(input, maximised),
            Dock::Left | Dock::Right => on_a_side(input, maximised),
        }
    };
    let splitter = match cartwall {
        Some(cart) if !shown.is_empty() => Some(gap_between(dock, players_area, cart)),
        _ => None,
    };
    let fit_content = dock == Dock::Bottom
        && !layout.cartwall.hidden
        && !shown.is_empty()
        && maximised.is_none()
        && layout.cartwall.share().is_none();
    let (widths, overflow) = player_widths(input, &shown, players_area.width(), maximised);
    Arrangement {
        players_area,
        widths,
        overflow,
        cartwall,
        splitter,
        fit_content,
    }
}

/// RL8, RL11, RL12: the least and the most size, in points along its axis
/// (its height at the bottom, its width on a side), that the open cartwall
/// can take beside the shown players: its own minimum, and what leaves the
/// players theirs, never below its own minimum. `None` while it is hidden
/// or collapsed, or with no player shown: then it has no size to choose.
pub fn cartwall_range(input: &PassInput<'_>) -> Option<(f32, f32)> {
    let layout = input.layout;
    let sizes = input.cartwall;
    let shown = layout.shown_players(input.players).len();
    if layout.cartwall.hidden || sizes.collapsed.is_some() || shown == 0 {
        return None;
    }
    let area = input.area;
    let (least, most) = if layout.cartwall.dock.is_side() {
        let count = shown as f32;
        let players = count * MIN_PLAYER_WIDTH + (count - 1.0) * PANEL_GAP;
        (sizes.min_width, area.width() - PANEL_GAP - players)
    } else {
        (
            sizes.min_height,
            area.height() - PANEL_GAP - MIN_PLAYER_HEIGHT,
        )
    };
    Some((least, most.max(least)))
}

/// The cartwall under the players: its content height ("fit the content",
/// exactly as before), or its share clamped to `cartwall_range` (RL8,
/// RL11); its least height next to a maximised player, its most when it is
/// maximised itself (RL16); collapsed, only its header (RL14).
fn at_the_bottom(input: &PassInput<'_>, maximised: Option<Panel>) -> (Rect, Option<Rect>) {
    let area = input.area;
    let sizes = input.cartwall;
    let height = match (cartwall_range(input), maximised, input.layout.cartwall.share()) {
        (None, _, _) => sizes.content_height,
        (Some((_, most)), Some(Panel::Cartwall), _) => most,
        (Some((least, _)), Some(Panel::Player(_)), _) => least,
        (Some(_), None, None) => sizes
            .content_height
            .min(area.height() * CONTENT_HEIGHT_CAP),
        (Some((least, most)), None, Some(share)) => (area.height() * share).min(most).max(least),
    };
    let top = (area.bottom() - height).max(area.top());
    let cartwall = Rect::from_min_max(pos2(area.left(), top), area.max);
    let players = Rect::from_min_max(
        area.min,
        pos2(area.right(), (top - PANEL_GAP).max(area.top())),
    );
    (players, Some(cartwall))
}

/// RL6: the cartwall the full height of the area, on its side: its share of
/// the width clamped to `cartwall_range` (RL8, RL11), its least width next
/// to a maximised player, its most when maximised itself (RL16), the
/// collapsed strip (RL14), or the whole area with no player shown. Never
/// wider than the area; the players take what is left.
fn on_a_side(input: &PassInput<'_>, maximised: Option<Panel>) -> (Rect, Option<Rect>) {
    let area = input.area;
    let layout = input.layout;
    let width = match (input.cartwall.collapsed, cartwall_range(input), maximised) {
        (Some(strip), _, _) => strip,
        (None, None, _) => area.width(),
        (None, Some((_, most)), Some(Panel::Cartwall)) => most,
        (None, Some((least, _)), Some(Panel::Player(_))) => least,
        (None, Some((least, most)), None) => {
            let share = layout.cartwall.share().unwrap_or(DEFAULT_SIDE_SHARE);
            (area.width() * share).min(most).max(least)
        }
    }
    .min(area.width())
    .max(0.0);
    if layout.cartwall.dock == Dock::Right {
        let left = area.right() - width;
        let cart = Rect::from_min_max(pos2(left, area.top()), area.max);
        let players = Rect::from_min_max(
            area.min,
            pos2((left - PANEL_GAP).max(area.left()), area.bottom()),
        );
        (players, Some(cart))
    } else {
        let right = area.left() + width;
        let cart = Rect::from_min_max(area.min, pos2(right, area.bottom()));
        let players = Rect::from_min_max(
            pos2((right + PANEL_GAP).min(area.right()), area.top()),
            area.max,
        );
        (players, Some(cart))
    }
}

/// RL12: the gap between the players' row and the cartwall.
fn gap_between(dock: Dock, players: Rect, cart: Rect) -> Rect {
    match dock {
        Dock::Bottom => Rect::from_min_max(
            pos2(cart.left(), players.bottom()),
            pos2(cart.right(), cart.top()),
        ),
        Dock::Left => Rect::from_min_max(
            pos2(cart.right(), cart.top()),
            pos2(players.left(), cart.bottom()),
        ),
        Dock::Right => Rect::from_min_max(
            pos2(players.right(), cart.top()),
            pos2(cart.left(), cart.bottom()),
        ),
    }
}
```

(`player_widths` is unchanged: it lays the players out in whatever width `players_area` has, which is plan 2's extension point.)

- [ ] **Step 6: The cartwall measures its least width**

`crates/fp-app/src/ui/cartwall.rs`:

1. Make the module doc read `//! The cartwall panel (Phase 2 spec P2.9, v3 layout): under the players or on a side (resizable layout spec RL6, RL8, RL13, RL14).`
2. Make the `cart_view` import read:

```rust
use super::cart_view::{
    BUTTON_HEIGHT, CartStatus, GAP, MIN_BUTTON_HEIGHT, MIN_BUTTON_WIDTH, button_height,
    cart_look, cart_view, page_on_air,
};
```

3. Replace the two constants at the top with:

```rust
const HEADER_HEIGHT: f32 = 24.0;
/// The space between the header's parts.
const HEADER_SPACING: f32 = 10.0;
/// Stop all's icon.
const STOP_ICON_WIDTH: f32 = 14.0;
```

4. Replace `sizes` with:

```rust
/// RL8: the cartwall's least width on a side: the larger of its header's
/// fixed parts (Stop all, measured with a two-digit count so that it does
/// not change while carts start and stop; the title with its caret; the
/// narrowest page tab; the spaces between them and before the hint) and the
/// shown page's columns at `MIN_BUTTON_WIDTH` with their gaps.
pub(crate) fn min_width(ui: &Ui, scene: &Scene<'_>) -> f32 {
    let t = scene.i18n;
    let wall = &scene.state.cartwall;
    let stop_all = stop_all_width(
        ui,
        &t.tr_args("cartwall-stop-all-count", &[("count", 99_usize.into())]),
    );
    let title = title_width(ui, &t.tr("cartwall-title"));
    let tab = wall
        .pages
        .iter()
        .enumerate()
        .map(|(i, p)| tab_width(ui, &page_name(scene, i, &p.name)))
        .reduce(f32::min)
        .unwrap_or(0.0);
    let header = stop_all + title + tab + 3.0 * HEADER_SPACING;
    let cols = wall.shown_page().map_or(0, |p| p.cols);
    let grid = if cols > 0 {
        f32::from(cols) * MIN_BUTTON_WIDTH + f32::from(cols - 1) * GAP
    } else {
        0.0
    };
    header.max(grid)
}

/// The sizes the pixel pass needs (RL8, RL11, RL14).
pub(crate) fn sizes(ui: &Ui, scene: &Scene<'_>) -> CartwallSizes {
    CartwallSizes {
        content_height: height(scene),
        min_height: min_height(scene),
        min_width: min_width(ui, scene),
        collapsed: (!scene.state.cartwall.open).then_some(HEADER_HEIGHT),
    }
}

/// Stop all's width for `label`.
fn stop_all_width(ui: &Ui, label: &str) -> f32 {
    let text = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font(10.0), theme::TEXT)
        .size()
        .x;
    6.0 + STOP_ICON_WIDTH + 4.0 + text + 8.0
}

/// The width of the title tile (the caret and CARTWALL).
fn title_width(ui: &Ui, title: &str) -> f32 {
    ui.painter()
        .layout_no_wrap(title.to_owned(), font(10.0), theme::NEUTRAL_300)
        .size()
        .x
        + 24.0
}

/// A page tab's width.
fn tab_width(ui: &Ui, name: &str) -> f32 {
    ui.painter()
        .layout_no_wrap(name.to_owned(), font(11.0), theme::TEXT)
        .size()
        .x
        + 32.0
}
```

5. Make the header, Stop all and the tabs use the same measures:
   - in `header`, both `ui.spacing_mut().item_spacing = vec2(10.0, 0.0);` become `ui.spacing_mut().item_spacing = vec2(HEADER_SPACING, 0.0);`, and the `let width = ui.painter().layout_no_wrap(title.clone(), …).size().x + 24.0;` statement becomes `let width = title_width(ui, &title);`;
   - in `stop_all`, delete `const ICON_WIDTH: f32 = 14.0;`, replace the `text_width` statement and `let width = 6.0 + ICON_WIDTH + 4.0 + text_width + 8.0;` with `let width = stop_all_width(ui, &label);`, and write `STOP_ICON_WIDTH` where the paint closure used `ICON_WIDTH`;
   - in `tabs`, the `let width = ui.painter().layout_no_wrap(name.clone(), …).size().x + 32.0;` statement becomes `let width = tab_width(ui, &name);`.

`crates/fp-app/src/ui/app.rs`, in `ui`: add `let cart_sizes = cartwall::sizes(ui, &scene);` right before `let pass = |maximised| {`, and in the closure's `PassInput` replace `cartwall: cartwall::sizes(&scene),` with `cartwall: cart_sizes,`.

- [ ] **Step 7: Run the tests**

Run: `cargo test -p fp-app --test panel_layout`, `cargo test -p fp-app --test layout_ui`, `cargo test -p fp-app --test cartwall_ui`, then `cargo test -p fp-app`.
Expected: PASS, plan 2's tests included (the default window's rectangles are unchanged: `rl11_the_default_layout_draws_todays_window`). `stop_all_sits_at_the_right_end_of_the_bar` and `stop_all_stays_at_the_right_end_in_a_narrow_window` in `cartwall_ui.rs` must stay green: the header draws exactly as before, only through the helpers.

- [ ] **Step 8: Docs**

`docs/technical/ui.md`, "Main window layout":

- In the first paragraph, replace "(`cartwall::sizes`: its content height and its least height, the rows at `MIN_BUTTON_HEIGHT`)" with "(`cartwall::sizes`: its content height, its least height, its least width and its thickness while collapsed)".
- Replace the bullet that begins `- **The cartwall** (at the bottom):` (it ends "The side docks are laid out at the bottom for now.") with:

```markdown
- **The cartwall** has a range, `panel_layout::cartwall_range`: from its
  least size (RL8: at the bottom `min_height`, the header and the rows at
  `MIN_BUTTON_HEIGHT`; on a side `min_width`, from `cartwall::min_width`:
  the larger of the header's fixed parts, Stop all measured with a
  two-digit count, and the page's columns at `MIN_BUTTON_WIDTH` with their
  gaps) to what leaves the shown players `MIN_PLAYER_HEIGHT` (bottom) or
  `MIN_PLAYER_WIDTH` each with their gaps (side), never below the least
  size. At the bottom with `size: None` it fits its content as before, at
  most `CONTENT_HEIGHT_CAP` (60 %) of the area, unclamped; a share is
  clamped to the range. On a side (`on_a_side`) it is the full height of
  the area and its share of the width (`CartwallLayout::share`, 0.3 by
  default) is clamped to the range; the players take the rest of the width,
  and only they scroll when it is too narrow. It is never wider than the
  area. With no player shown it takes the whole area. `Arrangement::splitter`
  is the gap beside it (none while it is hidden or no player is shown).
```

`docs/user/layout.md`: insert before `## Maximise a panel`:

```markdown
## Move and resize the cartwall

The cartwall sits under the players, or on the left or the right of the
window at its full height: choose **Cartwall at the bottom**, **Cartwall on
the left** or **Cartwall on the right** in [the layout menu](#the-layout-menu).
Under the players it is as tall as its rows need, at most six tenths of the
window; beside them it takes three tenths of the width at first.

The cartwall is never narrower than its header (**Stop all**, its title and
a page tab) or than its columns of buttons at their smallest. When the
window is too narrow for it and every shown player at their smallest, the
players' row scrolls sideways.
```

- [ ] **Step 9: Gate and commit**

Run the gate. Then:

```bash
git add crates/fp-app/src crates/fp-app/tests docs/technical/ui.md docs/user/layout.md
git commit -m "feat(ui): dock the cartwall on the left or the right

RL6, RL8 and RL11 of the resizable layout spec: a side cartwall is the
full height of the middle area and takes its share of the width, clamped
between its least width (its header's fixed parts or its columns at the
smallest button, whichever is larger) and what leaves the players their
minimum; the players take the rest and scroll only below their minimums.
One range function serves the pass and, next, the splitter. A collapsed
cartwall is never maximised, and at the bottom it is its header whatever
its share.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: The cartwall splitter (RL12)

**Files:**
- Modify: `crates/fp-app/src/ui/panel_layout.rs` (imports; append `CartwallDrag`)
- Modify: `crates/fp-app/src/ui/panels.rs` (imports; append `cartwall_drag_layout`, `cartwall_splitter`)
- Modify: `crates/fp-app/src/ui/app.rs` (`ViewState`; the middle-area block of `ui`)
- Modify: `docs/technical/ui.md`, `docs/user/layout.md`
- Test: `crates/fp-app/tests/panel_layout.rs`, `crates/fp-app/tests/layout_ui.rs`

**Interfaces:**
- Consumes: Task 1's `cartwall_range`, `Arrangement::splitter`, `CartwallSizes`; plan 2's `panels::splitter(ui, gap, id, cursor, active, dragging) -> Option<Response>`, `HOLD_SECS`, `ViewState::maximised`, `players_row(…, arranged, normal)`; plan 1's `Command::SetCartwallSize(Option<f32>)`, `DEFAULT_SIDE_SHARE`, `MIN_CARTWALL_SHARE`, `MAX_CARTWALL_SHARE`; the `layout_ui.rs` helpers `press`, `click_at`, `cursor`, `maximise`, `header_point`, and Task 1's `docked`, `wide`.
- Produces:
  - In `panel_layout` (`pub`): `struct CartwallDrag` (`Debug, Clone, PartialEq`) with `fn new(dock: Dock, area: Rect, range: (f32, f32), origin: Pos2, start: f32) -> Self`, `fn is_dragging(&self) -> bool`, `fn follow(&mut self, pointer: Pos2)`, `fn size(&self) -> f32`, `fn share(&self) -> f32`, `fn release(&mut self, size: Option<f32>, until: f64) -> Option<Command>`, `fn stale(&self, dock: Dock, area: Rect, range: Option<(f32, f32)>, size: Option<f32>, time: f64) -> bool`.
  - In `panels` (`pub(crate)`): `fn cartwall_drag_layout(ui: &Ui, scene: &Scene<'_>, view_state: &mut ViewState, input: &PassInput<'_>) -> Option<fp_model::layout::Layout>`; `fn cartwall_splitter(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState, input: &PassInput<'_>, arranged: &Arrangement, normal: &Arrangement)`.
  - `ViewState::cartwall_drag: Option<panel_layout::CartwallDrag>` (`pub(crate)`).
  - Test helpers in `layout_ui.rs`: `tall(AppState)`, `cartwall_splitter_at(&Harness) -> Pos2`, `drag_along(&mut Harness, Pos2, Vec2)`, `size_commands(&Fake) -> Vec<Command>`, `cart_rect(&Harness) -> Rect`.

- [ ] **Step 1: Write the failing pure tests**

In `crates/fp-app/tests/panel_layout.rs`, add `CartwallDrag` to the `fp_app::ui::panel_layout::{…}` list and append:

```rust
// --- the cartwall drag (RL12)

#[test]
fn rl12_a_bottom_drag_grows_the_cartwall_upwards_within_its_range() {
    let mut drag = CartwallDrag::new(
        Dock::Bottom,
        full_hd(),
        (92.0, 566.0),
        pos2(900.0, 928.0),
        116.0,
    );
    drag.follow(pos2(900.0, 868.0));
    assert_eq!(drag.size(), 176.0);
    drag.follow(pos2(900.0, 100.0));
    assert_eq!(drag.size(), 566.0, "the players keep their least height");
    drag.follow(pos2(900.0, 1040.0));
    assert_eq!(drag.size(), 92.0, "the cartwall keeps its own");
    drag.follow(pos2(f32::NAN, f32::NAN));
    assert_eq!(drag.size(), 92.0, "a pointer that is not a number moves nothing");
}

#[test]
fn rl12_a_side_drag_grows_the_cartwall_towards_the_players() {
    let mut left = CartwallDrag::new(
        Dock::Left,
        full_hd(),
        (490.0, 1128.0),
        pos2(583.0, 500.0),
        571.2,
    );
    left.follow(pos2(633.0, 500.0));
    assert!(near(left.size(), 621.2));
    let mut right = CartwallDrag::new(
        Dock::Right,
        full_hd(),
        (490.0, 1128.0),
        pos2(1336.0, 500.0),
        571.2,
    );
    right.follow(pos2(1286.0, 500.0));
    assert!(near(right.size(), 621.2));
}

#[test]
fn rl12_a_cartwall_drag_sends_its_share_once_and_holds_until_the_echo() {
    let area = full_hd();
    let range = Some((92.0, 566.0));
    let mut drag = CartwallDrag::new(Dock::Bottom, area, (92.0, 566.0), pos2(900.0, 928.0), 116.0);
    // 0.2 of the 1006-point area is 201.2: 85.2 points up.
    drag.follow(pos2(900.0, 842.8));
    let sent = drag.release(None, 10.5);
    let Some(Command::SetCartwallSize(Some(share))) = sent else {
        panic!("{sent:?}")
    };
    assert!((share - 0.2).abs() < 1e-4, "{share}");
    assert!(!drag.is_dragging());
    assert_eq!(drag.release(None, 10.5), None, "sent once");
    drag.follow(pos2(900.0, 500.0));
    assert!(near(drag.size(), 201.2), "a released drag no longer follows the pointer");
    assert!(!drag.stale(Dock::Bottom, area, range, None, 10.0), "held");
    assert!(
        drag.stale(Dock::Bottom, area, range, Some(0.2), 10.0),
        "the echo arrived"
    );
    assert!(drag.stale(Dock::Bottom, area, range, None, 10.5), "the hold ran out");
}

#[test]
fn rl12_a_cartwall_drag_back_to_where_it_began_sends_nothing() {
    let mut drag = CartwallDrag::new(
        Dock::Left,
        full_hd(),
        (490.0, 1128.0),
        pos2(583.0, 500.0),
        571.2,
    );
    drag.follow(pos2(683.0, 500.0));
    drag.follow(pos2(583.2, 500.0));
    assert_eq!(drag.release(Some(0.3), 1.0), None);
}

#[test]
fn rl12_a_dock_change_a_resized_window_or_a_new_range_ends_a_cartwall_drag() {
    let area = full_hd();
    let range = Some((92.0, 566.0));
    let drag = CartwallDrag::new(Dock::Bottom, area, (92.0, 566.0), pos2(900.0, 928.0), 116.0);
    assert!(!drag.stale(Dock::Bottom, area, range, None, 0.0));
    assert!(drag.stale(Dock::Left, area, range, None, 0.0), "docked elsewhere");
    let smaller = Rect::from_min_max(area.min, pos2(1912.0, 900.0));
    assert!(
        drag.stale(Dock::Bottom, smaller, range, None, 0.0),
        "the window was resized"
    );
    assert!(
        drag.stale(Dock::Bottom, area, Some((150.0, 566.0)), None, 0.0),
        "the page's rows changed"
    );
    assert!(
        drag.stale(Dock::Bottom, area, None, None, 0.0),
        "collapsed or hidden"
    );
}

#[test]
fn rl12_the_pass_draws_a_cartwall_drag_exactly() {
    let mut drag = CartwallDrag::new(
        Dock::Bottom,
        full_hd(),
        (92.0, 566.0),
        pos2(900.0, 928.0),
        116.0,
    );
    drag.follow(pos2(900.0, 777.0));
    let mut s = four();
    s.layout.cartwall.size = Some(drag.share());
    let a = pass(&s, full_hd(), None);
    assert!(near(a.cartwall.unwrap().height(), drag.size()), "{a:?}");
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-app --test panel_layout rl12`
Expected: FAIL to compile: no `CartwallDrag` in `panel_layout`.

- [ ] **Step 3: The drag's maths**

`crates/fp-app/src/ui/panel_layout.rs`: make the imports read

```rust
use egui::{Pos2, Rect, Vec2, pos2};
use fp_model::layout::{
    DEFAULT_SIDE_SHARE, Dock, MAX_CARTWALL_SHARE, MIN_CARTWALL_SHARE, Panel,
};
use fp_model::{AppState, Command, PlayerId, Transport};
```

and append:

```rust
/// RL12: the cartwall splitter being dragged, or just released: the size
/// is drawn from the pointer on every frame and sent once, on release, as
/// the cartwall's share of the middle area.
#[derive(Debug, Clone, PartialEq)]
pub struct CartwallDrag {
    dock: Dock,
    /// The middle area and the cartwall's range the drag began with; a
    /// change in either ends it.
    area: Rect,
    range: (f32, f32),
    /// The pointer where the drag began.
    origin: Pos2,
    /// The cartwall's size, in points along its axis, then and now.
    start: f32,
    now: f32,
    /// Set on release: the model's size then, and until when the released
    /// size is still drawn.
    released: Option<(Option<f32>, f64)>,
}

impl CartwallDrag {
    /// A drag of the splitter of a cartwall docked at `dock`, begun at
    /// `origin` with the cartwall `start` points along its axis.
    pub fn new(dock: Dock, area: Rect, range: (f32, f32), origin: Pos2, start: f32) -> Self {
        Self {
            dock,
            area,
            range,
            origin,
            start,
            now: start,
            released: None,
        }
    }

    /// The button is still down.
    pub fn is_dragging(&self) -> bool {
        self.released.is_none()
    }

    /// The pointer is at `pointer`: the size follows it (until the
    /// release), within the range. Dragging up grows a bottom cartwall,
    /// dragging right a left one, dragging left a right one.
    pub fn follow(&mut self, pointer: Pos2) {
        if !self.is_dragging() {
            return;
        }
        let moved = pointer - self.origin;
        let grow = match self.dock {
            Dock::Bottom => -moved.y,
            Dock::Left => moved.x,
            Dock::Right => -moved.x,
        };
        if grow.is_finite() {
            self.now = (self.start + grow).min(self.range.1).max(self.range.0);
        }
    }

    /// The size now, in points along the cartwall's axis.
    pub fn size(&self) -> f32 {
        self.now
    }

    /// The size now as the cartwall's share of the middle area: what the
    /// pass draws while the drag lives, and what the release sends.
    pub fn share(&self) -> f32 {
        let extent = if self.dock.is_side() {
            self.area.width()
        } else {
            self.area.height()
        };
        if extent > 0.0 {
            (self.now / extent)
                .min(MAX_CARTWALL_SHARE)
                .max(MIN_CARTWALL_SHARE)
        } else {
            MIN_CARTWALL_SHARE
        }
    }

    /// The button was released: the command to send, once, and only if
    /// the drag moved something. `size` is the model's size now; the
    /// released size is held until it changes or until `until`.
    pub fn release(&mut self, size: Option<f32>, until: f64) -> Option<Command> {
        if !self.is_dragging() {
            return None;
        }
        self.released = Some((size, until));
        ((self.now - self.start).abs() > 0.5).then(|| Command::SetCartwallSize(Some(self.share())))
    }

    /// Whether the drag no longer applies: the dock, the area or the range
    /// changed (or there is no range: hidden or collapsed), or, once
    /// released, the model's size changed (its echo arrived) or the hold
    /// ran out.
    pub fn stale(
        &self,
        dock: Dock,
        area: Rect,
        range: Option<(f32, f32)>,
        size: Option<f32>,
        time: f64,
    ) -> bool {
        let moved = |a: Pos2, b: Pos2| (a - b).length() > 0.5;
        self.dock != dock
            || moved(self.area.min, area.min)
            || moved(self.area.max, area.max)
            || range.is_none_or(|(least, most)| {
                (least - self.range.0).abs() > 0.5 || (most - self.range.1).abs() > 0.5
            })
            || self
                .released
                .is_some_and(|(before, until)| time >= until || before != size)
    }
}
```

Run: `cargo test -p fp-app --test panel_layout`
Expected: PASS.

- [ ] **Step 4: Write the failing UI tests**

In `crates/fp-app/tests/layout_ui.rs`, make two lines of the `use` block read

```rust
use egui::{CursorIcon, Event, Key, Modifiers, PointerButton, Pos2, Rect, Vec2, pos2, vec2};
```

```rust
use fp_model::layout::{CartwallLayout, DEFAULT_SIDE_SHARE, Dock, Panel};
```

and append:

```rust
// --- the cartwall splitter (RL12)

/// A window tall enough for the bottom cartwall to grow (its middle area is
/// 968 × 810: the players keep 432, so the cartwall can reach 370).
fn tall(s: AppState) -> (Harness<'static, AppUi>, std::sync::Arc<Fake>) {
    harness_sized(s, vec2(1000.0, 900.0), |ui| ui)
}

/// The middle of the gap between the players and the cartwall.
fn cartwall_splitter_at(h: &Harness<'_, AppUi>) -> Pos2 {
    let cart = h.state().cartwall_rect().unwrap();
    let first = columns(h)[0].1;
    if cart.top() > first.bottom() {
        pos2(cart.center().x, cart.top() - PANEL_GAP / 2.0)
    } else if cart.right() < first.left() {
        pos2(cart.right() + PANEL_GAP / 2.0, cart.center().y)
    } else {
        pos2(cart.left() - PANEL_GAP / 2.0, cart.center().y)
    }
}

/// Presses at `from` and moves by `by` in six frames; the button stays
/// down.
fn drag_along(h: &mut Harness<'_, AppUi>, from: Pos2, by: Vec2) {
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    press(h, from, PointerButton::Primary, true);
    h.run_steps(1);
    for step in 1..=6 {
        h.event(Event::PointerMoved(from + by * (step as f32 / 6.0)));
        h.run_steps(1);
    }
}

fn size_commands(fake: &Fake) -> Vec<Command> {
    fake.take_sent()
        .into_iter()
        .filter(|c| matches!(c, Command::SetCartwallSize(_)))
        .collect()
}

fn cart_rect(h: &Harness<'_, AppUi>) -> Rect {
    h.state().cartwall_rect().unwrap()
}

#[test]
fn rl12_dragging_the_bottom_splitter_resizes_the_cartwall_live() {
    let (mut h, fake) = tall(state(2, 3));
    let p1 = fake.player(0);
    let (cart, col) = (cart_rect(&h), column(&h, p1));
    let from = cartwall_splitter_at(&h);
    drag_along(&mut h, from, vec2(0.0, -60.0));
    let (cart2, col2) = (cart_rect(&h), column(&h, p1));
    assert!(near(cart2.height(), cart.height() + 60.0), "{cart:?} -> {cart2:?}");
    assert!(near(col2.height(), col.height() - 60.0), "{col:?} -> {col2:?}");
    assert!(
        size_commands(&fake).is_empty(),
        "nothing is sent while the button is down"
    );
}

#[test]
fn rl12_one_cartwall_size_on_release_and_the_size_stays() {
    let (mut h, fake) = tall(state(2, 3));
    let from = cartwall_splitter_at(&h);
    drag_along(&mut h, from, vec2(0.0, -60.0));
    let live = cart_rect(&h);
    press(&mut h, from + vec2(0.0, -60.0), PointerButton::Primary, false);
    h.run_steps(3);
    let sent = size_commands(&fake);
    assert_eq!(sent.len(), 1, "{sent:?}");
    let Command::SetCartwallSize(Some(share)) = sent[0].clone() else {
        panic!("{sent:?}")
    };
    let area = live.bottom() - column(&h, fake.player(0)).top();
    assert!((share * area - live.height()).abs() < 1.0, "{share} of {area}: {live:?}");
    // Past the hold, the model's share draws the same size.
    h.run_steps(40);
    assert!(near(cart_rect(&h).height(), live.height()), "{:?}", cart_rect(&h));
}

#[test]
fn rl12_a_click_on_the_cartwall_splitter_sends_nothing() {
    let (mut h, fake) = tall(state(2, 3));
    let at = cartwall_splitter_at(&h);
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    click_at(&mut h, at);
    h.run_steps(30);
    assert!(size_commands(&fake).is_empty());
}

#[test]
fn rl12_a_double_click_fits_the_content_again_at_the_bottom() {
    let mut s = state(2, 3);
    s.layout.cartwall.size = Some(0.4);
    let (mut h, fake) = tall(s);
    let at = cartwall_splitter_at(&h);
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    click_at(&mut h, at);
    click_at(&mut h, at);
    h.run_steps(2);
    assert_eq!(size_commands(&fake), vec![Command::SetCartwallSize(None)]);
    // Two rows of 40 under a 24-point header, 6-point gaps.
    assert!(near(cart_rect(&h).height(), 116.0), "{:?}", cart_rect(&h));
}

#[test]
fn rl12_a_double_click_on_a_side_gives_the_default_share() {
    let (mut h, fake) = wide(docked(state(2, 3), Dock::Left, Some(0.5)));
    let at = cartwall_splitter_at(&h);
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    click_at(&mut h, at);
    click_at(&mut h, at);
    h.run_steps(2);
    assert_eq!(
        size_commands(&fake),
        vec![Command::SetCartwallSize(Some(DEFAULT_SIDE_SHARE))]
    );
}

#[test]
fn rl12_the_cartwall_splitter_shows_the_resize_cursor() {
    let (mut h, _) = tall(state(2, 3));
    h.event(Event::PointerMoved(cartwall_splitter_at(&h)));
    h.run_steps(1);
    assert_eq!(cursor(&h), CursorIcon::ResizeRow);
    let (mut h, _) = wide(docked(state(2, 3), Dock::Right, Some(0.3)));
    h.event(Event::PointerMoved(cartwall_splitter_at(&h)));
    h.run_steps(1);
    assert_eq!(cursor(&h), CursorIcon::ResizeColumn);
}

#[test]
fn rl12_dragging_a_side_splitter_resizes_the_cartwall() {
    for (dock, dx) in [(Dock::Left, 50.0), (Dock::Right, -50.0)] {
        let (mut h, fake) = wide(docked(state(2, 3), dock, Some(0.3)));
        let before = cart_rect(&h).width();
        let from = cartwall_splitter_at(&h);
        drag_along(&mut h, from, vec2(dx, 0.0));
        assert!(near(cart_rect(&h).width(), before + 50.0), "{dock:?}");
        press(&mut h, from + vec2(dx, 0.0), PointerButton::Primary, false);
        h.run_steps(3);
        assert_eq!(size_commands(&fake).len(), 1, "{dock:?}");
    }
}

#[test]
fn rl12_the_cartwall_splitter_keeps_the_players_least_height() {
    let (mut h, fake) = tall(state(2, 3));
    let from = cartwall_splitter_at(&h);
    drag_along(&mut h, from, vec2(0.0, -800.0));
    let p1 = fake.player(0);
    assert!(near(column(&h, p1).height(), MIN_PLAYER_HEIGHT - 2.0));
    press(&mut h, from + vec2(0.0, -800.0), PointerButton::Primary, false);
    h.run_steps(40);
    assert_eq!(size_commands(&fake).len(), 1);
    assert!(fake.rejected.lock().unwrap().is_empty(), "the model took it");
    assert!(near(column(&h, p1).height(), MIN_PLAYER_HEIGHT - 2.0));
}

#[test]
fn rl12_a_locked_layout_has_an_inert_cartwall_splitter() {
    let mut s = state(2, 3);
    s.layout.locked = true;
    let (mut h, fake) = tall(s);
    let before = cart_rect(&h);
    let at = cartwall_splitter_at(&h);
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    assert_eq!(cursor(&h), CursorIcon::Default);
    click_at(&mut h, at);
    click_at(&mut h, at);
    drag_along(&mut h, at, vec2(0.0, -60.0));
    press(&mut h, at + vec2(0.0, -60.0), PointerButton::Primary, false);
    h.run_steps(3);
    assert!(size_commands(&fake).is_empty());
    assert_eq!(cart_rect(&h), before);
}

#[test]
fn rl12_a_collapsed_cartwall_has_an_inert_splitter() {
    for dock in [Dock::Bottom, Dock::Left] {
        let mut s = docked(state(2, 3), dock, Some(0.4));
        s.cartwall.open = false;
        let (mut h, fake) = wide(s);
        let before = cart_rect(&h);
        let at = cartwall_splitter_at(&h);
        h.event(Event::PointerMoved(at));
        h.run_steps(1);
        assert_eq!(cursor(&h), CursorIcon::Default, "{dock:?}");
        drag_along(&mut h, at, vec2(40.0, -40.0));
        press(&mut h, at + vec2(40.0, -40.0), PointerButton::Primary, false);
        h.run_steps(3);
        assert!(size_commands(&fake).is_empty(), "{dock:?}");
        assert_eq!(cart_rect(&h), before, "{dock:?}");
    }
}

#[test]
fn rl12_locking_during_a_cartwall_drag_ends_it_without_a_command() {
    let (mut h, fake) = tall(state(2, 3));
    let before = cart_rect(&h);
    let from = cartwall_splitter_at(&h);
    drag_along(&mut h, from, vec2(0.0, -60.0));
    Controller::send(fake.as_ref(), Command::SetLayoutLocked(true));
    h.run_steps(1);
    press(&mut h, from + vec2(0.0, -60.0), PointerButton::Primary, false);
    h.run_steps(3);
    assert!(size_commands(&fake).is_empty());
    assert!(near(cart_rect(&h).height(), before.height()), "the model's size");
}

#[test]
fn rl16_a_cartwall_drag_ends_a_maximise_first() {
    let (mut h, fake) = tall(state(2, 3));
    let before = cart_rect(&h).height();
    maximise(&mut h, header_point(&h, fake.player(0)));
    assert!(cart_rect(&h).height() < before, "at its least height");
    let from = cartwall_splitter_at(&h);
    drag_along(&mut h, from, vec2(0.0, -20.0));
    assert_eq!(h.state().maximised(), None);
    assert!(near(cart_rect(&h).height(), before + 20.0), "{:?}", cart_rect(&h));
}
```

- [ ] **Step 5: Run them to see them fail**

Run: `cargo test -p fp-app --test layout_ui rl12 rl16_a_cartwall`
Expected: FAIL: dragging the gap changes nothing, no `SetCartwallSize` is sent, the cursor stays `Default` over the gap. The two inert tests (`rl12_a_locked_…`, `rl12_a_collapsed_…`) and `rl12_a_click_…` may already pass: they guard the splitter you are about to add.

- [ ] **Step 6: The splitter and its drag**

`crates/fp-app/src/ui/panels.rs`: make the imports read

```rust
use std::collections::HashMap;

use egui::{Align, Layout, Rect, Sense, TextureHandle, Ui, UiBuilder, pos2, vec2};
use fp_model::layout::{DEFAULT_SIDE_SHARE, Dock, Panel};
use fp_model::{AppState, Command, PlayerId, TrackId};

use super::app::{Scene, ViewState};
use super::panel_layout::{
    self, Arrangement, CartwallDrag, LiveSplit, Maximised, PANEL_GAP, PassInput,
};
use super::player;
use super::theme;
use super::widgets::{self, TileStyle, font};
```

and append:

```rust
/// RL12: a cartwall splitter drag, before the pass: it follows the pointer,
/// sends `SetCartwallSize` once on release, and ends when it no longer
/// applies (the layout was locked; the dock, the area or the cartwall's
/// range changed; or, once released, the model's echo arrived or the hold
/// ran out). While it lives: the layout the pass draws, with the drag's
/// size. `input` is the pass without a maximised panel, on the model's
/// layout.
pub(crate) fn cartwall_drag_layout(
    ui: &Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    input: &PassInput<'_>,
) -> Option<fp_model::layout::Layout> {
    let layout = &scene.state.layout;
    if layout.locked {
        view_state.cartwall_drag = None;
        return None;
    }
    let drag = view_state.cartwall_drag.as_mut()?;
    if drag.is_dragging() {
        let (down, pointer) = ui.input(|i| (i.pointer.primary_down(), i.pointer.latest_pos()));
        if down {
            if let Some(pos) = pointer {
                drag.follow(pos);
            }
        } else if let Some(command) =
            drag.release(layout.cartwall.size, scene.time + panel_layout::HOLD_SECS)
        {
            scene.ctl.send(command);
        }
    }
    let range = panel_layout::cartwall_range(input);
    if drag.stale(
        layout.cartwall.dock,
        input.area,
        range,
        layout.cartwall.size,
        scene.time,
    ) {
        view_state.cartwall_drag = None;
        return None;
    }
    let mut live = layout.clone();
    live.cartwall.size = Some(drag.share());
    Some(live)
}

/// RL12: the splitter between the players and the cartwall: horizontal at
/// the bottom, vertical on a side. A drag ends a maximised panel and
/// starts a `CartwallDrag` from the size without it (RL16); a double-click
/// goes back to fitting the content at the bottom, or to the default share
/// on a side. Inert while the layout is locked or the cartwall has no size
/// to choose (collapsed).
pub(crate) fn cartwall_splitter(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    input: &PassInput<'_>,
    arranged: &Arrangement,
    normal: &Arrangement,
) {
    let Some(gap) = arranged.splitter else {
        return;
    };
    let layout = &scene.state.layout;
    let dock = layout.cartwall.dock;
    let range = panel_layout::cartwall_range(input);
    let dragging = view_state
        .cartwall_drag
        .as_ref()
        .is_some_and(CartwallDrag::is_dragging);
    let cursor = if dock.is_side() {
        egui::CursorIcon::ResizeColumn
    } else {
        egui::CursorIcon::ResizeRow
    };
    let active = !layout.locked && range.is_some();
    let id = egui::Id::new("cartwall-splitter");
    let Some(response) = splitter(ui, gap, id, cursor, active, dragging) else {
        return;
    };
    if response.double_clicked() {
        let size = dock.is_side().then_some(DEFAULT_SIDE_SHARE);
        scene.ctl.send(Command::SetCartwallSize(size));
    }
    if response.drag_started() {
        view_state.maximised = None;
        if let (Some(range), Some(cart)) = (range, normal.cartwall) {
            let start = if dock.is_side() {
                cart.width()
            } else {
                cart.height()
            };
            let origin = ui
                .input(|i| i.pointer.press_origin())
                .unwrap_or(gap.center());
            view_state.cartwall_drag =
                Some(CartwallDrag::new(dock, input.area, range, origin, start));
        }
    }
}
```

`crates/fp-app/src/ui/app.rs`:

- In `ViewState`, after `maximised`:

```rust
    /// The cartwall splitter being dragged or just released (RL12).
    pub(crate) cartwall_drag: Option<super::panel_layout::CartwallDrag>,
```

- In `ui`, replace from `// The middle area (resizable layout spec RL11): the pixel pass turns` down to the closing brace of `if let Some(cart_rect) = arranged.cartwall { … }` (inclusive) with:

```rust
        // The middle area (resizable layout spec RL11): the pixel pass turns
        // the layout into the players' row and the cartwall's rectangle.
        let inner = middle.shrink(8.0);
        let ids = state.player_ids();
        let model_input = panel_layout::PassInput {
            players: &ids,
            layout: &state.layout,
            area: inner,
            cartwall: cartwall::sizes(ui, &scene),
            maximised: None,
        };
        // RL12: a cartwall splitter drag draws its size until the echo.
        let dragged = panels::cartwall_drag_layout(ui, &scene, &mut self.view, &model_input);
        let layout = dragged.as_ref().unwrap_or(&state.layout);
        let pass = |maximised| {
            panel_layout::arrange(&panel_layout::PassInput {
                layout,
                maximised,
                ..model_input
            })
        };
        let maximised = self.view.maximised.as_ref().map(|m| m.panel);
        let arranged = pass(maximised);
        // RL16: a splitter drag starts from the sizes without the maximised
        // panel.
        let normal = if maximised.is_some() {
            pass(None)
        } else {
            arranged.clone()
        };
        self.view.player_rects.clear();
        if !arranged.widths.is_empty() {
            let mut players_ui = ui.new_child(
                UiBuilder::new()
                    .max_rect(arranged.players_area)
                    .layout(Layout::left_to_right(Align::Min)),
            );
            panels::players_row(
                &mut players_ui,
                &scene,
                &mut self.view,
                &mut self.covers,
                &arranged,
                &normal,
            );
        }
        self.view.cartwall_rect = arranged.cartwall;
        if let Some(cart_rect) = arranged.cartwall {
            let mut cart_ui = ui.new_child(
                UiBuilder::new()
                    .max_rect(cart_rect)
                    .id_salt("cartwall")
                    .layout(Layout::top_down(Align::Min)),
            );
            cart_ui.set_clip_rect(cart_rect);
            cartwall::strip(&mut cart_ui, &scene, &mut self.view);
        }
        // RL12: the splitter beside the cartwall, over the gap.
        panels::cartwall_splitter(
            ui,
            &scene,
            &mut self.view,
            &model_input,
            &arranged,
            &normal,
        );
```

(This replaces Task 1's `cart_sizes` line and keeps everything else of plan 2's block. If the closure's parameter type is not inferred, write `|maximised: Option<fp_model::layout::Panel>|`.)

- [ ] **Step 7: Run the tests**

Run: `cargo test -p fp-app --test layout_ui`, `cargo test -p fp-app --test panel_layout`, then `cargo test -p fp-app`.
Expected: PASS. Plan 2's player splitter tests must stay green: the cartwall splitter is only the gap beside the cartwall.

- [ ] **Step 8: Docs**

`docs/technical/ui.md`, at the end of the "Main window layout" section, add:

```markdown
**Cartwall splitter** (RL12). `Arrangement::splitter`, the gap beside the
cartwall, is a splitter (`panels::cartwall_splitter` over the generic
`panels::splitter`): `ResizeRow` at the bottom, `ResizeColumn` on a side;
inert while the layout is locked or the cartwall has no range
(`cartwall_range` is `None` while it is collapsed, on any dock). A drag
lives in `ViewState::cartwall_drag` (`panel_layout::CartwallDrag`): it
follows the pointer along the cartwall's axis within the range, and
`panels::cartwall_drag_layout`, called before the pass, hands the pass a
copy of the layout with the drag's share, so the drag shows in the same
frame. On release one `SetCartwallSize(Some(share))` goes out, only if the
drag moved something; the share is the drawn size over the area, so the
next pass draws the same size. The size is held until the model's size
changes or `HOLD_SECS` pass; a change of dock, area or range, or a lock,
ends the drag. A drag ends a maximise first and starts from the pass
without it (`normal`). A double-click sends `SetCartwallSize(None)` at the
bottom (fit the content) and `SetCartwallSize(Some(DEFAULT_SIDE_SHARE))`
on a side.
```

`docs/user/layout.md`:

- At the end of "Move and resize the cartwall", add:

```markdown
Drag the gap between the cartwall and the players to make it larger or
smaller: up or down when it is under the players, sideways beside them. The
players keep the room their controls need. Double-click the gap to go back
to the starting size: as tall as its rows need under the players, three
tenths of the width beside them. The size is saved with the layout; moving
the cartwall starts it again at its starting size.
```

- In "Maximise a panel", replace "and dragging the gap between two players" with "and dragging a gap between panels".
- In "The layout menu", in the **Lock layout** item, replace "the gaps between the players" with "the gaps between the panels" (rewrap the item).

- [ ] **Step 9: Gate and commit**

Run the gate. Then:

```bash
git add crates/fp-app/src crates/fp-app/tests docs/technical/ui.md docs/user/layout.md
git commit -m "feat(ui): resize the cartwall with a splitter on every dock

RL12: the gap between the players and the cartwall resizes it live,
upwards at the bottom and sideways on a side, within its least size and
what leaves the players theirs, and sends SetCartwallSize once on release
with the share it drew, held until the model's echo. A double-click goes
back to fitting the content at the bottom or to the default share on a
side. The splitter does not react while the layout is locked or the
cartwall is collapsed, and a drag ends a maximised panel first.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: Cart buttons scale (RL13)

**Files:**
- Modify: `crates/fp-app/src/ui/cart_view.rs` (append `filled_button_height` after `button_height`)
- Modify: `crates/fp-app/src/ui/cartwall.rs` (imports; `strip`; `grid`)
- Modify: `crates/fp-app/src/ui/app.rs` (the `cartwall::strip` call)
- Modify: `docs/technical/ui.md`, `docs/user/layout.md`
- Test: `crates/fp-app/tests/cartwall_ui.rs`, `crates/fp-app/tests/layout_ui.rs`

**Interfaces:**
- Consumes: Task 1's `Arrangement::fit_content`; `cart_view::{BUTTON_HEIGHT, MIN_BUTTON_HEIGHT, GAP, button_height}`; Task 1's `with_station_id`, `with_grid`, `docked`, `wide`; Task 2's `tall`, `cart_rect`.
- Produces: `pub fn cart_view::filled_button_height(available: f32, rows: usize) -> f32`; `pub(crate) fn cartwall::strip(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState, fill: bool)`.

- [ ] **Step 1: Write the failing tests**

`crates/fp-app/tests/cartwall_ui.rs`: make the `cart_view` import read

```rust
use fp_app::ui::cart_view::{
    BUTTON_HEIGHT, CartStatus, GAP, MIN_BUTTON_HEIGHT, button_height, cart_look, cart_view,
    filled_button_height, page_on_air,
};
```

and append:

```rust
#[test]
fn rl13_filled_buttons_take_all_the_height_above_the_minimum() {
    assert_eq!(filled_button_height(1000.0, 2), (1000.0 - GAP) / 2.0);
    assert_eq!(filled_button_height(4.0 * 40.0 + 3.0 * GAP, 4), 40.0);
    assert_eq!(filled_button_height(10.0, 4), MIN_BUTTON_HEIGHT);
    assert_eq!(filled_button_height(f32::NAN, 4), BUTTON_HEIGHT);
    assert_eq!(filled_button_height(f32::INFINITY, 4), BUTTON_HEIGHT);
    assert_eq!(filled_button_height(100.0, 0), BUTTON_HEIGHT);
}
```

`crates/fp-app/tests/layout_ui.rs`: make the `cart_view` import read

```rust
use fp_app::ui::cart_view::{BUTTON_HEIGHT, GAP, MIN_BUTTON_HEIGHT, MIN_BUTTON_WIDTH};
```

and append:

```rust
// --- buttons scale (RL13)

/// Cart `n` of the shown page, empty.
fn empty_cart(h: &Harness<'_, AppUi>, n: usize) -> Rect {
    h.get_all_by_label(&format!("Cart {n}, empty"))
        .next()
        .expect("the cart is built")
        .rect()
}

/// Where the text `text` is laid out: the galley's rectangle.
fn text_rect(h: &Harness<'_, AppUi>, text: &str) -> Option<Rect> {
    fn walk(shape: &egui::Shape, text: &str, found: &mut Option<Rect>) {
        match shape {
            egui::Shape::Text(t) if t.galley.text() == text => {
                *found = Some(Rect::from_min_size(t.pos, t.galley.size()));
            }
            egui::Shape::Vec(v) => v.iter().for_each(|s| walk(s, text, found)),
            _ => {}
        }
    }
    let mut found = None;
    for clipped in &h.output().shapes {
        walk(&clipped.shape, text, &mut found);
    }
    found
}

#[test]
fn rl13_fitting_the_content_keeps_todays_buttons() {
    let (h, _) = tall(state(2, 3));
    assert!(near(empty_cart(&h, 2).height(), BUTTON_HEIGHT));
}

#[test]
fn rl13_buttons_grow_with_a_bottom_share() {
    let mut s = state(2, 3);
    s.layout.cartwall.size = Some(0.5);
    let (h, _) = tall(s);
    let cart = cart_rect(&h);
    let (first, last) = (empty_cart(&h, 2), empty_cart(&h, 16));
    assert!(first.height() > BUTTON_HEIGHT, "{first:?}");
    assert!(near(first.height(), last.height()));
    assert!(
        near(last.bottom(), cart.bottom()),
        "the grid fills the panel: {last:?} in {cart:?}"
    );
}

#[test]
fn rl13_buttons_grow_with_a_side_dock() {
    let (h, _) = wide(docked(state(2, 3), Dock::Left, Some(0.3)));
    let cart = cart_rect(&h);
    let last = empty_cart(&h, 16);
    assert!(last.height() > BUTTON_HEIGHT, "{last:?}");
    assert!(near(last.bottom(), cart.bottom()), "{last:?} in {cart:?}");
    assert!(near(last.right(), cart.right()), "{last:?} in {cart:?}");
}

#[test]
fn rl13_at_the_least_size_the_grid_scrolls() {
    // Eight rows of 28 need 24 + 6 + 8 × 28 + 7 × 6 = 296 points; the middle
    // area of a 360-point window has 270.
    let s = docked(with_grid(state(2, 3), 8, 8), Dock::Left, Some(0.3));
    let (h, _) = harness_sized(s, vec2(2000.0, 360.0), |ui| ui);
    let cart = cart_rect(&h);
    assert!(near(empty_cart(&h, 1).height(), MIN_BUTTON_HEIGHT));
    let last = empty_cart(&h, 64);
    assert!(last.bottom() > cart.bottom(), "the last row is below: {last:?} {cart:?}");
}

#[test]
fn rl13_a_tall_buttons_text_keeps_its_size_and_is_centred() {
    let mut s = with_station_id(state(2, 3));
    s.layout.cartwall.size = Some(0.5);
    let (h, _) = tall(s);
    let button = h.get_all_by_label("Station ID").next().unwrap().rect();
    assert!(button.height() > 2.0 * BUTTON_HEIGHT, "{button:?}");
    // The text block of a 40-point button, inside its 4-point margins,
    // centred in the taller button.
    let name = text_rect(&h, "Station ID").expect("the name is painted");
    assert!(
        near(name.top(), button.center().y - BUTTON_HEIGHT / 2.0 + 4.0),
        "{name:?} in {button:?}"
    );
    let kind = text_rect(&h, "Jingle").expect("the kind is painted");
    assert!(
        near(kind.bottom(), button.center().y + BUTTON_HEIGHT / 2.0 - 4.0),
        "{kind:?} in {button:?}"
    );
}
```

(Only the last test gives cart 1 a file; the others measure empty carts, so `empty_cart` finds them by their "Cart n, empty" label.)

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-app --test cartwall_ui rl13` and `cargo test -p fp-app --test layout_ui rl13`
Expected: `cartwall_ui` fails to compile (no `filled_button_height`); in `layout_ui`, the grow and centred tests fail (buttons stay 40 points, the text sits at the button's top); `rl13_fitting_the_content_keeps_todays_buttons` and `rl13_at_the_least_size_the_grid_scrolls` already pass (they guard today's behaviour).

- [ ] **Step 3: Implement**

`crates/fp-app/src/ui/cart_view.rs`, after `button_height`:

```rust
/// RL13: the height of a cart button when the grid fills a panel that does
/// not fit its content: what fits, never under `MIN_BUTTON_HEIGHT`, with no
/// upper clamp. The grid scrolls only when even the minimum does not fit.
pub fn filled_button_height(available: f32, rows: usize) -> f32 {
    if rows == 0 || !available.is_finite() {
        return BUTTON_HEIGHT;
    }
    let gaps = GAP * (rows - 1) as f32;
    ((available - gaps) / rows as f32).max(MIN_BUTTON_HEIGHT)
}
```

`crates/fp-app/src/ui/cartwall.rs`:

- Make the `cart_view` import read:

```rust
use super::cart_view::{
    BUTTON_HEIGHT, CartStatus, GAP, MIN_BUTTON_HEIGHT, MIN_BUTTON_WIDTH, button_height,
    cart_look, cart_view, filled_button_height, page_on_air,
};
```

- `strip` gains `fill: bool` after `view_state` (doc: "`fill`: the buttons fill the panel (RL13); otherwise it fits its content, 40-point buttons at most."), and its last line becomes `.show(ui, |ui| grid(ui, scene, view_state, available, fill));`.
- `grid` gains `fill: bool` after `available`, and `let button = button_height(available, page.carts.chunks(cols).count());` becomes:

```rust
    let rows = page.carts.chunks(cols).count();
    // RL13: in a panel that does not fit its content the buttons fill it;
    // fitting its content they keep today's 40-point cap.
    let button = if fill {
        filled_button_height(available, rows)
    } else {
        button_height(available, rows)
    };
```

- In the button's paint closure, replace `let inner = r.shrink2(vec2(8.0, 4.0));` with:

```rust
                        // RL13: the text keeps its size; in a taller button
                        // the text block of a 40-point one is centred, while
                        // the waveform and the bars take the whole button.
                        let inner = Rect::from_center_size(
                            r.center(),
                            vec2(r.width(), r.height().min(BUTTON_HEIGHT)),
                        )
                        .shrink2(vec2(8.0, 4.0));
```

`crates/fp-app/src/ui/app.rs`, in `ui`: `cartwall::strip(&mut cart_ui, &scene, &mut self.view);` becomes `cartwall::strip(&mut cart_ui, &scene, &mut self.view, !arranged.fit_content);`.

- [ ] **Step 4: Run the tests**

Run: `cargo test -p fp-app --test cartwall_ui`, `cargo test -p fp-app --test layout_ui`, then `cargo test -p fp-app`.
Expected: PASS. Every existing cartwall test (`four_rows_fit_a_short_window_without_scrolling`, `rows_are_exactly_one_gap_apart`, the colour and progress tests) stays green: at the bottom without a share nothing changed, and for a button 40 points tall or less `inner` is the same rectangle as before.

- [ ] **Step 5: Docs**

`docs/technical/ui.md`, at the end of the "Main window layout" section, add:

```markdown
**Buttons scale** (RL13). `Arrangement::fit_content` is true only at the
bottom without a share, with nothing maximised and a player shown. Then
`cartwall::strip` sizes the buttons with `cart_view::button_height` (40
points at most, as before); otherwise with `cart_view::filled_button_height`
(what fits, never under `MIN_BUTTON_HEIGHT`, no upper clamp), so the grid
fills the panel. Widths always fill it, never under `MIN_BUTTON_WIDTH`.
Below the minimums the grid scrolls (`ScrollArea::both`). In a button
taller than `BUTTON_HEIGHT` the text block of a 40-point button is centred
vertically; the waveform, the remaining-time fill and the CUE outline take
the whole button.
```

`docs/user/layout.md`, at the end of "Move and resize the cartwall", add:

```markdown
Once you resize the cartwall, or while it sits beside the players, its
buttons grow to fill it; the names and times keep their size, in the middle
of each button. When the cartwall is smaller than its rows and columns need
at the smallest buttons, it scrolls.
```

- [ ] **Step 6: Gate and commit**

Run the gate. Then:

```bash
git add crates/fp-app/src crates/fp-app/tests docs/technical/ui.md docs/user/layout.md
git commit -m "feat(ui): cart buttons grow with the cartwall

RL13: when the cartwall has a size of its own (a share at the bottom, a
side dock, maximised or alone in the middle area) its buttons fill it with
no 40-point cap, never below the smallest button, and the grid scrolls
below that. The name and detail line keep their sizes, centred, and the
waveform grows with the button. Under the players fitting its content the
cartwall is exactly as before.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: The collapsed cartwall on a side (RL14)

**Files:**
- Modify: `crates/fp-app/src/ui/cartwall.rs` (imports; `strip`; new `collapsed_strip`)
- Modify: `crates/fp-app/src/ui/panels.rs` (`header_menu`, `settle_view`)
- Modify: `docs/technical/ui.md`, `docs/user/layout.md`
- Test: `crates/fp-app/tests/layout_ui.rs`

**Interfaces:**
- Consumes: Task 1's `CartwallSizes::collapsed` and the pass's strip; plan 2's `panels::header_menu(&Response, scene, view_state, Panel)`, `panel_layout::panel_on_air(state, Panel) -> bool`, `settle_view`; the `layout_ui.rs` helpers `red_dot_in`, `right_click`, `disabled`, `layout_commands`, `maximise`, `cartwall_header_point`, and Tasks 1–2's `docked`, `wide`, `tall`, `cart_rect`, `with_station_id`.
- Produces: `fn cartwall::collapsed_strip(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState, dock: Dock)` (private); `header_menu` disables Maximise on a collapsed cartwall; `settle_view` ends a cartwall maximise when it collapses.

- [ ] **Step 1: Write the failing tests**

In `crates/fp-app/tests/layout_ui.rs` append:

```rust
// --- collapsed (RL14)

fn collapsed(dock: Dock, size: Option<f32>) -> AppState {
    let mut s = docked(state(2, 3), dock, size);
    s.cartwall.open = false;
    s
}

#[test]
fn rl14_collapsed_on_a_side_is_a_narrow_strip_with_its_caret() {
    for dock in [Dock::Left, Dock::Right] {
        let (h, fake) = wide(collapsed(dock, Some(0.4)));
        let cart = cart_rect(&h);
        let p1 = column(&h, fake.player(0));
        assert!(cart.width() < MIN_BUTTON_WIDTH, "{dock:?} {cart:?}");
        assert!(near(cart.top(), p1.top()), "{dock:?} {cart:?}");
        assert!(near(cart.bottom(), p1.bottom() + 2.0), "{dock:?} {cart:?}");
        let caret = h.get_by_label("CARTWALL").rect();
        assert!(cart.contains_rect(caret), "{dock:?}: {caret:?} in {cart:?}");
        assert!(h.query_by_label("Stop all").is_none(), "{dock:?}: only the caret");
        assert!(h.query_by_label("Cart 1, empty").is_none(), "{dock:?}");
    }
}

#[test]
fn rl14_the_strip_is_as_wide_as_the_collapsed_header_is_high() {
    let (h, _) = wide(collapsed(Dock::Bottom, None));
    let header = cart_rect(&h).height();
    let (g, _) = wide(collapsed(Dock::Bottom, Some(0.5)));
    assert!(near(cart_rect(&g).height(), header), "a share changes nothing");
    let (s, _) = wide(collapsed(Dock::Left, None));
    assert!(near(cart_rect(&s).width(), header), "{:?}", cart_rect(&s));
}

#[test]
fn rl14_the_caret_expands_it_to_its_share() {
    let (mut h, fake) = wide(collapsed(Dock::Right, Some(0.4)));
    h.get_by_label("CARTWALL").click();
    h.run_steps(3);
    assert!(fake.take_sent().contains(&Command::SetCartwallOpen(true)));
    let cart = cart_rect(&h);
    let area = cart.right() - column(&h, fake.player(0)).left();
    assert!(near(cart.width(), 0.4 * area), "{cart:?} in {area}");
    assert!(h.query_by_label("Cart 1, empty").is_some(), "the grid is back");
}

#[test]
fn rl14_the_strip_shows_a_red_dot_while_a_cart_plays() {
    let mut s = docked(with_station_id(state(2, 3)), Dock::Left, None);
    s.cartwall.open = false;
    let (mut h, fake) = wide(s);
    let strip = cart_rect(&h);
    assert!(!red_dot_in(&h, strip));
    let cart = fake.state.load().cartwall.pages[0].carts[0].id;
    Controller::send(fake.as_ref(), Command::FireCart(cart));
    h.run_steps(2);
    assert!(red_dot_in(&h, strip));
    Controller::send(fake.as_ref(), Command::StopCart(cart));
    h.run_steps(2);
    assert!(!red_dot_in(&h, strip));
}

#[test]
fn rl14_the_strips_menu_hides_it_and_cannot_maximise_it() {
    let (mut h, fake) = wide(collapsed(Dock::Left, None));
    let strip = cart_rect(&h);
    right_click(&mut h, strip.center());
    assert!(disabled(&h, "Maximise"));
    h.get_by_label("Hide").click();
    h.run_steps(2);
    assert_eq!(
        layout_commands(&fake),
        vec![Command::SetPanelHidden {
            panel: Panel::Cartwall,
            hidden: true,
        }]
    );
}

#[test]
fn rl14_a_collapsed_bottom_cartwall_cannot_be_maximised() {
    let mut s = state(2, 3);
    s.cartwall.open = false;
    let (mut h, _) = tall(s);
    right_click(&mut h, cartwall_header_point(&h));
    assert!(disabled(&h, "Maximise"));
}

#[test]
fn rl14_collapsing_a_maximised_cartwall_ends_the_maximise() {
    let (mut h, fake) = tall(state(2, 3));
    maximise(&mut h, cartwall_header_point(&h));
    assert_eq!(h.state().maximised(), Some(Panel::Cartwall));
    Controller::send(fake.as_ref(), Command::SetCartwallOpen(false));
    h.run_steps(2);
    assert_eq!(h.state().maximised(), None);
    Controller::send(fake.as_ref(), Command::SetCartwallOpen(true));
    h.run_steps(2);
    assert_eq!(h.state().maximised(), None, "expanding does not bring it back");
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-app --test layout_ui rl14`
Expected: FAIL: on a side the collapsed cartwall draws its whole header in the strip (Stop all is there, the caret tile is cut), no dot is painted, Maximise is enabled, and collapsing keeps the maximise. `rl14_the_strip_is_as_wide_as_the_collapsed_header_is_high` already passes (Task 1's pass).

- [ ] **Step 3: The strip, the menu and the view**

`crates/fp-app/src/ui/cartwall.rs`:

- Make the egui import read `use egui::{Align, Color32, Layout, Rect, RichText, Sense, Stroke, StrokeKind, Ui, UiBuilder, pos2, vec2};`, the layout import `use fp_model::layout::{Dock, Panel};`, and the pass import `use super::panel_layout::{self, CartwallSizes};`.
- At the start of `strip`, before `header(ui, scene, view_state);`:

```rust
    // RL14: collapsed on a side, the cartwall is a narrow strip.
    let dock = scene.state.layout.cartwall.dock;
    if !scene.state.cartwall.open && dock.is_side() {
        collapsed_strip(ui, scene, view_state, dock);
        return;
    }
```

- After `strip`, add:

```rust
/// RL14: the cartwall collapsed on a side: a strip as wide as the header is
/// high, the expand caret at its top (labelled as the header's, pointing at
/// the players) and, while a cart plays, the on-air dot under it. Its
/// background opens the panel menu (RL16), where Maximise is disabled.
fn collapsed_strip(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState, dock: Dock) {
    let rect = ui.max_rect();
    // First, so that the caret, added after it, keeps its own clicks.
    let background = ui.interact(rect, ui.id().with("cartwall-strip"), Sense::click());
    ui.painter().rect(
        rect,
        0.0,
        theme::NEUTRAL_900,
        Stroke::new(1.0, theme::NEUTRAL_800),
        StrokeKind::Inside,
    );
    let title = scene.i18n.tr("cartwall-title");
    let caret = if dock == Dock::Right {
        icon::CARET_LEFT
    } else {
        icon::CARET_RIGHT
    };
    let style = TileStyle {
        border: Color32::TRANSPARENT,
        hover_fill: theme::NEUTRAL_800,
        active_fill: Color32::TRANSPARENT,
        ..TileStyle::plain()
    };
    let caret_rect = Rect::from_min_size(rect.min, vec2(rect.width(), HEADER_HEIGHT));
    let mut caret_ui = ui.new_child(UiBuilder::new().max_rect(caret_rect));
    if widgets::tile(&mut caret_ui, caret_rect.size(), &title, true, style, |p, r, c| {
        p.text(r.center(), egui::Align2::CENTER_CENTER, caret, font(12.0), c);
    })
    .clicked()
    {
        scene.ctl.send(Command::SetCartwallOpen(true));
    }
    if panel_layout::panel_on_air(scene.state, Panel::Cartwall) {
        ui.painter().rect_filled(
            Rect::from_center_size(
                pos2(rect.center().x, rect.top() + HEADER_HEIGHT + 8.0),
                vec2(6.0, 6.0),
            ),
            0.0,
            theme::ON_AIR_TEXT,
        );
    }
    super::panels::header_menu(&background, scene, view_state, Panel::Cartwall);
}
```

`crates/fp-app/src/ui/panels.rs`:

- In `header_menu`, replace `let can = !layout.locked && layout.shown_panels(&players) > 1;` with:

```rust
            // RL14: a collapsed cartwall has nothing to fill the room with.
            let collapsed = panel == Panel::Cartwall && !scene.state.cartwall.open;
            let can = !layout.locked && layout.shown_panels(&players) > 1 && !collapsed;
```

- In `settle_view`, replace the first statement (the `if view_state.maximised … { view_state.maximised = None; }` block) with:

```rust
    // RL16: the arrangement changed; RL14: a collapsed cartwall is never
    // maximised, and expanding it does not maximise it again.
    let ends = view_state.maximised.as_ref().is_some_and(|m| {
        !m.holds(state) || (m.panel == Panel::Cartwall && !state.cartwall.open)
    });
    if ends {
        view_state.maximised = None;
    }
```

and add "; a collapsed cartwall is not maximised (RL14)" to its doc comment's first sentence.

- [ ] **Step 4: Run the tests**

Run: `cargo test -p fp-app --test layout_ui`, `cargo test -p fp-app --test cartwall_ui`, then `cargo test -p fp-app`.
Expected: PASS. `the_cartwall_collapses_and_expands` (bottom) is unchanged.

- [ ] **Step 5: Docs**

`docs/technical/ui.md`, at the end of the "Main window layout" section, add:

```markdown
**Collapsed** (RL14). `CartwallSizes::collapsed` is the header's height
while the cartwall is collapsed. At the bottom the cartwall is then its
header, whatever its share; on a side it is a strip that wide
(`cartwall::collapsed_strip`): the expand caret (a tile labelled
`cartwall-title`, pointing at the players), the on-air dot while
`panel_on_air(Cartwall)`, and a background that opens
`panels::header_menu`. A collapsed cartwall has no range, so its splitter
is inert; it is never maximised: the pass ignores it, the menu disables
Maximise, and `settle_view` ends a cartwall maximise when it collapses.
```

`docs/user/layout.md`:

- At the end of "Move and resize the cartwall", add:

```markdown
Click **CARTWALL** to collapse the cartwall. Under the players only its
header stays; beside them it becomes a narrow strip the height of the
window, with an arrow to expand it again and, while a cart plays, a red
dot. Expanded, it has its size back. While it is collapsed, the gap beside
it does nothing.
```

- In "Maximise a panel", at the end of the **Maximise** item, add: "A collapsed cartwall cannot be maximised, and collapsing a maximised cartwall ends it."

- [ ] **Step 6: Gate and commit**

Run the gate. Then:

```bash
git add crates/fp-app/src crates/fp-app/tests docs/technical/ui.md docs/user/layout.md
git commit -m "feat(ui): a collapsed side cartwall becomes a narrow strip

RL14: collapsed on a side, the cartwall is a strip as wide as its header
is high, with the expand caret and, while a cart plays, an on-air dot; its
background opens the panel menu. Expanding brings back its share. A
collapsed cartwall cannot be maximised, and collapsing a maximised one
ends the maximise.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: The remaining docs (guide, technical docs, specs, README)

**Files:**
- Modify: `docs/user/cartwall.md` (intro L3-5; "Using it" bullets on collapsing and short windows, L38-42)
- Modify: `docs/user/layout.md` (read it whole; fix what plans 2 and 3 left inconsistent)
- Modify: `docs/technical/ui.md` (Structure rows of `ui/panel_layout.rs`, `ui/panels.rs`, `ui/cartwall.rs`)
- Modify: `docs/superpowers/specs/2026-10-21-resizable-layout-design.md` (RL8, RL12, RL13, RL14)
- Modify: `docs/superpowers/specs/2026-09-26-phase2-cartwall-settings-design.md` (P2.9)
- Modify: `README.md` (Features)

**Interfaces:**
- Consumes: Tasks 1–4 (names in the docs must match the code: `cartwall_range`, `CartwallDrag`, `cartwall_drag_layout`, `cartwall_splitter`, `collapsed_strip`, `filled_button_height`, `fit_content`, `min_width`).
- Produces: nothing code uses.

- [ ] **Step 1: The cartwall page**

`docs/user/cartwall.md`:

- Replace the first sentence, "The cartwall is the strip of buttons under the players.", with "The cartwall is the panel of buttons under the players, or beside them (see [Layout](layout.md#move-and-resize-the-cartwall))."
- Replace the bullet "Click **CARTWALL** to collapse the strip or expand it again." with:

```markdown
- Click **CARTWALL** to collapse the cartwall or expand it again. Collapsed
  beside the players it is a narrow strip with an arrow to expand it and,
  while a cart plays, a red dot.
```

- Replace the bullet that begins "When the window is short, the buttons shrink" with:

```markdown
- Under the players, as tall as its rows need, the buttons shrink when the
  window is short (down to a minimum height) so that all the configured
  rows fit. Once you resize the cartwall, or beside the players, the
  buttons grow to fill it, and the names and times keep their size in the
  middle of each button. Either way the cartwall scrolls only when even the
  smallest buttons do not fit.
```

- [ ] **Step 2: The layout page, whole**

Read `docs/user/layout.md` from top to bottom. It must say, in this order: the intro; "Show and hide panels"; "Resize the players"; "Move and resize the cartwall" (dock, least width, the gap and its double-click, buttons that grow, collapsing); "Maximise a panel"; "The layout menu". Check that:
- the intro mentions moving the cartwall: make its first sentence read "Arrange the main window for your shift: give the players and the cartwall the sizes you want, put the cartwall under the players or beside them, hide the panels you do not use and maximise one for a moment.";
- every "gap" sentence covers both kinds of gap (between two players, beside the cartwall);
- no sentence still says the cartwall is always under the players.

Fix what does not, in the guide's style (short sentences, bold for on-screen names).

- [ ] **Step 3: The technical docs**

`docs/technical/ui.md`:

- Replace the `ui/panel_layout.rs` row of the Structure table with:

```markdown
| `ui/panel_layout.rs` | The main window's pure pixel pass (resizable layout spec RL11): RL8's interface constants (`MIN_PLAYER_WIDTH`, `MIN_PLAYER_HEIGHT`, `PANEL_GAP`, `CONTENT_HEIGHT_CAP`), `arrange` (a `PassInput`: the players, the model's layout, the middle area, the cartwall's `CartwallSizes`, the maximised panel; to an `Arrangement`: the players' row, each shown player's width, the overflow, the cartwall's rectangle, the splitter beside it, whether it fits its content) on every dock, and `cartwall_range`; the splitter maths (`drag_pair`, `pair_shares`, `LiveSplit` for the players, `CartwallDrag` for the cartwall); `panel_on_air`; the maximise state (`Maximised`, `ArrangementKey`); see [Main window layout](#main-window-layout); unit-tested |
```

- Replace the `ui/panels.rs` row with:

```markdown
| `ui/panels.rs` | The main window's panels (resizable layout spec): `players_row` draws the shown players at the pass's widths, each column keyed by its `PlayerId`, with `player_splitter` in each gap; `cartwall_drag_layout` and `cartwall_splitter` for the cartwall's splitter; the generic `splitter`; `layout_controls` (the top bar's tiles and layout menu); `header_menu` (Maximise, Restore, Hide); `settle_view`; see [Main window layout](#main-window-layout) |
```

- In the `ui/cartwall.rs`, `ui/cart_view.rs` row, replace "The cartwall strip," with "The cartwall panel (at the bottom or on a side, and its collapsed side strip, `collapsed_strip`)," and replace the sentence that begins "Button height comes from `cart_view::button_height(available, rows)`" (it ends "the grid scrolls only below the minimum.") with: "Button height comes from `cart_view::button_height(available, rows)` (nominal 40 px, minimum 28 px) while the cartwall fits its content (`Arrangement::fit_content`; the 60 % cap is `panel_layout::CONTENT_HEIGHT_CAP`), and from `cart_view::filled_button_height` (minimum 28 px, no upper clamp) otherwise (RL13); widths fill the panel, never under `cart_view::MIN_BUTTON_WIDTH`; the grid scrolls only below the minimums. `cartwall::sizes` gives the pass its content and least heights, its least width (`min_width`: the header's fixed parts or the columns, whichever is larger) and its collapsed thickness."
- Read the whole "Main window layout" section once more: no sentence may still say the side docks are drawn at the bottom.

- [ ] **Step 4: The specs**

`docs/superpowers/specs/2026-10-21-resizable-layout-design.md` (each a clarifying sentence after the text it clarifies, as plan 2 did for RL11):

- RL8, after the cartwall width item: "Stop all is measured with a two-digit count, so the least width does not change while carts start and stop."
- RL12, at the end of the **Lock** bullet: "At the bottom a collapsed cartwall is only its header, so its splitter does not react either."
- RL13, at the end of the **Size** bullet: "The same holds whenever the panel does not fit its content: maximised, or alone in the middle area."
- RL13, at the end of the **Text** bullet: "Centred is vertical: the text block of a 40 px button sits in the middle of a taller one."
- RL14, at the end: "At the bottom a collapsed cartwall is its header even when it has a share. A collapsed cartwall cannot be maximised, and collapsing a maximised cartwall ends the maximise."

`docs/superpowers/specs/2026-09-26-phase2-cartwall-settings-design.md`, P2.9, after the line "  - a grid of `cols` columns, each button 40 px tall.":

```markdown
  Amended by the [resizable layout spec](2026-10-21-resizable-layout-design.md):
  the cartwall can also sit on the left or the right of the players, the
  full height of the window, and a splitter resizes it (RL6, RL12); with a
  size of its own its buttons fill it (RL13), and collapsed on a side it is
  a narrow strip (RL14). Under the players and fitting its content it is as
  described here.
```

- [ ] **Step 5: The README**

`README.md`, Features:

- After the **Independent players** bullet, add:

```markdown
- **Resizable layout:** drag the gaps to resize the players and the
  cartwall, put the cartwall under the players or on their left or right,
  hide the panels a shift does not use (a hidden panel that is on air keeps
  playing and shows a red dot on its tile), maximise one for a moment, and
  lock the layout against stray drags. The layout is saved with the
  session.
```

- In the **Cartwall** bullet, replace "The configured rows always fit the window\n  (the buttons shrink to a minimum before the area scrolls)." with:

```markdown
It sits under the players or beside
  them: under them the configured rows always fit the window (the buttons
  shrink to a minimum before the area scrolls), and resized or beside them
  the buttons grow to fill it.
```

(rewrap the bullet to the file's width).

- [ ] **Step 6: Check and commit**

Run: `scripts/site/build.sh "$SCRATCH/site"` (with `gh` logged in, or `FAUSTE_RELEASE_JSON` set as `scripts/site/build.sh` documents) and `scripts/site/check-links.sh "$SCRATCH/site"`, where `$SCRATCH` is your scratchpad directory.
Expected: the site builds and no link is dead (the anchors `layout.md#move-and-resize-the-cartwall`, `#show-and-hide-panels`, `#maximise-a-panel`, `#the-layout-menu` resolve). Then the gate (docs only, but cheap insurance).

```bash
git add docs/user/cartwall.md docs/user/layout.md docs/technical/ui.md \
  docs/superpowers/specs/2026-10-21-resizable-layout-design.md \
  docs/superpowers/specs/2026-09-26-phase2-cartwall-settings-design.md README.md
git commit -m "docs: the cartwall's dock, splitter and scaling

The guide's cartwall and layout pages describe the side docks, the
cartwall's splitter, buttons that grow with it and the collapsed side
strip; the technical docs list the new parts of the pass; the README gains
the resizable layout; the Phase 2 spec's P2.9 notes the amendment, and the
resizable layout spec gets the clarifications its plans ruled on.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: Retake the screenshots

**Files:**
- Modify: `docs/images/main-screen.png`, `docs/images/guide/*.png` (all retaken)
- Modify: `scripts/site/screenshots.sh` (only if a crop no longer frames its subject)

**Interfaces:**
- Consumes: the finished interface (plans 2 and 3). The default layout is unchanged (plan 2's `rl11_the_default_layout_draws_todays_window` pins the 1920 × 1080 rectangles the crops use); the top bar now has the panel tiles and the layout button on its left.
- Produces: the committed PNGs.

- [ ] **Step 1: Take them**

Run: `scripts/site/screenshots.sh`
Expected: it ends with "done" and rewrites `docs/images/main-screen.png` and the seventeen `docs/images/guide/*.png` (about, cartwall, cue-window, player, playlist, settings-analysis, settings-cartwall, settings-loudness, settings-meters, settings-midi, settings-outputs, settings-players, settings-playlists, settings-remote, settings-shortcuts, tag-editor, track-menu). It needs `xvfb`, `xdotool`, ImageMagick, `ffmpeg`, `python3` and `curl` (CLAUDE.md, Testing notes). If it dies, use superpowers:systematic-debugging; do not loosen a check.

- [ ] **Step 2: Check every PNG**

Open each of the eighteen files (the Read tool shows images) and check:
- `main-screen.png`: the top bar shows **P1**–**P4**, **Carts** and the layout button on its left, all lit; the four players and the cartwall are as before; nothing overlaps.
- `guide/player.png` (`472x262+7+41`) and `guide/playlist.png` (`472x614+7+309`): player 1's top block and its playlist, cut at the same places as before (the crop starts under the top bar).
- `guide/cartwall.png` (`1920x124+0+932`): the whole cartwall, header and both rows, 40-point buttons, the playing cart's colour.
- the windows and menus (`cue-window`, `track-menu`, `tag-editor`, `about`, the ten `settings-*`): whole, centred in their crops, no tooltip or hover left over.

If a crop no longer frames its subject, adjust its numbers in `shoot_guide` and take them again; record the change as a ruling.

- [ ] **Step 3: A long language**

Run: `scripts/site/screenshots.sh --lang de-DE --out "$SCRATCH/shots-de"`
Expected: it ends with "done" (so `topbar_buttons` still found About and Settings in the right half of the bar: the tiles stay on the left). Open `$SCRATCH/shots-de/main-screen.png`: the tiles and the layout button do not touch the right half's buttons. These files are not committed (CI makes the translated guide's screenshots on publish).

- [ ] **Step 4: Commit**

```bash
git status --short docs/images scripts/site
git add docs/images
git commit -m "docs: retake the screenshots with the layout controls

The top bar gained a tile per panel and the layout button; every guide
screenshot and the README image are taken again from the scripted scene.
The default layout, and so every crop, is unchanged.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

(Add `scripts/site/screenshots.sh` to the commit only if Step 2 changed it, and say so in the body.)

---

### Task 7: Whole-feature verification, review and merge

**Files:**
- Modify: only what a finding requires (each fix test-first, in its own commit)
- Update: `.superpowers/sdd/itr05/progress.md` (git-ignored)

**Interfaces:**
- Consumes: plans 1–3 on `feat/resizable-layout`.
- Produces: the merged pull request.

- [ ] **Step 1: The gate, from clean**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: all green. Then the feature's own suites by name, to read their counts: `cargo test -p fp-model --test layout`, `cargo test -p fp-store --test session_layout`, `cargo test -p fp-app --test panel_layout --test layout_ui --test cartwall_ui`.

- [ ] **Step 2: Every rule has a test**

Run:

```bash
for n in $(seq 1 19); do
  printf 'RL%-3s %s\n' "$n" "$(grep -rhoE "fn rl${n}_[a-z0-9_]+" crates | wc -l)"
done
```

Expected: every line shows at least 1. For each rule, compare its tests with spec §7's list for it (the "Model", "Restore and store" and "Interface" items); a §7 item without a test is a gap: add the test (it must fail first if it guards new behaviour) in the plan and the crate that own the rule.

- [ ] **Step 3: By hand**

Start the app on a scratch home: `FAUSTE_HOME="$SCRATCH/home" cargo run -p fp-app` (or `scripts/site/screenshots.sh --hold` on Xvfb, driven with `xdotool`). Check:
1. Dock the cartwall left, then right, then at the bottom from the layout menu.
2. Drag its gap on each dock; the players keep their minimum; double-click goes back.
3. Collapse it on a side: the strip, its arrow, the red dot while a cart plays; expand: same size as before.
4. Drag a player gap, hide a player that is playing (its tile shows the dot, its key still stops it), maximise a player and drag the cartwall's gap.
5. Lock the layout: tiles, menu entries, gaps and the headers' menu do nothing; collapsing still works.
6. Quit and start again: the layout (dock, sizes, hidden panels, lock) comes back; nothing plays by itself.

Note anything unexpected in the ledger and debug it with superpowers:systematic-debugging before going on.

- [ ] **Step 4: The ledger**

In `.superpowers/sdd/itr05/progress.md`, section P13, plan 3: every task done, every ruling of this plan (and any made during execution) as `Ruling: <decision> — <why> — <cost if wrong>`, and what Step 3 showed.

- [ ] **Step 5: Review**

Use superpowers:requesting-code-review on the whole branch (`git diff origin/master...feat/resizable-layout`), with a fresh reviewer on the most capable model, giving it the spec and the three plans. Fix every Critical and Important finding test-first (each in its own commit, through the gate); record the Minor ones in the ledger as deferred.

- [ ] **Step 6: The pull request**

```bash
git fetch origin
git rebase origin/master
cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
scripts/check-commits.sh origin/master
git push -u origin feat/resizable-layout
gh pr create --title "feat(ui): resizable layout" --body-file "$SCRATCH/pr.md"
```

`$SCRATCH/pr.md` follows `.github/pull_request_template.md` (its documentation checklist ticked against what the three plans changed: README, the guide's layout, cartwall, players, playlists and keyboard pages, `docs/technical/ui.md`, `persistence.md`, `architecture.md`, the specs and plans, every locale file), summarises the feature by rule groups (model and session; players, top bar, lock and maximise; the cartwall's dock, splitter and scaling) and ends with:

```
🤖 Generated with [Claude Code](https://claude.com/claude-code)
```

- [ ] **Step 7: CI and merge**

Run: `gh pr checks --watch`, then, as a separate step, `gh pr checks` and read it: every check on all three systems must have passed. A red check is fixed on the branch (test-first when it is a bug), never bypassed. When it is green and the review is done: rebase on the latest `origin/master` again (`git fetch origin && git rebase origin/master && git push --force-with-lease`), wait for CI green again and confirm it, then `gh pr merge --merge --delete-branch`, and update the local `master` (`git checkout master && git pull --ff-only`).

---

## Self-review (against the spec)

- RL6 on screen: Task 1 (pure: left, right, default share, no player shown; UI: left, right, the menu moves it). The model side is plan 1's.
- RL8: the cartwall's least width (Task 1: pure `rl8_a_side_cartwall_keeps_its_least_width`; UI: the grid's columns, a one-column page's header, the width while carts play); its least height and the players' minimums are plan 2's.
- RL11 on all three docks: Task 1 (side share clamps, too narrow with four players, narrower than the cartwall, no player shown; UI: a side dock in a narrow window, both sides). The bottom is plan 2's.
- RL12, the cartwall splitter: Task 2 (pure: bottom and side directions, the range clamp, one command, the hold, a drag back, what ends a drag, the pass draws the drag; UI: live bottom drag, one command and the size stays, a click, double-click at the bottom and on a side, the cursors, side drags, the players' least height, locked, collapsed on both docks, locking mid-drag, a drag ends a maximise).
- RL13: Task 3 (pure `filled_button_height`; UI: today's buttons, a bottom share, a side dock, the scroll at the minimum, centred text); `fit_content` in Task 1.
- RL14: Task 1 (pure: the strip's width, the bottom header with a share, never maximised); Task 4 (UI: the strip and its caret on both sides, its width equals the header's height, the caret expands to the share, the red dot, the menu, Maximise disabled at the bottom, collapsing ends a maximise); the inert splitter in Task 2.
- RL16 on a side: Task 1 (pure and UI); a cartwall drag ends a maximise: Task 2.
- §6: guide (Tasks 1–5), `ui.md` (Tasks 1–5), README and the Phase 2 amendment (Task 5), screenshots (Task 6); remote API and MIDI unchanged (no task touches them).
- §7 "Interface": RL11's "shrinking the window below the minimums, on all three docks" (Task 1 and plan 2), RL12's list (plan 2 and Task 2), RL13's three items (Task 3), RL14's "a side dock collapsed, then expanded" (Task 4).
- Review Focus: each line names its test, and each test is in its task's code.
- Placeholders: none; the plan-2 code this plan edits is named by function and quoted where an edit replaces it.
- Types: `CartwallSizes { content_height, min_height, min_width, collapsed }`, `Arrangement { players_area, widths, overflow, cartwall, splitter, fit_content }`, `cartwall_range(&PassInput) -> Option<(f32, f32)>`, `CartwallDrag::{new, is_dragging, follow, size, share, release, stale}`, `cartwall_drag_layout`, `cartwall_splitter`, `strip(…, fill)`, `filled_button_height`, `MIN_BUTTON_WIDTH` match between the tasks that define and use them.
