#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Phase 2 spec P2.7: the cart pages and the cartwall part of the session, in the database.

use std::path::PathBuf;

use fp_model::{AppState, CartColor, CartEdit, CartKind, Command, Config, apply};
use fp_store::{AppPaths, Store};

use crate::support::{edit_db, edit_doc, save_all};

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

#[test]
fn carts_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let mut state = edited_state();
    let cart = state.cartwall.pages[1].carts[3].id;
    apply(&mut state, Command::FireCart(cart)).unwrap();
    save_all(&mut s, &state, 0.0);
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
fn a_cart_page_row_that_does_not_parse_is_dropped_with_a_warning() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let state = edited_state();
    save_all(&mut s, &state, 0.0);
    let (first, second) = (state.cartwall.pages[0].id.0, state.cartwall.pages[1].id.0);
    edit_db(&s, |db| {
        db.conn()
            .execute(
                "UPDATE cart_pages SET doc = '{ not json' WHERE id = ?1",
                [second as i64],
            )
            .unwrap();
    });
    let loaded = s.load("Main");
    let ids: Vec<u64> = loaded.state.cartwall.pages.iter().map(|p| p.id.0).collect();
    assert!(ids.contains(&first), "the other page is kept");
    assert!(!ids.contains(&second));
    assert!(
        loaded.warnings.iter().any(|w| w.contains("carts")),
        "{:?}",
        loaded.warnings
    );
}

#[test]
fn an_empty_database_gives_one_default_cart_page() {
    let dir = tempfile::tempdir().unwrap();
    let loaded = store(&dir).load("Main");
    let wall = &loaded.state.cartwall;
    assert_eq!(wall.pages.len(), 1);
    assert_eq!(wall.pages[0].carts.len(), 16);
    assert!(wall.open);
}

#[test]
fn a_database_without_a_cartwall_session_still_loads() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    save_all(&mut s, &edited_state(), 0.0);
    edit_db(&s, |db| {
        db.conn()
            .execute("DELETE FROM kv WHERE key = 'cartwall_session'", [])
            .unwrap();
    });
    let loaded = s.load("Main");
    assert!(loaded.warnings.is_empty(), "{:?}", loaded.warnings);
    assert!(loaded.state.cartwall.open);
}

#[test]
fn carts_pointing_at_unknown_tracks_become_empty() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = store(&dir);
    let state = edited_state();
    save_all(&mut s, &state, 0.0);
    // The cart's track is gone from the library.
    edit_db(&s, |db| {
        db.conn().execute("DELETE FROM tracks", []).unwrap();
    });
    let loaded = s.load("Main");
    assert!(loaded.state.cartwall.pages[1].carts[3].track.is_none());
    assert!(
        loaded.warnings.iter().any(|w| w.contains("lost its file")),
        "{:?}",
        loaded.warnings
    );
}

/// Cart colour spec CC3: a hand edit that breaks a colour costs only that
/// colour, with a warning; the cart and the rest of the page load.
#[test]
fn cc3_an_unknown_colour_in_a_cart_page_row_loads_as_none_with_a_warning() {
    for bad in [r#""magenta""#, "7"] {
        let dir = tempfile::tempdir().unwrap();
        let mut s = store(&dir);
        let state = edited_state();
        save_all(&mut s, &state, 0.0);
        let page = state.cartwall.pages[1].id.0;
        edit_doc(&s, "cart_pages", page, |doc| {
            let carts = doc["carts"].as_array_mut().unwrap();
            let coloured: Vec<_> = carts.iter_mut().filter(|c| !c["color"].is_null()).collect();
            assert_eq!(coloured.len(), 1, "only the coloured cart");
            let bad: serde_json::Value = serde_json::from_str(bad).unwrap();
            coloured.into_iter().for_each(|c| c["color"] = bad.clone());
        });
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
