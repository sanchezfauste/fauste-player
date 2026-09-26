//! Settings → Keyboard shortcuts (Phase 2 spec P2.5): click an action, press
//! a key; a chord used elsewhere is named before it moves.

use egui::{Key, RichText, Ui, vec2};
use egui_phosphor::regular as icon;
use fp_model::{Command, KeyChord, ShortcutAction};

use super::super::app::Scene;
use super::super::theme;
use super::super::widgets::{self, TileStyle, font};
use super::{SettingsState, heading};

#[derive(Default)]
pub(crate) struct KeysState {
    /// Waiting for a key for this action.
    capture: Option<ShortcutAction>,
    /// A chord that another action uses: `(action, chord, current owner)`.
    pending: Option<(ShortcutAction, KeyChord, ShortcutAction)>,
}

/// The name of an action, as listed in Settings.
pub(crate) fn action_name(scene: &Scene<'_>, action: ShortcutAction) -> String {
    let t = scene.i18n;
    let n = |key: &str, n: u16| t.tr_args(key, &[("n", n.into())]);
    match action {
        ShortcutAction::PlayPlayer(p) => n("shortcut-play", p),
        ShortcutAction::PausePlayer(p) => n("shortcut-pause", p),
        ShortcutAction::StopPlayer(p) => n("shortcut-stop", p),
        ShortcutAction::FadeStopPlayer(p) => n("shortcut-fade-stop", p),
        ShortcutAction::CuePlayer(p) => n("shortcut-cue", p),
        ShortcutAction::FireCart(c) => n("shortcut-fire-cart", c),
        ShortcutAction::StopAllCarts => t.tr("shortcut-stop-all-carts"),
        ShortcutAction::ToggleCartwall => t.tr("shortcut-toggle-cartwall"),
        ShortcutAction::NextCartPage => t.tr("shortcut-next-page"),
        ShortcutAction::PreviousCartPage => t.tr("shortcut-previous-page"),
    }
}

/// Every action the current setup offers.
fn actions(scene: &Scene<'_>) -> Vec<ShortcutAction> {
    let players = scene.state.players.len() as u16;
    let carts = scene
        .state
        .cartwall
        .pages
        .iter()
        .map(|p| p.carts.len())
        .max()
        .unwrap_or(0) as u16;
    let mut out = Vec::new();
    for p in 1..=players {
        out.extend([
            ShortcutAction::PlayPlayer(p),
            ShortcutAction::PausePlayer(p),
            ShortcutAction::StopPlayer(p),
            ShortcutAction::FadeStopPlayer(p),
            ShortcutAction::CuePlayer(p),
        ]);
    }
    out.extend((1..=carts).map(ShortcutAction::FireCart));
    out.extend([
        ShortcutAction::StopAllCarts,
        ShortcutAction::ToggleCartwall,
        ShortcutAction::NextCartPage,
        ShortcutAction::PreviousCartPage,
    ]);
    out
}

/// The first key pressed this frame (not a repeat), as a chord.
fn pressed_chord(ui: &Ui) -> Option<KeyChord> {
    ui.input(|i| {
        i.events.iter().find_map(|e| match e {
            egui::Event::Key {
                key,
                pressed: true,
                repeat: false,
                modifiers,
                ..
            } => Some(KeyChord {
                key: key.name().to_owned(),
                ctrl: modifiers.ctrl,
                alt: modifiers.alt,
                shift: modifiers.shift,
                command: modifiers.mac_cmd,
            }),
            _ => None,
        })
    })
}

fn small(ui: &mut Ui, label: &str, glyph: &str) -> bool {
    let width = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font(12.0), theme::TEXT)
        .size()
        .x
        + 34.0;
    widgets::tile(
        ui,
        vec2(width, 26.0),
        label,
        true,
        TileStyle::plain(),
        |p, r, c| {
            p.text(
                r.center(),
                egui::Align2::CENTER_CENTER,
                format!("{glyph} {label}"),
                font(12.0),
                c,
            );
        },
    )
    .clicked()
}

pub(super) fn section(ui: &mut Ui, scene: &Scene<'_>, st: &mut SettingsState) {
    let t = scene.i18n;
    let k = &mut st.keys;
    let shortcuts = &scene.state.config.shortcuts;
    heading(ui, &t.tr("settings-tab-shortcuts"));
    if let Some(action) = k.capture
        && let Some(chord) = pressed_chord(ui)
    {
        k.capture = None;
        if chord.key != Key::Escape.name() {
            let owner = shortcuts
                .iter()
                .find(|s| s.chord == chord && s.action != action)
                .map(|s| s.action);
            match owner {
                Some(owner) => k.pending = Some((action, chord, owner)),
                None => {
                    scene.ctl.send(Command::SetShortcut {
                        action,
                        chord: Some(chord),
                    });
                }
            }
        }
    }
    if let Some((action, chord, owner)) = k.pending.clone() {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 0.0);
            ui.add(
                egui::Label::new(
                    RichText::new(t.tr_args(
                        "shortcut-conflict",
                        &[
                            ("chord", chord.to_string().into()),
                            ("action", action_name(scene, owner).into()),
                        ],
                    ))
                    .font(font(12.0))
                    .color(theme::AMBER),
                )
                .selectable(false),
            );
            if small(ui, &t.tr("shortcut-assign"), icon::CHECK) {
                scene.ctl.send(Command::SetShortcut {
                    action,
                    chord: Some(chord.clone()),
                });
                k.pending = None;
            }
            if small(ui, &t.tr("shortcut-cancel"), icon::X) {
                k.pending = None;
            }
        });
        ui.add_space(8.0);
    }
    ui.horizontal(|ui| {
        if small(ui, &t.tr("shortcut-reset"), icon::ARROW_COUNTER_CLOCKWISE) {
            scene.ctl.send(Command::ResetShortcuts);
        }
        ui.add(
            egui::Label::new(
                RichText::new(t.tr("shortcut-help"))
                    .font(font(11.0))
                    .color(theme::NEUTRAL_500),
            )
            .selectable(false),
        );
    });
    ui.add_space(8.0);
    for action in actions(scene) {
        let name = action_name(scene, action);
        let chord = shortcuts
            .iter()
            .find(|s| s.action == action)
            .map(|s| s.chord.to_string());
        let capturing = k.capture == Some(action);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 0.0);
            let shown = if capturing {
                t.tr("shortcut-press-key")
            } else {
                chord.clone().unwrap_or_else(|| "—".to_owned())
            };
            let style = TileStyle {
                border: if capturing {
                    theme::ACCENT
                } else {
                    theme::NEUTRAL_800
                },
                ..TileStyle::plain()
            };
            let width = (ui.available_width() - 110.0).max(200.0);
            if widgets::tile(ui, vec2(width, 28.0), &name, true, style, |p, r, c| {
                p.text(
                    r.left_center() + vec2(10.0, 0.0),
                    egui::Align2::LEFT_CENTER,
                    &name,
                    font(13.0),
                    c,
                );
                p.text(
                    r.right_center() - vec2(10.0, 0.0),
                    egui::Align2::RIGHT_CENTER,
                    &shown,
                    font(12.0),
                    if capturing {
                        theme::ACCENT_300
                    } else {
                        theme::NEUTRAL_300
                    },
                );
            })
            .clicked()
            {
                k.capture = Some(action);
                k.pending = None;
            }
            if chord.is_some() && small(ui, &t.tr("shortcut-unbind"), icon::X) {
                scene.ctl.send(Command::SetShortcut {
                    action,
                    chord: None,
                });
            }
        });
        ui.add_space(4.0);
    }
}
