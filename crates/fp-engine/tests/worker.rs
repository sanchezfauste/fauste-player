#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

mod support;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use fp_engine::source::{SourceConsumer, source_pair};
use fp_engine::worker::{
    LoadOptions, PlayerWorker, SampleSource, SourceKey, SourceOpener, WorkerFailure,
};
use support::{Exploding, counting_opener};

fn wait_until(what: &str, mut ok: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !ok() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn drain(c: &mut SourceConsumer, into: &mut Vec<f32>) {
    let mut buf = [0.0f32; 512];
    loop {
        let n = c.pop_frames(&mut buf);
        if n == 0 {
            return;
        }
        into.extend_from_slice(&buf[..n * 2]);
    }
}

fn worker(opener: SourceOpener) -> (PlayerWorker, crossbeam_channel::Receiver<WorkerFailure>) {
    let (tx, rx) = crossbeam_channel::unbounded();
    (
        PlayerWorker::spawn("test-worker", opener, 48_000, 100, tx).unwrap(),
        rx,
    )
}

#[test]
fn fills_the_ring_marks_ready_and_eof_and_keeps_order() {
    let (w, _failures) = worker(counting_opener(1_000));
    let (p, mut c) = source_pair(4_000);
    let shared = p.shared.clone();
    w.load(SourceKey(1), PathBuf::from("x"), 0.0, p);
    wait_until("eof", || shared.is_eof());
    assert!(shared.is_ready());
    let mut got = Vec::new();
    drain(&mut c, &mut got);
    let lefts: Vec<f32> = got.chunks(2).map(|f| f[0]).collect();
    assert_eq!(lefts, (0..1_000).map(|i| i as f32).collect::<Vec<_>>());
}

#[test]
fn refills_a_small_ring_as_it_is_consumed() {
    let (w, _failures) = worker(counting_opener(10_000));
    let (p, mut c) = source_pair(256);
    let shared = p.shared.clone();
    w.load(SourceKey(1), PathBuf::from("x"), 0.0, p);
    let mut got = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !(shared.is_eof() && got.len() >= 20_000) {
        assert!(Instant::now() < deadline);
        drain(&mut c, &mut got);
        std::thread::sleep(Duration::from_micros(200));
    }
    drain(&mut c, &mut got);
    assert_eq!(got.len(), 20_000);
    assert!(got.chunks(2).enumerate().all(|(i, f)| f[0] == i as f32));
}

#[test]
fn starts_at_the_requested_position() {
    let (w, _failures) = worker(counting_opener(96_000));
    let (p, mut c) = source_pair(128);
    w.load(SourceKey(1), PathBuf::from("x"), 1.0, p);
    let mut first = [0.0f32; 2];
    wait_until("first frame", || c.pop_frames(&mut first) == 1);
    assert_eq!(first[0], 48_000.0);
}

#[test]
fn an_open_failure_is_reported_and_marks_the_source_failed() {
    let opener: SourceOpener = Arc::new(|_, _, _| Err("cannot open".to_owned()));
    let (w, failures) = worker(opener);
    let (p, _c) = source_pair(64);
    let shared = p.shared.clone();
    w.load(SourceKey(7), PathBuf::from("x"), 0.0, p);
    let failure = failures.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(
        failure,
        WorkerFailure {
            key: SourceKey(7),
            error: "cannot open".to_owned()
        }
    );
    assert!(shared.is_failed());
}

#[test]
fn a_panicking_decoder_is_contained_and_the_worker_keeps_serving() {
    let opener: SourceOpener = Arc::new(|path, _, _| {
        if path.to_string_lossy() == "bomb" {
            Ok(Box::new(Exploding) as Box<dyn SampleSource>)
        } else {
            counting_opener(10)(path, 0.0, 48_000)
        }
    });
    let (w, failures) = worker(opener);
    let (p1, _c1) = source_pair(64);
    w.load(SourceKey(1), PathBuf::from("bomb"), 0.0, p1);
    let failure = failures.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(failure.key, SourceKey(1));
    assert!(failure.error.contains("panicked"));
    let (p2, _c2) = source_pair(64);
    let shared = p2.shared.clone();
    w.load(SourceKey(2), PathBuf::from("fine"), 0.0, p2);
    wait_until("second source eof", || shared.is_eof());
}

#[test]
fn dropping_a_source_releases_its_producer() {
    let (w, _failures) = worker(counting_opener(1_000_000));
    let (p, c) = source_pair(64);
    w.load(SourceKey(3), PathBuf::from("x"), 0.0, p);
    wait_until("ready", || c.shared.is_ready());
    w.drop_source(SourceKey(3));
    wait_until("producer dropped", || c.is_abandoned());
}

fn frames_secs(frames: u64) -> f64 {
    frames as f64 / 48_000.0
}

fn lefts(c: &mut SourceConsumer, count: usize) -> Vec<f32> {
    let mut got = Vec::new();
    wait_until("enough frames", || {
        drain(c, &mut got);
        got.len() >= count * 2
    });
    got.chunks(2).take(count).map(|f| f[0]).collect()
}

#[test]
fn a_source_with_until_ends_exactly_there() {
    let (w, _f) = worker(counting_opener(1_000));
    let (p, mut c) = source_pair(4_000);
    let shared = p.shared.clone();
    let options = LoadOptions {
        until_secs: Some(frames_secs(130)),
        looped: false,
        rate: None,
        fade_out_frames: 0,
    };
    w.load_with(
        SourceKey(1),
        PathBuf::from("x"),
        frames_secs(30),
        p,
        options,
    );
    wait_until("eof", || shared.is_eof());
    let mut got = Vec::new();
    drain(&mut c, &mut got);
    let l: Vec<f32> = got.chunks(2).map(|f| f[0]).collect();
    assert_eq!(l, (30..130).map(|i| i as f32).collect::<Vec<_>>());
}

#[test]
fn a_looped_source_repeats_without_gaps() {
    let (w, _f) = worker(counting_opener(1_000));
    let (p, mut c) = source_pair(4_000);
    let shared = p.shared.clone();
    let options = LoadOptions {
        until_secs: Some(frames_secs(70)),
        looped: true,
        rate: None,
        fade_out_frames: 0,
    };
    w.load_with(
        SourceKey(1),
        PathBuf::from("x"),
        frames_secs(10),
        p,
        options,
    );
    let l = lefts(&mut c, 600);
    let expected: Vec<f32> = (10..70).cycle().take(600).map(|i| i as f32).collect();
    assert_eq!(l, expected);
    assert!(!shared.is_eof(), "a loop never ends by itself");
}

#[test]
fn a_looped_source_without_until_loops_at_the_end_of_the_file() {
    let (w, _f) = worker(counting_opener(50));
    let (p, mut c) = source_pair(4_000);
    let options = LoadOptions {
        until_secs: None,
        looped: true,
        rate: None,
        fade_out_frames: 0,
    };
    w.load_with(SourceKey(1), PathBuf::from("x"), 0.0, p, options);
    let l = lefts(&mut c, 200);
    let expected: Vec<f32> = (0..50).cycle().take(200).map(|i| i as f32).collect();
    assert_eq!(l, expected);
}

#[test]
fn a_zero_length_loop_ends() {
    let (w, _f) = worker(counting_opener(1_000));
    let (p, mut c) = source_pair(4_000);
    let shared = p.shared.clone();
    let options = LoadOptions {
        until_secs: Some(frames_secs(10)),
        looped: true,
        rate: None,
        fade_out_frames: 0,
    };
    w.load_with(
        SourceKey(1),
        PathBuf::from("x"),
        frames_secs(10),
        p,
        options,
    );
    wait_until("eof", || shared.is_eof());
    let mut got = Vec::new();
    drain(&mut c, &mut got);
    assert!(got.is_empty());
}

/// Gives `frames` frames, then nothing more (a stalled read), counting calls.
struct Stalling {
    frames: usize,
    calls: Arc<std::sync::atomic::AtomicUsize>,
}

impl SampleSource for Stalling {
    fn next_block(&mut self, out: &mut Vec<f32>) -> Result<bool, String> {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        out.extend(std::iter::repeat_n(
            0.5,
            std::mem::take(&mut self.frames) * 2,
        ));
        std::thread::sleep(Duration::from_millis(1));
        Ok(true)
    }
}

fn stalling_after(frames: usize, calls: Arc<std::sync::atomic::AtomicUsize>) -> SourceOpener {
    Arc::new(move |_path, _from, _rate| {
        Ok(Box::new(Stalling {
            frames,
            calls: Arc::clone(&calls),
        }) as Box<dyn SampleSource>)
    })
}

#[test]
fn the_ready_threshold_is_the_same_time_at_any_source_rate() {
    // 100 frames at the worker's 48 kHz; 150 frames is enough there, but
    // only 1.56 ms at 96 kHz, short of the same 2.08 ms.
    for (rate, ready) in [(48_000, true), (96_000, false)] {
        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let (w, _failures) = worker(stalling_after(150, Arc::clone(&calls)));
        let (p, _c) = source_pair(4_000);
        let shared = p.shared.clone();
        w.load_at(SourceKey(1), PathBuf::from("x"), 0.0, p, rate);
        wait_until("the stall", || {
            calls.load(std::sync::atomic::Ordering::Acquire) >= 3
        });
        assert_eq!(shared.is_ready(), ready, "at {rate} Hz");
    }
}

#[test]
fn a_bounded_pass_fades_its_last_frames_to_zero() {
    let (w, _f) = worker(constant_opener(1_000));
    let (p, mut c) = source_pair(4_000);
    let shared = p.shared.clone();
    let options = LoadOptions {
        until_secs: Some(frames_secs(100)),
        looped: false,
        rate: None,
        fade_out_frames: 10,
    };
    w.load_with(SourceKey(1), PathBuf::from("x"), 0.0, p, options);
    wait_until("eof", || shared.is_eof());
    let mut got = Vec::new();
    drain(&mut c, &mut got);
    let l: Vec<f32> = got.chunks(2).map(|f| f[0]).collect();
    assert_eq!(l.len(), 100);
    assert!(l[..90].iter().all(|v| *v == 1.0));
    assert!(l[90..].windows(2).all(|w| w[1] < w[0]));
    assert!((l[99] - 0.1).abs() < 1e-6, "last {}", l[99]);
}

#[test]
fn a_looped_pass_is_not_faded() {
    let (w, _f) = worker(constant_opener(1_000));
    let (p, mut c) = source_pair(4_000);
    let options = LoadOptions {
        until_secs: Some(frames_secs(100)),
        looped: true,
        rate: None,
        fade_out_frames: 10,
    };
    w.load_with(SourceKey(1), PathBuf::from("x"), 0.0, p, options);
    let l = lefts(&mut c, 300);
    assert!(l.iter().all(|v| *v == 1.0));
}

fn constant_opener(frames: usize) -> SourceOpener {
    struct Constant(usize, bool);
    impl SampleSource for Constant {
        fn next_block(&mut self, out: &mut Vec<f32>) -> Result<bool, String> {
            if self.1 {
                return Ok(false);
            }
            self.1 = true;
            out.extend(std::iter::repeat_n(1.0f32, self.0 * 2));
            Ok(true)
        }
    }
    Arc::new(move |_, _, _| Ok(Box::new(Constant(frames, false))))
}
