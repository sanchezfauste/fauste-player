#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Resizable layout spec RL7, RL8, RL11, RL12 and RL15–RL18: the main
//! window's panels, driven headlessly. The harness's window has an 8-point
//! margin, so the tests compare rectangles with each other.

mod support;

use std::path::PathBuf;

use egui::accesskit::Role;
use egui::{CursorIcon, Event, Key, Modifiers, PointerButton, Pos2, Rect, Vec2, pos2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use fp_app::ui::app::AppUi;
use fp_app::ui::cart_view::{BUTTON_HEIGHT, GAP, MIN_BUTTON_HEIGHT, MIN_BUTTON_WIDTH};
use fp_app::ui::controller::Controller;
use fp_app::ui::panel_layout::{MIN_PLAYER_HEIGHT, MIN_PLAYER_WIDTH, PANEL_GAP};
use fp_app::ui::theme;
use fp_model::layout::{CartwallLayout, DEFAULT_SIDE_SHARE, Dock, Panel};
use fp_model::{
    AppState, CartEdit, CartKind, Command, KeyChord, PlayerId, Shortcut, ShortcutAction,
    TrackAnalysis, apply,
};
use support::{Fake, harness, harness_sized, state};

/// The shown players' columns in the last frame, in display order.
fn columns(h: &Harness<'_, AppUi>) -> Vec<(PlayerId, Rect)> {
    h.state().player_rects()
}

fn column(h: &Harness<'_, AppUi>, player: PlayerId) -> Rect {
    columns(h)
        .into_iter()
        .find(|(p, _)| *p == player)
        .map(|(_, r)| r)
        .expect("the player is shown")
}

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 1.0
}

/// Whether the track table in `col` shows `title`: a node with that label
/// below the column's top block (the info row may show a title too).
fn table_shows(h: &Harness<'_, AppUi>, col: Rect, title: &str) -> bool {
    h.get_all_by_label(title).any(|n| {
        let r = n.rect();
        col.contains(r.center()) && r.center().y > col.top() + 300.0
    })
}

#[test]
fn rl11_the_default_layout_draws_todays_rectangles() {
    let (h, fake) = harness(state(2, 3));
    let (p1, p2) = (column(&h, fake.player(0)), column(&h, fake.player(1)));
    let cart = h.state().cartwall_rect().unwrap();
    // Two equal columns 8 points apart, as wide together as the cartwall,
    // which keeps its content height (two rows) under them.
    assert!(near(p1.width(), p2.width()), "{p1:?} {p2:?}");
    assert!(near(p2.left(), p1.right() + PANEL_GAP), "{p1:?} {p2:?}");
    assert!(near(p1.left(), cart.left()) && near(p2.right(), cart.right()));
    assert!(near(cart.height(), 116.0), "{cart:?}");
    // The row ends 8 points above the cartwall; a column is 2 points
    // shorter than its row.
    assert!(
        near(p1.bottom() + 2.0 + PANEL_GAP, cart.top()),
        "{p1:?} {cart:?}"
    );
}

#[test]
fn rl11_the_columns_follow_the_shares() {
    let mut s = state(2, 3);
    let p = s.player_ids();
    s.layout.player_shares = vec![(p[0], 3.0), (p[1], 1.0)];
    let (h, _) = harness(s);
    let (a, b) = (column(&h, p[0]), column(&h, p[1]));
    let spare = a.width() + b.width() - 2.0 * MIN_PLAYER_WIDTH;
    assert!(
        near(a.width(), MIN_PLAYER_WIDTH + spare * 0.75),
        "{a:?} {b:?}"
    );
}

#[test]
fn rl11_a_hidden_player_leaves_no_column() {
    let mut s = state(3, 3);
    let p = s.player_ids();
    s.layout.hidden_players = vec![p[1]];
    let (h, _) = harness_sized(s, vec2(1400.0, 700.0), |ui| ui);
    let shown: Vec<PlayerId> = columns(&h).iter().map(|(id, _)| *id).collect();
    assert_eq!(shown, vec![p[0], p[2]]);
}

#[test]
fn rl11_a_hidden_cartwall_gives_its_height_to_the_players() {
    let (h, fake) = harness(state(1, 3));
    let with_cartwall = column(&h, fake.player(0)).height();
    let mut s = state(1, 3);
    s.layout.cartwall.hidden = true;
    let (h, fake) = harness(s);
    assert_eq!(h.state().cartwall_rect(), None);
    let without = column(&h, fake.player(0)).height();
    assert!(
        near(without, with_cartwall + 116.0 + PANEL_GAP),
        "{with_cartwall} -> {without}"
    );
}

#[test]
fn rl11_with_every_player_hidden_the_cartwall_fills_the_middle_area() {
    let mut s = state(2, 3);
    s.layout.hidden_players = s.player_ids();
    let (h, _) = harness(s);
    assert!(columns(&h).is_empty());
    let cart = h.state().cartwall_rect().unwrap();
    assert!(cart.height() > 500.0, "{cart:?}");
}

#[test]
fn rl11_too_narrow_a_window_scrolls_the_players_at_their_minimum() {
    let (h, _) = harness(state(4, 3));
    let c = columns(&h);
    assert_eq!(c.len(), 4);
    assert!(
        c.iter().all(|(_, r)| near(r.width(), MIN_PLAYER_WIDTH)),
        "{c:?}"
    );
}

#[test]
fn rl11_a_column_keeps_its_id_when_a_player_before_it_is_hidden() {
    let (mut h, fake) = harness_sized(state(3, 40), vec2(1400.0, 700.0), |ui| ui);
    let (p2, p3) = (fake.player(1), fake.player(2));
    let col = column(&h, p3);
    assert!(table_shows(&h, col, "Song 1"));
    h.hover_at(pos2(col.center().x, col.bottom() - 60.0));
    h.step();
    h.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Line,
        delta: vec2(0.0, -10.0),
        phase: egui::TouchPhase::Move,
        modifiers: egui::Modifiers::NONE,
    });
    h.run_steps(20);
    assert!(!table_shows(&h, column(&h, p3), "Song 1"), "P3 scrolled");
    Controller::send(
        fake.as_ref(),
        Command::SetPanelHidden {
            panel: Panel::Player(p2),
            hidden: true,
        },
    );
    h.run_steps(3);
    // P3 is now second among the shown players; keyed by its id, its table
    // keeps its own scroll instead of taking P2's.
    assert!(!table_shows(&h, column(&h, p3), "Song 1"));
}

#[test]
fn rl8_three_rows_fit_a_player_at_its_least_height() {
    let mut s = state(1, 3);
    s.layout.cartwall.hidden = true;
    // The harness's margin, the top and status bars, the middle area's margin.
    let window = 2.0 * 8.0 + 34.0 + 24.0 + 2.0 * 8.0 + MIN_PLAYER_HEIGHT;
    let (h, fake) = harness_sized(s, vec2(1000.0, window), |ui| ui);
    let col = column(&h, fake.player(0));
    assert!(near(col.height(), MIN_PLAYER_HEIGHT - 2.0), "{col:?}");
    let third = h
        .get_all_by_label("Song 3")
        .map(|n| n.rect())
        .find(|r| r.center().y > col.top() + 300.0)
        .expect("the third row is built");
    // The footer is the column's last 24 points.
    assert!(
        third.bottom() <= col.bottom() - 24.0 + 0.5,
        "{third:?} in {col:?}"
    );
}

// --- splitters (RL12)

fn press(h: &mut Harness<'_, AppUi>, at: Pos2, button: PointerButton, pressed: bool) {
    h.event(Event::PointerButton {
        pos: at,
        button,
        pressed,
        modifiers: Modifiers::NONE,
    });
}

/// The middle of the splitter on the right of `left`'s column.
fn splitter_at(h: &Harness<'_, AppUi>, left: PlayerId) -> Pos2 {
    let r = column(h, left);
    pos2(r.right() + PANEL_GAP / 2.0, r.center().y)
}

/// Presses at `from` and moves `dx` in six frames; returns the first two
/// players' widths after each. The button stays down.
fn drag_by(h: &mut Harness<'_, AppUi>, fake: &Fake, from: Pos2, dx: f32) -> Vec<(f32, f32)> {
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    press(h, from, PointerButton::Primary, true);
    h.run_steps(1);
    let mut seen = Vec::new();
    for step in 1..=6 {
        h.event(Event::PointerMoved(pos2(
            from.x + dx * step as f32 / 6.0,
            from.y,
        )));
        h.run_steps(1);
        seen.push((
            column(h, fake.player(0)).width(),
            column(h, fake.player(1)).width(),
        ));
    }
    seen
}

fn click_at(h: &mut Harness<'_, AppUi>, at: Pos2) {
    press(h, at, PointerButton::Primary, true);
    h.run_steps(1);
    press(h, at, PointerButton::Primary, false);
    h.run_steps(1);
}

fn width_commands(fake: &Fake) -> Vec<Command> {
    fake.take_sent()
        .into_iter()
        .filter(|c| {
            matches!(
                c,
                Command::SetPlayerShares { .. } | Command::EqualPlayerWidths
            )
        })
        .collect()
}

fn cursor(h: &Harness<'_, AppUi>) -> CursorIcon {
    h.output().platform_output.cursor_icon
}

#[test]
fn rl12_dragging_a_splitter_resizes_both_neighbours_live() {
    let (mut h, fake) = harness(state(2, 3));
    let before = (
        column(&h, fake.player(0)).width(),
        column(&h, fake.player(1)).width(),
    );
    let from = splitter_at(&h, fake.player(0));
    let seen = drag_by(&mut h, &fake, from, 60.0);
    let last = *seen.last().unwrap();
    assert!(
        near(last.0, before.0 + 60.0) && near(last.1, before.1 - 60.0),
        "{before:?} -> {last:?}"
    );
    assert!(seen.windows(2).all(|w| w[1].0 >= w[0].0), "{seen:?}");
    assert!(
        width_commands(&fake).is_empty(),
        "nothing is sent while the button is down"
    );
}

#[test]
fn rl12_one_command_on_release_and_the_widths_stay() {
    let (mut h, fake) = harness(state(2, 3));
    let (p1, p2) = (fake.player(0), fake.player(1));
    let from = splitter_at(&h, p1);
    drag_by(&mut h, &fake, from, 60.0);
    let live = (column(&h, p1).width(), column(&h, p2).width());
    press(
        &mut h,
        pos2(from.x + 60.0, from.y),
        PointerButton::Primary,
        false,
    );
    h.run_steps(3);
    let sent = width_commands(&fake);
    assert_eq!(sent.len(), 1, "{sent:?}");
    let Command::SetPlayerShares { left, right } = &sent[0] else {
        panic!("{sent:?}")
    };
    assert_eq!((left.0, right.0), (p1, p2));
    let ratio = (live.0 - MIN_PLAYER_WIDTH) / (live.1 - MIN_PLAYER_WIDTH);
    assert!(
        (left.1 / right.1 - ratio).abs() < 0.01,
        "{left:?} {right:?}"
    );
    // Past the hold, the model's shares draw the same widths.
    h.run_steps(40);
    assert!(near(column(&h, p1).width(), live.0) && near(column(&h, p2).width(), live.1));
}

#[test]
fn rl12_a_click_on_a_splitter_sends_nothing() {
    let (mut h, fake) = harness(state(2, 3));
    let at = splitter_at(&h, fake.player(0));
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    click_at(&mut h, at);
    h.run_steps(30);
    assert!(width_commands(&fake).is_empty());
}

#[test]
fn rl12_a_double_click_sends_equal_widths() {
    let mut s = state(2, 3);
    let p = s.player_ids();
    s.layout.player_shares = vec![(p[0], 3.0), (p[1], 1.0)];
    let (mut h, fake) = harness(s);
    let at = splitter_at(&h, p[0]);
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    click_at(&mut h, at);
    click_at(&mut h, at);
    h.run_steps(2);
    assert_eq!(width_commands(&fake), vec![Command::EqualPlayerWidths]);
    assert!(near(column(&h, p[0]).width(), column(&h, p[1]).width()));
}

#[test]
fn rl12_the_minimums_hold() {
    let (mut h, fake) = harness(state(2, 3));
    let (p1, p2) = (fake.player(0), fake.player(1));
    let from = splitter_at(&h, p1);
    drag_by(&mut h, &fake, from, 400.0);
    assert!(near(column(&h, p2).width(), MIN_PLAYER_WIDTH));
    press(
        &mut h,
        pos2(from.x + 400.0, from.y),
        PointerButton::Primary,
        false,
    );
    h.run_steps(40);
    assert_eq!(width_commands(&fake).len(), 1);
    assert!(
        fake.rejected.lock().unwrap().is_empty(),
        "the model took it"
    );
    assert!(near(column(&h, p2).width(), MIN_PLAYER_WIDTH));
}

#[test]
fn rl12_the_pointer_shows_the_resize_cursor_over_a_splitter() {
    let (mut h, fake) = harness(state(2, 3));
    h.event(Event::PointerMoved(splitter_at(&h, fake.player(0))));
    h.run_steps(1);
    assert_eq!(cursor(&h), CursorIcon::ResizeColumn);
}

#[test]
fn rl12_a_locked_layout_has_inert_splitters() {
    let mut s = state(2, 3);
    let p = s.player_ids();
    s.layout.player_shares = vec![(p[0], 3.0), (p[1], 1.0)];
    s.layout.locked = true;
    let (mut h, fake) = harness(s);
    let before = column(&h, p[0]).width();
    let at = splitter_at(&h, p[0]);
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    assert_eq!(cursor(&h), CursorIcon::Default);
    click_at(&mut h, at);
    click_at(&mut h, at);
    drag_by(&mut h, &fake, at, 60.0);
    press(
        &mut h,
        pos2(at.x + 60.0, at.y),
        PointerButton::Primary,
        false,
    );
    h.run_steps(3);
    assert!(width_commands(&fake).is_empty());
    assert!(near(column(&h, p[0]).width(), before));
}

// --- the top bar (RL15)

fn hidden_commands(fake: &Fake) -> Vec<Command> {
    fake.take_sent()
        .into_iter()
        .filter(|c| matches!(c, Command::SetPanelHidden { .. }))
        .collect()
}

fn disabled(h: &Harness<'_, AppUi>, label: &str) -> bool {
    h.get_by_label(label).accesskit_node().is_disabled()
}

fn open_layout_menu(h: &mut Harness<'_, AppUi>) {
    h.get_by_label("Layout").click();
    h.run_steps(2);
}

/// Whether a red square is painted inside `area`.
fn red_dot_in(h: &Harness<'_, AppUi>, area: Rect) -> bool {
    fn walk(shape: &egui::Shape, area: Rect, found: &mut bool) {
        match shape {
            egui::Shape::Rect(r) => {
                *found |= r.fill == theme::ON_AIR_TEXT && area.contains_rect(r.rect);
            }
            egui::Shape::Vec(v) => v.iter().for_each(|s| walk(s, area, found)),
            _ => {}
        }
    }
    let mut found = false;
    for clipped in &h.output().shapes {
        walk(&clipped.shape, area, &mut found);
    }
    found
}

#[test]
fn rl15_a_tile_hides_and_shows_its_player() {
    let (mut h, fake) = harness(state(2, 3));
    let p2 = fake.player(1);
    h.get_by_label("Hide P2").click();
    h.run_steps(2);
    assert_eq!(
        hidden_commands(&fake),
        vec![Command::SetPanelHidden {
            panel: Panel::Player(p2),
            hidden: true,
        }]
    );
    assert_eq!(columns(&h).len(), 1);
    h.get_by_label("Show P2").click();
    h.run_steps(2);
    assert_eq!(columns(&h).len(), 2);
}

#[test]
fn rl15_the_carts_tile_hides_and_shows_the_cartwall() {
    let (mut h, _) = harness(state(2, 3));
    h.get_by_label("Hide Cartwall").click();
    h.run_steps(2);
    assert_eq!(h.state().cartwall_rect(), None);
    h.get_by_label("Show Cartwall").click();
    h.run_steps(2);
    assert!(h.state().cartwall_rect().is_some());
}

#[test]
fn rl15_the_last_lit_tile_is_disabled() {
    let mut s = state(1, 3);
    s.layout.cartwall.hidden = true;
    let (mut h, fake) = harness(s);
    assert!(disabled(&h, "Hide P1"));
    assert!(!disabled(&h, "Show Cartwall"));
    h.get_by_label("Hide P1").click();
    h.run_steps(2);
    assert!(hidden_commands(&fake).is_empty());
}

#[test]
fn rl15_a_hidden_panel_on_air_shows_a_red_dot() {
    let (mut h, fake) = harness(state(2, 3));
    let p1 = fake.player(0);
    Controller::send(fake.as_ref(), Command::Play(p1));
    h.get_by_label("Hide P1").click();
    h.run_steps(2);
    let tile = h.get_by_label("Show P1 (on air)").rect();
    assert!(red_dot_in(&h, tile));
    Controller::send(fake.as_ref(), Command::Stop(p1));
    h.run_steps(2);
    let tile = h.get_by_label("Show P1").rect();
    assert!(!red_dot_in(&h, tile));
}

#[test]
fn rl15_the_layout_menu_sends_its_commands() {
    let (mut h, fake) = harness(state(2, 3));
    open_layout_menu(&mut h);
    h.get_by_role_and_label(Role::RadioButton, "Cartwall on the left")
        .click();
    h.run_steps(2);
    open_layout_menu(&mut h);
    h.get_by_label("Equal player widths").click();
    h.run_steps(2);
    open_layout_menu(&mut h);
    h.get_by_label("Reset layout").click();
    h.run_steps(2);
    open_layout_menu(&mut h);
    h.get_by_role_and_label(Role::CheckBox, "Lock layout")
        .click();
    h.run_steps(2);
    let sent: Vec<Command> = fake
        .take_sent()
        .into_iter()
        .filter(|c| {
            matches!(
                c,
                Command::SetCartwallDock(_)
                    | Command::EqualPlayerWidths
                    | Command::ResetLayout
                    | Command::SetLayoutLocked(_)
            )
        })
        .collect();
    assert_eq!(
        sent,
        vec![
            Command::SetCartwallDock(Dock::Left),
            Command::EqualPlayerWidths,
            Command::ResetLayout,
            Command::SetLayoutLocked(true),
        ]
    );
}

#[test]
fn rl15_the_menu_checks_the_current_dock() {
    let mut s = state(2, 3);
    s.layout.cartwall.dock = Dock::Right;
    s.layout.cartwall.size = Some(0.3);
    let (mut h, _) = harness(s);
    open_layout_menu(&mut h);
    for (label, on) in [
        ("Cartwall at the bottom", false),
        ("Cartwall on the left", false),
        ("Cartwall on the right", true),
    ] {
        let want = if on {
            egui::accesskit::Toggled::True
        } else {
            egui::accesskit::Toggled::False
        };
        assert_eq!(
            h.get_by_role_and_label(Role::RadioButton, label)
                .accesskit_node()
                .toggled(),
            Some(want),
            "{label}"
        );
    }
}

#[test]
fn rl15_a_locked_layout_disables_the_tiles_and_the_menu() {
    let mut s = state(2, 3);
    s.layout.locked = true;
    let (mut h, fake) = harness(s);
    for tile in ["Hide P1", "Hide P2", "Hide Cartwall"] {
        assert!(disabled(&h, tile), "{tile}");
    }
    open_layout_menu(&mut h);
    for label in [
        "Cartwall at the bottom",
        "Cartwall on the left",
        "Cartwall on the right",
    ] {
        assert!(
            h.get_by_role_and_label(Role::RadioButton, label)
                .accesskit_node()
                .is_disabled(),
            "{label}"
        );
    }
    assert!(disabled(&h, "Equal player widths"));
    assert!(disabled(&h, "Reset layout"));
    let lock = h.get_by_role_and_label(Role::CheckBox, "Lock layout");
    assert!(!lock.accesskit_node().is_disabled());
    assert_eq!(
        lock.accesskit_node().toggled(),
        Some(egui::accesskit::Toggled::True)
    );
    lock.click();
    h.run_steps(2);
    assert!(fake.take_sent().contains(&Command::SetLayoutLocked(false)));
}

#[test]
fn rl7_the_cartwall_shortcut_does_nothing_while_it_is_hidden() {
    let shortcut = Shortcut {
        action: ShortcutAction::ToggleCartwall,
        chord: KeyChord {
            key: "C".into(),
            ctrl: false,
            alt: false,
            shift: false,
            command: false,
        },
    };
    let toggles = |fake: &Fake| {
        fake.take_sent()
            .iter()
            .filter(|c| matches!(c, Command::SetCartwallOpen(_)))
            .count()
    };
    let mut s = state(2, 3);
    s.config.shortcuts.push(shortcut.clone());
    let (mut h, fake) = harness(s);
    h.key_press(Key::C);
    h.run_steps(2);
    assert_eq!(toggles(&fake), 1, "shown, the shortcut collapses it");
    let mut s = state(2, 3);
    s.config.shortcuts.push(shortcut);
    s.layout.cartwall.hidden = true;
    let (mut h, fake) = harness(s);
    h.key_press(Key::C);
    h.run_steps(2);
    assert_eq!(toggles(&fake), 0);
}

#[test]
fn rl7_a_hidden_player_still_answers_its_key() {
    let mut s = state(2, 3);
    let p = s.player_ids();
    s.layout.hidden_players = vec![p[1]];
    let (mut h, fake) = harness(s);
    h.key_press(Key::Num2);
    h.run_steps(2);
    assert!(fake.take_sent().contains(&Command::Play(p[1])));
}

// --- maximise (RL16)

/// A point on the background of a player's header: between the status
/// label on the left and the CUE, mode and BP tiles on the right (a column
/// is 480 points wide in the default harness).
fn header_point(h: &Harness<'_, AppUi>, player: PlayerId) -> Pos2 {
    let r = column(h, player);
    pos2(r.left() + 200.0, r.top() + 21.0)
}

/// A point on the background of the cartwall's header.
fn cartwall_header_point(h: &Harness<'_, AppUi>) -> Pos2 {
    let r = h.state().cartwall_rect().unwrap();
    pos2(r.center().x, r.top() + 12.0)
}

fn right_click(h: &mut Harness<'_, AppUi>, at: Pos2) {
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    press(h, at, PointerButton::Secondary, true);
    h.run_steps(1);
    press(h, at, PointerButton::Secondary, false);
    h.run_steps(2);
}

fn layout_commands(fake: &Fake) -> Vec<Command> {
    fake.take_sent()
        .into_iter()
        .filter(|c| {
            matches!(
                c,
                Command::SetPanelHidden { .. }
                    | Command::SetPlayerShares { .. }
                    | Command::EqualPlayerWidths
                    | Command::SetCartwallDock(_)
                    | Command::SetCartwallSize(_)
                    | Command::SetLayoutLocked(_)
                    | Command::ResetLayout
            )
        })
        .collect()
}

fn maximise(h: &mut Harness<'_, AppUi>, at: Pos2) {
    right_click(h, at);
    h.get_by_label("Maximize").click();
    h.run_steps(2);
}

#[test]
fn rl16_maximise_and_restore_a_player() {
    let (mut h, fake) = harness(state(2, 3));
    let (p1, p2) = (fake.player(0), fake.player(1));
    let before = (column(&h, p1).width(), column(&h, p2).width());
    let at = header_point(&h, p1);
    maximise(&mut h, at);
    assert_eq!(h.state().maximised(), Some(Panel::Player(p1)));
    assert!(near(column(&h, p2).width(), MIN_PLAYER_WIDTH));
    assert!(near(
        column(&h, p1).width(),
        before.0 + before.1 - MIN_PLAYER_WIDTH
    ));
    assert!(
        layout_commands(&fake).is_empty(),
        "maximising sends nothing"
    );
    let at = header_point(&h, p1);
    right_click(&mut h, at);
    h.get_by_label("Restore").click();
    h.run_steps(2);
    assert_eq!(h.state().maximised(), None);
    assert!(near(column(&h, p1).width(), before.0) && near(column(&h, p2).width(), before.1));
}

#[test]
fn rl16_a_maximised_cartwall_leaves_the_players_their_least_height() {
    let (mut h, fake) = harness(state(2, 3));
    let at = cartwall_header_point(&h);
    maximise(&mut h, at);
    assert_eq!(h.state().maximised(), Some(Panel::Cartwall));
    assert!(near(
        column(&h, fake.player(0)).height(),
        MIN_PLAYER_HEIGHT - 2.0
    ));
}

#[test]
fn rl16_hiding_locking_docking_or_a_new_player_count_end_it() {
    for command in [
        Command::SetPanelHidden {
            panel: Panel::Cartwall,
            hidden: true,
        },
        Command::SetLayoutLocked(true),
        Command::SetCartwallDock(Dock::Left),
        Command::SetPlayerCount(3),
    ] {
        let (mut h, fake) = harness(state(2, 3));
        let at = header_point(&h, fake.player(0));
        maximise(&mut h, at);
        Controller::send(fake.as_ref(), command.clone());
        h.run_steps(2);
        assert_eq!(h.state().maximised(), None, "{command:?}");
    }
    let (mut h, fake) = harness(state(2, 3));
    let at = header_point(&h, fake.player(1));
    maximise(&mut h, at);
    h.get_by_label("Hide P2").click();
    h.run_steps(2);
    assert_eq!(
        h.state().maximised(),
        None,
        "the maximised panel was hidden"
    );
}

#[test]
fn rl16_a_splitter_drag_restores_the_earlier_sizes_first() {
    let (mut h, fake) = harness(state(2, 3));
    let (p1, p2) = (fake.player(0), fake.player(1));
    let before = column(&h, p1).width();
    let at = header_point(&h, p1);
    maximise(&mut h, at);
    let from = splitter_at(&h, p1);
    drag_by(&mut h, &fake, from, 40.0);
    assert_eq!(h.state().maximised(), None);
    assert!(
        near(column(&h, p1).width(), before + 40.0),
        "{:?}",
        column(&h, p1)
    );
    press(
        &mut h,
        pos2(from.x + 40.0, from.y),
        PointerButton::Primary,
        false,
    );
    h.run_steps(40);
    assert!(near(column(&h, p1).width(), before + 40.0));
    assert!(near(column(&h, p2).width(), before - 40.0));
}

#[test]
fn rl16_hide_from_the_header_menu() {
    let (mut h, fake) = harness(state(2, 3));
    let p2 = fake.player(1);
    let at = header_point(&h, p2);
    right_click(&mut h, at);
    h.get_by_label("Hide").click();
    h.run_steps(2);
    assert_eq!(
        layout_commands(&fake),
        vec![Command::SetPanelHidden {
            panel: Panel::Player(p2),
            hidden: true,
        }]
    );
}

#[test]
fn rl16_a_locked_layout_disables_the_header_menu() {
    let mut s = state(2, 3);
    s.layout.locked = true;
    let (mut h, fake) = harness(s);
    let at = header_point(&h, fake.player(0));
    right_click(&mut h, at);
    assert!(disabled(&h, "Maximize"));
    assert!(disabled(&h, "Hide"));
}

#[test]
fn rl16_with_one_panel_shown_maximise_is_disabled() {
    let mut s = state(1, 3);
    s.layout.cartwall.hidden = true;
    let (mut h, fake) = harness(s);
    let at = header_point(&h, fake.player(0));
    right_click(&mut h, at);
    assert!(disabled(&h, "Maximize"));
    assert!(disabled(&h, "Hide"), "the last shown panel");
}

// --- hidden players (RL17, RL18)

#[derive(Debug)]
struct Dropped(std::path::PathBuf);

impl egui::DroppedFile for Dropped {
    fn path(&self) -> &std::path::Path {
        &self.0
    }
    fn bytes(&self) -> Result<Vec<u8>, String> {
        std::fs::read(&self.0).map_err(|e| e.to_string())
    }
}

/// Drops `path` with no pointer position, as some systems do.
fn drop_file(h: &mut Harness<'static, AppUi>, path: &std::path::Path) {
    h.remove_cursor();
    h.step();
    h.input_mut()
        .dropped_files
        .push(std::sync::Arc::new(Dropped(path.to_path_buf())));
    // Dropped paths are read on a helper thread: give it time to answer.
    for _ in 0..40 {
        h.run_steps(1);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

/// Clicks the table row `title` in `player`'s column.
fn click_row(h: &mut Harness<'_, AppUi>, player: PlayerId, title: &str) {
    let col = column(h, player);
    h.get_all_by_label(title)
        .find(|n| {
            let r = n.rect();
            col.contains(r.center()) && r.center().y > col.top() + 300.0
        })
        .expect("the row is shown")
        .click();
    h.run_steps(2);
}

fn removals(fake: &Fake) -> usize {
    fake.take_sent()
        .iter()
        .filter(|c| matches!(c, Command::RemoveEntry(_)))
        .count()
}

#[test]
fn rl17_hiding_a_player_clears_its_selection() {
    let (mut h, fake) = harness(state(2, 3));
    let (p1, p2) = (fake.player(0), fake.player(1));
    click_row(&mut h, p2, "Song 2");
    h.get_by_label("Hide P2").click();
    h.run_steps(2);
    h.key_press(Key::Delete);
    h.run_steps(2);
    assert_eq!(
        removals(&fake),
        0,
        "Delete removes nothing the operator cannot see"
    );
    h.get_by_label("Show P2").click();
    h.run_steps(2);
    h.key_press(Key::Delete);
    h.run_steps(2);
    assert_eq!(removals(&fake), 0, "shown again, it has no selection");
    // Control: a shown player's selection still works.
    click_row(&mut h, p1, "Song 3");
    h.key_press(Key::Delete);
    h.run_steps(2);
    assert_eq!(removals(&fake), 1);
}

#[test]
fn rl18_a_drop_goes_to_the_first_shown_player_when_the_last_used_is_hidden() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("new.wav");
    std::fs::write(&file, b"x").unwrap();
    let mut s = state(2, 3);
    let p = s.player_ids();
    fp_model::apply(
        &mut s,
        Command::CreatePlaylistIn {
            name: "Second".into(),
            player: p[1],
        },
    )
    .unwrap();
    let second = s.player(p[1]).unwrap().playlist;
    let (mut h, fake) = harness(s);
    click_row(&mut h, p[0], "Song 2");
    h.get_by_label("Hide P1").click();
    h.run_steps(2);
    fake.take_sent();
    drop_file(&mut h, &file);
    let sent = fake.take_sent();
    assert!(
        sent.iter().any(|c| matches!(
            c,
            Command::InsertPaths { playlist, index: 0, .. } if *playlist == second
        )),
        "{sent:?}"
    );
}

#[test]
fn rl18_with_no_player_shown_a_drop_is_refused_with_a_notice() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("new.wav");
    std::fs::write(&file, b"x").unwrap();
    let mut s = state(2, 3);
    s.layout.hidden_players = s.player_ids();
    let (mut h, fake) = harness(s);
    drop_file(&mut h, &file);
    assert!(
        !fake
            .take_sent()
            .iter()
            .any(|c| matches!(c, Command::InsertPaths { .. }))
    );
    assert!(
        h.query_by_label("No player is shown: show one to drop files on it.")
            .is_some()
    );
}

// --- review fixes

/// Where each painted text that `wanted` accepts is visible: its bounds
/// cut to its clip rectangle (empty when it is scrolled out of sight).
fn visible_texts(h: &Harness<'_, AppUi>, wanted: impl Fn(&str) -> bool) -> Vec<(String, Rect)> {
    fn walk(
        shape: &egui::Shape,
        clip: Rect,
        wanted: &dyn Fn(&str) -> bool,
        out: &mut Vec<(String, Rect)>,
    ) {
        match shape {
            egui::Shape::Text(t) if wanted(t.galley.text()) => {
                let r = t.visual_bounding_rect().intersect(clip);
                if r.is_positive() {
                    out.push((t.galley.text().to_owned(), r));
                }
            }
            egui::Shape::Vec(v) => v.iter().for_each(|s| walk(s, clip, wanted, out)),
            _ => {}
        }
    }
    let mut out = Vec::new();
    for clipped in &h.output().shapes {
        walk(&clipped.shape, clipped.clip_rect, &wanted, &mut out);
    }
    out
}

#[test]
fn rl15_many_tiles_stay_clear_of_the_clock_and_the_buttons() {
    let (h, _) = harness_sized(state(32, 3), vec2(1280.0, 700.0), |ui| ui);
    let settings = h.get_by_label("Settings").rect();
    let about = h.get_by_label("About Fauste Player").rect();
    // The clock is right of Settings: everything from the leftmost button
    // to the window's right edge belongs to them.
    let right_block = settings.left().min(about.left());
    // The tiles' names painted in the top bar (the columns' headers show
    // `P<n>` too, lower down).
    let tiles: Vec<(String, Rect)> = visible_texts(&h, |s| {
        s == "Cartwall" || (s.starts_with('P') && s[1..].parse::<u32>().is_ok())
    })
    .into_iter()
    .filter(|(_, r)| r.center().y < settings.bottom())
    .collect();
    assert!(tiles.iter().any(|(s, _)| s == "P1"), "{tiles:?}");
    for (text, r) in &tiles {
        assert!(
            r.right() <= right_block,
            "{text} at {r:?} reaches {right_block}"
        );
    }
    let layout = h.get_by_label("Layout").rect();
    assert!(
        layout.left() >= 0.0 && layout.right() <= right_block,
        "{layout:?}"
    );
    assert!(
        tiles.iter().all(|(_, r)| r.right() <= layout.left()),
        "the layout button is right of the visible tiles"
    );
}

#[test]
fn rl12_splitters_and_header_backgrounds_are_no_tab_stops() {
    let (mut h, fake) = harness(state(2, 3));
    let (p1, p2) = (fake.player(0), fake.player(1));
    let inert = [
        egui::Id::new(("player-splitter", p1.0, p2.0)),
        egui::Id::new(("player-header", p1.0)),
        egui::Id::new(("player-header", p2.0)),
        egui::Id::new("cartwall-header"),
    ];
    let mut focused = Vec::new();
    for _ in 0..150 {
        h.key_press(Key::Tab);
        h.run_steps(1);
        if let Some(id) = h.ctx.memory(|m| m.focused()) {
            focused.push(id);
        }
    }
    assert!(!focused.is_empty(), "Tab moves the focus");
    for id in inert {
        assert!(!focused.contains(&id), "{id:?} took the focus");
    }
}

#[test]
fn rl16_only_a_primary_drag_on_a_splitter_ends_a_maximise() {
    let (mut h, fake) = harness(state(2, 3));
    let p1 = fake.player(0);
    let at = header_point(&h, p1);
    maximise(&mut h, at);
    let from = splitter_at(&h, p1);
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    press(&mut h, from, PointerButton::Secondary, true);
    h.run_steps(1);
    for step in 1..=6 {
        h.event(Event::PointerMoved(pos2(
            from.x + 10.0 * step as f32,
            from.y,
        )));
        h.run_steps(1);
    }
    press(
        &mut h,
        pos2(from.x + 60.0, from.y),
        PointerButton::Secondary,
        false,
    );
    h.run_steps(2);
    assert_eq!(h.state().maximised(), Some(Panel::Player(p1)));
    assert!(layout_commands(&fake).is_empty());
}

#[test]
fn rl16_a_double_click_on_a_splitter_ends_a_maximise() {
    let (mut h, fake) = harness(state(2, 3));
    let p1 = fake.player(0);
    let at = header_point(&h, p1);
    maximise(&mut h, at);
    // Past egui's triple-click window (twice the double-click delay), so
    // the menu's clicks do not make the double click a triple one.
    h.run_steps(40);
    let at = splitter_at(&h, p1);
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    click_at(&mut h, at);
    click_at(&mut h, at);
    h.run_steps(2);
    assert_eq!(width_commands(&fake), vec![Command::EqualPlayerWidths]);
    assert_eq!(h.state().maximised(), None);
}

#[test]
fn rl16_showing_a_panel_ends_a_maximise() {
    let mut s = state(3, 3);
    let p = s.player_ids();
    s.layout.hidden_players = vec![p[2]];
    let (mut h, _) = harness_sized(s, vec2(1400.0, 700.0), |ui| ui);
    let at = header_point(&h, p[0]);
    maximise(&mut h, at);
    assert_eq!(h.state().maximised(), Some(Panel::Player(p[0])));
    h.get_by_label("Show P3").click();
    h.run_steps(2);
    assert_eq!(h.state().maximised(), None);
}

#[test]
fn rl16_a_right_click_on_a_tile_opens_no_header_menu() {
    let (mut h, _) = harness(state(2, 3));
    let at = h.get_by_label("Hide P1").rect().center();
    right_click(&mut h, at);
    assert!(h.query_by_label("Maximize").is_none());
    assert!(h.query_by_label("Restore").is_none());
}

#[test]
fn rl12_a_refused_drag_snaps_back_after_the_hold() {
    let (mut h, fake) = harness(state(2, 3));
    let (p1, p2) = (fake.player(0), fake.player(1));
    let before = (column(&h, p1).width(), column(&h, p2).width());
    fake.hold.store(true, std::sync::atomic::Ordering::SeqCst);
    let from = splitter_at(&h, p1);
    drag_by(&mut h, &fake, from, 60.0);
    press(
        &mut h,
        pos2(from.x + 60.0, from.y),
        PointerButton::Primary,
        false,
    );
    h.run_steps(3);
    assert_eq!(width_commands(&fake).len(), 1);
    assert!(near(column(&h, p1).width(), before.0 + 60.0), "held");
    h.run_steps(40);
    assert!(
        near(column(&h, p1).width(), before.0) && near(column(&h, p2).width(), before.1),
        "the model's widths come back"
    );
}

#[test]
fn rl8_an_overflowing_row_keeps_three_rows_above_its_scroll_bar() {
    let (mut h, fake) = harness(state(4, 3));
    let at = cartwall_header_point(&h);
    maximise(&mut h, at);
    assert_eq!(h.state().maximised(), Some(Panel::Cartwall));
    let col = column(&h, fake.player(0));
    assert!(near(col.height(), MIN_PLAYER_HEIGHT - 2.0), "{col:?}");
}

// --- the cartwall's dock (RL6, RL8, RL11)

fn docked(mut s: AppState, dock: Dock, size: Option<f32>) -> AppState {
    s.layout.cartwall = CartwallLayout {
        dock,
        size,
        hidden: false,
    };
    s
}

/// A window wide enough for two players beside a side cartwall at its
/// default share (the middle area is 1968 × 810).
fn wide(s: AppState) -> (Harness<'static, AppUi>, std::sync::Arc<Fake>) {
    harness_sized(s, vec2(2000.0, 900.0), |ui| ui)
}

/// `s` with "Station ID" (10 s, a Jingle) in the first cart of the first
/// page.
fn with_station_id(mut s: AppState) -> AppState {
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
    apply(
        &mut s,
        Command::ApplyAnalysis {
            track,
            analysis: Box::new(TrackAnalysis {
                duration_secs: 10.0,
                ..TrackAnalysis::default()
            }),
        },
    )
    .unwrap();
    let edit = CartEdit {
        name: "Station ID".into(),
        kind: CartKind::Jingle,
        looped: false,
        exclusive: false,
        color: None,
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

/// `s` with a first page of `rows` × `cols`.
fn with_grid(mut s: AppState, rows: u16, cols: u16) -> AppState {
    let page = s.cartwall.pages[0].id;
    apply(&mut s, Command::ResizeCartPage { page, rows, cols }).unwrap();
    s
}

/// Eight columns of the smallest button with their gaps.
fn eight_columns() -> f32 {
    8.0 * MIN_BUTTON_WIDTH + 7.0 * GAP
}

#[test]
fn rl6_a_left_dock_draws_the_cartwall_beside_the_players_at_full_height() {
    let (h, fake) = wide(docked(state(2, 3), Dock::Left, Some(0.3)));
    let cart = h.state().cartwall_rect().unwrap();
    let (p1, p2) = (column(&h, fake.player(0)), column(&h, fake.player(1)));
    assert!(near(p1.left(), cart.right() + PANEL_GAP), "{cart:?} {p1:?}");
    assert!(near(cart.top(), p1.top()), "{cart:?} {p1:?}");
    // A column is 2 points shorter than its row.
    assert!(near(cart.bottom(), p1.bottom() + 2.0), "{cart:?} {p1:?}");
    let area = p2.right() - cart.left();
    assert!(near(cart.width(), 0.3 * area), "{cart:?} in {area}");
}

#[test]
fn rl6_a_right_dock_draws_the_cartwall_right_of_the_players() {
    let (h, fake) = wide(docked(state(2, 3), Dock::Right, Some(0.3)));
    let cart = h.state().cartwall_rect().unwrap();
    let (p1, p2) = (column(&h, fake.player(0)), column(&h, fake.player(1)));
    assert!(near(cart.left(), p2.right() + PANEL_GAP), "{cart:?} {p2:?}");
    assert!(near(cart.top(), p1.top()), "{cart:?} {p1:?}");
    let area = cart.right() - p1.left();
    assert!(near(cart.width(), 0.3 * area), "{cart:?} in {area}");
}

#[test]
fn rl6_the_layout_menu_moves_the_cartwall() {
    let (mut h, fake) = wide(state(2, 3));
    let (p1, p2) = (fake.player(0), fake.player(1));
    for (label, beside) in [
        ("Cartwall on the left", Dock::Left),
        ("Cartwall on the right", Dock::Right),
        ("Cartwall at the bottom", Dock::Bottom),
    ] {
        open_layout_menu(&mut h);
        h.get_by_role_and_label(Role::RadioButton, label).click();
        h.run_steps(2);
        let cart = h.state().cartwall_rect().unwrap();
        let placed = match beside {
            Dock::Left => cart.right() < column(&h, p1).left(),
            Dock::Right => cart.left() > column(&h, p2).right(),
            Dock::Bottom => cart.top() > column(&h, p1).bottom(),
        };
        assert!(placed, "{label}: {cart:?}");
    }
}

#[test]
fn rl11_a_side_dock_in_too_narrow_a_window_scrolls_the_players() {
    for dock in [Dock::Left, Dock::Right] {
        let (h, _) = harness(docked(state(2, 3), dock, Some(0.3)));
        let cart = h.state().cartwall_rect().unwrap();
        assert!(near(cart.width(), eight_columns()), "{dock:?} {cart:?}");
        let c = columns(&h);
        assert!(
            c.iter().all(|(_, r)| near(r.width(), MIN_PLAYER_WIDTH)),
            "{dock:?} {c:?}"
        );
        if dock == Dock::Left {
            assert!(
                c[0].1.left() >= cart.right(),
                "nothing overlaps: {c:?} {cart:?}"
            );
        }
    }
}

#[test]
fn rl8_a_side_cartwall_is_never_narrower_than_its_grid() {
    let (h, _) = wide(docked(state(2, 3), Dock::Left, Some(0.05)));
    let cart = h.state().cartwall_rect().unwrap();
    assert!(near(cart.width(), eight_columns()), "{cart:?}");
    let eighth = h.get_all_by_label("Cart 8, empty").next().unwrap().rect();
    assert!(near(eighth.width(), MIN_BUTTON_WIDTH), "{eighth:?}");
    assert!(
        eighth.right() <= cart.right() + 0.5,
        "{eighth:?} in {cart:?}"
    );
}

#[test]
fn rl8_a_one_column_page_keeps_the_header_whole() {
    let s = docked(with_grid(state(2, 3), 2, 1), Dock::Left, Some(0.05));
    let (h, _) = wide(s);
    let cart = h.state().cartwall_rect().unwrap();
    assert!(
        cart.width() < eight_columns(),
        "the header sets it: {cart:?}"
    );
    let title = h.get_by_label("CARTWALL").rect();
    let tab = h.get_by_label("Carts 1").rect();
    let stop = h.get_by_label("Stop all").rect();
    assert!(title.left() >= cart.left() - 0.5, "{title:?} in {cart:?}");
    assert!(title.right() <= tab.left() + 0.5, "{title:?} {tab:?}");
    assert!(tab.right() <= stop.left() + 0.5, "{tab:?} {stop:?}");
    assert!(stop.right() <= cart.right() + 0.5, "{stop:?} in {cart:?}");
}

#[test]
fn rl8_the_least_width_does_not_change_while_carts_play() {
    let s = docked(
        with_grid(with_station_id(state(2, 3)), 2, 1),
        Dock::Left,
        Some(0.05),
    );
    let (mut h, fake) = wide(s);
    let before = h.state().cartwall_rect().unwrap();
    let cart = fake.state.load().cartwall.pages[0].carts[0].id;
    Controller::send(fake.as_ref(), Command::FireCart(cart));
    h.run_steps(2);
    assert!(h.query_by_label("Stop all (1)").is_some(), "a cart plays");
    assert_eq!(h.state().cartwall_rect(), Some(before));
}

#[test]
fn rl16_a_maximised_player_shrinks_a_side_cartwall_to_its_least_width() {
    let (mut h, fake) = wide(docked(state(2, 3), Dock::Right, Some(0.3)));
    let at = header_point(&h, fake.player(0));
    maximise(&mut h, at);
    let cart = h.state().cartwall_rect().unwrap();
    assert!(near(cart.width(), eight_columns()), "{cart:?}");
    assert!(near(column(&h, fake.player(1)).width(), MIN_PLAYER_WIDTH));
}

// --- the cartwall splitter (RL12)

/// A window tall enough for the bottom cartwall to grow (its middle area is
/// 968 × 810: the players keep 432, so the cartwall can reach 370).
fn tall(s: AppState) -> (Harness<'static, AppUi>, std::sync::Arc<Fake>) {
    harness_sized(s, vec2(1000.0, 900.0), |ui| ui)
}

/// The middle of the gap between the players and the cartwall.
fn cartwall_splitter_at(h: &Harness<'_, AppUi>) -> Pos2 {
    let cart = h.state().cartwall_rect().unwrap();
    let first = columns(h)[0].1;
    if cart.top() > first.bottom() {
        pos2(cart.center().x, cart.top() - PANEL_GAP / 2.0)
    } else if cart.right() < first.left() {
        pos2(cart.right() + PANEL_GAP / 2.0, cart.center().y)
    } else {
        pos2(cart.left() - PANEL_GAP / 2.0, cart.center().y)
    }
}

/// Presses at `from` and moves by `by` in six frames; the button stays
/// down.
fn drag_along(h: &mut Harness<'_, AppUi>, from: Pos2, by: Vec2) {
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    press(h, from, PointerButton::Primary, true);
    h.run_steps(1);
    for step in 1..=6 {
        h.event(Event::PointerMoved(from + by * (step as f32 / 6.0)));
        h.run_steps(1);
    }
}

fn size_commands(fake: &Fake) -> Vec<Command> {
    fake.take_sent()
        .into_iter()
        .filter(|c| matches!(c, Command::SetCartwallSize(_)))
        .collect()
}

fn cart_rect(h: &Harness<'_, AppUi>) -> Rect {
    h.state().cartwall_rect().unwrap()
}

#[test]
fn rl12_dragging_the_bottom_splitter_resizes_the_cartwall_live() {
    let (mut h, fake) = tall(state(2, 3));
    let p1 = fake.player(0);
    let (cart, col) = (cart_rect(&h), column(&h, p1));
    let from = cartwall_splitter_at(&h);
    drag_along(&mut h, from, vec2(0.0, -60.0));
    let (cart2, col2) = (cart_rect(&h), column(&h, p1));
    assert!(
        near(cart2.height(), cart.height() + 60.0),
        "{cart:?} -> {cart2:?}"
    );
    assert!(
        near(col2.height(), col.height() - 60.0),
        "{col:?} -> {col2:?}"
    );
    assert!(
        size_commands(&fake).is_empty(),
        "nothing is sent while the button is down"
    );
}

#[test]
fn rl12_one_cartwall_size_on_release_and_the_size_stays() {
    let (mut h, fake) = tall(state(2, 3));
    let from = cartwall_splitter_at(&h);
    drag_along(&mut h, from, vec2(0.0, -60.0));
    let live = cart_rect(&h);
    press(
        &mut h,
        from + vec2(0.0, -60.0),
        PointerButton::Primary,
        false,
    );
    h.run_steps(3);
    let sent = size_commands(&fake);
    assert_eq!(sent.len(), 1, "{sent:?}");
    let Command::SetCartwallSize(Some(share)) = sent[0].clone() else {
        panic!("{sent:?}")
    };
    let area = live.bottom() - column(&h, fake.player(0)).top();
    assert!(
        (share * area - live.height()).abs() < 1.0,
        "{share} of {area}: {live:?}"
    );
    // Past the hold, the model's share draws the same size.
    h.run_steps(40);
    assert!(
        near(cart_rect(&h).height(), live.height()),
        "{:?}",
        cart_rect(&h)
    );
}

#[test]
fn rl12_a_click_on_the_cartwall_splitter_sends_nothing() {
    let (mut h, fake) = tall(state(2, 3));
    let at = cartwall_splitter_at(&h);
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    click_at(&mut h, at);
    h.run_steps(30);
    assert!(size_commands(&fake).is_empty());
}

#[test]
fn rl12_a_double_click_fits_the_content_again_at_the_bottom() {
    let mut s = state(2, 3);
    s.layout.cartwall.size = Some(0.4);
    let (mut h, fake) = tall(s);
    let at = cartwall_splitter_at(&h);
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    click_at(&mut h, at);
    click_at(&mut h, at);
    h.run_steps(2);
    assert_eq!(size_commands(&fake), vec![Command::SetCartwallSize(None)]);
    // Two rows of 40 under a 24-point header, 6-point gaps.
    assert!(near(cart_rect(&h).height(), 116.0), "{:?}", cart_rect(&h));
}

#[test]
fn rl12_a_double_click_on_a_side_gives_the_default_share() {
    let (mut h, fake) = wide(docked(state(2, 3), Dock::Left, Some(0.5)));
    let at = cartwall_splitter_at(&h);
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    click_at(&mut h, at);
    click_at(&mut h, at);
    h.run_steps(2);
    assert_eq!(
        size_commands(&fake),
        vec![Command::SetCartwallSize(Some(DEFAULT_SIDE_SHARE))]
    );
}

#[test]
fn rl12_the_cartwall_splitter_shows_the_resize_cursor() {
    let (mut h, _) = tall(state(2, 3));
    h.event(Event::PointerMoved(cartwall_splitter_at(&h)));
    h.run_steps(1);
    assert_eq!(cursor(&h), CursorIcon::ResizeRow);
    let (mut h, _) = wide(docked(state(2, 3), Dock::Right, Some(0.3)));
    h.event(Event::PointerMoved(cartwall_splitter_at(&h)));
    h.run_steps(1);
    assert_eq!(cursor(&h), CursorIcon::ResizeColumn);
}

#[test]
fn rl12_dragging_a_side_splitter_resizes_the_cartwall() {
    for (dock, dx) in [(Dock::Left, 50.0), (Dock::Right, -50.0)] {
        let (mut h, fake) = wide(docked(state(2, 3), dock, Some(0.3)));
        let before = cart_rect(&h).width();
        let from = cartwall_splitter_at(&h);
        drag_along(&mut h, from, vec2(dx, 0.0));
        assert!(near(cart_rect(&h).width(), before + 50.0), "{dock:?}");
        press(&mut h, from + vec2(dx, 0.0), PointerButton::Primary, false);
        h.run_steps(3);
        assert_eq!(size_commands(&fake).len(), 1, "{dock:?}");
    }
}

#[test]
fn rl12_the_cartwall_splitter_keeps_the_players_least_height() {
    let (mut h, fake) = tall(state(2, 3));
    let from = cartwall_splitter_at(&h);
    drag_along(&mut h, from, vec2(0.0, -800.0));
    let p1 = fake.player(0);
    assert!(near(column(&h, p1).height(), MIN_PLAYER_HEIGHT - 2.0));
    press(
        &mut h,
        from + vec2(0.0, -800.0),
        PointerButton::Primary,
        false,
    );
    h.run_steps(40);
    assert_eq!(size_commands(&fake).len(), 1);
    assert!(
        fake.rejected.lock().unwrap().is_empty(),
        "the model took it"
    );
    assert!(near(column(&h, p1).height(), MIN_PLAYER_HEIGHT - 2.0));
}

#[test]
fn rl12_a_locked_layout_has_an_inert_cartwall_splitter() {
    let mut s = state(2, 3);
    s.layout.locked = true;
    let (mut h, fake) = tall(s);
    let before = cart_rect(&h);
    let at = cartwall_splitter_at(&h);
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    assert_eq!(cursor(&h), CursorIcon::Default);
    click_at(&mut h, at);
    click_at(&mut h, at);
    drag_along(&mut h, at, vec2(0.0, -60.0));
    press(&mut h, at + vec2(0.0, -60.0), PointerButton::Primary, false);
    h.run_steps(3);
    assert!(size_commands(&fake).is_empty());
    assert_eq!(cart_rect(&h), before);
}

#[test]
fn rl12_a_collapsed_cartwall_has_an_inert_splitter() {
    for dock in [Dock::Bottom, Dock::Left] {
        let mut s = docked(state(2, 3), dock, Some(0.4));
        s.cartwall.open = false;
        let (mut h, fake) = wide(s);
        let before = cart_rect(&h);
        let at = cartwall_splitter_at(&h);
        h.event(Event::PointerMoved(at));
        h.run_steps(1);
        assert_eq!(cursor(&h), CursorIcon::Default, "{dock:?}");
        drag_along(&mut h, at, vec2(40.0, -40.0));
        press(
            &mut h,
            at + vec2(40.0, -40.0),
            PointerButton::Primary,
            false,
        );
        h.run_steps(3);
        assert!(size_commands(&fake).is_empty(), "{dock:?}");
        assert_eq!(cart_rect(&h), before, "{dock:?}");
    }
}

#[test]
fn rl12_locking_during_a_cartwall_drag_ends_it_without_a_command() {
    let (mut h, fake) = tall(state(2, 3));
    let before = cart_rect(&h);
    let from = cartwall_splitter_at(&h);
    drag_along(&mut h, from, vec2(0.0, -60.0));
    Controller::send(fake.as_ref(), Command::SetLayoutLocked(true));
    h.run_steps(1);
    press(
        &mut h,
        from + vec2(0.0, -60.0),
        PointerButton::Primary,
        false,
    );
    h.run_steps(3);
    assert!(size_commands(&fake).is_empty());
    assert!(
        near(cart_rect(&h).height(), before.height()),
        "the model's size"
    );
}

#[test]
fn rl16_a_cartwall_drag_ends_a_maximise_first() {
    let (mut h, fake) = tall(state(2, 3));
    let before = cart_rect(&h).height();
    let at = header_point(&h, fake.player(0));
    maximise(&mut h, at);
    assert!(cart_rect(&h).height() < before, "at its least height");
    let from = cartwall_splitter_at(&h);
    drag_along(&mut h, from, vec2(0.0, -20.0));
    assert_eq!(h.state().maximised(), None);
    assert!(
        near(cart_rect(&h).height(), before + 20.0),
        "{:?}",
        cart_rect(&h)
    );
}

#[test]
fn rl16_a_double_click_on_the_cartwall_splitter_ends_a_maximise() {
    let (mut h, fake) = tall(state(2, 3));
    let at = header_point(&h, fake.player(0));
    maximise(&mut h, at);
    // Past egui's triple-click window, as for the player splitter.
    h.run_steps(40);
    let at = cartwall_splitter_at(&h);
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    click_at(&mut h, at);
    click_at(&mut h, at);
    h.run_steps(2);
    assert_eq!(size_commands(&fake), vec![Command::SetCartwallSize(None)]);
    assert_eq!(h.state().maximised(), None);
}

#[test]
fn rl16_only_a_primary_drag_on_the_cartwall_splitter_ends_a_maximise() {
    let (mut h, fake) = tall(state(2, 3));
    let p1 = fake.player(0);
    let at = header_point(&h, p1);
    maximise(&mut h, at);
    let from = cartwall_splitter_at(&h);
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    press(&mut h, from, PointerButton::Secondary, true);
    h.run_steps(1);
    for step in 1..=6 {
        h.event(Event::PointerMoved(pos2(
            from.x,
            from.y - 10.0 * step as f32,
        )));
        h.run_steps(1);
    }
    press(
        &mut h,
        pos2(from.x, from.y - 60.0),
        PointerButton::Secondary,
        false,
    );
    h.run_steps(2);
    assert_eq!(h.state().maximised(), Some(Panel::Player(p1)));
    assert!(layout_commands(&fake).is_empty());
}

// --- buttons scale (RL13)

/// Cart `n` of the shown page, empty.
fn empty_cart(h: &Harness<'_, AppUi>, n: usize) -> Rect {
    h.get_all_by_label(&format!("Cart {n}, empty"))
        .next()
        .expect("the cart is built")
        .rect()
}

/// Where the text `text` is laid out: the galley's rectangle.
fn text_rect(h: &Harness<'_, AppUi>, text: &str) -> Option<Rect> {
    fn walk(shape: &egui::Shape, text: &str, found: &mut Option<Rect>) {
        match shape {
            egui::Shape::Text(t) if t.galley.text() == text => {
                *found = Some(Rect::from_min_size(t.pos, t.galley.size()));
            }
            egui::Shape::Vec(v) => v.iter().for_each(|s| walk(s, text, found)),
            _ => {}
        }
    }
    let mut found = None;
    for clipped in &h.output().shapes {
        walk(&clipped.shape, text, &mut found);
    }
    found
}

#[test]
fn rl13_fitting_the_content_keeps_todays_buttons() {
    let (h, _) = tall(state(2, 3));
    // Today's: the header's item spacing (4 points) leaves the two rows 82
    // of the panel's 116, so each is 38 points tall, never more than 40.
    let r = empty_cart(&h, 2);
    assert!(
        (r.height() - (BUTTON_HEIGHT - 2.0)).abs() < 0.01,
        "{r:?} in {:?}",
        cart_rect(&h)
    );
}

#[test]
fn rl13_buttons_grow_with_a_bottom_share() {
    let mut s = state(2, 3);
    s.layout.cartwall.size = Some(0.5);
    let (h, _) = tall(s);
    let cart = cart_rect(&h);
    let (first, last) = (empty_cart(&h, 2), empty_cart(&h, 16));
    assert!(first.height() > BUTTON_HEIGHT, "{first:?}");
    assert!(near(first.height(), last.height()));
    assert!(
        near(last.bottom(), cart.bottom()),
        "the grid fills the panel: {last:?} in {cart:?}"
    );
}

#[test]
fn rl13_buttons_grow_with_a_side_dock() {
    let (h, _) = wide(docked(state(2, 3), Dock::Left, Some(0.3)));
    let cart = cart_rect(&h);
    let last = empty_cart(&h, 16);
    assert!(last.height() > BUTTON_HEIGHT, "{last:?}");
    assert!(near(last.bottom(), cart.bottom()), "{last:?} in {cart:?}");
    assert!(near(last.right(), cart.right()), "{last:?} in {cart:?}");
}

#[test]
fn rl13_at_the_least_size_the_grid_scrolls() {
    // Eight rows of 28 need 24 + 6 + 8 × 28 + 7 × 6 = 296 points; the middle
    // area of a 360-point window has 270.
    let s = docked(with_grid(state(2, 3), 8, 8), Dock::Left, Some(0.3));
    let (h, _) = harness_sized(s, vec2(2000.0, 360.0), |ui| ui);
    let cart = cart_rect(&h);
    assert!(near(empty_cart(&h, 1).height(), MIN_BUTTON_HEIGHT));
    let last = empty_cart(&h, 64);
    assert!(
        last.bottom() > cart.bottom(),
        "the last row is below: {last:?} {cart:?}"
    );
}

#[test]
fn rl13_a_tall_buttons_text_keeps_its_size_and_is_centred() {
    let mut s = with_station_id(state(2, 3));
    s.layout.cartwall.size = Some(0.5);
    let (h, _) = tall(s);
    let button = h.get_all_by_label("Station ID").next().unwrap().rect();
    assert!(button.height() > 2.0 * BUTTON_HEIGHT, "{button:?}");
    // The text block of a 40-point button, inside its 4-point margins,
    // centred in the taller button.
    let name = text_rect(&h, "Station ID").expect("the name is painted");
    assert!(
        near(name.top(), button.center().y - BUTTON_HEIGHT / 2.0 + 4.0),
        "{name:?} in {button:?}"
    );
    let kind = text_rect(&h, "Jingle").expect("the kind is painted");
    assert!(
        near(kind.bottom(), button.center().y + BUTTON_HEIGHT / 2.0 - 4.0),
        "{kind:?} in {button:?}"
    );
}

// --- collapsed (RL14)

fn collapsed(dock: Dock, size: Option<f32>) -> AppState {
    let mut s = docked(state(2, 3), dock, size);
    s.cartwall.open = false;
    s
}

#[test]
fn rl14_collapsed_on_a_side_is_a_narrow_strip_with_its_caret() {
    for dock in [Dock::Left, Dock::Right] {
        let (h, fake) = wide(collapsed(dock, Some(0.4)));
        let cart = cart_rect(&h);
        let p1 = column(&h, fake.player(0));
        assert!(cart.width() < MIN_BUTTON_WIDTH, "{dock:?} {cart:?}");
        assert!(near(cart.top(), p1.top()), "{dock:?} {cart:?}");
        assert!(near(cart.bottom(), p1.bottom() + 2.0), "{dock:?} {cart:?}");
        let caret = h.get_by_label("CARTWALL").rect();
        assert!(cart.contains_rect(caret), "{dock:?}: {caret:?} in {cart:?}");
        assert!(
            h.query_by_label("Stop all").is_none(),
            "{dock:?}: only the caret"
        );
        assert!(h.query_by_label("Cart 1, empty").is_none(), "{dock:?}");
    }
}

#[test]
fn rl14_the_strip_is_as_wide_as_the_collapsed_header_is_high() {
    let (h, _) = wide(collapsed(Dock::Bottom, None));
    let header = cart_rect(&h).height();
    let (g, _) = wide(collapsed(Dock::Bottom, Some(0.5)));
    assert!(
        near(cart_rect(&g).height(), header),
        "a share changes nothing"
    );
    let (s, _) = wide(collapsed(Dock::Left, None));
    assert!(near(cart_rect(&s).width(), header), "{:?}", cart_rect(&s));
}

#[test]
fn rl14_the_caret_expands_it_to_its_share() {
    let (mut h, fake) = wide(collapsed(Dock::Right, Some(0.4)));
    h.get_by_label("CARTWALL").click();
    h.run_steps(3);
    assert!(fake.take_sent().contains(&Command::SetCartwallOpen(true)));
    let cart = cart_rect(&h);
    let area = cart.right() - column(&h, fake.player(0)).left();
    assert!(near(cart.width(), 0.4 * area), "{cart:?} in {area}");
    assert!(
        h.query_by_label("Cart 1, empty").is_some(),
        "the grid is back"
    );
}

#[test]
fn rl14_the_strip_shows_a_red_dot_while_a_cart_plays() {
    let mut s = docked(with_station_id(state(2, 3)), Dock::Left, None);
    s.cartwall.open = false;
    let (mut h, fake) = wide(s);
    let strip = cart_rect(&h);
    assert!(!red_dot_in(&h, strip));
    let cart = fake.state.load().cartwall.pages[0].carts[0].id;
    Controller::send(fake.as_ref(), Command::FireCart(cart));
    h.run_steps(2);
    assert!(red_dot_in(&h, strip));
    Controller::send(fake.as_ref(), Command::StopCart(cart));
    h.run_steps(2);
    assert!(!red_dot_in(&h, strip));
}

#[test]
fn rl14_the_strips_menu_hides_it_and_cannot_maximise_it() {
    let (mut h, fake) = wide(collapsed(Dock::Left, None));
    let strip = cart_rect(&h);
    right_click(&mut h, strip.center());
    assert!(disabled(&h, "Maximize"));
    h.get_by_label("Hide").click();
    h.run_steps(2);
    assert_eq!(
        layout_commands(&fake),
        vec![Command::SetPanelHidden {
            panel: Panel::Cartwall,
            hidden: true,
        }]
    );
}

#[test]
fn rl14_a_collapsed_bottom_cartwall_cannot_be_maximised() {
    let mut s = state(2, 3);
    s.cartwall.open = false;
    let (mut h, _) = tall(s);
    let at = cartwall_header_point(&h);
    right_click(&mut h, at);
    assert!(disabled(&h, "Maximize"));
}

#[test]
fn rl14_collapsing_a_maximised_cartwall_ends_the_maximise() {
    let (mut h, fake) = tall(state(2, 3));
    let at = cartwall_header_point(&h);
    maximise(&mut h, at);
    assert_eq!(h.state().maximised(), Some(Panel::Cartwall));
    Controller::send(fake.as_ref(), Command::SetCartwallOpen(false));
    h.run_steps(2);
    assert_eq!(h.state().maximised(), None);
    Controller::send(fake.as_ref(), Command::SetCartwallOpen(true));
    h.run_steps(2);
    assert_eq!(
        h.state().maximised(),
        None,
        "expanding does not bring it back"
    );
}
