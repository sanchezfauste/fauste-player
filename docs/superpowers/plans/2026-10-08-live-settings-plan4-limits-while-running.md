# Live Settings, Plan 4: Limits While Running

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The limits the application copied at start (cover and tag limits in the analysis pool and its cache, the remote's cache reader, the crash report cap) follow the configuration while it runs (live settings spec §6).

**Architecture:** `Analyzer` keeps its `Limits` behind the same kind of lock as its settings (`update_limits`, like `update_settings`); `AnalysisCache` keeps its limits in atomics (`set_limits`), so the analysis pool and the remote's reader can update the one they hold. The services thread watches `state.config.limits` as it already watches `config.analysis`; the remote reads the limits from the model snapshot on each request. The panic hook reads its cap from an `Arc<AtomicUsize>` (`ReportCap`) that `main` sets after the configuration loads and the services thread sets on each change.

**Tech Stack:** Rust 2024, `fp-analysis`, `fp-app` (services, remote bridge, crash hook).

**Spec:** `docs/superpowers/specs/2026-10-07-live-settings-design.md` §6 (and the whole spec for context). Independent of plans 2 and 3; needs nothing from plan 1. Rules L18 and L19 (player and cart-grid limit reductions) are dropped by the maintainer's ruling of 2026-10-08 (limits never change while the application runs); the §6 rows for `max_players` and the cart grid need no work here.

## Global Constraints

- All code, identifiers, comments, docs and commit messages in English; never mention other playout or radio-automation products.
- §6: new cover and tag limits apply to analyses started after the change; the library is not analysed again (cache entries keyed by the old limits simply miss); existing tags are not trimmed again. `max_state_file_bytes`, `backup_count`, `max_playlist_file_bytes` are already read live: no change.
- The UI never blocks (rule 8); nothing here runs on the UI thread.
- `fp-analysis` denies `clippy::indexing_slicing`; `unwrap`/`expect`/`panic` denied outside tests; test-only hooks go behind the `test-hooks` feature of `fp-app`.
- Conventional Commits; every commit ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Gate for every task: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.

## Review Focus

1. **An analysis running while the limits change**: it finishes with the limits it started with, and its result is cached under the key it computed first (no mixed key). Test: Task 1, `new_limits_reach_the_jobs_that_start_after_them` (the first job's limits are recorded before the change).
2. **A cache entry written under the old cover limits**: after the change it simply misses (the key differs), it is not served. Test: Task 1, `the_cache_key_follows_new_cover_limits`.
3. **A crash report cap lowered below what was already written this run**: no more reports are written, none are deleted. Test: Task 2, `the_report_cap_follows_the_configuration` (raise, then lower below the count).
4. **The remote asked for a waveform right after a limit change**: its cache reader uses the snapshot's limits for that request. Covered by code review: `Bridge::analysis` sets them on every call (Task 1, Step 3); a test would need a running server.
5. **The first services step**: the limits the pool was spawned with equal the configuration, so nothing is cancelled or analysed again. Test: Task 1, `new_limits_reach_the_analyzer` checks the analyses count stays at its value.

---

## File Structure

| File | Responsibility |
|---|---|
| `crates/fp-analysis/src/analyzer.rs` | `Shared::limits` behind a lock; `update_limits`, `limits` |
| `crates/fp-analysis/src/cache.rs` | Limits in atomics; `set_limits` |
| `crates/fp-app/src/services.rs` | `follow_limits`; `with_report_cap`; test hook `analyzer_limits` |
| `crates/fp-app/src/remote.rs` | Cache reader takes the snapshot's limits per request |
| `crates/fp-app/src/crash.rs` | `ReportCap` |
| `crates/fp-app/src/main.rs` | Keeps the cap, sets it after load, hands it to the services |
| `docs/technical/persistence.md`, `docs/technical/threading-and-realtime.md` | Limits apply while running |

---

### Task 1: Cover and tag limits reach the analysis while running

**Files:**
- Modify: `crates/fp-analysis/src/analyzer.rs:82-89` (`Shared`), `127-135` (`with_analyze_fn`), `225-228` (after `update_settings`), `310-330` (`run`)
- Modify: `crates/fp-analysis/src/cache.rs:33-61` (`AnalysisCache`, `new`), `93-100` (`key`), `113-140` (size checks)
- Modify: `crates/fp-app/src/services.rs:318-395` (fields), `396-460` (`new`), `572-581` (`analysis_step`), new `follow_limits`, test hook
- Modify: `crates/fp-app/src/remote.rs:32-37` (`Bridge::analysis`)
- Test: `crates/fp-analysis/tests/analyzer.rs`, `crates/fp-app/tests/services.rs`

**Interfaces:**
- Produces:
  - `Analyzer::update_limits(&self, limits: Limits)`; `Analyzer::limits(&self) -> Limits`
  - `AnalysisCache::set_limits(&self, limits: &Limits)` (takes `&self`)
  - `Services::analyzer_limits(&self) -> Limits` behind `#[cfg(feature = "test-hooks")]`

- [ ] **Step 1: Write the failing tests**

Append to `crates/fp-analysis/tests/analyzer.rs` (add `use std::sync::Mutex;`):

```rust
#[test]
fn new_limits_reach_the_jobs_that_start_after_them() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let capture: AnalyzeFn = Arc::new(move |path, settings, limits, _cancelled| {
        record.lock().unwrap().push(limits.max_cover_pixels);
        analyze_file(path, settings, limits)
    });
    let dir = tempfile::tempdir().unwrap();
    let a = wav(dir.path(), "a.wav");
    let b = wav(dir.path(), "b.wav");
    let analyzer = Analyzer::with_analyze_fn(
        1,
        AnalysisSettings::default(),
        Limits::default(),
        None,
        capture,
    )
    .unwrap();
    analyzer.submit(TrackId(1), a);
    analyzer.results().recv_timeout(WAIT).unwrap();
    analyzer.update_limits(Limits {
        max_cover_pixels: 1_000,
        ..Limits::default()
    });
    assert_eq!(analyzer.limits().max_cover_pixels, 1_000);
    analyzer.submit(TrackId(2), b);
    analyzer.results().recv_timeout(WAIT).unwrap();
    assert_eq!(*seen.lock().unwrap(), vec![8_000, 1_000]);
}

#[test]
fn the_cache_key_follows_new_cover_limits() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "a.wav");
    let cache = AnalysisCache::new(dir.path().join("cache"), &Limits::default());
    let settings = AnalysisSettings::default();
    let before = cache.key(&path, &settings).unwrap();
    cache.set_limits(&Limits {
        max_cover_pixels: 1_000,
        ..Limits::default()
    });
    assert_ne!(cache.key(&path, &settings).unwrap(), before);
}
```

Append to `crates/fp-app/tests/services.rs`:

```rust
#[test]
fn new_limits_reach_the_analyzer() {
    let dir = tempfile::tempdir().unwrap();
    let a = wav(dir.path(), "a.wav", 1);
    let mut r = rig(&[a], dir);
    r.run_until("analysis", |r| {
        r.handle.model.load().library.iter().all(|t| t.analyzed)
    });
    let analysed = r.analyses.load(Ordering::SeqCst);
    let mut config = r.handle.model.load().config.clone();
    config.limits.max_cover_pixels = 1_000;
    r.handle.send(Command::UpdateConfig(Box::new(config)));
    r.run_until("the analyzer has the new limits", |r| {
        r.services.analyzer_limits().max_cover_pixels == 1_000
    });
    assert_eq!(
        r.analyses.load(Ordering::SeqCst),
        analysed,
        "the library is not analysed again"
    );
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p fp-analysis --test analyzer && cargo test -p fp-app --test services new_limits`
Expected: FAIL to compile: `no method named update_limits`, `no method named set_limits`, `no method named analyzer_limits`.

- [ ] **Step 3: Write the implementation**

`crates/fp-analysis/src/analyzer.rs`:
- `Shared`: `limits: Limits,` becomes `limits: Mutex<Limits>,`; in `with_analyze_fn`, `limits,` becomes `limits: Mutex::new(limits),`.
- After `update_settings`:

```rust
    /// Limits for jobs that start after this call (live settings spec §6);
    /// the cache keys follow them, so entries made under the old limits
    /// miss. Nothing is analysed again.
    pub fn update_limits(&self, limits: Limits) {
        if let Some(cache) = &self.shared.cache {
            cache.set_limits(&limits);
        }
        *lock(&self.shared.limits) = limits;
    }

    /// The limits jobs start with now.
    pub fn limits(&self) -> Limits {
        lock(&self.shared.limits).clone()
    }
```

- In `run` (L310-330), after `let settings = lock(&shared.settings).clone();` add `let limits = lock(&shared.limits).clone();` and pass `&limits` instead of `&shared.limits` to `(shared.analyze)(…)`.

`crates/fp-analysis/src/cache.rs`:
- `use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};` (keep the existing `AtomicU64` import, add `AtomicU32` and `Ordering` if missing).
- Fields: replace `max_bytes: u64,` and `cover_limits: (u64, u32),` with

```rust
    /// Limits read at each use: the services thread and the remote's
    /// reader update them while running (live settings spec §6).
    max_bytes: AtomicU64,
    cover_bytes: AtomicU64,
    cover_pixels: AtomicU32,
```

- `new`: `max_bytes: AtomicU64::new(limits.max_state_file_bytes), cover_bytes: AtomicU64::new(limits.max_cover_bytes), cover_pixels: AtomicU32::new(limits.max_cover_pixels),`
- New method after `new`:

```rust
    /// New limits for keys computed and entries read or written after
    /// this call.
    pub fn set_limits(&self, limits: &Limits) {
        self.max_bytes
            .store(limits.max_state_file_bytes, Ordering::Relaxed);
        self.cover_bytes.store(limits.max_cover_bytes, Ordering::Relaxed);
        self.cover_pixels
            .store(limits.max_cover_pixels, Ordering::Relaxed);
    }
```

- In `key`: `let (cover_bytes, cover_pixels) = self.cover_limits;` becomes

```rust
        let cover_bytes = self.cover_bytes.load(Ordering::Relaxed);
        let cover_pixels = self.cover_pixels.load(Ordering::Relaxed);
```

- Both `self.max_bytes` reads (L115, L138) become `self.max_bytes.load(Ordering::Relaxed)`.

`crates/fp-app/src/services.rs`:
- Field after `settings: Option<AnalysisSettings>,`: `/// The limits the analysis was last given (live settings spec §6).\n    limits: Option<fp_model::Limits>,` and `limits: None,` in `new`.
- In `analysis_step`, after `self.follow_settings(state);` add `self.follow_limits(state);`.
- After `follow_settings`:

```rust
    /// Live settings spec §6: new cover and tag limits reach the analysis
    /// and its cache for jobs that start after them; nothing is analysed
    /// again.
    fn follow_limits(&mut self, state: &AppState) {
        let current = &state.config.limits;
        if self.limits.as_ref() == Some(current) {
            return;
        }
        self.analyzer.update_limits(current.clone());
        self.limits = Some(current.clone());
    }

    /// The limits the analysis has now. Used by tests.
    #[cfg(feature = "test-hooks")]
    pub fn analyzer_limits(&self) -> fp_model::Limits {
        self.analyzer.limits()
    }
```

`crates/fp-app/src/remote.rs`, `Bridge::analysis`: after `let settings = &model.config.analysis;` add

```rust
        // The limits of this snapshot (live settings spec §6).
        self.cache.set_limits(&model.config.limits);
```

`docs/technical/persistence.md`, under "### `limits` (config file only)", before the table, add:

```markdown
Changed while the application runs (live settings spec §6): a higher limit
applies at once; a lower `max_players` removes the players above it from the
end once they are stopped, and a lower `max_cart_rows`/`max_cart_cols`
shrinks a page once nothing on it plays and no cart with a file would be
dropped. New cover and tag limits apply to analyses started afterwards (the
analysis pool, its cache and the remote's cache reader take them; nothing
is analysed again). The others are read where they are used.
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p fp-analysis && cargo test -p fp-app --test services`
Expected: PASS.

- [ ] **Step 5: Run the gate and commit**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`

```bash
git add crates/fp-analysis crates/fp-app/src/services.rs crates/fp-app/src/remote.rs \
  crates/fp-app/tests/services.rs docs/technical/persistence.md
git commit -m "$(cat <<'EOF'
feat(analysis): take new cover and tag limits while running

The analysis pool, its cache and the remote's cache reader follow the
configured limits (live settings spec §6); analyses started afterwards
use them and nothing is analysed again.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
EOF
)"
```

---

### Task 2: The crash report cap follows the configuration

**Files:**
- Modify: `crates/fp-app/src/crash.rs`
- Modify: `crates/fp-app/src/services.rs` (field, builder, `follow_limits`)
- Modify: `crates/fp-app/src/main.rs:104-107` (`install_panic_hook`), `run` signature and body
- Create: `crates/fp-app/tests/crash_cap_live.rs` (its own process: a panic hook is global)
- Modify: `docs/technical/threading-and-realtime.md:113`

**Interfaces:**
- Produces:
  - `pub struct ReportCap` (`Clone`), `ReportCap::set(&self, max: usize)`
  - `install_panic_hook(log_dir: PathBuf, max_reports: usize) -> ReportCap`
  - `Services::with_report_cap(self, cap: ReportCap) -> Self`

- [ ] **Step 1: Write the failing test**

Create `crates/fp-app/tests/crash_cap_live.rs`:

```rust
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Live settings spec §6: the crash report cap follows the configuration
//! while the application runs. Its own test binary: a panic hook is
//! global to the process.

use std::path::Path;

use fp_app::crash::install_panic_hook;

fn reports(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .unwrap()
        .filter(|e| {
            e.as_ref()
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with("crash-")
        })
        .count()
}

#[test]
fn the_report_cap_follows_the_configuration() {
    let dir = tempfile::tempdir().unwrap();
    let cap = install_panic_hook(dir.path().to_path_buf(), 1);
    cap.set(3);
    for _ in 0..5 {
        let _ = std::panic::catch_unwind(|| panic!("again"));
    }
    assert_eq!(reports(dir.path()), 3);
    cap.set(1);
    let _ = std::panic::catch_unwind(|| panic!("once more"));
    assert_eq!(reports(dir.path()), 3, "none written, none deleted");
}
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p fp-app --test crash_cap_live`
Expected: FAIL to compile: `no method named set found for unit type ()`.

- [ ] **Step 3: Write the implementation**

`crates/fp-app/src/crash.rs`: update the module doc's last sentence to "At most the configured number of files (`limits.max_crash_reports`, followed while the application runs) are written per run, …", add `use std::sync::Arc;`, and:

```rust
/// How many crash reports a run may write; shared with the panic hook so
/// that it follows `limits.max_crash_reports` while the application runs
/// (live settings spec §6).
#[derive(Debug, Clone)]
pub struct ReportCap(Arc<AtomicUsize>);

impl ReportCap {
    pub fn set(&self, max: usize) {
        self.0.store(max, Ordering::Release);
    }
}
```

Change `install_panic_hook` to return it:

```rust
pub fn install_panic_hook(log_dir: PathBuf, max_reports: usize) -> ReportCap {
    let cap = ReportCap(Arc::new(AtomicUsize::new(max_reports)));
    let max = cap.0.clone();
    let previous = std::panic::take_hook();
    let written = AtomicUsize::new(0);
    std::panic::set_hook(Box::new(move |info| {
        // … unchanged …
        if written.fetch_add(1, Ordering::AcqRel) < max.load(Ordering::Acquire) {
            // … unchanged …
        }
        tracing::error!(%thread, %location, "panic: {message}");
        previous(info);
    }));
    cap
}
```

(`crates/fp-app/tests/crash_cap.rs` keeps calling it as a statement; the returned value is dropped, which is fine: the hook holds its own `Arc`.)

`crates/fp-app/src/services.rs`:
- Field: `/// The crash report cap (live settings spec §6), when \`main\` gave it.\n    report_cap: Option<crate::crash::ReportCap>,` and `report_cap: None,` in `new`.
- Builder after `new`:

```rust
    /// Keeps the crash report cap in step with `limits.max_crash_reports`.
    pub fn with_report_cap(mut self, cap: crate::crash::ReportCap) -> Self {
        self.report_cap = Some(cap);
        self
    }
```

- In `follow_limits`, before `self.limits = Some(current.clone());`:

```rust
        if let Some(cap) = &self.report_cap {
            cap.set(current.max_crash_reports);
        }
```

`crates/fp-app/src/main.rs`:
- L104-107 becomes `let report_cap = crash::install_panic_hook(paths.log_dir.clone(), fp_model::Limits::default().max_crash_reports);` (the default until the configuration is read).
- `let result = run(paths, playlists, report_cap);` and `fn run(paths: fp_store::AppPaths, playlists: Vec<std::path::PathBuf>, report_cap: crash::ReportCap) -> …`.
- In `run`, after `let config = loaded.state.config.clone();` add `report_cap.set(config.limits.max_crash_reports);`, and build the services with `Services::new(handle.clone(), store, analyzer, media.clone()).with_report_cap(report_cap);`.

`docs/technical/threading-and-realtime.md:113`: replace "at most `limits.max_crash_reports` (20) per run" with "at most `limits.max_crash_reports` (20) per run (the default until the configuration is read; then the configured value, followed while the application runs)".

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo test -p fp-app --test crash_cap_live --test crash_cap`
Expected: PASS.

- [ ] **Step 5: Run the gate and commit**

Run: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`

```bash
git add crates/fp-app docs/technical/threading-and-realtime.md
git commit -m "$(cat <<'EOF'
feat(app): follow the configured crash report cap

The panic hook is armed before the configuration loads; its cap is now
set from the configuration after load and on each change (live
settings spec §6).

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
EOF
)"
```

---

## Self-Review (done while writing)

- **Spec coverage (§6):** `max_players`, `max_cart_rows/cols` (plan 1), `max_cover_bytes/pixels`, `max_tag_chars/values` (Task 1: pool, cache, remote; the tag editor and the tag pass already read `state.config.limits` per job), `max_state_file_bytes`, `backup_count`, `max_playlist_file_bytes` (already live; the cache's own `max_state_file_bytes` now follows too), `max_crash_reports` (Task 2).
- **Placeholders:** the two "unchanged" markers in the crash hook stand for code that stays exactly as it is; only the comparison line changes.
- **Type consistency:** `update_limits`, `limits`, `set_limits`, `ReportCap::set`, `with_report_cap`, `analyzer_limits` as used across tasks.
