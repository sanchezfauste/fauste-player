//! Stable ids and labels for the outputs of cpal's AAudio host (mobile spec
//! MB5). cpal names them `aaudio:<android id>`, and Android gives a device a
//! new id each time it connects, so a saved route would lose its Bluetooth
//! speaker after a reconnect. The id built here comes from what the device
//! is instead: its interface, its type and its product name, plus the
//! address of a Bluetooth device, or else an ordinal for outputs that share
//! all three.
//!
//! Saved routes hold these ids, so the interface and the type are written
//! with this module's own tokens, not cpal's display names, and a test pins
//! whole ids: a cpal upgrade that renames something fails it instead of
//! orphaning routes.

use std::collections::HashMap;

use cpal::{DeviceType, InterfaceType};

use crate::DeviceId;

/// One output as the AAudio host lists it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListedOutput {
    /// cpal's name: the product name, with the type in brackets when cpal
    /// knows it.
    pub name: String,
    pub device_type: DeviceType,
    pub interface: InterfaceType,
    /// The device's address (a Bluetooth device's MAC), when Android gives
    /// one.
    pub address: Option<String>,
}

/// The stable id and the label of each output, in order.
///
/// A Bluetooth device with an address carries it in its id (`@<address>`),
/// so two identical speakers keep their ids whichever of them is listed.
/// Other outputs that share an id with earlier ones get `#n` appended (n ≥
/// 2) and ` n` in their label: twins keep their ids while the list keeps its
/// order.
pub fn identities(outputs: &[ListedOutput]) -> Vec<(DeviceId, String)> {
    let mut seen: HashMap<String, usize> = HashMap::new();
    outputs
        .iter()
        .map(|o| {
            let mut base = format!(
                "aaudio:{}/{}/{}",
                interface_token(o.interface),
                type_token(o.device_type),
                escape(&o.name)
            );
            if o.interface == InterfaceType::Bluetooth
                && let Some(address) = o.address.as_deref().filter(|a| !a.is_empty())
            {
                base.push('@');
                base.push_str(&escape(address));
            }
            let count = seen.entry(base.clone()).or_default();
            *count += 1;
            match *count {
                1 => (DeviceId(base), label(o)),
                n => (DeviceId(format!("{base}#{n}")), format!("{} {n}", label(o))),
            }
        })
        .collect()
}

/// Whether `id` is one of the ids `identities` builds (they always hold a
/// `/`), as opposed to one of cpal's (`aaudio:-1`, `aaudio:<android id>`).
pub fn is_stable(id: &str) -> bool {
    id.starts_with("aaudio:") && id.contains('/')
}

/// cpal's name, plus the interface when cpal has no type for the device
/// (Bluetooth A2DP), so every label says what kind of output it is.
fn label(o: &ListedOutput) -> String {
    if o.device_type == DeviceType::Unknown && o.interface != InterfaceType::Unknown {
        format!("{} ({})", o.name, o.interface)
    } else {
        o.name.clone()
    }
}

fn interface_token(interface: InterfaceType) -> &'static str {
    match interface {
        InterfaceType::BuiltIn => "builtin",
        InterfaceType::Usb => "usb",
        InterfaceType::Bluetooth => "bluetooth",
        InterfaceType::Pci => "pci",
        InterfaceType::FireWire => "firewire",
        InterfaceType::Thunderbolt => "thunderbolt",
        InterfaceType::Hdmi => "hdmi",
        InterfaceType::Line => "line",
        InterfaceType::Spdif => "spdif",
        InterfaceType::Network => "network",
        InterfaceType::Virtual => "virtual",
        InterfaceType::DisplayPort => "displayport",
        InterfaceType::Aggregate => "aggregate",
        _ => "unknown",
    }
}

fn type_token(device_type: DeviceType) -> &'static str {
    match device_type {
        DeviceType::Speaker => "speaker",
        DeviceType::Microphone => "microphone",
        DeviceType::Headphones => "headphones",
        DeviceType::Headset => "headset",
        DeviceType::Earpiece => "earpiece",
        DeviceType::Handset => "handset",
        DeviceType::HearingAid => "hearing-aid",
        DeviceType::Dock => "dock",
        DeviceType::Tuner => "tuner",
        DeviceType::Virtual => "virtual",
        _ => "unknown",
    }
}

/// `%`, `/`, `#` and `@` percent-escaped: they separate the parts of an id.
fn escape(part: &str) -> String {
    let mut out = String::with_capacity(part.len());
    for c in part.chars() {
        match c {
            '%' => out.push_str("%25"),
            '/' => out.push_str("%2F"),
            '#' => out.push_str("%23"),
            '@' => out.push_str("%40"),
            c => out.push(c),
        }
    }
    out
}
