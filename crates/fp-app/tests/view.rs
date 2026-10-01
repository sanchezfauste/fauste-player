#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::float_cmp
)]
//! Pure view-model logic (spec §3 rules 1 and 17–20).

use std::path::PathBuf;

use fp_app::ui::format::{clock, countdown, number_width};
use fp_app::ui::view::{
    PlayerStatus, RowStatus, fader_from_gain, file_problem, gain_from_fader, player_view,
    playlist_times, row_status, volume_db,
};
use fp_model::{AppState, Command, Config, EntryId, FileState, MarkerKind, PlayerId, apply};

fn state(tracks: usize) -> (AppState, Vec<EntryId>, PlayerId) {
    let mut state = AppState::new(Config::default(), "Main");
    let playlist = state.playlists.first_id().unwrap();
    let paths = (0..tracks)
        .map(|i| PathBuf::from(format!("/m/Artist {i} - Song {i}.flac")))
        .collect();
    apply(
        &mut state,
        Command::InsertPaths {
            playlist,
            index: 0,
            paths,
        },
    )
    .unwrap();
    for (i, t) in state.library.iter_mut().enumerate() {
        t.duration_secs = 200.0;
        t.title = format!("Song {i}");
        t.artist = "Artist".into();
    }
    let entries = state
        .playlists
        .get(playlist)
        .unwrap()
        .entries
        .iter()
        .map(|e| e.id)
        .collect();
    let p = state.players[0].id;
    (state, entries, p)
}

fn entry(s: &fp_model::AppState, id: fp_model::EntryId) -> fp_model::PlaylistEntry {
    s.playlists.entry(id).unwrap().clone()
}

#[test]
fn countdown_shows_minutes_seconds_and_tenths() {
    assert_eq!(countdown(147.25), ("-02:27".to_owned(), ".2".to_owned()));
    assert_eq!(countdown(0.0), ("-00:00".to_owned(), ".0".to_owned()));
    assert_eq!(countdown(-3.0), ("-00:00".to_owned(), ".0".to_owned()));
    assert_eq!(countdown(3_723.9), ("-1:02:03".to_owned(), ".9".to_owned()));
}

#[test]
fn clocks_switch_to_hours_after_one_hour() {
    assert_eq!(clock(222.0), "03:42");
    assert_eq!(clock(7_401.0), "2:03:21");
    assert_eq!(clock(f64::NAN), "00:00");
}

#[test]
fn track_numbers_use_at_least_two_digits_and_grow_with_the_list() {
    assert_eq!(number_width(9), 2);
    assert_eq!(number_width(99), 2);
    assert_eq!(number_width(124), 3);
    assert_eq!(number_width(10_000), 5);
}

#[test]
fn rows_show_on_air_next_played_and_unavailable() {
    let (mut s, e, p) = state(4);
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap(); // e0 played, e1 on air, e2 next
    let t3 = s.playlists.entry(e[3]).unwrap().track;
    s.library.get_mut(t3).unwrap().file_state = FileState::Missing;
    assert_eq!(row_status(&s, p, &entry(&s, e[0])), RowStatus::Played);
    assert_eq!(row_status(&s, p, &entry(&s, e[1])), RowStatus::Current);
    assert_eq!(row_status(&s, p, &entry(&s, e[2])), RowStatus::Next);
    assert_eq!(row_status(&s, p, &entry(&s, e[3])), RowStatus::Unavailable);
}

#[test]
fn an_unavailable_track_says_why() {
    let (mut s, e, _p) = state(1);
    let id = s.playlists.entry(e[0]).unwrap().track;
    let t = s.library.get_mut(id).unwrap();
    assert_eq!(file_problem(t), None);
    t.file_state = FileState::Missing;
    assert_eq!(file_problem(t), Some("file-missing-tip"));
    t.file_state = FileState::Unreadable;
    assert_eq!(file_problem(t), Some("file-unreadable-tip"));
}

#[test]
fn rule17_the_countdown_turns_red_in_the_last_seconds() {
    let (mut s, _e, p) = state(2);
    apply(&mut s, Command::Play(p)).unwrap();
    let v = player_view(&s, p, Some(185.0), 0.0).unwrap();
    assert_eq!(v.status, PlayerStatus::OnAir);
    assert_eq!(
        (v.elapsed, v.total, v.remaining),
        (185.0, Some(200.0), 15.0)
    );
    assert!(!v.end_warning);
    let v = player_view(&s, p, Some(191.0), 0.0).unwrap();
    assert!(v.end_warning, "10 s is the default warning");
}

#[test]
fn rule18_the_intro_badge_counts_down_only_for_a_marked_intro_and_blinks_at_the_end() {
    let (mut s, e, p) = state(2);
    apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(
        player_view(&s, p, Some(1.0), 0.0).unwrap().intro,
        None,
        "no manual intro, no badge"
    );
    let t = s.playlists.entry(e[0]).unwrap().track;
    s.library
        .get_mut(t)
        .unwrap()
        .markers
        .set_manual(MarkerKind::IntroEnd, Some(12.0));
    let v = player_view(&s, p, Some(0.6), 0.0).unwrap();
    assert_eq!(v.intro, Some(11.4));
    assert!(
        v.intro_blink.is_none(),
        "no blinking with more than 3 s left"
    );
    let on = player_view(&s, p, Some(10.0), 0.1).unwrap();
    let off = player_view(&s, p, Some(10.0), 0.6).unwrap();
    assert_eq!((on.intro_blink, off.intro_blink), (Some(true), Some(false)));
    assert_eq!(player_view(&s, p, Some(12.5), 0.0).unwrap().intro, None);
}

#[test]
fn rule19_the_outro_badge_counts_down_to_cue_out() {
    let (mut s, e, p) = state(2);
    apply(&mut s, Command::Play(p)).unwrap();
    let t = s.playlists.entry(e[0]).unwrap().track;
    s.library
        .get_mut(t)
        .unwrap()
        .markers
        .set_auto(MarkerKind::OutroStart, Some(170.0));
    assert_eq!(player_view(&s, p, Some(160.0), 0.0).unwrap().outro, None);
    assert_eq!(
        player_view(&s, p, Some(175.0), 0.0).unwrap().outro,
        Some(25.0)
    );
}

#[test]
fn rule20_playlist_times_count_played_and_on_air_entries() {
    let (mut s, _e, p) = state(3);
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap(); // first played, second on air
    let playlist = s.playlists.first_id().unwrap();
    let t = playlist_times(&s, p, playlist, &[(p, 50.0)]);
    assert_eq!((t.total, t.elapsed, t.remaining), (600.0, 250.0, 350.0));
}

#[test]
fn the_fader_law_is_zero_db_at_the_top_and_silent_at_the_bottom() {
    assert_eq!(gain_from_fader(1.0), 1.0);
    assert_eq!(gain_from_fader(0.0), 0.0);
    let half = gain_from_fader(0.5);
    assert!((fader_from_gain(half) - 0.5).abs() < 1e-5);
    assert_eq!(volume_db(1.0), Some(0.0));
    assert_eq!(volume_db(0.0), None, "silence has no dB value");
    assert!((volume_db(0.5).unwrap() + 6.02).abs() < 0.01);
}

#[test]
fn an_idle_player_shows_nothing_playing_and_its_next() {
    let (s, _e, p) = state(2);
    let v = player_view(&s, p, None, 0.0).unwrap();
    assert_eq!(v.status, PlayerStatus::Stopped);
    assert!(v.title.is_none());
    assert_eq!(v.next_line.as_deref(), Some("Song 0 – Artist"));
}

#[test]
fn an_entry_on_air_on_another_player_is_marked_with_that_player() {
    let (mut s, e, p1) = state(3);
    let p2 = s.players[1].id;
    apply(&mut s, Command::Play(p1)).unwrap();
    apply(&mut s, Command::SetNext(p2, e[1])).unwrap();
    assert_eq!(row_status(&s, p1, &entry(&s, e[0])), RowStatus::Current);
    assert_eq!(
        row_status(&s, p2, &entry(&s, e[0])),
        RowStatus::OnAirElsewhere(1),
        "on air on player 1, not P2's own"
    );
}

#[test]
fn played_rows_belong_to_each_player() {
    let (mut s, e, p1) = state(3);
    let p2 = s.players[1].id;
    apply(&mut s, Command::Play(p1)).unwrap();
    apply(&mut s, Command::Play(p1)).unwrap();
    assert_eq!(row_status(&s, p1, &entry(&s, e[0])), RowStatus::Played);
    assert_eq!(
        row_status(&s, p2, &entry(&s, e[0])),
        RowStatus::Next,
        "P2's own next"
    );
    assert_eq!(row_status(&s, p2, &entry(&s, e[2])), RowStatus::Normal);
}

#[test]
fn playlist_times_follow_each_player() {
    let (mut s, _e, p1) = state(3);
    let p2 = s.players[1].id;
    let playlist = s.playlists.first_id().unwrap();
    apply(&mut s, Command::Play(p1)).unwrap();
    apply(&mut s, Command::Play(p1)).unwrap();
    let mine = playlist_times(&s, p1, playlist, &[(p1, 50.0)]);
    let theirs = playlist_times(&s, p2, playlist, &[(p1, 50.0)]);
    assert_eq!(mine.elapsed, 250.0, "one played, 50 s into the next");
    assert_eq!(theirs.elapsed, 0.0, "P2 has played nothing: {theirs:?}");
}

mod columns {
    use fp_app::ui::view::column_px;

    fn sum(px: [f32; 4]) -> f32 {
        px.iter().sum()
    }

    #[test]
    fn the_default_gives_the_minimums_and_splits_the_rest_60_40() {
        let px = column_px(None, 1000.0, 40.0, 60.0);
        assert_eq!((px[0], px[3]), (40.0, 60.0));
        assert!(
            (px[1] - 540.0).abs() < 0.01 && (px[2] - 360.0).abs() < 0.01,
            "{px:?}"
        );
        assert!((sum(px) - 1000.0).abs() < 0.01);
    }

    #[test]
    fn fractions_scale_with_the_width() {
        let f = Some([0.05, 0.5, 0.35, 0.1]);
        let a = column_px(f, 1000.0, 40.0, 60.0);
        let b = column_px(f, 1500.0, 40.0, 60.0);
        assert!((sum(b) - 1500.0).abs() < 0.01);
        assert!((b[1] / a[1] - 1.5).abs() < 0.01, "{a:?} {b:?}");
        assert!((b[2] / a[2] - 1.5).abs() < 0.01);
    }

    #[test]
    fn the_minimums_win_in_a_narrow_table() {
        let px = column_px(Some([0.01, 0.6, 0.38, 0.01]), 360.0, 40.0, 60.0);
        assert!(px[0] >= 40.0 && px[3] >= 60.0, "{px:?}");
        assert!(px.iter().all(|w| *w >= 0.0 && w.is_finite()));
        assert!((sum(px) - 360.0).abs() < 0.01);
        let tiny = column_px(None, 50.0, 40.0, 60.0);
        assert!(tiny.iter().all(|w| *w >= 0.0 && w.is_finite()), "{tiny:?}");
    }
}
