#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
//! The player header never overlaps (itr06 I12): at the least player width,
//! in the longest languages, with every badge on, no two items intersect.

mod support;

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::i18n::I18n;
use fp_app::services::MediaCache;
use fp_app::ui::app::AppUi;
use fp_model::{AppState, Command, DsdOnAir, apply};
use support::{Fake, lock_player, state};

/// One player on a bit-perfect device, playing, with the badges of the
/// header on: mixing, DSD holding the others, the gain badge, BP, and either
/// "stop after" or the entry's own notice.
fn busy(entry_notice: bool) -> AppState {
    let mut s = state(1, 3);
    s.config.loudness.normalize = true;
    lock_player(&mut s, 0);
    let p = s.players[0].id;
    apply(&mut s, Command::Play(p)).unwrap();
    let current = s.players[0].current.unwrap();
    if entry_notice {
        apply(&mut s, Command::SetEntryRepeat(current, true)).unwrap();
    } else {
        s.players[0].stop_after_current = true;
    }
    s.players[0].fading = true;
    s.players[0].fade_ms = 10_000;
    s.players[0].dsd = Some(DsdOnAir {
        entry: current,
        hold_others: true,
    });
    s
}

fn harness_in(lang: &str, state: AppState) -> Harness<'static, AppUi> {
    let fake = Fake::new(state);
    let ui = AppUi::new(fake, I18n::new(Some(lang)), MediaCache::default());
    let mut h = Harness::builder()
        .with_size(egui::vec2(412.0, 700.0))
        .with_step_dt(0.02)
        .build_ui_state(
            |ui, app: &mut AppUi| {
                app.guard_close(ui.ctx());
                app.ui(ui);
            },
            ui,
        );
    h.run_steps(4);
    h
}

/// The rectangle of each header item, `None` for one that is not drawn.
fn header_rects(h: &Harness<'static, AppUi>, t: &I18n) -> Vec<(String, Option<egui::Rect>)> {
    let names = [
        "P1".to_owned(),
        t.tr("status-on-air").to_uppercase(),
        t.tr("badge-mixing"),
        t.tr("badge-stop-after"),
        t.tr("tip-entry-repeat"),
        t.tr("tip-dsd-hold"),
        t.tr("tip-cue"),
        t.tr("tip-single"),
        t.tr("tip-cont"),
        t.tr("tip-bp-off"),
        t.tr("tip-not-normalized-bp"),
    ];
    let band = h.get_by_label("P1").rect();
    names
        .into_iter()
        .map(|n| {
            // Other parts of the screen may carry the same words.
            let rect = h
                .query_all_by_label(&n)
                .map(|node| node.rect())
                .find(|r| (r.center().y - band.center().y).abs() < 1.0);
            (n, rect)
        })
        .collect()
}

fn assert_no_overlap(rects: &[(String, Option<egui::Rect>)], context: &str) {
    let drawn: Vec<_> = rects
        .iter()
        .filter_map(|(n, r)| r.map(|r| (n, r)))
        .collect();
    for (i, (a, ra)) in drawn.iter().enumerate() {
        for (b, rb) in drawn.iter().skip(i + 1) {
            // Touching edges are fine (the two modes share a border).
            let common = ra.intersect(*rb);
            assert!(
                common.width() <= 0.0 || common.height() <= 0.0,
                "{context}: {a} {ra:?} overlaps {b} {rb:?}"
            );
        }
    }
}

fn missing(rects: &[(String, Option<egui::Rect>)]) -> Vec<&str> {
    rects
        .iter()
        .filter(|(_, r)| r.is_none())
        .map(|(n, _)| n.as_str())
        .collect()
}

#[test]
fn the_header_never_overlaps_at_the_least_width() {
    for lang in ["en-US", "es-ES", "de-DE"] {
        for notice in [false, true] {
            let t = I18n::new(Some(lang));
            let state = busy(notice);
            let player = state.players[0].id;
            let h = harness_in(lang, state);
            let context = format!("{lang}, entry notice {notice}");
            let column = h
                .state()
                .player_rects()
                .into_iter()
                .find(|(p, _)| *p == player)
                .map(|(_, r)| r)
                .unwrap();
            assert_eq!(
                column.width(),
                fp_app::ui::panel_layout::MIN_PLAYER_WIDTH,
                "{context}: the column is at its least width"
            );
            let rects = header_rects(&h, &t);
            // The player's number, the state text and the right group stay;
            // the right group stays inside the player's panel.
            let always = [
                "P1".to_owned(),
                t.tr("status-on-air").to_uppercase(),
                t.tr("tip-cue"),
                t.tr("tip-single"),
                t.tr("tip-cont"),
                t.tr("tip-bp-off"),
                t.tr("tip-not-normalized-bp"),
            ];
            for kept in &always {
                let rect = rects
                    .iter()
                    .find(|(n, _)| n == kept)
                    .and_then(|(_, r)| *r)
                    .unwrap_or_else(|| panic!("{context}: {kept} is shown, got {rects:?}"));
                assert!(
                    column.contains_rect(rect),
                    "{context}: {kept} {rect:?} leaves the panel {column:?}"
                );
            }
            assert_no_overlap(&rects, &context);
            // What is not shown has collapsed, and only from the least
            // important up: the other badges are shown or hidden whole.
            let hidden = missing(&rects);
            let optional = [
                t.tr("badge-mixing"),
                t.tr("badge-stop-after"),
                t.tr("tip-entry-repeat"),
                t.tr("tip-dsd-hold"),
            ];
            for name in hidden {
                assert!(optional.iter().any(|o| o == name), "{context}: lost {name}");
            }
        }
    }
}

#[test]
fn in_spanish_the_badges_collapse_and_the_state_text_stays() {
    let t = I18n::new(Some("es-ES"));
    let h = harness_in("es-ES", busy(false));
    let rects = header_rects(&h, &t);
    // Only the player's number, the dot and the truncated state text fit
    // beside the right group: every badge collapsed.
    let hidden: Vec<String> = missing(&rects).iter().map(|n| (*n).to_owned()).collect();
    assert_eq!(
        hidden,
        vec![
            t.tr("badge-mixing"),
            t.tr("badge-stop-after"),
            t.tr("tip-entry-repeat"),
            t.tr("tip-dsd-hold"),
        ]
    );
}

fn gain_text_colours(h: &Harness<'static, AppUi>, text: &str) -> Vec<egui::Color32> {
    fn find(shape: &egui::Shape, text: &str, out: &mut Vec<egui::Color32>) {
        match shape {
            egui::Shape::Text(t) if t.galley.text() == text => {
                out.push(t.galley.job.sections[0].format.color);
            }
            egui::Shape::Vec(v) => v.iter().for_each(|s| find(s, text, out)),
            _ => {}
        }
    }
    let mut found = Vec::new();
    for clipped in &h.output().shapes {
        find(&clipped.shape, text, &mut found);
    }
    found
}

#[test]
fn a_not_normalized_track_reads_0_0_db_in_amber_and_the_reason_is_its_name() {
    let h = harness_in("en-US", busy(false));
    assert_eq!(
        gain_text_colours(&h, "0.0 dB"),
        vec![fp_app::ui::theme::AMBER]
    );
    assert!(
        h.query_by_label("Bit-perfect output: tracks play unaltered.")
            .is_some()
    );
    assert!(h.query_by_label("Not normalized").is_none());
}

#[test]
fn the_gain_badge_keeps_its_width_whatever_the_gain() {
    let mut loud = state(1, 3);
    loud.config.loudness.normalize = true;
    for t in loud.library.iter_mut() {
        t.analyzed = true;
        t.duration_secs = 180.0;
        t.loudness = Some(fp_model::LoudnessMeasurement {
            integrated_lufs: Some(-20.0),
            true_peak_dbtp: -10.0,
        });
    }
    let p = loud.players[0].id;
    apply(&mut loud, Command::Play(p)).unwrap();
    let applied = harness_in("en-US", loud);
    let unmeasured = {
        let mut s = state(1, 3);
        s.config.loudness.normalize = true;
        let p = s.players[0].id;
        apply(&mut s, Command::Play(p)).unwrap();
        harness_in("en-US", s)
    };
    let width = |h: &Harness<'static, AppUi>, label: &str| {
        h.query_by_label_contains(label).unwrap().rect().width()
    };
    let a = width(&applied, "+4.0 dB");
    let b = width(&unmeasured, "Loudness not measured");
    assert!((a - b).abs() < 0.5, "{a} vs {b}");
}
