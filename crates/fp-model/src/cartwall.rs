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

/// A cart's colour (cart colour spec CC1). The state stores its name; the
/// interface's theme gives each one its tone, so the tones can change
/// without touching saved data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CartColor {
    Red,
    Orange,
    Amber,
    Green,
    Teal,
    Blue,
    Violet,
    Pink,
}

impl CartColor {
    /// Every colour, in the picker's order.
    pub const ALL: [CartColor; 8] = [
        CartColor::Red,
        CartColor::Orange,
        CartColor::Amber,
        CartColor::Green,
        CartColor::Teal,
        CartColor::Blue,
        CartColor::Violet,
        CartColor::Pink,
    ];

    /// The stable lower-case name, as files and the remote API write it.
    pub fn name(self) -> &'static str {
        match self {
            CartColor::Red => "red",
            CartColor::Orange => "orange",
            CartColor::Amber => "amber",
            CartColor::Green => "green",
            CartColor::Teal => "teal",
            CartColor::Blue => "blue",
            CartColor::Violet => "violet",
            CartColor::Pink => "pink",
        }
    }

    /// The colour called `name` (lower case), if there is one.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.name() == name)
    }
}

/// CC3: a colour that does not parse loads as none (the store reports it).
fn lenient_color<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<CartColor>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Lenient {
        Valid(Option<CartColor>),
        Other(serde::de::IgnoredAny),
    }
    Ok(match Lenient::deserialize(d)? {
        Lenient::Valid(color) => color,
        Lenient::Other(_) => None,
    })
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
    /// CC1: `None` is no colour. Left out of the file when there is none.
    #[serde(
        default,
        deserialize_with = "lenient_color",
        skip_serializing_if = "Option::is_none"
    )]
    pub color: Option<CartColor>,
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
            color: None,
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
    /// CC2: set with the other fields.
    pub color: Option<CartColor>,
}

/// What an `EditCart` does to the cart's file (phase 2 spec C11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CartFileChange {
    Keep,
    /// A track already in the library; the same track is a `Keep`.
    Track(TrackId),
    Clear,
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayingCart {
    pub cart: CartId,
    pub looped: bool,
    /// The cart's normalization, fixed while it sounds (loudness
    /// normalization spec N11); runtime only.
    pub normalization: crate::loudness::Normalization,
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

#[derive(Debug, Clone, PartialEq)]
pub struct Cartwall {
    pub pages: Vec<CartPage>,
    /// In firing order.
    pub playing: Vec<PlayingCart>,
    /// The cart pre-listening on the cartwall Cue route (C9).
    pub cue: Option<CartId>,
    /// The pre-listen's normalization (N11); runtime only.
    pub cue_normalization: Option<crate::loudness::Normalization>,
    /// The page on screen (`None`: the first one). Shortcuts that fire
    /// "cart n" use it.
    pub shown: Option<CartPageId>,
    /// Whether the strip is expanded.
    pub open: bool,
}

impl Default for Cartwall {
    fn default() -> Self {
        Self {
            pages: Vec::new(),
            playing: Vec::new(),
            cue: None,
            cue_normalization: None,
            shown: None,
            open: true,
        }
    }
}

/// The part of the cartwall kept in `session.json`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CartwallSession {
    pub open: bool,
    pub page: Option<CartPageId>,
}

impl Default for CartwallSession {
    fn default() -> Self {
        Self {
            open: true,
            page: None,
        }
    }
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

    /// The page on screen: the chosen one if it still exists, else the first.
    pub fn shown_page(&self) -> Option<&CartPage> {
        self.shown
            .and_then(|id| self.page(id))
            .or_else(|| self.pages.first())
    }

    pub fn session(&self) -> CartwallSession {
        CartwallSession {
            open: self.open,
            page: self.shown,
        }
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

    /// Repairs pages read from disk so every invariant holds: grids within
    /// the limits with exactly `rows × cols` carts (growing the grid rather
    /// than dropping a cart that has a file), unique page and cart ids, and
    /// no cart pointing at a track the library does not have. Returns what
    /// was repaired, for the load warnings. Idempotent.
    pub fn normalize(
        &mut self,
        limits: &crate::config::Limits,
        ids: &mut IdGen,
        library: &crate::track::Library,
    ) -> Vec<String> {
        let mut notes = Vec::new();
        let mut seen = std::collections::HashSet::new();
        let (max_rows, max_cols) = (limits.max_cart_rows.max(1), limits.max_cart_cols.max(1));
        for page in &mut self.pages {
            if !seen.insert(page.id.0) {
                page.id = ids.cart_page();
                notes.push(format!("cart page \"{}\" had a duplicate id", page.name));
            }
            let (mut rows, mut cols) = (page.rows.clamp(1, max_rows), page.cols.clamp(1, max_cols));
            let needed = page
                .carts
                .iter()
                .rposition(|c| c.track.is_some())
                .map_or(0, |i| i + 1);
            while usize::from(rows) * usize::from(cols) < needed
                && (rows < max_rows || cols < max_cols)
            {
                if rows < max_rows {
                    rows += 1;
                } else {
                    cols += 1;
                }
            }
            let count = usize::from(rows) * usize::from(cols);
            if (rows, cols) != (page.rows, page.cols) || page.carts.len() != count {
                notes.push(format!(
                    "cart page \"{}\": grid {}×{} with {} carts repaired to {rows}×{cols}",
                    page.name,
                    page.rows,
                    page.cols,
                    page.carts.len()
                ));
            }
            let lost = page
                .carts
                .iter()
                .skip(count)
                .filter(|c| c.track.is_some())
                .count();
            if lost > 0 {
                notes.push(format!(
                    "cart page \"{}\": {lost} carts beyond the largest grid dropped",
                    page.name
                ));
            }
            page.carts.truncate(count);
            while page.carts.len() < count {
                page.carts.push(Cart::empty(ids.cart()));
            }
            page.rows = rows;
            page.cols = cols;
            for cart in &mut page.carts {
                if !seen.insert(cart.id.0) {
                    cart.id = ids.cart();
                }
                if cart.track.is_some_and(|t| library.get(t).is_none()) {
                    cart.track = None;
                    notes.push(format!(
                        "cart \"{}\" on page \"{}\" lost its file (not in the library)",
                        cart.name, page.name
                    ));
                }
            }
        }
        notes
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
