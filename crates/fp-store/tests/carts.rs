#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Phase 2 spec P2.7: carts.json and the cartwall part of the session.

use std::fs;
use std::path::PathBuf;

use fp_model::{AppState, CartColor, CartEdit, CartKind, Command, Config, apply};
use fp_store::{AppPaths, Store};

fn store(dir: &tempfile::TempDir) -> Store {
    Store::new(AppPaths::under(dir.path()), Config::default().limits)
}

fn edited_state() -> AppState {
    let mut state = AppState::new(Config::default(), "Main");
    apply(
        &mut state,
        Command::CreateCartPage {
            name: "Effects".into(),
        },
    )
    .unwrap();
    let page = state.cartwall.pages[1].id;
    apply(
        &mut state,
        Command::AssignCartFile {
            page,
            index: 3,
            path: PathBuf::from("/carts/applause.wav"),
        },
    )
    .unwrap();
    let edit = CartEdit {
        name: "Applause".into(),
        kind: CartKind::Effect,
        looped: true,
        exclusive: true,
        color: Some(CartColor::Teal),
    };
    apply(
        &mut state,
        Command::SetCart {
            page,
            index: 3,
            edit,
        },
    )
    .unwrap();
    apply(&mut state, Command::ShowCartPage(page)).unwrap();
    apply(&mut state, Command::SetCartwallOpen(false)).unwrap();
    state
}

fn save_all(s: &Store, state: &AppState) {
    s.save_config(state).unwrap();
    s.save_playlists(state).unwrap();
    s.save_carts(state).unwrap();
    s.save_session(state, |_| 0.0).unwrap();
}

#[test]
fn carts_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
    let mut state = edited_state();
    let cart = state.cartwall.pages[1].carts[3].id;
    apply(&mut state, Command::FireCart(cart)).unwrap();
    save_all(&s, &state);
    let loaded = s.load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    let wall = &loaded.state.cartwall;
    assert_eq!(wall.pages, state.cartwall.pages);
    let track = wall.pages[1].carts[3].track.unwrap();
    assert_eq!(
        loaded.state.library.get(track).unwrap().path,
        PathBuf::from("/carts/applause.wav")
    );
    assert_eq!(wall.shown, Some(state.cartwall.pages[1].id));
    assert!(!wall.open);
    assert!(wall.playing.is_empty(), "C7: nothing plays after a restart");
    // New ids never collide with loaded ones.
    let mut restored = loaded.state;
    apply(
        &mut restored,
        Command::CreateCartPage { name: "New".into() },
    )
    .unwrap();
    let ids: std::collections::HashSet<u64> = restored
        .cartwall
        .pages
        .iter()
        .flat_map(|p| std::iter::once(p.id.0).chain(p.carts.iter().map(|c| c.id.0)))
        .collect();
    let count: usize = restored
        .cartwall
        .pages
        .iter()
        .map(|p| p.carts.len() + 1)
        .sum();
    assert_eq!(ids.len(), count);
}

#[test]
fn a_corrupt_carts_file_falls_back_to_the_backup() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
    let state = edited_state();
    save_all(&s, &state);
    s.save_carts(&state).unwrap(); // the previous version becomes .bak1
    fs::write(s.paths().carts_file(), b"{ not json").unwrap();
    let loaded = s.load("Main");
    assert_eq!(loaded.state.cartwall.pages, state.cartwall.pages);
    assert!(
        loaded.warnings.iter().any(|w| w.contains("carts")),
        "{:?}",
        loaded.warnings
    );
}

#[test]
fn missing_carts_file_gives_one_default_page() {
    let dir = tempfile::tempdir().unwrap();
    let loaded = store(&dir).load("Main");
    let wall = &loaded.state.cartwall;
    assert_eq!(wall.pages.len(), 1);
    assert_eq!(wall.pages[0].carts.len(), 16);
    assert!(wall.open);
}

#[test]
fn session_without_cartwall_fields_still_loads() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
    fs::create_dir_all(&s.paths().data_dir).unwrap();
    fs::write(
        s.paths().session_file(),
        br#"{ "schema_version": 1, "players": [] }"#,
    )
    .unwrap();
    let loaded = s.load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert!(loaded.state.cartwall.open);
}

#[test]
fn carts_pointing_at_unknown_tracks_become_empty() {
    let dir = tempfile::tempdir().unwrap();
    let s = store(&dir);
    let state = edited_state();
    save_all(&s, &state);
    // A playlists.json from elsewhere, without the cart's track.
    let fresh = AppState::new(Config::default(), "Main");
    s.save_playlists(&fresh).unwrap();
    let loaded = s.load("Main");
    assert!(loaded.state.cartwall.pages[1].carts[3].track.is_none());
    assert!(
        loaded.warnings.iter().any(|w| w.contains("lost its file")),
        "{:?}",
        loaded.warnings
    );
}

/// Cart colour spec CC3: a hand edit that breaks a colour costs only that
/// colour, with a warning; the cart and the rest of the file load.
#[test]
fn cc3_an_unknown_colour_in_carts_json_loads_as_none_with_a_warning() {
    for bad in [r#""magenta""#, "7"] {
        let dir = tempfile::tempdir().unwrap();
        let s = store(&dir);
        let state = edited_state();
        save_all(&s, &state);
        let file = s.paths().carts_file();
        let text = fs::read_to_string(&file).unwrap();
        assert_eq!(
            text.matches("\"color\"").count(),
            1,
            "only the coloured cart: {text}"
        );
        fs::write(&file, text.replace(r#""teal""#, bad)).unwrap();
        let loaded = s.load("Main");
        let cart = &loaded.state.cartwall.pages[1].carts[3];
        assert_eq!(cart.color, None, "{bad}");
        assert_eq!(
            (cart.name.as_str(), cart.kind, cart.looped, cart.exclusive),
            ("Applause", CartKind::Effect, true, true),
            "{bad}"
        );
        assert!(cart.track.is_some(), "{bad}: the file is kept");
        let notes: Vec<&String> = loaded
            .warnings
            .iter()
            .filter(|w| w.contains("colour"))
            .collect();
        assert_eq!(notes.len(), 1, "{bad}: {:?}", loaded.warnings);
        assert!(notes[0].contains(bad), "{bad}: {}", notes[0]);
    }
}
