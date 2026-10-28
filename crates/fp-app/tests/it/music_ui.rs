//! The phone's Music screen (phone layout spec §9, PL17–PL19b).

use crate::support;

use std::path::PathBuf;

use egui::{Vec2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use fp_app::ui::app::AppUi;
use fp_app::ui::media_browser::{MediaBrowser, MediaFile};
use support::{harness_sized, state};

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
    let browser = MediaBrowser::default();
    browser.set_files(library());
    let given = browser.clone();
    let (mut h, _fake) = harness_sized(state(2, 3), PORTRAIT, move |ui| {
        ui.with_touch().with_media_browser(given)
    });
    h.run_steps(3);
    h.get_by_label("Music").click();
    h.run_steps(3);
    (h, browser)
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
