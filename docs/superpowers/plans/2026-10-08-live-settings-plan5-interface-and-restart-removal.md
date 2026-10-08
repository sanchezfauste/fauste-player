# Live Settings, Plan 5: The Interface, and the Restart Goes

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The operator sees what waits and on what (**Settings pending** pill, pending panel, Settings footer), can **Apply now** with a confirmation that says what it briefly interrupts, and the application restart of feedback 2 O4 (pill, Restart now, relaunch, `RestartReason`, `restart_pending`, `restart_handoff_ms`) is removed (live settings spec §9, Q2).

**Architecture:** A new `ui/pending.rs` turns `fp_model::live::{pending, interruptions}` into text (pure functions, unit-tested) and draws two modals (the pending panel and Apply now's confirmation) in the style of the close guard. `AppUi` replaces the restart pill, the restart flag and the started configuration with three view flags (`pending_open`, `apply_now_requested`, `confirm_apply_now`); the status bar's label comes from `state.live.audio_system_in_use`. Then the relaunch code and the model's restart rules are deleted, with their strings, tests and docs.

**Tech Stack:** Rust 2024, egui (`egui::Modal`), `egui_kittest`, Fluent (`crates/fp-app/locales/*/main.ftl`).

**Spec:** `docs/superpowers/specs/2026-10-07-live-settings-design.md` §9, §12, §14 (read it whole, with the "Planning notes"). Plans 1–4 must be merged first (plan 1 gives `pending`, `interruptions`, `has_output_items`, `Command::ApplySettingsNow`; plan 2 the `AudioSystemInUse` report).


**Maintainer rulings (2026-10-08), binding:** (1) rules L18 and L19 are dropped: there are no `Players` or `CartPage` pending items, no `PlayerPaused` or `CartsBeyondGrid` causes, and the strings `pending-players`, `pending-cart-page`, `pending-cart-grid-on-load`, `apply-now-limits`, `cause-player-paused` and the grid cause string are not added; ignore every step below that builds them. (2) The pending panel and the Apply-now confirmation are separate pop-up windows; the pill opens the panel. (3) Route and audio-system failures drop the `{running}` part of `pending-failed`.

## Global Constraints

- All code, identifiers, comments, docs and commit messages in English; never mention other playout or radio-automation products.
- UI strings are Fluent messages; `crates/fp-app/locales/en-US/main.ftl` is the source. Every new or changed string goes into every locale file: en-US and es-ES written by hand (given below), the nine others (ca-ES, de-DE, eu-ES, fr-FR, gl-ES, it-IT, nl-NL, pl-PL, pt-PT) AI-translated. A removed key is removed from all eleven. `cargo test -p fp-app --test i18n` checks parity, variables and plural variants.
- The UI never blocks (rule 8): device names come from the list Settings already enumerated on its helper thread; an unknown id is shown as is.
- §9.3: **Apply now** sends `Command::ApplySettingsNow` at once when `interruptions` is empty; otherwise it opens the confirmation (Cancel is the default; Esc and closing it cancel). If the list empties while it is open, it closes and sends the command.
- The close guard (O6) is unchanged except that it no longer guards a restart.
- The user guide in `docs/user/` is English only (no `docs/i18n/` edits). `CHANGELOG.md` is never edited by hand.
- Conventional Commits; every commit ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Gate for every task: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.

## Review Focus

1. **Apply now pressed while the cause ends in the same frame** (a player stopped by MIDI or the remote): the confirmation closes by itself and the command is still sent once. Test: Task 2, `the_confirmation_applies_by_itself_when_what_it_would_cut_ends`.
2. **Esc while the confirmation is open over Settings**: it cancels the confirmation only; Settings stays open and nothing is sent. Test: Task 2, `esc_cancels_the_confirmation_and_keeps_settings_open`.
3. **Only limit items pending** (players or a cart page over a lower limit): the pill shows them, but Apply now is not offered, since it would do nothing. Test: Task 2, `limit_items_alone_offer_no_apply_now`.
4. **A device the device list does not know** (unplugged, another system): its id is shown as is. Test: Task 1, `items_and_causes_read_as_the_spec_says` (names by id).
5. **An old configuration file with `tuning.restart_handoff_ms`**: it still loads, the key is ignored with a warning. Test: Task 3, `an_old_restart_handoff_is_ignored_on_load`.

---

## File Structure

| File | Responsibility |
|---|---|
| `crates/fp-app/src/ui/pending.rs` (new) | Text for pending items, causes, failures and interruptions; the pending panel and Apply now's confirmation |
| `crates/fp-app/src/ui.rs` | `pub mod pending;` (module list) |
| `crates/fp-app/src/ui/app.rs` | Pill, panel and confirmation flow, Esc handling, status bar label; restart removed |
| `crates/fp-app/src/ui/settings.rs` | Footer notice and Apply now; `SettingsState::device_label` |
| `crates/fp-app/src/ui/settings/devices.rs` | `find` visible to `settings.rs` |
| `crates/fp-app/src/ui/exit_guard.rs` | `ExitIntent::Restart` removed; `button` shared |
| `crates/fp-app/src/main.rs` | Relaunch and platform label removed |
| `crates/fp-app/src/restart.rs`, `crates/fp-app/tests/relaunch.rs`, `crates/fp-app/tests/restart_ui.rs` | Deleted |
| `crates/fp-app/tests/live_settings.rs` (new) | Kittests for §9 |
| `crates/fp-model/src/restart.rs`, `crates/fp-model/tests/restart.rs` | Deleted |
| `crates/fp-model/src/config.rs` | `restart_handoff_ms` removed |
| `crates/fp-app/locales/*/main.ftl` (11 files) | Keys added and removed |
| `README.md`, `docs/user/{settings,getting-started,bit-perfect,troubleshooting}.md`, `docs/technical/{ui,architecture,persistence}.md`, three older specs | Docs |

---

### Task 1: Pending text and strings

**Files:**
- Create: `crates/fp-app/src/ui/pending.rs` (text functions only in this task; the modals come in Task 2)
- Modify: `crates/fp-app/src/ui.rs` (module list: add `pub mod pending;` in alphabetical place)
- Modify: `crates/fp-app/locales/*/main.ftl` (11 files)

**Interfaces:**
- Consumes (plan 1): `fp_model::{Pending, PendingItem, BusyCause, DeviceSettings, Holder, Target}`.
- Produces (in `crate::ui::pending`):
  - `pub(crate) type DeviceNames<'a> = &'a dyn Fn(&OutputDevice) -> String;`
  - `pub(crate) fn describe(t: &I18n, state: &AppState, names: DeviceNames<'_>, p: &Pending) -> String` (item, then " — waiting for …" when it has causes)
  - `pub(crate) fn describe_item(t: &I18n, state: &AppState, names: DeviceNames<'_>, item: &PendingItem) -> String`
  - `pub(crate) fn describe_cause(t: &I18n, state: &AppState, cause: &BusyCause) -> String`
  - `pub(crate) fn describe_failure(t: &I18n, names: DeviceNames<'_>, p: &Pending) -> Option<String>`
  - `pub(crate) fn target_label(t: &I18n, state: &AppState, names: DeviceNames<'_>, target: &Target) -> String`
  - `pub(crate) fn device_changes(t: &I18n, from: &DeviceSettings, to: &DeviceSettings) -> String`

- [ ] **Step 1: Add the strings**

Append to `crates/fp-app/locales/en-US/main.ftl`:

```ftl

# Settings pending (live settings)
top-settings-pending = Settings pending
settings-pending = Some changes wait until the outputs they affect are free.
settings-apply-now = Apply now
pending-device = { $device }: { $changes }
pending-change-rate = sample rate { $from } → { $to } kHz
pending-change-buffer = buffer { $from } → { $to }
pending-change-bit-perfect-on = bit-perfect on
pending-change-bit-perfect-off = bit-perfect off
pending-change-dsd = DSD: { $mode }
pending-change-dsd-mix = DSD mix
pending-change-dsd-silence = DSD silence { $from } → { $to } ms
pending-route = { $holder } → { $device }
holder-player-main = P{ $n } Main
holder-player-cue = P{ $n } CUE
holder-cartwall-main = Cartwall Main
holder-cartwall-cue = Cartwall CUE
pending-audio-system = Audio system: { $from } → { $to }
pending-players = Players: { $from } → { $to }
pending-cart-page = Cart page “{ $name }”: { $rows }×{ $cols } → { $r }×{ $c }
pending-waiting = waiting for { $causes }
cause-player-playing = P{ $n } playing
cause-player-fading = P{ $n } fading
cause-player-cue = P{ $n } CUE
cause-player-paused = P{ $n } paused
cause-cart-playing = cart “{ $name }” playing
cause-cart-cue = cart “{ $name }” CUE
cause-carts-beyond-grid = { $count ->
    [one] { $count } cart beyond the new grid
   *[other] { $count } carts beyond the new grid
}
pending-failed = { $device } did not take { $what }: { $reason }. Still { $running }.
pending-running = { $rate } kHz, buffer { $buffer }
pending-cart-grid-on-load = If the application starts before this is applied, carts beyond the new grid are dropped.
apply-now-title = Apply now?
apply-now-body = Applying now briefly interrupts the audio on:
apply-now-line = { $device }: { $causes }
apply-now-limits = Player and cart page limits keep waiting until what they affect is free.
apply-now-confirm = Interrupt and apply
```

Append to `crates/fp-app/locales/es-ES/main.ftl`:

```ftl

# Ajustes pendientes (ajustes en vivo)
top-settings-pending = Ajustes pendientes
settings-pending = Algunos cambios esperan a que las salidas que afectan queden libres.
settings-apply-now = Aplicar ahora
pending-device = { $device }: { $changes }
pending-change-rate = frecuencia de muestreo { $from } → { $to } kHz
pending-change-buffer = búfer { $from } → { $to }
pending-change-bit-perfect-on = bit-perfect activado
pending-change-bit-perfect-off = bit-perfect desactivado
pending-change-dsd = DSD: { $mode }
pending-change-dsd-mix = mezcla DSD
pending-change-dsd-silence = silencio DSD { $from } → { $to } ms
pending-route = { $holder } → { $device }
holder-player-main = P{ $n } Main
holder-player-cue = P{ $n } CUE
holder-cartwall-main = Cartuchera Main
holder-cartwall-cue = Cartuchera CUE
pending-audio-system = Sistema de audio: { $from } → { $to }
pending-players = Players: { $from } → { $to }
pending-cart-page = Página de cartuchos «{ $name }»: { $rows }×{ $cols } → { $r }×{ $c }
pending-waiting = esperando a { $causes }
cause-player-playing = P{ $n } sonando
cause-player-fading = P{ $n } con fundido
cause-player-cue = P{ $n } en CUE
cause-player-paused = P{ $n } en pausa
cause-cart-playing = cartucho «{ $name }» sonando
cause-cart-cue = cartucho «{ $name }» en CUE
cause-carts-beyond-grid = { $count ->
    [one] { $count } cartucho fuera de la nueva cuadrícula
   *[other] { $count } cartuchos fuera de la nueva cuadrícula
}
pending-failed = { $device } no ha aceptado { $what }: { $reason }. Sigue con { $running }.
pending-running = { $rate } kHz, búfer { $buffer }
pending-cart-grid-on-load = Si la aplicación arranca antes de aplicarlo, se descartarán los cartuchos fuera de la nueva cuadrícula.
apply-now-title = ¿Aplicar ahora?
apply-now-body = Aplicar ahora interrumpe brevemente el audio en:
apply-now-line = { $device }: { $causes }
apply-now-limits = Los límites de players y de páginas de cartuchos siguen esperando a que lo que afectan quede libre.
apply-now-confirm = Interrumpir y aplicar
```

For each of `ca-ES`, `de-DE`, `eu-ES`, `fr-FR`, `gl-ES`, `it-IT`, `nl-NL`, `pl-PL`, `pt-PT`: append the same 36 keys, translated from the en-US block into that language (AI translation, as the project does for these locales): keep every key and every `{ $variable }` exactly; keep `P{ $n }`, `Main`, `CUE`, `DSD`, `DoP`, `kHz`, `ms`, `bit-perfect` as the file already writes them (look at that file's `settings-main`, `settings-cue`, `settings-bit-perfect…` and `cartwall-title` lines and use the same words); give `cause-carts-beyond-grid` the plural variants that language uses (copy the selector shape of the same file's `footer-count` message); use the quotation marks that file already uses (see its `playlist-import-empty`). Put the block under a comment line in that language.

- [ ] **Step 2: Write the failing tests**

Create `crates/fp-app/src/ui/pending.rs` with only the tests first:

```rust
//! Settings pending (live settings spec §9): what waits, on what, and the
//! confirmation of Apply now. The rules are `fp_model::live`; this module
//! only describes them and asks.

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use fp_model::{BusyCause, Config, DsdMix, Pending, Route};

    fn en() -> I18n {
        I18n::new(Some("en-US"))
    }

    fn base() -> DeviceSettings {
        DeviceSettings {
            sample_rate: 48_000,
            buffer_frames: 512,
            bit_perfect: false,
            dsd: DsdOutput::Pcm,
            dsd_mix: None,
            dsd_silence_ms: None,
        }
    }

    fn dac() -> OutputDevice {
        OutputDevice {
            backend: "null".into(),
            device: "dac".into(),
        }
    }

    #[test]
    fn device_changes_name_each_value_that_changes() {
        let t = en();
        let to = DeviceSettings {
            sample_rate: 96_000,
            buffer_frames: 256,
            bit_perfect: true,
            ..base()
        };
        assert_eq!(
            device_changes(&t, &base(), &to),
            "sample rate 48 → 96 kHz, buffer 512 → 256, bit-perfect on"
        );
        let dop = DeviceSettings {
            bit_perfect: true,
            dsd: DsdOutput::Dop,
            dsd_mix: Some(DsdMix::ConvertToPcm),
            dsd_silence_ms: Some(200.0),
            ..base()
        };
        let held = DeviceSettings {
            dsd_mix: Some(DsdMix::HoldOthers),
            dsd_silence_ms: Some(400.0),
            ..dop
        };
        assert_eq!(
            device_changes(&t, &dop, &held),
            "DSD mix, DSD silence 200 → 400 ms"
        );
        let cd = DeviceSettings {
            sample_rate: 44_100,
            ..base()
        };
        assert_eq!(device_changes(&t, &cd, &base()), "sample rate 44.1 → 48 kHz");
        assert_eq!(device_changes(&t, &base(), &dop), "bit-perfect on, DSD: DoP");
    }

    #[test]
    fn items_and_causes_read_as_the_spec_says() {
        let t = en();
        let state = AppState::new(Config::default(), "Main");
        let p = state.players[0].id;
        // Names by id, as before the device lists are read.
        let names = |d: &OutputDevice| d.device.clone();
        let phones = Route {
            backend: "null".into(),
            device: "phones".into(),
            first_channel: 0,
        };
        let item = PendingItem::Route {
            holder: Holder::PlayerCue(p),
            from: None,
            to: Some(phones),
        };
        assert_eq!(describe_item(&t, &state, &names, &item), "P1 CUE → phones");
        let item = PendingItem::Route {
            holder: Holder::CartwallMain,
            from: None,
            to: None,
        };
        assert_eq!(
            describe_item(&t, &state, &names, &item),
            "Cartwall Main → System default"
        );
        let pending = Pending {
            item: PendingItem::Device {
                device: dac(),
                from: base(),
                to: DeviceSettings {
                    buffer_frames: 1024,
                    ..base()
                },
            },
            causes: vec![BusyCause::PlayerPlaying(p), BusyCause::PlayerCue(p)],
            failure: None,
        };
        assert_eq!(
            describe(&t, &state, &names, &pending),
            "dac: buffer 512 → 1024 — waiting for P1 playing, P1 CUE"
        );
        let item = PendingItem::AudioSystem {
            from: None,
            to: Some("null".into()),
        };
        assert_eq!(
            describe_item(&t, &state, &names, &item),
            "Audio system: System default → No output (silent)"
        );
        let page = state.cartwall.pages[0].id;
        assert_eq!(
            describe_item(
                &t,
                &state,
                &names,
                &PendingItem::CartPage {
                    page,
                    from: (2, 8),
                    to: (2, 4)
                }
            ),
            "Cart page “Carts 1”: 2×8 → 2×4"
        );
        assert_eq!(
            describe_cause(&t, &state, &BusyCause::CartsBeyondGrid { page, count: 2 }),
            "2 carts beyond the new grid"
        );
        assert_eq!(
            target_label(&t, &state, &names, &Target::Route(Holder::PlayerMain(p))),
            "P1 Main"
        );
    }

    #[test]
    fn a_refusal_says_what_the_device_keeps() {
        let t = en();
        let names = |d: &OutputDevice| d.device.clone();
        let pending = Pending {
            item: PendingItem::Device {
                device: dac(),
                from: base(),
                to: DeviceSettings {
                    sample_rate: 44_100,
                    ..base()
                },
            },
            causes: Vec::new(),
            failure: Some("44100 Hz refused".into()),
        };
        assert_eq!(
            describe_failure(&t, &names, &pending).as_deref(),
            Some("dac did not take sample rate 48 → 44.1 kHz: 44100 Hz refused. Still 48 kHz, buffer 512.")
        );
    }
}
```

Add `pub mod pending;` to the module list in `crates/fp-app/src/ui.rs`.

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test -p fp-app --lib pending`
Expected: FAIL to compile: `cannot find function device_changes in this scope`.

- [ ] **Step 4: Write the implementation**

Insert above the `#[cfg(test)]` module of `crates/fp-app/src/ui/pending.rs`:

```rust
use fp_model::{
    AppState, BusyCause, CartId, CartPageId, DeviceSettings, DsdOutput, Holder, OutputDevice,
    Pending, PendingItem, PlayerId, Route, Target,
};

use crate::i18n::I18n;

/// Names a device as the Settings pickers do; an unknown id as is.
pub(crate) type DeviceNames<'a> = &'a dyn Fn(&OutputDevice) -> String;

fn player_number(state: &AppState, id: PlayerId) -> usize {
    state
        .players
        .iter()
        .position(|p| p.id == id)
        .map_or(0, |i| i + 1)
}

fn holder_label(t: &I18n, state: &AppState, holder: Holder) -> String {
    match holder {
        Holder::PlayerMain(p) => {
            t.tr_args("holder-player-main", &[("n", player_number(state, p).into())])
        }
        Holder::PlayerCue(p) => {
            t.tr_args("holder-player-cue", &[("n", player_number(state, p).into())])
        }
        Holder::CartwallMain => t.tr("holder-cartwall-main"),
        Holder::CartwallCue => t.tr("holder-cartwall-cue"),
    }
}

fn route_label(t: &I18n, holder: Holder, route: Option<&Route>, names: DeviceNames<'_>) -> String {
    match route {
        Some(r) => names(&OutputDevice {
            backend: r.backend.clone(),
            device: r.device.clone(),
        }),
        // Main without a route plays on the default output; Cue has none.
        None if matches!(holder, Holder::PlayerMain(_) | Holder::CartwallMain) => {
            t.tr("settings-default-device")
        }
        None => t.tr("settings-none"),
    }
}

fn audio_system_label(t: &I18n, backend: Option<&str>) -> String {
    match backend {
        None => t.tr("settings-default-backend"),
        Some("null") => t.tr("settings-backend-null"),
        Some(id) => fp_backends::display_name(id).to_owned(),
    }
}

/// A rate in kHz as the pickers show it: 48, 44.1, 88.2.
fn khz(rate: u32) -> String {
    if rate % 1000 == 0 {
        (rate / 1000).to_string()
    } else {
        format!("{}", f64::from(rate) / 1000.0)
    }
}

fn dsd_mode_label(t: &I18n, mode: DsdOutput) -> String {
    t.tr(match mode {
        DsdOutput::Pcm => "settings-dsd-pcm",
        DsdOutput::Dop => "settings-dsd-dop",
        DsdOutput::Native => "settings-dsd-native",
    })
}

/// What changes between two device settings, as a list ("sample rate 48 →
/// 96 kHz, buffer 512 → 256"). The DSD mix and silence count only while
/// the device carries DSD on both sides (L4).
pub(crate) fn device_changes(t: &I18n, from: &DeviceSettings, to: &DeviceSettings) -> String {
    let mut parts = Vec::new();
    if from.sample_rate != to.sample_rate {
        parts.push(t.tr_args(
            "pending-change-rate",
            &[
                ("from", khz(from.sample_rate).into()),
                ("to", khz(to.sample_rate).into()),
            ],
        ));
    }
    if from.buffer_frames != to.buffer_frames {
        parts.push(t.tr_args(
            "pending-change-buffer",
            &[
                ("from", from.buffer_frames.to_string().into()),
                ("to", to.buffer_frames.to_string().into()),
            ],
        ));
    }
    if from.bit_perfect != to.bit_perfect {
        parts.push(t.tr(if to.bit_perfect {
            "pending-change-bit-perfect-on"
        } else {
            "pending-change-bit-perfect-off"
        }));
    }
    if from.dsd != to.dsd {
        parts.push(t.tr_args(
            "pending-change-dsd",
            &[("mode", dsd_mode_label(t, to.dsd).into())],
        ));
    }
    if let (Some(a), Some(b)) = (from.dsd_mix, to.dsd_mix)
        && a != b
    {
        parts.push(t.tr("pending-change-dsd-mix"));
    }
    if let (Some(a), Some(b)) = (from.dsd_silence_ms, to.dsd_silence_ms)
        && a.to_bits() != b.to_bits()
    {
        parts.push(t.tr_args(
            "pending-change-dsd-silence",
            &[
                ("from", format!("{a:.0}").into()),
                ("to", format!("{b:.0}").into()),
            ],
        ));
    }
    parts.join(", ")
}

/// A device's running settings, for "Still …".
fn device_values(t: &I18n, s: &DeviceSettings) -> String {
    t.tr_args(
        "pending-running",
        &[
            ("rate", khz(s.sample_rate).into()),
            ("buffer", s.buffer_frames.to_string().into()),
        ],
    )
}

/// A page's name as the cartwall shows it ("Carts n" while it has none).
fn page_label(t: &I18n, state: &AppState, page: CartPageId) -> String {
    match state.cartwall.pages.iter().enumerate().find(|(_, p)| p.id == page) {
        Some((_, p)) if !p.name.trim().is_empty() => p.name.clone(),
        Some((i, _)) => t.tr_args("cart-page-default", &[("n", (i + 1).into())]),
        None => String::new(),
    }
}

/// A cart's name as the close guard gives it: its name, else its track's
/// title, else "Cart n" (its position on its page).
fn cart_label(t: &I18n, state: &AppState, id: CartId) -> String {
    let found = state
        .cartwall
        .pages
        .iter()
        .find_map(|page| page.carts.iter().enumerate().find(|(_, c)| c.id == id));
    let Some((i, cart)) = found else {
        return String::new();
    };
    Some(cart.name.clone())
        .filter(|n| !n.trim().is_empty())
        .or_else(|| {
            cart.track
                .and_then(|track| state.library.get(track))
                .map(|track| track.title.clone())
        })
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| t.tr_args("on-air-cart-empty", &[("n", (i + 1).into())]))
}

/// One pending item (§9.2).
pub(crate) fn describe_item(
    t: &I18n,
    state: &AppState,
    names: DeviceNames<'_>,
    item: &PendingItem,
) -> String {
    match item {
        PendingItem::AudioSystem { from, to } => t.tr_args(
            "pending-audio-system",
            &[
                ("from", audio_system_label(t, from.as_deref()).into()),
                ("to", audio_system_label(t, to.as_deref()).into()),
            ],
        ),
        PendingItem::Route { holder, to, .. } => t.tr_args(
            "pending-route",
            &[
                ("holder", holder_label(t, state, *holder).into()),
                ("device", route_label(t, *holder, to.as_ref(), names).into()),
            ],
        ),
        PendingItem::Device { device, from, to } => t.tr_args(
            "pending-device",
            &[
                ("device", names(device).into()),
                ("changes", device_changes(t, from, to).into()),
            ],
        ),
        PendingItem::Players { from, to } => t.tr_args(
            "pending-players",
            &[("from", (*from).into()), ("to", (*to).into())],
        ),
        PendingItem::CartPage { page, from, to } => t.tr_args(
            "pending-cart-page",
            &[
                ("name", page_label(t, state, *page).into()),
                ("rows", from.0.into()),
                ("cols", from.1.into()),
                ("r", to.0.into()),
                ("c", to.1.into()),
            ],
        ),
    }
}

/// One cause (§9.2).
pub(crate) fn describe_cause(t: &I18n, state: &AppState, cause: &BusyCause) -> String {
    let player = |key: &str, p: PlayerId| t.tr_args(key, &[("n", player_number(state, p).into())]);
    match *cause {
        BusyCause::PlayerPlaying(p) => player("cause-player-playing", p),
        BusyCause::PlayerFading(p) => player("cause-player-fading", p),
        BusyCause::PlayerCue(p) => player("cause-player-cue", p),
        BusyCause::PlayerPaused(p) => player("cause-player-paused", p),
        BusyCause::CartPlaying(c) => {
            t.tr_args("cause-cart-playing", &[("name", cart_label(t, state, c).into())])
        }
        BusyCause::CartCue(c) => {
            t.tr_args("cause-cart-cue", &[("name", cart_label(t, state, c).into())])
        }
        BusyCause::CartsBeyondGrid { count, .. } => {
            t.tr_args("cause-carts-beyond-grid", &[("count", count.into())])
        }
    }
}

fn causes_list(t: &I18n, state: &AppState, causes: &[BusyCause]) -> String {
    causes
        .iter()
        .map(|c| describe_cause(t, state, c))
        .collect::<Vec<_>>()
        .join(", ")
}

/// An item and, when it waits, what for.
pub(crate) fn describe(t: &I18n, state: &AppState, names: DeviceNames<'_>, p: &Pending) -> String {
    let item = describe_item(t, state, names, &p.item);
    if p.causes.is_empty() {
        return item;
    }
    let waiting = t.tr_args(
        "pending-waiting",
        &[("causes", causes_list(t, state, &p.causes).into())],
    );
    format!("{item} — {waiting}")
}

/// Why the engine refused an item's value. Only a device can refuse (a
/// route or the audio system always applies).
pub(crate) fn describe_failure(t: &I18n, names: DeviceNames<'_>, p: &Pending) -> Option<String> {
    let reason = p.failure.as_ref()?;
    let PendingItem::Device { device, from, to } = &p.item else {
        return None;
    };
    Some(t.tr_args(
        "pending-failed",
        &[
            ("device", names(device).into()),
            ("what", device_changes(t, from, to).into()),
            ("reason", reason.clone().into()),
            ("running", device_values(t, from).into()),
        ],
    ))
}

/// What an Apply now interruption is about: a device, a player's or the
/// cartwall's output, or the audio system.
pub(crate) fn target_label(
    t: &I18n,
    state: &AppState,
    names: DeviceNames<'_>,
    target: &Target,
) -> String {
    match target {
        Target::AudioSystem => t.tr("settings-backend"),
        Target::Route(holder) => holder_label(t, state, *holder),
        Target::Device(device) => names(device),
    }
}
```

Check `i18n::Arg` has `From<u16>` and `From<String>` (it does, `crates/fp-app/src/i18n.rs:148,156`).

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p fp-app --lib pending && cargo test -p fp-app --test i18n`
Expected: PASS. `pending.rs` functions are not used outside the tests yet: if clippy reports `dead_code` in this task, add `#![allow(dead_code)] // Used by the interface from the next task on.` at the top of `pending.rs` and remove it in Task 2.

- [ ] **Step 6: Run the gate and commit**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`

```bash
git add crates/fp-app/src/ui/pending.rs crates/fp-app/src/ui.rs crates/fp-app/locales
git commit -m "$(cat <<'EOF'
feat(ui): describe pending output changes and what they wait for

Text for each pending item, cause, refusal and Apply now interruption
(live settings spec §9.2), in every locale.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
EOF
)"
```

---

### Task 2: Settings pending and Apply now replace the restart in the interface (§9.1, §9.3, §9.4)

**Files:**
- Modify: `crates/fp-app/src/ui/pending.rs` (modals)
- Modify: `crates/fp-app/src/ui/app.rs` (imports L3-38; `ViewState` L119-124; `AppUi` fields L345, L380-383 and `new` L406-450; `with_started_config`, `restart_flag`, `with_platform` L452-488; frame L662-905; `keyboard` L1105-1126; `begin_restart`, `restart_reason_key` L1520-1539; `top_bar` L1541-1645; `status_bar` L1705-1768)
- Modify: `crates/fp-app/src/ui/settings.rs` (`SettingsDeps` L174-185, `Outcome` L188-193, footer L326-366, `SettingsState` new method)
- Modify: `crates/fp-app/src/ui/settings/devices.rs:19` (`find` → `pub(super)`)
- Modify: `crates/fp-app/src/ui/exit_guard.rs` (`ExitIntent::Restart`, `button` → `pub(crate)`)
- Modify: `crates/fp-app/src/main.rs` (platform label)
- Delete: `crates/fp-app/tests/restart_ui.rs`
- Create: `crates/fp-app/tests/live_settings.rs`
- Modify: `crates/fp-app/locales/*/main.ftl` (remove `top-restart-pending`, `tip-restart-pending`, `settings-restart-pending`, `settings-restart-now`, `restart-reason-*` (8), `exit-guard-restart-body`, `exit-guard-restart-confirm`)
- Modify: `README.md:41-43`, `docs/user/settings.md:16-50`, `docs/user/getting-started.md:96-97`, `docs/user/bit-perfect.md:17,84,205`, `docs/user/troubleshooting.md:7-8,121`, `docs/technical/ui.md:309-346`

**Interfaces:**
- Consumes: Task 1; `fp_model::{pending, interruptions, has_output_items, Command::ApplySettingsNow}`; `state.live.audio_system_in_use`.
- Produces:
  - `pending::Answer { Close, ApplyNow }`; `pending::show_panel(ctx, scene, names) -> Option<Answer>`; `pending::show_confirm(ctx, scene, names) -> Option<bool>`
  - `SettingsState::device_label(&self, device: &OutputDevice) -> String`
  - `SettingsDeps { …, pending: bool, can_apply: bool, pending_tip: String }` (replaces `restart_pending`); `Outcome { open, apply_now }` (replaces `restart`)
  - `ViewState::{pending_open, apply_now_requested, confirm_apply_now}` (replace `restart_requested`)
  - `exit_guard::button` is `pub(crate)`; `ExitIntent` has only `Close`
  - Removed: `AppUi::{with_started_config, restart_flag, with_platform}`

- [ ] **Step 1: Write the failing tests**

Delete `crates/fp-app/tests/restart_ui.rs` (`git rm`). Create `crates/fp-app/tests/live_settings.rs`:

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Live settings spec §9: the Settings pending pill and panel, the
//! Settings footer, Apply now and its confirmation, the status bar label.

mod support;

use egui::Key;
use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_app::ui::controller::Controller;
use fp_model::{AppState, Command, EngineEvent, Holder, OutputDevice, Route, device_settings};
use support::{Fake, harness, state};

const PILL: &str = "Settings pending";
const NOTICE: &str = "Some changes wait until the outputs they affect are free.";
const CONFIRM: &str = "Apply now?";

fn null(device: &str) -> OutputDevice {
    OutputDevice {
        backend: "null".into(),
        device: device.into(),
    }
}

/// `state(players, 3)` as the engine reports it at start: every Main on
/// the null output, every Cue on `phones`.
fn reported(players: usize) -> AppState {
    let mut s = state(players, 3);
    fp_model::on_event(
        &mut s,
        EngineEvent::AudioSystemInUse {
            configured: None,
            in_use: "null".into(),
        },
    );
    for p in s.players.clone() {
        let main = null("null");
        let running = device_settings(&s.config.outputs, &main);
        fp_model::on_event(
            &mut s,
            EngineEvent::Placed {
                holder: Holder::PlayerMain(p.id),
                route: None,
                device: main,
                running,
            },
        );
        let phones = null("phones");
        let running = device_settings(&s.config.outputs, &phones);
        let route = Some(Route {
            backend: "null".into(),
            device: "phones".into(),
            first_channel: 0,
        });
        fp_model::on_event(
            &mut s,
            EngineEvent::Placed {
                holder: Holder::PlayerCue(p.id),
                route,
                device: phones,
                running,
            },
        );
    }
    s
}

fn playing(players: usize) -> AppState {
    let mut s = reported(players);
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    s
}

/// A new global buffer, through the model as Settings sends it.
fn set_buffer(h: &mut Harness<'static, AppUi>, fake: &Fake, frames: u32) {
    let mut config = fake.state.load().config.clone();
    config.outputs.buffer_frames = frames;
    fake.send(Command::UpdateConfig(Box::new(config)));
    fake.take_sent();
    h.run_steps(2);
}

fn open_settings_and_apply(h: &mut Harness<'static, AppUi>) {
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_label("Apply now").click();
    h.run_steps(2);
}

#[test]
fn nothing_waiting_shows_no_pill_and_settings_applies_at_once() {
    let (mut h, _fake) = harness(reported(1));
    assert!(h.query_by_label(PILL).is_none());
    h.get_by_label("Settings").click();
    h.run_steps(2);
    assert!(h.query_by_label(NOTICE).is_none());
    assert!(h.query_by_label("Apply now").is_none());
}

#[test]
fn a_waiting_change_shows_the_pill_and_the_panel_lists_it_with_its_cause() {
    let (mut h, fake) = harness(playing(1));
    set_buffer(&mut h, &fake, 1024);
    assert!(h.query_by_role_and_label(Role::Button, PILL).is_some());
    h.get_by_label(PILL).click();
    h.run_steps(2);
    assert!(
        h.query_by_label("null: buffer 512 → 1024 — waiting for P1 playing")
            .is_some()
    );
}

#[test]
fn apply_now_with_something_on_air_asks_first_and_cancel_sends_nothing() {
    let (mut h, fake) = harness(playing(1));
    set_buffer(&mut h, &fake, 1024);
    h.get_by_label("Settings").click();
    h.run_steps(2);
    assert!(h.query_by_label(NOTICE).is_some());
    h.get_by_label("Apply now").click();
    h.run_steps(2);
    assert!(h.query_by_label(CONFIRM).is_some());
    assert!(h.query_by_label("null: P1 playing").is_some());
    h.get_by_label("Cancel").click();
    h.run_steps(2);
    assert!(h.query_by_label(CONFIRM).is_none());
    assert!(fake.take_sent().is_empty());
}

#[test]
fn interrupt_and_apply_sends_apply_settings_now() {
    let (mut h, fake) = harness(playing(1));
    set_buffer(&mut h, &fake, 1024);
    open_settings_and_apply(&mut h);
    h.get_by_label("Interrupt and apply").click();
    h.run_steps(2);
    assert_eq!(fake.take_sent(), vec![Command::ApplySettingsNow]);
    assert!(h.query_by_label(CONFIRM).is_none());
}

#[test]
fn apply_now_with_nothing_on_air_applies_at_once() {
    let (mut h, fake) = harness(reported(1));
    set_buffer(&mut h, &fake, 1024);
    open_settings_and_apply(&mut h);
    assert!(h.query_by_label(CONFIRM).is_none());
    assert_eq!(fake.take_sent(), vec![Command::ApplySettingsNow]);
}

#[test]
fn the_confirmation_applies_by_itself_when_what_it_would_cut_ends() {
    let (mut h, fake) = harness(playing(1));
    set_buffer(&mut h, &fake, 1024);
    open_settings_and_apply(&mut h);
    assert!(h.query_by_label(CONFIRM).is_some());
    let p = fake.player(0);
    fake.send(Command::Stop(p));
    fake.take_sent();
    h.run_steps(2);
    assert!(h.query_by_label(CONFIRM).is_none());
    assert_eq!(fake.take_sent(), vec![Command::ApplySettingsNow]);
}

#[test]
fn esc_cancels_the_confirmation_and_keeps_settings_open() {
    let (mut h, fake) = harness(playing(1));
    set_buffer(&mut h, &fake, 1024);
    open_settings_and_apply(&mut h);
    h.key_press(Key::Escape);
    h.run_steps(2);
    assert!(h.query_by_label(CONFIRM).is_none());
    assert!(h.query_by_label(NOTICE).is_some(), "Settings is still open");
    assert!(fake.take_sent().is_empty());
}

#[test]
fn limit_items_alone_offer_no_apply_now() {
    let mut s = reported(4);
    let last = s.players[3].id;
    fp_model::apply(&mut s, Command::Play(last)).unwrap();
    let mut config = s.config.clone();
    config.limits.max_players = 3;
    fp_model::apply(&mut s, Command::UpdateConfig(Box::new(config))).unwrap();
    let (mut h, _fake) = harness(s);
    assert!(h.query_by_role_and_label(Role::Button, PILL).is_some());
    h.get_by_label(PILL).click();
    h.run_steps(2);
    assert!(h.query_by_label("Players: 4 → 3 — waiting for P4 playing").is_some());
    assert!(h.query_by_label("Apply now").is_none());
}

#[test]
fn the_status_bar_names_the_audio_system_in_use() {
    let (h, _fake) = harness(reported(1));
    let os = match std::env::consts::OS {
        "linux" => "Linux",
        "windows" => "Windows",
        "macos" => "macOS",
        other => other,
    };
    assert!(
        h.query_by_label(&format!("{os} · No output (silent)"))
            .is_some()
    );
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-app --test live_settings`
Expected: FAIL: no "Settings pending" pill (the top bar still shows the restart pill), no "Apply now".

- [ ] **Step 3: Write the implementation**

**`crates/fp-app/src/ui/exit_guard.rs`:** remove the `Restart` variant (and its doc) from `ExitIntent`; in `show`, replace the `let (body, confirm) = match intent { … };` with `let (body, confirm) = match intent { ExitIntent::Close => ("exit-guard-close-body", "exit-guard-close-confirm"), };`; make `fn button` `pub(crate) fn button`.

**`crates/fp-app/src/ui/settings/devices.rs:19`:** `fn find<'a>(` → `pub(super) fn find<'a>(`.

**`crates/fp-app/src/ui/settings.rs`:**
- `SettingsDeps`: replace `restart_pending: bool` (and its doc) with

```rust
    /// An output change waits (live settings spec §9.1).
    pub pending: bool,
    /// One of them is an output change Apply now can apply.
    pub can_apply: bool,
    /// What waits, one item per line (Apply now's tooltip).
    pub pending_tip: String,
```

- `Outcome`: replace `restart: bool` with `/// The operator pressed Apply now.\n    pub apply_now: bool,`.
- In `show`, rename the local `restart` flag to `apply_now`; the footer block becomes:

```rust
                    if deps.can_apply {
                        let label = t.tr("settings-apply-now");
                        let width = ui
                            .painter()
                            .layout_no_wrap(label.clone(), font_medium(13.0), theme::TEXT)
                            .size()
                            .x
                            + 28.0;
                        let style = TileStyle {
                            border: theme::AMBER,
                            ..TileStyle::plain()
                        };
                        if widgets::tile(ui, vec2(width, 30.0), &label, true, style, |p, r, c| {
                            p.text(
                                r.center(),
                                egui::Align2::CENTER_CENTER,
                                &label,
                                font_medium(13.0),
                                c,
                            );
                        })
                        .on_hover_text(&deps.pending_tip)
                        .clicked()
                        {
                            apply_now = true;
                        }
                    }
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                        ui.add_space(16.0);
                        let (text, color) = match (&deps.notice, deps.pending) {
                            (Some(n), _) => (n.clone(), theme::AMBER),
                            (None, true) => (t.tr("settings-pending"), theme::AMBER),
                            (None, false) => (t.tr("settings-applies-now"), theme::NEUTRAL_500),
                        };
                        ui.add(
                            egui::Label::new(RichText::new(text).font(font(11.0)).color(color))
                                .selectable(false)
                                .truncate(),
                        );
                    });
```

- Remove `if restart { remote::flush(scene, &mut st.remote); }` (the window stays open on Apply now) and return `Outcome { open, apply_now }`.
- In `impl SettingsState`, add:

```rust
    /// The name the device pickers give `device`, once the device lists
    /// have been read (Settings opened once); its id until then. Nothing
    /// is enumerated for this (rule 8).
    pub(crate) fn device_label(&self, device: &fp_model::OutputDevice) -> String {
        match &self.backends {
            Some(backends) => devices::find(backends, device).1,
            None => device.device.clone(),
        }
    }
```

**`crates/fp-app/src/ui/pending.rs`** (remove any `#![allow(dead_code)]` from Task 1), add the imports `use egui::{RichText, vec2};`, `use super::app::Scene;`, `use super::exit_guard::button;`, `use super::theme;`, `use super::widgets::{font, font_medium};`, and:

```rust
/// What the operator chose in the pending panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Answer {
    Close,
    ApplyNow,
}

fn text(ui: &mut egui::Ui, text: String, size: f32, color: egui::Color32) {
    ui.add(
        egui::Label::new(RichText::new(text).font(font(size)).color(color))
            .selectable(false)
            .wrap(),
    );
}

/// The pending panel (§9.1): every item, what it waits for, and any
/// refusal; Apply now while an output change is pending. `None` keeps it
/// open; Esc is answered by `AppUi::keyboard`, a click on the backdrop
/// closes it.
pub(crate) fn show_panel(
    ctx: &egui::Context,
    scene: &Scene<'_>,
    names: DeviceNames<'_>,
) -> Option<Answer> {
    let (t, state) = (scene.i18n, scene.state);
    let items = fp_model::pending(state);
    let width = (ctx.content_rect().width() - 48.0).clamp(280.0, 520.0);
    let mut answer = None;
    let modal = egui::Modal::new(egui::Id::new("settings-pending"))
        .frame(egui::Frame::new().fill(theme::SURFACE).inner_margin(20.0))
        .backdrop_color(theme::NEUTRAL_900.gamma_multiply(0.7))
        .show(ctx, |ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            ui.add(
                egui::Label::new(
                    RichText::new(t.tr("top-settings-pending"))
                        .font(font_medium(18.0))
                        .color(theme::TEXT),
                )
                .selectable(false),
            );
            text(ui, t.tr("settings-pending"), 12.0, theme::NEUTRAL_300);
            for p in &items {
                text(ui, describe(t, state, names, p), 12.0, theme::TEXT);
                if let Some(failure) = describe_failure(t, names, p) {
                    text(ui, failure, 11.0, theme::AMBER);
                }
            }
            if items
                .iter()
                .any(|p| matches!(p.item, PendingItem::CartPage { .. }))
            {
                text(ui, t.tr("pending-cart-grid-on-load"), 11.0, theme::NEUTRAL_400);
            }
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if button(ui, &t.tr("settings-close"), None) {
                    answer = Some(Answer::Close);
                }
                if fp_model::has_output_items(state)
                    && button(ui, &t.tr("settings-apply-now"), Some(theme::AMBER))
                {
                    answer = Some(Answer::ApplyNow);
                }
            });
        });
    if answer.is_none() && modal.should_close() {
        answer = Some(Answer::Close);
    }
    answer
}

/// Apply now's confirmation (§9.3): what it briefly interrupts, and that
/// limits keep waiting. `Some(true)` interrupts and applies; `Some(false)`
/// cancels (the button or the backdrop; Esc is answered by
/// `AppUi::keyboard`).
pub(crate) fn show_confirm(
    ctx: &egui::Context,
    scene: &Scene<'_>,
    names: DeviceNames<'_>,
) -> Option<bool> {
    let (t, state) = (scene.i18n, scene.state);
    let interruptions = fp_model::interruptions(state);
    let limits_wait = fp_model::pending(state)
        .iter()
        .any(|p| p.item.target().is_none());
    let width = (ctx.content_rect().width() - 48.0).clamp(280.0, 440.0);
    let mut answer = None;
    let modal = egui::Modal::new(egui::Id::new("apply-now"))
        .frame(egui::Frame::new().fill(theme::SURFACE).inner_margin(20.0))
        .backdrop_color(theme::NEUTRAL_900.gamma_multiply(0.7))
        .show(ctx, |ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            ui.add(
                egui::Label::new(
                    RichText::new(t.tr("apply-now-title"))
                        .font(font_medium(18.0))
                        .color(theme::TEXT),
                )
                .selectable(false),
            );
            text(ui, t.tr("apply-now-body"), 12.0, theme::NEUTRAL_300);
            for (target, causes) in &interruptions {
                let line = t.tr_args(
                    "apply-now-line",
                    &[
                        ("device", target_label(t, state, names, target).into()),
                        ("causes", causes_list(t, state, causes).into()),
                    ],
                );
                text(ui, line, 12.0, theme::TEXT);
            }
            if limits_wait {
                text(ui, t.tr("apply-now-limits"), 11.0, theme::NEUTRAL_400);
            }
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if button(ui, &t.tr("exit-guard-cancel"), None) {
                    answer = Some(false);
                }
                if button(ui, &t.tr("apply-now-confirm"), Some(theme::ON_AIR_ROW)) {
                    answer = Some(true);
                }
            });
        });
    if answer.is_none() && modal.should_close() {
        answer = Some(false);
    }
    answer
}
```


**`crates/fp-app/src/ui/app.rs`:**
- Imports: drop `RestartReason` from the `fp_model` list; add `super::pending` to the `super::…` imports; drop `AtomicBool` (and `Ordering` if the compiler reports it unused) from `use std::sync::atomic::{…}`.
- `ViewState`: replace `restart_requested` (L123-124) with

```rust
    /// The pending panel is open (live settings spec §9.1).
    pub pending_open: bool,
    /// Apply now was pressed (Settings footer or the pending panel).
    pub apply_now_requested: bool,
    /// Apply now's confirmation is open (§9.3).
    pub confirm_apply_now: bool,
```

- `AppUi`: remove the fields `platform` (L345), `started` and `restart` (L379-383) and their initialisers in `new` (`let started = …`, `platform: default_platform(),`, `started,`, `restart: Arc::new(AtomicBool::new(false)),`); remove the methods `with_started_config`, `restart_flag` and `with_platform`.
- Add this free function next to `default_platform`:

```rust
/// The status bar's label: the OS and the audio system the engine uses
/// (live settings spec §9.4); the OS alone until the engine has said.
fn platform_label(t: &I18n, state: &AppState) -> String {
    let os = default_platform();
    match state.live.audio_system_in_use.as_deref() {
        None => os,
        Some("null") => format!("{os} · {}", t.tr("settings-backend-null")),
        Some(id) => format!("{os} · {}", fp_backends::display_name(id)),
    }
}

```

- `status_bar`: drop the `platform: &str` parameter; inside, `RichText::new(platform)` becomes `RichText::new(platform_label(t, scene.state))`; the call site becomes `status_bar(&mut status_ui, &scene, &self.view, faults);`.
- In the frame (L662): `let pending = fp_model::pending(&state);` replaces the `restart_pending` line. The top bar call becomes

```rust
        top_bar(
            &mut top_ui,
            &scene,
            &mut self.view,
            &pending,
            &|d| self.settings.device_label(d),
        );
```

- `SettingsDeps { … }` (L746-760): replace `restart_pending: !pending.is_empty(),` with

```rust
                pending: !pending.is_empty(),
                can_apply: fp_model::has_output_items(&state),
                pending_tip: {
                    let names = |d: &fp_model::OutputDevice| self.settings.device_label(d);
                    pending
                        .iter()
                        .map(|p| pending::describe(&self.i18n, &state, &names, p))
                        .collect::<Vec<_>>()
                        .join("\n")
                },
```

  (compute this `pending_tip` into a local before `let deps = SettingsDeps { … }` if the borrow checker objects to borrowing `self.settings` while `settings::show` later borrows it mutably), and after `settings::show` replace `self.view.restart_requested |= outcome.restart;` with `self.view.apply_now_requested |= outcome.apply_now;`.
- Replace the whole "O4: Restart now asks the close guard first…" block (L872-879) with:

```rust
        // Live settings §9.3: Apply now sends at once when it interrupts
        // nothing; otherwise it asks first.
        if std::mem::take(&mut self.view.apply_now_requested) {
            if fp_model::interruptions(&state).is_empty() {
                scene.ctl.send(Command::ApplySettingsNow);
            } else {
                self.view.confirm_apply_now = true;
            }
        }
        if self.view.pending_open && !self.view.confirm_apply_now {
            if pending.is_empty() {
                self.view.pending_open = false;
            } else {
                match pending::show_panel(&ctx, &scene, &|d| self.settings.device_label(d)) {
                    Some(pending::Answer::Close) => self.view.pending_open = false,
                    Some(pending::Answer::ApplyNow) => self.view.apply_now_requested = true,
                    None => {}
                }
            }
        }
        if self.view.confirm_apply_now {
            if fp_model::interruptions(&state).is_empty() {
                // What it would interrupt ended meanwhile: nothing left to
                // confirm (as the close guard does), the request stands.
                self.view.confirm_apply_now = false;
                scene.ctl.send(Command::ApplySettingsNow);
            } else {
                match pending::show_confirm(&ctx, &scene, &|d| self.settings.device_label(d)) {
                    Some(true) => {
                        self.view.confirm_apply_now = false;
                        scene.ctl.send(Command::ApplySettingsNow);
                    }
                    Some(false) => self.view.confirm_apply_now = false,
                    None => {}
                }
            }
        }
```

- In the close guard block, the `match intent { ExitIntent::Close => { … } ExitIntent::Restart => { … } }` keeps only the `Close` arm.
- `keyboard`: after the close-guard block (L1110-1126), add

```rust
        // Apply now's confirmation, then the pending panel, own the keyboard
        // the same way: Esc cancels the top one (Cancel is the default). The
        // key is consumed, so Settings under the confirmation does not close
        // on the same press.
        if self.view.confirm_apply_now || self.view.pending_open {
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape)) {
                if self.view.confirm_apply_now {
                    self.view.confirm_apply_now = false;
                } else {
                    self.view.pending_open = false;
                }
            }
            return;
        }
```
- Delete `begin_restart` and `restart_reason_key` (L1520-1539).
- `top_bar`: change the signature to `fn top_bar(ui: &mut Ui, scene: &Scene<'_>, view_state: &mut ViewState, pending: &[fp_model::Pending], names: pending::DeviceNames<'_>)` and replace the `if !pending.is_empty() { … }` block with:

```rust
        if !pending.is_empty() {
            let i18n = scene.i18n;
            let label = i18n.tr("top-settings-pending");
            let tip = pending
                .iter()
                .map(|p| pending::describe(i18n, scene.state, names, p))
                .collect::<Vec<_>>()
                .join("\n");
            let width = ui
                .painter()
                .layout_no_wrap(label.clone(), font(12.0), theme::AMBER)
                .size()
                .x
                + 34.0;
            let style = TileStyle {
                border: theme::AMBER,
                content: theme::AMBER,
                hover_fill: theme::NEUTRAL_800,
                ..TileStyle::plain()
            };
            if widgets::tile(ui, vec2(width, 24.0), &label, true, style, |p, r, c| {
                p.text(
                    r.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("{} {label}", egui_phosphor::regular::HOURGLASS),
                    font(12.0),
                    c,
                );
            })
            .on_hover_text(tip)
            .clicked()
            {
                view_state.pending_open = true;
            }
        }
```

  (If `HOURGLASS` is not in the `egui_phosphor` version in `Cargo.lock`, use `egui_phosphor::regular::CLOCK`.)

**`crates/fp-app/src/main.rs`:** remove `.with_platform(platform)` and the `output`/`platform` locals (L189-194), the `os_name` function (L333-340) and `display_name` from the `fp_backends` import; `in_use` stays for the log line.

**Locales:** remove from all eleven `main.ftl` files the lines with the keys `top-restart-pending`, `tip-restart-pending`, `settings-restart-pending`, `settings-restart-now`, `restart-reason-audio-system`, `restart-reason-sample-rate`, `restart-reason-buffer-size`, `restart-reason-routes`, `restart-reason-bit-perfect`, `restart-reason-dsd`, `restart-reason-limits`, `restart-reason-tuning`, `exit-guard-restart-body`, `exit-guard-restart-confirm` (keep `restart-failed` until Task 3).

**Docs:**

`README.md` L41-43, replace the bullet with:

```markdown
- **Restore defaults:** Players, Meters, Analysis and Shortcuts can each be
  reset to their defaults.
- **Live settings:** every setting applies while the application runs. A
  change to the outputs waits until nothing plays on what it affects (a
  **Settings pending** pill says what waits and on what), or applies at once
  with **Apply now**, after a confirmation when it would briefly interrupt
  the audio.
```

`docs/user/settings.md`: replace the section "## Restart pending" (from its heading to the line before "## Audio outputs") with:

```markdown
## Settings pending

Every setting applies while the application runs. The audio system, the
sample rate, the buffer size (also a device's own), the Main and Cue outputs
(players and cartwall), the bit-perfect devices and the DSD settings change
how an output device is opened, so they apply as soon as that is safe:

- A device's settings apply when nothing plays on that device: no player
  routed to it is playing, fading or pre-listening (CUE, also when held),
  and no cart plays on it. A paused player, or one with a track only loaded,
  does not hold a device: its track stays paused where it was.
- A player's or the cartwall's output moves when that player (or the
  cartwall) is not playing; other players on the same devices are not
  interrupted.
- A new audio system applies when nothing plays anywhere.

A rate or buffer given to a device counts only when it changes what the
device opens with: giving a device the same value as the global one, or
clearing such a value, changes nothing.

Until then the top bar shows **Settings pending**. Hover it to see what
waits and on what; click it for the list, which also shows any change a
device refused (the device keeps its current settings; the change is tried
again when you change it or press **Apply now**). The Settings footer says
"Some changes wait until the outputs they affect are free." and offers
**Apply now**.

**Apply now** applies every waiting output change at once. When that would
interrupt something, it first lists the devices and what plays on them;
**Interrupt and apply** then briefly interrupts the audio on those devices:
what was playing goes on from where it was after a short gap, and nothing
paused or stopped starts. **Cancel** (or `Esc`) keeps waiting.

The limits and the engine tuning are edited in the configuration file with
the application closed (see [Data and backups](data-and-backups.md)).
```

  and in "## Audio outputs" replace "Changes in this section wait for a restart: see [Restart pending](#restart-pending)." with "Changes in this section apply while the application runs, as soon as nothing they affect is playing: see [Settings pending](#settings-pending)."

`docs/user/getting-started.md` L96-97: replace with "When a change to the outputs waits for something on air, a **Settings pending** pill appears in the top bar; it says what waits (see [Settings](settings.md#settings-pending))."

`docs/user/bit-perfect.md`: L17 "3. Restart the application." → "3. The device becomes bit-perfect as soon as nothing plays on it (see [Settings pending](settings.md#settings-pending))."; L83-84 "Changing a mode, the mixing setting or the DSD silence needs a restart, like the other output settings." → "Changing a mode, the mixing setting or the DSD silence applies once nothing plays on the device, like the other output settings."; L205 "Set the device to **DoP** (or **Native DSD** on Linux), restart, and play" → "Set the device to **DoP** (or **Native DSD** on Linux) and play".

`docs/user/troubleshooting.md`: L7-8 "Changes to outputs take effect after a restart: press **Restart now** in Settings." → "A change to the outputs applies once nothing plays on them; **Apply now** in Settings applies it at once."; L121 "(and press **Restart now**)" → "(it applies once nothing plays on that device, or at once with **Apply now**)".

`docs/technical/ui.md`: delete the paragraph that starts "`ExitIntent::Restart` opens the same modal" (L330-333); in "## Settings window", replace the sentences from "The footer spans the window width and holds the restart notice" to "**Restart now** stays." with "The footer spans the window width and holds the pending notice and **Apply now** while an output change waits (`SettingsDeps::{pending, can_apply}`). A transient notice (`SettingsDeps::notice`) briefly takes the place of the pending text; **Apply now** stays."; and add before "## Settings window":

```markdown
## Settings pending

`ui/pending.rs` turns `fp_model::live::pending` and `interruptions` into
text (`describe`, `describe_failure`, `target_label`; unit-tested) and draws
two modals in the close guard's style: the pending panel (`show_panel`,
opened from the top bar's **Settings pending** pill, whose tooltip lists the
same lines) and Apply now's confirmation (`show_confirm`). **Apply now**
(Settings footer or the panel) sets `ViewState::apply_now_requested`; the
next frame sends `Command::ApplySettingsNow` at once when `interruptions`
is empty, else sets `confirm_apply_now`. The confirmation closes and sends
the command by itself when the list becomes empty; Cancel, `Esc` (answered
in `keyboard`, before Settings sees it) and the backdrop cancel it. Device
names come from `SettingsState::device_label` (the lists Settings read on
its helper thread; the id until then). The status bar's label follows
`state.live.audio_system_in_use`.
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-app`
Expected: PASS (the new `live_settings` kittests, `i18n`, `exit_guard`, `settings`, every other UI test).

- [ ] **Step 5: Run the gate and commit**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`

```bash
git add -A crates/fp-app README.md docs/user docs/technical/ui.md
git commit -m "$(cat <<'EOF'
feat(ui): show pending settings and apply them now instead of restarting

The top bar's Settings pending pill and its panel say what waits and on
what; Apply now applies every output change, after a confirmation when
it would briefly interrupt the audio (live settings spec §9). The
restart pill and Restart now are gone.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
EOF
)"
```

---

### Task 3: The relaunch goes (§9.4)

**Files:**
- Modify: `crates/fp-app/src/main.rs` (`Exit`, `main`, `run`, `LOCK_ATTEMPTS` comment)
- Delete: `crates/fp-app/src/restart.rs`, `crates/fp-app/tests/relaunch.rs`
- Modify: `crates/fp-app/src/lib.rs:12` (`pub mod restart;` removed)
- Modify: `crates/fp-model/src/config.rs` (`Tuning::restart_handoff_ms` field L716-719, default L741, `validate` L1157-1163, test L1362-1378)
- Modify: `crates/fp-app/locales/*/main.ftl` (remove `restart-failed`)
- Test: `crates/fp-store/tests/store_roundtrip.rs`
- Modify: `docs/technical/architecture.md:60-87` (section "## Restart"), `docs/technical/persistence.md` (`tuning` table), the specs `2026-10-01-operator-feedback-2-design.md` (O4), `2026-10-06-operator-feedback-4-design.md` (Q12.6), `2026-09-25-fauste-player-design.md` (§8.4), `2026-09-26-phase4-bit-perfect-design.md` (B6)

**Interfaces:**
- Produces: `fn run(paths, playlists, report_cap) -> Result<(), Box<dyn std::error::Error>>` in `main.rs`; `Tuning` without `restart_handoff_ms`.

- [ ] **Step 1: Write the failing test**

Append to `crates/fp-store/tests/store_roundtrip.rs` (it already has `store` and `write_config` helpers):

```rust
#[test]
fn an_old_restart_handoff_is_ignored_on_load() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
    write_config(&s, r#"{"tuning":{"restart_handoff_ms":5000,"declick_ms":7}}"#);
    let loaded = s.load("Main");
    assert_eq!(loaded.state.config.tuning.declick_ms, 7.0, "the rest is kept");
    assert!(
        loaded
            .warnings
            .iter()
            .any(|w| w.contains("restart_handoff_ms")),
        "{:?}",
        loaded.warnings
    );
}
```

(If `loaded.warnings` holds a non-`String` type, compare its `to_string()`.)

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p fp-store --test store_roundtrip an_old_restart_handoff`
Expected: FAIL: no warning (the key is still a field of `Tuning`).

- [ ] **Step 3: Write the implementation**

`crates/fp-model/src/config.rs`: delete the `restart_handoff_ms` field and its doc comment from `Tuning`, its line in `Default for Tuning`, its `clamp_to(…"tuning.restart_handoff_ms"…)` in `validate`, and the unit test `the_restart_handoff_has_its_default_and_range`.

`crates/fp-app/src/main.rs`:
- Delete `enum Exit` and its doc comment.
- `main`: replace the `match result { … }` with

```rust
    match result {
        Ok(()) => {
            tracing::info!("stopped");
            ExitCode::SUCCESS
        }
        Err(e) => {
            tracing::error!(error = %e, "could not start");
            cli::emit(cli::Stream::Err, &format!("fauste-player: {e}\n"));
            ExitCode::FAILURE
        }
    }
```

  and delete `let data_dir = paths.data_dir.clone();` (only the relaunch used it).
- `run` returns `Result<(), Box<dyn std::error::Error>>`; delete `let restart = app.restart_flag();` (already gone with Task 2 if the compiler said so), the `final_config`/`handoff`/`language` lines after `eframe::run_native`, and the final `Ok(if restart.load(…) { … } else { … })`, ending with `result.map_err(|e| e.to_string())?;` and `Ok(())`.
- The `LOCK_ATTEMPTS` doc comment becomes "Tries at the instance lock, and the pause between them: an instance that is ending may hold it for an instant longer." (the retry itself stays).
- Update the doc comment of `stop_engine`: "Stops the conductor and the engine, closing the output streams, now. Every other holder of the handle must have stopped already."

Delete `crates/fp-app/src/restart.rs` and `crates/fp-app/tests/relaunch.rs` (`git rm`), and `pub mod restart;` from `crates/fp-app/src/lib.rs`. Remove the `restart-failed` line from all eleven locale files.

`docs/technical/architecture.md`: delete the whole "## Restart" section (from its heading to the line before "## Design principles").

`docs/technical/persistence.md`: delete the `restart_handoff_ms` row of the `tuning` table, and add under "### `tuning` (config file only)", before the table: "Every value is read while the application runs (live settings spec §7): a new value applies at its next use."

Specs (a short note each, the original text kept):
- `docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md`, right under the line `- **O4 Restart.**`: `  - *Replaced by the [live settings spec](2026-10-07-live-settings-design.md) (2026-10-07): output settings apply while the application runs, and the restart, its pill and the relaunch are removed.*`
- `docs/superpowers/specs/2026-10-06-operator-feedback-4-design.md`, at the end of the `**Q12.6**` item: ` *Replaced by the live settings spec (2026-10-07): an override applies while running, once its device is idle.*`
- `docs/superpowers/specs/2026-09-25-fauste-player-design.md`, §8.4, after the sentence that ends "…the restart goes through the on-air guard when something is on air.": ` *(Replaced by the live settings spec, 2026-10-07: these settings apply while running; a "Settings pending" pill and Apply now replace the restart.)*`
- `docs/superpowers/specs/2026-09-26-phase4-bit-perfect-design.md`, B6, after "Changes apply after a restart, like every output change.": ` *(Replaced by the live settings spec, 2026-10-07: they apply while running, once the device is idle.)*`

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-store && cargo test -p fp-model && cargo test -p fp-app --test i18n`
Expected: PASS.

- [ ] **Step 5: Run the gate and commit**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`

```bash
git add -A crates/ docs/technical docs/superpowers/specs
git commit -m "$(cat <<'EOF'
refactor(app)!: remove the application restart

Settings apply while running (live settings spec), so the relaunch,
its Flatpak hand-off and tuning.restart_handoff_ms go (Q2). An old
configuration with the key still loads; the key is ignored.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
EOF
)"
```

---

### Task 4: `RestartReason` and `restart_pending` go from the model

**Files:**
- Delete: `crates/fp-model/src/restart.rs`, `crates/fp-model/tests/restart.rs`
- Modify: `crates/fp-model/src/lib.rs` (`pub mod restart;`, `pub use restart::{RestartReason, restart_pending};`)
- Modify: `crates/fp-model/tests/device_overrides.rs:326`, `crates/fp-app/tests/settings.rs:759`
- Test: `crates/fp-model/tests/live.rs`

**Interfaces:**
- Removes `fp_model::{RestartReason, restart_pending}`; nothing else uses them after Task 2.

- [ ] **Step 1: Port the rules worth keeping as tests of L4**

Two rules of the old `tests/restart.rs` still hold, now as L4: a device's own value equal to the global one changes nothing, and an own value on a device no route names applies to nothing. Append to `crates/fp-model/tests/live.rs`:

```rust
#[test]
fn l4_an_own_value_equal_to_the_global_one_changes_nothing() {
    let mut c = Config::default();
    c.outputs.routes = vec![fp_model::PlayerRoutes {
        player: fp_model::PlayerId(1),
        main: Some(route_to("dac")),
        cue: None,
    }];
    let before = device_settings(&c.outputs, &dev("dac"));
    c.outputs.set_device_rate(&dev("dac"), Some(c.outputs.sample_rate));
    c.outputs.set_device_buffer(&dev("dac"), Some(c.outputs.buffer_frames));
    assert_eq!(device_settings(&c.outputs, &dev("dac")), before);
}

#[test]
fn l4_an_own_value_on_a_device_no_route_names_applies_to_nothing() {
    let mut c = Config::default();
    let before = device_settings(&c.outputs, &dev("spare"));
    c.outputs.set_device_rate(&dev("spare"), Some(96_000));
    assert_eq!(device_settings(&c.outputs, &dev("spare")), before);
}
```

- [ ] **Step 2: Run them**

Run: `cargo test -p fp-model --test live l4_`
Expected: PASS (they pin behaviour plan 1 already has; they replace the deleted restart tests).

- [ ] **Step 3: Remove the restart rules**

`git rm crates/fp-model/src/restart.rs crates/fp-model/tests/restart.rs`; in `crates/fp-model/src/lib.rs` remove `pub mod restart;` and `pub use restart::{RestartReason, restart_pending};`.

`crates/fp-model/tests/device_overrides.rs:326`: replace `assert!(fp_model::restart_pending(&started, &state.config).is_empty());` with

```rust
    assert_eq!(
        fp_model::device_settings(&state.config.outputs, &dev("dac")),
        fp_model::device_settings(&started.outputs, &dev("dac")),
        "the device opens as before"
    );
```

`crates/fp-app/tests/settings.rs:759`: delete the line `assert!(fp_model::restart_pending(&before, &basic).is_empty());` (the line before it already asserts that the outputs are unchanged).

Then check that nothing describes the restart any more:

Run: `grep -rn -i "restart_pending\|RestartReason\|restart now\|restart pending\|restart_handoff\|relaunch" crates docs/user docs/technical README.md`
Expected: no output (the transport's "Restart" button, "Restart interface" and `ui-restart` are other features and do not match these patterns).

- [ ] **Step 4: Run the gate**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: all green.

- [ ] **Step 5: Commit**

```bash
git add -A crates/
git commit -m "$(cat <<'EOF'
refactor(model)!: remove RestartReason and restart_pending

fp_model::live replaces them (live settings spec §4.2); the two rules
still true are kept as tests of device_settings (L4).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
EOF
)"
```

---

## Self-Review (done while writing)

- **Spec coverage:** §9.1 pill, tooltip, panel, footer (Task 2); §9.2 every string (Task 1; `pending-running` is an added key for "Still {running}"); §9.3 flow incl. auto-close (Task 2); §9.4 every removal: `RestartReason`, `restart_pending` (Task 4), `with_started_config`, `ExitIntent::Restart`, `begin_restart`, `restart_flag` (Task 2), `restart.rs`, `restart_handoff_ms`, `restart-failed` (Task 3), keys (Tasks 2, 3), platform label (Task 2); §10 nothing added to the remote or MIDI; §12 docs: settings.md, getting-started.md, bit-perfect.md (Task 2), architecture.md restart section (Task 3; the live flow came with plan 3), audio-engine.md and backends.md (plans 2, 3), the four spec notes (Task 3), README (Task 2), troubleshooting.md (Task 2, found while planning).
- **Placeholders:** the nine AI-translated locales are an instruction with exact rules, as the project's process asks (CLAUDE.md: "the others AI-translated"); two conditional fallbacks (`HOURGLASS`, Settings' Esc) name the exact alternative.
- **Type consistency:** `pending::{describe, describe_item, describe_cause, describe_failure, target_label, device_changes, show_panel, show_confirm, Answer, DeviceNames}`, `SettingsDeps::{pending, can_apply, pending_tip}`, `Outcome::apply_now`, `ViewState::{pending_open, apply_now_requested, confirm_apply_now}` match across tasks.
