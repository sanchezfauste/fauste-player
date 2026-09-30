#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Phase 2 spec P2.3: cartwall rules C1–C10.

mod common;

use std::path::PathBuf;

use common::{entries, fixture};
use fp_model::{
    AppState, CartEdit, CartId, CartKind, CartPageId, CartRequest, Command, EngineAction,
    EngineEvent, FileState, ModelError, SOURCE_END, TrackAnalysis, apply, on_event,
};

fn page(state: &AppState) -> CartPageId {
    state.cartwall.pages[0].id
}

fn cart(state: &AppState, index: usize) -> CartId {
    state.cartwall.pages[0].carts[index].id
}

/// Gives cart `index` of the first page a file of `secs` seconds.
fn load(state: &mut AppState, index: usize, secs: f64) -> CartId {
    let p = page(state);
    let path = PathBuf::from(format!("/carts/cart{index}.wav"));
    apply(
        state,
        Command::AssignCartFile {
            page: p,
            index,
            path,
        },
    )
    .unwrap();
    let track = state.cartwall.pages[0].carts[index].track.unwrap();
    let analysis = TrackAnalysis {
        duration_secs: secs,
        cue_in: Some(0.5),
        cue_out: Some(secs - 0.5),
        ..TrackAnalysis::default()
    };
    apply(
        state,
        Command::ApplyAnalysis {
            track,
            analysis: Box::new(analysis),
        },
    )
    .unwrap();
    cart(state, index)
}

fn edit(state: &mut AppState, index: usize, f: impl FnOnce(&mut CartEdit)) {
    let c = &state.cartwall.pages[0].carts[index];
    let mut e = CartEdit {
        name: c.name.clone(),
        kind: c.kind,
        looped: c.looped,
        exclusive: c.exclusive,
    };
    f(&mut e);
    let p = page(state);
    apply(
        state,
        Command::SetCart {
            page: p,
            index,
            edit: e,
        },
    )
    .unwrap();
}

fn started(actions: &[EngineAction]) -> Vec<&CartRequest> {
    actions
        .iter()
        .filter_map(|a| match a {
            EngineAction::StartCart(request) => Some(request),
            _ => None,
        })
        .collect()
}

fn stopped(actions: &[EngineAction]) -> Vec<CartId> {
    actions
        .iter()
        .filter_map(|a| match a {
            EngineAction::StopCart { cart } => Some(*cart),
            _ => None,
        })
        .collect()
}

#[test]
fn a_fresh_state_has_one_page_with_the_default_grid() {
    let state = fixture(0);
    assert_eq!(state.cartwall.pages.len(), 1);
    let p = &state.cartwall.pages[0];
    assert_eq!((p.rows, p.cols, p.carts.len()), (2, 8, 16));
    assert!(p.carts.iter().all(|c| c.track.is_none()));
}

#[test]
fn c1_firing_starts_the_cart_from_cue_in_and_overlaps() {
    let mut state = fixture(0);
    let a = load(&mut state, 0, 10.0);
    let b = load(&mut state, 1, 5.0);
    let actions = apply(&mut state, Command::FireCart(a)).unwrap();
    let s = started(&actions);
    assert_eq!(s.len(), 1);
    assert_eq!((s[0].cart, s[0].from_secs, s[0].until_secs), (a, 0.5, 9.5));
    assert!(!s[0].looped);
    apply(&mut state, Command::FireCart(b)).unwrap();
    let playing: Vec<CartId> = state.cartwall.playing.iter().map(|p| p.cart).collect();
    assert_eq!(playing, vec![a, b], "carts overlap by default");
}

#[test]
fn c2_firing_a_playing_cart_stops_it() {
    let mut state = fixture(0);
    let a = load(&mut state, 0, 10.0);
    apply(&mut state, Command::FireCart(a)).unwrap();
    let actions = apply(&mut state, Command::FireCart(a)).unwrap();
    assert_eq!(stopped(&actions), vec![a]);
    assert!(started(&actions).is_empty());
    assert!(state.cartwall.playing.is_empty());
}

#[test]
fn c3_an_exclusive_cart_stops_every_other_cart() {
    let mut state = fixture(0);
    let a = load(&mut state, 0, 10.0);
    let b = load(&mut state, 1, 10.0);
    let c = load(&mut state, 2, 10.0);
    edit(&mut state, 1, |e| e.looped = true);
    edit(&mut state, 2, |e| e.exclusive = true);
    apply(&mut state, Command::FireCart(a)).unwrap();
    apply(&mut state, Command::FireCart(b)).unwrap();
    apply(&mut state, Command::CueCart(a)).unwrap();
    let actions = apply(&mut state, Command::FireCart(c)).unwrap();
    let mut s = stopped(&actions);
    s.sort();
    assert_eq!(s, vec![a, b]);
    assert_eq!(started(&actions)[0].cart, c);
    assert_eq!(state.cartwall.cue, Some(a), "the cue is not affected");
}

#[test]
fn c4_looped_carts_are_started_looped() {
    let mut state = fixture(0);
    let a = load(&mut state, 0, 10.0);
    edit(&mut state, 0, |e| e.looped = true);
    let actions = apply(&mut state, Command::FireCart(a)).unwrap();
    assert!(started(&actions)[0].looped);
}

#[test]
fn c5_a_cart_ends_at_cue_out() {
    let mut state = fixture(0);
    let a = load(&mut state, 0, 10.0);
    apply(&mut state, Command::FireCart(a)).unwrap();
    on_event(&mut state, EngineEvent::CartEnded { cart: a });
    assert!(state.cartwall.playing.is_empty());
}

#[test]
fn c6_empty_or_unavailable_carts_do_nothing() {
    let mut state = fixture(0);
    let empty = cart(&state, 3);
    assert!(
        apply(&mut state, Command::FireCart(empty))
            .unwrap()
            .is_empty()
    );
    let a = load(&mut state, 0, 10.0);
    let track = state.cartwall.pages[0].carts[0].track.unwrap();
    apply(
        &mut state,
        Command::SetFileState {
            track,
            state: FileState::Missing,
        },
    )
    .unwrap();
    assert!(apply(&mut state, Command::FireCart(a)).unwrap().is_empty());
    assert!(state.cartwall.playing.is_empty());
}

#[test]
fn c6_unknown_carts_are_refused() {
    let mut state = fixture(0);
    assert_eq!(
        apply(&mut state, Command::FireCart(CartId(999_999))),
        Err(ModelError::UnknownCart(CartId(999_999)))
    );
}

#[test]
fn c8_changing_the_file_of_a_playing_cart_stops_it() {
    let mut state = fixture(0);
    let a = load(&mut state, 0, 10.0);
    apply(&mut state, Command::FireCart(a)).unwrap();
    let p = page(&state);
    let actions = apply(
        &mut state,
        Command::AssignCartFile {
            page: p,
            index: 0,
            path: PathBuf::from("/carts/other.wav"),
        },
    )
    .unwrap();
    assert_eq!(stopped(&actions), vec![a]);
    assert!(state.cartwall.playing.is_empty());
}

#[test]
fn c8_the_last_page_cannot_be_deleted() {
    let mut state = fixture(0);
    let p = page(&state);
    assert_eq!(
        apply(&mut state, Command::DeleteCartPage(p)),
        Err(ModelError::LastCartPage)
    );
}

#[test]
fn c8_deleting_a_page_stops_its_carts_and_forgets_their_tracks() {
    let mut state = fixture(0);
    apply(
        &mut state,
        Command::CreateCartPage {
            name: "Effects".into(),
        },
    )
    .unwrap();
    let a = load(&mut state, 0, 10.0);
    let track = state.cartwall.pages[0].carts[0].track.unwrap();
    apply(&mut state, Command::FireCart(a)).unwrap();
    let p = page(&state);
    let actions = apply(&mut state, Command::DeleteCartPage(p)).unwrap();
    assert_eq!(stopped(&actions), vec![a]);
    assert!(state.library.get(track).is_none());
    assert_eq!(state.cartwall.pages.len(), 1);
}

#[test]
fn c9_only_one_cart_pre_listens() {
    let mut state = fixture(0);
    let a = load(&mut state, 0, 10.0);
    let b = load(&mut state, 1, 10.0);
    let first = apply(&mut state, Command::CueCart(a)).unwrap();
    assert!(
        first
            .iter()
            .any(|x| matches!(x, EngineAction::StartCartCue(r) if r.cart == a))
    );
    let second = apply(&mut state, Command::CueCart(b)).unwrap();
    assert!(second.contains(&EngineAction::StopCartCue));
    assert_eq!(state.cartwall.cue, Some(b));
    let third = apply(&mut state, Command::CueCart(b)).unwrap();
    assert_eq!(third, vec![EngineAction::StopCartCue]);
    assert_eq!(state.cartwall.cue, None);
    assert!(state.cartwall.playing.is_empty(), "cue never plays on Main");
}

#[test]
fn c9_the_cue_ends_by_itself() {
    let mut state = fixture(0);
    let a = load(&mut state, 0, 10.0);
    apply(&mut state, Command::CueCart(a)).unwrap();
    on_event(&mut state, EngineEvent::CartCueEnded { cart: a });
    assert_eq!(state.cartwall.cue, None);
}

#[test]
fn a_stale_cart_cue_end_does_not_end_a_newer_cue() {
    let mut state = fixture(0);
    let a = load(&mut state, 0, 10.0);
    let b = load(&mut state, 1, 10.0);
    apply(&mut state, Command::CueCart(a)).unwrap();
    apply(&mut state, Command::CueCart(b)).unwrap();
    on_event(&mut state, EngineEvent::CartCueEnded { cart: a });
    assert_eq!(state.cartwall.cue, Some(b));
}

#[test]
fn c10_stop_all_stops_carts_and_cue() {
    let mut state = fixture(0);
    let a = load(&mut state, 0, 10.0);
    let b = load(&mut state, 1, 10.0);
    apply(&mut state, Command::FireCart(a)).unwrap();
    apply(&mut state, Command::CueCart(b)).unwrap();
    let actions = apply(&mut state, Command::StopAllCarts).unwrap();
    assert_eq!(stopped(&actions), vec![a]);
    assert!(actions.contains(&EngineAction::StopCartCue));
    assert!(state.cartwall.playing.is_empty() && state.cartwall.cue.is_none());
}

#[test]
fn shrinking_a_page_that_would_drop_files_is_refused() {
    let mut state = fixture(0);
    load(&mut state, 15, 10.0);
    let p = page(&state);
    assert_eq!(
        apply(
            &mut state,
            Command::ResizeCartPage {
                page: p,
                rows: 1,
                cols: 8
            }
        ),
        Err(ModelError::CartsWouldBeLost)
    );
    assert_eq!(state.cartwall.pages[0].carts.len(), 16);
    apply(
        &mut state,
        Command::ResizeCartPage {
            page: p,
            rows: 3,
            cols: 8,
        },
    )
    .unwrap();
    assert_eq!(state.cartwall.pages[0].carts.len(), 24);
    assert!(
        state.cartwall.pages[0].carts[15].track.is_some(),
        "kept by position"
    );
}

#[test]
fn grids_outside_the_limits_are_refused() {
    let mut state = fixture(0);
    let p = page(&state);
    for (rows, cols) in [(0, 8), (2, 0), (9, 8), (2, 17)] {
        assert_eq!(
            apply(
                &mut state,
                Command::ResizeCartPage {
                    page: p,
                    rows,
                    cols
                }
            ),
            Err(ModelError::CartGridOutOfRange),
            "{rows}x{cols}"
        );
    }
}

#[test]
fn a_track_used_by_a_cart_survives_playlist_removal() {
    let mut state = fixture(1);
    let e = entries(&state);
    let track = state.playlists.entry(e[0]).unwrap().track;
    let path = state.library.get(track).unwrap().path.clone();
    let p = page(&state);
    apply(
        &mut state,
        Command::AssignCartFile {
            page: p,
            index: 0,
            path,
        },
    )
    .unwrap();
    // The cart gets its own library track even for the same file; removing
    // the playlist entry must not touch it.
    let cart_track = state.cartwall.pages[0].carts[0].track.unwrap();
    apply(&mut state, Command::RemoveEntry(e[0])).unwrap();
    assert!(state.library.get(cart_track).is_some());
}

#[test]
fn clearing_a_cart_file_forgets_its_track() {
    let mut state = fixture(0);
    load(&mut state, 0, 10.0);
    let track = state.cartwall.pages[0].carts[0].track.unwrap();
    let p = page(&state);
    apply(&mut state, Command::ClearCartFile { page: p, index: 0 }).unwrap();
    assert!(state.cartwall.pages[0].carts[0].track.is_none());
    assert!(state.library.get(track).is_none());
}

#[test]
fn failed_carts_are_marked_unreadable_and_removed() {
    let mut state = fixture(0);
    let a = load(&mut state, 0, 10.0);
    apply(&mut state, Command::FireCart(a)).unwrap();
    on_event(&mut state, EngineEvent::CartFailed { cart: a });
    let track = state.cartwall.pages[0].carts[0].track.unwrap();
    assert_eq!(
        state.library.get(track).unwrap().file_state,
        FileState::Unreadable
    );
    assert!(state.cartwall.playing.is_empty());
}

#[test]
fn a_cart_without_a_known_length_plays_to_the_end_of_the_file() {
    let mut state = fixture(0);
    let p = page(&state);
    apply(
        &mut state,
        Command::AssignCartFile {
            page: p,
            index: 0,
            path: PathBuf::from("/carts/new.wav"),
        },
    )
    .unwrap();
    let a = cart(&state, 0);
    let actions = apply(&mut state, Command::FireCart(a)).unwrap();
    assert_eq!(started(&actions)[0].until_secs, SOURCE_END);
}

#[test]
fn new_pages_use_the_configured_grid_and_kinds_are_editable() {
    let mut state = fixture(0);
    state.config.cartwall.default_rows = 3;
    state.config.cartwall.default_cols = 4;
    apply(
        &mut state,
        Command::CreateCartPage {
            name: "Sports".into(),
        },
    )
    .unwrap();
    let last = state.cartwall.pages.last().unwrap();
    assert_eq!(
        (last.name.as_str(), last.rows, last.cols, last.carts.len()),
        ("Sports", 3, 4, 12)
    );
    edit(&mut state, 0, |e| {
        e.kind = CartKind::Spot;
        e.name = "Bakery".into();
    });
    let c = &state.cartwall.pages[0].carts[0];
    assert_eq!((c.kind, c.name.as_str()), (CartKind::Spot, "Bakery"));
}

#[test]
fn importing_duplicate_positions_keeps_no_orphan_track() {
    let mut state = fixture(0);
    let edit = CartEdit {
        name: "x".into(),
        kind: CartKind::Jingle,
        looped: false,
        exclusive: false,
    };
    let import = fp_model::CartPageImport {
        name: "Imported".into(),
        rows: 2,
        cols: 8,
        carts: vec![
            (0, edit.clone(), Some(PathBuf::from("/a.wav"))),
            (0, edit, Some(PathBuf::from("/b.wav"))),
        ],
    };
    apply(&mut state, Command::ImportCartPage(Box::new(import))).unwrap();
    assert_eq!(
        state.library.iter().count(),
        1,
        "only the referenced track remains"
    );
}
