#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The track table follows the current entry (feedback spec F18, §2.3).

mod support;

use egui::{Event, Modifiers, pos2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_model::{AppState, Command, EntryId, PlayerId, apply};
use support::{Fake, harness, state};

fn entries(s: &AppState) -> Vec<EntryId> {
    s.playlists
        .iter()
        .next()
        .unwrap()
        .entries
        .iter()
        .map(|e| e.id)
        .collect()
}

/// Makes `entry` P`n`'s current entry in the Fake's model.
fn play(fake: &Fake, n: usize, entry: EntryId) {
    let mut s = (*fake.state.load_full()).clone();
    let p: PlayerId = s.players[n].id;
    apply(&mut s, Command::Stop(p)).unwrap();
    apply(&mut s, Command::SetNext(p, entry)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    fake.state.store(std::sync::Arc::new(s));
}

/// Whether a table row labelled `label` is on screen (the player's info
/// row shows the current title too, above the table).
fn visible(h: &Harness<'_, AppUi>, label: &str) -> bool {
    let header = h
        .query_all_by_label("TITLE")
        .map(|n| n.rect().bottom())
        .fold(f32::INFINITY, f32::min);
    h.query_all_by_label(label)
        .any(|n| n.rect().top() >= header)
}

#[test]
fn a_new_current_scrolls_its_row_to_the_top() {
    let (mut h, fake) = harness(state(1, 200));
    let e = entries(&fake.state.load_full());
    assert!(!visible(&h, "Song 151"));
    play(&fake, 0, e[150]);
    h.run_steps(4);
    assert!(visible(&h, "Song 151"));
    assert!(!visible(&h, "Song 150"), "the current row is the first one");
}

#[test]
fn the_end_of_the_list_scrolls_as_far_as_it_can() {
    let (mut h, fake) = harness(state(1, 200));
    let e = entries(&fake.state.load_full());
    play(&fake, 0, e[199]);
    h.run_steps(4);
    assert!(visible(&h, "Song 200"));
}

#[test]
fn a_recent_scroll_holds_the_table_still() {
    let (mut h, fake) = harness(state(1, 200));
    let e = entries(&fake.state.load_full());
    let row = h.get_by_label("Song 3").rect().center();
    h.event(Event::PointerMoved(row));
    h.event(Event::MouseWheel {
        unit: egui::MouseWheelUnit::Line,
        delta: egui::vec2(0.0, -1.0),
        phase: egui::TouchPhase::Move,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(2);
    play(&fake, 0, e[150]);
    h.run_steps(4);
    assert!(!visible(&h, "Song 151"));
}

#[test]
fn a_zero_grace_never_follows() {
    let mut s = state(1, 200);
    s.config.ui.follow_current_grace_secs = 0.0;
    let (mut h, fake) = harness(s);
    let e = entries(&fake.state.load_full());
    play(&fake, 0, e[150]);
    h.run_steps(4);
    assert!(!visible(&h, "Song 151"));
}

#[test]
fn a_current_in_another_playlist_switches_the_tab_then_scrolls() {
    let mut s = state(1, 3);
    apply(
        &mut s,
        Command::CreatePlaylistFromPaths {
            name: "Other".into(),
            paths: (1..=120)
                .map(|n| std::path::PathBuf::from(format!("/music/Other {n}.mp3")))
                .collect(),
        },
    )
    .unwrap();
    let other = s.playlists.iter().nth(1).unwrap().clone();
    let (mut h, fake) = harness(s);
    play(&fake, 0, other.entries[100].id);
    h.run_steps(6);
    let p = fake.player(0);
    assert!(
        fake.take_sent()
            .contains(&Command::ShowPlaylist(p, other.id)),
        "the tab follows"
    );
    assert!(visible(&h, "Other 101"));
}

#[test]
fn each_player_follows_its_own_current() {
    let (mut h, fake) = harness(state(2, 200));
    let e = entries(&fake.state.load_full());
    play(&fake, 0, e[150]);
    play(&fake, 1, e[20]);
    h.run_steps(4);
    assert!(visible(&h, "Song 151"));
    assert!(visible(&h, "Song 21"));
    let _ = pos2(0.0, 0.0);
}
