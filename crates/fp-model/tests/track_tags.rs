#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O23: tag fields, `ApplyTags` and the edit rule.

mod common;

use std::path::PathBuf;

use common::{entries, fixture, p0};
use fp_model::{
    AppState, CartEdit, CartKind, Command, Config, FileState, TagEditBlock, Track, TrackAnalysis,
    TrackId, TrackTags, apply, parse_tag_date, tag_edit_block,
};

fn track_of(state: &AppState, n: usize) -> TrackId {
    state.playlists.entry(entries(state)[n]).unwrap().track
}

fn tags() -> TrackTags {
    TrackTags {
        title: "Real Title".into(),
        artist: "Real Artist".into(),
        album: "An Album".into(),
        album_artist: "Various".into(),
        date: Some("1999-03-07".into()),
        genre: "Pop".into(),
        composer: "A. Composer".into(),
        comment: "A note".into(),
    }
}

/// A state whose track `n` is analysed, its tags are read and its file is fine.
fn ready(n: usize) -> (AppState, TrackId) {
    let mut s = fixture(3);
    let t = track_of(&s, n);
    apply(
        &mut s,
        Command::ApplyAnalysis {
            track: t,
            analysis: Box::new(TrackAnalysis {
                duration_secs: 180.0,
                ..TrackAnalysis::default()
            }),
        },
    )
    .unwrap();
    apply(
        &mut s,
        Command::ApplyTags {
            track: t,
            tags: Box::new(tags()),
        },
    )
    .unwrap();
    (s, t)
}

#[test]
fn apply_tags_stores_every_field_and_marks_the_tags_read() {
    let (s, t) = ready(0);
    let track = s.library.get(t).unwrap();
    assert_eq!(track.tags(), tags());
    assert!(track.tags_read);
    assert_eq!(track.title, "Real Title");
    assert_eq!(track.date.as_deref(), Some("1999-03-07"));
}

#[test]
fn apply_tags_can_clear_a_field_but_never_the_title() {
    let (mut s, t) = ready(0);
    let cleared = TrackTags {
        title: String::new(),
        album: String::new(),
        date: None,
        ..tags()
    };
    apply(
        &mut s,
        Command::ApplyTags {
            track: t,
            tags: Box::new(cleared),
        },
    )
    .unwrap();
    let track = s.library.get(t).unwrap();
    assert_eq!(track.album, "");
    assert_eq!(track.date, None);
    assert_eq!(
        track.title, "Real Title",
        "an empty title keeps the old one"
    );
}

#[test]
fn apply_tags_for_a_removed_track_does_nothing() {
    let (mut s, _) = ready(0);
    let before = s.clone();
    apply(
        &mut s,
        Command::ApplyTags {
            track: TrackId(9999),
            tags: Box::new(tags()),
        },
    )
    .unwrap();
    assert_eq!(s, before);
}

#[test]
fn a_new_analysis_asks_for_the_tags_again_and_keeps_them_until_then() {
    let (mut s, t) = ready(0);
    assert!(!s.library.get(t).unwrap().needs_tag_read());
    apply(
        &mut s,
        Command::ApplyAnalysis {
            track: t,
            analysis: Box::new(TrackAnalysis {
                duration_secs: 180.0,
                ..TrackAnalysis::default()
            }),
        },
    )
    .unwrap();
    let track = s.library.get(t).unwrap();
    assert!(track.needs_tag_read());
    assert_eq!(track.genre, "Pop", "analysis does not touch the new fields");
}

#[test]
fn only_analysed_playable_tracks_need_a_tag_read() {
    let mut s = fixture(1);
    let t = track_of(&s, 0);
    assert!(!s.library.get(t).unwrap().needs_tag_read(), "not analysed");
    apply(
        &mut s,
        Command::ApplyAnalysis {
            track: t,
            analysis: Box::new(TrackAnalysis::default()),
        },
    )
    .unwrap();
    assert!(s.library.get(t).unwrap().needs_tag_read());
    s.library.get_mut(t).unwrap().file_state = FileState::Missing;
    assert!(
        !s.library.get(t).unwrap().needs_tag_read(),
        "no file to read"
    );
}

#[test]
fn an_old_library_entry_loads_with_empty_tags() {
    let json = r#"{"id":7,"path":"/m/a.flac","title":"A","artist":"B","album":"C",
        "duration_secs":10.0,"kind":"Music","file_state":"Ok","markers":{},"analyzed":true}"#;
    let track: Track = serde_json::from_str(json).unwrap();
    assert_eq!(track.date, None);
    assert_eq!(track.genre, "");
    assert!(!track.tags_read);
    assert!(track.needs_tag_read());
}

#[test]
fn tag_text_is_trimmed_and_cut() {
    let long = format!("  {}  ", "é".repeat(100));
    let t = TrackTags {
        title: long,
        comment: "   ".into(),
        ..TrackTags::default()
    }
    .clamped(10);
    assert_eq!(t.title, "é".repeat(10));
    assert_eq!(t.comment, "", "only spaces is empty");
}

#[test]
fn a_tag_date_is_iso_8601_and_may_be_partial() {
    assert_eq!(parse_tag_date(""), Ok(None));
    assert_eq!(parse_tag_date("   "), Ok(None));
    for ok in [
        "1984",
        "0001",
        "9999",
        "2019-05",
        "2019-05-14",
        "2019-05-14T08",
        "2019-05-14T08:30",
        "2019-05-14T23:59:59",
        "2019-12-31T00:00:00",
    ] {
        assert_eq!(parse_tag_date(ok), Ok(Some(ok.to_owned())), "{ok}");
    }
    assert_eq!(
        parse_tag_date("  2019-05-14 "),
        Ok(Some("2019-05-14".to_owned())),
        "trimmed"
    );
}

#[test]
fn anything_but_an_iso_8601_date_is_refused() {
    for bad in [
        "0",
        "0000",
        "19",
        "10000",
        "-3",
        "19x4",
        "1984.5",
        "٣٣٣٣",
        "2019-5",
        "2019-00",
        "2019-13",
        "2019-05-00",
        "2019-05-32",
        "2019/05/14",
        "abc",
        "2019-05-14T",
        "2019-05-14T25",
        "2019-05-14T08:60",
        "2019-05-14T08:30:60",
        "2019-05-14 08:30",
        "2019-05-14T8",
        "2019-05-14T08:30:00:00",
        "2019-05-14T08:30:00Z",
        "2019-",
    ] {
        assert!(parse_tag_date(bad).is_err(), "{bad}");
    }
}

#[test]
fn a_date_round_trips_through_apply_tags() {
    let mut track = Track::new(TrackId(1), PathBuf::from("a.mp3"));
    track.apply_tags(&TrackTags {
        date: Some("2019-05-14".into()),
        ..TrackTags::default()
    });
    assert_eq!(track.tags().date.as_deref(), Some("2019-05-14"));
    track.apply_tags(&TrackTags::default());
    assert_eq!(track.tags().date, None);
}

#[test]
fn the_text_cap_is_a_validated_config_field() {
    let mut c = Config::default();
    assert_eq!(c.limits.max_tag_chars, 2000);
    c.limits.max_tag_chars = 3;
    let warnings = c.validate();
    assert_eq!(c.limits.max_tag_chars, 64);
    assert!(warnings.iter().any(|w| w.field == "limits.max_tag_chars"));
    c.limits.max_tag_chars = 10_000_000;
    c.validate();
    assert_eq!(c.limits.max_tag_chars, 100_000);
}

#[test]
fn tag_edit_block_allows_a_quiet_readable_track() {
    let (s, t) = ready(1);
    assert_eq!(tag_edit_block(&s, t, true), None);
}

#[test]
fn tag_edit_block_names_each_reason() {
    let (mut s, t) = ready(0);
    assert_eq!(
        tag_edit_block(&s, t, false),
        Some(TagEditBlock::UnsupportedFormat)
    );
    assert_eq!(
        tag_edit_block(&s, TrackId(9999), true),
        Some(TagEditBlock::FileUnavailable),
        "an unknown track"
    );
    s.library.get_mut(t).unwrap().tags_read = false;
    assert_eq!(tag_edit_block(&s, t, true), Some(TagEditBlock::TagsNotRead));
    s.library.get_mut(t).unwrap().tags_read = true;
    for state in [FileState::Missing, FileState::Unreadable] {
        s.library.get_mut(t).unwrap().file_state = state;
        assert_eq!(
            tag_edit_block(&s, t, true),
            Some(TagEditBlock::FileUnavailable)
        );
    }
}

#[test]
fn tag_edit_block_refuses_a_track_on_air_or_cued() {
    let (mut s, t) = ready(0);
    let p = p0(&s);
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    assert_eq!(tag_edit_block(&s, t, true), Some(TagEditBlock::Cued));
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(tag_edit_block(&s, t, true), Some(TagEditBlock::OnAir));
    apply(&mut s, Command::Pause(p)).unwrap();
    assert_eq!(
        tag_edit_block(&s, t, true),
        Some(TagEditBlock::OnAir),
        "paused is still on air"
    );
    apply(&mut s, Command::Stop(p)).unwrap();
    assert_eq!(tag_edit_block(&s, t, true), None);
}

#[test]
fn tag_edit_block_judges_the_file_not_the_entry() {
    let (mut s, t) = ready(0);
    let playlist = s.playlists.first_id().unwrap();
    apply(
        &mut s,
        Command::InsertTracks {
            playlist,
            index: 3,
            tracks: vec![t],
        },
    )
    .unwrap();
    let p = p0(&s);
    apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(tag_edit_block(&s, t, true), Some(TagEditBlock::OnAir));
}

#[test]
fn tag_edit_block_refuses_a_track_on_a_playing_cart() {
    let (mut s, _) = ready(0);
    let page = s.cartwall.pages[0].id;
    apply(
        &mut s,
        Command::AssignCartFile {
            page,
            index: 0,
            path: PathBuf::from("/carts/id.wav"),
        },
    )
    .unwrap();
    let cart_track = s.cartwall.pages[0].carts[0].track.unwrap();
    let cart = s.cartwall.pages[0].carts[0].id;
    apply(
        &mut s,
        Command::ApplyAnalysis {
            track: cart_track,
            analysis: Box::new(TrackAnalysis {
                duration_secs: 10.0,
                cue_out: Some(9.5),
                ..TrackAnalysis::default()
            }),
        },
    )
    .unwrap();
    apply(
        &mut s,
        Command::ApplyTags {
            track: cart_track,
            tags: Box::new(tags()),
        },
    )
    .unwrap();
    let edit = CartEdit {
        name: "ID".into(),
        kind: CartKind::Jingle,
        looped: false,
        exclusive: false,
    };
    apply(
        &mut s,
        Command::SetCart {
            page,
            index: 0,
            edit,
        },
    )
    .unwrap();
    assert_eq!(tag_edit_block(&s, cart_track, true), None, "a cart at rest");
    apply(&mut s, Command::FireCart(cart)).unwrap();
    assert_eq!(
        tag_edit_block(&s, cart_track, true),
        Some(TagEditBlock::OnCart)
    );
}
