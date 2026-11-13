# Automation rework, plan 2: one name per concept — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Every UI string and user document uses the terms of the rework spec §3 (Log / Parrilla, Music fill / Musicalizador, Category track / Pista de categoría, Hard start / Hora fija), so no concept has two names.

**Architecture:** Strings only: Fluent messages in `crates/fp-app/locales/*/main.ftl` (message ids unchanged, so no code changes) and the docs that name them. Code identifiers (`schedule`, `filler`, `time_critical`) keep their names (spec §3, my call there).

**Tech Stack:** Fluent, Markdown.

**Spec:** `docs/superpowers/specs/2026-11-13-automation-rework-design.md` §3, AR1, AR2.

## Global Constraints

- en-US is the source; es-ES written with the change; the other nine locales (ca-ES, de-DE, eu-ES, fr-FR, gl-ES, it-IT, nl-NL, pl-PL, pt-PT) translated with AI in the same task.
- Never mention other playout products.
- `docs/i18n/` is not touched (regenerated before a release).
- Strings that plan 3 removes (`schedule-start`, `schedule-stop-after-current`, `schedule-running`, `schedule-stopped` and their tips) are left as they are.

## Review Focus

- A string still saying "Music" for the filler, or "Schedule" for the log, anywhere in the UI (grep the locales after the change).
- The UI tests that look widgets up by their English label (`crates/fp-app/tests/it/*schedule*`, `*filler*`, `*automation*`) break on renamed labels: update them to the new labels.
- Fluent placeables (`{ $name }`) kept intact in every translation.

---

### Task 1: Rename the strings

**Files:**
- Modify: `crates/fp-app/locales/*/main.ftl` (all eleven)
- Modify: UI tests that query a renamed label

The new values (en-US / es-ES):

| id | en-US | es-ES |
|---|---|---|
| `music-column` | Music fill | Musicalizador |
| `schedule-column` | Log | Parrilla |
| `schedule-item-category` | Category track: { $name } | Pista de categoría: { $name } |
| `schedule-item-rotation` | Music fill · { $name } | Musicalizador · { $name } |
| `schedule-time-critical` | Hard start: it plays only at its time | Hora fija: solo suena a su hora |
| `schedule-item-time-critical-check` | Hard start | Hora fija |
| `schedule-edit` | Edit log | Editar parrilla |
| `schedule-editor-title` | Log editor | Editor de parrilla |
| `schedule-add-category` | Add category track | Añadir pista de categoría |
| `schedule-music-after` | Then music fill | Después, musicalizador |
| `schedule-block-add` | Add block | Añadir bloque horario |
| `filler-playlist-name` | Music fill | Musicalizador |
| `schedule-playlist-name` | Log | Parrilla |
| `error-filler-playlist` | That list belongs to the music fill. | Esa lista pertenece al musicalizador. |
| `error-schedule-rotation-last` | A music fill must be the last item of a block. | El musicalizador debe ser el último elemento del bloque. |
| `settings-automation-rules` | Music fill | Musicalizador |
| `settings-hint-filler-lookahead` | How much music the music fill keeps queued | (unchanged) |
| `settings-hint-filler-kept` | Older played tracks leave the music fill’s list, so it never grows | (unchanged) |
| `settings-rotations-hint` | "music filler" → "music fill" | (unchanged) |
| `holder-filler-main` / `-cue` | Music fill Main / Music fill CUE | Musicalizador Main / Musicalizador CUE |
| `holder-schedule-main` / `-cue` | Log Main / Log CUE | (unchanged) |

- [ ] **Step 1:** Change en-US and es-ES; translate the same ids in the nine other locales with the same terms (a radio-sector word for "music fill", "log" and "hard start" in each language).
- [ ] **Step 2:** Run `cargo test -p fp-app --test it` — expected: failures only in tests that look up a renamed label; update those labels, run again — PASS. Also the i18n completeness test (every id in every locale) passes.
- [ ] **Step 3:** `grep -n "Music\b\|Schedule\b\|filler\|Time-critical" crates/fp-app/locales/en-US/main.ftl` shows no automation string with an old term (except the removed-in-plan-3 strings and `kind-music`, `rotation-default-name`).
- [ ] **Step 4: Commit** `feat(ui): one name per automation concept`

### Task 2: Docs follow the terms

**Files:**
- Modify: `docs/user/automation.md`, `docs/user/layout.md`, `docs/user/settings.md`, `docs/user/README.md`, `docs/technical/automation.md`, `docs/technical/ui.md`, `README.md` where they name the columns or items.

- [ ] **Step 1:** Replace the old UI terms with the new ones (Music column → Music fill; Schedule column → Log; time-critical → hard start; "one of a category" → category track). Technical docs may keep code names in backticks.
- [ ] **Step 2:** `scripts/site/build.sh /tmp/site-check && scripts/site/check-links.sh /tmp/site-check` — no dead links (skip if `mdbook` is unavailable and say so in the ledger).
- [ ] **Step 3: Commit** `docs: automation terms follow the rework`
