#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Feedback 2 spec O25: when DSD reaches a device unchanged, and what the
//! players do while it does.

mod common;

use common::{entries, fixture, p0};
use fp_model::{
    AppState, AudioFormat, BpBadge, Command, DsdFacts, DsdFallback, DsdOutput, DsdStreamMode,
    DsdTarget, EngineAction, EngineEvent, EntryId, MarkerKind, PlayMode, PlayerId, TransitionPlan,
    Transport, apply, bp_badge, dsd_decision, dsd_holds_others, on_event, plan_for,
};

const DSD64: u32 = 2_822_400;

fn dsd(rate: u32, channels: u32) -> Option<AudioFormat> {
    Some(AudioFormat {
        sample_rate: rate / 32,
        bits: None,
        channels,
        dsd_rate: Some(rate),
    })
}

fn facts() -> DsdFacts {
    DsdFacts {
        mode: DsdOutput::Dop,
        format: dsd(DSD64, 2),
        volume: 1.0,
        device_idle: true,
    }
}

#[test]
fn d1_pcm_mode_converts() {
    let f = DsdFacts {
        mode: DsdOutput::Pcm,
        ..facts()
    };
    assert_eq!(dsd_decision(&f), Err(DsdFallback::ModeIsPcm));
    assert!(!DsdFallback::ModeIsPcm.worth_logging());
}

#[test]
fn d2_a_pcm_file_is_not_dsd() {
    for format in [
        None,
        Some(AudioFormat {
            sample_rate: 88_200,
            bits: Some(24),
            channels: 2,
            dsd_rate: None,
        }),
    ] {
        let f = DsdFacts { format, ..facts() };
        assert_eq!(dsd_decision(&f), Err(DsdFallback::NotDsd));
    }
}

#[test]
fn d3_more_than_two_channels_convert() {
    let f = DsdFacts {
        format: dsd(DSD64, 6),
        ..facts()
    };
    assert_eq!(dsd_decision(&f), Err(DsdFallback::Multichannel));
    for channels in [0, 1, 2] {
        let f = DsdFacts {
            format: dsd(DSD64, channels),
            ..facts()
        };
        assert!(dsd_decision(&f).is_ok(), "{channels}");
    }
}

#[test]
fn d4_a_volume_below_unity_converts() {
    let f = DsdFacts {
        volume: 0.99,
        ..facts()
    };
    assert_eq!(dsd_decision(&f), Err(DsdFallback::VolumeNotUnity));
    assert!(DsdFallback::VolumeNotUnity.worth_logging());
}

#[test]
fn d5_a_busy_device_converts() {
    let f = DsdFacts {
        device_idle: false,
        ..facts()
    };
    assert_eq!(dsd_decision(&f), Err(DsdFallback::DeviceBusy));
}

#[test]
fn d6_the_word_rate_is_the_dsd_rate_over_16() {
    assert_eq!(
        dsd_decision(&facts()),
        Ok(DsdTarget {
            mode: DsdStreamMode::Dop,
            dsd_rate: DSD64,
            word_rate: 176_400
        })
    );
    let f = DsdFacts {
        mode: DsdOutput::Native,
        format: dsd(DSD64 * 4, 2),
        ..facts()
    };
    assert_eq!(
        dsd_decision(&f),
        Ok(DsdTarget {
            mode: DsdStreamMode::Native,
            dsd_rate: DSD64 * 4,
            word_rate: 705_600
        })
    );
    assert!(
        DsdFallback::RateRefused(705_600)
            .to_string()
            .contains("705600")
    );
}

#[test]
fn d7_the_badge_reads_dsd_while_dsd_goes_out() {
    assert_eq!(bp_badge(false, false), BpBadge::Off);
    assert_eq!(bp_badge(true, false), BpBadge::Pcm);
    assert_eq!(bp_badge(false, true), BpBadge::Dsd);
    assert_eq!(bp_badge(true, true), BpBadge::Dsd);
}

// ---------------------------------------------------------------- players

fn three() -> (AppState, PlayerId, [EntryId; 3]) {
    let state = fixture(3);
    let (e, p) = (entries(&state), p0(&state));
    (state, p, [e[0], e[1], e[2]])
}

/// Entry `a` plays and the engine reported it going out as DSD.
fn on_air(hold_others: bool) -> (AppState, PlayerId, [EntryId; 3]) {
    let (mut s, p, e) = three();
    apply(&mut s, Command::Play(p)).unwrap();
    on_event(
        &mut s,
        EngineEvent::DsdStarted {
            player: p,
            entry: e[0],
            hold_others,
        },
    );
    (s, p, e)
}

fn plan(s: &AppState, p: PlayerId) -> Option<TransitionPlan> {
    plan_for(s, s.player(p).unwrap())
}

fn with_segue(state: &mut AppState, secs: f64) {
    for t in state.library.iter_mut() {
        t.markers.set_auto(MarkerKind::SegueStart, Some(secs));
    }
}

fn started(out: &[EngineAction], p: PlayerId) -> Option<EntryId> {
    out.iter().find_map(|a| match a {
        EngineAction::StartCurrent { player, request } if *player == p => Some(request.entry),
        _ => None,
    })
}

#[test]
fn d8_dsd_started_marks_only_the_entry_on_air() {
    let (s, p, [a, b, _]) = on_air(true);
    assert_eq!(
        s.player(p).unwrap().dsd.map(|d| (d.entry, d.hold_others)),
        Some((a, true))
    );
    let (mut s2, p2, _) = three();
    on_event(
        &mut s2,
        EngineEvent::DsdStarted {
            player: p2,
            entry: b,
            hold_others: true,
        },
    );
    assert_eq!(s2.player(p2).unwrap().dsd, None, "stopped player: stale");
    let mut s3 = s.clone();
    on_event(
        &mut s3,
        EngineEvent::DsdStarted {
            player: p,
            entry: b,
            hold_others: false,
        },
    );
    assert_eq!(
        s3.player(p).unwrap().dsd.map(|d| d.entry),
        Some(a),
        "another entry: stale"
    );
}

#[test]
fn d9_dsd_ended_and_leaving_the_entry_clear_it() {
    let (mut s, p, [a, b, _]) = on_air(false);
    on_event(
        &mut s,
        EngineEvent::DsdEnded {
            player: p,
            entry: b,
        },
    );
    assert!(s.player(p).unwrap().dsd.is_some(), "another entry: stale");
    on_event(
        &mut s,
        EngineEvent::DsdEnded {
            player: p,
            entry: a,
        },
    );
    assert!(s.player(p).unwrap().dsd.is_none());

    let (mut s, p, _) = on_air(false);
    apply(&mut s, Command::Play(p)).unwrap(); // Play while playing: the next entry
    assert!(s.player(p).unwrap().dsd.is_none(), "advancing clears it");
    let (mut s, p, _) = on_air(false);
    apply(&mut s, Command::Stop(p)).unwrap();
    assert!(s.player(p).unwrap().dsd.is_none(), "stopping clears it");
}

#[test]
fn d10_hold_others_never_overlaps_the_next_entry() {
    let (mut s, p, _) = on_air(true);
    // A segue inside the track would overlap; held, the track plays to its end.
    with_segue(&mut s, 170.0);
    assert_eq!(plan(&s, p), Some(TransitionPlan::StopAt { at_secs: 180.0 }));
}

#[test]
fn d11_hold_others_starts_the_next_entry_when_the_dsd_track_ends() {
    let (mut s, p, [a, b, _]) = on_air(true);
    let out = on_event(
        &mut s,
        EngineEvent::ReachedEnd {
            player: p,
            entry: a,
        },
    );
    assert_eq!(started(&out, p), Some(b));
    let player = s.player(p).unwrap();
    assert_eq!(player.current, Some(b));
    assert_eq!(player.transport, Transport::Playing);
    assert_eq!(player.history.last(), Some(&a));
    assert!(player.dsd.is_none());
}

#[test]
fn d12_hold_others_repeats_a_repeating_entry_without_a_new_play() {
    let (mut s, p, [a, _, _]) = on_air(true);
    apply(&mut s, Command::SetEntryRepeat(a, true)).unwrap();
    let out = on_event(
        &mut s,
        EngineEvent::ReachedEnd {
            player: p,
            entry: a,
        },
    );
    assert_eq!(started(&out, p), Some(a));
    let player = s.player(p).unwrap();
    assert_eq!(player.current, Some(a));
    assert!(player.history.is_empty(), "a repeat pass is not a new play");
}

#[test]
fn d13_hold_others_still_obeys_every_stop_rule() {
    for setup in [
        (|s: &mut AppState, p: PlayerId, _a: EntryId| {
            apply(s, Command::SetMode(p, PlayMode::Single)).unwrap();
        }) as fn(&mut AppState, PlayerId, EntryId),
        |s, p, _a| {
            apply(s, Command::ToggleStopAfterCurrent(p)).unwrap();
        },
        |s, _p, a| {
            apply(s, Command::SetEntryStopAfter(a, true)).unwrap();
        },
    ] {
        let (mut s, p, [a, _, _]) = on_air(true);
        setup(&mut s, p, a);
        let out = on_event(
            &mut s,
            EngineEvent::ReachedEnd {
                player: p,
                entry: a,
            },
        );
        assert_eq!(started(&out, p), None);
        assert_eq!(s.player(p).unwrap().transport, Transport::Stopped);
    }
}

#[test]
fn d14_hold_others_starts_play_and_next_hard() {
    let (mut s, p, [_, b, _]) = on_air(true);
    let out = apply(&mut s, Command::Play(p)).unwrap(); // Play while playing
    assert_eq!(started(&out, p), Some(b));
    assert!(
        !out.iter()
            .any(|a| matches!(a, EngineAction::Crossfade { .. }))
    );
    assert!(!s.player(p).unwrap().fading);
}

#[test]
fn d15_convert_to_pcm_keeps_the_ordinary_plan() {
    let (mut s, p, _) = on_air(false);
    with_segue(&mut s, 170.0);
    assert_eq!(
        plan(&s, p),
        Some(TransitionPlan::StartNextAt {
            at_secs: 170.0,
            fade_current_until_secs: Some(180.0)
        })
    );
}

#[test]
fn d16_a_fade_stop_of_a_dsd_track_stops_at_once() {
    let (mut s, p, _) = on_air(false);
    let out = apply(&mut s, Command::FadeStop(p)).unwrap();
    assert!(
        out.iter()
            .any(|a| matches!(a, EngineAction::StopNow { player } if *player == p))
    );
    assert!(
        !out.iter()
            .any(|a| matches!(a, EngineAction::FadeOutAndStop { .. }))
    );
    assert_eq!(s.player(p).unwrap().transport, Transport::Stopped);
}

#[test]
fn d17_a_fader_move_below_unity_leaves_dsd() {
    let (mut s, p, _) = on_air(false);
    let out = apply(&mut s, Command::SetVolume(p, 1.0)).unwrap();
    assert!(
        !out.iter()
            .any(|a| matches!(a, EngineAction::LeaveDsd { .. }))
    );
    let out = apply(&mut s, Command::SetVolume(p, 0.5)).unwrap();
    let leave = out
        .iter()
        .position(|a| matches!(a, EngineAction::LeaveDsd { player } if *player == p));
    let volume = out
        .iter()
        .position(|a| matches!(a, EngineAction::SetVolume { .. }));
    assert!(leave.is_some() && leave < volume, "{out:?}");
    assert!(s.player(p).unwrap().dsd.is_none());
}

#[test]
fn d18_the_notice_shows_while_a_held_dsd_track_is_on_air() {
    let (s, p, _) = on_air(true);
    assert!(dsd_holds_others(&s, p));
    let (s, p, _) = on_air(false);
    assert!(!dsd_holds_others(&s, p));
    let (mut s, p, _) = on_air(true);
    apply(&mut s, Command::Stop(p)).unwrap();
    assert!(!dsd_holds_others(&s, p));
}
