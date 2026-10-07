//! The CUE window (feedback 2 spec O12): one floating, non-modal window per
//! running CUE, with the waveform panel the player uses (zoom, pan, intro,
//! outro and MIX markers and their editing; operator feedback 4, Q7) and
//! the position, the elapsed and remaining time, and Pause/Resume, Stop and
//! Set as next. Closing it stops the CUE. Everything it does is a command;
//! nothing here waits for the engine.

use egui::{Align, Layout, RichText, Stroke, pos2, vec2};
use egui_phosphor::regular as icon;
use fp_model::Command;

use super::app::{Scene, ViewState};
use super::format;
use super::glyphs::{self, TransportAction};
use super::theme;
use super::view::{self, CueWindowView};
use super::wave_panel::{self, WaveBadges, WavePanelInput};
use super::wave_view::WaveKey;
use super::widgets::{self, TileStyle, font, font_medium, font_semibold};

const WIDTH: f32 = 380.0;
const WAVE_HEIGHT: f32 = 56.0;
const BUTTON: egui::Vec2 = vec2(40.0, 28.0);

/// Draws the window of every player that has a CUE running.
pub(crate) fn show_all(ctx: &egui::Context, scene: &Scene<'_>, view_state: &mut ViewState) {
    for (index, player) in scene.state.players.iter().enumerate() {
        if player.cue.is_none() {
            forget(view_state, WaveKey::Cue(player.id));
            continue;
        }
        let position = scene
            .telemetry
            .players
            .iter()
            .find(|(id, _)| *id == player.id)
            .and_then(|(_, t)| t.cue_position_secs);
        if let Some(v) = view::cue_window_view(scene.state, player.id, position) {
            show(ctx, scene, view_state, index, &v);
        }
    }
}

/// A player without a CUE forgets its CUE waveform's zoom, menu point and
/// marker drag, so the next CUE opens on the whole file and a drag cut off
/// by Stop never lands on it.
fn forget(view_state: &mut ViewState, key: WaveKey) {
    view_state.wave_zoom.set(key, None);
    view_state.wave_menu.remove(&key);
    if view_state.marker_drag.is_some_and(|(k, _, _)| k == key) {
        view_state.marker_drag = None;
    }
}

fn show(
    ctx: &egui::Context,
    scene: &Scene<'_>,
    view_state: &mut ViewState,
    index: usize,
    v: &CueWindowView,
) {
    let t = scene.i18n;
    let stagger = 28.0 * index as f32;
    egui::Window::new(t.tr_args("cue-window-title", &[("n", (index + 1).into())]))
        .id(egui::Id::new(("cue-window", v.player)))
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .default_pos(pos2(80.0 + stagger, 120.0 + stagger))
        .frame(
            egui::Frame::new()
                .fill(theme::SURFACE)
                .stroke(Stroke::new(1.0, theme::CUE))
                .inner_margin(12.0),
        )
        .show(ctx, |ui| {
            ui.set_width(WIDTH);
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            title_row(ui, scene, index, v);
            ui.add(
                egui::Label::new(
                    RichText::new(&v.title)
                        .font(font_medium(15.0))
                        .color(theme::TEXT),
                )
                .selectable(false)
                .truncate(),
            );
            let artist = v.artist.clone().unwrap_or_else(|| t.tr("unknown-artist"));
            ui.add(
                egui::Label::new(
                    RichText::new(artist)
                        .font(font(13.0))
                        .color(theme::NEUTRAL_400),
                )
                .selectable(false)
                .truncate(),
            );
            wave(ui, scene, view_state, v);
            time_row(ui, v);
            buttons(ui, scene, v);
        });
}

fn title_row(ui: &mut egui::Ui, scene: &Scene<'_>, index: usize, v: &CueWindowView) {
    let t = scene.i18n;
    ui.horizontal(|ui| {
        ui.add(
            egui::Label::new(
                RichText::new(format!(
                    "{} {}",
                    glyphs::glyph_text(TransportAction::Cue),
                    t.tr_args("cue-window-title", &[("n", (index + 1).into())])
                ))
                .font(font_semibold(11.0))
                .color(theme::CUE),
            )
            .selectable(false),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let label = t.tr("cue-window-close");
            if widgets::tile(
                ui,
                vec2(24.0, 20.0),
                &label,
                true,
                TileStyle::plain(),
                |p, r, c| widgets::glyph(p, r, icon::X, 12.0, c, false),
            )
            .clicked()
            {
                scene.ctl.send(Command::SetCue(v.player, false));
            }
        });
    });
}

fn wave(ui: &mut egui::Ui, scene: &Scene<'_>, view_state: &mut ViewState, v: &CueWindowView) {
    let track = scene.state.playlists.entry(v.entry).map(|e| e.track);
    let media = track.and_then(|track| scene.media.get(track));
    let input = WavePanelInput {
        key: WaveKey::Cue(v.player),
        entry: Some(v.entry),
        track,
        media: media.as_ref(),
        total: v.total,
        markers: v.markers,
        mix_active: v.mix_active,
        // A playing CUE's zoom follows its position, as a playing player's
        // does; a paused CUE keeps the view the operator set to place markers.
        follow: !v.paused,
        badges: WaveBadges {
            intro: v.intro,
            // The talk-over warning is for audio on air; a CUE is not.
            intro_blink: None,
            outro: v.outro,
        },
        editable_markers: true,
        height: WAVE_HEIGHT,
    };
    if let Some(secs) = wave_panel::show(ui, scene, view_state, &input).seek {
        scene.ctl.send(Command::SeekCue(v.player, secs));
    }
}

fn time_row(ui: &mut egui::Ui, v: &CueWindowView) {
    ui.horizontal(|ui| {
        widgets::tabular_label(ui, &format::clock(v.elapsed), &font(12.0), theme::CUE);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let (minutes, _) = format::countdown(v.remaining);
            widgets::tabular_label(ui, &minutes, &font(12.0), theme::NEUTRAL_400);
        });
    });
}

fn buttons(ui: &mut egui::Ui, scene: &Scene<'_>, v: &CueWindowView) {
    let t = scene.i18n;
    ui.horizontal(|ui| {
        // Pause shows Pause while the CUE plays and Resume (a play
        // triangle) while it is held.
        let (action, key) = if v.paused {
            (TransportAction::Play, "cue-window-resume")
        } else {
            (TransportAction::Pause, "cue-window-pause")
        };
        let label = t.tr(key);
        let style = if v.paused {
            widgets::paused_style(widgets::blink(scene.time))
        } else {
            TileStyle::plain()
        };
        if widgets::tile(ui, BUTTON, &label, true, style, move |p, r, c| {
            glyphs::paint(p, r, action, 14.0, c)
        })
        .clicked()
        {
            scene.ctl.send(Command::SetCuePaused(v.player, !v.paused));
        }
        let label = t.tr("cue-window-stop");
        if widgets::tile(ui, BUTTON, &label, true, TileStyle::plain(), |p, r, c| {
            glyphs::paint(p, r, TransportAction::Stop, 14.0, c)
        })
        .clicked()
        {
            scene.ctl.send(Command::SetCue(v.player, false));
        }
        let label = t.tr("menu-set-next");
        let text = format!("{}  {label}", icon::ARROW_BEND_DOWN_RIGHT);
        let width = ui
            .painter()
            .layout_no_wrap(text.clone(), font_semibold(11.0), theme::TEXT)
            .size()
            .x
            + 24.0;
        let style = TileStyle {
            content: if v.can_load_next {
                theme::CUE
            } else {
                theme::NEUTRAL_600
            },
            ..TileStyle::plain()
        };
        if widgets::tile(
            ui,
            vec2(width, 28.0),
            &label,
            v.can_load_next,
            style,
            |p, r, c| {
                p.text(
                    r.center(),
                    egui::Align2::CENTER_CENTER,
                    &text,
                    font_semibold(11.0),
                    c,
                );
            },
        )
        .clicked()
        {
            scene.ctl.send(Command::CueToNext(v.player));
        }
    });
}
