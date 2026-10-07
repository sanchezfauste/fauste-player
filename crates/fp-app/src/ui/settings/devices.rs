//! Settings → Audio outputs, Advanced view (operator feedback 4, Q12, and
//! Phase 4 spec B6): each routed device's bit-perfect switch and DSD mode,
//! then the DSD settings every device shares.

use egui::Ui;
use fp_backends::DeviceInfo;
use fp_model::{DsdDevice, DsdMix, DsdOutput, OutputDevice};

use super::{BackendChoice, caption, note, row, toggle, update};
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

/// One device: its bit-perfect switch and, once it is bit-perfect, its DSD
/// mode.
fn device_rows(
    ui: &mut Ui,
    scene: &Scene<'_>,
    device: &OutputDevice,
    info: Option<&DeviceInfo>,
    name: &str,
) {
    let t = scene.i18n;
    let listed = &scene.state.config.outputs.bit_perfect;
    let capable = info.is_some_and(|d| d.exclusive_capable);
    let mut on = listed.contains(device);
    // A listed device can always be turned off, even when it is not
    // plugged in or cannot be exclusive any more.
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
        if on != listed.contains(device) {
            let device = device.clone();
            update(scene, move |c| {
                c.outputs.bit_perfect.retain(|d| d != &device);
                if on {
                    c.outputs.bit_perfect.push(device);
                }
            });
        }
    });
    if !listed.contains(device) {
        return;
    }
    let current = scene
        .state
        .config
        .outputs
        .dsd_output_for(&device.backend, &device.device);
    let label = t.tr_args("settings-dsd-mode", &[("device", name.into())]);
    row(ui, &label, None, |ui| {
        egui::ComboBox::from_id_salt(("dsd-mode", &device.backend, &device.device))
            .selected_text(dsd_mode_label(t, current))
            .show_ui(ui, |ui| {
                let caps = fp_model::DsdCaps {
                    // Only bit-perfect devices reach this row until Q3.
                    bit_perfect: true,
                    exclusive_capable: info.is_some_and(|d| d.exclusive_capable),
                    native_dsd: info.is_some_and(|d| d.native_dsd),
                    linux: std::env::consts::OS == "linux",
                };
                for mode in fp_model::offered_dsd_modes(caps, current) {
                    if ui
                        .selectable_label(mode == current, dsd_mode_label(t, mode))
                        .clicked()
                    {
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
                }
            });
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
}

fn dsd_mode_label(t: &crate::i18n::I18n, mode: DsdOutput) -> String {
    t.tr(match mode {
        DsdOutput::Pcm => "settings-dsd-pcm",
        DsdOutput::Dop => "settings-dsd-dop",
        DsdOutput::Native => "settings-dsd-native",
    })
}
