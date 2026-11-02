#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::float_cmp
)]
//! Track library spec §7.3 (LB9–LB11, LB27): the library window's category
//! tree, its dialogs and the two removal confirmations.

use crate::support;

use egui::accesskit::Role;
use egui::{Event, Modifiers, PointerButton, Pos2, pos2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_model::library_view::{LibrarySelection, LibraryView};
use fp_model::{AppState, CartColor, CategoryId, Command, TrackKind};
use support::{Fake, harness_sized, nth_track, state};

type H = Harness<'static, AppUi>;

/// The categories of `tree_state`.
struct Tree {
    rock: CategoryId,
    classic: CategoryId,
    jingles: CategoryId,
}

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

fn file(s: &mut AppState, n: usize, category: CategoryId) {
    let track = nth_track(s, n);
    fp_model::apply(
        s,
        Command::MoveTracksToCategory {
            tracks: vec![track],
            category: Some(category),
        },
    )
    .unwrap();
}

/// Rock (Song 1) with Classic (Song 2) under it, Jingles (empty), and
/// Song 3 uncategorised; the window open with Rock expanded.
fn tree_state() -> (AppState, Tree) {
    let mut s = state(1, 3);
    let rock = create(&mut s, None, "Rock");
    let classic = create(&mut s, Some(rock), "Classic");
    let jingles = create(&mut s, None, "Jingles");
    file(&mut s, 0, rock);
    file(&mut s, 1, classic);
    s.library_view = LibraryView {
        open: true,
        rect: Some([20.0, 60.0, 900.0, 600.0]),
        expanded: vec![rock],
        ..LibraryView::default()
    };
    (
        s,
        Tree {
            rock,
            classic,
            jingles,
        },
    )
}

fn open(s: AppState) -> (H, std::sync::Arc<Fake>) {
    let (mut h, fake) = harness_sized(s, vec2(1200.0, 800.0), |ui| ui);
    h.run_steps(2);
    fake.take_sent();
    (h, fake)
}

/// The library views sent since the last call.
fn views(sent: &[Command]) -> Vec<LibraryView> {
    sent.iter()
        .filter_map(|c| match c {
            Command::SetLibraryView(v) => Some((**v).clone()),
            _ => None,
        })
        .collect()
}

/// The commands sent, without the library views.
fn others(sent: Vec<Command>) -> Vec<Command> {
    sent.into_iter()
        .filter(|c| !matches!(c, Command::SetLibraryView(_)))
        .collect()
}

/// The count a tree item shows.
fn count(h: &H, item: &str) -> String {
    h.get_by_label(item).value().expect("the item's count")
}

fn menu(h: &mut H, item: &str) {
    h.get_by_label(item).click_secondary();
    h.run_steps(2);
}

fn press(h: &mut H, at: Pos2, pressed: bool) {
    h.event(Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    });
}

/// Presses at `from`, moves to `to` in steps and releases there.
fn drag(h: &mut H, from: Pos2, to: Pos2) {
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    press(h, from, true);
    h.run_steps(1);
    for step in 1..=10 {
        let t = step as f32 / 10.0;
        h.event(Event::PointerMoved(pos2(
            from.x + (to.x - from.x) * t,
            from.y + (to.y - from.y) * t,
        )));
        h.run_steps(1);
    }
    press(h, to, false);
    h.run_steps(3);
}

#[test]
fn the_tree_lists_all_categories_and_uncategorised_with_counts() {
    let (s, _) = tree_state();
    let (h, _) = open(s);
    let order: Vec<f32> = ["All tracks", "Rock", "Classic", "Jingles", "Uncategorised"]
        .iter()
        .map(|item| h.get_by_label(item).rect().top())
        .collect();
    assert!(order.windows(2).all(|w| w[0] < w[1]), "{order:?}");
    assert_eq!(count(&h, "All tracks"), "3");
    assert_eq!(count(&h, "Rock"), "2", "its own and its subcategory's");
    assert_eq!(count(&h, "Classic"), "1");
    assert_eq!(count(&h, "Jingles"), "0");
    assert_eq!(count(&h, "Uncategorised"), "1");
}

#[test]
fn selecting_a_category_sends_the_selection() {
    let (s, tree) = tree_state();
    let (mut h, fake) = open(s);
    h.get_by_label("Jingles").click();
    h.run_steps(2);
    let sent = views(&fake.take_sent());
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert_eq!(sent[0].selection, LibrarySelection::Category(tree.jingles));
    assert_eq!(sent[0].rect, Some([20.0, 60.0, 900.0, 600.0]));
    h.get_by_label("Uncategorised").click();
    h.run_steps(2);
    let sent = views(&fake.take_sent());
    assert_eq!(sent[0].selection, LibrarySelection::Uncategorised);
}

#[test]
fn collapsing_a_category_hides_its_children_and_saves_it() {
    let (s, tree) = tree_state();
    let (mut h, fake) = open(s);
    assert!(h.query_by_label("Classic").is_some());
    h.get_by_label("Collapse Rock").click();
    h.run_steps(3);
    let sent = views(&fake.take_sent());
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert!(sent[0].expanded.is_empty(), "{:?}", sent[0].expanded);
    assert!(h.query_by_label("Classic").is_none(), "Rock's child hidden");
    assert_eq!(
        fake.state.load().library_view.selection,
        LibrarySelection::All,
        "the arrow does not select"
    );
    h.get_by_label("Expand Rock").click();
    h.run_steps(3);
    let sent = views(&fake.take_sent());
    assert_eq!(sent[0].expanded, vec![tree.rock]);
    assert!(h.query_by_label("Classic").is_some());
}

#[test]
fn plus_creates_a_category_under_the_selected_one() {
    let (mut s, tree) = tree_state();
    s.library_view.selection = LibrarySelection::Category(tree.classic);
    let (mut h, fake) = open(s);
    h.get_by_label("New category").click();
    h.run_steps(3);
    h.get_by_label("Name").type_text("  Seventies ");
    h.run_steps(2);
    h.get_by_label("Create").click();
    h.run_steps(3);
    assert_eq!(
        others(fake.take_sent()),
        vec![Command::CreateCategory {
            parent: Some(tree.classic),
            name: "Seventies".into(),
            color: None,
            kind: TrackKind::Music,
        }]
    );
    assert!(
        h.query_by_label("Seventies").is_some(),
        "Classic expanded to show it"
    );
    // With All tracks selected, ＋ creates at the top level.
    h.get_by_label("All tracks").click();
    h.run_steps(2);
    h.get_by_label("New category").click();
    h.run_steps(3);
    h.get_by_label("Name").type_text("Ads");
    h.run_steps(2);
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    let created = others(fake.take_sent());
    assert!(
        matches!(&created[..], [Command::CreateCategory { parent: None, name, .. }] if name == "Ads"),
        "{created:?}"
    );
}

/// Opens Rename… on `item` and types `text` over its name (selected when
/// the dialog opens).
fn rename(h: &mut H, item: &str, text: &str) {
    menu(h, item);
    h.get_by_label("Rename…").click();
    h.run_steps(3);
    assert_eq!(h.get_by_label("Name").value().as_deref(), Some(item));
    h.get_by_label("Name").type_text(text);
    h.run_steps(2);
    h.get_by_label("Rename").click();
    h.run_steps(3);
}

#[test]
fn rename_sends_the_trimmed_name() {
    let (s, tree) = tree_state();
    let (mut h, fake) = open(s);
    rename(&mut h, "Jingles", "  Station IDs ");
    assert_eq!(
        others(fake.take_sent()),
        vec![Command::RenameCategory {
            category: tree.jingles,
            name: "Station IDs".into(),
        }]
    );
    assert!(h.query_by_label("Station IDs").is_some());
}

#[test]
fn a_taken_name_shows_the_error() {
    let (s, tree) = tree_state();
    let (mut h, fake) = open(s);
    rename(&mut h, "Jingles", "rock");
    assert_eq!(
        others(fake.take_sent()),
        vec![Command::RenameCategory {
            category: tree.jingles,
            name: "rock".into(),
        }]
    );
    assert!(
        h.query_by_label("There is already a category with that name here.")
            .is_some(),
        "the model's error is a notice"
    );
    assert!(h.query_by_label("Jingles").is_some(), "the name is kept");
}

#[test]
fn colour_and_kind_menus_send_their_commands() {
    let (s, tree) = tree_state();
    let (mut h, fake) = open(s);
    menu(&mut h, "Jingles");
    h.get_by_label("Colour").click();
    h.run_steps(2);
    h.get_by_label("Teal").click();
    h.run_steps(2);
    menu(&mut h, "Jingles");
    h.get_by_label("Kind").click();
    h.run_steps(2);
    h.get_by_label("Jingle").click();
    h.run_steps(2);
    assert_eq!(
        others(fake.take_sent()),
        vec![
            Command::SetCategoryColor {
                category: tree.jingles,
                color: Some(CartColor::Teal),
            },
            Command::SetCategoryKind {
                category: tree.jingles,
                kind: TrackKind::Jingle,
            },
        ]
    );
}

#[test]
fn dropping_a_category_on_another_moves_it_under_it() {
    let (s, tree) = tree_state();
    let (mut h, fake) = open(s);
    let from = h.get_by_label("Jingles").rect().center();
    let to = h.get_by_label("Classic").rect().center();
    drag(&mut h, from, to);
    assert_eq!(
        others(fake.take_sent()),
        vec![Command::MoveCategory {
            category: tree.jingles,
            parent: Some(tree.classic),
            index: 0,
        }]
    );
    // Into the gap above Rock: the top level, first.
    let from = h.get_by_label("Classic").rect().center();
    let rock = h.get_by_label("Rock").rect();
    drag(&mut h, from, pos2(rock.center().x, rock.top() + 1.0));
    assert_eq!(
        others(fake.take_sent()),
        vec![Command::MoveCategory {
            category: tree.classic,
            parent: None,
            index: 0,
        }]
    );
}

fn confirm_title(h: &H, text: &str) -> bool {
    h.query_by_label(text).is_some()
}

#[test]
fn deleting_an_unused_category_asks_once() {
    let (s, tree) = tree_state();
    let (mut h, fake) = open(s);
    menu(&mut h, "Jingles");
    h.get_by_label("Delete…").click();
    h.run_steps(3);
    assert!(confirm_title(&h, "Delete “Jingles”?"));
    assert!(
        h.query_by_label("No track is removed from the library.")
            .is_some()
    );
    assert!(
        others(fake.take_sent()).is_empty(),
        "nothing before the answer"
    );
    h.get_by_label("Delete").click();
    h.run_steps(3);
    assert_eq!(
        others(fake.take_sent()),
        vec![Command::DeleteCategory(tree.jingles)]
    );
    assert!(!confirm_title(&h, "Delete “Jingles”?"));
}

#[test]
fn deleting_a_used_category_asks_twice_with_the_counts() {
    let (s, tree) = tree_state();
    let (mut h, fake) = open(s);
    menu(&mut h, "Rock");
    h.get_by_label("Delete…").click();
    h.run_steps(3);
    assert!(confirm_title(&h, "Delete “Rock”?"));
    assert!(
        h.query_by_label("Its 1 subcategory is deleted with it.")
            .is_some()
    );
    assert!(
        h.query_by_label("2 tracks are removed from the library.")
            .is_some()
    );
    h.get_by_label("Delete").click();
    h.run_steps(3);
    assert!(others(fake.take_sent()).is_empty(), "a second question");
    assert!(
        h.query_by_label_contains("2 playlist entries and 0 carts use these tracks")
            .is_some()
    );
    h.get_by_label("Remove anyway").click();
    h.run_steps(3);
    assert_eq!(
        others(fake.take_sent()),
        vec![Command::DeleteCategory(tree.rock)]
    );
}

#[test]
fn cancelling_the_second_question_deletes_nothing() {
    let (s, _) = tree_state();
    let (mut h, fake) = open(s);
    menu(&mut h, "Rock");
    h.get_by_label("Delete…").click();
    h.run_steps(3);
    h.get_by_label("Delete").click();
    h.run_steps(3);
    h.get_by_label("Cancel").click();
    h.run_steps(3);
    assert!(others(fake.take_sent()).is_empty());
    assert!(h.query_by_label("Remove anyway").is_none());
}

#[test]
fn deleting_a_category_on_air_is_refused_before_asking() {
    let (mut s, _) = tree_state();
    let first = s.playlists.first_id().unwrap();
    let e0 = s.playlists.get(first).unwrap().entries[0].id;
    s.players[0].current = Some(e0);
    s.players[0].transport = fp_model::Transport::Playing;
    let (mut h, fake) = open(s);
    menu(&mut h, "Rock");
    h.get_by_label("Delete…").click();
    h.run_steps(3);
    assert!(!confirm_title(&h, "Delete “Rock”?"), "no question");
    assert!(
        h.query_by_label(
            "Some of these tracks are on air: 1 in playlists and 0 on the cartwall. \
             Nothing was removed."
        )
        .is_some()
    );
    assert!(others(fake.take_sent()).is_empty());
}

#[test]
fn a_menu_on_all_tracks_has_no_category_actions() {
    let (s, _) = tree_state();
    let (mut h, _) = open(s);
    h.get_by_label("All tracks").click_secondary();
    h.run_steps(2);
    assert!(h.query_by_label("Rename…").is_none());
    assert!(h.query_by_label("Delete…").is_none());
    assert!(
        h.query_all_by_role(Role::TextInput).next().is_none(),
        "no dialog"
    );
}

/// The gap below an open category with children is where its first child
/// goes: the line is drawn there.
#[test]
fn dropping_below_an_open_category_makes_it_the_first_child() {
    let (s, tree) = tree_state();
    let (mut h, fake) = open(s);
    let from = h.get_by_label("Jingles").rect().center();
    let rock = h.get_by_label("Rock").rect();
    drag(&mut h, from, pos2(rock.center().x, rock.bottom() - 1.0));
    assert_eq!(
        others(fake.take_sent()),
        vec![Command::MoveCategory {
            category: tree.jingles,
            parent: Some(tree.rock),
            index: 0,
        }]
    );
}

/// A category dropped onto a closed one opens it, so the moved category
/// stays in sight.
#[test]
fn dropping_onto_a_closed_category_opens_it() {
    let (s, tree) = tree_state();
    let (mut h, fake) = open(s);
    let from = h.get_by_label("Classic").rect().center();
    let to = h.get_by_label("Jingles").rect().center();
    drag(&mut h, from, to);
    let sent = fake.take_sent();
    assert_eq!(
        others(sent.clone()),
        vec![Command::MoveCategory {
            category: tree.classic,
            parent: Some(tree.jingles),
            index: 0,
        }]
    );
    let views = views(&sent);
    assert_eq!(views.len(), 1, "{views:?}");
    assert!(views[0].expanded.contains(&tree.jingles), "{views:?}");
    assert!(views[0].expanded.contains(&tree.rock), "kept: {views:?}");
    h.run_steps(2);
    assert!(h.query_by_label("Classic").is_some(), "still in sight");
}

/// What went on air while the question was open is judged on the answer:
/// the removal is refused, with the notice, and nothing is sent.
#[test]
fn a_removal_that_went_on_air_meanwhile_is_refused_on_the_answer() {
    let (s, _) = tree_state();
    let (mut h, fake) = open(s);
    menu(&mut h, "Rock");
    h.get_by_label("Delete…").click();
    h.run_steps(3);
    assert!(confirm_title(&h, "Delete “Rock”?"));
    let mut now = (**fake.state.load()).clone();
    let first = now.playlists.first_id().unwrap();
    let e0 = now.playlists.get(first).unwrap().entries[0].id;
    now.players[0].current = Some(e0);
    now.players[0].transport = fp_model::Transport::Playing;
    fake.state.store(std::sync::Arc::new(now));
    h.run_steps(2);
    h.get_by_label("Delete").click();
    h.run_steps(3);
    assert!(others(fake.take_sent()).is_empty());
    assert!(!confirm_title(&h, "Delete “Rock”?"), "the question closed");
    assert!(h.query_by_label("Remove anyway").is_none());
    assert!(
        h.query_by_label(
            "Some of these tracks are on air: 1 in playlists and 0 on the cartwall. \
             Nothing was removed."
        )
        .is_some()
    );
}
