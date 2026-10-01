//! Cartwall rules C1–C10 (Phase 2 spec P2.3), as pure state transitions.

use std::path::PathBuf;

use crate::cartwall::{Cart, CartEdit, CartFileChange, CartPage, CartPageImport, PlayingCart};
use crate::command::{CartRequest, EngineAction, SOURCE_END};
use crate::error::ModelError;
use crate::ids::{CartId, CartPageId, TrackId};
use crate::state::AppState;
use crate::track::{FileState, Track};

/// What the engine needs to play `cart`, if it has a playable file.
fn request(state: &AppState, cart: &Cart, looped: bool) -> Option<CartRequest> {
    let track = state.library.get(cart.track?)?;
    if !track.file_state.is_playable() {
        return None;
    }
    Some(CartRequest {
        cart: cart.id,
        track: track.id,
        path: track.path.clone(),
        from_secs: track.cue_in_secs(),
        until_secs: track.known_cue_out_secs().unwrap_or(SOURCE_END),
        looped,
        format: track.format,
    })
}

fn stop_playing(state: &mut AppState, cart: CartId, out: &mut Vec<EngineAction>) {
    if state.cartwall.is_playing(cart) {
        state.cartwall.playing.retain(|p| p.cart != cart);
        out.push(EngineAction::StopCart { cart });
    }
}

fn stop_cue(state: &mut AppState, out: &mut Vec<EngineAction>) {
    if state.cartwall.cue.take().is_some() {
        out.push(EngineAction::StopCartCue);
    }
}

/// C1–C3, C6.
pub(crate) fn fire(
    state: &mut AppState,
    id: CartId,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let cart = state
        .cartwall
        .cart(id)
        .cloned()
        .ok_or(ModelError::UnknownCart(id))?;
    if state.cartwall.is_playing(id) {
        stop_playing(state, id, out);
        return Ok(());
    }
    let Some(request) = request(state, &cart, cart.looped) else {
        return Ok(());
    };
    if cart.exclusive {
        let others: Vec<CartId> = state.cartwall.playing.iter().map(|p| p.cart).collect();
        for other in others {
            stop_playing(state, other, out);
        }
    }
    state.cartwall.playing.push(PlayingCart {
        cart: id,
        looped: cart.looped,
    });
    out.push(EngineAction::StartCart(request));
    Ok(())
}

pub(crate) fn stop(
    state: &mut AppState,
    id: CartId,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    if state.cartwall.cart(id).is_none() {
        return Err(ModelError::UnknownCart(id));
    }
    stop_playing(state, id, out);
    Ok(())
}

/// C10.
pub(crate) fn stop_all(state: &mut AppState, out: &mut Vec<EngineAction>) {
    let playing: Vec<CartId> = state.cartwall.playing.iter().map(|p| p.cart).collect();
    for cart in playing {
        stop_playing(state, cart, out);
    }
    stop_cue(state, out);
}

/// C9.
pub(crate) fn cue(
    state: &mut AppState,
    id: CartId,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let cart = state
        .cartwall
        .cart(id)
        .cloned()
        .ok_or(ModelError::UnknownCart(id))?;
    let was = state.cartwall.cue;
    stop_cue(state, out);
    if was == Some(id) {
        return Ok(());
    }
    if let Some(request) = request(state, &cart, false) {
        state.cartwall.cue = Some(id);
        out.push(EngineAction::StartCartCue(request));
    }
    Ok(())
}

fn check_grid(state: &AppState, rows: u16, cols: u16) -> Result<(), ModelError> {
    let limits = &state.config.limits;
    if (1..=limits.max_cart_rows).contains(&rows) && (1..=limits.max_cart_cols).contains(&cols) {
        Ok(())
    } else {
        Err(ModelError::CartGridOutOfRange)
    }
}

pub(crate) fn create_page(state: &mut AppState, name: String) {
    let (rows, cols) = (
        state.config.cartwall.default_rows,
        state.config.cartwall.default_cols,
    );
    let page = CartPage::new(&mut state.ids, name, rows, cols);
    state.cartwall.pages.push(page);
}

pub(crate) fn rename_page(
    state: &mut AppState,
    page: CartPageId,
    name: String,
) -> Result<(), ModelError> {
    let p = state
        .cartwall
        .page_mut(page)
        .ok_or(ModelError::UnknownCartPage(page))?;
    p.name = name;
    Ok(())
}

/// Tracks no longer referenced by any playlist entry or cart are dropped.
pub(crate) fn forget_tracks(state: &mut AppState, tracks: impl IntoIterator<Item = TrackId>) {
    for track in tracks {
        if !state.playlists.references_track(track) && !state.cartwall.references_track(track) {
            state.library.remove(track);
        }
    }
}

/// C8.
pub(crate) fn delete_page(
    state: &mut AppState,
    page: CartPageId,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let Some(index) = state.cartwall.pages.iter().position(|p| p.id == page) else {
        return Err(ModelError::UnknownCartPage(page));
    };
    if state.cartwall.pages.len() <= 1 {
        return Err(ModelError::LastCartPage);
    }
    let removed = state.cartwall.pages.remove(index);
    if state.cartwall.shown == Some(page) {
        state.cartwall.shown = None;
    }
    for cart in &removed.carts {
        stop_playing(state, cart.id, out);
        if state.cartwall.cue == Some(cart.id) {
            stop_cue(state, out);
        }
    }
    forget_tracks(state, removed.carts.iter().filter_map(|c| c.track));
    Ok(())
}

pub(crate) fn resize_page(
    state: &mut AppState,
    page: CartPageId,
    rows: u16,
    cols: u16,
) -> Result<(), ModelError> {
    check_grid(state, rows, cols)?;
    let count = usize::from(rows) * usize::from(cols);
    let p = state
        .cartwall
        .page(page)
        .ok_or(ModelError::UnknownCartPage(page))?;
    if p.carts.iter().skip(count).any(|c| c.track.is_some()) {
        return Err(ModelError::CartsWouldBeLost);
    }
    let mut carts = p.carts.clone();
    carts.truncate(count);
    while carts.len() < count {
        carts.push(Cart::empty(state.ids.cart()));
    }
    if let Some(p) = state.cartwall.page_mut(page) {
        p.rows = rows;
        p.cols = cols;
        p.carts = carts;
    }
    // Dropped carts have no file, so none of them can be playing or cueing
    // (clearing a file stops the cart first).
    Ok(())
}

fn cart_at(state: &AppState, page: CartPageId, index: usize) -> Result<&Cart, ModelError> {
    state
        .cartwall
        .page(page)
        .ok_or(ModelError::UnknownCartPage(page))?
        .carts
        .get(index)
        .ok_or(ModelError::UnknownCartPosition(index))
}

fn cart_at_mut(
    state: &mut AppState,
    page: CartPageId,
    index: usize,
) -> Result<&mut Cart, ModelError> {
    state
        .cartwall
        .page_mut(page)
        .ok_or(ModelError::UnknownCartPage(page))?
        .carts
        .get_mut(index)
        .ok_or(ModelError::UnknownCartPosition(index))
}

pub(crate) fn set_cart(
    state: &mut AppState,
    page: CartPageId,
    index: usize,
    edit: CartEdit,
) -> Result<(), ModelError> {
    let cart = cart_at_mut(state, page, index)?;
    cart.name = edit.name;
    cart.kind = edit.kind;
    cart.looped = edit.looped;
    cart.exclusive = edit.exclusive;
    Ok(())
}

/// C11: every check first (`resize_page` checks before it changes
/// anything), then every change.
pub(crate) fn edit_page(
    state: &mut AppState,
    page: CartPageId,
    name: Option<String>,
    grid: Option<(u16, u16)>,
) -> Result<(), ModelError> {
    state
        .cartwall
        .page(page)
        .ok_or(ModelError::UnknownCartPage(page))?;
    if let Some((rows, cols)) = grid {
        resize_page(state, page, rows, cols)?;
    }
    if let Some(name) = name {
        rename_page(state, page, name)?;
    }
    Ok(())
}

/// C8 and C11: a cart and its file change together or not at all; only a
/// different file stops the cart.
pub(crate) fn edit_cart(
    state: &mut AppState,
    page: CartPageId,
    index: usize,
    edit: CartEdit,
    file: CartFileChange,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let current = cart_at(state, page, index)?.track;
    let track = match file {
        CartFileChange::Keep => None,
        CartFileChange::Track(t) if current == Some(t) => None,
        CartFileChange::Track(t) => {
            if state.library.get(t).is_none() {
                return Err(ModelError::UnknownTrack(t));
            }
            Some(Some(t))
        }
        CartFileChange::Clear => current.map(|_| None),
    };
    set_cart(state, page, index, edit)?;
    match track {
        Some(track) => set_track(state, page, index, track, out),
        None => Ok(()),
    }
}

/// C8: a new file stops the cart first; the old track is forgotten.
pub(crate) fn assign_file(
    state: &mut AppState,
    page: CartPageId,
    index: usize,
    path: Option<PathBuf>,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    cart_at(state, page, index)?;
    let track = path.map(|path| {
        let track = state.ids.track();
        state.library.insert(Track::new(track, path));
        track
    });
    set_track(state, page, index, track, out)
}

pub(crate) fn assign_track(
    state: &mut AppState,
    page: CartPageId,
    index: usize,
    track: TrackId,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    if state.library.get(track).is_none() {
        return Err(ModelError::UnknownTrack(track));
    }
    set_track(state, page, index, Some(track), out)
}

/// Stops the cart, gives it `track` and forgets the old one if unused (C8).
fn set_track(
    state: &mut AppState,
    page: CartPageId,
    index: usize,
    track: Option<TrackId>,
    out: &mut Vec<EngineAction>,
) -> Result<(), ModelError> {
    let (id, old) = {
        let cart = cart_at(state, page, index)?;
        (cart.id, cart.track)
    };
    stop_playing(state, id, out);
    if state.cartwall.cue == Some(id) {
        stop_cue(state, out);
    }
    cart_at_mut(state, page, index)?.track = track;
    forget_tracks(state, old);
    Ok(())
}

pub(crate) fn import_page(state: &mut AppState, import: CartPageImport) -> Result<(), ModelError> {
    check_grid(state, import.rows, import.cols)?;
    let mut page = CartPage::new(&mut state.ids, import.name, import.rows, import.cols);
    for (position, edit, file) in import.carts {
        let track = file.map(|path| {
            let track = state.ids.track();
            state.library.insert(Track::new(track, path));
            track
        });
        if let Some(cart) = page.carts.get_mut(position) {
            // A position given twice: the earlier file is not kept.
            if let Some(old) = cart.track.take() {
                state.library.remove(old);
            }
            cart.name = edit.name;
            cart.kind = edit.kind;
            cart.looped = edit.looped;
            cart.exclusive = edit.exclusive;
            cart.track = track;
        } else if let Some(track) = track {
            // Outside the grid: do not keep an orphan track.
            state.library.remove(track);
        }
    }
    state.cartwall.pages.push(page);
    Ok(())
}

/// C5.
pub(crate) fn ended(state: &mut AppState, cart: CartId) {
    state.cartwall.playing.retain(|p| p.cart != cart);
}

pub(crate) fn failed(state: &mut AppState, cart: CartId) {
    state.cartwall.playing.retain(|p| p.cart != cart);
    if state.cartwall.cue == Some(cart) {
        state.cartwall.cue = None;
    }
    if let Some(track) = state.cartwall.cart(cart).and_then(|c| c.track)
        && let Some(t) = state.library.get_mut(track)
    {
        t.file_state = FileState::Unreadable;
    }
}

/// The pre-listen of `cart` ended; a newer pre-listen stays.
pub(crate) fn cue_ended(state: &mut AppState, cart: CartId) {
    if state.cartwall.cue == Some(cart) {
        state.cartwall.cue = None;
    }
}
