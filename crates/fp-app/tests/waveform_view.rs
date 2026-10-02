//! The waveform's reduction of analysis buckets to pixel columns.

use std::sync::Arc;

use fp_analysis::WavePeak;
use fp_app::services::TrackMedia;
use fp_app::ui::theme;
use fp_app::ui::view::MarkerFractions;
use fp_app::ui::widgets::{
    CueEdgeLook, WaveColumn, WaveInput, WaveMemo, cue_edge_look, memo_columns, wave_columns,
    waveform,
};

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

#[test]
fn the_last_bucket_is_drawn_when_the_span_is_the_audio() {
    // 29 × 0.01 s / 0.01 s is 28.999… in floating point: the last bucket
    // (the only loud one) must still land in the last column.
    let mut peaks = vec![bucket(0.1, 0.05); 29];
    peaks[28] = bucket(1.0, 0.5);
    let c = wave_columns(&peaks, 0.01, 29.0 * 0.01, 3);
    assert!(close(c[2].peak, 1.0), "{c:?}");
}

#[test]
fn silence_draws_nothing() {
    let peaks = vec![WavePeak::default(); 50];
    let c = wave_columns(&peaks, 0.01, 0.5, 10);
    assert!(c.iter().all(|c| *c == WaveColumn::default()), "{c:?}");
}

#[test]
fn a_nonsense_bucket_length_or_span_draws_nothing() {
    let peaks = vec![bucket(0.5, 0.2); 50];
    for (bucket_secs, span) in [
        (f64::NAN, 0.5),
        (f64::INFINITY, 0.5),
        (0.01, f64::NAN),
        (0.01, f64::INFINITY),
    ] {
        let c = wave_columns(&peaks, bucket_secs, span, 10);
        assert_eq!(c.len(), 10);
        assert!(
            c.iter().all(|c| *c == WaveColumn::default()),
            "{bucket_secs} {span}: {c:?}"
        );
    }
}

#[test]
fn a_huge_span_does_not_overflow() {
    // A corrupt duration of 1e30 s: the audio is a sliver at the start.
    let peaks = vec![bucket(0.5, 0.2); 50];
    let c = wave_columns(&peaks, 0.01, 1e30, 100);
    assert_eq!(c.len(), 100);
    assert!(c[1..].iter().all(|c| *c == WaveColumn::default()));
}

fn media(peak: f32) -> Arc<TrackMedia> {
    Arc::new(TrackMedia {
        peaks: vec![bucket(peak, peak / 2.0); 100],
        peak_bucket_secs: 0.01,
        cover_png: None,
    })
}

#[test]
fn the_columns_are_reduced_once_per_track_and_width() {
    let mut memo: Option<WaveMemo> = None;
    let track = media(0.5);
    let first = memo_columns(&mut memo, &track, 1.0, 40);
    let again = memo_columns(&mut memo, &track, 1.0, 40);
    assert!(Arc::ptr_eq(&first, &again), "the same frame again");
    assert_eq!(&*first, &wave_columns(&track.peaks, 0.01, 1.0, 40)[..]);
    let wider = memo_columns(&mut memo, &track, 1.0, 80);
    assert_eq!(wider.len(), 80, "a new width");
    let other = memo_columns(&mut memo, &media(0.9), 1.0, 80);
    assert!(close(other[0].peak, 0.9), "a new track: {:?}", other[0]);
    let longer = memo_columns(&mut memo, &media(0.9), 2.0, 80);
    assert!(longer[79] == WaveColumn::default(), "a new span");
}

#[test]
fn a_player_that_unloads_lets_go_of_its_waveform() {
    let track = media(0.5);
    let shown = std::rc::Rc::new(std::cell::Cell::new(true));
    let (held, flag) = (Arc::clone(&track), std::rc::Rc::clone(&shown));
    let mut harness = egui_kittest::Harness::new_ui(move |ui| {
        let input = WaveInput {
            id: egui::Id::new(("wave", 1)),
            media: flag.get().then_some(&held),
            total: Some(1.0),
            markers: MarkerFractions::default(),
            colors: theme::wave_colors("sand"),
            mix_active: false,
            mix_label: "MIX",
            accessible_label: "Waveform",
            view: None,
            shield: None,
            seekable: true,
        };
        waveform(ui, 40.0, &input);
    });
    harness.run();
    assert_eq!(
        Arc::strong_count(&track),
        3,
        "the memo holds it while shown"
    );
    shown.set(false);
    harness.run();
    assert_eq!(
        Arc::strong_count(&track),
        2,
        "only the test and the closure"
    );
}

mod view {
    use egui::{Rect, pos2, vec2};
    use fp_app::ui::wave_view::{WaveView, min_span};

    use super::{bucket, wave_columns};
    use fp_app::ui::widgets::wave_columns_in;

    fn rect() -> Rect {
        Rect::from_min_size(pos2(100.0, 10.0), vec2(400.0, 60.0))
    }

    fn near(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn the_full_view_maps_the_file_to_the_width() {
        let v = WaveView::full(200.0);
        assert!(v.is_full(200.0));
        assert_eq!(v.x_of(0.0, rect()), 100.0);
        assert_eq!(v.x_of(200.0, rect()), 500.0);
        assert!(near(v.secs_at(300.0, rect()), 100.0));
        assert!(near(v.secs_at(-50.0, rect()), 0.0), "clamped to the view");
    }

    #[test]
    fn zooming_keeps_the_time_under_the_pointer() {
        let v = WaveView::full(200.0);
        let x = 200.0; // 50 s
        let z = v.zoom_at(x, rect(), 0.5, 200.0, 0.1);
        assert!(near(z.span_secs, 100.0));
        assert!(near(z.secs_at(x, rect()), 50.0), "{z:?}");
        assert!(!z.is_full(200.0));
    }

    #[test]
    fn the_view_stays_inside_the_file() {
        let v = WaveView::full(200.0);
        let z = v.zoom_at(110.0, rect(), 0.25, 200.0, 0.1);
        assert!(z.start_secs >= 0.0);
        let z = v.zoom_at(500.0, rect(), 0.25, 200.0, 0.1);
        assert!(near(z.start_secs + z.span_secs, 200.0), "{z:?}");
        let panned = z.pan(10_000.0, rect(), 200.0);
        assert!(near(panned.start_secs, 0.0), "{panned:?}");
        let panned = z.pan(-10_000.0, rect(), 200.0);
        assert!(near(panned.start_secs + panned.span_secs, 200.0));
    }

    #[test]
    fn the_deepest_zoom_is_one_bucket_per_pixel() {
        let min = min_span(0.01, 400.0);
        assert!(near(min, 4.0));
        let mut v = WaveView::full(200.0);
        for _ in 0..50 {
            v = v.zoom_at(300.0, rect(), 0.5, 200.0, min);
        }
        assert!(near(v.span_secs, min));
    }

    #[test]
    fn zooming_out_past_the_file_gives_the_full_view() {
        let z = WaveView::full(200.0).zoom_at(300.0, rect(), 0.5, 200.0, 0.1);
        let out = z.zoom_at(300.0, rect(), 4.0, 200.0, 0.1);
        assert_eq!(out, WaveView::full(200.0));
    }

    #[test]
    fn following_brings_the_playhead_back_into_view() {
        let z = WaveView {
            start_secs: 20.0,
            span_secs: 40.0,
        };
        assert_eq!(z.follow(30.0, 200.0), z, "visible: unchanged");
        let f = z.follow(150.0, 200.0);
        assert!(f.start_secs <= 150.0 && 150.0 <= f.start_secs + f.span_secs);
        assert!(near(f.span_secs, 40.0));
        let end = z.follow(199.0, 200.0);
        assert!(near(end.start_secs + end.span_secs, 200.0));
    }

    #[test]
    fn a_zero_or_broken_length_is_harmless() {
        for total in [0.0, f64::NAN, -3.0] {
            let v = WaveView::full(total);
            let x = v.x_of(1.0, rect());
            assert!(x.is_finite());
            assert!(v.secs_at(250.0, rect()).is_finite());
            let z = v.zoom_at(250.0, rect(), 0.5, total, 0.1);
            assert!(z.start_secs.is_finite() && z.span_secs.is_finite());
        }
    }

    #[test]
    fn trimmed_regions_cover_the_head_and_tail() {
        let v = WaveView::full(200.0);
        let [head, tail] = v.trimmed(rect(), Some(20.0), Some(180.0), 200.0);
        let head = head.unwrap();
        let tail = tail.unwrap();
        assert_eq!((head.left(), head.right()), (100.0, 140.0));
        assert_eq!((tail.left(), tail.right()), (460.0, 500.0));
        let [none_head, none_tail] = v.trimmed(rect(), Some(0.0), Some(200.0), 200.0);
        assert!(none_head.is_none() && none_tail.is_none());
        // Zoomed past the head: only the tail part inside the view.
        let z = WaveView {
            start_secs: 100.0,
            span_secs: 100.0,
        };
        let [head, tail] = z.trimmed(rect(), Some(20.0), Some(180.0), 200.0);
        assert!(head.is_none());
        assert_eq!(tail.unwrap().left(), 420.0);
    }

    #[test]
    fn a_sub_range_is_the_matching_slice_of_the_full_reduction() {
        let peaks: Vec<_> = (0..1000)
            .map(|i| bucket((i % 100) as f32 / 100.0, 0.1))
            .collect();
        let full = wave_columns(&peaks, 0.01, 10.0, 100);
        let part = wave_columns_in(&peaks, 0.01, 2.0, 3.0, 30);
        assert_eq!(&full[20..50], &part[..]);
    }
}

#[test]
fn ignored_cue_marks_are_dimmed_and_do_not_shade_the_file() {
    assert_eq!(
        cue_edge_look(false),
        CueEdgeLook {
            shade_trimmed: true,
            line_alpha: 1.0
        }
    );
    let ignored = cue_edge_look(true);
    assert!(!ignored.shade_trimmed, "the head and tail will play");
    assert_eq!(ignored.line_alpha, theme::CUE_EDGE_IGNORED_ALPHA);
    assert!(ignored.line_alpha > 0.0 && ignored.line_alpha < 1.0);
}

#[test]
fn a_waveform_with_ignored_marks_draws_without_panicking() {
    let track = media(0.5);
    let mut harness = egui_kittest::Harness::new_ui(move |ui| {
        let input = WaveInput {
            id: egui::Id::new(("wave", 2)),
            media: Some(&track),
            total: Some(1.0),
            markers: MarkerFractions {
                cue_in: Some(0.1),
                cue_out: Some(0.9),
                ignored: true,
                ..MarkerFractions::default()
            },
            colors: theme::wave_colors("sand"),
            mix_active: false,
            mix_label: "MIX",
            accessible_label: "Waveform",
            view: None,
            shield: None,
            seekable: true,
        };
        waveform(ui, 40.0, &input);
    });
    harness.run();
}
