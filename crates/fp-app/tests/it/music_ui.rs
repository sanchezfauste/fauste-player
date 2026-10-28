//! The phone's Music screen (phone layout spec §9, PL17–PL19b).

use crate::support;

use std::path::PathBuf;

use egui::{Vec2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use fp_app::ui::app::AppUi;
use fp_app::ui::media_browser::{MediaBrowser, MediaFile};
use std::sync::Arc;
use support::{Fake, harness_sized, state};

const PORTRAIT: Vec2 = vec2(412.0, 892.0);

fn file(path: &str, title: &str, artist: &str) -> MediaFile {
    MediaFile {
        path: PathBuf::from(path),
        title: (!title.is_empty()).then(|| title.to_owned()),
        artist: (!artist.is_empty()).then(|| artist.to_owned()),
        album: None,
        duration_secs: Some(60.0),
    }
}

fn library() -> Vec<MediaFile> {
    vec![
        file("/sd/Songs/a.mp3", "Alpha", "Other"),
        file("/sd/Songs/Rock/b.mp3", "Beta", "Band"),
        file("/sd/Songs/Rock/c.mp3", "Gamma", "Band"),
    ]
}

/// A phone with the library, on the Music screen.
fn music() -> (Harness<'static, AppUi>, MediaBrowser) {
    let (h, browser, _fake) = music_in(state(2, 3));
    (h, browser)
}

fn music_in(s: fp_model::AppState) -> (Harness<'static, AppUi>, MediaBrowser, Arc<Fake>) {
    let browser = MediaBrowser::default();
    browser.set_files(library());
    let given = browser.clone();
    let (mut h, fake) = harness_sized(s, PORTRAIT, move |ui| {
        ui.with_touch().with_media_browser(given)
    });
    h.run_steps(3);
    h.get_by_label("Music").click();
    h.run_steps(3);
    (h, browser, fake)
}

fn back(h: &mut Harness<'static, AppUi>) -> bool {
    let ctx = h.ctx.clone();
    let went = h.state_mut().back(&ctx);
    h.run_steps(3);
    went
}

fn ticked(h: &Harness<'static, AppUi>, label: &str) -> bool {
    h.get_by_label(label).accesskit_node().toggled() == Some(egui::accesskit::Toggled::True)
}

/// PL1, PL17: a fourth tab with a search and the three ways to browse.
#[test]
fn the_music_tab_has_a_search_and_three_ways_to_browse() {
    let (h, _browser) = music();
    assert!(h.query_by_label("Search music").is_some());
    for name in ["Folders", "Artists", "Albums"] {
        assert!(h.query_by_label(name).is_some(), "{name}");
    }
}

/// PL18: a row shows title, artist and duration.
#[test]
fn a_row_shows_title_artist_and_duration() {
    let (h, _browser) = music();
    assert!(h.query_by_label("Alpha · Other · 01:00").is_some());
}

/// PL18: a tap on a row ticks it, another unticks it.
#[test]
fn a_tap_on_a_row_ticks_it() {
    let (mut h, _browser) = music();
    let row = "Alpha · Other · 01:00";
    assert!(!ticked(&h, row));
    h.get_by_label(row).click();
    h.run_steps(3);
    assert!(ticked(&h, row));
    h.get_by_label(row).click();
    h.run_steps(3);
    assert!(!ticked(&h, row));
}

/// PL17, PL2: a folder opens with a breadcrumb; Back goes up, then to Live.
#[test]
fn a_folder_opens_and_back_goes_up_then_to_live() {
    let (mut h, _browser) = music();
    h.get_by_label("Rock").click();
    h.run_steps(3);
    assert!(h.query_by_label("Beta · Band · 01:00").is_some());
    assert!(h.query_by_label("Alpha · Other · 01:00").is_none());
    assert!(h.query_by_label("Songs").is_some(), "the breadcrumb's top");
    assert!(back(&mut h));
    assert!(h.query_by_label("Alpha · Other · 01:00").is_some());
    assert!(back(&mut h));
    assert!(h.query_by_label("Folders").is_none(), "on Live");
}

/// PL17: Artists lists the artists, and one opens its tracks.
#[test]
fn artists_open_their_tracks() {
    let (mut h, _browser) = music();
    h.get_by_label("Artists").click();
    h.run_steps(3);
    h.get_by_label("Band").click();
    h.run_steps(3);
    assert!(h.query_by_label("Beta · Band · 01:00").is_some());
    assert!(h.query_by_label("Gamma · Band · 01:00").is_some());
    assert!(h.query_by_label("Alpha · Other · 01:00").is_none());
}

/// PL17: the search finds tracks in every folder.
#[test]
fn the_search_finds_tracks_everywhere() {
    let (mut h, _browser) = music();
    h.get_by_label("Search music").click();
    h.run_steps(2);
    h.get_by_label("Search music").type_text("gam");
    h.run_steps(3);
    assert!(h.query_by_label("Gamma · Band · 01:00").is_some());
    assert!(h.query_by_label("Alpha · Other · 01:00").is_none());
}

// --- The selection bar (PL19) ---

const ALPHA: &str = "Alpha · Other · 01:00";
const BETA: &str = "Beta · Band · 01:00";
const END: &str = "Add at the end";
const AFTER: &str = "Add after the next";

fn tap(h: &mut Harness<'static, AppUi>, label: &str) {
    h.get_by_label(label).click();
    h.run_steps(3);
}

fn inserted(fake: &Fake) -> Vec<(fp_model::PlaylistId, usize, Vec<PathBuf>)> {
    fake.take_sent()
        .into_iter()
        .filter_map(|c| match c {
            fp_model::Command::InsertPaths {
                playlist,
                index,
                paths,
            } => Some((playlist, index, paths)),
            _ => None,
        })
        .collect()
}

/// Beta (in Rock) then Alpha: the tap order.
fn tick_beta_then_alpha(h: &mut Harness<'static, AppUi>) {
    tap(h, "Rock");
    tap(h, BETA);
    assert!(back_music(h));
    tap(h, ALPHA);
}

fn back_music(h: &mut Harness<'static, AppUi>) -> bool {
    let ctx = h.ctx.clone();
    let went = h.state_mut().back(&ctx);
    h.run_steps(3);
    went
}

#[test]
fn the_bar_shows_while_something_is_selected() {
    let (mut h, _browser) = music();
    assert!(h.query_by_label(END).is_none());
    tap(&mut h, ALPHA);
    assert!(h.query_by_label("Selected: 1").is_some());
    assert!(h.query_by_label("P1 · Main").is_some(), "the target");
    assert!(h.query_by_label(END).is_some());
}

#[test]
fn add_at_the_end_keeps_the_tap_order_and_clears_the_selection() {
    let s = state(2, 3);
    let playlist = s.playlists.first_id().unwrap();
    let (mut h, _browser, fake) = music_in(s);
    tick_beta_then_alpha(&mut h);
    fake.take_sent();
    tap(&mut h, END);
    assert_eq!(
        inserted(&fake),
        [(
            playlist,
            3,
            vec![
                PathBuf::from("/sd/Songs/Rock/b.mp3"),
                PathBuf::from("/sd/Songs/a.mp3")
            ]
        )]
    );
    assert!(h.query_by_label(END).is_none(), "the selection is cleared");
}

#[test]
fn add_after_the_next_goes_right_after_it() {
    let mut s = state(2, 3);
    let playlist = s.playlists.first_id().unwrap();
    let first = s.playlists.get(playlist).unwrap().entries[0].id;
    let player = s.player_ids()[0];
    fp_model::apply(&mut s, fp_model::Command::SetNext(player, first)).unwrap();
    let (mut h, _browser, fake) = music_in(s);
    tap(&mut h, ALPHA);
    fake.take_sent();
    tap(&mut h, AFTER);
    assert_eq!(
        inserted(&fake),
        [(playlist, 1, vec![PathBuf::from("/sd/Songs/a.mp3")])]
    );
}

#[test]
fn without_a_next_in_the_target_only_add_at_the_end_shows() {
    let mut s = state(2, 3);
    let player = s.player_ids()[0];
    s.players.iter_mut().find(|p| p.id == player).unwrap().next = None;
    let (mut h, _browser, _fake) = music_in(s);
    tap(&mut h, ALPHA);
    assert!(h.query_by_label(END).is_some());
    assert!(h.query_by_label(AFTER).is_none());
}

#[test]
fn the_target_can_change_player() {
    let s = state(2, 3);
    let (mut h, _browser, fake) = music_in(s);
    tap(&mut h, ALPHA);
    tap(&mut h, "P1 · Main");
    tap(&mut h, "P2 · Main");
    assert!(h.query_by_label("P2 · Main").is_some());
    fake.take_sent();
    tap(&mut h, END);
    assert_eq!(inserted(&fake).len(), 1);
}
