#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Automation spec §8.2: the schedule editor.

use crate::support;

use egui_kittest::kittest::Queryable;
use fp_model::{AppState, Command, TemplateId, apply};
use support::{harness, state};

/// The Schedule column shown, a "Base" template and a "Weekend" one.
fn with_schedule() -> (AppState, TemplateId, TemplateId) {
    let mut s = state(4, 3);
    fp_model::rotation::ensure_default(&mut s, "Music");
    fp_model::filler::ensure(&mut s, "Music filler");
    fp_model::automation::ensure(&mut s, "Schedule");
    fp_model::schedule::ensure_default(&mut s, "Base");
    apply(
        &mut s,
        Command::CreateTemplate {
            name: "Weekend".into(),
        },
    )
    .unwrap();
    let _ = apply(&mut s, Command::SetLayoutLocked(false));
    let p = s.automation.as_ref().unwrap().player;
    s.layout.hidden_players.retain(|x| *x != p);
    s.local_now = chrono::NaiveDate::from_ymd_opt(2026, 11, 17)
        .unwrap()
        .and_hms_opt(8, 0, 0)
        .unwrap();
    let base = s.schedule.templates()[0].id;
    let weekend = s.schedule.templates()[1].id;
    (s, base, weekend)
}

fn open(h: &mut egui_kittest::Harness<'_, fp_app::ui::app::AppUi>) {
    h.get_by_label("Edit schedule").click();
    // The window settles its size over a few frames.
    h.run_steps(6);
}

#[test]
fn the_edit_button_opens_the_editor() {
    let (s, _, _) = with_schedule();
    let (mut h, _fake) = harness(s);
    assert!(h.query_by_label("Schedule editor").is_none());
    open(&mut h);
    assert!(h.query_by_label("Schedule editor").is_some());
}

#[test]
fn the_week_tab_sets_a_weekday_template() {
    let (s, _, weekend) = with_schedule();
    let (mut h, fake) = harness(s);
    open(&mut h);
    h.get_by_label("Week plan").click();
    h.run_steps(2);
    h.get_by_label("Saturday").click();
    h.run_steps(2);
    h.get_by_label("Weekend").click();
    h.run_steps(2);
    assert!(fake.take_sent().contains(&Command::SetWeekTemplate {
        weekday: chrono::Weekday::Sat,
        template: weekend,
    }));
}

#[test]
fn rotations_opens_the_automation_settings() {
    let (s, _, _) = with_schedule();
    let (mut h, _fake) = harness(s);
    open(&mut h);
    h.get_by_label("Rotations…").click();
    h.run_steps(3);
    assert!(h.query_all_by_label_contains("Look-ahead").count() >= 1);
}

/// A 10:00 block on Base with "Top of the Hour" (id 9000, `secs` long).
fn with_block(start: (u32, u32, u32), secs: f64) -> (AppState, TemplateId, fp_model::BlockId) {
    use fp_model::schedule::{Block, Item};
    let (mut s, base, _) = with_schedule();
    let mut t = fp_model::Track::new(fp_model::TrackId(9000), "/m/top.wav".into());
    t.title = "Top of the Hour".into();
    t.duration_secs = secs;
    s.library.insert(t);
    let start = fp_model::local_time::TimeOfDay::hms(start.0, start.1, start.2).unwrap();
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
    (s, base, block)
}

/// Replaces the text of the field labelled `label` and presses Enter.
fn retype(h: &mut egui_kittest::Harness<'_, fp_app::ui::app::AppUi>, label: &str, text: &str) {
    let role = egui::accesskit::Role::TextInput;
    h.get_by_role_and_label(role, label).click();
    h.run_steps(2);
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.run_steps(1);
    h.get_by_role_and_label(role, label).type_text(text);
    h.run_steps(1);
    h.key_press(egui::Key::Enter);
    h.run_steps(2);
}

#[test]
fn new_template_sends_create_template() {
    let (s, _, _) = with_schedule();
    let (mut h, fake) = harness(s);
    open(&mut h);
    h.get_by_label("New template").click();
    h.run_steps(2);
    assert!(fake.take_sent().contains(&Command::CreateTemplate {
        name: "New template".into()
    }));
}

#[test]
fn a_valid_start_sends_update_block() {
    let (s, base, block) = with_block((10, 0, 0), 12.0);
    let (mut h, fake) = harness(s);
    open(&mut h);
    retype(&mut h, "Starts at", "11:30:00");
    let sent = fake.take_sent();
    assert!(
        sent.iter()
            .any(|c| matches!(c, Command::UpdateBlock { template, block: b }
            if *template == base && b.id == block
                && b.start == fp_model::local_time::TimeOfDay::hms(11, 30, 0).unwrap())),
        "{sent:?}"
    );
}

#[test]
fn a_bad_start_is_not_sent() {
    let (s, _, _) = with_block((10, 0, 0), 12.0);
    let (mut h, fake) = harness(s);
    open(&mut h);
    retype(&mut h, "Starts at", "25:00");
    assert!(
        !fake
            .take_sent()
            .iter()
            .any(|c| matches!(c, Command::UpdateBlock { .. }))
    );
    assert!(h.query_by_label("Use HH:MM:SS").is_some());
}

#[test]
fn the_overrun_shows_on_the_block() {
    let (s, _, _) = with_block((23, 59, 0), 120.0);
    let (mut h, _fake) = harness(s);
    open(&mut h);
    assert!(h.query_by_label("Too long by 01:00").is_some());
}

#[test]
fn remove_block_sends_remove_block() {
    let (s, base, block) = with_block((10, 0, 0), 12.0);
    let (mut h, fake) = harness(s);
    open(&mut h);
    h.get_by_label("Remove block").click();
    h.run_steps(2);
    let sent = fake.take_sent();
    assert!(
        sent.contains(&Command::RemoveBlock {
            template: base,
            block
        }),
        "{sent:?}"
    );
}

mod adding {
    use super::*;
    use egui::{Event, Modifiers, PointerButton, Pos2, pos2, vec2};
    use fp_model::schedule::Item;
    use fp_model::{RotationId, TrackId};

    type H = egui_kittest::Harness<'static, fp_app::ui::app::AppUi>;

    fn press(h: &mut H, at: Pos2, pressed: bool) {
        h.event(Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        });
    }

    fn drag(h: &mut H, from: Pos2, to: Pos2) {
        h.event(Event::PointerMoved(from));
        h.run_steps(1);
        press(h, from, true);
        h.run_steps(1);
        for step in 1..=10 {
            let t = step as f32 / 10.0;
            h.event(Event::PointerMoved(pos2(
                from.x + (to.x - from.x) * t,
                from.y + (to.y - from.y) * t,
            )));
            h.run_steps(1);
        }
        h.run_steps(2);
        press(h, to, false);
        h.run_steps(3);
    }

    /// The UpdateBlock items sent, if any.
    fn updated(sent: &[Command]) -> Option<Vec<Item>> {
        sent.iter().rev().find_map(|c| match c {
            Command::UpdateBlock { block, .. } => Some(block.items.clone()),
            _ => None,
        })
    }

    fn top() -> Item {
        Item::Track {
            track: TrackId(9000),
            time_critical: false,
        }
    }

    #[test]
    fn dropping_tracks_adds_them_to_the_block() {
        let (mut s, _, _) = with_block((10, 0, 0), 12.0);
        apply(
            &mut s,
            Command::AddToLibrary {
                paths: vec!["/lib/Lib A.mp3".into()],
                category: None,
            },
        )
        .unwrap();
        let lib_a = s
            .library
            .track_at(std::path::Path::new("/lib/Lib A.mp3"))
            .unwrap();
        s.library_view = fp_model::library_view::LibraryView {
            open: true,
            // Right of the editor, clear of the Schedule column's bar.
            rect: Some([900.0, 300.0, 600.0, 400.0]),
            ..Default::default()
        };
        let (mut h, fake) = support::harness_sized(s, vec2(1700.0, 900.0), |ui| ui);
        h.run_steps(2);
        open(&mut h);
        fake.take_sent();
        let from = h
            .query_all_by_label("Lib A")
            .map(|n| n.rect())
            .find(|r| r.left() > 900.0 && r.top() > 400.0)
            .expect("Lib A in the library table")
            .center();
        // The label and its text run share the value.
        let to = h
            .query_all_by_value("Drop tracks from the library here")
            .next()
            .expect("the drop hint")
            .rect()
            .center();
        drag(&mut h, from, to);
        let sent = fake.take_sent();
        assert_eq!(
            updated(&sent),
            Some(vec![
                top(),
                Item::Track {
                    track: lib_a,
                    time_critical: false
                }
            ]),
            "{sent:?}"
        );
    }

    #[test]
    fn a_category_item_is_added_from_the_combo() {
        let (mut s, _, _) = with_block((10, 0, 0), 12.0);
        apply(
            &mut s,
            Command::CreateCategory {
                parent: None,
                name: "Spots".into(),
                color: None,
                kind: fp_model::TrackKind::Music,
            },
        )
        .unwrap();
        let spots = s.categories.iter().find(|c| c.name == "Spots").unwrap().id;
        let (mut h, fake) = harness(s);
        open(&mut h);
        h.get_by_label("Add from category").click();
        h.run_steps(2);
        h.get_by_label("Spots").click();
        h.run_steps(2);
        assert_eq!(
            updated(&fake.take_sent()),
            Some(vec![
                top(),
                Item::Category {
                    category: spots,
                    time_critical: false
                }
            ])
        );
    }

    #[test]
    fn the_rotation_item_is_set_and_cleared() {
        let (s, base, block) = with_block((10, 0, 0), 12.0);
        let music: RotationId = s.rotations.default_id().unwrap();
        let (mut h, fake) = harness(s.clone());
        open(&mut h);
        h.get_by_label("Music after").click();
        h.run_steps(2);
        h.get_all_by_label("Music").last().unwrap().click();
        h.run_steps(2);
        assert_eq!(
            updated(&fake.take_sent()),
            Some(vec![top(), Item::Rotation { rotation: music }])
        );
        // With the rotation item, Default rotation takes it away.
        let mut s = s;
        let start = s.schedule.template(base).unwrap().blocks[0].start;
        apply(
            &mut s,
            Command::UpdateBlock {
                template: base,
                block: fp_model::schedule::Block {
                    id: block,
                    start,
                    items: vec![top(), Item::Rotation { rotation: music }],
                },
            },
        )
        .unwrap();
        let (mut h, fake) = harness(s);
        open(&mut h);
        h.get_by_label("Music after").click();
        h.run_steps(2);
        h.get_by_label("Default rotation").click();
        h.run_steps(2);
        assert_eq!(updated(&fake.take_sent()), Some(vec![top()]));
    }
}
