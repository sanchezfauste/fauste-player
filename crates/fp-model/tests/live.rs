#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Live settings spec (2026-10-07): one test per rule L1–L10, L20,
//! L22–L24. The engine is stood in for by the events it reports.

mod common;

use fp_model::{
    AppState, Config, DeviceSettings, DsdDevice, DsdMix, DsdOutput, LiveSettings, OutputDevice,
    Route, device_settings,
};

fn dev(name: &str) -> OutputDevice {
    OutputDevice {
        backend: "null".into(),
        device: name.into(),
    }
}

fn route_to(name: &str) -> Route {
    Route {
        backend: "null".into(),
        device: name.into(),
        first_channel: 0,
    }
}

#[test]
fn the_live_state_starts_empty_and_unknown() {
    let s = AppState::new(Config::default(), "Main");
    assert_eq!(s.live, LiveSettings::default());
    assert_eq!(s.live.audio_system, None, "unknown until the engine says");
}

#[test]
fn l4_device_settings_follow_the_effective_rate_buffer_and_bit_perfect() {
    let mut c = Config::default();
    c.outputs.routes = vec![fp_model::PlayerRoutes {
        player: fp_model::PlayerId(1),
        main: Some(route_to("dac")),
        cue: None,
    }];
    c.outputs.set_device_rate(&dev("dac"), Some(96_000));
    c.outputs.set_device_buffer(&dev("dac"), Some(256));
    c.outputs.bit_perfect = vec![dev("dac")];
    assert_eq!(
        device_settings(&c.outputs, &dev("dac")),
        DeviceSettings {
            sample_rate: 96_000,
            buffer_frames: 256,
            bit_perfect: true,
            dsd: DsdOutput::Pcm,
            dsd_mix: None,
            dsd_silence_ms: None,
        }
    );
    // An unrouted device opens at the global values.
    let other = device_settings(&c.outputs, &dev("other"));
    assert_eq!((other.sample_rate, other.buffer_frames), (48_000, 512));
    assert!(!other.bit_perfect);
}

#[test]
fn l4_dsd_mix_and_silence_count_only_on_a_device_that_carries_dsd() {
    let mut c = Config::default();
    c.outputs.routes = vec![fp_model::PlayerRoutes {
        player: fp_model::PlayerId(1),
        main: Some(route_to("dac")),
        cue: Some(route_to("phones")),
    }];
    c.outputs.bit_perfect = vec![dev("dac")];
    c.outputs.dsd_output = vec![DsdDevice {
        backend: "null".into(),
        device: "dac".into(),
        mode: DsdOutput::Dop,
    }];
    let before_dac = device_settings(&c.outputs, &dev("dac"));
    let before_phones = device_settings(&c.outputs, &dev("phones"));
    c.outputs.dsd_mix = DsdMix::HoldOthers;
    c.outputs.dsd_silence_ms = 400.0;
    let dac = device_settings(&c.outputs, &dev("dac"));
    assert_eq!(dac.dsd, DsdOutput::Dop);
    assert_eq!(dac.dsd_mix, Some(DsdMix::HoldOthers));
    assert_eq!(dac.dsd_silence_ms, Some(400.0));
    assert_ne!(dac, before_dac);
    assert_eq!(
        device_settings(&c.outputs, &dev("phones")),
        before_phones,
        "a PCM device does not change"
    );
    // A DSD mode on a device that is not bit-perfect is PCM.
    c.outputs.bit_perfect.clear();
    assert_eq!(device_settings(&c.outputs, &dev("dac")).dsd, DsdOutput::Pcm);
}
