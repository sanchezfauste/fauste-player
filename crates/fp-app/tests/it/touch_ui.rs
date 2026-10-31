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
use egui_kittest::kittest::{NodeT, Queryable};
use fp_app::ui::app::AppUi;
use std::sync::Arc;
use support::{Fake, harness, harness_sized, state};

const PORTRAIT: Vec2 = vec2(412.0, 892.0);
const LANDSCAPE: Vec2 = vec2(892.0, 412.0);
const TABLET: Vec2 = vec2(800.0, 1280.0);
/// A Samsung A56 upright, in dp (1080×2340 at 450 dpi).
const A56: Vec2 = vec2(384.0, 832.0);

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

/// As `drag`, ended the way a finger ends on a device: the release and the
/// pointer leaving come in the same frame (egui-winit sends `PointerGone`
/// with a touch's end).
fn drag_lifting(h: &mut Harness<'static, AppUi>, from: egui::Pos2, to: egui::Pos2) {
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
    // `event` runs one frame per event: both go into one frame's input.
    let events = &mut h.input_mut().events;
    events.push(egui::Event::PointerButton {
        pos: to,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
    events.push(egui::Event::PointerGone);
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
    assert!(h.query_by_label("Audio outputs").is_some());
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

/// On a 6-inch phone held upright (PL5, PL29: finger-sized buttons) the
/// first card shows whole and the second one's top shows under it, so the
/// operator sees there is more to scroll to: about 400 × 860 dp, less the
/// system bars.
#[test]
fn two_cards_fit_a_portrait_phone() {
    let (h, _fake) = touch(3, vec2(400.0, 820.0));
    let carts_top = h.get_by_label("Cart 1, empty").rect().top();
    let plays: Vec<egui::Rect> = h.query_all_by_label("Play").map(|n| n.rect()).collect();
    assert!(plays[0].bottom() < carts_top, "the first card's Play shows");
    let second = h.get_all_by_label("P2").next().unwrap().rect();
    assert!(
        second.bottom() < carts_top,
        "the second card's header shows"
    );
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

/// PL10, on a device: lifting the finger ends the touch in the frame it
/// is released, and the entry still moves where the line showed.
#[test]
fn a_drag_from_the_handle_moves_the_entry_when_the_finger_lifts() {
    let (mut h, fake) = touch(1, PORTRAIT);
    tab(&mut h, "Playlists");
    fake.take_sent();
    let handle = h.get_all_by_label("Drag to move").nth(2).unwrap().rect();
    let to = h.get_all_by_label("Song 1").next().unwrap().rect();
    drag_lifting(
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

/// A finger held still at `at` for a second, then lifted.
fn long_press(h: &mut Harness<'static, AppUi>, at: egui::Pos2) {
    let touch = |phase| egui::Event::Touch {
        device_id: egui::TouchDeviceId(0),
        id: egui::TouchId(0),
        phase,
        pos: at,
        force: None,
    };
    h.event(touch(egui::TouchPhase::Start));
    h.event(egui::Event::PointerMoved(at));
    h.event(egui::Event::PointerButton {
        pos: at,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::NONE,
    });
    for _ in 0..60 {
        h.step();
    }
    h.event(touch(egui::TouchPhase::End));
    h.event(egui::Event::PointerButton {
        pos: at,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
    h.run_steps(3);
}

/// One player stopped on a 180 s analysed track; its cue-in set by hand
/// at 5 s when `manual`.
fn analysed(manual: bool) -> (Harness<'static, AppUi>, Arc<Fake>, fp_model::TrackId) {
    let mut s = state(1, 2);
    let track = s.playlists.iter().next().unwrap().entries[0].track;
    let analysis = fp_model::TrackAnalysis {
        duration_secs: 180.0,
        cue_in: Some(0.0),
        cue_out: Some(180.0),
        ..fp_model::TrackAnalysis::default()
    };
    fp_model::apply(
        &mut s,
        fp_model::Command::ApplyAnalysis {
            track,
            analysis: Box::new(analysis),
        },
    )
    .unwrap();
    if manual {
        fp_model::apply(
            &mut s,
            fp_model::Command::SetMarker {
                track,
                kind: fp_model::MarkerKind::CueIn,
                secs: Some(5.0),
            },
        )
        .unwrap();
    }
    let (mut h, fake) = harness_sized(s, PORTRAIT, |ui| ui.with_touch());
    h.run_steps(3);
    fake.take_sent();
    (h, fake, track)
}

fn markers_set(fake: &Fake) -> Vec<(fp_model::MarkerKind, Option<f64>)> {
    fake.take_sent()
        .into_iter()
        .filter_map(|c| match c {
            fp_model::Command::SetMarker { kind, secs, .. } => Some((kind, secs)),
            _ => None,
        })
        .collect()
}

/// PL11b: a long press on the Playlists waveform sets a marker there.
#[test]
fn a_long_press_on_the_playlists_waveform_sets_a_marker_there() {
    let (mut h, fake, _track) = analysed(false);
    tab(&mut h, "Playlists");
    let wave = waveform(&h);
    long_press(
        &mut h,
        egui::pos2(wave.left() + wave.width() * 0.25, wave.center().y),
    );
    h.get_by_label("Set intro end here").click();
    h.run_steps(2);
    let set = markers_set(&fake);
    assert_eq!(set.len(), 1, "{set:?}");
    assert_eq!(set[0].0, fp_model::MarkerKind::IntroEnd);
    assert!((set[0].1.unwrap() - 45.0).abs() < 3.0, "{set:?}");
}

/// PL11b: a marker set by hand can be removed from the same menu.
#[test]
fn a_manual_marker_can_be_removed_by_touch() {
    let (mut h, fake, _track) = analysed(true);
    tab(&mut h, "Playlists");
    let wave = waveform(&h);
    long_press(&mut h, wave.center());
    assert!(h.query_by_label("Remove intro end").is_none(), "not set");
    h.get_by_label("Remove cue in").click();
    h.run_steps(2);
    assert_eq!(
        markers_set(&fake),
        vec![(fp_model::MarkerKind::CueIn, None)]
    );
}

/// PL11b: the Live cards' waveforms never edit markers.
#[test]
fn a_long_press_on_a_live_waveform_opens_no_marker_menu() {
    let (mut h, _fake, _track) = analysed(false);
    let wave = waveform(&h);
    long_press(&mut h, wave.center());
    assert!(h.query_by_label("Set intro end here").is_none());
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

/// PL20: on a phone Settings fills the screen and opens on its list of
/// sections; a tap opens one, Back returns to the list, then closes.
#[test]
fn phone_settings_open_on_the_section_list() {
    let (mut h, _fake) = touch(2, PORTRAIT);
    more(&mut h, "Settings");
    for name in ["Audio outputs", "Players", "Meters", "Cartwall"] {
        assert!(h.query_by_label(name).is_some(), "{name}");
    }
    assert!(h.query_by_label("Buffer size").is_none(), "no section yet");
    let close = h.get_by_label("Close").rect();
    assert!(close.bottom() > PORTRAIT.y - 80.0, "full screen: {close:?}");
    h.get_by_label("Meters").click();
    h.run_steps(3);
    assert!(
        h.query_by_label("Audio outputs").is_none(),
        "the section alone"
    );
    assert!(h.query_by_label("Back to the sections").is_some());
    let ctx = h.ctx.clone();
    assert!(h.state_mut().back(&ctx));
    h.run_steps(3);
    assert!(
        h.query_by_label("Audio outputs").is_some(),
        "the list again"
    );
    assert!(h.state_mut().back(&ctx));
    h.run_steps(3);
    assert!(h.query_by_label("Close").is_none(), "Settings closed");
}

/// PL20: the arrow goes back to the list too.
#[test]
fn the_arrow_returns_to_the_section_list() {
    let (mut h, _fake) = touch(2, PORTRAIT);
    more(&mut h, "Settings");
    h.get_by_label("Audio outputs").click();
    // Devices are listed by a helper thread; the section's rows show once
    // they are (slow on a Windows runner).
    for _ in 0..200 {
        h.run_steps(1);
        if h.query_by_label("Buffer size").is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(h.query_by_label("Buffer size").is_some());
    h.get_by_label("Back to the sections").click();
    h.run_steps(3);
    assert!(h.query_by_label("Buffer size").is_none());
    assert!(h.query_by_label("Players").is_some());
}

/// PL20: editing a cart on a phone opens its section, not the list.
#[test]
fn editing_a_cart_opens_its_section() {
    let (mut h, _fake) = touch(2, PORTRAIT);
    tab(&mut h, "Carts");
    let cart = h.get_by_label("Cart 1, empty").rect();
    h.hover_at(cart.center());
    h.event(egui::Event::PointerButton {
        pos: cart.center(),
        button: egui::PointerButton::Secondary,
        pressed: true,
        modifiers: egui::Modifiers::NONE,
    });
    h.event(egui::Event::PointerButton {
        pos: cart.center(),
        button: egui::PointerButton::Secondary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
    h.run_steps(3);
    h.get_by_label_contains("Edit").click();
    h.run_steps(3);
    assert!(h.query_by_label("Back to the sections").is_some());
    assert!(h.query_by_label("Audio outputs").is_none(), "not the list");
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
    assert!(h.query_by_label("Close").is_none(), "Settings closed");
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
    // P1 plays: its Play button is Next, which also sends `Play`.
    h.get_by_label("Fade into the next track").click();
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

/// PL10: the drag handle leaves the duration room to be read.
#[test]
fn the_drag_handle_leaves_the_duration_whole() {
    let mut s = state(1, 3);
    for t in s.library.iter_mut() {
        t.duration_secs = 180.0;
    }
    let (mut h, _fake) = harness_sized(s, PORTRAIT, |ui| ui.with_touch());
    h.run_steps(3);
    tab(&mut h, "Playlists");
    let handle = h.get_all_by_label("Drag to move").next().unwrap().rect();
    let duration = h.get_all_by_label("03:00").next().unwrap().rect();
    assert!(duration.right() <= handle.left(), "{duration:?} {handle:?}");
    let header = h.get_by_label("DUR.").rect();
    assert!(
        duration.width() >= header.width() * 0.9,
        "{duration:?} {header:?}"
    );
}

/// The label and the control of Settings' "Number of players" row.
fn player_count_row(h: &Harness<'static, AppUi>) -> (egui::Rect, egui::Rect) {
    let mut nodes: Vec<egui::Rect> = h
        .get_all_by_label("Number of players")
        .map(|n| n.rect())
        .collect();
    assert_eq!(nodes.len(), 2, "the label and the slider");
    nodes.sort_by(|a, b| {
        (a.top(), a.left())
            .partial_cmp(&(b.top(), b.left()))
            .unwrap()
    });
    let (label, control) = (nodes[0], nodes[1]);
    if (label.top() - control.top()).abs() < 1.0 && control.left() < label.left() {
        (control, label)
    } else {
        (label, control)
    }
}

/// PL21: on a phone a setting's control is below its label.
#[test]
fn phone_settings_are_stacked() {
    let (mut h, _fake) = touch(2, PORTRAIT);
    more(&mut h, "Settings");
    h.get_by_label("Players").click();
    h.run_steps(3);
    let (label, control) = player_count_row(&h);
    assert!(control.top() >= label.bottom(), "{label:?} {control:?}");
}

/// The desktop keeps the label column beside the control.
#[test]
fn desktop_settings_keep_the_label_column() {
    let (mut h, _fake) = harness(state(2, 3));
    h.run_steps(3);
    h.get_by_label("Settings").click();
    h.run_steps(3);
    h.get_by_label("Players").click();
    h.run_steps(3);
    let (label, control) = player_count_row(&h);
    assert!(control.left() > label.right(), "{label:?} {control:?}");
}

/// PL20: a phone's Settings fills the screen the system bars leave.
#[test]
fn phone_settings_stay_clear_of_the_system_bars() {
    let (mut h, _fake) =
        support::harness_inset(state(2, 3), PORTRAIT, 24.0, 48.0, |ui| ui.with_touch());
    h.run_steps(3);
    more(&mut h, "Settings");
    let close = h.get_by_label("Close").rect();
    assert!(close.bottom() <= PORTRAIT.y - 48.0, "{close:?}");
    let first = h.get_by_label("Audio outputs").rect();
    assert!(first.top() >= 24.0 + 40.0, "under the header: {first:?}");
    let title = h.get_all_by_label("Settings").last().unwrap().rect();
    assert!(title.top() >= 24.0, "the header too: {title:?}");
}

/// PL20: a phone on its side still reaches Close.
#[test]
fn phone_settings_on_its_side_reach_close() {
    let (mut h, _fake) =
        support::harness_inset(state(2, 3), LANDSCAPE, 24.0, 24.0, |ui| ui.with_touch());
    h.run_steps(3);
    more(&mut h, "Settings");
    h.get_by_label("Audio outputs").click();
    h.run_steps(3);
    let close = h.get_by_label("Close").rect();
    assert!(close.bottom() <= LANDSCAPE.y - 24.0, "{close:?}");
}

/// PL20: Back over Restore defaults' question closes the question only.
#[test]
fn back_closes_the_restore_question_first() {
    let (mut h, _fake) = touch(2, PORTRAIT);
    more(&mut h, "Settings");
    h.get_by_label("Loudness").click();
    h.run_steps(3);
    h.get_by_label("Restore defaults").click();
    h.run_steps(3);
    let questions = h.get_all_by_label("Cancel").count();
    assert!(questions > 0, "the question shows");
    let ctx = h.ctx.clone();
    assert!(h.state_mut().back(&ctx));
    h.run_steps(3);
    assert!(h.query_by_label("Cancel").is_none(), "the question closed");
    assert!(
        h.query_by_label("Back to the sections").is_some(),
        "the section stays"
    );
}

/// PL21: the Playlists section's rows fit a phone's width.
#[test]
fn the_playlists_section_fits_a_phone() {
    let (mut h, _fake) = touch(2, PORTRAIT);
    more(&mut h, "Settings");
    h.get_all_by_label("Playlists").last().unwrap().click();
    h.run_steps(3);
    let delete = h.get_by_label("Delete").rect();
    assert!(delete.right() <= PORTRAIT.x, "{delete:?}");
}

/// Every text a frame drew.
fn drawn_texts(h: &Harness<'static, AppUi>) -> Vec<String> {
    drawn(h, false)
}

/// The texts a frame drew cut short ("…") in the top bar.
fn cut_in_top_bar(h: &Harness<'static, AppUi>) -> Vec<String> {
    let only_bar: Vec<_> = h
        .output()
        .shapes
        .iter()
        .filter_map(|c| match &c.shape {
            egui::Shape::Text(t) if t.galley.elided && t.pos.y < 56.0 => {
                Some(t.galley.text().to_owned())
            }
            _ => None,
        })
        .collect();
    only_bar
}

fn drawn(h: &Harness<'static, AppUi>, only_cut: bool) -> Vec<String> {
    let walk_cut = only_cut;
    fn walk(shape: &egui::Shape, only_cut: bool, out: &mut Vec<String>) {
        match shape {
            egui::Shape::Text(t) if !only_cut || t.galley.elided => {
                out.push(t.galley.text().to_owned());
            }
            egui::Shape::Vec(v) => v.iter().for_each(|s| walk(s, only_cut, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    for clipped in &h.output().shapes {
        walk(&clipped.shape, walk_cut, &mut out);
    }
    out
}

/// A phone in German, its tabs found by name, none cut short.
fn german(size: Vec2) -> Harness<'static, AppUi> {
    let mut s = state(2, 3);
    s.config.ui.language = Some("de-DE".to_owned());
    let (mut h, _fake) = harness_sized(s, size, |ui| ui.with_touch());
    h.run_steps(3);
    h
}

/// PL26: a phone's tabs are icons named for a screen reader; no name is
/// cut short with "…", in a long language either.
#[test]
fn a_phones_tabs_are_icons_and_none_is_cut_short() {
    for size in [PORTRAIT, LANDSCAPE, A56] {
        let (h, _fake) = touch(2, size);
        for name in ["Live", "Carts", "Playlists", "Music"] {
            assert!(h.query_by_label(name).is_some(), "{name} {size:?}");
        }
        let cut = cut_in_top_bar(&h);
        assert!(cut.is_empty(), "{cut:?} {size:?}");
        let h = german(size);
        let cut = cut_in_top_bar(&h);
        assert!(cut.is_empty(), "de-DE {cut:?} {size:?}");
    }
}

/// PL26: the chosen tab adds its name beside its icon when there is room
/// (in landscape; upright the clock keeps it).
#[test]
fn the_chosen_tab_shows_its_name() {
    let (mut h, _fake) = touch(2, LANDSCAPE);
    tab(&mut h, "Playlists");
    let chosen = h.get_by_label("Playlists").rect().width();
    let other = h.get_by_label("Carts").rect().width();
    assert!(chosen > other + 20.0, "{chosen} {other}");
    assert!(drawn_texts(&h).iter().any(|t| t == "Playlists"));
    assert!(!drawn_texts(&h).iter().any(|t| t == "Carts"));
}

/// PL27: in landscape the Playlists screen's player chips sit in the top
/// bar; upright they keep their own row under it.
#[test]
fn landscape_playlists_put_the_player_chips_in_the_top_bar() {
    let (mut h, _fake) = touch(4, LANDSCAPE);
    tab(&mut h, "Playlists");
    // The top bar is 56 high under the harness's 8 margin.
    let bar = h.get_by_label("Playlists").rect();
    let chip = h.get_by_label("P4").rect();
    assert!(
        chip.top() >= bar.top() && chip.bottom() <= bar.bottom(),
        "{chip:?} {bar:?}"
    );
    let (mut h, _fake) = touch(4, PORTRAIT);
    tab(&mut h, "Playlists");
    let bar = h.get_by_label("Playlists").rect();
    let chip = h.get_all_by_label("P4").next().unwrap().rect();
    assert!(chip.top() >= bar.bottom(), "{chip:?} {bar:?}");
}

/// PL28: a phone in landscape leaves the status legend out; upright and on
/// a tablet it stays.
#[test]
fn a_landscape_phone_leaves_the_legend_out() {
    let (h, _fake) = touch(2, LANDSCAPE);
    assert!(h.query_by_label("Intro").is_none());
    for size in [PORTRAIT, vec2(1280.0, 800.0)] {
        let (h, _fake) = touch(2, size);
        assert!(h.query_by_label("Intro").is_some(), "{size:?}");
    }
}

/// PL29: a phone's transport buttons are finger targets.
#[test]
fn a_phones_transport_buttons_are_finger_sized() {
    for size in [A56, LANDSCAPE] {
        let (h, _fake) = touch(2, size);
        for name in ["Stop", "Pause", "Previous track", "Fade stop"] {
            let r = h.get_all_by_label(name).next().unwrap().rect();
            assert!(
                r.height() >= 48.0 && r.width() >= 48.0,
                "{name} {r:?} {size:?}"
            );
        }
    }
}

/// PL29: a phone's card shows the elapsed time and the track's total.
#[test]
fn a_phones_card_shows_elapsed_and_total() {
    for size in [A56, LANDSCAPE] {
        let (mut h, _fake) = touch(1, size);
        // The test tracks have no length: the total shows as unknown.
        assert!(h.query_by_label("00:00 / —").is_some(), "{size:?}");
        tab(&mut h, "Playlists");
        assert!(h.query_by_label("00:00 / —").is_some(), "{size:?}");
    }
}

/// PL28: the Playlists card shows its transport whole in landscape.
#[test]
fn the_landscape_playlists_card_shows_its_transport() {
    let (mut h, _fake) = touch(4, LANDSCAPE);
    tab(&mut h, "Playlists");
    let screen = h.get_by_label("Playlists").rect().max.y;
    for name in ["Play", "Stop after the current track", "Fade stop"] {
        let r = h.get_all_by_label(name).next().unwrap().rect();
        assert!(
            r.bottom() <= LANDSCAPE.y && r.top() > screen,
            "{name} {r:?}"
        );
    }
}

/// PL30: a phone's meter and fader are narrow and draw no scale numbers.
#[test]
fn a_phones_meter_is_narrow_without_a_scale() {
    let (h, _fake) = touch(1, A56);
    let fader = h
        .get_all_by_role(egui::accesskit::Role::Slider)
        .next()
        .unwrap()
        .rect();
    assert!(fader.width() <= 30.0, "{fader:?}");
    let texts = drawn_texts(&h);
    assert!(!texts.iter().any(|t| t == "-20" || t == "-12"), "{texts:?}");
}

/// PL28: a compact card keeps its buttons inside it: the next card starts
/// below the first card's lowest button.
#[test]
fn a_compact_card_keeps_its_buttons_inside() {
    let (h, _fake) = touch(2, A56);
    let lowest = h.get_all_by_label("Fade stop").next().unwrap().rect();
    let next_wave = h
        .get_all_by_label("Waveform: click to seek")
        .nth(1)
        .unwrap()
        .rect();
    assert!(
        lowest.bottom() + 10.0 < next_wave.top(),
        "{lowest:?} {next_wave:?}"
    );
}

/// PL28: upright, the Playlists card shows its transport whole too.
#[test]
fn the_portrait_playlists_card_shows_its_transport() {
    // The A56's status bar (24) and gesture bar (24) take their share.
    let (mut h, _fake) = support::harness_inset(state(4, 3), A56, 24.0, 24.0, |ui| ui.with_touch());
    h.run_steps(3);
    tab(&mut h, "Playlists");
    // Above the status bar, whose legend sits in it.
    let bottom = h.get_by_label("Intro").rect().top() - 4.0;
    for name in ["Play", "Stop after the current track", "Fade stop"] {
        let r = h.get_all_by_label(name).next().unwrap().rect();
        assert!(r.bottom() <= bottom, "{name} {r:?}");
    }
}

/// PL31: the interface keeps drawing while a meter still shows a level
/// on its scale (a stopped player's meter falling), and stops once it is
/// at the bottom.
#[test]
fn a_falling_meter_keeps_the_interface_drawing() {
    let c = fp_model::MeterConfig::default();
    let mut reading = fp_engine::meter::MeterReading::default();
    assert!(!fp_app::ui::view::meter_moving(&reading, &c));
    reading.level_db = [-20.0, -24.0];
    assert!(fp_app::ui::view::meter_moving(&reading, &c));
    reading.level_db = [-119.0; 2];
    reading.hold_db = [-6.0; 2];
    assert!(
        fp_app::ui::view::meter_moving(&reading, &c),
        "the hold still shows"
    );
}

/// PL32: idle, the next frame is just after the clock's next second (a
/// frame a hair early would only ask for another one at once).
#[test]
fn idle_waits_for_the_next_second() {
    use fp_app::ui::view::idle_wait;
    use std::time::Duration;
    assert_eq!(idle_wait(10), Duration::from_millis(1000));
    assert_eq!(idle_wait(250), Duration::from_millis(760));
    assert_eq!(idle_wait(999), Duration::from_millis(11));
}

/// Opens Settings → `section` on an upright A56.
fn a56_settings(section: &str) -> Harness<'static, AppUi> {
    let (mut h, _fake) = touch(2, A56);
    more(&mut h, "Settings");
    h.get_by_label(section).click();
    h.run_steps(3);
    h
}

/// PL33: the cartwall editor fits an upright phone: the cart grid (eight
/// columns by default) wraps to the width, and the rows and columns
/// sliders get a line each, wide enough for a finger.
#[test]
fn the_cartwall_editor_fits_an_upright_phone() {
    let h = a56_settings("Cartwall");
    let carts: Vec<egui::Rect> = h
        .query_all_by_label_contains("Cart ")
        .filter(|n| n.accesskit_node().role() == egui::accesskit::Role::Button)
        .map(|n| n.rect())
        .collect();
    assert!(carts.len() >= 16, "{}", carts.len());
    for cart in &carts {
        assert!(cart.right() <= A56.x, "{cart:?}");
        assert!(cart.width() >= 48.0, "{cart:?}");
    }
    let slider = |name: &str| {
        h.get_all_by_role(egui::accesskit::Role::Slider)
            .find(|n| n.accesskit_node().label().as_deref() == Some(name))
            .unwrap()
            .rect()
    };
    let (rows, cols) = (slider("Rows"), slider("Columns"));
    assert!(cols.top() >= rows.bottom(), "{rows:?} {cols:?}");
    for slider in [rows, cols] {
        assert!(
            slider.right() <= A56.x && slider.width() >= 150.0,
            "{slider:?}"
        );
    }
}

/// The A56 lying down, in dp.
const A56_LANDSCAPE: Vec2 = vec2(832.0, 384.0);

/// The widgets of what shows that reach past the screen's right edge.
fn overflowing(h: &Harness<'static, AppUi>, width: f32) -> Vec<String> {
    h.root()
        .query_all(egui_kittest::kittest::By::new().predicate(|_| true))
        .filter_map(|n| {
            let b = n.accesskit_node().bounding_box()?;
            (b.width() > 0.0 && b.x1 as f32 > width + 0.5).then(|| {
                format!(
                    "{:?} {:?} x {:.0}..{:.0}",
                    n.accesskit_node().role(),
                    n.accesskit_node().label(),
                    b.x0,
                    b.x1
                )
            })
        })
        .collect()
}

/// PL33: every Settings section fits a phone, upright and lying down: no
/// widget reaches past the screen's right edge.
#[test]
fn every_settings_section_fits_a_phone() {
    let sections = [
        "Audio outputs",
        "Players",
        "Meters",
        "Analysis",
        "Loudness",
        "Playlists",
        "Cartwall",
        "Keyboard shortcuts",
    ];
    let mut found = Vec::new();
    for size in [A56, A56_LANDSCAPE] {
        for section in sections {
            let (mut h, _fake) = touch(2, size);
            more(&mut h, "Settings");
            h.get_all_by_label(section).last().unwrap().click();
            h.run_steps(3);
            for o in overflowing(&h, size.x) {
                found.push(format!("{size:?} {section}: {o}"));
            }
        }
    }
    assert!(found.is_empty(), "{}", found.join("\n"));
}

/// PL33: the four screens and About fit a phone, upright and lying down.
#[test]
fn every_phone_screen_fits() {
    let mut found = Vec::new();
    for size in [A56, A56_LANDSCAPE] {
        for screen in ["Live", "Carts", "Playlists", "Music", "About"] {
            let (mut h, _fake) = touch(4, size);
            if screen == "About" {
                more(&mut h, "About");
            } else {
                tab(&mut h, screen);
            }
            for o in overflowing(&h, size.x) {
                found.push(format!("{size:?} {screen}: {o}"));
            }
        }
    }
    assert!(found.is_empty(), "{}", found.join("\n"));
}

/// PL32: the interface gives the conductor its waker.
#[test]
fn the_interface_gives_its_waker() {
    let (_h, fake) = touch(1, A56);
    assert!(fake.waker_set.load(std::sync::atomic::Ordering::Relaxed));
}

/// PL32: when the next frame comes. Something moving: at `max_fps`. A
/// paused player blinks (2 Hz): four frames a second. Idle, a touch device
/// waits for the clock's next second, a desktop draws ten frames a second
/// as before.
#[test]
fn the_next_frame_follows_what_moves() {
    use fp_app::ui::view::next_frame;
    use std::time::Duration;
    let telemetry = fp_engine::conductor::Telemetry::default();
    let mut s = state(1, 3);
    assert_eq!(
        next_frame(&s, &telemetry, false, 250),
        Duration::from_millis(100)
    );
    assert_eq!(
        next_frame(&s, &telemetry, true, 250),
        Duration::from_millis(760)
    );
    let p = s.players[0].id;
    fp_model::apply(&mut s, fp_model::Command::Play(p)).unwrap();
    assert!(next_frame(&s, &telemetry, true, 250) <= Duration::from_millis(34));
    fp_model::apply(&mut s, fp_model::Command::Pause(p)).unwrap();
    assert_eq!(s.players[0].transport, fp_model::Transport::Paused);
    assert!(next_frame(&s, &telemetry, true, 250) <= Duration::from_millis(250));
}
