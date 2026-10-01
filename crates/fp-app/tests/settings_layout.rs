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
