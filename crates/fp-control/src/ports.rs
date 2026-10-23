//! The system's MIDI ports through `midir` (ALSA, CoreMIDI, WinMM).

use crossbeam_channel::Sender;

use crate::service::{MidiOutput, MidiPorts};

/// The client name other MIDI software sees.
pub const CLIENT: &str = "Fauste Player";

/// A port's name without the numbers ALSA appends (`"APC MINI:APC MINI
/// MIDI 1 20:0"` → `"APC MINI:APC MINI MIDI 1"`): they follow enumeration
/// order, so a binding would stop matching after a reboot or a replug.
pub fn stable_name(raw: &str) -> String {
    if let Some((head, tail)) = raw.rsplit_once(' ')
        && let Some((client, port)) = tail.split_once(':')
        && !client.is_empty()
        && !port.is_empty()
        && client.chars().all(|c| c.is_ascii_digit())
        && port.chars().all(|c| c.is_ascii_digit())
    {
        return head.to_owned();
    }
    raw.to_owned()
}

/// Ports as the system lists them. Each connection opens its own client,
/// as `midir` requires.
#[derive(Debug, Default)]
pub struct MidirPorts;

impl MidiPorts for MidirPorts {
    fn inputs(&mut self) -> Vec<String> {
        match midir::MidiInput::new(CLIENT) {
            Ok(input) => input
                .ports()
                .iter()
                .filter_map(|p| input.port_name(p).ok().map(|n| stable_name(&n)))
                .collect(),
            Err(error) => {
                tracing::debug!(%error, "no MIDI input");
                Vec::new()
            }
        }
    }

    fn outputs(&mut self) -> Vec<String> {
        match midir::MidiOutput::new(CLIENT) {
            Ok(output) => output
                .ports()
                .iter()
                .filter_map(|p| output.port_name(p).ok().map(|n| stable_name(&n)))
                .collect(),
            Err(error) => {
                tracing::debug!(%error, "no MIDI output");
                Vec::new()
            }
        }
    }

    fn connect_input(
        &mut self,
        name: &str,
        sink: Sender<(String, Vec<u8>)>,
    ) -> Result<Box<dyn Send>, String> {
        let input = midir::MidiInput::new(CLIENT).map_err(|e| e.to_string())?;
        // The first port of that name, so a replugged device is found again.
        let mut named: Vec<midir::MidiInputPort> = input
            .ports()
            .into_iter()
            .filter(|p| input.port_name(p).is_ok_and(|n| stable_name(&n) == name))
            .collect();
        let ids: Vec<String> = named.iter().map(midir::MidiInputPort::id).collect();
        log_opening("input", name, &ids);
        if named.is_empty() {
            return Err(format!("{name} is gone"));
        }
        let port = named.swap_remove(0);
        let owned = name.to_owned();
        let connection = input
            .connect(
                &port,
                "fauste-player-in",
                move |_stamp, bytes, _| {
                    // Never blocks the driver's thread: the channel is unbounded.
                    let _ = sink.send((owned.clone(), bytes.to_vec()));
                },
                (),
            )
            .map_err(|e| e.to_string())?;
        Ok(Box::new(connection))
    }

    fn connect_output(&mut self, name: &str) -> Result<Box<dyn MidiOutput>, String> {
        let output = midir::MidiOutput::new(CLIENT).map_err(|e| e.to_string())?;
        let mut named: Vec<midir::MidiOutputPort> = output
            .ports()
            .into_iter()
            .filter(|p| output.port_name(p).is_ok_and(|n| stable_name(&n) == name))
            .collect();
        let ids: Vec<String> = named.iter().map(midir::MidiOutputPort::id).collect();
        log_opening("output", name, &ids);
        if named.is_empty() {
            return Err(format!("{name} is gone"));
        }
        let port = named.swap_remove(0);
        let connection = output
            .connect(&port, "fauste-player-out")
            .map_err(|e| e.to_string())?;
        Ok(Box::new(Output(connection)))
    }
}

/// Which system port a name opens (the first of `ids`), so a field log
/// shows whether the same device is opened each time and whether another
/// port shares its name.
fn log_opening(direction: &str, name: &str, ids: &[String]) {
    match ids {
        [] => {}
        [id] => tracing::info!(direction, port = %name, %id, "opening a MIDI port"),
        [id, ..] => tracing::warn!(
            direction,
            port = %name,
            %id,
            ?ids,
            "several MIDI ports share this name; opening the first"
        ),
    }
}

struct Output(midir::MidiOutputConnection);

impl MidiOutput for Output {
    fn send(&mut self, bytes: &[u8]) -> Result<(), String> {
        self.0.send(bytes).map_err(|e| e.to_string())
    }
}
