#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Automation plan 5: the filler's last songs before a block fit (AU25).

use fp_model::Config;

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
