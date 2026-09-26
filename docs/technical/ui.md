# User interface (`fp-app`)

## Structure

| Module | Role |
|---|---|
| `main.rs` | Wiring: paths, logging, crash hook, store, engine, conductor, analyzer, services, eframe |
| `bootstrap.rs` | `AppPaths` from `FAUSTE_HOME` or the OS directories |
| `logging.rs` | tracing to a daily-rotated file (14 kept) via a non-blocking writer; stderr in debug builds; `RUST_LOG` overrides |
| `crash.rs` | Panic hook writing `crash-<nanos>.txt` (message, location, backtrace, version, OS), then chaining |
| `i18n.rs` | Fluent bundles (`locales/en-US`, `locales/es-ES`), per-key fallback to `en-US` |
| `services.rs` | The services thread (analysis and autosave; see [Persistence](persistence.md) and [Analysis](analysis.md)) |
| `ui/app.rs` | `AppUi`: the main screen, keyboard, notices, OS drops, file-dialog results |
| `ui/player.rs` | One player column: header, info row, transport, waveform, tabs, footer |
| `ui/table.rs` | The track table (virtualised rows, drag and drop, context menu, column widths) |
| `ui/settings.rs` | The Settings modal |
| `ui/shell.rs` | Panic isolation around each frame |
| `ui/view.rs`, `ui/format.rs` | Pure view model: what to show, how to format it (unit-tested) |
| `ui/widgets.rs`, `ui/icons.rs`, `ui/theme.rs` | Painted widgets (tiles, VU, fader, waveform), drawn icons, the Nocturne theme |
| `ui/controller.rs` | The `Controller` trait between the UI and the rest |
| `ui/files.rs` | Accepted audio extensions and folder expansion |

## Data flow

Each frame, `AppUi::ui`:

1. loads `ctl.model()` (an `Arc<AppState>`) and `ctl.telemetry()`;
2. drains refusals (shown as a transient notice) and results of helper threads
   (file dialogs, dropped folders, device scans);
3. handles the keyboard, unless a text field has focus: only first presses
   count, and repeats are ignored;
4. draws the screen and sends `Command`s through `ctl.send`;
5. requests a repaint: continuously while anything plays, fades or cues,
   otherwise every 100 ms for the clock.

The UI keeps only **view state**: selection, drag target, meter peak holds,
column widths being dragged, the Settings section. Everything else comes
from the snapshot.

`Controller` is implemented by `ConductorHandle`. The tests use a fake that
applies commands with `fp_model::apply` and records them and their refusals,
so the UI is tested end to end without audio.

## Nothing blocks the UI thread

Native file and folder dialogs (`rfd::AsyncFileDialog` driven by `pollster`),
scans of dropped folders and device enumeration all run on helper threads.
Their results come back through channels.

## Theme and fonts

`ui/theme.rs` holds the Nocturne tokens as sRGB constants. The design's
`oklch` colours are converted once. Corners are square. Inter (400/500/600,
OFL) is embedded, and Phosphor icons come from `egui-phosphor`, regular and
fill. Waveform colours are a named palette (`WAVE_PALETTE`); an unknown name
falls back to Sand. Fonts are installed on the first frame, and drawing starts
on the next one, when they are bound.

## Panic isolation

`Shell::ui` runs `AppUi::ui` inside `catch_unwind`. After a panic it draws only
a banner with **Restart interface**, which calls `AppUi::reset_view`.
The `test-hooks` feature adds `AppUi::fail_next_frame` and
`Services::fail_next_step` for tests. Release builds do not contain them.

## Internationalisation

- Every visible string is a Fluent message in `locales/en-US/main.ftl`
  (the source) and `locales/es-ES/main.ftl`.
- `tests/i18n.rs` fails if the two key sets differ.
- Arguments use `tr_args(key, &[("name", value.into())])`.
- To add a language, add `locales/<tag>/main.ftl`, register it in `i18n.rs`,
  and extend the key-set test.
