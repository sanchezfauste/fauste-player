#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Automation spec AU39: thirty simulated days of automation, with a week
//! plan, a date override, the music filler and the as-run log saved every
//! hour. Nothing that runs for weeks may grow without bound, and every
//! block starts on time.

use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{Datelike, Days, NaiveDate, NaiveDateTime, TimeDelta, Timelike, Weekday};
use fp_model::as_run::{AsRunEntry, AsRunOutcome, AsRunSource};
use fp_model::local_time::TimeOfDay;
use fp_model::schedule::{Block, BlockOrigin, Item};
use fp_model::{
    AppState, Command, Config, EngineAction, EngineEvent, EntryId, PlayerId, Track, TrackId,
    TrackKind, Transport, WallTime, apply, on_event,
};
use fp_store::{AppPaths, Store};

use crate::support::save_all;

const DAYS: u64 = 30;
const STEP_SECS: i64 = 10;
const HOURLY_ID: TrackId = TrackId(100);

fn start_day() -> NaiveDate {
    // A Monday.
    NaiveDate::from_ymd_opt(2026, 11, 16).unwrap()
}

fn song(n: u64) -> Item {
    Item::Track {
        track: TrackId(n),
        time_critical: false,
    }
}

/// The model and a fake engine: a start is on air at once, a fade out
/// ends at once, and a song ends at its length.
struct Sim {
    s: AppState,
    /// The row on air of each automation player, when it started and
    /// how long it lasts.
    on_air: HashMap<PlayerId, (EntryId, NaiveDateTime, f64)>,
}

impl Sim {
    fn engine(&mut self, actions: Vec<EngineAction>) {
        for action in actions {
            let event = match action {
                EngineAction::StartCurrent { player, request }
                | EngineAction::Crossfade {
                    player, request, ..
                } => EngineEvent::TransitionStarted {
                    player,
                    entry: request.entry,
                    normalization: fp_model::Normalization::default(),
                },
                EngineAction::FadeOutAndStop { player, .. } => {
                    let Some(entry) = self.player(player).current else {
                        continue;
                    };
                    EngineEvent::ReachedEnd { player, entry }
                }
                _ => continue,
            };
            let more = on_event(&mut self.s, event);
            self.engine(more);
        }
    }

    fn player(&self, id: PlayerId) -> &fp_model::PlayerState {
        self.s.players.iter().find(|p| p.id == id).unwrap()
    }

    fn apply(&mut self, command: Command) {
        let actions = apply(&mut self.s, command).unwrap();
        self.engine(actions);
    }

    fn players(&self) -> [PlayerId; 2] {
        [
            self.s.automation.as_ref().unwrap().player,
            self.s.filler.as_ref().unwrap().player,
        ]
    }

    /// One tick at `at`, then every song that reached its end ends.
    fn tick(&mut self, at: NaiveDateTime) {
        self.s.local_now = at;
        self.s.wall_now = WallTime(at.and_utc().timestamp_millis());
        let (actions, _) = fp_model::automation::tick(&mut self.s);
        self.engine(actions);
        for player in self.players() {
            let p = self.player(player).clone();
            let current = p.current.filter(|_| p.transport == Transport::Playing);
            let seen = self.on_air.get(&player).map(|(e, _, _)| *e);
            if current != seen {
                match current {
                    Some(e) => {
                        let secs = self.s.track_for_entry(e).unwrap().duration_secs;
                        self.on_air.insert(player, (e, at, secs));
                    }
                    None => {
                        self.on_air.remove(&player);
                    }
                }
            }
            let Some((entry, since, secs)) = self.on_air.get(&player).copied() else {
                continue;
            };
            if (at - since).num_milliseconds() as f64 / 1000.0 < secs {
                continue;
            }
            let p = self.player(player).clone();
            let event = match (p.next, p.stop_after_current) {
                (Some(next), false) => EngineEvent::TransitionStarted {
                    player,
                    entry: next,
                    normalization: fp_model::Normalization::default(),
                },
                _ => EngineEvent::ReachedEnd { player, entry },
            };
            self.on_air.remove(&player);
            let more = on_event(&mut self.s, event);
            self.engine(more);
        }
    }
}

/// Weekdays: a block every hour from 01:00 to 23:00 (the hourly ID and a
/// song). Weekends: 09:00, 15:00 and 21:00 (two songs). Day 10 adds a
/// 12:30 block for that date alone.
fn setup() -> Sim {
    let mut s = AppState::new(Config::default(), "Main");
    s.config.automation.as_run_keep_days = 3;
    for n in 1..=60u64 {
        let mut t = Track::new(TrackId(n), PathBuf::from(format!("/m/{n}.wav")));
        t.duration_secs = 150.0 + (n % 7) as f64 * 15.0;
        t.kind = TrackKind::Music;
        t.artist = format!("Artist {n}");
        s.library.insert(t);
    }
    let mut id = Track::new(HOURLY_ID, PathBuf::from("/m/id.wav"));
    id.duration_secs = 10.0;
    id.kind = TrackKind::HourlyId;
    s.library.insert(id);
    s.ids.observe(1000);
    fp_model::rotation::ensure_default(&mut s, "Music");
    fp_model::filler::ensure(&mut s, "Music filler");
    fp_model::automation::ensure(&mut s, "Schedule");
    fp_model::schedule::ensure_default(&mut s, "Base");
    let mut sim = Sim {
        s,
        on_air: HashMap::new(),
    };
    sim.s.local_now = start_day().and_hms_opt(0, 0, 0).unwrap();
    let base = sim.s.schedule.templates()[0].id;
    for h in 1..24 {
        add_block(
            &mut sim,
            base,
            h,
            0,
            vec![
                Item::Track {
                    track: HOURLY_ID,
                    time_critical: false,
                },
                song(h as u64),
            ],
        );
    }
    sim.apply(Command::CreateTemplate {
        name: "Weekend".into(),
    });
    let weekend = sim.s.schedule.templates()[1].id;
    for h in [9, 15, 21] {
        add_block(
            &mut sim,
            weekend,
            h,
            0,
            vec![song(h as u64), song(h as u64 + 30)],
        );
    }
    for weekday in [Weekday::Sat, Weekday::Sun] {
        sim.apply(Command::SetWeekTemplate {
            weekday,
            template: weekend,
        });
    }
    let day10 = start_day().checked_add_days(Days::new(9)).unwrap();
    let start = TimeOfDay::hms(12, 30, 0).unwrap();
    sim.apply(Command::AddBlockOnDate { date: day10, start });
    let added = sim
        .s
        .schedule
        .day(day10)
        .into_iter()
        .find(|b| b.origin == BlockOrigin::DateOnly)
        .unwrap()
        .id;
    sim.apply(Command::ChangeBlockOnDate {
        date: day10,
        block: Block {
            id: added,
            start,
            items: vec![song(50)],
        },
    });
    sim
}

fn add_block(sim: &mut Sim, template: fp_model::TemplateId, h: u32, m: u32, items: Vec<Item>) {
    let start = TimeOfDay::hms(h, m, 0).unwrap();
    sim.apply(Command::AddBlock { template, start });
    let id = sim
        .s
        .schedule
        .template(template)
        .unwrap()
        .blocks
        .iter()
        .find(|b| b.start == start)
        .unwrap()
        .id;
    sim.apply(Command::UpdateBlock {
        template,
        block: Block { id, start, items },
    });
}

fn count(s: &Store, sql: &str) -> i64 {
    let db = fp_store::Db::open(&s.paths().database_file()).unwrap();
    db.conn().query_row(sql, [], |r| r.get(0)).unwrap()
}

#[test]
fn thirty_days_of_automation_stay_bounded_and_on_time() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::new(AppPaths::under(dir.path()), Config::default().limits);
    let mut sim = setup();
    let begin = start_day().and_hms_opt(0, 0, 0).unwrap();
    sim.tick(begin);
    sim.apply(Command::StartAutomation);
    let mut log: Vec<AsRunEntry> = Vec::new();
    let mut saved_seq = 0;
    let mut peak_rows = 0;
    let end = begin + TimeDelta::days(DAYS as i64);
    let mut t = begin;
    while t < end {
        t += TimeDelta::seconds(STEP_SECS);
        sim.tick(t);
        if t.time().second() == 0 && t.time().minute() == 0 {
            // Every hour: save, and check that nothing grows.
            save_all(&mut store, &sim.s, 0.0);
            log.extend(sim.s.as_run.since(saved_seq).cloned());
            saved_seq = sim.s.as_run.next_seq();
            let s = &sim.s;
            let kept = s.config.automation.filler_played_kept as usize;
            let filler = s
                .playlists
                .get(s.filler.as_ref().unwrap().playlist)
                .unwrap();
            assert!(filler.entries.len() <= kept + 25, "filler list {t}");
            let a = s.automation.as_ref().unwrap();
            let schedule = s.playlists.get(a.playlist).unwrap();
            assert!(schedule.entries.len() <= 3, "schedule list {t}");
            assert!(
                a.started.len() <= 3 * 24,
                "started {t}: {}",
                a.started.len()
            );
            assert!(a.missed.len() <= 3 * 48, "missed {t}: {}", a.missed.len());
            let yesterday = t.date().pred_opt().unwrap();
            assert!(
                s.as_run.entries().all(|e| e.date >= yesterday),
                "window {t}"
            );
            let rows = count(&store, "SELECT COUNT(*) FROM as_run");
            peak_rows = peak_rows.max(rows);
            let cutoff = t.date().checked_sub_days(Days::new(2)).unwrap();
            let old = count(
                &store,
                &format!(
                    "SELECT COUNT(*) FROM as_run WHERE date < {}",
                    cutoff.num_days_from_ce()
                ),
            );
            assert_eq!(old, 0, "rows older than the keep days at {t}");
        }
    }
    // Four days of entries at most stay in the table, whatever the run.
    let per_day = 23 * 2 + 600;
    assert!(peak_rows <= 4 * per_day, "peak {peak_rows}");
    // Every block of the run started on time.
    let mut missing = Vec::new();
    for d in 0..DAYS {
        let date = start_day().checked_add_days(Days::new(d)).unwrap();
        for block in sim.s.schedule.day(date) {
            let scheduled = date.and_time(
                chrono::NaiveTime::from_num_seconds_from_midnight_opt(block.start.secs(), 0)
                    .unwrap(),
            );
            if scheduled >= end {
                continue;
            }
            let aired = log.iter().find(|e| {
                e.source == AsRunSource::Schedule
                    && e.outcome == AsRunOutcome::Aired
                    && e.item
                        .is_some_and(|i| i.date == date && i.block == block.id && i.index == 0)
            });
            match aired.and_then(|e| e.started) {
                Some(started) if (started - scheduled).num_seconds() <= STEP_SECS => {}
                other => missing.push((scheduled, other)),
            }
        }
    }
    assert!(missing.is_empty(), "blocks not on time: {missing:?}");
    assert!(
        !log.iter().any(|e| e.outcome == AsRunOutcome::Missed),
        "missed items in a run that never stopped"
    );
}
