#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback spec §6: ports, hot-plug, routing, feedback and learn, driven
//! with fake ports and a fake controller.

use crate::support;

use std::sync::Arc;
use std::time::{Duration, Instant};

use fp_control::service::{MidiControl, MidiRequest};
use fp_model::{Command, MidiAction, MidiTrigger, ShortcutAction};
use support::{FakePorts, control, core};

#[test]
fn a_bound_note_becomes_a_command() {
    let ports = FakePorts::default();
    ports.plug("APC");
    let c = control(true);
    let (mut core, handle) = core(&ports, &c);
    let t = Instant::now();
    core.step(t);
    assert_eq!(handle.status.load().inputs, vec![("APC".to_owned(), true)]);
    ports.press("APC", &[0x90, 36, 100]);
    core.step(t);
    let p = c.model().players[0].id;
    assert_eq!(*c.sent.lock().unwrap(), vec![Command::Play(p)]);
}

#[test]
fn disabled_midi_connects_nothing() {
    let ports = FakePorts::default();
    ports.plug("APC");
    let c = control(false);
    let (mut core, handle) = core(&ports, &c);
    core.step(Instant::now());
    assert_eq!(handle.status.load().inputs, vec![], "no scan while off");
    assert_eq!(ports.0.lock().unwrap().scans, 0);
    ports.press("APC", &[0x90, 36, 100]);
    core.step(Instant::now());
    assert!(c.sent.lock().unwrap().is_empty());
}

#[test]
fn an_unplugged_device_is_reconnected_by_name_and_its_leds_refreshed() {
    let ports = FakePorts::default();
    ports.plug("APC");
    let c = control(true);
    let (mut core, handle) = core(&ports, &c);
    let mut t = Instant::now();
    core.step(t);
    assert!(
        !ports.take_sent().is_empty(),
        "the LEDs are set at connection"
    );
    ports.unplug("APC");
    t += Duration::from_millis(2100);
    core.step(t);
    assert_eq!(handle.status.load().inputs, vec![]);
    ports.plug("APC");
    t += Duration::from_millis(2100);
    core.step(t);
    assert_eq!(handle.status.load().inputs, vec![("APC".to_owned(), true)]);
    assert!(!ports.take_sent().is_empty(), "the LEDs are refreshed");
    ports.press("APC", &[0x90, 36, 100]);
    core.step(t);
    assert_eq!(c.sent.lock().unwrap().len(), 1);
}

#[test]
fn learn_binds_the_next_matching_message_instead_of_acting_on_it() {
    let ports = FakePorts::default();
    ports.plug("APC");
    let c = control(true);
    let (mut core, handle) = core(&ports, &c);
    let t = Instant::now();
    core.step(t);
    let stop = MidiAction::Button(ShortcutAction::StopPlayer(1));
    handle.requests.send(MidiRequest::Learn(stop)).unwrap();
    core.step(t);
    ports.press("APC", &[0x80, 36, 0]); // a release: not taken
    ports.press("APC", &[0x90, 36, 100]);
    core.step(t);
    let learned = handle.learned.try_recv().unwrap();
    assert_eq!(
        (learned.action, learned.device.as_str(), learned.trigger),
        (
            stop,
            "APC",
            MidiTrigger::Note {
                channel: 0,
                note: 36
            }
        )
    );
    assert!(
        c.sent.lock().unwrap().is_empty(),
        "the press was for learning"
    );
    // Learning ends: the next press acts again.
    ports.press("APC", &[0x90, 36, 100]);
    core.step(t);
    assert_eq!(c.sent.lock().unwrap().len(), 1);
}

#[test]
fn a_cancelled_or_replaced_learn_binds_once() {
    let ports = FakePorts::default();
    ports.plug("APC");
    let c = control(true);
    let (mut core, handle) = core(&ports, &c);
    let t = Instant::now();
    core.step(t);
    handle
        .requests
        .send(MidiRequest::Learn(MidiAction::Volume(1)))
        .unwrap();
    handle
        .requests
        .send(MidiRequest::Learn(MidiAction::Button(
            ShortcutAction::StopPlayer(1),
        )))
        .unwrap();
    core.step(t);
    ports.press("APC", &[0x90, 40, 100]);
    core.step(t);
    let learned = handle.learned.try_recv().unwrap();
    assert_eq!(
        learned.action,
        MidiAction::Button(ShortcutAction::StopPlayer(1)),
        "the last request wins"
    );
    assert!(handle.learned.try_recv().is_err());
    handle
        .requests
        .send(MidiRequest::Learn(MidiAction::Volume(1)))
        .unwrap();
    handle.requests.send(MidiRequest::CancelLearn).unwrap();
    core.step(t);
    ports.press("APC", &[0xB0, 7, 10]);
    core.step(t);
    assert!(handle.learned.try_recv().is_err());
}

#[test]
fn the_service_thread_never_loses_a_learn_request() {
    let ports = FakePorts::default();
    ports.plug("APC");
    let c = control(true);
    let service = fp_control::service::spawn(Box::new(ports.clone()), c.clone()).unwrap();
    let handle = service.handle();
    // Wait for the connection.
    let deadline = Instant::now() + Duration::from_secs(5);
    while handle.status.load().inputs != vec![("APC".to_owned(), true)] {
        assert!(Instant::now() < deadline, "not connected");
        std::thread::sleep(Duration::from_millis(5));
    }
    for _ in 0..20 {
        let stop = MidiAction::Button(ShortcutAction::StopPlayer(1));
        handle.requests.send(MidiRequest::Learn(stop)).unwrap();
        std::thread::sleep(Duration::from_millis(30));
        ports.press("APC", &[0x90, 36, 100]);
        let learned = handle
            .learned
            .recv_timeout(Duration::from_secs(2))
            .expect("the learn request was lost");
        assert_eq!(learned.action, stop);
    }
}

#[test]
fn shutting_the_service_down_ends_its_thread_and_releases_the_control() {
    let ports = FakePorts::default();
    ports.plug("APC");
    let c = control(true);
    let service = fp_control::service::spawn(Box::new(ports.clone()), c.clone()).unwrap();
    let handle = service.handle();
    let deadline = Instant::now() + Duration::from_secs(5);
    while handle.status.load().inputs != vec![("APC".to_owned(), true)] {
        assert!(Instant::now() < deadline, "not connected");
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(Arc::strong_count(&c) > 1, "the service holds the control");
    service.shutdown();
    // Joined: the thread's state is gone, with its share of the control
    // (the conductor in the application) and its input connections.
    assert_eq!(Arc::strong_count(&c), 1);
    let sink = ports.0.lock().unwrap().sinks.get("APC").cloned().unwrap();
    assert!(sink.send(("APC".to_owned(), vec![0x90, 36, 100])).is_err());
    // The interface's side outlives it harmlessly.
    assert!(handle.requests.send(MidiRequest::CancelLearn).is_err());
}

#[test]
fn dropping_the_service_stops_it_too() {
    let ports = FakePorts::default();
    let c = control(true);
    let service = fp_control::service::spawn(Box::new(ports.clone()), c.clone()).unwrap();
    drop(service);
    assert_eq!(Arc::strong_count(&c), 1);
}

#[test]
fn its_own_ports_are_never_connected() {
    let ports = FakePorts::default();
    ports.plug("APC");
    ports.plug("Fauste Player:fauste-player-out");
    let c = control(true);
    let (mut core, handle) = core(&ports, &c);
    core.step(Instant::now());
    let names: Vec<String> = handle
        .status
        .load()
        .inputs
        .iter()
        .map(|i| i.0.clone())
        .collect();
    assert_eq!(names, vec!["APC".to_owned()]);
    assert!(
        !ports
            .0
            .lock()
            .unwrap()
            .sinks
            .contains_key("Fauste Player:fauste-player-out")
    );
}

#[test]
fn only_bound_devices_are_opened_except_while_learning() {
    let ports = FakePorts::default();
    ports.plug("APC");
    ports.plug("Synth");
    let c = control(true);
    let (mut core, handle) = core(&ports, &c);
    let mut t = Instant::now();
    core.step(t);
    assert_eq!(
        handle.status.load().inputs,
        vec![("APC".to_owned(), true), ("Synth".to_owned(), false)],
        "a device with no binding is left alone"
    );
    handle
        .requests
        .send(MidiRequest::Learn(MidiAction::Volume(1)))
        .unwrap();
    t += Duration::from_millis(10);
    core.step(t);
    assert_eq!(
        handle.status.load().inputs[1],
        ("Synth".to_owned(), true),
        "learning listens to every input"
    );
}

#[test]
fn reconnect_reopens_every_port_and_refreshes_the_leds() {
    let ports = FakePorts::default();
    ports.plug("APC");
    let c = control(true);
    let (mut core, handle) = core(&ports, &c);
    let t = Instant::now();
    core.step(t);
    assert_eq!(ports.0.lock().unwrap().connects, 1);
    ports.take_sent();
    handle.requests.send(MidiRequest::Reconnect).unwrap();
    core.step(t + Duration::from_millis(10));
    assert_eq!(
        ports.0.lock().unwrap().connects,
        2,
        "opened again at once, without waiting for the rescan"
    );
    assert!(!ports.take_sent().is_empty(), "the LEDs are refreshed");
    assert_eq!(handle.status.load().inputs, vec![("APC".to_owned(), true)]);
    ports.press("APC", &[0x90, 36, 100]);
    core.step(t + Duration::from_millis(20));
    assert_eq!(c.sent.lock().unwrap().len(), 1);
}

#[test]
fn messages_queued_from_a_closed_connection_do_nothing() {
    let ports = FakePorts::default();
    ports.plug("APC");
    let c = control(true);
    let (mut core, handle) = core(&ports, &c);
    let mut t = Instant::now();
    core.step(t);
    // Queued, then the ports are reopened before the service reads it.
    ports.press("APC", &[0x90, 36, 100]);
    handle.requests.send(MidiRequest::Reconnect).unwrap();
    core.step(t);
    assert!(c.sent.lock().unwrap().is_empty(), "stale after Reconnect");
    // Queued, then the port goes away.
    ports.press("APC", &[0x90, 36, 100]);
    ports.unplug("APC");
    t += Duration::from_millis(2100);
    core.step(t);
    assert!(c.sent.lock().unwrap().is_empty(), "stale after a drop");
}

#[test]
fn a_port_is_never_opened_twice() {
    let ports = FakePorts::default();
    ports.plug("APC");
    let c = control(true);
    let (mut core, handle) = core(&ports, &c);
    let t = Instant::now();
    core.step(t);
    handle.requests.send(MidiRequest::Reconnect).unwrap();
    core.step(t);
    // The fake refuses a second open of a live port: the old one was
    // closed before the new one opened.
    assert_eq!(handle.status.load().inputs, vec![("APC".to_owned(), true)]);
    assert_eq!(
        ports.0.lock().unwrap().live.len(),
        2,
        "one input, one output"
    );
}
