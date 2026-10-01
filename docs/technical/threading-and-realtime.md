# Threading and real time

## Threads

| Thread | Name | Priority | Owns | May block? |
|---|---|---|---|---|
| Device callback, one per open output | set by the backend | real-time (rtkit/SCHED_FIFO on Linux, MMCSS on Windows, time-constraint on macOS, through cpal's `audio_thread_priority`) | nothing: it borrows the bus `Mixer` through `try_lock` | **never** |
| Null output, one per open Null device | `fp-null-output` | normal | paces the mixer of a Null bus in real time | sleeps between blocks |
| Virtual clock, one per lost bus | `fp-virtual-clock` | normal | renders the same mixer at real-time pace while the device is lost | sleeps between blocks |
| Decode worker, one per player | `fp-player-<id>` | normal | producer halves of the player's sources; decoders and resamplers | file I/O |
| Conductor | `fp-conductor` | normal | `AppState`, the `Engine` and all bus bookkeeping | no: it polls with a tick of `tuning.conductor_tick_ms` (5 ms) |
| Services | `fp-services` | normal | the analyzer handle, the store, the media cache writer | file I/O (saves) |
| Remote | `fp-remote` | normal | the tokio runtime of the HTTP API, the event publisher (every 50 ms) and the OSC socket; follows `config.remote` every 250 ms | network I/O (async); covers and peaks in `spawn_blocking` |
| Analysis pool (2) | `fp-analysis-<n>` | normal | one job at a time each | file I/O, CPU |
| Helpers | `fp-file-dialog`, `fp-folder-dialog`, `fp-drop-scan`, `fp-device-scan` | normal | native dialogs, folder scans and device enumeration, off the UI thread | yes (by design) |
| UI | main thread | normal | view state only | no |

## Rules for the real-time thread

The mixer (`fp-engine/src/mixer.rs`) runs on the device callback. It must not:

- **allocate or free.** Every buffer is preallocated when the mixer is built,
  or arrives with a command. Sources that leave the mixer are sent back
  through the `Retired` queue and dropped on the conductor thread. If that
  queue is full they are *forgotten* (a counted leak) rather than freed on
  the RT thread.
- **block or lock.** The only lock is the bus mixer's `Mutex`, and the
  callback only calls `try_lock`. On contention, which only happens while the
  virtual clock hands the mixer back, it outputs silence for one block and
  increments `lock_misses`.
- **log, do I/O or panic.** Counters (`xruns`, `underruns`, `leaked`,
  `misrouted`, `lock_misses`) are atomics that the conductor turns into log
  lines and UI alerts. Indexing uses `get` (enforced by
  `clippy::indexing_slicing`), and there is no `unwrap`.

Tests enforce this: `assert_no_alloc` wraps `Mixer::render` and fails on any
allocation or free.

## Communication

| From → to | Channel | Blocking? |
|---|---|---|
| UI → conductor | `crossbeam-channel` bounded (1024) `Command` | `try_send`; a full queue drops and logs |
| UI → conductor (test tones) | bounded (16) `EngineRequest` | `try_send` |
| Conductor → UI and services | `arc-swap` of `Arc<AppState>` and `Arc<Telemetry>` | wait-free reads |
| Conductor → mixer | `rtrb` SPSC `BusCommand` (at most `tuning.max_commands_per_block` drained per block) | wait-free |
| Mixer → conductor | `rtrb` SPSC `BusEvent` and `Retired` | wait-free |
| Worker ↔ mixer | one `rtrb` ring of interleaved stereo `f32` per source, plus atomics (`SourceShared`) | wait-free |
| Conductor → worker | `crossbeam-channel` | non-RT on both ends |
| Services → analysis pool → services | `crossbeam-channel` jobs and results | non-blocking polls |
| Helper threads → UI | `crossbeam-channel` | `try_recv` each frame |

No lock is shared between the UI and anything on the audio path.

## Panic containment

| Where | What happens |
|---|---|
| A decoder panics | caught by the worker (`catch_unwind`); the source becomes `Failed`, drains what is buffered, and the model marks the entry `Unreadable` and skips it |
| An analysis job panics | caught; reported as `Unreadable` |
| A services step panics | analysis and autosave are caught separately, so a fault in one never stops the other; faults are counted and shown as a status-bar alert |
| A UI frame panics | caught by `ui::shell::Shell`; the screen shows a banner with **Restart interface**; audio is unaffected |
| Any panic | the panic hook writes `crash-<nanos>.txt` to the log directory before unwinding, at most `limits.max_crash_reports` (20) per run; later panics are only logged |

Every profile builds with `panic = "unwind"`, which the containment above
relies on.
