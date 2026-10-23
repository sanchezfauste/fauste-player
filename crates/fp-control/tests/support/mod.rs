#![allow(dead_code, clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Fake ports and a fake controller for the MIDI service tests.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use arc_swap::ArcSwap;
use crossbeam_channel::Sender;
use fp_control::service::{MidiControl, MidiCore, MidiOutput, MidiPorts, Opened};
use fp_model::{AppState, Command, Config, MidiAction, MidiTrigger, ShortcutAction, apply};

#[derive(Default)]
pub struct Wires {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub sinks: HashMap<String, Sender<(String, Vec<u8>)>>,
    pub sent: Vec<(String, Vec<u8>)>,
    pub scans: usize,
    pub connects: usize,
    /// Ports opened now, by direction ("in", "out") and name: a second
    /// open fails as a held WinMM port does, until the first is dropped.
    pub live: HashSet<(&'static str, String)>,
    /// Ports another program holds: every open fails.
    pub refuse: Vec<String>,
}

/// An open port; dropping it closes it.
struct Live(&'static str, String, Arc<Mutex<Wires>>);

impl Drop for Live {
    fn drop(&mut self) {
        if let Ok(mut w) = self.2.lock() {
            w.live.remove(&(self.0, self.1.clone()));
        }
    }
}

impl Wires {
    fn open(&mut self, direction: &'static str, name: &str) -> Result<(), String> {
        if self.refuse.iter().any(|r| r == name) {
            return Err("allocated by another program".into());
        }
        if !self.live.insert((direction, name.to_owned())) {
            return Err("allocated: already open".into());
        }
        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct FakePorts(pub Arc<Mutex<Wires>>);

struct FakeOut(String, Arc<Mutex<Wires>>, #[allow(dead_code)] Live);

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
    ) -> Result<Opened<Box<dyn Send>>, String> {
        let mut w = self.0.lock().unwrap();
        if !w.inputs.iter().any(|i| i == name) {
            return Err("gone".into());
        }
        w.open("in", name)?;
        w.sinks.insert(name.to_owned(), sink);
        w.connects += 1;
        Ok(Opened {
            connection: Box::new(Live("in", name.to_owned(), self.0.clone())),
            id: format!("fake-in:{name}"),
            namesakes: 0,
        })
    }
    fn connect_output(&mut self, name: &str) -> Result<Opened<Box<dyn MidiOutput>>, String> {
        let mut w = self.0.lock().unwrap();
        if !w.outputs.iter().any(|o| o == name) {
            return Err("gone".into());
        }
        w.open("out", name)?;
        Ok(Opened {
            connection: Box::new(FakeOut(
                name.to_owned(),
                self.0.clone(),
                Live("out", name.to_owned(), self.0.clone()),
            )),
            id: format!("fake-out:{name}"),
            namesakes: 0,
        })
    }
}

impl FakePorts {
    pub fn plug(&self, name: &str) {
        let mut w = self.0.lock().unwrap();
        w.inputs.push(name.to_owned());
        w.outputs.push(name.to_owned());
    }
    pub fn unplug(&self, name: &str) {
        let mut w = self.0.lock().unwrap();
        w.inputs.retain(|i| i != name);
        w.outputs.retain(|o| o != name);
        w.sinks.remove(name);
    }
    /// The device sends `bytes`, if it is connected.
    pub fn press(&self, name: &str, bytes: &[u8]) {
        let w = self.0.lock().unwrap();
        if let Some(s) = w.sinks.get(name) {
            s.send((name.to_owned(), bytes.to_vec())).unwrap();
        }
    }
    pub fn take_sent(&self) -> Vec<(String, Vec<u8>)> {
        std::mem::take(&mut self.0.lock().unwrap().sent)
    }
}

pub struct FakeControl {
    pub model: ArcSwap<AppState>,
    pub sent: Mutex<Vec<Command>>,
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

pub fn control(enabled: bool) -> Arc<FakeControl> {
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

pub fn core(
    ports: &FakePorts,
    control: &Arc<FakeControl>,
) -> (MidiCore, fp_control::service::MidiHandle) {
    MidiCore::new(Box::new(ports.clone()), control.clone())
}
