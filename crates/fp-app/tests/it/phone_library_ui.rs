#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The phone's Library tab (track library spec §11.1, LB38, LB39): the
//! category selector, a category's list and Device music.

use crate::support;

use std::path::PathBuf;
use std::sync::Arc;

use egui::{Vec2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use fp_app::ui::app::AppUi;
use fp_app::ui::controller::Controller;
use fp_app::ui::media_browser::{BrowseMode, MediaBrowser, MediaFile};
use fp_app::ui::phone_library::{LibraryPage, PhoneLibraryView};
use fp_model::library_view::LibrarySelection;
use fp_model::{AppState, Categories, CategoryId, Command, TrackId, TrackKind};
use support::{Fake, harness_sized, state};

type H = Harness<'static, AppUi>;

const PORTRAIT: Vec2 = vec2(412.0, 892.0);
const BACK: &str = "Back to the categories";
const DEVICE: &str = "Device music";
const ALPHA: &str = "Alpha · Other · 01:00";

fn create(s: &mut AppState, name: &str) -> CategoryId {
    fp_model::apply(
        s,
        Command::CreateCategory {
            parent: None,
            name: name.into(),
            color: None,
            kind: TrackKind::Music,
        },
    )
    .unwrap();
    s.categories.iter().find(|c| c.name == name).unwrap().id
}

/// Two players, the categories Rock and Jazz.
fn with_categories() -> (AppState, CategoryId) {
    let mut s = state(2, 3);
    let rock = create(&mut s, "Rock");
    create(&mut s, "Jazz");
    (s, rock)
}

fn files() -> Vec<MediaFile> {
    vec![MediaFile {
        path: PathBuf::from("/sd/Songs/a.mp3"),
        title: Some("Alpha".into()),
        artist: Some("Other".into()),
        album: None,
        duration_secs: Some(60.0),
    }]
}

/// A phone with the device's music, on Live.
fn phone(s: AppState) -> (H, MediaBrowser, Arc<Fake>) {
    let browser = MediaBrowser::default();
    browser.set_files(files());
    let given = browser.clone();
    let (mut h, fake) = harness_sized(s, PORTRAIT, move |ui| {
        ui.with_touch().with_media_browser(given)
    });
    h.run_steps(3);
    (h, browser, fake)
}

fn tap(h: &mut H, label: &str) {
    h.get_by_label(label).click();
    h.run_steps(3);
}

fn back(h: &mut H) -> bool {
    let ctx = h.ctx.clone();
    let went = h.state_mut().back(&ctx);
    h.run_steps(3);
    went
}

fn on_selector(h: &H) -> bool {
    h.query_by_label("All tracks").is_some() && h.query_by_label(DEVICE).is_some()
}

fn on_live(h: &H) -> bool {
    h.query_by_label("Stop all").is_some() && !on_selector(h)
}

/// LB38: the tab opens on the selector, in order.
#[test]
fn lb38_the_library_tab_opens_on_the_selector() {
    let (s, _rock) = with_categories();
    let (mut h, _browser, _fake) = phone(s);
    tap(&mut h, "Library");
    // The categories in the tree's order (as created, here).
    let names = ["All tracks", "Rock", "Jazz", "Uncategorised", DEVICE];
    let tops: Vec<f32> = names
        .iter()
        .map(|n| h.get_by_label(n).rect().top())
        .collect();
    assert!(tops.windows(2).all(|w| w[0] < w[1]), "{names:?} {tops:?}");
}

/// LB38: each row is tall enough for a finger.
#[test]
fn lb38_the_selector_rows_are_finger_sized() {
    let (s, _rock) = with_categories();
    let (mut h, _browser, _fake) = phone(s);
    tap(&mut h, "Library");
    for name in ["All tracks", "Rock", "Uncategorised", DEVICE] {
        let rect = h.get_by_label(name).rect();
        // touch::TARGET, a finger's 48 pt.
        assert!(rect.height() >= 48.0, "{name} {rect:?}");
    }
}

/// LB38: a subcategory is indented under its parent, and a row shows its
/// count.
#[test]
fn lb38_a_subcategory_is_indented_and_rows_show_counts() {
    let (mut s, rock) = with_categories();
    fp_model::apply(
        &mut s,
        Command::CreateCategory {
            parent: Some(rock),
            name: "Classic rock".into(),
            color: None,
            kind: TrackKind::Music,
        },
    )
    .unwrap();
    let (mut h, _browser, _fake) = phone(s);
    tap(&mut h, "Library");
    let rock_row = h.get_by_label("Rock");
    let classic = h.get_by_label("Classic rock");
    assert!(rock_row.rect().top() < classic.rect().top());
    assert_eq!(
        rock_row.accesskit_node().value().as_deref(),
        Some("0"),
        "the count"
    );
}

/// LB39, PL2: a category opens its list; the header's back button and the
/// system Back return to the selector, and Back from there leaves the tab.
#[test]
fn lb39_a_category_opens_its_list_and_back_returns() {
    let (s, rock) = with_categories();
    let (mut h, _browser, fake) = phone(s);
    tap(&mut h, "Library");
    fake.take_sent();
    tap(&mut h, "Rock");
    let sent = fake.take_sent();
    assert!(
        sent.iter().any(|c| matches!(c, Command::SetLibraryView(v)
            if v.selection == LibrarySelection::Category(rock))),
        "{sent:?}"
    );
    assert!(h.query_by_label(BACK).is_some(), "the list's header");
    assert!(!on_selector(&h));
    tap(&mut h, BACK);
    assert!(on_selector(&h));
    tap(&mut h, "Rock");
    assert!(back(&mut h), "the system Back");
    assert!(on_selector(&h));
    assert!(back(&mut h), "Back from the selector");
    assert!(on_live(&h));
}

/// LB38: Device music is today's Music screen; Back returns to the
/// selector.
#[test]
fn lb38_device_music_is_todays_music_screen() {
    let (s, _rock) = with_categories();
    let (mut h, _browser, _fake) = phone(s);
    tap(&mut h, "Library");
    tap(&mut h, DEVICE);
    for name in ["Folders", "Artists", "Albums"] {
        assert!(h.query_by_label(name).is_some(), "{name}");
    }
    assert!(h.query_by_label(ALPHA).is_some());
    assert!(back(&mut h));
    assert!(on_selector(&h));
}

/// PL19b: a pick opens the Library tab on Device music and returns to the
/// page that was shown when it asked.
#[test]
fn pl19b_a_pick_opens_device_music_and_returns() {
    let (s, _rock) = with_categories();
    let (mut h, browser, _fake) = phone(s);
    tap(&mut h, "Library");
    tap(&mut h, "Rock");
    let mode = BrowseMode::one_file(&["mp3"]).unwrap();
    let thread = std::thread::spawn(move || browser.browse(mode, None));
    for _ in 0..200 {
        h.run_steps(1);
        if h.query_by_label(ALPHA).is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(h.query_by_label("Folders").is_some(), "Device music");
    tap(&mut h, ALPHA);
    for _ in 0..200 {
        h.run_steps(1);
        if thread.is_finished() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    h.run_steps(3);
    assert_eq!(thread.join().unwrap(), [PathBuf::from("/sd/Songs/a.mp3")]);
    assert!(h.query_by_label(BACK).is_some(), "back on Rock's list");
    assert!(h.query_by_label("Folders").is_none());
}

/// LB39 (settled details): an open category that goes away sends the tab
/// back to the selector.
#[test]
fn a_deleted_open_category_returns_to_the_selector() {
    let (s, rock) = with_categories();
    let (mut h, _browser, fake) = phone(s);
    tap(&mut h, "Library");
    tap(&mut h, "Rock");
    assert!(h.query_by_label(BACK).is_some());
    assert!(fake.send(Command::DeleteCategory(rock)));
    h.run_steps(3);
    assert!(on_selector(&h));
    assert!(h.query_by_label("Rock").is_none());
}

// --- The page rules, without a screen ---

fn categories() -> (Categories, CategoryId) {
    let (s, rock) = with_categories();
    (s.categories.clone(), rock)
}

fn ticked() -> Vec<TrackId> {
    let mut s = state(1, 0);
    fp_model::apply(
        &mut s,
        Command::AddToLibrary {
            paths: vec![PathBuf::from("/music/x.mp3")],
            category: None,
        },
    )
    .unwrap();
    s.library.ids()
}

#[test]
fn back_from_a_list_goes_to_the_selector_and_clears_the_ticks() {
    let mut view = PhoneLibraryView {
        page: LibraryPage::Tracks,
        ticked: ticked(),
        ..PhoneLibraryView::default()
    };
    assert!(view.back());
    assert_eq!(view.page, LibraryPage::Selector);
    assert!(view.ticked.is_empty());
    assert!(!view.back(), "the selector is the tab's top");
}

#[test]
fn back_from_device_music_goes_where_it_was_opened_from() {
    let mut view = PhoneLibraryView {
        page: LibraryPage::Device,
        ..PhoneLibraryView::default()
    };
    assert!(view.back());
    assert_eq!(view.page, LibraryPage::Selector);
    let mut view = PhoneLibraryView {
        page: LibraryPage::Device,
        adding_to: Some(None),
        ..PhoneLibraryView::default()
    };
    assert!(view.back());
    assert_eq!(view.page, LibraryPage::Tracks);
    assert_eq!(view.adding_to, None);
}

#[test]
fn a_list_whose_category_is_gone_settles_on_the_selector() {
    let (categories, rock) = categories();
    let mut view = PhoneLibraryView {
        page: LibraryPage::Tracks,
        ticked: ticked(),
        ..PhoneLibraryView::default()
    };
    view.shown = Some(rock);
    view.settle(&categories);
    assert_eq!(view.page, LibraryPage::Tracks, "Rock is still there");
    view.settle(&Categories::default());
    assert_eq!(view.page, LibraryPage::Selector);
    assert!(view.ticked.is_empty());
}
