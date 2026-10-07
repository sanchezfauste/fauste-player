//! Settings → Audio outputs, Advanced view (operator feedback 4, Q12, and
//! Phase 4 spec B6): each routed device's bit-perfect switch and DSD mode,
//! then the DSD settings every device shares.

use egui::Ui;
use fp_backends::DeviceInfo;
use fp_model::{DsdCaps, DsdDevice, DsdMix, DsdNotOffered, DsdOutput, OutputDevice};

use super::{
    BUFFER_SIZES, BackendChoice, SAMPLE_RATES, caption, labelled_row, note, row, slider, toggle,
    update,
};
use crate::ui::app::Scene;
use crate::ui::theme;

/// The device as its backend lists it, and the label the pickers give it
/// (its id when it is not listed: unplugged, or another system).
fn find<'a>(
    backends: &'a [BackendChoice],
    device: &OutputDevice,
) -> (Option<&'a DeviceInfo>, String) {
    let list = backends
        .iter()
        .find(|b| b.id == device.backend)
        .map(|b| b.devices.as_slice())
        .unwrap_or_default();
    let labels = fp_backends::device_labels(list);
    list.iter()
        .zip(labels)
        .find(|(d, _)| d.id.0 == device.device)
        .map_or((None, device.device.clone()), |(d, label)| (Some(d), label))
}

pub(super) fn section(ui: &mut Ui, scene: &Scene<'_>, backends: &[BackendChoice]) {
    let t = scene.i18n;
    ui.add_space(12.0);
    caption(ui, &t.tr("settings-devices"));
    note(ui, &t.tr("settings-bit-perfect-hint"), theme::NEUTRAL_400);
    ui.add_space(4.0);
    let devices = scene.state.config.outputs.routed_devices();
    if devices.is_empty() {
        note(ui, &t.tr("settings-devices-none"), theme::NEUTRAL_500);
        return;
    }
    for device in &devices {
        let (info, name) = find(backends, device);
        device_rows(ui, scene, device, info, &name);
    }
    dsd_settings(ui, scene);
}

/// One routed device: its bit-perfect switch, its own rate and buffer, and
/// its DSD mode with why the other modes are not offered (Q3).
fn device_rows(
    ui: &mut Ui,
    scene: &Scene<'_>,
    device: &OutputDevice,
    info: Option<&DeviceInfo>,
    name: &str,
) {
    let t = scene.i18n;
    let outputs = &scene.state.config.outputs;
    let listed = outputs.bit_perfect.contains(device);
    let capable = info.is_some_and(|d| d.exclusive_capable);

    // Bit-perfect (Phase 4 spec B6). A listed device can always be turned
    // off, even when it is not plugged in or cannot be exclusive any more.
    let mut on = listed;
    let enabled = capable || on;
    let label = t.tr_args("settings-bit-perfect-device", &[("device", name.into())]);
    row(ui, name, None, |ui| {
        let response = ui
            .add_enabled_ui(enabled, |ui| toggle(ui, &mut on, &label))
            .response;
        if !enabled {
            response.on_disabled_hover_text(t.tr("bp-not-capable"));
            return;
        }
        if on != listed {
            let device = device.clone();
            update(scene, move |c| {
                c.outputs.bit_perfect.retain(|d| d != &device);
                if on {
                    c.outputs.bit_perfect.push(device);
                }
            });
        }
    });

    // Its own rate (Q12.3).
    let own = outputs.device_override(device);
    let own_rate = own.and_then(|o| o.sample_rate);
    let own_buffer = own.and_then(|o| o.buffer_frames);
    let hz = |r: u32| t.tr_args("unit-hz", &[("value", r.into())]);
    let global_rate = t.tr_args(
        "settings-device-global-rate",
        &[("value", outputs.sample_rate.into())],
    );
    let reported = info.map_or(&[][..], |d| d.sample_rates.as_slice());
    let rates = fp_model::offered_rates(reported, &SAMPLE_RATES, own_rate);
    let rate_label = t.tr_args("settings-device-rate", &[("device", name.into())]);
    labelled_row(ui, &rate_label, None, |ui, label| {
        let shown = own_rate.map_or_else(|| global_rate.clone(), hz);
        let id = ("device-rate", &device.backend, &device.device);
        let response = egui::ComboBox::from_id_salt(id)
            .selected_text(shown)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(own_rate.is_none(), &global_rate)
                    .clicked()
                {
                    set_rate(scene, device, None);
                }
                for r in rates {
                    if ui.selectable_label(own_rate == Some(r), hz(r)).clicked() {
                        set_rate(scene, device, Some(r));
                    }
                }
            })
            .response;
        response.labelled_by(label);
    });

    // Its own buffer, with the latency at the rate it runs.
    let rate = outputs.rate_for(&device.backend, &device.device);
    let buffer = outputs.buffer_for(&device.backend, &device.device);
    let latency_ms = f64::from(buffer) / f64::from(rate.max(1)) * 1000.0;
    let latency = t.tr_args(
        "settings-latency",
        &[("ms", format!("{latency_ms:.1}").into())],
    );
    let global_buffer = t.tr_args(
        "settings-device-global-buffer",
        &[("value", outputs.buffer_frames.into())],
    );
    let reported = info.and_then(|d| d.buffer_frames);
    let buffers = fp_model::offered_buffers(reported, &BUFFER_SIZES, own_buffer);
    let buffer_label = t.tr_args("settings-device-buffer", &[("device", name.into())]);
    labelled_row(ui, &buffer_label, Some(&latency), |ui, label| {
        let shown = own_buffer.map_or_else(|| global_buffer.clone(), |b| b.to_string());
        let id = ("device-buffer", &device.backend, &device.device);
        let response = egui::ComboBox::from_id_salt(id)
            .selected_text(shown)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(own_buffer.is_none(), &global_buffer)
                    .clicked()
                {
                    set_buffer(scene, device, None);
                }
                for b in buffers {
                    if ui
                        .selectable_label(own_buffer == Some(b), b.to_string())
                        .clicked()
                    {
                        set_buffer(scene, device, Some(b));
                    }
                }
            })
            .response;
        response.labelled_by(label);
    });

    // Its DSD mode, on every routed device (Q3.1), with the default (Q3.3)
    // and why a mode is missing (Q3.2).
    let caps = DsdCaps {
        bit_perfect: listed,
        exclusive_capable: capable,
        native_dsd: info.is_some_and(|d| d.native_dsd),
        linux: std::env::consts::OS == "linux",
    };
    let current = outputs.dsd_output_for(&device.backend, &device.device);
    let modes = fp_model::offered_dsd_modes(caps, current);
    let default_hint = t.tr("settings-dsd-default-pcm");
    let dsd_label = t.tr_args("settings-dsd-mode", &[("device", name.into())]);
    labelled_row(ui, &dsd_label, Some(&default_hint), |ui, label| {
        ui.add_enabled_ui(modes.len() > 1, |ui| {
            let id = ("dsd-mode", &device.backend, &device.device);
            let response = egui::ComboBox::from_id_salt(id)
                .selected_text(dsd_mode_label(t, current))
                .show_ui(ui, |ui| {
                    for mode in modes {
                        if ui
                            .selectable_label(mode == current, dsd_mode_label(t, mode))
                            .clicked()
                        {
                            set_dsd_mode(scene, device, mode);
                        }
                    }
                })
                .response;
            response.labelled_by(label);
        });
    });
    if let Some(why) = fp_model::dsd_not_offered(caps) {
        note(ui, &t.tr(why_key(why)), theme::NEUTRAL_400);
        ui.add_space(4.0);
    }
}

fn why_key(why: DsdNotOffered) -> &'static str {
    match why {
        DsdNotOffered::NotExclusive => "settings-dsd-why-not-exclusive",
        DsdNotOffered::BitPerfectOff => "settings-dsd-why-bit-perfect-off",
        DsdNotOffered::NativeNeedsLinux => "settings-dsd-why-native-linux",
        DsdNotOffered::NoNativeDsd => "settings-dsd-why-no-native",
    }
}

fn set_rate(scene: &Scene<'_>, device: &OutputDevice, rate: Option<u32>) {
    let device = device.clone();
    update(scene, move |c| c.outputs.set_device_rate(&device, rate));
}

fn set_buffer(scene: &Scene<'_>, device: &OutputDevice, frames: Option<u32>) {
    let device = device.clone();
    update(scene, move |c| c.outputs.set_device_buffer(&device, frames));
}

fn set_dsd_mode(scene: &Scene<'_>, device: &OutputDevice, mode: DsdOutput) {
    let device = device.clone();
    update(scene, move |c| {
        c.outputs.dsd_output.retain(|d| {
            (d.backend.as_str(), d.device.as_str())
                != (device.backend.as_str(), device.device.as_str())
        });
        if mode != DsdOutput::Pcm {
            c.outputs.dsd_output.push(DsdDevice {
                backend: device.backend,
                device: device.device,
                mode,
            });
        }
    });
}

/// The DSD settings every device shares.
fn dsd_settings(ui: &mut Ui, scene: &Scene<'_>) {
    let t = scene.i18n;
    ui.add_space(4.0);
    note(ui, &t.tr("settings-dsd-hint"), theme::NEUTRAL_400);
    let mix = scene.state.config.outputs.dsd_mix;
    let mix_label = |m: DsdMix| {
        t.tr(match m {
            DsdMix::ConvertToPcm => "settings-dsd-mix-convert",
            DsdMix::HoldOthers => "settings-dsd-mix-hold",
        })
    };
    row(ui, &t.tr("settings-dsd-mix"), None, |ui| {
        egui::ComboBox::from_id_salt("dsd-mix")
            .selected_text(mix_label(mix))
            .show_ui(ui, |ui| {
                for choice in [DsdMix::ConvertToPcm, DsdMix::HoldOthers] {
                    if ui
                        .selectable_label(choice == mix, mix_label(choice))
                        .clicked()
                    {
                        update(scene, |c| c.outputs.dsd_mix = choice);
                    }
                }
            });
    });
    let mut silence = scene.state.config.outputs.dsd_silence_ms;
    let label = t.tr("settings-dsd-silence");
    row(ui, &label, Some(&t.tr("settings-dsd-silence-hint")), |ui| {
        if slider(ui, &mut silence, 0.0..=2000.0, 10.0, " ms", &label) {
            update(scene, |c| c.outputs.dsd_silence_ms = silence);
        }
    });
}

fn dsd_mode_label(t: &crate::i18n::I18n, mode: DsdOutput) -> String {
    t.tr(match mode {
        DsdOutput::Pcm => "settings-dsd-pcm",
        DsdOutput::Dop => "settings-dsd-dop",
        DsdOutput::Native => "settings-dsd-native",
    })
}
