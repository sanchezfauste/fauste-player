#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Automation spec AU5–AU7: the time of day, the local zone, items.

use std::path::PathBuf;

use chrono::{FixedOffset, NaiveDate, TimeZone, Utc};
use fp_model::local_time::{LocalZone, TimeOfDay};
use fp_model::schedule::{Item, check_items};
use fp_model::{
    AppState, CategoryId, Command, Config, ModelError, RotationId, Track, TrackId, TrackKind,
    WallTime, apply,
};

#[test]
fn time_of_day_bounds() {
    assert!(TimeOfDay::new(86_399).is_some());
    assert!(TimeOfDay::new(86_400).is_none());
    assert!(TimeOfDay::hms(24, 0, 0).is_none());
    assert!(TimeOfDay::hms(7, 60, 0).is_none());
    assert_eq!(
        TimeOfDay::hms(7, 30, 5).unwrap().secs(),
        7 * 3600 + 30 * 60 + 5
    );
}

#[test]
fn time_of_day_text() {
    let t = TimeOfDay::hms(7, 30, 0).unwrap();
    assert_eq!(t.to_string(), "07:30:00");
    assert_eq!(TimeOfDay::parse("7:30"), Some(t));
    assert_eq!(TimeOfDay::parse("07:30:00"), Some(t));
    assert_eq!(TimeOfDay::parse("25:00"), None);
    assert_eq!(TimeOfDay::parse("x"), None);
}

#[test]
fn fixed_offset_zone() {
    let zone = FixedOffset::east_opt(3600).unwrap();
    let local = NaiveDate::from_ymd_opt(2026, 3, 1)
        .unwrap()
        .and_hms_opt(10, 0, 0)
        .unwrap();
    let utc = Utc.with_ymd_and_hms(2026, 3, 1, 9, 0, 0).unwrap();
    assert_eq!(zone.earliest(local), Some(WallTime(utc.timestamp_millis())));
}

fn state() -> AppState {
    AppState::new(Config::default(), "P")
}

fn track(s: &mut AppState, id: u64, kind: TrackKind) -> TrackId {
    let mut t = Track::new(TrackId(id), PathBuf::from(format!("/m/{id}.wav")));
    t.kind = kind;
    s.library.insert(t);
    TrackId(id)
}

fn category(s: &mut AppState, kind: TrackKind) -> CategoryId {
    apply(
        s,
        Command::CreateCategory {
            parent: None,
            name: format!("{kind:?}"),
            color: None,
            kind,
        },
    )
    .unwrap();
    s.categories.iter().map(|c| c.id).max().unwrap()
}

#[test]
fn hourly_id_items_are_time_critical() {
    let mut s = state();
    let id = track(&mut s, 1, TrackKind::HourlyId);
    let song = track(&mut s, 2, TrackKind::Music);
    let ids = category(&mut s, TrackKind::HourlyId);
    let critical = |s: &AppState, item: Item| item.is_time_critical(s);
    assert!(critical(
        &s,
        Item::Track {
            track: id,
            time_critical: false
        }
    ));
    assert!(!critical(
        &s,
        Item::Track {
            track: song,
            time_critical: false
        }
    ));
    assert!(critical(
        &s,
        Item::Category {
            category: ids,
            time_critical: false
        }
    ));
    assert!(critical(
        &s,
        Item::Track {
            track: song,
            time_critical: true
        }
    ));
    assert!(!critical(
        &s,
        Item::Rotation {
            rotation: RotationId(1)
        }
    ));
    assert!(!critical(
        &s,
        Item::Track {
            track: TrackId(999),
            time_critical: false
        }
    ));
}

#[test]
fn a_rotation_item_must_be_last() {
    let rotation = Item::Rotation {
        rotation: RotationId(1),
    };
    let song = Item::Track {
        track: TrackId(1),
        time_critical: false,
    };
    assert_eq!(
        check_items(&[rotation.clone(), song.clone()]),
        Err(ModelError::RotationNotLast)
    );
    assert_eq!(check_items(&[song, rotation]), Ok(()));
    assert_eq!(check_items(&[]), Ok(()));
}
