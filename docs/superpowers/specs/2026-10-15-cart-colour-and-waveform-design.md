# Cart colour and waveform — Design Spec

- **Date:** 2026-10-15.
- **Status:** Approved (2026-10-15); implemented (plan
  [`2026-10-21-itr05-p7-cart-colour-waveform.md`](../plans/2026-10-21-itr05-p7-cart-colour-waveform.md)).
- **Amends:** [the cartwall spec](2026-09-26-phase2-cartwall-settings-design.md)
  (the cart's fields, `CartEdit`, C11, the cart page file) and
  [the remote control spec](2026-10-01-remote-control-design.md) (the cart
  DTO and cart edits).
- **Numbering:** rules are `CC<n>`. Each rule is testable and gets at least
  one test (§6).

---

## 1. Goal

Operators find a cart faster by colour than by reading its name: all
spots in one colour, the station IDs in another. Each cart gets an
optional colour from a fixed palette, shown discreetly on its button.
Each cart also shows its waveform faintly behind the text, so its shape
(a hit at the start, a long fade) is visible at a glance.

## 2. Model (`fp-model`)

- **CC1. Colour field.** `Cart` gains `color: Option<CartColor>`;
  `None` is "no colour", the default. `CartColor` is an enum with stable
  names: `Red`, `Orange`, `Amber`, `Green`, `Teal`, `Blue`, `Violet`,
  `Pink`. The state stores the name, never an RGB value, so the theme can
  change the tones without touching saved data.
- **CC2. Editing.** `CartEdit` gains `color: Option<CartColor>`. `SetCart`
  and `EditCart` set it with the other fields, atomically (C11). An empty
  cart keeps its colour; clearing a cart's file does not clear it.
- **CC3. Lenient loading.** A missing `color` loads as `None`; an
  unknown name loads as `None` with a log line, and the rest of the cart
  loads as usual.
- **CC4. Cart page files.** Export writes each cart's colour; import
  reads it with CC3's leniency. A file without colours imports as today.

## 3. Interface (`fp-app`)

- **CC5. Picker.** The cart edit dialog has a "Colour" row with nine
  swatches: the eight colours and "No colour", each with its localised
  name as tooltip and accessible label; the chosen one is outlined.
- **CC6. Idle cart.** A cart with a colour has a 1 px border in the full
  colour and an interior wash of that colour at 7 % over the cart
  background. The kind marker (the small square) takes the cart's colour.
  A cart without a colour is drawn as today.
- **CC7. Playing cart.** A playing cart with a colour has the border in
  the full colour, an interior wash at 16 %, and the remaining-time fill
  in the colour at 38 %, so the countdown stays visible. A playing cart
  without a colour is drawn as today (red).
- **CC8. Waveform.** A cart with a file whose analysis has peaks draws its
  waveform behind the text, across the whole button, in the cart's colour
  (light grey without a colour) at 13 % opacity idle and 22 % playing.
  Without peaks (not analysed yet, unreadable) or for an empty cart,
  nothing is drawn. It uses the same peak data and one column per pixel,
  like the players' waveform.
- The palette's sRGB tones live in `theme.rs` (`cart_color(CartColor)`):
  Red `#d8646a`, Orange `#e8925a`, Amber `#d9b45a`, Green `#7fb08a`,
  Teal `#4fb3a9`, Blue `#5b9fe0`, Violet `#9184d9`, Pink `#d27fb4`.
- Every new string goes into all locale files.

## 4. Remote control (`fp-remote`)

- **CC9.** The cart DTO has `color` (the lower-case name, or `null`).
  The cart edit operation accepts an optional `color` (name or `null`);
  an unknown name answers `400`. Omitting it keeps the colour.

## 5. Out of scope

- Custom colours (a colour picker or hex values).
- A colour per cart page.

## 6. Testing

- `fp-model`: CC1–CC4 (default, edit, empty-cart keep, unknown name,
  missing field, export/import round trip).
- `fp-remote`: CC9 (DTO, edit, `null`, unknown name `400`, omitted keeps).
- `fp-app`: `egui_kittest` tests of the picker (CC5), and of the drawn
  border and fill for idle and playing carts with and without a colour
  (CC6, CC7); the waveform drawn only with peaks (CC8).
