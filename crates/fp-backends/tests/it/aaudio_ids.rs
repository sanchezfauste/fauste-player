//! Stable ids and labels of AAudio outputs (mobile spec MB5).

use ::cpal::{DeviceType, InterfaceType};
use fp_backends::aaudio_ids::{ListedOutput, identities};

fn output(name: &str, device_type: DeviceType, interface: InterfaceType) -> ListedOutput {
    ListedOutput {
        name: name.to_owned(),
        device_type,
        interface,
    }
}

fn ids(outputs: &[ListedOutput]) -> Vec<String> {
    identities(outputs)
        .into_iter()
        .map(|(id, _)| id.0)
        .collect()
}

#[test]
fn the_speaker_and_the_earpiece_get_different_ids() {
    let ids = ids(&[
        output(
            "Pixel (Speaker)",
            DeviceType::Speaker,
            InterfaceType::BuiltIn,
        ),
        output(
            "Pixel (Earpiece)",
            DeviceType::Earpiece,
            InterfaceType::BuiltIn,
        ),
    ]);
    assert_ne!(ids[0], ids[1]);
}

#[test]
fn an_id_survives_a_reconnect() {
    let before = ids(&[
        output(
            "Pixel (Speaker)",
            DeviceType::Speaker,
            InterfaceType::BuiltIn,
        ),
        output("JBL Flip", DeviceType::Unknown, InterfaceType::Bluetooth),
    ]);
    // Reconnected: Android gives it a new id and lists it first.
    let after = ids(&[
        output("JBL Flip", DeviceType::Unknown, InterfaceType::Bluetooth),
        output(
            "Pixel (Speaker)",
            DeviceType::Speaker,
            InterfaceType::BuiltIn,
        ),
    ]);
    assert_eq!(before[1], after[0]);
    assert_eq!(before[0], after[1]);
}

#[test]
fn twins_get_an_ordinal() {
    let speaker = || {
        output(
            "Pixel (Speaker)",
            DeviceType::Speaker,
            InterfaceType::BuiltIn,
        )
    };
    let ids = ids(&[speaker(), speaker()]);
    assert_ne!(ids[0], ids[1]);
    assert_eq!(ids[1], format!("{}#2", ids[0]));
}

#[test]
fn a_bluetooth_a2dp_device_is_labelled_bluetooth() {
    let list = identities(&[output(
        "JBL Flip",
        DeviceType::Unknown,
        InterfaceType::Bluetooth,
    )]);
    assert_eq!(list[0].1, "JBL Flip (Bluetooth)");
}

#[test]
fn a_typed_device_keeps_cpals_name() {
    let list = identities(&[output(
        "Pixel (Speaker)",
        DeviceType::Speaker,
        InterfaceType::BuiltIn,
    )]);
    assert_eq!(list[0].1, "Pixel (Speaker)");
    let unknown = identities(&[output("Thing", DeviceType::Unknown, InterfaceType::Unknown)]);
    assert_eq!(unknown[0].1, "Thing");
}

#[test]
fn ids_start_with_the_aaudio_host() {
    for id in ids(&[
        output(
            "Pixel (Speaker)",
            DeviceType::Speaker,
            InterfaceType::BuiltIn,
        ),
        output("-1", DeviceType::Unknown, InterfaceType::Unknown),
    ]) {
        assert!(id.starts_with("aaudio:"), "{id}");
        assert_ne!(id, "aaudio:-1");
    }
}

#[test]
fn separators_in_names_do_not_collide() {
    let ids = ids(&[
        output("A#2", DeviceType::Speaker, InterfaceType::BuiltIn),
        output("A", DeviceType::Speaker, InterfaceType::BuiltIn),
        output("A", DeviceType::Speaker, InterfaceType::BuiltIn),
        output("x/y", DeviceType::Speaker, InterfaceType::BuiltIn),
        output("x%2Fy", DeviceType::Speaker, InterfaceType::BuiltIn),
    ]);
    let mut unique = ids.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), ids.len(), "{ids:?}");
}
