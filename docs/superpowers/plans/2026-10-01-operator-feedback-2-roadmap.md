# Operator Feedback 2 — Roadmap of Plans

**Spec:** [`docs/superpowers/specs/2026-10-01-operator-feedback-2-design.md`](../specs/2026-10-01-operator-feedback-2-design.md)

The spec groups items O1–O24 into ten plans. Plan 1 is written in full in
[`2026-10-01-feedback2-plan1-window-lifecycle.md`](2026-10-01-feedback2-plan1-window-lifecycle.md).
Each later plan is written with `superpowers:writing-plans` just before it
runs, against the code that the earlier plans left. Each plan reaches
`master` through its own pull request (see `CLAUDE.md` → Pull requests).

## Running a plan

1. Start from an up-to-date `master` that includes the previous plan.
2. Write the plan, if it is not written yet. Name it
   `docs/superpowers/plans/2026-10-01-feedback2-planN-<topic>.md`, keeping
   this roadmap's date so that the series sorts together.
3. Execute it with `superpowers:subagent-driven-development`, one fresh
   subagent per task, in sequence. The coordinator only dispatches, reviews
   and keeps the ledger.
4. Keep the ledger in `.superpowers/sdd/feedback2-planN/progress.md`.
5. Finish with `superpowers:requesting-code-review` (a fresh reviewer on the
   most capable model), then `superpowers:finishing-a-development-branch`.

## Plans

| # | Plan | Items | Branch | Status |
|---|---|---|---|---|
| 1 | Window and lifecycle | O6, O14, O20 | `feat/window-lifecycle` | done |
| 2 | Settings window | O2, O3, O4, O5 | `feat/settings-window` | done |
| 3 | Meter scale | O11, O13 | `fix/meter-scale` | done |
| 4 | Cartwall stop | O18, O19 | `feat/cartwall-stop-all` | done |
| 5 | Cue markers off | O15 | `feat/cue-markers-off` | done |
| 6 | Player and CUE | O8, O10, O12, O17 | `feat/cue-window` | outline |
| 7 | Track tags | O23 | `feat/track-tags` | outline |
| 8 | Track table | O7, O9, O16, O22, O24 | `feat/table-columns` | outline |
| 9 | Audit follow-ups | O21 | `fix/audit-follow-ups` | outline |
| 10 | Website | O1 | `docs/website` | outline |

## Notes for the later plans

- **Plan 2** reuses plan 1's on-air guard. `ExitIntent` gains a `Restart`
  variant, and the dialog takes its own text for it.
- **Plan 4.** `Command::StopAllCarts` already exists, so the plan adds only the
  button, its count, the shared glyphs module and the shortcut.
- **Plan 6** draws the CUE window's icons through plan 4's `ui/glyphs.rs`.
- **Plan 8** builds the new columns on plan 7's tag fields.
- **Plan 10** runs last, so that it publishes the final docs.
