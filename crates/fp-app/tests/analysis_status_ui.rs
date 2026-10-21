#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Main spec §6.1: the status bar shows the analysis activity while it is
//! busy, with the detail in its tooltip, and never over the legend.

mod support;

use egui::Rect;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::activity::{AnalysisActivity, PassProgress};
use fp_app::i18n::I18n;
use fp_app::ui::app::{AppUi, status_legend};
use support::{activity_cell, harness_sized, nth_track, state};

/// Three tracks: the first running, the other two waiting, in a pass.
fn busy(s: &fp_model::AppState) -> AnalysisActivity {
    let mut activity = AnalysisActivity::default();
    activity.running.push(nth_track(s, 0));
    activity.queued.insert(nth_track(s, 1));
    activity.queued.insert(nth_track(s, 2));
    activity.pass = Some(PassProgress { done: 1, total: 3 });
    activity
}

fn open(
    mut s: fp_model::AppState,
    activity: AnalysisActivity,
    width: f32,
) -> Harness<'static, AppUi> {
    let cell = activity_cell(activity);
    let lang = s.config.ui.language.take();
    let (mut h, fake) = harness_sized(s, egui::vec2(width, 700.0), move |ui| {
        ui.with_analysis_activity(cell)
    });
    // The interface follows the configured language from a snapshot after
    // its first frame, as when the operator changes it.
    if lang.is_some() {
        let mut s = (**fake.state.load()).clone();
        s.config.ui.language = lang;
        fake.state.store(std::sync::Arc::new(s));
    }
    h.run_steps(2);
    h
}

#[test]
fn nothing_is_shown_while_the_analysis_is_idle() {
    let h = open(state(1, 3), AnalysisActivity::default(), 1000.0);
    assert!(h.query_by_label_contains("Analysing").is_none());
    assert!(h.query_by_label_contains("Measuring").is_none());
}

#[test]
fn the_item_counts_the_tracks_and_its_tooltip_gives_the_detail() {
    let s = state(1, 3);
    let mut activity = busy(&s);
    activity.loudness = 2;
    let mut h = open(s, activity, 1000.0);
    h.get_by_label("Analysing 3 tracks").hover();
    h.run_steps(40);
    for line in [
        "Analysing now:",
        "Song 1.mp3",
        "2 tracks waiting",
        "Re-analysing the library: 1 of 3",
        "Measuring the loudness of 2 tracks",
    ] {
        assert!(h.query_by_label_contains(line).is_some(), "{line}");
    }
}

#[test]
fn loudness_jobs_alone_show_the_measurement() {
    let activity = AnalysisActivity {
        loudness: 2,
        ..Default::default()
    };
    let h = open(state(1, 3), activity, 1000.0);
    assert!(
        h.query_by_label("Measuring the loudness of 2 tracks")
            .is_some()
    );
    assert!(h.query_by_label_contains("Analysing").is_none());
}

/// The status bar's nodes labelled `label` (the bar is the bottom 24 px).
fn in_bar(h: &Harness<'_, AppUi>, label: &str, height: f32) -> Vec<Rect> {
    h.query_all_by_label(label)
        .map(|n| n.rect())
        .filter(|r| r.center().y > height - 24.0)
        .collect()
}

/// The item stays inside the window and clear of every legend label shown,
/// in `lang` at `width`.
fn clear_of_the_legend(lang: &str, width: f32) {
    let mut s = state(1, 3);
    s.config.ui.language = Some(lang.to_owned());
    let activity = busy(&s);
    let h = open(s, activity, width);
    let t = I18n::new(Some(lang));
    let label = t.tr_args("status-analysing", &[("count", 3.into())]);
    let item = in_bar(&h, &label, 700.0);
    let item = *item
        .first()
        .unwrap_or_else(|| panic!("{lang} {width}: no item"));
    assert!(
        item.left() >= 0.0 && item.right() <= width,
        "{lang} {width}: {item:?}"
    );
    for (_, key) in status_legend() {
        for legend in in_bar(&h, &t.tr(key), 700.0) {
            assert!(
                !legend.intersects(item),
                "{lang} {width}: {key} {legend:?} over {item:?}"
            );
            assert!(legend.left() >= 0.0, "{lang} {width}: {key} {legend:?}");
        }
    }
}

#[test]
fn the_item_is_clear_of_the_legend_at_the_default_width() {
    clear_of_the_legend("en-US", 1000.0);
}

#[test]
fn the_item_is_clear_of_the_legend_in_a_narrow_window() {
    clear_of_the_legend("en-US", 420.0);
}

#[test]
fn the_item_is_clear_of_the_legend_in_german() {
    clear_of_the_legend("de-DE", 1000.0);
    clear_of_the_legend("de-DE", 420.0);
}
