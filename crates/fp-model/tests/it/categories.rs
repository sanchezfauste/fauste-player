#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Track categories (track library spec LB9, LB10, LB13).

use fp_model::{
    AppState, Categories, Category, CategoryId, Command, Config, ModelError, Track, TrackKind,
    apply,
};

fn state() -> AppState {
    AppState::new(Config::default(), "P")
}

fn create(
    s: &mut AppState,
    parent: Option<CategoryId>,
    name: &str,
) -> Result<CategoryId, ModelError> {
    apply(
        s,
        Command::CreateCategory {
            parent,
            name: name.into(),
            color: None,
            kind: TrackKind::Music,
        },
    )?;
    Ok(s.categories.iter().map(|c| c.id).max().unwrap())
}

fn names(s: &AppState, parent: Option<CategoryId>) -> Vec<String> {
    s.categories
        .children(parent)
        .map(|c| c.name.clone())
        .collect()
}

#[test]
fn names_are_trimmed_and_must_not_be_empty() {
    let mut s = state();
    let id = create(&mut s, None, "  Gold ").unwrap();
    assert_eq!(s.categories.get(id).unwrap().name, "Gold");
    assert_eq!(
        create(&mut s, None, "   "),
        Err(ModelError::CategoryNameEmpty)
    );
}

#[test]
fn siblings_cannot_share_a_name_ignoring_case() {
    let mut s = state();
    let gold = create(&mut s, None, "Gold").unwrap();
    assert_eq!(
        create(&mut s, None, "gold"),
        Err(ModelError::CategoryNameTaken)
    );
    let other = create(&mut s, None, "Other").unwrap();
    create(&mut s, Some(other), "gold").unwrap();
    apply(
        &mut s,
        Command::RenameCategory {
            category: gold,
            name: "GOLD".into(),
        },
    )
    .unwrap();
    assert_eq!(s.categories.get(gold).unwrap().name, "GOLD");
}

#[test]
fn depth_is_limited_by_config() {
    let mut s = state();
    s.config.library.max_category_depth = 2;
    let a = create(&mut s, None, "A").unwrap();
    let b = create(&mut s, Some(a), "B").unwrap();
    assert_eq!(
        create(&mut s, Some(b), "C"),
        Err(ModelError::CategoryTooDeep)
    );
    let root = create(&mut s, None, "Root").unwrap();
    let err = apply(
        &mut s,
        Command::MoveCategory {
            category: a,
            parent: Some(root),
            index: 0,
        },
    );
    assert_eq!(err.unwrap_err(), ModelError::CategoryTooDeep);
}

#[test]
fn a_category_cannot_move_under_itself() {
    let mut s = state();
    let a = create(&mut s, None, "A").unwrap();
    let b = create(&mut s, Some(a), "B").unwrap();
    for parent in [b, a] {
        let err = apply(
            &mut s,
            Command::MoveCategory {
                category: a,
                parent: Some(parent),
                index: 0,
            },
        );
        assert_eq!(err.unwrap_err(), ModelError::CategoryIntoItself);
    }
}

#[test]
fn move_category_sets_parent_and_sibling_order() {
    let mut s = state();
    create(&mut s, None, "A").unwrap();
    create(&mut s, None, "B").unwrap();
    let c = create(&mut s, None, "C").unwrap();
    let d = create(&mut s, Some(c), "D").unwrap();
    apply(
        &mut s,
        Command::MoveCategory {
            category: c,
            parent: None,
            index: 0,
        },
    )
    .unwrap();
    assert_eq!(names(&s, None), ["C", "A", "B"]);
    assert_eq!(s.categories.get(d).unwrap().parent, Some(c));
    assert_eq!(s.categories.subtree(c), [c, d]);
    // Into another parent, past the end: clamped.
    let a = s.categories.children(None).nth(1).unwrap().id;
    apply(
        &mut s,
        Command::MoveCategory {
            category: c,
            parent: Some(a),
            index: 99,
        },
    )
    .unwrap();
    assert_eq!(names(&s, None), ["A", "B"]);
    assert_eq!(names(&s, Some(a)), ["C"]);
    assert_eq!(s.categories.depth(d), 3);
    assert!(s.categories.is_in_subtree(d, a));
}

#[test]
fn changing_a_category_kind_does_not_change_its_tracks() {
    let mut s = state();
    let a = create(&mut s, None, "A").unwrap();
    let id = s.ids.track();
    let mut track = Track::new(id, "/x.wav".into());
    track.category = Some(a);
    s.library.insert(track);
    apply(
        &mut s,
        Command::SetCategoryKind {
            category: a,
            kind: TrackKind::Ad,
        },
    )
    .unwrap();
    assert_eq!(s.categories.get(a).unwrap().kind, TrackKind::Ad);
    assert_eq!(s.library.get(id).unwrap().kind, TrackKind::Music);
}

#[test]
fn repaired_moves_orphans_and_cycles_to_the_root() {
    let cat = |id: u64, parent: Option<u64>| Category {
        id: CategoryId(id),
        parent: parent.map(CategoryId),
        name: format!("c{id}"),
        color: None,
        kind: TrackKind::Music,
    };
    let (fixed, warnings) = Categories::from(vec![
        cat(1, Some(99)),
        cat(2, Some(3)),
        cat(3, Some(2)),
        cat(4, Some(4)),
    ])
    .repaired();
    assert_eq!(warnings.len(), 3, "{warnings:?}");
    assert_eq!(fixed.children(None).count(), 3);
    assert_eq!(fixed.len(), 4);
    assert!(
        fixed
            .iter()
            .all(|c| fixed.depth(c.id) >= 1 && fixed.subtree(c.id).first() == Some(&c.id))
    );
    assert_eq!(
        fixed.get(CategoryId(3)).unwrap().parent,
        Some(CategoryId(2))
    );
    assert_eq!(fixed.iter().map(|c| fixed.depth(c.id)).max(), Some(2));
}

fn cat(id: u64, parent: Option<u64>) -> Category {
    Category {
        id: CategoryId(id),
        parent: parent.map(CategoryId),
        name: format!("c{id}"),
        color: None,
        kind: TrackKind::Music,
    }
}

#[test]
fn repaired_puts_parents_before_children_and_drops_repeated_ids() {
    let (fixed, warnings) = Categories::from(vec![
        cat(3, Some(2)),
        cat(2, Some(1)),
        cat(1, None),
        cat(2, None),
    ])
    .repaired();
    let order: Vec<u64> = fixed.iter().map(|c| c.id.0).collect();
    assert_eq!(order, [1, 2, 3]);
    assert_eq!(
        fixed.get(CategoryId(2)).unwrap().parent,
        Some(CategoryId(1))
    );
    assert_eq!(warnings.len(), 1, "{warnings:?}");
}

#[test]
fn a_three_level_subtree_moves_contiguously_and_respects_depth() {
    let mut s = state();
    let a = create(&mut s, None, "A").unwrap();
    let b = create(&mut s, Some(a), "B").unwrap();
    let c = create(&mut s, Some(b), "C").unwrap();
    let x = create(&mut s, None, "X").unwrap();
    let y = create(&mut s, None, "Y").unwrap();
    apply(
        &mut s,
        Command::MoveCategory {
            category: a,
            parent: None,
            index: 1,
        },
    )
    .unwrap();
    let order: Vec<CategoryId> = s.categories.iter().map(|k| k.id).collect();
    assert_eq!(order, [x, a, b, c, y]);
    assert_eq!(s.categories.get(a).unwrap().parent, None);
    // A grandchild that would exceed the limit.
    s.config.library.max_category_depth = 3;
    let err = apply(
        &mut s,
        Command::MoveCategory {
            category: a,
            parent: Some(x),
            index: 0,
        },
    );
    assert_eq!(err.unwrap_err(), ModelError::CategoryTooDeep);
}

/// A chain of `n` categories: 1 at the top, each next one under the last.
fn chain(n: u64) -> Vec<Category> {
    (1..=n).map(|i| cat(i, (i > 1).then(|| i - 1))).collect()
}

/// Runs `f` on a thread with a small stack, so recursion as deep as the
/// tree would overflow.
fn on_small_stack<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(256 * 1024)
        .spawn(f)
        .unwrap()
        .join()
        .unwrap()
}

#[test]
fn a_deep_tree_is_walked_without_recursion() {
    let (len, height_ok) = on_small_stack(|| {
        let cats = Categories::from(chain(10_000));
        let subtree = cats.subtree(CategoryId(1));
        (subtree.len(), subtree.last() == Some(&CategoryId(10_000)))
    });
    assert_eq!(len, 10_000);
    assert!(height_ok);
}

#[test]
fn repaired_lifts_categories_deeper_than_32_levels_to_the_root() {
    let (fixed, warnings) = on_small_stack(|| Categories::from(chain(10_000)).repaired());
    assert_eq!(fixed.len(), 10_000);
    // Every 32nd level starts a new chain at the top.
    assert_eq!(warnings.len(), 312, "{:?}", warnings.first());
    assert_eq!(fixed.children(None).count(), 313);
    assert_eq!(fixed.iter().map(|c| fixed.depth(c.id)).max(), Some(32));
    assert_eq!(fixed.get(CategoryId(33)).unwrap().parent, None);
    assert_eq!(
        fixed.get(CategoryId(34)).unwrap().parent,
        Some(CategoryId(33))
    );

    let mut s = state();
    s.categories = fixed;
    apply(&mut s, Command::DeleteCategory(CategoryId(1))).unwrap();
    assert_eq!(s.categories.len(), 10_000 - 32);
    assert!(s.categories.get(CategoryId(32)).is_none());
    assert!(s.categories.get(CategoryId(33)).is_some());
}
