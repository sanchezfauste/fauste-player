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

use egui_kittest::kittest::Queryable;
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
