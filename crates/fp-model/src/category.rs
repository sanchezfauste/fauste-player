//! The tree of track categories (track library spec §5). A category has a
//! name, an optional colour and a kind that new tracks filed under it take.
//! The tree is stored as a flat list in display order: the children of a
//! category are the entries with that parent, in list order.

use serde::{Deserialize, Serialize};

use crate::cartwall::CartColor;
use crate::error::ModelError;
use crate::ids::CategoryId;
use crate::track::TrackKind;

/// One node of the category tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Category {
    pub id: CategoryId,
    /// `None` for a top-level category.
    #[serde(default)]
    pub parent: Option<CategoryId>,
    pub name: String,
    #[serde(default)]
    pub color: Option<CartColor>,
    /// The kind a track takes when it is filed under this category.
    #[serde(default)]
    pub kind: TrackKind,
}

/// Every category, in display order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Categories {
    list: Vec<Category>,
}

impl From<Vec<Category>> for Categories {
    fn from(list: Vec<Category>) -> Self {
        Self { list }
    }
}

impl Categories {
    pub fn get(&self, id: CategoryId) -> Option<&Category> {
        self.list.iter().find(|c| c.id == id)
    }

    pub fn get_mut(&mut self, id: CategoryId) -> Option<&mut Category> {
        self.list.iter_mut().find(|c| c.id == id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Category> {
        self.list.iter()
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    /// Drops the categories in `ids`.
    pub(crate) fn remove_all(&mut self, ids: &[CategoryId]) {
        self.list.retain(|c| !ids.contains(&c.id));
    }

    pub fn into_vec(self) -> Vec<Category> {
        self.list
    }

    /// The largest raw id in use (0 when empty).
    pub fn max_raw_id(&self) -> u64 {
        self.list.iter().map(|c| c.id.0).max().unwrap_or(0)
    }

    /// The children of `parent` (`None`: the top level), in order.
    pub fn children(&self, parent: Option<CategoryId>) -> impl Iterator<Item = &Category> {
        self.list.iter().filter(move |c| c.parent == parent)
    }

    /// Levels from the top down to `id`: a top-level category is 1; an
    /// unknown id is 0.
    pub fn depth(&self, id: CategoryId) -> usize {
        let mut depth = 0;
        let mut at = Some(id);
        while let Some(current) = at {
            let Some(category) = self.get(current) else {
                break;
            };
            depth += 1;
            if depth > self.list.len() {
                break; // a cycle (repaired on load); stop anyway
            }
            at = category.parent;
        }
        depth
    }

    /// The depth of `parent`, 0 for the top level (`None`).
    pub(crate) fn depth_below(&self, parent: Option<CategoryId>) -> usize {
        parent.map_or(0, |p| self.depth(p))
    }

    /// `id` followed by all its descendants (depth first, in order). Empty
    /// for an unknown id.
    pub fn subtree(&self, id: CategoryId) -> Vec<CategoryId> {
        let mut out = Vec::new();
        if self.get(id).is_none() {
            return out;
        }
        self.collect(id, &mut out);
        out
    }

    fn collect(&self, id: CategoryId, out: &mut Vec<CategoryId>) {
        if out.contains(&id) {
            return;
        }
        out.push(id);
        for child in self.children(Some(id)) {
            self.collect(child.id, out);
        }
    }

    /// Whether `id` is `root` or lies below it.
    pub fn is_in_subtree(&self, id: CategoryId, root: CategoryId) -> bool {
        let mut at = Some(id);
        for _ in 0..=self.list.len() {
            match at {
                Some(current) if current == root => return true,
                Some(current) => at = self.get(current).and_then(|c| c.parent),
                None => return false,
            }
        }
        false
    }

    /// Levels of the subtree rooted at `id` (a leaf is 1).
    fn height(&self, id: CategoryId) -> usize {
        let base = self.depth(id);
        self.subtree(id)
            .into_iter()
            .map(|c| self.depth(c).saturating_sub(base) + 1)
            .max()
            .unwrap_or(0)
    }

    /// Loaded data may be damaged: a category whose parent is unknown, or
    /// that lies on a cycle, goes to the top level. One warning each.
    pub fn repaired(mut self) -> (Categories, Vec<String>) {
        let mut warnings = Vec::new();
        let ids: Vec<CategoryId> = self.list.iter().map(|c| c.id).collect();
        for i in 0..self.list.len() {
            let Some(category) = self.list.get(i) else {
                continue;
            };
            let (id, name, parent) = (category.id, category.name.clone(), category.parent);
            let Some(parent) = parent else {
                continue;
            };
            let broken = if !ids.contains(&parent) {
                Some("has an unknown parent")
            } else if self.is_in_subtree(parent, id) {
                Some("is part of a cycle")
            } else {
                None
            };
            if let Some(why) = broken {
                warnings.push(format!(
                    "category {} ({name:?}) {why}; moved to the top level",
                    id.0
                ));
                if let Some(c) = self.list.get_mut(i) {
                    c.parent = None;
                }
            }
        }
        (self, warnings)
    }

    /// Adds a category at the end of its siblings. The caller has checked it.
    pub(crate) fn push(&mut self, category: Category) {
        let at = self.end_of_children(category.parent);
        self.list.insert(at, category);
    }

    /// The list position after the last child (with its subtree) of
    /// `parent`, or right after `parent` when it has none.
    fn end_of_children(&self, parent: Option<CategoryId>) -> usize {
        match self.children(parent).last() {
            Some(last) => self.after_subtree(last.id),
            None => match parent.and_then(|p| self.position(p)) {
                Some(p) => p + 1,
                None => self.list.len(),
            },
        }
    }

    fn position(&self, id: CategoryId) -> Option<usize> {
        self.list.iter().position(|c| c.id == id)
    }

    fn after_subtree(&self, id: CategoryId) -> usize {
        self.subtree(id)
            .into_iter()
            .filter_map(|c| self.position(c))
            .max()
            .map_or(self.list.len(), |p| p + 1)
    }

    /// Checks and performs a move of `id` (with its subtree) under `parent`
    /// at `index` among the new siblings (clamped).
    pub(crate) fn move_to(
        &mut self,
        id: CategoryId,
        parent: Option<CategoryId>,
        index: usize,
        max_depth: usize,
    ) -> Result<(), ModelError> {
        let moved_name = self
            .get(id)
            .ok_or(ModelError::UnknownCategory(id))?
            .name
            .clone();
        if let Some(p) = parent {
            if self.get(p).is_none() {
                return Err(ModelError::UnknownCategory(p));
            }
            if self.is_in_subtree(p, id) {
                return Err(ModelError::CategoryIntoItself);
            }
        }
        let parent_depth = self.depth_below(parent);
        if parent_depth + self.height(id) > max_depth {
            return Err(ModelError::CategoryTooDeep);
        }
        check_name(self, parent, &moved_name, Some(id))?;
        // Take the subtree out, keeping its order.
        let moved = self.subtree(id);
        let mut taken = Vec::with_capacity(moved.len());
        let mut rest = Vec::with_capacity(self.list.len());
        for c in std::mem::take(&mut self.list) {
            if moved.contains(&c.id) {
                taken.push(c);
            } else {
                rest.push(c);
            }
        }
        self.list = rest;
        if let Some(first) = taken.first_mut() {
            first.parent = parent;
        }
        let siblings: Vec<CategoryId> = self.children(parent).map(|c| c.id).collect();
        let at = match siblings.get(index) {
            Some(sibling) => self.position(*sibling).unwrap_or(self.list.len()),
            None => self.end_of_children(parent),
        };
        self.list.splice(at..at, taken);
        Ok(())
    }
}

/// Checks a category name (spec LB9): trimmed, not empty, and unique among
/// the siblings under `parent` ignoring case. `except` is the category being
/// renamed or moved. Returns the trimmed name.
pub fn check_name(
    cats: &Categories,
    parent: Option<CategoryId>,
    name: &str,
    except: Option<CategoryId>,
) -> Result<String, ModelError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(ModelError::CategoryNameEmpty);
    }
    let lower = name.to_lowercase();
    let taken = cats
        .children(parent)
        .any(|c| Some(c.id) != except && c.name.to_lowercase() == lower);
    if taken {
        return Err(ModelError::CategoryNameTaken);
    }
    Ok(name.to_string())
}
