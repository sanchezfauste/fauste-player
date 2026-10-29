# Phone layout plan 5: Settings — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Settings on a phone (phone layout spec §10, PL20–PL21): full
screen, the list of sections first, then the chosen section with every
setting stacked; Back returns to the list, then closes.

**Architecture:** `settings::show` gets `SettingsDeps::phone`. On a phone
the modal takes the whole screen; `SettingsState::phone_section` says
whether a section or the list shows. `labelled_row`, which every section
uses, stacks label and control when a per-frame flag (set by `body`, like
`set_body_right`) says so. `AppUi::back` asks `SettingsState::phone_back`
before closing Settings. The chips of #173 go: only phones used them.

**Tech Stack:** Rust, egui 0.37, egui_kittest.

**Spec:** `docs/superpowers/specs/2026-10-28-phone-layout-design.md`

## Global Constraints

- Phones only; tablets and the desktop keep the window with its side list.
- Changes apply at once, as now; no new settings.
- New strings in all 11 locales.

## Review Focus

1. Editing a cart on a phone (long press → Edit) opens its Cartwall
   section directly, not the list.
2. Back with a question over Settings (Restore defaults, Apply now)
   closes the question first.
3. A pick asked from a section (plan 4) returns to that section.
4. A wide control (the output pickers, sliders with values) fits the
   phone's width when stacked.
5. A phone turned sideways (412 pt high) still reaches Close.

---

### Task 1: full screen and the section list (PL20)

**Files:** `crates/fp-app/src/ui/settings.rs` (`show`, `SettingsState`,
`phone_back`, a list), `crates/fp-app/src/ui/app.rs` (deps, Back), tests
`crates/fp-app/tests/it/touch_ui.rs`.

- [ ] Tests: on a phone Settings opens on the list (every section's name,
  no "Buffer size") and fills the screen; a tap opens the section with a
  back arrow; Back returns to the list, then closes; editing a cart opens
  its section; a tablet keeps the side list. Replace
  `phone_settings_put_the_sections_above_the_content`.
- [ ] Implement; string `settings-back` (the arrow's label). Remove
  `nav_chips` and `COMPACT_WIDTH`. Commit.

### Task 2: stacked settings (PL21)

**Files:** `settings.rs` (`labelled_row`), tests in `touch_ui.rs`.

- [ ] Tests: on a phone a row's control is below its label and starts at
  the label's left; the desktop row keeps the label column.
- [ ] Implement. Commit.

### Task 3: documentation and the emulator

- [ ] `docs/user/android.md`, `docs/technical/mobile.md`; screenshots of
  the list and a section, portrait and landscape. Commit.
