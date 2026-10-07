#![allow(clippy::unwrap_used)]
mod support;

use fp_model::volume::gain_from_fader;
use fp_model::{CartId, Command, EntryId, PlayMode, PlayerId, PlaylistId};
use fp_remote::api::{ApiError, Edit, Operation as O, plan, plan_edit};
use support::demo_state;

#[test]
fn play_on_a_player_with_a_next_entry_is_one_play_command() {
    let s = demo_state();
    let p = s.players[0].id;
    assert_eq!(plan(&s, O::Play(p)).unwrap(), vec![Command::Play(p)]);
}

#[test]
fn an_unavailable_action_is_a_conflict() {
    let s = demo_state();
    let p = s.players[0].id;
    // Nothing is on air: Pause, Stop, Fade stop, Restart and Previous are off (R28).
    for op in [
        O::Pause(p),
        O::Stop(p),
        O::FadeStop(p),
        O::Restart(p),
        O::Previous(p),
    ] {
        let e = plan(&s, op).unwrap_err();
        assert_eq!(e.status(), 409, "{op:?}");
        assert_eq!(e.code(), "unavailable");
    }
}

#[test]
fn an_unknown_player_is_not_found_even_for_transport() {
    let s = demo_state();
    assert_eq!(
        plan(&s, O::Play(PlayerId(999_999))).unwrap_err(),
        ApiError::NotFound
    );
    assert_eq!(
        plan(&s, O::SetFader(PlayerId(999_999), 0.5)).unwrap_err(),
        ApiError::NotFound
    );
}

#[test]
fn toggles_take_the_desired_value() {
    let mut s = demo_state();
    let p = s.players[0].id;
    assert_eq!(plan(&s, O::SetStopAfterCurrent(p, false)).unwrap(), vec![]);
    assert_eq!(
        plan(&s, O::SetStopAfterCurrent(p, true)).unwrap(),
        vec![Command::SetStopAfterCurrent(p, true)]
    );
    assert_eq!(plan(&s, O::SetCue(p, false)).unwrap(), vec![]);
    assert_eq!(
        plan(&s, O::SetCue(p, true)).unwrap(),
        vec![Command::SetCue(p, true)]
    );
    let e = s.playlists.iter().next().unwrap().entries[0].id;
    assert_eq!(plan(&s, O::SetEntryRepeat(e, false)).unwrap(), vec![]);
    fp_model::apply(&mut s, Command::ToggleEntryRepeat(e)).unwrap();
    assert_eq!(plan(&s, O::SetEntryRepeat(e, true)).unwrap(), vec![]);
    assert_eq!(
        plan(&s, O::SetEntryRepeat(e, false)).unwrap(),
        vec![Command::SetEntryRepeat(e, false)]
    );
}

#[test]
fn stop_after_current_in_single_mode_is_a_conflict() {
    let mut s = demo_state();
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::SetMode(p, PlayMode::Single)).unwrap();
    assert_eq!(
        plan(&s, O::SetStopAfterCurrent(p, true))
            .unwrap_err()
            .status(),
        409
    );
}

#[test]
fn setting_the_entry_on_air_as_next_is_accepted() {
    let mut s = demo_state();
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    let current = s.players[0].current.unwrap();
    assert_eq!(
        plan(&s, O::SetNext(p, current)).unwrap(),
        vec![Command::SetNext(p, current)]
    );
}

#[test]
fn stop_after_current_in_single_mode_is_accepted_while_the_entry_repeats() {
    let mut s = demo_state();
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::SetMode(p, PlayMode::Single)).unwrap();
    let first = s.playlists.iter().next().unwrap().entries[0].id;
    fp_model::apply(&mut s, Command::ToggleEntryRepeat(first)).unwrap();
    fp_model::apply(&mut s, Command::SetNext(p, first)).unwrap();
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(
        plan(&s, O::SetStopAfterCurrent(p, true)).unwrap(),
        vec![Command::SetStopAfterCurrent(p, true)]
    );
}

#[test]
fn the_fader_maps_through_the_ui_curve_and_must_be_in_range() {
    let s = demo_state();
    let p = s.players[0].id;
    assert_eq!(
        plan(&s, O::SetFader(p, 0.8)).unwrap(),
        vec![Command::SetVolume(p, gain_from_fader(0.8))]
    );
    for bad in [-0.1, 1.1, f32::NAN, f32::INFINITY] {
        assert_eq!(
            plan(&s, O::SetFader(p, bad)).unwrap_err().status(),
            400,
            "{bad}"
        );
    }
}

#[test]
fn seek_while_playing_stays_inside_the_cue_range_of_what_plays() {
    let mut s = demo_state();
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(
        plan(&s, O::Seek(p, 10.0)).unwrap(),
        vec![Command::Seek(p, 10.0)]
    );
    assert_eq!(plan(&s, O::Seek(p, 500.0)).unwrap_err().status(), 400);
    assert_eq!(plan(&s, O::Seek(p, f64::NAN)).unwrap_err().status(), 400);
}

#[test]
fn next_and_cue_entry_need_known_ids() {
    let s = demo_state();
    let p = s.players[0].id;
    let e = s.playlists.iter().next().unwrap().entries[2].id;
    assert_eq!(
        plan(&s, O::SetNext(p, e)).unwrap(),
        vec![Command::SetNext(p, e)]
    );
    assert_eq!(
        plan(&s, O::CueEntry(p, e)).unwrap(),
        vec![Command::CueEntry(p, e)]
    );
    assert_eq!(
        plan(&s, O::SetNext(p, EntryId(999_999))).unwrap_err(),
        ApiError::NotFound
    );
}

#[test]
fn the_dry_run_turns_a_model_refusal_into_a_conflict() {
    let mut s = demo_state();
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    let current = s.players[0].current.unwrap();
    let e = plan_edit(&s, Edit::RemoveEntry(current)).unwrap_err();
    assert_eq!(e.status(), 409);
    assert!(e.message().contains("on air"), "{}", e.message());
}

#[test]
fn an_id_that_vanished_is_not_found() {
    let mut s = demo_state();
    let night = s.playlists.iter().nth(1).unwrap().id;
    let p = s.players[0].id;
    assert_eq!(
        plan(&s, O::ShowPlaylist(p, night)).unwrap(),
        vec![Command::ShowPlaylist(p, night)]
    );
    fp_model::apply(&mut s, Command::DeletePlaylist(night)).unwrap();
    assert_eq!(
        plan(&s, O::ShowPlaylist(p, night)).unwrap_err(),
        ApiError::NotFound
    );
    assert_eq!(
        plan(&s, O::ShowPlaylist(p, PlaylistId(424_242))).unwrap_err(),
        ApiError::NotFound
    );
}

#[test]
fn carts_fire_stop_and_cue_by_id() {
    let s = demo_state();
    let c = s.cartwall.pages[0].carts[0].id;
    assert_eq!(
        plan(&s, O::FireCart(c)).unwrap(),
        vec![Command::FireCart(c)]
    );
    assert_eq!(
        plan(&s, O::StopCart(c)).unwrap(),
        vec![Command::StopCart(c)]
    );
    assert_eq!(plan(&s, O::SetCartCue(c, false)).unwrap(), vec![]);
    assert_eq!(
        plan(&s, O::SetCartCue(c, true)).unwrap(),
        vec![Command::SetCartCue(c, true)]
    );
    assert_eq!(
        plan(&s, O::StopAllCarts).unwrap(),
        vec![Command::StopAllCarts]
    );
    assert_eq!(
        plan(&s, O::FireCart(CartId(999_999))).unwrap_err(),
        ApiError::NotFound
    );
    let page = s.cartwall.pages[0].id;
    assert_eq!(
        plan(&s, O::ShowCartPage(page)).unwrap(),
        vec![Command::ShowCartPage(page)]
    );
}

#[test]
fn every_error_has_its_status_and_code() {
    let cases = [
        (ApiError::BadRequest("x".into()), 400, "bad_request"),
        (ApiError::Unauthorized, 401, "unauthorized"),
        (ApiError::ForbiddenOrigin, 403, "forbidden_origin"),
        (ApiError::NotFound, 404, "not_found"),
        (ApiError::NotAnalyzed, 404, "not_analyzed"),
        (ApiError::Unavailable("x".into()), 409, "unavailable"),
        (ApiError::PayloadTooLarge, 413, "payload_too_large"),
        (
            ApiError::UnsupportedMediaType,
            415,
            "unsupported_media_type",
        ),
        (ApiError::MethodNotAllowed, 405, "method_not_allowed"),
        (ApiError::Busy, 503, "busy"),
    ];
    for (e, status, code) in cases {
        assert_eq!((e.status(), e.code()), (status, code));
        assert!(!e.message().is_empty());
    }
}

#[test]
fn a_desired_value_planned_twice_from_one_snapshot_stays_set() {
    // Two "on" requests decided before the conductor publishes the first
    // one's effect must not cancel each other.
    let s = demo_state();
    let p = s.players[0].id;
    let e = s.playlists.iter().next().unwrap().entries[0].id;
    let c = s.cartwall.pages[0].carts[0].id;
    let mut applied = s.clone();
    for op in [
        O::SetCue(p, true),
        O::SetStopAfterCurrent(p, true),
        O::SetEntryRepeat(e, true),
        O::SetCartCue(c, true),
    ] {
        let first = plan(&s, op).unwrap();
        let second = plan(&s, op).unwrap();
        for command in first.into_iter().chain(second) {
            fp_model::apply(&mut applied, command).unwrap();
        }
    }
    assert!(applied.players[0].cue.is_some());
    assert!(applied.players[0].stop_after_current);
    assert!(applied.playlists.entry(e).unwrap().repeat);
    assert_eq!(applied.cartwall.cue, Some(c));
}

/// Operator feedback 4, Q8 (rule 3a): a seek on a stopped player sets
/// where Play starts its next, inside that entry's cue range.
#[test]
fn seek_on_a_stopped_player_sets_where_play_starts_its_next() {
    let mut s = demo_state();
    let p = s.players[0].id;
    assert_eq!(
        plan(&s, O::Seek(p, 10.0)).unwrap(),
        vec![Command::Seek(p, 10.0)]
    );
    assert_eq!(plan(&s, O::Seek(p, 500.0)).unwrap_err().status(), 400);
    // With no next there is nothing to seek.
    let all: Vec<EntryId> = s
        .playlists
        .iter()
        .flat_map(|l| l.entries.iter().map(|e| e.id))
        .collect();
    for e in all {
        fp_model::apply(&mut s, Command::RemoveEntry(e)).unwrap();
    }
    assert_eq!(s.player(p).unwrap().next, None);
    assert_eq!(plan(&s, O::Seek(p, 10.0)).unwrap_err().status(), 409);
}

/// Rule 3a stores nothing for a start at or past the end of the next's
/// play range, so the seek is refused instead of answering success.
#[test]
fn seek_on_a_stopped_player_at_the_end_of_its_next_is_a_bad_request() {
    let s = demo_state();
    let p = s.players[0].id;
    let next = s.player(p).unwrap().next.unwrap();
    let end = s
        .track_for_entry(next)
        .unwrap()
        .play_range(s.config.players.use_cue_markers)
        .cue_out;
    let e = plan(&s, O::Seek(p, end)).unwrap_err();
    assert_eq!(e.status(), 400);
    assert_eq!(e.code(), "bad_request");
    assert!(plan(&s, O::Seek(p, end - 1.0)).is_ok());
}

/// Rule 3a stores nothing for a next whose file cannot be played, so the
/// seek is refused as unavailable.
#[test]
fn seek_on_a_stopped_player_whose_next_cannot_be_played_is_a_conflict() {
    for state in [
        fp_model::FileState::Missing,
        fp_model::FileState::Unreadable,
    ] {
        let mut s = demo_state();
        let p = s.players[0].id;
        let next = s.player(p).unwrap().next.unwrap();
        let track = s.playlists.entry(next).unwrap().track;
        s.library.get_mut(track).unwrap().file_state = state;
        let e = plan(&s, O::Seek(p, 10.0)).unwrap_err();
        assert_eq!(e.status(), 409, "{state:?}");
        assert_eq!(e.code(), "unavailable");
    }
}

#[test]
fn seek_bounds_follow_the_play_range() {
    let mut s = demo_state();
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    let entry = s.player(p).unwrap().current.unwrap();
    let track = s.playlists.entry(entry).unwrap().track;
    fp_model::apply(
        &mut s,
        Command::SetMarker {
            track,
            kind: fp_model::MarkerKind::CueOut,
            secs: Some(100.0),
        },
    )
    .unwrap();
    assert_eq!(plan(&s, O::Seek(p, 150.0)).unwrap_err().status(), 400);
    s.config.players.use_cue_markers = false;
    assert_eq!(
        plan(&s, O::Seek(p, 150.0)).unwrap(),
        vec![Command::Seek(p, 150.0)]
    );
}

/// Spec §4.6: a CUE needs a Cue output apart from Main; without one every
/// way to start a CUE is unavailable, as the console dims it.
#[test]
fn a_cue_without_a_cue_output_apart_from_main_is_a_conflict() {
    let mut s = demo_state();
    let p = s.players[0].id;
    let e = s.playlists.iter().next().unwrap().entries[2].id;
    let c = s.cartwall.pages[0].carts[0].id;
    for r in &mut s.config.outputs.routes {
        r.main = r.cue.clone();
    }
    s.config.outputs.cartwall.main = s.config.outputs.cartwall.cue.clone();
    for op in [
        O::SetCue(p, true),
        O::CueEntry(p, e),
        O::SetCartCue(c, true),
    ] {
        let err = plan(&s, op).unwrap_err();
        assert_eq!(err.code(), "unavailable", "{op:?}");
    }
    assert_eq!(plan(&s, O::SetCue(p, false)).unwrap(), vec![]);
    assert_eq!(plan(&s, O::SetCartCue(c, false)).unwrap(), vec![]);
}

/// The reducer raises a stopped player's seek before the cue-in to the
/// cue-in (`pending_start_at`), as the local wave does; the API does too,
/// with the same command, instead of answering 400.
#[test]
fn seek_on_a_stopped_player_before_the_cue_in_goes_to_the_cue_in() {
    let mut s = demo_state();
    let p = s.players[0].id;
    let next = s.player(p).unwrap().next.unwrap();
    let track = s.playlists.entry(next).unwrap().track;
    fp_model::apply(
        &mut s,
        Command::SetMarker {
            track,
            kind: fp_model::MarkerKind::CueIn,
            secs: Some(20.0),
        },
    )
    .unwrap();
    assert_eq!(
        plan(&s, O::Seek(p, 5.0)).unwrap(),
        vec![Command::Seek(p, 5.0)]
    );
    let mut applied = s.clone();
    fp_model::apply(&mut applied, Command::Seek(p, 5.0)).unwrap();
    assert_eq!(applied.player(p).unwrap().pending_start, Some((next, 20.0)));
    assert_eq!(plan(&s, O::Seek(p, f64::NAN)).unwrap_err().status(), 400);
}
