#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Spec §3 rules 13–15 and 21, playlist management, volume, seek and config.

mod common;

use common::{entries, fixture, p0};
use fp_model::{
    ColumnWidths, Command, Config, EngineAction, EngineEvent, ModelError, PlaylistId, TableColumn,
    Transport, apply, on_event,
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
    on_event(
        &mut state,
        EngineEvent::CueEnded {
            player: p,
            entry: e[2],
        },
    );
    assert_eq!(state.player(p).unwrap().cue, None);
}

#[test]
fn a_stale_cue_end_does_not_end_a_newer_cue() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::CueEntry(p, e[1])).unwrap();
    apply(&mut state, Command::CueEntry(p, e[2])).unwrap();
    // The first cue's end arrives after the second one started.
    on_event(
        &mut state,
        EngineEvent::CueEnded {
            player: p,
            entry: e[1],
        },
    );
    assert_eq!(state.player(p).unwrap().cue.map(|c| c.entry), Some(e[2]));
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
fn volume_is_clamped_and_nan_keeps_the_volume() {
    let mut state = fixture(1);
    let p = p0(&state);
    apply(&mut state, Command::SetVolume(p, 1.7)).unwrap();
    assert_eq!(state.player(p).unwrap().volume, 1.0);
    apply(&mut state, Command::SetVolume(p, 0.5)).unwrap();
    // A broken value never silences what is on air.
    let actions = apply(&mut state, Command::SetVolume(p, f32::NAN)).unwrap();
    assert!(actions.is_empty(), "{actions:?}");
    assert_eq!(state.player(p).unwrap().volume, 0.5);
}

#[test]
fn seek_is_clamped_and_starts_nothing_when_stopped() {
    let mut state = fixture(1);
    let p = p0(&state);
    // Rule 3a: a seek while stopped only moves the preload of the next.
    let actions = apply(&mut state, Command::Seek(p, 10.0)).unwrap();
    assert!(
        actions
            .iter()
            .all(|a| matches!(a, EngineAction::Preload { .. })),
        "{actions:?}"
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
    let widths = ColumnWidths::keyed([
        (TableColumn::Number, 0.1),
        (TableColumn::Title, 0.5),
        (TableColumn::Artist, 0.3),
        (TableColumn::Duration, 0.1),
    ]);
    apply(&mut state, Command::SetColumnWidths(p, widths.clone())).unwrap();
    assert_eq!(state.player(p).unwrap().columns, widths);
    assert_eq!(state.players[1].columns, ColumnWidths::default());
    assert_eq!(state.player(p).unwrap().transport, Transport::Stopped);
}

#[test]
fn a_playlist_can_be_created_from_imported_paths() {
    let mut state = common::fixture(0);
    let paths = vec![
        std::path::PathBuf::from("/m/a.mp3"),
        std::path::PathBuf::from("/m/b.mp3"),
    ];
    fp_model::apply(
        &mut state,
        fp_model::Command::CreatePlaylistFromPaths {
            name: "Imported".into(),
            paths,
        },
    )
    .unwrap();
    let list = state
        .playlists
        .iter()
        .find(|p| p.name == "Imported")
        .unwrap();
    let files: Vec<_> = list
        .entries
        .iter()
        .map(|e| state.library.get(e.track).unwrap().path.clone())
        .collect();
    assert_eq!(
        files,
        vec![
            std::path::PathBuf::from("/m/a.mp3"),
            std::path::PathBuf::from("/m/b.mp3")
        ]
    );
}

#[test]
fn a_config_update_is_validated_and_a_lower_player_limit_removes_idle_players() {
    let mut state = fixture(1);
    apply(&mut state, Command::SetPlayerCount(4)).unwrap();
    let last = state.players[3].id;
    let mut config = state.config.clone();
    config.limits.max_players = 3;
    config.players.fade_ms = 0; // out of range: validated
    let actions = apply(&mut state, Command::UpdateConfig(Box::new(config))).unwrap();
    assert!(actions.contains(&EngineAction::RemovePlayer { player: last }));
    assert_eq!(state.players.len(), 3);
    assert_eq!(state.config.players.count, 3);
    assert_eq!(state.config.players.fade_ms, 50, "clamped to its range");
}

#[test]
fn a_lower_player_limit_is_refused_while_a_player_it_removes_is_busy() {
    let mut state = fixture(1);
    apply(&mut state, Command::SetPlayerCount(2)).unwrap();
    let second = state.players[1].id;
    apply(&mut state, Command::Play(second)).unwrap();
    let mut config = state.config.clone();
    config.limits.max_players = 1;
    let before = state.clone();
    assert!(apply(&mut state, Command::UpdateConfig(Box::new(config))).is_err());
    assert_eq!(state, before, "state unchanged on refusal");
}

/// Rule 13 forbids removing an entry on air, not moving it (rule 22).
#[test]
fn rule13_the_entry_on_air_can_be_moved() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    let list = state.playlists.first_id().unwrap();
    apply(
        &mut state,
        Command::MoveEntry {
            entry: e[0],
            to: list,
            index: 3,
        },
    )
    .unwrap();
    assert_eq!(state.player(p).unwrap().current, Some(e[0]), "still on air");
    assert_eq!(entries(&state), vec![e[1], e[2], e[0]]);
    assert_eq!(
        state.player(p).unwrap().next,
        None,
        "nothing after it any more"
    );
}
