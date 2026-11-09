#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The adaptive frame rate (adaptive frame rate spec AF1–AF8).

use std::time::Duration;

use fp_app::ui::view::{AdaptiveRate, RateNote, frame_cost, is_software_renderer, target_fps};
use fp_model::UiConfig;

const FRAME: f64 = 1.0 / 30.0;

/// Observes `n` moving frames of `cost` seconds, one every `FRAME` from
/// `*now`; returns the notes they produced.
fn run(
    rate: &mut AdaptiveRate,
    ui: &UiConfig,
    cost: f32,
    n: usize,
    now: &mut f64,
) -> Vec<RateNote> {
    let mut notes = Vec::new();
    for _ in 0..n {
        *now += FRAME;
        notes.extend(rate.observe(Some(cost), true, *now, ui));
    }
    notes
}

/// AF2.
#[test]
fn software_renderers_are_recognised() {
    for name in [
        "llvmpipe (LLVM 19.1.7, 128 bits)",
        "softpipe",
        "Google SwiftShader",
        "Apple Software Renderer",
        "GDI Generic",
        "Software Rasterizer",
        "Microsoft Basic Render Driver",
        "SWRAST",
    ] {
        assert!(is_software_renderer(name), "{name}");
    }
    for name in [
        "Mesa Intel(R) HD Graphics 620 (KBL GT2)",
        "NVIDIA GeForce GTX 1060/PCIe/SSE2",
        "AMD Radeon RX 6600 (radeonsi, navi23, LLVM 19.1.7, DRM 3.61)",
        "Apple M1",
        "",
    ] {
        assert!(!is_software_renderer(name), "{name}");
    }
}

/// AF1: the swap counts only with a software renderer.
#[test]
fn a_gpu_swap_never_counts() {
    assert_eq!(frame_cost(Some(0.003), Some(0.016), false), Some(0.003));
    let soft = frame_cost(Some(0.003), Some(0.016), true).unwrap();
    assert!((soft - 0.019).abs() < 1e-6);
    assert_eq!(frame_cost(Some(0.003), None, true), Some(0.003));
    assert_eq!(frame_cost(None, Some(0.016), true), None);
}

/// AF4.
#[test]
fn the_target_follows_the_budget() {
    let ui = UiConfig::default();
    let t = target_fps(0.030, &ui);
    assert!((t - 40.0 / 100.0 / 0.030).abs() < 0.01, "{t}");
    assert_eq!(target_fps(0.002, &ui), 30.0);
    assert_eq!(target_fps(10.0, &ui), 8.0, "held at the floor");
}

/// AF8: the floor never lifts the rate above `max_fps`.
#[test]
fn the_floor_never_exceeds_the_ceiling() {
    let ui = UiConfig {
        min_fps: 20,
        max_fps: 10,
        ..UiConfig::default()
    };
    assert_eq!(target_fps(1.0, &ui), 10.0);
    let mut rate = AdaptiveRate::default();
    let mut now = 0.0;
    run(&mut rate, &ui, 1.0, 60, &mut now);
    assert_eq!(rate.fps(&ui), 10.0);
}

/// AF4: `max_fps` 0 has no ceiling; a cheap frame keeps every display frame.
#[test]
fn an_unlimited_ceiling_stays_unlimited() {
    let ui = UiConfig {
        max_fps: 0,
        ..UiConfig::default()
    };
    let mut rate = AdaptiveRate::default();
    let mut now = 0.0;
    let notes = run(&mut rate, &ui, 0.002, 60, &mut now);
    assert!(notes.is_empty(), "{notes:?}");
    assert_eq!(rate.period(&ui), Duration::ZERO);
    // A slow renderer limits it all the same.
    run(&mut rate, &ui, 0.050, 120, &mut now);
    assert!(rate.period(&ui) >= Duration::from_millis(120));
}

/// AF3: one slow frame does not move the rate.
#[test]
fn one_slow_frame_does_not_drop_the_rate() {
    let ui = UiConfig::default();
    let mut rate = AdaptiveRate::default();
    let mut now = 0.0;
    run(&mut rate, &ui, 0.002, 60, &mut now);
    let notes = run(&mut rate, &ui, 0.100, 1, &mut now);
    assert!(notes.is_empty());
    assert_eq!(rate.fps(&ui), 30.0);
    assert_eq!(rate.period(&ui), Duration::from_secs(1) / 30);
}

/// AF3–AF5, AF7: a sustained cost limits the rate (one note), cheap frames
/// bring it back by at most 5 fps a second, and a note says so.
#[test]
fn a_sustained_cost_drops_the_rate_and_it_climbs_back_slowly() {
    let ui = UiConfig::default();
    let mut rate = AdaptiveRate::default();
    let mut now = 0.0;
    let notes = run(&mut rate, &ui, 0.030, 60, &mut now);
    let limited = rate.fps(&ui);
    assert!(limited <= 14.0, "{limited}");
    assert_eq!(
        notes
            .iter()
            .filter(|n| matches!(n, RateNote::Limited { .. }))
            .count(),
        1,
        "{notes:?}"
    );
    run(&mut rate, &ui, 0.002, 30, &mut now);
    let after_a_second = rate.fps(&ui);
    assert!(after_a_second <= limited + 5.0 + 1e-3, "{after_a_second}");
    assert!(after_a_second > limited, "it climbs");
    let notes = run(&mut rate, &ui, 0.002, 300, &mut now);
    assert_eq!(rate.fps(&ui), 30.0);
    assert!(
        notes.iter().any(|n| matches!(n, RateNote::Restored { .. })),
        "{notes:?}"
    );
}

/// AF7: while limited, a note only when the rate moves by 5 fps or more.
#[test]
fn notes_are_few() {
    let ui = UiConfig::default();
    let mut rate = AdaptiveRate::default();
    let mut now = 0.0;
    let notes = run(&mut rate, &ui, 0.030, 600, &mut now);
    assert_eq!(notes.len(), 1, "{notes:?}");
}

/// AF3: only frames drawn while something moves count.
#[test]
fn only_moving_frames_count() {
    let ui = UiConfig::default();
    let mut rate = AdaptiveRate::default();
    for i in 0..120 {
        assert!(rate.observe(Some(0.5), false, f64::from(i), &ui).is_none());
    }
    assert_eq!(rate.fps(&ui), 30.0);
}

#[test]
fn no_measurement_keeps_the_ceiling() {
    let ui = UiConfig::default();
    let mut rate = AdaptiveRate::default();
    for i in 0..120 {
        assert!(
            rate.observe(None, true, f64::from(i) * FRAME, &ui)
                .is_none()
        );
    }
    assert_eq!(rate.fps(&ui), 30.0);
}

#[test]
fn a_degenerate_cost_keeps_the_ceiling() {
    let ui = UiConfig::default();
    assert_eq!(target_fps(0.0, &ui), 30.0);
    assert_eq!(target_fps(f32::NAN, &ui), 30.0);
    assert_eq!(target_fps(-1.0, &ui), 30.0);
    let mut rate = AdaptiveRate::default();
    let mut now = 0.0;
    run(&mut rate, &ui, f32::NAN, 30, &mut now);
    run(&mut rate, &ui, 0.0, 30, &mut now);
    assert_eq!(rate.fps(&ui), 30.0);
}
