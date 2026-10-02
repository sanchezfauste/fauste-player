#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The background analysis pool.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use fp_analysis::analyzer::{AnalyzeFn, Analyzer};
use fp_analysis::cache::AnalysisCache;
use fp_analysis::{AnalysisError, analyze_file};
use fp_model::{AnalysisSettings, Limits, TrackId};

fn wav(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 8_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    for i in 0..8_000 {
        w.write_sample(((i as f32 * 0.3).sin() * 10_000.0) as i16)
            .unwrap();
    }
    w.finalize().unwrap();
    path
}

const WAIT: Duration = Duration::from_secs(10);

#[test]
fn submitted_files_are_analysed_in_the_background() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "one.wav");
    let analyzer =
        Analyzer::spawn(2, AnalysisSettings::default(), Limits::default(), None).unwrap();
    analyzer.submit(TrackId(1), path);
    let result = analyzer.results().recv_timeout(WAIT).unwrap();
    assert_eq!(result.track, TrackId(1));
    assert!((result.outcome.unwrap().analysis.duration_secs - 1.0).abs() < 1e-6);
}

#[test]
fn cancelled_jobs_produce_no_result() {
    let slow: AnalyzeFn = Arc::new(|path, settings, limits, _cancelled| {
        std::thread::sleep(Duration::from_millis(200));
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
        slow,
    )
    .unwrap();
    analyzer.submit(TrackId(1), a);
    analyzer.submit(TrackId(2), b);
    analyzer.cancel(TrackId(2));
    assert_eq!(
        analyzer.results().recv_timeout(WAIT).unwrap().track,
        TrackId(1)
    );
    assert!(
        analyzer
            .results()
            .recv_timeout(Duration::from_millis(600))
            .is_err()
    );
}

#[test]
fn cached_results_are_returned_without_analysing_again() {
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = calls.clone();
    let counting: AnalyzeFn = Arc::new(move |path, settings, limits, _cancelled| {
        counter.fetch_add(1, Ordering::SeqCst);
        analyze_file(path, settings, limits)
    });
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "a.wav");
    let cache = AnalysisCache::new(dir.path().join("cache"), &Limits::default());
    let analyzer = Analyzer::with_analyze_fn(
        1,
        AnalysisSettings::default(),
        Limits::default(),
        Some(cache),
        counting,
    )
    .unwrap();
    analyzer.submit(TrackId(1), path.clone());
    let first = analyzer
        .results()
        .recv_timeout(WAIT)
        .unwrap()
        .outcome
        .unwrap();
    analyzer.submit(TrackId(1), path);
    let second = analyzer
        .results()
        .recv_timeout(WAIT)
        .unwrap()
        .outcome
        .unwrap();
    assert_eq!(first, second);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn a_panicking_decoder_is_reported_as_unreadable() {
    let exploding: AnalyzeFn = Arc::new(|_, _, _, _| panic!("boom"));
    let analyzer = Analyzer::with_analyze_fn(
        1,
        AnalysisSettings::default(),
        Limits::default(),
        None,
        exploding,
    )
    .unwrap();
    analyzer.submit(TrackId(9), PathBuf::from("/whatever.wav"));
    let result = analyzer.results().recv_timeout(WAIT).unwrap();
    assert!(matches!(result.outcome, Err(AnalysisError::Unreadable(_))));
}

#[test]
fn duplicate_submissions_are_analysed_once() {
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = calls.clone();
    let slow: AnalyzeFn = Arc::new(move |path, settings, limits, _cancelled| {
        counter.fetch_add(1, Ordering::SeqCst);
        std::thread::sleep(Duration::from_millis(100));
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
        slow,
    )
    .unwrap();
    analyzer.submit(TrackId(1), a);
    analyzer.submit(TrackId(2), b.clone());
    analyzer.submit(TrackId(2), b);
    analyzer.results().recv_timeout(WAIT).unwrap();
    analyzer.results().recv_timeout(WAIT).unwrap();
    assert!(
        analyzer
            .results()
            .recv_timeout(Duration::from_millis(400))
            .is_err()
    );
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn dropping_the_analyzer_does_not_wait_for_queued_jobs() {
    let slow: AnalyzeFn = Arc::new(|path, settings, limits, _cancelled| {
        std::thread::sleep(Duration::from_millis(100));
        analyze_file(path, settings, limits)
    });
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "a.wav");
    let analyzer = Analyzer::with_analyze_fn(
        1,
        AnalysisSettings::default(),
        Limits::default(),
        None,
        slow,
    )
    .unwrap();
    for n in 0..30 {
        analyzer.submit(TrackId(n), path.clone());
    }
    let started = std::time::Instant::now();
    drop(analyzer);
    assert!(
        started.elapsed() < Duration::from_millis(800),
        "drop took {:?}",
        started.elapsed()
    );
}

#[test]
fn cancel_stops_a_running_analysis() {
    let stopped = Arc::new(AtomicUsize::new(0));
    let flag = stopped.clone();
    let endless: AnalyzeFn = Arc::new(move |_, _, _, cancelled| {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while std::time::Instant::now() < deadline {
            if cancelled() {
                flag.fetch_add(1, Ordering::SeqCst);
                return Err(AnalysisError::Cancelled);
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        Err(AnalysisError::Unreadable("not cancelled".into()))
    });
    let analyzer = Analyzer::with_analyze_fn(
        1,
        AnalysisSettings::default(),
        Limits::default(),
        None,
        endless,
    )
    .unwrap();
    analyzer.submit(TrackId(1), PathBuf::from("/long.flac"));
    std::thread::sleep(Duration::from_millis(50));
    analyzer.cancel(TrackId(1));
    assert!(
        analyzer
            .results()
            .recv_timeout(Duration::from_millis(500))
            .is_err(),
        "no result for a cancelled job"
    );
    assert_eq!(
        stopped.load(Ordering::SeqCst),
        1,
        "the running analysis noticed the cancellation"
    );
}

#[test]
fn a_file_changed_during_analysis_is_not_cached_under_its_new_key() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "a.wav");
    let cache_dir = dir.path().join("cache");
    let rewriting: AnalyzeFn = Arc::new(|path, settings, limits, _cancelled| {
        let result = analyze_file(path, settings, limits);
        // The operator re-records the file while it is being analysed.
        std::fs::write(path, vec![0u8; 12_345]).unwrap();
        result
    });
    let cache = AnalysisCache::new(cache_dir.clone(), &Limits::default());
    let analyzer = Analyzer::with_analyze_fn(
        1,
        AnalysisSettings::default(),
        Limits::default(),
        Some(cache),
        rewriting,
    )
    .unwrap();
    analyzer.submit(TrackId(1), path.clone());
    analyzer.results().recv_timeout(WAIT).unwrap();
    let check = AnalysisCache::new(cache_dir, &Limits::default());
    assert!(
        check.load(&path, &AnalysisSettings::default()).is_none(),
        "stale result cached for the new file"
    );
}

#[test]
fn the_pool_sweeps_old_cache_entries_off_the_callers_thread() {
    let dir = tempfile::tempdir().unwrap();
    let cache_dir = dir.path().join("cache");
    std::fs::create_dir_all(&cache_dir).unwrap();
    let stale = cache_dir.join("0123456789abcdef.bin");
    std::fs::write(&stale, b"unversioned").unwrap();
    let cache = AnalysisCache::new(cache_dir, &Limits::default());
    assert!(stale.exists(), "opening the cache does not sweep");
    let analyzer = Analyzer::spawn(
        1,
        AnalysisSettings::default(),
        Limits::default(),
        Some(cache),
    )
    .unwrap();
    analyzer.submit(TrackId(1), wav(dir.path(), "a.wav"));
    analyzer.results().recv_timeout(WAIT).unwrap();
    assert!(!stale.exists(), "swept before the first job");
}

/// One worker, 30 ms per analysis, tracks 1 to 10 queued.
fn busy_pool(dir: &Path) -> (Analyzer, PathBuf) {
    let slow: AnalyzeFn = Arc::new(|path, settings, limits, _cancelled| {
        std::thread::sleep(Duration::from_millis(30));
        analyze_file(path, settings, limits)
    });
    let file = wav(dir, "a.wav");
    let analyzer = Analyzer::with_analyze_fn(
        1,
        AnalysisSettings::default(),
        Limits::default(),
        None,
        slow,
    )
    .unwrap();
    for n in 1..=10 {
        analyzer.submit(TrackId(n), file.clone());
    }
    (analyzer, file)
}

fn order(analyzer: &Analyzer, count: usize) -> Vec<TrackId> {
    (0..count)
        .map(|_| analyzer.results().recv_timeout(WAIT).unwrap().track)
        .collect()
}

/// A track a player shows needs its waveform now, not after the library.
#[test]
fn urgent_jobs_go_ahead_of_the_queue() {
    let dir = tempfile::tempdir().unwrap();
    let (analyzer, file) = busy_pool(dir.path());
    analyzer.submit_urgent(TrackId(99), file);
    let order = order(&analyzer, 11);
    let at = order.iter().position(|t| *t == TrackId(99)).unwrap();
    assert!(at <= 1, "after at most the running job: {order:?}");
}

#[test]
fn a_queued_job_promoted_goes_ahead_once() {
    let dir = tempfile::tempdir().unwrap();
    let (analyzer, file) = busy_pool(dir.path());
    analyzer.promote(TrackId(10), &file);
    analyzer.promote(TrackId(10), &file);
    let order = order(&analyzer, 10);
    let at = order.iter().position(|t| *t == TrackId(10)).unwrap();
    assert!(at <= 1, "after at most the running job: {order:?}");
    assert!(
        analyzer
            .results()
            .recv_timeout(Duration::from_millis(200))
            .is_err(),
        "analysed once"
    );
}

#[test]
fn promoting_a_job_already_answered_does_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let file = wav(dir.path(), "a.wav");
    let analyzer =
        Analyzer::spawn(1, AnalysisSettings::default(), Limits::default(), None).unwrap();
    analyzer.submit(TrackId(1), file.clone());
    analyzer.results().recv_timeout(WAIT).unwrap();
    analyzer.promote(TrackId(1), &file);
    assert!(
        analyzer
            .results()
            .recv_timeout(Duration::from_millis(300))
            .is_err()
    );
}

/// Analysis is background work (main spec §2.2): its threads run at the
/// lowest priority, so decoding and the interface keep the processor.
#[cfg(target_os = "linux")]
#[test]
fn the_pool_runs_at_low_priority() {
    use std::sync::Mutex;
    let seen = Arc::new(Mutex::new(Vec::new()));
    let record = seen.clone();
    let probe: AnalyzeFn = Arc::new(move |path, settings, limits, _cancelled| {
        // The nice value of the thread running this job (field 19).
        let stat = std::fs::read_to_string("/proc/thread-self/stat").unwrap();
        let nice: i32 = stat
            .rsplit_once(')')
            .unwrap()
            .1
            .split_whitespace()
            .nth(16)
            .unwrap()
            .parse()
            .unwrap();
        record.lock().unwrap().push(nice);
        analyze_file(path, settings, limits)
    });
    let dir = tempfile::tempdir().unwrap();
    let analyzer = Analyzer::with_analyze_fn(
        2,
        AnalysisSettings::default(),
        Limits::default(),
        None,
        probe,
    )
    .unwrap();
    for n in 0..4 {
        analyzer.submit(TrackId(n), wav(dir.path(), &format!("{n}.wav")));
    }
    for _ in 0..4 {
        analyzer.results().recv_timeout(WAIT).unwrap();
    }
    assert_eq!(*seen.lock().unwrap(), vec![19; 4]);
}
