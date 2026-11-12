#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Automation spec AU13, AU14, AU38: a date's computed schedule, local
//! time, and the pruning of old date overrides.

use chrono::{NaiveDate, NaiveDateTime, TimeZone, Utc};
use fp_model::local_time::{LocalZone, TimeOfDay};
use fp_model::schedule::{Block, BlockOrigin, Item, block_start, ensure_default, prune_dates};
use fp_model::{AppState, Command, Config, TemplateId, TrackId, WallTime, apply};

fn at(h: u32, m: u32) -> TimeOfDay {
    TimeOfDay::hms(h, m, 0).unwrap()
}

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// A Tuesday.
fn tuesday() -> NaiveDate {
    date(2026, 11, 17)
}

fn song(n: u64) -> Item {
    Item::Track {
        track: TrackId(n),
        time_critical: false,
    }
}

/// "Base" with blocks at 07:00, 08:00 and 09:00, on every day.
fn based() -> (AppState, TemplateId) {
    let mut s = AppState::new(Config::default(), "P");
    ensure_default(&mut s, "Base");
    let base = s.schedule.templates()[0].id;
    for h in [7, 8, 9] {
        apply(
            &mut s,
            Command::AddBlock {
                template: base,
                start: at(h, 0),
            },
        )
        .unwrap();
    }
    (s, base)
}

fn block_at(s: &AppState, t: TemplateId, start: TimeOfDay) -> Block {
    s.schedule
        .template(t)
        .unwrap()
        .blocks
        .iter()
        .find(|b| b.start == start)
        .unwrap()
        .clone()
}

fn summary(s: &AppState, d: NaiveDate) -> Vec<(TimeOfDay, BlockOrigin)> {
    s.schedule
        .day(d)
        .iter()
        .map(|b| (b.start, b.origin))
        .collect()
}

/// Changes the 08:00 block of the Tuesday to play song 1.
fn change_eight(s: &mut AppState, base: TemplateId) -> Block {
    let mut block = block_at(s, base, at(8, 0));
    block.items = vec![song(1)];
    apply(
        s,
        Command::ChangeBlockOnDate {
            date: tuesday(),
            block: block.clone(),
        },
    )
    .unwrap();
    block
}

#[test]
fn a_plain_day_is_its_template() {
    let (s, base) = based();
    let day = s.schedule.day(tuesday());
    let template = &s.schedule.template(base).unwrap().blocks;
    assert_eq!(day.len(), 3);
    for (d, t) in day.iter().zip(template) {
        assert_eq!(
            (d.id, d.start, &d.items, d.origin),
            (t.id, t.start, &t.items, BlockOrigin::Template)
        );
    }
}

#[test]
fn template_edits_reach_unchanged_blocks() {
    let (mut s, base) = based();
    change_eight(&mut s, base);
    let mut seven = block_at(&s, base, at(7, 0));
    seven.items = vec![song(7)];
    apply(
        &mut s,
        Command::UpdateBlock {
            template: base,
            block: seven,
        },
    )
    .unwrap();
    let mut eight = block_at(&s, base, at(8, 0));
    eight.items = vec![song(8)];
    apply(
        &mut s,
        Command::UpdateBlock {
            template: base,
            block: eight,
        },
    )
    .unwrap();
    let day = s.schedule.day(tuesday());
    assert_eq!(day[0].items, vec![song(7)]);
    assert_eq!(day[0].origin, BlockOrigin::Template);
    assert_eq!(day[1].items, vec![song(1)]);
    assert_eq!(day[1].origin, BlockOrigin::Changed);
}

#[test]
fn a_changed_block_follows_its_template_time() {
    let (mut s, base) = based();
    change_eight(&mut s, base);
    let mut eight = block_at(&s, base, at(8, 0));
    eight.start = at(8, 15);
    apply(
        &mut s,
        Command::UpdateBlock {
            template: base,
            block: eight,
        },
    )
    .unwrap();
    let day = s.schedule.day(tuesday());
    assert_eq!(
        (day[1].start, day[1].origin),
        (at(8, 15), BlockOrigin::Changed)
    );
    assert_eq!(day[1].items, vec![song(1)]);
}

#[test]
fn a_changed_block_outlives_its_template_block() {
    let (mut s, base) = based();
    let changed = change_eight(&mut s, base);
    apply(
        &mut s,
        Command::RemoveBlock {
            template: base,
            block: changed.id,
        },
    )
    .unwrap();
    assert_eq!(
        summary(&s, tuesday()),
        vec![
            (at(7, 0), BlockOrigin::Template),
            (at(8, 0), BlockOrigin::DateOnly),
            (at(9, 0), BlockOrigin::Template)
        ]
    );
    // Other days lose the block.
    assert_eq!(s.schedule.day(date(2026, 11, 18)).len(), 2);
}

#[test]
fn changes_survive_a_template_switch() {
    let (mut s, base) = based();
    let changed = change_eight(&mut s, base);
    apply(
        &mut s,
        Command::CreateTemplate {
            name: "Weekend".into(),
        },
    )
    .unwrap();
    let weekend = s.schedule.templates().last().unwrap().id;
    apply(
        &mut s,
        Command::SetDateTemplate {
            date: tuesday(),
            template: Some(weekend),
        },
    )
    .unwrap();
    let day = s.schedule.day(tuesday());
    assert_eq!(day.len(), 1);
    assert_eq!(
        (day[0].id, day[0].start, day[0].origin),
        (changed.id, at(8, 0), BlockOrigin::DateOnly)
    );
    assert_eq!(day[0].items, vec![song(1)]);
}

#[test]
fn a_removed_block_is_not_in_the_day() {
    let (mut s, base) = based();
    let nine = block_at(&s, base, at(9, 0)).id;
    apply(
        &mut s,
        Command::RemoveBlockOnDate {
            date: tuesday(),
            block: nine,
        },
    )
    .unwrap();
    assert_eq!(
        summary(&s, tuesday()),
        vec![
            (at(7, 0), BlockOrigin::Template),
            (at(8, 0), BlockOrigin::Template)
        ]
    );
}

#[test]
fn a_date_block_wins_a_time_collision() {
    let (mut s, base) = based();
    apply(
        &mut s,
        Command::AddBlockOnDate {
            date: tuesday(),
            start: at(10, 0),
        },
    )
    .unwrap();
    apply(
        &mut s,
        Command::AddBlock {
            template: base,
            start: at(10, 0),
        },
    )
    .unwrap();
    let day = s.schedule.day(tuesday());
    let tens: Vec<_> = day.iter().filter(|b| b.start == at(10, 0)).collect();
    assert_eq!(tens.len(), 1);
    assert_eq!(tens[0].origin, BlockOrigin::DateOnly);
    // Another day keeps the template's 10:00.
    let other = s.schedule.day(date(2026, 11, 18));
    assert_eq!(other.last().unwrap().origin, BlockOrigin::Template);
}

/// UTC+1, and UTC+2 from local 2026-03-29 02:00 (which jumps to 03:00) to
/// local 2026-10-25 03:00 (which falls back to 02:00).
struct TestZone;

impl LocalZone for TestZone {
    fn earliest(&self, local: NaiveDateTime) -> Option<WallTime> {
        let spring = date(2026, 3, 29).and_hms_opt(2, 0, 0).unwrap();
        let spring_end = date(2026, 3, 29).and_hms_opt(3, 0, 0).unwrap();
        let autumn = date(2026, 10, 25).and_hms_opt(2, 0, 0).unwrap();
        let autumn_end = date(2026, 10, 25).and_hms_opt(3, 0, 0).unwrap();
        let summer = local >= spring_end && local < autumn_end;
        let offset_h = if local >= spring && local < spring_end {
            return None;
        } else if summer || (local >= autumn && local < autumn_end) {
            2
        } else {
            1
        };
        let utc = local - chrono::Duration::hours(offset_h);
        Some(WallTime(Utc.from_utc_datetime(&utc).timestamp_millis()))
    }
}

fn utc_ms(y: i32, mo: u32, d: u32, h: u32, mi: u32) -> WallTime {
    WallTime(
        Utc.with_ymd_and_hms(y, mo, d, h, mi, 0)
            .unwrap()
            .timestamp_millis(),
    )
}

#[test]
fn a_block_starts_at_its_local_time() {
    assert_eq!(
        block_start(&TestZone, date(2026, 1, 5), at(8, 0)),
        Some(utc_ms(2026, 1, 5, 7, 0))
    );
    assert_eq!(
        block_start(&TestZone, date(2026, 7, 5), at(8, 0)),
        Some(utc_ms(2026, 7, 5, 6, 0))
    );
}

#[test]
fn spring_gap_has_no_start() {
    assert_eq!(block_start(&TestZone, date(2026, 3, 29), at(2, 30)), None);
    assert_eq!(
        block_start(&TestZone, date(2026, 3, 29), at(3, 0)),
        Some(utc_ms(2026, 3, 29, 1, 0))
    );
}

#[test]
fn autumn_overlap_starts_at_the_first() {
    assert_eq!(
        block_start(&TestZone, date(2026, 10, 25), at(2, 30)),
        Some(utc_ms(2026, 10, 25, 0, 30))
    );
}

#[test]
fn old_overrides_are_pruned() {
    let (mut s, _) = based();
    for d in [date(2026, 1, 1), date(2026, 2, 10), date(2026, 2, 20)] {
        apply(
            &mut s,
            Command::AddBlockOnDate {
                date: d,
                start: at(12, 0),
            },
        )
        .unwrap();
    }
    s.config.automation.overrides_keep_days = 30;
    assert_eq!(prune_dates(&mut s, date(2026, 3, 12)), 1);
    assert!(s.schedule.date_plan(date(2026, 1, 1)).is_none());
    assert!(s.schedule.date_plan(date(2026, 2, 10)).is_some());
    assert!(s.schedule.date_plan(date(2026, 2, 20)).is_some());
    assert_eq!(prune_dates(&mut s, date(2026, 3, 12)), 0);
}

#[test]
fn overrides_keep_days_is_clamped() {
    let mut config = Config::default();
    assert_eq!(config.automation.overrides_keep_days, 30);
    config.automation.overrides_keep_days = 0;
    let warnings = config.validate();
    assert_eq!(config.automation.overrides_keep_days, 1);
    assert!(
        warnings
            .iter()
            .any(|w| w.field == "automation.overrides_keep_days")
    );
    config.automation.overrides_keep_days = 99_999;
    config.validate();
    assert_eq!(config.automation.overrides_keep_days, 3_650);
}
