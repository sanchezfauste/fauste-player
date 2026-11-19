#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! A simulated broadcast night, end to end: the real engine on the
//! offline backend, the conductor, the model and automation, on a clock the
//! test runs (`Conductor::tick_at`). A station library (current hits, gold,
//! jingles, spots, hourly IDs), a rotation, templates with hourly IDs and
//! spot breaks, a live show on a player, operator actions on the Log, a
//! broken spot file and midnight. The checks are what a listener and the
//! station's traffic department would notice: dead air, blocks off their
//! time, hourly IDs off their hour, repeats, fragments, an incomplete
//! as-run log.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{NaiveDate, NaiveDateTime, TimeDelta, Weekday};
use fp_backends::{AudioBackend, OfflineBackend, OfflineDevice};
use fp_engine::conductor::{Conductor, ConductorHandle};
use fp_engine::engine::{Engine, EngineSettings};
use fp_engine::worker::{SampleSource, SourceOpener};
use fp_model::as_run::{AsRunOutcome, AsRunSource};
use fp_model::local_time::TimeOfDay;
use fp_model::rotation::{RotationStep, StepSource};
use fp_model::schedule::{Block, Item};
use fp_model::{
    AppState, BlockId, CategoryId, Command, Config, PlayerId, RotationId, Track, TrackId,
    TrackKind, Transport, WallTime,
};

const RATE: u32 = 8_000;
/// One conductor tick and one rendered block: 20 ms.
const BLOCK: usize = 160;
const TICK: Duration = Duration::from_millis(20);

/// A source of `frames` frames of a steady level: silence in the output
/// means nothing plays.
struct Steady {
    left: u64,
    fails: bool,
}

impl SampleSource for Steady {
    fn next_block(&mut self, out: &mut Vec<f32>) -> Result<bool, String> {
        if self.left == 0 {
            return if self.fails {
                Err("read error".to_owned())
            } else {
                Ok(false)
            };
        }
        let n = self.left.min(512);
        for _ in 0..n {
            out.push(0.1);
            out.push(0.1);
        }
        self.left -= n;
        Ok(true)
    }
}

/// Opens `/sim/<name>` with the length the library gives it; a path in
/// `broken` fails after a second of audio (a damaged file).
fn opener(lengths: Arc<HashMap<PathBuf, (f64, bool)>>) -> SourceOpener {
    Arc::new(move |path: &Path, from_secs, rate| {
        let (len, broken) = lengths
            .get(path)
            .copied()
            .ok_or_else(|| format!("cannot open {}", path.display()))?;
        let len = if broken { 1.0 } else { len };
        let left = ((len - from_secs).max(0.0) * f64::from(rate)).round() as u64;
        Ok(Box::new(Steady {
            left,
            fails: broken,
        }) as Box<dyn SampleSource>)
    })
}

/// xorshift, deterministic across runs.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (self.next() % 10_000) as f64 / 10_000.0 * (hi - lo)
    }
}

struct Station {
    spots: CategoryId,
    day: RotationId,
    /// A spot whose file is damaged (decoding fails a second in).
    broken_spot: TrackId,
    lengths: HashMap<PathBuf, (f64, bool)>,
}

fn add_category(s: &mut AppState, name: &str, kind: TrackKind) -> CategoryId {
    fp_model::apply(
        s,
        Command::CreateCategory {
            parent: None,
            name: name.into(),
            color: None,
            kind,
        },
    )
    .unwrap();
    s.categories.iter().last().unwrap().id
}

fn station_state(seed: u64) -> (AppState, Station) {
    let mut config = Config::default();
    config.players.count = 1;
    config.outputs.backend = Some("offline".into());
    config.outputs.sample_rate = RATE;
    config.outputs.buffer_frames = BLOCK as u32;
    // The render runs far faster than real time: the workers keep a minute
    // ahead, and a source starts with ten seconds ready.
    config.tuning.prebuffer_secs = 60.0;
    config.tuning.ready_threshold_ms = 10_000.0;
    let mut s = AppState::new(config, "Live");
    s.ids.observe(10_000);
    let current = add_category(&mut s, "Current", TrackKind::Music);
    let gold = add_category(&mut s, "Gold", TrackKind::Music);
    let jingles = add_category(&mut s, "Jingles", TrackKind::Jingle);
    let spots = add_category(&mut s, "Spots", TrackKind::Ad);
    let ids = add_category(&mut s, "Station IDs", TrackKind::HourlyId);
    let mut rng = Rng(seed);
    let mut lengths = HashMap::new();
    let mut n = 0u64;
    let mut add = |s: &mut AppState,
                   cat: CategoryId,
                   kind: TrackKind,
                   artist: String,
                   len: f64,
                   rng: &mut Rng|
     -> TrackId {
        n += 1;
        let path = PathBuf::from(format!("/sim/{n}"));
        let mut t = Track::new(TrackId(n), path.clone());
        t.title = format!("Title {n}");
        t.artist = artist;
        t.duration_secs = len;
        t.kind = kind;
        t.category = Some(cat);
        t.analyzed = true;
        if kind == TrackKind::Music {
            // Songs have a segue point a few seconds before their end.
            t.markers.segue_start = Some(fp_model::Marker {
                secs: len - rng.range(2.0, 8.0),
                source: fp_model::MarkerSource::Auto,
            });
        }
        lengths.insert(path, (len, false));
        s.library.insert(t);
        TrackId(n)
    };
    for i in 0..45 {
        let len = rng.range(150.0, 320.0);
        add(
            &mut s,
            current,
            TrackKind::Music,
            format!("Artist {}", i % 22),
            len,
            &mut rng,
        );
    }
    for i in 0..35 {
        let len = rng.range(160.0, 300.0);
        add(
            &mut s,
            gold,
            TrackKind::Music,
            format!("Artist {}", 22 + i % 15),
            len,
            &mut rng,
        );
    }
    for _ in 0..8 {
        let len = rng.range(4.0, 9.0);
        add(
            &mut s,
            jingles,
            TrackKind::Jingle,
            String::new(),
            len,
            &mut rng,
        );
    }
    let mut broken_spot = TrackId(0);
    for i in 0..14 {
        let len = [20.0, 30.0, 30.0, 40.0][i % 4];
        broken_spot = add(
            &mut s,
            spots,
            TrackKind::Ad,
            format!("Advertiser {i}"),
            len,
            &mut rng,
        );
    }
    for _ in 0..2 {
        add(
            &mut s,
            ids,
            TrackKind::HourlyId,
            String::new(),
            8.0,
            &mut rng,
        );
    }
    // The last spot's file is damaged; the library does not know it yet.
    let bad = s.library.get(broken_spot).unwrap().path.clone();
    lengths.get_mut(&bad).unwrap().1 = true;

    fp_model::rotation::ensure_default(&mut s, "Music");
    fp_model::filler::ensure(&mut s, "Music fill");
    fp_model::automation::ensure(&mut s, "Log");
    fp_model::schedule::ensure_default(&mut s, "Base");
    let step = |source, min, max| RotationStep { source, min, max };
    fp_model::apply(
        &mut s,
        Command::CreateRotation {
            name: "Day".into(),
            steps: vec![
                step(StepSource::Category(current), 2, 2),
                step(StepSource::Category(jingles), 1, 1),
                step(StepSource::Category(gold), 1, 1),
            ],
        },
    )
    .unwrap();
    let day = s.rotations.iter().last().unwrap().id;
    fp_model::apply(
        &mut s,
        Command::CreateRotation {
            name: "Night".into(),
            steps: vec![
                step(StepSource::Category(gold), 1, 2),
                step(StepSource::Category(current), 1, 1),
            ],
        },
    )
    .unwrap();
    let night = s.rotations.iter().last().unwrap().id;
    let base = s.schedule.templates()[0].id;
    fp_model::apply(
        &mut s,
        Command::CreateTemplate {
            name: "Late night".into(),
        },
    )
    .unwrap();
    let late = s.schedule.templates().last().unwrap().id;
    let st = Station {
        spots,
        day,
        broken_spot,
        lengths,
    };
    let cat = |category| Item::Category {
        category,
        time_critical: false,
    };
    // Base (Tuesday): the hour opens with the ID and a jingle, spots at :15
    // and :45, a jingle at :30.
    let base_blocks = vec![
        (
            0,
            vec![cat(ids), cat(jingles), Item::Rotation { rotation: day }],
        ),
        (
            15,
            vec![
                cat(jingles),
                cat(spots),
                cat(spots),
                cat(spots),
                cat(jingles),
                Item::Rotation { rotation: day },
            ],
        ),
        (30, vec![cat(jingles), Item::Rotation { rotation: day }]),
        (
            45,
            vec![
                cat(spots),
                cat(spots),
                cat(jingles),
                Item::Rotation { rotation: day },
            ],
        ),
    ];
    // Late night (Wednesday): the ID on the hour, one spot break at :30.
    let late_blocks = vec![
        (
            0,
            vec![cat(ids), cat(jingles), Item::Rotation { rotation: night }],
        ),
        (
            30,
            vec![
                cat(spots),
                cat(spots),
                cat(jingles),
                Item::Rotation { rotation: night },
            ],
        ),
    ];
    for (template, blocks) in [(base, base_blocks), (late, late_blocks)] {
        for h in 0..24 {
            for (m, items) in &blocks {
                let start = TimeOfDay::hms(h, *m, 0).unwrap();
                fp_model::apply(&mut s, Command::AddBlock { template, start }).unwrap();
                let id = s
                    .schedule
                    .template(template)
                    .unwrap()
                    .blocks
                    .iter()
                    .find(|b| b.start == start)
                    .unwrap()
                    .id;
                fp_model::apply(
                    &mut s,
                    Command::UpdateBlock {
                        template,
                        block: Block {
                            id,
                            start,
                            items: items.clone(),
                        },
                    },
                )
                .unwrap();
            }
        }
    }
    for weekday in [
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Thu,
        Weekday::Fri,
        Weekday::Sat,
        Weekday::Sun,
    ] {
        fp_model::apply(
            &mut s,
            Command::SetWeekTemplate {
                weekday,
                template: base,
            },
        )
        .unwrap();
    }
    fp_model::apply(
        &mut s,
        Command::SetWeekTemplate {
            weekday: Weekday::Wed,
            template: late,
        },
    )
    .unwrap();
    (s, st)
}

/// What aired on one automation player: the track and when it started.
#[derive(Debug, Clone)]
struct Airing {
    player: &'static str,
    track: TrackId,
    entry: fp_model::EntryId,
    started: NaiveDateTime,
    /// When its fade (a hand-over or a stop) started, if it was faded.
    faded: Option<NaiveDateTime>,
    ended: Option<NaiveDateTime>,
}

struct Sim {
    conductor: Conductor,
    handle: ConductorHandle,
    device: OfflineDevice,
    now: Instant,
    local: NaiveDateTime,
    log_player: PlayerId,
    fill_player: PlayerId,
    live_player: PlayerId,
    airings: Vec<Airing>,
    /// Open airings per player.
    on: HashMap<PlayerId, usize>,
    /// Silence while automation runs: (from, length in ms).
    gaps: Vec<(NaiveDateTime, u64)>,
    silent_since: Option<NaiveDateTime>,
    refused: Vec<(NaiveDateTime, String)>,
    /// Ticks where more than two automation sources sounded.
    stacked: Vec<NaiveDateTime>,
}

fn date() -> NaiveDate {
    // A Tuesday.
    NaiveDate::from_ymd_opt(2026, 11, 17).unwrap()
}

impl Sim {
    fn new(seed: u64, start: NaiveDateTime) -> (Sim, Station) {
        let (state, st) = station_state(seed);
        let backend = OfflineBackend::new();
        let device = backend.add_device("main", 2);
        let settings = EngineSettings::from_config(&state.config);
        let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
        let engine = Engine::new(backends, settings, opener(Arc::new(st.lengths.clone())));
        let now = Instant::now();
        let log_player = state.automation.as_ref().unwrap().player;
        let fill_player = state.filler.as_ref().unwrap().player;
        let live_player = state.players[0].id;
        let (mut conductor, handle) = Conductor::new(state, Vec::new(), engine, now);
        conductor.reseed(seed);
        let mut sim = Sim {
            conductor,
            handle,
            device,
            now,
            local: start,
            log_player,
            fill_player,
            live_player,
            airings: Vec::new(),
            on: HashMap::new(),
            gaps: Vec::new(),
            silent_since: None,
            refused: Vec::new(),
            stacked: Vec::new(),
        };
        sim.tick();
        (sim, st)
    }

    fn wall(&self) -> WallTime {
        WallTime(self.local.and_utc().timestamp_millis())
    }

    fn state(&self) -> &AppState {
        self.conductor.state()
    }

    fn running(&self) -> bool {
        self.state().automation.as_ref().is_some_and(|a| a.running)
    }

    fn send(&mut self, command: Command) {
        assert!(self.handle.send(command));
    }

    fn tick(&mut self) {
        let wall = self.wall();
        self.conductor.tick_at(self.now, wall, self.local);
        // Before a block is rendered, every source is filled (the render is
        // far faster than real time).
        let deadline = Instant::now() + Duration::from_secs(60);
        while self.conductor.engine().unsettled_sources() > 0 {
            assert!(Instant::now() < deadline, "the sources never settled");
            std::thread::sleep(Duration::from_micros(200));
            self.conductor.tick_at(self.now, wall, self.local);
        }
        while let Ok(e) = self.handle.rejected.try_recv() {
            self.refused.push((self.local, format!("{e:?}")));
        }
        self.observe();
    }

    /// Notes what started and ended on the automation players and the live
    /// player.
    fn observe(&mut self) {
        let local = self.local;
        let players = [
            (self.log_player, "log"),
            (self.fill_player, "fill"),
            (self.live_player, "live"),
        ];
        let mut sounding = 0;
        for (id, name) in players {
            let p = self.conductor.state().player(id).unwrap().clone();
            let playing = p.transport != Transport::Stopped;
            let current = p.current.filter(|_| playing);
            if name != "live" && playing {
                sounding += 1;
                if p.fading {
                    sounding += 1;
                }
            }
            let open = self.on.get(&id).copied();
            let same = open.is_some_and(|i| Some(self.airings[i].entry) == current);
            if !same {
                if let Some(i) = open {
                    self.airings[i].ended = Some(local);
                    self.on.remove(&id);
                }
                if let Some(entry) = current {
                    let track = self.conductor.state().playlists.entry(entry).unwrap().track;
                    self.airings.push(Airing {
                        player: name,
                        track,
                        entry,
                        started: local,
                        faded: None,
                        ended: None,
                    });
                    self.on.insert(id, self.airings.len() - 1);
                }
            }
            if let Some(i) = self.on.get(&id).copied()
                && p.fade_stopping_now()
                && self.airings[i].faded.is_none()
            {
                self.airings[i].faded = Some(local);
            }
        }
        // A crossfade into the next song counts the outgoing one: at most
        // a Log item, a fill song and one fading out of either.
        if sounding > 3 {
            self.stacked.push(local);
        }
    }

    /// Renders one block and moves the clock on.
    fn step(&mut self) {
        let out = self.device.render(BLOCK).unwrap();
        let silent = out.iter().all(|v| *v == 0.0);
        let on_air = self.running();
        match (silent && on_air, self.silent_since) {
            (true, None) => self.silent_since = Some(self.local),
            (false, Some(from)) => {
                let ms = (self.local - from).num_milliseconds() as u64;
                self.gaps.push((from, ms));
                self.silent_since = None;
            }
            _ => {}
        }
        self.now += TICK;
        self.local += TimeDelta::milliseconds(20);
        self.tick();
    }

    fn run_until(&mut self, at: NaiveDateTime) {
        while self.local < at {
            self.step();
        }
    }

    fn run_until_hms(&mut self, d: NaiveDate, h: u32, m: u32, s: u32) {
        self.run_until(d.and_hms_opt(h, m, s).unwrap());
    }
}

trait FadeStopping {
    fn fade_stopping_now(&self) -> bool;
}

impl FadeStopping for fp_model::PlayerState {
    fn fade_stopping_now(&self) -> bool {
        self.fading && self.fade_stopping()
    }
}

/// The night: 22:40 Tuesday to 01:20 Wednesday.
#[test]
#[ignore = "long simulation: a few minutes; run with --ignored"]
fn a_simulated_night_on_air() {
    let start = date().and_hms_opt(22, 40, 0).unwrap();
    let seed = std::env::var("SIM_SEED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0x5eed_0001);
    let (mut sim, st) = Sim::new(seed, start);
    let d0 = date();
    let d1 = d0.succ_opt().unwrap();

    // 22:40:05 the operator arrives and presses Play on the Log.
    sim.run_until_hms(d0, 22, 40, 5);
    sim.send(Command::Play(sim.log_player));
    sim.run_until_hms(d0, 23, 1, 0);

    // 23:01: after the hourly ID, Stop at end on the Log; the live show.
    sim.send(Command::ToggleStopAfterCurrent(sim.log_player));
    sim.run_until_hms(d0, 23, 4, 0);
    let live_list = sim.state().players[0].playlist;
    let show: Vec<TrackId> = vec![TrackId(3), TrackId(50)];
    sim.send(Command::InsertTracks {
        playlist: live_list,
        index: 0,
        tracks: show,
    });
    sim.run_until_hms(d0, 23, 4, 1);
    sim.send(Command::Play(sim.live_player));
    // 23:14:30 the operator plays the spot break by hand: back to the
    // clock, Play.
    sim.run_until_hms(d0, 23, 14, 30);
    sim.send(Command::ClearScheduleNext);
    sim.run_until_hms(d0, 23, 14, 31);
    sim.send(Command::Play(sim.log_player));
    sim.run_until_hms(d0, 23, 40, 0);

    // 23:40 a block is added live today at 23:52: two spots.
    sim.send(Command::AddBlockOnDate {
        date: d0,
        start: TimeOfDay::hms(23, 52, 0).unwrap(),
    });
    sim.run_until_hms(d0, 23, 40, 1);
    let added: BlockId = sim
        .state()
        .schedule
        .day(d0)
        .into_iter()
        .find(|b| b.start == TimeOfDay::hms(23, 52, 0).unwrap())
        .map(|b| b.id)
        .expect("the added block");
    sim.send(Command::ChangeBlockOnDate {
        date: d0,
        block: Block {
            id: added,
            start: TimeOfDay::hms(23, 52, 0).unwrap(),
            items: vec![
                Item::Category {
                    category: st.spots,
                    time_critical: false,
                },
                Item::Track {
                    track: st.broken_spot,
                    time_critical: false,
                },
                Item::Rotation { rotation: st.day },
            ],
        },
    });
    // Midnight: Wednesday's template.
    sim.run_until_hms(d1, 0, 20, 0);
    // 00:20 a pause of ninety seconds, then Play.
    sim.send(Command::Pause(sim.log_player));
    sim.run_until_hms(d1, 0, 21, 30);
    sim.send(Command::Play(sim.log_player));
    // 00:40 Stop, and Play again three minutes later.
    sim.run_until_hms(d1, 0, 40, 0);
    sim.send(Command::Stop(sim.log_player));
    sim.run_until_hms(d1, 0, 43, 0);
    sim.send(Command::Play(sim.log_player));
    sim.run_until_hms(d1, 1, 20, 0);

    report(&sim, &st, d0, d1);
}

/// A whole day with nobody at the station: 05:55 Tuesday to 06:05
/// Wednesday, Play pressed once.
#[test]
#[ignore = "long simulation: about a minute in release; run with --ignored"]
fn a_simulated_day_unattended() {
    let start = date().and_hms_opt(5, 55, 0).unwrap();
    let seed = std::env::var("SIM_SEED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0x5eed_0002);
    let (mut sim, st) = Sim::new(seed, start);
    let d0 = date();
    let d1 = d0.succ_opt().unwrap();
    sim.run_until_hms(d0, 5, 55, 5);
    sim.send(Command::Play(sim.log_player));
    sim.run_until_hms(d1, 6, 5, 0);
    // Nothing is missed with nobody touching anything, but the spot whose
    // file is damaged.
    let s = sim.state();
    let a = s.automation.as_ref().unwrap();
    let missed: Vec<String> = s
        .as_run
        .entries()
        .filter(|e| e.outcome == AsRunOutcome::Missed)
        .filter(|e| e.item.and_then(|i| a.chosen.get(&i)) != Some(&st.broken_spot))
        .map(|e| format!("{e:?} chosen {:?}", e.item.and_then(|i| a.chosen.get(&i))))
        .collect();
    report(&sim, &st, d0, d1);
    assert!(missed.is_empty(), "{missed:#?}");
}

fn report(sim: &Sim, st: &Station, d0: NaiveDate, d1: NaiveDate) {
    let s = sim.state();
    let name = |t: TrackId| {
        let tr = s.library.get(t).unwrap();
        format!("{:?}#{} {}", tr.kind, t.0, tr.artist)
    };
    let mut problems: Vec<String> = Vec::new();
    println!("--- airings ---");
    for a in &sim.airings {
        println!(
            "{} {:5} {:28} {:>7.1}s{}{}",
            a.started.format("%H:%M:%S%.3f"),
            a.player,
            name(a.track),
            a.ended
                .map_or(0.0, |e| (e - a.started).num_milliseconds() as f64 / 1000.0),
            a.faded
                .map(|f| format!(" faded at {}", f.format("%H:%M:%S%.3f")))
                .unwrap_or_default(),
            if a.ended.is_none() { " (on air)" } else { "" }
        );
    }
    println!("--- as-run ---");
    for e in s.as_run.entries() {
        println!(
            "{:?} {:?} sched {:?} started {:?} {:?}",
            e.source,
            e.outcome,
            e.scheduled.map(|t| t.format("%H:%M").to_string()),
            e.started.map(|t| t.format("%H:%M:%S").to_string()),
            e.track.map(name)
        );
    }
    // Dead air while automation ran.
    for (from, ms) in &sim.gaps {
        if *ms > 100 {
            problems.push(format!(
                "dead air {ms} ms at {}",
                from.format("%H:%M:%S%.3f")
            ));
        }
    }
    for at in &sim.stacked {
        problems.push(format!("more than three sources at {at}"));
    }
    for (at, e) in &sim.refused {
        problems.push(format!("refused at {at}: {e}"));
    }
    // Blocks: each block of the window either started (its first aired
    // item within a tick of its start when the clock reached it with
    // automation running) or was missed.
    let mut starts: BTreeMap<NaiveDateTime, Vec<&fp_model::as_run::AsRunEntry>> = BTreeMap::new();
    for e in s.as_run.entries() {
        if e.source == AsRunSource::Schedule
            && let Some(at) = e.scheduled
        {
            starts.entry(at).or_default().push(e);
        }
    }
    println!("--- blocks ---");
    for (at, entries) in &starts {
        let first = entries
            .iter()
            .filter(|e| e.outcome == AsRunOutcome::Aired)
            .filter_map(|e| e.started)
            .min();
        let missed = entries
            .iter()
            .filter(|e| e.outcome == AsRunOutcome::Missed)
            .count();
        println!(
            "{} first {:?} aired {} missed {}",
            at.format("%d %H:%M"),
            first.map(|f| (f - *at).num_milliseconds()),
            entries.len() - missed,
            missed
        );
    }
    // Hourly IDs: on the hour within the tolerance, or missed.
    let tolerance = TimeDelta::seconds(i64::from(s.config.automation.late_tolerance_secs));
    for e in s.as_run.entries() {
        let Some(track) = e.track else { continue };
        if s.library.get(track).unwrap().kind != TrackKind::HourlyId {
            continue;
        }
        if let (Some(sched), Some(started)) = (e.scheduled, e.started)
            && started > sched + tolerance
        {
            problems.push(format!(
                "hourly ID of {sched} aired late at {}",
                started.format("%H:%M:%S")
            ));
        }
        // AU24: the fade ends at the block's start; nothing starts early.
        if let (Some(sched), Some(started)) = (e.scheduled, e.started)
            && started < sched - TimeDelta::milliseconds(100)
        {
            problems.push(format!(
                "hourly ID of {sched} aired early at {}",
                started.format("%H:%M:%S%.3f")
            ));
        }
    }
    // Artist and track separation among music.
    let sep = TimeDelta::minutes(i64::from(s.config.automation.artist_separation_mins));
    let music: Vec<&Airing> = sim
        .airings
        .iter()
        .filter(|a| a.player != "live")
        .filter(|a| s.library.get(a.track).unwrap().kind == TrackKind::Music)
        .collect();
    for (i, a) in music.iter().enumerate() {
        for b in music.iter().skip(i + 1) {
            if b.started - a.started >= sep {
                break;
            }
            let (ta, tb) = (
                s.library.get(a.track).unwrap(),
                s.library.get(b.track).unwrap(),
            );
            if ta.artist == tb.artist && b.faded.is_some() {
                // The last song before a block: a fit may relax the artist
                // rule (plan 5's order: the fit's class first).
                println!(
                    "note: artist {} at {} and {} (fitted before a block)",
                    ta.artist,
                    a.started.format("%H:%M:%S"),
                    b.started.format("%H:%M:%S")
                );
            } else if ta.artist == tb.artist {
                problems.push(format!(
                    "artist {} at {} ({}) and {} ({})",
                    ta.artist,
                    a.started.format("%H:%M:%S"),
                    a.player,
                    b.started.format("%H:%M:%S"),
                    b.player
                ));
            }
        }
    }
    // Fragments (AU25): a fill song faded at a hand-over after less than
    // `min_song_play_secs`, or losing more than `max_fade_cut_secs`.
    let min_play = f64::from(s.config.automation.min_song_play_secs);
    // The cut counts from the block's start, where the fade ends.
    let fade = f64::from(s.config.automation.block_fade_ms) / 1000.0;
    let max_cut = f64::from(s.config.automation.max_fade_cut_secs) + fade;
    let shortest = s
        .library
        .iter()
        .filter(|t| t.kind == TrackKind::Music)
        .map(|t| t.duration_secs)
        .fold(f64::INFINITY, f64::min);
    for a in sim.airings.iter().filter(|a| a.player == "fill") {
        let Some(faded) = a.faded else { continue };
        let played = (faded - a.started).num_milliseconds() as f64 / 1000.0;
        let len = s.library.get(a.track).unwrap().duration_secs;
        let cut = len - played;
        // With less than the minimum before the fade, or a gap no song of
        // the library can close within the cut, no song fits, and the one
        // that cuts least plays (AU25).
        if played < min_play || played + max_cut < shortest {
            println!(
                "note: {} started {} with {played:.1}s before the fade",
                name(a.track),
                a.started.format("%H:%M:%S")
            );
        } else if cut > max_cut {
            problems.push(format!(
                "fill song {} started {} faded after {played:.1}s, losing {cut:.1}s",
                name(a.track),
                a.started.format("%H:%M:%S")
            ));
        }
    }
    // Everything the automation players aired is in the as-run log.
    let logged: Vec<(TrackId, NaiveDateTime)> = s
        .as_run
        .entries()
        .filter(|e| e.outcome == AsRunOutcome::Aired)
        .filter_map(|e| Some((e.track?, e.started?)))
        .collect();
    for a in sim.airings.iter().filter(|a| a.player != "live") {
        let found = logged
            .iter()
            .any(|(t, at)| *t == a.track && (*at - a.started).num_milliseconds().abs() <= 100);
        if !found && a.started.date() >= d0 {
            problems.push(format!(
                "{} aired at {} on {} but is not in the as-run log",
                name(a.track),
                a.started.format("%H:%M:%S"),
                a.player
            ));
        }
    }
    let underruns: u64 = [sim.log_player, sim.fill_player, sim.live_player]
        .iter()
        .map(|p| sim.conductor.engine().telemetry(*p).underruns)
        .sum();
    println!("underruns (the simulation outran a worker): {underruns}");
    let _ = (st, d1);
    println!("--- problems ({}) ---", problems.len());
    for p in &problems {
        println!("{p}");
    }
    assert!(problems.is_empty(), "{} problems", problems.len());
}
