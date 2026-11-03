#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::float_cmp
)]
//! Track library spec §7.4, §7.5 (LB28, LB29): the library window's track
//! table, its search, and files added to the library.

use std::path::{Path, PathBuf};

use crate::support;

use egui::accesskit::{Role, Toggled};
use egui::{Modifiers, Rect, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use fp_app::ui::app::AppUi;
use fp_app::ui::files::canonical_path;
use fp_model::library_view::{LibraryColumn, LibrarySelection, LibrarySort, LibraryView};
use fp_model::{AppState, CategoryId, Command, FileState, TableColumn, TrackKind};
use support::{Fake, harness_sized, state};

type H = Harness<'static, AppUi>;

fn create(s: &mut AppState, parent: Option<CategoryId>, name: &str) -> CategoryId {
    fp_model::apply(
        s,
        Command::CreateCategory {
            parent,
            name: name.into(),
            color: None,
            kind: TrackKind::Music,
        },
    )
    .unwrap();
    s.categories
        .iter()
        .find(|c| c.parent == parent && c.name == name)
        .unwrap()
        .id
}

fn lib_path(name: &str) -> PathBuf {
    PathBuf::from(format!("/lib/{name}.mp3"))
}

/// Adds library tracks titled `names` (their file stems).
fn add(s: &mut AppState, names: &[&str], category: Option<CategoryId>) {
    fp_model::apply(
        s,
        Command::AddToLibrary {
            paths: names.iter().map(|n| lib_path(n)).collect(),
            category,
        },
    )
    .unwrap();
}

/// One player with an empty playlist and the library window open.
fn base() -> AppState {
    let mut s = state(1, 0);
    s.library_view = LibraryView {
        open: true,
        rect: Some([20.0, 60.0, 1100.0, 680.0]),
        ..LibraryView::default()
    };
    s
}

fn open(s: AppState) -> (H, std::sync::Arc<Fake>) {
    let (mut h, fake) = harness_sized(s, vec2(1200.0, 800.0), |ui| ui);
    h.run_steps(2);
    fake.take_sent();
    (h, fake)
}

fn views(sent: &[Command]) -> Vec<LibraryView> {
    sent.iter()
        .filter_map(|c| match c {
            Command::SetLibraryView(v) => Some((**v).clone()),
            _ => None,
        })
        .collect()
}

/// Types `text` into the search field; the table filters in the same frame.
fn search(h: &mut H, text: &str) {
    h.get_by_label("Search the library").click();
    h.run_steps(1);
    h.get_by_label("Search the library").type_text(text);
    h.run_steps(1);
}

fn shown(h: &H, title: &str) -> bool {
    h.query_by_label(title).is_some()
}

/// The titles in the order the table shows them, top to bottom.
fn order(h: &H, titles: &[&str]) -> Vec<String> {
    let mut at: Vec<(f32, String)> = titles
        .iter()
        .map(|t| (h.get_by_label(t).rect().top(), (*t).to_owned()))
        .collect();
    at.sort_by(|a, b| a.0.total_cmp(&b.0));
    at.into_iter().map(|(_, t)| t).collect()
}

/// The rows the table shows as selected.
fn selected_rows(h: &H) -> Vec<Rect> {
    h.query_all_by_role(Role::Row)
        .filter(|n| n.accesskit_node().toggled() == Some(Toggled::True))
        .map(|n| n.rect())
        .collect()
}

fn is_selected(h: &H, title: &str) -> bool {
    let c = h.get_by_label(title).rect().center();
    selected_rows(h).iter().any(|r| r.contains(c))
}

#[test]
fn the_table_shows_the_selected_categorys_tracks_and_its_subcategories() {
    let mut s = base();
    let rock = create(&mut s, None, "Rock");
    let classic = create(&mut s, Some(rock), "Classic");
    let jingles = create(&mut s, None, "Jingles");
    add(&mut s, &["Song A"], Some(rock));
    add(&mut s, &["Song B"], Some(classic));
    add(&mut s, &["Song C"], Some(jingles));
    add(&mut s, &["Song D"], None);
    s.library_view.selection = LibrarySelection::Category(rock);
    let (h, _) = open(s);
    assert!(shown(&h, "Song A"), "its own track");
    assert!(shown(&h, "Song B"), "its subcategory's track");
    assert!(!shown(&h, "Song C"));
    assert!(!shown(&h, "Song D"));
}

#[test]
fn typing_filters_as_you_type() {
    let mut s = base();
    add(&mut s, &["Canción del mar", "Other tune"], None);
    let (mut h, fake) = open(s);
    assert!(shown(&h, "Other tune"));
    search(&mut h, "cancion");
    assert!(shown(&h, "Canción del mar"), "case and accents folded");
    assert!(!shown(&h, "Other tune"));
    assert!(
        views(&fake.take_sent()).is_empty(),
        "the search text is not saved"
    );
}

#[test]
fn whole_library_searches_beyond_the_category() {
    let mut s = base();
    let rock = create(&mut s, None, "Rock");
    add(&mut s, &["Song A"], Some(rock));
    add(&mut s, &["Song C"], None);
    s.library_view.selection = LibrarySelection::Category(rock);
    let (mut h, fake) = open(s);
    search(&mut h, "song");
    assert_eq!(
        h.get_by_label("Search the library").value().as_deref(),
        Some("song")
    );
    assert!(shown(&h, "Song A"));
    assert!(!shown(&h, "Song C"), "outside the category");
    h.get_by_label("Whole library").click();
    h.run_steps(2);
    let sent = views(&fake.take_sent());
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert!(sent[0].whole_library);
    assert_eq!(sent[0].rect, Some([20.0, 60.0, 1100.0, 680.0]));
    assert!(shown(&h, "Song A"));
    assert!(shown(&h, "Song C"), "the whole library is searched");
}

#[test]
fn only_visible_rows_are_laid_out() {
    let mut s = base();
    let names: Vec<String> = (0..20_000).map(|n| format!("Row {n:05}")).collect();
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    add(&mut s, &names, None);
    let (h, _) = open(s);
    let laid_out = h.query_all_by_label_contains("Row ").count();
    assert!(laid_out > 0, "some rows are shown");
    assert!(laid_out < 100, "{laid_out} rows laid out");
}

#[test]
fn clicking_a_header_sorts_and_again_reverses() {
    let mut s = base();
    add(&mut s, &["B song", "A song", "C song"], None);
    let (mut h, fake) = open(s);
    let titles = ["A song", "B song", "C song"];
    h.get_by_role_and_label(Role::ColumnHeader, "Title").click();
    h.run_steps(2);
    let sent = views(&fake.take_sent());
    assert_eq!(sent.len(), 1, "{sent:?}");
    let by_title = LibraryColumn::Tag(TableColumn::Title);
    assert_eq!(
        sent[0].sort,
        Some(LibrarySort {
            column: by_title,
            ascending: true
        })
    );
    assert_eq!(order(&h, &titles), ["A song", "B song", "C song"]);
    h.get_by_role_and_label(Role::ColumnHeader, "Title").click();
    h.run_steps(2);
    let sent = views(&fake.take_sent());
    assert_eq!(
        sent[0].sort,
        Some(LibrarySort {
            column: by_title,
            ascending: false
        })
    );
    assert_eq!(order(&h, &titles), ["C song", "B song", "A song"]);
}

#[test]
fn shift_click_selects_a_range() {
    let mut s = base();
    add(&mut s, &["T1", "T2", "T3", "T4", "T5"], None);
    let (mut h, _) = open(s);
    h.get_by_label("T2").click();
    h.run_steps(2);
    assert!(is_selected(&h, "T2"));
    assert!(!is_selected(&h, "T1"));
    h.get_by_label("T4").click_modifiers(Modifiers::SHIFT);
    h.run_steps(2);
    for t in ["T2", "T3", "T4"] {
        assert!(is_selected(&h, t), "{t} selected");
    }
    for t in ["T1", "T5"] {
        assert!(!is_selected(&h, t), "{t} not selected");
    }
    h.get_by_label("T1").click_modifiers(Modifiers::COMMAND);
    h.run_steps(2);
    for t in ["T1", "T2", "T3", "T4"] {
        assert!(is_selected(&h, t), "{t} added");
    }
    h.get_by_label("T5").click();
    h.run_steps(2);
    assert_eq!(selected_rows(&h).len(), 1, "a plain click selects one row");
    assert!(is_selected(&h, "T5"));
}

#[test]
fn a_missing_track_shows_the_missing_flag() {
    let mut s = base();
    add(&mut s, &["Gone", "Here"], None);
    let gone = s.library.track_at(&lib_path("Gone")).unwrap();
    s.library.get_mut(gone).unwrap().file_state = FileState::Missing;
    let (h, _) = open(s);
    let tip = format!("File not found: {}", lib_path("Gone").display());
    let flags: Vec<_> = h.query_all_by_label(&tip).collect();
    assert_eq!(flags.len(), 1, "one flag, on the missing track's row");
    let row = h.get_by_label("Gone").rect();
    assert!((flags[0].rect().center().y - row.center().y).abs() < 6.0);
}

#[derive(Debug)]
struct Dropped(PathBuf);

impl egui::DroppedFile for Dropped {
    fn path(&self) -> &Path {
        &self.0
    }
    fn bytes(&self) -> Result<Vec<u8>, String> {
        std::fs::read(&self.0).map_err(|e| e.to_string())
    }
}

fn drop_file(h: &mut H, path: &Path) {
    h.input_mut()
        .dropped_files
        .push(std::sync::Arc::new(Dropped(path.to_path_buf())));
    // Dropped paths are read on a helper thread: give it time to answer.
    for _ in 0..40 {
        h.run_steps(1);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

/// Drops a new audio file on the middle of the library window and returns
/// the file and the commands sent.
fn drop_on_window(s: AppState) -> (PathBuf, Vec<Command>) {
    let dir = tempfile::tempdir().unwrap();
    let base = canonical_path(dir.path());
    let file = base.join("new.wav");
    std::fs::write(&file, b"x").unwrap();
    let (mut h, fake) = open(s);
    h.hover_at(egui::pos2(600.0, 400.0));
    h.step();
    drop_file(&mut h, &file);
    (file, fake.take_sent())
}

#[test]
fn files_dropped_on_the_window_join_the_selected_category() {
    let mut s = base();
    let rock = create(&mut s, None, "Rock");
    add(&mut s, &["Song A"], Some(rock));
    s.library_view.selection = LibrarySelection::Category(rock);
    let (file, sent) = drop_on_window(s);
    assert!(
        sent.iter().any(|c| matches!(
            c,
            Command::AddToLibrary { paths, category: Some(c) } if paths == &vec![file.clone()] && *c == rock
        )),
        "{sent:?}"
    );
    assert!(
        !sent
            .iter()
            .any(|c| matches!(c, Command::InsertPaths { .. })),
        "nothing goes to a playlist"
    );
}

#[test]
fn files_dropped_with_all_tracks_selected_are_uncategorised() {
    let mut s = base();
    add(&mut s, &["Song A"], None);
    let (file, sent) = drop_on_window(s);
    assert!(
        sent.iter().any(|c| matches!(
            c,
            Command::AddToLibrary { paths, category: None } if paths == &vec![file.clone()]
        )),
        "{sent:?}"
    );
}

#[test]
fn dragging_a_column_edge_saves_the_widths() {
    use egui::{Event, PointerButton, pos2};
    let mut s = base();
    add(&mut s, &["Song A"], None);
    let (mut h, fake) = open(s);
    let title = h.get_by_role_and_label(Role::ColumnHeader, "Title").rect();
    let from = pos2(title.right(), title.center().y);
    let to = pos2(title.right() + 60.0, title.center().y);
    let press = |h: &mut H, at, pressed| {
        h.event(Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        });
    };
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    press(&mut h, from, true);
    h.run_steps(1);
    for step in 1..=10 {
        let x = from.x + (to.x - from.x) * step as f32 / 10.0;
        h.event(Event::PointerMoved(pos2(x, from.y)));
        h.run_steps(1);
    }
    press(&mut h, to, false);
    h.run_steps(3);
    let sent = views(&fake.take_sent());
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert!(sent[0].sort.is_none(), "a drag does not sort");
    let width = |c: LibraryColumn| {
        sent[0]
            .columns
            .iter()
            .find(|w| w.column == c)
            .map(|w| w.width)
            .unwrap()
    };
    let wider = width(LibraryColumn::Tag(TableColumn::Title));
    assert!((wider - title.width() - 60.0).abs() < 2.0, "{wider}");
    let after = h.get_by_role_and_label(Role::ColumnHeader, "Title").rect();
    assert!(
        (after.width() - wider).abs() < 2.0,
        "the table keeps the width"
    );
}

/// I3: the sort keys are kept between snapshots and follow a library
/// change: a retitled track moves to its new place in a sorted table, and
/// a sort by another column orders by that column's own keys.
#[test]
fn a_sorted_table_follows_a_retitled_track() {
    let mut s = base();
    add(&mut s, &["B song", "A song", "C song"], None);
    s.library_view.sort = Some(LibrarySort {
        column: LibraryColumn::Tag(TableColumn::Title),
        ascending: true,
    });
    let (mut h, fake) = open(s);
    assert_eq!(
        order(&h, &["A song", "B song", "C song"]),
        ["A song", "B song", "C song"]
    );
    let mut next = (*fake.state.load_full()).clone();
    let id = next
        .library
        .iter()
        .find(|t| t.title == "A song")
        .unwrap()
        .id;
    next.library.get_mut(id).unwrap().title = "Z song".into();
    fake.state.store(std::sync::Arc::new(next.clone()));
    h.run_steps(2);
    assert_eq!(
        order(&h, &["Z song", "B song", "C song"]),
        ["B song", "C song", "Z song"]
    );
    // Descending by file name: the stems, which kept "A song" for Z.
    next.library_view.sort = Some(LibrarySort {
        column: LibraryColumn::Tag(TableColumn::FileName),
        ascending: false,
    });
    fake.state.store(std::sync::Arc::new(next));
    h.run_steps(2);
    assert_eq!(
        order(&h, &["Z song", "B song", "C song"]),
        ["C song", "B song", "Z song"]
    );
}
