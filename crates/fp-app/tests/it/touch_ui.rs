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
use support::{Fake, harness_sized, state};

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

// --- Playlists (interim) and the rest ---

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
    h.get_by_label("Clear the played marks of this playlist")
        .click();
    h.run_steps(3);
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
