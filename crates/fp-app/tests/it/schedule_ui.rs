#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Automation spec §8.1: the Schedule column.

use crate::support;

use egui_kittest::kittest::Queryable;
use fp_model::{AppState, Command, apply};
use support::{harness, state};

/// Four players, three tracks, the filler and the schedule's player, its
/// column shown.
fn with_schedule() -> AppState {
    let mut s = state(4, 3);
    fp_model::rotation::ensure_default(&mut s, "Music");
    fp_model::filler::ensure(&mut s, "Music filler");
    fp_model::automation::ensure(&mut s, "Schedule");
    fp_model::schedule::ensure_default(&mut s, "Base");
    let _ = apply(&mut s, Command::SetLayoutLocked(false));
    let p = s.automation.as_ref().unwrap().player;
    s.layout.hidden_players.retain(|x| *x != p);
    s
}

#[test]
fn the_log_has_no_start_button() {
    let (h, _fake) = harness(with_schedule());
    assert!(h.query_by_label("Start").is_none());
    assert!(h.query_by_label("Stop after current").is_none());
    assert!(h.query_by_label("Running").is_none());
}

#[test]
fn play_on_the_log_sends_play() {
    let mut s = with_schedule();
    let log = s.automation.as_ref().unwrap().player;
    // A next to play: today's clock gives none without blocks, so pin one.
    s.automation.as_mut().unwrap().next = Some(fp_model::automation::ItemRef {
        date: s.local_now.date(),
        block: fp_model::BlockId(1),
        index: 0,
    });
    let (mut h, fake) = harness(s);
    h.get_all_by_label("Play").next().unwrap().click();
    h.run();
    assert!(fake.take_sent().contains(&Command::Play(log)));
}

#[test]
fn during_music_the_log_header_shows_the_song() {
    let mut s = with_schedule();
    let f = s.filler.as_ref().unwrap().clone();
    let entry = s.playlists.get(f.playlist).unwrap().entries[0].clone();
    let before = {
        let (h, _fake) = harness(s.clone());
        h.query_all_by_label("Fade into the next track").count()
    };
    s.automation.as_mut().unwrap().running = true;
    let p = s.players.iter_mut().find(|p| p.id == f.player).unwrap();
    p.current = Some(entry.id);
    p.transport = fp_model::Transport::Playing;
    // The filler's own column stays hidden: only the Log shows it on air
    // (Play becomes Next).
    let (h, _fake) = harness(s);
    assert_eq!(
        h.query_all_by_label("Fade into the next track").count(),
        before + 1
    );
}

/// Today at 08:00, a 10:00 block with "Top of the Hour" (id 9000) and a
/// category item.
fn with_block() -> (AppState, fp_model::automation::ItemRef) {
    use fp_model::schedule::{Block, Item};
    let mut s = with_schedule();
    let date = chrono::NaiveDate::from_ymd_opt(2026, 11, 17).unwrap();
    s.local_now = date.and_hms_opt(8, 0, 0).unwrap();
    let mut t = fp_model::Track::new(fp_model::TrackId(9000), "/m/top.wav".into());
    t.title = "Top of the Hour".into();
    t.duration_secs = 12.0;
    s.library.insert(t);
    let base = s.schedule.templates()[0].id;
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
    let item = fp_model::automation::ItemRef {
        date,
        block,
        index: 0,
    };
    (s, item)
}

#[test]
fn the_list_shows_blocks_and_their_items() {
    let (s, _) = with_block();
    let (h, _fake) = harness(s);
    // The template is every day's: today's, yesterday's last and
    // tomorrow's first block are all at 10:00.
    assert_eq!(h.query_all_by_label_contains("10:00:00").count(), 3);
    assert_eq!(h.query_all_by_label_contains("Top of the Hour").count(), 3);
    assert_eq!(h.query_all_by_label_contains("tomorrow").count(), 1);
}

#[test]
fn a_missed_item_is_struck_through() {
    let (mut s, item) = with_block();
    // Missed, and not next: the clock had moved on.
    let a = s.automation.as_mut().unwrap();
    a.missed.insert(item);
    a.next = None;
    let (h, _fake) = harness(s);
    let row = h.get_by_label_contains("did not air");
    assert!(
        format!("{row:?}").contains("Top of the Hour"),
        "the missed row is the item's"
    );
}

#[test]
fn an_empty_schedule_says_so() {
    let (h, _fake) = harness(with_schedule());
    assert!(h.query_by_label("No blocks today").is_some());
}

#[test]
fn a_deleted_track_shows_not_available() {
    let (mut s, _) = with_block();
    s.library.remove(fp_model::TrackId(9000));
    let (h, _fake) = harness(s);
    assert!(h.query_all_by_label_contains("Not available").count() >= 1);
}

/// Today's row of "Top of the Hour" (yesterday's last block comes first).
fn todays_row(h: &egui_kittest::Harness<'_, fp_app::ui::app::AppUi>) -> egui::Pos2 {
    h.get_all_by_label_contains("Top of the Hour")
        .nth(1)
        .unwrap()
        .rect()
        .center()
}

#[test]
fn double_click_sets_the_item_as_next() {
    use egui::{Event, Modifiers, PointerButton};
    let (s, item) = with_block();
    let (mut h, fake) = harness(s);
    let at = todays_row(&h);
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    for _ in 0..2 {
        for pressed in [true, false] {
            h.event(Event::PointerButton {
                pos: at,
                button: PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            });
        }
        h.step();
    }
    h.run_steps(2);
    assert!(fake.take_sent().contains(&Command::SetScheduleNext(item)));
}

#[test]
fn the_menu_sets_and_clears_next() {
    let (mut s, item) = with_block();
    let (mut h, fake) = harness(s.clone());
    h.get_all_by_label_contains("Top of the Hour")
        .nth(1)
        .unwrap()
        .click_secondary();
    h.run_steps(2);
    h.get_by_label("Play next").click();
    h.run_steps(2);
    assert!(fake.take_sent().contains(&Command::SetScheduleNext(item)));
    // A next set by hand can go back to the clock.
    {
        let a = s.automation.as_mut().unwrap();
        a.next = Some(item);
        a.next_by_hand = true;
    }
    let (mut h, fake) = harness(s);
    h.get_all_by_label_contains("Top of the Hour")
        .nth(1)
        .unwrap()
        .click_secondary();
    h.run_steps(2);
    h.get_by_label("Back to the clock").click();
    h.run_steps(2);
    assert!(fake.take_sent().contains(&Command::ClearScheduleNext));
}

/// Review of rework plan 3: during the music, the Log header's buttons
/// act on the Log.
#[test]
fn during_music_the_log_header_buttons_act_on_the_log() {
    let mut s = with_schedule();
    let f = s.filler.as_ref().unwrap().clone();
    let log = s.automation.as_ref().unwrap().player;
    let entry = s.playlists.get(f.playlist).unwrap().entries[0].clone();
    s.automation.as_mut().unwrap().running = true;
    let p = s.players.iter_mut().find(|p| p.id == f.player).unwrap();
    p.current = Some(entry.id);
    p.transport = fp_model::Transport::Playing;
    let (mut h, fake) = harness(s);
    // The Log's column is the first one.
    h.get_all_by_label("Stop").next().unwrap().click();
    h.run();
    let sent = fake.take_sent();
    assert!(sent.contains(&Command::Stop(log)), "{sent:?}");
    assert!(!sent.contains(&Command::Stop(f.player)));
}

// Rework plan 4: the Log's list is drawn as a playlist.

#[test]
fn the_log_list_has_the_playlist_columns() {
    let (s, _) = with_block();
    let (h, _fake) = harness(s);
    // Time is the Log's own; the players' tables have the others too.
    assert_eq!(h.query_all_by_label("TIME").count(), 1);
    for label in ["TITLE", "ARTIST", "DUR."] {
        assert!(h.query_all_by_label(label).count() >= 2, "{label}");
    }
}

#[test]
fn a_category_row_shows_its_track() {
    use fp_model::schedule::{Block, Item};
    let (mut s, item) = with_block();
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
    let cat = s.categories.iter().last().unwrap().id;
    let mut t = fp_model::Track::new(fp_model::TrackId(9100), "/m/bakery.wav".into());
    t.title = "Riverside Bakery".into();
    t.duration_secs = 20.0;
    s.library.insert(t);
    let base = s.schedule.templates()[0].id;
    let start = fp_model::local_time::TimeOfDay::hms(10, 0, 0).unwrap();
    apply(
        &mut s,
        Command::UpdateBlock {
            template: base,
            block: Block {
                id: item.block,
                start,
                items: vec![Item::Category {
                    category: cat,
                    time_critical: false,
                }],
            },
        },
    )
    .unwrap();
    // Its block is loaded: the Log's list holds the chosen track.
    let a = s.automation.clone().unwrap();
    let entry = fp_model::EntryId(77_777);
    s.playlists
        .get_mut(a.playlist)
        .unwrap()
        .entries
        .push(fp_model::PlaylistEntry::new(entry, fp_model::TrackId(9100)));
    s.automation.as_mut().unwrap().rows.insert(entry, item);
    let (h, _fake) = harness(s);
    assert!(h.query_by_label_contains("Riverside Bakery").is_some());
}

#[test]
fn a_late_next_says_so() {
    let (mut s, item) = with_block();
    s.local_now = item.date.and_hms_opt(10, 30, 0).unwrap();
    let a = s.automation.as_mut().unwrap();
    a.next = Some(item);
    a.next_by_hand = true;
    let (h, _fake) = harness(s);
    assert!(h.query_by_label_contains("its time has passed").is_some());
}

// Rework plan 4, Task 4: editing the Log from its list.

fn edits(sent: &[Command]) -> Vec<fp_model::log_edit::LogEdit> {
    sent.iter()
        .filter_map(|c| match c {
            Command::EditPlan { edit, .. } => Some(edit.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn add_block_sends_add_block_with_a_free_time() {
    let (s, _) = with_block();
    let (mut h, fake) = harness(s);
    h.get_by_label("Add to the log").click();
    h.run();
    h.get_by_label("Block…").click();
    h.run();
    h.get_by_label("OK").click();
    h.run();
    assert_eq!(
        edits(&fake.take_sent()),
        vec![fp_model::log_edit::LogEdit::AddBlock {
            start: fp_model::local_time::TimeOfDay::hms(11, 0, 0).unwrap()
        }]
    );
}

#[test]
fn add_music_fill_sends_insert() {
    let (s, item) = with_block();
    let rotation = s.rotations.default_id().unwrap();
    let (mut h, fake) = harness(s);
    h.get_by_label("Add to the log").click();
    h.run();
    h.get_by_label("Music fill (rotation)…").click();
    h.run();
    h.get_by_label("OK").click();
    h.run();
    assert_eq!(
        edits(&fake.take_sent()),
        vec![fp_model::log_edit::LogEdit::Insert {
            block: item.block,
            index: usize::MAX,
            items: vec![fp_model::schedule::Item::Rotation { rotation }],
        }]
    );
}

#[test]
fn remove_sends_remove() {
    let (s, item) = with_block();
    let (mut h, fake) = harness(s);
    // Today's row (yesterday's and tomorrow's show the same title).
    let rows: Vec<_> = h.query_all_by_label_contains("Top of the Hour").collect();
    rows[1].click_secondary();
    h.run();
    h.get_by_label("Remove from today's log").click();
    h.run();
    assert_eq!(
        edits(&fake.take_sent()),
        vec![fp_model::log_edit::LogEdit::Remove {
            block: item.block,
            index: 0
        }]
    );
}

#[test]
fn a_past_row_has_no_remove() {
    let (s, _) = with_block();
    let (mut h, _fake) = harness(s);
    let rows: Vec<_> = h.query_all_by_label_contains("Top of the Hour").collect();
    rows[0].click_secondary();
    h.run();
    assert!(h.query_by_label("Remove from today's log").is_none());
}

/// Today's block header (yesterday's and tomorrow's show too).
fn todays_header(h: &egui_kittest::Harness<'_, fp_app::ui::app::AppUi>) -> egui::Pos2 {
    h.query_all_by_label_contains("Block ·")
        .find(|n| {
            let text = format!("{n:?}");
            !text.contains("tomorrow") && !text.contains("yesterday")
        })
        .unwrap()
        .rect()
        .center()
}

fn secondary_at(h: &mut egui_kittest::Harness<'_, fp_app::ui::app::AppUi>, pos: egui::Pos2) {
    h.hover_at(pos);
    h.run();
    h.event(egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Secondary,
        pressed: true,
        modifiers: egui::Modifiers::NONE,
    });
    h.event(egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Secondary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    });
    h.run();
}

#[test]
fn add_category_track_sends_insert() {
    let (mut s, item) = with_block();
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
    let (mut h, fake) = harness(s);
    h.get_by_label("Add to the log").click();
    h.run();
    h.get_by_label("Category track…").click();
    h.run();
    h.get_by_label("OK").click();
    h.run();
    assert_eq!(
        edits(&fake.take_sent()),
        vec![fp_model::log_edit::LogEdit::Insert {
            block: item.block,
            index: usize::MAX,
            items: vec![fp_model::schedule::Item::Category {
                category,
                time_critical: false,
            }],
        }]
    );
}

#[test]
fn cancel_sends_nothing() {
    let (s, _) = with_block();
    let (mut h, fake) = harness(s);
    h.get_by_label("Add to the log").click();
    h.run();
    h.get_by_label("Block…").click();
    h.run();
    h.get_by_label("Cancel").click();
    h.run();
    assert!(edits(&fake.take_sent()).is_empty());
    assert!(h.query_by_label("OK").is_none());
}

#[test]
fn add_track_opens_the_library() {
    let (s, _) = with_block();
    let (mut h, fake) = harness(s);
    h.get_by_label("Add to the log").click();
    h.run();
    h.get_by_label("Track…").click();
    h.run();
    assert!(
        fake.take_sent()
            .iter()
            .any(|c| matches!(c, Command::SetLibraryView(v) if v.open))
    );
}

#[test]
fn the_block_menu_changes_its_time() {
    let (s, item) = with_block();
    let (mut h, fake) = harness(s);
    let at = todays_header(&h);
    secondary_at(&mut h, at);
    h.get_by_label("Change time…").click();
    h.run();
    h.get_by_label("Starts at").focus();
    h.run();
    h.get_by_label("Starts at").type_text("x");
    h.run();
    // A time that does not parse: the field says so.
    assert!(h.query_by_label("Use HH:MM:SS").is_some());
    h.get_by_label("Cancel").click();
    h.run();
    secondary_at(&mut h, at);
    h.get_by_label("Change time…").click();
    h.run();
    h.get_by_label("OK").click();
    h.run();
    assert_eq!(
        edits(&fake.take_sent()),
        vec![fp_model::log_edit::LogEdit::MoveBlock {
            block: item.block,
            start: fp_model::local_time::TimeOfDay::hms(10, 0, 0).unwrap()
        }]
    );
}

#[test]
fn the_block_menu_removes_the_block() {
    let (s, item) = with_block();
    let (mut h, fake) = harness(s);
    let at = todays_header(&h);
    secondary_at(&mut h, at);
    h.get_by_label("Remove block").click();
    h.run();
    assert_eq!(
        edits(&fake.take_sent()),
        vec![fp_model::log_edit::LogEdit::RemoveBlock { block: item.block }]
    );
}

#[test]
fn a_changed_block_goes_back_to_the_template() {
    let (mut s, item) = with_block();
    apply(
        &mut s,
        Command::EditPlan {
            target: fp_model::log_edit::EditTarget::Date(item.date),
            edit: fp_model::log_edit::LogEdit::SetHardStart {
                block: item.block,
                index: 0,
                on: true,
            },
        },
    )
    .unwrap();
    let (mut h, fake) = harness(s);
    let at = h
        .get_by_label_contains("changed for this day")
        .rect()
        .center();
    secondary_at(&mut h, at);
    h.get_by_label("Back to the template").click();
    h.run();
    assert_eq!(
        edits(&fake.take_sent()),
        vec![fp_model::log_edit::LogEdit::Revert { block: item.block }]
    );
}

#[test]
fn the_row_menu_toggles_hard_start() {
    let (s, item) = with_block();
    let (mut h, fake) = harness(s);
    let rows: Vec<_> = h.query_all_by_label_contains("Top of the Hour").collect();
    rows[1].click_secondary();
    h.run();
    h.get_by_label("Hard start").click();
    h.run();
    assert_eq!(
        edits(&fake.take_sent()),
        vec![fp_model::log_edit::LogEdit::SetHardStart {
            block: item.block,
            index: 0,
            on: true
        }]
    );
}

#[test]
fn the_footer_says_when_the_next_block_starts() {
    let (mut s, item) = with_block();
    let a = s.automation.as_mut().unwrap();
    a.running = true;
    a.next = Some(item);
    let (h, _fake) = harness(s);
    assert!(h.query_by_label_contains("Next block 10:00").is_some());
}

#[test]
fn library_tracks_dropped_on_a_row_go_before_it() {
    let (s, item) = with_block();
    let (mut h, fake) = harness(s);
    let rows: Vec<_> = h.query_all_by_label_contains("Top of the Hour").collect();
    // The row's upper half.
    let at = rows[1].rect().left_top() + egui::vec2(4.0, 1.0);
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
        vec![fp_model::log_edit::LogEdit::Insert {
            block: item.block,
            index: 0,
            items: vec![fp_model::schedule::Item::Track {
                track: fp_model::TrackId(1),
                time_critical: false,
            }],
        }]
    );
}

#[test]
fn add_skips_todays_block_that_already_played() {
    let (mut s, item) = with_block();
    // 11:00: the 10:00 block played and nothing is next.
    s.local_now = item.date.and_hms_opt(11, 0, 0).unwrap();
    let a = s.automation.as_mut().unwrap();
    a.started.insert((item.date, item.block));
    a.next = None;
    let (mut h, _fake) = harness(s);
    h.get_by_label("Add to the log").click();
    h.run();
    assert!(h.query_by_label("Block…").is_some());
    assert!(h.query_by_label("Category track…").is_none());
}

#[test]
fn library_tracks_dropped_on_a_rows_lower_half_go_after_it() {
    let (s, item) = with_block();
    let (mut h, fake) = harness(s);
    let rows: Vec<_> = h.query_all_by_label_contains("Top of the Hour").collect();
    // The lower half of the block's last row: the block's end.
    let at = rows[1].rect().left_bottom() + egui::vec2(4.0, -1.0);
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
        vec![fp_model::log_edit::LogEdit::Insert {
            block: item.block,
            index: 1,
            items: vec![fp_model::schedule::Item::Track {
                track: fp_model::TrackId(1),
                time_critical: false,
            }],
        }]
    );
}

#[test]
fn add_skips_a_coming_block_already_played_by_hand() {
    let (mut s, item) = with_block();
    // 08:00: the 10:00 block was played early by hand; nothing is next.
    let a = s.automation.as_mut().unwrap();
    a.started.insert((item.date, item.block));
    a.next = None;
    let (mut h, _fake) = harness(s);
    h.get_by_label("Add to the log").click();
    h.run();
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
fn files_dropped_on_a_log_row_go_before_it() {
    let dir = tempfile::tempdir().unwrap();
    // The paths come back canonical (macOS temp dirs are symlinks).
    let file = std::fs::canonicalize(dir.path()).unwrap().join("new.wav");
    std::fs::write(&file, b"x").unwrap();
    let (s, item) = with_block();
    let (mut h, fake) = harness(s);
    let rows: Vec<_> = h.query_all_by_label_contains("Top of the Hour").collect();
    let at = rows[1].rect().left_top() + egui::vec2(4.0, 1.0);
    h.hover_at(at);
    h.step();
    fake.take_sent();
    h.input_mut()
        .dropped_files
        .push(std::sync::Arc::new(Dropped(file.clone())));
    // Dropped paths are read on a helper thread: give it time to answer.
    for _ in 0..40 {
        h.run_steps(1);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let sent = fake.take_sent();
    assert!(
        sent.iter().any(|c| matches!(
            c,
            Command::EditPlan {
                target: fp_model::log_edit::EditTarget::Date(d),
                edit: fp_model::log_edit::LogEdit::InsertPaths { block, index: 0, paths },
            } if *d == item.date && *block == item.block && paths == &vec![file.clone()]
        )),
        "{sent:?}"
    );
    assert!(
        !sent
            .iter()
            .any(|c| matches!(c, Command::InsertPaths { .. })),
        "not into a player's list"
    );
}
