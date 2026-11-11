#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Automation spec §8.1: the music filler's Music column.

use crate::support;

use egui_kittest::kittest::Queryable;
use fp_model::{AppState, Command, PlayerId, RotationStep, StepSource, TrackKind, apply};
use support::{harness, state};

/// Four players, three tracks, the default rotation and the filler.
fn with_filler(tracks: usize) -> AppState {
    let mut s = state(4, tracks);
    fp_model::rotation::ensure_default(&mut s, "Music");
    fp_model::filler::ensure(&mut s, "Music filler");
    let _ = apply(&mut s, Command::SetLayoutLocked(false));
    s
}

fn filler(s: &AppState) -> PlayerId {
    s.filler.as_ref().unwrap().player
}

fn shown(s: AppState) -> AppState {
    let mut s = s;
    let f = filler(&s);
    s.layout.hidden_players.retain(|p| *p != f);
    s
}

#[test]
fn the_music_column_is_hidden_until_shown_from_the_top_bar() {
    let s = with_filler(3);
    let f = filler(&s);
    let (mut h, _fake) = harness(s);
    assert!(h.state().player_rects().iter().all(|(p, _)| *p != f));
    h.get_by_label("Show Music").click();
    h.run();
    let rects = h.state().player_rects();
    assert_eq!(rects.first().map(|(p, _)| *p), Some(f), "left of P1");
}

#[test]
fn the_auxiliary_players_keep_their_numbers() {
    let (h, _fake) = harness(shown(with_filler(3)));
    for n in 1..=4 {
        assert!(h.query_by_label(&format!("Hide P{n}")).is_some(), "P{n}");
    }
    assert!(h.query_by_label("Hide Music").is_some());
    assert!(h.query_by_label("Hide P5").is_none());
}

#[test]
fn the_rotation_selector_sends_set_filler_rotation() {
    let mut s = shown(with_filler(3));
    apply(
        &mut s,
        Command::CreateRotation {
            name: "Night".into(),
            steps: vec![RotationStep {
                source: StepSource::Kind(TrackKind::Music),
                min: 1,
                max: 1,
            }],
        },
    )
    .unwrap();
    let night = s.rotations.iter().last().unwrap().id;
    let (mut h, fake) = harness(s);
    h.get_by_label("Rotation").click();
    h.run();
    h.get_by_label("Night").click();
    h.run();
    assert!(
        fake.take_sent()
            .contains(&Command::SetFillerRotation(Some(night)))
    );
}

#[test]
fn a_starved_filler_shows_the_alert() {
    let (h, _fake) = harness(shown(with_filler(0)));
    assert!(
        h.query_by_label_contains("Nothing to play").is_some(),
        "the alert is shown"
    );
}

#[test]
fn a_manual_row_carries_the_hand_marker() {
    let mut s = shown(with_filler(3));
    let list = s.filler.as_ref().unwrap().playlist;
    let track = s.library.ids()[0];
    apply(
        &mut s,
        Command::InsertTracks {
            playlist: list,
            index: 0,
            tracks: vec![track],
        },
    )
    .unwrap();
    let (h, _fake) = harness(s);
    assert_eq!(h.get_all_by_label("Added by hand").count(), 1);
}

#[test]
fn settings_playlists_leaves_the_filler_list_out() {
    let s = with_filler(3);
    let (mut h, _) = harness(s);
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(egui::accesskit::Role::Button, "Playlists")
        .click();
    h.run_steps(3);
    assert!(h.query_by_value("Music filler").is_none());
}
