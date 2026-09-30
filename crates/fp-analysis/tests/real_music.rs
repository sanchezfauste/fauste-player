#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Opt-in checks on the local real-music corpus (feedback spec §4.1):
//! `cargo test --release -p fp-analysis --test real_music -- --ignored`.

#[path = "support/corpus.rs"]
mod corpus;

use fp_analysis::analyze_file;
use fp_analysis::signal::EnvelopeBuilder;
use fp_decode::FileDecoder;
use fp_model::{AnalysisSettings, Limits};

#[test]
#[ignore = "needs the local real-music corpus"]
fn trimming_never_cuts_audio_and_overlaps_stay_short() {
    let s = AnalysisSettings::default();
    let mut failures = Vec::new();
    for path in corpus::files() {
        let name = path.display().to_string();
        let a = match analyze_file(&path, &s, &Limits::default()) {
            Ok(a) => a,
            Err(e) => {
                failures.push(format!("{name}: {e}"));
                continue;
            }
        };
        let t = &a.analysis;
        let (cue_in, cue_out) = (
            t.cue_in.unwrap_or(0.0),
            t.cue_out.unwrap_or(t.duration_secs),
        );
        // The stereo bucket peaks, decoded again independently.
        let mut decoder = FileDecoder::open(&path).unwrap();
        let mut b = EnvelopeBuilder::new(decoder.sample_rate(), s.rms_window_ms, s.peak_bucket_ms);
        let mut block = Vec::new();
        while let Ok(true) = decoder.next_block(&mut block) {
            b.push(&block);
            block.clear();
        }
        let env = b.finish();
        for (i, db) in env.peak_db.iter().enumerate() {
            let (start, end) = (i as f64 * env.bucket_secs, (i + 1) as f64 * env.bucket_secs);
            if *db >= s.trim_threshold_db as f32
                && (start < cue_in - 1e-6 || end.min(env.duration_secs) > cue_out + 1e-6)
            {
                failures.push(format!(
                    "{name}: {db:.1} dB at {start:.2}s is outside {cue_in:.2}–{cue_out:.2}"
                ));
                break;
            }
        }
        if let Some(segue) = t.segue_start
            && cue_out - segue > s.segue_max_secs + 1e-6
        {
            failures.push(format!("{name}: overlap {:.2}s", cue_out - segue));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
