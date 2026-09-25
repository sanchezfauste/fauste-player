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
    let slow: AnalyzeFn = Arc::new(|path, settings, limits| {
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
    let counting: AnalyzeFn = Arc::new(move |path, settings, limits| {
        counter.fetch_add(1, Ordering::SeqCst);
        analyze_file(path, settings, limits)
    });
    let dir = tempfile::tempdir().unwrap();
    let path = wav(dir.path(), "a.wav");
    let cache = AnalysisCache::new(
        dir.path().join("cache"),
        Limits::default().max_state_file_bytes,
    );
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
    let exploding: AnalyzeFn = Arc::new(|_, _, _| panic!("boom"));
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
    let slow: AnalyzeFn = Arc::new(move |path, settings, limits| {
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
