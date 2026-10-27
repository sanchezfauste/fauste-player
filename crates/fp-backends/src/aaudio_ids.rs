//! Stable ids and labels for the outputs of cpal's AAudio host (mobile spec
//! MB5). cpal names them `aaudio:<android id>`, and Android gives a device a
//! new id each time it connects, so a saved route would lose its Bluetooth
//! speaker after a reconnect. The id built here comes from what the device
//! is instead: its interface, its type and its product name, with an
//! ordinal for outputs that share all three.

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
}

/// The stable id and the label of each output, in order. The n-th output
/// (n ≥ 2) sharing an id with earlier ones gets `#n` appended, so twins keep
/// their ids while the list keeps its order.
pub fn identities(outputs: &[ListedOutput]) -> Vec<(DeviceId, String)> {
    let mut seen: HashMap<String, usize> = HashMap::new();
    outputs
        .iter()
        .map(|o| {
            let base = format!(
                "aaudio:{}/{}/{}",
                escape(&o.interface.to_string()),
                escape(&o.device_type.to_string()),
                escape(&o.name)
            );
            let count = seen.entry(base.clone()).or_default();
            *count += 1;
            let id = match *count {
                1 => base,
                n => format!("{base}#{n}"),
            };
            (DeviceId(id), label(o))
        })
        .collect()
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

/// `%`, `/` and `#` percent-escaped: they separate the parts of an id.
fn escape(part: &str) -> String {
    let mut out = String::with_capacity(part.len());
    for c in part.chars() {
        match c {
            '%' => out.push_str("%25"),
            '/' => out.push_str("%2F"),
            '#' => out.push_str("%23"),
            c => out.push(c),
        }
    }
    out
}
