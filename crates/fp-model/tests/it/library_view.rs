#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! The library window's saved state (plan 3, task 1).

use fp_model::{
    AppState, Command, Config, LibraryColumn, LibraryColumnWidth, LibrarySelection, LibraryView,
    RestoreParts, TrackKind, apply, lenient_library_view,
};

fn state() -> AppState {
    AppState::new(Config::default(), "P")
}

fn create(s: &mut AppState) -> fp_model::CategoryId {
    apply(
        s,
        Command::CreateCategory {
            parent: None,
            name: "Jingles".into(),
            color: None,
            kind: TrackKind::Music,
        },
    )
    .unwrap();
    s.categories.iter().map(|c| c.id).max().unwrap()
}

#[test]
fn set_library_view_stores_it() {
    let mut s = state();
    let view = LibraryView {
        open: true,
        rect: Some([1.0, 2.0, 300.0, 200.0]),
        whole_library: true,
        tree_width: Some(120.0),
        ..LibraryView::default()
    };
    apply(&mut s, Command::SetLibraryView(Box::new(view.clone()))).unwrap();
    assert_eq!(s.library_view, view);
}

#[test]
fn a_deleted_selected_category_falls_back_to_all_tracks() {
    let mut s = state();
    let id = create(&mut s);
    let view = LibraryView {
        selection: LibrarySelection::Category(id),
        expanded: vec![id],
        ..LibraryView::default()
    };
    apply(&mut s, Command::SetLibraryView(Box::new(view))).unwrap();
    assert_eq!(s.library_view.selection, LibrarySelection::Category(id));
    apply(&mut s, Command::DeleteCategory(id)).unwrap();
    assert_eq!(s.library_view.selection, LibrarySelection::All);
    assert!(s.library_view.expanded.is_empty());
}

fn bad_view(unknown: fp_model::CategoryId) -> LibraryView {
    let col = |width| LibraryColumnWidth {
        column: LibraryColumn::Kind,
        width,
    };
    LibraryView {
        selection: LibrarySelection::Category(unknown),
        expanded: vec![unknown],
        rect: Some([0.0, 0.0, f32::NAN, 100.0]),
        columns: vec![col(f32::INFINITY), col(0.0), col(-3.0), col(80.0)],
        tree_width: Some(0.0),
        ..LibraryView::default()
    }
}

#[test]
fn repaired_drops_unknown_categories_and_bad_sizes() {
    let mut s = state();
    let known = create(&mut s);
    let unknown = fp_model::CategoryId(known.0 + 100);
    let mut view = bad_view(unknown);
    view.expanded.push(known);
    let r = view.clone().repaired(&s.categories);
    assert_eq!(r.selection, LibrarySelection::All);
    assert_eq!(r.expanded, vec![known]);
    assert_eq!(r.rect, None);
    assert_eq!(r.columns.len(), 1);
    assert_eq!(r.columns[0].width, 80.0);
    assert_eq!(r.tree_width, None);
    view.rect = Some([0.0, 0.0, 100.0, 0.0]);
    assert_eq!(view.repaired(&s.categories).rect, None);
    let ok = LibraryView {
        rect: Some([-5.0, 3.0, 100.0, 50.0]),
        ..LibraryView::default()
    };
    assert_eq!(ok.clone().repaired(&s.categories), ok);
}

#[test]
fn restore_repairs_the_saved_view() {
    let mut s = state();
    let known = create(&mut s);
    let unknown = fp_model::CategoryId(known.0 + 100);
    let parts = RestoreParts {
        rotations: Default::default(),
        air_times: Default::default(),
        filler: None,
        config: s.config.clone(),
        categories: s.categories.clone(),
        ids: s.ids.clone(),
        library_view: bad_view(unknown),
        ..RestoreParts::default()
    };
    let (restored, _) = AppState::restore(parts, &[], "P");
    assert_eq!(restored.library_view.selection, LibrarySelection::All);
    assert!(restored.library_view.expanded.is_empty());
    assert_eq!(restored.library_view.rect, None);
}

#[test]
fn a_lenient_view_keeps_the_good_fields() {
    let v =
        lenient_library_view(serde_json::json!({"open": "yes", "rect": 3, "whole_library": true}))
            .unwrap();
    assert!(!v.open);
    assert_eq!(v.rect, None);
    assert!(v.whole_library);
    assert!(lenient_library_view(serde_json::json!(7)).is_none());
}

// Track library spec §8.5 (LB60): the library CUE's listening level.

#[test]
fn lb60_a_default_view_has_full_cue_volume() {
    assert_eq!(LibraryView::default().cue_volume, 1.0);
}

#[test]
fn lb60_a_saved_view_without_cue_volume_loads_full() {
    let v = lenient_library_view(serde_json::json!({"open": true})).unwrap();
    assert_eq!(v.cue_volume, 1.0);
    assert!(v.open);
}

#[test]
fn lb60_a_bad_cue_volume_loads_full_and_keeps_the_rest() {
    let v = lenient_library_view(serde_json::json!({"open": true, "cue_volume": "loud"})).unwrap();
    assert_eq!(v.cue_volume, 1.0);
    assert!(v.open);
}

#[test]
fn lb60_a_saved_cue_volume_round_trips() {
    let v = LibraryView {
        cue_volume: 0.4,
        ..LibraryView::default()
    };
    let back = lenient_library_view(serde_json::to_value(&v).unwrap()).unwrap();
    assert_eq!(back.cue_volume, 0.4);
}

#[test]
fn lb60_repaired_clamps_the_cue_volume() {
    let s = state();
    let fix = |v: f32| {
        LibraryView {
            cue_volume: v,
            ..LibraryView::default()
        }
        .repaired(&s.categories)
        .cue_volume
    };
    assert_eq!(fix(1.7), 1.0);
    assert_eq!(fix(-0.2), 0.0);
    assert_eq!(fix(f32::NAN), 1.0);
    assert_eq!(fix(f32::INFINITY), 1.0);
    assert_eq!(fix(0.3), 0.3);
}
