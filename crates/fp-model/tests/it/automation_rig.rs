#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! A small test rig for running the schedule: a Base template with given
//! blocks, music in the library, the filler and the schedule's player, and
//! a fake engine that reports what the model asked for.

use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{NaiveDate, NaiveDateTime};
use fp_model::automation::{ItemRef, tick};
use fp_model::local_time::TimeOfDay;
use fp_model::schedule::{Block, Item, ensure_default};
use fp_model::{
    AppState, BlockId, Command, Config, EngineAction, EngineEvent, PlayerId, TemplateId, Track,
    TrackId, TrackKind, Transport, WallTime, apply, on_event,
};

pub const SONG_SECS: f64 = 180.0;

pub fn today() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 11, 17).unwrap()
}

pub fn tod(h: u32, m: u32) -> TimeOfDay {
    TimeOfDay::hms(h, m, 0).unwrap()
}

pub fn song(n: u64) -> Item {
    Item::Track {
        track: TrackId(n),
        time_critical: false,
    }
}

pub fn hourly_id(n: u64) -> Item {
    Item::Track {
        track: TrackId(n),
        time_critical: true,
    }
}

pub struct Rig {
    pub s: AppState,
    pub base: TemplateId,
    pub blocks: HashMap<(u32, u32), BlockId>,
    /// Every engine action sent so far.
    pub log: Vec<EngineAction>,
}

impl Rig {
    /// Base with `blocks` (hour, minute, items); tracks 1–40 are music of
    /// `SONG_SECS`, 900+ are free for the test. Separation rules are off.
    pub fn new(blocks: Vec<(u32, u32, Vec<Item>)>) -> Self {
        let mut s = AppState::new(Config::default(), "P");
        s.config.automation.track_separation_mins = 0;
        s.config.automation.track_separation_percent = 0;
        s.config.automation.artist_separation_mins = 0;
        for n in 1..=40 {
            let mut t = Track::new(TrackId(n), PathBuf::from(format!("/m/{n}.wav")));
            t.duration_secs = SONG_SECS;
            t.kind = TrackKind::Music;
            s.library.insert(t);
        }
        s.ids.observe(1000);
        fp_model::rotation::ensure_default(&mut s, "Music");
        fp_model::filler::ensure(&mut s, "Music filler");
        fp_model::automation::ensure(&mut s, "Schedule");
        ensure_default(&mut s, "Base");
        let base = s.schedule.templates()[0].id;
        let mut map = HashMap::new();
        for (h, m, items) in blocks {
            apply(
                &mut s,
                Command::AddBlock {
                    template: base,
                    start: tod(h, m),
                },
            )
            .unwrap();
            let id = s
                .schedule
                .template(base)
                .unwrap()
                .blocks
                .iter()
                .find(|b| b.start == tod(h, m))
                .unwrap()
                .id;
            apply(
                &mut s,
                Command::UpdateBlock {
                    template: base,
                    block: Block {
                        id,
                        start: tod(h, m),
                        items,
                    },
                },
            )
            .unwrap();
            map.insert((h, m), id);
        }
        let mut rig = Rig {
            s,
            base,
            blocks: map,
            log: Vec::new(),
        };
        rig.set_time(today().and_hms_opt(0, 0, 0).unwrap());
        rig
    }

    pub fn block(&self, h: u32, m: u32) -> BlockId {
        self.blocks[&(h, m)]
    }

    pub fn item(&self, date: NaiveDate, h: u32, m: u32, index: usize) -> ItemRef {
        ItemRef {
            date,
            block: self.block(h, m),
            index,
        }
    }

    pub fn schedule_player(&self) -> PlayerId {
        self.s.automation.as_ref().unwrap().player
    }

    pub fn filler_player(&self) -> PlayerId {
        self.s.filler.as_ref().unwrap().player
    }

    fn set_time(&mut self, at: NaiveDateTime) {
        self.s.local_now = at;
        // The wall clock follows the local one (UTC here).
        self.s.wall_now = WallTime(at.and_utc().timestamp_millis());
    }

    /// Moves the clock forward to `at` the way time passes, ticking at
    /// least every 30 s (no step is a clock jump); returns the actions of
    /// the last tick. A step back is a jump.
    pub fn at(&mut self, at: NaiveDateTime) -> Vec<EngineAction> {
        let step = chrono::TimeDelta::seconds(30);
        while self.s.local_now + step < at {
            let t = self.s.local_now + step;
            self.jump(t);
        }
        self.jump(at)
    }

    /// Sets the clock to `at` at once (a clock jump when far) and ticks.
    pub fn jump(&mut self, at: NaiveDateTime) -> Vec<EngineAction> {
        self.set_time(at);
        let (actions, _) = tick(&mut self.s);
        self.engine(&actions);
        actions
    }

    /// Jumps to today's `h:m:s`.
    pub fn jump_hms(&mut self, h: u32, m: u32, s: u32) -> Vec<EngineAction> {
        self.jump(today().and_hms_opt(h, m, s).unwrap())
    }

    /// Ticks at today's `h:m:s`.
    pub fn at_hms(&mut self, h: u32, m: u32, s: u32) -> Vec<EngineAction> {
        self.at(today().and_hms_opt(h, m, s).unwrap())
    }

    pub fn apply(&mut self, command: Command) -> Vec<EngineAction> {
        let actions = apply(&mut self.s, command).unwrap();
        self.engine(&actions);
        actions
    }

    /// The fake engine: a start is reported at once, a fade out ends at once.
    pub fn engine(&mut self, actions: &[EngineAction]) {
        self.log.extend(actions.iter().cloned());
        for action in actions {
            let event = match action {
                EngineAction::StartCurrent { player, request }
                | EngineAction::Crossfade {
                    player, request, ..
                } => EngineEvent::TransitionStarted {
                    player: *player,
                    entry: request.entry,
                    normalization: fp_model::Normalization::default(),
                },
                EngineAction::FadeOutAndStop { player, .. } => {
                    let Some(entry) = self.player(*player).current else {
                        continue;
                    };
                    EngineEvent::ReachedEnd {
                        player: *player,
                        entry,
                    }
                }
                _ => continue,
            };
            let more = on_event(&mut self.s, event);
            self.engine(&more);
        }
    }

    /// The current entry of `player` ends: the engine starts its next (a
    /// continuous player with a next and no stop) or reports the end.
    pub fn end_current(&mut self, player: PlayerId) {
        let p = self.player(player).clone();
        let Some(current) = p.current else { return };
        let continuous = p.mode == fp_model::PlayMode::Continuous;
        let event = match (p.next, p.stop_after_current) {
            (Some(next), false) if continuous => EngineEvent::TransitionStarted {
                player,
                entry: next,
                normalization: fp_model::Normalization::default(),
            },
            _ => EngineEvent::ReachedEnd {
                player,
                entry: current,
            },
        };
        let more = on_event(&mut self.s, event);
        self.engine(&more);
    }

    pub fn player(&self, id: PlayerId) -> &fp_model::PlayerState {
        self.s.players.iter().find(|p| p.id == id).unwrap()
    }

    pub fn playing(&self, id: PlayerId) -> Option<TrackId> {
        let p = self.player(id);
        if p.transport == Transport::Stopped {
            return None;
        }
        p.current
            .and_then(|e| self.s.playlists.entry(e))
            .map(|e| e.track)
    }

    pub fn missed(&self) -> &std::collections::BTreeSet<ItemRef> {
        &self.s.automation.as_ref().unwrap().missed
    }
}
