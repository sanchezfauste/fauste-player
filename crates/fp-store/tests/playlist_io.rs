#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Phase 2 spec P2.7: playlist files and cart page files.

use std::path::{Path, PathBuf};

use fp_model::{AppState, CartEdit, CartKind, Command, Config, Limits, apply};
use fp_store::playlist_io::{
    CartPageFileError, ExportEntry, PlaylistFileError, parse_cart_page, parse_playlist,
    write_cart_page, write_m3u8,
};

fn limits() -> Limits {
    Limits::default()
}

fn paths(bytes: &[u8], source: &str) -> Vec<PathBuf> {
    parse_playlist(bytes, Path::new(source), &limits())
        .unwrap()
        .entries
        .into_iter()
        .map(|e| e.path)
        .collect()
}

#[test]
fn m3u_with_extinf_and_comments() {
    let text = b"#EXTM3U\n# a comment\n\n#EXTINF:215,Band - Song\n/music/song.flac\n#EXTVLCOPT:foo\n/music/other.mp3\n";
    let list = parse_playlist(text, Path::new("/lists/a.m3u"), &limits()).unwrap();
    assert_eq!(list.entries.len(), 2);
    assert_eq!(list.entries[0].path, PathBuf::from("/music/song.flac"));
    assert_eq!(list.entries[0].title.as_deref(), Some("Band - Song"));
    assert_eq!(list.entries[0].duration_secs, Some(215.0));
    assert_eq!(
        list.entries[1].title, None,
        "EXTINF applies to the next entry only"
    );
}

#[test]
fn m3u8_utf8_and_bom_and_crlf() {
    let text = "\u{feff}#EXTM3U\r\n#EXTINF:-1,Canción\r\n/música/año.flac\r\n".as_bytes();
    let list = parse_playlist(text, Path::new("/l/x.m3u8"), &limits()).unwrap();
    assert_eq!(list.entries[0].path, PathBuf::from("/música/año.flac"));
    assert_eq!(list.entries[0].title.as_deref(), Some("Canción"));
    assert_eq!(list.entries[0].duration_secs, None, "-1 means unknown");
}

#[test]
fn m3u_latin1_fallback() {
    // "/música/año.mp3" in Windows-1252.
    let mut text = b"/m".to_vec();
    text.push(0xFA);
    text.extend_from_slice(b"sica/a");
    text.push(0xF1);
    text.extend_from_slice(b"o.mp3\n");
    assert_eq!(
        paths(&text, "/l/x.m3u"),
        vec![PathBuf::from("/música/año.mp3")]
    );
}

#[test]
fn relative_and_parent_paths_resolve_against_the_playlist() {
    let text = b"song.mp3\nsub/one.flac\n../up/two.wav\n./three.ogg\n";
    assert_eq!(
        paths(text, "/lists/radio/a.m3u"),
        vec![
            PathBuf::from("/lists/radio/song.mp3"),
            PathBuf::from("/lists/radio/sub/one.flac"),
            PathBuf::from("/lists/up/two.wav"),
            PathBuf::from("/lists/radio/three.ogg"),
        ]
    );
}

#[test]
fn file_urls_are_percent_decoded() {
    let text = b"file:///music/My%20Song%20%C3%B1.flac\nfile://localhost/music/b.mp3\n";
    assert_eq!(
        paths(text, "/l/a.m3u8"),
        vec![
            PathBuf::from("/music/My Song ñ.flac"),
            PathBuf::from("/music/b.mp3")
        ]
    );
}

#[cfg(unix)]
#[test]
fn windows_relative_paths_on_unix() {
    let text = b"Music\\Band\\song.mp3\n";
    assert_eq!(
        paths(text, "/lists/a.m3u"),
        vec![PathBuf::from("/lists/Music/Band/song.mp3")]
    );
}

#[test]
fn streams_are_skipped_and_counted() {
    let text = b"http://radio.example/stream\nHTTPS://x.example/a.mp3\n/music/a.mp3\nrtsp://x/y\n";
    let list = parse_playlist(text, Path::new("/l/a.m3u"), &limits()).unwrap();
    assert_eq!(list.entries.len(), 1);
    assert_eq!(list.skipped_streams, 3);
}

#[test]
fn pls_in_any_order() {
    let text = b"[playlist]\nTitle2=Second\nFile2=/m/b.mp3\nFile1=a.mp3\nLength1=61\nLength2=-1\nNumberOfEntries=5\nVersion=2\n";
    let list = parse_playlist(text, Path::new("/l/x.pls"), &limits()).unwrap();
    let got: Vec<_> = list
        .entries
        .iter()
        .map(|e| (e.path.clone(), e.title.clone(), e.duration_secs))
        .collect();
    assert_eq!(
        got,
        vec![
            (PathBuf::from("/l/a.mp3"), None, Some(61.0)),
            (PathBuf::from("/m/b.mp3"), Some("Second".into()), None),
        ]
    );
}

#[test]
fn pls_is_detected_by_content() {
    let text = b"[playlist]\nFile1=/m/a.mp3\n";
    assert_eq!(paths(text, "/l/list.txt"), vec![PathBuf::from("/m/a.mp3")]);
}

#[test]
fn oversized_files_are_refused() {
    let small = Limits {
        max_playlist_file_bytes: 10,
        ..Limits::default()
    };
    assert!(matches!(
        parse_playlist(b"/music/a-long-name.mp3\n", Path::new("/a.m3u"), &small),
        Err(PlaylistFileError::TooLarge { .. })
    ));
}

#[test]
fn binary_garbage_does_not_panic() {
    let junk: Vec<u8> = (0..=255u8).cycle().take(4096).collect();
    let _ = parse_playlist(&junk, Path::new("/a.m3u"), &limits());
    let _ = parse_playlist(&junk, Path::new("/a.pls"), &limits());
}

#[test]
fn m3u8_export_round_trips() {
    let entries = vec![
        ExportEntry {
            path: PathBuf::from("/music/á b.flac"),
            title: Some("Band - Song".into()),
            duration_secs: Some(214.6),
        },
        ExportEntry {
            path: PathBuf::from("/music/c.mp3"),
            title: None,
            duration_secs: None,
        },
    ];
    let text = write_m3u8(&entries);
    assert!(
        text.starts_with("#EXTM3U\n#EXTINF:215,Band - Song\n/music/á b.flac\n"),
        "{text}"
    );
    let back = parse_playlist(text.as_bytes(), Path::new("/x/a.m3u8"), &limits()).unwrap();
    let paths: Vec<_> = back.entries.iter().map(|e| e.path.clone()).collect();
    assert_eq!(
        paths,
        vec![
            PathBuf::from("/music/á b.flac"),
            PathBuf::from("/music/c.mp3")
        ]
    );
}

fn page_state() -> AppState {
    let mut state = AppState::new(Config::default(), "Main");
    let page = state.cartwall.pages[0].id;
    apply(
        &mut state,
        Command::RenameCartPage {
            page,
            name: "General".into(),
        },
    )
    .unwrap();
    apply(
        &mut state,
        Command::AssignCartFile {
            page,
            index: 2,
            path: PathBuf::from("/carts/id.wav"),
        },
    )
    .unwrap();
    let edit = CartEdit {
        name: "Station ID".into(),
        kind: CartKind::Jingle,
        looped: false,
        exclusive: true,
    };
    apply(
        &mut state,
        Command::SetCart {
            page,
            index: 2,
            edit,
        },
    )
    .unwrap();
    let edit = CartEdit {
        name: "Bed".into(),
        kind: CartKind::Effect,
        looped: true,
        exclusive: false,
    };
    apply(
        &mut state,
        Command::SetCart {
            page,
            index: 5,
            edit,
        },
    )
    .unwrap();
    state
}

#[test]
fn cart_page_round_trips() {
    let state = page_state();
    let text = write_cart_page(&state.cartwall.pages[0], &state.library);
    let import =
        parse_cart_page(text.as_bytes(), Path::new("/x/p.cartpage.json"), &limits()).unwrap();
    assert_eq!(
        (import.name.as_str(), import.rows, import.cols),
        ("General", 2, 8)
    );
    let id = import.carts.iter().find(|c| c.0 == 2).unwrap();
    assert_eq!(
        id.1,
        CartEdit {
            name: "Station ID".into(),
            kind: CartKind::Jingle,
            looped: false,
            exclusive: true
        }
    );
    assert_eq!(id.2, Some(PathBuf::from("/carts/id.wav")));
    let bed = import.carts.iter().find(|c| c.0 == 5).unwrap();
    assert!(bed.1.looped && bed.2.is_none());
}

#[test]
fn cart_page_with_bad_grid_is_clamped_and_relative_files_resolve() {
    let text = br#"{"format":"fauste-cart-page","version":1,"name":"X","rows":0,"cols":500,
        "carts":[{"position":1,"name":"A","kind":"spot","file":"a.wav"},{"position":9999,"name":"far"}]}"#;
    let import = parse_cart_page(text, Path::new("/pages/x.cartpage.json"), &limits()).unwrap();
    assert_eq!((import.rows, import.cols), (1, 16));
    assert_eq!(import.carts.len(), 2);
    assert_eq!(import.carts[0].0, 0, "positions are 1-based in the file");
    assert_eq!(import.carts[0].1.kind, CartKind::Spot);
    assert_eq!(import.carts[0].2, Some(PathBuf::from("/pages/a.wav")));
}

#[test]
fn cart_page_with_wrong_format_is_refused() {
    let text = br#"{"format":"something-else","version":1}"#;
    assert!(matches!(
        parse_cart_page(text, Path::new("/x.json"), &limits()),
        Err(CartPageFileError::NotACartPage)
    ));
    assert!(matches!(
        parse_cart_page(b"not json", Path::new("/x.json"), &limits()),
        Err(CartPageFileError::Invalid(_))
    ));
    let newer = br#"{"format":"fauste-cart-page","version":99}"#;
    assert!(matches!(
        parse_cart_page(newer, Path::new("/x.json"), &limits()),
        Err(CartPageFileError::TooNew(99))
    ));
}
