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
        Ok(Retired::Source { slot: 0, .. })
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
    assert!(matches!(
        h.retired.pop(),
        Ok(Retired::Source { slot: 0, .. })
    ));
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
