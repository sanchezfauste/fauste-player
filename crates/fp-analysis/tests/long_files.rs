#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Opt-in checks on programme-length recordings (plan 4, item 15), never in
//! CI: `FAUSTE_LONG_MUSIC=<dir> cargo test --release -p fp-analysis --test
//! long_files -- --ignored --nocapture`. Each file is analysed, cached and
//! sought near its end.

use std::time::Instant;

use fp_analysis::analyze_file;
use fp_analysis::cache::AnalysisCache;
use fp_decode::FileDecoder;
use fp_model::{AnalysisSettings, Limits};

#[test]
#[ignore = "needs FAUSTE_LONG_MUSIC"]
fn long_files_analyse_cache_and_seek() {
    let Some(dir) = std::env::var_os("FAUSTE_LONG_MUSIC") else {
        println!("FAUSTE_LONG_MUSIC is not set: nothing to check");
        return;
    };
    let s = AnalysisSettings::default();
    let limits = Limits::default();
    let cache_dir = std::env::temp_dir().join(format!("fp-long-cache-{}", std::process::id()));
    let cache = AnalysisCache::new(cache_dir.clone(), &limits);
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file())
        .collect();
    paths.sort();
    let mut failures = Vec::new();
    for path in paths {
        let name = path.display().to_string();
        let t0 = Instant::now();
        let a = match analyze_file(&path, &s, &limits) {
            Ok(a) => a,
            Err(e) => {
                failures.push(format!("{name}: {e}"));
                continue;
            }
        };
        let took = t0.elapsed();
        let t = &a.analysis;
        let key = cache.key(&path, &s).unwrap();
        let stored = cache.store_key(&key, &a);
        let cached = cache.load_key(&key).is_some();
        let mut decoder = FileDecoder::open(&path).unwrap();
        let target = t.duration_secs - 60.0;
        let t1 = Instant::now();
        let sought = decoder.seek(target);
        let seek_took = t1.elapsed();
        println!(
            "{name}: {:.1} s long, analysed in {took:.1?}, {} peaks, cache {stored:?} read back {cached}, cue-out {:?}, seek to {target:.0} s {sought:?} in {seek_took:.1?}",
            t.duration_secs,
            a.peaks.len(),
            t.cue_out
        );
        if stored.is_err() || !cached {
            failures.push(format!("{name}: not cached"));
        }
        if t.cue_out.is_none_or(|c| c < t.duration_secs - 5.0) {
            failures.push(format!("{name}: cue-out {:?}", t.cue_out));
        }
        if sought.is_err() || seek_took.as_secs_f64() > 1.0 {
            failures.push(format!("{name}: seek {sought:?} took {seek_took:?}"));
        }
    }
    let _ = std::fs::remove_dir_all(cache_dir);
    assert!(failures.is_empty(), "{failures:#?}");
}

/// Decoding alone, to tell a slow decoder from a slow analysis.
#[test]
#[ignore = "needs FAUSTE_LONG_MUSIC"]
fn long_files_decode_speed() {
    let Some(dir) = std::env::var_os("FAUSTE_LONG_MUSIC") else {
        return;
    };
    for e in std::fs::read_dir(dir).unwrap() {
        let path = e.unwrap().path();
        let mut decoder = FileDecoder::open(&path).unwrap();
        let rate = f64::from(decoder.sample_rate());
        let (t0, mut frames) = (Instant::now(), 0usize);
        let mut block = Vec::new();
        while let Ok(true) = decoder.next_block(&mut block) {
            frames += block.len() / 2;
            block.clear();
        }
        let secs = frames as f64 / rate;
        let took = t0.elapsed().as_secs_f64();
        println!(
            "{}: {secs:.0} s decoded in {took:.1} s ({:.0}× real time)",
            path.display(),
            secs / took
        );
    }
}
