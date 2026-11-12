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
