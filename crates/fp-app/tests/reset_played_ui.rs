#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O22: the Reset played button and its question.

mod support;

use std::sync::Arc;

use egui::{Event, Key, Modifiers};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use fp_app::ui::app::AppUi;
use fp_model::{AppState, Command, PlaylistId, apply};
use support::{Fake, harness, state};

const BUTTON: &str = "Clear the played marks of this playlist";
const QUESTION: &str = "Clear the played mark of every track in this playlist?";

/// The model with the first `n` entries played on P1 (and P1 stopped).
fn played(n: usize) -> AppState {
    let mut s = state(1, 5);
    let p = s.players[0].id;
    let e: Vec<_> = s
        .playlists
        .iter()
        .next()
        .unwrap()
        .entries
        .iter()
        .map(|e| e.id)
        .collect();
    for entry in e.iter().take(n) {
        apply(&mut s, Command::SetNext(p, *entry)).unwrap();
        apply(&mut s, Command::Play(p)).unwrap();
        apply(&mut s, Command::Stop(p)).unwrap();
    }
    s
}

fn playlist(fake: &Fake) -> PlaylistId {
    fake.state.load().playlists.first_id().unwrap()
}

fn button_enabled(h: &Harness<'_, AppUi>) -> bool {
    !h.get_by_label(BUTTON).accesskit_node().is_disabled()
}

fn press(h: &mut Harness<'_, AppUi>, key: Key) {
    h.event(Event::Key {
        key,
        physical_key: Some(key),
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    });
    h.event(Event::Key {
        key,
        physical_key: Some(key),
        pressed: false,
        repeat: false,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(2);
}

fn open(h: &mut Harness<'_, AppUi>) {
    h.get_by_label(BUTTON).click();
    h.run_steps(3);
}

fn marked(fake: &Arc<Fake>) -> usize {
    let s = fake.state.load();
    s.playlists
        .iter()
        .next()
        .unwrap()
        .entries
        .iter()
        .filter(|e| !e.played_by.is_empty())
        .count()
}

#[test]
fn the_button_is_off_while_nothing_is_played() {
    let (h, _) = harness(state(1, 5));
    assert!(!button_enabled(&h));
}

#[test]
fn the_button_is_on_when_a_mark_can_be_cleared() {
    let (h, _) = harness(played(2));
    assert!(button_enabled(&h));
}

#[test]
fn pressing_it_asks_first_and_sends_nothing() {
    let (mut h, fake) = harness(played(2));
    fake.take_sent();
    open(&mut h);
    assert!(h.query_by_label(QUESTION).is_some());
    assert!(fake.take_sent().is_empty());
    assert_eq!(marked(&fake), 2);
}

#[test]
fn confirming_resets_the_playlist_and_closes_the_question() {
    let (mut h, fake) = harness(played(2));
    fake.take_sent();
    open(&mut h);
    h.get_by_label("Reset played").click();
    h.run_steps(3);
    assert_eq!(
        fake.take_sent(),
        vec![Command::ResetPlayed(playlist(&fake))]
    );
    assert!(h.query_by_label(QUESTION).is_none());
    assert_eq!(marked(&fake), 0);
    assert!(!button_enabled(&h), "nothing left to clear");
}

#[test]
fn cancel_changes_nothing() {
    let (mut h, fake) = harness(played(2));
    fake.take_sent();
    open(&mut h);
    h.get_by_label("Cancel").click();
    h.run_steps(3);
    assert!(h.query_by_label(QUESTION).is_none());
    assert!(fake.take_sent().is_empty());
    assert_eq!(marked(&fake), 2);
}

#[test]
fn escape_cancels() {
    let (mut h, fake) = harness(played(2));
    fake.take_sent();
    open(&mut h);
    press(&mut h, Key::Escape);
    assert!(h.query_by_label(QUESTION).is_none());
    assert!(fake.take_sent().is_empty());
}

#[test]
fn no_shortcut_acts_under_the_question() {
    let (mut h, fake) = harness(played(2));
    h.get_by_label("Song 4").click();
    h.run_steps(2);
    open(&mut h);
    fake.take_sent();
    press(&mut h, Key::Delete);
    press(&mut h, Key::Space);
    assert!(h.query_by_label(QUESTION).is_some());
    assert!(fake.take_sent().is_empty(), "Delete and Space did nothing");
}

#[test]
fn the_question_closes_when_the_playlist_is_deleted() {
    let mut s = played(2);
    apply(
        &mut s,
        Command::CreatePlaylist {
            name: "Other".into(),
        },
    )
    .unwrap();
    let (mut h, fake) = harness(s);
    open(&mut h);
    let main = playlist(&fake);
    let mut next = (*fake.state.load_full()).clone();
    apply(&mut next, Command::DeletePlaylist(main)).unwrap();
    fake.state.store(Arc::new(next));
    h.run_steps(3);
    assert!(h.query_by_label(QUESTION).is_none());
    assert!(
        !fake
            .take_sent()
            .iter()
            .any(|c| matches!(c, Command::ResetPlayed(_)))
    );
}

#[test]
fn an_unplayed_current_entry_alone_leaves_the_button_off() {
    let mut s = state(1, 5);
    let p = s.players[0].id;
    apply(&mut s, Command::Play(p)).unwrap();
    let (h, _) = harness(s);
    assert!(!button_enabled(&h));
}
