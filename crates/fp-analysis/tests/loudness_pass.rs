#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The loudness-only pass in the analysis pool (loudness normalization
//! spec N6b).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fp_analysis::analyzer::{AnalyzeFn, Analyzer, LoudnessFn};
use fp_analysis::cache::AnalysisCache;
use fp_analysis::{AnalysisError, analyze_file, measure_loudness_cancellable};
use fp_model::{AnalysisSettings, LOUDNESS_VERSION, Limits, LoudnessMeasurement, TrackId};

const WAIT: Duration = Duration::from_secs(10);

fn wav(dir: &Path, name: &str, secs: u32) -> PathBuf {
    let path = dir.join(name);
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 8_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    for i in 0..8_000 * secs {
        w.write_sample(((i as f32 * 0.3).sin() * 10_000.0) as i16)
            .unwrap();
    }
    w.finalize().unwrap();
    path
}

fn real_analyze() -> AnalyzeFn {
    Arc::new(|path, settings, limits, cancelled| {
        fp_analysis::analyze_file_cancellable(path, settings, limits, cancelled)
    })
}

fn real_loudness() -> LoudnessFn {
    Arc::new(|path, cancelled| measure_loudness_cancellable(path, cancelled))
}

/// A loudness function that counts its runs and measures for real.
fn counting_loudness(runs: &Arc<AtomicUsize>) -> LoudnessFn {
    let runs = runs.clone();
    Arc::new(move |path, cancelled| {
        runs.fetch_add(1, Ordering::SeqCst);
        measure_loudness_cancellable(path, cancelled)
    })
}

#[test]
fn a_loudness_job_measures_and_fills_the_cache_entry_in_place() {
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "a.wav", 2);
    let cache_dir = dir.path().join("cache");
    let settings = AnalysisSettings::default();
    let limits = Limits::default();
    // An entry as the previous release wrote it: no loudness.
    let old = AnalysisCache::new(cache_dir.clone());
    let mut a = analyze_file(&path, &settings, &limits).unwrap();
    a.analysis.loudness = None;
    a.analysis.loudness_pass = None;
    old.store(&path, &settings, &limits, &a).unwrap();
    let analyzer = Analyzer::with_fns(
        2,
        settings.clone(),
        limits.clone(),
        Some(AnalysisCache::new(cache_dir)),
        real_analyze(),
        real_loudness(),
    )
    .unwrap();
    analyzer.submit_loudness(TrackId(1), path.clone());
    let r = analyzer.loudness_results().recv_timeout(WAIT).unwrap();
    assert_eq!(r.track, TrackId(1));
    let measured = r.outcome.unwrap();
    assert!(measured.is_some());
    let back = old.load(&path, &settings, &limits).unwrap();
    assert_eq!(back.analysis.loudness, measured);
    assert_eq!(back.analysis.loudness_pass, Some(LOUDNESS_VERSION));
    assert_eq!(back.peaks, a.peaks, "markers and peaks untouched");
    assert_eq!(back.analysis.segue_start, a.analysis.segue_start);
    assert!(analyzer.results().try_recv().is_err(), "no full result");
}

#[test]
fn a_loudness_job_reads_a_measurement_already_cached() {
    // The library lost the value (an older backup restored, say) but the
    // cache entry has it from the current pass: nothing is decoded.
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "a.wav", 1);
    let cache_dir = dir.path().join("cache");
    let settings = AnalysisSettings::default();
    let limits = Limits::default();
    let cached = LoudnessMeasurement {
        integrated_lufs: Some(-7.5),
        true_peak_dbtp: -0.5,
    };
    let mut a = analyze_file(&path, &settings, &limits).unwrap();
    a.analysis.loudness = Some(cached);
    a.analysis.loudness_pass = Some(LOUDNESS_VERSION);
    AnalysisCache::new(cache_dir.clone())
        .store(&path, &settings, &limits, &a)
        .unwrap();
    let runs = Arc::new(AtomicUsize::new(0));
    let analyzer = Analyzer::with_fns(
        1,
        settings,
        limits,
        Some(AnalysisCache::new(cache_dir)),
        real_analyze(),
        counting_loudness(&runs),
    )
    .unwrap();
    analyzer.submit_loudness(TrackId(1), path);
    let r = analyzer.loudness_results().recv_timeout(WAIT).unwrap();
    assert_eq!(r.outcome, Ok(Some(cached)));
    assert_eq!(runs.load(Ordering::SeqCst), 0);
}

#[test]
fn a_new_normal_job_goes_ahead_of_waiting_loudness_jobs() {
    let dir = tempfile::tempdir().unwrap();
    let file = wav(dir.path(), "a.wav", 1);
    let gate = Arc::new(AtomicBool::new(false));
    let order = Arc::new(Mutex::new(Vec::<&'static str>::new()));
    let (g, o) = (gate.clone(), order.clone());
    let analyze: AnalyzeFn = Arc::new(move |path, settings, limits, _| {
        while !g.load(Ordering::Acquire) {
            std::thread::sleep(Duration::from_millis(1));
        }
        o.lock().unwrap().push("normal");
        analyze_file(path, settings, limits)
    });
    let o = order.clone();
    let loudness: LoudnessFn = Arc::new(move |path, cancelled| {
        o.lock().unwrap().push("loudness");
        measure_loudness_cancellable(path, cancelled)
    });
    let analyzer = Analyzer::with_fns(
        1,
        AnalysisSettings::default(),
        Limits::default(),
        None,
        analyze,
        loudness,
    )
    .unwrap();
    // The only worker is busy with a normal job…
    analyzer.submit(TrackId(1), file.clone());
    std::thread::sleep(Duration::from_millis(50));
    // …while two loudness jobs and then a normal one are queued.
    analyzer.submit_loudness(TrackId(2), file.clone());
    analyzer.submit_loudness(TrackId(3), file.clone());
    analyzer.submit(TrackId(4), file.clone());
    gate.store(true, Ordering::Release);
    for _ in 0..2 {
        analyzer.results().recv_timeout(WAIT).unwrap();
    }
    for _ in 0..2 {
        analyzer.loudness_results().recv_timeout(WAIT).unwrap();
    }
    assert_eq!(
        *order.lock().unwrap(),
        vec!["normal", "normal", "loudness", "loudness"]
    );
}

#[test]
fn a_normal_and_a_loudness_job_for_the_same_track_both_run() {
    let dir = tempfile::tempdir().unwrap();
    let file = wav(dir.path(), "a.wav", 1);
    let analyzer =
        Analyzer::spawn(2, AnalysisSettings::default(), Limits::default(), None).unwrap();
    analyzer.submit_loudness(TrackId(1), file.clone());
    analyzer.submit(TrackId(1), file);
    assert!(analyzer.results().recv_timeout(WAIT).is_ok());
    assert!(analyzer.loudness_results().recv_timeout(WAIT).is_ok());
}

#[test]
fn a_loudness_job_pending_for_a_track_is_not_queued_twice() {
    let dir = tempfile::tempdir().unwrap();
    let file = wav(dir.path(), "a.wav", 1);
    let runs = Arc::new(AtomicUsize::new(0));
    let analyzer = Analyzer::with_fns(
        1,
        AnalysisSettings::default(),
        Limits::default(),
        None,
        real_analyze(),
        counting_loudness(&runs),
    )
    .unwrap();
    analyzer.submit_loudness(TrackId(1), file.clone());
    analyzer.submit_loudness(TrackId(1), file);
    assert!(analyzer.loudness_results().recv_timeout(WAIT).is_ok());
    assert!(
        analyzer
            .loudness_results()
            .recv_timeout(Duration::from_millis(300))
            .is_err()
    );
    assert_eq!(runs.load(Ordering::SeqCst), 1);
}

#[test]
fn a_cancelled_loudness_job_delivers_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let file = wav(dir.path(), "a.wav", 1);
    let started = Arc::new(AtomicBool::new(false));
    let s = started.clone();
    let loudness: LoudnessFn = Arc::new(move |_, cancelled| {
        s.store(true, Ordering::Release);
        while !cancelled() {
            std::thread::sleep(Duration::from_millis(1));
        }
        Err(AnalysisError::Cancelled)
    });
    let analyzer = Analyzer::with_fns(
        1,
        AnalysisSettings::default(),
        Limits::default(),
        None,
        real_analyze(),
        loudness,
    )
    .unwrap();
    analyzer.submit_loudness(TrackId(1), file);
    while !started.load(Ordering::Acquire) {
        std::thread::sleep(Duration::from_millis(1));
    }
    analyzer.cancel_loudness(TrackId(1));
    assert!(
        analyzer
            .loudness_results()
            .recv_timeout(Duration::from_millis(300))
            .is_err()
    );
}

#[test]
fn a_crash_in_the_loudness_job_is_reported_as_unreadable() {
    let dir = tempfile::tempdir().unwrap();
    let file = wav(dir.path(), "a.wav", 1);
    let loudness: LoudnessFn = Arc::new(|_, _| panic!("injected"));
    let analyzer = Analyzer::with_fns(
        1,
        AnalysisSettings::default(),
        Limits::default(),
        None,
        real_analyze(),
        loudness,
    )
    .unwrap();
    analyzer.submit_loudness(TrackId(1), file);
    let r = analyzer.loudness_results().recv_timeout(WAIT).unwrap();
    assert!(matches!(r.outcome, Err(AnalysisError::Unreadable(_))));
}

#[test]
fn the_loudness_share_leaves_one_worker_free() {
    for (threads, share) in [(0, 1), (1, 1), (2, 1), (3, 2), (4, 3)] {
        let a = Analyzer::spawn(
            threads,
            AnalysisSettings::default(),
            Limits::default(),
            None,
        )
        .unwrap();
        assert_eq!(a.loudness_share(), share, "{threads} threads");
    }
}
