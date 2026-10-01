#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O6: closing while audio is on air asks first.

mod support;

use egui::{Key, ViewportCommand, ViewportEvent, ViewportId};
use egui_kittest::kittest::Queryable;
use fp_model::{Command, PlayingCart};
use support::{harness, state};

const GUARD: &str = "Audio is on air";

fn request_close(h: &mut egui_kittest::Harness<'static, fp_app::ui::app::AppUi>) {
    h.input_mut()
        .viewports
        .entry(ViewportId::ROOT)
        .or_default()
        .events
        .push(ViewportEvent::Close);
    // The event lives for one step only; kittest keeps that step's output.
    h.step();
}

/// A close request, then enough frames for the dialog to be drawn.
fn open_guard(h: &mut egui_kittest::Harness<'static, fp_app::ui::app::AppUi>) {
    request_close(h);
    h.run_steps(2);
}

fn sent_viewport_command(
    h: &egui_kittest::Harness<'static, fp_app::ui::app::AppUi>,
    wanted: &ViewportCommand,
) -> bool {
    h.output()
        .viewport_output
        .get(&ViewportId::ROOT)
        .is_some_and(|v| v.commands.contains(wanted))
}

fn cancelled(h: &egui_kittest::Harness<'static, fp_app::ui::app::AppUi>) -> bool {
    sent_viewport_command(h, &ViewportCommand::CancelClose)
}

fn playing(players: usize) -> fp_model::AppState {
    let mut s = state(players, 3);
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    s
}

#[test]
fn closing_with_nothing_on_air_shows_no_dialog() {
    let (mut h, fake) = harness(state(1, 3));
    request_close(&mut h);
    assert!(!cancelled(&h));
    h.run_steps(2);
    assert!(h.query_by_label(GUARD).is_none());
    assert!(fake.take_sent().is_empty());
}

#[test]
fn closing_while_playing_cancels_and_asks() {
    let (mut h, fake) = harness(playing(1));
    request_close(&mut h);
    assert!(cancelled(&h));
    assert!(sent_viewport_command(
        &h,
        &ViewportCommand::Minimized(false)
    ));
    assert!(sent_viewport_command(&h, &ViewportCommand::Focus));
    h.run_steps(2);
    assert!(h.query_by_label(GUARD).is_some());
    assert!(h.query_by_label_contains("P1 — Song 1").is_some());
    assert!(fake.take_sent().is_empty());
}

#[test]
fn cancel_and_escape_keep_everything_playing() {
    let (mut h, fake) = harness(playing(1));
    open_guard(&mut h);
    h.get_by_label("Cancel").click();
    h.run_steps(2);
    assert!(h.query_by_label(GUARD).is_none());
    open_guard(&mut h);
    h.key_press(Key::Escape);
    h.run_steps(2);
    assert!(h.query_by_label(GUARD).is_none());
    assert!(fake.take_sent().is_empty());
}

#[test]
fn stop_and_close_stops_players_and_carts_then_closes() {
    let mut s = playing(2);
    let cart = s.cartwall.pages[0].carts[0].id;
    s.cartwall.playing.push(PlayingCart {
        cart,
        looped: false,
    });
    let p = s.players[0].id;
    let (mut h, fake) = harness(s);
    open_guard(&mut h);
    h.get_by_label("Stop and close").click();
    h.run_steps(1);
    assert_eq!(
        fake.take_sent(),
        vec![Command::Stop(p), Command::StopAllCarts]
    );
    assert!(sent_viewport_command(&h, &ViewportCommand::Close));
    // Once confirmed, the close that follows goes through, even if audio
    // is on air again by then (only the confirmation lets it pass).
    fake.state.store(std::sync::Arc::new(playing(2)));
    request_close(&mut h);
    assert!(!cancelled(&h));
}

#[test]
fn a_second_close_request_keeps_one_dialog_and_stops_nothing() {
    let (mut h, fake) = harness(playing(1));
    open_guard(&mut h);
    open_guard(&mut h);
    assert_eq!(h.query_all_by_label(GUARD).count(), 1);
    assert!(fake.take_sent().is_empty());
}

#[test]
fn the_guard_shows_over_settings() {
    let (mut h, _) = harness(playing(1));
    h.get_by_label_contains("Settings").click();
    h.run_steps(2);
    open_guard(&mut h);
    assert!(h.query_by_label(GUARD).is_some());
}

#[test]
fn shortcuts_do_nothing_while_the_guard_is_open() {
    let (mut h, fake) = harness(playing(1));
    open_guard(&mut h);
    h.key_press(Key::Space);
    h.key_press(Key::Num1);
    h.run_steps(2);
    assert!(fake.take_sent().is_empty());
}

#[test]
fn cancel_works_over_settings() {
    let (mut h, fake) = harness(playing(1));
    h.get_by_label_contains("Settings").click();
    h.run_steps(2);
    open_guard(&mut h);
    h.get_by_label("Cancel").click();
    h.run_steps(2);
    assert!(h.query_by_label(GUARD).is_none());
    assert!(fake.take_sent().is_empty());
}

#[test]
fn the_shell_guards_the_close_in_logic_alone() {
    // eframe calls only `logic` while the window is minimized.
    let fake = support::Fake::new(playing(1));
    let app = fp_app::ui::app::AppUi::new(
        fake.clone(),
        fp_app::i18n::I18n::new(Some("en-US")),
        fp_app::services::MediaCache::default(),
    );
    let mut shell = fp_app::ui::shell::Shell::new(app);
    let ctx = egui::Context::default();
    let mut input = egui::RawInput::default();
    input
        .viewports
        .entry(ViewportId::ROOT)
        .or_default()
        .events
        .push(ViewportEvent::Close);
    let output = ctx.run_logic(&input, |ctx| shell.logic(ctx));
    let commands = &output.viewport_commands[&ViewportId::ROOT];
    assert!(commands.contains(&ViewportCommand::CancelClose));
    assert!(commands.contains(&ViewportCommand::Focus));
}

#[test]
fn a_degraded_shell_does_not_guard_the_close() {
    // With the recovery banner up the dialog cannot be drawn, and a
    // cancelled close with no dialog would trap the operator.
    let fake = support::Fake::new(playing(1));
    let app = fp_app::ui::app::AppUi::new(
        fake.clone(),
        fp_app::i18n::I18n::new(Some("en-US")),
        fp_app::services::MediaCache::default(),
    );
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1000.0, 700.0))
        .with_step_dt(0.02)
        .build_ui_state(
            |ui, shell: &mut fp_app::ui::shell::Shell| shell.ui(ui),
            fp_app::ui::shell::Shell::new(app),
        );
    h.run_steps(1);
    h.state_mut().app_mut().fail_next_frame();
    h.run_steps(2);
    assert!(h.state().degraded());
    let mut shell = h.into_state();
    let ctx = egui::Context::default();
    let mut input = egui::RawInput::default();
    input
        .viewports
        .entry(ViewportId::ROOT)
        .or_default()
        .events
        .push(ViewportEvent::Close);
    let output = ctx.run_logic(&input, |ctx| shell.logic(ctx));
    let cancelled = output
        .viewport_commands
        .get(&ViewportId::ROOT)
        .is_some_and(|c| c.contains(&ViewportCommand::CancelClose));
    assert!(!cancelled);
}

#[test]
fn a_player_with_no_title_shows_its_number_alone() {
    let mut s = state(1, 3);
    s.players[0].transport = fp_model::Transport::Paused;
    s.players[0].current = None;
    let (mut h, _) = harness(s);
    open_guard(&mut h);
    assert!(h.query_by_label("P1").is_some());
}

#[test]
fn carts_show_their_number_on_the_page() {
    let mut s = state(1, 3);
    let page = &mut s.cartwall.pages[0];
    page.carts[0].name = "Jingle".to_owned();
    let named = page.carts[0].id;
    // The third cart has no name and no track.
    let empty = page.carts[2].id;
    for cart in [named, empty] {
        s.cartwall.playing.push(PlayingCart {
            cart,
            looped: false,
        });
    }
    let (mut h, _) = harness(s);
    open_guard(&mut h);
    assert!(h.query_by_label("Cart 1 — Jingle").is_some());
    assert!(h.query_by_label("Cart 3").is_some());
}

#[test]
fn a_click_on_the_backdrop_cancels() {
    let (mut h, fake) = harness(playing(1));
    open_guard(&mut h);
    let corner = egui::pos2(4.0, 4.0);
    h.event(egui::Event::PointerMoved(corner));
    for pressed in [true, false] {
        h.event(egui::Event::PointerButton {
            pos: corner,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        });
        h.step();
    }
    h.run_steps(2);
    assert!(h.query_by_label(GUARD).is_none());
    assert!(fake.take_sent().is_empty());
}

#[test]
fn escape_cancels_even_with_a_text_field_focused() {
    let fake = support::Fake::new(playing(1));
    let app = fp_app::ui::app::AppUi::new(
        fake.clone(),
        fp_app::i18n::I18n::new(Some("en-US")),
        fp_app::services::MediaCache::default(),
    );
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1000.0, 700.0))
        .with_step_dt(0.02)
        .build_ui_state(
            |ui, (app, text): &mut (fp_app::ui::app::AppUi, String)| {
                app.guard_close(ui.ctx());
                app.ui(ui);
                // Drawn last, so it still holds the focus while the
                // interface reads the keys.
                ui.add(egui::TextEdit::singleline(text).id_salt("editor"));
            },
            (app, String::new()),
        );
    h.run_steps(2);
    h.get_by_role(egui::accesskit::Role::TextInput).focus();
    h.run_steps(1);
    assert!(h.ctx.text_edit_focused());
    h.input_mut()
        .viewports
        .entry(ViewportId::ROOT)
        .or_default()
        .events
        .push(ViewportEvent::Close);
    // Esc in the frame that opens the guard, while the field still holds
    // the focus (the modal takes it from the next frame on).
    h.key_press(Key::Escape);
    h.run_steps(3);
    assert!(h.query_by_label(GUARD).is_none());
    assert!(fake.take_sent().is_empty());
}
