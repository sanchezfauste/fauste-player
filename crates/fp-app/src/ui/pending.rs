//! Settings pending (live settings spec §9): what waits, on what, and the
//! confirmation of Apply now. The rules are `fp_model::live`; this module
//! only describes them and asks.

use egui::{RichText, vec2};
use fp_model::{
    AppState, BusyCause, CartId, DeviceSettings, DsdOutput, Holder, OutputDevice, Pending,
    PendingItem, PlayerId, Route, Target,
};

use super::app::Scene;
use super::exit_guard::button;
use super::theme;
use super::widgets::{font, font_medium};
use crate::i18n::I18n;

/// Names a device as the Settings pickers do; an unknown id as is.
pub(crate) type DeviceNames<'a> = &'a dyn Fn(&OutputDevice) -> String;

fn player_number(state: &AppState, id: PlayerId) -> usize {
    state
        .players
        .iter()
        .position(|p| p.id == id)
        .map_or(0, |i| i + 1)
}

fn holder_label(t: &I18n, state: &AppState, holder: Holder) -> String {
    match holder {
        Holder::PlayerMain(p) => t.tr_args(
            "holder-player-main",
            &[("n", player_number(state, p).into())],
        ),
        Holder::PlayerCue(p) => t.tr_args(
            "holder-player-cue",
            &[("n", player_number(state, p).into())],
        ),
        Holder::CartwallMain => t.tr("holder-cartwall-main"),
        Holder::CartwallCue => t.tr("holder-cartwall-cue"),
    }
}

fn route_label(t: &I18n, holder: Holder, route: Option<&Route>, names: DeviceNames<'_>) -> String {
    match route {
        Some(r) => names(&OutputDevice {
            backend: r.backend.clone(),
            device: r.device.clone(),
        }),
        // Main without a route plays on the default output; Cue has none.
        None if matches!(holder, Holder::PlayerMain(_) | Holder::CartwallMain) => {
            t.tr("settings-default-device")
        }
        None => t.tr("settings-none"),
    }
}

fn audio_system_label(t: &I18n, backend: Option<&str>) -> String {
    match backend {
        None => t.tr("settings-default-backend"),
        Some("null") => t.tr("settings-backend-null"),
        Some(id) => fp_backends::display_name(id).to_owned(),
    }
}

/// A rate in kHz as the pickers show it: 48, 44.1, 88.2.
fn khz(rate: u32) -> String {
    if rate.is_multiple_of(1000) {
        (rate / 1000).to_string()
    } else {
        format!("{}", f64::from(rate) / 1000.0)
    }
}

fn dsd_mode_label(t: &I18n, mode: DsdOutput) -> String {
    t.tr(match mode {
        DsdOutput::Pcm => "settings-dsd-pcm",
        DsdOutput::Dop => "settings-dsd-dop",
        DsdOutput::Native => "settings-dsd-native",
    })
}

/// What changes between two device settings, as a list ("sample rate 48 →
/// 96 kHz, buffer 512 → 256"). The DSD mix and silence count only while
/// the device carries DSD on both sides (L4).
pub(crate) fn device_changes(t: &I18n, from: &DeviceSettings, to: &DeviceSettings) -> String {
    let mut parts = Vec::new();
    if from.sample_rate != to.sample_rate {
        parts.push(t.tr_args(
            "pending-change-rate",
            &[
                ("from", khz(from.sample_rate).into()),
                ("to", khz(to.sample_rate).into()),
            ],
        ));
    }
    if from.buffer_frames != to.buffer_frames {
        parts.push(t.tr_args(
            "pending-change-buffer",
            &[
                ("from", from.buffer_frames.to_string().into()),
                ("to", to.buffer_frames.to_string().into()),
            ],
        ));
    }
    if from.bit_perfect != to.bit_perfect {
        parts.push(t.tr(if to.bit_perfect {
            "pending-change-bit-perfect-on"
        } else {
            "pending-change-bit-perfect-off"
        }));
    }
    if from.dsd != to.dsd {
        parts.push(t.tr_args(
            "pending-change-dsd",
            &[("mode", dsd_mode_label(t, to.dsd).into())],
        ));
    }
    if let (Some(a), Some(b)) = (from.dsd_mix, to.dsd_mix)
        && a != b
    {
        parts.push(t.tr("pending-change-dsd-mix"));
    }
    if let (Some(a), Some(b)) = (from.dsd_silence_ms, to.dsd_silence_ms)
        && a.to_bits() != b.to_bits()
    {
        parts.push(t.tr_args(
            "pending-change-dsd-silence",
            &[
                ("from", format!("{a:.0}").into()),
                ("to", format!("{b:.0}").into()),
            ],
        ));
    }
    parts.join(", ")
}

/// A device's running settings, for "Still …".
fn device_values(t: &I18n, s: &DeviceSettings) -> String {
    t.tr_args(
        "pending-running",
        &[
            ("rate", khz(s.sample_rate).into()),
            ("buffer", s.buffer_frames.to_string().into()),
        ],
    )
}

/// A cart's name as the close guard gives it: its name, else its track's
/// title, else "Cart n" (its position on its page).
fn cart_label(t: &I18n, state: &AppState, id: CartId) -> String {
    let found = state
        .cartwall
        .pages
        .iter()
        .find_map(|page| page.carts.iter().enumerate().find(|(_, c)| c.id == id));
    let Some((i, cart)) = found else {
        return String::new();
    };
    Some(cart.name.clone())
        .filter(|n| !n.trim().is_empty())
        .or_else(|| {
            cart.track
                .and_then(|track| state.library.get(track))
                .map(|track| track.title.clone())
        })
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| t.tr_args("on-air-cart-empty", &[("n", (i + 1).into())]))
}

/// One pending item (§9.2).
pub(crate) fn describe_item(
    t: &I18n,
    state: &AppState,
    names: DeviceNames<'_>,
    item: &PendingItem,
) -> String {
    match item {
        PendingItem::AudioSystem { from, to } => t.tr_args(
            "pending-audio-system",
            &[
                ("from", audio_system_label(t, from.as_deref()).into()),
                ("to", audio_system_label(t, to.as_deref()).into()),
            ],
        ),
        PendingItem::Route { holder, to, .. } => t.tr_args(
            "pending-route",
            &[
                ("holder", holder_label(t, state, *holder).into()),
                ("device", route_label(t, *holder, to.as_ref(), names).into()),
            ],
        ),
        PendingItem::Device { device, from, to } => t.tr_args(
            "pending-device",
            &[
                ("device", names(device).into()),
                ("changes", device_changes(t, from, to).into()),
            ],
        ),
    }
}

/// One cause (§9.2).
pub(crate) fn describe_cause(t: &I18n, state: &AppState, cause: &BusyCause) -> String {
    let player = |key: &str, p: PlayerId| t.tr_args(key, &[("n", player_number(state, p).into())]);
    match *cause {
        BusyCause::PlayerPlaying(p) => player("cause-player-playing", p),
        BusyCause::PlayerFading(p) => player("cause-player-fading", p),
        BusyCause::PlayerCue(p) => player("cause-player-cue", p),
        BusyCause::CartPlaying(c) => t.tr_args(
            "cause-cart-playing",
            &[("name", cart_label(t, state, c).into())],
        ),
        BusyCause::CartCue(c) => t.tr_args(
            "cause-cart-cue",
            &[("name", cart_label(t, state, c).into())],
        ),
    }
}

pub(crate) fn causes_list(t: &I18n, state: &AppState, causes: &[BusyCause]) -> String {
    causes
        .iter()
        .map(|c| describe_cause(t, state, c))
        .collect::<Vec<_>>()
        .join(", ")
}

/// An item and, when it waits, what for.
pub(crate) fn describe(t: &I18n, state: &AppState, names: DeviceNames<'_>, p: &Pending) -> String {
    let item = describe_item(t, state, names, &p.item);
    if p.causes.is_empty() {
        return item;
    }
    let waiting = t.tr_args(
        "pending-waiting",
        &[("causes", causes_list(t, state, &p.causes).into())],
    );
    format!("{item} — {waiting}")
}

/// Why the engine refused an item's value. A device's refusal says what
/// it keeps running with; a route's or the audio system's does not (the
/// engine keeps the old one without saying which values it was running).
pub(crate) fn describe_failure(
    t: &I18n,
    state: &AppState,
    names: DeviceNames<'_>,
    p: &Pending,
) -> Option<String> {
    let reason = p.failure.as_ref()?;
    Some(match &p.item {
        PendingItem::Device { device, from, to } => t.tr_args(
            "pending-failed-device",
            &[
                ("device", names(device).into()),
                ("what", device_changes(t, from, to).into()),
                ("reason", reason.clone().into()),
                ("running", device_values(t, from).into()),
            ],
        ),
        PendingItem::Route { holder, to, .. } => t.tr_args(
            "pending-failed",
            &[
                ("target", holder_label(t, state, *holder).into()),
                ("what", route_label(t, *holder, to.as_ref(), names).into()),
                ("reason", reason.clone().into()),
            ],
        ),
        PendingItem::AudioSystem { to, .. } => t.tr_args(
            "pending-failed",
            &[
                ("target", t.tr("settings-backend").into()),
                ("what", audio_system_label(t, to.as_deref()).into()),
                ("reason", reason.clone().into()),
            ],
        ),
    })
}

/// What an Apply now interruption is about: a device, a player's or the
/// cartwall's output, or the audio system.
pub(crate) fn target_label(
    t: &I18n,
    state: &AppState,
    names: DeviceNames<'_>,
    target: &Target,
) -> String {
    match target {
        Target::AudioSystem => t.tr("settings-backend"),
        Target::Route(holder) => holder_label(t, state, *holder),
        Target::Device(device) => names(device),
    }
}

/// What the operator chose in the pending panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Answer {
    Close,
    ApplyNow,
}

fn text(ui: &mut egui::Ui, text: String, size: f32, color: egui::Color32) {
    ui.add(
        egui::Label::new(RichText::new(text).font(font(size)).color(color))
            .selectable(false)
            .wrap(),
    );
}

fn title(ui: &mut egui::Ui, text: String) {
    ui.add(
        egui::Label::new(
            RichText::new(text)
                .font(font_medium(18.0))
                .color(theme::TEXT),
        )
        .selectable(false),
    );
}

/// The pending panel (§9.1): every item, what it waits for, and any
/// refusal; Apply now while an output change is pending. `None` keeps it
/// open; Esc is answered by `AppUi::keyboard`, a click on the backdrop
/// closes it.
pub(crate) fn show_panel(
    ctx: &egui::Context,
    scene: &Scene<'_>,
    names: DeviceNames<'_>,
) -> Option<Answer> {
    let (t, state) = (scene.i18n, scene.state);
    let items = fp_model::pending(state);
    let width = (ctx.content_rect().width() - 48.0).clamp(280.0, 520.0);
    let mut answer = None;
    let modal = egui::Modal::new(egui::Id::new("settings-pending"))
        .frame(egui::Frame::new().fill(theme::SURFACE).inner_margin(20.0))
        .backdrop_color(theme::NEUTRAL_900.gamma_multiply(0.7))
        .show(ctx, |ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            title(ui, t.tr("top-settings-pending"));
            text(ui, t.tr("settings-pending"), 12.0, theme::NEUTRAL_300);
            for p in &items {
                text(ui, describe(t, state, names, p), 12.0, theme::TEXT);
                if let Some(failure) = describe_failure(t, state, names, p) {
                    text(ui, failure, 11.0, theme::AMBER);
                }
            }
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if button(ui, &t.tr("settings-close"), None) {
                    answer = Some(Answer::Close);
                }
                if fp_model::has_output_items(state)
                    && button(ui, &t.tr("settings-apply-now"), Some(theme::AMBER))
                {
                    answer = Some(Answer::ApplyNow);
                }
            });
        });
    if answer.is_none() && modal.should_close() {
        answer = Some(Answer::Close);
    }
    answer
}

/// Apply now's confirmation (§9.3): what it briefly interrupts. `Some(true)`
/// interrupts and applies; `Some(false)` cancels (the button or the
/// backdrop; Esc is answered by `AppUi::keyboard`).
pub(crate) fn show_confirm(
    ctx: &egui::Context,
    scene: &Scene<'_>,
    names: DeviceNames<'_>,
) -> Option<bool> {
    let (t, state) = (scene.i18n, scene.state);
    let interruptions = fp_model::interruptions(state);
    let width = (ctx.content_rect().width() - 48.0).clamp(280.0, 440.0);
    let mut answer = None;
    let modal = egui::Modal::new(egui::Id::new("apply-now"))
        .frame(egui::Frame::new().fill(theme::SURFACE).inner_margin(20.0))
        .backdrop_color(theme::NEUTRAL_900.gamma_multiply(0.7))
        .show(ctx, |ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            title(ui, t.tr("apply-now-title"));
            text(ui, t.tr("apply-now-body"), 12.0, theme::NEUTRAL_300);
            for (target, causes) in &interruptions {
                let line = t.tr_args(
                    "apply-now-line",
                    &[
                        ("device", target_label(t, state, names, target).into()),
                        ("causes", causes_list(t, state, causes).into()),
                    ],
                );
                text(ui, line, 12.0, theme::TEXT);
            }
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if button(ui, &t.tr("exit-guard-cancel"), None) {
                    answer = Some(false);
                }
                if button(ui, &t.tr("apply-now-confirm"), Some(theme::ON_AIR_ROW)) {
                    answer = Some(true);
                }
            });
        });
    if answer.is_none() && modal.should_close() {
        answer = Some(false);
    }
    answer
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;
    use fp_model::{BusyCause, Config, DsdMix, Pending, Route};

    fn en() -> I18n {
        I18n::new(Some("en-US"))
    }

    fn base() -> DeviceSettings {
        DeviceSettings {
            sample_rate: 48_000,
            buffer_frames: 512,
            bit_perfect: false,
            dsd: DsdOutput::Pcm,
            dsd_mix: None,
            dsd_silence_ms: None,
        }
    }

    fn dac() -> OutputDevice {
        OutputDevice {
            backend: "null".into(),
            device: "dac".into(),
        }
    }

    fn route(device: &str) -> Route {
        Route {
            backend: "null".into(),
            device: device.into(),
            first_channel: 0,
        }
    }

    #[test]
    fn device_changes_name_each_value_that_changes() {
        let t = en();
        let to = DeviceSettings {
            sample_rate: 96_000,
            buffer_frames: 256,
            bit_perfect: true,
            ..base()
        };
        assert_eq!(
            device_changes(&t, &base(), &to),
            "sample rate 48 → 96 kHz, buffer 512 → 256, bit-perfect on"
        );
        let dop = DeviceSettings {
            bit_perfect: true,
            dsd: DsdOutput::Dop,
            dsd_mix: Some(DsdMix::ConvertToPcm),
            dsd_silence_ms: Some(200.0),
            ..base()
        };
        let held = DeviceSettings {
            dsd_mix: Some(DsdMix::HoldOthers),
            dsd_silence_ms: Some(400.0),
            ..dop
        };
        assert_eq!(
            device_changes(&t, &dop, &held),
            "DSD mix, DSD silence 200 → 400 ms"
        );
        let cd = DeviceSettings {
            sample_rate: 44_100,
            ..base()
        };
        assert_eq!(
            device_changes(&t, &cd, &base()),
            "sample rate 44.1 → 48 kHz"
        );
        assert_eq!(
            device_changes(&t, &base(), &dop),
            "bit-perfect on, DSD: DoP"
        );
    }

    #[test]
    fn items_and_causes_read_as_the_spec_says() {
        let t = en();
        let state = AppState::new(Config::default(), "Main");
        let p = state.players[0].id;
        // Names by id, as before the device lists are read.
        let names = |d: &OutputDevice| d.device.clone();
        let item = PendingItem::Route {
            holder: Holder::PlayerCue(p),
            from: None,
            to: Some(route("phones")),
        };
        assert_eq!(describe_item(&t, &state, &names, &item), "P1 CUE → phones");
        let item = PendingItem::Route {
            holder: Holder::CartwallMain,
            from: Some(route("phones")),
            to: None,
        };
        assert_eq!(
            describe_item(&t, &state, &names, &item),
            "Cartwall Main → System default"
        );
        let pending = Pending {
            item: PendingItem::Device {
                device: dac(),
                from: base(),
                to: DeviceSettings {
                    buffer_frames: 1024,
                    ..base()
                },
            },
            causes: vec![BusyCause::PlayerPlaying(p), BusyCause::PlayerCue(p)],
            failure: None,
        };
        assert_eq!(
            describe(&t, &state, &names, &pending),
            "dac: buffer 512 → 1024 — waiting for P1 playing, P1 CUE"
        );
        let item = PendingItem::AudioSystem {
            from: None,
            to: Some("null".into()),
        };
        assert_eq!(
            describe_item(&t, &state, &names, &item),
            "Audio system: System default → No output (silent)"
        );
        assert_eq!(
            target_label(&t, &state, &names, &Target::Route(Holder::PlayerMain(p))),
            "P1 Main"
        );
        assert_eq!(
            target_label(&t, &state, &names, &Target::Device(dac())),
            "dac"
        );
    }

    #[test]
    fn a_device_refusal_says_what_the_device_keeps() {
        let t = en();
        let names = |d: &OutputDevice| d.device.clone();
        let pending = Pending {
            item: PendingItem::Device {
                device: dac(),
                from: base(),
                to: DeviceSettings {
                    sample_rate: 44_100,
                    ..base()
                },
            },
            causes: Vec::new(),
            failure: Some("44100 Hz refused".into()),
        };
        assert_eq!(
            describe_failure(
                &t,
                &AppState::new(Config::default(), "Main"),
                &names,
                &pending
            )
            .as_deref(),
            Some(
                "dac did not take sample rate 48 → 44.1 kHz: 44100 Hz refused. Still 48 kHz, buffer 512."
            )
        );
    }

    #[test]
    fn a_route_or_audio_system_refusal_has_no_still_part() {
        let t = en();
        let state = AppState::new(Config::default(), "Main");
        let p = state.players[0].id;
        let names = |d: &OutputDevice| d.device.clone();
        let route_item = Pending {
            item: PendingItem::Route {
                holder: Holder::PlayerCue(p),
                from: None,
                to: Some(route("phones")),
            },
            causes: Vec::new(),
            failure: Some("gone".into()),
        };
        assert_eq!(
            describe_failure(&t, &state, &names, &route_item).as_deref(),
            Some("P1 CUE did not take phones: gone.")
        );
        let system = Pending {
            item: PendingItem::AudioSystem {
                from: None,
                to: Some("null".into()),
            },
            causes: Vec::new(),
            failure: Some("no such system".into()),
        };
        assert_eq!(
            describe_failure(&t, &state, &names, &system).as_deref(),
            Some("Audio system did not take No output (silent): no such system.")
        );
    }

    #[test]
    fn a_pending_item_without_a_refusal_has_no_failure_text() {
        let t = en();
        let state = AppState::new(Config::default(), "Main");
        let names = |d: &OutputDevice| d.device.clone();
        let pending = Pending {
            item: PendingItem::AudioSystem {
                from: None,
                to: Some("null".into()),
            },
            causes: Vec::new(),
            failure: None,
        };
        assert_eq!(describe_failure(&t, &state, &names, &pending), None);
    }
}
