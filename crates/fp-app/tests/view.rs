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
    PlayerStatus, RowStatus, TipField, cue_follow_target, cue_window_view, fader_from_gain,
    file_icon, file_problem, gain_from_fader, player_view, playlist_times, row_status, shown_entry,
    start_scroll_target, tag_edit_availability, track_tooltip, volume_db,
};
use fp_model::{
    AppState, AudioFormat, Command, Config, EntryId, FileState, MarkerKind, PlayerId, Track, apply,
};

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
    assert_eq!(file_icon(t), egui_phosphor::regular::FILE_X);
    t.file_state = FileState::Unreadable;
    assert_eq!(file_problem(t), Some("file-unreadable-tip"));
    assert_eq!(file_icon(t), egui_phosphor::regular::WARNING);
}

/// A stopped player shows the track Play will start (its next), ready at
/// its cue-in, with its waveform.
#[test]
fn a_stopped_player_shows_its_next_track_ready_to_play() {
    let (mut s, e, p) = state(3);
    let t1 = s.playlists.entry(e[1]).unwrap().track;
    s.library
        .get_mut(t1)
        .unwrap()
        .markers
        .set_auto(MarkerKind::CueIn, Some(2.0));
    apply(&mut s, Command::SetNext(p, e[1])).unwrap();
    assert_eq!(shown_entry(&s, p), Some(e[1]));
    // A position left over from the last track is not this one's.
    let v = player_view(&s, p, Some(150.0), 0.0).unwrap();
    assert_eq!(v.status, PlayerStatus::Stopped);
    assert_eq!(v.title.as_deref(), Some("Song 1"));
    assert_eq!((v.elapsed, v.total, v.remaining), (2.0, Some(200.0), 198.0));
    assert!(!v.end_warning);
    assert!(v.markers.position.is_some(), "the waveform has a position");
    // A short intro waiting to be played does not blink: nothing is on air.
    s.library
        .get_mut(t1)
        .unwrap()
        .markers
        .set_auto(MarkerKind::IntroEnd, Some(4.0));
    let v = player_view(&s, p, None, 0.25).unwrap();
    assert_eq!(v.intro, Some(2.0));
    assert_eq!(v.intro_blink, None);
    // On air, the current one is shown.
    apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(shown_entry(&s, p), Some(e[1]));
    apply(&mut s, Command::Stop(p)).unwrap();
    assert_eq!(shown_entry(&s, p), Some(e[2]));
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
fn an_idle_player_shows_its_next_and_one_without_next_shows_nothing() {
    let (s, _e, p) = state(2);
    let v = player_view(&s, p, None, 0.0).unwrap();
    assert_eq!(v.status, PlayerStatus::Stopped);
    assert_eq!(v.title.as_deref(), Some("Song 0"));
    assert_eq!(v.next_line.as_deref(), Some("Song 0 – Artist"));
    let (s, _e, p) = state(0);
    let v = player_view(&s, p, None, 0.0).unwrap();
    assert!(v.title.is_none());
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

fn mark(s: &mut AppState, track: fp_model::TrackId, kind: MarkerKind, secs: f64) {
    apply(
        s,
        Command::SetMarker {
            track,
            kind,
            secs: Some(secs),
        },
    )
    .unwrap();
}

fn use_markers(s: &mut AppState, on: bool) {
    let mut config = s.config.clone();
    config.players.use_cue_markers = on;
    apply(s, Command::UpdateConfig(Box::new(config))).unwrap();
}

#[test]
fn the_countdown_and_the_outro_follow_the_play_range() {
    let (mut s, e, p) = state(2);
    let t = s.playlists.entry(e[0]).unwrap().track;
    mark(&mut s, t, MarkerKind::CueOut, 190.0);
    s.library
        .get_mut(t)
        .unwrap()
        .markers
        .set_auto(MarkerKind::OutroStart, Some(170.0));
    apply(&mut s, Command::Play(p)).unwrap();
    let v = player_view(&s, p, Some(185.0), 0.0).unwrap();
    assert_eq!((v.remaining, v.outro), (5.0, Some(5.0)));
    use_markers(&mut s, false);
    let v = player_view(&s, p, Some(185.0), 0.0).unwrap();
    assert_eq!((v.remaining, v.outro), (15.0, Some(15.0)));
    assert!(!v.end_warning, "15 s is outside the default 10 s warning");
}

#[test]
fn a_waiting_player_shows_the_start_of_the_range_as_elapsed() {
    let (mut s, e, p) = state(2);
    let t = s.playlists.entry(e[0]).unwrap().track;
    mark(&mut s, t, MarkerKind::CueIn, 12.0);
    assert_eq!(player_view(&s, p, None, 0.0).unwrap().elapsed, 12.0);
    use_markers(&mut s, false);
    assert_eq!(player_view(&s, p, None, 0.0).unwrap().elapsed, 0.0);
}

#[test]
fn playlist_times_follow_the_play_range() {
    let (mut s, e, p) = state(3);
    for entry in &e {
        let t = s.playlists.entry(*entry).unwrap().track;
        mark(&mut s, t, MarkerKind::CueIn, 10.0);
        mark(&mut s, t, MarkerKind::CueOut, 190.0);
    }
    let playlist = s.playlists.first_id().unwrap();
    assert_eq!(playlist_times(&s, p, playlist, &[]).total, 540.0);
    use_markers(&mut s, false);
    assert_eq!(playlist_times(&s, p, playlist, &[]).total, 600.0);
}

#[test]
fn the_view_says_when_the_cue_marks_are_ignored() {
    let (mut s, e, p) = state(2);
    let t = s.playlists.entry(e[0]).unwrap().track;
    mark(&mut s, t, MarkerKind::CueIn, 12.0);
    let on = player_view(&s, p, None, 0.0).unwrap().markers;
    assert!(!on.ignored);
    use_markers(&mut s, false);
    let off = player_view(&s, p, None, 0.0).unwrap().markers;
    assert!(off.ignored);
    assert_eq!(off.cue_in, on.cue_in, "the marks are kept, only dimmed");
}

#[test]
fn the_player_view_carries_the_entry_notice() {
    let (mut s, e, p) = state(2);
    apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(player_view(&s, p, None, 0.0).unwrap().entry_notice, None);
    apply(&mut s, Command::ToggleEntryRepeat(e[0])).unwrap();
    assert_eq!(
        player_view(&s, p, None, 0.0).unwrap().entry_notice,
        Some(fp_model::EntryNotice::Repeats)
    );
}

#[test]
fn the_cue_window_view_needs_a_running_cue() {
    let (s, _, p) = state(2);
    assert_eq!(cue_window_view(&s, p, None), None);
}

#[test]
fn the_cue_window_view_shows_the_cued_entry_its_times_and_its_state() {
    let (mut s, e, p) = state(3);
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    let v = cue_window_view(&s, p, Some(50.0)).unwrap();
    assert_eq!(v.entry, e[0]);
    assert_eq!(v.title, "Song 0");
    assert_eq!(v.artist.as_deref(), Some("Artist"));
    assert_eq!(v.elapsed, 50.0);
    assert_eq!(v.total, Some(200.0));
    assert_eq!(v.remaining, 150.0, "to the end of the file");
    assert_eq!(v.position, Some(0.25));
    assert!(!v.paused);
    apply(&mut s, Command::SetCuePaused(p, true)).unwrap();
    assert!(cue_window_view(&s, p, Some(50.0)).unwrap().paused);
}

#[test]
fn a_cue_window_without_a_position_shows_the_start_of_the_range() {
    let (mut s, e, p) = state(2);
    let t = s.playlists.entry(e[0]).unwrap().track;
    mark(&mut s, t, MarkerKind::CueIn, 12.0);
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    let v = cue_window_view(&s, p, None).unwrap();
    assert_eq!(v.elapsed, 12.0);
    let broken = cue_window_view(&s, p, Some(f64::NAN)).unwrap();
    assert_eq!(broken.elapsed, 12.0);
}

#[test]
fn a_position_past_the_end_is_clamped() {
    let (mut s, _, p) = state(2);
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    let v = cue_window_view(&s, p, Some(900.0)).unwrap();
    assert_eq!((v.elapsed, v.remaining), (200.0, 0.0));
}

#[test]
fn an_unknown_length_gives_zero_times_and_no_position() {
    let (mut s, e, p) = state(2);
    let t = s.playlists.entry(e[0]).unwrap().track;
    s.library.get_mut(t).unwrap().duration_secs = 0.0;
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    let v = cue_window_view(&s, p, Some(5.0)).unwrap();
    assert_eq!((v.total, v.remaining, v.position), (None, 0.0, None));
}

#[test]
fn load_as_next_is_offered_only_when_it_changes_something() {
    let (mut s, e, p) = state(3);
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    // Cued entry is the derived next: loading it makes it explicit.
    assert!(cue_window_view(&s, p, None).unwrap().can_load_next);
    apply(&mut s, Command::CueToNext(p)).unwrap();
    assert!(
        !cue_window_view(&s, p, None).unwrap().can_load_next,
        "already explicit"
    );
    apply(&mut s, Command::CueEntry(p, e[2])).unwrap();
    assert!(cue_window_view(&s, p, None).unwrap().can_load_next);
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::CueEntry(p, e[0])).unwrap();
    assert!(
        !cue_window_view(&s, p, None).unwrap().can_load_next,
        "the current entry cannot be the next"
    );
}

#[test]
fn a_click_moves_a_running_cue_to_a_playable_other_row() {
    let (mut s, e, p) = state(3);
    assert_eq!(cue_follow_target(&s, p, e[2]), None, "no CUE running");
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    assert_eq!(cue_follow_target(&s, p, e[2]), Some(e[2]));
    assert_eq!(cue_follow_target(&s, p, e[0]), None, "already cued");
}

#[test]
fn a_click_on_a_missing_or_unreadable_file_leaves_the_cue() {
    let (mut s, e, p) = state(3);
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    for file in [FileState::Missing, FileState::Unreadable] {
        let track = s.playlists.entry(e[1]).unwrap().track;
        apply(&mut s, Command::SetFileState { track, state: file }).unwrap();
        assert_eq!(cue_follow_target(&s, p, e[1]), None, "{file:?}");
    }
}

#[test]
fn an_unknown_player_or_entry_gives_none() {
    let (mut s, e, p) = state(2);
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    assert_eq!(cue_follow_target(&s, PlayerId(99), e[1]), None);
    assert_eq!(cue_follow_target(&s, p, EntryId(9999)), None);
}

fn tip_track() -> Track {
    let mut t = Track::new(fp_model::TrackId(1), PathBuf::from("/m/Artist - Song.flac"));
    t.title = "Song".into();
    t.artist = "Artist".into();
    t.album = "Album".into();
    t.date = Some("1999-03-07".into());
    t.genre = "Pop".into();
    t.duration_secs = 200.0;
    t.format = Some(AudioFormat {
        sample_rate: 44_100,
        bits: Some(16),
        channels: 2,
    });
    t
}

#[test]
fn the_tooltip_lists_what_the_track_has_in_order() {
    let tip = track_tooltip(&tip_track());
    let fields: Vec<TipField> = tip.iter().map(|(f, _)| *f).collect();
    assert_eq!(
        fields,
        [
            TipField::Title,
            TipField::Artist,
            TipField::Album,
            TipField::Date,
            TipField::Genre,
            TipField::Duration,
            TipField::Format,
            TipField::Path
        ]
    );
    let value = |f| tip.iter().find(|(g, _)| *g == f).unwrap().1.clone();
    assert_eq!(value(TipField::Date), "1999-03-07");
    assert_eq!(value(TipField::Duration), "03:20");
    assert_eq!(value(TipField::Format), "FLAC · 44.1 kHz · 16 bit");
    assert_eq!(value(TipField::Path), "/m/Artist - Song.flac");
}

#[test]
fn a_missing_field_is_left_out() {
    let mut t = tip_track();
    t.album.clear();
    t.date = None;
    t.genre.clear();
    t.duration_secs = 0.0;
    t.format = None;
    let fields: Vec<TipField> = track_tooltip(&t).iter().map(|(f, _)| *f).collect();
    // The file extension still names the codec, so the format line stays.
    assert_eq!(
        fields,
        [
            TipField::Title,
            TipField::Artist,
            TipField::Format,
            TipField::Path
        ]
    );
}

#[test]
fn the_format_line_leaves_out_what_is_unknown() {
    let mut t = tip_track();
    t.format = Some(AudioFormat {
        sample_rate: 48_000,
        bits: None,
        channels: 2,
    });
    let line = track_tooltip(&t)
        .into_iter()
        .find(|(f, _)| *f == TipField::Format)
        .unwrap()
        .1;
    assert_eq!(
        line, "FLAC · 48 kHz",
        "lossy: no bit depth; whole kHz without a decimal"
    );
    t.path = PathBuf::from("/m/no extension");
    t.format = None;
    assert!(
        track_tooltip(&t)
            .iter()
            .all(|(f, _)| *f != TipField::Format)
    );
}

#[test]
fn tag_edit_availability_combines_the_rule_and_the_extension() {
    let (mut s, entries, p) = state(2);
    for t in s.library.iter_mut() {
        t.analyzed = true;
        t.tags_read = true;
    }
    let track = |s: &AppState, e: EntryId| s.playlists.entry(e).unwrap().track;
    let t0 = track(&s, entries[0]);
    assert_eq!(tag_edit_availability(&s, t0), None, "a flac at rest");
    s.library.get_mut(t0).unwrap().path = PathBuf::from("/m/a.dsf");
    assert_eq!(
        tag_edit_availability(&s, t0),
        Some(fp_model::TagEditBlock::UnsupportedFormat)
    );
    s.library.get_mut(t0).unwrap().path = PathBuf::from("/m/a.flac");
    apply(&mut s, Command::SetNext(p, entries[0])).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(
        tag_edit_availability(&s, t0),
        Some(fp_model::TagEditBlock::OnAir)
    );
}

#[test]
fn the_start_scroll_goes_to_the_next_entry_of_the_shown_playlist() {
    let (mut s, e, p) = state(5);
    assert_eq!(start_scroll_target(&s, p), Some(e[0]), "the derived next");
    apply(&mut s, Command::SetNext(p, e[3])).unwrap();
    assert_eq!(start_scroll_target(&s, p), Some(e[3]));
}

#[test]
fn there_is_no_start_scroll_without_a_next_or_for_another_playlist() {
    let (mut s, _, p) = state(0);
    assert_eq!(start_scroll_target(&s, p), None, "an empty playlist");
    apply(
        &mut s,
        Command::CreatePlaylistFromPaths {
            name: "Other".into(),
            paths: vec![PathBuf::from("/m/other.flac")],
        },
    )
    .unwrap();
    let other = s.playlists.iter().nth(1).unwrap().entries[0].id;
    apply(&mut s, Command::SetNext(p, other)).unwrap();
    assert_eq!(
        start_scroll_target(&s, p),
        None,
        "next is in another playlist"
    );
    assert_eq!(start_scroll_target(&s, PlayerId(999_999)), None);
}
