//! Local time for the schedule (automation spec AU5, AU14): a time of day,
//! and the zone that turns a local date and time into a `WallTime`. The
//! model never reads the zone itself: the caller passes it in.

use std::fmt;

use chrono::{NaiveDateTime, TimeZone};
use serde::{Deserialize, Serialize};

use crate::wall_time::WallTime;

pub const SECS_PER_DAY: u32 = 86_400;

/// Seconds after local midnight, `0..SECS_PER_DAY`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct TimeOfDay(u32);

impl TimeOfDay {
    pub fn new(secs: u32) -> Option<Self> {
        (secs < SECS_PER_DAY).then_some(Self(secs))
    }

    pub fn hms(h: u32, m: u32, s: u32) -> Option<Self> {
        if h >= 24 || m >= 60 || s >= 60 {
            return None;
        }
        Self::new(h * 3600 + m * 60 + s)
    }

    pub fn secs(self) -> u32 {
        self.0
    }

    /// `H:MM` or `HH:MM:SS`.
    pub fn parse(text: &str) -> Option<Self> {
        let mut parts = text.trim().split(':');
        let mut number = |required: bool| -> Option<Option<u32>> {
            match parts.next() {
                Some(p)
                    if !p.is_empty() && p.len() <= 2 && p.bytes().all(|b| b.is_ascii_digit()) =>
                {
                    p.parse().ok().map(Some)
                }
                None if !required => Some(None),
                _ => None,
            }
        };
        let h = number(true)??;
        let m = number(true)??;
        let s = number(false)?.unwrap_or(0);
        if parts.next().is_some() {
            return None;
        }
        Self::hms(h, m, s)
    }
}

impl fmt::Display for TimeOfDay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.0;
        write!(f, "{:02}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
    }
}

/// Turns a local date and time into a moment (AU14): `None` when the local
/// time does not exist (the spring change), the earlier moment when it
/// occurs twice (the autumn change).
pub trait LocalZone {
    fn earliest(&self, local: NaiveDateTime) -> Option<WallTime>;
}

fn earliest_in<Tz: TimeZone>(zone: &Tz, local: NaiveDateTime) -> Option<WallTime> {
    zone.from_local_datetime(&local)
        .earliest()
        .map(|t| WallTime(t.timestamp_millis()))
}

impl LocalZone for chrono::Local {
    fn earliest(&self, local: NaiveDateTime) -> Option<WallTime> {
        earliest_in(self, local)
    }
}

impl LocalZone for chrono::FixedOffset {
    fn earliest(&self, local: NaiveDateTime) -> Option<WallTime> {
        earliest_in(self, local)
    }
}
