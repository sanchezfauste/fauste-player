# Multilingual landing page and privacy policy — design

Status: draft, 2026-10-24.

## Intent

The website has two parts: the landing page (`site/index.html`, with
`site/privacy.html`) and the user guide (`docs/user`, an mdBook). The guide
is published in English and in every translation in `docs/i18n`. The landing
page and the privacy policy are English only. A visitor who reads the guide
in their own language therefore lands on an English front page, and the
privacy policy that the stores link to is English only too.

This change publishes the landing page and the privacy policy in the same
languages as the guide. Both parts of the site share one language list, one
language menu behaviour, one stored language choice and one check command.
The difference is the update rule. The landing page and the privacy policy
are short and are what a newcomer reads first, so they must **never lag
behind English**: a pull request that changes their English text updates
every language in the same PR, the same rule as the interface strings.
CI fails when a translation is stale.

## Goals

- `/` (English) and `/<lang>/` for every language in `docs/i18n` (`es`,
  `ca`, `de`, `eu`, `fr`, `gl`, `it`, `nl`, `pl`, `pt`); `privacy.html`
  likewise (`/privacy.html`, `/<lang>/privacy.html`).
- One HTML template per page and the texts apart, one file per language,
  so the page structure (the download section and its asset patterns
  above all) exists once.
- Translated pages link to the guide in their language (`guide/<lang>/…`)
  and show the localized main-screen screenshot when the build has it.
- A language menu on every landing and privacy page, a first-visit jump to
  the browser's language, and an explicit choice that is remembered and
  **kept**: one storage key for the whole site, so a language chosen on the
  landing page also applies in the guide, and the other way round, and an
  English page opened later takes the visitor back to their language.
- Every translated page says it was translated with AI and links to the
  English page.
- CI fails when a translation is missing a key, has an extra key, or was
  made from older English text.

## Non-goals

- The guide's translation rules do not change. Its translations still
  follow English `docs/user` loosely and are regenerated before each
  release. (Its language *behaviour* does change; see "Language menu and
  first visit".)
- No new languages. The language list is the folders of `docs/i18n`.
- No server-side language negotiation (GitHub Pages serves static files).
- `README.md` and the release notes are not translated.

## Anti-goals (a failure even if it works)

- The download section (the `data-asset` patterns, rows and table) copied
  into each language. It must exist once, in the template.
- The landing languages differing from the guide languages.
- A page that cannot be read, or a language that cannot be chosen, without
  JavaScript. Pages are rendered statically and the menu is plain links.
  JavaScript is only used for the first-visit jump.
- The automatic jump overriding a language the visitor chose.
- A translated landing page that silently shows older text than English.

## Design

### Files

| Path | Content |
|---|---|
| `site/index.html`, `site/privacy.html` | Templates: the current pages with every visible text replaced by a placeholder |
| `site/locales/en.toml` | The English texts: the only source |
| `site/locales/<lang>.toml` | One translation per language in `docs/i18n` |
| `site/download.js`, `site/style.css` | Unchanged in role; shared by every language |

The translations live next to the landing page, not in `docs/i18n/<lang>/`,
because they follow a different rule than the guide (always current versus
regenerated per release). Keeping them apart also keeps them out of the
guide's mirror check. The language *set* is still the guide's: the check
fails when `site/locales` and `docs/i18n` disagree.

### Strings file

TOML, read with `tomllib` like `translation.toml`. Flat keys, or one table
per page (`[index]`, `[privacy]`, `[common]`). The exact key names are the
builder's call. Values may contain inline HTML (`<code>`, `<a href>`), so
the privacy policy's links and the landing's `<code>.sha256</code>` stay in
the text where a translation can move them. A translation file also
carries:

- `language`, `name` (for the menu) and `label` (the menu's label, e.g.
  "Idioma"), which match `translation.toml` of the same language. The check
  compares them.
- `notice` and `english-link`: the AI notice for these pages. It has no
  version or commit, since the text is always current. The privacy notice
  must say that the English version prevails.
- `source-hash`, one per page table (`[index]`, `[privacy]`, `[common]`):
  the SHA-256 of the English texts of that table this translation was made
  from (designer's call: per page, so a fix to the privacy policy does not
  mark the landing texts stale). It is computed over the parsed values
  (canonical JSON of the key→value map, sorted keys), so comments and
  formatting in `en.toml` do not count. Both `scripts/site/guide.py` and the
  check print the current hashes, so whoever regenerates a translation can
  write them.
- The metadata keys (`language`, `name`, `label`, `notice`,
  `english-link`, `source-hash`) are not texts of `en.toml` and are exempt
  from the key-set comparison.

`<title>`, the meta description, image `alt` texts, `aria-label`s and the
"Last updated" date of the privacy policy are texts like any other
(designer's call for the date: each language writes it in its own words,
and a date change marks the privacy table stale like any edit). Texts that
`download.js` puts on the page (the tab list's `aria-label`) reach it from
the template through a `data-` attribute.

### Template and rendering

Placeholders are `{{ key }}`. They also cover a few build-time values that
are not texts: `{{ root }}` (`""` for English, `"../"` for a translation,
used for `style.css`, `favicon.svg`, `download.js`, `images/`, the other
page), `{{ guide }}` (`guide/` or `guide/<lang>/`), `{{ lang }}` (the
`<html lang>`), the main-screen image path, the language menu and the
alternate links. Rendering fails, so the build fails, on an unknown
placeholder, a placeholder left in the output, or a missing key.

`scripts/site/build.sh` renders each template once per language. It writes
English to the site root and each translation to `<lang>/`, then runs the
existing download-link filling on every rendered `index.html`, unchanged in
logic. `build.sh` copies `site/*` without `-R` today; it must copy the
static files only, not `site/locales` or the raw templates. The rendering lives in Python next to the guide's (a new
subcommand of `scripts/site/guide.py`, or a sibling module it imports; the
builder's call). The output folder rules of `build.sh` stay.

Screenshot: a translated landing page uses `images/<lang>/main-screen.png`
when `FAUSTE_SHOTS_DIR` provided it, else `images/main-screen.png`. This is
the same fallback the guide uses.

Each page gets `<link rel="alternate" hreflang="…">` for every language plus
`x-default` (English), with absolute URLs under the published address
<https://sanchezfauste.com/fauste-player/> (see release-process.md). The
address is one constant in the renderer, overridable by an environment
variable for other deployments (designer's call). `check-links.sh` already
ignores absolute URLs. Each translated page also gets the AI notice in its
HTML, visible without JavaScript, linking to the English page with
`?lang=en`.

### Language menu and first visit

- **Menu:** a language button in the landing header, rendered statically as
  a list of links (to the same page in each language; English links carry
  `?lang=en`). It looks like the guide's globe menu, in the site's own
  styles. Opening and closing it works without JavaScript (for example
  `<details>`).
- **Choice:** clicking a language stores it under the guide's key
  `fauste-player-guide-language`. Opening a page with `?lang=en` stores
  English.
- **Opening an English page** (landing, privacy or guide), in this order:
  1. with `?lang=en`: store English and stay;
  2. with a stored language: stay when it is English; when it is a
     translation, go to the same page in it (its start page when it lacks
     the page, as the guide's menu does). A stored code that is no longer a
     language is ignored as if nothing were stored;
  3. with nothing stored, the first visit: the first entry of
     `navigator.languages` that is English (stay) or matches a language code
     (`pt-BR` → `pt`) decides, and the visitor goes to the same page in that
     language. The jump does not store anything; only a choice does.
- **This changes the guide.** Today the guide stops jumping once anything
  is stored, so a visitor who chose Spanish and later opens an English
  guide page stays in English. With this design, they go to the Spanish
  page (decided by the maintainer: the chosen language is kept wherever
  possible). Every link to English that means "show me English" (menu
  entries, the AI notice's link) already carries `?lang=en`, which stores
  English and so ends the redirect. Plain links between English pages do
  not carry it, so they never fight the choice.
- **Translated pages never redirect.** A visitor with English stored who
  opens `/es/` stays there; following a link is respected. Only English
  pages, the default a search engine or a bare URL leads to, redirect.
- The detection and storage logic should not be written twice. Sharing it
  between `docs/book-assets/language-menu.js` and the landing (a small
  common script, copied into both places by the build) is preferred. The
  builder may keep two copies only if sharing fights mdBook's asset
  handling, and must then add a note in each copy pointing at the other.

### The check

`scripts/site/check-translations.sh [<site>]` stays the one command. It
gains these failures:

- `site/locales/*.toml` languages ≠ `docs/i18n` folders;
- a translation missing a key of `en.toml`, or having a key `en.toml` lacks;
- `source-hash` ≠ the hash of the current `en.toml` (the message names the
  file and prints the expected hash);
- `language`, `name` or `label` differing from that language's
  `translation.toml`;
- a placeholder in a value that the template does not know;
- given a built site: a landing or privacy page in `<lang>/` without
  `lang="<lang>"`, without the notice, or without the menu.

`check-links.sh` already walks every HTML page, so the new pages and their
`../` links are covered. `pages.yml` already runs both checks on pull
requests touching `site/**`.

### Update rule (documented in CLAUDE.md, the PR template and release-process.md)

A pull request that changes `site/locales/en.toml` updates every
`site/locales/<lang>.toml` in the same PR (generated with AI, like the
interface strings) and their `source-hash`. A change to the template alone
(structure, styles) needs no translation. The guide keeps its own rule.

## Documentation to update

- `docs/technical/release-process.md`, "The website": languages, layout,
  the rule, the check; and "The browser's language" in the guide section,
  for the kept choice.
- `CLAUDE.md`, "Documentation is part of every change" and the paths table
  (`site/locales`), with `AGENTS.md` unchanged (it is a pointer).
- `.github/pull_request_template.md`: a checklist line for the landing
  strings.
- `README.md` only if it describes the website's languages.

## Left to the builder

- Key names and the table layout of the strings file.
- Where the renderer lives (in `guide.py` or a module beside it).
- The menu's exact markup and styles, within the Nocturne tokens of
  `site/style.css`.
- How the shared language-detection script is wired into mdBook and the
  landing.
- The initial translations: generated with AI from `en.toml`, all ten in
  the first PR.
