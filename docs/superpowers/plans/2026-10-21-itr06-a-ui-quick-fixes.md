# itr06 plan A: UI quick fixes

Goal: six small operator-feedback fixes (I5, I9, I15, I7, I3, I2). One commit per item, test first.

| Item | Change | Files | Tests | Strings |
|---|---|---|---|---|
| I5 | A one-line hint for target, true-peak ceiling, maximum boost and maximum trim | `ui/settings/loudness.rs` | `settings_analysis_hints.rs` | `settings-hint-normalize-{target,ceiling,max-boost,max-trim}` |
| I9 | Outro in the status-bar legend; swatch before its label in the right-to-left bar | `app.rs` (`status_legend`) | `ui_polish.rs` | `legend-outro` |
| I15 | Fader readout centred on the knob axis (shared `fader_axis_x`) | `ui/widgets.rs`, `ui/player.rs` | `fader_readout_ui.rs` | none |
| I7 | Cart path: buttons laid out first, label truncated in the rest | `ui/settings/carts.rs` | `settings_layout.rs` | none |
| I3 | "Loudness…" in the playlist row menu | `ui/table.rs`, caller | `loudness_ui.rs` | `menu-loudness` |
| I2 | Gain badge not dimmed; tooltip states the applied gain; trim line only when not 0 | `ui/player.rs`, `ui/view.rs` | `loudness_ui.rs`, `ui_polish.rs` | `tip-gain-applied`, `tip-gain-limited-*`, `tip-gain-trim` |

Docs: `docs/user/loudness.md`, `docs/user/players.md`, `docs/user/playlists.md`, the loudness spec (N18-N22) and the main spec legend line.
All strings go to the 11 locales.
