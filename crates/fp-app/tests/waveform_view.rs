//! The waveform's reduction of analysis buckets to pixel columns.

use fp_analysis::WavePeak;
use fp_app::ui::widgets::{WaveColumn, wave_columns};

const FULL: f32 = i16::MAX as f32;

fn bucket(peak: f32, rms: f32) -> WavePeak {
    WavePeak {
        min: -((peak * FULL) as i16),
        max: (peak * FULL) as i16,
        rms: (rms * FULL) as i16,
    }
}

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-3
}

#[test]
fn there_is_one_column_per_pixel() {
    let peaks = vec![bucket(0.5, 0.2); 100];
    assert_eq!(wave_columns(&peaks, 0.01, 1.0, 37).len(), 37);
}

#[test]
fn a_column_shows_its_largest_peak_and_the_rms_of_its_mean_square() {
    // Two buckets per column: peaks 1.0 and 0.5, RMS 0.3 and 0.4.
    let peaks = [bucket(1.0, 0.3), bucket(0.5, 0.4)];
    let c = wave_columns(&peaks, 0.01, 0.02, 1);
    let [WaveColumn { peak, rms }] = c[..] else {
        panic!("{c:?}")
    };
    assert!(close(peak, 1.0), "{peak}");
    let expected = ((0.09f32 + 0.16) / 2.0).sqrt();
    assert!(close(rms, expected), "{rms} vs {expected}");
}

#[test]
fn a_loud_master_keeps_its_body_below_the_peaks() {
    let peaks = vec![bucket(0.98, 0.35); 50];
    for c in wave_columns(&peaks, 0.01, 0.5, 20) {
        assert!(close(c.peak, 0.98) && close(c.rms, 0.35), "{c:?}");
    }
}

#[test]
fn more_pixels_than_buckets_leave_no_gaps() {
    let peaks = [bucket(0.8, 0.4), bucket(0.6, 0.3)];
    let c = wave_columns(&peaks, 0.01, 0.02, 10);
    assert!(c.iter().all(|c| c.peak > 0.5 && c.rms > 0.2), "{c:?}");
}

#[test]
fn columns_past_the_audio_are_empty() {
    // The span is longer than the audio (the track's total exceeds the
    // analysed length): the rest of the widget is silent.
    let peaks = vec![bucket(0.8, 0.4); 10];
    let c = wave_columns(&peaks, 0.01, 0.2, 20);
    assert!(close(c[5].peak, 0.8));
    assert_eq!(c[15], WaveColumn::default());
}

#[test]
fn full_scale_saturates_without_overflow() {
    let peaks = [WavePeak {
        min: i16::MIN,
        max: i16::MAX,
        rms: i16::MAX,
    }];
    let c = wave_columns(&peaks, 0.01, 0.01, 1);
    assert!(c[0].peak <= 1.0 && close(c[0].rms, 1.0), "{c:?}");
}

#[test]
fn nothing_to_show_gives_empty_columns() {
    assert!(
        wave_columns(&[], 0.01, 1.0, 4)
            .iter()
            .all(|c| *c == WaveColumn::default())
    );
    assert!(
        wave_columns(&[bucket(1.0, 1.0)], 0.0, 0.0, 4)
            .iter()
            .all(|c| *c == WaveColumn::default())
    );
}
