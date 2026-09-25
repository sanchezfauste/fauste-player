#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Spec §3 rules 13–15 and 21, playlist management, volume, seek and config.

mod common;

use common::{entries, fixture, p0};
use fp_model::{
    ColumnWidths, Command, Config, EngineAction, EngineEvent, ModelError, PlaylistId, Transport,
    apply, on_event,
};

#[test]
fn rule13_an_entry_on_air_cannot_be_removed() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    assert_eq!(
        apply(&mut state, Command::RemoveEntry(e[0])),
        Err(ModelError::EntryOnAir(e[0]))
    );
    assert_eq!(entries(&state).len(), 3);
}

#[test]
fn rule13_removing_the_next_entry_moves_next_to_the_following_one() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    apply(&mut state, Command::RemoveEntry(e[1])).unwrap();
    assert_eq!(state.player(p).unwrap().next, Some(e[2]));
}

#[test]
fn removing_the_last_reference_drops_the_track_from_the_library() {
    let mut state = fixture(2);
    let e = entries(&state);
    apply(&mut state, Command::DuplicateEntry(e[1])).unwrap();
    assert_eq!(state.library.len(), 2);
    let copy = entries(&state)[2];
    apply(&mut state, Command::RemoveEntry(copy)).unwrap();
    assert_eq!(
        state.library.len(),
        2,
        "the original still references the track"
    );
    apply(&mut state, Command::RemoveEntry(e[1])).unwrap();
    assert_eq!(state.library.len(), 1);
}

#[test]
fn rule14_switching_tabs_keeps_current_and_next() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(
        &mut state,
        Command::CreatePlaylist {
            name: "Other".into(),
        },
    )
    .unwrap();
    let other = state.playlists.iter().nth(1).unwrap().id;
    apply(&mut state, Command::Play(p)).unwrap();
    apply(&mut state, Command::ShowPlaylist(p, other)).unwrap();
    let player = state.player(p).unwrap();
    assert_eq!(
        (player.playlist, player.current, player.next),
        (other, Some(e[0]), Some(e[1]))
    );
}

#[test]
fn rule15_cue_toggles_prelisten_of_next_and_ends_on_event() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    let actions = apply(&mut state, Command::ToggleCue(p)).unwrap();
    assert!(
        matches!(actions.first(), Some(EngineAction::StartCue { request, .. }) if request.entry == e[0])
    );
    assert_eq!(state.player(p).unwrap().cue.map(|c| c.entry), Some(e[0]));
    assert_eq!(
        apply(&mut state, Command::ToggleCue(p)).unwrap(),
        vec![EngineAction::StopCue { player: p }]
    );
    apply(&mut state, Command::CueEntry(p, e[2])).unwrap();
    on_event(&mut state, EngineEvent::CueEnded { player: p });
    assert_eq!(state.player(p).unwrap().cue, None);
}

#[test]
fn removing_a_cued_entry_stops_the_cue() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::CueEntry(p, e[2])).unwrap();
    let actions = apply(&mut state, Command::RemoveEntry(e[2])).unwrap();
    assert!(actions.contains(&EngineAction::StopCue { player: p }));
}

#[test]
fn move_and_duplicate_entries() {
    let mut state = fixture(3);
    let e = entries(&state);
    let first = state.playlists.first_id().unwrap();
    apply(
        &mut state,
        Command::MoveEntry {
            entry: e[0],
            to: first,
            index: 3,
        },
    )
    .unwrap();
    assert_eq!(entries(&state), vec![e[1], e[2], e[0]]);
    apply(&mut state, Command::DuplicateEntry(e[1])).unwrap();
    assert_eq!(entries(&state).len(), 4);
}

#[test]
fn playlists_can_be_created_renamed_and_deleted() {
    let mut state = fixture(2);
    apply(
        &mut state,
        Command::CreatePlaylist {
            name: "Night".into(),
        },
    )
    .unwrap();
    let night = state.playlists.iter().nth(1).unwrap().id;
    apply(
        &mut state,
        Command::RenamePlaylist {
            playlist: night,
            name: "Late".into(),
        },
    )
    .unwrap();
    assert_eq!(state.playlists.get(night).unwrap().name, "Late");
    let p = p0(&state);
    apply(&mut state, Command::ShowPlaylist(p, night)).unwrap();
    apply(&mut state, Command::DeletePlaylist(night)).unwrap();
    assert_eq!(
        state.player(p).unwrap().playlist,
        state.playlists.first_id().unwrap()
    );
}

#[test]
fn a_playlist_on_air_or_the_last_one_cannot_be_deleted() {
    let mut state = fixture(2);
    let first = state.playlists.first_id().unwrap();
    assert_eq!(
        apply(&mut state, Command::DeletePlaylist(first)),
        Err(ModelError::LastPlaylist)
    );
    apply(
        &mut state,
        Command::CreatePlaylist {
            name: "Spare".into(),
        },
    )
    .unwrap();
    let p = p0(&state);
    apply(&mut state, Command::Play(p)).unwrap();
    assert_eq!(
        apply(&mut state, Command::DeletePlaylist(first)),
        Err(ModelError::PlaylistOnAir(first))
    );
    assert_eq!(
        apply(&mut state, Command::DeletePlaylist(PlaylistId(424242))),
        Err(ModelError::UnknownPlaylist(PlaylistId(424242)))
    );
}

#[test]
fn rule21_player_count_grows_without_a_fixed_maximum() {
    let mut state = fixture(2);
    let e = entries(&state);
    let actions = apply(&mut state, Command::SetPlayerCount(8)).unwrap();
    assert_eq!(state.players.len(), 8);
    assert_eq!(state.config.players.count, 8);
    assert_eq!(
        actions
            .iter()
            .filter(|a| matches!(a, EngineAction::AddPlayer { .. }))
            .count(),
        4
    );
    assert!(state.players.iter().all(|p| p.next == Some(e[0])));
}

#[test]
fn rule21_player_count_respects_the_resource_limit_and_busy_players() {
    let mut state = fixture(2);
    let max = state.config.limits.max_players;
    assert_eq!(
        apply(&mut state, Command::SetPlayerCount(max + 1)),
        Err(ModelError::PlayerCountOutOfRange {
            requested: max + 1,
            max
        })
    );
    assert!(matches!(
        apply(&mut state, Command::SetPlayerCount(0)),
        Err(ModelError::PlayerCountOutOfRange { .. })
    ));
    let last = state.players[3].id;
    apply(&mut state, Command::Play(last)).unwrap();
    assert_eq!(
        apply(&mut state, Command::SetPlayerCount(2)),
        Err(ModelError::PlayerBusy(last))
    );
    apply(&mut state, Command::Stop(last)).unwrap();
    let actions = apply(&mut state, Command::SetPlayerCount(2)).unwrap();
    assert!(actions.contains(&EngineAction::RemovePlayer { player: last }));
    assert_eq!(state.players.len(), 2);
}

#[test]
fn volume_is_clamped_and_nan_is_silence() {
    let mut state = fixture(1);
    let p = p0(&state);
    apply(&mut state, Command::SetVolume(p, 1.7)).unwrap();
    assert_eq!(state.player(p).unwrap().volume, 1.0);
    let actions = apply(&mut state, Command::SetVolume(p, f32::NAN)).unwrap();
    assert_eq!(
        actions,
        vec![EngineAction::SetVolume {
            player: p,
            volume: 0.0
        }]
    );
}

#[test]
fn seek_is_clamped_and_ignored_when_idle() {
    let mut state = fixture(1);
    let p = p0(&state);
    assert!(
        apply(&mut state, Command::Seek(p, 10.0))
            .unwrap()
            .is_empty()
    );
    apply(&mut state, Command::Play(p)).unwrap();
    let actions = apply(&mut state, Command::Seek(p, 999.0)).unwrap();
    assert_eq!(
        actions,
        vec![EngineAction::Seek {
            player: p,
            secs: 180.0
        }]
    );
}

#[test]
fn update_config_keeps_the_player_count() {
    let mut state = fixture(1);
    let mut config = Config::default();
    config.players.count = 9;
    config.players.fade_ms = 2500;
    apply(&mut state, Command::UpdateConfig(Box::new(config))).unwrap();
    assert_eq!(state.config.players.fade_ms, 2500);
    assert_eq!(state.config.players.count, 4);
    assert_eq!(state.players.len(), 4);
}

#[test]
fn column_widths_are_stored_per_player() {
    let mut state = fixture(1);
    let p = p0(&state);
    let widths = ColumnWidths {
        number: Some(40.0),
        title: Some(200.0),
        duration: 60.0,
    };
    apply(&mut state, Command::SetColumnWidths(p, widths)).unwrap();
    assert_eq!(state.player(p).unwrap().columns, widths);
    assert_eq!(state.players[1].columns, ColumnWidths::default());
    assert_eq!(state.player(p).unwrap().transport, Transport::Stopped);
}
