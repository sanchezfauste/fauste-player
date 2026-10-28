# Phone plan 2 — Live and Carts screens, the lock

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** a phone's top bar with Live · Carts · Playlists, the lock and a
⋮ menu; a Live screen with every player's card and the cartwall; a Carts
screen; a lock that guards faders and waveforms.

**Architecture:** `touch.rs` owns the phone's screens (`PhoneScreen`), the
top bar's left part and the swipe; a new `player::card` draws the PL4 card
from the existing parts (`wave`, `header`, `top_block`); the lock is a
`Layout` flag set by a new command, read by the card, the column and the
waveform. Playlists stays the current one-player column under player chips
until plan 3.

**Tech Stack:** Rust, egui 0.37, egui_kittest, Fluent.

**Spec:** `docs/superpowers/specs/2026-10-28-phone-layout-design.md`
(PL1, PL3, PL4–PL7, PL12, PL13).

## Global Constraints

- Phones only (`FormFactor::Phone`); a tablet and the desktop do not change.
- No unwrap/expect/panic outside tests; the UI never blocks.
- The lock never blocks Play, a transport button, a cart, a playlist or navigation; zoom and pan of a waveform still work (PL12).
- New strings in all 11 locales (en-US and es-ES written, the others translated).
- Behaviour in `fp-model`: the lock flag and its command, with a reducer test.

## Review Focus

- A scroll of the Live cards that starts on a waveform or a button sends no command (cards scroll with a finger, as the list did).
- Locked: a drag on a fader sends no `SetVolume`, a tap on a waveform no `Seek`, a long press on it opens no marker menu; Play still sends `Play`.
- The lock survives a restart (saved with the layout).
- Rotation keeps the screen; a removed player leaves the Playlists chips.
- A narrow phone (360 pt) fits the top bar: tabs shrink, the clock goes first.

---

### Task 1: The lock in the model (PL12, PL13)

**Files:** `crates/fp-model/src/layout.rs` (`Layout::touch_locked: bool`, `#[serde(default)]`), `command.rs` (`Command::SetTouchLocked(bool)`), `reducer.rs`; tests in `crates/fp-model/tests/it/` next to the `SetLayoutLocked` test, and the store round trip next to `layout.locked`'s.

- [ ] Test `set_touch_locked_sets_the_flag` (and that it is independent of `layout.locked`); test the store keeps it across save and load; a layout file without the field loads as unlocked.
- [ ] Watch them fail; implement; commit `feat(model): a touch lock for faders and waveforms`.

### Task 2: Phone navigation (PL1, PL3, PL2 for screens)

**Files:** `crates/fp-app/src/ui/touch.rs`, `ui/app.rs` (`top_bar`, `top_bar_right`, `AppUi::back`), locales; tests `crates/fp-app/tests/it/touch_ui.rs` (rewrite the tests of the removed views).

- `PhoneScreen { Live (default), Carts, Playlists }`. `PhoneView { screen, playlists: Option<PlayerId> }` (the player the Playlists screen shows; `None` = the first). Remove `pair`, `pairs`, `swipe`, `pair_players`, `all_players`, the "Open Pn" and "All" chips and their strings (`touch-players`, `touch-open-player`, `touch-all-players`).
- Top bar on a phone, left to right: the three tabs (`touch-live` "Live", `touch-carts` "Carts", `touch-playlists` "Playlists"), shrinking to fit; then the lock tile (Task 5); then a ⋮ tile (`touch-more` "More") whose menu has Settings and About; the clock right of them while it fits (left out below 400 pt of width — my call).
- Playlists (interim until plan 3): player chips (P1…Pn) and that player's column (`columns(... fader = !locked)`).
- `AppUi::back`: after windows and menus, a screen other than Live goes to Live (Android's convention for top-level tabs — my call).
- [ ] Tests: `the_phone_top_bar_has_the_tabs_the_lock_and_more`, `more_opens_settings_and_about`, `playlists_shows_the_chosen_player_with_its_playlist`, `back_from_another_screen_goes_to_live`, `the_top_bar_fits_a_narrow_phone` (360 pt: every tab and the lock are inside the screen).
- [ ] Watch them fail; implement; commit `feat(ui): phone tabs Live, Carts and Playlists`.

### Task 3: The Live screen (PL4–PL6)

**Files:** `crates/fp-app/src/ui/player.rs` (`pub(crate) fn card(ui, scene, view_state, covers, index, locked)`), `ui/touch.rs` (`live`); remove `player::compact` and `progress_bar`; tests in `touch_ui.rs`.

- `card`: a frame like `compact`'s; top to bottom: the waveform (`wave` with height `CARD_WAVE_HEIGHT` = 40, `editable_markers: false`, seek off when locked), `header`, `top_block(.., fader = !locked)`. No cover: `info_row` takes a `cover: bool` (false in the card — my call, it leaves the title room).
- `live`: portrait — the cards in a vertical `ScrollArea` on top, at most `LIVE_CARDS_SHARE` = 0.6 of the height (less when the cards are shorter), the cartwall (`cartwall::strip(.., fill = true)`) below in the rest; landscape — the cards left (`LIVE_CARDS_SHARE` of the width) scrolling, the cartwall right.
- [ ] Tests: `live_shows_every_player_card_and_the_cartwall` (portrait and landscape: a Play per player and the cart tiles), `a_card_has_the_waveform_above_its_buttons` (the waveform's rect is above Play's), `a_tap_on_a_live_waveform_seeks`, `a_drag_on_a_card_fader_moves_it`, `a_scroll_of_the_cards_sends_nothing` (drag up 200 from a card's title).
- [ ] Watch them fail; implement; commit `feat(ui): the phone's Live screen`.

### Task 4: The Carts screen (PL7)

**Files:** `ui/touch.rs` (`carts`, the swipe tracker generalised from `follow_swipe`), tests in `touch_ui.rs`.

- The cartwall full screen as today (`strip(.., true)`, turned in portrait). A sideways swipe that no widget took sends `Command::ShowCartPage` with the next or previous page (wrapping: no — it stops at the ends; my call).
- [ ] Tests: `a_swipe_on_the_carts_screen_shows_the_next_page` (two pages; swipe left from empty space → `ShowCartPage(second)`), `a_swipe_on_the_last_page_does_nothing`, keep `a_portrait_cartwall_turns_a_wide_page` and `a_landscape_cartwall_keeps_the_page` passing on the Carts screen.
- [ ] Watch them fail; implement; commit `feat(ui): the phone's Carts screen with page swipes`.

### Task 5: The lock in the interface (PL12, PL13)

**Files:** `ui/app.rs` or `ui/touch.rs` (the lock tile), `ui/widgets.rs` (`WaveInput::locked: bool`: no seek, drawn dimmed), `ui/wave_panel.rs` (`WavePanelInput::locked`: no marker editing, no marker menu; wheel, pinch and pan still work), `ui/player.rs` (fader `!locked`, dimmed); tests in `touch_ui.rs` and `waveform_ui.rs`.

- The tile shows a closed padlock lit (`theme::NEUTRAL_800` fill, the text colour) when locked and an open one otherwise (`egui_phosphor` `LOCK_SIMPLE` / `LOCK_SIMPLE_OPEN`); tooltip/label `tip-touch-lock` "Lock faders and waveforms" / `tip-touch-unlock` "Unlock faders and waveforms". A tap sends `SetTouchLocked(!locked)`. Shown on phones only.
- [ ] Tests: `the_lock_button_sends_set_touch_locked`, `locked_a_card_fader_does_not_move`, `locked_a_tap_on_a_waveform_does_not_seek`, `locked_play_still_plays`, `locked_a_pinch_still_zooms` (waveform_ui), `locked_the_playlists_fader_does_not_move`, `a_tablet_has_no_lock_button`.
- [ ] Watch them fail; implement; commit `feat(ui): the phone lock for faders and waveforms`.

### Task 6: Docs, checks and review

- [ ] `docs/user/android.md` (the tabs, Live, Carts, the lock, More), `docs/technical/mobile.md` (the touch layout section: screens, card, lock flag), the old spec's interim notes if any.
- [ ] Locales: every new key in the 11 files, removed keys removed (`scripts`/tests that check locales pass).
- [ ] Full verification (fmt, clippy, tests, Android clippy, isolation); emulator screenshots of Live (portrait and landscape), Carts and Playlists for the maintainer; fresh opus reviewer; fix Critical/Important; PR; CI; merge.
