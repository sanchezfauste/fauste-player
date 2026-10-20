# Cart Colour and Waveform Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Each cart gets an optional colour from a fixed palette of eight (stored by name, picked in Settings → Cartwall, settable over the remote API), shown as a border, a wash and the kind marker on its button, and each cart draws its waveform faintly behind its text.

**Architecture:** `fp-model` gains `CartColor` (lower-case serde names, `ALL`, `name`, `from_name`), `Cart::color` (lenient on load, omitted when `None`) and `CartEdit::color`, applied by `set_cart` and `import_page`. `fp-store` reports an unknown colour in `carts.json` as a load warning (`parse_carts`) and reads and writes `color` in cart page files (`parse_cart_page_noting`). `fp-remote` adds `color` to `CartDto` and an optional `color` to the cart edit body (`Edit::SetCart::color`, `None` keeps). `fp-app` puts the palette in `theme.rs` (`cart_color`), derives each button's colours in the pure `cart_view::cart_look`, draws the waveform with `widgets::paint_wave_columns` over the play range (`CartView::wave_range`), keeps the peaks of the shown cart page in the `MediaCache` (`Services::wanted`), and adds a swatch row to the cart editor.

**Tech Stack:** Rust 2024, serde/serde_json, egui + `egui_kittest`, Fluent, axum (`fp-remote`).

**Spec:** `docs/superpowers/specs/2026-10-15-cart-colour-and-waveform-design.md` (CC1–CC9). It amends `2026-09-26-phase2-cartwall-settings-design.md` (cart fields, `CartEdit`, C11, the cart page file) and `2026-10-01-remote-control-design.md` (cart DTO and edit).

Branch `feat/cart-colour-waveform`, worktree `/home/msanchez/github/fauste-player/.claude/worktrees/p7-cart-colour` (already rebased on `master`).

## Rulings (planning)

**Ruling: one name form, lower case, everywhere** (`"red"` … `"pink"`): `carts.json`, cart page files and the remote API. `CartColor` derives serde with `rename_all = "lowercase"`. — Why: CC1 says the state stores "the name"; CC9 says "the lower-case name"; one form means one parser. — Cost if wrong: a rename attribute and a migration of saved names.

**Ruling: the remote API is strict, files are lenient.** `"Teal"` (capitalised), a number or an unknown name in the remote body is `400`, as an unknown `kind` already is. The same in `carts.json` or a page file loads as no colour with a log line (CC3, CC4). — Why: CC9 "an unknown name answers 400"; CC3 "an unknown name loads as None". — Cost if wrong: one `from_name` call in the handler.

**Ruling: `color` is left out of `carts.json` and page files when it is `None`, and no schema or format version changes** (`CARTS_SCHEMA` stays 1, `CART_PAGE_VERSION` stays 1). The field is additive and optional: older builds ignore an unknown field in both documents (neither uses `deny_unknown_fields`), and a file without colours reads as today (CC4). — Cost if wrong: a version bump with an empty migration.

**Ruling: the "log line" of CC3/CC4 is a load note.** `carts.json`: `fp-store` returns it in `LoadedState::warnings`, which `main` already logs (`tracing::warn!(%warning, "while loading the saved state")`). Page import: `parse_cart_page_noting(…, notes)` fills a `Vec<String>` that the import thread in `fp-app` logs with `tracing::warn!`. `fp-model` and `fp-store` take no logging dependency. `parse_cart_page` keeps its signature (the fuzz target and the tests use it). — Cost if wrong: a `tracing` dependency in `fp-store`.

**Ruling: the waveform spans the play range (cue-in to cue-out), not the whole file.** The remaining-time fill of a playing cart already maps the play range across the button, so the shape and the countdown line up. Only idle and playing carts draw one (an unavailable cart does not, even if old peaks are still in memory). — Why: CC8 says "the same peak data and one column per pixel, like the players' waveform" and is silent on the span. — Cost if wrong: one argument (`0.0, duration` instead of `cue_in, length`).

**Ruling: the `MediaCache` keeps the peaks of the cart page shown** (collapsed or not), next to the players' tracks: `Services::wanted` includes them. Peaks of other pages are let go, as a player's are when it moves on; the analysis cache brings them back when the page is shown. — Why: today only the players' tracks get peaks in memory, so CC8 would never draw. — Cost if wrong: a page switch waits for the analysis cache (milliseconds per cart).

**Ruling: the wash follows hover and press.** A coloured idle cart's fill is the colour at 7 % over `NEUTRAL_900`; its hover and pressed fills are the colour at the same strength over `NEUTRAL_800` and `NEUTRAL_700`, so hovering keeps the colour. Playing: the same at 16 %. — Cost if wrong: two fields of `CartLook`.

**Ruling: the kind marker takes the cart's colour idle and playing; an empty cart's marker stays grey and an unavailable cart's stays amber** (a warning outranks a label). An empty cart with a colour shows its wash and its border, dimmed like the rest of an empty cart (`dim_when_disabled`). — Cost if wrong: one `match` arm.

**Ruling: the light grey of an uncoloured waveform is `theme::NEUTRAL_300` (`#cfd3e5`).** — Cost if wrong: one constant.

**Ruling: the picker's order is "No colour" first, then the palette order** (Red, Orange, Amber, Green, Teal, Blue, Violet, Pink), in a "Colour" row right after "Type". The grid of carts in Settings stays as it is. — Cost if wrong: the order of an iterator.

**Ruling: `Edit::SetCart` gains `color: Option<Option<CartColor>>`** (`None`: keep the cart's colour; `Some(c)`: set `c`, `Some(None)` clears). It overrides `edit.color`, which the HTTP handler cannot know. The JSON body tells an absent field from `null` with a `present` deserializer. — Cost if wrong: a second field shape in one enum variant.

## Global Constraints

- All code, identifiers, comments, docs and commit messages in English; never mention other playout or radio-automation products anywhere.
- Identifiers use `color` (`CartColor`, `cart_color`, `color` fields, Fluent keys `cart-color-*`); English UI text uses the spelling the en-US file already uses ("Colour", "No colour"; the file says "analysed").
- CC1 verbatim: "`CartColor` is an enum with stable names: `Red`, `Orange`, `Amber`, `Green`, `Teal`, `Blue`, `Violet`, `Pink`. The state stores the name, never an RGB value".
- §3 verbatim palette: Red `#d8646a`, Orange `#e8925a`, Amber `#d9b45a`, Green `#7fb08a`, Teal `#4fb3a9`, Blue `#5b9fe0`, Violet `#9184d9`, Pink `#d27fb4`, in `theme.rs` (`cart_color(CartColor)`).
- CC6/CC7/CC8 strengths verbatim: wash 7 % idle, 16 % playing; remaining-time fill 38 %; waveform 13 % idle, 22 % playing; 1 px border in the full colour.
- CLAUDE.md rule 1: every new string goes into **all eleven** locale files (`ca-ES de-DE en-US es-ES eu-ES fr-FR gl-ES it-IT nl-NL pl-PL pt-PT`); en-US is the source, es-ES is written with it, the other nine are AI translations given in Task 7.
- CLAUDE.md rule 8: the UI thread never blocks; peaks come from the `MediaCache` (filled by the services thread), never read on the UI thread.
- CLAUDE.md rule 9: bad data in `carts.json` or a page file degrades to "no colour" with a log line; the rest of the cart and of the file loads.
- CLAUDE.md rule 6: no `unwrap`/`expect`/`panic` outside tests; prefer `get` over indexing.
- The user guide is edited in English only (`docs/user`); `docs/i18n` is never touched.
- UI tests use `egui_kittest` with the recording `Fake` controller (`crates/fp-app/tests/support`); remote tests use `tower::ServiceExt::oneshot` and `FakeControl` (`crates/fp-remote/tests/support`).
- Conventional Commits (scopes `model`, `store`, `remote`, `ui`, `app`, `docs`); every commit message ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Never push.
- **Environment.** `cargo` is at `$HOME/.cargo/bin`. The full suite is slow in debug; every gate runs in this shell setup:

  ```sh
  export PATH="$HOME/.cargo/bin:$PATH"
  export CARGO_PROFILE_DEV_OPT_LEVEL=2
  export CARGO_TARGET_DIR=/home/msanchez/github/fauste-player/.claude/worktrees/p7-cart-colour/target-opt
  ```

  `target-opt/` is not git-ignored: stage only the files each task names, never `git add -A` or `git add .`.
- **Gate before every commit:** `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`. Known environment failure in this profile, not caused by this plan and not to be "fixed": `fp-backends` `cpal_backend_enumerates_without_panicking_and_opens_the_default_device_if_any`. Every other test must pass.
- Progress ledger: `/home/msanchez/github/fauste-player/.superpowers/sdd/itr05/progress.md` (section P7). Record deviations as `Ruling: <decision> — <why> — <cost if wrong>`.

## Review Focus

1. **A hand-edited `carts.json` with `"color": "magenta"` or `"color": 7`**: the whole file must still load (no fall back to a backup or to an empty cartwall), the cart keeps its name, kind and file, and one warning names the bad value. Test: Task 2, `cc3_an_unknown_colour_in_carts_json_loads_as_none_with_a_warning`.
2. **A remote client that sends `"color": "Teal"` (capitalised) or `"color": 3`**: `400`, nothing queued, the cart unchanged. Test: Task 4, `cc9_the_cart_route_takes_a_lower_case_colour_or_null`.
3. **An empty cart whose only setting is its colour**: it must survive a cleared file, and an export/import round trip (the exporter used to skip carts with no file, name, loop or exclusive). Tests: Task 1, `cc2_an_empty_cart_keeps_its_colour_and_clearing_the_file_does_not_clear_it`; Task 3, `cc4_an_empty_cart_with_only_a_colour_is_exported`.
4. **Switching the cart page shown**: the new page's carts get their waveform without a restart, and the old page's peaks are let go. Test: Task 6, `peaks_are_kept_for_the_carts_of_the_page_shown`.
5. **A cart with peaks but no usable play range** (length unknown, or cue-in at cue-out): nothing drawn, no division by zero, no panic. Test: Task 6, `cc8_the_waveform_spans_the_play_range_of_an_idle_or_playing_cart` (its last assertions).

---

## File Structure

| File | Responsibility |
|---|---|
| `crates/fp-model/src/cartwall.rs` | `CartColor`, `Cart::color` (+ `lenient_color`), `CartEdit::color` |
| `crates/fp-model/src/cart_rules.rs` (`set_cart` L259-271, `import_page` L374-400) | apply the colour |
| `crates/fp-model/src/lib.rs` (L39-42) | re-export `CartColor` |
| `crates/fp-model/tests/cartwall.rs` | CC1–CC4 model tests |
| `crates/fp-store/src/store.rs` (`parse_session` L47-75 is the model; `load` L182-201; `fuzz_documents` L109) | `parse_carts` with notes |
| `crates/fp-store/src/playlist_io.rs` (`CartFile` L436-450, `parse_cart_page` L466-522, `write_cart_page` L524-552) | page file colour |
| `crates/fp-store/tests/carts.rs`, `crates/fp-store/tests/playlist_io.rs` | store tests |
| `crates/fp-remote/src/dto.rs` (`CartDto` L172-181, `page_dto` L374-392) | `color` in the DTO |
| `crates/fp-remote/src/api.rs` (`Edit::SetCart` L341-346, `edit_commands` L474-496) | keep/set the colour |
| `crates/fp-remote/src/http/handlers.rs` (`CartBody` L399-407, `set_cart` L491-519) | the body field |
| `crates/fp-remote/tests/dto.rs`, `crates/fp-remote/tests/edit.rs` | remote tests |
| `crates/fp-app/src/ui/theme.rs` | palette and strengths |
| `crates/fp-app/src/ui/cart_view.rs` | `CartView::{color, wave_range}`, `CartLook`, `cart_look` |
| `crates/fp-app/src/ui/cartwall.rs` (`grid` L249-…) | border, wash, fill, marker, waveform |
| `crates/fp-app/src/ui/widgets.rs` (after `waveform`, near L1700) | `paint_wave_columns` |
| `crates/fp-app/src/services.rs` (`wanted` L888-897) | peaks of the shown page |
| `crates/fp-app/src/ui/settings/carts.rs` (import L193-208, editor L387-…) | picker, import notes |
| `crates/fp-app/locales/*/main.ftl` (11 files, after `settings-cart-kind`) | strings |
| `crates/fp-app/tests/support/mod.rs` (`harness_sized` L142-170) | `harness_with_media` |
| `crates/fp-app/tests/theme.rs`, `cartwall_ui.rs`, `services.rs`, `settings.rs` | UI and services tests |
| CartEdit literals (Task 1 Step 4) | `color` added |
| `docs/user/cartwall.md`, `docs/user/remote-control.md`, `docs/technical/remote-api.md`, `persistence.md`, `ui.md`, `analysis.md`, `README.md`, the spec's status | Task 8 |

---

### Task 1: `CartColor` in the model (CC1, CC2, CC3 deserialisation)

**Files:**
- Modify: `crates/fp-model/src/cartwall.rs`, `crates/fp-model/src/cart_rules.rs`, `crates/fp-model/src/lib.rs`
- Modify (add the field to every `CartEdit` literal): see Step 4
- Test: `crates/fp-model/tests/cartwall.rs`

**Interfaces:**
- Produces:
  - `pub enum fp_model::CartColor { Red, Orange, Amber, Green, Teal, Blue, Violet, Pink }` — `Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize`, serde `rename_all = "lowercase"`.
  - `CartColor::ALL: [CartColor; 8]` (that order), `CartColor::name(self) -> &'static str`, `CartColor::from_name(&str) -> Option<CartColor>` (exact lower-case match).
  - `Cart::color: Option<CartColor>`; `CartEdit::color: Option<CartColor>`.

- [ ] **Step 1: Write the failing tests**

In `crates/fp-model/tests/cartwall.rs`, change the import line to add `Cart` and `CartColor`:

```rust
use fp_model::{
    AppState, Cart, CartColor, CartEdit, CartFileChange, CartId, CartKind, CartPageId,
    CartRequest, Command, EngineAction, EngineEvent, FileState, ModelError, SOURCE_END,
    TrackAnalysis, apply, on_event,
};
```

Change the `edit` helper (L53-72) to carry the colour:

```rust
    let mut e = CartEdit {
        name: c.name.clone(),
        kind: c.kind,
        looped: c.looped,
        exclusive: c.exclusive,
        color: c.color,
    };
```

Append these tests at the end of the file:

```rust
/// Cart colour spec CC1.
#[test]
fn cc1_a_cart_has_no_colour_by_default() {
    let s = fixture(0);
    assert!(s.cartwall.pages[0].carts.iter().all(|c| c.color.is_none()));
}

#[test]
fn cc1_every_colour_has_a_stable_lower_case_name() {
    let names: Vec<&str> = CartColor::ALL.iter().map(|c| c.name()).collect();
    assert_eq!(
        names,
        ["red", "orange", "amber", "green", "teal", "blue", "violet", "pink"]
    );
    for c in CartColor::ALL {
        assert_eq!(CartColor::from_name(c.name()), Some(c));
        assert_eq!(serde_json::to_value(c).unwrap(), c.name());
    }
    assert_eq!(CartColor::from_name("Red"), None, "names are lower case");
    assert_eq!(CartColor::from_name("magenta"), None);
}

#[test]
fn cc1_the_state_stores_the_name_and_leaves_out_no_colour() {
    let mut s = fixture(0);
    edit(&mut s, 0, |e| e.color = Some(CartColor::Teal));
    let coloured = serde_json::to_value(&s.cartwall.pages[0].carts[0]).unwrap();
    assert_eq!(coloured["color"], "teal");
    let plain = serde_json::to_value(&s.cartwall.pages[0].carts[1]).unwrap();
    assert!(plain.get("color").is_none(), "no colour, no field: {plain}");
}

/// CC2: `SetCart` and `EditCart` set it with the other fields.
#[test]
fn cc2_set_cart_and_edit_cart_set_the_colour() {
    let mut s = fixture(1);
    load(&mut s, 0, 30.0);
    edit(&mut s, 0, |e| {
        e.name = "ID".into();
        e.color = Some(CartColor::Amber);
    });
    let c = &s.cartwall.pages[0].carts[0];
    assert_eq!((c.name.as_str(), c.color), ("ID", Some(CartColor::Amber)));
    let p = page(&s);
    apply(
        &mut s,
        Command::EditCart {
            page: p,
            index: 0,
            edit: CartEdit {
                color: Some(CartColor::Blue),
                ..plain("ID")
            },
            file: CartFileChange::Keep,
        },
    )
    .unwrap();
    assert_eq!(s.cartwall.pages[0].carts[0].color, Some(CartColor::Blue));
    edit(&mut s, 0, |e| e.color = None);
    assert_eq!(s.cartwall.pages[0].carts[0].color, None, "cleared");
}

/// CC2 with C11: a refused edit changes nothing, the colour included.
#[test]
fn cc2_a_refused_cart_edit_keeps_the_old_colour() {
    let mut s = fixture(1);
    load(&mut s, 0, 30.0);
    edit(&mut s, 0, |e| e.color = Some(CartColor::Green));
    let p = page(&s);
    let unknown = fp_model::TrackId(9_999);
    let e = apply(
        &mut s,
        Command::EditCart {
            page: p,
            index: 0,
            edit: CartEdit {
                color: Some(CartColor::Red),
                ..plain("X")
            },
            file: CartFileChange::Track(unknown),
        },
    );
    assert_eq!(e, Err(ModelError::UnknownTrack(unknown)));
    assert_eq!(s.cartwall.pages[0].carts[0].color, Some(CartColor::Green));
}

#[test]
fn cc2_an_empty_cart_keeps_its_colour_and_clearing_the_file_does_not_clear_it() {
    let mut s = fixture(0);
    let p = page(&s);
    edit(&mut s, 3, |e| e.color = Some(CartColor::Pink));
    assert_eq!(s.cartwall.pages[0].carts[3].track, None, "still empty");
    assert_eq!(s.cartwall.pages[0].carts[3].color, Some(CartColor::Pink));
    load(&mut s, 3, 30.0);
    assert_eq!(
        s.cartwall.pages[0].carts[3].color,
        Some(CartColor::Pink),
        "a file does not change it"
    );
    apply(&mut s, Command::ClearCartFile { page: p, index: 3 }).unwrap();
    assert_eq!(s.cartwall.pages[0].carts[3].color, Some(CartColor::Pink));
    load(&mut s, 3, 30.0);
    apply(
        &mut s,
        Command::EditCart {
            page: p,
            index: 3,
            edit: CartEdit {
                color: Some(CartColor::Pink),
                ..plain("Cleared")
            },
            file: CartFileChange::Clear,
        },
    )
    .unwrap();
    let c = &s.cartwall.pages[0].carts[3];
    assert_eq!((c.track, c.color), (None, Some(CartColor::Pink)));
}

/// CC3: a missing field or an unknown name loads as no colour; the rest
/// of the cart loads as usual.
#[test]
fn cc3_a_missing_or_unknown_colour_loads_as_none() {
    let missing: Cart = serde_json::from_str(r#"{"id":5,"name":"A"}"#).unwrap();
    assert_eq!(missing.color, None);
    for raw in [r#""magenta""#, r#""Red""#, "7", "[]", "{}", "null"] {
        let json = format!(r#"{{"id":5,"name":"A","kind":"Spot","looped":true,"color":{raw}}}"#);
        let c: Cart = serde_json::from_str(&json).unwrap_or_else(|e| panic!("{raw}: {e}"));
        assert_eq!(c.color, None, "{raw}");
        assert_eq!(
            (c.name.as_str(), c.kind, c.looped),
            ("A", CartKind::Spot, true),
            "{raw}"
        );
    }
    let teal: Cart = serde_json::from_str(r#"{"id":5,"color":"teal"}"#).unwrap();
    assert_eq!(teal.color, Some(CartColor::Teal));
}

/// CC4 (model side): an imported page keeps each cart's colour.
#[test]
fn cc4_an_imported_page_keeps_each_carts_colour() {
    let mut s = fixture(0);
    let import = fp_model::CartPageImport {
        name: "Imported".into(),
        rows: 1,
        cols: 2,
        carts: vec![
            (
                0,
                CartEdit {
                    color: Some(CartColor::Violet),
                    ..plain("a")
                },
                None,
            ),
            (1, plain("b"), None),
        ],
    };
    apply(&mut s, Command::ImportCartPage(Box::new(import))).unwrap();
    let page = s.cartwall.pages.last().unwrap();
    assert_eq!(page.carts[0].color, Some(CartColor::Violet));
    assert_eq!(page.carts[1].color, None);
}
```

Also give the existing literals in this file the field: in `importing_duplicate_positions_keeps_no_orphan_track` (L482-487) and in `plain` (L505-512) add `color: None,` after `exclusive: false,`.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-model --test cartwall cc`
Expected: FAIL to compile — `cannot find type CartColor in crate fp_model`, `no field color on type Cart`.

- [ ] **Step 3: Implement the model**

In `crates/fp-model/src/cartwall.rs`, after `CartKind` (L20), add:

```rust
/// A cart's colour (cart colour spec CC1). The state stores its name; the
/// interface's theme gives each one its tone, so the tones can change
/// without touching saved data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CartColor {
    Red,
    Orange,
    Amber,
    Green,
    Teal,
    Blue,
    Violet,
    Pink,
}

impl CartColor {
    /// Every colour, in the picker's order.
    pub const ALL: [CartColor; 8] = [
        CartColor::Red,
        CartColor::Orange,
        CartColor::Amber,
        CartColor::Green,
        CartColor::Teal,
        CartColor::Blue,
        CartColor::Violet,
        CartColor::Pink,
    ];

    /// The stable lower-case name, as files and the remote API write it.
    pub fn name(self) -> &'static str {
        match self {
            CartColor::Red => "red",
            CartColor::Orange => "orange",
            CartColor::Amber => "amber",
            CartColor::Green => "green",
            CartColor::Teal => "teal",
            CartColor::Blue => "blue",
            CartColor::Violet => "violet",
            CartColor::Pink => "pink",
        }
    }

    /// The colour called `name` (lower case), if there is one.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.name() == name)
    }
}

/// CC3: a colour that does not parse loads as none (the store reports it).
fn lenient_color<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<CartColor>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Lenient {
        Valid(Option<CartColor>),
        Other(serde::de::IgnoredAny),
    }
    Ok(match Lenient::deserialize(d)? {
        Lenient::Valid(color) => color,
        Lenient::Other(_) => None,
    })
}
```

In `Cart`, after `exclusive` (L37), add:

```rust
    /// CC1: `None` is no colour. Left out of the file when there is none.
    #[serde(
        default,
        deserialize_with = "lenient_color",
        skip_serializing_if = "Option::is_none"
    )]
    pub color: Option<CartColor>,
```

In `Cart::empty` add `color: None,`. In `CartEdit` (L55-60) add:

```rust
    /// CC2: set with the other fields.
    pub color: Option<CartColor>,
```

In `crates/fp-model/src/cart_rules.rs`, `set_cart` (after `cart.exclusive = edit.exclusive;`): `cart.color = edit.color;`. In `import_page` (after `cart.exclusive = edit.exclusive;`): `cart.color = edit.color;`.

In `crates/fp-model/src/lib.rs` (L39-42), add `CartColor` to the `pub use cartwall::{…}` list.

- [ ] **Step 4: Give every other `CartEdit` literal the field**

Run `cargo check --workspace --all-targets` and add the field at every `missing field color` error. The sites are:

| Site | Add |
|---|---|
| `crates/fp-app/src/ui/settings/carts.rs` L392 (`edit_base`) | `color: cart.color,` |
| `crates/fp-app/src/ui/settings/carts.rs` L415 (the name kept for the previous cart) | `color: old.color,` |
| `crates/fp-store/src/playlist_io.rs` L509 (`parse_cart_page`) | `color: None,` (Task 3 reads it) |
| `crates/fp-remote/src/http/handlers.rs` L502 (`set_cart`) | `color: None,` (Task 4 replaces it) |
| `crates/fp-app/examples/demo_session.rs` L110 | `color: None,` |
| `crates/fp-engine/tests/conductor.rs` L363 | `color: None,` |
| `crates/fp-model/tests/cartwall_props.rs` L100 | `color: None` (inside the one-line literal) |
| `crates/fp-model/tests/track_tags.rs` L356 | `color: None,` |
| `crates/fp-store/tests/carts.rs` L33 | `color: None,` (Task 2 changes it) |
| `crates/fp-store/tests/playlist_io.rs` L199, L214, L245 | `color: None,` (Task 3 changes L199 and L245) |
| `crates/fp-remote/tests/edit.rs` L184 | `color: None,` |
| `crates/fp-app/tests/cartwall_ui.rs` L51 | `color: None,` |

Then `cargo check --workspace --all-targets` must be clean.

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p fp-model --test cartwall`
Expected: PASS (all old and new tests).

- [ ] **Step 6: Gate and commit**

Run the gate (Global Constraints). Then:

```bash
git add crates/fp-model crates/fp-store/src/playlist_io.rs crates/fp-store/tests crates/fp-remote/src/http/handlers.rs crates/fp-remote/tests/edit.rs crates/fp-app/src/ui/settings/carts.rs crates/fp-app/examples/demo_session.rs crates/fp-app/tests/cartwall_ui.rs crates/fp-engine/tests/conductor.rs
git commit -m "feat(model): cart colour

A cart has an optional colour from a fixed palette of eight, stored by
its lower-case name. SetCart, EditCart and a page import set it with the
other fields; an empty cart keeps it and clearing the file does not
clear it. A missing or unknown name loads as no colour (cart colour
spec CC1-CC3).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: `carts.json` reports an unknown colour (CC3)

**Files:**
- Modify: `crates/fp-store/src/store.rs`
- Test: `crates/fp-store/tests/carts.rs`

**Interfaces:**
- Consumes: `fp_model::CartColor::from_name` (Task 1).
- Produces: `fn parse_carts(bytes: &[u8]) -> Result<(CartsDoc, Vec<String>), ParseError>` (private), used by `Store::load` and `fuzz_documents`.

- [ ] **Step 1: Write the failing tests**

In `crates/fp-store/tests/carts.rs`, import `CartColor` (`use fp_model::{AppState, CartColor, CartEdit, CartKind, Command, Config, apply};`) and give `edited_state`'s edit a colour (L33-38):

```rust
    let edit = CartEdit {
        name: "Applause".into(),
        kind: CartKind::Effect,
        looped: true,
        exclusive: true,
        color: Some(CartColor::Teal),
    };
```

(`carts_round_trip` then pins the round trip: it compares the loaded pages with the saved ones.) Append:

```rust
/// Cart colour spec CC3: a hand edit that breaks a colour costs only that
/// colour, with a warning; the cart and the rest of the file load.
#[test]
fn cc3_an_unknown_colour_in_carts_json_loads_as_none_with_a_warning() {
    for bad in [r#""magenta""#, "7"] {
        let dir = tempfile::tempdir().unwrap();
        let s = store(&dir);
        let state = edited_state();
        save_all(&s, &state);
        let file = s.paths().carts_file();
        let text = fs::read_to_string(&file).unwrap();
        assert_eq!(text.matches("\"color\"").count(), 1, "only the coloured cart: {text}");
        fs::write(&file, text.replace(r#""teal""#, bad)).unwrap();
        let loaded = s.load("Main");
        let cart = &loaded.state.cartwall.pages[1].carts[3];
        assert_eq!(cart.color, None, "{bad}");
        assert_eq!(
            (cart.name.as_str(), cart.kind, cart.looped, cart.exclusive),
            ("Applause", CartKind::Effect, true, true),
            "{bad}"
        );
        assert!(cart.track.is_some(), "{bad}: the file is kept");
        let notes: Vec<&String> = loaded
            .warnings
            .iter()
            .filter(|w| w.contains("colour"))
            .collect();
        assert_eq!(notes.len(), 1, "{bad}: {:?}", loaded.warnings);
        assert!(notes[0].contains(bad), "{bad}: {}", notes[0]);
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-store --test carts`
Expected: `carts_round_trip` PASSES already (Task 1 serialises the colour); `cc3_an_unknown_colour_in_carts_json_loads_as_none_with_a_warning` FAILS at `assert_eq!(notes.len(), 1, …)` (no warning yet).

- [ ] **Step 3: Implement `parse_carts`**

In `crates/fp-store/src/store.rs`, after `parse_session` (L75), add:

```rust
/// Parses `carts.json`. A cart colour that does not parse loads as none
/// (the model decides how, cart colour spec CC3); each one is reported.
fn parse_carts(bytes: &[u8]) -> Result<(CartsDoc, Vec<String>), ParseError> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| ParseError::Corrupt(e.to_string()))?;
    let value = upgrade(value, CARTS_SCHEMA, CARTS_MIGRATIONS)?;
    let notes = unknown_cart_colors(&value);
    let doc: CartsDoc =
        serde_json::from_value(value).map_err(|e| ParseError::Corrupt(e.to_string()))?;
    Ok((doc, notes))
}

/// One note per cart whose `color` is present, not `null` and not a known
/// name.
fn unknown_cart_colors(value: &serde_json::Value) -> Vec<String> {
    items(value.get("pages"))
        .iter()
        .flat_map(|page| items(page.get("carts")))
        .filter_map(|cart| {
            let raw = cart.get("color").filter(|c| !c.is_null())?;
            let known = raw.as_str().and_then(fp_model::CartColor::from_name);
            known.is_none().then(|| {
                let id = cart
                    .get("id")
                    .map_or_else(|| "?".to_owned(), ToString::to_string);
                format!("carts: cart {id} has an unknown colour {raw}; it has none")
            })
        })
        .collect()
}

/// The elements of a JSON array, or none.
fn items(value: Option<&serde_json::Value>) -> &[serde_json::Value] {
    value
        .and_then(serde_json::Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default()
}
```

In `fuzz_documents` (L109) replace the carts line with:

```rust
    let carts: Option<CartsDoc> = parse_carts(bytes).ok().map(|(d, _)| d);
```

In `Store::load`, replace the `carts` load (L182-189) with:

```rust
        let carts: Loaded<(CartsDoc, Vec<String>)> = load_with_fallback(
            &self.paths.carts_file(),
            limits.backup_count,
            limits.max_state_file_bytes,
            parse_carts,
        );
        warnings.extend(carts.warnings);
        let carts_doc = carts.value.map(|(doc, notes)| {
            warnings.extend(notes);
            doc
        });
```

and the later `let cart_pages = carts.value.map(|d| d.pages).unwrap_or_default();` (L201) with:

```rust
        let cart_pages = carts_doc.map(|d| d.pages).unwrap_or_default();
```

If `load_doc` is now unused for carts only, it stays (playlists use it). If clippy reports an unused import, remove it.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-store --test carts && cargo test -p fp-store --test store_roundtrip`
Expected: PASS.

- [ ] **Step 5: Gate and commit**

```bash
git add crates/fp-store/src/store.rs crates/fp-store/tests/carts.rs
git commit -m "feat(store): report an unknown cart colour in carts.json

A colour a hand edit broke loads as no colour and becomes a load
warning, which the app logs; the cart and the rest of the file load
as usual (cart colour spec CC3).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: Cart page files carry the colour (CC4)

**Files:**
- Modify: `crates/fp-store/src/playlist_io.rs`, `crates/fp-app/src/ui/settings/carts.rs` (import, L193-208)
- Test: `crates/fp-store/tests/playlist_io.rs`

**Interfaces:**
- Consumes: `CartColor::{name, from_name}` (Task 1).
- Produces: `pub fn fp_store::playlist_io::parse_cart_page_noting(bytes: &[u8], source: &Path, limits: &Limits, notes: &mut Vec<String>) -> Result<CartPageImport, CartPageFileError>`; `parse_cart_page` keeps its signature and calls it.

- [ ] **Step 1: Write the failing tests**

In `crates/fp-store/tests/playlist_io.rs`: add `CartColor` to the `fp_model` import and `parse_cart_page_noting` to the `fp_store::playlist_io` import. In `page_state`, give "Station ID" (L199-204) `color: Some(CartColor::Amber),` and in `cart_page_round_trips` the expected `CartEdit` (L245-251) `color: Some(CartColor::Amber)`. Append:

```rust
/// Cart colour spec CC4: a page file's colours are read with CC3's
/// leniency; a file without colours imports as before.
#[test]
fn cc4_cart_page_colours_are_read_leniently() {
    let text = br#"{"format":"fauste-cart-page","version":1,"name":"X","rows":1,"cols":5,
        "carts":[{"position":1,"name":"A","color":"teal"},
                 {"position":2,"name":"B","color":"magenta"},
                 {"position":3,"name":"C","color":3},
                 {"position":4,"name":"D","color":null},
                 {"position":5,"name":"E"}]}"#;
    let mut notes = Vec::new();
    let import =
        parse_cart_page_noting(text, Path::new("/x.cartpage.json"), &limits(), &mut notes).unwrap();
    let colors: Vec<_> = import.carts.iter().map(|c| c.1.color).collect();
    assert_eq!(colors, [Some(CartColor::Teal), None, None, None, None]);
    let names: Vec<_> = import.carts.iter().map(|c| c.1.name.as_str()).collect();
    assert_eq!(names, ["A", "B", "C", "D", "E"], "every cart is read");
    assert_eq!(notes.len(), 2, "{notes:?}");
    assert!(notes[0].contains("\"magenta\""), "{notes:?}");
    assert!(notes[1].ends_with("colour 3; it has none"), "{notes:?}");
}

#[test]
fn cc4_an_empty_cart_with_only_a_colour_is_exported() {
    let mut state = page_state();
    let page = state.cartwall.pages[0].id;
    apply(
        &mut state,
        Command::SetCart {
            page,
            index: 7,
            edit: CartEdit {
                name: String::new(),
                kind: CartKind::Jingle,
                looped: false,
                exclusive: false,
                color: Some(CartColor::Green),
            },
        },
    )
    .unwrap();
    let text = write_cart_page(&state.cartwall.pages[0], &state.library);
    let import =
        parse_cart_page(text.as_bytes(), Path::new("/x/p.cartpage.json"), &limits()).unwrap();
    let empty = import.carts.iter().find(|c| c.0 == 7).expect("exported");
    assert_eq!((empty.1.color, &empty.2), (Some(CartColor::Green), &None));
    let plain = import.carts.iter().find(|c| c.0 == 5).unwrap();
    assert_eq!(plain.1.color, None);
    assert_eq!(
        text.matches("\"color\"").count(),
        2,
        "no field for a cart without a colour: {text}"
    );
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-store --test playlist_io cart_page`
Expected: FAIL to compile — `no function parse_cart_page_noting`.

- [ ] **Step 3: Implement**

In `crates/fp-store/src/playlist_io.rs`, add `CartColor` to the `fp_model` import (L16). In `CartFile` (after `exclusive`, L449):

```rust
    /// Cart colour spec CC4: the colour's name; left out when there is none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    color: Option<serde_json::Value>,
```

Replace `pub fn parse_cart_page(…)` by the pair:

```rust
/// Reads a cart page file. The grid is clamped to the limits; relative
/// files resolve against the file's folder; carts outside the grid are
/// kept in the result and dropped by the model.
pub fn parse_cart_page(
    bytes: &[u8],
    source: &Path,
    limits: &Limits,
) -> Result<CartPageImport, CartPageFileError> {
    parse_cart_page_noting(bytes, source, limits, &mut Vec::new())
}

/// As [`parse_cart_page`]; what it reads leniently (a cart colour that is
/// not a known name loads as none, cart colour spec CC4) is added to
/// `notes`.
pub fn parse_cart_page_noting(
    bytes: &[u8],
    source: &Path,
    limits: &Limits,
    notes: &mut Vec<String>,
) -> Result<CartPageImport, CartPageFileError> {
```

with the old body, where the per-cart `map` builds the edit as:

```rust
            let color = match c.color {
                None | Some(serde_json::Value::Null) => None,
                Some(raw) => {
                    let known = raw.as_str().and_then(CartColor::from_name);
                    if known.is_none() {
                        notes.push(format!(
                            "cart page: cart {} has an unknown colour {raw}; it has none",
                            c.position
                        ));
                    }
                    known
                }
            };
            let edit = CartEdit {
                name: c.name,
                kind: kind_from(&c.kind),
                looped: c.looped,
                exclusive: c.exclusive,
                color,
            };
```

In `write_cart_page`, the filter becomes

```rust
        .filter(|(_, c)| {
            c.track.is_some() || !c.name.is_empty() || c.looped || c.exclusive || c.color.is_some()
        })
```

and the `CartFile` gets `color: c.color.map(|k| serde_json::Value::from(k.name())),`.

In `crates/fp-app/src/ui/settings/carts.rs`, import `parse_cart_page_noting` instead of `parse_cart_page` (L10) and replace the `and_then` of the import (L203-205):

```rust
                        .and_then(|bytes| {
                            let mut notes = Vec::new();
                            let import =
                                parse_cart_page_noting(&bytes, &path, &limits, &mut notes)
                                    .map_err(FileError::from);
                            for note in &notes {
                                tracing::warn!(%note, "while importing a cart page");
                            }
                            import
                        }),
```

(It runs on the `fp-cartwall-io` helper thread, never on the UI thread.)

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-store --test playlist_io`
Expected: PASS. Also `cargo check --manifest-path fuzz/Cargo.toml` if the fuzz crate builds on this machine (it uses `parse_cart_page`, unchanged); skip with a ledger note if `cargo-fuzz`'s toolchain is missing.

- [ ] **Step 5: Gate and commit**

```bash
git add crates/fp-store/src/playlist_io.rs crates/fp-store/tests/playlist_io.rs crates/fp-app/src/ui/settings/carts.rs
git commit -m "feat(store): cart colours in cart page files

Export writes each cart's colour (an empty cart with only a colour is
written too); import reads it leniently and logs an unknown name. A
file without colours imports as before (cart colour spec CC4).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: Remote control (CC9)

**Files:**
- Modify: `crates/fp-remote/src/dto.rs`, `crates/fp-remote/src/api.rs`, `crates/fp-remote/src/http/handlers.rs`
- Test: `crates/fp-remote/tests/dto.rs`, `crates/fp-remote/tests/edit.rs`

**Interfaces:**
- Consumes: `CartColor` (Task 1).
- Produces: `CartDto::color: Option<&'static str>` (serialised `null` when none); `Edit::SetCart { page, index, edit, track, color: Option<Option<CartColor>> }` — `None` keeps the cart's colour and overrides `edit.color`.

- [ ] **Step 1: Write the failing tests**

`crates/fp-remote/tests/dto.rs`: append

```rust
/// Cart colour spec CC9: the cart DTO has the colour's name, or `null`.
#[test]
fn cc9_a_cart_has_its_colour_name_or_null() {
    let mut s = demo_state();
    let page = s.cartwall.pages[0].id;
    let c = s.cartwall.pages[0].carts[0].clone();
    fp_model::apply(
        &mut s,
        Command::SetCart {
            page,
            index: 0,
            edit: fp_model::CartEdit {
                name: c.name,
                kind: c.kind,
                looped: c.looped,
                exclusive: c.exclusive,
                color: Some(fp_model::CartColor::Violet),
            },
        },
    )
    .unwrap();
    let out = dto::cartwall(&s, &Playback::default());
    assert_eq!(out.pages[0].carts[0].color, Some("violet"));
    assert_eq!(out.pages[0].carts[1].color, None);
    let json = serde_json::to_value(&out).unwrap();
    assert_eq!(json["pages"][0]["carts"][0]["color"], "violet");
    let plain = &json["pages"][0]["carts"][1];
    assert!(plain.get("color").is_some_and(serde_json::Value::is_null), "{plain}");
}
```

`crates/fp-remote/tests/edit.rs`: add `CartColor` and `AppState` to the `fp_model` import. In `a_cart_edit_keeping_its_track_does_not_reassign_it`, add `color: None` to each of the four `Edit::SetCart { … }` (after `track: …`). Append:

```rust
/// CC9: the edit sets, clears or (omitted) keeps the colour.
#[test]
fn cc9_a_cart_edit_sets_clears_or_keeps_the_colour() {
    let mut s = demo_state();
    let page = s.cartwall.pages[0].id;
    let t = s.cartwall.pages[0].carts[0].track.unwrap();
    let edit = CartEdit {
        name: "Top".into(),
        kind: CartKind::Spot,
        looped: false,
        exclusive: false,
        color: None,
    };
    let planned = |s: &AppState, color: Option<Option<CartColor>>| {
        let commands = plan_edit(
            s,
            Edit::SetCart {
                page,
                index: 0,
                edit: edit.clone(),
                track: Some(t),
                color,
            },
        )
        .unwrap();
        match commands.as_slice() {
            [Command::EditCart { edit, .. }] => edit.color,
            other => panic!("{other:?}"),
        }
    };
    assert_eq!(planned(&s, Some(Some(CartColor::Teal))), Some(CartColor::Teal));
    fp_model::apply(
        &mut s,
        Command::SetCart {
            page,
            index: 0,
            edit: CartEdit {
                color: Some(CartColor::Red),
                ..edit.clone()
            },
        },
    )
    .unwrap();
    assert_eq!(planned(&s, None), Some(CartColor::Red), "omitted keeps it");
    assert_eq!(planned(&s, Some(None)), None, "null clears it");
}

#[tokio::test]
async fn cc9_the_cart_route_takes_a_lower_case_colour_or_null() {
    let mut s = demo_state();
    let page = s.cartwall.pages[0].id;
    let t = s.cartwall.pages[0].carts[0].track.unwrap().0;
    fp_model::apply(
        &mut s,
        Command::SetCart {
            page,
            index: 0,
            edit: CartEdit {
                name: "ID".into(),
                kind: CartKind::Jingle,
                looped: false,
                exclusive: false,
                color: Some(CartColor::Red),
            },
        },
    )
    .unwrap();
    let fake = FakeControl::new(s);
    let uri = format!("/api/v1/cartwall/pages/{}/carts/0", page.0);
    let body = |color: Option<Value>| {
        let mut b = json!({"name": "ID", "kind": "jingle", "looped": false,
                           "exclusive": false, "track": t});
        if let Some(c) = color {
            b["color"] = c;
        }
        b
    };
    let sent_color = |fake: &Arc<FakeControl>| match fake.take_sent().as_slice() {
        [Command::EditCart { edit, .. }] => edit.color,
        other => panic!("{other:?}"),
    };
    for bad in [json!("Teal"), json!("magenta"), json!(3), json!(true)] {
        let status = call(&fake, "PUT", &uri, Some(body(Some(bad.clone())))).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{bad}");
        assert!(fake.take_sent().is_empty(), "{bad}: nothing queued");
    }
    let status = call(&fake, "PUT", &uri, Some(body(Some(json!("teal"))))).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(sent_color(&fake), Some(CartColor::Teal));
    let status = call(&fake, "PUT", &uri, Some(body(Some(Value::Null)))).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(sent_color(&fake), None, "null clears");
    let status = call(&fake, "PUT", &uri, Some(body(None))).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(sent_color(&fake), Some(CartColor::Red), "omitted keeps");
}
```

(`FakeControl::take_sent` is the recording helper the HTTP tests already use; `FakeControl::new` returns an `Arc`, which `edit.rs` already imports.)

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-remote --test dto cc9 ; cargo test -p fp-remote --test edit cc9`
Expected: FAIL to compile — `no field color on CartDto`, `struct variant Edit::SetCart has no field named color`.

- [ ] **Step 3: Implement**

`crates/fp-remote/src/dto.rs`: in `CartDto` after `exclusive`:

```rust
    /// Cart colour spec CC9: the colour's lower-case name, or `null`.
    pub color: Option<&'static str>,
```

and in `page_dto`: `color: c.color.map(fp_model::CartColor::name),`.

`crates/fp-remote/src/api.rs`: in `Edit::SetCart` after `track`:

```rust
        /// CC9: `None` keeps the cart's colour; `Some` sets it (`Some(None)`
        /// clears it). It replaces `edit.color`.
        color: Option<Option<CartColor>>,
```

(add `CartColor` to its `fp_model` import) and in `edit_commands`:

```rust
        Edit::SetCart {
            page: id,
            index,
            edit,
            track: wanted,
            color,
        } => {
            let cart = page(id)?.carts.get(index).ok_or(ApiError::NotFound)?;
            let edit = CartEdit {
                color: color.unwrap_or(cart.color),
                ..edit
            };
            // … the existing `file` match and `EditCart` unchanged …
```

`crates/fp-remote/src/http/handlers.rs`: add `CartColor` to the `fp_model` import; `CartBody` becomes

```rust
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CartBody {
    name: String,
    kind: CartKindName,
    looped: bool,
    exclusive: bool,
    track: Option<u64>,
    /// CC9: absent keeps the colour, `null` clears it, a lower-case name
    /// sets it; anything else is `400`.
    #[serde(default, deserialize_with = "present")]
    color: Option<Option<CartColor>>,
}

/// A field that may be absent (`None`, through `default`), `null`
/// (`Some(None)`) or a value (`Some(Some(v))`).
fn present<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(d).map(Some)
}
```

and in `set_cart` the edit keeps `color: None,` (Task 1) while the `Edit::SetCart` gets `color: b.color,`.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-remote`
Expected: PASS.

- [ ] **Step 5: Gate and commit**

```bash
git add crates/fp-remote
git commit -m "feat(remote): cart colour in the cart DTO and the cart edit

The cart DTO carries the colour's lower-case name or null. The cart
edit takes an optional colour: a name sets it, null clears it, leaving
it out keeps it, and anything else is 400 (cart colour spec CC9).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: The palette and the coloured cart button (CC6, CC7)

**Files:**
- Modify: `crates/fp-app/src/ui/theme.rs`, `crates/fp-app/src/ui/cart_view.rs`, `crates/fp-app/src/ui/cartwall.rs`
- Test: `crates/fp-app/tests/theme.rs`, `crates/fp-app/tests/cartwall_ui.rs`

**Interfaces:**
- Consumes: `CartColor` (Task 1).
- Produces:
  - `pub fn theme::cart_color(CartColor) -> Color32`; `pub const theme::{CART_WASH_IDLE, CART_WASH_PLAYING, CART_PROGRESS_ALPHA, CART_WAVE_IDLE_ALPHA, CART_WAVE_PLAYING_ALPHA}: f32`; `pub const theme::CART_WAVE_PLAIN: Color32`.
  - `CartView::color: Option<CartColor>`.
  - `pub struct cart_view::CartLook { fill, hover_fill, active_fill, border, progress, marker: Option<Color32>, wave: Color32 }` (all `Color32` but `marker`), `pub fn cart_view::cart_look(color: Option<CartColor>, playing: bool) -> CartLook`.

- [ ] **Step 1: Write the failing tests**

`crates/fp-app/tests/theme.rs`: append

```rust
/// Cart colour spec §3: the palette's tones.
#[test]
fn every_cart_colour_has_its_tone() {
    use fp_model::CartColor::*;
    let tones = [
        (Red, 0xd8646a),
        (Orange, 0xe8925a),
        (Amber, 0xd9b45a),
        (Green, 0x7fb08a),
        (Teal, 0x4fb3a9),
        (Blue, 0x5b9fe0),
        (Violet, 0x9184d9),
        (Pink, 0xd27fb4),
    ];
    for (color, hex) in tones {
        let [_, r, g, b] = u32::to_be_bytes(hex);
        assert_eq!(theme::cart_color(color), Color32::from_rgb(r, g, b), "{color:?}");
    }
}
```

`crates/fp-app/tests/cartwall_ui.rs`: extend the imports:

```rust
use egui::{Color32, Rect};
use fp_app::ui::cart_view::{
    BUTTON_HEIGHT, CartStatus, GAP, MIN_BUTTON_HEIGHT, button_height, cart_look, cart_view,
    page_on_air,
};
use fp_app::ui::theme;
use fp_model::{AppState, CartColor, CartEdit, CartKind, Command, FileState, TrackAnalysis, apply};
```

and append:

```rust
type H = egui_kittest::Harness<'static, fp_app::ui::app::AppUi>;

/// Gives cart `index` of the first page `color`, keeping its other fields.
fn paint_cart(s: &mut AppState, index: usize, color: Option<CartColor>) {
    let page = s.cartwall.pages[0].id;
    let c = s.cartwall.pages[0].carts[index].clone();
    let edit = CartEdit {
        name: c.name,
        kind: c.kind,
        looped: c.looped,
        exclusive: c.exclusive,
        color,
    };
    apply(s, Command::SetCart { page, index, edit }).unwrap();
}

/// Every filled or stroked rectangle painted inside `area` in the last frame.
fn rects_in(h: &H, area: Rect) -> Vec<egui::epaint::RectShape> {
    fn walk(shape: &egui::Shape, out: &mut Vec<egui::epaint::RectShape>) {
        match shape {
            egui::Shape::Rect(r) => out.push(r.clone()),
            egui::Shape::Vec(v) => v.iter().for_each(|s| walk(s, out)),
            _ => {}
        }
    }
    let mut all = Vec::new();
    for clipped in &h.output().shapes {
        walk(&clipped.shape, &mut all);
    }
    all.retain(|r| area.expand(0.5).contains_rect(r.rect));
    all
}

fn same(a: Rect, b: Rect) -> bool {
    (a.min - b.min).length() < 0.5 && (a.max - b.max).length() < 0.5
}

/// The button's background, its border and whether a 6×6 marker in
/// `marker` is drawn on it.
fn button_paint(h: &H, label: &str, marker: Color32) -> (Color32, Color32, bool) {
    let button = h.get_by_label(label).rect();
    let rects = rects_in(h, button);
    let fill = rects
        .iter()
        .find(|r| same(r.rect, button) && r.fill != Color32::TRANSPARENT)
        .map(|r| r.fill)
        .unwrap_or_else(|| panic!("no background for {label}"));
    let border = rects
        .iter()
        .find(|r| same(r.rect, button) && r.stroke.width > 0.0)
        .map(|r| r.stroke.color)
        .unwrap_or_else(|| panic!("no border for {label}"));
    let has_marker = rects.iter().any(|r| {
        (r.rect.width() - 6.0).abs() < 0.5 && (r.rect.height() - 6.0).abs() < 0.5 && r.fill == marker
    });
    (fill, border, has_marker)
}

/// The width of the remaining-time fill in `progress` on the button.
fn progress_width(h: &H, label: &str, progress: Color32) -> f32 {
    let button = h.get_by_label(label).rect();
    rects_in(h, button)
        .iter()
        .filter(|r| r.fill == progress)
        .map(|r| r.rect.width())
        .fold(0.0, f32::max)
}

/// The spec's strengths (CC6, CC7), and a cart without a colour as before.
#[test]
fn cart_looks_follow_the_spec() {
    let red = theme::cart_color(CartColor::Red);
    let idle = cart_look(Some(CartColor::Red), false);
    assert_eq!(idle.border, red);
    assert_eq!(idle.fill, theme::NEUTRAL_900.lerp_to_gamma(red, 0.07));
    assert_eq!(idle.hover_fill, theme::NEUTRAL_800.lerp_to_gamma(red, 0.07));
    assert_eq!(idle.active_fill, theme::NEUTRAL_700.lerp_to_gamma(red, 0.07));
    assert_eq!(idle.marker, Some(red));
    let playing = cart_look(Some(CartColor::Red), true);
    assert_eq!(playing.border, red);
    assert_eq!(playing.fill, theme::NEUTRAL_900.lerp_to_gamma(red, 0.16));
    assert_eq!(playing.progress, red.gamma_multiply(0.38));
    let plain = cart_look(None, false);
    assert_eq!(
        (plain.fill, plain.hover_fill, plain.active_fill, plain.border, plain.marker),
        (theme::NEUTRAL_900, theme::NEUTRAL_800, theme::NEUTRAL_700, theme::NEUTRAL_800, None)
    );
    let plain_playing = cart_look(None, true);
    assert_eq!(plain_playing.border, theme::ON_AIR_TEXT);
    assert_eq!(plain_playing.fill, theme::NEUTRAL_900);
    assert_eq!(plain_playing.progress, theme::ON_AIR_ROW.gamma_multiply(0.5));
}

#[test]
fn a_cart_view_carries_its_colour() {
    let mut s = with_cart();
    paint_cart(&mut s, 0, Some(CartColor::Teal));
    let id = s.cartwall.pages[0].carts[0].id;
    assert_eq!(
        cart_view(&s, &Telemetry::default(), id).unwrap().color,
        Some(CartColor::Teal)
    );
}

/// CC6.
#[test]
fn cc6_an_idle_cart_with_a_colour_has_its_border_wash_and_marker() {
    let mut s = with_cart();
    paint_cart(&mut s, 0, Some(CartColor::Teal));
    let (h, _fake) = harness(s);
    let look = cart_look(Some(CartColor::Teal), false);
    let teal = theme::cart_color(CartColor::Teal);
    assert_eq!(button_paint(&h, "Station ID", teal), (look.fill, teal, true));
}

#[test]
fn cc6_a_cart_without_a_colour_is_drawn_as_before() {
    let (h, _fake) = harness(with_cart());
    // A jingle's marker is the accent violet.
    assert_eq!(
        button_paint(&h, "Station ID", theme::ACCENT),
        (theme::NEUTRAL_900, theme::NEUTRAL_800, true)
    );
}

/// CC7.
#[test]
fn cc7_a_playing_cart_with_a_colour_keeps_it_and_its_countdown_shows() {
    let mut s = with_cart();
    paint_cart(&mut s, 0, Some(CartColor::Amber));
    let id = s.cartwall.pages[0].carts[0].id;
    apply(&mut s, Command::FireCart(id)).unwrap();
    let (h, _fake) = harness(s);
    let look = cart_look(Some(CartColor::Amber), true);
    let amber = theme::cart_color(CartColor::Amber);
    assert_eq!(button_paint(&h, "Station ID", amber), (look.fill, amber, true));
    let button = h.get_by_label("Station ID").rect();
    assert!(
        (progress_width(&h, "Station ID", look.progress) - button.width()).abs() < 1.0,
        "the whole play range is still to play"
    );
}

#[test]
fn cc7_a_playing_cart_without_a_colour_is_red_as_before() {
    let mut s = with_cart();
    let id = s.cartwall.pages[0].carts[0].id;
    apply(&mut s, Command::FireCart(id)).unwrap();
    let (h, _fake) = harness(s);
    let (fill, border, _) = button_paint(&h, "Station ID", theme::ACCENT);
    assert_eq!((fill, border), (theme::NEUTRAL_900, theme::ON_AIR_TEXT));
    assert!(progress_width(&h, "Station ID", theme::ON_AIR_ROW.gamma_multiply(0.5)) > 0.0);
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-app --test theme --test cartwall_ui`
Expected: FAIL to compile — `cannot find function cart_color in module theme`, `cart_look`.

- [ ] **Step 3: Implement the palette**

`crates/fp-app/src/ui/theme.rs`: add `use fp_model::CartColor;` and, after the waveform marker colours:

```rust
/// Cart colours (cart colour spec §3): the tone of each `CartColor`. The
/// state stores the name; only this table knows the tones.
pub fn cart_color(color: CartColor) -> Color32 {
    match color {
        CartColor::Red => Color32::from_rgb(0xd8, 0x64, 0x6a),
        CartColor::Orange => Color32::from_rgb(0xe8, 0x92, 0x5a),
        CartColor::Amber => Color32::from_rgb(0xd9, 0xb4, 0x5a),
        CartColor::Green => Color32::from_rgb(0x7f, 0xb0, 0x8a),
        CartColor::Teal => Color32::from_rgb(0x4f, 0xb3, 0xa9),
        CartColor::Blue => Color32::from_rgb(0x5b, 0x9f, 0xe0),
        CartColor::Violet => Color32::from_rgb(0x91, 0x84, 0xd9),
        CartColor::Pink => Color32::from_rgb(0xd2, 0x7f, 0xb4),
    }
}

/// How strongly a cart's colour washes its button, idle (CC6) and playing
/// (CC7).
pub const CART_WASH_IDLE: f32 = 0.07;
pub const CART_WASH_PLAYING: f32 = 0.16;
/// The remaining-time fill of a playing cart with a colour (CC7).
pub const CART_PROGRESS_ALPHA: f32 = 0.38;
/// The waveform behind a cart's text, idle and playing (CC8).
pub const CART_WAVE_IDLE_ALPHA: f32 = 0.13;
pub const CART_WAVE_PLAYING_ALPHA: f32 = 0.22;
/// The waveform of a cart without a colour: light grey (CC8).
pub const CART_WAVE_PLAIN: Color32 = NEUTRAL_300;
```

- [ ] **Step 4: Implement `CartView::color` and `cart_look`**

`crates/fp-app/src/ui/cart_view.rs`: imports become

```rust
use egui::Color32;
use fp_engine::conductor::Telemetry;
use fp_model::{AppState, CartColor, CartId, CartKind, CartPageId};

use super::{format, theme};
```

`CartView` gets, after `cueing`:

```rust
    /// The cart's colour (cart colour spec CC1).
    pub color: Option<CartColor>,
```

and `cart_view` sets `color: cart.color,`. Append:

```rust
/// The colours of a cart button (cart colour spec CC6–CC8).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CartLook {
    pub fill: Color32,
    pub hover_fill: Color32,
    pub active_fill: Color32,
    pub border: Color32,
    /// The remaining-time fill while it plays.
    pub progress: Color32,
    /// The kind marker of an idle or playing cart; `None`: the kind's own.
    pub marker: Option<Color32>,
    /// The waveform behind the text.
    pub wave: Color32,
}

pub fn cart_look(color: Option<CartColor>, playing: bool) -> CartLook {
    let wave_alpha = if playing {
        theme::CART_WAVE_PLAYING_ALPHA
    } else {
        theme::CART_WAVE_IDLE_ALPHA
    };
    match color {
        None => CartLook {
            fill: theme::NEUTRAL_900,
            hover_fill: theme::NEUTRAL_800,
            active_fill: theme::NEUTRAL_700,
            border: if playing {
                theme::ON_AIR_TEXT
            } else {
                theme::NEUTRAL_800
            },
            progress: theme::ON_AIR_ROW.gamma_multiply(0.5),
            marker: None,
            wave: theme::CART_WAVE_PLAIN.gamma_multiply(wave_alpha),
        },
        Some(color) => {
            let tone = theme::cart_color(color);
            let wash = if playing {
                theme::CART_WASH_PLAYING
            } else {
                theme::CART_WASH_IDLE
            };
            CartLook {
                fill: theme::NEUTRAL_900.lerp_to_gamma(tone, wash),
                hover_fill: theme::NEUTRAL_800.lerp_to_gamma(tone, wash),
                active_fill: theme::NEUTRAL_700.lerp_to_gamma(tone, wash),
                border: tone,
                progress: tone.gamma_multiply(theme::CART_PROGRESS_ALPHA),
                marker: Some(tone),
                wave: tone.gamma_multiply(wave_alpha),
            }
        }
    }
}
```

- [ ] **Step 5: Draw it**

`crates/fp-app/src/ui/cartwall.rs`: import `cart_look` from `super::cart_view`. In `grid`, replace the `style` (L280-293) with:

```rust
                let look = cart_look(view.color, playing);
                let style = TileStyle {
                    fill: look.fill,
                    border: look.border,
                    content: theme::TEXT,
                    hover_fill: look.hover_fill,
                    hover_content: theme::TEXT,
                    active_fill: look.active_fill,
                    border_width: 1.0,
                    dim_when_disabled: true,
                };
```

In the paint closure, the progress bar's fill becomes `look.progress` (was `theme::ON_AIR_ROW.gamma_multiply(0.5)`), and the marker's last arm becomes `_ => look.marker.unwrap_or_else(|| kind_color(view.kind)),`.

- [ ] **Step 6: Run the tests to verify they pass**

Run: `cargo test -p fp-app --test theme --test cartwall_ui`
Expected: PASS (old and new).

- [ ] **Step 7: Gate and commit**

```bash
git add crates/fp-app/src/ui/theme.rs crates/fp-app/src/ui/cart_view.rs crates/fp-app/src/ui/cartwall.rs crates/fp-app/tests/theme.rs crates/fp-app/tests/cartwall_ui.rs
git commit -m "feat(ui): draw a cart in its colour

A cart with a colour has a border in the colour, a wash of it over the
button (7 % idle, 16 % playing), its kind marker in it, and the
remaining-time fill at 38 % while it plays. A cart without one is drawn
as before (cart colour spec CC6, CC7).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: The waveform behind the cart (CC8)

**Files:**
- Modify: `crates/fp-app/src/ui/cart_view.rs`, `crates/fp-app/src/ui/widgets.rs`, `crates/fp-app/src/ui/cartwall.rs`, `crates/fp-app/src/services.rs`
- Modify: `crates/fp-app/tests/support/mod.rs`
- Test: `crates/fp-app/tests/cartwall_ui.rs`, `crates/fp-app/tests/services.rs`

**Interfaces:**
- Consumes: `CartLook::wave` (Task 5); `widgets::{memo_columns_in, WaveMemo, WaveColumn}`; `MediaCache::{get, seed}`, `TrackMedia`.
- Produces:
  - `CartView::wave_range: Option<(f64, f64)>` — `(start_secs, span_secs)` of the play range for an idle or playing cart with a known positive length, else `None`.
  - `pub fn widgets::paint_wave_columns(painter: &Painter, rect: Rect, columns: &[WaveColumn], color: Color32)`.
  - `pub fn support::harness_with_media(state: AppState, media: MediaCache) -> (Harness<'static, AppUi>, Arc<Fake>)`.

- [ ] **Step 1: Add the harness helper (test support, no behaviour)**

In `crates/fp-app/tests/support/mod.rs`, replace `harness_sized` (L142-170) by:

```rust
/// As `harness_from`, in a window of `size`.
pub fn harness_sized(
    state: AppState,
    size: egui::Vec2,
    build: impl FnOnce(AppUi) -> AppUi,
) -> (Harness<'static, AppUi>, Arc<Fake>) {
    harness_built(state, size, MediaCache::default(), build)
}

/// As `harness`, with `media` as the peaks and covers the interface draws.
pub fn harness_with_media(
    state: AppState,
    media: MediaCache,
) -> (Harness<'static, AppUi>, Arc<Fake>) {
    harness_built(state, egui::vec2(1000.0, 700.0), media, |ui| ui)
}

fn harness_built(
    state: AppState,
    size: egui::Vec2,
    media: MediaCache,
    build: impl FnOnce(AppUi) -> AppUi,
) -> (Harness<'static, AppUi>, Arc<Fake>) {
    let fake = Fake::new(state);
    let ui = build(AppUi::new(fake.clone(), I18n::new(Some("en-US")), media));
    // Short frames, so that two clicks fall within the double-click delay.
    let mut harness = Harness::builder()
        .with_size(size)
        .with_step_dt(0.02)
        .build_ui_state(
            |ui, app: &mut AppUi| {
                // As eframe: `logic`, then `ui`.
                app.guard_close(ui.ctx());
                app.ui(ui);
            },
            ui,
        );
    harness.run_steps(2);
    (harness, fake)
}
```

- [ ] **Step 2: Write the failing tests**

`crates/fp-app/tests/cartwall_ui.rs`: add to the imports `use fp_app::services::{MediaCache, TrackMedia};` and `harness_with_media` to `use support::{…}`. Append:

```rust
/// Peaks for cart `index`'s track: 100 buckets of 0.1 s (the 10 s file of
/// `with_cart`/`give_file`), each at half scale with its RMS at a quarter.
fn peaks_for(s: &AppState, index: usize) -> MediaCache {
    let media = MediaCache::default();
    let track = s.cartwall.pages[0].carts[index].track.unwrap();
    media.seed(
        track,
        TrackMedia {
            peaks: vec![
                fp_analysis::WavePeak {
                    min: -16_000,
                    max: 16_000,
                    rms: 8_000,
                };
                100
            ],
            peak_bucket_secs: 0.1,
            cover_png: None,
        },
    );
    media
}

/// How many one-pixel columns in `color` the meshes painted on `label`'s
/// button hold.
fn wave_columns_on(h: &H, label: &str, color: Color32) -> usize {
    let button = h.get_by_label(label).rect();
    let mut quads = 0;
    for clipped in &h.output().shapes {
        if let egui::Shape::Mesh(mesh) = &clipped.shape
            && button.expand(0.5).contains_rect(mesh.calc_bounds())
        {
            quads += mesh.vertices.iter().filter(|v| v.color == color).count() / 4;
        }
    }
    quads
}

/// CC8 (view side): the play range, only for a cart that can play.
#[test]
fn cc8_the_waveform_spans_the_play_range_of_an_idle_or_playing_cart() {
    let mut s = with_cart();
    let id = s.cartwall.pages[0].carts[0].id;
    let view = |s: &AppState, id| cart_view(s, &Telemetry::default(), id).unwrap();
    assert_eq!(view(&s, id).wave_range, Some((0.5, 9.0)), "cue-in, length");
    apply(&mut s, Command::FireCart(id)).unwrap();
    assert_eq!(view(&s, id).wave_range, Some((0.5, 9.0)), "playing too");
    let empty = s.cartwall.pages[0].carts[1].id;
    assert_eq!(view(&s, empty).wave_range, None);
    let track = s.cartwall.pages[0].carts[0].track.unwrap();
    apply(
        &mut s,
        Command::SetFileState {
            track,
            state: FileState::Unreadable,
        },
    )
    .unwrap();
    assert_eq!(view(&s, id).wave_range, None, "unavailable");
    // No usable play range: nothing to span.
    let mut s = with_cart();
    s.library.get_mut(track).unwrap().duration_secs = 0.0;
    assert_eq!(view(&s, id).wave_range, None, "length unknown");
    let mut s = with_cart();
    apply(
        &mut s,
        Command::ApplyAnalysis {
            track,
            analysis: Box::new(TrackAnalysis {
                duration_secs: 10.0,
                cue_in: Some(4.0),
                cue_out: Some(4.0),
                ..TrackAnalysis::default()
            }),
        },
    )
    .unwrap();
    assert_eq!(view(&s, id).wave_range, None, "cue-in at cue-out");
}

/// CC8: in the cart's colour at 13 % idle, one column per pixel.
#[test]
fn cc8_a_cart_with_peaks_draws_its_waveform_in_its_colour() {
    let mut s = with_cart();
    paint_cart(&mut s, 0, Some(CartColor::Teal));
    let media = peaks_for(&s, 0);
    let (mut h, _fake) = harness_with_media(s, media);
    h.run_steps(2);
    let button = h.get_by_label("Station ID").rect();
    let wave = cart_look(Some(CartColor::Teal), false).wave;
    assert_eq!(
        wave_columns_on(&h, "Station ID", wave),
        (button.width() - 2.0) as usize,
        "one RMS column per pixel inside the border"
    );
}

#[test]
fn cc8_without_a_colour_it_is_light_grey_and_stronger_while_playing() {
    let mut s = with_cart();
    let media = peaks_for(&s, 0);
    let id = s.cartwall.pages[0].carts[0].id;
    apply(&mut s, Command::FireCart(id)).unwrap();
    let (mut h, _fake) = harness_with_media(s, media);
    h.run_steps(2);
    let playing = cart_look(None, true).wave;
    assert_eq!(playing, theme::CART_WAVE_PLAIN.gamma_multiply(0.22));
    assert!(wave_columns_on(&h, "Station ID", playing) > 0);
    assert_eq!(
        wave_columns_on(&h, "Station ID", cart_look(None, false).wave),
        0,
        "not the idle strength"
    );
}

#[test]
fn cc8_without_peaks_nothing_is_drawn() {
    let mut s = with_cart();
    paint_cart(&mut s, 0, Some(CartColor::Teal));
    let (mut h, _fake) = harness(s);
    h.run_steps(2);
    let wave = cart_look(Some(CartColor::Teal), false).wave;
    assert_eq!(wave_columns_on(&h, "Station ID", wave), 0, "not analysed yet");
}
```

`crates/fp-app/tests/services.rs`: append

```rust
/// Cart colour spec CC8: the carts of the page shown are what the screen
/// draws, so their peaks are kept; another page's are let go.
#[test]
fn peaks_are_kept_for_the_carts_of_the_page_shown() {
    let dir = tempfile::tempdir().unwrap();
    let song = wav(dir.path(), "song.wav", 1);
    let jingle = wav(dir.path(), "jingle.wav", 1);
    let mut r = rig_with(&[song], dir, Duration::ZERO, |state| {
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
    });
    let cart_track = r.handle.model.load().cartwall.pages[0].carts[0]
        .track
        .unwrap();
    r.run_until("peaks for the cart of the page shown", |r| {
        r.media.contains(cart_track)
    });
    r.handle.send(Command::CreateCartPage {
        name: "Other".into(),
    });
    r.run_until("the second page", |r| {
        r.handle.model.load().cartwall.pages.len() == 2
    });
    let other = r.handle.model.load().cartwall.pages[1].id;
    r.handle.send(Command::ShowCartPage(other));
    r.run_until("the first page's peaks let go", |r| {
        !r.media.contains(cart_track)
    });
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test -p fp-app --test cartwall_ui cc8 ; cargo test -p fp-app --test services peaks_are_kept_for_the_carts`
Expected: `cartwall_ui` FAILS to compile (`no field wave_range`); `services` FAILS by timing out in `run_until("peaks for the cart of the page shown")`.

- [ ] **Step 4: Implement `wave_range`**

`crates/fp-app/src/ui/cart_view.rs`: `CartView` gets

```rust
    /// The play range the waveform spans, `(start_secs, span_secs)` (cart
    /// colour spec CC8): only for an idle or playing cart whose length is
    /// known.
    pub wave_range: Option<(f64, f64)>,
```

initialised `wave_range: None,` in `cart_view`, and after `let known = …;`:

```rust
    if known && matches!(status, CartStatus::Idle | CartStatus::Playing) {
        view.wave_range = Some((track.cue_in_secs(), length));
    }
```

(`known` already requires `duration_secs > 0.0 && length > 0.0`, so cue-in at cue-out gives `None`.)

- [ ] **Step 5: Implement `paint_wave_columns`**

`crates/fp-app/src/ui/widgets.rs`, after `waveform` (it reuses the private `WAVE_PEAK_ALPHA`):

```rust
/// Paints `columns` across `rect`, one column per pixel from its left edge,
/// in one mesh: the peaks as a faint outline and the RMS level as the body,
/// in `color` (the cart waveform, cart colour spec CC8).
pub fn paint_wave_columns(painter: &Painter, rect: Rect, columns: &[WaveColumn], color: Color32) {
    let mid = rect.center().y;
    let amp = (rect.height() / 2.0 - 1.0).max(1.0);
    let mut mesh = egui::Mesh::default();
    for (i, column) in columns.iter().enumerate() {
        let px = rect.left() + i as f32;
        for (level, tone) in [
            (column.peak, color.gamma_multiply(WAVE_PEAK_ALPHA)),
            (column.rms, color),
        ] {
            let h = level * amp;
            if h > 0.0 {
                mesh.add_colored_rect(
                    Rect::from_min_max(pos2(px, mid - h), pos2(px + 1.0, mid + h)),
                    tone,
                );
            }
        }
    }
    if !mesh.is_empty() {
        painter.add(mesh);
    }
}
```

- [ ] **Step 6: Draw it on the cart**

`crates/fp-app/src/ui/cartwall.rs`: import `use super::widgets::WaveMemo;` (or reach it as `widgets::WaveMemo`). In `grid`, after `let look = cart_look(…);` and before `widgets::tile(…)`:

```rust
                // CC8: the play range's waveform, from the peaks the
                // services thread keeps for the page shown, reduced once
                // per width and remembered per cart.
                let memo_id = egui::Id::new(("cart-wave", cart.id));
                let wave = view.wave_range.and_then(|(start, span)| {
                    let media = cart
                        .track
                        .and_then(|t| scene.media.get(t))
                        .filter(|m| !m.peaks.is_empty())?;
                    let mut memo = ui
                        .data(|d| d.get_temp::<Option<WaveMemo>>(memo_id))
                        .flatten();
                    let columns = widgets::memo_columns_in(
                        &mut memo,
                        &media,
                        start,
                        span,
                        (width - 2.0).max(0.0) as usize,
                    );
                    ui.data_mut(|d| d.insert_temp(memo_id, memo));
                    Some(columns)
                });
                if wave.is_none() {
                    // Let go of the last file's peaks.
                    ui.data_mut(|d| d.remove_temp::<Option<WaveMemo>>(memo_id));
                }
```

In the paint closure, right after the progress bar block and before `let inner = …`:

```rust
                        if let Some(columns) = &wave {
                            let area = r.shrink(1.0);
                            widgets::paint_wave_columns(
                                &p.with_clip_rect(area),
                                area,
                                columns,
                                look.wave,
                            );
                        }
```

- [ ] **Step 7: Keep the shown page's peaks**

`crates/fp-app/src/services.rs`, `wanted` (L888-897):

```rust
    /// Tracks the screen draws: current, next and cue entries of the
    /// players, and the carts of the page shown (cart colour spec CC8).
    fn wanted(state: &AppState) -> HashSet<TrackId> {
        let players = state
            .players
            .iter()
            .flat_map(|p| [p.current, p.next, p.cue.map(|c| c.entry)])
            .flatten()
            .filter_map(|e| state.playlists.entry(e))
            .map(|e| e.track);
        let carts = state
            .cartwall
            .shown_page()
            .into_iter()
            .flat_map(|page| page.carts.iter())
            .filter_map(|cart| cart.track);
        players.chain(carts).collect()
    }
```

Update the comment above `self.media.retain` ("Peaks and covers are kept only for what the players show") to "…only for what the screen shows (the players and the cart page shown)".

- [ ] **Step 8: Run the tests to verify they pass**

Run: `cargo test -p fp-app --test cartwall_ui --test services --test waveform_view --test waveform_ui`
Expected: PASS. `a_cart_track_without_a_format_is_analysed_even_when_the_operator_said_later` still passes (only the cart's track jumps the queue: it is on the page shown).

- [ ] **Step 9: Gate and commit**

```bash
git add crates/fp-app/src/ui/cart_view.rs crates/fp-app/src/ui/widgets.rs crates/fp-app/src/ui/cartwall.rs crates/fp-app/src/services.rs crates/fp-app/tests/support/mod.rs crates/fp-app/tests/cartwall_ui.rs crates/fp-app/tests/services.rs
git commit -m "feat(ui): draw each cart's waveform behind its text

A cart whose analysis has peaks draws the waveform of its play range
across the button, one column per pixel, in its colour (light grey
without one) at 13 % idle and 22 % playing. The services thread keeps
the peaks of the cart page shown (cart colour spec CC8).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 7: The colour picker and its strings (CC5)

**Files:**
- Modify: `crates/fp-app/locales/{ca-ES,de-DE,en-US,es-ES,eu-ES,fr-FR,gl-ES,it-IT,nl-NL,pl-PL,pt-PT}/main.ftl`
- Modify: `crates/fp-app/src/ui/settings/carts.rs`
- Test: `crates/fp-app/tests/settings.rs`

**Interfaces:**
- Consumes: `CartColor::ALL`, `theme::cart_color`, `CartEdit::color`.
- Produces: Fluent keys `settings-cart-color`, `cart-color-none`, `cart-color-{red,orange,amber,green,teal,blue,violet,pink}`.

- [ ] **Step 1: Write the failing test**

`crates/fp-app/tests/settings.rs`: append

```rust
/// Cart colour spec CC5: nine swatches, the chosen one marked.
#[test]
fn cc5_the_colour_row_sets_and_clears_the_cart_colour() {
    use egui::accesskit::Toggled;
    use fp_model::CartColor;
    let (mut h, fake) = harness(with_cart());
    open_section(&mut h, "Cartwall");
    h.get_by_role_and_label(Role::Button, "Cart 1").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::RadioButton, "Pink")
        .scroll_to_me();
    h.run_steps(3);
    let toggled = |h: &egui_kittest::Harness<'static, fp_app::ui::app::AppUi>, name: &str| {
        h.get_by_role_and_label(Role::RadioButton, name)
            .accesskit_node()
            .toggled()
    };
    for name in [
        "No colour", "Red", "Orange", "Amber", "Green", "Teal", "Blue", "Violet", "Pink",
    ] {
        let expected = if name == "No colour" {
            Toggled::True
        } else {
            Toggled::False
        };
        assert_eq!(toggled(&h, name), Some(expected), "{name}");
    }
    h.get_by_role_and_label(Role::RadioButton, "Teal").click();
    h.run_steps(3);
    assert_eq!(
        fake.state.load().cartwall.pages[0].carts[0].color,
        Some(CartColor::Teal)
    );
    assert_eq!(toggled(&h, "Teal"), Some(Toggled::True));
    assert_eq!(toggled(&h, "No colour"), Some(Toggled::False));
    h.get_by_role_and_label(Role::RadioButton, "No colour").click();
    h.run_steps(3);
    assert_eq!(fake.state.load().cartwall.pages[0].carts[0].color, None);
    // Each swatch says its name on hover too.
    h.get_by_role_and_label(Role::RadioButton, "Violet").hover();
    h.run_steps(40);
    assert!(
        h.query_all_by_label("Violet").count() >= 2,
        "the tooltip repeats the name"
    );
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p fp-app --test settings cc5`
Expected: FAIL — panics in `get_by_role_and_label(Role::RadioButton, "Pink")` (no such node).

- [ ] **Step 3: Add the strings to all eleven locale files**

In each file, insert the block right after the line that starts with `settings-cart-kind =`:

`en-US`:
```ftl
settings-cart-color = Colour
cart-color-none = No colour
cart-color-red = Red
cart-color-orange = Orange
cart-color-amber = Amber
cart-color-green = Green
cart-color-teal = Teal
cart-color-blue = Blue
cart-color-violet = Violet
cart-color-pink = Pink
```

`es-ES`:
```ftl
settings-cart-color = Color
cart-color-none = Sin color
cart-color-red = Rojo
cart-color-orange = Naranja
cart-color-amber = Ámbar
cart-color-green = Verde
cart-color-teal = Turquesa
cart-color-blue = Azul
cart-color-violet = Violeta
cart-color-pink = Rosa
```

`ca-ES`:
```ftl
settings-cart-color = Color
cart-color-none = Sense color
cart-color-red = Vermell
cart-color-orange = Taronja
cart-color-amber = Ambre
cart-color-green = Verd
cart-color-teal = Turquesa
cart-color-blue = Blau
cart-color-violet = Violeta
cart-color-pink = Rosa
```

`de-DE`:
```ftl
settings-cart-color = Farbe
cart-color-none = Keine Farbe
cart-color-red = Rot
cart-color-orange = Orange
cart-color-amber = Bernstein
cart-color-green = Grün
cart-color-teal = Türkis
cart-color-blue = Blau
cart-color-violet = Violett
cart-color-pink = Rosa
```

`eu-ES`:
```ftl
settings-cart-color = Kolorea
cart-color-none = Kolorerik ez
cart-color-red = Gorria
cart-color-orange = Laranja
cart-color-amber = Anbarra
cart-color-green = Berdea
cart-color-teal = Turkesa
cart-color-blue = Urdina
cart-color-violet = Morea
cart-color-pink = Arrosa
```

`fr-FR`:
```ftl
settings-cart-color = Couleur
cart-color-none = Aucune couleur
cart-color-red = Rouge
cart-color-orange = Orange
cart-color-amber = Ambre
cart-color-green = Vert
cart-color-teal = Turquoise
cart-color-blue = Bleu
cart-color-violet = Violet
cart-color-pink = Rose
```

`gl-ES`:
```ftl
settings-cart-color = Cor
cart-color-none = Sen cor
cart-color-red = Vermello
cart-color-orange = Laranxa
cart-color-amber = Ámbar
cart-color-green = Verde
cart-color-teal = Turquesa
cart-color-blue = Azul
cart-color-violet = Violeta
cart-color-pink = Rosa
```

`it-IT`:
```ftl
settings-cart-color = Colore
cart-color-none = Nessun colore
cart-color-red = Rosso
cart-color-orange = Arancione
cart-color-amber = Ambra
cart-color-green = Verde
cart-color-teal = Turchese
cart-color-blue = Blu
cart-color-violet = Viola
cart-color-pink = Rosa
```

`nl-NL`:
```ftl
settings-cart-color = Kleur
cart-color-none = Geen kleur
cart-color-red = Rood
cart-color-orange = Oranje
cart-color-amber = Amber
cart-color-green = Groen
cart-color-teal = Turquoise
cart-color-blue = Blauw
cart-color-violet = Violet
cart-color-pink = Roze
```

`pl-PL`:
```ftl
settings-cart-color = Kolor
cart-color-none = Bez koloru
cart-color-red = Czerwony
cart-color-orange = Pomarańczowy
cart-color-amber = Bursztynowy
cart-color-green = Zielony
cart-color-teal = Turkusowy
cart-color-blue = Niebieski
cart-color-violet = Fioletowy
cart-color-pink = Różowy
```

`pt-PT`:
```ftl
settings-cart-color = Cor
cart-color-none = Sem cor
cart-color-red = Vermelho
cart-color-orange = Laranja
cart-color-amber = Âmbar
cart-color-green = Verde
cart-color-teal = Turquesa
cart-color-blue = Azul
cart-color-violet = Violeta
cart-color-pink = Rosa
```

Run: `cargo test -p fp-app --test i18n`
Expected: PASS (every locale defines exactly the English messages).

- [ ] **Step 4: Implement the picker**

`crates/fp-app/src/ui/settings/carts.rs`: imports become `use egui::{Color32, RichText, Stroke, StrokeKind, Ui, vec2};` and `use fp_model::{CartColor, CartEdit, CartKind, CartPageId, CartPageImport, Command};`. After the "Type" `row(…)` block (the one ending with the kind tiles), add:

```rust
    // CC5: no colour, then the palette.
    row(ui, &t.tr("settings-cart-color"), None, |ui| {
        ui.spacing_mut().item_spacing = vec2(4.0, 0.0);
        let choices = std::iter::once(None).chain(CartColor::ALL.into_iter().map(Some));
        for choice in choices {
            let name = t.tr(color_key(choice));
            if swatch(ui, choice, cart.color == choice, &name) && cart.color != choice {
                send_edit(CartEdit {
                    color: choice,
                    ..edit_base.clone()
                });
            }
        }
    });
```

and at the end of the file:

```rust
fn color_key(color: Option<CartColor>) -> &'static str {
    match color {
        None => "cart-color-none",
        Some(CartColor::Red) => "cart-color-red",
        Some(CartColor::Orange) => "cart-color-orange",
        Some(CartColor::Amber) => "cart-color-amber",
        Some(CartColor::Green) => "cart-color-green",
        Some(CartColor::Teal) => "cart-color-teal",
        Some(CartColor::Blue) => "cart-color-blue",
        Some(CartColor::Violet) => "cart-color-violet",
        Some(CartColor::Pink) => "cart-color-pink",
    }
}

/// One swatch of the colour row (CC5): a square of the colour, or a
/// crossed-out square for no colour; the chosen one is outlined. Its name
/// is the tooltip and the accessible label. Returns whether it was clicked.
fn swatch(ui: &mut Ui, color: Option<CartColor>, chosen: bool, name: &str) -> bool {
    const SIZE: f32 = 22.0;
    let (rect, response) = ui.allocate_exact_size(vec2(SIZE, SIZE), egui::Sense::click());
    let label = name.to_owned();
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::RadioButton, true, chosen, label.clone())
    });
    let response = response.on_hover_text(name);
    if ui.is_rect_visible(rect) {
        let p = ui.painter();
        let inner = rect.shrink(3.0);
        match color {
            Some(c) => p.rect_filled(inner, 0.0, theme::cart_color(c)),
            None => {
                p.rect_filled(inner, 0.0, theme::NEUTRAL_900);
                p.line_segment(
                    [inner.left_bottom(), inner.right_top()],
                    Stroke::new(1.0, theme::NEUTRAL_500),
                );
            }
        }
        let outline = if chosen {
            Stroke::new(2.0, theme::TEXT)
        } else if response.hovered() {
            Stroke::new(1.0, theme::NEUTRAL_500)
        } else {
            Stroke::new(1.0, theme::NEUTRAL_700)
        };
        p.rect_stroke(rect, 0.0, outline, StrokeKind::Inside);
    }
    response.clicked()
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p fp-app --test settings && cargo test -p fp-app --test i18n`
Expected: PASS. If the Cartwall section's layout tests (`settings_layout.rs`) report the editor overflowing, keep the swatches at 22 px and let the row wrap (`ui.horizontal_wrapped`) rather than shrinking them; note it in the ledger.

- [ ] **Step 6: Gate and commit**

```bash
git add crates/fp-app/locales crates/fp-app/src/ui/settings/carts.rs crates/fp-app/tests/settings.rs
git commit -m "feat(ui): choose a cart's colour in the cart editor

Settings -> Cartwall has a Colour row with nine swatches, no colour and
the eight colours, each named in its tooltip and accessible label; the
chosen one is outlined. Strings in every locale (cart colour spec CC5).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 8: User guide, remote API reference, technical docs, spec status

**Files:**
- Modify: `docs/user/cartwall.md`, `docs/user/remote-control.md`, `README.md`
- Modify: `docs/technical/remote-api.md`, `docs/technical/persistence.md`, `docs/technical/ui.md`, `docs/technical/analysis.md`
- Modify: `docs/superpowers/specs/2026-10-15-cart-colour-and-waveform-design.md` (status line)

No code; the "test" is the site checks and a reread against the spec.

- [ ] **Step 1: User guide, `docs/user/cartwall.md`**

In **Using it**, after the bullet "While a cart plays, its border turns red, …", replace that bullet with:

```markdown
- While a cart plays, its border turns red, a red bar shrinks as it plays,
  and its time counts down. A cart with a [colour](#setting-up-carts) keeps
  it: the border and the bar take the cart's colour instead of red.
- Once a cart's file has been analysed, its waveform shows faintly behind
  its name, so you can tell a sound that hits at once from one that fades
  in. It covers what the cart plays, from its cue-in to its cue-out.
```

In the **Button look** table, replace the first three rows with:

```markdown
| Violet / amber / grey square | Jingle / effect / spot (commercial). A cart with a colour shows its square in that colour |
| Coloured border and tint | The cart's colour, chosen in Settings |
```

In **Setting up carts**, the **Carts:** bullet becomes:

```markdown
- **Carts:** click a cart in the grid, then set its name, file (**Choose…**
  or **Clear**), type, **Colour**, **Loop** and **Stop other carts when
  fired**. **Colour** offers no colour and eight colours (red, orange,
  amber, green, teal, blue, violet, pink): give all the spots one colour
  and the station IDs another to find them at a glance. An empty cart
  keeps its colour, and clearing a cart's file does not remove it. A cart
  with a file also has **Exclude from normalization** and **Trim**, its
  track's [loudness normalization](loudness.md#per-track-exclude-and-trim)
  settings, with the measurement under the label.
```

and the **Import… / Export…** bullet gains at its end: "Colours travel with the page; a file from an older version imports without them."

- [ ] **Step 2: User guide, `docs/user/remote-control.md` and `README.md`**

`remote-control.md` L80: "set up a cart with a loaded track" becomes "set up a cart (name, type, colour, loop, exclusive) with a loaded track".

`README.md`, the **Cartwall** feature bullet: after "They have their own Main and Cue outputs." insert "Each cart can take one of eight colours and shows its waveform faintly behind its name."

- [ ] **Step 3: Remote API reference, `docs/technical/remote-api.md`**

The **Cart** DTO sentence (L91-93) becomes:

```markdown
**Cart:** `id`, `index` (row-major, 0-based in its page), `name`, `kind`
(`jingle`, `effect` or `spot`), `looped`, `exclusive`, `color` (`null` or
`red`, `orange`, `amber`, `green`, `teal`, `blue`, `violet`, `pink`),
`track` (`null` or a Track).
```

The `PUT /cartwall/pages/{id}/carts/{index}` row (L176) becomes:

```markdown
| `PUT /cartwall/pages/{id}/carts/{index}` | `{name, kind, looped, exclusive, track: id \| null, color?: name \| null}` | Set up a cart; `kind` is `jingle`, `effect` or `spot`. Keeping its track does not stop it; a new track or `null` does (C8). `color` is one of the Cart's colour names; `null` removes the colour and leaving it out keeps it; another value (an upper-case name included) is `400` (cart colour spec CC9) |
```

- [ ] **Step 4: Technical docs**

`docs/technical/persistence.md` L9 (`data/carts.json` row): "`carts[]` with id, name, track, kind, looped, exclusive)" becomes "`carts[]` with id, name, track, kind, looped, exclusive and `color`, a lower-case name left out when the cart has none; one that does not parse loads as none with a load warning (`store::parse_carts`))". The cart page bullet (L86-88) gains: "Each cart may carry `color` (a lower-case name, written only when set; an unknown one imports as none, reported through `parse_cart_page_noting`'s notes, which the import logs). The version stays 1: older readers ignore the field."

`docs/technical/ui.md` L28 (`ui/cartwall.rs`, `ui/cart_view.rs` row), append: "A button's colours come from the pure `cart_view::cart_look(color, playing)` (`CartLook`: fill, hover and pressed fills, border, remaining-time fill, kind marker, waveform), with the palette in `theme::cart_color` and the strengths in `theme::CART_*`. The waveform behind the text spans `CartView::wave_range` (the play range, idle and playing carts only), reduced with `widgets::memo_columns_in` into a memo kept in egui temp data per cart (`(\"cart-wave\", cart id)`) and drawn by `widgets::paint_wave_columns`. The cart editor's colour row (`settings/carts.rs`, `swatch`) exposes each swatch as a selected/unselected radio button."

`docs/technical/analysis.md` L293: "they are **shown** (current, next or cue on any player)" becomes "they are **shown** (current, next or cue on any player, or a cart of the cart page shown)"; L374-375: "are kept in the `MediaCache` only for shown tracks" stays and gains "(`Services::wanted`)".

- [ ] **Step 5: Spec status**

In `docs/superpowers/specs/2026-10-15-cart-colour-and-waveform-design.md`, the status line becomes:

```markdown
- **Status:** Approved (2026-10-15); implemented (plan
  [`2026-10-21-itr05-p7-cart-colour-waveform.md`](../plans/2026-10-21-itr05-p7-cart-colour-waveform.md)).
```

- [ ] **Step 6: Site checks and gate**

Run `scripts/site/check-translations.sh` only if it runs without network (a "changed since" report for `docs/user/cartwall.md` is expected and is not a failure: translations are regenerated before a release). Then the gate.

- [ ] **Step 7: Commit**

```bash
git add docs/user/cartwall.md docs/user/remote-control.md README.md docs/technical/remote-api.md docs/technical/persistence.md docs/technical/ui.md docs/technical/analysis.md docs/superpowers/specs/2026-10-15-cart-colour-and-waveform-design.md
git commit -m "docs: cart colour and waveform

The user guide (cartwall, remote control), the README, the remote API
reference (color in the cart DTO and edit), persistence (carts.json and
cart page files), the interface and analysis notes describe cart colours
and the cart waveform; the spec is marked implemented.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

## Self-review (against the spec)

- CC1: Task 1 (`CartColor`, names, default `None`, state stores the name) and Task 5 (`theme::cart_color`, the tones).
- CC2: Task 1 (`SetCart`, `EditCart`, C11 refusal, empty cart, cleared file); the editor sends it in Task 7.
- CC3: Task 1 (deserialisation), Task 2 (`carts.json` warning, the rest loads).
- CC4: Task 1 (model import), Task 3 (export, lenient import, notes logged, a file without colours).
- CC5: Task 7 (nine swatches, tooltip and accessible label, outline, strings in all locales).
- CC6, CC7: Task 5 (border, wash 7 %/16 %, marker, fill 38 %, uncoloured unchanged).
- CC8: Task 6 (peaks only, colour or light grey, 13 %/22 %, one column per pixel, empty and unanalysed draw nothing; peaks kept for the page shown).
- CC9: Task 4 (DTO name or `null`, set, `null`, unknown `400`, omitted keeps).
- §6 testing list: every item has a test in the task named above. §5 (out of scope): no custom colours, no page colour.
- Docs (CLAUDE.md "Documentation is part of every change"): Task 8; `docs/i18n` untouched; `CHANGELOG.md` untouched.

## Execution handoff

Executor: one Sonnet implementer subagent for the whole plan, then a fresh Opus review (superpowers:requesting-code-review on the whole branch; Critical and Important findings fixed test-first, Minor ones deferred in the ledger).
