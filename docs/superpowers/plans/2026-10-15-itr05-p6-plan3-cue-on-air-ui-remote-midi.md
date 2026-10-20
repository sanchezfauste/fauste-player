# Pre-listen on an On-air Output, Plan 3: Interface, Remote, MIDI and Docs

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The operator sees and answers what plans 1 and 2 decide: a modal confirmation for a CA3 request (*Pre-listen anyway* / *Cancel*, Cancel the default), a status-bar notice when CA5 stops a pre-listen, a CUE tooltip and a Settings note for an on-air Cue output; the remote API refuses a CUE that would ask with `409 cue_on_air` and accepts `"force": true`; MIDI refuses it; the user guide, the remote API reference and the technical docs say so.

**Architecture:** The popup is drawn from the model (`AppState::cue_on_air.request`), by a new `fp-app/src/ui/cue_on_air.rs` (`show`, `notice_text`); `AppUi` remembers the id it answered (`ViewState::cue_answered`) so the popup closes at once while the conductor catches up, and the id of the last notice it showed (`ViewState::cue_notice_seen`). Esc and Enter cancel (`AppUi::keyboard`). Answers are `Command::ForceCue(request.start)` and `Command::CancelCueOnAir(request.id)`. `fp-remote` turns `Operation::{SetCue, CueEntry, SetCartCue}` into struct variants with `force: bool` and asks `fp_model::cue_on_air::{cue_start, needs_confirmation}`; `fp-control`'s router asks `needs_confirmation`.

**Tech Stack:** Rust 2024, egui + `egui_kittest`, Fluent, axum (`fp-remote`), `fp-control`.

**Spec:** `docs/superpowers/specs/2026-10-15-cue-on-air-output-design.md` (§5, CA6, §7 `fp-remote`/`fp-app`, §8 plan 3). Plans 1 and 2 must be on the branch first.

This is plan 3 of 3, executed strictly in order on branch `feat/cue-on-air-itr05`.

## Rulings (planning)

**Ruling: the popup and the notice say "Player 2", as the spec's text does, not the header's "P2".** §5 quotes *"…where Player 2 is playing…"* and *"Pre-listen stopped: Player 2 started on the same output."* — Cost if wrong: four strings per locale.

**Ruling: the popup's body lists every holder that sounds, joined with ", ".** "Player 2 is playing", "Player 3 is paused" (BR3.5 counts it), "Player 4 is pre-listening", "the cartwall is playing", "the cartwall is pre-listening". — Why: §5 gives one example; CA3's request carries the list. — Cost if wrong: one join.

**Ruling: Cancel reuses `exit-guard-cancel`, as Apply now's confirmation already does.** — Cost if wrong: one key.

**Ruling: the CA5 notice names a player or "a cart".** By CA5's ruling in plan 1 only a Main starts it: a player's Main ("Player 2 started…") or the cartwall's ("a cart started…"). — Cost if wrong: one string.

**Ruling: with `"force": true` the remote API always sends `ForceCue`, even on a quiet output.** CA4: "The same commands with `force: true` start the CUE … Such a CUE is **confirmed**." A client that forces wants a pre-listen CA5 will not cut. — Cost if wrong: one condition in `cue_on`.

**Ruling: OSC has no force.** The OSC CUE address takes one positional on/off argument; a CUE that would ask is refused with the code `cue_on_air`, which the OSC server already logs (at most one line per source per second). — Why: CA6 names the remote API's JSON body; OSC is as unattended as MIDI. — Cost if wrong: an optional second argument on `/fauste/player/{n}/cue`.

**Ruling: the keyboard shortcut for CUE asks like the button.** It is local: the operator sees the popup. — Cost if wrong: one `needs_confirmation` check in `AppUi::keyboard`.

**Ruling: the Settings note shows when the configured Cue route overlaps a configured Main route, or when the engine already places the Cue on air.** The first covers what the operator just chose; the second covers a missing Main route on the default output, which only the engine resolves. — Cost if wrong: the note misses that case until the operator applies.

## Global Constraints

- All code, identifiers, comments, docs and commit messages in English; never mention other playout or radio-automation products.
- CLAUDE.md rule 1: every new or changed string goes into **all eleven** locale files (`ca-ES de-DE en-US es-ES eu-ES fr-FR gl-ES it-IT nl-NL pl-PL pt-PT`); en-US is the source, es-ES is written with it, the other nine are AI translations made here.
- CLAUDE.md rule 8: the UI never blocks; it only reads snapshots and sends commands.
- §5 verbatim: "Cancel is the default (Escape and Enter cancel); Pre-listen anyway sends the request again with `force: true`. The same popup serves a player's CUE button, the cue window and cart pre-listen." "The warning under a Cue route that overlaps a Main route no longer says the CUE is off".
- CA6 verbatim: "The remote API and MIDI cannot answer a confirmation: CA3 from them is refused with the reason `cue_on_air` (remote: `409`). The remote API accepts `"force": true` on the CUE operations (CA4); MIDI has no force."
- UI tests use `egui_kittest` with the recording `Fake` controller (`crates/fp-app/tests/support`), which applies every command to an in-memory model as the conductor does.
- Remote tests drive the router with `tower::ServiceExt::oneshot` and `FakeControl` (`crates/fp-remote/tests/support`).
- The user guide is edited in English only (`docs/user`); `docs/i18n` is regenerated before a release, never here.
- Conventional Commits (scopes `ui`, `remote`, `control`, `docs`); every commit ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- `cargo` is at `$HOME/.cargo/bin`. Gate for every task: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.
- Progress ledger: `.superpowers/sdd/itr05/progress.md` (section P6, plan 3).

## Review Focus

1. **The output goes quiet while the popup is open** (the other player stops): the popup must close by itself and nothing must start. Test: Task 2, `the_question_goes_when_the_output_goes_quiet`.
2. **Answering faster than the conductor**: after *Cancel* the popup must not show again on the next frame because the snapshot still holds the request. Test: Task 2, `cancel_escape_and_enter_cancel` (the `Fake` applies at once; the `cue_answered` guard is what a real conductor needs — the test asserts the popup is gone after the answer frame and that exactly one command was sent).
3. **A remote client that sends `"force": "yes"`** or another unknown field: `400`, nothing queued. Test: Task 4, `force_must_be_a_boolean`.
4. **A MIDI CUE button that stops a running pre-listen** on a sounding on-air output: a stop never needs a confirmation and must still work. Test: Task 5, `ca6_a_cue_button_that_would_ask_does_nothing` (last assertion).
5. **The cart pre-listen from the cartwall menu** on a sounding on-air output: the same popup, and *Pre-listen anyway* forces the cart. Test: Task 2, `a_cart_pre_listen_asks_the_same_way`.

---

## File Structure

| File | Responsibility |
|---|---|
| `crates/fp-app/locales/*/main.ftl` (11 files) | new and changed strings |
| `crates/fp-app/src/ui/cue_on_air.rs` (new) | the popup (`show`), the notice text (`notice_text`) |
| `crates/fp-app/src/ui.rs` | `mod cue_on_air;` |
| `crates/fp-app/src/ui/app.rs` (`ViewState` L98-180, `ui` L556-…, the modals L895-940, `keyboard` L1137-1162) | wiring, Esc/Enter |
| `crates/fp-app/src/ui/pending.rs` (L20) | `player_number` becomes `pub(crate)` |
| `crates/fp-app/src/ui/player.rs` (L289-295) | the CUE tooltip |
| `crates/fp-app/src/ui/settings.rs` (L978-980, L1005-1007, L1025-1036) | the on-air note |
| `crates/fp-model/src/config.rs` | `cue_equals_main`, `player_cue_equals_main`, `cartwall_cue_equals_main` removed |
| `crates/fp-model/tests/cue_output.rs` | the test of `cue_equals_main` removed |
| `crates/fp-app/tests/support/mod.rs` | `on_desk` |
| `crates/fp-app/tests/cue_on_air_ui.rs` (new), `crates/fp-app/tests/settings.rs` (L946-959) | UI tests |
| `crates/fp-remote/src/api.rs`, `src/http/handlers.rs`, `src/osc.rs` | CA6, CA4 |
| `crates/fp-remote/tests/support/mod.rs`, `tests/api.rs`, `tests/http.rs`, `tests/osc.rs` | remote tests |
| `crates/fp-control/src/router.rs` (L104-110), `crates/fp-control/tests/router.rs` | MIDI |
| `docs/user/players.md`, `cartwall.md`, `settings.md`, `midi.md`, `remote-control.md`; `docs/technical/remote-api.md`, `ui.md` | Task 6 |

---

### Task 1: The strings, in every locale

**Files:**
- Modify: `crates/fp-app/locales/{ca-ES,de-DE,en-US,es-ES,eu-ES,fr-FR,gl-ES,it-IT,nl-NL,pl-PL,pt-PT}/main.ftl`, `crates/fp-app/src/ui/settings.rs` (L1031), `crates/fp-app/tests/settings.rs` (L946)
- Test: `crates/fp-app/tests/i18n.rs` (existing: `every_locale_defines_exactly_the_english_messages_and_variables`)

**Interfaces:**
- Produces the keys later tasks use: `cue-on-air-title`, `cue-on-air-body` (`$sounding`), `cue-on-air-confirm`, `cue-on-air-player-playing` / `-paused` / `-cue` (`$n`), `cue-on-air-cartwall`, `cue-on-air-cartwall-cue`, `cue-on-air-stopped-player` (`$n`), `cue-on-air-stopped-cartwall`, `tip-cue-on-air`, `settings-cue-on-air`; changed `tip-cue-no-output`, `tip-cart-cue-no-output`; removed `settings-cue-equals-main` (its one use is replaced in Task 3, so this task keeps `settings.rs` compiling by switching that call to `settings-cue-on-air` already).

- [ ] **Step 1: Write the strings with a one-off script**

Save as `/tmp/claude-…/scratchpad/cue_on_air_strings.py` (the session scratchpad, not the repo) and run it from the worktree root with `python3 -I <path>`:

```python
import pathlib
import sys

S = {
    "en-US": {
        "tip-cue-no-output": "This player has no Cue output: choose one in Settings › Audio outputs.",
        "tip-cart-cue-no-output": "The cartwall has no Cue output: choose one in Settings › Audio outputs.",
        "tip-cue-on-air": "Pre-listen (PFL) the next track. It goes out on the on-air output: while something plays there, it asks first.",
        "settings-cue-on-air": "This Cue output shares an on-air output: a pre-listen there is heard on air, and while something plays there it asks first.",
        "cue-on-air-title": "Pre-listen on air?",
        "cue-on-air-body": "This pre-listen would go out on the on-air output, where { $sounding }. It would be heard on air.",
        "cue-on-air-confirm": "Pre-listen anyway",
        "cue-on-air-player-playing": "Player { $n } is playing",
        "cue-on-air-player-paused": "Player { $n } is paused",
        "cue-on-air-player-cue": "Player { $n } is pre-listening",
        "cue-on-air-cartwall": "the cartwall is playing",
        "cue-on-air-cartwall-cue": "the cartwall is pre-listening",
        "cue-on-air-stopped-player": "Pre-listen stopped: Player { $n } started on the same output.",
        "cue-on-air-stopped-cartwall": "Pre-listen stopped: a cart started on the same output.",
    },
    "es-ES": {
        "tip-cue-no-output": "Este player no tiene salida Cue: elige una en Configuración › Salidas de audio.",
        "tip-cart-cue-no-output": "La cartuchera no tiene salida Cue: elige una en Configuración › Salidas de audio.",
        "tip-cue-on-air": "Preescucha (PFL) de la siguiente. Sale por la salida al aire: mientras algo suena allí, pregunta antes.",
        "settings-cue-on-air": "Esta salida Cue comparte una salida al aire: allí la preescucha se oye al aire y, mientras algo suena, pregunta antes.",
        "cue-on-air-title": "¿Preescucha al aire?",
        "cue-on-air-body": "Esta preescucha saldría por la salida al aire, donde { $sounding }. Se oiría al aire.",
        "cue-on-air-confirm": "Preescuchar igualmente",
        "cue-on-air-player-playing": "el player { $n } está sonando",
        "cue-on-air-player-paused": "el player { $n } está en pausa",
        "cue-on-air-player-cue": "el player { $n } está preescuchando",
        "cue-on-air-cartwall": "la cartuchera está sonando",
        "cue-on-air-cartwall-cue": "la cartuchera está preescuchando",
        "cue-on-air-stopped-player": "Preescucha detenida: el player { $n } ha empezado en la misma salida.",
        "cue-on-air-stopped-cartwall": "Preescucha detenida: un cartucho ha empezado en la misma salida.",
    },
    "ca-ES": {
        "tip-cue-no-output": "Aquest reproductor no té sortida Cue: tria'n una a Configuració › Sortides d'àudio.",
        "tip-cart-cue-no-output": "La cartutxera no té sortida Cue: tria'n una a Configuració › Sortides d'àudio.",
        "tip-cue-on-air": "Preescolta (PFL) de la pista següent. Surt per la sortida d'antena: mentre hi sona alguna cosa, pregunta abans.",
        "settings-cue-on-air": "Aquesta sortida Cue comparteix una sortida d'antena: la preescolta s'hi sent a l'antena i, mentre hi sona alguna cosa, pregunta abans.",
        "cue-on-air-title": "Preescolta a l'antena?",
        "cue-on-air-body": "Aquesta preescolta sortiria per la sortida d'antena, on { $sounding }. Se sentiria a l'antena.",
        "cue-on-air-confirm": "Preescoltar igualment",
        "cue-on-air-player-playing": "el reproductor { $n } està sonant",
        "cue-on-air-player-paused": "el reproductor { $n } està en pausa",
        "cue-on-air-player-cue": "el reproductor { $n } està preescoltant",
        "cue-on-air-cartwall": "la cartutxera està sonant",
        "cue-on-air-cartwall-cue": "la cartutxera està preescoltant",
        "cue-on-air-stopped-player": "Preescolta aturada: el reproductor { $n } ha començat a la mateixa sortida.",
        "cue-on-air-stopped-cartwall": "Preescolta aturada: un cartutx ha començat a la mateixa sortida.",
    },
    "de-DE": {
        "tip-cue-no-output": "Dieser Player hat keinen Cue-Ausgang: unter Einstellungen › Audioausgänge einen wählen.",
        "tip-cart-cue-no-output": "Die Cartwall hat keinen Cue-Ausgang: unter Einstellungen › Audioausgänge einen wählen.",
        "tip-cue-on-air": "Nächsten Track vorhören (PFL). Er geht über den Sendeausgang: Solange dort etwas spielt, wird zuerst gefragt.",
        "settings-cue-on-air": "Dieser Cue-Ausgang teilt sich einen Sendeausgang: Vorhören ist dort auf Sendung zu hören, und solange dort etwas spielt, wird zuerst gefragt.",
        "cue-on-air-title": "Vorhören auf Sendung?",
        "cue-on-air-body": "Dieses Vorhören ginge über den Sendeausgang, auf dem { $sounding }. Es wäre auf Sendung zu hören.",
        "cue-on-air-confirm": "Trotzdem vorhören",
        "cue-on-air-player-playing": "Player { $n } spielt",
        "cue-on-air-player-paused": "Player { $n } pausiert",
        "cue-on-air-player-cue": "Player { $n } vorhört",
        "cue-on-air-cartwall": "die Cartwall spielt",
        "cue-on-air-cartwall-cue": "die Cartwall vorhört",
        "cue-on-air-stopped-player": "Vorhören gestoppt: Player { $n } hat auf demselben Ausgang begonnen.",
        "cue-on-air-stopped-cartwall": "Vorhören gestoppt: Ein Cart hat auf demselben Ausgang begonnen.",
    },
    "eu-ES": {
        "tip-cue-no-output": "Erreproduzitzaile honek ez du Cue irteerarik: aukeratu bat Ezarpenak › Audio-irteerak atalean.",
        "tip-cart-cue-no-output": "Kartutxo-panelak ez du Cue irteerarik: aukeratu bat Ezarpenak › Audio-irteerak atalean.",
        "tip-cue-on-air": "Aurrez entzun (PFL) hurrengo pista. Aireko irteeratik ateratzen da: bertan zerbait jotzen ari den bitartean, lehenik galdetzen du.",
        "settings-cue-on-air": "Cue irteera honek aireko irteera bat partekatzen du: bertan aurrez entzutea airean entzuten da, eta zerbait jotzen ari den bitartean lehenik galdetzen du.",
        "cue-on-air-title": "Aurrez entzun airean?",
        "cue-on-air-body": "Aurrez entzute hau aireko irteeratik aterako litzateke; bertan: { $sounding }. Airean entzungo litzateke.",
        "cue-on-air-confirm": "Aurrez entzun hala ere",
        "cue-on-air-player-playing": "{ $n }. erreproduzitzailea jotzen ari da",
        "cue-on-air-player-paused": "{ $n }. erreproduzitzailea pausan dago",
        "cue-on-air-player-cue": "{ $n }. erreproduzitzailea aurrez entzuten ari da",
        "cue-on-air-cartwall": "kartutxo-panela jotzen ari da",
        "cue-on-air-cartwall-cue": "kartutxo-panela aurrez entzuten ari da",
        "cue-on-air-stopped-player": "Aurrez entzutea geldituta: { $n }. erreproduzitzailea irteera berean hasi da.",
        "cue-on-air-stopped-cartwall": "Aurrez entzutea geldituta: kartutxo bat irteera berean hasi da.",
    },
    "fr-FR": {
        "tip-cue-no-output": "Ce lecteur n'a pas de sortie Cue : choisissez-en une dans Paramètres › Sorties audio.",
        "tip-cart-cue-no-output": "Le mur de cartouches n'a pas de sortie Cue : choisissez-en une dans Paramètres › Sorties audio.",
        "tip-cue-on-air": "Pré-écoute (PFL) de la piste suivante. Elle sort sur la sortie antenne : tant que quelque chose y joue, elle demande d'abord.",
        "settings-cue-on-air": "Cette sortie Cue partage une sortie antenne : la pré-écoute s'y entend à l'antenne et, tant que quelque chose y joue, elle demande d'abord.",
        "cue-on-air-title": "Pré-écoute à l'antenne ?",
        "cue-on-air-body": "Cette pré-écoute sortirait sur la sortie antenne, où { $sounding }. Elle s'entendrait à l'antenne.",
        "cue-on-air-confirm": "Pré-écouter quand même",
        "cue-on-air-player-playing": "le lecteur { $n } joue",
        "cue-on-air-player-paused": "le lecteur { $n } est en pause",
        "cue-on-air-player-cue": "le lecteur { $n } est en pré-écoute",
        "cue-on-air-cartwall": "le mur de cartouches joue",
        "cue-on-air-cartwall-cue": "le mur de cartouches est en pré-écoute",
        "cue-on-air-stopped-player": "Pré-écoute arrêtée : le lecteur { $n } a démarré sur la même sortie.",
        "cue-on-air-stopped-cartwall": "Pré-écoute arrêtée : une cartouche a démarré sur la même sortie.",
    },
    "gl-ES": {
        "tip-cue-no-output": "Este reprodutor non ten saída Cue: escolle unha en Configuración › Saídas de audio.",
        "tip-cart-cue-no-output": "A cartucheira non ten saída Cue: escolle unha en Configuración › Saídas de audio.",
        "tip-cue-on-air": "Preescoita (PFL) da seguinte pista. Sae pola saída de antena: mentres algo soa alí, pregunta antes.",
        "settings-cue-on-air": "Esta saída Cue comparte unha saída de antena: alí a preescoita óese en antena e, mentres algo soa, pregunta antes.",
        "cue-on-air-title": "Preescoita en antena?",
        "cue-on-air-body": "Esta preescoita sairía pola saída de antena, onde { $sounding }. Oiríase en antena.",
        "cue-on-air-confirm": "Preescoitar igualmente",
        "cue-on-air-player-playing": "o reprodutor { $n } está soando",
        "cue-on-air-player-paused": "o reprodutor { $n } está en pausa",
        "cue-on-air-player-cue": "o reprodutor { $n } está preescoitando",
        "cue-on-air-cartwall": "a cartucheira está soando",
        "cue-on-air-cartwall-cue": "a cartucheira está preescoitando",
        "cue-on-air-stopped-player": "Preescoita detida: o reprodutor { $n } comezou na mesma saída.",
        "cue-on-air-stopped-cartwall": "Preescoita detida: un cartucho comezou na mesma saída.",
    },
    "it-IT": {
        "tip-cue-no-output": "Questo lettore non ha un'uscita Cue: scegline una in Impostazioni › Uscite audio.",
        "tip-cart-cue-no-output": "La cartwall non ha un'uscita Cue: scegline una in Impostazioni › Uscite audio.",
        "tip-cue-on-air": "Preascolto (PFL) del brano successivo. Esce sull'uscita in onda: finché lì suona qualcosa, chiede prima.",
        "settings-cue-on-air": "Questa uscita Cue condivide un'uscita in onda: lì il preascolto si sente in onda e, finché qualcosa suona, chiede prima.",
        "cue-on-air-title": "Preascolto in onda?",
        "cue-on-air-body": "Questo preascolto uscirebbe sull'uscita in onda, dove { $sounding }. Si sentirebbe in onda.",
        "cue-on-air-confirm": "Preascolta comunque",
        "cue-on-air-player-playing": "il lettore { $n } sta suonando",
        "cue-on-air-player-paused": "il lettore { $n } è in pausa",
        "cue-on-air-player-cue": "il lettore { $n } è in preascolto",
        "cue-on-air-cartwall": "la cartwall sta suonando",
        "cue-on-air-cartwall-cue": "la cartwall è in preascolto",
        "cue-on-air-stopped-player": "Preascolto fermato: il lettore { $n } è partito sulla stessa uscita.",
        "cue-on-air-stopped-cartwall": "Preascolto fermato: una cartuccia è partita sulla stessa uscita.",
    },
    "nl-NL": {
        "tip-cue-no-output": "Deze speler heeft geen Cue-uitgang: kies er een in Instellingen › Audio-uitgangen.",
        "tip-cart-cue-no-output": "De cartwall heeft geen Cue-uitgang: kies er een in Instellingen › Audio-uitgangen.",
        "tip-cue-on-air": "Het volgende nummer voorbeluisteren (PFL). Het gaat via de on-air-uitgang: zolang daar iets speelt, wordt eerst gevraagd.",
        "settings-cue-on-air": "Deze Cue-uitgang deelt een on-air-uitgang: voorbeluisteren is daar on air te horen, en zolang daar iets speelt, wordt eerst gevraagd.",
        "cue-on-air-title": "Voorbeluisteren on air?",
        "cue-on-air-body": "Dit voorbeluisteren zou via de on-air-uitgang gaan, waar { $sounding }. Het zou on air te horen zijn.",
        "cue-on-air-confirm": "Toch voorbeluisteren",
        "cue-on-air-player-playing": "speler { $n } speelt",
        "cue-on-air-player-paused": "speler { $n } gepauzeerd is",
        "cue-on-air-player-cue": "speler { $n } voorbeluistert",
        "cue-on-air-cartwall": "de cartwall speelt",
        "cue-on-air-cartwall-cue": "de cartwall voorbeluistert",
        "cue-on-air-stopped-player": "Voorbeluisteren gestopt: speler { $n } is op dezelfde uitgang gestart.",
        "cue-on-air-stopped-cartwall": "Voorbeluisteren gestopt: een cart is op dezelfde uitgang gestart.",
    },
    "pl-PL": {
        "tip-cue-no-output": "Ten odtwarzacz nie ma wyjścia Cue: wybierz je w Ustawienia › Wyjścia audio.",
        "tip-cart-cue-no-output": "Cartwall nie ma wyjścia Cue: wybierz je w Ustawienia › Wyjścia audio.",
        "tip-cue-on-air": "Odsłuch wstępny (PFL) następnego utworu. Wychodzi wyjściem antenowym: gdy coś tam gra, najpierw pyta.",
        "settings-cue-on-air": "To wyjście Cue dzieli wyjście antenowe: odsłuch wstępny jest tam słyszalny na antenie, a gdy coś tam gra, najpierw pyta.",
        "cue-on-air-title": "Odsłuch wstępny na antenie?",
        "cue-on-air-body": "Ten odsłuch wstępny wyszedłby wyjściem antenowym, gdzie { $sounding }. Byłby słyszalny na antenie.",
        "cue-on-air-confirm": "Odsłuchaj mimo to",
        "cue-on-air-player-playing": "odtwarzacz { $n } gra",
        "cue-on-air-player-paused": "odtwarzacz { $n } jest wstrzymany",
        "cue-on-air-player-cue": "odtwarzacz { $n } prowadzi odsłuch wstępny",
        "cue-on-air-cartwall": "cartwall gra",
        "cue-on-air-cartwall-cue": "cartwall prowadzi odsłuch wstępny",
        "cue-on-air-stopped-player": "Odsłuch wstępny zatrzymany: odtwarzacz { $n } wystartował na tym samym wyjściu.",
        "cue-on-air-stopped-cartwall": "Odsłuch wstępny zatrzymany: cart wystartował na tym samym wyjściu.",
    },
    "pt-PT": {
        "tip-cue-no-output": "Este leitor não tem saída Cue: escolha uma em Definições › Saídas de áudio.",
        "tip-cart-cue-no-output": "A cartucheira não tem saída Cue: escolha uma em Definições › Saídas de áudio.",
        "tip-cue-on-air": "Pré-escuta (PFL) da faixa seguinte. Sai pela saída do ar: enquanto algo toca lá, pergunta primeiro.",
        "settings-cue-on-air": "Esta saída Cue partilha uma saída do ar: lá a pré-escuta ouve-se no ar e, enquanto algo toca, pergunta primeiro.",
        "cue-on-air-title": "Pré-escuta no ar?",
        "cue-on-air-body": "Esta pré-escuta sairia pela saída do ar, onde { $sounding }. Ouvir-se-ia no ar.",
        "cue-on-air-confirm": "Pré-escutar mesmo assim",
        "cue-on-air-player-playing": "o leitor { $n } está a tocar",
        "cue-on-air-player-paused": "o leitor { $n } está em pausa",
        "cue-on-air-player-cue": "o leitor { $n } está em pré-escuta",
        "cue-on-air-cartwall": "a cartucheira está a tocar",
        "cue-on-air-cartwall-cue": "a cartucheira está em pré-escuta",
        "cue-on-air-stopped-player": "Pré-escuta parada: o leitor { $n } começou na mesma saída.",
        "cue-on-air-stopped-cartwall": "Pré-escuta parada: um cartucho começou na mesma saída.",
    },
}

REPLACED = ("tip-cue-no-output", "tip-cart-cue-no-output")
for tag, s in S.items():
    path = pathlib.Path(f"crates/fp-app/locales/{tag}/main.ftl")
    out, seen = [], set()
    for line in path.read_text(encoding="utf-8").splitlines():
        key = line.split(" = ", 1)[0]
        if key in REPLACED:
            out.append(f"{key} = {s[key]}")
            seen.add(key)
            if key == "tip-cart-cue-no-output":
                out.append(f"tip-cue-on-air = {s['tip-cue-on-air']}")
        elif key == "settings-cue-equals-main":
            out.append(f"settings-cue-on-air = {s['settings-cue-on-air']}")
            seen.add(key)
        else:
            out.append(line)
    missing = {*REPLACED, "settings-cue-equals-main"} - seen
    if missing:
        sys.exit(f"{tag}: {sorted(missing)} not found")
    out += ["", "# Pre-listen on an on-air output"]
    out += [f"{k} = {v}" for k, v in s.items() if k.startswith("cue-on-air-")]
    path.write_text("\n".join(out) + "\n", encoding="utf-8")
    print(tag, "ok")
```

Expected output: eleven `… ok` lines.

- [ ] **Step 2: Keep `settings.rs` and its test in step**

In `crates/fp-app/src/ui/settings.rs` L1031, `scene.i18n.tr("settings-cue-equals-main")` becomes `scene.i18n.tr("settings-cue-on-air")` (Task 3 renames the function and changes when it shows). In `crates/fp-app/tests/settings.rs` L946, the constant becomes `const CUE_EQUALS_MAIN: &str = "This Cue output shares an on-air output";` (the note still shows for equal routes only; Task 3 renames the test and widens it).

- [ ] **Step 3: Run the locale parity test and the UI tests that read the changed texts**

Run: `cargo test -p fp-app --test i18n && cargo test -p fp-app --test cue_output_ui --test cartwall_ui --test settings`
Expected: PASS.

- [ ] **Step 4: Gate and commit**

Run the gate. Then:

```bash
git add crates/fp-app/locales crates/fp-app/src/ui/settings.rs crates/fp-app/tests/settings.rs
git commit -m "feat(ui): strings for pre-listen on an on-air output

On-air pre-listen spec §5: the confirmation, the notice, the CUE
tooltip and the Settings note, in every locale (en-US and es-ES
written, the others AI-translated). The no-output tooltips no longer
ask for a Cue output apart from Main (CA1).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 2: The confirmation popup

**Files:**
- Create: `crates/fp-app/src/ui/cue_on_air.rs`
- Modify: `crates/fp-app/src/ui.rs`, `crates/fp-app/src/ui/app.rs`, `crates/fp-app/src/ui/pending.rs` (L20)
- Modify: `crates/fp-app/tests/support/mod.rs`
- Test: `crates/fp-app/tests/cue_on_air_ui.rs` (new)

**Interfaces:**
- Consumes: `AppState::cue_on_air.request: Option<CueConfirmation { id, holder, start, sounding }>`, `Command::ForceCue(CueStart)`, `Command::CancelCueOnAir(u64)` (plan 1).
- Produces: `pub(crate) fn cue_on_air::show(ctx: &egui::Context, scene: &Scene<'_>, request: &CueConfirmation) -> Option<bool>` (`Some(true)` pre-listen anyway, `Some(false)` cancel); `ViewState::cue_answered: Option<u64>`; test helper `support::on_desk(state: &mut AppState, holders: &[Holder])`.

- [ ] **Step 1: The test helper**

Append to `crates/fp-app/tests/support/mod.rs`:

```rust
/// Routes `holders` to one `desk` output (channels 1–2) and reports the
/// engine placing them there (live settings L3): a Cue among them is on
/// air next to a Main among them (on-air pre-listen spec §2).
pub fn on_desk(state: &mut AppState, holders: &[fp_model::Holder]) {
    use fp_model::Holder;
    let desk = Route {
        backend: "null".to_owned(),
        device: "desk".to_owned(),
        first_channel: 0,
    };
    let device = fp_model::OutputDevice {
        backend: "null".to_owned(),
        device: "desk".to_owned(),
    };
    for holder in holders {
        let outputs = &mut state.config.outputs;
        match *holder {
            Holder::PlayerMain(p) | Holder::PlayerCue(p) => {
                if let Some(r) = outputs.routes.iter_mut().find(|r| r.player == p) {
                    if matches!(holder, Holder::PlayerMain(_)) {
                        r.main = Some(desk.clone());
                    } else {
                        r.cue = Some(desk.clone());
                    }
                }
            }
            Holder::CartwallMain => outputs.cartwall.main = Some(desk.clone()),
            Holder::CartwallCue => outputs.cartwall.cue = Some(desk.clone()),
        }
    }
    let running = fp_model::device_settings(&state.config.outputs, &device);
    for holder in holders {
        fp_model::on_event(
            state,
            fp_model::EngineEvent::Placed {
                holder: *holder,
                route: Some(desk.clone()),
                device: device.clone(),
                running,
            },
        );
    }
}
```

- [ ] **Step 2: The tests**

Create `crates/fp-app/tests/cue_on_air_ui.rs`:

```rust
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! On-air pre-listen spec §5: the confirmation, the notice and the CUE
//! tooltip.

mod support;

use std::path::PathBuf;
use std::sync::Arc;

use egui::Key;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use fp_app::ui::app::AppUi;
use fp_model::{AppState, Command, CueStart, Holder};
use support::{Fake, harness, on_desk, state};

const TITLE: &str = "Pre-listen on air?";
const BODY_P2: &str = "This pre-listen would go out on the on-air output, where Player 2 is playing. It would be heard on air.";
const ANYWAY: &str = "Pre-listen anyway";
const ON_AIR_TIP: &str = "Pre-listen (PFL) the next track. It goes out on the on-air output: while something plays there, it asks first.";
const STOPPED_P2: &str = "Pre-listen stopped: Player 2 started on the same output.";

/// Two players over three tracks: player 1's Cue and player 2's Main share
/// `desk`; player 1's Main stays on the default output. Nothing plays.
fn shared() -> AppState {
    let mut s = state(2, 3);
    let (p1, p2) = (s.players[0].id, s.players[1].id);
    on_desk(&mut s, &[Holder::PlayerCue(p1), Holder::PlayerMain(p2)]);
    s
}

/// `shared`, with player 2 playing.
fn sounding() -> AppState {
    let mut s = shared();
    let p2 = s.players[1].id;
    fp_model::apply(&mut s, Command::Play(p2)).unwrap();
    s
}

/// `sounding`, after a click on player 1's CUE.
fn asked() -> (Harness<'static, AppUi>, Arc<Fake>) {
    let (mut h, fake) = harness(sounding());
    h.get_by_label(ON_AIR_TIP).click();
    h.run_steps(2);
    fake.take_sent();
    (h, fake)
}

fn cueing(fake: &Fake, n: usize) -> bool {
    fake.state.load().player(fake.player(n)).unwrap().cue.is_some()
}

#[test]
fn the_cue_button_of_an_on_air_cue_says_where_it_goes() {
    let (h, _) = harness(shared());
    assert_eq!(h.query_all_by_label(ON_AIR_TIP).count(), 1, "player 1 only");
    assert!(!h.get_by_label(ON_AIR_TIP).accesskit_node().is_disabled());
}

#[test]
fn on_a_quiet_on_air_output_the_cue_starts_without_asking() {
    let (mut h, fake) = harness(shared());
    h.get_by_label(ON_AIR_TIP).click();
    h.run_steps(2);
    assert!(h.query_by_label(TITLE).is_none());
    assert_eq!(fake.take_sent(), vec![Command::ToggleCue(fake.player(0))]);
    assert!(cueing(&fake, 0));
}

#[test]
fn on_a_sounding_on_air_output_the_cue_asks_first() {
    let (h, fake) = asked();
    assert!(h.query_by_label(TITLE).is_some());
    assert!(h.query_by_label(BODY_P2).is_some());
    assert!(!cueing(&fake, 0), "nothing started");
}

#[test]
fn pre_listen_anyway_sends_the_request_forced() {
    let (mut h, fake) = asked();
    let start = fake.state.load().cue_on_air.request.clone().unwrap().start;
    h.get_by_label(ANYWAY).click();
    h.run_steps(2);
    assert_eq!(fake.take_sent(), vec![Command::ForceCue(start)]);
    assert!(h.query_by_label(TITLE).is_none());
    assert!(cueing(&fake, 0));
}

/// Review Focus 2.
#[test]
fn cancel_escape_and_enter_cancel() {
    for how in ["button", "escape", "enter"] {
        let (mut h, fake) = asked();
        let id = fake.state.load().cue_on_air.request.clone().unwrap().id;
        match how {
            "button" => h.get_by_label("Cancel").click(),
            "escape" => h.key_press(Key::Escape),
            _ => h.key_press(Key::Enter),
        }
        h.run_steps(2);
        assert_eq!(fake.take_sent(), vec![Command::CancelCueOnAir(id)], "{how}");
        assert!(h.query_by_label(TITLE).is_none(), "{how}");
        assert!(!cueing(&fake, 0), "{how}");
    }
}

/// Review Focus 1.
#[test]
fn the_question_goes_when_the_output_goes_quiet() {
    let (mut h, fake) = asked();
    fake.send(Command::Stop(fake.player(1)));
    fake.take_sent();
    h.run_steps(2);
    assert!(h.query_by_label(TITLE).is_none());
    assert!(fake.take_sent().is_empty(), "nothing starts by itself");
    assert!(!cueing(&fake, 0));
}

/// Review Focus 5.
#[test]
fn a_cart_pre_listen_asks_the_same_way() {
    let mut s = sounding();
    let page = s.cartwall.pages[0].id;
    fp_model::apply(
        &mut s,
        Command::AssignCartFile {
            page,
            index: 0,
            path: PathBuf::from("/carts/Station ID.wav"),
        },
    )
    .unwrap();
    let cart = s.cartwall.pages[0].carts[0].id;
    on_desk(&mut s, &[Holder::CartwallCue]);
    let (mut h, fake) = harness(s);
    fake.send(Command::CueCart(cart));
    fake.take_sent();
    h.run_steps(2);
    assert!(h.query_by_label(BODY_P2).is_some());
    h.get_by_label(ANYWAY).click();
    h.run_steps(2);
    assert_eq!(fake.take_sent(), vec![Command::ForceCue(CueStart::Cart(cart))]);
    assert_eq!(fake.state.load().cartwall.cue, Some(cart));
}
```

- [ ] **Step 3: Run them to see them fail**

Run: `cargo test -p fp-app --test cue_on_air_ui`
Expected: FAIL — no CUE tile carries `ON_AIR_TIP` yet (every test finds the button by that label), and no popup. Steps 4 and 5 add both.

- [ ] **Step 4: The popup module**

Create `crates/fp-app/src/ui/cue_on_air.rs`:

```rust
//! Pre-listen on an on-air output (on-air pre-listen spec §5): the
//! confirmation a CA3 request asks for, and the notice CA5 leaves. The
//! rules are `fp_model::cue_on_air`; this module only describes and asks.

use egui::{RichText, vec2};
use fp_model::{AppState, CueConfirmation, CueStopped, Holder, Transport};

use super::app::Scene;
use super::exit_guard::button;
use super::pending::player_number;
use super::theme;
use super::widgets::{font, font_medium};
use crate::i18n::I18n;

/// What `holder` is doing on the output, for the question's body.
fn sounding_text(t: &I18n, state: &AppState, holder: Holder) -> String {
    let n = |p| player_number(state, p);
    match holder {
        Holder::PlayerMain(p) => {
            let paused = state
                .player(p)
                .is_ok_and(|pl| pl.transport == Transport::Paused);
            let key = if paused {
                "cue-on-air-player-paused"
            } else {
                "cue-on-air-player-playing"
            };
            t.tr_args(key, &[("n", n(p).into())])
        }
        Holder::PlayerCue(p) => t.tr_args("cue-on-air-player-cue", &[("n", n(p).into())]),
        Holder::CartwallMain => t.tr("cue-on-air-cartwall"),
        Holder::CartwallCue => t.tr("cue-on-air-cartwall-cue"),
    }
}

/// CA5's notice: who started on the pre-listen's output.
pub(crate) fn notice_text(t: &I18n, state: &AppState, stopped: &CueStopped) -> String {
    match stopped.by {
        Holder::PlayerMain(p) | Holder::PlayerCue(p) => t.tr_args(
            "cue-on-air-stopped-player",
            &[("n", player_number(state, p).into())],
        ),
        Holder::CartwallMain | Holder::CartwallCue => t.tr("cue-on-air-stopped-cartwall"),
    }
}

/// The question for `request`. `Some(true)` is *Pre-listen anyway*,
/// `Some(false)` is *Cancel* (the button or the backdrop; Esc and Enter are
/// answered by `AppUi::keyboard`), `None` keeps it open.
pub(crate) fn show(
    ctx: &egui::Context,
    scene: &Scene<'_>,
    request: &CueConfirmation,
) -> Option<bool> {
    let (t, state) = (scene.i18n, scene.state);
    let sounding = request
        .sounding
        .iter()
        .map(|h| sounding_text(t, state, *h))
        .collect::<Vec<_>>()
        .join(", ");
    let body = t.tr_args("cue-on-air-body", &[("sounding", sounding.into())]);
    let width = (ctx.content_rect().width() - 48.0).clamp(280.0, 440.0);
    let mut answer = None;
    let modal = egui::Modal::new(egui::Id::new("cue-on-air"))
        .frame(egui::Frame::new().fill(theme::SURFACE).inner_margin(20.0))
        .backdrop_color(theme::NEUTRAL_900.gamma_multiply(0.7))
        .show(ctx, |ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            ui.add(
                egui::Label::new(
                    RichText::new(t.tr("cue-on-air-title"))
                        .font(font_medium(18.0))
                        .color(theme::TEXT),
                )
                .selectable(false),
            );
            ui.add(
                egui::Label::new(
                    RichText::new(body.clone())
                        .font(font(12.0))
                        .color(theme::NEUTRAL_300),
                )
                .selectable(false)
                .wrap(),
            );
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if button(ui, &t.tr("exit-guard-cancel"), None) {
                    answer = Some(false);
                }
                if button(ui, &t.tr("cue-on-air-confirm"), Some(theme::ON_AIR_ROW)) {
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

Register it in `crates/fp-app/src/ui.rs` (`mod cue_on_air;` after `mod cartwall;`), and make `player_number` in `crates/fp-app/src/ui/pending.rs` (L20) `pub(crate) fn player_number`.

- [ ] **Step 5: Wire it in `app.rs`**

1. `ViewState` (after `confirm_apply_now`):

```rust
    /// The on-air pre-listen request the operator answered (its id): not
    /// shown again while the snapshot still holds it (on-air pre-listen
    /// spec §5).
    pub cue_answered: Option<u64>,
    /// The id of the last on-air pre-listen notice (CA5) put in the status
    /// bar.
    pub cue_notice_seen: Option<u64>,
```

2. An accessor on `AppUi` (next to `keyboard`):

```rust
    /// The pending on-air pre-listen request the operator has not answered.
    fn cue_question<'a>(&self, state: &'a AppState) -> Option<&'a fp_model::CueConfirmation> {
        state
            .cue_on_air
            .request
            .as_ref()
            .filter(|r| self.view.cue_answered != Some(r.id))
    }
```

3. `keyboard`, right after the close guard's block (before "Apply now's confirmation, then the pending panel…"):

```rust
        // On-air pre-listen spec §5: the question owns the keyboard; Esc
        // and Enter cancel (Cancel is the default), and no shortcut acts.
        if let Some(id) = self.cue_question(state).map(|r| r.id) {
            let cancel = ctx.input_mut(|i| {
                i.consume_key(egui::Modifiers::NONE, Key::Escape)
                    | i.consume_key(egui::Modifiers::NONE, Key::Enter)
            });
            if cancel {
                self.view.cue_answered = Some(id);
                self.ctl.send(Command::CancelCueOnAir(id));
            }
            return;
        }
```

(`|`, not `||`: both keys are consumed in the frame either is pressed.)

4. In `ui`, right before "The guard takes precedence over Settings and About" (L914):

```rust
        // On-air pre-listen spec §5: a CA3 request asks the operator.
        if let Some(request) = self.cue_question(&state).cloned() {
            match cue_on_air::show(&ctx, &scene, &request) {
                Some(true) => {
                    self.view.cue_answered = Some(request.id);
                    scene.ctl.send(Command::ForceCue(request.start));
                }
                Some(false) => {
                    self.view.cue_answered = Some(request.id);
                    scene.ctl.send(Command::CancelCueOnAir(request.id));
                }
                None => {}
            }
        }
```

and add `use super::cue_on_air;` to the imports.

5. The CUE button's tooltip (§5 "CUE button"; the tests find the button by it): in `crates/fp-app/src/ui/player.rs` replace L289-295 with:

```rust
                // Without a Cue route the CUE is dimmed (spec §4.6, CA1);
                // on an on-air output its tooltip says where it goes (§5).
                let cue_tip = if fp_model::cue_on_air::cue_is_on_air(
                    scene.state,
                    fp_model::Holder::PlayerCue(id),
                ) {
                    t.tr("tip-cue-on-air")
                } else if pv.cueing || scene.state.config.outputs.player_has_cue(id) {
                    t.tr("tip-cue")
                } else {
                    t.tr("tip-cue-no-output")
                };
```

- [ ] **Step 6: Run the tests**

Run: `cargo test -p fp-app --test cue_on_air_ui`
Expected: PASS. If a label is not found because the status bar or a tooltip repeats it, use `query_all_by_label(…).count()` as in `the_cue_button_of_an_on_air_cue_says_where_it_goes`.

- [ ] **Step 7: Gate and commit**

Run the gate. Then:

```bash
git add crates/fp-app
git commit -m "feat(ui): ask before a pre-listen goes out on a sounding on-air output

On-air pre-listen spec §5: a modal question for the model's CA3 request,
from a player's CUE button, the row menu, the cue window or a cart
pre-listen. Pre-listen anyway sends the request forced; Cancel, the
backdrop, Esc and Enter cancel it. The CUE button's tooltip says when it
goes out on the on-air output.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 3: The notice and the Settings note

**Files:**
- Modify: `crates/fp-app/src/ui/app.rs` (after the rejections loop, L606-608)
- Modify: `crates/fp-app/src/ui/settings.rs` (L978-980, L1005-1007, L1025-1036)
- Modify: `crates/fp-model/src/config.rs` (remove `cue_equals_main`, `player_cue_equals_main`, `cartwall_cue_equals_main`), `crates/fp-model/tests/cue_output.rs`
- Test: `crates/fp-app/tests/cue_on_air_ui.rs`, `crates/fp-app/tests/settings.rs` (L946-959)

**Interfaces:**
- Consumes: `AppState::cue_on_air.notice: Option<CueStopped { id, cue, by }>`, `OutputsConfig::cue_shares_main(Holder)`, `cue_on_air::cue_is_on_air`, Task 2's `notice_text`.

- [ ] **Step 1: Tests**

Append to `crates/fp-app/tests/cue_on_air_ui.rs`:

```rust
#[test]
fn a_pre_listen_stopped_by_a_start_says_why() {
    let (mut h, fake) = harness(shared());
    fake.send(Command::ToggleCue(fake.player(0)));
    fake.send(Command::Play(fake.player(1)));
    h.run_steps(2);
    assert!(h.query_by_label(STOPPED_P2).is_some());
    assert!(!cueing(&fake, 0));
}
```

In `crates/fp-app/tests/settings.rs`, replace `CUE_EQUALS_MAIN` and `a_cue_output_equal_to_main_is_warned_about` (L946-959) with:

```rust
const CUE_ON_AIR: &str = "This Cue output shares an on-air output";

#[test]
fn a_cue_output_on_an_on_air_output_is_noted() {
    let count = |cue, cartwall| {
        let (h, _fake) = outputs_routed(Vec::new(), cue, cartwall);
        h.query_all_by_label_contains(CUE_ON_AIR).count()
    };
    assert_eq!(count("speakers", None), 0, "separate outputs");
    assert_eq!(count("dac", None), 1, "a player's Cue on its Main");
    assert_eq!(count("speakers", Some(("dac", "dac"))), 1, "the cartwall's");
    assert_eq!(count("dac", Some(("dac", "dac"))), 2, "both");
    assert_eq!(count("speakers", Some(("dac", "speakers"))), 0);
    assert_eq!(
        count("speakers", Some(("speakers", "dac"))),
        2,
        "each Cue on the other's Main"
    );
}
```

- [ ] **Step 2: Run them to see them fail**

Run: `cargo test -p fp-app --test cue_on_air_ui a_pre_listen_stopped && cargo test -p fp-app --test settings a_cue_output_on_an_on_air_output`
Expected: FAIL — no notice; the note follows `*_cue_equals_main` (the last case counts 0).

- [ ] **Step 3: The notice**

In `crates/fp-app/src/ui/app.rs`, after `while let Some(error) = self.ctl.take_rejection() { … }`:

```rust
        // On-air pre-listen spec §5: CA5's notice, once per notice.
        if let Some(stopped) = state.cue_on_air.notice
            && self.view.cue_notice_seen != Some(stopped.id)
        {
            self.view.cue_notice_seen = Some(stopped.id);
            let text = cue_on_air::notice_text(&self.i18n, &state, &stopped);
            self.view.notice = Some((text, time + NOTICE_SECS));
        }
```

- [ ] **Step 4: The Settings note**

In `crates/fp-app/src/ui/settings.rs`, replace the two conditions:

```rust
                let cue = fp_model::Holder::PlayerCue(player.id);
                if config.outputs.cue_shares_main(cue)
                    || fp_model::cue_on_air::cue_is_on_air(scene.state, cue)
                {
                    cue_on_air_note(ui, scene);
                }
```

```rust
            let cue = fp_model::Holder::CartwallCue;
            if config.outputs.cue_shares_main(cue)
                || fp_model::cue_on_air::cue_is_on_air(scene.state, cue)
            {
                cue_on_air_note(ui, scene);
            }
```

and the function:

```rust
/// On-air pre-listen spec §5: a Cue route that shares an on-air output is
/// usable; say that its pre-listen is heard on air and asks first while
/// something plays there.
fn cue_on_air_note(ui: &mut Ui, scene: &Scene<'_>) {
    ui.add(
        egui::Label::new(
            RichText::new(scene.i18n.tr("settings-cue-on-air"))
                .font(font(11.0))
                .color(theme::AMBER_TEXT),
        )
        .wrap(),
    );
}
```

- [ ] **Step 5: Remove what the note no longer uses**

In `crates/fp-model/src/config.rs`, delete `OutputsConfig::player_cue_equals_main`, `OutputsConfig::cartwall_cue_equals_main` and `pub fn cue_equals_main`. In `crates/fp-model/tests/cue_output.rs`, delete `equal_routes_are_still_recognised_until_settings_says_otherwise` and `cue_equals_main` from the import.

- [ ] **Step 6: Run the tests**

Run: `cargo test -p fp-app --test cue_on_air_ui --test settings && cargo test -p fp-model --test cue_output`
Expected: PASS.

- [ ] **Step 7: Gate and commit**

```bash
git add crates/fp-app crates/fp-model
git commit -m "feat(ui): say why a pre-listen stopped and note an on-air Cue output

On-air pre-listen spec §5: CA5's notice goes to the status bar once,
and Settings › Audio outputs notes a Cue route that shares an on-air
output (any player's or the cartwall's Main) instead of saying the CUE
is off.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 4: Remote control: `409 cue_on_air` and `"force": true` (CA6, CA4)

**Files:**
- Modify: `crates/fp-remote/src/api.rs` (`Operation` L15-38, `ApiError` L42-98, `commands_for` L122-252)
- Modify: `crates/fp-remote/src/http/handlers.rs` (bodies L158-162, `set_cue` L207-213, `cue_entry` L226-235, `cart_cue` L311-317), `crates/fp-remote/src/osc.rs` (L118)
- Modify: `crates/fp-remote/tests/support/mod.rs`, `tests/api.rs`, `tests/http.rs`, `tests/osc.rs`

**Interfaces:**
- Consumes: `fp_model::cue_on_air::{cue_start, needs_confirmation}`, `Command::ForceCue` (plan 1).
- Produces: `Operation::SetCue { player: PlayerId, on: bool, force: bool }`, `Operation::CueEntry { player: PlayerId, entry: EntryId, force: bool }`, `Operation::SetCartCue { cart: CartId, on: bool, force: bool }`; `ApiError::CueOnAir` (`409`, `cue_on_air`); test helper `support::on_air_cue_state() -> AppState`.

- [ ] **Step 1: Move the existing tests to the new shapes**

```bash
perl -pi -e 's/O::SetCue\((\w+), (\w+)\)/O::SetCue { player: $1, on: $2, force: false }/g; s/O::CueEntry\((\w+), (\w+)\)/O::CueEntry { player: $1, entry: $2, force: false }/g; s/O::SetCartCue\((\w+), (\w+)\)/O::SetCartCue { cart: $1, on: $2, force: false }/g' crates/fp-remote/tests/api.rs crates/fp-remote/tests/osc.rs
```

- [ ] **Step 2: The helper and the tests**

Append to `crates/fp-remote/tests/support/mod.rs`:

```rust
/// `demo_state`, with player 1's Cue, player 2's Main and the cartwall's
/// Cue placed on one `desk` output (on-air pre-listen spec §2) and player 2
/// playing: a pre-listen of player 1 or of a cart would be heard on air.
pub fn on_air_cue_state() -> AppState {
    let mut s = demo_state();
    let (p1, p2) = (s.players[0].id, s.players[1].id);
    let desk = fp_model::Route {
        backend: "null".into(),
        device: "desk".into(),
        first_channel: 0,
    };
    let device = fp_model::OutputDevice {
        backend: "null".into(),
        device: "desk".into(),
    };
    for r in &mut s.config.outputs.routes {
        if r.player == p1 {
            r.cue = Some(desk.clone());
        }
        if r.player == p2 {
            r.main = Some(desk.clone());
        }
    }
    s.config.outputs.cartwall.cue = Some(desk.clone());
    let running = fp_model::device_settings(&s.config.outputs, &device);
    for holder in [
        fp_model::Holder::PlayerCue(p1),
        fp_model::Holder::PlayerMain(p2),
        fp_model::Holder::CartwallCue,
    ] {
        fp_model::on_event(
            &mut s,
            fp_model::EngineEvent::Placed {
                holder,
                route: Some(desk.clone()),
                device: device.clone(),
                running,
            },
        );
    }
    fp_model::apply(&mut s, Command::Play(p2)).unwrap();
    s
}
```

Append to `crates/fp-remote/tests/api.rs` (add `CueStart` to its `use fp_model::{…}`):

```rust
#[test]
fn ca6_a_cue_that_would_ask_is_a_cue_on_air_conflict() {
    let s = support::on_air_cue_state();
    let p = s.players[0].id;
    let e = s.playlists.iter().next().unwrap().entries[2].id;
    let c = s.cartwall.pages[0].carts[0].id;
    for op in [
        O::SetCue { player: p, on: true, force: false },
        O::CueEntry { player: p, entry: e, force: false },
        O::SetCartCue { cart: c, on: true, force: false },
    ] {
        let err = plan(&s, op).unwrap_err();
        assert_eq!(err.status(), 409, "{op:?}");
        assert_eq!(err.code(), "cue_on_air", "{op:?}");
    }
}

#[test]
fn ca4_force_starts_it_anyway() {
    let s = support::on_air_cue_state();
    let p = s.players[0].id;
    let next = s.player(p).unwrap().next.unwrap();
    let e = s.playlists.iter().next().unwrap().entries[2].id;
    let c = s.cartwall.pages[0].carts[0].id;
    assert_eq!(
        plan(&s, O::SetCue { player: p, on: true, force: true }).unwrap(),
        vec![Command::ForceCue(CueStart::Entry { player: p, entry: next })]
    );
    assert_eq!(
        plan(&s, O::CueEntry { player: p, entry: e, force: true }).unwrap(),
        vec![Command::ForceCue(CueStart::Entry { player: p, entry: e })]
    );
    assert_eq!(
        plan(&s, O::SetCartCue { cart: c, on: true, force: true }).unwrap(),
        vec![Command::ForceCue(CueStart::Cart(c))]
    );
}

#[test]
fn ca6_a_quiet_on_air_output_needs_no_force_and_a_stop_never_does() {
    let mut s = support::on_air_cue_state();
    let (p1, p2) = (s.players[0].id, s.players[1].id);
    fp_model::apply(&mut s, Command::Stop(p2)).unwrap();
    assert_eq!(
        plan(&s, O::SetCue { player: p1, on: true, force: false }).unwrap(),
        vec![Command::SetCue(p1, true)]
    );
    fp_model::apply(&mut s, Command::SetCue(p1, true)).unwrap();
    fp_model::apply(&mut s, Command::Play(p2)).unwrap();
    // Play stopped the unconfirmed pre-listen (CA5): start it forced, then
    // stop it without force.
    let next = s.player(p1).unwrap().next.unwrap();
    fp_model::apply(
        &mut s,
        Command::ForceCue(CueStart::Entry { player: p1, entry: next }),
    )
    .unwrap();
    assert_eq!(
        plan(&s, O::SetCue { player: p1, on: false, force: false }).unwrap(),
        vec![Command::SetCue(p1, false)]
    );
}
```

Append to `crates/fp-remote/tests/http.rs` (add `on_air_cue_state` to the `use support::{…}`):

```rust
#[tokio::test]
async fn ca6_a_cue_on_a_sounding_on_air_output_is_409_and_force_starts_it() {
    let s = on_air_cue_state();
    let p = s.players[0].id.0;
    let fake = FakeControl::new(s);
    let uri = format!("/api/v1/players/{p}/cue");
    let (status, body, _) = call(ctx(&fake), "PUT", &uri, Some(json!({"on": true}))).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"], "cue_on_air");
    assert!(fake.take_sent().is_empty());
    let (status, _, _) = call(
        ctx(&fake),
        "PUT",
        &uri,
        Some(json!({"on": true, "force": true})),
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert!(matches!(fake.take_sent().as_slice(), [Command::ForceCue(_)]));
}

/// Review Focus 3.
#[tokio::test]
async fn force_must_be_a_boolean() {
    let s = on_air_cue_state();
    let p = s.players[0].id.0;
    let fake = FakeControl::new(s);
    for body in [
        json!({"on": true, "force": "yes"}),
        json!({"on": true, "forced": true}),
    ] {
        let (status, _, _) = call(
            ctx(&fake),
            "PUT",
            &format!("/api/v1/players/{p}/cue"),
            Some(body.clone()),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    }
    assert!(fake.take_sent().is_empty());
}
```

- [ ] **Step 3: Run them to see them fail**

Run: `cargo test -p fp-remote`
Expected: compile errors (`Operation::SetCue` is a tuple variant, no `ApiError::CueOnAir`).

- [ ] **Step 4: `api.rs`**

`Operation`:

```rust
    /// Pre-listen the next entry, or stop. `force` starts it even on a
    /// sounding on-air output (on-air pre-listen spec CA4).
    SetCue { player: PlayerId, on: bool, force: bool },
    SetNext(PlayerId, EntryId),
    /// Pre-listen an entry; `force` as for `SetCue`.
    CueEntry {
        player: PlayerId,
        entry: EntryId,
        force: bool,
    },
```

```rust
    /// Pre-listen a cart, or stop; `force` as for `SetCue`.
    SetCartCue { cart: CartId, on: bool, force: bool },
```

`ApiError`: add the variant `CueOnAir` after `Unavailable(String)`; `status`: `Self::Unavailable(_) | Self::CueOnAir => 409`; `code`: `Self::CueOnAir => "cue_on_air"`; `message`: `Self::CueOnAir => "the pre-listen would be heard on air, where something plays now; send \"force\": true to pre-listen anyway".to_owned()`.

`commands_for`, the three arms:

```rust
        O::SetCue { player: p, on, force } => {
            if player(p)?.cue.is_some() == on {
                return Ok(Vec::new());
            }
            available(state, Command::ToggleCue(p))?;
            cue_on(state, Command::SetCue(p, on), force)
        }
```

```rust
        O::CueEntry {
            player: p,
            entry: e,
            force,
        } => {
            player(p)?;
            entry(e)?;
            available(state, Command::CueEntry(p, e))?;
            cue_on(state, Command::CueEntry(p, e), force)
        }
```

```rust
        O::SetCartCue { cart: c, on, force } => {
            cart(c)?;
            if (state.cartwall.cue == Some(c)) == on {
                return Ok(Vec::new());
            }
            available(state, Command::CueCart(c))?;
            cue_on(state, Command::SetCartCue(c, on), force)
        }
```

and after `available`:

```rust
/// On-air pre-listen spec CA4, CA6: `command` as is, unless it starts a
/// pre-listen. With `force` it becomes `ForceCue` (confirmed); without, a
/// start the console would ask about (its on-air output sounds) is
/// refused, since a remote client cannot answer the question.
fn cue_on(state: &AppState, command: Command, force: bool) -> Result<Vec<Command>, ApiError> {
    match fp_model::cue_on_air::cue_start(state, &command) {
        Some(start) if force => Ok(vec![Command::ForceCue(start)]),
        _ if fp_model::cue_on_air::needs_confirmation(state, &command) => {
            Err(ApiError::CueOnAir)
        }
        _ => Ok(vec![command]),
    }
}
```

- [ ] **Step 5: The HTTP bodies and OSC**

In `crates/fp-remote/src/http/handlers.rs`, after `struct On`:

```rust
/// `{"on": bool}` with an optional `"force": bool` (on-air pre-listen spec
/// CA4).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CueOn {
    on: bool,
    #[serde(default)]
    force: bool,
}

/// `{"entry": id}` with an optional `"force": bool`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CueEntryBody {
    entry: u64,
    #[serde(default)]
    force: bool,
}
```

`set_cue` takes `JsonBody(b): JsonBody<CueOn>` and runs `Operation::SetCue { player: PlayerId(id(&raw)?), on: b.on, force: b.force }`; `cue_entry` takes `JsonBody<CueEntryBody>` and runs `Operation::CueEntry { player: PlayerId(id(&raw)?), entry: EntryId(b.entry), force: b.force }`; `cart_cue` takes `JsonBody<CueOn>` and runs `Operation::SetCartCue { cart: CartId(id(&raw)?), on: b.on, force: b.force }`.

`crates/fp-remote/src/osc.rs` L118:

```rust
                // OSC has no force (on-air pre-listen spec CA6): a CUE the
                // console would ask about is refused with `cue_on_air`.
                "cue" => {
                    return Ok(OscRequest::Op(Operation::SetCue {
                        player: p,
                        on: switch(args)?,
                        force: false,
                    }));
                }
```

- [ ] **Step 6: Run the tests**

Run: `cargo test -p fp-remote`
Expected: PASS.

- [ ] **Step 7: Gate and commit**

```bash
git add crates/fp-remote
git commit -m "feat(remote): refuse a pre-listen that would ask, accept force

On-air pre-listen spec CA6 and CA4: a CUE operation that would start a
pre-listen on a sounding on-air output is refused with 409 cue_on_air,
since a remote client cannot answer the console's question; with
\"force\": true it is sent as ForceCue and the pre-listen is confirmed.
OSC has no force.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 5: MIDI refuses a CUE that would ask (CA6)

**Files:**
- Modify: `crates/fp-control/src/router.rs` (L104-110)
- Test: `crates/fp-control/tests/router.rs`

**Interfaces:**
- Consumes: `fp_model::cue_on_air::needs_confirmation`.

- [ ] **Step 1: The test**

Append to `crates/fp-control/tests/router.rs`:

```rust
/// P1 (playing) and P2's Cue share `desk` (on-air pre-listen spec §2): a
/// pre-listen of P2 would be heard on air.
fn p2_cue_on_air(mut s: AppState) -> AppState {
    let desk = fp_model::Route {
        backend: "null".into(),
        device: "desk".into(),
        first_channel: 0,
    };
    let device = OutputDevice {
        backend: "null".into(),
        device: "desk".into(),
    };
    let (p1, p2) = (s.players[0].id, s.players[1].id);
    for r in &mut s.config.outputs.routes {
        if r.player == p1 {
            r.main = Some(desk.clone());
        }
        if r.player == p2 {
            r.cue = Some(desk.clone());
        }
    }
    let running = device_settings(&s.config.outputs, &device);
    for holder in [Holder::PlayerMain(p1), Holder::PlayerCue(p2)] {
        on_event(
            &mut s,
            EngineEvent::Placed {
                holder,
                route: Some(desk.clone()),
                device: device.clone(),
                running,
            },
        );
    }
    s
}

/// CA6, Review Focus 4.
#[test]
fn ca6_a_cue_button_that_would_ask_does_nothing() {
    let s = p2_cue_on_air(state());
    let (p1, p2) = (s.players[0].id, s.players[1].id);
    let mut r = router(&[(
        note(1),
        MidiAction::Button(ShortcutAction::CuePlayer(2)),
    )]);
    assert_eq!(r.on_message("APC", on(1), &s), None, "MIDI cannot confirm");
    let mut quiet = s.clone();
    apply(&mut quiet, Command::Stop(p1)).unwrap();
    assert_eq!(
        r.on_message("APC", on(1), &quiet),
        Some(Command::ToggleCue(p2))
    );
    // A running pre-listen on the sounding output can always be stopped.
    let mut running = quiet.clone();
    apply(&mut running, Command::ToggleCue(p2)).unwrap();
    let entry = running.player(p2).unwrap().cue.unwrap().entry;
    apply(
        &mut running,
        Command::ForceCue(fp_model::CueStart::Entry { player: p2, entry }),
    )
    .unwrap();
    apply(&mut running, Command::Play(p1)).unwrap();
    assert_eq!(
        r.on_message("APC", on(1), &running),
        Some(Command::ToggleCue(p2))
    );
}
```

- [ ] **Step 2: Run it to see it fail**

Run: `cargo test -p fp-control --test router ca6`
Expected: FAIL — the first assertion gets `Some(ToggleCue(p2))`.

- [ ] **Step 3: The router**

In `crates/fp-control/src/router.rs`, the `MidiAction::Button` arm:

```rust
            MidiAction::Button(action) => {
                if !pressed {
                    return None;
                }
                let command = player_command(state, action)?;
                if !command_available(state, &command) {
                    return None;
                }
                // On-air pre-listen spec CA6: MIDI cannot answer the
                // question a pre-listen on a sounding on-air output asks,
                // and has no force: such a CUE does nothing.
                if fp_model::cue_on_air::needs_confirmation(state, &command) {
                    tracing::info!("MIDI CUE refused: it would be heard on air (cue_on_air)");
                    return None;
                }
                Some(command)
            }
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p fp-control`
Expected: PASS.

- [ ] **Step 5: Gate and commit**

```bash
git add crates/fp-control
git commit -m "feat(control): a MIDI CUE that would be heard on air does nothing

On-air pre-listen spec CA6: MIDI cannot answer the question a pre-listen
on a sounding on-air output asks and has no force, so such a CUE button
is ignored with a log line. Stopping a running pre-listen still works.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

### Task 6: User guide, remote API reference, technical docs

**Files:**
- Modify: `docs/user/players.md` (L172-180), `docs/user/cartwall.md` (L26), `docs/user/settings.md` (L81-83), `docs/user/midi.md` (after L25), `docs/user/remote-control.md` (L74)
- Modify: `docs/technical/remote-api.md` (L18-25, L132, L134, L144, L233, L288), `docs/technical/ui.md` (L54-58)

- [ ] **Step 1: `docs/user/players.md`**

Replace "A player needs a Cue output that is not its Main output: without one, **CUE** and **Pre-listen on CUE** are dimmed, and hovering them says so." with:

```markdown
A player needs a Cue output: without one, **CUE** and **Pre-listen on CUE**
are dimmed, and hovering them says so.

With one sound card, the Cue output can be the on-air output itself (or
share it with another player or the cartwall). A pre-listen there is heard
on air, so:

- while nothing plays on that output, CUE starts at once;
- while something plays there (a player playing or paused, a cart, another
  pre-listen), CUE asks first: **Pre-listen anyway** or **Cancel** (Cancel
  is the default: Esc and Enter cancel). The same question comes from the
  track menu, the CUE window and the cartwall;
- when something starts on that output while a pre-listen you did not
  confirm plays there, the pre-listen stops and the status bar says why. A
  pre-listen you confirmed keeps playing.

The CUE button's tooltip says when its pre-listen goes out on the on-air
output.
```

- [ ] **Step 2: The other guide pages**

`docs/user/cartwall.md` L26 row: `| Pre-listen on CUE | Play it on the cartwall's CUE output (dimmed when the cartwall has no Cue output). On the on-air output it asks first while something plays there (see [CUE](players.md#cue-pre-listen)) |`

`docs/user/settings.md`: in the "Outputs per player" row append "A Cue output that shares an on-air output (any player's Main or the cartwall's) is allowed: a note under it says its pre-listen is heard on air and asks first while something plays there."; in the "Cartwall" row replace "Without a Cue there is no cart pre-listen." with "Without a Cue there is no cart pre-listen; a Cue on an on-air output works as for players."

`docs/user/midi.md`, after the list of actions (L25): "A **CUE** button does nothing when its pre-listen would be heard on air while something plays there: MIDI cannot answer the question the screen asks. Stopping a pre-listen always works."

`docs/user/remote-control.md` L74: after "pre-listen," insert "(a pre-listen that would be heard on air while something plays there is refused unless the request says `\"force\": true`; see the API reference),".

- [ ] **Step 3: `docs/technical/remote-api.md`**

- L20-21: "…and so is a player CUE, a `cue-entry` or a cart cue without a Cue route; one that would start a pre-listen on a sounding on-air output (the console would ask, on-air pre-listen spec CA6) is refused with `409 cue_on_air` unless the body says `\"force\": true`."
- Rows: `| PUT /players/{id}/cue | {"on": bool, "force": bool?} | Pre-listen the next entry, or stop pre-listening. `force` (default false) starts it even on a sounding on-air output; the pre-listen is then confirmed and is not stopped when something starts there |`; `| POST /players/{id}/cue-entry | {"entry": id, "force": bool?} | Pre-listen an entry; `force` as for `cue` |`; `| PUT /carts/{id}/cue | {"on": bool, "force": bool?} | Pre-listen a cart; `force` as for a player's `cue` |`.
- OSC `cue` row (L233): append "There is no force: a CUE that would ask is refused (logged as `cue_on_air`)."
- Error table, after the `409 unavailable` row: `| 409 | `cue_on_air` | A pre-listen would be heard on air while something plays there; send `"force": true` to start it anyway |`.

- [ ] **Step 4: `docs/technical/ui.md` L54-58**

Replace "A CUE dimmed for want of a Cue output apart from Main (`OutputsConfig::player_has_cue`) takes `tip-cue-no-output` as its tooltip, and the row and cart menus' *Pre-listen on CUE* items say the same when disabled;" with:

```markdown
   A CUE dimmed for want of a Cue route (`OutputsConfig::player_has_cue`)
   takes `tip-cue-no-output` as its tooltip, and the row and cart menus'
   *Pre-listen on CUE* items say the same when disabled; an on-air Cue
   (`fp_model::cue_on_air::cue_is_on_air`) takes `tip-cue-on-air`. The
   model's on-air pre-listen request (`AppState::cue_on_air.request`) opens
   `ui::cue_on_air::show`, a modal whose answer is `Command::ForceCue` or
   `Command::CancelCueOnAir`; `ViewState::cue_answered` hides it until the
   snapshot drops it, and `AppUi::keyboard` gives Esc and Enter to Cancel.
   CA5's notice (`AppState::cue_on_air.notice`) goes to the status bar once
   per id (`ViewState::cue_notice_seen`);
```

- [ ] **Step 5: Site checks and gate**

Run: `scripts/site/check-translations.sh` only if it runs without network (it compares `docs/i18n` with `docs/user`; the translations are regenerated before a release, so a "changed since" report is expected and is not a failure). Then the gate.

- [ ] **Step 6: Commit**

```bash
git add docs/user docs/technical
git commit -m "docs: pre-listen on an on-air output

The user guide (CUE, cartwall, settings, MIDI, remote control), the
remote API reference (force, 409 cue_on_air) and the interface notes
describe the on-air pre-listen of the 2026-10-15 spec.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"
```

---

## Self-review (against the spec)

- §5 confirmation: Task 2 (player button, cart pre-listen via the model; the row menu and the cue window send the same model commands, `CueEntry` and `SetCue`/`CueEntry`, so they reach the same request — no new code path), both buttons, Esc, Enter, the backdrop, the output going quiet.
- §5 notice: Task 3. §5 CUE button: Task 2 Step 5 (tooltip, not dimmed). §5 Settings: Task 3. §5 strings: Task 1.
- CA6: Task 4 (remote `409 cue_on_air`, `force`), Task 5 (MIDI, no force); OSC ruling.
- §8 plan 3 docs: Task 6 (user guide audio outputs, CUE, cartwall; remote API reference; technical docs).
