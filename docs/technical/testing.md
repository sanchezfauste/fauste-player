# Testing

Run everything with `cargo test --workspace`. It takes about a minute on a
laptop and needs no sound card and no display.

| Layer | Where | How |
|---|---|---|
| Player rules (spec §3) | `fp-model/tests/*.rs` | Tests named after the rules they cover (`ruleN_…`); the display rules (1, 16–20) are tested in `fp-app` (view model and UI tests); plus proptest properties for playlist operations (nothing lost or duplicated, the current entry never removed) |
| Store | `fp-store/tests`, unit tests in `fp-store/src` | Round trips, migrations, corrupt-file fallback, atomic-write crash simulation, lenient config |
| Decoding | `fp-decode/tests`, `fp-engine/tests/decode.rs`, `worker.rs` | Formats, mono/multichannel downmix, seeks, truncated files, resampling in the worker |
| Backends | `fp-backends/tests` | Null and Offline behaviour; cpal format conversion |
| Mixer | `fp-engine/tests/mixer.rs` | Sample-exact fades and starts, underruns, and `assert_no_alloc` around `render` |
| Engine | `fp-engine/tests/engine.rs`, `review_fixes.rs` | The `Offline` backend renders on demand; tests drive time and check exact frames, events and slot bookkeeping (`used_slots`, `unsettled_sources`) |
| Buses | `fp-engine/tests/bus.rs` | Watchdog, virtual clock takeover, reconnect (Offline devices can be unplugged and replugged) |
| Conductor | `fp-engine/tests/conductor.rs` | End to end through the model; a stress test runs 8 players with random commands over 10 simulated minutes and several seeds, checking invariants |
| Soak | `fp-engine/tests/conductor.rs` (`#[ignore]`) | 6 simulated hours: `cargo test --release -p fp-engine --test conductor -- --ignored six_simulated_hours` |
| Analysis | `fp-analysis/tests`, unit tests in `signal.rs` and `cache.rs` | Markers on generated signals, tags and covers (including hostile inputs), the cache, the pool (cancellation, duplicates, panics) |
| App | `fp-app/tests` | bootstrap, i18n key parity, the view model, services (real analyzer and Offline engine), and the UI with `egui_kittest` (clicks, keys, context menu, drag targets, Settings, panic isolation) |

## Conventions

- **TDD.** Write the test, watch it fail for the expected reason, then write
  the code. Bug fixes start with a failing test that reproduces the bug.
- Tests may use `unwrap`, `expect` and `panic` (`clippy.toml`); production
  code may not.
- Time is always passed in (`now: Instant`) so tests control it; no test
  sleeps to wait for audio.
- UI tests use `Harness::builder().with_step_dt(0.02)` so that two clicks
  fall within the double-click delay.
- The `test-hooks` feature of `fp-app` is enabled for its own tests through
  a self dev-dependency.

## Fuzzing

`fuzz/` holds [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz) targets
for every parser of untrusted input. It is a separate crate, excluded from
the workspace, and needs the nightly toolchain:

| Target | Input |
|---|---|
| `m3u`, `pls` | playlist files (`fp_store::playlist_io::parse_playlist`) |
| `cart_page` | cart page files, parsed and then imported into the model |
| `store_documents` | the same bytes as `config.json`, `playlists.json`, `session.json` and `carts.json`, then the restore |

```sh
rustup toolchain install nightly --profile minimal
cargo install cargo-fuzz --locked
cd fuzz && cargo +nightly fuzz run m3u -- -max_total_time=60
```

The `fuzz` workflow runs each target for five minutes every night and
uploads any crash as an artefact.

## Checks run in CI

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo deny check          # licences, advisories, sources
```

fmt, clippy and tests run on Linux, Windows and macOS for pushes to `master`
and for every pull request; `cargo deny` and the release-bump check run on
Linux.
