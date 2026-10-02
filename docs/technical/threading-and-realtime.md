# Threading and real time

## Threads

| Thread | Name | Priority | Owns | May block? |
|---|---|---|---|---|
| Device callback, one per open output | set by the backend | real-time (rtkit/SCHED_FIFO on Linux, MMCSS on Windows, time-constraint on macOS, through cpal's `audio_thread_priority`) | nothing: it borrows the bus `Mixer` through `try_lock` | **never** |
| Null output, one per open Null device | `fp-null-output` | normal | paces the mixer of a Null bus in real time | sleeps between blocks |
| Virtual clock, one per lost bus | `fp-virtual-clock` | normal | renders the same mixer at real-time pace while the device is lost | sleeps between blocks |
| Decode worker, one per player, and one for the cartwall | `fp-player-<id>`, `fp-cartwall` | above normal, not real time (`fp_decode::priority`) | producer halves of the player's sources; decoders and resamplers | file I/O |
| Conductor | `fp-conductor` | normal | `AppState`, the `Engine` and all bus bookkeeping | no: it polls with a tick of `tuning.conductor_tick_ms` (5 ms) |
| Services | `fp-services` | normal | the analyzer handle, the store, the media cache writer | file I/O (saves) |
| Remote | `fp-remote` | normal | the tokio runtime of the HTTP API, the event publisher (every 50 ms) and the OSC socket; follows `config.remote` every 250 ms | network I/O (async); covers and peaks in `spawn_blocking` |
| Analysis pool (2) | `fp-analysis-<n>` | low (`fp_decode::priority`) | one job at a time each | file I/O, CPU |
| Helpers | `fp-file-dialog`, `fp-folder-dialog`, `fp-drop-scan`, `fp-device-scan` | normal | native dialogs, folder scans and device enumeration, off the UI thread | yes (by design) |
| UI | main thread | normal | view state only | no |

## Thread priority

Main spec §2.2: only the device callback is real time. The two other classes
of thread that do heavy work are tuned with the `thread-priority` crate,
through `fp_decode::priority::set_current(Priority, role)` called by the thread
itself as its first action:

| Class | Request | Linux | Windows | macOS |
|---|---|---|---|---|
| Decode workers (`PlayerWorker`) | `Priority::AboveNormal`: the crate's cross-platform value 60 (37 on macOS, inside the 15..=47 range the crate checks there) | nice −5 | above normal | a step above the default |
| Analysis pool (`Analyzer`) | `Priority::Low`: the crate's minimum | nice 19 | lowest | lowest |

- **Neither is real time.** A decoder at a real-time class could starve the
  interface and the conductor, and it has a whole ring buffer of slack; the
  device callback is the one thread with a deadline.
- **Lowering a priority always works; raising one may not.** On Linux an
  ordinary user may not lower a nice value (`RLIMIT_NICE` or `CAP_SYS_NICE`
  is needed), so decoders
  often stay at normal priority there, and the analysis pool, whose request is
  always allowed, still yields to them.
- **A refusal is logged once and ignored.** `set_current` returns `false` and
  the thread keeps running at the normal priority. The first refusal of the run
  is a `warn` line (`the system refused a thread priority change`, with the
  role, the request and the operating system's error); later ones are silent,
  since every thread would repeat it.
- The conductor, the services thread, the remote thread and the helpers stay at
  normal priority. The services thread's saves are short and bursty, and the
  spec's "persistence writes on the background pool" is served by it running
  beside the pool, not inside it.
- Tests: `fp-decode` (`priority.rs`) reads the thread's nice value from
  `/proc` on Linux; `fp-engine/tests/decoder_priority.rs` checks that decode
  workers ask and that two refusals are one log line; `fp-analysis`
  (`the_pool_runs_at_low_priority`) checks nice 19 inside a job.

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
