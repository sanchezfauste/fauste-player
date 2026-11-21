#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Automation rework AR21–AR22: a template or a date in a playlist-like
//! popup.

use crate::support;

use egui_kittest::kittest::Queryable;
use fp_model::log_edit::{EditTarget, LogEdit};
use fp_model::schedule::{Block, Item};
use fp_model::{AppState, Command, TemplateId, apply};
use support::{harness_with, state};

/// "Base" with a 10:00 block holding "Top of the Hour", and "Weekend";
/// today is Tuesday 2026-11-17, 08:00.
fn with_templates() -> (AppState, TemplateId, TemplateId) {
    let mut s = state(4, 3);
    fp_model::rotation::ensure_default(&mut s, "Music");
    fp_model::filler::ensure(&mut s, "Music fill");
    fp_model::automation::ensure(&mut s, "Log");
    fp_model::schedule::ensure_default(&mut s, "Base");
    apply(
        &mut s,
        Command::CreateTemplate {
            name: "Weekend".into(),
        },
    )
    .unwrap();
    s.local_now = chrono::NaiveDate::from_ymd_opt(2026, 11, 17)
        .unwrap()
        .and_hms_opt(8, 0, 0)
        .unwrap();
    let mut t = fp_model::Track::new(fp_model::TrackId(9000), "/m/top.wav".into());
    t.title = "Top of the Hour".into();
    t.duration_secs = 12.0;
    s.library.insert(t);
    let base = s.schedule.templates()[0].id;
    let weekend = s.schedule.templates()[1].id;
    let start = fp_model::local_time::TimeOfDay::hms(10, 0, 0).unwrap();
    apply(
        &mut s,
        Command::AddBlock {
            template: base,
            start,
        },
    )
    .unwrap();
    let block = s.schedule.template(base).unwrap().blocks[0].id;
    apply(
        &mut s,
        Command::UpdateBlock {
            template: base,
            block: Block {
                id: block,
                start,
                items: vec![Item::Track {
                    track: fp_model::TrackId(9000),
                    time_critical: false,
                }],
            },
        },
    )
    .unwrap();
    (s, base, weekend)
}

fn open(
    s: AppState,
    target: EditTarget,
) -> (
    egui_kittest::Harness<'static, fp_app::ui::app::AppUi>,
    std::sync::Arc<support::Fake>,
) {
    let (mut h, fake) = harness_with(s, |ui| ui.open_plan_window(target));
    h.run_steps(4);
    (h, fake)
}

fn edits(sent: &[Command]) -> Vec<(EditTarget, LogEdit)> {
    sent.iter()
        .filter_map(|c| match c {
            Command::EditPlan { target, edit } => Some((*target, edit.clone())),
            _ => None,
        })
        .collect()
}

#[test]
fn the_popup_lists_a_templates_blocks() {
    let (s, base, _) = with_templates();
    let (h, _fake) = open(s, EditTarget::Template(base));
    assert!(h.query_by_label("Template · Base").is_some());
    // The Log shows yesterday's, today's and tomorrow's 10:00 block (its
    // column is hidden here); the popup shows the template's one.
    assert_eq!(h.query_all_by_label_contains("Top of the Hour").count(), 1);
    assert!(h.query_by_label_contains("10:00:00").is_some());
}

#[test]
fn add_block_in_the_popup_edits_the_template() {
    let (s, base, _) = with_templates();
    let (mut h, fake) = open(s, EditTarget::Template(base));
    fake.take_sent();
    h.get_by_label("Block…").click();
    h.run();
    h.get_by_label("OK").click();
    h.run();
    let sent = edits(&fake.take_sent());
    assert_eq!(
        sent,
        vec![(
            EditTarget::Template(base),
            LogEdit::AddBlock {
                start: fp_model::local_time::TimeOfDay::hms(11, 0, 0).unwrap()
            }
        )]
    );
}

#[test]
fn remove_in_the_popup_edits_the_template() {
    let (s, base, _) = with_templates();
    let block = s.schedule.template(base).unwrap().blocks[0].id;
    let (mut h, fake) = open(s, EditTarget::Template(base));
    fake.take_sent();
    h.get_by_label_contains("Top of the Hour").click_secondary();
    h.run();
    h.get_by_label("Remove").click();
    h.run();
    assert_eq!(
        edits(&fake.take_sent()),
        vec![(
            EditTarget::Template(base),
            LogEdit::Remove { block, index: 0 }
        )]
    );
}

#[test]
fn the_popup_closes_when_its_template_is_deleted() {
    let (mut s, _, weekend) = with_templates();
    apply(&mut s, Command::DeleteTemplate(weekend)).unwrap();
    let (h, _fake) = open(s, EditTarget::Template(weekend));
    assert!(h.query_by_label_contains("Template ·").is_none());
}

#[test]
fn a_date_popup_edits_that_date() {
    let (s, _, _) = with_templates();
    let thursday = chrono::NaiveDate::from_ymd_opt(2026, 11, 19).unwrap();
    let (mut h, fake) = open(s, EditTarget::Date(thursday));
    assert!(h.query_by_label("Day · 2026-11-19").is_some());
    fake.take_sent();
    h.get_by_label_contains("Top of the Hour").click_secondary();
    h.run();
    h.get_by_label("Remove").click();
    h.run();
    let sent = edits(&fake.take_sent());
    assert!(matches!(
        sent.as_slice(),
        [(EditTarget::Date(d), LogEdit::Remove { index: 0, .. })] if *d == thursday
    ));
}

#[test]
fn tracks_dropped_on_the_popup_go_into_the_template() {
    let (s, base, _) = with_templates();
    let block = s.schedule.template(base).unwrap().blocks[0].id;
    let (mut h, fake) = open(s, EditTarget::Template(base));
    fake.take_sent();
    // The row's upper half: before it.
    let at = h.get_by_label_contains("Top of the Hour").rect().left_top() + egui::vec2(4.0, 1.0);
    h.event(egui::Event::PointerMoved(at));
    h.run();
    fp_app::ui::app::AppUi::start_track_drag(&h.ctx, vec![fp_model::TrackId(1)]);
    h.run();
    h.event(egui::Event::PointerButton {
        pos: at,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
    h.run();
    assert_eq!(
        edits(&fake.take_sent()),
        vec![(
            EditTarget::Template(base),
            LogEdit::Insert {
                block,
                index: 0,
                items: vec![Item::Track {
                    track: fp_model::TrackId(1),
                    time_critical: false,
                }],
            }
        )]
    );
}

#[test]
fn an_empty_template_has_no_blocks_not_none_today() {
    let (s, _, weekend) = with_templates();
    let (h, _fake) = open(s, EditTarget::Template(weekend));
    assert!(h.query_by_label("Template · Weekend").is_some());
    assert!(h.query_by_label("No blocks yet").is_some());
    assert!(h.query_by_label("No blocks today").is_none());
}

#[test]
fn a_past_dates_popup_closes() {
    let (s, _, _) = with_templates();
    let monday = chrono::NaiveDate::from_ymd_opt(2026, 11, 16).unwrap();
    let (h, _fake) = open(s, EditTarget::Date(monday));
    assert!(h.query_by_label_contains("Day ·").is_none());
}

#[test]
fn the_popups_close_button_closes_it() {
    let (s, base, _) = with_templates();
    let (mut h, _fake) = open(s, EditTarget::Template(base));
    h.get_by_label("Close window").click();
    h.run();
    h.run();
    assert!(h.query_by_label_contains("Template ·").is_none());
}

#[test]
fn escape_cancels_the_popups_dialog_and_keeps_the_popup() {
    let (s, base, _) = with_templates();
    let (mut h, fake) = open(s, EditTarget::Template(base));
    fake.take_sent();
    h.get_by_label("Block…").click();
    h.run();
    assert!(h.query_by_label("OK").is_some());
    h.key_press(egui::Key::Escape);
    h.run();
    assert!(h.query_by_label("OK").is_none());
    assert!(h.query_by_label("Template · Base").is_some());
    assert!(edits(&fake.take_sent()).is_empty());
}

#[test]
fn todays_popup_adds_nothing_to_a_block_that_already_played() {
    let (mut s, _, _) = with_templates();
    let today = s.local_now.date();
    s.local_now = today.and_hms_opt(11, 0, 0).unwrap();
    let block = s.schedule.day(today)[0].id;
    s.automation
        .as_mut()
        .unwrap()
        .started
        .insert((today, block));
    let (h, _fake) = open(s, EditTarget::Date(today));
    assert!(h.query_by_label("Day · 2026-11-17").is_some());
    assert!(h.query_by_label("Block…").is_some());
    assert!(h.query_by_label("Category track…").is_none());
}

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

#[test]
fn files_dropped_on_the_popup_go_into_the_template() {
    let dir = tempfile::tempdir().unwrap();
    let file = fp_app::ui::files::canonical_path(dir.path()).join("new.wav");
    std::fs::write(&file, b"x").unwrap();
    let (s, base, _) = with_templates();
    let block = s.schedule.template(base).unwrap().blocks[0].id;
    let (mut h, fake) = open(s, EditTarget::Template(base));
    let at = h.get_by_label_contains("Top of the Hour").rect().left_top() + egui::vec2(4.0, 1.0);
    h.hover_at(at);
    h.step();
    fake.take_sent();
    h.input_mut()
        .dropped_files
        .push(std::sync::Arc::new(Dropped(file.clone())));
    for _ in 0..40 {
        h.run_steps(1);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let sent = fake.take_sent();
    assert_eq!(
        edits(&sent),
        vec![(
            EditTarget::Template(base),
            LogEdit::InsertPaths {
                block,
                index: 0,
                paths: vec![file],
            }
        )]
    );
    assert!(
        !sent
            .iter()
            .any(|c| matches!(c, Command::InsertPaths { .. }))
    );
}

#[test]
fn the_popups_footer_shows_every_add_button() {
    let (s, base, _) = with_templates();
    let (h, _fake) = open(s, EditTarget::Template(base));
    for label in [
        "Track…",
        "Category track…",
        "Music fill (rotation)…",
        "Block…",
    ] {
        assert!(h.query_by_label(label).is_some(), "{label}");
    }
    assert!(h.query_by_label("More to add").is_none());
}

#[test]
fn a_column_edge_of_the_list_resizes_like_a_playlists() {
    let (s, base, _) = with_templates();
    let log = s.automation.as_ref().unwrap().player;
    let (mut h, fake) = open(s, EditTarget::Template(base));
    // The popup's header: the one with a Time column.
    let time = h.get_by_label("TIME").rect();
    let artist = h
        .query_all_by_label("ARTIST")
        .map(|n| n.rect())
        .find(|r| (r.center().y - time.center().y).abs() < 1.0)
        .unwrap();
    let from = egui::pos2(artist.left() - 8.0, artist.center().y);
    h.event(egui::Event::PointerMoved(from));
    h.run_steps(1);
    let press = |h: &mut egui_kittest::Harness<'static, fp_app::ui::app::AppUi>,
                 pos: egui::Pos2,
                 pressed: bool| {
        h.event(egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        });
    };
    press(&mut h, from, true);
    h.run_steps(2);
    fake.take_sent();
    let to = egui::pos2(from.x + 60.0, from.y);
    for step in 1..=6 {
        h.event(egui::Event::PointerMoved(egui::pos2(
            from.x + step as f32 * 10.0,
            from.y,
        )));
        h.run_steps(1);
    }
    press(&mut h, to, false);
    h.run_steps(2);
    let sent: Vec<_> = fake
        .take_sent()
        .into_iter()
        .filter_map(|c| match c {
            Command::SetColumnWidths(player, widths) => Some((player, widths)),
            _ => None,
        })
        .collect();
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert_eq!(sent[0].0, log);
    let title = sent[0].1.fraction(fp_model::TableColumn::Title).unwrap();
    let artist = sent[0].1.fraction(fp_model::TableColumn::Artist).unwrap();
    assert!(title > artist, "{:?}", sent[0].1);
}

#[test]
fn the_popups_add_goes_below_the_selected_row_and_shows_it() {
    let (mut s, base, _) = with_templates();
    let rotation = s.rotations.default_id().unwrap();
    // A block every hour, each a song and a Music fill: more rows than
    // the popup shows.
    for hour in (0..24u32).filter(|h| *h != 10) {
        let start = fp_model::local_time::TimeOfDay::hms(hour, 0, 0).unwrap();
        apply(
            &mut s,
            Command::AddBlock {
                template: base,
                start,
            },
        )
        .unwrap();
    }
    let blocks: Vec<_> = s.schedule.template(base).unwrap().blocks.clone();
    for b in blocks {
        let items = vec![
            Item::Track {
                track: fp_model::TrackId(9000),
                time_critical: false,
            },
            Item::Rotation { rotation },
        ];
        apply(
            &mut s,
            Command::UpdateBlock {
                template: base,
                block: Block { items, ..b },
            },
        )
        .unwrap();
    }
    apply(
        &mut s,
        Command::CreateCategory {
            parent: None,
            name: "Spots".into(),
            color: None,
            kind: fp_model::TrackKind::Ad,
        },
    )
    .unwrap();
    let category = s.categories.iter().next().unwrap().id;
    let first = s
        .schedule
        .template(base)
        .unwrap()
        .blocks
        .iter()
        .min_by_key(|b| b.start)
        .unwrap()
        .id;
    let (mut h, fake) = open(s, EditTarget::Template(base));
    let at = h
        .query_all_by_label_contains("Top of the Hour")
        .next()
        .unwrap()
        .rect()
        .center();
    h.hover_at(at);
    h.run();
    for pressed in [true, false] {
        h.event(egui::Event::PointerButton {
            pos: at,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        });
    }
    h.run();
    // Scrolled to the end: the selected row is out of sight.
    h.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Point,
        delta: egui::vec2(0.0, -20000.0),
        modifiers: egui::Modifiers::NONE,
        phase: egui::TouchPhase::Move,
        source: egui::MouseWheelSource::Unknown,
    });
    h.run_steps(30);
    assert!(h.query_by_label_contains("00:00:00").is_none());
    fake.take_sent();
    h.get_by_label("Category track…").click();
    h.run();
    h.get_by_label("OK").click();
    h.run_steps(30);
    assert_eq!(
        edits(&fake.take_sent()),
        vec![(
            EditTarget::Template(base),
            LogEdit::Insert {
                block: first,
                index: 1,
                items: vec![Item::Category {
                    category,
                    time_critical: false,
                }],
            }
        )]
    );
    // The added row is brought into view.
    assert!(h.query_by_label_contains("Spots").is_some());
}

#[test]
fn a_dropped_track_is_selected_so_add_follows_it() {
    let (mut s, base, _) = with_templates();
    apply(
        &mut s,
        Command::CreateCategory {
            parent: None,
            name: "Spots".into(),
            color: None,
            kind: fp_model::TrackKind::Ad,
        },
    )
    .unwrap();
    let category = s.categories.iter().next().unwrap().id;
    let block = s.schedule.template(base).unwrap().blocks[0].id;
    let (mut h, fake) = open(s, EditTarget::Template(base));
    // Before "Top of the Hour", on its upper half.
    let at = h.get_by_label_contains("Top of the Hour").rect().left_top() + egui::vec2(4.0, 1.0);
    h.event(egui::Event::PointerMoved(at));
    h.run();
    fp_app::ui::app::AppUi::start_track_drag(&h.ctx, vec![fp_model::TrackId(1)]);
    h.run();
    h.event(egui::Event::PointerButton {
        pos: at,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
    h.run();
    fake.take_sent();
    h.get_by_label("Category track…").click();
    h.run();
    h.get_by_label("OK").click();
    h.run();
    assert_eq!(
        edits(&fake.take_sent()),
        vec![(
            EditTarget::Template(base),
            LogEdit::Insert {
                block,
                index: 1,
                items: vec![Item::Category {
                    category,
                    time_critical: false,
                }],
            }
        )]
    );
}

#[test]
fn a_far_date_popup_adds_below_the_selected_row() {
    let (mut s, _, _) = with_templates();
    apply(
        &mut s,
        Command::CreateCategory {
            parent: None,
            name: "Spots".into(),
            color: None,
            kind: fp_model::TrackKind::Ad,
        },
    )
    .unwrap();
    let category = s.categories.iter().next().unwrap().id;
    // Past tomorrow: outside the Log's days.
    let thursday = chrono::NaiveDate::from_ymd_opt(2026, 11, 19).unwrap();
    let (mut h, fake) = open(s, EditTarget::Date(thursday));
    // Its row menu has the item's Hard start too.
    h.get_by_label_contains("Top of the Hour").click_secondary();
    h.run();
    assert!(h.query_by_label("Hard start").is_some());
    h.key_press(egui::Key::Escape);
    h.run();
    h.get_by_label_contains("Top of the Hour").click();
    h.run();
    fake.take_sent();
    h.get_by_label("Category track…").click();
    h.run();
    h.get_by_label("OK").click();
    h.run();
    let sent = edits(&fake.take_sent());
    assert!(
        matches!(
            sent.as_slice(),
            [(EditTarget::Date(d), LogEdit::Insert { index: 1, items, .. })]
                if *d == thursday && items == &vec![Item::Category { category, time_critical: false }]
        ),
        "{sent:?}"
    );
}
