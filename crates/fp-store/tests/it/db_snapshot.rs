#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use std::path::PathBuf;
use std::time::Instant;

use fp_model::{
    Cart, CartColor, CartId, CartKind, CartPage, CartPageId, CartwallSession, ColumnWidths, Dock,
    EntryId, IdGen, Layout, Library, Marker, MarkerSource, PlayMode, PlayerId, PlayerSession,
    Playlist, PlaylistEntry, PlaylistId, Playlists, Track, TrackId,
};
use fp_store::Db;
use fp_store::Snapshot;
use fp_store::db::snapshot::{read, write_changes, write_full};
use tempfile::TempDir;

fn open() -> (TempDir, Db) {
    let dir = tempfile::tempdir().unwrap();
    let db = Db::open(&dir.path().join("fauste.db")).unwrap();
    (dir, db)
}

fn track(n: u64) -> Track {
    Track::new(TrackId(n), PathBuf::from(format!("/music/{n}.wav")))
}

fn player(n: u64, playlist: u64) -> PlayerSession {
    PlayerSession {
        id: PlayerId(n),
        playlist: PlaylistId(playlist),
        current: None,
        next: None,
        next_explicit: false,
        mode: PlayMode::Continuous,
        stop_after_current: false,
        position_secs: 0.0,
        volume: 1.0,
        columns: ColumnWidths::default(),
        history: Vec::new(),
        pending_start: None,
    }
}

fn rich() -> Snapshot {
    let mut library = Library::default();
    for n in 1..=3 {
        library.insert(track(n));
    }
    let mut playlists = Playlists::default();
    let mut main = Playlist::new(PlaylistId(10), "Main");
    let mut played = PlaylistEntry::new(EntryId(100), TrackId(1));
    played.played_by = vec![PlayerId(1)];
    played.repeat = true;
    played.stop_after = true;
    main.entries.push(played);
    main.entries
        .push(PlaylistEntry::new(EntryId(101), TrackId(2)));
    playlists.add(main);
    let mut second = Playlist::new(PlaylistId(11), "Second");
    second
        .entries
        .push(PlaylistEntry::new(EntryId(102), TrackId(3)));
    playlists.add(second);
    playlists.add(Playlist::new(PlaylistId(12), "Empty"));
    let mut p1 = player(1, 10);
    p1.current = Some(EntryId(100));
    p1.next = Some(EntryId(101));
    p1.next_explicit = true;
    p1.pending_start = Some((EntryId(101), 12.5));
    p1.position_secs = 3.25;
    p1.history = vec![EntryId(102)];
    let mut p2 = player(2, 11);
    p2.mode = PlayMode::Single;
    p2.stop_after_current = true;
    p2.volume = 0.5;
    let mut cart = Cart::empty(CartId(500));
    cart.name = "Jingle".into();
    cart.track = Some(TrackId(3));
    cart.kind = CartKind::default();
    cart.looped = true;
    cart.exclusive = true;
    cart.color = Some(CartColor::Red);
    let page = |id: u64, carts: Vec<Cart>| CartPage {
        id: CartPageId(id),
        name: format!("Page {id}"),
        rows: 2,
        cols: 3,
        carts,
    };
    let mut layout = Layout {
        player_shares: vec![(PlayerId(1), 1.5), (PlayerId(2), 0.5)],
        hidden_players: vec![PlayerId(2)],
        ..Layout::default()
    };
    layout.cartwall.dock = Dock::Left;
    layout.locked = true;
    let mut ids = IdGen::default();
    ids.observe(600);
    Snapshot {
        library,
        categories: Vec::new(),
        playlists,
        ids,
        players: vec![p1, p2],
        cartwall: CartwallSession {
            open: false,
            page: Some(CartPageId(21)),
        },
        layout,
        library_view: fp_model::LibraryView {
            open: true,
            rect: Some([10.0, 20.0, 800.0, 600.0]),
            ..Default::default()
        },
        cart_pages: vec![page(20, vec![cart]), page(21, Vec::new())],
        rotations: Vec::new(),
        default_rotation: None,
        air_times: Default::default(),
        filler: None,
        automation: None,
        as_run: Vec::new(),
        as_run_next: 0,
        as_run_before: None,
        templates: Vec::new(),
        week_plan: Default::default(),
        date_plans: Vec::new(),
    }
}

fn saved(snap: &Snapshot) -> (TempDir, Db) {
    let (dir, mut db) = open();
    write_full(&mut db, snap).unwrap();
    (dir, db)
}

fn total(db: &Db) -> i64 {
    i64::try_from(db.conn().total_changes()).unwrap()
}

fn count(db: &Db, table: &str) -> i64 {
    db.conn()
        .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}

#[test]
fn a_snapshot_round_trips() {
    let snap = rich();
    let (_dir, db) = saved(&snap);
    let (back, warnings) = read(&db);
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(back, snap);
}

#[test]
fn an_empty_database_reads_as_the_default() {
    let (_dir, db) = open();
    let (back, warnings) = read(&db);
    assert!(warnings.is_empty());
    assert_eq!(back, Snapshot::default());
}

#[test]
fn write_full_replaces_what_was_there() {
    let (_dir, mut db) = saved(&rich());
    let small = Snapshot::default();
    write_full(&mut db, &small).unwrap();
    assert_eq!(read(&db).0, small);
    assert_eq!(count(&db, "entries"), 0);
}

#[test]
fn changes_write_only_what_changed() {
    let a = rich();
    let (_dir, mut db) = saved(&a);
    let mut b = a.clone();
    b.library.get_mut(TrackId(2)).unwrap().markers.cue_in = Some(Marker {
        secs: 1.5,
        source: MarkerSource::Manual,
    });
    let before = total(&db);
    write_changes(&mut db, &a, &b).unwrap();
    assert_eq!(total(&db) - before, 1);
    let (back, warnings) = read(&db);
    assert!(warnings.is_empty());
    assert_eq!(back, b);
}

#[test]
fn an_unchanged_snapshot_writes_nothing() {
    let a = rich();
    let (_dir, mut db) = saved(&a);
    let before = total(&db);
    write_changes(&mut db, &a, &a.clone()).unwrap();
    assert_eq!(total(&db), before);
}

#[test]
fn changes_cover_every_part() {
    let a = rich();
    let (_dir, mut db) = saved(&a);
    let mut b = a.clone();
    // Tracks: one added, one removed (its entry goes with the playlist edit).
    b.library.insert(track(4));
    b.library.remove(TrackId(1));
    // Playlists: reorder, rename, an entry moves to another playlist.
    let mut lists: Vec<Playlist> = b.playlists.iter().cloned().collect();
    lists.reverse();
    lists[0].name = "Renamed".into();
    let moved = lists[2].entries.remove(1);
    lists[1].entries.push(moved);
    lists[2].entries.remove(0);
    let mut playlists = Playlists::default();
    for l in lists {
        playlists.add(l);
    }
    b.playlists = playlists;
    b.players.reverse();
    b.players[0].volume = 0.25;
    b.cart_pages.remove(0);
    b.cart_pages[0].name = "Other".into();
    b.cartwall.open = true;
    b.layout.locked = false;
    b.ids.observe(900);
    write_changes(&mut db, &a, &b).unwrap();
    let (back, warnings) = read(&db);
    assert!(warnings.is_empty(), "{warnings:?}");
    assert_eq!(back, b);
}

#[test]
fn removing_a_playlist_removes_its_entries() {
    let a = rich();
    let (_dir, mut db) = saved(&a);
    let mut b = a.clone();
    let mut playlists = Playlists::default();
    for l in a.playlists.iter().filter(|l| l.id != PlaylistId(10)) {
        playlists.add(l.clone());
    }
    b.playlists = playlists;
    write_changes(&mut db, &a, &b).unwrap();
    assert_eq!(count(&db, "entries"), 1);
    assert_eq!(read(&db).0, b);
}

#[test]
fn a_bad_track_row_loads_as_a_bare_track_with_a_warning() {
    let snap = rich();
    let (_dir, db) = saved(&snap);
    db.conn()
        .execute("UPDATE tracks SET doc = '{' WHERE id = 2", [])
        .unwrap();
    let (back, warnings) = read(&db);
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert_eq!(back.library.get(TrackId(2)), Some(&track(2)));
    assert_eq!(back.library.len(), 3);
    assert_eq!(back.library.get(TrackId(1)), snap.library.get(TrackId(1)));
    assert_eq!(back.playlists, snap.playlists);
}

#[test]
fn a_bad_entry_row_loads_as_a_bare_entry_with_a_warning() {
    let snap = rich();
    let (_dir, db) = saved(&snap);
    db.conn()
        .execute("UPDATE entries SET doc = 'x' WHERE id = 100", [])
        .unwrap();
    let (back, warnings) = read(&db);
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    let entry = &back.playlists.get(PlaylistId(10)).unwrap().entries[0];
    assert_eq!(*entry, PlaylistEntry::new(EntryId(100), TrackId(1)));
    assert_eq!(back.playlists.get(PlaylistId(10)).unwrap().entries.len(), 2);
}

#[test]
fn a_bad_player_row_is_dropped_with_a_warning() {
    let snap = rich();
    let (_dir, db) = saved(&snap);
    db.conn()
        .execute("UPDATE players SET doc = '[]' WHERE id = 1", [])
        .unwrap();
    let (back, warnings) = read(&db);
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert_eq!(back.players, vec![snap.players[1].clone()]);
}

#[test]
fn a_bad_cart_page_row_is_dropped_with_a_warning() {
    let snap = rich();
    let (_dir, db) = saved(&snap);
    db.conn()
        .execute("UPDATE cart_pages SET doc = '3' WHERE id = 20", [])
        .unwrap();
    let (back, warnings) = read(&db);
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert_eq!(back.cart_pages, vec![snap.cart_pages[1].clone()]);
}

#[test]
fn a_bad_pending_start_is_dropped_with_a_warning() {
    let snap = rich();
    let (_dir, db) = saved(&snap);
    let bad: String = db
        .conn()
        .query_row("SELECT doc FROM players WHERE id = 1", [], |r| r.get(0))
        .unwrap();
    let bad = bad.replace("[101,12.5]", "\"soon\"");
    assert!(bad.contains("soon"));
    db.conn()
        .execute("UPDATE players SET doc = ?1 WHERE id = 1", [bad])
        .unwrap();
    let (back, warnings) = read(&db);
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert_eq!(back.players[0].pending_start, None);
}

#[test]
fn a_bad_layout_loads_as_the_default_with_its_notes() {
    let snap = rich();
    let (_dir, db) = saved(&snap);
    db.conn()
        .execute(
            "UPDATE kv SET value = '{\"locked\":3,\"hidden_players\":[2,\"x\"]}' WHERE key = 'layout'",
            [],
        )
        .unwrap();
    let (back, warnings) = read(&db);
    assert_eq!(warnings.len(), 2, "{warnings:?}");
    assert!(!back.layout.locked);
    assert_eq!(back.layout.hidden_players, vec![PlayerId(2)]);
    db.conn()
        .execute("UPDATE kv SET value = '[1' WHERE key = 'layout'", [])
        .unwrap();
    let (back, warnings) = read(&db);
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert_eq!(back.layout, Layout::default());
}

#[test]
fn a_bad_ids_or_cartwall_session_loads_as_the_default() {
    let snap = rich();
    let (_dir, db) = saved(&snap);
    db.conn()
        .execute(
            "UPDATE kv SET value = '\"x\"' WHERE key IN ('ids', 'cartwall_session')",
            [],
        )
        .unwrap();
    let (back, warnings) = read(&db);
    assert_eq!(warnings.len(), 2, "{warnings:?}");
    assert_eq!(back.ids, IdGen::default());
    assert_eq!(back.cartwall, CartwallSession::default());
}

#[test]
fn the_key_columns_win_over_the_document() {
    let snap = rich();
    let (_dir, db) = saved(&snap);
    db.conn()
        .execute("UPDATE tracks SET path = '/elsewhere.wav' WHERE id = 1", [])
        .unwrap();
    let (back, _) = read(&db);
    assert_eq!(
        back.library.get(TrackId(1)).unwrap().path,
        PathBuf::from("/elsewhere.wav")
    );
}

#[test]
#[ignore = "measured; run with --release"]
fn one_marker_edit_saves_in_under_10_ms_with_20000_tracks() {
    let mut library = Library::default();
    for n in 1..=20_000u64 {
        library.insert(track(n));
    }
    let mut playlists = Playlists::default();
    for p in 0..100u64 {
        let mut list = Playlist::new(PlaylistId(100_000 + p), format!("List {p}"));
        for e in 0..50u64 {
            list.entries.push(PlaylistEntry::new(
                EntryId(1_000_000 + p * 50 + e),
                TrackId(1 + (p * 50 + e) % 20_000),
            ));
        }
        playlists.add(list);
    }
    let a = Snapshot {
        library,
        playlists,
        ..Snapshot::default()
    };
    let (_dir, mut db) = saved(&a);
    let mut b = a.clone();
    b.library.get_mut(TrackId(777)).unwrap().markers.cue_in = Some(Marker {
        secs: 2.0,
        source: MarkerSource::Manual,
    });
    let start = Instant::now();
    write_changes(&mut db, &a, &b).unwrap();
    let took = start.elapsed();
    eprintln!("write_changes of one marker edit: {took:?}");
    assert!(took.as_millis() < 10, "took {took:?}");
}

#[test]
fn a_blob_doc_loads_as_a_bare_track_and_spares_the_other_rows() {
    let snap = rich();
    let (_dir, db) = saved(&snap);
    db.conn()
        .execute("UPDATE tracks SET doc = X'00' WHERE id = 2", [])
        .unwrap();
    let (back, warnings) = read(&db);
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert_eq!(back.library.len(), 3);
    assert_eq!(back.library.get(TrackId(2)), Some(&track(2)));
    assert_eq!(back.library.get(TrackId(3)), snap.library.get(TrackId(3)));
}

#[test]
fn an_entry_with_a_text_track_is_dropped_and_spares_the_other_entries() {
    let snap = rich();
    let (_dir, db) = saved(&snap);
    db.conn()
        .execute_batch(
            "PRAGMA foreign_keys = OFF; \
             UPDATE entries SET track = 'x' WHERE id = 100;",
        )
        .unwrap();
    let (back, warnings) = read(&db);
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    let main = back.playlists.get(PlaylistId(10)).unwrap();
    assert_eq!(main.entries.len(), 1);
    assert_eq!(main.entries[0].id, EntryId(101));
    assert_eq!(back.playlists.get(PlaylistId(11)).unwrap().entries.len(), 1);
}

#[test]
fn players_and_cart_pages_are_added_through_changes() {
    let a = Snapshot {
        players: vec![player(1, 10)],
        cart_pages: vec![rich().cart_pages[0].clone()],
        ..rich()
    };
    let (_dir, mut db) = saved(&a);
    let mut b = a.clone();
    b.players.push(player(2, 11));
    b.cart_pages.push(rich().cart_pages[1].clone());
    write_changes(&mut db, &a, &b).unwrap();
    assert_eq!(read(&db).0, b);
}

#[test]
fn renaming_a_playlist_in_place_writes_only_its_row() {
    let a = rich();
    let (_dir, mut db) = saved(&a);
    let mut b = a.clone();
    b.playlists.get_mut(PlaylistId(12)).unwrap().name = "Renamed".into();
    let before = total(&db);
    write_changes(&mut db, &a, &b).unwrap();
    assert_eq!(total(&db) - before, 1);
    assert_eq!(read(&db).0, b);
}

#[test]
fn removing_a_referenced_track_fails_and_leaves_the_previous_state() {
    let a = rich();
    let (_dir, mut db) = saved(&a);
    let mut b = a.clone();
    b.library.remove(TrackId(1));
    assert!(write_changes(&mut db, &a, &b).is_err());
    let (back, warnings) = read(&db);
    assert!(warnings.is_empty());
    assert_eq!(back, a);
}
