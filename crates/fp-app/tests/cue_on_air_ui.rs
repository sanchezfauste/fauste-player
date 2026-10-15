#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! On-air pre-listen spec §5: the confirmation, the notice and the CUE
//! tooltip.

mod support;

use std::path::PathBuf;
use std::sync::Arc;

use egui::Key;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use fp_app::ui::app::AppUi;
use fp_app::ui::controller::Controller;
use fp_model::{AppState, Command, CueStart, Holder};
use support::{Fake, harness, on_desk, state};

const TITLE: &str = "Pre-listen on air?";
const BODY_P2: &str = "This pre-listen would go out on the on-air output, where Player 2 is playing. It would be heard on air.";
const ANYWAY: &str = "Pre-listen anyway";
const ON_AIR_TIP: &str = "Pre-listen (PFL) the next track. It goes out on the on-air output: while something plays there, it asks first.";
const STOPPED_P2: &str = "Pre-listen stopped: Player 2 started on the same output.";

/// Two players over three tracks: player 1's Cue and player 2's Main share
/// `desk`; player 1's Main stays on the default output. Nothing plays.
fn shared() -> AppState {
    let mut s = state(2, 3);
    let (p1, p2) = (s.players[0].id, s.players[1].id);
    on_desk(&mut s, &[Holder::PlayerCue(p1), Holder::PlayerMain(p2)]);
    s
}

/// `shared`, with player 2 playing.
fn sounding() -> AppState {
    let mut s = shared();
    let p2 = s.players[1].id;
    fp_model::apply(&mut s, Command::Play(p2)).unwrap();
    s
}

/// `sounding`, after a click on player 1's CUE.
fn asked() -> (Harness<'static, AppUi>, Arc<Fake>) {
    let (mut h, fake) = harness(sounding());
    h.get_by_label(ON_AIR_TIP).click();
    // The modal centres itself over two frames: let it settle before a click.
    h.run_steps(4);
    fake.take_sent();
    (h, fake)
}

fn cueing(fake: &Fake, n: usize) -> bool {
    fake.state
        .load()
        .player(fake.player(n))
        .unwrap()
        .cue
        .is_some()
}

#[test]
fn the_cue_button_of_an_on_air_cue_says_where_it_goes() {
    let (h, _) = harness(shared());
    assert_eq!(h.query_all_by_label(ON_AIR_TIP).count(), 1, "player 1 only");
    assert!(!h.get_by_label(ON_AIR_TIP).accesskit_node().is_disabled());
}

#[test]
fn on_a_quiet_on_air_output_the_cue_starts_without_asking() {
    let (mut h, fake) = harness(shared());
    h.get_by_label(ON_AIR_TIP).click();
    h.run_steps(2);
    assert!(h.query_by_label(TITLE).is_none());
    assert_eq!(fake.take_sent(), vec![Command::ToggleCue(fake.player(0))]);
    assert!(cueing(&fake, 0));
}

#[test]
fn on_a_sounding_on_air_output_the_cue_asks_first() {
    let (h, fake) = asked();
    assert!(h.query_by_label(TITLE).is_some());
    assert!(h.query_by_label(BODY_P2).is_some());
    assert!(!cueing(&fake, 0), "nothing started");
}

#[test]
fn pre_listen_anyway_sends_the_request_forced() {
    let (mut h, fake) = asked();
    let start = fake.state.load().cue_on_air.request.clone().unwrap().start;
    h.get_by_label(ANYWAY).click();
    h.run_steps(2);
    assert_eq!(fake.take_sent(), vec![Command::ForceCue(start)]);
    assert!(h.query_by_label(TITLE).is_none());
    assert!(cueing(&fake, 0));
}

/// Review Focus 2.
#[test]
fn cancel_escape_and_enter_cancel() {
    for how in ["button", "escape", "enter"] {
        let (mut h, fake) = asked();
        let id = fake.state.load().cue_on_air.request.clone().unwrap().id;
        match how {
            "button" => h.get_by_label("Cancel").click(),
            "escape" => h.key_press(Key::Escape),
            _ => h.key_press(Key::Enter),
        }
        h.run_steps(2);
        assert_eq!(fake.take_sent(), vec![Command::CancelCueOnAir(id)], "{how}");
        assert!(h.query_by_label(TITLE).is_none(), "{how}");
        assert!(!cueing(&fake, 0), "{how}");
    }
}

/// Review Focus 1.
#[test]
fn the_question_goes_when_the_output_goes_quiet() {
    let (mut h, fake) = asked();
    fake.send(Command::Stop(fake.player(1)));
    fake.take_sent();
    h.run_steps(2);
    assert!(h.query_by_label(TITLE).is_none());
    assert!(fake.take_sent().is_empty(), "nothing starts by itself");
    assert!(!cueing(&fake, 0));
}

/// Review Focus 5.
#[test]
fn a_cart_pre_listen_asks_the_same_way() {
    let mut s = sounding();
    let page = s.cartwall.pages[0].id;
    fp_model::apply(
        &mut s,
        Command::AssignCartFile {
            page,
            index: 0,
            path: PathBuf::from("/carts/Station ID.wav"),
        },
    )
    .unwrap();
    let cart = s.cartwall.pages[0].carts[0].id;
    on_desk(&mut s, &[Holder::CartwallCue]);
    let (mut h, fake) = harness(s);
    fake.send(Command::CueCart(cart));
    fake.take_sent();
    h.run_steps(4);
    assert!(h.query_by_label(BODY_P2).is_some());
    h.get_by_label(ANYWAY).click();
    h.run_steps(2);
    assert_eq!(
        fake.take_sent(),
        vec![Command::ForceCue(CueStart::Cart(cart))]
    );
    assert_eq!(fake.state.load().cartwall.cue, Some(cart));
}

#[test]
fn a_pre_listen_stopped_by_a_start_says_why() {
    let (mut h, fake) = harness(shared());
    fake.send(Command::ToggleCue(fake.player(0)));
    fake.send(Command::Play(fake.player(1)));
    h.run_steps(2);
    assert!(h.query_by_label(STOPPED_P2).is_some());
    assert!(!cueing(&fake, 0));
}
