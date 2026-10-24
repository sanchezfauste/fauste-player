# Multilingual Landing Page Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Publish the landing page and the privacy policy in every guide language, rendered from one template per page and one strings file per language, with the language choice shared with the guide and kept across pages.

**Architecture:** A new `scripts/site/landing.py` renders `site/index.html` and `site/privacy.html` (now templates) once per language from `site/locales/<lang>.toml` into `<out>/` and `<out>/<lang>/`. It also checks the translations, and `guide.py check` calls it. One shared browser script, `docs/book-assets/site-language.js`, holds the language decision (stored choice, `?lang=en`, browser languages). The guide's `language-menu.js` and a new `site/landing-language.js` both use it.

**Tech Stack:** Python 3.11+ stdlib (`tomllib`, `hashlib`, `json`, `re`, `unittest`), plain ES5 browser JavaScript, `node --test` for the JS unit tests, POSIX sh (`build.sh`).

**Spec:** `docs/superpowers/specs/2026-10-24-multilingual-landing-design.md`

## Global Constraints

- Languages: exactly the folders of `docs/i18n` (`es ca de eu fr gl it nl pl pt`) plus English; English is `/`, each translation `/<lang>/`.
- Storage key `fauste-player-guide-language`, shared by the guide and the landing.
- Published address: `https://sanchezfauste.com/fauste-player/`, overridable with `FAUSTE_SITE_URL`.
- Pages render and work without JavaScript. JavaScript only redirects and stores the choice.
- The download section (`data-asset` patterns, rows, table) exists only in `site/index.html`.
- Every translated page carries the AI notice in its HTML, linking to the English page with `?lang=en`. The privacy notice says the English version prevails.
- Update rule: a PR that changes `site/locales/en.toml` updates every translation and its `source-hash` in the same PR.
- All code, comments and docs in English. Never name other playout products.
- Commit only with `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` green (CLAUDE.md), plus this plan's site tests.

## Review Focus

1. **Download links on translated pages:** `/<lang>/index.html` gets its rows and version filled like `/index.html`, and `guide/index.html` and `guide/<lang>/index.html` are never touched by the fill. Pinned in Task 3, Step 7.
2. **Redirect loops and dead ends:** a stored translation on an English page whose translation lacks that page goes to the translation's start page. A translated page never redirects. `?lang=en` always stops the redirect. Pinned in Task 2's tests.
3. **A stale or invalid stored value** (`"xx"`, a removed language, storage that throws) behaves as if nothing were stored. Pinned in Task 2's tests.
4. **Text inside attributes** (`<title>`, meta description, `alt`, `aria-label`) containing `"` or `<` must not break the markup. Pinned in Task 1 (`attr:` placeholders escape).
5. **Relative paths one level down** (`../style.css`, `../images/…`, `../guide/<lang>/`, the menu's links): every link on every rendered page resolves. Pinned in Task 3, Step 7 (`check-links.sh` on a full build).

---

### Task 0: Branch

- [ ] **Step 1:** `git switch master && git pull && git switch -c feat/multilingual-landing`
- [ ] **Step 2:** Commit the spec and this plan: `git add docs/superpowers/specs/2026-10-24-multilingual-landing-design.md docs/superpowers/plans/2026-10-24-multilingual-landing.md && git commit -m "docs(site): design and plan for a multilingual landing page"`, with the co-author trailer. Run the gate command first. These are docs only, but CLAUDE.md requires a green gate before every commit.

### Task 1: Strings, hashing and rendering core

**Files:**
- Create: `scripts/site/landing.py`
- Create: `scripts/site/tests/test_landing.py`
- Modify: `.github/workflows/pages.yml` (a step before "Build the site": `python3 -m unittest discover -s scripts/site/tests -p 'test_*.py'` and `node --test scripts/site/tests/`)

**Interfaces:**
- Produces, in `landing.py`:
  - constants: `ROOT`, `LOCALES = ROOT/site/locales`, `I18N = ROOT/docs/i18n`, `TABLES = ("common", "index", "privacy")`, `META_KEYS = ("language", "name", "label", "notice", "privacy-notice", "english-link")`, `PAGES = {"index.html": "index", "privacy.html": "privacy"}`, `BASE_URL = os.environ.get("FAUSTE_SITE_URL", "https://sanchezfauste.com/fauste-player/")`
  - `class RenderError(Exception)`
  - `load(lang: str, locales: str = LOCALES) -> dict`: `tomllib` parse of `<locales>/<lang>.toml`
  - `texts(data: dict) -> dict[str, str]`: flattens `TABLES` to `"table.key" -> value`
  - `table_hash(table: dict) -> str`: `sha256(json.dumps(table, sort_keys=True, ensure_ascii=False, separators=(",", ":")).encode("utf-8")).hexdigest()`
  - `render(template: str, texts: dict[str, str], values: dict[str, str]) -> str`

Placeholder grammar: `{{ name }}` or `{{ attr:name }}`, regex `\{\{\s*(attr:)?([a-z0-9_.-]+)\s*\}\}`, one pass. Names with a dot are texts, names without are build values. `attr:` applies `html.escape(value, quote=True)`. An unknown name raises `RenderError("unknown placeholder <name>")`. `{{` left in the result raises `RenderError("placeholder left in output")`.

- [ ] **Step 1: Write the failing tests** in `test_landing.py` (`unittest`; `sys.path` insert of `scripts/site`):
  - `test_render_replaces_text_and_value`: `render("<p>{{ index.pitch }}</p><a href='{{ root }}x'>", {"index.pitch": "Hi <b>x</b>"}, {"root": "../"})` == `"<p>Hi <b>x</b></p><a href='../x'>"`
  - `test_attr_escapes`: `render('<img alt="{{ attr:index.alt }}">', {"index.alt": 'a "b" <c>'}, {})` == `'<img alt="a &quot;b&quot; &lt;c&gt;">'`
  - `test_unknown_placeholder_raises`: `{{ index.nope }}` with empty dicts raises `RenderError` whose message contains `index.nope`
  - `test_value_with_braces_raises`: text `"{{ x }}"` raises `RenderError`
  - `test_hash_ignores_order`: `table_hash({"a": "1", "b": "2"}) == table_hash({"b": "2", "a": "1"})` and `!= table_hash({"a": "1", "b": "3"})`
  - `test_texts_flattens_tables_only`: `texts({"language": "es", "common": {"x": "1"}, "index": {"y": "2"}, "privacy": {}, "source-hash": {"common": "h"}})` == `{"common.x": "1", "index.y": "2"}`
- [ ] **Step 2:** Run `python3 -m unittest discover -s scripts/site/tests -p 'test_*.py'`. Expected: FAIL (`No module named 'landing'`).
- [ ] **Step 3:** Implement the interfaces above in `scripts/site/landing.py`. Write a module docstring in `guide.py`'s style, listing the CLI subcommands added in Tasks 3 and 4.
- [ ] **Step 4:** Run the same command. Expected: 6 tests OK.
- [ ] **Step 5:** Add the CI step to `pages.yml`. The `node --test` half has no files until Task 2, so add it in Task 2 if `node --test` errors on an empty folder. Gate, then commit `build(site): add the landing page renderer core`.

### Task 2: Shared language decision; the guide keeps the choice

**Files:**
- Create: `docs/book-assets/site-language.js`
- Create: `scripts/site/tests/site-language.test.js`
- Modify: `docs/book-assets/language-menu.js` (`stored`, `remember`, `STORE` and `detect` now use the shared script)
- Modify: `docs/book.toml:15` (`additional-js = ["book-assets/languages.js", "book-assets/site-language.js", "book-assets/language-menu.js"]`)
- Modify: `docs/technical/release-process.md`, "The browser's language" bullet (the kept choice, per the spec's "Opening an English page")

**Interfaces:**
- Produces `site-language.js`, a UMD-style IIFE: `module.exports` under Node, `window.faustePlayerLanguage` in a browser. API:
  - `STORE = "fauste-player-guide-language"`
  - `stored() -> string|null` and `remember(code) -> void`, each in try/catch around `localStorage`
  - `decide(o) -> { go: string|null, remember: string|null }`, where `o = { current, codes, search, stored, wanted }`. `codes` is the translation codes without `"en"`. `wanted` is `navigator.languages`.

Rules, in order:
1. `current !== "en"` → `{go: null, remember: null}`.
2. `search` matches `/[?&]lang=en(&|$)/` → `{null, "en"}`.
3. `stored === "en"` → `{null, null}`.
4. `stored` is in `codes` → `{go: stored, remember: null}`.
5. Any other `stored` value is ignored.
6. The first entry of `wanted` whose primary subtag (lower-cased, split on `-`/`_`) is `"en"` → `{null, null}`; is in `codes` → `{go: it, remember: null}`.
7. Otherwise `{null, null}`.

- [ ] **Step 1: Write the failing tests** (`node:test`, `node:assert/strict`, `require("../../../docs/book-assets/site-language.js")`), with `codes = ["es", "pt"]`:
  - `translated page never redirects`: `{current: "es", stored: "pt", wanted: ["pt"]}` → `{go: null, remember: null}`
  - `?lang=en stores English`: `{current: "en", search: "?lang=en", stored: "es"}` → `{go: null, remember: "en"}`
  - `stored translation is kept`: `{current: "en", search: "", stored: "es", wanted: ["en-US"]}` → `go: "es"`
  - `stored English stays`: `{current: "en", stored: "en", wanted: ["es"]}` → `go: null`
  - `invalid stored value is ignored`: `{current: "en", stored: "xx", wanted: ["pt-BR"]}` → `go: "pt"`
  - `first visit, English first`: `{current: "en", stored: null, wanted: ["en-GB", "es"]}` → `go: null`
  - `first visit, no match`: `{current: "en", stored: null, wanted: ["ja"]}` → `go: null`
  - `missing wanted`: `{current: "en", stored: null, wanted: undefined}` → `go: null`
- [ ] **Step 2:** `node --test scripts/site/tests/`. Expected: FAIL (cannot find module).
- [ ] **Step 3:** Implement `site-language.js`. Then rewrite `detect()` in `language-menu.js`:
  - call `faustePlayerLanguage.decide` with `current: data.current`, `codes: data.languages.slice(1).map(code)`, `search: location.search`, `stored: stored()`, `wanted: navigator.languages || [navigator.language]`;
  - `remember` when the result says so;
  - when `go` is set, `location.replace(href(language with that code))`. `href` already falls back to the start page when the page is missing.

  Drop the local `STORE`, `stored` and `remember` in favour of the shared ones. Update the file's header comment.
- [ ] **Step 4:** `node --test scripts/site/tests/` → 8 pass. Then `scripts/site/build.sh target/site-t2` with `FAUSTE_RELEASE_JSON` unset, and `scripts/site/check-translations.sh target/site-t2` → passes (the guide pages still load the menu script).
- [ ] **Step 5:** Manual check: serve `target/site-t2` with `python3 -m http.server -d target/site-t2`. In a browser, set `localStorage["fauste-player-guide-language"]="es"`, open `/guide/players.html` → lands on `/guide/es/players.html`. Open `/guide/players.html?lang=en` → stays in English, and reloading `/guide/players.html` stays too.
- [ ] **Step 6:** Update release-process.md. Gate, then commit `feat(site): keep the chosen guide language on English pages`.

### Task 3: Templates, English strings, rendering per language, build integration

**Files:**
- Modify: `site/index.html`, `site/privacy.html` (become templates)
- Create: `site/locales/en.toml`
- Create: `site/landing-language.js`
- Modify: `site/download.js` (tab list label from `container.dataset.tablistLabel`, fallback `"Operating system"`)
- Modify: `site/style.css` (`.language-menu` and `.ai-notice` styles in the Nocturne tokens)
- Modify: `scripts/site/landing.py` (`page_values`, `build`, CLI `build`)
- Modify: `scripts/site/build.sh`
- Test: `scripts/site/tests/test_landing.py`

**Interfaces:**
- Consumes: Task 1's `load`, `texts`, `render`, `PAGES`, `BASE_URL`. Task 2's `site-language.js` API.
- Produces:
  - `codes(locales: str = LOCALES) -> list[str]`: sorted stems of `<locales>/*.toml` except `en`
  - `page_values(lang: str, page: str, data: dict, langs: list[dict], out: str) -> dict[str, str]`. `langs` holds `{"code", "name"}`, English first; `data` is the loaded file of `lang`. Values:
    - `root`: `""` for English, `"../"` otherwise
    - `guide`: `root + "guide/" + ("" if en else code + "/")`
    - `lang`
    - `shot`: `root + "images/" + (code + "/" if os.path.isfile(out/images/code/main-screen.png) else "") + "main-screen.png"`
    - `menu`: `<details class="language-menu"><summary title=… aria-label=…>` (label, globe SVG from `language-menu.js`), then `<ul>` with one `<a href lang hreflang>` per language. The href is `root + ("" if en else code + "/") + ("" if page == "index.html" else page)`, plus `"?lang=en"` for English. The current language gets `aria-current="true"`.
    - `alternates`: `<link rel="alternate" hreflang="<code>" href="BASE_URL + dir + page">` per language, plus `hreflang="x-default"` for English (absolute; `index.html` is `""`)
    - `notice`: `""` for English. Otherwise `<div class="ai-notice" role="note">{notice or privacy-notice} <a href="{root}{page-or-empty}?lang=en" hreflang="en">{english-link}</a></div>`
  - `build(out: str, locales: str = LOCALES, site: str = ROOT/site) -> None`: renders both pages for English (into `out/`) and every code (into `out/<code>/`). Templates are read from `site`.
  - CLI: `python3 landing.py build <out>`

Template conventions: every visible text, `<title>`, meta description, `alt`, `aria-label`, `title=` is a `{{ table.key }}`, with `attr:` inside attributes. Assets use `{{ root }}`. The brand and page links stay within the language folder (`./`, `privacy.html`, `./#download`). `<html lang="{{ lang }}">`. `{{ alternates }}` goes in `<head>`, `{{ menu }}` at the end of `.top nav`, `{{ notice }}` first in `<main>`. Before `download.js` come `<script src="{{ root }}site-language.js"></script><script src="{{ root }}landing-language.js"></script>`. `<div id="tabs" data-tablist-label="{{ attr:index.os-tabs }}">`. Key names follow the content (`index.pitch`, `index.feature-cartwall-title`, …). The download section's `data-asset` attributes, `data-row`, `<span data-version>` and hrefs stay byte-for-byte.

`landing-language.js`:
- reads `document.documentElement.lang`;
- the menu links `.language-menu a[hreflang]` give the codes (all but `en`) and the targets;
- calls `decide` (search, `stored()`, `navigator.languages`) and `remember`s when told;
- on `go`, `location.replace` to that code's menu link;
- each menu link and the notice link `remember` their `hreflang` on click.

`build.sh` changes:
- replace `cp "$root"/site/* "$out/"` with copies of `site/*.css` and `site/*.js` plus `docs/book-assets/site-language.js`;
- run `python3 "$root/scripts/site/landing.py" build "$out"` after the screenshots step;
- the download-fill heredoc takes `"$out"` and fills `$out/index.html` plus `$out/<code>/index.html` for each `site/locales/<code>.toml` other than `en`, with the per-file logic unchanged.

- [ ] **Step 1: Write the failing tests:**
  - `test_english_index_renders`: `render(read(site/index.html), texts(load("en")), page_values("en", "index.html", load("en"), [{"code": "en", "name": "English"}], tmp))` succeeds. It contains `<html lang="en">`, every `data-asset="` of the current master page (compare the sets parsed with `re.findall(r'data-asset="([^"]*)"', …)` against the template), and no `ai-notice`.
  - `test_english_privacy_renders`: same for `privacy.html`.
  - `test_translated_values`: with a temp `out`, `page_values("es", "privacy.html", {"notice": "N", "privacy-notice": "P", "english-link": "E"}, [en, {"code": "es", "name": "Español"}], out)` gives `root == "../"`, `guide == "../guide/es/"`, `shot == "../images/main-screen.png"`. After creating `out/images/es/main-screen.png`, `shot == "../images/es/main-screen.png"`. `notice` contains `P`, `href="../privacy.html?lang=en"` and not `N`. `menu` contains `href="../privacy.html?lang=en"` and `href="../es/privacy.html"` with `aria-current="true"`. `alternates` contains `hreflang="x-default" href="https://sanchezfauste.com/fauste-player/privacy.html"`.
  - `test_build_writes_each_language`: a temp locales dir holding a copy of `en.toml` and an `xx.toml`. The `xx.toml` copies `en.toml`'s tables and adds `language = "xx"`, `name`, `label`, `notice`, `privacy-notice` and `english-link`. `build(out, locales=tmp_locales)` writes `out/index.html`, `out/privacy.html`, `out/xx/index.html` and `out/xx/privacy.html`; the last has `<html lang="xx">` and `class="ai-notice"`.
- [ ] **Step 2:** Run the unittest command. Expected: the 4 new tests FAIL.
- [ ] **Step 3:** Write `en.toml` with the current English texts verbatim (top level `language = "en"`, `name = "English"`, `label = "Language"`). Convert both templates, then implement `codes`, `page_values` and `build`.
- [ ] **Step 4:** Unittest command → all pass.
- [ ] **Step 5:** Write `landing-language.js` and update `download.js`, `style.css` and `build.sh`. Menu styling:
  - `summary` is a globe icon button in `--muted` that turns `--text` on hover;
  - the list is an absolutely positioned panel with `--surface` background, `--line` border and 8px radius;
  - `.ai-notice` is a `--accent-900` box with an `--accent` left border;
  - the header still wraps at 480px with no horizontal scroll.
- [ ] **Step 6:** Build: `FAUSTE_RELEASE_JSON=<tmp json> scripts/site/build.sh target/site-t3`. Write the JSON to a scratch file as `{"tagName": "v9.9.9", "url": "…", "assets": [{"name": "fauste-player-9.9.9-x86_64-pc-windows-msvc.msi", "url": "https://example.invalid/a.msi"}, {"name": "fauste-player_9.9.9_amd64.deb", "url": "https://example.invalid/b.deb"}]}`. Expected: `Site built`.
- [ ] **Step 7: Verify the Review Focus items 1 and 5:**
  - `scripts/site/check-links.sh target/site-t3` exits 0;
  - `grep -c 'v9.9.9' target/site-t3/index.html` ≥ 1;
  - `git diff --no-index` of the visible text of English `index.html` before/after (build master into `target/site-before` first) shows only the added menu, alternates and scripts;
  - `grep -L 'data-asset' target/site-t3/guide/index.html` lists the file (untouched).
- [ ] **Step 8:** Manual check over `python3 -m http.server`:
  - the English page looks as before plus the globe;
  - the menu opens without JavaScript (disable it in devtools);
  - a phone width (375px) has no horizontal scroll;
  - the OS tabs still work.
- [ ] **Step 9:** Gate, then commit `feat(site): render the landing page and privacy policy from templates`.

### Task 4: The check

**Files:**
- Modify: `scripts/site/landing.py` (`check`, CLI `hash`)
- Modify: `scripts/site/guide.py` (`check` appends `landing.check(site)`; `import landing` inside `check` to avoid an import cycle; update the docstrings of `guide.py` and `check-translations.sh`)
- Test: `scripts/site/tests/test_landing.py`

**Interfaces:**
- Produces:
  - `check(site: str|None, locales: str = LOCALES, i18n: str = I18N) -> list[str]`: problem lines, empty when fine.
  - CLI `python3 landing.py hash`: prints `[source-hash]` then `common = "<hash>"`, `index = …` and `privacy = …` for the current `en.toml`.

Problems, with exact message stems (tests match on them):
- `site/locales: languages <sorted codes> differ from docs/i18n <sorted folders>`
- `site/locales/<code>.toml: missing <keys>` (META_KEYS and `source-hash.<table>`)
- `site/locales/<code>.toml: language is '<x>', not '<code>'`
- `site/locales/<code>.toml: <name|label> differs from docs/i18n/<code>/translation.toml`
- `site/locales/<code>.toml: [<table>] missing <keys>` / `[<table>] has keys English lacks: <keys>`
- `site/locales/<code>.toml: [<table>] was translated from older English text (source-hash should be <hash>)`
- `site/locales/<code>.toml: <table.key> contains a placeholder`
- Site given: `<code>/<page>: not built`, `<code>/<page>: not marked lang=<code>`, `<code>/<page>: no AI-translation notice`, `<code>/<page>: no language menu`

- [ ] **Step 1: Write the failing tests.** Use temp `locales` and `i18n` dirs. Fixture: `i18n/xx/translation.toml` with `name = "Xx"`, `label = "L"`, and a valid `xx.toml` whose hashes come from `table_hash`.
  - `test_valid_translation_passes`: `check(None, locales, i18n) == []`
  - `test_language_sets_differ`: an extra `i18n/yy/` folder → one problem starting `site/locales: languages`
  - `test_stale_hash`: change one value in `en.toml`'s `[privacy]` → exactly one problem containing `[privacy] was translated from older English text` and the new hash. No `[index]` problem.
  - `test_key_sets`: drop `index.pitch` from `xx` and add `index.extra` → both messages
  - `test_name_mismatch`: `name = "Other"` → `name differs from docs/i18n/xx/translation.toml`
  - `test_placeholder_in_value`: a value containing `{{ root }}` → `contains a placeholder`
  - `test_built_site`: `build(out, locales)` then `check(out, locales, i18n) == []`. Delete the notice div from `out/xx/privacy.html` → `xx/privacy.html: no AI-translation notice`
- [ ] **Step 2:** Run the unittest command. Expected: the 7 new tests FAIL.
- [ ] **Step 3:** Implement `check` and `hash`. Wire `guide.py check` to print the landing problems with its own and count them in its exit status.
- [ ] **Step 4:** Unittest command → all pass. `scripts/site/check-translations.sh` on the repo fails now with `site/locales: languages … differ`. That is expected until Task 5; record it in the ledger and do not commit a red CI state alone. Commit Tasks 4 and 5 together, or commit Task 4 with a `Ruling:` in the ledger and push only after Task 5.
- [ ] **Step 5:** Gate (the cargo half), then commit `feat(site): check the landing page translations`.

### Task 5: The ten translations and the documentation

**Files:**
- Create: `site/locales/{es,ca,de,eu,fr,gl,it,nl,pl,pt}.toml`
- Modify: `docs/technical/release-process.md` ("The website": `/` and `/<lang>/`, files, the update rule, the check, `FAUSTE_SITE_URL`, the unit tests)
- Modify: `CLAUDE.md`:
  - "Documentation is part of every change": a bullet saying the landing and privacy texts live in `site/locales`, `en.toml` is the source, and every translation plus its `source-hash` (`python3 scripts/site/landing.py hash`) is updated in the same PR, with AI;
  - "Where things are": the `site/` row mentions `site/locales`;
  - "Commands": `python3 -m unittest discover -s scripts/site/tests -p 'test_*.py'` and `node --test scripts/site/tests/`.
- Modify: `.github/pull_request_template.md` (checklist line: `Landing/privacy texts in every site/locales file (en.toml is the source)`)
- Modify: `README.md` only if it describes the website's languages (grep for `guide` and `languages`)

- [ ] **Step 1:** For each language, write the file:
  - copy `language`, `name` and `label` from `docs/i18n/<code>/translation.toml`;
  - write `notice` ("This page was translated with AI from the English page and may contain errors." in that language);
  - write `privacy-notice` (the same, plus "the English version prevails");
  - write `english-link` (as in `translation.toml`);
  - translate every `[common]`, `[index]` and `[privacy]` value from `en.toml`, keeping inline HTML, URLs and product names;
  - paste `[source-hash]` from `landing.py hash`.

  Use the guide's terminology for each language (read its `README.md` and `getting-started.md`).
- [ ] **Step 2:** `scripts/site/check-translations.sh` → passes with no landing problems.
- [ ] **Step 3:** Full build with the fake release JSON, then:
  - `check-links.sh` and `check-translations.sh target/site-t5` both pass;
  - over `http.server`, with storage cleared and the browser language set to Spanish (devtools sensors or `--lang=es`), `/` goes to `/es/`;
  - choosing English in the menu returns to `/?lang=en` and stays there after a reload;
  - choosing Deutsch on the landing page, then opening `/guide/players.html`, shows the German guide page.
- [ ] **Step 4:** Write the documentation changes above.
- [ ] **Step 5:** Gate plus the two site test commands, then commit `feat(site): translate the landing page and privacy policy`.

### Task 6: Review and PR

- [ ] **Step 1:** `superpowers:requesting-code-review` with a fresh reviewer on the most capable model, over the whole branch against this plan and the spec. Fix Critical and Important findings test-first; defer Minor ones in `.superpowers/sdd/multilingual-landing/progress.md`.
- [ ] **Step 2:** Push and open the PR (`gh pr create`, template filled; title `feat(site): publish the landing page and privacy policy in every guide language`). Wait for `gh pr checks --watch`; the `pages.yml` PR job builds the site with Spanish screenshots and runs both checks.
- [ ] **Step 3:** Rebase on master if needed, then `gh pr merge --merge --delete-branch` and update local master.
