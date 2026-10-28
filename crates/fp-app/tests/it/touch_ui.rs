#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The touch layout on a phone (phone layout spec) and a tablet (mobile
//! spec §6).

use crate::support;

use egui::{Vec2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use std::sync::Arc;
use support::{Fake, harness, harness_sized, state};

const PORTRAIT: Vec2 = vec2(412.0, 892.0);
const LANDSCAPE: Vec2 = vec2(892.0, 412.0);
const TABLET: Vec2 = vec2(800.0, 1280.0);

fn touch(players: usize, size: Vec2) -> (Harness<'static, AppUi>, Arc<Fake>) {
    let (mut h, fake) = harness_sized(state(players, 3), size, |ui| ui.with_touch());
    h.run_steps(3);
    (h, fake)
}

/// A finger drag. The touch event makes egui see a touch screen, as on a
/// phone: only then does a drag scroll a scroll area.
fn drag(h: &mut Harness<'static, AppUi>, from: egui::Pos2, to: egui::Pos2) {
    h.event(egui::Event::Touch {
        device_id: egui::TouchDeviceId(0),
        id: egui::TouchId(0),
        phase: egui::TouchPhase::Cancel,
        pos: from,
        force: None,
    });
    h.event(egui::Event::PointerMoved(from));
    h.event(egui::Event::PointerButton {
        pos: from,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::NONE,
    });
    h.step();
    for k in 1..=5 {
        h.event(egui::Event::PointerMoved(
            from + (to - from) * (k as f32 / 5.0),
        ));
        h.step();
    }
    h.event(egui::Event::PointerButton {
        pos: to,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
    h.run_steps(3);
}

const ADD: &str = "Add tracks to this playlist";

/// Opens the top bar's More menu and picks `entry` (Settings, About).
fn more(h: &mut Harness<'static, AppUi>, entry: &str) {
    h.get_by_label("More").click();
    h.run_steps(2);
    h.get_by_label(entry).click();
    h.run_steps(3);
}

fn tab(h: &mut Harness<'static, AppUi>, name: &str) {
    h.get_by_label(name).click();
    h.run_steps(3);
}

fn volume_sent(fake: &Fake) -> bool {
    fake.take_sent()
        .iter()
        .any(|c| matches!(c, fp_model::Command::SetVolume(..)))
}

// --- Navigation (PL1, PL3) ---

#[test]
fn the_phone_top_bar_has_the_tabs_and_more() {
    let (h, _fake) = touch(2, PORTRAIT);
    for label in ["Live", "Carts", "Playlists", "More"] {
        assert!(h.query_by_label(label).is_some(), "{label}");
    }
    for gone in ["Hide P1", "Open P1", "All", "Players"] {
        assert!(h.query_by_label(gone).is_none(), "{gone}");
    }
}

#[test]
fn more_opens_settings_and_about() {
    let (mut h, _fake) = touch(2, PORTRAIT);
    more(&mut h, "Settings");
    assert!(h.query_by_label("Buffer size").is_some());
    let (mut h, _fake) = touch(2, PORTRAIT);
    more(&mut h, "About");
    assert!(h.query_by_label_contains("Fauste Player by").is_some());
}

#[test]
fn a_tablet_keeps_the_panel_tiles() {
    let (h, _fake) = touch(2, TABLET);
    assert!(h.query_by_label("Hide P1").is_some());
    assert!(h.query_by_label("Live").is_none());
}

#[test]
fn without_touch_nothing_changes() {
    let (mut h, _fake) = harness_sized(state(2, 3), PORTRAIT, |ui| ui);
    h.run_steps(3);
    assert!(h.query_by_label("Hide P1").is_some());
    assert!(h.query_by_label("Live").is_none());
}

#[test]
fn landscape_is_a_phone_too() {
    let (h, _fake) = touch(2, LANDSCAPE);
    assert!(h.query_by_label("Live").is_some());
}

#[test]
fn the_top_bar_fits_a_narrow_phone() {
    let (h, _fake) = touch(2, vec2(360.0, 780.0));
    let tiles: Vec<egui::Rect> = ["Live", "Carts", "Playlists", "More"]
        .iter()
        .map(|l| h.get_by_label(l).rect())
        .collect();
    for (i, a) in tiles.iter().enumerate() {
        assert!(a.right() <= 360.0, "{a:?} leaves the screen");
        for b in tiles.iter().skip(i + 1) {
            assert!(!a.intersects(*b), "{a:?} overlaps {b:?}");
        }
    }
}

#[test]
fn playlists_shows_the_chosen_player_with_its_playlist() {
    let (mut h, _fake) = touch(3, PORTRAIT);
    tab(&mut h, "Playlists");
    assert_eq!(h.query_all_by_label(ADD).count(), 1, "the first player");
    tab(&mut h, "P3");
    assert_eq!(h.query_all_by_label(ADD).count(), 1);
    assert!(h.query_by_label("P1").is_some(), "the chips stay");
}

#[test]
fn rotation_keeps_the_screen() {
    let (mut h, _fake) = touch(3, PORTRAIT);
    tab(&mut h, "Playlists");
    h.set_size(LANDSCAPE);
    h.run_steps(3);
    assert_eq!(h.query_all_by_label(ADD).count(), 1);
}

#[test]
fn a_removed_player_leaves_the_playlists_chips() {
    let (mut h, fake) = touch(2, PORTRAIT);
    tab(&mut h, "Playlists");
    tab(&mut h, "P2");
    let mut fewer = (**fake.state.load()).clone();
    fewer.players.truncate(1);
    fake.state.store(std::sync::Arc::new(fewer));
    h.run_steps(3);
    assert!(h.query_by_label("P2").is_none());
    assert_eq!(h.query_all_by_label(ADD).count(), 1, "P1 shows");
}

#[test]
fn a_phone_has_no_desktop_panel_buttons() {
    let (mut h, _fake) = touch(2, PORTRAIT);
    for screen in ["Live", "Carts", "Playlists"] {
        tab(&mut h, screen);
        assert!(h.query_by_label("Maximize").is_none(), "{screen}");
    }
    let (h, _fake) = touch(2, TABLET);
    assert!(h.query_all_by_label("Maximize").count() > 0);
}

// --- Live (PL4–PL6) ---

#[test]
fn live_shows_every_player_card_and_the_cartwall() {
    for size in [PORTRAIT, LANDSCAPE] {
        let (h, _fake) = touch(2, size);
        assert_eq!(h.query_all_by_label("Play").count(), 2, "{size:?}");
        assert!(h.query_by_label(ADD).is_none(), "no playlist on Live");
        assert!(
            h.query_all_by_label_contains(", empty").count() > 0,
            "{size:?}: the carts show"
        );
    }
}

#[test]
fn live_puts_the_cartwall_below_in_portrait_and_right_in_landscape() {
    let (h, _fake) = touch(2, PORTRAIT);
    let play = h.query_all_by_label("Play").next().unwrap().rect();
    let cart = h.get_by_label("Cart 1, empty").rect();
    assert!(cart.top() > play.bottom(), "portrait: below");
    let (h, _fake) = touch(2, LANDSCAPE);
    let play = h.query_all_by_label("Play").next().unwrap().rect();
    let cart = h.get_by_label("Cart 1, empty").rect();
    assert!(cart.left() > play.right(), "landscape: right");
}

/// The Live screen's cartwall is short and wide in portrait: a page too
/// wide for it is laid out in fewer columns, in the same reading order, so
/// every cart is inside the screen and wide enough to read.
#[test]
fn the_live_cartwall_fits_a_portrait_phone() {
    let (h, _fake) = touch(2, PORTRAIT);
    let carts: Vec<egui::Rect> = h
        .query_all_by_label_contains(", empty")
        .map(|n| n.rect())
        .collect();
    assert!(carts.len() > 2);
    for cart in &carts {
        assert!(cart.right() <= PORTRAIT.x, "{cart:?} leaves the screen");
        assert!(cart.width() >= 80.0, "{cart:?} is narrow");
    }
    let one = h.get_by_label("Cart 1, empty").rect();
    let two = h.get_by_label("Cart 2, empty").rect();
    assert!(two.left() > one.right(), "2 right of 1");
}

/// Two cards show at once on a 6-inch phone held upright (PL5): about
/// 400 × 860 dp, less the system bars.
#[test]
fn two_cards_fit_a_portrait_phone() {
    let (h, _fake) = touch(3, vec2(400.0, 820.0));
    let carts_top = h.get_by_label("Cart 1, empty").rect().top();
    let plays: Vec<egui::Rect> = h.query_all_by_label("Play").map(|n| n.rect()).collect();
    assert!(
        plays[1].bottom() < carts_top,
        "the second card's Play shows"
    );
    let faders: Vec<egui::Rect> = h
        .query_all_by_label_contains("Volume")
        .map(|n| n.rect())
        .collect();
    assert!(faders[1].bottom() < carts_top, "and its fader");
}

#[test]
fn a_card_has_the_waveform_above_its_buttons() {
    let (h, _fake) = touch(1, PORTRAIT);
    let wave = h.get_by_label("Waveform: click to seek").rect();
    let play = h.get_by_label("Play").rect();
    assert!(wave.bottom() <= play.top(), "{wave:?} above {play:?}");
}

#[test]
fn play_on_a_card_sends_play() {
    let (mut h, fake) = touch(1, PORTRAIT);
    fake.take_sent();
    h.get_by_label("Play").click();
    h.run_steps(2);
    assert!(
        fake.take_sent()
            .iter()
            .any(|c| matches!(c, fp_model::Command::Play(_))),
        "no play command"
    );
}

/// PL14: a drag that starts on a card's fader moves it.
#[test]
fn a_drag_on_a_card_fader_moves_it() {
    for size in [PORTRAIT, LANDSCAPE] {
        let (mut h, fake) = touch(3, size);
        fake.take_sent();
        let fader = h.get_all_by_label_contains("Volume").next().unwrap().rect();
        // Down: at 0 dB the fader is at its top, where up changes nothing.
        drag(&mut h, fader.center(), fader.center() + vec2(0.0, 60.0));
        assert!(volume_sent(&fake), "{size:?}: the fader did not move");
    }
}

/// A scroll of the cards that starts on a title moves nothing.
#[test]
fn a_scroll_of_the_cards_sends_nothing() {
    let (mut h, fake) = touch(4, PORTRAIT);
    fake.take_sent();
    let fader = h.get_all_by_label_contains("Volume").next().unwrap().rect();
    let from = egui::pos2(120.0, fader.center().y);
    drag(&mut h, from, from + vec2(0.0, -200.0));
    let sent = fake.take_sent();
    assert!(sent.is_empty(), "a scroll sent {sent:?}");
    let moved = h.get_all_by_label_contains("Volume").next().unwrap().rect();
    assert!(moved.top() < fader.top(), "the cards did not scroll");
}

// --- Carts (PL7) ---

#[test]
fn the_carts_screen_shows_the_carts() {
    let (mut h, _fake) = touch(2, PORTRAIT);
    tab(&mut h, "Carts");
    assert!(h.query_all_by_label("Play").count() == 0, "no players");
    assert!(h.query_by_label_contains("Stop all").is_some());
}

/// Two cart pages; the first shown.
fn two_pages() -> (Harness<'static, AppUi>, Arc<Fake>, fp_model::CartPageId) {
    let mut s = state(2, 3);
    fp_model::apply(
        &mut s,
        fp_model::Command::CreateCartPage {
            name: "Second".into(),
        },
    )
    .unwrap();
    let second = s.cartwall.pages[1].id;
    s.cartwall.shown = Some(s.cartwall.pages[0].id);
    let (mut h, fake) = harness_sized(s, PORTRAIT, |ui| ui.with_touch());
    h.run_steps(3);
    tab(&mut h, "Carts");
    fake.take_sent();
    (h, fake, second)
}

fn pages_shown(fake: &Fake) -> Vec<fp_model::CartPageId> {
    fake.take_sent()
        .into_iter()
        .filter_map(|c| match c {
            fp_model::Command::ShowCartPage(p) => Some(p),
            _ => None,
        })
        .collect()
}

/// PL7: a sideways swipe on the Carts screen shows the next page.
#[test]
fn a_swipe_on_the_carts_screen_shows_the_next_page() {
    let (mut h, fake, second) = two_pages();
    let cart = h.get_by_label("Cart 1, empty").rect();
    let from = egui::pos2(PORTRAIT.x - 40.0, cart.bottom() + 100.0);
    drag(&mut h, from, from - vec2(250.0, 0.0));
    assert_eq!(pages_shown(&fake), vec![second]);
}

#[test]
fn a_swipe_back_on_the_first_page_does_nothing() {
    let (mut h, fake, _second) = two_pages();
    let cart = h.get_by_label("Cart 1, empty").rect();
    let from = egui::pos2(40.0, cart.bottom() + 100.0);
    drag(&mut h, from, from + vec2(250.0, 0.0));
    assert_eq!(pages_shown(&fake), vec![]);
}

/// A drag along the page tabs scrolls them and changes no page.
#[test]
fn a_drag_on_the_page_tabs_changes_no_page() {
    let mut s = state(2, 3);
    for i in 0..12 {
        fp_model::apply(
            &mut s,
            fp_model::Command::CreateCartPage {
                name: format!("A long page name {i}"),
            },
        )
        .unwrap();
    }
    s.cartwall.shown = Some(s.cartwall.pages[0].id);
    let (mut h, fake) = harness_sized(s, PORTRAIT, |ui| ui.with_touch());
    h.run_steps(3);
    tab(&mut h, "Carts");
    fake.take_sent();
    let tab = h.get_by_label("A long page name 0").rect();
    drag(&mut h, tab.center(), tab.center() - vec2(150.0, 0.0));
    assert_eq!(pages_shown(&fake), vec![]);
}

/// A tap on a cart fires it and changes no page.
#[test]
fn a_tap_on_the_carts_screen_shows_no_page() {
    let (mut h, fake, _second) = two_pages();
    let cart = h.get_by_label("Cart 1, empty").rect();
    drag(&mut h, cart.center(), cart.center() + vec2(-4.0, 0.0));
    assert_eq!(pages_shown(&fake), vec![]);
}

#[test]
fn the_phone_cartwall_never_folds() {
    let mut closed = state(2, 3);
    closed.cartwall.open = false;
    let (mut h, _fake) = harness_sized(closed, PORTRAIT, |ui| ui.with_touch());
    h.run_steps(3);
    tab(&mut h, "Carts");
    assert!(
        h.query_all_by_label_contains("empty").count() > 0,
        "the carts show"
    );
    assert!(h.query_by_label("CARTWALL").is_none(), "no folding caret");
}

/// A page wider than tall (8 × 2 by default) is laid out the other way
/// round on a portrait screen, in the same reading order: wide buttons,
/// all inside the screen.
#[test]
fn a_portrait_cartwall_turns_a_wide_page() {
    let (mut h, _fake) = touch(2, PORTRAIT);
    tab(&mut h, "Carts");
    let carts: Vec<egui::Rect> = h
        .query_all_by_label_contains(", empty")
        .map(|n| n.rect())
        .collect();
    assert!(carts.len() > 2);
    for cart in &carts {
        assert!(cart.right() <= PORTRAIT.x, "{cart:?} leaves the screen");
        assert!(cart.width() > 120.0, "{cart:?} is narrow");
    }
    let one = h.get_by_label("Cart 1, empty").rect();
    let two = h.get_by_label("Cart 2, empty").rect();
    let three = h.get_by_label("Cart 3, empty").rect();
    assert!(two.left() > one.right(), "2 right of 1");
    assert!(three.top() > one.bottom(), "3 under 1");
}

/// Landscape keeps the page as it is.
#[test]
fn a_landscape_cartwall_keeps_the_page() {
    let (mut h, _fake) = touch(2, LANDSCAPE);
    tab(&mut h, "Carts");
    let one = h.get_by_label("Cart 1, empty").rect();
    let three = h.get_by_label("Cart 3, empty").rect();
    assert!((three.top() - one.top()).abs() < 1.0, "3 on the row of 1");
}

// --- Playlists (PL8–PL11b) ---

fn waveform(h: &Harness<'static, AppUi>) -> egui::Rect {
    h.get_all_by_label("Waveform: click to seek")
        .next()
        .unwrap()
        .rect()
}

/// PL9: held upright, the track list is on top and the player's card at
/// the bottom.
#[test]
fn the_playlists_list_is_over_the_card() {
    let (mut h, _fake) = touch(2, PORTRAIT);
    tab(&mut h, "Playlists");
    let row = h.get_all_by_label("Song 1").next().unwrap().rect();
    let play = h.get_by_label("Play").rect();
    assert!(row.bottom() < waveform(&h).top(), "the list first");
    assert!(play.bottom() > PORTRAIT.y * 0.6, "the card at the bottom");
}

/// PL9: turned sideways, the list is on the left and the card on the right.
#[test]
fn in_landscape_the_list_is_left_of_the_card() {
    let (mut h, _fake) = touch(2, LANDSCAPE);
    tab(&mut h, "Playlists");
    let row = h.get_all_by_label("Song 1").next().unwrap().rect();
    let play = h.get_by_label("Play").rect();
    assert!(row.right() <= waveform(&h).left(), "the list on the left");
    assert!(play.left() > LANDSCAPE.x / 2.0, "the card on the right");
}

/// PL9: the Playlists card's waveform is larger than a Live card's, for
/// preparing a track.
#[test]
fn the_playlists_card_has_a_larger_waveform() {
    let (mut h, _fake) = touch(2, PORTRAIT);
    let live = waveform(&h).height();
    tab(&mut h, "Playlists");
    assert!(waveform(&h).height() > live + 20.0);
}

/// PL8: the chosen player's playlist tabs, with + for a new playlist.
#[test]
fn the_playlists_screen_has_tabs_and_a_new_playlist_button() {
    let (mut h, _fake) = touch(2, PORTRAIT);
    tab(&mut h, "Playlists");
    assert!(h.query_by_label("New playlist").is_some());
    let chip = h.get_all_by_label("P1").next().unwrap().rect();
    let new = h.get_by_label("New playlist").rect();
    assert!(chip.bottom() <= new.top() + 1.0, "the tabs under the chips");
}

/// Opens the row menu of `title` on the Playlists screen.
fn row_menu(h: &mut Harness<'static, AppUi>, title: &str) {
    h.get_all_by_label(title).next().unwrap().click_secondary();
    h.run_steps(2);
}

fn moves(fake: &Fake) -> Vec<fp_model::Command> {
    fake.take_sent()
        .into_iter()
        .filter(|c| matches!(c, fp_model::Command::MoveEntry { .. }))
        .collect()
}

fn disabled(h: &Harness<'static, AppUi>, label: &str) -> bool {
    use egui_kittest::kittest::NodeT;
    h.get_by_label(label).accesskit_node().is_disabled()
}

/// PL10: Move up and Move down in the row menu.
#[test]
fn the_row_menu_moves_an_entry_up_and_down() {
    let (mut h, fake) = touch(1, PORTRAIT);
    tab(&mut h, "Playlists");
    let (e, main) = (fake.entries(), fake.state.load().players[0].playlist);
    fake.take_sent();
    row_menu(&mut h, "Song 2");
    h.get_by_label("Move up").click();
    h.run_steps(2);
    let up = fp_model::Command::MoveEntry {
        entry: e[1],
        to: main,
        index: 0,
    };
    assert_eq!(moves(&fake), vec![up]);
    // The move applied, Song 2 is first: down is before the third row.
    row_menu(&mut h, "Song 2");
    h.get_by_label("Move down").click();
    h.run_steps(2);
    let down = fp_model::Command::MoveEntry {
        entry: e[1],
        to: main,
        index: 2,
    };
    assert_eq!(moves(&fake), vec![down]);
}

#[test]
fn the_first_row_cannot_move_up_nor_the_last_down() {
    let (mut h, _fake) = touch(1, PORTRAIT);
    tab(&mut h, "Playlists");
    row_menu(&mut h, "Song 1");
    assert!(disabled(&h, "Move up"));
    assert!(!disabled(&h, "Move down"));
    h.key_press(egui::Key::Escape);
    h.run_steps(2);
    row_menu(&mut h, "Song 3");
    assert!(disabled(&h, "Move down"));
}

/// Two players, P2 showing a new empty playlist "Other".
fn two_players_two_playlists() -> (Harness<'static, AppUi>, Arc<Fake>, fp_model::PlaylistId) {
    let mut s = state(2, 3);
    let p2 = s.players[1].id;
    fp_model::apply(
        &mut s,
        fp_model::Command::CreatePlaylistIn {
            name: "Other".into(),
            player: p2,
        },
    )
    .unwrap();
    let other = s.players[1].playlist;
    let (mut h, fake) = harness_sized(s, PORTRAIT, |ui| ui.with_touch());
    h.run_steps(3);
    tab(&mut h, "Playlists");
    fake.take_sent();
    (h, fake, other)
}

/// PL10b: Move to another player puts the entry after that player's next
/// in the playlist it shows (the end of an empty one).
#[test]
fn the_row_menu_moves_an_entry_to_another_player() {
    let (mut h, fake, other) = two_players_two_playlists();
    let e = fake.entries();
    row_menu(&mut h, "Song 2");
    h.get_by_label("Move to another player").click();
    h.run_steps(2);
    h.get_all_by_label("P2").last().unwrap().click();
    h.run_steps(2);
    let to_p2 = fp_model::Command::MoveEntry {
        entry: e[1],
        to: other,
        index: 0,
    };
    assert_eq!(moves(&fake), vec![to_p2]);
}

/// PL10b: the entry on air stays with its player.
#[test]
fn the_entry_on_air_cannot_move_to_another_player() {
    let mut s = state(2, 3);
    let first = s.playlists.first_id().unwrap();
    let e0 = s.playlists.get(first).unwrap().entries[0].id;
    s.players[0].current = Some(e0);
    s.players[0].transport = fp_model::Transport::Playing;
    let (mut h, _fake) = harness_sized(s, PORTRAIT, |ui| ui.with_touch());
    h.run_steps(3);
    tab(&mut h, "Playlists");
    row_menu(&mut h, "Song 1");
    assert!(disabled(&h, "Move to another player"));
}

/// The desktop's row menu does not change.
#[test]
fn the_desktop_row_menu_has_no_phone_moves() {
    let (mut h, _fake) = harness(state(2, 3));
    h.run_steps(2);
    h.get_all_by_label("Song 2")
        .next()
        .unwrap()
        .click_secondary();
    h.run_steps(2);
    assert!(h.query_by_label("Remove from playlist").is_some());
    assert!(h.query_by_label("Move up").is_none());
    assert!(h.query_by_label("Move to another player").is_none());
}

/// PL10: a drag from a row's handle moves the entry.
#[test]
fn a_drag_from_the_handle_moves_the_entry() {
    let (mut h, fake) = touch(1, PORTRAIT);
    tab(&mut h, "Playlists");
    fake.take_sent();
    let handle = h.get_all_by_label("Drag to move").nth(2).unwrap().rect();
    let to = h.get_all_by_label("Song 1").next().unwrap().rect();
    drag(
        &mut h,
        handle.center(),
        egui::pos2(handle.center().x, to.top() + 2.0),
    );
    let sent = moves(&fake);
    assert!(
        matches!(
            sent.as_slice(),
            [fp_model::Command::MoveEntry { index: 0, .. }]
        ),
        "{sent:?}"
    );
}

const RESET: &str = "Clear the played marks of this playlist";

/// Reset played, from the footer's ⋯ menu.
fn reset_played(h: &mut Harness<'static, AppUi>) {
    h.get_by_label("More playlist actions").click();
    h.run_steps(2);
    h.get_by_label(RESET).click();
    h.run_steps(3);
}

/// PL11: a phone's footer keeps Add and puts its other actions behind ⋯.
#[test]
fn the_phone_footer_puts_reset_played_behind_more() {
    let mut s = state(1, 3);
    let playlist = s.playlists.iter().next().unwrap().id;
    s.playlists.get_mut(playlist).unwrap().entries[1].legacy_played = true;
    let (mut h, _fake) = harness_sized(s, PORTRAIT, |ui| ui.with_touch());
    h.run_steps(3);
    tab(&mut h, "Playlists");
    assert!(h.query_by_label(ADD).is_some());
    assert!(h.query_by_label(RESET).is_none(), "not in the footer");
    reset_played(&mut h);
    assert!(
        h.query_by_label("Clear the played mark of every track in this playlist?")
            .is_some(),
        "the question"
    );
}

#[test]
fn the_desktop_footer_keeps_reset_played() {
    let (h, _fake) = harness(state(1, 3));
    assert!(h.query_by_label("More playlist actions").is_none());
    assert!(h.query_by_label(RESET).is_some());
}

// --- The rest ---

#[test]
fn a_finger_drag_on_the_playlist_moves_no_row() {
    let (mut h, fake) = touch(1, PORTRAIT);
    tab(&mut h, "Playlists");
    fake.take_sent();
    let from = h.get_all_by_label("Song 3").last().unwrap().rect().center();
    let to = h.get_all_by_label("Song 1").last().unwrap().rect().center();
    drag(&mut h, from, to);
    assert!(
        !fake
            .take_sent()
            .iter()
            .any(|c| matches!(c, fp_model::Command::MoveEntry { .. })),
        "a scroll moved a row"
    );
}

/// PL14: on the Playlists screen too, a drag that starts on the fader moves
/// it, and a scroll beside it does not.
#[test]
fn the_playlists_fader_moves_by_touch_and_not_by_a_scroll() {
    let (mut h, fake) = touch(2, LANDSCAPE);
    tab(&mut h, "Playlists");
    fake.take_sent();
    let fader = h.get_all_by_label_contains("Volume").next().unwrap().rect();
    let from = egui::pos2(60.0, fader.center().y);
    drag(&mut h, from, from + vec2(0.0, -150.0));
    assert!(!volume_sent(&fake), "a scroll moved a fader");
    let (mut h, fake) = touch(2, LANDSCAPE);
    tab(&mut h, "Playlists");
    fake.take_sent();
    let fader = h.get_all_by_label_contains("Volume").next().unwrap().rect();
    drag(&mut h, fader.center(), fader.center() + vec2(0.0, 60.0));
    assert!(volume_sent(&fake), "the fader did not move");
}

/// The status bar's shortcut hints name keys, double clicks and right
/// clicks: a touch screen leaves them out.
#[test]
fn touch_mode_shows_no_keyboard_and_mouse_hints() {
    for size in [PORTRAIT, TABLET] {
        let (h, _fake) = touch(2, size);
        assert!(h.query_by_label_contains("Keys 1–9").is_none(), "{size:?}");
    }
    let (mut h, _fake) = harness_sized(state(2, 3), TABLET, |ui| ui);
    h.run_steps(3);
    assert!(h.query_by_label_contains("Keys 1–9").is_some());
}

/// On a phone the Settings sections are chips above the section, which
/// takes the whole width (a side list would leave it no room).
#[test]
fn phone_settings_put_the_sections_above_the_content() {
    let (mut h, _fake) = touch(2, PORTRAIT);
    more(&mut h, "Settings");
    let buffer = h.get_by_label("Buffer size").rect();
    assert!(buffer.left() < 80.0, "{buffer:?} sits right of a side list");
    let meters = h.get_by_label("Meters").rect();
    assert!(meters.bottom() < buffer.top(), "the chips are above");
    h.get_by_label("Meters").click();
    h.run_steps(3);
    assert!(h.query_by_label("Buffer size").is_none(), "another section");
}

/// A touch tablet keeps the desktop's cartwall grid and Settings side list.
#[test]
fn a_touch_tablet_keeps_the_desktop_cartwall_and_settings() {
    let (mut h, _fake) = touch(2, TABLET);
    let one = h.get_by_label("Cart 1, empty").rect();
    let three = h.get_by_label("Cart 3, empty").rect();
    assert!((three.top() - one.top()).abs() < 1.0, "3 on the row of 1");
    h.get_by_label("Settings").click();
    h.run_steps(3);
    let buffer = h.get_by_label("Buffer size").rect();
    let meters = h.get_by_label("Meters").rect();
    assert!(
        meters.right() < buffer.left(),
        "the side list is left of the body"
    );
}

// --- Back (PL2) ---

#[test]
fn back_closes_settings_before_anything_else() {
    let (mut h, _fake) = touch(2, PORTRAIT);
    tab(&mut h, "Playlists");
    more(&mut h, "Settings");
    let ctx = h.ctx.clone();
    assert!(h.state_mut().back(&ctx));
    h.run_steps(3);
    assert!(h.query_by_label("Buffer size").is_none(), "Settings closed");
    assert!(h.query_by_label(ADD).is_some(), "still on Playlists");
}

#[test]
fn back_from_another_screen_goes_to_live() {
    for screen in ["Carts", "Playlists"] {
        let (mut h, _fake) = touch(2, PORTRAIT);
        tab(&mut h, screen);
        let ctx = h.ctx.clone();
        assert!(h.state_mut().back(&ctx), "{screen}");
        h.run_steps(3);
        assert_eq!(h.query_all_by_label("Play").count(), 2, "{screen}: Live");
        assert!(h.query_by_label(ADD).is_none());
    }
}

#[test]
fn back_with_nothing_open_reports_nothing_to_go_back_to() {
    let (mut h, _fake) = touch(2, PORTRAIT);
    let ctx = h.ctx.clone();
    assert!(!h.state_mut().back(&ctx));
}

/// Back closes an open menu before the screen under it.
#[test]
fn back_closes_a_menu_before_the_screen() {
    let (mut h, _fake) = touch(2, PORTRAIT);
    tab(&mut h, "Playlists");
    let ctx = h.ctx.clone();
    egui::Popup::open_id(&ctx, egui::Id::unique("a menu"));
    assert!(h.state_mut().back(&ctx));
    assert!(!egui::Popup::is_any_open(&ctx));
    h.run_steps(3);
    assert!(h.query_by_label(ADD).is_some(), "still on Playlists");
}

/// Back closes a confirmation before the screen under it.
#[test]
fn back_closes_a_confirmation_before_the_screen() {
    let mut s = state(2, 3);
    let playlist = s.playlists.iter().next().unwrap().id;
    s.playlists.get_mut(playlist).unwrap().entries[1].legacy_played = true;
    let (mut h, _fake) = harness_sized(s, PORTRAIT, |ui| ui.with_touch());
    h.run_steps(3);
    tab(&mut h, "Playlists");
    reset_played(&mut h);
    assert!(
        h.query_by_label("Cancel").is_some(),
        "the confirmation shows"
    );
    let ctx = h.ctx.clone();
    assert!(h.state_mut().back(&ctx));
    h.run_steps(3);
    assert!(
        h.query_by_label("Cancel").is_none(),
        "the confirmation closed"
    );
    assert!(h.query_by_label(ADD).is_some(), "still on Playlists");
}

// --- The lock (PL12, PL13) ---

const LOCK: &str = "Lock faders and waveforms";
const UNLOCK: &str = "Unlock faders and waveforms";

/// A phone with two players, P1 playing a 180 s track, the lock on or off.
fn locked(on: bool) -> (Harness<'static, AppUi>, Arc<Fake>) {
    let mut s = state(2, 3);
    let e = s.playlists.iter().next().unwrap().entries[0].clone();
    fp_model::apply(
        &mut s,
        fp_model::Command::ApplyAnalysis {
            track: e.track,
            analysis: Box::new(fp_model::TrackAnalysis {
                duration_secs: 180.0,
                ..fp_model::TrackAnalysis::default()
            }),
        },
    )
    .unwrap();
    let p = s.players[0].id;
    fp_model::apply(&mut s, fp_model::Command::Play(p)).unwrap();
    s.layout.touch_locked = on;
    let (mut h, fake) = harness_sized(s, PORTRAIT, |ui| ui.with_touch());
    h.run_steps(3);
    fake.take_sent();
    (h, fake)
}

fn seek_sent(fake: &Fake) -> bool {
    fake.take_sent()
        .iter()
        .any(|c| matches!(c, fp_model::Command::Seek(..)))
}

/// A finger tap at `at`.
fn tap(h: &mut Harness<'static, AppUi>, at: egui::Pos2) {
    drag(h, at, at);
}

#[test]
fn the_lock_button_sends_set_touch_locked() {
    let (mut h, fake) = locked(false);
    h.get_by_label(LOCK).click();
    h.run_steps(2);
    assert!(
        fake.take_sent()
            .iter()
            .any(|c| matches!(c, fp_model::Command::SetTouchLocked(true)))
    );
    let (mut h, fake) = locked(true);
    h.get_by_label(UNLOCK).click();
    h.run_steps(2);
    assert!(
        fake.take_sent()
            .iter()
            .any(|c| matches!(c, fp_model::Command::SetTouchLocked(false)))
    );
}

/// PL13: a locked fader's level readout is dimmed with it.
#[test]
fn locked_the_fader_readout_is_dimmed() {
    use egui_kittest::kittest::NodeT;
    for on in [false, true] {
        let (h, _fake) = locked(on);
        let readout = h.get_all_by_label("0.0").next().unwrap();
        assert_eq!(readout.accesskit_node().is_disabled(), on, "locked: {on}");
    }
}

#[test]
fn locked_a_card_fader_does_not_move() {
    let (mut h, fake) = locked(true);
    let fader = h.get_all_by_label_contains("Volume").next().unwrap().rect();
    drag(&mut h, fader.center(), fader.center() + vec2(0.0, 60.0));
    assert!(!volume_sent(&fake), "a locked fader moved");
}

#[test]
fn locked_a_tap_on_a_waveform_does_not_seek() {
    let (mut h, fake) = locked(false);
    let wave = h
        .get_all_by_label("Waveform: click to seek")
        .next()
        .unwrap()
        .rect();
    tap(&mut h, wave.center());
    assert!(seek_sent(&fake), "unlocked, a tap seeks");
    let (mut h, fake) = locked(true);
    let wave = h
        .get_all_by_label("Waveform: click to seek")
        .next()
        .unwrap()
        .rect();
    tap(&mut h, wave.center());
    assert!(!seek_sent(&fake), "locked, a tap seeks nothing");
}

#[test]
fn locked_play_still_plays() {
    let (mut h, fake) = locked(true);
    h.get_all_by_label("Play").last().unwrap().click();
    h.run_steps(2);
    assert!(
        fake.take_sent()
            .iter()
            .any(|c| matches!(c, fp_model::Command::Play(_)))
    );
}

#[test]
fn locked_the_playlists_fader_and_waveform_ignore_touches() {
    let (mut h, fake) = locked(true);
    tab(&mut h, "Playlists");
    fake.take_sent();
    let fader = h.get_all_by_label_contains("Volume").next().unwrap().rect();
    drag(&mut h, fader.center(), fader.center() + vec2(0.0, 60.0));
    let wave = h.get_by_label("Waveform: click to seek").rect();
    tap(&mut h, wave.center());
    let sent = fake.take_sent();
    assert!(
        !sent.iter().any(|c| matches!(
            c,
            fp_model::Command::SetVolume(..) | fp_model::Command::Seek(..)
        )),
        "{sent:?}"
    );
}

#[test]
fn a_tablet_has_no_lock_button() {
    let (h, _fake) = touch(2, TABLET);
    assert!(h.query_by_label(LOCK).is_none());
    let (h, _fake) = touch(2, PORTRAIT);
    assert!(h.query_by_label(LOCK).is_some());
}
