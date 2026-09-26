#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Crash recovery: restoring players from the last session snapshot.

mod common;

use common::{entries, fixture, p0};
use fp_model::{
    AppState, Command, Config, EngineAction, EntryId, IdGen, Library, PlayMode, PlayerSession,
    PlaylistId, Playlists, RestoreParts, Transport, apply,
};

fn parts(state: &AppState) -> RestoreParts {
    RestoreParts {
        config: state.config.clone(),
        library: state.library.clone(),
        playlists: state.playlists.clone(),
        cart_pages: state.cartwall.pages.clone(),
        ids: state.ids.clone(),
    }
}

#[test]
fn a_playing_player_comes_back_paused_at_its_position() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    let sessions = state.sessions(|_| 42.0);
    let (restored, actions) = AppState::restore(parts(&state), &sessions, "Main");
    let player = restored.player(p).unwrap();
    assert_eq!(
        (player.current, player.next, player.transport),
        (Some(e[0]), Some(e[1]), Transport::Paused)
    );
    assert!(actions.iter().any(|a| matches!(
        a,
        EngineAction::LoadPaused { player, request } if *player == p && request.entry == e[0] && request.from_secs == 42.0
    )));
    assert!(
        restored
            .players
            .iter()
            .all(|pl| pl.transport != Transport::Playing)
    );
}

#[test]
fn dangling_session_references_are_dropped() {
    let state = fixture(3);
    let e = entries(&state);
    let session = PlayerSession {
        id: state.players[0].id,
        playlist: PlaylistId(999_999),
        current: Some(EntryId(888_888)),
        next: Some(EntryId(777_777)),
        next_explicit: true,
        mode: PlayMode::Single,
        stop_after_current: true,
        position_secs: f64::NAN,
        volume: 7.0,
        columns: Default::default(),
    };
    let (restored, actions) = AppState::restore(parts(&state), &[session], "Main");
    let player = &restored.players[0];
    assert_eq!(player.playlist, restored.playlists.first_id().unwrap());
    assert_eq!(player.current, None);
    assert_eq!(
        player.next,
        Some(e[0]),
        "an idle player picks the first playable entry"
    );
    assert_eq!(player.transport, Transport::Stopped);
    assert!(
        !player.stop_after_current,
        "stop-after is meaningless in Single mode"
    );
    assert_eq!(player.volume, 1.0);
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, EngineAction::LoadPaused { .. }))
    );
}

#[test]
fn restoring_with_no_playlists_creates_a_default_one_and_moves_ids_forward() {
    let parts = RestoreParts {
        config: Config::default(),
        library: Library::default(),
        playlists: Playlists::default(),
        cart_pages: Vec::new(),
        ids: IdGen::default(),
    };
    let duplicate = PlayerSession {
        id: fp_model::PlayerId(500),
        playlist: PlaylistId(1),
        current: None,
        next: None,
        next_explicit: false,
        mode: PlayMode::Continuous,
        stop_after_current: false,
        position_secs: 0.0,
        volume: 0.5,
        columns: Default::default(),
    };
    let (mut restored, _) = AppState::restore(parts, &[duplicate.clone(), duplicate], "Main");
    assert_eq!(restored.playlists.len(), 1);
    assert_eq!(restored.players.len(), 4);
    assert_eq!(restored.players[0].id, fp_model::PlayerId(500));
    assert_ne!(
        restored.players[1].id,
        fp_model::PlayerId(500),
        "duplicate ids get a fresh id"
    );
    assert_eq!(restored.players[0].volume, 0.5);
    assert!(restored.ids.next_raw() > 500);
}

#[test]
fn rule22_restored_idle_players_do_not_point_at_an_entry_on_air() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    // A session saved with only the first player (for example before a
    // player was added): the others are created fresh on restore.
    let sessions: Vec<_> = state.sessions(|_| 10.0).into_iter().take(1).collect();
    let (restored, _) = AppState::restore(parts(&state), &sessions, "Main");
    for other in restored.players.iter().skip(1) {
        assert_ne!(other.next, Some(e[0]), "e0 is on air on P1");
    }
}
