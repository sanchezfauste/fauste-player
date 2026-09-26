#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! A panic while drawing never takes the application down (spec §8.5).

mod support;

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::i18n::I18n;
use fp_app::services::MediaCache;
use fp_app::ui::app::AppUi;
use fp_app::ui::shell::Shell;
use support::{Fake, state};

#[test]
fn a_panicking_frame_shows_the_banner_and_keeps_running() {
    let fake = Fake::new(state(1, 2));
    let app = AppUi::new(
        fake.clone(),
        I18n::new(Some("en-US")),
        MediaCache::default(),
    );
    let mut h = Harness::builder()
        .with_size(egui::vec2(1000.0, 700.0))
        .with_step_dt(0.02)
        .build_ui_state(|ui, shell: &mut Shell| shell.ui(ui), Shell::new(app));
    h.run_steps(3);
    assert!(h.query_by_label("Play").is_some());
    h.state_mut().app_mut().fail_next_frame();
    h.run_steps(3);
    assert!(
        h.query_by_label("The interface hit an error. Audio is not affected.")
            .is_some()
    );
    assert!(h.state().degraded());
    h.get_by_label("Restart interface").click();
    h.run_steps(3);
    assert!(!h.state().degraded());
    assert!(
        h.query_by_label("Play").is_some(),
        "the main screen is back"
    );
}
