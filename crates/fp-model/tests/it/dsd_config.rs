#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O25: the DSD settings are `Config` fields with defaults,
//! ranges and lenient loading.

use fp_model::{Config, DsdDevice, DsdMix, DsdOutput, OutputDevice};

fn device(mode: DsdOutput) -> DsdDevice {
    DsdDevice {
        backend: "alsa".into(),
        device: "hw:CARD=D,DEV=0".into(),
        mode,
    }
}

fn bit_perfect(c: &mut Config) {
    c.outputs.bit_perfect.push(OutputDevice {
        backend: "alsa".into(),
        device: "hw:CARD=D,DEV=0".into(),
    });
}

#[test]
fn defaults_convert_dsd_to_pcm() {
    let c = Config::default();
    assert!(c.outputs.dsd_output.is_empty());
    assert_eq!(c.outputs.dsd_mix, DsdMix::ConvertToPcm);
    assert_eq!(c.outputs.dsd_silence_ms, 200.0);
    assert_eq!(
        c.outputs.dsd_output_for("alsa", "hw:CARD=D,DEV=0"),
        DsdOutput::Pcm
    );
}

#[test]
fn a_mode_applies_only_to_a_bit_perfect_device() {
    let mut c = Config::default();
    c.outputs.dsd_output.push(device(DsdOutput::Dop));
    assert_eq!(
        c.outputs.dsd_output_for("alsa", "hw:CARD=D,DEV=0"),
        DsdOutput::Pcm,
        "not bit-perfect: converted"
    );
    bit_perfect(&mut c);
    assert_eq!(
        c.outputs.dsd_output_for("alsa", "hw:CARD=D,DEV=0"),
        DsdOutput::Dop
    );
    assert_eq!(
        c.outputs.dsd_output_for("alsa", "hw:CARD=E,DEV=0"),
        DsdOutput::Pcm
    );
}

#[test]
fn the_silence_time_is_clamped() {
    for (given, kept) in [
        (-5.0, 0.0),
        (f64::NAN, 0.0),
        (90_000.0, 2000.0),
        (150.0, 150.0),
    ] {
        let mut c = Config::default();
        c.outputs.dsd_silence_ms = given;
        let warnings = c.validate();
        assert_eq!(c.outputs.dsd_silence_ms, kept, "{given}");
        assert_eq!(warnings.is_empty(), given == kept, "{given}: {warnings:?}");
    }
}

#[test]
fn a_device_listed_twice_keeps_its_first_mode() {
    let mut c = Config::default();
    c.outputs.dsd_output = vec![device(DsdOutput::Native), device(DsdOutput::Dop)];
    let warnings = c.validate();
    assert_eq!(c.outputs.dsd_output, vec![device(DsdOutput::Native)]);
    assert_eq!(warnings.len(), 1, "{warnings:?}");
}

#[test]
fn the_modes_serialize_by_name() {
    let json = serde_json::to_string(&device(DsdOutput::Dop)).unwrap();
    assert!(json.contains("\"Dop\""), "{json}");
    assert_eq!(
        serde_json::to_string(&DsdMix::HoldOthers).unwrap(),
        "\"HoldOthers\""
    );
}
