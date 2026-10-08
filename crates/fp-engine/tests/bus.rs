#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Device loss, the virtual clock and reconnection (spec §4.7).

use std::sync::Arc;
use std::time::{Duration, Instant};

use fp_backends::{OfflineBackend, OfflineDevice, StreamConfig};
use fp_engine::atomic::AtomicF32;
use fp_engine::bus::{Bus, BusHealth, BusKey, BusTiming};
use fp_engine::mixer::{BusCommand, MixerConfig};
use fp_engine::source::source_pair;

const CONFIG: StreamConfig = StreamConfig {
    sample_rate: 48_000,
    buffer_frames: 480,
    channels: 2,
    exclusive: false,
    dsd: None,
    exact_buffer: false,
};
const MIXER: MixerConfig = MixerConfig {
    volume_smoothing_frames: 1,
    declick_frames: 0,
    max_commands_per_block: 64,
};
const TIMING: BusTiming = BusTiming {
    watchdog_timeout: Duration::from_millis(500),
    reconnect_interval: Duration::from_secs(2),
    startup_grace: Duration::from_secs(5),
    busy_retries: 2,
    busy_retry_interval: Duration::ZERO,
};

fn setup(plugged: bool) -> (OfflineBackend, OfflineDevice, Bus, Instant) {
    let backend = OfflineBackend::new();
    let device = backend.add_device("card", 2);
    if !plugged {
        device.unplug();
    }
    let key = BusKey {
        backend: "offline".into(),
        device: "card".into(),
    };
    let t0 = Instant::now();
    let bus = Bus::open(key, Arc::new(backend.clone()), CONFIG, 4, MIXER, TIMING, t0);
    (backend, device, bus, t0)
}

#[test]
fn a_healthy_device_drives_the_bus_clock() {
    let (_b, device, mut bus, t0) = setup(true);
    assert_eq!(bus.health(), BusHealth::Ok);
    device.render(480).unwrap();
    device.render(480).unwrap();
    bus.supervise(t0 + Duration::from_millis(20));
    assert_eq!(bus.now_frame(), 960);
    assert_eq!(bus.health(), BusHealth::Ok);
}

#[test]
fn unplugging_switches_to_the_virtual_clock_and_the_timeline_keeps_moving() {
    let (_b, device, mut bus, t0) = setup(true);
    device.render(480).unwrap();
    device.unplug();
    bus.supervise(t0 + Duration::from_millis(10));
    assert_eq!(bus.health(), BusHealth::Lost);
    let before = bus.now_frame();
    std::thread::sleep(Duration::from_millis(150));
    let advanced = bus.now_frame() - before;
    assert!(
        advanced >= 2_400,
        "virtual clock rendered only {advanced} frames"
    );
}

#[test]
fn a_device_that_stops_rendering_is_declared_lost_by_the_watchdog() {
    let (_b, device, mut bus, t0) = setup(true);
    device.render(480).unwrap();
    bus.supervise(t0 + Duration::from_millis(100));
    bus.supervise(t0 + Duration::from_millis(500));
    assert_eq!(bus.health(), BusHealth::Ok);
    bus.supervise(t0 + Duration::from_millis(700));
    assert_eq!(bus.health(), BusHealth::Lost);
}

#[test]
fn a_slow_starting_device_gets_a_grace_period_before_its_first_block() {
    let (_b, _device, mut bus, t0) = setup(true);
    bus.supervise(t0 + Duration::from_millis(600));
    assert_eq!(
        bus.health(),
        BusHealth::Ok,
        "Bluetooth and bridged devices can take a while to start"
    );
    bus.supervise(t0 + Duration::from_secs(6));
    assert_eq!(
        bus.health(),
        BusHealth::Lost,
        "but a device that never starts is still detected"
    );
}

#[test]
fn a_returning_device_takes_the_mixer_back_without_losing_time() {
    let (_b, device, mut bus, t0) = setup(true);
    device.unplug();
    bus.supervise(t0);
    std::thread::sleep(Duration::from_millis(60));
    device.replug();
    bus.supervise(t0 + Duration::from_secs(1));
    assert_eq!(
        bus.health(),
        BusHealth::Lost,
        "retries wait for the reconnect interval"
    );
    bus.supervise(t0 + Duration::from_secs(3));
    assert_eq!(
        bus.health(),
        BusHealth::Lost,
        "reopened, but not back before its first block"
    );
    device.render(480).unwrap();
    bus.supervise(t0 + Duration::from_millis(3_010));
    assert_eq!(bus.health(), BusHealth::Ok);
    let at_handover = bus.now_frame();
    assert!(at_handover > 0);
    std::thread::sleep(Duration::from_millis(40));
    assert_eq!(
        bus.now_frame(),
        at_handover,
        "the virtual clock has stopped"
    );
    device.render(480).unwrap();
    assert_eq!(
        bus.now_frame(),
        at_handover + 480,
        "the device continues the same timeline"
    );
}

#[test]
fn a_device_missing_at_startup_starts_lost_on_the_virtual_clock() {
    let (_b, _device, bus, _t0) = setup(false);
    assert_eq!(bus.health(), BusHealth::Lost);
    assert!(bus.last_error().is_some());
    std::thread::sleep(Duration::from_millis(60));
    assert!(bus.now_frame() > 0);
}

#[test]
fn slots_are_reused_only_after_the_mixer_hands_them_back() {
    let (_b, device, mut bus, _t0) = setup(true);
    let slot = bus.alloc_slot().unwrap();
    let (_p, c) = source_pair(16);
    assert!(bus.send(BusCommand::Attach {
        slot,
        source: c,
        volume: Arc::new(AtomicF32::new(1.0)),
        first_channel: 0
    }));
    assert!(bus.send(BusCommand::Detach { slot }));
    for _ in 0..3 {
        bus.alloc_slot().unwrap();
    }
    assert_eq!(bus.alloc_slot(), None, "all four slots are taken");
    device.render(16).unwrap();
    bus.poll();
    assert_eq!(bus.alloc_slot(), Some(slot));
    bus.ensure_capacity(6);
    assert_eq!(bus.capacity(), 6);
    device.render(16).unwrap();
    assert!(bus.alloc_slot().is_some());
}

#[test]
fn a_failed_reconnection_keeps_the_virtual_clock_running() {
    let (_b, _device, mut bus, t0) = setup(false);
    bus.supervise(t0 + Duration::from_secs(3)); // retries; the device is still away
    assert_eq!(bus.health(), BusHealth::Lost);
    let before = bus.now_frame();
    std::thread::sleep(Duration::from_millis(100));
    assert!(bus.now_frame() > before, "the timeline keeps moving");
}

#[test]
fn a_native_dsd_stream_sends_dsd_silence_before_dsd_mode_reaches_the_mixer() {
    use fp_backends::dsd::{DsdStream, silence_sample};
    let backend = OfflineBackend::new();
    let device = backend.add_device("dac", 2);
    device.set_exclusive_capable(true);
    device.set_native_dsd(true);
    let key = BusKey {
        backend: "offline".into(),
        device: "dac".into(),
    };
    let native = StreamConfig {
        sample_rate: 176_400,
        exclusive: true,
        dsd: Some(DsdStream::Native),
        ..CONFIG
    };
    let _bus = Bus::open(
        key,
        Arc::new(backend),
        native,
        4,
        MIXER,
        TIMING,
        Instant::now(),
    );
    // No `DsdMode` was sent: the mixer is still in PCM mode.
    let first = device.render(480).unwrap();
    assert!(
        first.iter().all(|s| *s == silence_sample()),
        "the first native period is DSD silence, not packed zeros"
    );
}

#[test]
fn the_watchdog_falls_back_to_the_pcm_rate_before_dsd_instead_of_retrying_forever() {
    let backend = OfflineBackend::new();
    let device = backend.add_device("dac", 2);
    device.set_max_pcm_rate(192_000);
    let key = BusKey {
        backend: "offline".into(),
        device: "dac".into(),
    };
    // Left at a word rate the device does not take as PCM.
    let word_rate = StreamConfig {
        sample_rate: 352_800,
        ..CONFIG
    };
    let t0 = Instant::now();
    let mut bus = Bus::open(key, Arc::new(backend), word_rate, 4, MIXER, TIMING, t0);
    assert_eq!(bus.health(), BusHealth::Lost);
    bus.set_pcm_fallback(CONFIG);
    bus.supervise(t0 + Duration::from_secs(3));
    device.render(480).unwrap();
    bus.supervise(t0 + Duration::from_millis(3_010));
    assert_eq!(bus.health(), BusHealth::Ok);
    assert_eq!(device.config().unwrap().sample_rate, 48_000);
    assert_eq!(bus.sample_rate(), 48_000);
    assert_eq!(bus.take_rate_change(), Some(352_800));
    assert_eq!(bus.take_rate_change(), None);
}

#[test]
fn a_rate_change_retries_a_device_that_was_busy_for_a_moment() {
    let (_b, device, mut bus, t0) = setup(true);
    let before = device.open_attempts();
    device.set_busy(1);
    assert!(bus.reopen_at(44_100, t0, &mut bus.busy_budget(true)));
    assert_eq!(bus.sample_rate(), 44_100);
    assert_eq!(bus.health(), BusHealth::Ok);
    assert_eq!(device.config().unwrap().sample_rate, 44_100);
    assert_eq!(device.open_attempts() - before, 2);
}

#[test]
fn a_device_busy_beyond_the_retries_keeps_the_rate_without_refusing_it() {
    let (_b, device, mut bus, t0) = setup(true);
    let before = device.open_attempts();
    // Busy for the first try and both retries; free again for the restore.
    device.set_busy(3);
    assert!(!bus.reopen_at(44_100, t0, &mut bus.busy_budget(true)));
    assert_eq!(device.open_attempts() - before, 4);
    assert_eq!(bus.sample_rate(), 48_000);
    assert_eq!(bus.health(), BusHealth::Ok);
    assert_eq!(device.config().unwrap().sample_rate, 48_000);
    // Busy is not a refusal of the rate: the next start asks again.
    assert!(bus.reopen_at(44_100, t0, &mut bus.busy_budget(true)));
    assert_eq!(bus.sample_rate(), 44_100);
    assert_eq!(device.config().unwrap().sample_rate, 44_100);
}

#[test]
fn a_rate_the_device_refuses_is_still_remembered() {
    let (_b, device, mut bus, t0) = setup(true);
    device.refuse_rate(44_100);
    let before = device.open_attempts();
    assert!(!bus.reopen_at(44_100, t0, &mut bus.busy_budget(true)));
    let tried = device.open_attempts() - before;
    // One try at the rate (no retries: it is not busy), one restore.
    assert_eq!(tried, 2);
    assert_eq!(bus.sample_rate(), 48_000);
    assert!(!bus.reopen_at(44_100, t0, &mut bus.busy_budget(true)));
    assert_eq!(device.open_attempts() - before, tried);
}

#[derive(Clone, Default)]
struct Lines(Arc<std::sync::Mutex<Vec<u8>>>);

impl std::io::Write for Lines {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Lines {
    fn count(&self, needle: &str) -> usize {
        String::from_utf8_lossy(&self.0.lock().unwrap())
            .lines()
            .filter(|l| l.contains(needle))
            .count()
    }
}

/// Runs `f` with this thread's log lines captured.
fn capture<T>(f: impl FnOnce() -> T) -> (T, Lines) {
    let lines = Lines::default();
    let writer = lines.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_writer(move || writer.clone())
        .with_ansi(false)
        .finish();
    let out = tracing::subscriber::with_default(subscriber, f);
    (out, lines)
}

/// A bus whose device was lost at `t0` and is plugged in again.
fn lost_and_replugged() -> (OfflineBackend, OfflineDevice, Bus, Instant) {
    let (b, device, mut bus, t0) = setup(true);
    device.unplug();
    bus.supervise(t0);
    assert_eq!(bus.health(), BusHealth::Lost);
    device.replug();
    (b, device, bus, t0)
}

#[test]
fn a_reopened_device_is_back_only_once_it_renders_its_first_block() {
    let (_b, device, mut bus, t0) = lost_and_replugged();
    let reopen = t0 + Duration::from_secs(3);
    bus.supervise(reopen);
    assert!(device.is_open(), "the watchdog reopened the device");
    assert_eq!(
        bus.health(),
        BusHealth::Lost,
        "an open stream that has not asked for audio yet is not back"
    );
    bus.supervise(reopen + Duration::from_secs(1));
    assert_eq!(bus.health(), BusHealth::Lost);
    device.render(480).unwrap();
    bus.supervise(reopen + Duration::from_millis(1010));
    assert_eq!(
        bus.health(),
        BusHealth::Ok,
        "its first block brings it back"
    );
}

#[test]
fn a_device_still_waiting_for_its_first_block_leaves_the_mixer_to_the_virtual_clock() {
    let (_b, device, mut bus, t0) = lost_and_replugged();
    bus.supervise(t0 + Duration::from_secs(3));
    let frames = bus.now_frame();
    // One second of audio asked for at once: rendering it would move the
    // timeline by 48 000 frames, far more than the virtual clock renders
    // in the moment this takes.
    let out = device.render(48_000).unwrap();
    assert!(out.iter().all(|s| *s == 0.0), "the stream plays silence");
    let advanced = bus.now_frame() - frames;
    assert!(
        advanced < 48_000,
        "the device rendered the mixer ({advanced} frames) while the virtual clock owns it"
    );
}

#[test]
fn a_reopened_device_that_never_starts_stays_lost_and_is_retried() {
    let (_b, device, mut bus, t0) = lost_and_replugged();
    let reopen = t0 + Duration::from_secs(3);
    bus.supervise(reopen);
    assert!(device.is_open());
    let attempts = device.open_attempts();
    // No block within the startup grace: closed, still lost.
    bus.supervise(reopen + Duration::from_millis(5_100));
    assert_eq!(bus.health(), BusHealth::Lost);
    assert!(!device.is_open(), "a stream that never starts is closed");
    // Retried after the reconnect interval, still not back.
    bus.supervise(reopen + Duration::from_millis(7_200));
    assert!(device.is_open());
    assert_eq!(device.open_attempts(), attempts + 1);
    assert_eq!(bus.health(), BusHealth::Lost);
    // It starts at last.
    device.render(480).unwrap();
    bus.supervise(reopen + Duration::from_millis(7_210));
    assert_eq!(bus.health(), BusHealth::Ok);
}

#[test]
fn a_device_that_keeps_opening_without_starting_is_reported_once() {
    let (_b, device, mut bus, t0) = lost_and_replugged();
    let ((), lines) = capture(|| {
        // Ten cycles: open, five seconds without a block, closed, retried.
        let mut now = t0;
        for _ in 0..10 {
            now += Duration::from_secs(2);
            bus.supervise(now);
            assert!(device.is_open());
            now += Duration::from_millis(5_100);
            bus.supervise(now);
            assert_eq!(bus.health(), BusHealth::Lost);
        }
    });
    assert_eq!(lines.count("opened but never started"), 1);
    assert_eq!(lines.count("output device back"), 0);
    assert_eq!(lines.count("output device lost"), 0);
}

#[test]
fn a_busy_restore_after_a_refused_rate_is_retried() {
    let (_b, device, mut bus, t0) = setup(true);
    device.refuse_rate(44_100);
    // The refusal does not spend the busy opens: the restore meets them.
    device.set_busy(1);
    let before = device.open_attempts();
    let mut budget = bus.busy_budget(true);
    assert!(!bus.reopen_at(44_100, t0, &mut budget));
    // The rate, the busy restore, its retry.
    assert_eq!(device.open_attempts() - before, 3);
    assert_eq!(bus.health(), BusHealth::Ok);
    assert_eq!(device.config().unwrap().sample_rate, 48_000);
}

#[test]
fn one_reopen_spends_one_busy_budget_for_the_rate_and_the_restore() {
    let (_b, device, mut bus, t0) = setup(true);
    device.set_busy(100);
    let before = device.open_attempts();
    let mut budget = bus.busy_budget(true);
    assert!(!bus.reopen_at(44_100, t0, &mut budget));
    // The rate, its two retries (TIMING.busy_retries), the restore once.
    assert_eq!(device.open_attempts() - before, 4);
    assert_eq!(bus.health(), BusHealth::Lost);
}

#[test]
fn a_sounding_bus_never_waits_for_a_busy_device() {
    let (_b, device, mut bus, t0) = setup(true);
    device.set_busy(1);
    let before = device.open_attempts();
    let mut budget = bus.busy_budget(false);
    assert!(!bus.reopen_at(44_100, t0, &mut budget));
    // The rate once, the restore once: no retries.
    assert_eq!(device.open_attempts() - before, 2);
    assert_eq!(bus.health(), BusHealth::Ok);
    assert_eq!(bus.sample_rate(), 48_000);
    // Not refused: asked again.
    assert!(bus.reopen_at(44_100, t0, &mut bus.busy_budget(false)));
}

#[test]
fn a_device_that_never_starts_is_reported_again_after_a_failed_reopen() {
    let (_b, device, mut bus, t0) = lost_and_replugged();
    let ((), lines) = capture(|| {
        let mut now = t0 + Duration::from_secs(3);
        bus.supervise(now);
        now += Duration::from_millis(5_100);
        bus.supervise(now);
        // It starts at last.
        now += Duration::from_secs(2);
        bus.supervise(now);
        device.render(480).unwrap();
        now += Duration::from_millis(10);
        bus.supervise(now);
        assert_eq!(bus.health(), BusHealth::Ok);
        // A rate change on a device gone again: lost, on the virtual clock.
        device.unplug();
        assert!(!bus.reopen_at(44_100, now, &mut bus.busy_budget(true)));
        assert_eq!(bus.health(), BusHealth::Lost);
        device.replug();
        now += Duration::from_secs(3);
        bus.supervise(now);
        assert!(device.is_open());
        now += Duration::from_millis(5_100);
        bus.supervise(now);
    });
    assert_eq!(lines.count("opened but never started"), 2);
}

#[test]
fn the_bus_reports_the_buffer_its_stream_runs_with() {
    let backend = OfflineBackend::new();
    let device = backend.add_device("card", 2);
    device.set_reported_buffers((512, 4096));
    device.set_default_buffer(2048);
    let key = BusKey {
        backend: "offline".into(),
        device: "card".into(),
    };
    let t0 = Instant::now();
    let bus = Bus::open(key, Arc::new(backend.clone()), CONFIG, 4, MIXER, TIMING, t0);
    assert_eq!(bus.config().buffer_frames, 480, "asked for");
    assert_eq!(bus.buffer_frames(), 2048, "the device's default");
}

#[test]
fn a_new_timing_applies_at_the_next_supervision() {
    let (_b, _device, mut bus, t0) = setup(true);
    bus.set_timing(BusTiming {
        startup_grace: Duration::from_millis(10),
        ..TIMING
    });
    bus.supervise(t0 + Duration::from_millis(50));
    assert_eq!(bus.health(), BusHealth::Lost, "the shorter grace applies");
}
