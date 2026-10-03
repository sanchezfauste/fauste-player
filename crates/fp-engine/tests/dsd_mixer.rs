#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The bus mixer's DSD mode (feedback 2 spec O25), block by block.

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use assert_no_alloc::{AllocDisabler, assert_no_alloc};
use fp_backends::Renderer;
use fp_backends::dsd::{DopEncoder, silence_sample, word_to_sample};
use fp_engine::atomic::AtomicF32;
use fp_engine::mixer::{BusCommand, BusEvent, Mixer, MixerConfig, MixerHandle, MixerRenderer};
use fp_engine::ramp::Curve;
use fp_engine::source::{SourceConsumer, SourceProducer, source_pair, source_pair_dsd};

#[cfg(debug_assertions)]
#[global_allocator]
static ALLOCATOR: AllocDisabler = AllocDisabler;

const CONFIG: MixerConfig = MixerConfig {
    volume_smoothing_frames: 1,
    declick_frames: 0,
    max_commands_per_block: 64,
};

fn send(h: &mut MixerHandle, c: BusCommand) {
    assert!(h.commands.push(c).is_ok());
}

fn render(m: &mut Mixer, frames: usize) -> Vec<f32> {
    let mut out = vec![0.0; frames * 2];
    assert_no_alloc(|| m.render(&mut out, 2));
    out
}

fn events(h: &mut MixerHandle) -> Vec<BusEvent> {
    std::iter::from_fn(|| h.events.pop().ok()).collect()
}

/// A DSD source of `frames` frames: words 1, 2, 3… (as samples) on the
/// left, their negatives on the right; its PCM ring holds 0.25 everywhere.
fn dsd_source(frames: usize, volume: f32) -> (SourceProducer, SourceConsumer, Arc<AtomicF32>) {
    let (mut p, c) = source_pair_dsd(frames);
    let words: Vec<f32> = (1..=frames)
        .flat_map(|i| {
            let w = word_to_sample((i >> 8) as u8, i as u8);
            [w, -w]
        })
        .collect();
    let pcm = vec![0.25; frames * 2];
    assert_eq!(p.push_pair(&pcm, &words), frames);
    (p, c, Arc::new(AtomicF32::new(volume)))
}

fn attach(h: &mut MixerHandle, slot: usize, source: SourceConsumer, volume: Arc<AtomicF32>) {
    send(
        h,
        BusCommand::Attach {
            slot,
            source,
            volume,
            first_channel: 0,
        },
    );
}

#[test]
fn in_dsd_mode_the_words_are_copied_exactly_whatever_the_volume() {
    let (mut m, mut h) = Mixer::new(4, CONFIG);
    let (_p, c, v) = dsd_source(8, 0.3);
    attach(&mut h, 0, c, v);
    send(
        &mut h,
        BusCommand::DsdMode {
            on: true,
            at_frame: 0,
        },
    );
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 2,
        },
    );
    let out = render(&mut m, 6);
    assert_eq!(out[0], silence_sample(), "before the start: DSD silence");
    assert_eq!(out[2], silence_sample());
    assert_eq!(
        out[4],
        word_to_sample(0, 1),
        "first word, unchanged by the 0.3 volume"
    );
    assert_eq!(out[5], -word_to_sample(0, 1));
    assert_eq!(out[10], word_to_sample(0, 4));
    assert!(m.shared().dsd_on.load(Ordering::Acquire));
}

#[test]
fn in_dsd_mode_other_sources_are_muted_but_advance() {
    let (mut m, mut h) = Mixer::new(4, CONFIG);
    let (_p, c, v) = dsd_source(16, 1.0);
    attach(&mut h, 0, c, v);
    let (mut p2, c2) = source_pair(16);
    p2.push(&[0.5; 32]);
    send(
        &mut h,
        BusCommand::Attach {
            slot: 1,
            source: c2,
            volume: Arc::new(AtomicF32::new(1.0)),
            first_channel: 0,
        },
    );
    send(
        &mut h,
        BusCommand::DsdMode {
            on: true,
            at_frame: 0,
        },
    );
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
    let out = render(&mut m, 4);
    assert_eq!(
        out[0],
        word_to_sample(0, 1),
        "nothing summed into the DSD word"
    );
    assert_eq!(p2.shared.frames_played(), 4, "the muted source advanced");
    assert_eq!(h.shared.misrouted.load(Ordering::Relaxed), 0);
}

#[test]
fn the_dsd_slot_is_metered_from_its_pcm_ring() {
    let (mut m, mut h) = Mixer::new(4, CONFIG);
    let (p, c, v) = dsd_source(8, 1.0);
    attach(&mut h, 0, c, v);
    send(
        &mut h,
        BusCommand::DsdMode {
            on: true,
            at_frame: 0,
        },
    );
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 0,
        },
    );
    render(&mut m, 8);
    assert_eq!(
        p.shared.peak_l.load(),
        0.25,
        "the PCM conversion, not the words"
    );
    assert_eq!(p.shared.measured_frames.load(Ordering::Acquire), 8);
    assert!(
        !p.shared.unaltered.load(Ordering::Acquire),
        "BP never lights in DSD mode"
    );
}

#[test]
fn with_dsd_mode_off_the_dsd_slot_mixes_its_pcm_with_gain() {
    let (mut m, mut h) = Mixer::new(4, CONFIG);
    let (_p, c, v) = dsd_source(8, 0.5);
    attach(&mut h, 0, c, v);
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 0,
        },
    );
    let out = render(&mut m, 4);
    assert_eq!(out[0], 0.125);
    assert!(!m.shared().dsd_on.load(Ordering::Acquire));
}

#[test]
fn hold_all_freezes_every_slot_and_defers_starts_to_the_next_block() {
    let (mut m, mut h) = Mixer::new(4, CONFIG);
    let (p, c, v) = dsd_source(16, 1.0);
    attach(&mut h, 0, c, v);
    send(
        &mut h,
        BusCommand::DsdMode {
            on: true,
            at_frame: 0,
        },
    );
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 0,
        },
    );
    render(&mut m, 4); // words 1..=4
    send(&mut h, BusCommand::HoldAll { until_frame: 8 });
    send(
        &mut h,
        BusCommand::DsdMode {
            on: false,
            at_frame: 8,
        },
    );
    let held = render(&mut m, 4); // frames 4..8: held, silence words
    assert!(held.iter().all(|s| *s == silence_sample()));
    assert_eq!(p.shared.frames_played(), 4, "nothing consumed while held");
    let after = render(&mut m, 4); // frames 8..12: PCM now, from where it held
    assert_eq!(after[0], 0.25);
    assert_eq!(p.shared.frames_played(), 8);
}

#[test]
fn the_renderer_encodes_dop_only_while_the_bus_is_in_dsd_mode() {
    let (m, mut h) = Mixer::new(4, CONFIG);
    let shared = m.shared().clone();
    let mixer = Arc::new(Mutex::new(m));
    let mut r = MixerRenderer {
        mixer: mixer.clone(),
        shared: shared.clone(),
        dop: Some(DopEncoder::new()),
    };
    send(
        &mut h,
        BusCommand::DsdMode {
            on: true,
            at_frame: 0,
        },
    );
    let mut out = vec![0.0; 4 * 2];
    assert_no_alloc(|| r.render(&mut out, 2));
    let marker = |s: f32| ((s * 8_388_608.0) as i32 as u32 >> 16) & 0xFF;
    assert_eq!(
        out.iter().map(|s| marker(*s)).collect::<Vec<_>>(),
        vec![5, 5, 0xFA, 0xFA, 5, 5, 0xFA, 0xFA]
    );
    // A lock miss keeps the stream valid DoP: silence, markers continuing.
    let guard = mixer.lock().unwrap();
    assert_no_alloc(|| r.render(&mut out, 2));
    drop(guard);
    assert_eq!(marker(out[0]), 5);
    assert_eq!(((out[0] * 8_388_608.0) as i32 as u32) & 0xFFFF, 0x6969);
    assert_eq!(shared.lock_misses.load(Ordering::Relaxed), 1);
    send(
        &mut h,
        BusCommand::DsdMode {
            on: false,
            at_frame: 0,
        },
    );
    assert_no_alloc(|| r.render(&mut out, 2));
    assert!(
        out.iter().all(|s| *s == 0.0),
        "PCM silence once DSD mode is off"
    );
}

#[test]
fn no_gain_of_any_kind_touches_the_words() {
    let (mut m, mut h) = Mixer::new(4, CONFIG);
    let (_p, c, v) = dsd_source(16, 1.0);
    attach(&mut h, 0, c, v.clone());
    send(
        &mut h,
        BusCommand::DsdMode {
            on: true,
            at_frame: 0,
        },
    );
    // A1: ramp to 0, start, ramp to 1; A2: a cut ramp; a volume change.
    send(
        &mut h,
        BusCommand::Ramp {
            slot: 0,
            to: 0.0,
            frames: 4,
            curve: Curve::Linear,
            at_frame: 0,
        },
    );
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 0,
        },
    );
    send(
        &mut h,
        BusCommand::RampOutBeforeCut {
            slot: 0,
            frames: 4,
            at_frame: 2,
        },
    );
    v.store(0.2);
    let out = render(&mut m, 8);
    for i in 0..8 {
        assert_eq!(out[i * 2], word_to_sample(0, (i + 1) as u8), "frame {i}");
        assert_eq!(out[i * 2 + 1], -word_to_sample(0, (i + 1) as u8));
    }
}

#[test]
fn pause_holds_at_once_resume_continues_and_stop_cuts_on_its_frame() {
    let (mut m, mut h) = Mixer::new(4, CONFIG);
    let (_p, c, v) = dsd_source(32, 1.0);
    attach(&mut h, 0, c, v);
    send(
        &mut h,
        BusCommand::DsdMode {
            on: true,
            at_frame: 0,
        },
    );
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 0,
        },
    );
    render(&mut m, 2); // words 1, 2
    send(
        &mut h,
        BusCommand::Pause {
            slot: 0,
            ramp_frames: 64,
        },
    );
    let out = render(&mut m, 2);
    assert!(out.iter().all(|s| *s == silence_sample()), "paused at once");
    send(
        &mut h,
        BusCommand::Resume {
            slot: 0,
            ramp_frames: 64,
        },
    );
    send(
        &mut h,
        BusCommand::StopAt {
            slot: 0,
            at_frame: 6,
        },
    );
    let out = render(&mut m, 4); // frames 4..8
    assert_eq!(out[0], word_to_sample(0, 3), "continues at once");
    assert_eq!(out[2], word_to_sample(0, 4));
    assert_eq!(out[4], silence_sample(), "cut on frame 6");
    assert!(
        events(&mut h)
            .iter()
            .any(|e| matches!(e, BusEvent::Finished { frame: 6, .. }))
    );
}

#[test]
fn a_ramp_pending_across_a_mode_switch_does_not_fire_with_a_jump() {
    let (mut m, mut h) = Mixer::new(4, CONFIG);
    let (_p, c, v) = dsd_source(64, 1.0);
    attach(&mut h, 0, c, v);
    send(
        &mut h,
        BusCommand::DsdMode {
            on: true,
            at_frame: 0,
        },
    );
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 0,
        },
    );
    let first = render(&mut m, 2);
    assert_eq!(first[2], word_to_sample(0, 2));
    // A fade out due at frame 4, inside DSD mode; the mode ends at frame 8.
    send(
        &mut h,
        BusCommand::Ramp {
            slot: 0,
            to: 0.0,
            frames: 8,
            curve: Curve::Linear,
            at_frame: 4,
        },
    );
    send(
        &mut h,
        BusCommand::DsdMode {
            on: false,
            at_frame: 8,
        },
    );
    let dsd = render(&mut m, 6);
    assert_eq!(dsd[10], word_to_sample(0, 8), "words untouched by the fade");
    // The fade is settled as if it had run: PCM resumes at its end gain,
    // with no sample at a gain the fade never passed through (no jump up).
    let pcm = render(&mut m, 4);
    assert!(pcm.iter().all(|s| *s == 0.0), "{pcm:?}");
}

#[test]
fn a_ramp_in_progress_is_frozen_in_dsd_mode_and_continues_from_its_level() {
    let (mut m, mut h) = Mixer::new(4, CONFIG);
    let (_p, c, v) = dsd_source(64, 1.0);
    attach(&mut h, 0, c, v);
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
            frames: 8,
            curve: Curve::Linear,
            at_frame: 0,
        },
    );
    let a = render(&mut m, 4); // PCM: gains 1, 7/8, 6/8, 5/8
    assert_eq!(a[6], 0.25 * 0.625);
    send(
        &mut h,
        BusCommand::DsdMode {
            on: true,
            at_frame: 4,
        },
    );
    let dsd = render(&mut m, 4);
    assert!(m.shared().dsd_on.load(Ordering::Acquire));
    assert_eq!(dsd[0], word_to_sample(0, 5), "words untouched");
    send(
        &mut h,
        BusCommand::DsdMode {
            on: false,
            at_frame: 8,
        },
    );
    let b = render(&mut m, 4);
    assert!(!m.shared().dsd_on.load(Ordering::Acquire));
    assert_eq!(
        b[0],
        0.25 * 0.5,
        "continues from where it froze: gain 4/8, no step"
    );
}

#[test]
fn a_ramp_issued_in_dsd_mode_settles_like_a_scheduled_one() {
    let (mut m, mut h) = Mixer::new(4, CONFIG);
    let (_p, c, v) = dsd_source(64, 1.0);
    attach(&mut h, 0, c, v);
    send(
        &mut h,
        BusCommand::DsdMode {
            on: true,
            at_frame: 0,
        },
    );
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 0,
        },
    );
    render(&mut m, 4);
    send(
        &mut h,
        BusCommand::Ramp {
            slot: 0,
            to: 0.5,
            frames: 8,
            curve: Curve::Linear,
            at_frame: 0,
        },
    );
    render(&mut m, 4);
    send(
        &mut h,
        BusCommand::DsdMode {
            on: false,
            at_frame: 8,
        },
    );
    let out = render(&mut m, 2);
    assert_eq!(
        out[0],
        0.25 * 0.5,
        "already at the target, not ramping from 1"
    );
}

#[test]
fn two_switches_queued_together_both_happen_in_order() {
    let (mut m, mut h) = Mixer::new(4, CONFIG);
    let (_p, c, v) = dsd_source(64, 1.0);
    attach(&mut h, 0, c, v);
    send(
        &mut h,
        BusCommand::DsdMode {
            on: true,
            at_frame: 0,
        },
    );
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 0,
        },
    );
    render(&mut m, 4);
    // Off at 4 and on at 8, queued in one drain.
    send(
        &mut h,
        BusCommand::DsdMode {
            on: false,
            at_frame: 4,
        },
    );
    send(
        &mut h,
        BusCommand::DsdMode {
            on: true,
            at_frame: 8,
        },
    );
    let pcm = render(&mut m, 4);
    assert!(
        !m.shared().dsd_on.load(Ordering::Acquire),
        "PCM between the switches"
    );
    assert_eq!(pcm[0], 0.25);
    let dsd = render(&mut m, 4);
    assert!(m.shared().dsd_on.load(Ordering::Acquire));
    assert_eq!(dsd[0], word_to_sample(0, 9));
}

#[test]
fn a_full_switch_queue_drops_the_oldest_and_counts_it() {
    let (mut m, mut h) = Mixer::new(1, CONFIG);
    for i in 0..5u64 {
        send(
            &mut h,
            BusCommand::DsdMode {
                on: i % 2 == 0,
                at_frame: 100 + i,
            },
        );
    }
    render(&mut m, 2);
    assert_eq!(h.shared.dropped_events.load(Ordering::Relaxed), 1);
}

#[test]
fn a_switch_inside_a_block_lands_at_the_next_block_start() {
    let (mut m, mut h) = Mixer::new(4, CONFIG);
    let (_p, c, v) = dsd_source(64, 1.0);
    attach(&mut h, 0, c, v);
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 0,
        },
    );
    send(
        &mut h,
        BusCommand::DsdMode {
            on: true,
            at_frame: 2,
        },
    );
    let first = render(&mut m, 4); // frames 0..4 contain frame 2
    assert!(
        !m.shared().dsd_on.load(Ordering::Acquire),
        "the block stays PCM"
    );
    assert_eq!(first[0], 0.25);
    render(&mut m, 4);
    assert!(m.shared().dsd_on.load(Ordering::Acquire));
}

#[test]
fn a_block_straddling_the_hold_end_is_still_held() {
    let (mut m, mut h) = Mixer::new(4, CONFIG);
    let (p, c, v) = dsd_source(64, 1.0);
    attach(&mut h, 0, c, v);
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 0,
        },
    );
    send(&mut h, BusCommand::HoldAll { until_frame: 6 });
    render(&mut m, 4); // frames 0..4, held
    render(&mut m, 4); // frames 4..8 start before 6: held
    assert_eq!(p.shared.frames_played(), 0);
    render(&mut m, 4);
    assert_eq!(p.shared.frames_played(), 4);
}

#[test]
fn a_failed_dsd_source_stops_its_words_and_leaves_silence_without_a_ramp() {
    let config = MixerConfig {
        declick_frames: 8,
        ..CONFIG
    };
    let (mut m, mut h) = Mixer::new(4, config);
    let (p, c, v) = dsd_source(4, 1.0);
    attach(&mut h, 0, c, v);
    send(
        &mut h,
        BusCommand::DsdMode {
            on: true,
            at_frame: 0,
        },
    );
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 0,
        },
    );
    p.shared.failed.store(true, Ordering::Release);
    let out = render(&mut m, 8);
    for i in 0..4 {
        assert_eq!(
            out[i * 2],
            word_to_sample(0, (i + 1) as u8),
            "no ramp on word {i}"
        );
    }
    for s in &out[8..] {
        assert_eq!(*s, silence_sample());
    }
    assert!(
        events(&mut h)
            .iter()
            .any(|e| matches!(e, BusEvent::Finished { .. }))
    );
}

#[test]
fn an_underrun_in_dsd_mode_leaves_the_silence_fill() {
    let (mut m, mut h) = Mixer::new(4, CONFIG);
    let (p, c, v) = dsd_source(2, 1.0);
    attach(&mut h, 0, c, v);
    send(
        &mut h,
        BusCommand::DsdMode {
            on: true,
            at_frame: 0,
        },
    );
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 0,
        },
    );
    let out = render(&mut m, 4);
    assert_eq!(out[2], word_to_sample(0, 2));
    assert_eq!(out[4], silence_sample());
    assert_eq!(p.shared.underruns.load(Ordering::Relaxed), 1);
}

#[test]
fn the_rest_of_the_renderer_paths_never_allocate() {
    let (mut m, mut h) = Mixer::new(4, CONFIG);
    let (_p, c, v) = dsd_source(64, 1.0);
    attach(&mut h, 0, c, v);
    send(
        &mut h,
        BusCommand::DsdMode {
            on: true,
            at_frame: 0,
        },
    );
    send(
        &mut h,
        BusCommand::Start {
            slot: 0,
            at_frame: 0,
        },
    );
    send(&mut h, BusCommand::HoldAll { until_frame: 8 });
    render(&mut m, 8);
    render(&mut m, 8);
    send(
        &mut h,
        BusCommand::Pause {
            slot: 0,
            ramp_frames: 4,
        },
    );
    render(&mut m, 8);
}
