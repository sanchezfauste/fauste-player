# Resizable Layout, Plan 2: the Players, the Top Bar, the Lock and Maximise

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The main window draws the players from the model's layout (RL8's player minimums, RL11's pixel pass for the players and the bottom cartwall), the players can be resized with splitters (RL12), panels can be hidden and shown from the top bar, which also has the layout menu with the lock (RL15), a panel can be maximised from its header (RL16), and hiding a player clears its selection (RL17) and redirects file drops (RL18).

**Architecture:** A new pure module `fp_app::ui::panel_layout` holds the interface constants (RL8), the pixel pass `arrange(&PassInput) -> Arrangement`, the splitter maths (`drag_pair`, `pair_shares`, `LiveSplit`), `panel_on_air` and the maximise state (`Maximised`, `ArrangementKey`); it is unit-tested like `table_layout`. A new private module `ui/panels.rs` draws with it: the players' row with its splitters (moved out of `app.rs`), the generic `splitter` interaction, the top bar's panel tiles and layout menu, the headers' menu and `settle_view`, which brings the view state in line with the model before the keyboard runs. `AppUi::ui` calls the pass once per frame (twice while a panel is maximised: the second, without it, is where a splitter drag starts). The model is plan 1's; nothing in `fp-model` changes.

**Tech Stack:** Rust 2024, egui 0.36 (`Popup::menu`, `Response::context_menu`, `Sense::click_and_drag`), `egui_kittest` 0.36, Fluent.

**Spec:** `docs/superpowers/specs/2026-10-21-resizable-layout-design.md` (§2, RL7's shortcut, RL8, RL11, RL12, RL15–RL18, §6, §7 "Interface"). Read it whole before starting, and read plan 1's "What plans 2 and 3 consume".

This is plan 2 of 3, executed strictly in order on one branch, `feat/resizable-layout`, after plan 1:
1. `2026-10-21-p13-plan1-layout-model.md` — the model, the session document, the error strings (done before this plan starts);
2. **this plan** — the pixel pass for the players and the bottom cartwall, the player splitters, the top-bar tiles and layout menu, the lock in the interface, maximise, RL8's player minimums, RL17 and RL18, and their docs;
3. `2026-10-21-p13-plan3-…` — the side docks in the pixel pass and on screen, the cartwall splitter on every dock, RL13 (buttons scale), RL14 (collapsed on a side), the cartwall's minimum width, the remaining docs, the README and the screenshots.

Between plan 2 and plan 3 the dock entries of the layout menu change the model (and end a maximise), but a cartwall docked on a side is still drawn at the bottom.

## Rulings (planning)

Record each one in `.superpowers/sdd/itr05/progress.md` (section P13, plan 2) when the task that applies it is done.

**Ruling: the cartwall splitter, on every dock, is plan 3's.** Plan 2 builds the splitter interaction generic over its direction (`panels::splitter`, Task 3) and the pass already honours a bottom share, but no splitter sets it. — Why: RL12's cartwall splitter has one behaviour per dock (horizontal at the bottom, vertical on a side, inert when collapsed on a side) and its size only becomes useful with RL13 (without it, a taller bottom cartwall shows empty space under 40-point buttons). One task in plan 3 then covers the three docks. — Cost if wrong: one task moves back here.

**Ruling: at the bottom with `size: None` ("fit the content") the cartwall's height is exactly today's: the content height, at most 60 % of the middle area, with no further clamp.** RL11's clamp (between the cartwall's minimum and what leaves the players `MIN_PLAYER_HEIGHT`) applies to a share (`Some`) and to a maximised panel. — Why: RL13 says that the panel with `None` is "exactly today's", and §6 says the default window does not change; clamping `None` to the players' minimum height would shrink a tall cart grid in a low window that today shows fine (and change the button heights `cart_view::button_height` gives). The spec's RL11 gets a clarifying sentence (Task 2). — Cost if wrong: one more `.min(room).max(min)` in `at_the_bottom` and two test numbers.

**Ruling: until plan 3, every dock is laid out at the bottom.** `arrange` calls `at_the_bottom` for every dock; plan 3 replaces that one call with a `match` on the dock (the extension point, see "What plan 3 consumes"). — Why: the caller of this plan asked for the side docks to be plan 3's. — Cost if wrong: none; the branch is merged only after plan 3.

**Ruling: a splitter drag sends each player's room above its minimum as its share.** `pair_shares((540, 420))` is `(160, 40)` with `MIN_PLAYER_WIDTH` 380. The pass gives each shown player its minimum plus its fraction of the spare room, and the model scales the pair to keep its sum (plan 1), so the next pass draws exactly the dragged widths. A player dragged to its minimum sends `SHARE_FLOOR` (1e-4) of the pair's room, because the model refuses 0; a pair with no room above its minimums sends nothing. — Why: sending the pixel widths would not reproduce them (the minimums are added before the shares). — Cost if wrong: one function.

**Ruling: a splitter is exactly the 8-point gap between two columns, and it senses `click_and_drag`.** The drag starts after egui's drag threshold, so a click is never a drag (nothing is sent) and a double-click is `EqualPlayerWidths`. Nothing is painted at rest (the default window looks as today); a line is painted while it is hovered or dragged, as the table's column edges do. — Why: a grab zone wider than the gap would overlap the tables' vertical scroll bars at the columns' right edges. — Cost if wrong: one `expand` on the rect.

**Ruling: "on air" for a tile's red dot is `transport != Stopped`** (playing or paused), which is `fp_model::on_air`'s definition; for the cartwall, any cart playing. — Why: RL15 "a current entry that is not stopped"; a paused player still holds its entry on air. — Cost if wrong: one `matches!`.

**Ruling: a tile's accessible label (and tooltip) says what a click does:** "Hide P2", "Show P2", "Show P2 (on air)", "Hide Carts". The painted text stays `P2` or `Carts`. A disabled tile's tooltip reuses `error-last-panel` or `error-layout-locked`. — Why: the bare "P1" is already the label of the close guard's list rows (`tests/exit_guard.rs` queries it), and a label that names the action reads right to a screen reader. — Cost if wrong: strings.

**Ruling: the header menu keeps its two entries while the layout is locked, disabled, and Maximise is also disabled when only one panel is shown.** — Why: a menu that appears with greyed entries says why nothing happens; maximising the only panel changes nothing. — Cost if wrong: one condition.

**Ruling: "a second Maximise" (RL16) is Maximise on another panel: it moves the maximise there.** The maximised panel's own menu shows **Restore**, which ends it. Either way the layout is untouched, so ending it always brings back the sizes from before. — Why: the menu can only show one of Maximise and Restore per panel; the reading where Maximise on a second panel first restores and then maximises that panel gives the same pixels. — Cost if wrong: one branch in `header_menu`.

**Ruling: RL17 runs every frame, before the keyboard** (`panels::settle_view`, called right before `AppUi::keyboard`). It drops the selection of every hidden player and forgets a hidden last player used. — Why: a hide can come from the top bar, a header menu or a session; doing it before the keyboard means Delete can never act on a hidden player, whatever hid it. — Cost if wrong: none known.

**Ruling: with no player shown, playlist files dropped on the window are still imported** (they become new playlists, which need no player); only audio files are refused, with the notice. — Why: RL18 is about where tracks go; an import has no target player. — Cost if wrong: an early return.

**Ruling: `MIN_PLAYER_HEIGHT` is the players' row, 432 points: the column's 430 plus the 2 points the row keeps below each column.** The column's 430 is its border (2), the frame's margins (20), header (20), info row and transport (136), waveform (56), time line (14 − 4), the item gaps (3 × 8), tabs (30), table header (24), three rows (3 × 28) and footer (24). Task 2's `rl8_three_rows_fit_a_player_at_its_least_height` checks it on screen; if it fails, raise the constant by what the test reports and record the change here. — Cost if wrong: one constant.

**Ruling: the README and guide screenshots are retaken in plan 3, not here.** The default layout does not change (Task 1 pins the 1920 × 1080 rectangles the crops use), but the top bar gains the tiles and the layout button, so `docs/images/main-screen.png` changes; plan 3 changes the cartwall's look too, so one retake at the end covers both. `scripts/site/screenshots.sh` needs no change: `topbar_buttons` reads only the right half of the bar, and the tiles are on the left. — Cost if wrong: one script run.

## Global Constraints

- All code, identifiers, comments, docs and commit messages in English; never mention other playout or radio-automation products.
- Behaviour lives in `fp-model` (CLAUDE.md rule 7). Plan 2 adds no model rule: the layout's rules are plan 1's; the interface computes rectangles and view state only. RL17 and RL18 are interface rules (plan 1's ruling).
- RL8: "Minimum sizes come from the panels' content, so they are interface constants, not operator limits" — no `Config` field.
- RL11: "Only when the minimums alone do not fit does the players' row scroll horizontally, as today." "A column's egui id comes from its `PlayerId`, not from its position among the shown players. Its label stays `P<n>`, from its place among all players."
- RL12: "When the drag ends, `SetPlayerShares` is sent once, and only if the drag moved something. Until the model's echo arrives, the dragged widths are held." "While the layout is locked … its splitter does not react, and the pointer over it shows the normal cursor."
- RL15: "While the layout is locked, every entry except `Lock layout` is disabled, and so are the panel toggles." "The last lit tile is disabled."
- RL16: "it lives in `ViewState`, is never saved and sends no command. It is disabled while the layout is locked." "Locking, hiding or showing any panel, a dock change and a change in the number of players end it too. A splitter drag first restores the earlier sizes, then applies itself."
- RL7: "a hidden player that is on air keeps playing. Keys, MIDI, the remote API, the CUE windows and automation act on it as before." "The shortcut that toggles the cartwall does nothing while it is hidden."
- §6: "The README and guide screenshots use the default layout, which does not change."
- The UI never blocks (rule 8); never crash on bad data (rule 9); `unwrap`, `expect`, `panic` denied outside tests; prefer `get` over indexing.
- In `fp-app`, write `fp_model::layout::Layout` (never a bare `Layout` import: `egui::Layout` is everywhere); `Panel` and `Dock` come from `fp_model::layout`.
- `SetPlayerShares` refuses a share of 0 and anything not finite (plan 1); the model scales the pair to keep its sum.
- Every new string goes into all eleven locale files (`crates/fp-app/locales/{en-US,es-ES,ca-ES,de-DE,eu-ES,fr-FR,gl-ES,it-IT,nl-NL,pl-PL,pt-PT}/main.ftl`): en-US and es-ES written, the other nine AI-translated in the same change. Spanish names the layout "distribución" and the cartwall "cartuchera" and uses "player" (as plan 1 and the existing file). `docs/i18n/` is never touched.
- Test names start with the rule (`rl11_…`, `rl16_…`); UI tests use `egui_kittest` with the `support` harness (`with_step_dt(0.02)`), whose window has an 8-point outer margin: assert relations between rectangles, not absolute coordinates.
- Conventional Commits, scope `ui` (docs-only commits `docs`); every commit ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- `cargo` is at `$HOME/.cargo/bin` (`export PATH="$HOME/.cargo/bin:$PATH"`). Gate for every task: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`. Unused imports in a test file fail clippy: each task gives the test file's whole `use` block.
- Progress ledger: `.superpowers/sdd/itr05/progress.md` (section P13, plan 2). Record deviations as `Ruling: <decision> — <why> — <cost if wrong>`.

## Review Focus

1. **The default 1920 × 1080 window must draw today's rectangles** (four players of 470 points, the cartwall from y 932 to 1048), because the screenshot crops are fixed numbers. Test: Task 1, `rl11_the_default_layout_draws_todays_window`.
2. **The window is resized, or a player is removed (remote `SetPlayerCount`), in the middle of a splitter drag**: the drag ends without a command and the columns follow the new window. Test: Task 3, `rl12_a_window_resized_or_a_player_gone_ends_a_drag`.
3. **A hidden player that is on air must still answer its number key** (an operator hides P2 to make room and then cannot stop it from the keyboard). Test: Task 4, `rl7_a_hidden_player_still_answers_its_key`.
4. **A session saved locked**: from the first frame the tiles and the menu entries are disabled and the splitters inert. Tests: Task 3, `rl12_a_locked_layout_has_inert_splitters`; Task 4, `rl15_a_locked_layout_disables_the_tiles_and_the_menu` (both build the state locked before the first frame).
5. **Maximising the only shown panel, or hiding the maximised panel from its tile**: nothing is maximised and the window is the normal one. Tests: Task 5, `rl16_with_one_panel_shown_maximise_is_disabled`, `rl16_hiding_locking_docking_or_a_new_player_count_end_it`.

---

## File Structure

| File | Responsibility |
|---|---|
| `crates/fp-app/src/ui/panel_layout.rs` (new, `pub`) | RL8 constants; the pixel pass (`CartwallSizes`, `PassInput`, `Arrangement`, `arrange`); the splitter maths (`drag_pair`, `pair_shares`, `LiveSplit`, `HOLD_SECS`, `SHARE_FLOOR`); `panel_on_air`; `ArrangementKey`, `Maximised` |
| `crates/fp-app/src/ui/panels.rs` (new, private) | `players_row` (moved from `app.rs`), `live_widths`, `splitter`, `player_splitter`, `layout_controls` (tiles and menu), `header_menu`, `settle_view` |
| `crates/fp-app/src/ui.rs` | `pub mod panel_layout;`, `mod panels;` |
| `crates/fp-app/src/ui/app.rs` | `ViewState` fields; `AppUi::ui` calls the pass; accessors `player_rects`, `cartwall_rect`, `maximised`; `top_bar` calls `layout_controls`; `shortcut_command` (RL7); `file_drops` (RL18); `MIN_COLUMN_WIDTH` and `players_row` removed |
| `crates/fp-app/src/ui/cartwall.rs` | `min_height`, `sizes`; the header's background menu |
| `crates/fp-app/src/ui/player.rs` | the header's background menu |
| `crates/fp-app/locales/*/main.ftl` | the tiles, the menu, the header menu, the drop notice |
| `crates/fp-app/tests/panel_layout.rs` (new) | the pure maths |
| `crates/fp-app/tests/layout_ui.rs` (new) | the window, driven headlessly |
| `docs/technical/ui.md` | structure rows; "Main window layout" section |
| `docs/user/layout.md` (new), `docs/user/SUMMARY.md`, `players.md`, `playlists.md`, `keyboard.md`, `cartwall.md` | the guide |
| `docs/superpowers/specs/2026-10-21-resizable-layout-design.md` | RL11's sentence on "fit the content" |

---

### Task 1: The pixel pass (RL8 constants, RL11, RL16's sizes)

**Files:**
- Create: `crates/fp-app/src/ui/panel_layout.rs`
- Modify: `crates/fp-app/src/ui.rs` (module list, L3-31)
- Modify: `docs/technical/ui.md` (Structure table, after the `ui/table_layout.rs` row, L19)
- Test: `crates/fp-app/tests/panel_layout.rs` (new)

**Interfaces:**
- Consumes (plan 1): `fp_model::layout::{Layout, Panel}`, `Layout::shown_players(&[PlayerId]) -> Vec<PlayerId>`, `Layout::shown_fractions(&[PlayerId]) -> Vec<(PlayerId, f32)>` (adds up to 1), `Layout::is_hidden(Panel)`, `CartwallLayout::share() -> Option<f32>`, `AppState::player_ids() -> Vec<PlayerId>`.
- Produces (`fp_app::ui::panel_layout`, all `pub`):
  - `const PANEL_GAP: f32 = 8.0; const MIN_PLAYER_WIDTH: f32 = 380.0; const MIN_PLAYER_HEIGHT: f32 = 432.0; const CONTENT_HEIGHT_CAP: f32 = 0.6;`
  - `struct CartwallSizes { pub content_height: f32, pub min_height: f32 }` (`Debug, Clone, Copy, PartialEq`)
  - `struct PassInput<'a> { pub players: &'a [PlayerId], pub layout: &'a fp_model::layout::Layout, pub area: Rect, pub cartwall: CartwallSizes, pub maximised: Option<Panel> }` (`Debug, Clone, Copy`)
  - `struct Arrangement { pub players_area: Rect, pub widths: Vec<(PlayerId, f32)>, pub overflow: bool, pub cartwall: Option<Rect> }` (`Debug, Clone, PartialEq`)
  - `fn arrange(input: &PassInput<'_>) -> Arrangement`

- [ ] **Step 1: Write the failing tests**

Create `crates/fp-app/tests/panel_layout.rs`:

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Resizable layout spec RL8, RL11 and RL16: the pure pixel pass, one test
//! per rule, named after it.

use egui::{Rect, pos2, vec2};
use fp_app::ui::panel_layout::{
    Arrangement, CONTENT_HEIGHT_CAP, CartwallSizes, MIN_PLAYER_HEIGHT, MIN_PLAYER_WIDTH,
    PANEL_GAP, PassInput, arrange,
};
use fp_model::layout::Panel;
use fp_model::{AppState, Config, PlayerId};

/// The middle area of a 1920 × 1080 window, inside its 8-point margin:
/// under the 34-point top bar, over the 24-point status bar.
fn full_hd() -> Rect {
    Rect::from_min_max(pos2(8.0, 42.0), pos2(1912.0, 1048.0))
}

/// A page of two rows: today's content height (24 + 6 + 2 × 40 + 6) and
/// the least height (the rows at 28).
const SIZES: CartwallSizes = CartwallSizes {
    content_height: 116.0,
    min_height: 92.0,
};

/// The default state: four players.
fn four() -> AppState {
    AppState::new(Config::default(), "Main")
}

fn pass_with(s: &AppState, area: Rect, sizes: CartwallSizes, maximised: Option<Panel>) -> Arrangement {
    let players = s.player_ids();
    arrange(&PassInput {
        players: &players,
        layout: &s.layout,
        area,
        cartwall: sizes,
        maximised,
    })
}

fn pass(s: &AppState, area: Rect, maximised: Option<Panel>) -> Arrangement {
    pass_with(s, area, SIZES, maximised)
}

fn widths(a: &Arrangement) -> Vec<f32> {
    a.widths.iter().map(|(_, w)| *w).collect()
}

fn ids(a: &Arrangement) -> Vec<PlayerId> {
    a.widths.iter().map(|(p, _)| *p).collect()
}

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.01
}

fn all_near(got: &[f32], want: &[f32]) -> bool {
    got.len() == want.len() && got.iter().zip(want).all(|(a, b)| near(*a, *b))
}

#[test]
fn rl11_the_default_layout_draws_todays_window() {
    let s = four();
    let a = pass(&s, full_hd(), None);
    // (1904 − 3 × 8) / 4, as the old equal split.
    assert!(all_near(&widths(&a), &[470.0; 4]), "{a:?}");
    assert_eq!(ids(&a), s.player_ids());
    assert!(!a.overflow);
    // The screenshot script crops the cartwall from y 932.
    assert_eq!(
        a.cartwall,
        Some(Rect::from_min_max(pos2(8.0, 932.0), pos2(1912.0, 1048.0)))
    );
    assert_eq!(
        a.players_area,
        Rect::from_min_max(pos2(8.0, 42.0), pos2(1912.0, 924.0))
    );
}

#[test]
fn rl11_the_spare_room_is_split_by_share() {
    let mut s = four();
    let p = s.player_ids();
    s.layout.player_shares = vec![(p[0], 3.0), (p[1], 1.0), (p[2], 1.0), (p[3], 1.0)];
    let a = pass(&s, full_hd(), None);
    // 1904 − 3 gaps − 4 × 380 = 360 spare: 3/6 of it, then 1/6 each.
    assert!(all_near(&widths(&a), &[560.0, 440.0, 440.0, 440.0]), "{a:?}");
}

#[test]
fn rl11_hidden_players_leave_their_room_to_the_others() {
    let mut s = four();
    let p = s.player_ids();
    s.layout.hidden_players = vec![p[1]];
    let a = pass(&s, full_hd(), None);
    assert_eq!(ids(&a), vec![p[0], p[2], p[3]]);
    // (1904 − 2 × 8) / 3
    assert!(all_near(&widths(&a), &[1888.0 / 3.0; 3]), "{a:?}");
}

#[test]
fn rl11_only_a_window_narrower_than_the_minimums_scrolls() {
    let s = four();
    let narrow = Rect::from_min_size(pos2(8.0, 42.0), vec2(968.0, 610.0));
    let a = pass(&s, narrow, None);
    assert!(a.overflow);
    assert!(all_near(&widths(&a), &[MIN_PLAYER_WIDTH; 4]));
    let exact = Rect::from_min_size(
        pos2(8.0, 42.0),
        vec2(4.0 * MIN_PLAYER_WIDTH + 3.0 * PANEL_GAP, 610.0),
    );
    let a = pass(&s, exact, None);
    assert!(!a.overflow, "the minimums fit exactly");
    assert!(all_near(&widths(&a), &[MIN_PLAYER_WIDTH; 4]));
}

#[test]
fn rl11_a_hidden_cartwall_leaves_the_players_the_middle_area() {
    let mut s = four();
    s.layout.cartwall.hidden = true;
    let a = pass(&s, full_hd(), None);
    assert_eq!(a.cartwall, None);
    assert_eq!(a.players_area, full_hd());
}

#[test]
fn rl11_with_no_player_shown_the_cartwall_takes_the_middle_area() {
    let mut s = four();
    s.layout.hidden_players = s.player_ids();
    let a = pass(&s, full_hd(), None);
    assert!(a.widths.is_empty());
    assert!(!a.overflow);
    assert_eq!(a.cartwall, Some(full_hd()));
}

#[test]
fn rl11_fitting_the_content_is_capped_at_sixty_percent_as_before() {
    let s = four();
    let tall = CartwallSizes {
        content_height: 900.0,
        min_height: 300.0,
    };
    let a = pass_with(&s, full_hd(), tall, None);
    assert!(near(a.cartwall.unwrap().height(), 1006.0 * CONTENT_HEIGHT_CAP));
}

#[test]
fn rl11_a_bottom_share_keeps_both_minimums() {
    let mut s = four();
    s.layout.cartwall.size = Some(0.95);
    let a = pass(&s, full_hd(), None);
    // RL8: the players keep their least height.
    assert!(near(
        a.cartwall.unwrap().height(),
        1006.0 - PANEL_GAP - MIN_PLAYER_HEIGHT
    ));
    assert!(near(a.players_area.height(), MIN_PLAYER_HEIGHT));
    s.layout.cartwall.size = Some(0.05);
    let a = pass(&s, full_hd(), None);
    assert!(
        near(a.cartwall.unwrap().height(), SIZES.min_height),
        "the cartwall keeps its own"
    );
}

#[test]
fn rl11_too_low_a_window_keeps_the_cartwall_minimum() {
    let mut s = four();
    s.layout.cartwall.size = Some(0.5);
    let low = Rect::from_min_size(pos2(8.0, 42.0), vec2(1904.0, 400.0));
    let a = pass(&s, low, None);
    assert!(near(a.cartwall.unwrap().height(), SIZES.min_height));
    assert!(near(a.players_area.height(), 400.0 - SIZES.min_height - PANEL_GAP));
}

#[test]
fn rl16_a_maximised_player_takes_the_spare_room_and_the_others_their_minimum() {
    let mut s = four();
    let p = s.player_ids();
    s.layout.player_shares = vec![(p[0], 3.0)];
    let a = pass(&s, full_hd(), Some(Panel::Player(p[1])));
    assert!(
        all_near(&widths(&a), &[380.0, 740.0, 380.0, 380.0]),
        "{a:?}"
    );
    assert!(
        near(a.cartwall.unwrap().height(), SIZES.min_height),
        "the cartwall shrinks to its minimum too"
    );
}

#[test]
fn rl16_a_maximised_cartwall_leaves_the_players_their_least_height() {
    let s = four();
    let a = pass(&s, full_hd(), Some(Panel::Cartwall));
    assert!(near(a.players_area.height(), MIN_PLAYER_HEIGHT));
    assert!(all_near(&widths(&a), &[470.0; 4]), "the widths do not change");
}

#[test]
fn rl16_a_hidden_panel_is_never_maximised() {
    let mut s = four();
    let p = s.player_ids();
    s.layout.hidden_players = vec![p[0]];
    assert_eq!(
        pass(&s, full_hd(), Some(Panel::Player(p[0]))),
        pass(&s, full_hd(), None)
    );
    s.layout.cartwall.hidden = true;
    assert_eq!(
        pass(&s, full_hd(), Some(Panel::Cartwall)),
        pass(&s, full_hd(), None)
    );
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-app --test panel_layout`
Expected: FAIL to compile: no module `panel_layout` in `fp_app::ui`.

- [ ] **Step 3: Create `crates/fp-app/src/ui/panel_layout.rs`**

```rust
//! The main window's pixel pass (resizable layout spec RL11): from the
//! layout, the middle area and the panels' minimums (RL8) to the players'
//! widths and the cartwall's rectangle, with a maximised panel (RL16).
//! Pure and unit-tested; `ui/panels.rs` draws with it.

use egui::{Rect, Vec2, pos2};
use fp_model::PlayerId;
use fp_model::layout::Panel;

/// The gap between two panels; between two players it is a splitter
/// (RL12).
pub const PANEL_GAP: f32 = 8.0;
/// RL8: a player column's least width: its fixed parts (the widest meter
/// with its rulers, the fader with its scale, the transport) beside a
/// countdown at its smallest.
pub const MIN_PLAYER_WIDTH: f32 = 380.0;
/// RL8: the players' row's least height: a column's header, info row and
/// transport, waveform, time line, tabs, table header, three rows and
/// footer (430 points, borders and margins included), and the 2 points the
/// row keeps under each column.
pub const MIN_PLAYER_HEIGHT: f32 = 432.0;
/// RL13 ("fit the content"): a bottom cartwall that fits its content never
/// takes more than this share of the middle area's height, as before the
/// resizable layout.
pub const CONTENT_HEIGHT_CAP: f32 = 0.6;

/// The cartwall's heights from its content (`cartwall::sizes`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CartwallSizes {
    /// What its content wants at the bottom: the header, or the header and
    /// the shown page's rows at their nominal height.
    pub content_height: f32,
    /// RL8: its least height: the header, or the header and the rows at the
    /// smallest button.
    pub min_height: f32,
}

/// What the pass reads.
#[derive(Debug, Clone, Copy)]
pub struct PassInput<'a> {
    /// Every player, in display order (`AppState::player_ids`).
    pub players: &'a [PlayerId],
    pub layout: &'a fp_model::layout::Layout,
    /// The middle area, inside its margin.
    pub area: Rect,
    pub cartwall: CartwallSizes,
    /// RL16: the panel that takes all the free space for now.
    pub maximised: Option<Panel>,
}

/// Where everything goes this frame.
#[derive(Debug, Clone, PartialEq)]
pub struct Arrangement {
    /// The players' row (its horizontal scroll area). Empty when no player
    /// is shown.
    pub players_area: Rect,
    /// Each shown player with its column's width, in display order.
    pub widths: Vec<(PlayerId, f32)>,
    /// RL11: the minimums alone do not fit, so the row scrolls.
    pub overflow: bool,
    /// The cartwall's rectangle; `None` while it is hidden.
    pub cartwall: Option<Rect>,
}

/// RL11: the pass. The cartwall first, then the players split the rest:
/// each gets its minimum width and the spare room is split by share.
pub fn arrange(input: &PassInput<'_>) -> Arrangement {
    let layout = input.layout;
    let shown = layout.shown_players(input.players);
    // A hidden panel is never maximised (RL16).
    let maximised = input.maximised.filter(|panel| match panel {
        Panel::Player(id) => shown.contains(id),
        Panel::Cartwall => !layout.cartwall.hidden,
    });
    let area = input.area;
    let (players_area, cartwall) = if layout.cartwall.hidden {
        (area, None)
    } else if shown.is_empty() {
        (Rect::from_min_size(area.min, Vec2::ZERO), Some(area))
    } else {
        // Plan 3 makes this a `match` on `layout.cartwall.dock`, with an arm
        // for the side docks; until then every dock is laid out here.
        at_the_bottom(input, maximised)
    };
    let (widths, overflow) = player_widths(input, &shown, players_area.width(), maximised);
    Arrangement {
        players_area,
        widths,
        overflow,
        cartwall,
    }
}

/// The cartwall under the players: its content height ("fit the content",
/// exactly as before), or its share clamped between its minimum and what
/// leaves the players `MIN_PLAYER_HEIGHT` (RL8, RL11); its minimum next to
/// a maximised player, everything but the players' least height when it is
/// maximised itself (RL16).
fn at_the_bottom(input: &PassInput<'_>, maximised: Option<Panel>) -> (Rect, Option<Rect>) {
    let area = input.area;
    let sizes = input.cartwall;
    let room = area.height() - PANEL_GAP - MIN_PLAYER_HEIGHT;
    let height = match (maximised, input.layout.cartwall.share()) {
        (Some(Panel::Cartwall), _) => room.max(sizes.min_height),
        (Some(Panel::Player(_)), _) => sizes.min_height,
        (None, None) => sizes
            .content_height
            .min(area.height() * CONTENT_HEIGHT_CAP),
        (None, Some(share)) => (area.height() * share).min(room).max(sizes.min_height),
    };
    let top = (area.bottom() - height).max(area.top());
    let cartwall = Rect::from_min_max(pos2(area.left(), top), area.max);
    let players = Rect::from_min_max(
        area.min,
        pos2(area.right(), (top - PANEL_GAP).max(area.top())),
    );
    (players, Some(cartwall))
}

/// RL11: each shown player gets `MIN_PLAYER_WIDTH`, and the spare room of
/// `width` is split by the players' fractions (all of it to a maximised
/// player, RL16). When the minimums alone do not fit, every column is at
/// its minimum and the row overflows (half a point of rounding is not an
/// overflow, as before).
fn player_widths(
    input: &PassInput<'_>,
    shown: &[PlayerId],
    width: f32,
    maximised: Option<Panel>,
) -> (Vec<(PlayerId, f32)>, bool) {
    if shown.is_empty() {
        return (Vec::new(), false);
    }
    let count = shown.len() as f32;
    let spare = width - PANEL_GAP * (count - 1.0) - MIN_PLAYER_WIDTH * count;
    if spare <= 0.0 {
        let widths = shown.iter().map(|p| (*p, MIN_PLAYER_WIDTH)).collect();
        return (widths, spare < -0.5);
    }
    let fractions: Vec<(PlayerId, f32)> = match maximised {
        Some(Panel::Player(big)) => shown
            .iter()
            .map(|p| (*p, if *p == big { 1.0 } else { 0.0 }))
            .collect(),
        _ => input.layout.shown_fractions(input.players),
    };
    let widths = fractions
        .into_iter()
        .map(|(p, f)| (p, MIN_PLAYER_WIDTH + spare * f))
        .collect();
    (widths, false)
}
```

`crates/fp-app/src/ui.rs`: insert `pub mod panel_layout;` between `mod notice;` and `pub mod pending;` (the list is alphabetical).

- [ ] **Step 4: Run the tests**

Run: `cargo test -p fp-app --test panel_layout`
Expected: PASS (12 tests).

- [ ] **Step 5: The structure row**

`docs/technical/ui.md`, Structure table, after the `ui/table_layout.rs` row, add:

```markdown
| `ui/panel_layout.rs` | The main window's pure pixel pass (resizable layout spec RL11): RL8's interface constants (`MIN_PLAYER_WIDTH`, `MIN_PLAYER_HEIGHT`, `PANEL_GAP`, `CONTENT_HEIGHT_CAP`) and `arrange`, which turns a `PassInput` (the players, the model's layout, the middle area, the cartwall's `CartwallSizes`, the maximised panel) into an `Arrangement` (the players' row, each shown player's width, the overflow, the cartwall's rectangle); see [Main window layout](#main-window-layout); unit-tested |
```

- [ ] **Step 6: Gate and commit**

Run the gate. Then:

```bash
git add crates/fp-app/src/ui/panel_layout.rs crates/fp-app/src/ui.rs \
  crates/fp-app/tests/panel_layout.rs docs/technical/ui.md
git commit -m "feat(ui): the main window's pixel pass

RL8 and RL11 of the resizable layout spec: each shown player gets its
minimum width and the spare room is split by share; the row scrolls only
when the minimums alone do not fit. A bottom cartwall that fits its content
keeps today's height; a share is clamped between its minimum and what
leaves the players theirs. A maximised panel (RL16) takes the free space.
Pure and unit-tested; nothing draws with it yet.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

(The `ui.md` row links to a section Task 2 writes; the link checker runs on the built site, not on this commit.)

---

### Task 2: The window draws from the pass (RL11, RL8)

**Files:**
- Create: `crates/fp-app/src/ui/panels.rs`
- Modify: `crates/fp-app/src/ui.rs` (module list)
- Modify: `crates/fp-app/src/ui/app.rs` (`MIN_COLUMN_WIDTH` L40; `ViewState` L97-186; accessors after `column_widths` L558-562; `AppUi::ui` L692-716; `players_row` L1745-1788 removed)
- Modify: `crates/fp-app/src/ui/cartwall.rs` (imports L7-14; after `height` L19-28)
- Modify: `docs/technical/ui.md` (the `ui/cartwall.rs` row, L28; Structure table; a new section before "## Data flow", L41)
- Modify: `docs/superpowers/specs/2026-10-21-resizable-layout-design.md` (RL11, item 1)
- Test: `crates/fp-app/tests/layout_ui.rs` (new)

**Interfaces:**
- Consumes: Task 1's `arrange`, `PassInput`, `Arrangement`, `CartwallSizes`, `PANEL_GAP`; `player::column(ui, scene, view_state, covers, index)`; `AppState::player_index(PlayerId) -> Result<usize, ModelError>`.
- Produces:
  - `pub(crate) fn cartwall::min_height(scene: &Scene<'_>) -> f32` and `pub(crate) fn cartwall::sizes(scene: &Scene<'_>) -> CartwallSizes`.
  - `pub(crate) fn panels::players_row(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState, covers: &mut HashMap<TrackId, TextureHandle>, arranged: &Arrangement)` (Task 3 adds a `normal: &Arrangement` parameter).
  - `ViewState::player_rects: Vec<(PlayerId, Rect)>` and `ViewState::cartwall_rect: Option<Rect>` (`pub(crate)`), the last frame's.
  - `pub fn AppUi::player_rects(&self) -> Vec<(PlayerId, egui::Rect)>` and `pub fn AppUi::cartwall_rect(&self) -> Option<egui::Rect>`.
  - A column's egui id salt is `("player", PlayerId.0)`.

- [ ] **Step 1: Write the failing tests**

Create `crates/fp-app/tests/layout_ui.rs`:

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Resizable layout spec RL7, RL8, RL11, RL12 and RL15–RL18: the main
//! window's panels, driven headlessly. The harness's window has an 8-point
//! margin, so the tests compare rectangles with each other.

mod support;

use egui::{Rect, pos2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use fp_app::ui::app::AppUi;
use fp_app::ui::controller::Controller;
use fp_app::ui::panel_layout::{MIN_PLAYER_HEIGHT, MIN_PLAYER_WIDTH, PANEL_GAP};
use fp_model::layout::Panel;
use fp_model::{Command, PlayerId};
use support::{harness, harness_sized, state};

/// The shown players' columns in the last frame, in display order.
fn columns(h: &Harness<'_, AppUi>) -> Vec<(PlayerId, Rect)> {
    h.state().player_rects()
}

fn column(h: &Harness<'_, AppUi>, player: PlayerId) -> Rect {
    columns(h)
        .into_iter()
        .find(|(p, _)| *p == player)
        .map(|(_, r)| r)
        .expect("the player is shown")
}

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 1.0
}

/// Whether the track table in `col` shows `title`: a node with that label
/// below the column's top block (the info row may show a title too).
fn table_shows(h: &Harness<'_, AppUi>, col: Rect, title: &str) -> bool {
    h.get_all_by_label(title).any(|n| {
        let r = n.rect();
        col.contains(r.center()) && r.center().y > col.top() + 300.0
    })
}

#[test]
fn rl11_the_default_layout_draws_todays_rectangles() {
    let (h, fake) = harness(state(2, 3));
    let (p1, p2) = (column(&h, fake.player(0)), column(&h, fake.player(1)));
    let cart = h.state().cartwall_rect().unwrap();
    // Two equal columns 8 points apart, as wide together as the cartwall,
    // which keeps its content height (two rows) under them.
    assert!(near(p1.width(), p2.width()), "{p1:?} {p2:?}");
    assert!(near(p2.left(), p1.right() + PANEL_GAP), "{p1:?} {p2:?}");
    assert!(near(p1.left(), cart.left()) && near(p2.right(), cart.right()));
    assert!(near(cart.height(), 116.0), "{cart:?}");
    // The row ends 8 points above the cartwall; a column is 2 points
    // shorter than its row.
    assert!(near(p1.bottom() + 2.0 + PANEL_GAP, cart.top()), "{p1:?} {cart:?}");
}

#[test]
fn rl11_the_columns_follow_the_shares() {
    let mut s = state(2, 3);
    let p = s.player_ids();
    s.layout.player_shares = vec![(p[0], 3.0), (p[1], 1.0)];
    let (h, _) = harness(s);
    let (a, b) = (column(&h, p[0]), column(&h, p[1]));
    let spare = a.width() + b.width() - 2.0 * MIN_PLAYER_WIDTH;
    assert!(
        near(a.width(), MIN_PLAYER_WIDTH + spare * 0.75),
        "{a:?} {b:?}"
    );
}

#[test]
fn rl11_a_hidden_player_leaves_no_column() {
    let mut s = state(3, 3);
    let p = s.player_ids();
    s.layout.hidden_players = vec![p[1]];
    let (h, _) = harness_sized(s, vec2(1400.0, 700.0), |ui| ui);
    let shown: Vec<PlayerId> = columns(&h).iter().map(|(id, _)| *id).collect();
    assert_eq!(shown, vec![p[0], p[2]]);
}

#[test]
fn rl11_a_hidden_cartwall_gives_its_height_to_the_players() {
    let (h, fake) = harness(state(1, 3));
    let with_cartwall = column(&h, fake.player(0)).height();
    let mut s = state(1, 3);
    s.layout.cartwall.hidden = true;
    let (h, fake) = harness(s);
    assert_eq!(h.state().cartwall_rect(), None);
    let without = column(&h, fake.player(0)).height();
    assert!(near(without, with_cartwall + 116.0 + PANEL_GAP), "{with_cartwall} -> {without}");
}

#[test]
fn rl11_with_every_player_hidden_the_cartwall_fills_the_middle_area() {
    let mut s = state(2, 3);
    s.layout.hidden_players = s.player_ids();
    let (h, _) = harness(s);
    assert!(columns(&h).is_empty());
    let cart = h.state().cartwall_rect().unwrap();
    assert!(cart.height() > 500.0, "{cart:?}");
}

#[test]
fn rl11_too_narrow_a_window_scrolls_the_players_at_their_minimum() {
    let (h, _) = harness(state(4, 3));
    let c = columns(&h);
    assert_eq!(c.len(), 4);
    assert!(c.iter().all(|(_, r)| near(r.width(), MIN_PLAYER_WIDTH)), "{c:?}");
}

#[test]
fn rl11_a_column_keeps_its_id_when_a_player_before_it_is_hidden() {
    let (mut h, fake) = harness_sized(state(3, 40), vec2(1400.0, 700.0), |ui| ui);
    let (p2, p3) = (fake.player(1), fake.player(2));
    let col = column(&h, p3);
    assert!(table_shows(&h, col, "Song 1"));
    h.hover_at(pos2(col.center().x, col.bottom() - 60.0));
    h.step();
    h.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Line,
        delta: vec2(0.0, -10.0),
        phase: egui::TouchPhase::Move,
        modifiers: egui::Modifiers::NONE,
    });
    h.run_steps(20);
    assert!(!table_shows(&h, column(&h, p3), "Song 1"), "P3 scrolled");
    Controller::send(
        fake.as_ref(),
        Command::SetPanelHidden {
            panel: Panel::Player(p2),
            hidden: true,
        },
    );
    h.run_steps(3);
    // P3 is now second among the shown players; keyed by its id, its table
    // keeps its own scroll instead of taking P2's.
    assert!(!table_shows(&h, column(&h, p3), "Song 1"));
}

#[test]
fn rl8_three_rows_fit_a_player_at_its_least_height() {
    let mut s = state(1, 3);
    s.layout.cartwall.hidden = true;
    // The harness's margin, the top and status bars, the middle area's margin.
    let window = 2.0 * 8.0 + 34.0 + 24.0 + 2.0 * 8.0 + MIN_PLAYER_HEIGHT;
    let (h, fake) = harness_sized(s, vec2(1000.0, window), |ui| ui);
    let col = column(&h, fake.player(0));
    assert!(near(col.height(), MIN_PLAYER_HEIGHT - 2.0), "{col:?}");
    let third = h
        .get_all_by_label("Song 3")
        .map(|n| n.rect())
        .find(|r| r.center().y > col.top() + 300.0)
        .expect("the third row is built");
    // The footer is the column's last 24 points.
    assert!(third.bottom() <= col.bottom() - 24.0 + 0.5, "{third:?} in {col:?}");
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-app --test layout_ui`
Expected: FAIL to compile: no method `player_rects` or `cartwall_rect` on `AppUi`.

- [ ] **Step 3: The cartwall's sizes**

`crates/fp-app/src/ui/cartwall.rs`: make the `cart_view` import read

```rust
use super::cart_view::{
    BUTTON_HEIGHT, CartStatus, GAP, MIN_BUTTON_HEIGHT, button_height, cart_look, cart_view,
    page_on_air,
};
```

add `use super::panel_layout::CartwallSizes;` and, after `height`:

```rust
/// RL8: the least height of the cartwall at the bottom: its header, or its
/// header and the shown page's rows at `MIN_BUTTON_HEIGHT`.
pub(crate) fn min_height(scene: &Scene<'_>) -> f32 {
    let wall = &scene.state.cartwall;
    let rows = wall.shown_page().map_or(0, |p| p.rows);
    if wall.open && rows > 0 {
        HEADER_HEIGHT + GAP + f32::from(rows) * MIN_BUTTON_HEIGHT + f32::from(rows - 1) * GAP
    } else {
        HEADER_HEIGHT
    }
}

/// The heights the pixel pass needs (RL11).
pub(crate) fn sizes(scene: &Scene<'_>) -> CartwallSizes {
    CartwallSizes {
        content_height: height(scene),
        min_height: min_height(scene),
    }
}
```

- [ ] **Step 4: Create `crates/fp-app/src/ui/panels.rs`**

```rust
//! The main window's panels (resizable layout spec RL11–RL18): the players'
//! row with its splitters, the top bar's panel tiles and layout menu, the
//! headers' menu, and the view state the layout keeps. The pixel maths is
//! `panel_layout`.

use std::collections::HashMap;

use egui::{Align, Layout, Sense, TextureHandle, Ui, UiBuilder, vec2};
use fp_model::TrackId;

use super::app::{Scene, ViewState};
use super::panel_layout::{Arrangement, PANEL_GAP};
use super::player;

/// The shown players side by side (RL11), each at its width from the pass;
/// the row scrolls horizontally only when the minimums alone do not fit.
pub(crate) fn players_row(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    covers: &mut HashMap<TrackId, TextureHandle>,
    arranged: &Arrangement,
) {
    let widths = arranged.widths.clone();
    if widths.is_empty() {
        return;
    }
    // The horizontal scroll bar gets its own strip instead of covering the footers.
    let bar = if arranged.overflow {
        ui.spacing().scroll.bar_width + ui.spacing().scroll.bar_inner_margin + 2.0
    } else {
        0.0
    };
    let height = ui.available_height() - bar;
    egui::ScrollArea::horizontal()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing = vec2(PANEL_GAP, 0.0);
                for (id, width) in &widths {
                    let (rect, _) =
                        ui.allocate_exact_size(vec2(*width, height - 2.0), Sense::hover());
                    view_state.player_rects.push((*id, rect));
                    if !ui.is_rect_visible(rect) {
                        continue;
                    }
                    let Ok(index) = scene.state.player_index(*id) else {
                        continue;
                    };
                    // RL11: the id comes from the player, not from its place
                    // among the shown ones; its label stays `P<n>` (`index`).
                    let mut column_ui = ui.new_child(
                        UiBuilder::new()
                            .max_rect(rect)
                            .id_salt(("player", id.0))
                            .layout(Layout::top_down(Align::Min)),
                    );
                    column_ui.set_clip_rect(rect.intersect(ui.clip_rect()));
                    player::column(&mut column_ui, scene, view_state, covers, index);
                }
            });
        });
}
```

`crates/fp-app/src/ui.rs`: add `mod panels;` right after `pub mod panel_layout;`.

- [ ] **Step 5: `AppUi` draws from the pass**

`crates/fp-app/src/ui/app.rs`:

1. Delete `const MIN_COLUMN_WIDTH: f32 = 380.0;` and the whole `fn players_row(…) { … }` at the end of the file.
2. Add to the imports: `use super::panel_layout;` and `use super::panels;` (after `use super::notice;`, keeping the list sorted).
3. In `ViewState`, after `new_playlist`:

```rust
    /// The shown players' columns in the last frame (resizable layout spec
    /// RL11), for tests.
    pub(crate) player_rects: Vec<(PlayerId, Rect)>,
    /// The cartwall's rectangle in the last frame; `None` while hidden.
    pub(crate) cartwall_rect: Option<Rect>,
```

4. After `column_widths`:

```rust
    /// The shown players' column rectangles in the last frame, in display
    /// order (resizable layout spec RL11).
    pub fn player_rects(&self) -> Vec<(PlayerId, Rect)> {
        self.view.player_rects.clone()
    }

    /// The cartwall's rectangle in the last frame; `None` while it is
    /// hidden.
    pub fn cartwall_rect(&self) -> Option<Rect> {
        self.view.cartwall_rect
    }
```

5. In `ui`, replace from `// The cartwall strip takes the bottom of the middle area.` down to `cartwall::strip(&mut cart_ui, &scene, &mut self.view);` (inclusive) with:

```rust
        // The middle area (resizable layout spec RL11): the pixel pass turns
        // the layout into the players' row and the cartwall's rectangle.
        let inner = middle.shrink(8.0);
        let ids = state.player_ids();
        let pass = |maximised| {
            panel_layout::arrange(&panel_layout::PassInput {
                players: &ids,
                layout: &state.layout,
                area: inner,
                cartwall: cartwall::sizes(&scene),
                maximised,
            })
        };
        let arranged = pass(None);
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
```

(`pass` takes `Option<fp_model::layout::Panel>`; Task 5 passes the maximised panel. If the compiler cannot infer the closure's parameter type, write `|maximised: Option<fp_model::layout::Panel>|`.)

- [ ] **Step 6: Run the tests**

Run: `cargo test -p fp-app --test layout_ui` and `cargo test -p fp-app`
Expected: PASS. The whole `fp-app` suite must stay green: the default window is unchanged. If `rl8_three_rows_fit_a_player_at_its_least_height` fails on its last assertion, raise `MIN_PLAYER_HEIGHT` by the overshoot the message shows (rounded up to a whole point), rerun, and record the new value as a ruling.

- [ ] **Step 7: Docs**

`docs/technical/ui.md`:

- In the `ui/cartwall.rs`, `ui/cart_view.rs` row, replace "(the 60% cap in `AppUi::ui` is unchanged)" with "(the 60 % cap of a cartwall that fits its content is `panel_layout::CONTENT_HEIGHT_CAP`; `cartwall::sizes` gives the pass its content and least heights)".
- In the Structure table, after the `ui/panel_layout.rs` row, add:

```markdown
| `ui/panels.rs` | The main window's panels (resizable layout spec): `players_row` draws the shown players at the pass's widths, each column keyed by its `PlayerId`; see [Main window layout](#main-window-layout) |
```

- Before `## Data flow`, add:

```markdown
## Main window layout

Resizable layout spec RL8, RL11. `AppUi::ui` splits the window into the
top bar (`TOP_BAR_HEIGHT`), the status bar and the middle area, and the
middle area (inside an 8-point margin) goes through the pure pixel pass
`panel_layout::arrange`. Its `PassInput` is every player id in display
order, the model's `Layout`, the area, the cartwall's `CartwallSizes`
(`cartwall::sizes`: its content height and its least height, the rows at
`MIN_BUTTON_HEIGHT`) and the maximised panel; its `Arrangement` is the
players' row, each shown player's width, whether the row overflows, and
the cartwall's rectangle (`None` while it is hidden).

- **The cartwall** (at the bottom): with `size: None` it fits its content
  as before, at most `CONTENT_HEIGHT_CAP` (60 %) of the area; a share is
  clamped between its least height and what leaves the players
  `MIN_PLAYER_HEIGHT`. With no player shown it takes the whole area. The
  side docks are laid out at the bottom for now.
- **The players** each get `MIN_PLAYER_WIDTH` first; the spare room is
  split by `Layout::shown_fractions`. When the minimums alone do not fit
  (by more than half a point) every column is at its minimum and the row
  scrolls horizontally (`panels::players_row`).
- A column's egui id is `("player", PlayerId)`, so hiding a player never
  hands its table's scroll or its popups to its neighbour; its label is
  still `P<n>` from its place among all players.
- `AppUi::player_rects` and `AppUi::cartwall_rect` give the last frame's
  rectangles (tests).
```

`docs/superpowers/specs/2026-10-21-resizable-layout-design.md`, RL11 item 1: after "…what leaves the players their minimum (RL8)." add the sentence:

```markdown
     At the bottom with `None`, the size is RL13's "fit the content",
     exactly today's, and is not clamped further.
```

- [ ] **Step 8: Gate and commit**

Run the gate. Then:

```bash
git add crates/fp-app/src crates/fp-app/tests/layout_ui.rs docs/technical/ui.md \
  docs/superpowers/specs/2026-10-21-resizable-layout-design.md
git commit -m "feat(ui): draw the players and the cartwall from the layout

RL11 and RL8: the main window lays out the shown players and the bottom
cartwall from the pixel pass. Hidden players leave no column, a hidden
cartwall leaves its height to the players, and with every player hidden
the cartwall fills the middle area. A column is keyed by its player, so a
hidden neighbour never hands it another table's scroll. The default
window draws the same rectangles as before.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: Player splitters (RL12)

**Files:**
- Modify: `crates/fp-app/src/ui/panel_layout.rs` (append)
- Modify: `crates/fp-app/src/ui/panels.rs`
- Modify: `crates/fp-app/src/ui/app.rs` (`ViewState`; the `players_row` call)
- Create: `docs/user/layout.md`
- Modify: `docs/user/SUMMARY.md`, `docs/technical/ui.md` (the "Main window layout" section)
- Test: `crates/fp-app/tests/panel_layout.rs`, `crates/fp-app/tests/layout_ui.rs`

**Interfaces:**
- Consumes: Task 1's `MIN_PLAYER_WIDTH`, `Arrangement`; Task 2's `players_row`; plan 1's `Command::SetPlayerShares { left: (PlayerId, f32), right: (PlayerId, f32) }`, `Command::EqualPlayerWidths`, `Layout::player_shares`, `Layout::locked`.
- Produces:
  - In `panel_layout` (`pub`): `const HOLD_SECS: f64 = 0.5; const SHARE_FLOOR: f32 = 1e-4;` `fn drag_pair(start: (f32, f32), dx: f32) -> (f32, f32)`; `fn pair_shares(widths: (f32, f32)) -> Option<(f32, f32)>`; `struct LiveSplit` (`Debug, Clone, PartialEq`) with `fn new(pair: (PlayerId, PlayerId), shown: Vec<PlayerId>, row_width: f32, start_x: f32, start: (f32, f32)) -> Self`, `fn pair(&self) -> (PlayerId, PlayerId)`, `fn is_dragging(&self) -> bool`, `fn follow(&mut self, x: f32)`, `fn release(&mut self, shares: Vec<(PlayerId, f32)>, until: f64) -> Option<Command>`, `fn stale(&self, shown: &[PlayerId], row_width: f32, shares: &[(PlayerId, f32)], time: f64) -> bool`, `fn apply(&self, widths: &mut [(PlayerId, f32)])`.
  - In `panels` (`pub(crate)`): `fn splitter(ui: &mut Ui, gap: Rect, id: egui::Id, cursor: egui::CursorIcon, active: bool, dragging: bool) -> Option<egui::Response>` — the generic splitter interaction plan 3 reuses for the cartwall splitter (`CursorIcon::ResizeRow` at the bottom, `ResizeColumn` on a side).
  - `players_row` gains `normal: &Arrangement` (the widths without a maximised panel; a drag starts from them). Until Task 5 the caller passes `&arranged` twice.
  - `ViewState::live_split: Option<panel_layout::LiveSplit>` (`pub(crate)`).

- [ ] **Step 1: Write the failing pure tests**

In `crates/fp-app/tests/panel_layout.rs`, make the `use` block read:

```rust
use egui::{Rect, pos2, vec2};
use fp_app::ui::panel_layout::{
    Arrangement, CONTENT_HEIGHT_CAP, CartwallSizes, LiveSplit, MIN_PLAYER_HEIGHT,
    MIN_PLAYER_WIDTH, PANEL_GAP, PassInput, SHARE_FLOOR, arrange, drag_pair, pair_shares,
};
use fp_model::layout::Panel;
use fp_model::{AppState, Command, Config, PlayerId};
```

and append:

```rust
#[test]
fn rl12_a_drag_moves_the_pair_and_keeps_its_total_and_minimums() {
    assert_eq!(drag_pair((480.0, 480.0), 60.0), (540.0, 420.0));
    assert_eq!(drag_pair((480.0, 480.0), 400.0), (580.0, MIN_PLAYER_WIDTH));
    assert_eq!(drag_pair((480.0, 480.0), -400.0), (MIN_PLAYER_WIDTH, 580.0));
    assert_eq!(
        drag_pair((380.0, 380.0), 50.0),
        (380.0, 380.0),
        "no room: nothing moves"
    );
    assert_eq!(drag_pair((480.0, 480.0), f32::NAN), (480.0, 480.0));
}

#[test]
fn rl12_the_shares_are_the_room_above_the_minimums() {
    assert_eq!(pair_shares((540.0, 420.0)), Some((160.0, 40.0)));
    let (left, right) = pair_shares((580.0, 380.0)).unwrap();
    assert_eq!(left, 200.0);
    assert!(
        right > 0.0 && right <= 200.0 * SHARE_FLOOR,
        "a player at its minimum sends a sliver, never 0: {right}"
    );
    assert_eq!(pair_shares((380.0, 380.0)), None);
    assert_eq!(pair_shares((f32::NAN, 400.0)), Some((SHARE_FLOOR * 20.0, 20.0)));
}

#[test]
fn rl12_a_live_split_sends_once_and_holds_until_the_echo() {
    let (a, b, c) = (PlayerId(1), PlayerId(2), PlayerId(3));
    let shown = vec![a, b, c];
    let mut live = LiveSplit::new((a, b), shown.clone(), 1400.0, 500.0, (480.0, 480.0));
    assert!(live.is_dragging());
    live.follow(560.0);
    let mut w = vec![(a, 480.0), (b, 480.0), (c, 424.0)];
    live.apply(&mut w);
    assert_eq!(w, vec![(a, 540.0), (b, 420.0), (c, 424.0)]);
    let before: Vec<(PlayerId, f32)> = Vec::new();
    assert_eq!(
        live.release(before.clone(), 10.5),
        Some(Command::SetPlayerShares {
            left: (a, 160.0),
            right: (b, 40.0),
        })
    );
    assert!(!live.is_dragging());
    assert_eq!(live.release(before.clone(), 10.5), None, "sent once");
    live.follow(900.0);
    let mut w = vec![(a, 480.0), (b, 480.0), (c, 424.0)];
    live.apply(&mut w);
    assert_eq!(w[0], (a, 540.0), "a released drag no longer follows the pointer");
    assert!(!live.stale(&shown, 1400.0, &before, 10.0), "held");
    assert!(live.stale(&shown, 1400.0, &[(a, 160.0)], 10.0), "the echo arrived");
    assert!(live.stale(&shown, 1400.0, &before, 10.5), "the hold ran out");
}

#[test]
fn rl12_a_drag_back_to_where_it_began_sends_nothing() {
    let (a, b) = (PlayerId(1), PlayerId(2));
    let mut live = LiveSplit::new((a, b), vec![a, b], 968.0, 500.0, (480.0, 480.0));
    live.follow(560.0);
    live.follow(500.2);
    assert_eq!(live.release(Vec::new(), 1.0), None);
}

#[test]
fn rl12_a_window_resized_or_a_player_gone_ends_a_drag() {
    let (a, b, c) = (PlayerId(1), PlayerId(2), PlayerId(3));
    let live = LiveSplit::new((a, b), vec![a, b, c], 1400.0, 500.0, (480.0, 480.0));
    assert!(!live.stale(&[a, b, c], 1400.0, &[], 0.0));
    assert!(live.stale(&[a, b, c], 1300.0, &[], 0.0), "the window was resized");
    assert!(live.stale(&[a, b], 1400.0, &[], 0.0), "a player went away");
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-app --test panel_layout`
Expected: FAIL to compile: no `LiveSplit`, `SHARE_FLOOR`, `drag_pair`, `pair_shares` in `panel_layout`.

- [ ] **Step 3: The splitter maths**

`crates/fp-app/src/ui/panel_layout.rs`: make `use fp_model::PlayerId;` read `use fp_model::{Command, PlayerId};` and append:

```rust
/// RL12: how long a released splitter's widths are held while the model's
/// echo is on its way, as the table's column resize holds its widths.
pub const HOLD_SECS: f64 = 0.5;
/// RL12, RL4: the share a player dragged to its minimum width sends, as a
/// part of the pair's spare room; the model refuses 0.
pub const SHARE_FLOOR: f32 = 1e-4;

/// RL12: the pair's widths after the splitter between them moved `dx`
/// from where the drag began. The pair keeps its total and neither goes
/// under `MIN_PLAYER_WIDTH`; a pair without room, or a `dx` that is not a
/// number, does not move.
pub fn drag_pair(start: (f32, f32), dx: f32) -> (f32, f32) {
    let total = start.0 + start.1;
    if !dx.is_finite() || total < 2.0 * MIN_PLAYER_WIDTH {
        return start;
    }
    let left = (start.0 + dx).clamp(MIN_PLAYER_WIDTH, total - MIN_PLAYER_WIDTH);
    (left, total - left)
}

/// RL12, RL4: the shares a drag sends for the pair's new widths: each
/// one's room above its minimum, so that the next pass draws exactly these
/// widths (the model scales the pair to keep its sum). A player at its
/// minimum sends `SHARE_FLOOR` of the pair's room. `None` when the pair has
/// no room above its minimums.
pub fn pair_shares(widths: (f32, f32)) -> Option<(f32, f32)> {
    let room_of = |w: f32| if w.is_finite() { (w - MIN_PLAYER_WIDTH).max(0.0) } else { 0.0 };
    let (left, right) = (room_of(widths.0), room_of(widths.1));
    let room = left + right;
    if !(room.is_finite() && room > 0.0) {
        return None;
    }
    let floor = room * SHARE_FLOOR;
    Some((left.max(floor), right.max(floor)))
}

/// RL12: a player splitter being dragged, or just released: its widths are
/// drawn from the pointer on every frame and sent once, on release.
#[derive(Debug, Clone, PartialEq)]
pub struct LiveSplit {
    left: PlayerId,
    right: PlayerId,
    /// The shown players and the row's width the drag began with; a change
    /// in either ends it.
    shown: Vec<PlayerId>,
    row_width: f32,
    start_x: f32,
    start: (f32, f32),
    /// The pair's widths now.
    now: (f32, f32),
    /// Set on release: the shares the model had then, and until when the
    /// released widths are still drawn.
    released: Option<(Vec<(PlayerId, f32)>, f64)>,
}

impl LiveSplit {
    /// A drag of the splitter between `pair` (left, right), begun at
    /// pointer x `start_x` with the pair `start` wide.
    pub fn new(
        pair: (PlayerId, PlayerId),
        shown: Vec<PlayerId>,
        row_width: f32,
        start_x: f32,
        start: (f32, f32),
    ) -> Self {
        Self {
            left: pair.0,
            right: pair.1,
            shown,
            row_width,
            start_x,
            start,
            now: start,
            released: None,
        }
    }

    pub fn pair(&self) -> (PlayerId, PlayerId) {
        (self.left, self.right)
    }

    /// The button is still down.
    pub fn is_dragging(&self) -> bool {
        self.released.is_none()
    }

    /// The pointer is at `x`: the pair follows it (until the release).
    pub fn follow(&mut self, x: f32) {
        if self.is_dragging() {
            self.now = drag_pair(self.start, x - self.start_x);
        }
    }

    /// The button was released: the command to send, once, and only if the
    /// drag moved something. `shares` are the model's shares now, and the
    /// released widths are held until they change or until `until`.
    pub fn release(&mut self, shares: Vec<(PlayerId, f32)>, until: f64) -> Option<Command> {
        if !self.is_dragging() {
            return None;
        }
        self.released = Some((shares, until));
        if (self.now.0 - self.start.0).abs() <= 0.5 {
            return None;
        }
        let (left, right) = pair_shares(self.now)?;
        Some(Command::SetPlayerShares {
            left: (self.left, left),
            right: (self.right, right),
        })
    }

    /// Whether the drag's widths no longer apply: the shown players or the
    /// row's width changed, or, once released, the model's shares changed
    /// (its echo arrived) or the hold ran out.
    pub fn stale(
        &self,
        shown: &[PlayerId],
        row_width: f32,
        shares: &[(PlayerId, f32)],
        time: f64,
    ) -> bool {
        self.shown != shown
            || (self.row_width - row_width).abs() > 0.5
            || self
                .released
                .as_ref()
                .is_some_and(|(before, until)| time >= *until || before.as_slice() != shares)
    }

    /// `widths` with the pair's replaced by the drag's.
    pub fn apply(&self, widths: &mut [(PlayerId, f32)]) {
        for (id, width) in widths.iter_mut() {
            if *id == self.left {
                *width = self.now.0;
            } else if *id == self.right {
                *width = self.now.1;
            }
        }
    }
}
```

Run: `cargo test -p fp-app --test panel_layout`
Expected: PASS.

- [ ] **Step 4: Write the failing UI tests**

In `crates/fp-app/tests/layout_ui.rs`, make the `use` block read:

```rust
use egui::{CursorIcon, Event, Modifiers, PointerButton, Pos2, Rect, pos2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use fp_app::ui::app::AppUi;
use fp_app::ui::controller::Controller;
use fp_app::ui::panel_layout::{MIN_PLAYER_HEIGHT, MIN_PLAYER_WIDTH, PANEL_GAP};
use fp_model::layout::Panel;
use fp_model::{Command, PlayerId};
use support::{Fake, harness, harness_sized, state};
```

and append:

```rust
// --- splitters (RL12)

fn press(h: &mut Harness<'_, AppUi>, at: Pos2, button: PointerButton, pressed: bool) {
    h.event(Event::PointerButton {
        pos: at,
        button,
        pressed,
        modifiers: Modifiers::NONE,
    });
}

/// The middle of the splitter on the right of `left`'s column.
fn splitter_at(h: &Harness<'_, AppUi>, left: PlayerId) -> Pos2 {
    let r = column(h, left);
    pos2(r.right() + PANEL_GAP / 2.0, r.center().y)
}

/// Presses at `from` and moves `dx` in six frames; returns the first two
/// players' widths after each. The button stays down.
fn drag_by(h: &mut Harness<'_, AppUi>, fake: &Fake, from: Pos2, dx: f32) -> Vec<(f32, f32)> {
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    press(h, from, PointerButton::Primary, true);
    h.run_steps(1);
    let mut seen = Vec::new();
    for step in 1..=6 {
        h.event(Event::PointerMoved(pos2(from.x + dx * step as f32 / 6.0, from.y)));
        h.run_steps(1);
        seen.push((
            column(h, fake.player(0)).width(),
            column(h, fake.player(1)).width(),
        ));
    }
    seen
}

fn click_at(h: &mut Harness<'_, AppUi>, at: Pos2) {
    press(h, at, PointerButton::Primary, true);
    h.run_steps(1);
    press(h, at, PointerButton::Primary, false);
    h.run_steps(1);
}

fn width_commands(fake: &Fake) -> Vec<Command> {
    fake.take_sent()
        .into_iter()
        .filter(|c| {
            matches!(
                c,
                Command::SetPlayerShares { .. } | Command::EqualPlayerWidths
            )
        })
        .collect()
}

fn cursor(h: &Harness<'_, AppUi>) -> CursorIcon {
    h.output().platform_output.cursor_icon
}

#[test]
fn rl12_dragging_a_splitter_resizes_both_neighbours_live() {
    let (mut h, fake) = harness(state(2, 3));
    let before = (
        column(&h, fake.player(0)).width(),
        column(&h, fake.player(1)).width(),
    );
    let from = splitter_at(&h, fake.player(0));
    let seen = drag_by(&mut h, &fake, from, 60.0);
    let last = *seen.last().unwrap();
    assert!(
        near(last.0, before.0 + 60.0) && near(last.1, before.1 - 60.0),
        "{before:?} -> {last:?}"
    );
    assert!(seen.windows(2).all(|w| w[1].0 >= w[0].0), "{seen:?}");
    assert!(
        width_commands(&fake).is_empty(),
        "nothing is sent while the button is down"
    );
}

#[test]
fn rl12_one_command_on_release_and_the_widths_stay() {
    let (mut h, fake) = harness(state(2, 3));
    let (p1, p2) = (fake.player(0), fake.player(1));
    let from = splitter_at(&h, p1);
    drag_by(&mut h, &fake, from, 60.0);
    let live = (column(&h, p1).width(), column(&h, p2).width());
    press(&mut h, pos2(from.x + 60.0, from.y), PointerButton::Primary, false);
    h.run_steps(3);
    let sent = width_commands(&fake);
    assert_eq!(sent.len(), 1, "{sent:?}");
    let Command::SetPlayerShares { left, right } = &sent[0] else {
        panic!("{sent:?}")
    };
    assert_eq!((left.0, right.0), (p1, p2));
    let ratio = (live.0 - MIN_PLAYER_WIDTH) / (live.1 - MIN_PLAYER_WIDTH);
    assert!((left.1 / right.1 - ratio).abs() < 0.01, "{left:?} {right:?}");
    // Past the hold, the model's shares draw the same widths.
    h.run_steps(40);
    assert!(near(column(&h, p1).width(), live.0) && near(column(&h, p2).width(), live.1));
}

#[test]
fn rl12_a_click_on_a_splitter_sends_nothing() {
    let (mut h, fake) = harness(state(2, 3));
    let at = splitter_at(&h, fake.player(0));
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    click_at(&mut h, at);
    h.run_steps(30);
    assert!(width_commands(&fake).is_empty());
}

#[test]
fn rl12_a_double_click_sends_equal_widths() {
    let mut s = state(2, 3);
    let p = s.player_ids();
    s.layout.player_shares = vec![(p[0], 3.0), (p[1], 1.0)];
    let (mut h, fake) = harness(s);
    let at = splitter_at(&h, p[0]);
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    click_at(&mut h, at);
    click_at(&mut h, at);
    h.run_steps(2);
    assert_eq!(width_commands(&fake), vec![Command::EqualPlayerWidths]);
    assert!(near(column(&h, p[0]).width(), column(&h, p[1]).width()));
}

#[test]
fn rl12_the_minimums_hold() {
    let (mut h, fake) = harness(state(2, 3));
    let (p1, p2) = (fake.player(0), fake.player(1));
    let from = splitter_at(&h, p1);
    drag_by(&mut h, &fake, from, 400.0);
    assert!(near(column(&h, p2).width(), MIN_PLAYER_WIDTH));
    press(&mut h, pos2(from.x + 400.0, from.y), PointerButton::Primary, false);
    h.run_steps(40);
    assert_eq!(width_commands(&fake).len(), 1);
    assert!(fake.rejected.lock().unwrap().is_empty(), "the model took it");
    assert!(near(column(&h, p2).width(), MIN_PLAYER_WIDTH));
}

#[test]
fn rl12_the_pointer_shows_the_resize_cursor_over_a_splitter() {
    let (mut h, fake) = harness(state(2, 3));
    h.event(Event::PointerMoved(splitter_at(&h, fake.player(0))));
    h.run_steps(1);
    assert_eq!(cursor(&h), CursorIcon::ResizeColumn);
}

#[test]
fn rl12_a_locked_layout_has_inert_splitters() {
    let mut s = state(2, 3);
    let p = s.player_ids();
    s.layout.player_shares = vec![(p[0], 3.0), (p[1], 1.0)];
    s.layout.locked = true;
    let (mut h, fake) = harness(s);
    let before = column(&h, p[0]).width();
    let at = splitter_at(&h, p[0]);
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    assert_eq!(cursor(&h), CursorIcon::Default);
    click_at(&mut h, at);
    click_at(&mut h, at);
    drag_by(&mut h, &fake, at, 60.0);
    press(&mut h, pos2(at.x + 60.0, at.y), PointerButton::Primary, false);
    h.run_steps(3);
    assert!(width_commands(&fake).is_empty());
    assert!(near(column(&h, p[0]).width(), before));
}
```

- [ ] **Step 5: Run them to see them fail**

Run: `cargo test -p fp-app --test layout_ui rl12`
Expected: FAIL: the drag changes no width, no command is sent, the cursor stays `Default` (no splitter yet).

- [ ] **Step 6: The splitters**

`crates/fp-app/src/ui/app.rs`, in `ViewState` after `cartwall_rect`:

```rust
    /// A player splitter being dragged or just released (RL12).
    pub(crate) live_split: Option<super::panel_layout::LiveSplit>,
```

and in `ui`, make the `players_row` call pass the arrangement twice:

```rust
            panels::players_row(
                &mut players_ui,
                &scene,
                &mut self.view,
                &mut self.covers,
                &arranged,
                &arranged,
            );
```

`crates/fp-app/src/ui/panels.rs`: replace the file's `use` block and `players_row` with:

```rust
use std::collections::HashMap;

use egui::{Align, Layout, Rect, Sense, TextureHandle, Ui, UiBuilder, pos2, vec2};
use fp_model::{Command, PlayerId, TrackId};

use super::app::{Scene, ViewState};
use super::panel_layout::{self, Arrangement, LiveSplit, PANEL_GAP};
use super::player;

/// The shown players side by side (RL11), each at its width from the pass
/// or from a splitter's drag (RL12), with a splitter in each gap; the row
/// scrolls horizontally only when the minimums alone do not fit. `normal`
/// is the pass without a maximised panel: a drag starts from its widths.
pub(crate) fn players_row(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    covers: &mut HashMap<TrackId, TextureHandle>,
    arranged: &Arrangement,
    normal: &Arrangement,
) {
    let widths = live_widths(ui, scene, view_state, arranged);
    if widths.is_empty() {
        return;
    }
    // The horizontal scroll bar gets its own strip instead of covering the footers.
    let bar = if arranged.overflow {
        ui.spacing().scroll.bar_width + ui.spacing().scroll.bar_inner_margin + 2.0
    } else {
        0.0
    };
    let height = ui.available_height() - bar;
    egui::ScrollArea::horizontal()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing = vec2(PANEL_GAP, 0.0);
                for (k, (id, width)) in widths.iter().enumerate() {
                    let (rect, _) =
                        ui.allocate_exact_size(vec2(*width, height - 2.0), Sense::hover());
                    view_state.player_rects.push((*id, rect));
                    if let Some((right, _)) = widths.get(k + 1) {
                        let gap = Rect::from_min_max(
                            pos2(rect.right(), rect.top()),
                            pos2(rect.right() + PANEL_GAP, rect.bottom()),
                        );
                        player_splitter(ui, scene, view_state, (*id, *right), gap, normal);
                    }
                    if !ui.is_rect_visible(rect) {
                        continue;
                    }
                    let Ok(index) = scene.state.player_index(*id) else {
                        continue;
                    };
                    // RL11: the id comes from the player, not from its place
                    // among the shown ones; its label stays `P<n>` (`index`).
                    let mut column_ui = ui.new_child(
                        UiBuilder::new()
                            .max_rect(rect)
                            .id_salt(("player", id.0))
                            .layout(Layout::top_down(Align::Min)),
                    );
                    column_ui.set_clip_rect(rect.intersect(ui.clip_rect()));
                    player::column(&mut column_ui, scene, view_state, covers, index);
                }
            });
        });
}

/// RL12: the widths to draw: the pass's, or a splitter drag's (or one just
/// released, until the model's echo). Called before the row is drawn, so a
/// drag shows in the same frame as the pointer move.
fn live_widths(
    ui: &Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    arranged: &Arrangement,
) -> Vec<(PlayerId, f32)> {
    let mut widths = arranged.widths.clone();
    let Some(live) = view_state.live_split.as_mut() else {
        return widths;
    };
    let shares = &scene.state.layout.player_shares;
    if live.is_dragging() {
        let (down, pointer) = ui.input(|i| (i.pointer.primary_down(), i.pointer.latest_pos()));
        if down {
            if let Some(pos) = pointer {
                live.follow(pos.x);
            }
        } else if let Some(command) =
            live.release(shares.clone(), scene.time + panel_layout::HOLD_SECS)
        {
            scene.ctl.send(command);
        }
    }
    let shown: Vec<PlayerId> = arranged.widths.iter().map(|(p, _)| *p).collect();
    if live.stale(&shown, arranged.players_area.width(), shares, scene.time) {
        view_state.live_split = None;
    } else {
        live.apply(&mut widths);
    }
    widths
}

/// RL12: a splitter's interaction in `gap`. While `active` it senses clicks
/// and drags (a drag starts past egui's threshold, so a click is never a
/// drag), shows `cursor` while hovered or `dragging`, and paints a line
/// along its middle then; at rest nothing is painted. Inactive (a locked
/// layout) it senses nothing and the pointer keeps the normal cursor:
/// `None`.
pub(crate) fn splitter(
    ui: &mut Ui,
    gap: Rect,
    id: egui::Id,
    cursor: egui::CursorIcon,
    active: bool,
    dragging: bool,
) -> Option<egui::Response> {
    if !active {
        return None;
    }
    let response = ui.interact(gap, id, Sense::click_and_drag());
    let hot = (response.hovered() && !ui.input(|i| i.pointer.any_down())) || dragging;
    if hot {
        ui.set_cursor_icon(cursor);
        let visuals = ui.visuals();
        let stroke = if dragging {
            visuals.widgets.active.bg_stroke
        } else {
            visuals.widgets.hovered.bg_stroke
        };
        let line = if gap.width() <= gap.height() {
            [pos2(gap.center().x, gap.top()), pos2(gap.center().x, gap.bottom())]
        } else {
            [pos2(gap.left(), gap.center().y), pos2(gap.right(), gap.center().y)]
        };
        ui.painter().line_segment(line, stroke);
    }
    Some(response)
}

/// RL12: the splitter between two neighbouring shown players. A drag
/// starts a `LiveSplit` from the widths without a maximised panel; a
/// double-click gives every player the same width.
fn player_splitter(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    pair: (PlayerId, PlayerId),
    gap: Rect,
    normal: &Arrangement,
) {
    let dragging = view_state
        .live_split
        .as_ref()
        .is_some_and(|l| l.pair() == pair && l.is_dragging());
    let id = egui::Id::new(("player-splitter", pair.0.0, pair.1.0));
    let active = !scene.state.layout.locked;
    let Some(response) = splitter(ui, gap, id, egui::CursorIcon::ResizeColumn, active, dragging)
    else {
        return;
    };
    if response.double_clicked() {
        scene.ctl.send(Command::EqualPlayerWidths);
    }
    if response.drag_started() {
        let width = |p: PlayerId| {
            normal
                .widths
                .iter()
                .find(|(id, _)| *id == p)
                .map(|(_, w)| *w)
        };
        if let (Some(left), Some(right)) = (width(pair.0), width(pair.1)) {
            let start_x = ui
                .input(|i| i.pointer.press_origin())
                .map_or(gap.center().x, |origin| origin.x);
            let shown = normal.widths.iter().map(|(p, _)| *p).collect();
            view_state.live_split = Some(LiveSplit::new(
                pair,
                shown,
                normal.players_area.width(),
                start_x,
                (left, right),
            ));
        }
    }
}
```

- [ ] **Step 7: Run the tests**

Run: `cargo test -p fp-app --test layout_ui` and `cargo test -p fp-app --test panel_layout`
Expected: PASS. Then `cargo test -p fp-app` (the fader's "shift-wheel does not scroll the players' row" test and the table drag tests must stay green: the splitter is only the gap).

- [ ] **Step 8: Docs**

Create `docs/user/layout.md`:

```markdown
# Layout

Arrange the main window for your shift: give the players the widths you
want, hide the panels you do not use and maximise one for a moment. The
layout is saved with the session and comes back at the next start; a new
installation starts with every player the same width and the cartwall
under them.

## Resize the players

Drag the gap between two players to move width from one to the other. The
other players do not move, and no player gets narrower than its controls
need. Double-click the gap to give every player the same width again.

When the window is too narrow for every shown player at its smallest, the
players' row scrolls sideways, as it always has.
```

`docs/user/SUMMARY.md`: after `- [Cartwall](cartwall.md)` add `- [Layout](layout.md)`.

`docs/technical/ui.md`, at the end of the "Main window layout" section, add:

```markdown
**Splitters** (RL12). The 8-point gap between two shown players is a
splitter (`panels::player_splitter` over the generic `panels::splitter`,
which senses `click_and_drag`, so a click is never a drag, shows the resize
cursor and paints a line only while hovered or dragged, and senses nothing
while the layout is locked). A drag lives in `ViewState::live_split`
(`panel_layout::LiveSplit`): `drag_pair` moves the pair with the pointer
and keeps its total and both minimums, and `panels::live_widths` applies it
before the row is drawn, so the drag shows in the same frame. On release
one `SetPlayerShares` goes out, only if the drag moved something, with
`pair_shares`: each player's room above its minimum (a player at its
minimum sends `SHARE_FLOOR` of the pair's room, as the model refuses 0), so
the next pass draws exactly the dragged widths. The released widths are
held until the model's shares change or `HOLD_SECS` pass, as the table's
column resize does; a change in the shown players or the row's width ends
the drag. A double-click sends `EqualPlayerWidths`.
```

- [ ] **Step 9: Gate and commit**

Run the gate. Then:

```bash
git add crates/fp-app/src crates/fp-app/tests docs/user/layout.md docs/user/SUMMARY.md \
  docs/technical/ui.md
git commit -m "feat(ui): resize the players with splitters

RL12: dragging the gap between two shown players resizes both live and
sends SetPlayerShares once on release, only if it moved, with each
player's room above its minimum so the next pass draws the same widths;
the dragged widths are held until the model's echo. A double-click gives
every player the same width. A locked layout's splitters do not react.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: The top bar's panel tiles and layout menu (RL15, RL7's shortcut)

**Files:**
- Modify: `crates/fp-app/src/ui/panel_layout.rs` (append `panel_on_air`)
- Modify: `crates/fp-app/src/ui/panels.rs` (append)
- Modify: `crates/fp-app/src/ui/app.rs` (`top_bar` L1631-1743; `shortcut_command` L1520)
- Modify: `crates/fp-app/locales/*/main.ftl` (all eleven, appended)
- Modify: `docs/user/layout.md`, `docs/user/cartwall.md` (Keyboard, L58-63), `docs/technical/ui.md`
- Test: `crates/fp-app/tests/panel_layout.rs`, `crates/fp-app/tests/layout_ui.rs`

**Interfaces:**
- Consumes: plan 1's `Layout::{is_hidden, is_last_shown, locked, cartwall}`, `Dock::ALL`, `Command::{SetPanelHidden, SetCartwallDock, SetLayoutLocked, EqualPlayerWidths, ResetLayout}`; the existing `error-last-panel`, `error-layout-locked`, `player-label` strings; `widgets::{tile, tile_without_tooltip, TileStyle, font}`.
- Produces:
  - `pub fn panel_layout::panel_on_air(state: &AppState, panel: Panel) -> bool` (plan 3's collapsed side strip uses it for its dot).
  - `pub(crate) fn panels::layout_controls(ui: &mut Ui, scene: &Scene<'_>)`.
  - Strings `layout-panel-carts`, `tip-layout-hide`, `tip-layout-show`, `tip-layout-show-on-air` (`$panel`), `tip-layout-menu`, `layout-dock-bottom`, `layout-dock-left`, `layout-dock-right`, `layout-lock`, `layout-equal-widths`, `layout-reset`.

- [ ] **Step 1: Write the failing tests**

In `crates/fp-app/tests/panel_layout.rs`, make the `use` block read:

```rust
use std::path::PathBuf;

use egui::{Rect, pos2, vec2};
use fp_app::ui::panel_layout::{
    Arrangement, CONTENT_HEIGHT_CAP, CartwallSizes, LiveSplit, MIN_PLAYER_HEIGHT,
    MIN_PLAYER_WIDTH, PANEL_GAP, PassInput, SHARE_FLOOR, arrange, drag_pair, pair_shares,
    panel_on_air,
};
use fp_model::layout::Panel;
use fp_model::{AppState, Command, Config, PlayerId, TrackAnalysis, apply};
```

and append:

```rust
#[test]
fn rl15_a_panel_is_on_air_while_it_sounds() {
    let mut s = four();
    let playlist = s.playlists.first_id().unwrap();
    apply(
        &mut s,
        Command::InsertPaths {
            playlist,
            index: 0,
            paths: vec![PathBuf::from("/music/a.flac"), PathBuf::from("/music/b.flac")],
        },
    )
    .unwrap();
    let p = s.player_ids();
    assert!(!panel_on_air(&s, Panel::Player(p[0])));
    apply(&mut s, Command::Play(p[0])).unwrap();
    assert!(panel_on_air(&s, Panel::Player(p[0])));
    apply(&mut s, Command::Pause(p[0])).unwrap();
    assert!(panel_on_air(&s, Panel::Player(p[0])), "paused is still on air");
    apply(&mut s, Command::Stop(p[0])).unwrap();
    assert!(!panel_on_air(&s, Panel::Player(p[0])));
    assert!(!panel_on_air(&s, Panel::Player(PlayerId(999_999))));

    assert!(!panel_on_air(&s, Panel::Cartwall));
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
    let cart = &s.cartwall.pages[0].carts[0];
    let (id, track) = (cart.id, cart.track.unwrap());
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
    apply(&mut s, Command::FireCart(id)).unwrap();
    assert!(panel_on_air(&s, Panel::Cartwall));
}
```

In `crates/fp-app/tests/layout_ui.rs`, make the `use` block read:

```rust
use egui::accesskit::Role;
use egui::{CursorIcon, Event, Key, Modifiers, PointerButton, Pos2, Rect, pos2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use fp_app::ui::app::AppUi;
use fp_app::ui::controller::Controller;
use fp_app::ui::panel_layout::{MIN_PLAYER_HEIGHT, MIN_PLAYER_WIDTH, PANEL_GAP};
use fp_app::ui::theme;
use fp_model::layout::{Dock, Panel};
use fp_model::{Command, KeyChord, PlayerId, Shortcut, ShortcutAction};
use support::{Fake, harness, harness_sized, state};
```

and append:

```rust
// --- the top bar (RL15)

fn hidden_commands(fake: &Fake) -> Vec<Command> {
    fake.take_sent()
        .into_iter()
        .filter(|c| matches!(c, Command::SetPanelHidden { .. }))
        .collect()
}

fn disabled(h: &Harness<'_, AppUi>, label: &str) -> bool {
    h.get_by_label(label).accesskit_node().is_disabled()
}

fn open_layout_menu(h: &mut Harness<'_, AppUi>) {
    h.get_by_label("Layout").click();
    h.run_steps(2);
}

/// Whether a red square is painted inside `area`.
fn red_dot_in(h: &Harness<'_, AppUi>, area: Rect) -> bool {
    fn walk(shape: &egui::Shape, area: Rect, found: &mut bool) {
        match shape {
            egui::Shape::Rect(r) => {
                *found |= r.fill == theme::ON_AIR_TEXT && area.contains_rect(r.rect);
            }
            egui::Shape::Vec(v) => v.iter().for_each(|s| walk(s, area, found)),
            _ => {}
        }
    }
    let mut found = false;
    for clipped in &h.output().shapes {
        walk(&clipped.shape, area, &mut found);
    }
    found
}

#[test]
fn rl15_a_tile_hides_and_shows_its_player() {
    let (mut h, fake) = harness(state(2, 3));
    let p2 = fake.player(1);
    h.get_by_label("Hide P2").click();
    h.run_steps(2);
    assert_eq!(
        hidden_commands(&fake),
        vec![Command::SetPanelHidden {
            panel: Panel::Player(p2),
            hidden: true,
        }]
    );
    assert_eq!(columns(&h).len(), 1);
    h.get_by_label("Show P2").click();
    h.run_steps(2);
    assert_eq!(columns(&h).len(), 2);
}

#[test]
fn rl15_the_carts_tile_hides_and_shows_the_cartwall() {
    let (mut h, _) = harness(state(2, 3));
    h.get_by_label("Hide Carts").click();
    h.run_steps(2);
    assert_eq!(h.state().cartwall_rect(), None);
    h.get_by_label("Show Carts").click();
    h.run_steps(2);
    assert!(h.state().cartwall_rect().is_some());
}

#[test]
fn rl15_the_last_lit_tile_is_disabled() {
    let mut s = state(1, 3);
    s.layout.cartwall.hidden = true;
    let (mut h, fake) = harness(s);
    assert!(disabled(&h, "Hide P1"));
    assert!(!disabled(&h, "Show Carts"));
    h.get_by_label("Hide P1").click();
    h.run_steps(2);
    assert!(hidden_commands(&fake).is_empty());
}

#[test]
fn rl15_a_hidden_panel_on_air_shows_a_red_dot() {
    let (mut h, fake) = harness(state(2, 3));
    let p1 = fake.player(0);
    Controller::send(fake.as_ref(), Command::Play(p1));
    h.get_by_label("Hide P1").click();
    h.run_steps(2);
    let tile = h.get_by_label("Show P1 (on air)").rect();
    assert!(red_dot_in(&h, tile));
    Controller::send(fake.as_ref(), Command::Stop(p1));
    h.run_steps(2);
    let tile = h.get_by_label("Show P1").rect();
    assert!(!red_dot_in(&h, tile));
}

#[test]
fn rl15_the_layout_menu_sends_its_commands() {
    let (mut h, fake) = harness(state(2, 3));
    open_layout_menu(&mut h);
    h.get_by_role_and_label(Role::RadioButton, "Cartwall on the left")
        .click();
    h.run_steps(2);
    open_layout_menu(&mut h);
    h.get_by_label("Equal player widths").click();
    h.run_steps(2);
    open_layout_menu(&mut h);
    h.get_by_label("Reset layout").click();
    h.run_steps(2);
    open_layout_menu(&mut h);
    h.get_by_role_and_label(Role::CheckBox, "Lock layout").click();
    h.run_steps(2);
    let sent: Vec<Command> = fake
        .take_sent()
        .into_iter()
        .filter(|c| {
            matches!(
                c,
                Command::SetCartwallDock(_)
                    | Command::EqualPlayerWidths
                    | Command::ResetLayout
                    | Command::SetLayoutLocked(_)
            )
        })
        .collect();
    assert_eq!(
        sent,
        vec![
            Command::SetCartwallDock(Dock::Left),
            Command::EqualPlayerWidths,
            Command::ResetLayout,
            Command::SetLayoutLocked(true),
        ]
    );
}

#[test]
fn rl15_the_menu_checks_the_current_dock() {
    let mut s = state(2, 3);
    s.layout.cartwall.dock = Dock::Right;
    s.layout.cartwall.size = Some(0.3);
    let (mut h, _) = harness(s);
    open_layout_menu(&mut h);
    for (label, on) in [
        ("Cartwall at the bottom", false),
        ("Cartwall on the left", false),
        ("Cartwall on the right", true),
    ] {
        let want = if on {
            egui::accesskit::Toggled::True
        } else {
            egui::accesskit::Toggled::False
        };
        assert_eq!(
            h.get_by_role_and_label(Role::RadioButton, label)
                .accesskit_node()
                .toggled(),
            Some(want),
            "{label}"
        );
    }
}

#[test]
fn rl15_a_locked_layout_disables_the_tiles_and_the_menu() {
    let mut s = state(2, 3);
    s.layout.locked = true;
    let (mut h, fake) = harness(s);
    for tile in ["Hide P1", "Hide P2", "Hide Carts"] {
        assert!(disabled(&h, tile), "{tile}");
    }
    open_layout_menu(&mut h);
    for label in [
        "Cartwall at the bottom",
        "Cartwall on the left",
        "Cartwall on the right",
    ] {
        assert!(
            h.get_by_role_and_label(Role::RadioButton, label)
                .accesskit_node()
                .is_disabled(),
            "{label}"
        );
    }
    assert!(disabled(&h, "Equal player widths"));
    assert!(disabled(&h, "Reset layout"));
    let lock = h.get_by_role_and_label(Role::CheckBox, "Lock layout");
    assert!(!lock.accesskit_node().is_disabled());
    assert_eq!(
        lock.accesskit_node().toggled(),
        Some(egui::accesskit::Toggled::True)
    );
    lock.click();
    h.run_steps(2);
    assert!(fake.take_sent().contains(&Command::SetLayoutLocked(false)));
}

#[test]
fn rl7_the_cartwall_shortcut_does_nothing_while_it_is_hidden() {
    let shortcut = Shortcut {
        action: ShortcutAction::ToggleCartwall,
        chord: KeyChord {
            key: "C".into(),
            ctrl: false,
            alt: false,
            shift: false,
            command: false,
        },
    };
    let toggles = |fake: &Fake| {
        fake.take_sent()
            .iter()
            .filter(|c| matches!(c, Command::SetCartwallOpen(_)))
            .count()
    };
    let mut s = state(2, 3);
    s.config.shortcuts.push(shortcut.clone());
    let (mut h, fake) = harness(s);
    h.key_press(Key::C);
    h.run_steps(2);
    assert_eq!(toggles(&fake), 1, "shown, the shortcut collapses it");
    let mut s = state(2, 3);
    s.config.shortcuts.push(shortcut);
    s.layout.cartwall.hidden = true;
    let (mut h, fake) = harness(s);
    h.key_press(Key::C);
    h.run_steps(2);
    assert_eq!(toggles(&fake), 0);
}

#[test]
fn rl7_a_hidden_player_still_answers_its_key() {
    let mut s = state(2, 3);
    let p = s.player_ids();
    s.layout.hidden_players = vec![p[1]];
    let (mut h, fake) = harness(s);
    h.key_press(Key::Num2);
    h.run_steps(2);
    assert!(fake.take_sent().contains(&Command::Play(p[1])));
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-app --test panel_layout rl15` and `cargo test -p fp-app --test layout_ui rl15 rl7`
Expected: `panel_layout` fails to compile (no `panel_on_air`); `layout_ui` tests fail: no node labelled "Hide P2" or "Layout"; the shortcut test fails on its second count. `rl7_a_hidden_player_still_answers_its_key` may already pass (it guards against a regression).

- [ ] **Step 3: `panel_on_air`**

`crates/fp-app/src/ui/panel_layout.rs`: make the `fp_model` import read `use fp_model::{AppState, Command, PlayerId, Transport};` and append:

```rust
/// RL15: whether `panel` is on air, for the red dot on a hidden panel's
/// tile: a player whose current entry is playing or paused (as the close
/// guard counts it), or the cartwall while any cart plays.
pub fn panel_on_air(state: &AppState, panel: Panel) -> bool {
    match panel {
        Panel::Player(id) => state
            .player(id)
            .is_ok_and(|p| p.transport != Transport::Stopped),
        Panel::Cartwall => !state.cartwall.playing.is_empty(),
    }
}
```

- [ ] **Step 4: The tiles and the menu**

`crates/fp-app/src/ui/panels.rs`: make the imports read

```rust
use std::collections::HashMap;

use egui::{Align, Layout, Rect, Sense, TextureHandle, Ui, UiBuilder, pos2, vec2};
use fp_model::layout::{Dock, Panel};
use fp_model::{Command, PlayerId, TrackId};

use super::app::{Scene, ViewState};
use super::panel_layout::{self, Arrangement, LiveSplit, PANEL_GAP};
use super::player;
use super::theme;
use super::widgets::{self, TileStyle, font};
```

and append:

```rust
/// RL15: on the left of the top bar, one tile per player (`P1` … `Pn`) and
/// one for the cartwall (`Carts`), lit while the panel is shown, then the
/// layout menu.
pub(crate) fn layout_controls(ui: &mut Ui, scene: &Scene<'_>) {
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing = vec2(4.0, 0.0);
        let t = scene.i18n;
        for (n, id) in scene.state.player_ids().into_iter().enumerate() {
            let name = t.tr_args("player-label", &[("n", (n + 1).into())]);
            panel_tile(ui, scene, Panel::Player(id), &name);
        }
        panel_tile(ui, scene, Panel::Cartwall, &t.tr("layout-panel-carts"));
        ui.add_space(4.0);
        layout_menu(ui, scene);
    });
}

/// One panel tile: a click hides or shows the panel (RL7). Its label says
/// what a click does; a hidden panel on air shows a red dot. Disabled for
/// the last shown panel and while the layout is locked.
fn panel_tile(ui: &mut Ui, scene: &Scene<'_>, panel: Panel, name: &str) {
    let t = scene.i18n;
    let layout = &scene.state.layout;
    let players = scene.state.player_ids();
    let shown = !layout.is_hidden(panel);
    let on_air = !shown && panel_layout::panel_on_air(scene.state, panel);
    let last = layout.is_last_shown(&players, panel);
    let enabled = !layout.locked && !last;
    let key = match (shown, on_air) {
        (true, _) => "tip-layout-hide",
        (false, true) => "tip-layout-show-on-air",
        (false, false) => "tip-layout-show",
    };
    let label = t.tr_args(key, &[("panel", name.to_owned().into())]);
    let style = if shown {
        TileStyle {
            fill: theme::NEUTRAL_800,
            border: theme::NEUTRAL_700,
            content: theme::TEXT,
            hover_fill: theme::NEUTRAL_700,
            ..TileStyle::plain()
        }
    } else {
        TileStyle {
            border: theme::NEUTRAL_800,
            content: theme::NEUTRAL_500,
            hover_fill: theme::NEUTRAL_800,
            ..TileStyle::plain()
        }
    };
    let width = ui
        .painter()
        .layout_no_wrap(name.to_owned(), font(11.0), theme::TEXT)
        .size()
        .x
        + 18.0;
    let response = widgets::tile_without_tooltip(
        ui,
        vec2(width.max(30.0), 24.0),
        &label,
        enabled,
        style,
        |p, r, c| {
            p.text(r.center(), egui::Align2::CENTER_CENTER, name, font(11.0), c);
            if on_air {
                p.rect_filled(
                    Rect::from_min_size(pos2(r.right() - 8.0, r.top() + 2.0), vec2(6.0, 6.0)),
                    0.0,
                    theme::ON_AIR_TEXT,
                );
            }
        },
    );
    let tip = if layout.locked {
        t.tr("error-layout-locked")
    } else if last {
        t.tr("error-last-panel")
    } else {
        label
    };
    if response.on_hover_text(tip).clicked() {
        scene.ctl.send(Command::SetPanelHidden {
            panel,
            hidden: shown,
        });
    }
}

fn dock_key(dock: Dock) -> &'static str {
    match dock {
        Dock::Bottom => "layout-dock-bottom",
        Dock::Left => "layout-dock-left",
        Dock::Right => "layout-dock-right",
    }
}

/// RL15: the layout button and its menu: the cartwall's dock, Lock layout,
/// Equal player widths and Reset layout. While the layout is locked every
/// entry but Lock layout is disabled.
fn layout_menu(ui: &mut Ui, scene: &Scene<'_>) {
    let t = scene.i18n;
    let label = t.tr("tip-layout-menu");
    let style = TileStyle {
        border: theme::NEUTRAL_800,
        hover_fill: theme::NEUTRAL_800,
        ..TileStyle::plain()
    };
    let response = widgets::tile(ui, vec2(24.0, 24.0), &label, true, style, |p, r, c| {
        p.text(
            r.center(),
            egui::Align2::CENTER_CENTER,
            egui_phosphor::regular::LAYOUT,
            font(14.0),
            c,
        );
    });
    egui::Popup::menu(&response).show(|ui| {
        ui.set_min_width(220.0);
        let layout = &scene.state.layout;
        let locked = layout.locked;
        for dock in Dock::ALL {
            let checked = layout.cartwall.dock == dock;
            let radio = egui::RadioButton::new(checked, t.tr(dock_key(dock)));
            if ui.add_enabled(!locked, radio).clicked() && !checked {
                scene.ctl.send(Command::SetCartwallDock(dock));
                ui.close();
            }
        }
        ui.separator();
        let mut lock = locked;
        if ui.checkbox(&mut lock, t.tr("layout-lock")).changed() {
            scene.ctl.send(Command::SetLayoutLocked(lock));
        }
        ui.separator();
        let equal = egui::Button::new(t.tr("layout-equal-widths"));
        if ui.add_enabled(!locked, equal).clicked() {
            scene.ctl.send(Command::EqualPlayerWidths);
            ui.close();
        }
        let reset = egui::Button::new(t.tr("layout-reset"));
        if ui.add_enabled(!locked, reset).clicked() {
            scene.ctl.send(Command::ResetLayout);
            ui.close();
        }
    });
}
```

`crates/fp-app/src/ui/app.rs`:

- In `top_bar`, right after the first `ui.add_space(10.0);` (before `ui.with_layout(Layout::right_to_left(Align::Center), …)`), add:

```rust
    // RL15: the panel tiles and the layout menu, on the left.
    panels::layout_controls(ui, scene);
```

- In `shortcut_command`, replace `ShortcutAction::ToggleCartwall => Command::SetCartwallOpen(!wall.open),` with:

```rust
        // RL7: while the cartwall is hidden, its collapse toggle does
        // nothing; the top bar's Carts tile shows it.
        ShortcutAction::ToggleCartwall if state.layout.cartwall.hidden => return None,
        ShortcutAction::ToggleCartwall => Command::SetCartwallOpen(!wall.open),
```

- [ ] **Step 5: The strings**

Append to each locale file (a blank line, then the block):

`en-US`:
```
## Resizable layout: the top bar's panel tiles and layout menu
layout-panel-carts = Carts
tip-layout-hide = Hide { $panel }
tip-layout-show = Show { $panel }
tip-layout-show-on-air = Show { $panel } (on air)
tip-layout-menu = Layout
layout-dock-bottom = Cartwall at the bottom
layout-dock-left = Cartwall on the left
layout-dock-right = Cartwall on the right
layout-lock = Lock layout
layout-equal-widths = Equal player widths
layout-reset = Reset layout
```
`es-ES`:
```
## Resizable layout: the top bar's panel tiles and layout menu
layout-panel-carts = Cartuchos
tip-layout-hide = Ocultar { $panel }
tip-layout-show = Mostrar { $panel }
tip-layout-show-on-air = Mostrar { $panel } (sonando)
tip-layout-menu = Distribución
layout-dock-bottom = Cartuchera abajo
layout-dock-left = Cartuchera a la izquierda
layout-dock-right = Cartuchera a la derecha
layout-lock = Bloquear la distribución
layout-equal-widths = Igualar el ancho de los players
layout-reset = Restablecer la distribución
```
`ca-ES`:
```
## Resizable layout: the top bar's panel tiles and layout menu
layout-panel-carts = Cartutxos
tip-layout-hide = Amagar { $panel }
tip-layout-show = Mostrar { $panel }
tip-layout-show-on-air = Mostrar { $panel } (en antena)
tip-layout-menu = Disposició
layout-dock-bottom = Cartutxera a baix
layout-dock-left = Cartutxera a l'esquerra
layout-dock-right = Cartutxera a la dreta
layout-lock = Bloquejar la disposició
layout-equal-widths = Igualar l'amplada dels reproductors
layout-reset = Restablir la disposició
```
`de-DE`:
```
## Resizable layout: the top bar's panel tiles and layout menu
layout-panel-carts = Carts
tip-layout-hide = { $panel } ausblenden
tip-layout-show = { $panel } einblenden
tip-layout-show-on-air = { $panel } einblenden (On Air)
tip-layout-menu = Layout
layout-dock-bottom = Cartwall unten
layout-dock-left = Cartwall links
layout-dock-right = Cartwall rechts
layout-lock = Layout sperren
layout-equal-widths = Player gleich breit
layout-reset = Layout zurücksetzen
```
`eu-ES`:
```
## Resizable layout: the top bar's panel tiles and layout menu
layout-panel-carts = Kartutxoak
tip-layout-hide = Ezkutatu { $panel }
tip-layout-show = Erakutsi { $panel }
tip-layout-show-on-air = Erakutsi { $panel } (airean)
tip-layout-menu = Diseinua
layout-dock-bottom = Kartutxo-panela behean
layout-dock-left = Kartutxo-panela ezkerrean
layout-dock-right = Kartutxo-panela eskuinean
layout-lock = Blokeatu diseinua
layout-equal-widths = Berdindu erreproduzitzaileen zabalera
layout-reset = Berrezarri diseinua
```
`fr-FR`:
```
## Resizable layout: the top bar's panel tiles and layout menu
layout-panel-carts = Cartouches
tip-layout-hide = Masquer { $panel }
tip-layout-show = Afficher { $panel }
tip-layout-show-on-air = Afficher { $panel } (à l'antenne)
tip-layout-menu = Disposition
layout-dock-bottom = Cartouches en bas
layout-dock-left = Cartouches à gauche
layout-dock-right = Cartouches à droite
layout-lock = Verrouiller la disposition
layout-equal-widths = Lecteurs de même largeur
layout-reset = Réinitialiser la disposition
```
`gl-ES`:
```
## Resizable layout: the top bar's panel tiles and layout menu
layout-panel-carts = Cartuchos
tip-layout-hide = Ocultar { $panel }
tip-layout-show = Mostrar { $panel }
tip-layout-show-on-air = Mostrar { $panel } (en antena)
tip-layout-menu = Disposición
layout-dock-bottom = Cartucheira abaixo
layout-dock-left = Cartucheira á esquerda
layout-dock-right = Cartucheira á dereita
layout-lock = Bloquear a disposición
layout-equal-widths = Igualar o ancho dos reprodutores
layout-reset = Restablecer a disposición
```
`it-IT`:
```
## Resizable layout: the top bar's panel tiles and layout menu
layout-panel-carts = Cart
tip-layout-hide = Nascondi { $panel }
tip-layout-show = Mostra { $panel }
tip-layout-show-on-air = Mostra { $panel } (in onda)
tip-layout-menu = Layout
layout-dock-bottom = Cartwall in basso
layout-dock-left = Cartwall a sinistra
layout-dock-right = Cartwall a destra
layout-lock = Blocca il layout
layout-equal-widths = Lettori di uguale larghezza
layout-reset = Reimposta il layout
```
`nl-NL`:
```
## Resizable layout: the top bar's panel tiles and layout menu
layout-panel-carts = Carts
tip-layout-hide = { $panel } verbergen
tip-layout-show = { $panel } tonen
tip-layout-show-on-air = { $panel } tonen (on air)
tip-layout-menu = Indeling
layout-dock-bottom = Cartwall onderaan
layout-dock-left = Cartwall links
layout-dock-right = Cartwall rechts
layout-lock = Indeling vergrendelen
layout-equal-widths = Spelers even breed
layout-reset = Indeling terugzetten
```
`pl-PL`:
```
## Resizable layout: the top bar's panel tiles and layout menu
layout-panel-carts = Carty
tip-layout-hide = Ukryj { $panel }
tip-layout-show = Pokaż { $panel }
tip-layout-show-on-air = Pokaż { $panel } (na antenie)
tip-layout-menu = Układ
layout-dock-bottom = Cartwall na dole
layout-dock-left = Cartwall po lewej
layout-dock-right = Cartwall po prawej
layout-lock = Zablokuj układ
layout-equal-widths = Równa szerokość odtwarzaczy
layout-reset = Resetuj układ
```
`pt-PT`:
```
## Resizable layout: the top bar's panel tiles and layout menu
layout-panel-carts = Cartuchos
tip-layout-hide = Ocultar { $panel }
tip-layout-show = Mostrar { $panel }
tip-layout-show-on-air = Mostrar { $panel } (no ar)
tip-layout-menu = Disposição
layout-dock-bottom = Cartucheira em baixo
layout-dock-left = Cartucheira à esquerda
layout-dock-right = Cartucheira à direita
layout-lock = Bloquear a disposição
layout-equal-widths = Leitores com a mesma largura
layout-reset = Repor a disposição
```

- [ ] **Step 6: Run the tests**

Run: `cargo test -p fp-app --test panel_layout`, `cargo test -p fp-app --test layout_ui`, `cargo test -p fp-app --test i18n`, then `cargo test -p fp-app`.
Expected: PASS. (The exit guard's `query_by_label("P1")` still finds one node: the tiles are labelled "Hide P1".)

- [ ] **Step 7: Docs**

`docs/user/layout.md`: insert before `## Resize the players`:

```markdown
## Show and hide panels

The tiles on the left of the top bar, **P1** … **Pn** and **Carts**, show
which panels are on screen: a lit tile is shown. Click a tile to hide or
show its panel. The other panels take the room it leaves, and a player
shown again comes back at its width.

Hiding only takes a panel off the screen. A hidden player keeps playing,
and its number key, MIDI, the remote API, its CUE window and automation
still control it. While a hidden panel is on air its tile shows a red dot:
a player that is playing or paused, or the cartwall while a cart plays.

At least one panel stays shown: the last lit tile cannot be clicked.
While the cartwall is hidden, the shortcut that collapses or expands it
does nothing; click **Carts** to show it.
```

and append at the end of the file:

```markdown
## The layout menu

The layout button, right of the tiles, opens a menu with:

- **Cartwall at the bottom**, **Cartwall on the left**, **Cartwall on the
  right**: where the cartwall sits;
- **Lock layout**: while it is checked, nothing changes the layout by
  accident: the tiles, the other entries of this menu, the gaps between
  the players and the headers' **Maximise** and **Hide** do nothing.
  Playback, and collapsing or expanding the cartwall, work as usual. The
  lock is saved with the layout;
- **Equal player widths**: every player the same width;
- **Reset layout**: back to a new installation's layout.
```

`docs/user/cartwall.md`, Keyboard section: after "See [Keyboard](keyboard.md) to change them." add a paragraph: "The shortcut that collapses or expands the cartwall does nothing while the cartwall is hidden from the top bar (see [Layout](layout.md#show-and-hide-panels))."

`docs/technical/ui.md`, at the end of the "Main window layout" section, add:

```markdown
**Top bar** (RL15). `panels::layout_controls`, on the left of `top_bar`,
draws one tile per player (`player-label`, its place among all players) and
one for the cartwall (`layout-panel-carts`), lit while shown; a click sends
`SetPanelHidden`. A tile's accessible label and tooltip say what a click
does (`tip-layout-hide`, `tip-layout-show`, `tip-layout-show-on-air`); the
last shown panel's tile and every tile of a locked layout are disabled,
with `error-last-panel` or `error-layout-locked` as the tooltip. A hidden
panel that `panel_layout::panel_on_air` (a player not stopped, or any cart
playing) gets a red square in its corner. The layout button
(`tip-layout-menu`) opens an `egui::Popup::menu` with the dock radio items
(`SetCartwallDock`), Lock layout (`SetLayoutLocked`), Equal player widths
and Reset layout; while locked every entry but the lock is disabled. While
the cartwall is hidden, the `ToggleCartwall` shortcut maps to no command
(`shortcut_command`).
```

- [ ] **Step 8: Gate and commit**

Run the gate. Then:

```bash
git add crates/fp-app docs/user/layout.md docs/user/cartwall.md docs/technical/ui.md
git commit -m "feat(ui): panel tiles and the layout menu in the top bar

RL15: a tile per player and one for the cartwall hide and show their
panel; a hidden panel on air shows a red dot, and the last shown panel's
tile is disabled. The layout menu docks the cartwall, locks the layout,
equals the players' widths and resets the layout; while locked only the
lock reacts. The cartwall's collapse shortcut does nothing while it is
hidden (RL7).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: Maximise and the headers' menu (RL16)

**Files:**
- Modify: `crates/fp-app/src/ui/panel_layout.rs` (append)
- Modify: `crates/fp-app/src/ui/panels.rs` (append; `player_splitter`)
- Modify: `crates/fp-app/src/ui/app.rs` (`ViewState`; accessor; `ui`: `settle_view` before `self.keyboard`, the maximised pass)
- Modify: `crates/fp-app/src/ui/player.rs` (`column` L78; `header` L209-374; imports)
- Modify: `crates/fp-app/src/ui/cartwall.rs` (`strip` L41-53; `header` L55-111; imports)
- Modify: `crates/fp-app/locales/*/main.ftl` (all eleven, appended)
- Modify: `docs/user/layout.md`, `docs/user/players.md` (after the Header table, L20), `docs/technical/ui.md`
- Test: `crates/fp-app/tests/panel_layout.rs`, `crates/fp-app/tests/layout_ui.rs`

**Interfaces:**
- Consumes: Task 1's `arrange` with `PassInput::maximised`; Task 3's `players_row(…, arranged, normal)`, `player_splitter`; plan 1's `Layout::shown_panels`, `Layout::is_last_shown`.
- Produces:
  - In `panel_layout` (`pub`): `struct ArrangementKey` (`Debug, Clone, PartialEq`) with `fn of(state: &AppState) -> Self`; `struct Maximised { pub panel: Panel, /* key */ }` (`Debug, Clone, PartialEq`) with `fn new(panel: Panel, state: &AppState) -> Self` and `fn holds(&self, state: &AppState) -> bool`.
  - In `panels` (`pub(crate)`): `fn header_menu(response: &egui::Response, scene: &Scene<'_>, view_state: &mut ViewState, panel: Panel)` (plan 3 attaches it to the collapsed side strip too); `fn settle_view(view_state: &mut ViewState, state: &AppState)`.
  - `ViewState::maximised: Option<panel_layout::Maximised>` (`pub(crate)`); `pub fn AppUi::maximised(&self) -> Option<Panel>`.
  - Strings `layout-maximise`, `layout-restore`, `layout-hide`.

- [ ] **Step 1: Write the failing tests**

In `crates/fp-app/tests/panel_layout.rs`, add `Maximised` to the `fp_app::ui::panel_layout::{…}` import list, make the `fp_model::layout` import read `use fp_model::layout::{Dock, Panel};`, and append:

```rust
#[test]
fn rl16_maximise_ends_when_the_arrangement_changes() {
    let s = four();
    let p = s.player_ids();
    let m = Maximised::new(Panel::Player(p[0]), &s);
    assert!(m.holds(&s));
    let mut resized = s.clone();
    apply(&mut resized, Command::EqualPlayerWidths).unwrap();
    assert!(m.holds(&resized), "a resize keeps it");
    for command in [
        Command::SetPanelHidden {
            panel: Panel::Player(p[1]),
            hidden: true,
        },
        Command::SetPanelHidden {
            panel: Panel::Cartwall,
            hidden: true,
        },
        Command::SetCartwallDock(Dock::Left),
        Command::SetLayoutLocked(true),
        Command::SetPlayerCount(3),
    ] {
        let mut t = s.clone();
        apply(&mut t, command.clone()).unwrap();
        assert!(!m.holds(&t), "{command:?}");
    }
}
```

In `crates/fp-app/tests/layout_ui.rs` append:

```rust
// --- maximise (RL16)

/// A point on the background of a player's header: between the status
/// label on the left and the CUE, mode and BP tiles on the right (a column
/// is 480 points wide in the default harness).
fn header_point(h: &Harness<'_, AppUi>, player: PlayerId) -> Pos2 {
    let r = column(h, player);
    pos2(r.left() + 200.0, r.top() + 21.0)
}

/// A point on the background of the cartwall's header.
fn cartwall_header_point(h: &Harness<'_, AppUi>) -> Pos2 {
    let r = h.state().cartwall_rect().unwrap();
    pos2(r.center().x, r.top() + 12.0)
}

fn right_click(h: &mut Harness<'_, AppUi>, at: Pos2) {
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    press(h, at, PointerButton::Secondary, true);
    h.run_steps(1);
    press(h, at, PointerButton::Secondary, false);
    h.run_steps(2);
}

fn layout_commands(fake: &Fake) -> Vec<Command> {
    fake.take_sent()
        .into_iter()
        .filter(|c| {
            matches!(
                c,
                Command::SetPanelHidden { .. }
                    | Command::SetPlayerShares { .. }
                    | Command::EqualPlayerWidths
                    | Command::SetCartwallDock(_)
                    | Command::SetCartwallSize(_)
                    | Command::SetLayoutLocked(_)
                    | Command::ResetLayout
            )
        })
        .collect()
}

fn maximise(h: &mut Harness<'_, AppUi>, at: Pos2) {
    right_click(h, at);
    h.get_by_label("Maximise").click();
    h.run_steps(2);
}

#[test]
fn rl16_maximise_and_restore_a_player() {
    let (mut h, fake) = harness(state(2, 3));
    let (p1, p2) = (fake.player(0), fake.player(1));
    let before = (column(&h, p1).width(), column(&h, p2).width());
    maximise(&mut h, header_point(&h, p1));
    assert_eq!(h.state().maximised(), Some(Panel::Player(p1)));
    assert!(near(column(&h, p2).width(), MIN_PLAYER_WIDTH));
    assert!(near(column(&h, p1).width(), before.0 + before.1 - MIN_PLAYER_WIDTH));
    assert!(layout_commands(&fake).is_empty(), "maximising sends nothing");
    right_click(&mut h, header_point(&h, p1));
    h.get_by_label("Restore").click();
    h.run_steps(2);
    assert_eq!(h.state().maximised(), None);
    assert!(near(column(&h, p1).width(), before.0) && near(column(&h, p2).width(), before.1));
}

#[test]
fn rl16_a_maximised_cartwall_leaves_the_players_their_least_height() {
    let (mut h, fake) = harness(state(2, 3));
    maximise(&mut h, cartwall_header_point(&h));
    assert_eq!(h.state().maximised(), Some(Panel::Cartwall));
    assert!(near(column(&h, fake.player(0)).height(), MIN_PLAYER_HEIGHT - 2.0));
}

#[test]
fn rl16_hiding_locking_docking_or_a_new_player_count_end_it() {
    for command in [
        Command::SetPanelHidden {
            panel: Panel::Cartwall,
            hidden: true,
        },
        Command::SetLayoutLocked(true),
        Command::SetCartwallDock(Dock::Left),
        Command::SetPlayerCount(3),
    ] {
        let (mut h, fake) = harness(state(2, 3));
        maximise(&mut h, header_point(&h, fake.player(0)));
        Controller::send(fake.as_ref(), command.clone());
        h.run_steps(2);
        assert_eq!(h.state().maximised(), None, "{command:?}");
    }
    let (mut h, fake) = harness(state(2, 3));
    maximise(&mut h, header_point(&h, fake.player(1)));
    h.get_by_label("Hide P2").click();
    h.run_steps(2);
    assert_eq!(h.state().maximised(), None, "the maximised panel was hidden");
}

#[test]
fn rl16_a_splitter_drag_restores_the_earlier_sizes_first() {
    let (mut h, fake) = harness(state(2, 3));
    let (p1, p2) = (fake.player(0), fake.player(1));
    let before = column(&h, p1).width();
    maximise(&mut h, header_point(&h, p1));
    let from = splitter_at(&h, p1);
    drag_by(&mut h, &fake, from, 40.0);
    assert_eq!(h.state().maximised(), None);
    assert!(near(column(&h, p1).width(), before + 40.0), "{:?}", column(&h, p1));
    press(&mut h, pos2(from.x + 40.0, from.y), PointerButton::Primary, false);
    h.run_steps(40);
    assert!(near(column(&h, p1).width(), before + 40.0));
    assert!(near(column(&h, p2).width(), before - 40.0));
}

#[test]
fn rl16_hide_from_the_header_menu() {
    let (mut h, fake) = harness(state(2, 3));
    let p2 = fake.player(1);
    right_click(&mut h, header_point(&h, p2));
    h.get_by_label("Hide").click();
    h.run_steps(2);
    assert_eq!(
        layout_commands(&fake),
        vec![Command::SetPanelHidden {
            panel: Panel::Player(p2),
            hidden: true,
        }]
    );
}

#[test]
fn rl16_a_locked_layout_disables_the_header_menu() {
    let mut s = state(2, 3);
    s.layout.locked = true;
    let (mut h, fake) = harness(s);
    right_click(&mut h, header_point(&h, fake.player(0)));
    assert!(disabled(&h, "Maximise"));
    assert!(disabled(&h, "Hide"));
}

#[test]
fn rl16_with_one_panel_shown_maximise_is_disabled() {
    let mut s = state(1, 3);
    s.layout.cartwall.hidden = true;
    let (mut h, fake) = harness(s);
    right_click(&mut h, header_point(&h, fake.player(0)));
    assert!(disabled(&h, "Maximise"));
    assert!(disabled(&h, "Hide"), "the last shown panel");
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-app --test panel_layout rl16` and `cargo test -p fp-app --test layout_ui rl16`
Expected: FAIL to compile: no `Maximised` in `panel_layout`, no method `maximised` on `AppUi`.

- [ ] **Step 3: The maximise state**

`crates/fp-app/src/ui/panel_layout.rs`: make the `fp_model::layout` import read `use fp_model::layout::{Dock, Panel};` and append:

```rust
/// RL16: what ends a maximised panel when it changes: the players, the
/// hidden panels, the cartwall's dock and the lock. A resize does not.
#[derive(Debug, Clone, PartialEq)]
pub struct ArrangementKey {
    players: Vec<PlayerId>,
    hidden_players: Vec<PlayerId>,
    cartwall_hidden: bool,
    dock: Dock,
    locked: bool,
}

impl ArrangementKey {
    pub fn of(state: &AppState) -> Self {
        let layout = &state.layout;
        Self {
            players: state.player_ids(),
            hidden_players: layout.hidden_players.clone(),
            cartwall_hidden: layout.cartwall.hidden,
            dock: layout.cartwall.dock,
            locked: layout.locked,
        }
    }
}

/// RL16: the panel maximised for the moment. View state only: never saved,
/// no command.
#[derive(Debug, Clone, PartialEq)]
pub struct Maximised {
    pub panel: Panel,
    key: ArrangementKey,
}

impl Maximised {
    pub fn new(panel: Panel, state: &AppState) -> Self {
        Self {
            panel,
            key: ArrangementKey::of(state),
        }
    }

    /// Whether it still holds: nothing that ends it changed since.
    pub fn holds(&self, state: &AppState) -> bool {
        self.key == ArrangementKey::of(state)
    }
}
```

- [ ] **Step 4: The menu, the view and the pass**

`crates/fp-app/src/ui/panels.rs`: make the imports read

```rust
use std::collections::HashMap;

use egui::{Align, Layout, Rect, Sense, TextureHandle, Ui, UiBuilder, pos2, vec2};
use fp_model::layout::{Dock, Panel};
use fp_model::{AppState, Command, PlayerId, TrackId};

use super::app::{Scene, ViewState};
use super::panel_layout::{self, Arrangement, LiveSplit, Maximised, PANEL_GAP};
use super::player;
use super::theme;
use super::widgets::{self, TileStyle, font};
```

in `player_splitter`, inside `if response.drag_started() { … }`, as its first statement, add:

```rust
        // RL16: a drag first ends a maximised panel, then applies itself to
        // the sizes from before (`normal`).
        view_state.maximised = None;
```

and append:

```rust
/// Brings the view state in line with the model before the frame's input
/// is handled: a maximised panel ends when the arrangement changed (RL16).
pub(crate) fn settle_view(view_state: &mut ViewState, state: &AppState) {
    if view_state
        .maximised
        .as_ref()
        .is_some_and(|m| !m.holds(state))
    {
        view_state.maximised = None;
    }
}

/// RL16: the menu of a panel header's background (right click): Maximise,
/// or Restore on the maximised panel, and Hide. Both are disabled while the
/// layout is locked; Maximise also with one panel shown, Hide on the last
/// shown panel.
pub(crate) fn header_menu(
    response: &egui::Response,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    panel: Panel,
) {
    response.context_menu(|ui| {
        let t = scene.i18n;
        let layout = &scene.state.layout;
        let players = scene.state.player_ids();
        ui.set_min_width(160.0);
        let maximised = view_state
            .maximised
            .as_ref()
            .is_some_and(|m| m.panel == panel);
        if maximised {
            if ui.button(t.tr("layout-restore")).clicked() {
                view_state.maximised = None;
                ui.close();
            }
        } else {
            let can = !layout.locked && layout.shown_panels(&players) > 1;
            let item = egui::Button::new(t.tr("layout-maximise"));
            if ui.add_enabled(can, item).clicked() {
                view_state.maximised = Some(Maximised::new(panel, scene.state));
                ui.close();
            }
        }
        let can_hide = !layout.locked && !layout.is_last_shown(&players, panel);
        if ui
            .add_enabled(can_hide, egui::Button::new(t.tr("layout-hide")))
            .clicked()
        {
            scene.ctl.send(Command::SetPanelHidden {
                panel,
                hidden: true,
            });
            ui.close();
        }
    });
}
```

`crates/fp-app/src/ui/app.rs`:

- In `ViewState`, after `live_split`:

```rust
    /// The panel maximised for the moment (RL16); never saved.
    pub(crate) maximised: Option<super::panel_layout::Maximised>,
```

- After `cartwall_rect`:

```rust
    /// The panel maximised for the moment (resizable layout spec RL16).
    pub fn maximised(&self) -> Option<fp_model::layout::Panel> {
        self.view.maximised.as_ref().map(|m| m.panel)
    }
```

- In `ui`, right before `self.keyboard(&ctx, &state);`:

```rust
        // The view follows the model before the keyboard acts (RL16, RL17).
        panels::settle_view(&mut self.view, &state);
```

- Replace `let arranged = pass(None);` with:

```rust
        let maximised = self.view.maximised.as_ref().map(|m| m.panel);
        let arranged = pass(maximised);
        // RL16: a splitter drag starts from the sizes without the maximised
        // panel.
        let normal = if maximised.is_some() {
            pass(None)
        } else {
            arranged.clone()
        };
```

and pass `&normal` as the last argument of `panels::players_row` (instead of the second `&arranged`).

`crates/fp-app/src/ui/player.rs`: add `use fp_model::layout::Panel;`; in `column`, call `header(ui, scene, view_state, id, index, &pv);`; make `header` read:

```rust
fn header(
    ui: &mut Ui,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    id: PlayerId,
    index: usize,
    pv: &PlayerView,
) {
    let t = scene.i18n;
    // RL16: a right click on the header's background opens the panel menu;
    // the tiles, added after it, keep their own clicks.
    let row = Rect::from_min_size(ui.cursor().min, vec2(ui.available_width(), 20.0));
    let background = ui.interact(row, egui::Id::new(("player-header", id.0)), Sense::click());
    ui.allocate_ui_with_layout(
        // … the existing body, unchanged …
    );
    super::panels::header_menu(&background, scene, view_state, Panel::Player(id));
}
```

(Keep every line of the existing `allocate_ui_with_layout(…)` call as it is; only the signature, the two lines before it and the line after it are new.)

`crates/fp-app/src/ui/cartwall.rs`: add `use fp_model::layout::Panel;`; in `strip`, call `header(ui, scene, view_state);`; make `header` begin and end:

```rust
fn header(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState) {
    let t = scene.i18n;
    let wall = &scene.state.cartwall;
    // RL16: a right click on the header's background opens the panel menu;
    // the title, tabs and Stop all, added after it, keep their own clicks.
    let row = Rect::from_min_size(ui.cursor().min, vec2(ui.available_width(), HEADER_HEIGHT));
    let background = ui.interact(row, ui.id().with("cartwall-header"), Sense::click());
    ui.allocate_ui_with_layout(
        // … the existing body, unchanged …
    );
    super::panels::header_menu(&background, scene, view_state, Panel::Cartwall);
}
```

- [ ] **Step 5: The strings**

Append to each locale file, after Task 4's block:

`en-US`:
```
layout-maximise = Maximise
layout-restore = Restore
layout-hide = Hide
```
`es-ES`:
```
layout-maximise = Maximizar
layout-restore = Restaurar
layout-hide = Ocultar
```
`ca-ES`:
```
layout-maximise = Maximitzar
layout-restore = Restaurar
layout-hide = Amagar
```
`de-DE`:
```
layout-maximise = Maximieren
layout-restore = Wiederherstellen
layout-hide = Ausblenden
```
`eu-ES`:
```
layout-maximise = Maximizatu
layout-restore = Leheneratu
layout-hide = Ezkutatu
```
`fr-FR`:
```
layout-maximise = Agrandir
layout-restore = Restaurer
layout-hide = Masquer
```
`gl-ES`:
```
layout-maximise = Maximizar
layout-restore = Restaurar
layout-hide = Ocultar
```
`it-IT`:
```
layout-maximise = Ingrandisci
layout-restore = Ripristina
layout-hide = Nascondi
```
`nl-NL`:
```
layout-maximise = Maximaliseren
layout-restore = Herstellen
layout-hide = Verbergen
```
`pl-PL`:
```
layout-maximise = Maksymalizuj
layout-restore = Przywróć
layout-hide = Ukryj
```
`pt-PT`:
```
layout-maximise = Maximizar
layout-restore = Restaurar
layout-hide = Ocultar
```

- [ ] **Step 6: Run the tests**

Run: `cargo test -p fp-app --test panel_layout`, `cargo test -p fp-app --test layout_ui`, `cargo test -p fp-app --test i18n`, then `cargo test -p fp-app` (the player header and cartwall header tests, the tab and Stop all clicks, must stay green: the background is under them).
Expected: PASS. If `header_point` lands on a header tile in your build (the menu does not open), move it to the gap between the status label and the CUE tile as measured from the header's tiles (`h.get_by_label("CUE…")` rects) and note it in the ledger.

- [ ] **Step 7: Docs**

`docs/user/layout.md`: insert before `## The layout menu`:

```markdown
## Maximise a panel

Right-click the background of a player's header, or of the cartwall's
header (not one of their buttons), for its menu:

- **Maximise** gives that panel all the room: the other panels shrink to
  their smallest. It is only for the moment and is not saved. **Restore**
  in the same menu ends it, and so do hiding or showing a panel, locking
  the layout, moving the cartwall, changing the number of players and
  dragging the gap between two players; the sizes from before come back.
- **Hide** hides the panel, as its tile in the top bar does.
```

`docs/user/players.md`: after the Header table, add a paragraph: "Right-click the header's background (not one of its buttons) to maximise the player for a moment or to hide it (see [Layout](layout.md#maximise-a-panel))."

`docs/technical/ui.md`, at the end of the "Main window layout" section, add:

```markdown
**Maximise** (RL16). `ViewState::maximised` holds a `panel_layout::Maximised`
(the panel and an `ArrangementKey`: the players, the hidden panels, the
dock and the lock); it is never saved and sends no command. The pass gives
a maximised player all the spare width and the others their minimum, and
shrinks the bottom cartwall to its least height; a maximised bottom
cartwall leaves the players `MIN_PLAYER_HEIGHT`. `panels::settle_view`,
called before the keyboard, ends it when the key changed. A splitter drag
ends it and starts from the pass without it (`AppUi::ui` runs the pass a
second time, `normal`, while a panel is maximised). The headers' background
(an `interact` with `Sense::click` added before the header's tiles, so the
tiles keep their clicks) opens `panels::header_menu`: Maximise or Restore,
and Hide; both disabled while locked, Maximise also with one panel shown.
```

- [ ] **Step 8: Gate and commit**

Run the gate. Then:

```bash
git add crates/fp-app docs/user/layout.md docs/user/players.md docs/technical/ui.md
git commit -m "feat(ui): maximise a panel from its header

RL16: a right click on a player's or the cartwall's header background
opens a menu with Maximise (or Restore) and Hide. A maximised panel takes
the free space and the others shrink to their minimum; it lives in the
view only, and locking, hiding or showing a panel, a dock change, a new
player count or a splitter drag end it. Disabled while the layout is
locked.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: Hidden players, the selection and file drops (RL17, RL18)

**Files:**
- Modify: `crates/fp-app/src/ui/panels.rs` (`settle_view`)
- Modify: `crates/fp-app/src/ui/app.rs` (`file_drops`, L1420-1479)
- Modify: `crates/fp-app/locales/*/main.ftl` (all eleven, appended)
- Modify: `docs/user/layout.md`, `docs/user/playlists.md` (Drag and drop, L212-215), `docs/user/keyboard.md` (Delete row, L8), `docs/technical/ui.md`
- Test: `crates/fp-app/tests/layout_ui.rs`

**Interfaces:**
- Consumes: Task 5's `settle_view` (called before the keyboard); `ViewState::{selection, active_player, file_drop}`; `Layout::shown_players`.
- Produces: string `notice-drop-no-player`. `settle_view` also clears hidden players' selection and active player.

- [ ] **Step 1: Write the failing tests**

In `crates/fp-app/tests/layout_ui.rs` append:

```rust
// --- hidden players (RL17, RL18)

#[derive(Debug)]
struct Dropped(std::path::PathBuf);

impl egui::DroppedFile for Dropped {
    fn path(&self) -> &std::path::Path {
        &self.0
    }
    fn bytes(&self) -> Result<Vec<u8>, String> {
        std::fs::read(&self.0).map_err(|e| e.to_string())
    }
}

/// Drops `path` with no pointer position, as some systems do.
fn drop_file(h: &mut Harness<'static, AppUi>, path: &std::path::Path) {
    h.remove_cursor();
    h.step();
    h.input_mut()
        .dropped_files
        .push(std::sync::Arc::new(Dropped(path.to_path_buf())));
    // Dropped paths are read on a helper thread: give it time to answer.
    for _ in 0..40 {
        h.run_steps(1);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

/// Clicks the table row `title` in `player`'s column.
fn click_row(h: &mut Harness<'_, AppUi>, player: PlayerId, title: &str) {
    let col = column(h, player);
    h.get_all_by_label(title)
        .find(|n| {
            let r = n.rect();
            col.contains(r.center()) && r.center().y > col.top() + 300.0
        })
        .expect("the row is shown")
        .click();
    h.run_steps(2);
}

fn removals(fake: &Fake) -> usize {
    fake.take_sent()
        .iter()
        .filter(|c| matches!(c, Command::RemoveEntry(_)))
        .count()
}

#[test]
fn rl17_hiding_a_player_clears_its_selection() {
    let (mut h, fake) = harness(state(2, 3));
    let (p1, p2) = (fake.player(0), fake.player(1));
    click_row(&mut h, p2, "Song 2");
    h.get_by_label("Hide P2").click();
    h.run_steps(2);
    h.key_press(Key::Delete);
    h.run_steps(2);
    assert_eq!(removals(&fake), 0, "Delete removes nothing the operator cannot see");
    h.get_by_label("Show P2").click();
    h.run_steps(2);
    h.key_press(Key::Delete);
    h.run_steps(2);
    assert_eq!(removals(&fake), 0, "shown again, it has no selection");
    // Control: a shown player's selection still works.
    click_row(&mut h, p1, "Song 3");
    h.key_press(Key::Delete);
    h.run_steps(2);
    assert_eq!(removals(&fake), 1);
}

#[test]
fn rl18_a_drop_goes_to_the_first_shown_player_when_the_last_used_is_hidden() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("new.wav");
    std::fs::write(&file, b"x").unwrap();
    let mut s = state(2, 3);
    let p = s.player_ids();
    fp_model::apply(
        &mut s,
        Command::CreatePlaylistIn {
            name: "Second".into(),
            player: p[1],
        },
    )
    .unwrap();
    let second = s.player(p[1]).unwrap().playlist;
    let (mut h, fake) = harness(s);
    click_row(&mut h, p[0], "Song 2");
    h.get_by_label("Hide P1").click();
    h.run_steps(2);
    fake.take_sent();
    drop_file(&mut h, &file);
    let sent = fake.take_sent();
    assert!(
        sent.iter().any(|c| matches!(
            c,
            Command::InsertPaths { playlist, index: 0, .. } if *playlist == second
        )),
        "{sent:?}"
    );
}

#[test]
fn rl18_with_no_player_shown_a_drop_is_refused_with_a_notice() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("new.wav");
    std::fs::write(&file, b"x").unwrap();
    let mut s = state(2, 3);
    s.layout.hidden_players = s.player_ids();
    let (mut h, fake) = harness(s);
    drop_file(&mut h, &file);
    assert!(
        !fake
            .take_sent()
            .iter()
            .any(|c| matches!(c, Command::InsertPaths { .. }))
    );
    assert!(
        h.query_by_label("No player is shown: show one to drop files on it.")
            .is_some()
    );
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-app --test layout_ui rl17 rl18`
Expected: FAIL: Delete sends a `RemoveEntry` for the hidden player's selection; the drop goes to the hidden P1's playlist; with no player shown the drop goes to the first player and no notice shows.

- [ ] **Step 3: The view and the drops**

`crates/fp-app/src/ui/panels.rs`, make `settle_view` read:

```rust
/// Brings the view state in line with the model before the frame's input
/// is handled: a maximised panel ends when the arrangement changed (RL16);
/// a hidden player keeps no selection and is not the last player used, so
/// Delete cannot remove an entry the operator cannot see (RL17).
pub(crate) fn settle_view(view_state: &mut ViewState, state: &AppState) {
    if view_state
        .maximised
        .as_ref()
        .is_some_and(|m| !m.holds(state))
    {
        view_state.maximised = None;
    }
    let hidden = &state.layout.hidden_players;
    view_state
        .selection
        .retain(|player, _| !hidden.contains(player));
    if view_state
        .active_player
        .is_some_and(|player| hidden.contains(&player))
    {
        view_state.active_player = None;
    }
}
```

`crates/fp-app/src/ui/app.rs`, in `file_drops`, replace from `// Pointer positions are not reported during an OS drag on every` down to the `let Some(target) = self.view.file_drop.or_else(fallback) else { return; };` statement (inclusive) with:

```rust
        // Pointer positions are not reported during an OS drag on every
        // platform: without one, the files go to the end of the list shown
        // by the last player used, or else by the first shown player
        // (resizable layout spec RL18).
        let shown = state.layout.shown_players(&state.player_ids());
        let fallback = || {
            let id = self
                .view
                .active_player
                .filter(|p| shown.contains(p))
                .or_else(|| shown.first().copied())?;
            let player = state.player(id).ok()?;
            let list = state.playlists.get(player.playlist)?;
            Some(DropTarget {
                player: player.id,
                playlist: list.id,
                index: list.entries.len(),
            })
        };
        let Some(target) = self.view.file_drop.or_else(fallback) else {
            if shown.is_empty() {
                // RL18: no player is shown to take the files.
                let until = ctx.input(|i| i.time) + NOTICE_SECS;
                self.view.notice = Some((self.i18n.tr("notice-drop-no-player"), until));
            }
            return;
        };
```

- [ ] **Step 4: The string**

Append to each locale file, after Task 5's lines:

`en-US`: `notice-drop-no-player = No player is shown: show one to drop files on it.`
`es-ES`: `notice-drop-no-player = No se ve ningún player: muestra uno para soltar archivos en él.`
`ca-ES`: `notice-drop-no-player = No es veu cap reproductor: mostra'n un per deixar-hi anar fitxers.`
`de-DE`: `notice-drop-no-player = Kein Player ist sichtbar: einen einblenden, um Dateien darauf abzulegen.`
`eu-ES`: `notice-drop-no-player = Ez da erreproduzitzailerik ikusten: erakutsi bat fitxategiak bertan jaregiteko.`
`fr-FR`: `notice-drop-no-player = Aucun lecteur n'est affiché : affichez-en un pour y déposer des fichiers.`
`gl-ES`: `notice-drop-no-player = Non se ve ningún reprodutor: mostra un para soltar ficheiros nel.`
`it-IT`: `notice-drop-no-player = Nessun lettore è visibile: mostrane uno per rilasciarvi i file.`
`nl-NL`: `notice-drop-no-player = Er is geen speler zichtbaar: toon er een om er bestanden op neer te zetten.`
`pl-PL`: `notice-drop-no-player = Żaden odtwarzacz nie jest widoczny: pokaż jeden, aby upuścić na niego pliki.`
`pt-PT`: `notice-drop-no-player = Nenhum leitor está visível: mostre um para largar ficheiros nele.`

- [ ] **Step 5: Run the tests**

Run: `cargo test -p fp-app --test layout_ui`, `cargo test -p fp-app --test main_screen`, `cargo test -p fp-app --test i18n`, then `cargo test -p fp-app`.
Expected: PASS (`main_screen`'s drop tests keep their behaviour: with every player shown, the fallback is the last player used or the first player, as before).

- [ ] **Step 6: Docs**

`docs/user/layout.md`, "Show and hide panels": after the paragraph that begins "Hiding only takes a panel off the screen.", add: "Hiding a player clears its selected track, so **Delete** cannot remove a track you cannot see. Files dropped on the window away from a list go to the last player you used, or to the first shown player; with every player hidden they are refused, and the status bar says so."

`docs/user/playlists.md`, Drag and drop: replace "If the system does not report the position, they go to the end of the list shown." with "If the system does not report the position, they go to the end of the list shown by the last player you used, or by the first shown player; with every player hidden nothing is inserted and the status bar says so (see [Layout](layout.md#show-and-hide-panels))."

`docs/user/keyboard.md`, the `Delete` / `Backspace` row: "Remove the selected track from the playlist shown (not while it is on air). Hiding a player clears its selection."

`docs/technical/ui.md`, at the end of the "Main window layout" section, add:

```markdown
**Hidden players** (RL17, RL18). `panels::settle_view` also drops the
`ViewState::selection` of every hidden player and clears `active_player`
when it is hidden, before `AppUi::keyboard` runs, so Delete never removes
an entry the operator cannot see, whatever hid the player. `file_drops`'s
fallback (no pointer position) is the last player used if shown, else the
first shown player; with none shown the audio files are refused with
`notice-drop-no-player` (playlist files are still imported).
```

- [ ] **Step 7: Gate and commit**

Run the gate. Then:

```bash
git add crates/fp-app docs/user docs/technical/ui.md
git commit -m "feat(ui): hidden players keep no selection and take no drops

RL17: hiding a player clears its table selection and its place as the
last player used, before the keyboard acts, so Delete cannot remove an
entry the operator cannot see. RL18: files dropped away from a table go
to the last player used or the first shown player; with no player shown
they are refused with a notice.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

## Self-review (against the spec)

- RL7's interface parts: the shortcut while hidden (Task 4, `rl7_the_cartwall_shortcut_does_nothing_while_it_is_hidden`); keys still reach a hidden player (Task 4, `rl7_a_hidden_player_still_answers_its_key`). The rest of RL7 is plan 1's.
- RL8: `MIN_PLAYER_WIDTH`, `MIN_PLAYER_HEIGHT` (Task 1), checked on screen (Task 2, `rl8_…`); the cartwall's least height (`cartwall::min_height`, Task 2). The cartwall's least width is plan 3's (side docks).
- RL11: the pass (Task 1: default, shares, hidden players, overflow and exact fit, hidden cartwall, no player shown, content cap, share clamp, too low a window); on screen (Task 2: default rectangles, shares, hidden player, hidden cartwall, no player shown, too narrow, the column id). Side docks in the pass: plan 3. The label `P<n>` from the player's place among all players is by construction (`player::column` gets `player_index`); it has no accessible text to test.
- RL12: players (Task 3: live drag, one command on release, held widths, click, double-click, minimums, cursor, locked, a stale drag); the cartwall splitter: plan 3 (ruling).
- RL13, RL14: plan 3.
- RL15: tiles, cartwall tile, last tile, on-air dot, the menu's commands, the checked dock, locked (Task 4).
- RL16: the sizes (Task 1); maximise and restore, the cartwall, what ends it, a drag first restores, Hide from the menu, locked, one panel (Task 5). On a side dock: plan 3.
- RL17, RL18: Task 6.
- §6: no remote or MIDI change; the guide (`layout.md` and the pages it touches) and `ui.md` grow per task; the cartwall page's dock and scaling text, `persistence.md` (plan 1), the README and the screenshots: plan 3.
- Placeholders: the two "the existing body, unchanged" notes in Task 5 point at code that stays exactly as it is; every new line is given.
- Types: `PassInput`, `Arrangement`, `CartwallSizes`, `LiveSplit`, `Maximised`, `panel_on_air`, `players_row(…, arranged, normal)` match between the tasks that define and use them.

## What plan 3 consumes

- **The pass** (`fp_app::ui::panel_layout`): `PANEL_GAP`, `MIN_PLAYER_WIDTH`, `MIN_PLAYER_HEIGHT`, `CONTENT_HEIGHT_CAP`; `CartwallSizes { content_height, min_height }`; `PassInput { players, layout, area, cartwall, maximised }`; `Arrangement { players_area, widths, overflow, cartwall }`; `arrange`.
  - **Extension point:** in `arrange`, the line `at_the_bottom(input, maximised)` (under the comment "Plan 3 makes this a `match`…") becomes `match layout.cartwall.dock { Dock::Bottom => at_the_bottom(input, maximised), Dock::Left | Dock::Right => on_a_side(input, maximised) }`. `on_a_side` returns `(players_area, Some(cartwall_rect))` with the cartwall the full height of the area; `player_widths(input, &shown, players_area.width(), maximised)` already lays the players out in whatever width is left, so the side docks reuse it unchanged.
  - Plan 3 adds fields to `CartwallSizes` (the least width; the collapsed strip's width) and, if its splitter needs it, a `splitter: Option<Rect>` to `Arrangement`; `cartwall::sizes` fills them. Existing test helpers build `CartwallSizes` with a struct literal: plan 3 updates the `SIZES` constant in `tests/panel_layout.rs`.
  - `at_the_bottom` already honours `CartwallLayout::share()` (`Some`) between the cartwall's least height and the players' least height, and a maximised panel; plan 3's bottom splitter only has to send `SetCartwallSize`.
- **Splitters** (`panels`): `splitter(ui, gap, id, cursor, active, dragging) -> Option<Response>` (use `CursorIcon::ResizeRow` at the bottom, `ResizeColumn` on a side; `active` is `!locked && !collapsed_on_a_side`); `panel_layout::HOLD_SECS` for the cartwall drag's hold; the pattern of `LiveSplit` and `live_widths` (drag state in `ViewState`, follow before drawing, one command on release, hold until the echo). A cartwall drag must also set `view_state.maximised = None` on start (RL16).
- **View state** (`ViewState`, `pub(crate)`): `player_rects`, `cartwall_rect`, `live_split`, `maximised`; `panels::settle_view(view_state, state)` runs before the keyboard; `Maximised::{new, holds}`, `ArrangementKey::of` (already includes the dock).
- **Menus and dots**: `panels::header_menu(&response, scene, view_state, Panel::Cartwall)` for the collapsed side strip's background; `panel_layout::panel_on_air(state, Panel::Cartwall)` for its on-air dot (RL14).
- **Top bar**: `panels::layout_controls` already sends `SetCartwallDock` from the dock radio items (strings `layout-dock-bottom`, `layout-dock-left`, `layout-dock-right`).
- **Accessors for tests**: `AppUi::player_rects() -> Vec<(PlayerId, Rect)>`, `AppUi::cartwall_rect() -> Option<Rect>`, `AppUi::maximised() -> Option<Panel>`; the `layout_ui.rs` helpers `column`, `splitter_at`, `drag_by`, `press`, `right_click`, `cartwall_header_point`, `layout_commands`, `cursor`.
- **Cartwall**: `cartwall::min_height(scene)`, `cartwall::sizes(scene)`; `HEADER_HEIGHT` (private to `cartwall.rs`) gives the collapsed strip's width; `cart_view::{MIN_BUTTON_HEIGHT, BUTTON_HEIGHT, button_height}` for RL13.
- **Docs to finish**: `docs/user/layout.md` (a "Move and resize the cartwall" section; "The layout menu" already lists the docks), `docs/user/cartwall.md` (the dock and that buttons scale), `docs/technical/ui.md` "Main window layout" (replace "The side docks are laid out at the bottom for now" with the side docks, the cartwall splitter, RL13, RL14), the Phase 2 spec amendment (P2.9), the README feature list, and the screenshots (`scripts/site/screenshots.sh`: check every PNG; the top bar now has the tiles on its left).
