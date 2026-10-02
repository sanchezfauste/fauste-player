#![allow(clippy::unwrap_used)]
mod support;

use fp_model::{Command, MarkerKind, PlayerId};
use fp_remote::control::Playback;
use fp_remote::dto;
use support::demo_state;

#[test]
fn the_state_lists_players_in_display_order_with_one_based_positions() {
    let s = demo_state();
    let out = dto::state(
        &s,
        &Playback {
            revision: 7,
            ..Default::default()
        },
    );
    assert_eq!(out.revision, 7);
    assert_eq!(out.players.len(), 4);
    assert_eq!(out.players[0].position, 1);
    assert_eq!(out.players[3].position, 4);
    assert_eq!(out.players[0].transport, "stopped");
    assert_eq!(out.players[0].mode, "continuous");
    assert_eq!(out.playlists.len(), 2);
    assert_eq!(out.playlists[0].entry_count, 3);
}

#[test]
fn a_player_reports_next_fader_and_times_like_the_ui() {
    let mut s = demo_state();
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    s.players[0].volume = 1.0;
    let playback = Playback {
        players: vec![(p, 30.0)],
        ..Default::default()
    };
    let out = dto::player(&s, &playback, p).unwrap();
    assert_eq!(out.transport, "playing");
    assert_eq!(out.fader, 1.0);
    let current = out.current.unwrap();
    assert_eq!(current.track.title, format!("Title {}", current.track.id.0));
    assert_eq!(out.elapsed_secs, Some(30.0));
    assert_eq!(out.remaining_secs, Some(150.0));
    assert!(out.next.is_some());
    assert!(dto::player(&s, &playback, PlayerId(999_999)).is_none());
}

#[test]
fn a_stopped_player_without_current_has_no_times() {
    let s = demo_state();
    let out = dto::player(&s, &Playback::default(), s.players[1].id).unwrap();
    assert!(out.current.is_none());
    assert_eq!(out.elapsed_secs, None);
    assert_eq!(out.remaining_secs, None);
}

#[test]
fn playlist_entries_say_who_plays_them() {
    let mut s = demo_state();
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    let list = s.playlists.first_id().unwrap();
    let out = dto::playlist(&s, list).unwrap();
    assert_eq!(out.entries.len(), 3);
    assert_eq!(out.entries[0].on_air, vec![p]);
    assert!(out.entries[1].next_on.contains(&p));
    assert!(!out.entries[0].repeat);
}

#[test]
fn a_track_carries_markers_and_hides_its_path() {
    let mut s = demo_state();
    let id = s.playlists.iter().next().unwrap().entries[0].track;
    fp_model::apply(
        &mut s,
        Command::SetMarker {
            track: id,
            kind: MarkerKind::IntroEnd,
            secs: Some(12.0),
        },
    )
    .unwrap();
    let out = dto::track(&s, id).unwrap();
    assert_eq!(out.kind, "music");
    assert_eq!(out.file_state, "ok");
    let intro = out.markers.intro_end.clone().unwrap();
    assert_eq!(intro.secs, 12.0);
    assert_eq!(intro.source, "manual");
    let json = serde_json::to_string(&out).unwrap();
    assert!(!json.contains("/m/"), "{json}");
}

#[test]
fn the_cartwall_shows_its_page_and_carts() {
    let s = demo_state();
    let out = dto::cartwall(&s, &Playback::default());
    let page = &out.pages[0];
    assert_eq!(out.shown_page, Some(page.id));
    assert_eq!(page.carts[0].index, 0);
    assert!(page.carts[0].track.is_some());
    assert!(page.carts[1].track.is_none());
    assert_eq!(page.carts[0].kind, "jingle");
    assert!(out.playing.is_empty());
    assert_eq!(out.cue, None);
}

#[test]
fn a_playing_cart_reports_its_times() {
    let mut s = demo_state();
    let cart = s.cartwall.pages[0].carts[0].id;
    fp_model::apply(&mut s, Command::FireCart(cart)).unwrap();
    let playback = Playback {
        carts: vec![(cart, 10.0)],
        ..Default::default()
    };
    let out = dto::cartwall(&s, &playback);
    assert_eq!(out.playing[0].cart, cart);
    assert_eq!(out.playing[0].elapsed_secs, Some(10.0));
    assert_eq!(out.playing[0].remaining_secs, Some(170.0));
}

#[test]
fn cart_times_are_clamped_and_unknown_without_a_duration() {
    let mut s = demo_state();
    let cart = s.cartwall.pages[0].carts[0].id;
    fp_model::apply(&mut s, Command::FireCart(cart)).unwrap();
    let late = Playback {
        carts: vec![(cart, 500.0)],
        ..Default::default()
    };
    assert_eq!(
        dto::cartwall(&s, &late).playing[0].remaining_secs,
        Some(0.0)
    );
    let t = s.cartwall.pages[0].carts[0].track.unwrap();
    s.library.get_mut(t).unwrap().duration_secs = 0.0;
    let out = dto::cartwall(
        &s,
        &Playback {
            carts: vec![(cart, 3.0)],
            ..Default::default()
        },
    );
    assert_eq!(out.playing[0].remaining_secs, None);
}

/// A long waveform is reduced to at most `max` buckets: whole groups of
/// buckets, each with the lowest minimum, the highest maximum and the RMS
/// of their RMS.
#[test]
fn a_waveform_reduces_to_at_most_n_buckets() {
    let w = fp_remote::control::WaveformData {
        bucket_secs: 0.01,
        peaks: vec![[-1, 2, 3], [-5, 1, 4], [0, 9, 0], [-2, 2, 2], [-3, 3, 3]],
    };
    let r = w.reduced(2);
    assert_eq!(r.peaks.len(), 2);
    assert!((r.bucket_secs - 0.03).abs() < 1e-12, "{}", r.bucket_secs);
    // sqrt((9 + 16 + 0) / 3) = 2.886… → 3
    assert_eq!(r.peaks[0], [-5, 9, 3]);
    assert_eq!(r.peaks[1], [-3, 3, 3]);
    assert_eq!(w.reduced(10), w, "short enough already");
    assert_eq!(w.reduced(0), w, "0 means no reduction");
}

fn cut_the_current_track_at(s: &mut fp_model::AppState, p: PlayerId, secs: f64) {
    let entry = s.player(p).unwrap().current.unwrap();
    let track = s.playlists.entry(entry).unwrap().track;
    fp_model::apply(
        s,
        Command::SetMarker {
            track,
            kind: MarkerKind::CueOut,
            secs: Some(secs),
        },
    )
    .unwrap();
}

#[test]
fn a_player_reports_its_remaining_time_to_the_play_range() {
    let mut s = demo_state();
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    cut_the_current_track_at(&mut s, p, 100.0);
    let playback = Playback {
        players: vec![(p, 30.0)],
        ..Default::default()
    };
    assert_eq!(
        dto::player(&s, &playback, p).unwrap().remaining_secs,
        Some(70.0)
    );
    s.config.players.use_cue_markers = false;
    assert_eq!(
        dto::player(&s, &playback, p).unwrap().remaining_secs,
        Some(150.0)
    );
}
