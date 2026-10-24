#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
//! The "+" at the right end of a player's playlist tabs (rule 29).

use crate::support;

use egui::accesskit::Role;
use egui::vec2;
use egui_kittest::kittest::{NodeT, Queryable};
use fp_model::Command;

type H = egui_kittest::Harness<'static, fp_app::ui::app::AppUi>;

fn setup(players: usize) -> (H, std::sync::Arc<support::Fake>) {
    let (mut h, fake) =
        support::harness_sized(support::state(players, 1), vec2(1000.0, 700.0), |ui| ui);
    h.run_steps(3);
    (h, fake)
}

fn created(fake: &support::Fake) -> Vec<Command> {
    fake.take_sent()
        .into_iter()
        .filter(|c| matches!(c, Command::CreatePlaylistIn { .. }))
        .collect()
}

#[test]
fn plus_name_enter_creates_the_playlist_in_this_player_and_closes() {
    let (mut h, fake) = setup(2);
    // Each player has its own "+": the first one is player 1's.
    h.query_all_by_label("New playlist").next().unwrap().click();
    h.run_steps(3);
    h.get_by_role(Role::TextInput).type_text("Jingles");
    h.run_steps(2);
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    assert_eq!(
        created(&fake),
        vec![Command::CreatePlaylistIn {
            name: "Jingles".into(),
            player: fake.player(0),
        }]
    );
    let state = fake.state.load();
    let shown = state.player(fake.player(0)).unwrap().playlist;
    assert_eq!(state.playlists.get(shown).unwrap().name, "Jingles");
    assert!(h.query_by_role(Role::TextInput).is_none(), "popup closed");
}

#[test]
fn the_create_button_needs_a_name_and_creates() {
    let (mut h, fake) = setup(1);
    h.get_by_label("New playlist").click();
    h.run_steps(3);
    assert!(h.get_by_label("Create").accesskit_node().is_disabled());
    h.get_by_role(Role::TextInput).type_text("  Ads  ");
    h.run_steps(2);
    h.get_by_label("Create").click();
    h.run_steps(3);
    assert_eq!(
        created(&fake),
        vec![Command::CreatePlaylistIn {
            name: "Ads".into(),
            player: fake.player(0),
        }]
    );
}

#[test]
fn enter_on_an_empty_name_creates_nothing() {
    let (mut h, fake) = setup(1);
    h.get_by_label("New playlist").click();
    h.run_steps(3);
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    assert!(created(&fake).is_empty());
}

#[test]
fn escape_cancels_and_the_name_is_forgotten() {
    let (mut h, fake) = setup(1);
    h.get_by_label("New playlist").click();
    h.run_steps(3);
    h.get_by_role(Role::TextInput).type_text("Draft");
    h.run_steps(2);
    h.key_press(egui::Key::Escape);
    h.run_steps(3);
    assert!(created(&fake).is_empty());
    assert!(h.query_by_role(Role::TextInput).is_none(), "popup closed");
    h.get_by_label("New playlist").click();
    h.run_steps(3);
    assert_eq!(
        h.get_by_role(Role::TextInput).value().unwrap_or_default(),
        ""
    );
}

#[test]
fn the_plus_sits_right_of_the_scroll_arrows() {
    let mut state = support::state(1, 0);
    for k in 2..=30 {
        fp_model::apply(
            &mut state,
            Command::CreatePlaylist {
                name: format!("List {k}"),
            },
        )
        .unwrap();
    }
    let (mut h, _) = support::harness_sized(state, vec2(1000.0, 700.0), |ui| ui);
    h.run_steps(3);
    let plus = h.get_by_label("New playlist").rect();
    let right = h.get_by_label("Scroll tabs right").rect();
    assert!(plus.left() >= right.right() - 0.5, "{plus:?} {right:?}");
}

#[test]
fn the_name_keeps_the_focus_while_the_popup_is_open() {
    let (mut h, fake) = setup(1);
    h.get_by_label("New playlist").click();
    h.run_steps(3);
    // Something took the focus away: it goes back to the name, so the app's
    // shortcuts do not fire under what is typed.
    h.ctx
        .memory_mut(|m| m.surrender_focus(m.focused().unwrap()));
    h.run_steps(3);
    assert!(h.get_by_role(Role::TextInput).accesskit_node().is_focused());
    h.key_press(egui::Key::Escape);
    h.run_steps(3);
    assert!(h.query_by_role(Role::TextInput).is_none(), "popup closed");
    assert!(fake.take_sent().is_empty(), "Escape only closed the popup");
}
