#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Keyboard on the playlist: Up/Down select, Enter sets Next, a second
//! Enter within the double-click delay plays (itr06 I24).

use crate::support;

use egui::Key;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_model::Command;
use support::{Fake, harness, state};

type H = Harness<'static, AppUi>;

/// The entry an Enter would set as Next: it shows the selection. The
/// press is let go of afterwards, past the double-click delay.
fn selected(h: &mut H, fake: &Fake) -> Option<fp_model::EntryId> {
    fake.take_sent();
    h.key_press(Key::Enter);
    h.run_steps(2);
    let found = fake.take_sent().into_iter().find_map(|c| match c {
        Command::SetNext(_, e) => Some(e),
        _ => None,
    });
    h.run_steps(25);
    found
}

/// Clicks the row of "Song n" (the player's header may name it too).
fn click_row(h: &mut H, n: usize) {
    h.get_all_by_label(&format!("Song {n}"))
        .last()
        .unwrap()
        .click();
    h.run_steps(2);
}

/// A row clicked, so the player is the active one, then no selection.
fn with_active_player(h: &mut H, fake: &Fake) {
    click_row(h, 1);
    h.key_press(Key::Escape);
    h.run_steps(2);
    fake.take_sent();
}

fn press(h: &mut H, key: Key) {
    h.key_press(key);
    h.run_steps(1);
}

#[test]
fn down_without_a_selection_selects_the_next_entry_else_the_first() {
    let (mut h, fake) = harness(state(1, 4));
    let e = fake.entries();
    with_active_player(&mut h, &fake);
    let p = fake.player(0);
    // The model's default next is the first entry; make it another one.
    let mut s = (**fake.state.load()).clone();
    fp_model::apply(&mut s, Command::SetNext(p, e[2])).unwrap();
    fake.state.store(std::sync::Arc::new(s));
    h.run_steps(2);
    press(&mut h, Key::ArrowDown);
    assert_eq!(selected(&mut h, &fake), Some(e[2]), "the Next entry");
}

#[test]
fn up_and_down_move_the_selection_and_stop_at_the_ends() {
    let (mut h, fake) = harness(state(1, 3));
    let e = fake.entries();
    click_row(&mut h, 2);
    press(&mut h, Key::ArrowDown);
    press(&mut h, Key::ArrowDown);
    press(&mut h, Key::ArrowDown);
    assert_eq!(selected(&mut h, &fake), Some(e[2]), "stops at the last");
    click_row(&mut h, 3);
    press(&mut h, Key::ArrowUp);
    assert_eq!(selected(&mut h, &fake), Some(e[1]));
    click_row(&mut h, 1);
    press(&mut h, Key::ArrowUp);
    assert_eq!(selected(&mut h, &fake), Some(e[0]), "stops at the first");
}

#[test]
fn the_selection_scrolls_into_view() {
    let (mut h, fake) = harness(state(1, 60));
    click_row(&mut h, 1);
    for _ in 0..40 {
        press(&mut h, Key::ArrowDown);
    }
    h.run_steps(3);
    assert!(h.query_by_label("Song 41").is_some(), "row 41 is built");
    assert!(h.query_by_label("Song 2").is_none(), "scrolled away");
    drop(fake);
}

#[test]
fn enter_sets_the_selected_entry_as_next() {
    let (mut h, fake) = harness(state(1, 3));
    let e = fake.entries();
    click_row(&mut h, 2);
    fake.take_sent();
    press(&mut h, Key::Enter);
    assert_eq!(
        fake.take_sent(),
        vec![Command::SetNext(fake.player(0), e[1])]
    );
}

#[test]
fn a_second_enter_within_the_double_click_delay_plays() {
    let (mut h, fake) = harness(state(1, 3));
    let e = fake.entries();
    click_row(&mut h, 2);
    fake.take_sent();
    press(&mut h, Key::Enter);
    h.run_steps(3);
    press(&mut h, Key::Enter);
    assert_eq!(
        fake.take_sent(),
        vec![
            Command::SetNext(fake.player(0), e[1]),
            Command::Play(fake.player(0))
        ]
    );
}

#[test]
fn two_enters_further_apart_than_the_delay_do_not_play() {
    let (mut h, fake) = harness(state(1, 3));
    let e = fake.entries();
    click_row(&mut h, 2);
    fake.take_sent();
    press(&mut h, Key::Enter);
    // 0.02 s a step: 25 steps are 0.5 s, past egui's 0.3 s.
    h.run_steps(25);
    press(&mut h, Key::Enter);
    assert_eq!(
        fake.take_sent(),
        vec![
            Command::SetNext(fake.player(0), e[1]),
            Command::SetNext(fake.player(0), e[1])
        ],
        "a first Enter again"
    );
}

#[test]
fn another_key_between_two_enters_breaks_the_pair() {
    let (mut h, fake) = harness(state(1, 3));
    click_row(&mut h, 2);
    fake.take_sent();
    press(&mut h, Key::Enter);
    press(&mut h, Key::A);
    press(&mut h, Key::Enter);
    assert!(
        !fake.take_sent().contains(&Command::Play(fake.player(0))),
        "no play"
    );
}

#[test]
fn moving_the_selection_between_two_enters_breaks_the_pair() {
    let (mut h, fake) = harness(state(1, 3));
    let e = fake.entries();
    click_row(&mut h, 1);
    fake.take_sent();
    press(&mut h, Key::Enter);
    press(&mut h, Key::ArrowDown);
    press(&mut h, Key::ArrowUp);
    press(&mut h, Key::Enter);
    let sent = fake.take_sent();
    assert!(!sent.contains(&Command::Play(fake.player(0))), "{sent:?}");
    assert_eq!(sent.last(), Some(&Command::SetNext(fake.player(0), e[0])));
}

#[test]
fn a_click_between_two_enters_breaks_the_pair() {
    let (mut h, fake) = harness(state(1, 3));
    click_row(&mut h, 2);
    fake.take_sent();
    press(&mut h, Key::Enter);
    click_row(&mut h, 2);
    press(&mut h, Key::Enter);
    assert!(!fake.take_sent().contains(&Command::Play(fake.player(0))));
}

#[test]
fn enter_without_a_selection_does_nothing() {
    let (mut h, fake) = harness(state(1, 3));
    with_active_player(&mut h, &fake);
    press(&mut h, Key::Enter);
    assert!(fake.take_sent().is_empty());
}

#[test]
fn the_keys_are_ignored_while_editing_text() {
    let fake = Fake::new(state(1, 3));
    let app = AppUi::new(
        fake.clone(),
        fp_app::i18n::I18n::new(Some("en-US")),
        fp_app::services::MediaCache::default(),
    );
    let mut h = Harness::builder()
        .with_size(egui::vec2(1000.0, 700.0))
        .with_step_dt(0.02)
        .build_ui_state(
            |ui, (app, text): &mut (AppUi, String)| {
                // After the interface, as the app's own fields are.
                app.ui(ui);
                ui.add(
                    egui::TextEdit::singleline(text)
                        .id_salt("editor")
                        .hint_text("Editor"),
                );
            },
            (app, String::new()),
        );
    h.run_steps(2);
    // Select a row with the keyboard first.
    h.key_press(Key::ArrowDown);
    h.run_steps(2);
    fake.take_sent();
    h.get_by_role(egui::accesskit::Role::TextInput).focus();
    h.run_steps(1);
    h.key_press(Key::ArrowDown);
    h.key_press(Key::Enter);
    h.run_steps(2);
    let sent = fake.take_sent();
    assert!(sent.is_empty(), "{sent:?}");
}

#[test]
fn a_shortcut_bound_to_enter_wins() {
    let mut s = state(1, 3);
    s.config.shortcuts = vec![fp_model::Shortcut {
        action: fp_model::ShortcutAction::PausePlayer(1),
        chord: fp_model::KeyChord::key("Enter"),
    }];
    let (mut h, fake) = harness(s);
    click_row(&mut h, 2);
    fake.take_sent();
    press(&mut h, Key::Enter);
    let sent = fake.take_sent();
    assert!(
        !sent.iter().any(|c| matches!(c, Command::SetNext(..))),
        "{sent:?}"
    );
}

#[test]
fn a_modal_owns_the_keyboard() {
    let (mut h, fake) = harness(state(1, 3));
    click_row(&mut h, 2);
    h.get_by_label("Settings").click();
    h.run_steps(2);
    fake.take_sent();
    press(&mut h, Key::Enter);
    press(&mut h, Key::ArrowDown);
    assert!(fake.take_sent().is_empty());
}

/// Changes the model behind the interface, as the services thread would.
fn change(fake: &Fake, f: impl FnOnce(&mut fp_model::AppState)) {
    let mut s = (**fake.state.load()).clone();
    f(&mut s);
    fake.state.store(std::sync::Arc::new(s));
}

#[test]
fn a_second_enter_does_not_play_once_the_next_entry_changed() {
    let (mut h, fake) = harness(state(1, 3));
    let e = fake.entries();
    click_row(&mut h, 2);
    press(&mut h, Key::Enter);
    let p = fake.player(0);
    change(&fake, |s| {
        fp_model::apply(s, Command::SetNext(p, e[2])).unwrap();
    });
    fake.take_sent();
    press(&mut h, Key::Enter);
    let sent = fake.take_sent();
    assert!(!sent.contains(&Command::Play(p)), "{sent:?}");
    assert_eq!(sent, vec![Command::SetNext(p, e[1])], "a first Enter again");
}

#[test]
fn a_second_enter_does_not_play_once_the_current_entry_changed() {
    let (mut h, fake) = harness(state(1, 3));
    let e = fake.entries();
    click_row(&mut h, 2);
    press(&mut h, Key::Enter);
    let p = fake.player(0);
    // Another track went on air since the first Enter; Song 2 is Next again.
    change(&fake, |s| {
        let i = s.player_index(p).unwrap();
        s.players[i].current = Some(e[0]);
        s.players[i].next = Some(e[1]);
    });
    fake.take_sent();
    press(&mut h, Key::Enter);
    assert!(!fake.take_sent().contains(&Command::Play(p)));
}

#[test]
fn a_held_enter_never_plays() {
    let (mut h, fake) = harness(state(1, 3));
    click_row(&mut h, 2);
    fake.take_sent();
    // Held down: no release until the end.
    h.key_down(Key::Enter);
    h.run_steps(1);
    for _ in 0..4 {
        h.event(egui::Event::Key {
            key: Key::Enter,
            physical_key: Some(Key::Enter),
            pressed: true,
            repeat: true,
            modifiers: egui::Modifiers::NONE,
        });
        h.run_steps(1);
    }
    let sent = fake.take_sent();
    assert!(!sent.contains(&Command::Play(fake.player(0))), "{sent:?}");
    assert_eq!(sent.len(), 1, "{sent:?}");
}

#[test]
fn enter_does_nothing_when_the_active_player_is_hidden() {
    let (mut h, fake) = harness(state(2, 3));
    click_row(&mut h, 2);
    // The last "Song 2" is on the second player.
    let p = fake.player(1);
    change(&fake, |s| {
        fp_model::apply(
            s,
            Command::SetPanelHidden {
                panel: fp_model::Panel::Player(p),
                hidden: true,
            },
        )
        .unwrap();
    });
    h.run_steps(2);
    fake.take_sent();
    press(&mut h, Key::Enter);
    assert!(fake.take_sent().is_empty());
}

#[test]
fn an_open_menu_keeps_enter() {
    let (mut h, fake) = harness(state(1, 3));
    click_row(&mut h, 2);
    h.get_all_by_label("Song 2")
        .last()
        .unwrap()
        .click_secondary();
    h.run_steps(2);
    fake.take_sent();
    press(&mut h, Key::Enter);
    let sent = fake.take_sent();
    assert!(
        !sent.iter().any(|c| matches!(c, Command::SetNext(..))),
        "{sent:?}"
    );
}
