//! The tree of track categories (track library spec §5). A category has a
//! name, an optional colour and a kind that new tracks filed under it take.
//! The tree is stored as a flat list in display order: the children of a
//! category are the entries with that parent, in list order.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::cartwall::CartColor;
use crate::config::MAX_CATEGORY_DEPTH;
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
    /// LB19: the folder this category watches; files that appear there join
    /// it. At most one category watches a given folder.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub watched_folder: Option<PathBuf>,
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
        let ids: HashSet<CategoryId> = ids.iter().copied().collect();
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
        self.subtree_levels(id)
            .into_iter()
            .map(|(c, _)| c)
            .collect()
    }

    /// The subtree of `id` as `subtree` gives it, each category with its
    /// level below `id` (`id` itself is 1). Iterative, so a tree of any
    /// depth is walked without recursion; a category met twice (a cycle in
    /// data not yet repaired) is walked once.
    fn subtree_levels(&self, id: CategoryId) -> Vec<(CategoryId, usize)> {
        let mut out = Vec::new();
        if self.get(id).is_none() {
            return out;
        }
        let mut children: HashMap<CategoryId, Vec<CategoryId>> = HashMap::new();
        for c in &self.list {
            if let Some(parent) = c.parent {
                children.entry(parent).or_default().push(c.id);
            }
        }
        let mut visited = HashSet::new();
        let mut stack = vec![(id, 1)];
        while let Some((at, level)) = stack.pop() {
            if !visited.insert(at) {
                continue;
            }
            out.push((at, level));
            if let Some(kids) = children.get(&at) {
                stack.extend(kids.iter().rev().map(|k| (*k, level + 1)));
            }
        }
        out
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
        self.subtree_levels(id)
            .into_iter()
            .map(|(_, level)| level)
            .max()
            .unwrap_or(0)
    }

    /// Loaded data may be damaged: a category whose id repeats is dropped
    /// (the first stays); one whose parent is unknown, or the first (in list
    /// order) of each cycle, goes to the top level; and one deeper than
    /// `MAX_CATEGORY_DEPTH` levels goes to the top level with its subtree.
    /// One warning each. A folder watched by several categories stays on the
    /// first in stored order (LB19). The list is then put in tree order: every parent
    /// before its children, each subtree contiguous. Linear in the number
    /// of categories, without recursion.
    pub fn repaired(mut self) -> (Categories, Vec<String>) {
        let mut warnings = Vec::new();
        let mut seen: HashSet<CategoryId> = HashSet::new();
        self.list.retain(|c| {
            if seen.insert(c.id) {
                true
            } else {
                warnings.push(format!(
                    "category {} ({:?}) repeats an id; dropped",
                    c.id.0, c.name
                ));
                false
            }
        });
        let mut watched: HashSet<PathBuf> = HashSet::new();
        for c in &mut self.list {
            let Some(folder) = c.watched_folder.clone() else {
                continue;
            };
            if !watched.insert(folder) {
                c.watched_folder = None;
                warnings.push(format!(
                    "category {} ({:?}) watches a folder another category already watches; cleared",
                    c.id.0, c.name
                ));
            }
        }
        let position: HashMap<CategoryId, usize> = self
            .list
            .iter()
            .enumerate()
            .map(|(i, c)| (c.id, i))
            .collect();
        // Each category's parent as an index; `None` at the top level.
        let mut parent: Vec<Option<usize>> = Vec::with_capacity(self.list.len());
        let mut lifted: Vec<Option<&'static str>> = vec![None; self.list.len()];
        for (i, c) in self.list.iter().enumerate() {
            match c.parent {
                None => parent.push(None),
                Some(p) => match position.get(&p) {
                    Some(&at) => parent.push(Some(at)),
                    None => {
                        parent.push(None);
                        if let Some(slot) = lifted.get_mut(i) {
                            *slot = Some("has an unknown parent");
                        }
                    }
                },
            }
        }
        // Cycles: each category has one parent, so a walk up either ends at
        // the top or enters a cycle. The first category of a cycle in list
        // order goes to the top, which breaks it.
        const UNSEEN: u8 = 0;
        const ON_WALK: u8 = 1;
        const DONE: u8 = 2;
        let mut mark = vec![UNSEEN; self.list.len()];
        for start in 0..self.list.len() {
            let mut walk = Vec::new();
            let mut at = Some(start);
            while let Some(i) = at {
                match mark.get(i).copied() {
                    Some(UNSEEN) => {
                        if let Some(m) = mark.get_mut(i) {
                            *m = ON_WALK;
                        }
                        walk.push(i);
                        at = parent.get(i).copied().flatten();
                    }
                    Some(ON_WALK) => {
                        // `i` is on a cycle: the cycle is the walk from `i` on.
                        let from = walk.iter().position(|w| *w == i).unwrap_or(0);
                        if let Some(&first) = walk.get(from..).and_then(|c| c.iter().min()) {
                            if let Some(p) = parent.get_mut(first) {
                                *p = None;
                            }
                            if let Some(slot) = lifted.get_mut(first) {
                                *slot = Some("is part of a cycle");
                            }
                        }
                        break;
                    }
                    _ => break,
                }
            }
            for i in walk {
                if let Some(m) = mark.get_mut(i) {
                    *m = DONE;
                }
            }
        }
        // Depth: from each root down, with an explicit stack.
        let mut children: Vec<Vec<usize>> = vec![Vec::new(); self.list.len()];
        let mut roots = Vec::new();
        for (i, p) in parent.iter().enumerate() {
            match p {
                Some(p) => {
                    if let Some(kids) = children.get_mut(*p) {
                        kids.push(i);
                    }
                }
                None => roots.push(i),
            }
        }
        let mut stack: Vec<(usize, usize)> = roots.iter().rev().map(|r| (*r, 1)).collect();
        while let Some((i, depth)) = stack.pop() {
            let depth = if depth > MAX_CATEGORY_DEPTH {
                if let Some(p) = parent.get_mut(i) {
                    *p = None;
                }
                if let Some(slot) = lifted.get_mut(i) {
                    *slot = Some("is too deep");
                }
                1
            } else {
                depth
            };
            if let Some(kids) = children.get(i) {
                stack.extend(kids.iter().rev().map(|k| (*k, depth + 1)));
            }
        }
        for (i, why) in lifted.iter().enumerate() {
            let (Some(why), Some(c)) = (why, self.list.get(i)) else {
                continue;
            };
            warnings.push(format!(
                "category {} ({:?}) {why}; moved to the top level",
                c.id.0, c.name
            ));
        }
        let ids: Vec<CategoryId> = self.list.iter().map(|c| c.id).collect();
        for (c, p) in self.list.iter_mut().zip(&parent) {
            c.parent = p.and_then(|p| ids.get(p).copied());
        }
        // Tree order: depth first from the roots, children in list order.
        let mut children: Vec<Vec<usize>> = vec![Vec::new(); self.list.len()];
        let mut roots = Vec::new();
        for (i, p) in parent.iter().enumerate() {
            match p.and_then(|p| children.get_mut(p)) {
                Some(kids) => kids.push(i),
                None => roots.push(i),
            }
        }
        let mut order = Vec::with_capacity(self.list.len());
        let mut stack: Vec<usize> = roots.into_iter().rev().collect();
        while let Some(i) = stack.pop() {
            order.push(i);
            if let Some(kids) = children.get(i) {
                stack.extend(kids.iter().rev());
            }
        }
        let mut slots: Vec<Option<Category>> = self.list.into_iter().map(Some).collect();
        self.list = order
            .into_iter()
            .filter_map(|i| slots.get_mut(i).and_then(Option::take))
            .collect();
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
        let subtree: HashSet<CategoryId> = self.subtree(id).into_iter().collect();
        self.list
            .iter()
            .rposition(|c| subtree.contains(&c.id))
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
        let moved: HashSet<CategoryId> = self.subtree(id).into_iter().collect();
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
        if let Some(root) = taken.iter_mut().find(|c| c.id == id) {
            root.parent = parent;
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

/// LB19: the category whose watched folder holds `path`, the innermost when
/// several do; `None` when none does.
pub fn watching_category(categories: &Categories, path: &Path) -> Option<CategoryId> {
    categories
        .iter()
        .filter_map(|c| {
            let folder = c.watched_folder.as_deref()?;
            path.starts_with(folder)
                .then(|| (folder.components().count(), c.id))
        })
        .max_by_key(|(depth, _)| *depth)
        .map(|(_, id)| id)
}
