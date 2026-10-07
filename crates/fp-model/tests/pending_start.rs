#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::float_cmp
)]
//! Rule 3a, pending start (operator feedback 4, Q8): a seek on a stopped
//! player chooses where Play starts its next entry.

mod common;

use common::{entries, fixture, p0};
use fp_model::{
    AppState, Command, EngineAction, EngineEvent, EntryId, MarkerKind, PlayerId, TrackAnalysis,
    Transport, apply, on_event,
};

fn set_cue_in(s: &mut AppState, entry: EntryId, secs: f64) {
    let t = s.playlists.entry(entry).unwrap().track;
    s.library
        .get_mut(t)
        .unwrap()
        .markers
        .set_auto(MarkerKind::CueIn, Some(secs));
}

fn starts(actions: &[EngineAction]) -> Vec<(EntryId, f64)> {
    actions
        .iter()
        .filter_map(|a| match a {
            EngineAction::StartCurrent { request, .. }
            | EngineAction::Crossfade { request, .. } => Some((request.entry, request.from_secs)),
            _ => None,
        })
        .collect()
}

fn preloads(actions: &[EngineAction], p: PlayerId) -> Vec<(EntryId, f64)> {
    actions
        .iter()
        .filter_map(|a| match a {
            EngineAction::Preload {
                player,
                request: Some(r),
            } if *player == p => Some((r.entry, r.from_secs)),
            _ => None,
        })
        .collect()
}

fn pending(s: &AppState, p: PlayerId) -> Option<(EntryId, f64)> {
    s.player(p).unwrap().pending_start
}

#[test]
fn q8_2_a_seek_while_stopped_stores_where_play_starts_the_next_and_starts_nothing() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    let actions = apply(&mut s, Command::Seek(p, 30.0)).unwrap();
    assert_eq!(pending(&s, p), Some((e[0], 30.0)));
    let player = s.player(p).unwrap();
    assert_eq!(
        (player.transport, player.current),
        (Transport::Stopped, None)
    );
    assert!(
        actions.iter().all(|a| !matches!(
            a,
            EngineAction::StartCurrent { .. }
                | EngineAction::Seek { .. }
                | EngineAction::LoadPaused { .. }
                | EngineAction::Resume { .. }
        )),
        "{actions:?}"
    );
}

#[test]
fn q8_2_the_pending_start_is_raised_to_the_cue_in() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    set_cue_in(&mut s, e[0], 2.0);
    apply(&mut s, Command::Seek(p, 1.0)).unwrap();
    assert_eq!(pending(&s, p), Some((e[0], 2.0)));
}

#[test]
fn q8_2_a_seek_at_or_past_the_end_of_the_play_range_stores_nothing() {
    let mut s = fixture(3);
    let p = p0(&s);
    apply(&mut s, Command::Seek(p, 30.0)).unwrap();
    apply(&mut s, Command::Seek(p, 180.0)).unwrap();
    assert_eq!(pending(&s, p), None, "Play would end the track at once");
    apply(&mut s, Command::Seek(p, 500.0)).unwrap();
    assert_eq!(pending(&s, p), None);
}

#[test]
fn q8_2_a_broken_seek_changes_nothing() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::Seek(p, 30.0)).unwrap();
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        apply(&mut s, Command::Seek(p, bad)).unwrap();
        assert_eq!(pending(&s, p), Some((e[0], 30.0)), "{bad}");
    }
}

#[test]
fn q8_2_without_a_next_a_seek_does_nothing() {
    let mut s = fixture(1);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::RemoveEntry(e[0])).unwrap();
    assert_eq!(s.player(p).unwrap().next, None);
    apply(&mut s, Command::Seek(p, 10.0)).unwrap();
    assert_eq!(pending(&s, p), None);
}

#[test]
fn q8_2_a_next_whose_file_cannot_be_played_gets_no_pending_start() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    let track = s.playlists.entry(e[0]).unwrap().track;
    apply(
        &mut s,
        Command::SetFileState {
            track,
            state: fp_model::FileState::Missing,
        },
    )
    .unwrap();
    apply(&mut s, Command::Seek(p, 10.0)).unwrap();
    assert_eq!(pending(&s, p), None);
}

#[test]
fn q8_3_play_starts_the_next_at_its_pending_start_once() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::Seek(p, 30.0)).unwrap();
    let actions = apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(starts(&actions), vec![(e[0], 30.0)]);
    assert_eq!(pending(&s, p), None, "used");
    apply(&mut s, Command::Stop(p)).unwrap();
    apply(&mut s, Command::SetNext(p, e[0])).unwrap();
    let actions = apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(starts(&actions), vec![(e[0], 0.0)], "only once");
}

#[test]
fn q8_4_changing_the_next_clears_the_pending_start() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::Seek(p, 30.0)).unwrap();
    apply(&mut s, Command::SetNext(p, e[1])).unwrap();
    assert_eq!(pending(&s, p), None);
}

#[test]
fn q8_4_setting_the_same_next_again_clears_it_too() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::Seek(p, 30.0)).unwrap();
    apply(&mut s, Command::SetNext(p, e[0])).unwrap();
    assert_eq!(pending(&s, p), None);
}

#[test]
fn q8_4_removing_its_entry_clears_it() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::Seek(p, 30.0)).unwrap();
    apply(&mut s, Command::RemoveEntry(e[0])).unwrap();
    assert_eq!(pending(&s, p), None);
}

#[test]
fn q8_4_moving_its_entry_clears_it() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    let playlist = s.playlists.first_id().unwrap();
    apply(&mut s, Command::Seek(p, 30.0)).unwrap();
    apply(
        &mut s,
        Command::MoveEntry {
            entry: e[0],
            to: playlist,
            index: 2,
        },
    )
    .unwrap();
    assert_eq!(pending(&s, p), None);
}

#[test]
fn q8_4_stop_clears_it() {
    let mut s = fixture(3);
    let p = p0(&s);
    apply(&mut s, Command::Seek(p, 30.0)).unwrap();
    apply(&mut s, Command::Stop(p)).unwrap();
    assert_eq!(pending(&s, p), None);
}

#[test]
fn q8_4_play_now_of_another_entry_starts_it_at_its_cue_in() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::Seek(p, 30.0)).unwrap();
    apply(&mut s, Command::SetNext(p, e[2])).unwrap();
    let actions = apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(starts(&actions), vec![(e[2], 0.0)]);
}

#[test]
fn q8_5_restart_the_automatic_advance_and_previous_keep_the_cue_in() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    set_cue_in(&mut s, e[0], 2.0);
    apply(&mut s, Command::Seek(p, 30.0)).unwrap();
    assert_eq!(
        starts(&apply(&mut s, Command::Play(p)).unwrap()),
        vec![(e[0], 30.0)]
    );
    // Restart goes back to the cue-in, not to the pending start.
    let actions = apply(&mut s, Command::Restart(p)).unwrap();
    assert!(
        actions
            .iter()
            .any(|a| matches!(a, EngineAction::Seek { secs, .. } if *secs == 2.0)),
        "{actions:?}"
    );
    // The automatic advance: the engine starts e1; e2 is preloaded at its cue-in.
    let actions = on_event(
        &mut s,
        EngineEvent::TransitionStarted {
            player: p,
            entry: e[1],
        },
    );
    assert_eq!(s.player(p).unwrap().current, Some(e[1]));
    assert_eq!(preloads(&actions, p), vec![(e[2], 0.0)]);
    // Previous goes back to e0 from its cue-in.
    let actions = apply(&mut s, Command::Previous(p)).unwrap();
    assert_eq!(starts(&actions), vec![(e[0], 2.0)]);
}

#[test]
fn q8_7_the_next_is_preloaded_at_the_pending_start_and_again_when_it_changes() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    let actions = apply(&mut s, Command::Seek(p, 30.0)).unwrap();
    assert_eq!(preloads(&actions, p), vec![(e[0], 30.0)]);
    let actions = apply(&mut s, Command::Seek(p, 45.0)).unwrap();
    assert_eq!(preloads(&actions, p), vec![(e[0], 45.0)]);
    let actions = apply(&mut s, Command::SetNext(p, e[1])).unwrap();
    assert_eq!(preloads(&actions, p), vec![(e[1], 0.0)]);
}

#[test]
fn q8_7_play_starts_exactly_where_the_preload_is() {
    let mut s = fixture(3);
    let p = p0(&s);
    let preloaded = preloads(&apply(&mut s, Command::Seek(p, 30.0)).unwrap(), p);
    let started = starts(&apply(&mut s, Command::Play(p)).unwrap());
    assert_eq!(preloaded, started, "the engine matches them exactly");
}

#[test]
fn q8_8_a_pending_start_never_goes_on_air_by_itself() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::Seek(p, 30.0)).unwrap();
    let track = s.playlists.entry(e[0]).unwrap().track;
    let mut actions = apply(&mut s, Command::SetVolume(p, 0.5)).unwrap();
    actions.extend(
        apply(
            &mut s,
            Command::ApplyAnalysis {
                track,
                analysis: Box::new(TrackAnalysis {
                    duration_secs: 180.0,
                    ..TrackAnalysis::default()
                }),
            },
        )
        .unwrap(),
    );
    actions.extend(on_event(&mut s, EngineEvent::FadeCompleted { player: p }));
    assert!(starts(&actions).is_empty(), "{actions:?}");
    assert_eq!(s.player(p).unwrap().transport, Transport::Stopped);
    assert_eq!(pending(&s, p), Some((e[0], 30.0)));
}

#[test]
fn a_pending_start_is_clamped_again_when_the_analysis_moves_the_cue_in() {
    let mut s = fixture(3);
    let (e, p) = (entries(&s), p0(&s));
    apply(&mut s, Command::Seek(p, 1.0)).unwrap();
    let track = s.playlists.entry(e[0]).unwrap().track;
    let actions = apply(
        &mut s,
        Command::ApplyAnalysis {
            track,
            analysis: Box::new(TrackAnalysis {
                duration_secs: 180.0,
                cue_in: Some(4.0),
                ..TrackAnalysis::default()
            }),
        },
    )
    .unwrap();
    assert_eq!(pending(&s, p), Some((e[0], 4.0)));
    assert_eq!(preloads(&actions, p), vec![(e[0], 4.0)]);
}
