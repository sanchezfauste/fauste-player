//! The cartwall (Phase 2 spec P2.3): pages of carts fired on their own bus.
//! Carts are ordinary library tracks, so analysis and file state apply to
//! them unchanged. What is playing is never persisted (rule C7).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::ids::{CartId, CartPageId, IdGen, TrackId};

/// What a cart is for; it only changes how the button is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CartKind {
    #[default]
    Jingle,
    Effect,
    /// A commercial.
    Spot,
}

/// One button of the cartwall.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cart {
    pub id: CartId,
    #[serde(default)]
    pub name: String,
    /// The file, as a library track; `None` is an empty cart.
    #[serde(default)]
    pub track: Option<TrackId>,
    #[serde(default)]
    pub kind: CartKind,
    /// Restarts at cue-in when it reaches cue-out, until stopped (C4).
    #[serde(default)]
    pub looped: bool,
    /// Stops every other cart when fired (C3).
    #[serde(default)]
    pub exclusive: bool,
}

impl Cart {
    pub fn empty(id: CartId) -> Self {
        Self {
            id,
            name: String::new(),
            track: None,
            kind: CartKind::default(),
            looped: false,
            exclusive: false,
        }
    }
}

/// The editable fields of a cart (everything but its id and file).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CartEdit {
    pub name: String,
    pub kind: CartKind,
    pub looped: bool,
    pub exclusive: bool,
}

/// A tab of carts laid out as `rows × cols`, row-major.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CartPage {
    pub id: CartPageId,
    /// Empty until the user names it (the UI shows a localised default).
    #[serde(default)]
    pub name: String,
    pub rows: u16,
    pub cols: u16,
    pub carts: Vec<Cart>,
}

impl CartPage {
    /// A page of empty carts.
    pub fn new(ids: &mut IdGen, name: impl Into<String>, rows: u16, cols: u16) -> Self {
        let count = usize::from(rows) * usize::from(cols);
        Self {
            id: ids.cart_page(),
            name: name.into(),
            rows,
            cols,
            carts: (0..count).map(|_| Cart::empty(ids.cart())).collect(),
        }
    }
}

/// A cart that is on air.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayingCart {
    pub cart: CartId,
    pub looped: bool,
}

/// A cart page read from a file, before its files become library tracks.
#[derive(Debug, Clone, PartialEq)]
pub struct CartPageImport {
    pub name: String,
    pub rows: u16,
    pub cols: u16,
    /// `(position, fields, file)`; positions outside the grid are ignored.
    pub carts: Vec<(usize, CartEdit, Option<PathBuf>)>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Cartwall {
    pub pages: Vec<CartPage>,
    /// In firing order.
    pub playing: Vec<PlayingCart>,
    /// The cart pre-listening on the cartwall Cue route (C9).
    pub cue: Option<CartId>,
}

impl Cartwall {
    pub fn cart(&self, id: CartId) -> Option<&Cart> {
        self.pages
            .iter()
            .flat_map(|p| p.carts.iter())
            .find(|c| c.id == id)
    }

    pub fn page(&self, id: CartPageId) -> Option<&CartPage> {
        self.pages.iter().find(|p| p.id == id)
    }

    pub fn page_mut(&mut self, id: CartPageId) -> Option<&mut CartPage> {
        self.pages.iter_mut().find(|p| p.id == id)
    }

    pub fn is_playing(&self, id: CartId) -> bool {
        self.playing.iter().any(|p| p.cart == id)
    }

    pub fn references_track(&self, track: TrackId) -> bool {
        self.pages
            .iter()
            .flat_map(|p| p.carts.iter())
            .any(|c| c.track == Some(track))
    }

    /// The largest id used by pages and carts (to seed `IdGen` on load).
    pub fn max_raw_id(&self) -> u64 {
        self.pages
            .iter()
            .flat_map(|p| std::iter::once(p.id.0).chain(p.carts.iter().map(|c| c.id.0)))
            .max()
            .unwrap_or(0)
    }
}
