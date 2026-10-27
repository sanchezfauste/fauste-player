//! Stable ids and labels of AAudio outputs (mobile spec MB5).

use ::cpal::{DeviceType, InterfaceType};
use fp_backends::aaudio_ids::{ListedOutput, identities, is_stable};

fn output(name: &str, device_type: DeviceType, interface: InterfaceType) -> ListedOutput {
    ListedOutput {
        name: name.to_owned(),
        device_type,
        interface,
        address: None,
    }
}

fn bluetooth(name: &str, address: &str) -> ListedOutput {
    ListedOutput {
        address: Some(address.to_owned()),
        ..output(name, DeviceType::Unknown, InterfaceType::Bluetooth)
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

/// Saved routes hold these ids: a change here (or in cpal's names) orphans
/// every route saved on a phone, so it must be deliberate.
#[test]
fn ids_are_pinned() {
    assert_eq!(
        ids(&[
            output(
                "Pixel (Speaker)",
                DeviceType::Speaker,
                InterfaceType::BuiltIn,
            ),
            output("JBL Flip", DeviceType::Unknown, InterfaceType::Bluetooth),
            output("DAC", DeviceType::Headphones, InterfaceType::Usb),
            output("TV", DeviceType::HearingAid, InterfaceType::Spdif),
            bluetooth("JBL Flip", "XX:XX:XX:XX:AB:CD"),
        ]),
        [
            "aaudio:builtin/speaker/Pixel (Speaker)",
            "aaudio:bluetooth/unknown/JBL Flip",
            "aaudio:usb/headphones/DAC",
            "aaudio:spdif/hearing-aid/TV",
            "aaudio:bluetooth/unknown/JBL Flip@XX:XX:XX:XX:AB:CD",
        ]
    );
}

#[test]
fn bluetooth_twins_keep_their_ids_when_one_leaves() {
    let both = ids(&[bluetooth("JBL", "AA"), bluetooth("JBL", "BB")]);
    let only_b = ids(&[bluetooth("JBL", "BB")]);
    assert_ne!(both[0], both[1]);
    assert_eq!(both[1], only_b[0]);
}

#[test]
fn an_at_sign_in_a_name_does_not_collide_with_an_address() {
    let ids = ids(&[bluetooth("A", "B"), bluetooth("A@B", "")]);
    assert_ne!(ids[0], ids[1]);
}

#[test]
fn twins_get_readable_labels() {
    let speaker = || {
        output(
            "Pixel (Speaker)",
            DeviceType::Speaker,
            InterfaceType::BuiltIn,
        )
    };
    let list = identities(&[speaker(), speaker()]);
    assert_eq!(list[0].1, "Pixel (Speaker)");
    assert_eq!(list[1].1, "Pixel (Speaker) 2");
}

#[test]
fn only_stable_ids_are_looked_up_in_the_list() {
    for (id, _) in identities(&[output("x", DeviceType::Unknown, InterfaceType::Unknown)]) {
        assert!(is_stable(&id.0), "{}", id.0);
    }
    assert!(!is_stable("aaudio:-1"));
    assert!(!is_stable("aaudio:12"));
}
