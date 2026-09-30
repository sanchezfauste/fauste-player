#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Phase 2 spec P2.5: keyboard shortcuts.

use fp_model::{
    AppState, Command, Config, KeyChord, Shortcut, ShortcutAction, apply, default_shortcuts,
};

fn chord_of(shortcuts: &[Shortcut], action: ShortcutAction) -> Option<KeyChord> {
    shortcuts
        .iter()
        .find(|s| s.action == action)
        .map(|s| s.chord.clone())
}

#[test]
fn default_shortcuts_play_players_and_fire_carts() {
    let d = default_shortcuts();
    assert_eq!(
        chord_of(&d, ShortcutAction::PlayPlayer(1)),
        Some(KeyChord::key("1"))
    );
    assert_eq!(
        chord_of(&d, ShortcutAction::PlayPlayer(9)),
        Some(KeyChord::key("9"))
    );
    assert_eq!(
        chord_of(&d, ShortcutAction::FireCart(1)),
        Some(KeyChord::key("F1"))
    );
    assert_eq!(
        chord_of(&d, ShortcutAction::FireCart(12)),
        Some(KeyChord::key("F12"))
    );
    let stop_all = chord_of(&d, ShortcutAction::StopAllCarts).unwrap();
    assert_eq!(stop_all.to_string(), "Ctrl+Space");
    assert_eq!(Config::default().shortcuts, d);
}

#[test]
fn rebinding_a_used_chord_moves_it() {
    let mut state = AppState::new(Config::default(), "Main");
    apply(
        &mut state,
        Command::SetShortcut {
            action: ShortcutAction::PausePlayer(1),
            chord: Some(KeyChord::key("1")),
        },
    )
    .unwrap();
    let s = &state.config.shortcuts;
    assert_eq!(
        chord_of(s, ShortcutAction::PausePlayer(1)),
        Some(KeyChord::key("1"))
    );
    assert_eq!(
        chord_of(s, ShortcutAction::PlayPlayer(1)),
        None,
        "moved away"
    );
}

#[test]
fn unbinding_removes_the_shortcut_and_reset_restores_defaults() {
    let mut state = AppState::new(Config::default(), "Main");
    apply(
        &mut state,
        Command::SetShortcut {
            action: ShortcutAction::PlayPlayer(2),
            chord: None,
        },
    )
    .unwrap();
    assert_eq!(
        chord_of(&state.config.shortcuts, ShortcutAction::PlayPlayer(2)),
        None
    );
    apply(&mut state, Command::ResetShortcuts).unwrap();
    assert_eq!(state.config.shortcuts, default_shortcuts());
}

#[test]
fn duplicate_chords_keep_the_first_binding() {
    let mut config = Config {
        shortcuts: vec![
            Shortcut {
                action: ShortcutAction::PlayPlayer(1),
                chord: KeyChord::key("A"),
            },
            Shortcut {
                action: ShortcutAction::StopPlayer(1),
                chord: KeyChord::key("A"),
            },
            Shortcut {
                action: ShortcutAction::PlayPlayer(1),
                chord: KeyChord::key("B"),
            },
        ],
        ..Config::default()
    };
    let warnings = config.validate();
    assert_eq!(config.shortcuts.len(), 1);
    assert_eq!(config.shortcuts[0].action, ShortcutAction::PlayPlayer(1));
    assert_eq!(
        warnings.iter().filter(|w| w.field == "shortcuts").count(),
        2
    );
}

#[test]
fn chords_display_with_their_modifiers() {
    let chord = KeyChord {
        key: "F5".into(),
        ctrl: true,
        alt: true,
        shift: true,
        command: false,
    };
    assert_eq!(chord.to_string(), "Ctrl+Alt+Shift+F5");
}

#[test]
fn shortcuts_round_trip_through_json() {
    let d = default_shortcuts();
    let json = serde_json::to_string(&d).unwrap();
    let back: Vec<Shortcut> = serde_json::from_str(&json).unwrap();
    assert_eq!(back, d);
}

#[test]
fn position_zero_is_not_a_valid_target() {
    let mut config = Config {
        shortcuts: vec![
            Shortcut {
                action: ShortcutAction::PlayPlayer(0),
                chord: KeyChord::key("A"),
            },
            Shortcut {
                action: ShortcutAction::FireCart(0),
                chord: KeyChord::key("B"),
            },
            Shortcut {
                action: ShortcutAction::FireCart(1),
                chord: KeyChord::key("C"),
            },
        ],
        ..Config::default()
    };
    config.validate();
    assert_eq!(config.shortcuts.len(), 1);
}

#[test]
fn restart_and_previous_have_no_default_key() {
    let d = default_shortcuts();
    for action in [
        ShortcutAction::RestartPlayer(1),
        ShortcutAction::PreviousPlayer(1),
    ] {
        assert_eq!(chord_of(&d, action), None, "{action:?}");
        assert_eq!(action.position(), Some(1));
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(
            serde_json::from_str::<ShortcutAction>(&json).unwrap(),
            action
        );
    }
}
