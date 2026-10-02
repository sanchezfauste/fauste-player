#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Phase 2 spec P2.9: the cartwall strip.

mod support;

use std::path::PathBuf;
use std::sync::Arc;

use egui_kittest::kittest::{NodeT, Queryable};
use fp_app::ui::cart_view::{CartStatus, cart_view, page_on_air};
use fp_engine::conductor::Telemetry;
use fp_engine::engine::CartTelemetry;
use fp_model::{AppState, CartEdit, CartKind, Command, FileState, TrackAnalysis, apply};
use support::{Fake, harness, state};

/// A state whose first page has "Station ID" (10 s, cue 0.5–9.5) in cell 0.
fn with_cart() -> AppState {
    let mut s = state(1, 0);
    let page = s.cartwall.pages[0].id;
    apply(
        &mut s,
        Command::AssignCartFile {
            page,
            index: 0,
            path: PathBuf::from("/carts/id.wav"),
        },
    )
    .unwrap();
    let track = s.cartwall.pages[0].carts[0].track.unwrap();
    let analysis = TrackAnalysis {
        duration_secs: 10.0,
        cue_in: Some(0.5),
        cue_out: Some(9.5),
        ..TrackAnalysis::default()
    };
    apply(
        &mut s,
        Command::ApplyAnalysis {
            track,
            analysis: Box::new(analysis),
        },
    )
    .unwrap();
    let edit = CartEdit {
        name: "Station ID".into(),
        kind: CartKind::Jingle,
        looped: false,
        exclusive: false,
    };
    apply(
        &mut s,
        Command::SetCart {
            page,
            index: 0,
            edit,
        },
    )
    .unwrap();
    s
}

#[test]
fn cart_views_show_length_countdown_and_progress() {
    let mut s = with_cart();
    let id = s.cartwall.pages[0].carts[0].id;
    let idle = cart_view(&s, &Telemetry::default(), id).unwrap();
    assert_eq!(idle.status, CartStatus::Idle);
    assert_eq!(idle.time, "00:09");
    apply(&mut s, Command::FireCart(id)).unwrap();
    let telemetry = Telemetry {
        carts: vec![(
            id,
            CartTelemetry {
                position_secs: 3.0,
                peak: 0.0,
            },
        )],
        ..Telemetry::default()
    };
    let playing = cart_view(&s, &telemetry, id).unwrap();
    assert_eq!(playing.status, CartStatus::Playing);
    assert_eq!(playing.time, "-00:06");
    assert!((playing.remaining_fraction - 6.5 / 9.0).abs() < 1e-3);
}

#[test]
fn empty_and_unavailable_carts_are_told_apart() {
    let mut s = with_cart();
    let empty = s.cartwall.pages[0].carts[1].id;
    assert_eq!(
        cart_view(&s, &Telemetry::default(), empty).unwrap().status,
        CartStatus::Empty
    );
    let track = s.cartwall.pages[0].carts[0].track.unwrap();
    apply(
        &mut s,
        Command::SetFileState {
            track,
            state: FileState::Missing,
        },
    )
    .unwrap();
    let id = s.cartwall.pages[0].carts[0].id;
    assert_eq!(
        cart_view(&s, &Telemetry::default(), id).unwrap().status,
        CartStatus::Unavailable
    );
}

#[test]
fn a_playing_page_is_marked() {
    let mut s = with_cart();
    let page = s.cartwall.pages[0].id;
    assert!(!page_on_air(&s, page));
    let id = s.cartwall.pages[0].carts[0].id;
    apply(&mut s, Command::FireCart(id)).unwrap();
    assert!(page_on_air(&s, page));
}

#[test]
fn clicking_a_cart_fires_it() {
    let (mut h, fake) = harness(with_cart());
    h.get_by_label("Station ID").click();
    h.run_steps(2);
    let id = fake.state.load().cartwall.pages[0].carts[0].id;
    assert!(fake.take_sent().contains(&Command::FireCart(id)));
}

#[test]
fn clicking_an_empty_cart_sends_nothing() {
    let (mut h, fake) = harness(with_cart());
    h.get_by_label("Cart 2, empty").click();
    h.run_steps(2);
    assert!(
        !fake
            .take_sent()
            .iter()
            .any(|c| matches!(c, Command::FireCart(_)))
    );
}

#[test]
fn the_cartwall_collapses_and_expands() {
    let (mut h, fake) = harness(with_cart());
    h.get_by_label("CARTWALL").click();
    h.run_steps(3);
    assert!(fake.take_sent().contains(&Command::SetCartwallOpen(false)));
    assert!(h.query_by_label("Station ID").is_none(), "collapsed");
    h.get_by_label("CARTWALL").click();
    h.run_steps(3);
    assert!(h.query_by_label("Station ID").is_some());
}

#[test]
fn cart_page_tabs_show_default_names_and_switch_pages() {
    let mut s = with_cart();
    apply(
        &mut s,
        Command::CreateCartPage {
            name: "Sports".into(),
        },
    )
    .unwrap();
    let (mut h, fake) = harness(s);
    assert!(
        h.query_by_label("Carts 1").is_some(),
        "an unnamed page gets a default name"
    );
    h.get_by_label("Sports").click();
    h.run_steps(2);
    let sports = fake.state.load().cartwall.pages[1].id;
    assert!(fake.take_sent().contains(&Command::ShowCartPage(sports)));
}

#[test]
fn the_cart_menu_pre_listens_on_cue() {
    let (mut h, fake) = harness(with_cart());
    h.get_by_label("Station ID").click_secondary();
    h.run_steps(2);
    h.get_by_label("Pre-listen on CUE").click();
    h.run_steps(2);
    let id = fake.state.load().cartwall.pages[0].carts[0].id;
    assert!(fake.take_sent().contains(&Command::CueCart(id)));
    let _ = Arc::strong_count(&fake);
}

#[allow(dead_code)]
fn unused(_: &Fake) {}

#[test]
fn the_last_cart_is_reachable_in_a_small_window() {
    let fake = support::Fake::new(with_cart());
    let app = fp_app::ui::app::AppUi::new(
        fake.clone(),
        fp_app::i18n::I18n::new(Some("en-US")),
        fp_app::services::MediaCache::default(),
    );
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(420.0, 480.0))
        .with_step_dt(0.02)
        .build_ui_state(|ui, app: &mut fp_app::ui::app::AppUi| app.ui(ui), app);
    h.run_steps(3);
    let last = fake.state.load().cartwall.pages[0].carts[15].id;
    let node = h.get_all_by_label("Cart 16, empty").next().unwrap();
    node.scroll_to_me();
    h.run_steps(3);
    // Give the last cart a file so a click fires it.
    let page = fake.state.load().cartwall.pages[0].id;
    fp_app::ui::controller::Controller::send(
        fake.as_ref(),
        Command::AssignCartFile {
            page,
            index: 15,
            path: PathBuf::from("/carts/last.wav"),
        },
    );
    h.run_steps(3);
    let _ = fake.take_sent();
    h.get_by_label("last").scroll_to_me();
    h.run_steps(3);
    h.get_by_label("last").click();
    h.run_steps(2);
    assert!(fake.take_sent().contains(&Command::FireCart(last)));
}

#[test]
fn an_empty_cart_can_be_edited_from_its_menu() {
    let (mut h, _fake) = harness(with_cart());
    h.get_by_label("Cart 2, empty").click_secondary();
    h.run_steps(2);
    assert!(
        h.query_by_label("Pre-listen on CUE").is_none(),
        "nothing to cue"
    );
    h.get_by_label("Edit…").click();
    h.run_steps(3);
    // Settings opens on that cart's editor, where a file is chosen.
    assert!(h.query_by_label("No file").is_some());
    assert!(h.query_by_label("Choose…").is_some());
}

#[test]
fn hovering_an_unavailable_cart_says_why() {
    let mut s = with_cart();
    let track = s.cartwall.pages[0].carts[0].track.unwrap();
    apply(
        &mut s,
        Command::SetFileState {
            track,
            state: FileState::Missing,
        },
    )
    .unwrap();
    let (mut h, _fake) = harness(s);
    h.get_by_label("Station ID").hover();
    h.run_steps(40);
    assert!(
        h.query_by_label_contains("File not found: /carts/id.wav")
            .is_some(),
        "the tooltip gives the reason and the path"
    );
}

/// Gives cart `index` of the first page a 10 s file called `name`.
fn give_file(s: &mut AppState, index: usize, name: &str) {
    let page = s.cartwall.pages[0].id;
    apply(
        s,
        Command::AssignCartFile {
            page,
            index,
            path: PathBuf::from(format!("/carts/{name}.wav")),
        },
    )
    .unwrap();
    let track = s.cartwall.pages[0].carts[index].track.unwrap();
    let analysis = TrackAnalysis {
        duration_secs: 10.0,
        cue_in: Some(0.5),
        cue_out: Some(9.5),
        ..TrackAnalysis::default()
    };
    apply(
        s,
        Command::ApplyAnalysis {
            track,
            analysis: Box::new(analysis),
        },
    )
    .unwrap();
}

/// A state with `n` carts (the first is "Station ID") all playing.
fn with_playing_carts(n: usize) -> AppState {
    let mut s = with_cart();
    for i in 1..n {
        give_file(&mut s, i, &format!("c{i}"));
    }
    for i in 0..n {
        let id = s.cartwall.pages[0].carts[i].id;
        apply(&mut s, Command::FireCart(id)).unwrap();
    }
    s
}

fn stop_all_sent(fake: &Fake) -> bool {
    fake.take_sent().contains(&Command::StopAllCarts)
}

#[test]
fn stop_all_is_dimmed_and_has_no_count_while_no_cart_plays() {
    let (mut h, fake) = harness(with_cart());
    let button = h.get_by_label("Stop all");
    assert!(button.accesskit_node().is_disabled());
    button.click();
    h.run_steps(2);
    assert!(!stop_all_sent(&fake), "a dimmed button sends nothing");
}

#[test]
fn stop_all_shows_the_number_of_playing_carts() {
    let (mut h, _fake) = harness(with_playing_carts(2));
    h.run_steps(2);
    let button = h.get_by_label("Stop all (2)");
    assert!(!button.accesskit_node().is_disabled());
    assert!(
        h.query_by_label("Stop all").is_none(),
        "no label without a count"
    );
}

#[test]
fn stop_all_counts_carts_on_every_page() {
    let mut s = with_playing_carts(2);
    apply(
        &mut s,
        Command::CreateCartPage {
            name: "Sports".into(),
        },
    )
    .unwrap();
    let sports = s.cartwall.pages[1].id;
    apply(&mut s, Command::ShowCartPage(sports)).unwrap();
    let (h, _fake) = harness(s);
    assert!(
        h.query_by_label("Station ID").is_none(),
        "the carts playing are on the page not shown"
    );
    assert!(h.query_by_label("Stop all (2)").is_some());
}

#[test]
fn clicking_stop_all_stops_every_cart() {
    let (mut h, fake) = harness(with_playing_carts(2));
    h.get_by_label("Stop all (2)").click();
    h.run_steps(3);
    assert!(stop_all_sent(&fake));
    assert!(fake.state.load().cartwall.playing.is_empty());
}

#[test]
fn stop_all_dims_again_when_the_last_cart_stops() {
    let (mut h, fake) = harness(with_playing_carts(1));
    assert!(h.query_by_label("Stop all (1)").is_some());
    let id = fake.state.load().cartwall.pages[0].carts[0].id;
    fp_app::ui::controller::Controller::send(fake.as_ref(), Command::StopCart(id));
    h.run_steps(3);
    assert!(h.query_by_label("Stop all (1)").is_none(), "no stale count");
    assert!(h.get_by_label("Stop all").accesskit_node().is_disabled());
}

#[test]
fn stop_all_works_with_the_cartwall_collapsed() {
    let (mut h, fake) = harness(with_playing_carts(2));
    h.get_by_label("CARTWALL").click();
    h.run_steps(3);
    assert!(h.query_by_label("Station ID").is_none(), "collapsed");
    h.get_by_label("Stop all (2)").click();
    h.run_steps(3);
    assert!(stop_all_sent(&fake));
}

#[test]
fn a_cart_cue_alone_does_not_enable_stop_all() {
    let mut s = with_cart();
    let id = s.cartwall.pages[0].carts[0].id;
    apply(&mut s, Command::CueCart(id)).unwrap();
    assert!(s.cartwall.cue.is_some() && s.cartwall.playing.is_empty());
    let (h, _fake) = harness(s);
    assert!(h.get_by_label("Stop all").accesskit_node().is_disabled());
}

#[test]
fn stop_all_sits_at_the_right_end_of_the_bar() {
    let (h, _fake) = harness(with_playing_carts(2));
    let stop = h.get_by_label("Stop all (2)").rect();
    let title = h.get_by_label("CARTWALL").rect();
    assert!(stop.left() > title.right(), "after the collapse control");
    assert!(stop.right() > 1000.0 - 60.0, "at the right end: {stop:?}");
}

#[test]
fn stop_all_stays_at_the_right_end_in_a_narrow_window() {
    let mut s = with_playing_carts(2);
    for n in 0..8 {
        apply(
            &mut s,
            Command::CreateCartPage {
                name: format!("A rather long page name {n}"),
            },
        )
        .unwrap();
    }
    let (mut h, _fake) = support::harness_sized(s, egui::vec2(420.0, 480.0), |ui| ui);
    h.run_steps(3);
    let stop = h.get_by_label("Stop all (2)").rect();
    assert!(stop.right() <= 420.0 + 0.5, "inside the window: {stop:?}");
    assert!(stop.right() > 420.0 - 60.0, "at the right end: {stop:?}");
    assert!(stop.width() > 40.0, "not squeezed: {stop:?}");
}
