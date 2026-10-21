# itr06 plan E: analysis queue feedback

Goal: show what the analysis is doing, guard the re-analyse requests and make an explicit re-analysis read the file again (I8, I11). Spec: main design spec §6.1.

## Task 1. Pool: activity, fresh jobs, stale jobs (`fp-analysis`)
- `crates/fp-analysis/src/analyzer.rs`:
  - `Analyzer::activity() -> Activity { queued, running, loudness }` from the bookkeeping (a running job that was cancelled is not running).
  - `Job::fresh`; `submit_fresh` and `submit_urgent_fresh` skip the cache lookup in `run` and still store the result.
  - Stale jobs: `cancel` counts them; a submission drops them from the normal queue once they outnumber the live jobs.
- Tests: `crates/fp-analysis/tests/analyzer.rs` (activity while one job runs and others wait; a fresh job analyses a cached file and stores the new result); unit test in `analyzer.rs` (the normal queue stays bounded after repeated cancel and resubmit).

## Task 2. Services: pass, guards, fresh, published activity (`fp-app`)
- `crates/fp-app/src/activity.rs`: `AnalysisActivity` (queued, running, loudness, pass) and `ActivityCell` (`Arc<ArcSwap<…>>`).
- `crates/fp-app/src/services.rs`:
  - `library_pass` + `pass_total`; `ReanalyseAll` is ignored while it is non-empty; a settings restart still cancels and restarts the pass;
  - `reanalyse_track` ignores a track in flight (not peaks-only);
  - `fresh` set, filled by the two explicit requests, used up in `consider`;
  - the activity is rebuilt each step and stored when it changes; `Services::activity()`.
- `main.rs` hands the cell to the UI.
- Tests: `crates/fp-app/tests/services.rs` (a second ReanalyseAll cancels nothing; ReanalyseTrack of a queued track does nothing; ReanalyseTrack re-reads an edited file that kept size and mtime; the activity shows the pass and ends empty).

## Task 3. UI
- `AppUi::with_analysis_activity`, `Scene::analysis`.
- Settings → Analysis: the button disabled with "Analysing… N of M" while a pass runs.
- Row menu: Re-analyse disabled for a queued or running track, with the reason.
- Row flags: running → waveform icon, queued → hourglass (Ruling: the hourglass keeps meaning "waiting for analysis"; queued and never-analysed share it with different tooltips).
- Status bar: "Analysing N tracks" left of the legend; tooltip with running files, waiting count, pass progress, loudness-only jobs; legend items that do not fit are left out.
- Tests (kittest): `reanalyse_ui.rs` (button, menu item), `table_icons.rs` (icons), `analysis_status_ui.rs` (status item, tooltip, narrow width and de-DE).

## Strings (11 locales)
`status-analysing`, `status-measuring`, `tip-analysis-running`, `tip-analysis-waiting`, `tip-analysis-pass`, `settings-reanalyse-running`, `flag-analysis-queued`, `flag-analysis-running`, `menu-reanalyse-queued`, `menu-reanalyse-running`.

## Docs
- `docs/user/playlists.md` (icons, menu), `docs/user/settings.md` (button), `docs/user/getting-started.md` (status bar), `docs/user/loudness.md` (re-analyse re-reads the file).
- `docs/technical/analysis.md`, `docs/technical/ui.md`.

## Commits
1. `docs: spec and plan itr06-E` 2. `feat(analysis): …` 3. `feat(app): …` 4. `feat(ui): …` 5. docs folded into the commit they describe.
