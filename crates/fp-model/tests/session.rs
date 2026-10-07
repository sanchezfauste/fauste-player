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
        cartwall_session: state.cartwall.session(),
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
        history: Vec::new(),
        pending_start: None,
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
        cartwall_session: fp_model::CartwallSession::default(),
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
        history: Vec::new(),
        pending_start: None,
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
fn played_marks_from_before_independent_players_count_for_every_player() {
    let playlist: fp_model::Playlist = serde_json::from_str(
        r#"{"id":1,"name":"Main","entries":[{"id":7,"track":3,"played":true}]}"#,
    )
    .unwrap();
    let mut state = fixture(0);
    let players: Vec<_> = state.players.iter().map(|p| p.id).collect();
    let mut lists = fp_model::Playlists::default();
    lists.add(playlist);
    state.playlists = lists;
    state.normalize_played_marks();
    let entry = state.playlists.entry(fp_model::EntryId(7)).unwrap();
    assert!(players.iter().all(|p| entry.is_played_by(*p)));
}

#[test]
fn restored_cart_pages_are_normalised() {
    use fp_model::{Cart, CartId, CartPage, CartPageId, CartwallSession};
    let state = fixture(0);
    let mut cart = Cart::empty(CartId(900));
    cart.name = "kept".into();
    let broken = CartPage {
        id: CartPageId(800),
        name: "Broken".into(),
        rows: 0,
        cols: 60_000,
        carts: vec![cart],
    };
    let parts = RestoreParts {
        config: state.config.clone(),
        library: state.library.clone(),
        playlists: state.playlists.clone(),
        cart_pages: vec![broken.clone(), broken],
        cartwall_session: CartwallSession::default(),
        ids: state.ids.clone(),
    };
    let (restored, _) = AppState::restore(parts, &[], "Main");
    let limits = &restored.config.limits;
    let mut ids = std::collections::HashSet::new();
    for page in &restored.cartwall.pages {
        assert!((1..=limits.max_cart_rows).contains(&page.rows));
        assert!((1..=limits.max_cart_cols).contains(&page.cols));
        assert_eq!(
            page.carts.len(),
            usize::from(page.rows) * usize::from(page.cols)
        );
        assert!(ids.insert(page.id.0), "duplicate page id");
        for c in &page.carts {
            assert!(ids.insert(c.id.0), "duplicate cart id");
        }
        assert_eq!(page.carts[0].name, "kept");
    }
    assert_eq!(restored.cartwall.pages.len(), 2);
}

#[test]
fn orphan_library_tracks_are_dropped_on_restore() {
    let mut state = fixture(1);
    let orphan = fp_model::Track::new(fp_model::TrackId(777), std::path::PathBuf::from("/x.wav"));
    state.library.insert(orphan);
    let (restored, _) = AppState::restore(parts(&state), &[], "Main");
    assert!(restored.library.get(fp_model::TrackId(777)).is_none());
    assert_eq!(restored.library.iter().count(), 1);
}

#[test]
fn the_history_survives_a_session_round_trip_and_drops_gone_entries() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    for _ in 0..3 {
        apply(&mut state, Command::Play(p)).unwrap();
        fp_model::on_event(
            &mut state,
            fp_model::EngineEvent::FadeCompleted { player: p },
        );
    }
    assert_eq!(state.player(p).unwrap().history, vec![e[0], e[1]]);
    let sessions = state.sessions(|_| 0.0);
    apply(&mut state, Command::RemoveEntry(e[1])).unwrap();
    let (restored, _) = AppState::restore(parts(&state), &sessions, "Main");
    assert_eq!(restored.player(p).unwrap().history, vec![e[0]]);
}

#[test]
fn a_broken_history_loads_as_empty() {
    let good = serde_json::to_value(fixture(1).sessions(|_| 0.0).remove(0)).unwrap();
    for broken in [serde_json::json!("oops"), serde_json::json!([3, "x", 4])] {
        let mut doc = good.clone();
        doc["history"] = broken;
        let s: PlayerSession = serde_json::from_value(doc).unwrap();
        assert!(s.history.is_empty());
    }
}

#[test]
fn column_fractions_are_normalised_and_old_pixel_widths_ignored() {
    use fp_model::{ColumnWidths, TableColumn};
    let c = ColumnWidths::keyed([
        (TableColumn::Number, 1.0),
        (TableColumn::Title, 3.0),
        (TableColumn::Artist, 2.0),
        (TableColumn::Duration, 2.0),
    ]);
    let f = c.fractions.unwrap();
    assert!((f.values().sum::<f32>() - 1.0).abs() < 1e-6);
    assert!((f[&TableColumn::Title] - 0.375).abs() < 1e-6);
    for broken in [f32::NAN, -1.0] {
        let c = ColumnWidths {
            fractions: Some([(TableColumn::Title, broken), (TableColumn::Artist, 1.0)].into()),
        };
        assert_eq!(c.normalized().fractions, None, "{broken:?}");
    }
    let zero = ColumnWidths {
        fractions: Some([(TableColumn::Title, 0.0)].into()),
    };
    assert_eq!(zero.normalized().fractions, None);
    let old: ColumnWidths =
        serde_json::from_str(r#"{"number":40.0,"title":300.0,"duration":52.0}"#).unwrap();
    assert_eq!(old, ColumnWidths::default());
    assert_eq!(ColumnWidths::default().fractions, None);
}

#[test]
fn broken_column_fractions_load_as_the_default_layout() {
    let good = serde_json::to_value(fixture(1).sessions(|_| 0.0).remove(0)).unwrap();
    for broken in [
        serde_json::json!({"fractions": "wide"}),
        serde_json::json!({"fractions": [0.5, 0.5]}),
        serde_json::json!("oops"),
    ] {
        let mut doc = good.clone();
        doc["columns"] = broken;
        let s: PlayerSession = serde_json::from_value(doc).unwrap();
        assert_eq!(s.columns, fp_model::ColumnWidths::default());
    }
}

/// The `from_secs` the restored player `p` is loaded at.
fn loaded_at(actions: &[EngineAction], p: fp_model::PlayerId) -> f64 {
    actions
        .iter()
        .find_map(|a| match a {
            EngineAction::LoadPaused { player, request } if *player == p => Some(request.from_secs),
            _ => None,
        })
        .unwrap()
}

/// Crash recovery: a position at or past the cue-out would end the track
/// the moment Play resumes it (stopped at once in Single, the next track
/// at once in Continuous). It comes back at the cue-in instead.
#[test]
fn a_position_restored_at_the_end_comes_back_at_the_cue_in() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    let track = state.playlists.entry(e[0]).unwrap().track;
    for (kind, secs) in [
        (fp_model::MarkerKind::CueIn, 0.5),
        (fp_model::MarkerKind::CueOut, 179.0),
    ] {
        apply(
            &mut state,
            Command::SetMarker {
                track,
                kind,
                secs: Some(secs),
            },
        )
        .unwrap();
    }
    apply(&mut state, Command::Play(p)).unwrap();
    for (saved, restored) in [(179.0, 0.5), (240.0, 0.5), (178.9, 178.9)] {
        let sessions = state.sessions(|_| saved);
        let (restored_state, actions) = AppState::restore(parts(&state), &sessions, "Main");
        assert_eq!(loaded_at(&actions, p), restored, "saved at {saved}");
        assert_eq!(
            restored_state.player(p).unwrap().transport,
            Transport::Paused,
            "nothing goes on air by itself"
        );
    }
}

#[test]
fn a_restored_self_next_is_kept_and_the_player_stays_paused() {
    let mut state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    apply(&mut state, Command::Play(p)).unwrap();
    apply(&mut state, Command::SetNext(p, e[0])).unwrap();
    let sessions = state.sessions(|_| 10.0);
    let (restored, _) = AppState::restore(parts(&state), &sessions, "Main");
    let player = restored.player(p).unwrap();
    assert_eq!(
        (player.current, player.next, player.next_explicit),
        (Some(e[0]), Some(e[0]), true)
    );
    assert_eq!(player.transport, Transport::Paused);
}
