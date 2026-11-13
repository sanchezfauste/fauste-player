//! egui user interface.

pub mod about;
pub mod app;
pub mod cart_view;
mod cartwall;
pub mod controller;
mod cue_on_air;
mod cue_window;
pub mod exit_guard;
pub mod files;
pub mod format;
pub mod glyphs;
pub mod header_layout;
pub mod icons;
mod library;
mod loudness_edit;
pub mod media_browser;
pub mod music;
mod notice;
pub mod panel_layout;
mod panels;
pub mod pending;
pub mod phone_library;
mod phone_prelisten;
mod plan_window;
mod player;
pub mod playlist_files;
pub(crate) mod playlist_keys;
mod reset_played;
mod schedule;
mod schedule_editor;
mod settings;
pub mod shell;
pub mod tab_strip;
mod table;
pub mod table_layout;
mod tag_editor;
pub mod theme;
pub mod touch;
mod track_cells;
pub mod view;
mod wave_panel;
pub mod wave_view;
pub mod widgets;

/// The name the operator sees for a player: "Music" for the music filler,
/// "Schedule" for the schedule's player (automation spec §8.1), `P<n>` among
/// the auxiliary players otherwise.
pub(crate) fn player_name(
    t: &crate::i18n::I18n,
    state: &fp_model::AppState,
    id: fp_model::PlayerId,
) -> String {
    if state.is_filler(id) {
        return t.tr("music-column");
    }
    if state.automation.as_ref().is_some_and(|a| a.player == id) {
        return t.tr("schedule-column");
    }
    t.tr_args(
        "player-label",
        &[("n", pending::player_number(state, id).into())],
    )
}
