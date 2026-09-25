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
};
const MIXER: MixerConfig = MixerConfig {
    volume_smoothing_frames: 1,
    max_commands_per_block: 64,
};
const TIMING: BusTiming = BusTiming {
    watchdog_timeout: Duration::from_millis(500),
    reconnect_interval: Duration::from_secs(2),
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
fn a_silent_device_is_declared_lost_by_the_watchdog() {
    let (_b, _device, mut bus, t0) = setup(true);
    bus.supervise(t0 + Duration::from_millis(400));
    assert_eq!(bus.health(), BusHealth::Ok);
    bus.supervise(t0 + Duration::from_millis(600));
    assert_eq!(bus.health(), BusHealth::Lost);
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
