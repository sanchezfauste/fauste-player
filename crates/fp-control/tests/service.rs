#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback spec §6: ports, hot-plug, routing, feedback and learn, driven
//! with fake ports and a fake controller.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use arc_swap::ArcSwap;
use crossbeam_channel::Sender;
use fp_control::service::{MidiControl, MidiCore, MidiOutput, MidiPorts, MidiRequest};
use fp_model::{AppState, Command, Config, MidiAction, MidiTrigger, ShortcutAction, apply};

#[derive(Default)]
struct Wires {
    inputs: Vec<String>,
    outputs: Vec<String>,
    sinks: HashMap<String, Sender<(String, Vec<u8>)>>,
    sent: Vec<(String, Vec<u8>)>,
    scans: usize,
}

#[derive(Clone, Default)]
struct FakePorts(Arc<Mutex<Wires>>);

struct FakeOut(String, Arc<Mutex<Wires>>);

impl MidiOutput for FakeOut {
    fn send(&mut self, bytes: &[u8]) -> Result<(), String> {
        self.1
            .lock()
            .unwrap()
            .sent
            .push((self.0.clone(), bytes.to_vec()));
        Ok(())
    }
}

impl MidiPorts for FakePorts {
    fn inputs(&mut self) -> Vec<String> {
        let mut w = self.0.lock().unwrap();
        w.scans += 1;
        w.inputs.clone()
    }
    fn outputs(&mut self) -> Vec<String> {
        self.0.lock().unwrap().outputs.clone()
    }
    fn connect_input(
        &mut self,
        name: &str,
        sink: Sender<(String, Vec<u8>)>,
    ) -> Result<Box<dyn Send>, String> {
        let mut w = self.0.lock().unwrap();
        if !w.inputs.iter().any(|i| i == name) {
            return Err("gone".into());
        }
        w.sinks.insert(name.to_owned(), sink);
        Ok(Box::new(()))
    }
    fn connect_output(&mut self, name: &str) -> Result<Box<dyn MidiOutput>, String> {
        if !self.0.lock().unwrap().outputs.iter().any(|o| o == name) {
            return Err("gone".into());
        }
        Ok(Box::new(FakeOut(name.to_owned(), self.0.clone())))
    }
}

impl FakePorts {
    fn plug(&self, name: &str) {
        let mut w = self.0.lock().unwrap();
        w.inputs.push(name.to_owned());
        w.outputs.push(name.to_owned());
    }
    fn unplug(&self, name: &str) {
        let mut w = self.0.lock().unwrap();
        w.inputs.retain(|i| i != name);
        w.outputs.retain(|o| o != name);
        w.sinks.remove(name);
    }
    /// The device sends `bytes`, if it is connected.
    fn press(&self, name: &str, bytes: &[u8]) {
        let w = self.0.lock().unwrap();
        if let Some(s) = w.sinks.get(name) {
            s.send((name.to_owned(), bytes.to_vec())).unwrap();
        }
    }
    fn take_sent(&self) -> Vec<(String, Vec<u8>)> {
        std::mem::take(&mut self.0.lock().unwrap().sent)
    }
}

struct FakeControl {
    model: ArcSwap<AppState>,
    sent: Mutex<Vec<Command>>,
}

impl MidiControl for FakeControl {
    fn model(&self) -> Arc<AppState> {
        self.model.load_full()
    }
    fn send(&self, command: Command) -> bool {
        self.sent.lock().unwrap().push(command);
        true
    }
}

fn control(enabled: bool) -> Arc<FakeControl> {
    let mut s = AppState::new(Config::default(), "Main");
    let playlist = s.playlists.first_id().unwrap();
    let paths = (1..=3)
        .map(|n| PathBuf::from(format!("/m/{n}.mp3")))
        .collect();
    apply(
        &mut s,
        Command::InsertPaths {
            playlist,
            index: 0,
            paths,
        },
    )
    .unwrap();
    s.config.midi.enabled = enabled;
    s.config.midi.bind(
        MidiAction::Button(ShortcutAction::PlayPlayer(1)),
        "APC",
        MidiTrigger::Note {
            channel: 0,
            note: 36,
        },
    );
    Arc::new(FakeControl {
        model: ArcSwap::from_pointee(s),
        sent: Mutex::new(Vec::new()),
    })
}

fn core(
    ports: &FakePorts,
    control: &Arc<FakeControl>,
) -> (MidiCore, fp_control::service::MidiHandle) {
    MidiCore::new(Box::new(ports.clone()), control.clone())
}

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
