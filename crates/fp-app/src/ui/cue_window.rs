//! The CUE window (feedback 2 spec O12): one floating, non-modal window per
//! running CUE, with the waveform and position, the elapsed and remaining
//! time, and Pause/Resume, Stop and Load as next. Closing it stops the CUE.
//! Everything it does is a command; nothing here waits for the engine.

use egui::{Align, Layout, RichText, Stroke, pos2, vec2};
use egui_phosphor::regular as icon;
use fp_model::Command;

use super::app::Scene;
use super::format;
use super::glyphs::{self, TransportAction};
use super::theme;
use super::view::{self, CueWindowView};
use super::widgets::{self, TileStyle, font, font_medium, font_semibold};

const WIDTH: f32 = 380.0;
const WAVE_HEIGHT: f32 = 56.0;
const BUTTON: egui::Vec2 = vec2(40.0, 28.0);
const LOAD_NEXT_WIDTH: f32 = 130.0;

/// Draws the window of every player that has a CUE running.
pub(crate) fn show_all(ctx: &egui::Context, scene: &Scene<'_>) {
    for (index, player) in scene.state.players.iter().enumerate() {
        if player.cue.is_none() {
            continue;
        }
        let position = scene
            .telemetry
            .players
            .iter()
            .find(|(id, _)| *id == player.id)
            .and_then(|(_, t)| t.cue_position_secs);
        if let Some(v) = view::cue_window_view(scene.state, player.id, position) {
            show(ctx, scene, index, &v);
        }
    }
}

fn show(ctx: &egui::Context, scene: &Scene<'_>, index: usize, v: &CueWindowView) {
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
            wave(ui, scene, v);
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

fn wave(ui: &mut egui::Ui, scene: &Scene<'_>, v: &CueWindowView) {
    let track = scene.state.playlists.entry(v.entry).map(|e| e.track);
    let media = track.and_then(|track| scene.media.get(track));
    let markers = view::MarkerFractions {
        position: v.position,
        ..view::MarkerFractions::default()
    };
    let label = scene.i18n.tr("tip-cue-waveform");
    let mix_label = String::new();
    let input = widgets::WaveInput {
        id: egui::Id::new(("cue-waveform", v.player)),
        media: media.as_ref(),
        total: v.total,
        markers,
        colors: theme::wave_colors(&scene.state.config.ui.wave_color),
        mix_active: false,
        mix_label: &mix_label,
        accessible_label: &label,
        view: None,
        shield: None,
        seekable: true,
    };
    let output = widgets::waveform(ui, WAVE_HEIGHT, &input);
    if let Some(secs) = output.seek {
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
        if widgets::tile(
            ui,
            BUTTON,
            &label,
            true,
            TileStyle::plain(),
            move |p, r, c| glyphs::paint(p, r, action, 14.0, c),
        )
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
        let label = t.tr("cue-window-load-next");
        let text = format!("{}  {label}", icon::ARROW_BEND_DOWN_RIGHT);
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
            vec2(LOAD_NEXT_WIDTH, 28.0),
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
