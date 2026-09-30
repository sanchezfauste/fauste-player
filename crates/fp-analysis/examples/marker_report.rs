//! Prints the automatic markers of every file in the real-music corpus, to
//! review them by ear and tune the defaults (feedback spec §4.1).
//!
//! `cargo run --release -p fp-analysis --example marker_report -- [--set key=value]… [dir]`
//!
//! `--set` overrides a field of `AnalysisSettings` (for example
//! `--set segue_drop_db=12`). The folder defaults to `FAUSTE_TEST_MUSIC`,
//! else `test-music/`.

#[path = "../tests/support/corpus.rs"]
mod corpus;

use std::process::ExitCode;

use fp_analysis::analyze_file;
use fp_model::{AnalysisSettings, Limits};

fn main() -> ExitCode {
    let mut settings = serde_json::to_value(AnalysisSettings::default()).unwrap_or_default();
    let mut folder = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--set" {
            let Some((key, value)) = args.next().and_then(|kv| {
                kv.split_once('=')
                    .map(|(k, v)| (k.to_owned(), v.to_owned()))
            }) else {
                eprintln!("--set needs key=value");
                return ExitCode::FAILURE;
            };
            let parsed = serde_json::from_str(&value).unwrap_or(serde_json::Value::String(value));
            if let Some(map) = settings.as_object_mut() {
                map.insert(key, parsed);
            }
        } else {
            folder = Some(std::path::PathBuf::from(arg));
        }
    }
    let settings: AnalysisSettings = match serde_json::from_value(settings) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("invalid --set: {e}");
            return ExitCode::FAILURE;
        }
    };
    let files = folder.map_or_else(corpus::files, |f| corpus::files_in(&f));
    println!(
        "{:<48} {:>8} {:>7} {:>8} {:>8} {:>7} {:>8}",
        "file", "length", "cue in", "cue out", "segue", "overlap", "outro"
    );
    let mut overlaps = Vec::new();
    for path in &files {
        let name: String = path
            .file_name()
            .map(|n| n.to_string_lossy().chars().take(48).collect())
            .unwrap_or_default();
        match analyze_file(path, &settings, &Limits::default()) {
            Ok(a) => {
                let t = &a.analysis;
                let cue_in = t.cue_in.unwrap_or(0.0);
                let cue_out = t.cue_out.unwrap_or(t.duration_secs);
                let overlap = t.segue_start.map(|s| cue_out - s);
                if let Some(o) = overlap {
                    overlaps.push(o);
                }
                let opt = |v: Option<f64>| v.map_or("—".to_owned(), |v| format!("{v:.2}"));
                println!(
                    "{name:<48} {:>8.2} {cue_in:>7.2} {cue_out:>8.2} {:>8} {:>7} {:>8}",
                    t.duration_secs,
                    opt(t.segue_start),
                    opt(overlap),
                    opt(t.outro_start)
                );
            }
            Err(e) => println!("{name:<48} {e}"),
        }
    }
    if !overlaps.is_empty() {
        overlaps.sort_by(f64::total_cmp);
        let at = |q: f64| {
            overlaps
                .get(((overlaps.len() - 1) as f64 * q).round() as usize)
                .copied()
                .unwrap_or_default()
        };
        println!(
            "\noverlap over {} files: p10 {:.2}s, median {:.2}s, p90 {:.2}s, max {:.2}s",
            overlaps.len(),
            at(0.1),
            at(0.5),
            at(0.9),
            at(1.0)
        );
    }
    ExitCode::SUCCESS
}
