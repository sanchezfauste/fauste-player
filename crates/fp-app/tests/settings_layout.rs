#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Feedback 2 spec O3: the Settings window's size and grid.

mod support;

use std::sync::Arc;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::ui::app::AppUi;
use fp_backends::{AudioBackend, OfflineBackend};
use fp_model::{AppState, PlayerRoutes, Route};
use support::{harness_sized, state};

const SCREEN: egui::Vec2 = egui::vec2(1400.0, 900.0);
/// The window size the spec asks for (`settings::WINDOW_SIZE`).
const WINDOW: egui::Vec2 = egui::vec2(900.0, 640.0);
const SECTIONS: [&str; 9] = [
    "Audio outputs",
    "Players",
    "Meters",
    "Analysis",
    "Playlists",
    "Cartwall",
    "Keyboard shortcuts",
    "MIDI",
    "Remote",
];
const LONG_DEVICE: &str = "USB Audio Interface With A Remarkably Long Product Name";

/// Two players; player 1 plays on a device with a long name.
fn routed() -> AppState {
    let mut s = state(2, 1);
    let route = |device: &str| Route {
        backend: "offline".into(),
        device: device.into(),
        first_channel: 0,
    };
    s.config.outputs.backend = Some("offline".into());
    s.config.outputs.routes = vec![PlayerRoutes {
        player: s.players[0].id,
        main: Some(route(LONG_DEVICE)),
        cue: Some(route("speakers")),
    }];
    s
}

fn backends() -> Vec<Arc<dyn AudioBackend>> {
    let backend = OfflineBackend::new();
    backend.add_device(LONG_DEVICE, 2);
    backend.add_device("speakers", 2);
    vec![Arc::new(backend)]
}

fn open_settings(size: egui::Vec2, s: AppState) -> Harness<'static, AppUi> {
    let backends = backends();
    let (mut h, _) = harness_sized(s, size, move |ui| ui.with_backends(backends));
    h.get_by_label("Settings").click();
    h.run_steps(3);
    h
}

fn open(h: &mut Harness<'static, AppUi>, section: &str) {
    h.get_by_role_and_label(Role::Button, section).click();
    h.run_steps(3);
}

/// Devices are listed by a helper thread.
fn wait_for_devices(h: &mut Harness<'static, AppUi>) {
    for _ in 0..200 {
        h.run_steps(1);
        if h.query_all_by_label("Test Main").next().is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    h.run_steps(3);
}

fn close_rect(h: &Harness<'static, AppUi>) -> egui::Rect {
    h.get_by_label("Close").rect()
}

#[test]
fn every_section_has_the_same_window() {
    let mut h = open_settings(SCREEN, routed());
    let mut seen = Vec::new();
    for section in SECTIONS {
        open(&mut h, section);
        if section == "Audio outputs" {
            wait_for_devices(&mut h);
        }
        seen.push((section, close_rect(&h)));
    }
    // The window is centred: Close sits 16 px inside its right edge.
    let right = (SCREEN.x + WINDOW.x) / 2.0 - 16.0;
    for (section, rect) in seen {
        assert!(
            (rect.right() - right).abs() < 1.5,
            "{section}: Close ends at {} instead of {right}",
            rect.right()
        );
        let bottom = (SCREEN.y + WINDOW.y) / 2.0;
        assert!(
            rect.bottom() < bottom && rect.bottom() > bottom - 52.0,
            "{section}: Close is not in the footer"
        );
    }
}

#[test]
fn the_footer_spans_the_window() {
    let mut h = open_settings(SCREEN, routed());
    open(&mut h, "Audio outputs");
    wait_for_devices(&mut h);
    let left = (SCREEN.x - WINDOW.x) / 2.0 + 16.0;
    let notice = h.get_by_label("Changes apply at once").rect();
    assert!(
        (notice.left() - left).abs() < 1.5,
        "notice at {}",
        notice.left()
    );
    let right = (SCREEN.x + WINDOW.x) / 2.0 - 16.0;
    assert!((close_rect(&h).right() - right).abs() < 1.5);
}

#[test]
fn a_small_screen_keeps_one_size_inside_it() {
    let small = egui::vec2(700.0, 500.0);
    let mut h = open_settings(small, routed());
    let mut rights = Vec::new();
    for section in SECTIONS {
        open(&mut h, section);
        if section == "Audio outputs" {
            wait_for_devices(&mut h);
        }
        rights.push(close_rect(&h).right());
    }
    assert!(
        rights.iter().all(|r| (r - rights[0]).abs() < 0.5),
        "{rights:?}"
    );
    assert!(rights[0] <= small.x - 16.0);
}

fn left_of(h: &Harness<'static, AppUi>, role: Role, label: &str) -> f32 {
    h.get_by_role_and_label(role, label).rect().left()
}

#[test]
fn rows_share_one_label_column() {
    let mut h = open_settings(SCREEN, state(1, 1));
    open(&mut h, "Analysis");
    let short = left_of(&h, Role::Slider, "Trim margin");
    let long = left_of(&h, Role::Slider, "Minimum length for mix and outro markers");
    assert!((short - long).abs() < 0.5, "{short} vs {long}");
}

#[test]
fn outputs_test_buttons_line_up_in_columns() {
    let mut h = open_settings(SCREEN, routed());
    open(&mut h, "Audio outputs");
    wait_for_devices(&mut h);
    for label in ["Test Main", "Test Cue"] {
        let lefts: Vec<f32> = h.get_all_by_label(label).map(|n| n.rect().left()).collect();
        // Player 1 (routed), player 2 (not routed) and the cartwall.
        assert_eq!(lefts.len(), 3, "{label}");
        assert!(
            lefts.iter().all(|x| (x - lefts[0]).abs() < 0.5),
            "{label}: {lefts:?}"
        );
    }
    let main = h.get_all_by_label("Test Main").next().unwrap().rect();
    let cue = h.get_all_by_label("Test Cue").next().unwrap().rect();
    assert!((main.left() - cue.left()).abs() < 0.5);
    assert!((main.right() - cue.right()).abs() < 0.5);
}

#[test]
fn a_long_device_name_stays_inside_its_box() {
    let mut h = open_settings(SCREEN, routed());
    open(&mut h, "Audio outputs");
    wait_for_devices(&mut h);
    let test = h.get_all_by_label("Test Main").next().unwrap().rect();
    let tag = h.get_all_by_label("MAIN").next().unwrap().rect();
    let boxes: Vec<egui::Rect> = h
        .get_all_by_role(Role::ComboBox)
        .map(|n| n.rect())
        .filter(|r| (r.top() - test.top()).abs() < 20.0)
        .collect();
    assert!(!boxes.is_empty());

    for r in &boxes {
        assert!(r.right() <= test.left() + 0.5, "{r:?} vs test {test:?}");
        assert!(r.left() >= tag.right() - 0.5, "{r:?} vs tag {tag:?}");
    }
}

#[test]
fn controls_start_at_the_same_x_in_every_section() {
    let mut h = open_settings(SCREEN, state(1, 1));
    open(&mut h, "Players");
    let players = left_of(&h, Role::Slider, "Fade time");

    let mut with_cart = state(1, 0);
    let page = with_cart.cartwall.pages[0].id;
    fp_model::apply(
        &mut with_cart,
        fp_model::Command::AssignCartFile {
            page,
            index: 0,
            path: std::path::PathBuf::from("/carts/id.wav"),
        },
    )
    .unwrap();
    let mut h = open_settings(SCREEN, with_cart);
    open(&mut h, "Cartwall");
    let cartwall = left_of(&h, Role::TextInput, "Cart name");

    let cell = Arc::new(arc_swap::ArcSwap::from_pointee(
        fp_remote::RemoteStatus::default(),
    ));
    let (mut h, _) = harness_sized(state(1, 0), SCREEN, move |ui| ui.with_remote_status(cell));
    h.get_by_label("Settings").click();
    h.run_steps(3);
    open(&mut h, "Remote");
    let remote = h
        .get_all_by_role_and_label(Role::TextInput, "Address")
        .next()
        .unwrap()
        .rect()
        .left();

    assert!((players - cartwall).abs() < 0.5, "{players} vs {cartwall}");
    assert!((players - remote).abs() < 0.5, "{players} vs {remote}");
}
