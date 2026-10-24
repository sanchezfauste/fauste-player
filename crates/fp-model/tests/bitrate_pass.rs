#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The bit-rate-only pass: tracks analysed before the format carried a bit
//! rate (or whose cached analysis has none) get it read from the file once.

mod common;

use common::{entries, fixture};
use fp_model::{AppState, AudioFormat, Command, FileState, TrackAnalysis, TrackId, apply};

fn track_of(state: &AppState, n: usize) -> TrackId {
    state.playlists.entry(entries(state)[n]).unwrap().track
}

fn pcm(bitrate: Option<u32>) -> AudioFormat {
    AudioFormat {
        sample_rate: 44_100,
        bits: Some(16),
        channels: 2,
        dsd_rate: None,
        bitrate,
    }
}

/// A state whose track 0 is analysed with `format`.
fn analysed(format: Option<AudioFormat>) -> (AppState, TrackId) {
    let mut s = fixture(2);
    let t = track_of(&s, 0);
    apply(
        &mut s,
        Command::ApplyAnalysis {
            track: t,
            analysis: Box::new(TrackAnalysis {
                duration_secs: 180.0,
                format,
                ..TrackAnalysis::default()
            }),
        },
    )
    .unwrap();
    (s, t)
}

#[test]
fn an_analysis_without_a_bit_rate_asks_for_the_pass() {
    let (s, t) = analysed(Some(pcm(None)));
    assert!(s.library.get(t).unwrap().needs_bitrate_read());
}

#[test]
fn an_analysis_with_a_bit_rate_needs_no_pass() {
    let (s, t) = analysed(Some(pcm(Some(958_000))));
    assert!(!s.library.get(t).unwrap().needs_bitrate_read());
}

#[test]
fn a_track_not_analysed_dsd_unknown_format_or_unplayable_needs_no_pass() {
    let s = fixture(1);
    let t = track_of(&s, 0);
    assert!(
        !s.library.get(t).unwrap().needs_bitrate_read(),
        "not analysed"
    );
    let dsd = AudioFormat {
        dsd_rate: Some(2_822_400),
        ..pcm(None)
    };
    let (s, t) = analysed(Some(dsd));
    assert!(
        !s.library.get(t).unwrap().needs_bitrate_read(),
        "DSD has none"
    );
    let (s, t) = analysed(None);
    assert!(!s.library.get(t).unwrap().needs_bitrate_read(), "no format");
    let (mut s, t) = analysed(Some(pcm(None)));
    s.library.get_mut(t).unwrap().file_state = FileState::Missing;
    assert!(!s.library.get(t).unwrap().needs_bitrate_read(), "missing");
}

#[test]
fn apply_bitrate_stores_it_and_ends_the_pass() {
    let (mut s, t) = analysed(Some(pcm(None)));
    apply(
        &mut s,
        Command::ApplyBitrate {
            track: t,
            bitrate: Some(958_000),
        },
    )
    .unwrap();
    let track = s.library.get(t).unwrap();
    assert_eq!(track.format.unwrap().bitrate, Some(958_000));
    assert!(!track.needs_bitrate_read());
}

#[test]
fn a_file_without_a_readable_bit_rate_is_not_asked_again() {
    let (mut s, t) = analysed(Some(pcm(None)));
    apply(
        &mut s,
        Command::ApplyBitrate {
            track: t,
            bitrate: None,
        },
    )
    .unwrap();
    let track = s.library.get(t).unwrap();
    assert_eq!(track.format.unwrap().bitrate, None);
    assert!(!track.needs_bitrate_read());
}

#[test]
fn a_zero_bit_rate_counts_as_none() {
    let (mut s, t) = analysed(Some(pcm(None)));
    apply(
        &mut s,
        Command::ApplyBitrate {
            track: t,
            bitrate: Some(0),
        },
    )
    .unwrap();
    assert_eq!(s.library.get(t).unwrap().format.unwrap().bitrate, None);
}

#[test]
fn a_new_analysis_without_a_bit_rate_asks_again() {
    let (mut s, t) = analysed(Some(pcm(None)));
    apply(
        &mut s,
        Command::ApplyBitrate {
            track: t,
            bitrate: None,
        },
    )
    .unwrap();
    apply(
        &mut s,
        Command::ApplyAnalysis {
            track: t,
            analysis: Box::new(TrackAnalysis {
                duration_secs: 180.0,
                format: Some(pcm(None)),
                ..TrackAnalysis::default()
            }),
        },
    )
    .unwrap();
    assert!(s.library.get(t).unwrap().needs_bitrate_read());
}

#[test]
fn apply_bitrate_for_a_gone_track_is_ignored() {
    let (mut s, _) = analysed(Some(pcm(None)));
    apply(
        &mut s,
        Command::ApplyBitrate {
            track: TrackId(9999),
            bitrate: Some(1),
        },
    )
    .unwrap();
}

#[test]
fn an_older_library_without_the_flag_reads_the_bit_rate_once() {
    let (s, t) = analysed(Some(pcm(None)));
    let mut json = serde_json::to_value(s.library.get(t).unwrap()).unwrap();
    json.as_object_mut().unwrap().remove("bitrate_read");
    let back: fp_model::Track = serde_json::from_value(json).unwrap();
    assert!(back.needs_bitrate_read());
}
