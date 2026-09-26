#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Random cartwall sessions keep `playing` and `cue` consistent: only carts
//! that exist and have a playable file, never twice.

use std::collections::HashSet;
use std::path::PathBuf;

use fp_model::{
    AppState, CartEdit, CartKind, Command, Config, EngineEvent, FileState, apply, on_event,
};
use proptest::prelude::*;

#[derive(Debug, Clone)]
enum Op {
    Fire(usize),
    Stop(usize),
    Cue(usize),
    StopAll,
    Assign(usize),
    Clear(usize),
    Edit {
        pick: usize,
        looped: bool,
        exclusive: bool,
    },
    Resize {
        rows: u16,
        cols: u16,
    },
    NewPage,
    DeleteFirstPage,
    Ended(usize),
    Failed(usize),
    Missing(usize),
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        (0usize..40).prop_map(Op::Fire),
        (0usize..40).prop_map(Op::Stop),
        (0usize..40).prop_map(Op::Cue),
        Just(Op::StopAll),
        (0usize..40).prop_map(Op::Assign),
        (0usize..40).prop_map(Op::Clear),
        (0usize..40, any::<bool>(), any::<bool>()).prop_map(|(pick, looped, exclusive)| Op::Edit {
            pick,
            looped,
            exclusive
        }),
        (1u16..4, 1u16..10).prop_map(|(rows, cols)| Op::Resize { rows, cols }),
        Just(Op::NewPage),
        Just(Op::DeleteFirstPage),
        (0usize..40).prop_map(Op::Ended),
        (0usize..40).prop_map(Op::Failed),
        (0usize..40).prop_map(Op::Missing),
    ]
}

/// (page, index) of the `pick`-th cart across all pages.
fn locate(
    state: &AppState,
    pick: usize,
) -> Option<(fp_model::CartPageId, usize, fp_model::CartId)> {
    let all: Vec<_> = state
        .cartwall
        .pages
        .iter()
        .flat_map(|p| {
            p.carts
                .iter()
                .enumerate()
                .map(move |(i, c)| (p.id, i, c.id))
        })
        .collect();
    if all.is_empty() {
        return None;
    }
    all.get(pick % all.len()).copied()
}

proptest! {
    #[test]
    fn playing_carts_stay_consistent(ops in prop::collection::vec(op(), 1..120)) {
        let mut state = AppState::new(Config::default(), "Main");
        for (n, op) in ops.into_iter().enumerate() {
            let cart = |pick| locate(&state, pick);
            match op {
                Op::Fire(p) => if let Some((_, _, c)) = cart(p) { let _ = apply(&mut state, Command::FireCart(c)); },
                Op::Stop(p) => if let Some((_, _, c)) = cart(p) { let _ = apply(&mut state, Command::StopCart(c)); },
                Op::Cue(p) => if let Some((_, _, c)) = cart(p) { let _ = apply(&mut state, Command::CueCart(c)); },
                Op::StopAll => { let _ = apply(&mut state, Command::StopAllCarts); },
                Op::Assign(p) => if let Some((page, index, _)) = cart(p) {
                    let path = PathBuf::from(format!("/carts/{n}.wav"));
                    let _ = apply(&mut state, Command::AssignCartFile { page, index, path });
                },
                Op::Clear(p) => if let Some((page, index, _)) = cart(p) {
                    let _ = apply(&mut state, Command::ClearCartFile { page, index });
                },
                Op::Edit { pick, looped, exclusive } => if let Some((page, index, _)) = cart(pick) {
                    let edit = CartEdit { name: format!("c{n}"), kind: CartKind::Effect, looped, exclusive };
                    let _ = apply(&mut state, Command::SetCart { page, index, edit });
                },
                Op::Resize { rows, cols } => {
                    let page = state.cartwall.pages[0].id;
                    let _ = apply(&mut state, Command::ResizeCartPage { page, rows, cols });
                }
                Op::NewPage => { let _ = apply(&mut state, Command::CreateCartPage { name: format!("p{n}") }); },
                Op::DeleteFirstPage => {
                    let page = state.cartwall.pages[0].id;
                    let _ = apply(&mut state, Command::DeleteCartPage(page));
                }
                Op::Ended(p) => if let Some((_, _, c)) = cart(p) { on_event(&mut state, EngineEvent::CartEnded { cart: c }); },
                Op::Failed(p) => if let Some((_, _, c)) = cart(p) { on_event(&mut state, EngineEvent::CartFailed { cart: c }); },
                Op::Missing(p) => if let Some(track) = cart(p).and_then(|(_, _, c)| state.cartwall.cart(c).and_then(|c| c.track)) {
                    let _ = apply(&mut state, Command::SetFileState { track, state: FileState::Missing });
                },
            }
            let mut seen = HashSet::new();
            for playing in &state.cartwall.playing {
                prop_assert!(seen.insert(playing.cart), "duplicate {:?}", playing.cart);
                let cart = state.cartwall.cart(playing.cart);
                prop_assert!(cart.is_some(), "playing cart {:?} no longer exists", playing.cart);
                let track = cart.and_then(|c| c.track);
                prop_assert!(track.is_some_and(|t| state.library.get(t).is_some()), "playing cart without a file");
            }
            if let Some(cue) = state.cartwall.cue {
                prop_assert!(state.cartwall.cart(cue).is_some_and(|c| c.track.is_some()));
            }
            prop_assert!(!state.cartwall.pages.is_empty());
            for page in &state.cartwall.pages {
                prop_assert_eq!(page.carts.len(), usize::from(page.rows) * usize::from(page.cols));
            }
            // Every cart track is in the library.
            for c in state.cartwall.pages.iter().flat_map(|p| p.carts.iter()) {
                if let Some(t) = c.track { prop_assert!(state.library.get(t).is_some()); }
            }
        }
    }
}
