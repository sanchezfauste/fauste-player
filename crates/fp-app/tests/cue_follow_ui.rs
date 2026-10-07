#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Operator feedback 4, Q6: with a CUE running on a player, a click or a
//! double-click on a row of a playlist that player shows moves the CUE to
//! that row's entry (Q6.1, Q6.2), with the CUE window open and on any of
//! the player's tabs (Q6.3). Real pointer events, as an operator's mouse.

mod support;

use std::path::PathBuf;

use egui::{Event, Modifiers, PointerButton, Pos2, Rect, Vec2, pos2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_app::ui::controller::Controller;
use fp_model::{AppState, Command, PlayerRoutes, PlaylistId, Route};
use support::{harness_sized, state};

const PAUSE: &str = "Pause CUE";
const RESUME: &str = "Resume CUE";
const STOP: &str = "Stop CUE";
const LOAD_NEXT: &str = "Set as next";
const CLOSE: &str = "Close and stop CUE";
const CUE_WAVE: &str = "CUE waveform: click to seek";
const MENU_CUE: &str = "Pre-listen on CUE";

/// How the CUE was started.
#[derive(Debug, Clone, Copy)]
enum Start {
    /// The player's CUE: it plays the next entry (Song 1).
    Button,
    /// "Pre-listen on CUE" in a row's menu: it plays Song 2.
    RowMenu,
}

#[derive(Debug, Clone, Copy)]
enum Gesture {
    Click,
    DoubleClick,
}

#[derive(Debug, Clone, Copy)]
enum Tab {
    /// The player shows "Main", where the cued entry is.
    Same,
    /// The player shows "Other": the cued entry is not on screen.
    Other,
}

#[derive(Debug, Clone, Copy)]
struct Case {
    size: Vec2,
    players: usize,
    /// The player whose CUE runs and whose table is clicked (0-based).
    player: usize,
    start: Start,
    /// The CUE was paused from its window before the gesture.
    paused: bool,
    tab: Tab,
    gesture: Gesture,
}

enum Outcome {
    Moved,
    /// The row is under the CUE window: there is nothing to click.
    Covered,
    Failed(String),
}

fn route(device: &str) -> Route {
    Route {
        backend: "cpal".to_owned(),
        device: device.to_owned(),
        first_channel: 0,
    }
}

/// `players` players showing "Main" (Song 1–4) and a playlist "Other"
/// (Other 1–3). Every track is 180 s long, and every player has a Cue
/// output apart from its Main output (so a CUE can run whether or not
/// the `player_has_cue` guard is in the code).
fn setup(players: usize) -> (AppState, PlaylistId) {
    let mut s = state(players, 4);
    fp_model::apply(
        &mut s,
        Command::CreatePlaylistFromPaths {
            name: "Other".to_owned(),
            paths: (1..=3)
                .map(|n| PathBuf::from(format!("/music/Other {n}.mp3")))
                .collect(),
        },
    )
    .unwrap();
    let other = s.playlists.iter().find(|l| l.name == "Other").unwrap().id;
    for t in s.library.iter_mut() {
        t.duration_secs = 180.0;
    }
    s.config.outputs.routes = s
        .players
        .iter()
        .map(|p| PlayerRoutes {
            player: p.id,
            main: Some(route("desk")),
            cue: Some(route("phones")),
        })
        .collect();
    (s, other)
}

/// Moves the pointer to `at` and presses and releases `button` there.
fn press(h: &mut Harness<'_, AppUi>, at: Pos2, button: PointerButton) {
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    for pressed in [true, false] {
        h.event(Event::PointerButton {
            pos: at,
            button,
            pressed,
            modifiers: Modifiers::NONE,
        });
    }
    h.step();
}

/// The rect of the row titled `title` in the `column`-th table (from the
/// left) that shows it. Only table rows carry these labels in these
/// cases. A playlist shown by one player alone has one such row.
fn row(h: &Harness<'_, AppUi>, title: &str, column: usize) -> Rect {
    let mut rects: Vec<Rect> = h.get_all_by_label(title).map(|n| n.rect()).collect();
    rects.sort_by(|a, b| a.min.x.total_cmp(&b.min.x));
    rects[column.min(rects.len() - 1)]
}

/// The CUE window's area: its controls, grown by the window's frame.
fn cue_window(h: &Harness<'_, AppUi>) -> Option<Rect> {
    [PAUSE, RESUME, STOP, LOAD_NEXT, CLOSE, CUE_WAVE]
        .into_iter()
        .filter_map(|label| h.query_by_label(label))
        .map(|n| n.rect())
        .reduce(|a, b| a.union(b))
        .map(|r| r.expand(13.0))
}

/// A point of `row` that the CUE window does not cover, if any.
fn visible_point(row: Rect, window: Option<Rect>) -> Option<Pos2> {
    let y = row.center().y;
    [row.center().x, row.left() + 2.0, row.right() - 2.0]
        .into_iter()
        .map(|x| pos2(x, y))
        .find(|p| window.is_none_or(|w| !w.contains(*p)))
}

/// Lets more than twice egui's double-click delay (0.3 s) pass, as an
/// operator's hand does between a click elsewhere (Pause, a menu item)
/// and the gesture on a row. Without it, egui counts the second click of
/// a double-click as a third click (its triple-click check spans 0.6 s
/// and compares the position with the last click only), and the
/// double-click is not seen.
fn settle(h: &mut Harness<'_, AppUi>) {
    h.run_steps(35);
}

fn run(case: Case) -> Outcome {
    let (s, other) = setup(case.players);
    let (mut h, fake) = harness_sized(s, case.size, |ui| ui);
    let p = fake.player(case.player);
    let main = fake.entries();
    let cued = match case.start {
        Start::Button => {
            fake.send(Command::ToggleCue(p));
            h.run_steps(2);
            main[0]
        }
        Start::RowMenu => {
            let r = row(&h, "Song 2", case.player);
            press(&mut h, r.center(), PointerButton::Secondary);
            h.run_steps(2);
            h.get_by_label(MENU_CUE).click();
            h.run_steps(2);
            main[1]
        }
    };
    if fake.state.load().players[case.player].cue.map(|c| c.entry) != Some(cued) {
        return Outcome::Failed(format!("{case:?}: the CUE did not start on {cued:?}"));
    }
    if case.paused {
        h.get_by_label(PAUSE).click();
        h.run_steps(2);
    }
    let (title, target) = match case.tab {
        Tab::Same => ("Song 3", main[2]),
        Tab::Other => {
            fake.send(Command::ShowPlaylist(p, other));
            h.run_steps(2);
            let entry = fake.state.load().playlists.get(other).unwrap().entries[1].id;
            ("Other 2", entry)
        }
    };
    if h.query_by_label(STOP).is_none() {
        return Outcome::Failed(format!("{case:?}: the CUE window is not open"));
    }
    let Some(at) = visible_point(row(&h, title, case.player), cue_window(&h)) else {
        return Outcome::Covered;
    };
    settle(&mut h);
    fake.take_sent();
    press(&mut h, at, PointerButton::Primary);
    if matches!(case.gesture, Gesture::DoubleClick) {
        press(&mut h, at, PointerButton::Primary);
    }
    h.run_steps(2);
    let sent = fake.take_sent();
    if matches!(case.gesture, Gesture::DoubleClick) && !sent.contains(&Command::SetNext(p, target))
    {
        return Outcome::Failed(format!(
            "{case:?}: the double-click did not set the next; sent {sent:?}"
        ));
    }
    let now = fake.state.load().players[case.player].cue.map(|c| c.entry);
    if now == Some(target) {
        Outcome::Moved
    } else {
        Outcome::Failed(format!(
            "{case:?}: the CUE is on {now:?}, not {target:?}; sent {sent:?}"
        ))
    }
}

fn expect_moved(case: Case) {
    match run(case) {
        Outcome::Moved => {}
        Outcome::Covered => panic!("{case:?}: the row is under the CUE window"),
        Outcome::Failed(why) => panic!("{why}"),
    }
}

fn simple(gesture: Gesture) -> Case {
    Case {
        size: vec2(1000.0, 700.0),
        players: 1,
        player: 0,
        start: Start::Button,
        paused: false,
        tab: Tab::Same,
        gesture,
    }
}

#[test]
fn q6_1_a_click_on_a_row_moves_the_cue_with_its_window_open() {
    expect_moved(simple(Gesture::Click));
}

#[test]
fn q6_2_a_double_click_on_a_row_moves_the_cue_with_its_window_open() {
    expect_moved(simple(Gesture::DoubleClick));
}

#[test]
fn q6_3_the_cue_follows_in_every_layout_tab_and_state() {
    // (window size, players, the player whose CUE runs)
    let layouts = [
        (vec2(1000.0, 700.0), 1, 0),
        (vec2(1920.0, 1080.0), 4, 0),
        (vec2(1920.0, 1080.0), 4, 3),
    ];
    let mut failed = Vec::new();
    let mut covered = Vec::new();
    for (size, players, player) in layouts {
        for start in [Start::Button, Start::RowMenu] {
            for paused in [false, true] {
                for tab in [Tab::Same, Tab::Other] {
                    for gesture in [Gesture::Click, Gesture::DoubleClick] {
                        let case = Case {
                            size,
                            players,
                            player,
                            start,
                            paused,
                            tab,
                            gesture,
                        };
                        match run(case) {
                            Outcome::Moved => {}
                            Outcome::Covered => covered.push(case),
                            Outcome::Failed(why) => failed.push(why),
                        }
                    }
                }
            }
        }
    }
    // A row under the CUE window cannot be clicked; Task 2 looks at these.
    eprintln!("rows under the CUE window: {covered:#?}");
    assert!(
        failed.is_empty(),
        "{} cases did not move the CUE:\n{}",
        failed.len(),
        failed.join("\n")
    );
}

/// H2: a press that moves a few pixels before its release. egui counts it
/// as a click below its drag threshold (6 px). Above it, the row starts a
/// drag and becomes the selection, but the CUE does not follow.
#[test]
fn h2_a_click_with_a_small_movement_still_moves_the_cue() {
    let (s, _) = setup(1);
    let (mut h, fake) = harness_sized(s, vec2(1000.0, 700.0), |ui| ui);
    let p = fake.player(0);
    let e = fake.entries();
    fake.send(Command::ToggleCue(p));
    h.run_steps(2);
    let at = visible_point(row(&h, "Song 3", 0), cue_window(&h)).unwrap();
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    h.event(Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    h.step();
    let to = pos2(at.x + 3.0, at.y + 1.0);
    h.event(Event::PointerMoved(to));
    h.step();
    h.event(Event::PointerButton {
        pos: to,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(2);
    assert_eq!(
        fake.state.load().players[0].cue.map(|c| c.entry),
        Some(e[2])
    );
}

/// H3: a click held as long as a slow operator's (0.5 s) before release.
#[test]
fn h3_a_slow_click_still_moves_the_cue() {
    let (s, _) = setup(1);
    let (mut h, fake) = harness_sized(s, vec2(1000.0, 700.0), |ui| ui);
    let p = fake.player(0);
    let e = fake.entries();
    fake.send(Command::ToggleCue(p));
    h.run_steps(2);
    let at = visible_point(row(&h, "Song 3", 0), cue_window(&h)).unwrap();
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    h.event(Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(25);
    h.event(Event::PointerButton {
        pos: at,
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(2);
    assert_eq!(
        fake.state.load().players[0].cue.map(|c| c.entry),
        Some(e[2])
    );
}

/// H4: the window was used last (it holds the focus) before the row is
/// clicked: Pause, then Resume, then the row.
#[test]
fn h4_a_click_after_using_the_cue_window_moves_the_cue() {
    let (s, _) = setup(1);
    let (mut h, fake) = harness_sized(s, vec2(1000.0, 700.0), |ui| ui);
    let p = fake.player(0);
    let e = fake.entries();
    fake.send(Command::ToggleCue(p));
    h.run_steps(2);
    h.get_by_label(PAUSE).click();
    h.run_steps(2);
    h.get_by_label(RESUME).click();
    h.run_steps(2);
    let at = visible_point(row(&h, "Song 3", 0), cue_window(&h)).unwrap();
    press(&mut h, at, PointerButton::Primary);
    h.run_steps(2);
    assert_eq!(
        fake.state.load().players[0].cue.map(|c| c.entry),
        Some(e[2])
    );
}

/// H5: the current entry is on air (the player plays Song 1) and the CUE
/// pre-listens the next (Song 2); then a click on Song 3.
#[test]
fn h5_a_click_while_the_player_is_on_air_moves_the_cue() {
    let (s, _) = setup(1);
    let (mut h, fake) = harness_sized(s, vec2(1000.0, 700.0), |ui| ui);
    let p = fake.player(0);
    let e = fake.entries();
    fake.send(Command::Play(p));
    fake.send(Command::ToggleCue(p));
    h.run_steps(2);
    assert_eq!(
        fake.state.load().players[0].cue.map(|c| c.entry),
        Some(e[1])
    );
    let at = visible_point(row(&h, "Song 3", 0), cue_window(&h)).unwrap();
    press(&mut h, at, PointerButton::Primary);
    h.run_steps(2);
    assert_eq!(
        fake.state.load().players[0].cue.map(|c| c.entry),
        Some(e[2])
    );
}
