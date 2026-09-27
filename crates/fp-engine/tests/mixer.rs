#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Sample-accurate behaviour of the bus mixer, driven block by block.

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use assert_no_alloc::{AllocDisabler, assert_no_alloc};
use fp_backends::Renderer;
use fp_engine::atomic::AtomicF32;
use fp_engine::mixer::{
    BusCommand, BusEvent, Mixer, MixerConfig, MixerHandle, MixerRenderer, Retired, SlotStorage,
};
use fp_engine::ramp::Curve;
use fp_engine::source::{SourceProducer, source_pair};

#[cfg(debug_assertions)]
#[global_allocator]
static ALLOCATOR: AllocDisabler = AllocDisabler;

const CONFIG: MixerConfig = MixerConfig {
    volume_smoothing_frames: 1,
    max_commands_per_block: 64,
};

fn mixer(slots: usize) -> (Mixer, MixerHandle) {
    Mixer::new(slots, CONFIG)
}

/// A source whose left samples are 1, 2, 3… and right samples are negative.
fn counting_source(
    frames: usize,
    eof: bool,
) -> (SourceProducer, fp_engine::source::SourceConsumer) {
    let (mut p, c) = source_pair(frames.max(1));
    let samples: Vec<f32> = (1..=frames).flat_map(|i| [i as f32, -(i as f32)]).collect();
    assert_eq!(p.push(&samples), samples.len());
    if eof {
        p.shared.eof.store(true, Ordering::Release);
    }
    (p, c)
}

fn full_volume() -> Arc<AtomicF32> {
    Arc::new(AtomicF32::new(1.0))
}

fn attach(
    h: &mut MixerHandle,
    slot: usize,
    source: fp_engine::source::SourceConsumer,
    first_channel: u16,
) {
    assert!(
        h.commands
            .push(BusCommand::Attach {
                slot,
                source,
                volume: full_volume(),
                first_channel
            })
            .is_ok()
    );
}

fn send(h: &mut MixerHandle, c: BusCommand) {
    assert!(h.commands.push(c).is_ok());
}

fn render(m: &mut Mixer, frames: usize, channels: usize) -> Vec<f32> {
    let mut out = vec![0.0; frames * channels];
    assert_no_alloc(|| m.render(&mut out, channels));
    out
}

fn events(h: &mut MixerHandle) -> Vec<BusEvent> {
    std::iter::from_fn(|| h.events.pop().ok()).collect()
}

fn left(out: &[f32]) -> Vec<f32> {
    out.chunks(2).map(|f| f[0]).collect()
}

#[test]
fn an_attached_source_is_silent_until_started_then_starts_on_the_exact_frame() {
    let (mut m, mut h) = mixer(4);
    let (_p, c) = counting_source(16, false);
    attach(&mut h, 0, c, 0);
    assert!(render(&mut m, 4, 2).iter().all(|s| *s == 0.0));
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 6,
        },
    );
    let out = render(&mut m, 4, 2); // frames 4..8
    assert_eq!(left(&out), vec![0.0, 0.0, 1.0, 2.0]);
    assert_eq!(out[5], -1.0, "right channel carries the right samples");
    assert_eq!(
        events(&mut h),
        vec![BusEvent::Started { slot: 0, frame: 6 }]
    );
}

#[test]
fn sources_are_summed_and_routed_to_their_channel_pair() {
    let (mut m, mut h) = mixer(4);
    let (_p1, a) = counting_source(4, false);
    let (_p2, b) = counting_source(4, false);
    attach(&mut h, 0, a, 0);
    attach(&mut h, 1, b, 2);
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 0,
        },
    );
    send(
        &mut h,
        BusCommand::Start {
            slot: 1,
            at_frame: 0,
        },
    );
    let out = render(&mut m, 2, 4);
    assert_eq!(out, vec![1.0, -1.0, 1.0, -1.0, 2.0, -2.0, 2.0, -2.0]);
}

#[test]
fn stop_at_ends_on_the_exact_frame_and_reports_finished() {
    let (mut m, mut h) = mixer(2);
    let (_p, c) = counting_source(16, false);
    attach(&mut h, 0, c, 0);
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 0,
        },
    );
    send(
        &mut h,
        BusCommand::StopAt {
            slot: 0,
            at_frame: 3,
        },
    );
    let out = render(&mut m, 5, 2);
    assert_eq!(left(&out), vec![1.0, 2.0, 3.0, 0.0, 0.0]);
    assert_eq!(
        events(&mut h),
        vec![
            BusEvent::Started { slot: 0, frame: 0 },
            BusEvent::Finished { slot: 0, frame: 3 }
        ]
    );
}

#[test]
fn a_drained_source_at_eof_finishes_on_its_last_frame() {
    let (mut m, mut h) = mixer(2);
    let (_p, c) = counting_source(3, true);
    attach(&mut h, 0, c, 0);
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 0,
        },
    );
    render(&mut m, 8, 2);
    assert!(events(&mut h).contains(&BusEvent::Finished { slot: 0, frame: 3 }));
}

#[test]
fn an_underrun_is_silent_counted_and_the_timeline_keeps_moving() {
    let (mut m, mut h) = mixer(2);
    let (p, c) = counting_source(2, false);
    let shared = p.shared.clone();
    attach(&mut h, 0, c, 0);
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 0,
        },
    );
    let out = render(&mut m, 4, 2);
    assert_eq!(left(&out), vec![1.0, 2.0, 0.0, 0.0]);
    assert_eq!(shared.underruns.load(Ordering::Relaxed), 1);
    assert_eq!(h.shared.frames_rendered(), 4);
    assert!(
        events(&mut h)
            .iter()
            .all(|e| !matches!(e, BusEvent::Finished { .. }))
    );
}

#[test]
fn a_ramp_scheduled_at_a_frame_starts_exactly_there() {
    let (mut m, mut h) = mixer(2);
    let (mut p, c) = source_pair(16);
    p.push(&[1.0; 32]);
    attach(&mut h, 0, c, 0);
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 0,
        },
    );
    send(
        &mut h,
        BusCommand::Ramp {
            slot: 0,
            to: 0.0,
            frames: 4,
            curve: Curve::Linear,
            at_frame: 2,
        },
    );
    let out = render(&mut m, 8, 2);
    assert_eq!(left(&out), vec![1.0, 1.0, 1.0, 0.75, 0.5, 0.25, 0.0, 0.0]);
}

#[test]
fn pause_fades_out_holds_the_position_and_resume_continues_from_it() {
    let (mut m, mut h) = mixer(2);
    let (p, c) = counting_source(32, false);
    let shared = p.shared.clone();
    attach(&mut h, 0, c, 0);
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 0,
        },
    );
    render(&mut m, 2, 2);
    send(
        &mut h,
        BusCommand::Pause {
            slot: 0,
            ramp_frames: 2,
        },
    );
    let out = render(&mut m, 6, 2);
    assert_eq!(left(&out), vec![3.0, 2.0, 0.0, 0.0, 0.0, 0.0]);
    let held = shared.frames_played();
    render(&mut m, 8, 2);
    assert_eq!(
        shared.frames_played(),
        held,
        "a paused source consumes nothing"
    );
    send(
        &mut h,
        BusCommand::Resume {
            slot: 0,
            ramp_frames: 1,
        },
    );
    let out = render(&mut m, 2, 2);
    assert_eq!(left(&out), vec![0.0, 6.0]);
}

#[test]
fn volume_changes_are_smoothed() {
    let config = MixerConfig {
        volume_smoothing_frames: 4,
        max_commands_per_block: 64,
    };
    let (mut m, mut h) = Mixer::new(2, config);
    let (mut p, c) = source_pair(16);
    p.push(&[1.0; 32]);
    let volume = full_volume();
    assert!(
        h.commands
            .push(BusCommand::Attach {
                slot: 0,
                source: c,
                volume: volume.clone(),
                first_channel: 0
            })
            .is_ok()
    );
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 0,
        },
    );
    render(&mut m, 1, 2);
    volume.store(0.0);
    let out = render(&mut m, 5, 2);
    assert_eq!(left(&out), vec![0.75, 0.5, 0.25, 0.0, 0.0]);
}

#[test]
fn detached_sources_and_old_storage_come_back_for_freeing_off_the_rt_thread() {
    let (mut m, mut h) = mixer(1);
    let (_p, c) = counting_source(8, false);
    attach(&mut h, 0, c, 0);
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 0,
        },
    );
    render(&mut m, 1, 2);
    send(&mut h, BusCommand::Grow(SlotStorage::with_capacity(3)));
    let out = render(&mut m, 1, 2);
    assert_eq!(
        left(&out),
        vec![2.0],
        "growing keeps playing sources in place"
    );
    assert!(matches!(h.retired.pop(), Ok(Retired::Storage(s)) if s.len() == 1));
    let (_p2, extra) = counting_source(8, false);
    attach(&mut h, 2, extra, 0);
    send(&mut h, BusCommand::Detach { slot: 0 });
    render(&mut m, 1, 2);
    assert!(matches!(
        h.retired.pop(),
        Ok(Retired::Source { slot: Some(0), .. })
    ));
}

#[test]
fn attaching_to_an_occupied_slot_hands_the_source_back() {
    let (mut m, mut h) = mixer(1);
    let (_p1, a) = counting_source(4, false);
    let (_p2, b) = counting_source(4, false);
    attach(&mut h, 0, a, 0);
    attach(&mut h, 0, b, 0);
    render(&mut m, 1, 2);
    // Handed back as "never attached", so the conductor keeps slot 0 marked busy.
    assert!(matches!(
        h.retired.pop(),
        Ok(Retired::Source { slot: None, .. })
    ));
}

#[test]
fn a_full_retired_queue_never_frees_memory_on_the_rt_thread() {
    let (mut m, mut h) = mixer(1);
    // Nobody drains `retired`: every detached source must be kept, not dropped.
    for _ in 0..1_100 {
        let (_p, c) = counting_source(4, false);
        attach(&mut h, 0, c, 0);
        send(&mut h, BusCommand::Detach { slot: 0 });
        render(&mut m, 1, 2);
    }
    assert!(h.shared.leaked.load(Ordering::Relaxed) > 0);
}

#[test]
fn a_command_flood_is_spread_over_several_blocks() {
    let config = MixerConfig {
        volume_smoothing_frames: 1,
        max_commands_per_block: 4,
    };
    let (mut m, mut h) = Mixer::new(2, config);
    for _ in 0..10 {
        send(&mut h, BusCommand::Cancel { slot: 0 });
    }
    render(&mut m, 1, 2);
    assert_eq!(h.commands.slots(), h.commands.buffer().capacity() - 6);
}

#[test]
fn the_renderer_outputs_silence_instead_of_waiting_for_a_held_lock() {
    let (m, h) = mixer(1);
    let mixer = Arc::new(Mutex::new(m));
    let mut renderer = MixerRenderer {
        mixer: mixer.clone(),
        shared: h.shared.clone(),
    };
    let _guard = mixer.lock().unwrap();
    let mut out = vec![1.0; 8];
    renderer.render(&mut out, 2);
    assert!(out.iter().all(|s| *s == 0.0));
    assert_eq!(h.shared.lock_misses.load(Ordering::Relaxed), 1);
}

fn start(h: &mut MixerHandle, slot: usize) {
    send(h, BusCommand::Start { slot, at_frame: 0 });
}

#[test]
fn a_source_is_unaltered_only_at_unity_and_alone_on_its_pair() {
    let (mut m, mut h) = mixer(4);
    let (_pa, a) = counting_source(64, false);
    let alone = a.shared.clone();
    attach(&mut h, 0, a, 0);
    start(&mut h, 0);
    render(&mut m, 4, 4);
    assert!(alone.unaltered.load(Ordering::Acquire), "unity, alone");

    // Another source on channels 3/4 does not touch channels 1/2.
    let (_pb, b) = counting_source(64, false);
    let other_pair = b.shared.clone();
    attach(&mut h, 1, b, 2);
    start(&mut h, 1);
    render(&mut m, 4, 4);
    assert!(alone.unaltered.load(Ordering::Acquire));
    assert!(other_pair.unaltered.load(Ordering::Acquire));

    // A third one on channels 1/2 is mixed into the first.
    let (_pc, c) = counting_source(64, false);
    let mixed_in = c.shared.clone();
    attach(&mut h, 2, c, 0);
    start(&mut h, 2);
    render(&mut m, 4, 4);
    assert!(!alone.unaltered.load(Ordering::Acquire), "summed");
    assert!(!mixed_in.unaltered.load(Ordering::Acquire));
    assert!(other_pair.unaltered.load(Ordering::Acquire));
}

#[test]
fn a_gain_below_unity_alters_a_source() {
    let (mut m, mut h) = mixer(2);
    let (_p, c) = counting_source(64, false);
    let shared = c.shared.clone();
    let volume = Arc::new(AtomicF32::new(0.5));
    assert!(
        h.commands
            .push(BusCommand::Attach {
                slot: 0,
                source: c,
                volume: volume.clone(),
                first_channel: 0,
            })
            .is_ok()
    );
    start(&mut h, 0);
    render(&mut m, 4, 2);
    assert!(!shared.unaltered.load(Ordering::Acquire));
    volume.store(1.0);
    render(&mut m, 4, 2);
    render(&mut m, 4, 2);
    assert!(shared.unaltered.load(Ordering::Acquire), "back at unity");
}

#[test]
fn mixer_accumulates_energy_without_allocating() {
    let (mut m, mut h) = mixer(2);
    h.shared.sample_rate.store(48_000, Ordering::Release);
    let (_p, c) = counting_source(64, false);
    let shared = c.shared.clone();
    attach(&mut h, 0, c, 0);
    start(&mut h, 0);
    render(&mut m, 8, 2);
    let expected: f64 = (1..=8).map(|i| f64::from(i * i)).sum();
    assert_eq!(shared.measured_frames.load(Ordering::Acquire), 8);
    assert_eq!(shared.sum_sq_l.take(), expected);
    assert_eq!(shared.sum_sq_r.take(), expected);
    assert!(
        shared.k_sum_l.take() > 0.0,
        "K-weighted energy accumulates too"
    );
}

#[test]
fn true_peak_can_be_switched_on_for_a_bus() {
    let (mut m, mut h) = mixer(2);
    h.shared.sample_rate.store(48_000, Ordering::Release);
    // fs/4 at 45°: sample peak 0.707, true peak 1.0.
    let (mut p, c) = source_pair(4_096);
    let samples: Vec<f32> = (0..2_000)
        .flat_map(|n| {
            let x = (std::f32::consts::FRAC_PI_2 * n as f32 + std::f32::consts::FRAC_PI_4).sin();
            [x, x]
        })
        .collect();
    assert_eq!(p.push(&samples), samples.len());
    let shared = c.shared.clone();
    attach(&mut h, 0, c, 0);
    start(&mut h, 0);
    render(&mut m, 200, 2);
    let sample_peak = shared.peak_l.take();
    assert!((sample_peak - 0.707).abs() < 0.01, "{sample_peak}");
    h.shared.true_peak.store(true, Ordering::Release);
    render(&mut m, 200, 2);
    let true_peak = shared.peak_l.take();
    assert!(true_peak > 0.97, "{true_peak}");
}

/// Reading, relative to the steady tone, of a 5 kHz burst of `burst_ms` on
/// the programme meter `ballistics` configures (the mixer's integration).
fn burst_reading_db(ballistics: fp_model::MeterBallistics, burst_ms: f64) -> f32 {
    let config = fp_model::MeterConfig {
        ballistics,
        ..fp_model::MeterConfig::default()
    };
    let (tau1, tau2, fall) = fp_engine::meter::mixer_integration(&config);
    let read = |ms: f64| {
        let (mut m, mut h) = mixer(2);
        h.shared.sample_rate.store(48_000, Ordering::Release);
        h.shared.ppm_tau1_ms.store(tau1);
        h.shared.ppm_tau2_ms.store(tau2);
        h.shared.fall_db_per_sec.store(fall);
        let frames = 48_000usize;
        let burst = (ms * 48.0).round() as usize;
        let (mut p, c) = source_pair(frames);
        let samples: Vec<f32> = (0..frames)
            .flat_map(|n| {
                let x = if n < burst {
                    (std::f64::consts::TAU * 5_000.0 * n as f64 / 48_000.0).sin() as f32
                } else {
                    0.0
                };
                [x, x]
            })
            .collect();
        assert_eq!(p.push(&samples), samples.len());
        let shared = c.shared.clone();
        attach(&mut h, 0, c, 0);
        start(&mut h, 0);
        let mut peak = 0.0f32;
        for _ in 0..(burst + 4_800) / 480 + 1 {
            render(&mut m, 480, 2);
            peak = peak.max(shared.peak_l.take());
        }
        peak
    };
    20.0 * (read(burst_ms) / read(400.0)).log10()
}

#[test]
fn the_ebu_ppm_meets_tech_3205_table_2() {
    // EBU Tech 3205-E, table 2 (normal mode): burst → reading re steady tone.
    for (ms, expected, tolerance) in [
        (100.0, 0.0, 0.5),
        (10.0, -2.0, 0.5),
        (5.0, -4.0, 0.75),
        (1.5, -9.0, 1.0),
        (0.5, -17.0, 2.0),
    ] {
        let r = burst_reading_db(fp_model::MeterBallistics::EbuPpm, ms);
        assert!(
            (r - expected).abs() <= tolerance,
            "{ms} ms: {r} dB, expected {expected} ± {tolerance}"
        );
    }
}

#[test]
fn the_din_ppm_has_a_5_ms_integration_time() {
    // IEC definition: a burst of the integration time reads 2 dB low.
    let r = burst_reading_db(fp_model::MeterBallistics::DinPpm, 5.0);
    assert!((r + 2.0).abs() <= 0.5, "{r}");
}

#[test]
fn a_digital_peak_meter_shows_even_the_shortest_burst() {
    let r = burst_reading_db(fp_model::MeterBallistics::DigitalPeak, 0.5);
    assert!(r.abs() < 0.1, "{r}");
}
