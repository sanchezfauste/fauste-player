#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
//! The playlist tabs shrink, cut and scroll (feedback 2 spec O35).

mod support;

use egui::{Event, Modifiers, MouseWheelUnit, vec2};
use egui_kittest::kittest::{NodeT, Queryable};
use fp_model::{AppState, Command, PlaylistId};

fn many_playlists(n: usize) -> AppState {
    let mut state = support::state(1, 0);
    let first = state.playlists.first_id().unwrap();
    fp_model::apply(
        &mut state,
        Command::RenamePlaylist {
            playlist: first,
            name: "List 1".into(),
        },
    )
    .unwrap();
    for k in 2..=n {
        fp_model::apply(
            &mut state,
            Command::CreatePlaylist {
                name: format!("List {k}"),
            },
        )
        .unwrap();
    }
    state
}

fn id_of(state: &AppState, name: &str) -> PlaylistId {
    state
        .playlists
        .iter()
        .find(|p| p.name == name)
        .map(|p| p.id)
        .unwrap()
}

fn show(state: &mut AppState, name: &str) {
    let player = state.players[0].id;
    let playlist = id_of(state, name);
    fp_model::apply(state, Command::ShowPlaylist(player, playlist)).unwrap();
}

fn setup(
    state: AppState,
) -> (
    egui_kittest::Harness<'static, fp_app::ui::app::AppUi>,
    std::sync::Arc<support::Fake>,
) {
    let (mut h, fake) = support::harness_sized(state, vec2(1000.0, 700.0), |ui| ui);
    h.run_steps(3);
    (h, fake)
}

#[test]
fn a_name_of_200_characters_is_cut_and_the_strip_stays_in_its_column() {
    let mut state = many_playlists(2);
    let long = "A".repeat(200);
    let second = id_of(&state, "List 2");
    fp_model::apply(
        &mut state,
        Command::RenamePlaylist {
            playlist: second,
            name: long.clone(),
        },
    )
    .unwrap();
    let (h, _) = setup(state);
    let tab = h.get_by_label(&long).rect();
    assert!(tab.width() <= 1000.0 / 2.0, "{tab:?}");
    assert!(tab.right() <= 1000.0, "{tab:?}");
}

#[test]
fn thirty_tabs_scroll_and_the_shown_one_is_in_view() {
    let mut state = many_playlists(30);
    show(&mut state, "List 30");
    let (mut h, _) = setup(state);
    h.run_steps(3);
    let tab = h.get_by_label("List 30").rect();
    let left = h.get_by_label("Scroll tabs left").rect();
    let right = h.get_by_label("Scroll tabs right");
    assert!(tab.right() <= right.rect().left() + 1.0, "{tab:?}");
    assert!(tab.left() >= left.right() - 1.0, "{tab:?}");
    assert!(right.accesskit_node().is_disabled());
}

#[test]
fn the_arrows_move_the_strip_by_one_tab() {
    let (mut h, _) = setup(many_playlists(30));
    let x0 = h.get_by_label("List 4").rect().left();
    h.get_by_label("Scroll tabs right").click();
    h.run_steps(3);
    let x1 = h.get_by_label("List 4").rect().left();
    assert!((x0 - x1 - 72.0).abs() < 0.5, "{x0} {x1}");
    h.get_by_label("Scroll tabs left").click();
    h.run_steps(3);
    let x2 = h.get_by_label("List 4").rect().left();
    assert!((x2 - x0).abs() < 0.5, "{x0} {x2}");
}

#[test]
fn the_wheel_over_the_strip_scrolls_it() {
    let (mut h, _) = setup(many_playlists(30));
    let tab = h.get_by_label("List 2").rect();
    let x0 = h.get_by_label("List 4").rect().left();
    h.hover_at(tab.center());
    h.run_steps(1);
    h.event(Event::MouseWheel {
        unit: MouseWheelUnit::Line,
        delta: vec2(0.0, -3.0),
        phase: egui::TouchPhase::Move,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(8);
    let x1 = h.get_by_label("List 4").rect().left();
    assert!(x1 < x0 - 10.0, "{x0} {x1}");
}

#[test]
fn a_tab_click_still_shows_the_playlist() {
    let (mut h, fake) = setup(many_playlists(30));
    let want = id_of(&fake.state.load(), "List 3");
    fake.take_sent();
    h.get_by_label("List 3").click();
    h.run_steps(3);
    let player = fake.player(0);
    assert!(
        fake.take_sent()
            .iter()
            .any(|c| matches!(c, Command::ShowPlaylist(p, l) if *p == player && *l == want))
    );
}

#[test]
fn few_tabs_show_no_arrows() {
    let (h, _) = setup(support::state(1, 0));
    assert!(h.query_by_label("Scroll tabs left").is_none());
    assert!(h.query_by_label("Scroll tabs right").is_none());
}
