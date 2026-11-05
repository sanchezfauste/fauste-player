#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The phone's Library tab (track library spec §11.1, LB38–LB42): the
//! category selector, a category's track list and Device music.

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
use fp_model::{AppState, Categories, CategoryId, Command, FileState, TrackId, TrackKind};
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

// --- The track list (LB40–LB42) ---

const SEARCH: &str = "Search tracks";
const WHOLE: &str = "Whole library";
const END: &str = "Add at the end";
const AFTER: &str = "Add after the next";

/// Adds a library track from `/lib/<file>.mp3` with these tags.
fn track(
    s: &mut AppState,
    file: &str,
    title: &str,
    artist: &str,
    category: Option<CategoryId>,
) -> TrackId {
    let path = PathBuf::from(format!("/lib/{file}.mp3"));
    fp_model::apply(
        s,
        Command::AddToLibrary {
            paths: vec![path.clone()],
            category,
        },
    )
    .unwrap();
    let id = s.library.track_at(&path).unwrap();
    let t = s.library.get_mut(id).unwrap();
    t.title = title.into();
    t.artist = artist.into();
    t.duration_secs = 205.0;
    id
}

/// A row's accessible name: title · artist · duration.
fn row(title: &str, artist: &str) -> String {
    format!("{title} · {artist} · 03:25")
}

/// A phone on Rock's list.
fn rock_list(s: AppState) -> (H, Arc<Fake>) {
    let (mut h, _browser, fake) = phone(s);
    tap(&mut h, "Library");
    tap(&mut h, "Rock");
    fake.take_sent();
    (h, fake)
}

fn search(h: &mut H, text: &str) {
    h.get_by_label(SEARCH).click();
    h.run_steps(2);
    h.get_by_label(SEARCH).type_text(text);
    h.run_steps(3);
}

fn is_ticked(h: &H, label: &str) -> bool {
    h.get_by_label(label).accesskit_node().toggled() == Some(egui::accesskit::Toggled::True)
}

fn inserted(fake: &Fake) -> Vec<(fp_model::PlaylistId, usize, Vec<TrackId>)> {
    fake.take_sent()
        .into_iter()
        .filter_map(|c| match c {
            Command::InsertTracks {
                playlist,
                index,
                tracks,
            } => Some((playlist, index, tracks)),
            _ => None,
        })
        .collect()
}

#[test]
fn lb40_the_search_filters_the_open_category() {
    let (mut s, rock) = with_categories();
    track(&mut s, "one", "Canción uno", "Band", Some(rock));
    track(&mut s, "other", "Other", "Band", Some(rock));
    let (mut h, _fake) = rock_list(s);
    assert!(h.query_by_label(&row("Other", "Band")).is_some());
    search(&mut h, "cancion");
    assert!(h.query_by_label(&row("Canción uno", "Band")).is_some());
    assert!(h.query_by_label(&row("Other", "Band")).is_none());
}

#[test]
fn lb40_whole_library_searches_everything() {
    let (mut s, rock) = with_categories();
    track(&mut s, "one", "Canción uno", "Band", Some(rock));
    track(&mut s, "two", "Canción dos", "Band", None);
    let (mut h, fake) = rock_list(s);
    search(&mut h, "cancion");
    assert!(h.query_by_label(&row("Canción dos", "Band")).is_none());
    fake.take_sent();
    tap(&mut h, WHOLE);
    let sent = fake.take_sent();
    assert!(
        sent.iter().any(|c| matches!(c, Command::SetLibraryView(v)
            if v.whole_library && v.selection == LibrarySelection::Category(rock))),
        "{sent:?}"
    );
    assert!(h.query_by_label(&row("Canción uno", "Band")).is_some());
    assert!(
        h.query_by_label(&row("Canción dos", "Band")).is_some(),
        "the whole library is searched"
    );
}

#[test]
fn lb41_a_row_shows_title_artist_duration_and_status() {
    let (mut s, rock) = with_categories();
    let id = track(&mut s, "gone", "Gone", "Band", Some(rock));
    s.library.get_mut(id).unwrap().file_state = FileState::Missing;
    // No title tag: the file name stands in.
    track(&mut s, "untitled", "", "Band", Some(rock));
    let (h, _fake) = rock_list(s);
    let label = row("Gone", "Band");
    let gone = h.get_by_label(&label);
    assert_eq!(
        gone.accesskit_node().role(),
        egui::accesskit::Role::CheckBox
    );
    let mark = h.get_by_label_contains("File not found");
    assert!(
        gone.rect().contains(mark.rect().center()),
        "the missing mark sits in its row"
    );
    assert!(h.query_by_label(&row("untitled.mp3", "Band")).is_some());
}

#[test]
fn lb41_rows_are_finger_sized() {
    let (mut s, rock) = with_categories();
    track(&mut s, "a", "A", "Band", Some(rock));
    let (h, _fake) = rock_list(s);
    assert!(h.get_by_label(&row("A", "Band")).rect().height() >= 48.0);
}

#[test]
fn lb42_ticks_add_in_tick_order_and_clear() {
    let (mut s, rock) = with_categories();
    let a = track(&mut s, "a", "A", "Band", Some(rock));
    let b = track(&mut s, "b", "B", "Band", Some(rock));
    let playlist = s.playlists.first_id().unwrap();
    let len = s.playlists.get(playlist).unwrap().entries.len();
    let (mut h, fake) = rock_list(s);
    assert!(h.query_by_label(END).is_none(), "no bar without ticks");
    tap(&mut h, &row("B", "Band"));
    tap(&mut h, &row("A", "Band"));
    assert!(is_ticked(&h, &row("A", "Band")));
    assert!(h.query_by_label("Selected: 2").is_some());
    fake.take_sent();
    tap(&mut h, END);
    assert_eq!(inserted(&fake), [(playlist, len, vec![b, a])]);
    assert!(!is_ticked(&h, &row("A", "Band")));
    assert!(!is_ticked(&h, &row("B", "Band")));
    assert!(h.query_by_label(END).is_none(), "the bar goes");
}

#[test]
fn lb42_add_after_the_next_uses_the_players_next() {
    let (mut s, rock) = with_categories();
    let a = track(&mut s, "a", "A", "Band", Some(rock));
    let playlist = s.playlists.first_id().unwrap();
    let first = s.playlists.get(playlist).unwrap().entries[0].id;
    let player = s.player_ids()[0];
    fp_model::apply(&mut s, Command::SetNext(player, first)).unwrap();
    let (mut h, fake) = rock_list(s);
    tap(&mut h, &row("A", "Band"));
    fake.take_sent();
    tap(&mut h, AFTER);
    assert_eq!(inserted(&fake), [(playlist, 1, vec![a])]);
}

#[test]
fn ticks_hidden_by_the_search_are_not_added() {
    let (mut s, rock) = with_categories();
    let a = track(&mut s, "a", "Alpha", "Band", Some(rock));
    track(&mut s, "b", "Beta", "Band", Some(rock));
    let playlist = s.playlists.first_id().unwrap();
    let len = s.playlists.get(playlist).unwrap().entries.len();
    let (mut h, fake) = rock_list(s);
    tap(&mut h, &row("Alpha", "Band"));
    tap(&mut h, &row("Beta", "Band"));
    search(&mut h, "alpha");
    assert!(h.query_by_label("Selected: 1").is_some());
    fake.take_sent();
    tap(&mut h, END);
    assert_eq!(inserted(&fake), [(playlist, len, vec![a])]);
}

#[test]
fn lb39_leaving_the_list_clears_its_ticks() {
    let (mut s, rock) = with_categories();
    track(&mut s, "a", "A", "Band", Some(rock));
    let (mut h, _fake) = rock_list(s);
    tap(&mut h, &row("A", "Band"));
    tap(&mut h, BACK);
    tap(&mut h, "Rock");
    assert!(!is_ticked(&h, &row("A", "Band")));
    assert!(h.query_by_label(END).is_none());
}

#[test]
fn an_empty_list_says_so() {
    let (s, _rock) = with_categories();
    let (h, _fake) = rock_list(s);
    assert!(h.query_by_label("No tracks here").is_some());
}

#[test]
fn lb41_twenty_thousand_tracks_lay_out_only_visible_rows() {
    let (mut s, rock) = with_categories();
    let paths: Vec<PathBuf> = (0..20_000)
        .map(|n| PathBuf::from(format!("/lib/Row {n:05}.mp3")))
        .collect();
    fp_model::apply(
        &mut s,
        Command::AddToLibrary {
            paths,
            category: Some(rock),
        },
    )
    .unwrap();
    let (h, _fake) = rock_list(s);
    let laid_out = h.query_all_by_label_contains("Row ").count();
    assert!(laid_out > 0, "some rows are shown");
    assert!(laid_out < 100, "{laid_out} rows laid out");
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
    view.shown = Some(LibrarySelection::Category(rock));
    view.settle(&categories);
    assert_eq!(view.page, LibraryPage::Tracks, "Rock is still there");
    view.settle(&Categories::default());
    assert_eq!(view.page, LibraryPage::Selector);
    assert!(view.ticked.is_empty());
}

// --- Pre-listen from the list (LB45, LB47) ---

const PAUSE: &str = "Pause pre-listen";
const STOP: &str = "Stop pre-listen";

fn headphones(title: &str) -> String {
    format!("Pre-listen {title}")
}

fn cue_commands(fake: &Fake) -> Vec<Command> {
    fake.take_sent()
        .into_iter()
        .filter(|c| {
            matches!(
                c,
                Command::SelectLibraryCue(_)
                    | Command::PlayLibraryCue
                    | Command::SetLibraryCuePaused(_)
                    | Command::StopLibraryCue
            )
        })
        .collect()
}

/// Rock with the tracks Alpha and Beta, and a library CUE route.
fn with_tracks() -> (AppState, TrackId, TrackId) {
    let (mut s, rock) = with_categories();
    support::give_library_cue_route(&mut s);
    let a = track(&mut s, "a", "Alpha", "Band", Some(rock));
    let b = track(&mut s, "b", "Beta", "Band", Some(rock));
    (s, a, b)
}

fn previewing(s: &mut AppState, track: TrackId) {
    fp_model::apply(s, Command::SelectLibraryCue(Some(track))).unwrap();
    fp_model::apply(s, Command::PlayLibraryCue).unwrap();
    assert_eq!(s.library_cue.transport, fp_model::Transport::Playing);
}

#[test]
fn lb45_the_headphones_start_the_track_from_its_cue_in() {
    let (s, a, _b) = with_tracks();
    let (mut h, fake) = rock_list(s);
    tap(&mut h, &headphones("Alpha"));
    assert_eq!(
        cue_commands(&fake),
        [Command::SelectLibraryCue(Some(a)), Command::PlayLibraryCue]
    );
    assert!(
        !is_ticked(&h, &row("Alpha", "Band")),
        "the row is not ticked"
    );
    assert_eq!(
        fake.state.load().library_cue.transport,
        fp_model::Transport::Playing
    );
}

#[test]
fn lb45_another_rows_headphones_switch_a_running_preview() {
    let (mut s, a, b) = with_tracks();
    previewing(&mut s, a);
    let (mut h, fake) = rock_list(s);
    tap(&mut h, &headphones("Beta"));
    assert_eq!(cue_commands(&fake), [Command::SelectLibraryCue(Some(b))]);
}

#[test]
fn lb45_the_strip_pauses_and_stops() {
    let (mut s, a, _b) = with_tracks();
    let (mut h, fake) = rock_list(s.clone());
    assert!(h.query_by_label(STOP).is_none(), "no strip while stopped");
    previewing(&mut s, a);
    fake.state.store(Arc::new(s));
    h.run_steps(3);
    assert!(h.query_by_label("Alpha").is_some(), "the strip's title");
    tap(&mut h, PAUSE);
    assert_eq!(cue_commands(&fake), [Command::SetLibraryCuePaused(true)]);
    // Paused: the same button releases it.
    tap(&mut h, "Pre-listen");
    assert_eq!(cue_commands(&fake), [Command::SetLibraryCuePaused(false)]);
    tap(&mut h, STOP);
    assert_eq!(cue_commands(&fake), [Command::StopLibraryCue]);
    assert!(h.query_by_label(STOP).is_none(), "the strip goes");
}

#[test]
fn lb45_changing_the_list_keeps_the_preview() {
    let (s, _a, _b) = with_tracks();
    let (mut h, fake) = rock_list(s);
    tap(&mut h, &headphones("Alpha"));
    tap(&mut h, WHOLE);
    tap(&mut h, BACK);
    tap(&mut h, "Jazz");
    assert_eq!(
        fake.state.load().library_cue.transport,
        fp_model::Transport::Playing
    );
}

#[test]
fn leaving_the_library_tab_stops_the_preview() {
    let (mut s, a, _b) = with_tracks();
    previewing(&mut s, a);
    let (mut h, fake) = rock_list(s);
    tap(&mut h, BACK);
    assert!(
        cue_commands(&fake).is_empty(),
        "the selector is the same tab"
    );
    tap(&mut h, "Live");
    assert_eq!(cue_commands(&fake), [Command::StopLibraryCue]);
    h.run_steps(3);
    assert!(cue_commands(&fake).is_empty(), "once");
}

#[test]
fn the_system_back_out_of_the_tab_stops_the_preview() {
    let (mut s, a, _b) = with_tracks();
    previewing(&mut s, a);
    let (mut h, fake) = rock_list(s);
    assert!(back(&mut h));
    assert!(back(&mut h));
    assert!(on_live(&h));
    assert_eq!(cue_commands(&fake), [Command::StopLibraryCue]);
}

#[test]
fn prelisten_of_a_missing_track_starts_nothing() {
    let (mut s, a, _b) = with_tracks();
    s.library.get_mut(a).unwrap().file_state = FileState::Missing;
    let (mut h, fake) = rock_list(s);
    let label = headphones("Alpha");
    assert!(h.get_by_label(&label).accesskit_node().is_disabled());
    tap(&mut h, &headphones("Alpha"));
    assert!(cue_commands(&fake).is_empty());
    assert!(!is_ticked(&h, &row("Alpha", "Band")), "nor ticks the row");
}

#[test]
fn prelisten_without_a_route_is_disabled() {
    let (mut s, _a, _b) = with_tracks();
    s.config.outputs.library.cue = None;
    s.config.outputs.library.default_output = false;
    let (h, _fake) = rock_list(s);
    assert!(
        h.get_by_label(&headphones("Alpha"))
            .accesskit_node()
            .is_disabled()
    );
}

#[test]
fn prelisten_on_a_sounding_output_asks_first() {
    let (mut s, _a, _b) = with_tracks();
    let p2 = s.players[1].id;
    support::on_desk(
        &mut s,
        &[
            fp_model::Holder::PlayerMain(p2),
            fp_model::Holder::LibraryCue,
        ],
    );
    fp_model::apply(&mut s, Command::Play(p2)).unwrap();
    let (mut h, fake) = rock_list(s);
    tap(&mut h, &headphones("Alpha"));
    h.run_steps(3);
    assert!(fake.state.load().cue_on_air.request.is_some());
    assert!(h.query_by_label("Pre-listen anyway").is_some());
}

/// LB45 (Task 5 carry-over): a window that stops being a phone while the
/// Library tab previews leaves the tab: the preview stops.
#[test]
fn leaving_the_phone_layout_stops_the_preview() {
    let (mut s, a, _b) = with_tracks();
    previewing(&mut s, a);
    let (mut h, fake) = rock_list(s);
    h.set_size(vec2(1280.0, 800.0));
    h.run_steps(3);
    assert_eq!(cue_commands(&fake), [Command::StopLibraryCue]);
    h.run_steps(3);
    assert!(cue_commands(&fake).is_empty(), "once");
}

// --- The touch-and-hold menu (LB43) ---

const PLUS: &str = "Add from device music";
const MOVE: &str = "Move to category";
const REMOVE: &str = "Remove from library…";
const MENU_PRELISTEN: &str = "Pre-listen";

fn click_at(h: &mut H, at: egui::Pos2, button: egui::PointerButton) {
    h.event(egui::Event::PointerMoved(at));
    h.run_steps(1);
    for pressed in [true, false] {
        h.event(egui::Event::PointerButton {
            pos: at,
            button,
            pressed,
            modifiers: egui::Modifiers::NONE,
        });
    }
    h.run_steps(3);
}

/// A long press on the row labelled `label` (egui opens the menu on a
/// long touch as on a secondary click).
fn hold(h: &mut H, label: &str) {
    let at = h.get_by_label(label).rect().center();
    click_at(h, at, egui::PointerButton::Secondary);
}

/// Picks the menu entries labelled `path` in turn.
fn choose(h: &mut H, path: &[&str]) {
    for label in path {
        let at = h.get_by_label(label).rect().center();
        click_at(h, at, egui::PointerButton::Primary);
    }
}

/// Rock with Alpha and Beta, and the category Gold.
fn with_gold() -> (AppState, TrackId, TrackId, CategoryId) {
    let (mut s, a, b) = with_tracks();
    let gold = create(&mut s, "Gold");
    (s, a, b, gold)
}

#[test]
fn lb43_the_phone_menu_has_only_its_items() {
    let (s, _a, _b) = with_tracks();
    let (mut h, _fake) = rock_list(s);
    hold(&mut h, &row("Alpha", "Band"));
    for item in ["Add to", MENU_PRELISTEN, MOVE, "Re-analyse", REMOVE] {
        assert!(h.query_by_label(item).is_some(), "{item}");
    }
    for item in [
        "Add to cart",
        "Kind",
        "Edit tags…",
        "Loudness…",
        "Reset markers to automatic",
        "Show file location",
    ] {
        assert!(h.query_by_label(item).is_none(), "{item}");
    }
}

#[test]
fn lb43_the_menus_pre_listen_plays_the_row() {
    let (s, a, _b) = with_tracks();
    let (mut h, fake) = rock_list(s);
    hold(&mut h, &row("Alpha", "Band"));
    choose(&mut h, &[MENU_PRELISTEN]);
    assert_eq!(
        cue_commands(&fake),
        [Command::SelectLibraryCue(Some(a)), Command::PlayLibraryCue]
    );
}

#[test]
fn lb43_a_long_press_on_an_unticked_row_joins_the_ticks() {
    let (s, a, b, gold) = with_gold();
    let (mut h, fake) = rock_list(s);
    tap(&mut h, &row("Alpha", "Band"));
    hold(&mut h, &row("Beta", "Band"));
    assert!(h.query_by_label("2 tracks selected").is_some(), "heading");
    assert!(is_ticked(&h, &row("Beta", "Band")), "ticked first");
    fake.take_sent();
    choose(&mut h, &[MOVE, "Gold"]);
    let moved: Vec<Command> = fake
        .take_sent()
        .into_iter()
        .filter(|c| matches!(c, Command::MoveTracksToCategory { .. }))
        .collect();
    assert_eq!(
        moved,
        [Command::MoveTracksToCategory {
            tracks: vec![a, b],
            category: Some(gold),
        }]
    );
}

#[test]
fn lb43_a_long_press_with_nothing_ticked_acts_on_that_row() {
    let (s, _a, b, gold) = with_gold();
    let (mut h, fake) = rock_list(s);
    hold(&mut h, &row("Beta", "Band"));
    assert!(!is_ticked(&h, &row("Beta", "Band")), "not ticked");
    fake.take_sent();
    choose(&mut h, &[MOVE, "Gold"]);
    assert!(fake.take_sent().contains(&Command::MoveTracksToCategory {
        tracks: vec![b],
        category: Some(gold),
    }));
}

#[test]
fn lb43_pre_listen_is_not_offered_while_rows_are_ticked() {
    let (s, _a, _b) = with_tracks();
    let (mut h, _fake) = rock_list(s);
    tap(&mut h, &row("Alpha", "Band"));
    hold(&mut h, &row("Alpha", "Band"));
    assert!(h.query_by_label(MOVE).is_some(), "the menu is open");
    assert!(h.query_by_label(MENU_PRELISTEN).is_none());
}

#[test]
fn lb43_remove_asks_first() {
    let (s, a, _b) = with_tracks();
    let (mut h, fake) = rock_list(s);
    hold(&mut h, &row("Alpha", "Band"));
    fake.take_sent();
    choose(&mut h, &[REMOVE]);
    assert!(
        h.query_by_label("Remove 1 track from the library?")
            .is_some()
    );
    assert!(
        !fake
            .take_sent()
            .iter()
            .any(|c| matches!(c, Command::RemoveTracks(_))),
        "nothing until confirmed"
    );
    h.get_by_label("Remove").click();
    h.run_steps(3);
    assert!(fake.take_sent().contains(&Command::RemoveTracks(vec![a])));
}

// --- Device music from ＋ (LB44) ---

fn two_files() -> Vec<MediaFile> {
    ["a", "b"]
        .into_iter()
        .map(|n| MediaFile {
            path: PathBuf::from(format!("/sd/Songs/{n}.mp3")),
            title: Some(n.to_uppercase()),
            artist: Some("Other".into()),
            album: None,
            duration_secs: Some(60.0),
        })
        .collect()
}

fn added(fake: &Fake) -> Vec<(Vec<PathBuf>, Option<CategoryId>)> {
    fake.take_sent()
        .into_iter()
        .filter_map(|c| match c {
            Command::AddToLibrary { paths, category } => Some((paths, category)),
            _ => None,
        })
        .collect()
}

/// A phone on `category`'s list with two device files.
fn list_of(s: AppState, category: &str) -> (H, MediaBrowser, Arc<Fake>) {
    let (mut h, browser, fake) = phone(s);
    browser.set_files(two_files());
    tap(&mut h, "Library");
    tap(&mut h, category);
    fake.take_sent();
    (h, browser, fake)
}

const FILE_A: &str = "A · Other · 01:00";
const FILE_B: &str = "B · Other · 01:00";

#[test]
fn lb44_plus_opens_device_music_and_adds_to_the_category() {
    let (s, _a, _b, gold) = with_gold();
    let (mut h, _browser, fake) = list_of(s, "Gold");
    let plus = h.get_by_label(PLUS).rect();
    assert!(
        plus.height() >= 48.0 && plus.right() <= PORTRAIT.x,
        "{plus:?}"
    );
    tap(&mut h, PLUS);
    assert!(h.query_by_label("Folders").is_some(), "Device music");
    tap(&mut h, FILE_B);
    tap(&mut h, FILE_A);
    assert!(h.query_by_label("Add to Gold").is_some());
    assert!(h.query_by_label(END).is_none(), "no player target");
    tap(&mut h, "Add to Gold");
    assert_eq!(
        added(&fake),
        [(
            vec![
                PathBuf::from("/sd/Songs/b.mp3"),
                PathBuf::from("/sd/Songs/a.mp3")
            ],
            Some(gold)
        )]
    );
    assert!(h.query_by_label(BACK).is_some(), "back on Gold's list");
    assert!(h.query_by_label("Folders").is_none());
    tap(&mut h, PLUS);
    assert!(!is_ticked(&h, FILE_A), "the ticks are cleared");
}

#[test]
fn lb44_plus_on_all_tracks_adds_uncategorised() {
    let (s, _a, _b) = with_tracks();
    let (mut h, _browser, fake) = list_of(s, "All tracks");
    tap(&mut h, PLUS);
    tap(&mut h, FILE_A);
    tap(&mut h, "Add to Uncategorised");
    assert_eq!(
        added(&fake),
        [(vec![PathBuf::from("/sd/Songs/a.mp3")], None)]
    );
}

#[test]
fn lb44_back_from_device_music_returns_to_the_list() {
    let (s, _a, _b, _gold) = with_gold();
    let (mut h, _browser, _fake) = list_of(s, "Gold");
    tap(&mut h, PLUS);
    assert!(back(&mut h));
    assert!(h.query_by_label(BACK).is_some(), "Gold's list");
}

#[test]
fn lb44_device_music_from_the_selector_has_no_add_to_category() {
    let (s, _rock) = with_categories();
    let (mut h, _browser, _fake) = phone(s);
    tap(&mut h, "Library");
    tap(&mut h, DEVICE);
    tap(&mut h, ALPHA);
    assert!(h.query_by_label(END).is_some(), "the player bar");
    assert!(h.query_by_label_contains("Add to ").is_none());
}

/// Task 2 carry-over: a system Back while a pick is under way at Device
/// music (its request just answered, the frame not yet run) leaves the
/// Library tab's ＋ as it was.
#[test]
fn back_during_a_pick_keeps_the_add_to_category() {
    let (s, _a, _b, _gold) = with_gold();
    let (mut h, browser, _fake) = list_of(s, "Gold");
    tap(&mut h, PLUS);
    let mode = BrowseMode::one_file(&["mp3"]).unwrap();
    let asking = browser.clone();
    let thread = std::thread::spawn(move || asking.browse(mode, None));
    for _ in 0..200 {
        h.run_steps(1);
        if h.query_by_label("Cancel").is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(h.query_by_label("Cancel").is_some(), "the pick is shown");
    browser.cancel_pending();
    let ctx = h.ctx.clone();
    h.state_mut().back(&ctx);
    h.run_steps(3);
    thread.join().unwrap();
    tap(&mut h, FILE_A);
    assert!(h.query_by_label("Add to Gold").is_some());
}

/// LB43 with a real touch: holding a finger on a row opens its menu and
/// neither ticks nor unticks it.
#[test]
fn lb43_a_held_finger_opens_the_menu_without_a_tick() {
    let (s, _a, _b) = with_tracks();
    let (mut h, _fake) = rock_list(s);
    let at = h.get_by_label(&row("Alpha", "Band")).rect().center();
    let touch = |phase| egui::Event::Touch {
        device_id: egui::TouchDeviceId(0),
        id: egui::TouchId(0),
        phase,
        pos: at,
        force: None,
    };
    let press = |pressed| egui::Event::PointerButton {
        pos: at,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    };
    h.event(touch(egui::TouchPhase::Start));
    h.event(egui::Event::PointerMoved(at));
    h.event(press(true));
    for _ in 0..60 {
        h.step();
    }
    h.event(touch(egui::TouchPhase::End));
    h.event(press(false));
    h.run_steps(3);
    assert!(h.query_by_label(MOVE).is_some(), "the menu is open");
    assert!(!is_ticked(&h, &row("Alpha", "Band")));
}

const IDLE: &str = "Tap a track’s headphones to pre-listen to it";

/// LB49: the strip's room is kept, so the rows stay where they are when a
/// preview starts.
#[test]
fn lb49_the_rows_do_not_move_when_a_preview_starts() {
    let (s, a, _b) = with_tracks();
    let (mut h, fake) = rock_list(s);
    let alpha = row("Alpha", "Band");
    let before = h.get_by_label(&alpha).rect().top();
    assert!(h.query_by_label(IDLE).is_some(), "the idle hint");
    // The list's own state (its open category), now pre-listening.
    let mut s = (**fake.state.load()).clone();
    previewing(&mut s, a);
    fake.state.store(Arc::new(s));
    h.run_steps(3);
    assert!(h.query_by_label(STOP).is_some());
    assert!(h.query_by_label(IDLE).is_none());
    assert_eq!(h.get_by_label(&alpha).rect().top(), before);
}

const BACK_TO_LIST: &str = "Back to the list";

/// LB50: Device music opened from ＋ has a back button to the list.
#[test]
fn lb50_device_music_from_plus_has_a_back_button() {
    let (s, _a, _b, _gold) = with_gold();
    let (mut h, _browser, _fake) = list_of(s, "Gold");
    tap(&mut h, PLUS);
    assert!(h.query_by_label("Folders").is_some(), "Device music");
    let button = h.get_by_label(BACK_TO_LIST).rect();
    assert!(button.height() >= 48.0, "{button:?}");
    tap(&mut h, BACK_TO_LIST);
    assert!(h.query_by_label("Folders").is_none());
    assert!(h.query_by_label(PLUS).is_some(), "Gold's list");
}

/// LB50: opened from the selector, it goes back to the selector.
#[test]
fn lb50_device_music_from_the_selector_has_a_back_button() {
    let (s, _rock) = with_categories();
    let (mut h, _browser, _fake) = phone(s);
    tap(&mut h, "Library");
    tap(&mut h, DEVICE);
    assert!(h.query_by_label("Folders").is_some());
    assert!(h.query_by_label(BACK_TO_LIST).is_none());
    tap(&mut h, BACK);
    assert!(on_selector(&h));
}

/// LB50: a pick keeps its Cancel and shows no header.
#[test]
fn lb50_a_pick_shows_no_header() {
    let (s, _rock) = with_categories();
    let (mut h, browser, _fake) = phone(s);
    let mode = BrowseMode::one_file(&["mp3"]).unwrap();
    let asking = browser.clone();
    let thread = std::thread::spawn(move || asking.browse(mode, None));
    for _ in 0..200 {
        h.run_steps(1);
        if h.query_by_label("Cancel").is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(h.query_by_label("Cancel").is_some(), "the pick is shown");
    assert!(h.query_by_label(BACK).is_none());
    assert!(h.query_by_label(BACK_TO_LIST).is_none());
    browser.cancel_pending();
    h.run_steps(3);
    thread.join().unwrap();
}

/// LB51: a list opened from the selector starts with an empty search; the
/// *Whole library* scope stays.
#[test]
fn lb51_opening_another_category_clears_the_search() {
    let (mut s, rock) = with_categories();
    let jazz = s.categories.iter().find(|c| c.name == "Jazz").unwrap().id;
    track(&mut s, "one", "Canción uno", "Band", Some(rock));
    track(&mut s, "two", "Blue", "Band", Some(jazz));
    let (mut h, _fake) = rock_list(s);
    search(&mut h, "cancion");
    tap(&mut h, BACK);
    tap(&mut h, "Jazz");
    assert!(h.query_by_label(&row("Blue", "Band")).is_some());
    assert_eq!(h.get_by_label(SEARCH).value().as_deref(), Some(""));
}

fn media(path: &str) -> MediaFile {
    MediaFile {
        path: PathBuf::from(path),
        title: None,
        artist: None,
        album: None,
        duration_secs: None,
    }
}

/// LB53: whole path components, in the list's order.
#[test]
fn lb53_folder_files_take_only_what_is_under_the_folder() {
    use fp_app::ui::phone_library::folder_files;
    let files = [
        media("/sd/Rock/b.mp3"),
        media("/sd/Rocks/c.mp3"),
        media("/sd/Rock/live/a.mp3"),
        media("/sd/Other/d.mp3"),
    ];
    assert_eq!(
        folder_files(&files, std::path::Path::new("/sd/Rock")),
        [
            PathBuf::from("/sd/Rock/b.mp3"),
            PathBuf::from("/sd/Rock/live/a.mp3")
        ]
    );
    assert!(folder_files(&[], std::path::Path::new("/sd/Rock")).is_empty());
}

const REREAD: &str = "Re-read the device folder";

/// Rock takes its files from `/sd/Rock`; the device lists two files there
/// and one in `/sd/Rocks`. A phone on Rock's list.
fn rock_with_folder(files: Vec<MediaFile>) -> (H, Arc<Fake>, CategoryId) {
    let (mut s, rock) = with_categories();
    fp_model::apply(
        &mut s,
        Command::SetWatchedFolder {
            category: rock,
            folder: Some(PathBuf::from("/sd/Rock")),
        },
    )
    .unwrap();
    let (mut h, browser, fake) = phone(s);
    browser.set_files(files);
    tap(&mut h, "Library");
    tap(&mut h, "Rock");
    fake.take_sent();
    (h, fake, rock)
}

#[test]
fn lb53_reread_adds_the_files_under_the_folder() {
    let files = vec![
        media("/sd/Rock/b.mp3"),
        media("/sd/Rocks/c.mp3"),
        media("/sd/Rock/a.mp3"),
    ];
    let (mut h, fake, rock) = rock_with_folder(files);
    let button = h.get_by_label(REREAD).rect();
    assert!(button.height() >= 48.0, "{button:?}");
    tap(&mut h, REREAD);
    assert_eq!(
        added(&fake),
        [(
            vec![
                PathBuf::from("/sd/Rock/b.mp3"),
                PathBuf::from("/sd/Rock/a.mp3")
            ],
            Some(rock)
        )]
    );
    assert!(
        h.query_by_label_contains("Files read from /sd/Rock: 2")
            .is_some()
    );
}

#[test]
fn lb53_reread_with_no_files_says_so() {
    let (mut h, fake, _rock) = rock_with_folder(vec![media("/sd/Rocks/c.mp3")]);
    tap(&mut h, REREAD);
    assert!(added(&fake).is_empty());
    assert!(
        h.query_by_label_contains("No music files in /sd/Rock on this device")
            .is_some()
    );
}

/// Rock takes its files from `/sd/Rock` and already holds `held` (path,
/// missing); the device lists `files`. A phone on Rock's list, with the
/// services' requests.
fn rock_holding(
    held: &[(&str, bool)],
    files: Vec<MediaFile>,
) -> (
    H,
    Arc<Fake>,
    crossbeam_channel::Receiver<fp_app::services::ServiceRequest>,
    Vec<TrackId>,
) {
    let (mut s, rock) = with_categories();
    fp_model::apply(
        &mut s,
        Command::SetWatchedFolder {
            category: rock,
            folder: Some(PathBuf::from("/sd/Rock")),
        },
    )
    .unwrap();
    let mut ids = Vec::new();
    for (path, missing) in held {
        let path = PathBuf::from(path);
        fp_model::apply(
            &mut s,
            Command::AddToLibrary {
                paths: vec![path.clone()],
                category: Some(rock),
            },
        )
        .unwrap();
        let id = s.library.track_at(&path).unwrap();
        if *missing {
            s.library.get_mut(id).unwrap().file_state = FileState::Missing;
        }
        ids.push(id);
    }
    let browser = MediaBrowser::default();
    browser.set_files(files);
    let (tx, rx) = crossbeam_channel::bounded(8);
    let (mut h, fake) = harness_sized(s, PORTRAIT, move |ui| {
        ui.with_touch()
            .with_media_browser(browser)
            .with_services(tx)
    });
    h.run_steps(3);
    tap(&mut h, "Library");
    tap(&mut h, "Rock");
    fake.take_sent();
    (h, fake, rx, ids)
}

/// LB21 on a re-read: a file the device no longer lists under the folder
/// is marked missing, one it lists again is analysed again (which brings
/// it back), and nothing outside the folder is touched.
#[test]
fn lb53_reread_marks_the_gone_files_missing() {
    let (mut h, fake, rx, ids) = rock_holding(
        &[
            ("/sd/Rock/a.mp3", false),
            ("/sd/Rock/gone.mp3", false),
            ("/sd/Rock/back.mp3", true),
            ("/sd/Rocks/x.mp3", false),
        ],
        vec![media("/sd/Rock/a.mp3"), media("/sd/Rock/back.mp3")],
    );
    let (gone, back) = (ids[1], ids[2]);
    tap(&mut h, REREAD);
    let marked: Vec<(TrackId, FileState)> = fake
        .take_sent()
        .into_iter()
        .filter_map(|c| match c {
            Command::SetFileState { track, state } => Some((track, state)),
            _ => None,
        })
        .collect();
    assert_eq!(marked, [(gone, FileState::Missing)]);
    assert_eq!(
        rx.try_iter().collect::<Vec<_>>(),
        [fp_app::services::ServiceRequest::ReanalyseTrack(back)]
    );
}

/// With no permission the device lists nothing: nothing is marked.
#[test]
fn lb53_reread_with_no_permission_marks_nothing() {
    let (mut h, fake, rx, _ids) = rock_holding(&[("/sd/Rock/a.mp3", false)], Vec::new());
    tap(&mut h, REREAD);
    assert!(
        !fake
            .take_sent()
            .iter()
            .any(|c| matches!(c, Command::SetFileState { .. }))
    );
    assert!(rx.try_recv().is_err());
}

#[test]
fn lb53_a_category_without_a_folder_has_no_reread() {
    let (s, _rock) = with_categories();
    let (mut h, _browser, _fake) = phone(s);
    tap(&mut h, "Library");
    tap(&mut h, "Rock");
    assert!(h.query_by_label(REREAD).is_none());
}

/// A finger held at `at` for about a second, as the phone sends it.
fn hold_finger(h: &mut H, at: egui::Pos2) {
    let touch = |phase| egui::Event::Touch {
        device_id: egui::TouchDeviceId(0),
        id: egui::TouchId(0),
        phase,
        pos: at,
        force: None,
    };
    let press = |pressed| egui::Event::PointerButton {
        pos: at,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    };
    h.event(touch(egui::TouchPhase::Start));
    h.event(egui::Event::PointerMoved(at));
    h.event(press(true));
    for _ in 0..60 {
        h.step();
    }
    h.event(touch(egui::TouchPhase::End));
    h.event(press(false));
    h.run_steps(3);
}

/// PL38: a held finger's tap belongs to what is on top. Held on Settings,
/// it must not open the selector row drawn beneath.
#[test]
fn a_held_finger_on_settings_does_not_open_the_row_beneath() {
    let (s, _rock) = with_categories();
    let (mut h, _browser, fake) = phone(s);
    tap(&mut h, "Library");
    let at = h.get_by_label("Uncategorised").rect().center();
    tap(&mut h, "More");
    tap(&mut h, "Settings");
    fake.take_sent();
    hold_finger(&mut h, at);
    let opened = fake
        .take_sent()
        .into_iter()
        .any(|c| matches!(c, Command::SetLibraryView(_)));
    assert!(!opened, "a list opened under Settings");
}

/// The header's buttons sit at its right edge, whatever the name's length.
#[test]
fn the_header_buttons_sit_at_the_right() {
    let (s, _a, _b, _gold) = with_gold();
    let (h, _browser, _fake) = list_of(s, "Gold");
    let plus = h.get_by_label(PLUS).rect();
    assert!(plus.right() >= PORTRAIT.x - 24.0, "{plus:?}");
}
