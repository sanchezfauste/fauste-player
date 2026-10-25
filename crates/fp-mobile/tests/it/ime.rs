#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The IME bridge: the text the Android keyboard holds becomes egui key
//! and text events (mobile spec §11: winit drops GameActivity's text).

use egui::{Event, Key};
use fp_mobile::ime::ImeBridge;

fn backspaces(events: &[Event]) -> usize {
    events
        .iter()
        .filter(|e| {
            matches!(
                e,
                Event::Key {
                    key: Key::Backspace,
                    pressed: true,
                    ..
                }
            )
        })
        .count()
}

fn texts(events: &[Event]) -> Vec<String> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::Text(t) => Some(t.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn typing_adds_text() {
    let mut ime = ImeBridge::new();
    let events = ime.update("Hola");
    assert_eq!(backspaces(&events), 0);
    assert_eq!(texts(&events), vec!["Hola".to_owned()]);
}

#[test]
fn deleting_sends_backspaces() {
    let mut ime = ImeBridge::new();
    ime.update("Hola");
    let events = ime.update("Ho");
    assert_eq!(backspaces(&events), 2);
    assert!(texts(&events).is_empty());
}

#[test]
fn replacing_inside_the_text() {
    let mut ime = ImeBridge::new();
    ime.update("teh cat");
    let events = ime.update("the cat");
    assert_eq!(backspaces(&events), 6);
    assert_eq!(texts(&events), vec!["he cat".to_owned()]);
    // The backspaces come before the text.
    let last_backspace = events
        .iter()
        .rposition(|e| matches!(e, Event::Key { .. }))
        .unwrap();
    let text = events
        .iter()
        .position(|e| matches!(e, Event::Text(_)))
        .unwrap();
    assert!(last_backspace < text);
}

#[test]
fn non_ascii_counts_characters() {
    let mut ime = ImeBridge::new();
    ime.update("año");
    let events = ime.update("añ");
    assert_eq!(backspaces(&events), 1);
    assert!(texts(&events).is_empty());
}

#[test]
fn reset_emits_nothing() {
    let mut ime = ImeBridge::new();
    ime.reset("abc");
    assert!(ime.update("abc").is_empty());
}

fn tap() -> Event {
    Event::PointerButton {
        pos: egui::pos2(10.0, 10.0),
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::NONE,
    }
}

#[test]
fn another_field_restarts_the_keyboard_text() {
    let mut ime = ImeBridge::new();
    let a = Some(egui::Id::unique("a"));
    let b = Some(egui::Id::unique("b"));
    assert!(ime.restart_for(a, &[]));
    ime.update("hello");
    assert!(!ime.restart_for(a, &[]));
    assert!(ime.restart_for(b, &[]));
    // The keyboard was cleared: what it holds now is new typing only.
    assert_eq!(texts(&ime.update(" ")), vec![" ".to_owned()]);
    assert_eq!(backspaces(&ime.update("x")), 1);
}

#[test]
fn a_tap_inside_the_field_restarts_the_keyboard_text() {
    // The cursor may have moved: the keyboard's text no longer matches
    // what is before it.
    let mut ime = ImeBridge::new();
    let a = Some(egui::Id::unique("a"));
    ime.restart_for(a, &[]);
    ime.update("abc def");
    assert!(ime.restart_for(a, &[tap()]));
    let events = ime.update("x");
    assert_eq!(backspaces(&events), 0);
    assert_eq!(texts(&events), vec!["x".to_owned()]);
}

#[test]
fn no_field_needs_no_keyboard_text() {
    let mut ime = ImeBridge::new();
    assert!(!ime.restart_for(None, &[tap()]));
}
