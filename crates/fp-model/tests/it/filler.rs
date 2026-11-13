#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Automation spec AU27–AU29: the music filler.

use fp_model::filler::ensure;
use fp_model::{AppState, Config, EngineAction, Transport};

fn state() -> AppState {
    let mut s = AppState::new(Config::default(), "P");
    fp_model::rotation::ensure_default(&mut s, "Music");
    s
}

#[test]
fn ensure_creates_a_stopped_filler_last_once() {
    let mut s = state();
    let first = ensure(&mut s, "Music filler");
    let filler = s.filler.clone().unwrap();
    assert_eq!(s.players.last().unwrap().id, filler.player);
    assert_eq!(s.players.last().unwrap().transport, Transport::Stopped);
    assert_eq!(
        first,
        vec![EngineAction::AddPlayer {
            player: filler.player
        }]
    );
    assert!(ensure(&mut s, "Music filler").is_empty());
    assert_eq!(s.players.iter().filter(|p| s.is_filler(p.id)).count(), 1);
    assert_eq!(s.aux_players().count(), s.config.players.count);
    assert!(s.is_filler_playlist(filler.playlist));
    assert_eq!(
        s.playlists.get(filler.playlist).unwrap().name,
        "Music filler"
    );
}

#[test]
fn filler_settings_defaults_and_ranges() {
    let mut c = Config::default();
    assert_eq!(
        (
            c.automation.filler_lookahead_mins,
            c.automation.filler_played_kept
        ),
        (30, 50)
    );
    c.automation.filler_lookahead_mins = 4;
    c.automation.filler_played_kept = 2_000;
    c.validate();
    assert_eq!(c.automation.filler_lookahead_mins, 5);
    assert_eq!(c.automation.filler_played_kept, 1_000);
}

#[test]
fn player_count_changes_leave_the_filler_last() {
    let mut s = state();
    ensure(&mut s, "Music filler");
    let filler = s.filler.as_ref().unwrap().player;
    fp_model::apply(&mut s, fp_model::Command::SetPlayerCount(2)).unwrap();
    assert_eq!(s.players.last().unwrap().id, filler);
    assert_eq!(s.aux_players().count(), 2);
    assert_eq!(s.players.len(), 3);
    fp_model::apply(&mut s, fp_model::Command::SetPlayerCount(5)).unwrap();
    assert_eq!(s.players.last().unwrap().id, filler);
    assert_eq!(s.aux_players().count(), 5);
    assert_eq!(s.config.players.count, 5);
}

#[test]
fn numbered_shortcuts_never_reach_the_filler() {
    let mut s = state();
    ensure(&mut s, "Music filler");
    assert_eq!(s.aux_players().count(), 4);
    assert_eq!(
        fp_model::player_command(&s, fp_model::ShortcutAction::PlayPlayer(5)),
        None
    );
    let first = s.players[0].id;
    assert_eq!(
        fp_model::player_command(&s, fp_model::ShortcutAction::PlayPlayer(1)),
        Some(fp_model::Command::Play(first))
    );
}

#[test]
fn sessions_leave_the_filler_out() {
    let mut s = state();
    ensure(&mut s, "Music filler");
    let filler = s.filler.as_ref().unwrap().player;
    let sessions = s.sessions(|_| 0.0);
    assert_eq!(sessions.len(), 4);
    assert!(sessions.iter().all(|p| p.id != filler));
}

// --- Task 3: topping up and trimming (AU27, AU28) ---

use std::path::PathBuf;

use fp_model::filler::MAX_ADDED_PER_SETTLE;
use fp_model::{Command, PlayerId, PlaylistId, Track, TrackId, apply};

/// A state with `n` music tracks of `secs` seconds, the rules off and the
/// filler in place (its first settle done by a harmless command).
fn filled(n: u64, secs: f64) -> AppState {
    let mut s = state();
    s.config.automation.track_separation_mins = 0;
    s.config.automation.track_separation_percent = 0;
    s.config.automation.artist_separation_mins = 0;
    for id in 1..=n {
        let mut t = Track::new(TrackId(10_000 + id), PathBuf::from(format!("/m/{id}.flac")));
        t.duration_secs = secs;
        s.library.insert(t);
    }
    ensure(&mut s, "Music filler");
    settle(&mut s);
    s
}

/// Any command runs the filler's settle; this one changes nothing.
fn settle(s: &mut AppState) {
    let _ = apply(s, Command::SetLayoutLocked(false));
}

/// Next on the filler, and the engine reporting the crossfade done.
fn next(s: &mut AppState, player: PlayerId) {
    apply(s, Command::Play(player)).unwrap();
    fp_model::on_event(s, fp_model::EngineEvent::FadeCompleted { player });
}

fn filler(s: &AppState) -> (PlayerId, PlaylistId) {
    let f = s.filler.as_ref().unwrap();
    (f.player, f.playlist)
}

fn rows(s: &AppState) -> Vec<fp_model::EntryId> {
    let (_, list) = filler(s);
    s.playlists
        .get(list)
        .unwrap()
        .entries
        .iter()
        .map(|e| e.id)
        .collect()
}

#[test]
fn settle_fills_up_to_the_lookahead() {
    let s = filled(20, 180.0);
    let r = rows(&s);
    assert_eq!(r.len(), 10, "30 min of 180 s tracks");
    let f = s.filler.as_ref().unwrap();
    assert!(r.iter().all(|e| f.generated.contains(e)));
    assert!(!f.starved);
    assert_eq!(s.players.last().unwrap().next, Some(r[0]));
}

#[test]
fn played_rows_beyond_the_kept_count_are_trimmed() {
    let mut s = filled(20, 180.0);
    s.config.automation.filler_played_kept = 10;
    let (player, list) = filler(&s);
    for _ in 0..25 {
        next(&mut s, player);
    }
    let current = s.players.last().unwrap().current.unwrap();
    let position = s.playlists.get(list).unwrap().position(current).unwrap();
    assert_eq!(position, 10, "ten played rows kept before the current one");
    let r = rows(&s);
    assert!(
        s.filler
            .as_ref()
            .unwrap()
            .generated
            .iter()
            .all(|e| r.contains(e))
    );
}

#[test]
fn an_operator_row_is_never_replaced() {
    let mut s = filled(20, 180.0);
    let (player, list) = filler(&s);
    apply(&mut s, Command::Play(player)).unwrap();
    let operator_track = TrackId(10_005);
    apply(
        &mut s,
        Command::InsertTracks {
            playlist: list,
            index: 3,
            tracks: vec![operator_track],
        },
    )
    .unwrap();
    let operator_row = rows(&s)[3];
    assert!(!s.filler.as_ref().unwrap().generated.contains(&operator_row));
    let rotation = s.rotations.default_id();
    apply(&mut s, Command::SetFillerRotation(rotation)).unwrap();
    assert!(rows(&s).contains(&operator_row));
    assert!(!s.filler.as_ref().unwrap().generated.contains(&operator_row));
}

#[test]
fn a_removed_row_does_not_count_as_aired() {
    let mut s = filled(20, 180.0);
    let victim = rows(&s)[4];
    apply(&mut s, Command::RemoveEntry(victim)).unwrap();
    assert!(s.air_times.is_empty());
    assert!(!rows(&s).contains(&victim));
}

#[test]
fn an_empty_library_adds_nothing_and_flags_it() {
    let mut s = state();
    ensure(&mut s, "Music filler");
    settle(&mut s);
    assert!(rows(&s).is_empty());
    assert!(s.filler.as_ref().unwrap().starved);
    let mut t = Track::new(TrackId(1), PathBuf::from("/m/a.flac"));
    t.duration_secs = 180.0;
    s.library.insert(t);
    settle(&mut s);
    assert!(!rows(&s).is_empty());
    assert!(!s.filler.as_ref().unwrap().starved);
}

#[test]
fn unknown_durations_do_not_fill_without_end() {
    let s = filled(20, 0.0);
    let r = rows(&s);
    assert!(r.len() <= MAX_ADDED_PER_SETTLE);
    assert_eq!(r.len(), 10, "UNKNOWN_LENGTH_SECS x 10 = 30 min");
}

#[test]
fn the_list_stays_bounded_over_a_day() {
    let mut s = filled(40, 180.0);
    let (player, _) = filler(&s);
    let kept = s.config.automation.filler_played_kept as usize;
    for _ in 0..500 {
        next(&mut s, player);
        assert!(rows(&s).len() <= kept + 10 + 1, "{}", rows(&s).len());
    }
}

#[test]
fn set_filler_rotation_regenerates_only_generated_rows() {
    let mut s = filled(20, 180.0);
    let before = rows(&s);
    let other = {
        apply(
            &mut s,
            Command::CreateRotation {
                name: "Other".into(),
                steps: vec![fp_model::RotationStep {
                    source: fp_model::StepSource::Kind(fp_model::TrackKind::Music),
                    min: 1,
                    max: 1,
                }],
            },
        )
        .unwrap();
        s.rotations.iter().last().unwrap().id
    };
    apply(&mut s, Command::SetFillerRotation(Some(other))).unwrap();
    let after = rows(&s);
    assert_eq!(after.len(), 10);
    assert!(
        after.iter().all(|e| !before.contains(e)),
        "a stopped filler regenerates all"
    );
    assert_eq!(s.filler.as_ref().unwrap().rotation, Some(other));
    assert_eq!(
        apply(
            &mut s,
            Command::SetFillerRotation(Some(fp_model::RotationId(99_999)))
        ),
        Err(fp_model::ModelError::UnknownRotation(fp_model::RotationId(
            99_999
        )))
    );
}

// --- Task 5: display order and the hidden column ---

#[test]
fn the_filler_comes_first_in_display_order_and_starts_hidden() {
    let mut s = state();
    ensure(&mut s, "Music filler");
    let f = s.filler.as_ref().unwrap().player;
    let ids = s.player_ids();
    assert_eq!(ids[0], f, "the Music column is left of P1");
    assert_eq!(ids.len(), 5);
    assert!(s.layout.hidden_players.contains(&f));
    assert_eq!(s.aux_player_ids(), ids[1..].to_vec());
}

// --- Final review: the filler's list is not an operator playlist ---

fn other_rotation(s: &mut AppState, name: &str) -> fp_model::RotationId {
    apply(
        s,
        Command::CreateRotation {
            name: name.into(),
            steps: vec![fp_model::RotationStep {
                source: fp_model::StepSource::Kind(fp_model::TrackKind::Music),
                min: 1,
                max: 1,
            }],
        },
    )
    .unwrap();
    s.rotations.iter().last().unwrap().id
}

#[test]
fn the_filler_playlist_cannot_be_renamed_or_deleted() {
    let mut s = filled(5, 180.0);
    let (_, list) = filler(&s);
    assert_eq!(
        apply(
            &mut s,
            Command::RenamePlaylist {
                playlist: list,
                name: "Mine".into()
            }
        ),
        Err(fp_model::ModelError::FillerPlaylist(list))
    );
    assert_eq!(
        apply(&mut s, Command::DeletePlaylist(list)),
        Err(fp_model::ModelError::FillerPlaylist(list))
    );
}

#[test]
fn the_last_operator_playlist_cannot_be_deleted_with_the_filler_present() {
    let mut s = filled(5, 180.0);
    let (_, filler_list) = filler(&s);
    let mine: Vec<PlaylistId> = s
        .playlists
        .iter()
        .map(|l| l.id)
        .filter(|id| *id != filler_list)
        .collect();
    assert_eq!(mine.len(), 1, "a new state has one operator playlist");
    assert_eq!(
        apply(&mut s, Command::DeletePlaylist(mine[0])),
        Err(fp_model::ModelError::LastPlaylist)
    );
}

#[test]
fn deleting_a_playlist_never_shows_the_filler_list_on_a_player() {
    let mut s = filled(5, 180.0);
    let (_, filler_list) = filler(&s);
    apply(
        &mut s,
        Command::CreatePlaylist {
            name: "Second".into(),
        },
    )
    .unwrap();
    let first = s.aux_players().next().unwrap().playlist;
    apply(&mut s, Command::DeletePlaylist(first)).unwrap();
    assert!(s.aux_players().all(|p| p.playlist != filler_list));
}

#[test]
fn players_cannot_swap_lists_with_the_filler() {
    let mut s = filled(5, 180.0);
    let (player, filler_list) = filler(&s);
    let aux = s.aux_players().next().unwrap();
    let (aux_id, aux_list) = (aux.id, aux.playlist);
    assert_eq!(
        apply(&mut s, Command::ShowPlaylist(player, aux_list)),
        Err(fp_model::ModelError::FillerPlaylist(aux_list))
    );
    assert_eq!(
        apply(&mut s, Command::ShowPlaylist(aux_id, filler_list)),
        Err(fp_model::ModelError::FillerPlaylist(filler_list))
    );
}

#[test]
fn the_filler_next_stays_in_its_own_list() {
    let mut s = filled(5, 180.0);
    let (player, _) = filler(&s);
    let aux_list = s.aux_players().next().unwrap().playlist;
    apply(
        &mut s,
        Command::InsertTracks {
            playlist: aux_list,
            index: 0,
            tracks: vec![TrackId(10_001)],
        },
    )
    .unwrap();
    let elsewhere = s.playlists.get(aux_list).unwrap().entries[0].id;
    assert_eq!(
        apply(&mut s, Command::SetNext(player, elsewhere)),
        Err(fp_model::ModelError::FillerPlaylist(aux_list))
    );
}

#[test]
fn settings_changes_ignore_the_filler_at_the_player_maximum() {
    let mut s = filled(5, 180.0);
    let max = s.config.limits.max_players;
    apply(&mut s, Command::SetPlayerCount(max)).unwrap();
    let (player, _) = filler(&s);
    apply(&mut s, Command::Play(player)).unwrap();
    let mut c = s.config.clone();
    c.automation.filler_lookahead_mins = 60;
    apply(&mut s, Command::UpdateConfig(Box::new(c))).unwrap();
    assert_eq!(s.config.automation.filler_lookahead_mins, 60);
    assert_eq!(s.aux_players().count(), max);
}

#[test]
fn a_new_default_rotation_retargets_a_filler_on_the_default() {
    let mut s = filled(5, 180.0);
    let other = other_rotation(&mut s, "Other");
    apply(&mut s, Command::SetDefaultRotation(other)).unwrap();
    assert_eq!(s.filler.as_ref().unwrap().cursor.rotation, other);
}

#[test]
fn deleting_the_filler_rotation_falls_back_to_the_default() {
    let mut s = filled(5, 180.0);
    let other = other_rotation(&mut s, "Other");
    apply(&mut s, Command::SetFillerRotation(Some(other))).unwrap();
    apply(&mut s, Command::DeleteRotation(other)).unwrap();
    let f = s.filler.as_ref().unwrap();
    assert_eq!(f.rotation, None);
    assert_eq!(Some(f.cursor.rotation), s.rotations.default_id());
}

#[test]
fn deleting_the_default_retargets_a_filler_on_the_default() {
    let mut s = filled(5, 180.0);
    let old = s.rotations.default_id().unwrap();
    other_rotation(&mut s, "Other");
    apply(&mut s, Command::DeleteRotation(old)).unwrap();
    let f = s.filler.as_ref().unwrap();
    assert_eq!(Some(f.cursor.rotation), s.rotations.default_id());
    assert_ne!(f.cursor.rotation, old);
}

#[test]
fn choosing_the_last_row_as_next_tops_up_after_it() {
    let mut s = filled(40, 180.0);
    let (player, _) = filler(&s);
    fp_model::apply(&mut s, fp_model::Command::Play(player)).unwrap();
    let last = *rows(&s).last().unwrap();
    fp_model::apply(&mut s, fp_model::Command::SetNext(player, last)).unwrap();
    let after = rows(&s);
    let at = after.iter().position(|e| *e == last).unwrap();
    assert!(
        after.len() - at > 9,
        "30 min of 3-min songs after the chosen next: {} rows",
        after.len() - at - 1
    );
    assert_eq!(s.player(player).unwrap().next, Some(last));
}

#[test]
fn rows_added_after_the_last_row_on_air_become_next() {
    let mut s = filled(40, 180.0);
    let (player, _) = filler(&s);
    fp_model::apply(&mut s, fp_model::Command::Play(player)).unwrap();
    let last = *rows(&s).last().unwrap();
    fp_model::apply(&mut s, fp_model::Command::SetNext(player, last)).unwrap();
    next(&mut s, player);
    let p = s.player(player).unwrap();
    assert_eq!(p.current, Some(last));
    let list = rows(&s);
    let at = list.iter().position(|e| *e == last).unwrap();
    assert!(p.next.is_some());
    assert_eq!(p.next, list.get(at + 1).copied());
}

#[test]
fn a_next_before_the_row_on_air_still_tops_up() {
    let mut s = filled(40, 180.0);
    let (player, _) = filler(&s);
    fp_model::apply(&mut s, fp_model::Command::Play(player)).unwrap();
    next(&mut s, player);
    next(&mut s, player);
    let first = rows(&s)[0];
    fp_model::apply(&mut s, fp_model::Command::SetNext(player, first)).unwrap();
    let len = rows(&s).len();
    settle(&mut s);
    assert_eq!(
        rows(&s).len(),
        len,
        "the rows from next on already cover 30 min"
    );
    assert_eq!(s.player(player).unwrap().next, Some(first));
}
