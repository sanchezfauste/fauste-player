#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Automation plan 5: the filler's last songs before a block fit (AU25).

use crate::automation_rig::{Rig, song, today};
use fp_model::track::MarkerKind;
use fp_model::{Command, Config, TrackId};

#[test]
fn automation_fit_settings_have_their_defaults_and_ranges() {
    let mut c = Config::default();
    assert_eq!(c.automation.max_fade_cut_secs, 30);
    assert_eq!(c.automation.min_song_play_secs, 45);
    c.automation.max_fade_cut_secs = 999;
    c.automation.min_song_play_secs = 999;
    c.validate();
    assert_eq!(c.automation.max_fade_cut_secs, 300);
    assert_eq!(c.automation.min_song_play_secs, 600);
}

/// The rig's music with a segue `before_end` seconds before the end.
fn with_segue(r: &mut Rig, before_end: f64) {
    for n in 1..=40 {
        let t = r.s.library.get_mut(TrackId(n)).unwrap();
        t.markers
            .set_auto(MarkerKind::SegueStart, Some(t.duration_secs - before_end));
    }
}

#[test]
fn the_timeline_uses_the_segue_point() {
    for (auto_segue, lead) in [(true, 170), (false, 180)] {
        let mut r = Rig::new(vec![(12, 0, vec![song(1)])]);
        r.s.config.players.auto_segue = auto_segue;
        with_segue(&mut r, 10.0);
        r.at_hms(10, 0, 0);
        r.apply(Command::StartAutomation);
        assert!(r.playing(r.filler_player()).is_some());
        let starts = fp_model::filler::starts_ahead(&r.s);
        let t0 = today().and_hms_opt(10, 0, 0).unwrap();
        let secs: Vec<i64> = starts
            .iter()
            .take(2)
            .map(|(_, at)| (*at - t0).num_seconds())
            .collect();
        assert_eq!(secs, vec![lead, 2 * lead], "auto_segue {auto_segue}");
    }
}

mod choosing {
    use std::path::PathBuf;

    use fp_model::rotation::StepSource;
    use fp_model::selection::{Fit, pick, pick_fitted};
    use fp_model::{AppState, Config, SplitMix64, Track, TrackId, TrackKind, WallTime};

    const NOW: WallTime = WallTime(1_000_000_000_000);

    /// Music tracks 901, 902, … of the given lengths (0: unknown).
    fn library(lengths: &[f64]) -> AppState {
        let mut s = AppState::new(Config::default(), "P");
        s.config.automation.track_separation_mins = 0;
        s.config.automation.track_separation_percent = 0;
        s.config.automation.artist_separation_mins = 0;
        for (i, len) in lengths.iter().enumerate() {
            let id = 901 + i as u64;
            let mut t = Track::new(TrackId(id), PathBuf::from(format!("/m/{id}.wav")));
            t.duration_secs = *len;
            t.kind = TrackKind::Music;
            s.library.insert(t);
        }
        s
    }

    fn fit(gap: f64) -> Fit {
        Fit {
            gap_secs: gap,
            max_cut_secs: 30.0,
            min_play_secs: 45.0,
        }
    }

    fn choose(s: &AppState, gap: f64, seed: u64) -> fp_model::selection::Picked {
        let music = StepSource::Kind(TrackKind::Music);
        pick_fitted(s, &music, NOW, &[], &mut SplitMix64(seed), Some(&fit(gap))).unwrap()
    }

    #[test]
    fn a_song_that_ends_the_gap_is_preferred() {
        let s = library(&[120.0, 210.0, 300.0]);
        for seed in 0..20 {
            let p = choose(&s, 200.0, seed);
            assert_eq!(p.track, TrackId(902), "seed {seed}");
            assert!(!p.unfitted);
        }
    }

    #[test]
    fn a_song_that_leaves_room_for_another_is_fine() {
        let s = library(&[180.0; 6]);
        let p = choose(&s, 600.0, 3);
        assert!(!p.unfitted);
    }

    #[test]
    fn no_song_starts_that_would_play_too_little() {
        let s = library(&[180.0, 25.0]);
        for seed in 0..20 {
            assert_eq!(choose(&s, 30.0, seed).track, TrackId(902), "seed {seed}");
        }
    }

    #[test]
    fn the_least_cut_wins_when_nothing_fits() {
        let s = library(&[240.0, 180.0]);
        let p = choose(&s, 100.0, 1);
        assert_eq!(p.track, TrackId(902));
        assert!(p.unfitted);
    }

    /// Found by the simulated night: when nothing fits, the least cut
    /// came from the whole pool, separation rules or not: an artist aired
    /// a quarter of an hour before came back.
    #[test]
    fn when_nothing_fits_the_separation_rules_still_come_first() {
        let mut s = library(&[240.0, 180.0, 200.0]);
        s.config.automation.track_separation_mins = 60;
        s.config.automation.artist_separation_mins = 30;
        for (id, artist) in [(901, "A"), (902, "B"), (903, "C"), (904, "B")] {
            if id == 904 {
                let mut t = Track::new(TrackId(id), PathBuf::from("/m/904.wav"));
                t.duration_secs = 200.0;
                t.kind = TrackKind::Music;
                s.library.insert(t);
            }
            s.library.get_mut(TrackId(id)).unwrap().artist = artist.into();
        }
        // B aired a quarter of an hour ago: 902, the shortest, is held back
        // by its artist; 903 by its own last airing.
        s.air_times
            .record(TrackId(904), WallTime(NOW.0 - 15 * 60_000));
        s.air_times
            .record(TrackId(903), WallTime(NOW.0 - 20 * 60_000));
        for seed in 0..20 {
            let p = choose(&s, 100.0, seed);
            assert_eq!(p.track, TrackId(901), "seed {seed}");
            assert!(p.unfitted);
        }
        // With every track held back, the artist rule gives way first.
        s.air_times
            .record(TrackId(901), WallTime(NOW.0 - 10 * 60_000));
        let p = choose(&s, 100.0, 1);
        assert_eq!(p.track, TrackId(902));
    }

    /// Maintainer, 2026-11-19: the fit and the separation rules both,
    /// whenever possible. A song that leaves room for another, separated,
    /// beats one that ends the gap but repeats an artist.
    #[test]
    fn a_fit_that_keeps_the_separation_beats_a_closer_one_that_breaks_it() {
        let mut s = library(&[210.0, 100.0, 180.0]);
        s.config.automation.artist_separation_mins = 30;
        for (id, artist) in [(901, "B"), (902, "C"), (903, "B")] {
            s.library.get_mut(TrackId(id)).unwrap().artist = artist.into();
        }
        s.air_times
            .record(TrackId(903), WallTime(NOW.0 - 10 * 60_000));
        for seed in 0..20 {
            let p = choose(&s, 200.0, seed);
            assert_eq!(p.track, TrackId(902), "seed {seed}");
            assert_eq!(p.relaxed, fp_model::selection::Relaxed::None);
        }
    }

    #[test]
    fn a_track_of_unknown_length_is_never_a_fit() {
        let s = library(&[0.0, 210.0]);
        for seed in 0..20 {
            let p = choose(&s, 200.0, seed);
            assert_eq!(p.track, TrackId(902), "seed {seed}");
        }
    }

    #[test]
    fn without_a_fit_the_choice_is_unchanged() {
        let s = library(&[120.0, 210.0, 300.0, 180.0, 200.0, 150.0, 260.0]);
        let music = StepSource::Kind(TrackKind::Music);
        for seed in 0..20 {
            let a = pick(&s, &music, NOW, &[], &mut SplitMix64(seed));
            let b = pick_fitted(&s, &music, NOW, &[], &mut SplitMix64(seed), None);
            assert_eq!(a, b, "seed {seed}");
        }
    }
}

mod before_a_block {
    use chrono::{NaiveDateTime, TimeDelta};
    use fp_model::{Command, EngineAction, EntryId, TrackId};

    use crate::automation_rig::{Rig, song, today};

    /// The rig's music in ten lengths, 150 s to 330 s.
    fn varied(r: &mut Rig) {
        for n in 1..=40u64 {
            let t = r.s.library.get_mut(TrackId(n)).unwrap();
            t.duration_secs = 150.0 + ((n * 7) % 10) as f64 * 20.0;
        }
    }

    fn length(r: &Rig, e: EntryId) -> f64 {
        r.s.track_for_entry(e).unwrap().duration_secs
    }

    /// The hand-over seen: how long the faded song had played and how
    /// much of it was left.
    #[derive(Debug, Clone, Copy)]
    struct HandOver {
        played: f64,
        left: f64,
    }

    /// A filler song on air: its row, when it started, its length.
    type OnAir = (EntryId, NaiveDateTime, f64);

    /// Walks the clock a second at a time from `from` to `until`; each
    /// filler song gives way to its next `segue` seconds before its end
    /// (the engine's segue), or ends at its end when it has no next.
    /// Returns the hand-over, if one happened.
    fn run(r: &mut Rig, from: NaiveDateTime, until: NaiveDateTime, segue: f64) -> Option<HandOver> {
        let f = r.filler_player();
        let mut on_air: Option<OnAir> = None;
        let mut seen = None;
        let mut t = from;
        while t <= until {
            let before = r.log.len();
            r.jump(t);
            let faded = r.log[before..]
                .iter()
                .any(|a| matches!(a, EngineAction::FadeOutAndStop { player, .. } if *player == f));
            if faded && let Some((_, since, len)) = on_air {
                let played = (t - since).num_milliseconds() as f64 / 1000.0;
                seen = Some(HandOver {
                    played,
                    left: len - played,
                });
            }
            let current = r.playing(f).and(r.player(f).current);
            if current != on_air.map(|(e, _, _)| e) {
                on_air = current.map(|e| (e, t, length(r, e)));
            }
            // With no next the player plays to the end, not to the segue.
            let lead = |len: f64| match r.player(f).next {
                Some(_) => len - segue,
                None => len,
            };
            if let Some((_, since, len)) = on_air
                && (t - since).num_seconds() as f64 >= lead(len)
            {
                r.end_current(f);
                let current = r.playing(f).and(r.player(f).current);
                on_air = current.map(|e| (e, t, length(r, e)));
            }
            t += TimeDelta::seconds(1);
        }
        seen
    }

    fn hms(h: u32, m: u32, s: u32) -> NaiveDateTime {
        today().and_hms_opt(h, m, s).unwrap()
    }

    #[test]
    fn the_filler_ends_on_a_song_that_fits_the_block() {
        let mut r = Rig::new(vec![(11, 0, vec![song(1)])]);
        r.s.config.players.auto_segue = false;
        varied(&mut r);
        r.at_hms(10, 40, 0);
        r.apply(Command::StartAutomation);
        let h = run(&mut r, hms(10, 40, 0), hms(11, 0, 5), 0.0).expect("a hand-over");
        assert!(h.played >= 45.0, "{h:?}");
        assert!(h.left <= 30.0 + 2.0, "{h:?}");
        assert_eq!(r.s.filler.as_ref().unwrap().unfitted, 0);
    }

    #[test]
    fn the_last_song_is_still_on_air_when_the_block_starts() {
        // AU24: the fade ends at the block's start, so the last song must
        // last until then; one that ends inside the fade would leave the
        // block to start early (or silence).
        let fade = 2.0;
        for k in 0..60i64 {
            let mut r = Rig::new(vec![(11, 0, vec![song(1)])]);
            r.s.config.players.auto_segue = false;
            r.s.config.automation.block_fade_ms = 2_000;
            varied(&mut r);
            let from = hms(10, 50, 0) + TimeDelta::seconds(k * 7);
            r.jump(from);
            r.apply(Command::StartAutomation);
            let h = run(&mut r, from, hms(11, 0, 5), 0.0).expect("a hand-over");
            assert!(h.left >= fade, "start +{}s: {h:?}", k * 7);
        }
    }

    #[test]
    fn a_drifting_queue_is_fitted_again_when_a_song_starts() {
        let mut r = Rig::new(vec![(11, 0, vec![song(1)])]);
        varied(&mut r);
        for n in 1..=40u64 {
            let t = r.s.library.get_mut(TrackId(n)).unwrap();
            let at = t.duration_secs - 10.0;
            t.markers
                .set_auto(fp_model::track::MarkerKind::SegueStart, Some(at));
        }
        // The queue is planned without the segues, so it drifts.
        r.s.config.players.auto_segue = false;
        r.at_hms(10, 40, 0);
        r.apply(Command::StartAutomation);
        r.s.config.players.auto_segue = true;
        let h = run(&mut r, hms(10, 40, 0), hms(11, 0, 5), 10.0).expect("a hand-over");
        assert!(h.played >= 45.0, "{h:?}");
        assert!(h.left <= 30.0 + 2.0, "{h:?}");
    }

    #[test]
    fn with_segues_no_song_starts_just_before_the_fade() {
        for segue in [15.0, 25.0, 40.0] {
            for start_min in [38u32, 40, 41, 43, 44, 46, 47] {
                let mut r = Rig::new(vec![(11, 0, vec![song(1)])]);
                varied(&mut r);
                for n in 1..=40u64 {
                    let t = r.s.library.get_mut(TrackId(n)).unwrap();
                    let at = t.duration_secs - segue;
                    t.markers
                        .set_auto(fp_model::track::MarkerKind::SegueStart, Some(at));
                }
                r.at_hms(10, start_min, 0);
                r.apply(Command::StartAutomation);
                let h =
                    run(&mut r, hms(10, start_min, 0), hms(11, 0, 5), segue).expect("a hand-over");
                let case = format!("segue {segue}, start 10:{start_min}: {h:?}");
                assert!(h.played >= 45.0, "{case}");
                assert!(h.left <= 30.0 + 2.0, "{case}");
            }
        }
    }

    #[test]
    fn the_operator_rows_are_never_refitted() {
        let mut r = Rig::new(vec![(11, 0, vec![song(1)])]);
        r.s.config.players.auto_segue = false;
        varied(&mut r);
        r.at_hms(10, 40, 0);
        r.apply(Command::StartAutomation);
        let list = r.s.filler.as_ref().unwrap().playlist;
        let len = r.s.playlists.get(list).unwrap().entries.len();
        // A 900-second song of the operator's near the end of the queue.
        let mut t = fp_model::Track::new(TrackId(950), "/m/950.wav".into());
        t.duration_secs = 900.0;
        t.kind = fp_model::TrackKind::Music;
        r.s.library.insert(t);
        r.apply(Command::InsertTracks {
            playlist: list,
            index: len - 1,
            tracks: vec![TrackId(950)],
        });
        let theirs = r.s.playlists.get(list).unwrap().entries[len - 1].id;
        run(&mut r, hms(10, 40, 0), hms(10, 50, 0), 0.0);
        assert!(
            r.s.playlists.entry(theirs).is_some(),
            "the operator's row left"
        );
    }

    #[test]
    fn nothing_fits_counts_once_per_pick() {
        // Every song is 700 s: the row before the block starts 98 s before
        // the fade, and no song can end that gap within the cut.
        let mut r = Rig::new(vec![(11, 0, vec![song(1)])]);
        r.s.config.players.auto_segue = false;
        for n in 1..=40u64 {
            r.s.library.get_mut(TrackId(n)).unwrap().duration_secs = 700.0;
        }
        r.at_hms(10, 0, 0);
        r.apply(Command::StartAutomation);
        run(&mut r, hms(10, 0, 0), hms(10, 50, 0), 0.0);
        let picks = r.s.filler.as_ref().unwrap().unfitted;
        assert!(picks >= 1, "no fallback pick");
        // Ticks with no song change pick nothing again.
        for s in 1..30 {
            r.at_hms(10, 50, s);
        }
        assert_eq!(r.s.filler.as_ref().unwrap().unfitted, picks);
    }
}

#[test]
fn far_from_a_block_a_track_of_unknown_length_may_play() {
    use fp_model::rotation::StepSource;
    use fp_model::selection::{Fit, pick_fitted};
    use fp_model::{AppState, SplitMix64, Track, TrackKind, WallTime};
    let mut s = AppState::new(Config::default(), "P");
    s.config.automation.track_separation_mins = 0;
    s.config.automation.track_separation_percent = 0;
    s.config.automation.artist_separation_mins = 0;
    let mut t = Track::new(TrackId(901), "/m/901.wav".into());
    t.kind = TrackKind::Music;
    s.library.insert(t);
    let fit = Fit {
        gap_secs: 3600.0,
        max_cut_secs: 30.0,
        min_play_secs: 45.0,
    };
    let music = StepSource::Kind(TrackKind::Music);
    let p = pick_fitted(&s, &music, WallTime(0), &[], &mut SplitMix64(1), Some(&fit)).unwrap();
    assert_eq!(p.track, TrackId(901));
    assert!(!p.unfitted);
}

/// Review of rework plan 1: a next set before the row on air, with a block
/// coming, never takes the row on air out of the music fill's list.
#[test]
fn a_next_before_the_row_on_air_keeps_it_in_the_list() {
    let mut r = Rig::new(vec![(12, 0, vec![song(1)])]);
    r.at_hms(11, 40, 0);
    r.apply(Command::StartAutomation);
    let filler = r.filler_player();
    r.end_current(filler);
    r.end_current(filler);
    r.end_current(filler);
    r.at_hms(11, 51, 0);
    let list = r.s.filler.as_ref().unwrap().playlist;
    let current = r.player(filler).current.unwrap();
    let first = r.s.playlists.get(list).unwrap().entries[0].id;
    assert_ne!(first, current);
    r.apply(Command::SetNext(filler, first));
    r.at_hms(11, 51, 1);
    let rows = &r.s.playlists.get(list).unwrap().entries;
    assert!(rows.iter().any(|e| e.id == current), "the row on air stays");
    assert_eq!(r.player(filler).current, Some(current));
}

/// Found by the simulated night: the queue the filler built while
/// automation was stopped (no block coming, nothing to fit) played as it
/// was when the Log's Play started the filler before a block, so its
/// second song was faded after half a minute.
#[test]
fn the_queue_built_while_stopped_is_fitted_when_automation_starts() {
    let mut r = Rig::new(vec![(10, 0, vec![song(1)])]);
    for n in 2..=40 {
        r.s.library.get_mut(TrackId(n)).unwrap().duration_secs =
            if n <= 20 { 200.0 } else { 310.0 };
    }
    // Stopped, the filler queued half an hour of music with no fit.
    r.at_hms(9, 50, 0);
    // 4 min 58 s before the fade: one song of 310 s ends it within the cut.
    r.at_hms(9, 55, 0);
    r.apply(Command::StartAutomation);
    let first = r.playing(r.filler_player()).unwrap();
    assert!(first.0 > 20, "{first:?} does not fit the gap");
}

/// A filler song: its row, track, when it started and how long it has
/// played (pauses do not count).
type Song = (fp_model::EntryId, TrackId, chrono::NaiveDateTime, f64);

/// Runs the rig to `until`, ending each filler song exactly when its
/// length has played.
fn play_filler_until(r: &mut Rig, until: chrono::NaiveDateTime, songs: &mut Vec<Song>) {
    let f = r.filler_player();
    while r.s.local_now < until {
        let p = r.player(f).clone();
        let now = r.s.local_now;
        let playing = p.transport == fp_model::Transport::Playing && !p.fading;
        let mut left = f64::INFINITY;
        if let (Some(track), Some(entry)) = (r.playing(f), p.current) {
            if songs.last().is_none_or(|s| s.0 != entry) {
                songs.push((entry, track, now, 0.0));
            }
            if playing {
                left = r.s.library.get(track).unwrap().duration_secs - songs.last().unwrap().3;
            }
        }
        let end = now + chrono::TimeDelta::milliseconds((left.min(1e7) * 1000.0) as i64);
        let target = end.min(until).min(now + chrono::TimeDelta::seconds(30));
        r.at(target);
        if playing {
            let song = songs.last_mut().unwrap();
            song.3 += (target - now).num_milliseconds() as f64 / 1000.0;
            if target == end {
                r.end_current(f);
            }
        }
    }
}

/// Found by the simulated night: a pause of the Log during the music
/// moved every song after it, but the loaded next (chosen for the old
/// timeline) was never fitted again: the last song before the block lost
/// a minute and a half.
#[test]
fn after_a_pause_the_last_song_before_the_block_still_fits() {
    let mut bad = Vec::new();
    for (pause_at, pause) in [0, 3, 6, 9, 11, 14, 16, 18, 19]
        .into_iter()
        .flat_map(|m| [20, 60, 100, 140, 180].map(move |k| (m, k)))
    {
        let mut r = Rig::new(vec![(10, 0, vec![song(1)])]);
        for n in 2..=40u64 {
            r.s.library.get_mut(TrackId(n)).unwrap().duration_secs = 150.0 + (n * 37 % 170) as f64;
        }
        r.at_hms(9, 30, 0);
        r.apply(Command::StartAutomation);
        let mut songs = Vec::new();
        let at = today().and_hms_opt(9, 35 + pause_at, 0).unwrap();
        play_filler_until(&mut r, at, &mut songs);
        let log = r.schedule_player();
        r.apply(Command::Pause(log));
        play_filler_until(&mut r, at + chrono::TimeDelta::seconds(pause), &mut songs);
        r.apply(Command::Play(log));
        r.slow_fades = true;
        play_filler_until(&mut r, today().and_hms_opt(9, 59, 58).unwrap(), &mut songs);
        let (_, last, _, played) = *songs.last().unwrap();
        // Measured at the block's start, 2 s after the fade starts.
        let cut = r.s.library.get(last).unwrap().duration_secs - played - 2.0;
        // The shortest song lasts 150 s: a gap under 120 s fits none, and
        // the one that cuts least plays (AU25).
        let fittable = played + 2.0 + 30.0 >= 150.0;
        if fittable && !(played >= 45.0 && cut <= 30.0) {
            bad.push((pause_at, pause, played, cut));
        }
    }
    assert!(bad.is_empty(), "{bad:?}");
}
