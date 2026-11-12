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
