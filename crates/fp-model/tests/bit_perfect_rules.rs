#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Bit-perfect rate and fader spec (2026-10-08): one test per rule BR1–BR14
//! and BF1–BF8 that the model decides. The engine is stood in for by the
//! events it reports.

mod common;

use common::{bp_device, entries, fixture, p0, place_on_dac, report_stream_rate, set_track_rates};
use fp_model::{
    AppState, Command, CueState, DeviceSettings, DsdOnAir, EngineAction, EngineEvent, EntryId,
    Holder, MarkerKind, PlayMode, PlayerId, SourceRequest, Target, TrackAnalysis, TransitionPlan,
    Wanted, apply, bit_perfect_player, fader_locked, on_event, plan_for, rate_cut,
};

#[test]
fn br1_a_stream_rate_report_is_kept_per_device() {
    let mut s = fixture(3);
    let p = p0(&s);
    place_on_dac(&mut s, Holder::PlayerMain(p));
    assert_eq!(
        s.live.stream_rates.get(&bp_device()),
        None,
        "unknown until reported"
    );
    report_stream_rate(&mut s, 96_000, vec![]);
    assert_eq!(s.live.stream_rates.get(&bp_device()), Some(&96_000));
    assert_eq!(s.live.refused_rates.get(&bp_device()), Some(&vec![]));
    report_stream_rate(&mut s, 44_100, vec![192_000]);
    assert_eq!(s.live.stream_rates.get(&bp_device()), Some(&44_100));
    assert_eq!(s.live.refused_rates.get(&bp_device()), Some(&vec![192_000]));
}

#[test]
fn br1_the_rates_are_forgotten_with_the_last_holder() {
    let mut s = fixture(3);
    let (p, q) = (s.players[0].id, s.players[1].id);
    place_on_dac(&mut s, Holder::PlayerMain(p));
    place_on_dac(&mut s, Holder::PlayerMain(q));
    report_stream_rate(&mut s, 96_000, vec![44_100]);
    fp_model::on_event(
        &mut s,
        EngineEvent::Gone {
            holder: Holder::PlayerMain(p),
        },
    );
    assert_eq!(
        s.live.stream_rates.get(&bp_device()),
        Some(&96_000),
        "q still plays there"
    );
    fp_model::on_event(
        &mut s,
        EngineEvent::Unplaced {
            holder: Holder::PlayerMain(q),
            route: None,
        },
    );
    assert_eq!(s.live.stream_rates.get(&bp_device()), None);
    assert_eq!(s.live.refused_rates.get(&bp_device()), None);
}

/// `fixture(3)`: P1's Main on `dac`, which runs bit-perfect at 44.1 kHz;
/// every track 44.1 kHz 16-bit; P1 plays its first entry.
fn on_air() -> (AppState, PlayerId, [EntryId; 3]) {
    let mut s = fixture(3);
    let p = p0(&s);
    let e = entries(&s);
    place_on_dac(&mut s, Holder::PlayerMain(p));
    report_stream_rate(&mut s, 44_100, vec![]);
    set_track_rates(&mut s, &[44_100, 44_100, 44_100]);
    apply(&mut s, Command::Play(p)).unwrap();
    (s, p, [e[0], e[1], e[2]])
}

/// The request P1 preloads for `entry`, at its cue-in.
fn next_request(s: &AppState, entry: EntryId) -> SourceRequest {
    s.request_from_cue_in(entry).unwrap()
}

fn rates(s: &mut AppState, next: u32) {
    set_track_rates(s, &[44_100, next, 44_100]);
}

#[test]
fn br2_a_main_placed_on_a_device_running_bit_perfect() {
    let (s, p, _) = on_air();
    assert!(bit_perfect_player(&s, p));
    assert!(!bit_perfect_player(&s, s.players[1].id), "not placed yet");
}

#[test]
fn br2_the_running_value_counts_not_the_wanted_one() {
    let mut s = fixture(3);
    let p = p0(&s);
    place_on_dac(&mut s, Holder::PlayerMain(p));
    // Wanted off, still running on: pending until applied.
    s.config.outputs.bit_perfect.clear();
    assert!(bit_perfect_player(&s, p));
    let mut s = fixture(3);
    let p = p0(&s);
    place_on_dac(&mut s, Holder::PlayerMain(p));
    let running = s.live.devices.get_mut(&bp_device()).unwrap();
    running.bit_perfect = false;
    // Wanted on (the configuration lists it), not running yet.
    assert!(!bit_perfect_player(&s, p));
}

#[test]
fn br2_a_cue_route_on_the_device_does_not_count() {
    let mut s = fixture(3);
    let p = p0(&s);
    place_on_dac(&mut s, Holder::PlayerCue(p));
    assert!(!bit_perfect_player(&s, p));
}

#[test]
fn bf1_the_fader_is_locked_exactly_for_a_bit_perfect_player() {
    let (s, p, _) = on_air();
    assert!(fader_locked(&s, p));
    let q = s.players[1].id;
    assert_eq!(fader_locked(&s, q), bit_perfect_player(&s, q));
    assert!(!fader_locked(&s, q), "unplaced is not locked");
}

#[test]
fn br3_all_conditions_hold_gives_the_next_rate() {
    let (mut s, p, [_, b, _]) = on_air();
    rates(&mut s, 96_000);
    assert_eq!(rate_cut(&s, p, &next_request(&s, b)), Some(96_000));
}

#[test]
fn br3_1_not_a_bit_perfect_player() {
    let (mut s, p, [_, b, _]) = on_air();
    rates(&mut s, 96_000);
    s.live.devices.get_mut(&bp_device()).unwrap().bit_perfect = false;
    assert_eq!(rate_cut(&s, p, &next_request(&s, b)), None);
}

#[test]
fn br3_2_the_stream_rate_is_unknown() {
    let (mut s, p, [_, b, _]) = on_air();
    rates(&mut s, 96_000);
    s.live.stream_rates.clear();
    assert_eq!(rate_cut(&s, p, &next_request(&s, b)), None);
}

#[test]
fn br3_3_the_next_format_is_unknown_dsd_zero_or_the_same_rate() {
    let (mut s, p, [_, b, _]) = on_air();
    let track = s.playlists.entry(b).unwrap().track;
    for format in [
        None,
        Some(fp_model::AudioFormat {
            sample_rate: 88_200,
            bits: None,
            channels: 2,
            dsd_rate: Some(2_822_400),
        }),
        Some(fp_model::AudioFormat {
            sample_rate: 0,
            bits: Some(16),
            channels: 2,
            dsd_rate: None,
        }),
        Some(fp_model::AudioFormat {
            sample_rate: 44_100,
            bits: Some(16),
            channels: 2,
            dsd_rate: None,
        }),
    ] {
        s.library.get_mut(track).unwrap().format = format;
        assert_eq!(rate_cut(&s, p, &next_request(&s, b)), None, "{format:?}");
    }
}

#[test]
fn br3_4_a_rate_the_device_refused() {
    let (mut s, p, [_, b, _]) = on_air();
    rates(&mut s, 96_000);
    report_stream_rate(&mut s, 44_100, vec![96_000]);
    assert_eq!(rate_cut(&s, p, &next_request(&s, b)), None);
}

#[test]
fn br3_5_something_else_sounds_on_the_device() {
    // Another player playing there.
    let (mut s, p, [_, b, _]) = on_air();
    rates(&mut s, 96_000);
    let q = s.players[1].id;
    place_on_dac(&mut s, Holder::PlayerMain(q));
    assert_eq!(
        rate_cut(&s, p, &next_request(&s, b)),
        Some(96_000),
        "q is stopped"
    );
    apply(&mut s, Command::Play(q)).unwrap();
    assert_eq!(rate_cut(&s, p, &next_request(&s, b)), None);
    // This player's own CUE placed on the device.
    let (mut s, p, [_, b, c]) = on_air();
    rates(&mut s, 96_000);
    s.live.placement.insert(Holder::PlayerCue(p), bp_device());
    s.players[0].cue = Some(CueState {
        entry: c,
        paused: false,
    });
    assert_eq!(rate_cut(&s, p, &next_request(&s, b)), None);
}

#[test]
fn br3_5_a_paused_player_on_the_device_counts_as_sounding() {
    // The engine keeps a paused source's timeline on its bus and never
    // reopens under it, so the model must not plan a cut either.
    let (mut s, p, [_, b, _]) = on_air();
    rates(&mut s, 96_000);
    let q = s.players[1].id;
    place_on_dac(&mut s, Holder::PlayerMain(q));
    apply(&mut s, Command::Play(q)).unwrap();
    apply(&mut s, Command::Pause(q)).unwrap();
    assert_eq!(s.player(q).unwrap().transport, fp_model::Transport::Paused);
    assert_eq!(rate_cut(&s, p, &next_request(&s, b)), None);
}

#[test]
fn br3_6_a_dsd_stream_on_air() {
    let (mut s, p, [a, b, _]) = on_air();
    rates(&mut s, 96_000);
    s.players[0].dsd = Some(DsdOnAir {
        entry: a,
        hold_others: false,
    });
    assert_eq!(rate_cut(&s, p, &next_request(&s, b)), None);
}

fn plan(s: &AppState, p: PlayerId) -> Option<TransitionPlan> {
    plan_for(s, s.player(p).unwrap())
}

/// A MIX point at 170 s on every track (they are 180 s long).
fn with_segue(s: &mut AppState) {
    s.config.players.auto_segue = true;
    for t in s.library.iter_mut() {
        t.markers.set_auto(MarkerKind::SegueStart, Some(170.0));
    }
}

fn hard(rate_cut: Option<u32>) -> Option<TransitionPlan> {
    Some(TransitionPlan::StartNextAt {
        at_secs: 180.0,
        fade_current_until_secs: None,
        rate_cut,
    })
}

const SEGUE: Option<TransitionPlan> = Some(TransitionPlan::StartNextAt {
    at_secs: 170.0,
    fade_current_until_secs: Some(180.0),
    rate_cut: None,
});

/// The last `Schedule` sent for `p` in `out`.
fn scheduled(out: &[EngineAction], p: PlayerId) -> Option<Option<TransitionPlan>> {
    out.iter().rev().find_map(|a| match a {
        EngineAction::Schedule { player, plan } if *player == p => Some(*plan),
        _ => None,
    })
}

#[test]
fn br4_a_bit_perfect_player_plays_to_its_end_and_never_segues() {
    let (mut s, p, _) = on_air();
    with_segue(&mut s);
    assert_eq!(plan(&s, p), hard(None), "same rate: no reopen");
    rates(&mut s, 96_000);
    assert_eq!(plan(&s, p), hard(Some(96_000)));
}

#[test]
fn br4_stop_plans_are_unchanged() {
    let (mut s, p, _) = on_air();
    with_segue(&mut s);
    apply(&mut s, Command::SetMode(p, PlayMode::Single)).unwrap();
    assert_eq!(plan(&s, p), Some(TransitionPlan::StopAt { at_secs: 180.0 }));
}

#[test]
fn br4_off_bit_perfect_the_plan_is_the_segue() {
    let (mut s, p, _) = on_air();
    with_segue(&mut s);
    s.live.devices.get_mut(&bp_device()).unwrap().bit_perfect = false;
    assert_eq!(plan(&s, p), SEGUE);
}

#[test]
fn br5_a_cart_on_the_device_drops_the_reopen_and_its_end_brings_it_back() {
    let (mut s, p, _) = on_air();
    with_segue(&mut s);
    rates(&mut s, 96_000);
    // Any command reconciles: the plan now carries the cut.
    apply(&mut s, Command::SetMode(p, PlayMode::Continuous)).unwrap();
    assert_eq!(plan(&s, p), hard(Some(96_000)));
    place_on_dac(&mut s, Holder::CartwallMain);
    let cart = load_cart(&mut s);
    let out = apply(&mut s, Command::FireCart(cart)).unwrap();
    assert_eq!(scheduled(&out, p), Some(hard(None)), "{out:?}");
    let out = on_event(&mut s, EngineEvent::CartEnded { cart });
    assert_eq!(scheduled(&out, p), Some(hard(Some(96_000))), "{out:?}");
}

#[test]
fn br5_an_analysed_next_brings_its_rate_cut() {
    let (mut s, p, [_, b, _]) = on_air();
    let track = s.playlists.entry(b).unwrap().track;
    s.library.get_mut(track).unwrap().format = None;
    // Any command reconciles; the plan is hard with no reopen.
    apply(&mut s, Command::SetMode(p, PlayMode::Continuous)).unwrap();
    assert_eq!(plan(&s, p), hard(None));
    let analysis = TrackAnalysis {
        duration_secs: 180.0,
        format: Some(fp_model::AudioFormat {
            sample_rate: 96_000,
            bits: Some(24),
            channels: 2,
            dsd_rate: None,
        }),
        ..TrackAnalysis::default()
    };
    let out = apply(
        &mut s,
        Command::ApplyAnalysis {
            track,
            analysis: Box::new(analysis),
        },
    )
    .unwrap();
    assert_eq!(scheduled(&out, p), Some(hard(Some(96_000))), "{out:?}");
}

#[test]
fn br5_a_refused_rate_drops_the_cut() {
    let (mut s, p, _) = on_air();
    rates(&mut s, 96_000);
    apply(&mut s, Command::SetMode(p, PlayMode::Continuous)).unwrap();
    assert_eq!(plan(&s, p), hard(Some(96_000)));
    let out = report_stream_rate(&mut s, 44_100, vec![96_000]);
    assert_eq!(scheduled(&out, p), Some(hard(None)));
}

#[test]
fn br5_moved_off_the_device_gives_back_the_segue() {
    let (mut s, p, _) = on_air();
    with_segue(&mut s);
    apply(&mut s, Command::SetMode(p, PlayMode::Continuous)).unwrap();
    let speakers = fp_model::OutputDevice {
        backend: "null".into(),
        device: "speakers".into(),
    };
    let running = fp_model::device_settings(&s.config.outputs, &speakers);
    let out = on_event(
        &mut s,
        EngineEvent::Placed {
            holder: Holder::PlayerMain(p),
            route: None,
            device: speakers,
            running,
        },
    );
    assert_eq!(scheduled(&out, p), Some(SEGUE));
}

#[test]
fn br10_an_unanalysed_next_is_hard_without_a_reopen() {
    let (mut s, p, [_, b, _]) = on_air();
    let track = s.playlists.entry(b).unwrap().track;
    s.library.get_mut(track).unwrap().format = None;
    assert_eq!(plan(&s, p), hard(None));
}

#[test]
fn br10_dsd_next_or_dsd_on_air_takes_no_cut_and_the_hold_still_stops() {
    let (mut s, p, [a, b, _]) = on_air();
    let track = s.playlists.entry(b).unwrap().track;
    s.library.get_mut(track).unwrap().format = Some(fp_model::AudioFormat {
        sample_rate: 88_200,
        bits: None,
        channels: 2,
        dsd_rate: Some(2_822_400),
    });
    assert_eq!(plan(&s, p), hard(None), "the PCM of a DSD next");
    rates(&mut s, 96_000);
    s.players[0].dsd = Some(DsdOnAir {
        entry: a,
        hold_others: false,
    });
    assert_eq!(plan(&s, p), hard(None), "a DSD stream on air");
    s.players[0].dsd = Some(DsdOnAir {
        entry: a,
        hold_others: true,
    });
    assert_eq!(plan(&s, p), Some(TransitionPlan::StopAt { at_secs: 180.0 }));
}

#[test]
fn br10_another_player_on_an_ordinary_device_keeps_its_segue() {
    let (mut s, _, _) = on_air();
    with_segue(&mut s);
    let q = s.players[1].id;
    apply(&mut s, Command::Play(q)).unwrap();
    assert_eq!(plan(&s, q), SEGUE);
}

#[test]
fn br14_bit_perfect_turned_off_gives_back_the_ordinary_plan() {
    let (mut s, p, _) = on_air();
    with_segue(&mut s);
    apply(&mut s, Command::SetMode(p, PlayMode::Continuous)).unwrap();
    let off = DeviceSettings {
        bit_perfect: false,
        ..*s.live.devices.get(&bp_device()).unwrap()
    };
    let out = on_event(
        &mut s,
        EngineEvent::Applied {
            target: Target::Device(bp_device()),
            wanted: Wanted::Device(off),
            outcome: Ok(()),
        },
    );
    assert_eq!(scheduled(&out, p), Some(SEGUE));
}

/// Gives cart 0 of the first page a 10 s file.
fn load_cart(state: &mut AppState) -> fp_model::CartId {
    let page = state.cartwall.pages[0].id;
    let path = std::path::PathBuf::from("/carts/cart0.wav");
    apply(
        state,
        Command::AssignCartFile {
            page,
            index: 0,
            path,
        },
    )
    .unwrap();
    let track = state.cartwall.pages[0].carts[0].track.unwrap();
    let analysis = TrackAnalysis {
        duration_secs: 10.0,
        ..TrackAnalysis::default()
    };
    apply(
        state,
        Command::ApplyAnalysis {
            track,
            analysis: Box::new(analysis),
        },
    )
    .unwrap();
    state.cartwall.pages[0].carts[0].id
}

/// What a manual start sent: `Ok(entry)` for `StartCurrent`, `Err(entry)` for `Crossfade`.
fn start_of(out: &[EngineAction], p: PlayerId) -> Option<Result<EntryId, EntryId>> {
    out.iter().find_map(|a| match a {
        EngineAction::StartCurrent { player, request } if *player == p => Some(Ok(request.entry)),
        EngineAction::Crossfade {
            player, request, ..
        } if *player == p => Some(Err(request.entry)),
        _ => None,
    })
}

#[test]
fn br7_play_while_playing_and_previous_start_hard_at_any_rate() {
    for next_rate in [44_100, 96_000] {
        let (mut s, p, [a, b, _]) = on_air();
        rates(&mut s, next_rate);
        let out = apply(&mut s, Command::Play(p)).unwrap();
        assert_eq!(start_of(&out, p), Some(Ok(b)), "{next_rate}");
        assert!(!s.player(p).unwrap().fading, "no Mixing badge");
        let out = apply(&mut s, Command::Previous(p)).unwrap();
        assert_eq!(start_of(&out, p), Some(Ok(a)), "{next_rate}");
        assert!(!s.player(p).unwrap().fading);
    }
}

#[test]
fn br7_play_on_the_entry_on_air_set_as_next_starts_hard() {
    let (mut s, p, [a, _, _]) = on_air();
    apply(&mut s, Command::SetNext(p, a)).unwrap();
    let out = apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(start_of(&out, p), Some(Ok(a)));
}

#[test]
fn br7_another_player_still_crossfades() {
    let (mut s, _, _) = on_air();
    let q = s.players[1].id;
    apply(&mut s, Command::Play(q)).unwrap();
    let out = apply(&mut s, Command::Play(q)).unwrap();
    assert!(matches!(start_of(&out, q), Some(Err(_))), "{out:?}");
    assert!(s.player(q).unwrap().fading);
}

#[test]
fn br9_fade_stop_pause_and_restart_are_unchanged() {
    let (mut s, p, _) = on_air();
    let fade_ms = s.config.players.fade_ms;
    let out = apply(&mut s, Command::Pause(p)).unwrap();
    assert!(out.contains(&EngineAction::Pause { player: p }));
    apply(&mut s, Command::Play(p)).unwrap(); // resume
    let out = apply(&mut s, Command::Restart(p)).unwrap();
    assert!(
        out.iter()
            .any(|a| matches!(a, EngineAction::Seek { player, .. } if *player == p))
    );
    let out = apply(&mut s, Command::FadeStop(p)).unwrap();
    assert!(out.contains(&EngineAction::FadeOutAndStop { player: p, fade_ms }));
}

fn set_volumes(out: &[EngineAction], p: PlayerId) -> Vec<f32> {
    out.iter()
        .filter_map(|a| match a {
            EngineAction::SetVolume { player, volume } if *player == p => Some(*volume),
            _ => None,
        })
        .collect()
}

/// `fixture(3)` with P1 at half volume, not placed yet.
fn half() -> (AppState, PlayerId) {
    let mut s = fixture(3);
    let p = p0(&s);
    apply(&mut s, Command::SetVolume(p, 0.5)).unwrap();
    (s, p)
}

#[test]
fn bf2_placed_on_a_bit_perfect_device_the_fader_goes_to_full() {
    let (mut s, p) = half();
    let out = place_on_dac(&mut s, Holder::PlayerMain(p));
    assert_eq!(s.player(p).unwrap().volume, 1.0);
    assert_eq!(set_volumes(&out, p), vec![1.0]);
}

#[test]
fn bf2_bit_perfect_turned_on_for_its_device_resets_the_fader() {
    let (mut s, p) = half();
    // P1 on dac, which runs shared: not locked yet.
    let dac = bp_device();
    let shared = fp_model::device_settings(&s.config.outputs, &dac);
    on_event(
        &mut s,
        EngineEvent::Placed {
            holder: Holder::PlayerMain(p),
            route: None,
            device: dac.clone(),
            running: shared,
        },
    );
    assert_eq!(s.player(p).unwrap().volume, 0.5);
    let on = DeviceSettings {
        bit_perfect: true,
        ..shared
    };
    let out = on_event(
        &mut s,
        EngineEvent::Applied {
            target: Target::Device(dac),
            wanted: Wanted::Device(on),
            outcome: Ok(()),
        },
    );
    assert_eq!(s.player(p).unwrap().volume, 1.0);
    assert_eq!(set_volumes(&out, p), vec![1.0]);
}

#[test]
fn bf2_a_restored_lower_volume_goes_to_full_when_placed() {
    let (s, p) = half();
    let mut s = common::roundtrip(&s);
    assert_eq!(s.player(p).unwrap().volume, 0.5, "restored as saved");
    let out = place_on_dac(&mut s, Holder::PlayerMain(p));
    assert_eq!(set_volumes(&out, p), vec![1.0]);
}

#[test]
fn bf2_the_reset_is_sent_once() {
    let (mut s, p) = half();
    place_on_dac(&mut s, Holder::PlayerMain(p));
    let out = report_stream_rate(&mut s, 44_100, vec![]);
    assert!(set_volumes(&out, p).is_empty(), "already at 100 %");
}

#[test]
fn bf3_a_locked_fader_refuses_any_level_but_full() {
    let (mut s, p, _) = on_air();
    let before = s.clone();
    assert_eq!(
        apply(&mut s, Command::SetVolume(p, 0.5)),
        Err(fp_model::ModelError::FaderLocked(p))
    );
    assert_eq!(s, before, "a refused command changes nothing");
    let out = apply(&mut s, Command::SetVolume(p, 1.0)).unwrap();
    assert!(
        set_volumes(&out, p).is_empty(),
        "accepted, nothing to change"
    );
    assert!(
        apply(&mut s, Command::SetVolume(p, f32::NAN)).is_ok(),
        "ignored as today"
    );
    assert_eq!(
        fp_model::ModelError::FaderLocked(p).to_string(),
        format!(
            "the volume of player {} is held at 100 % on a bit-perfect output",
            p.0
        )
    );
}

#[test]
fn bf3_availability_says_the_fader_is_locked() {
    let (s, p, _) = on_air();
    assert!(!fp_model::availability(&s, p).fader);
    assert!(!fp_model::command_available(
        &s,
        &Command::SetVolume(p, 0.5)
    ));
    assert!(fp_model::command_available(&s, &Command::SetVolume(p, 1.0)));
    let q = s.players[1].id;
    assert!(fp_model::availability(&s, q).fader);
}

#[test]
fn bf4_unlocking_keeps_full_volume_and_frees_the_fader() {
    let (mut s, p, _) = on_air();
    let off = DeviceSettings {
        bit_perfect: false,
        ..*s.live.devices.get(&bp_device()).unwrap()
    };
    let out = on_event(
        &mut s,
        EngineEvent::Applied {
            target: Target::Device(bp_device()),
            wanted: Wanted::Device(off),
            outcome: Ok(()),
        },
    );
    assert!(set_volumes(&out, p).is_empty(), "nothing moves it");
    assert_eq!(s.player(p).unwrap().volume, 1.0);
    let out = apply(&mut s, Command::SetVolume(p, 0.5)).unwrap();
    assert_eq!(set_volumes(&out, p), vec![0.5]);
}

#[test]
fn bf5_fade_stop_does_not_touch_the_volume() {
    let (mut s, p, _) = on_air();
    let out = apply(&mut s, Command::FadeStop(p)).unwrap();
    assert!(
        out.iter()
            .any(|a| matches!(a, EngineAction::FadeOutAndStop { .. }))
    );
    assert!(set_volumes(&out, p).is_empty());
    assert_eq!(s.player(p).unwrap().volume, 1.0);
}

#[test]
fn bf6_a_cue_route_on_the_device_locks_nothing_and_cue_plays() {
    let (mut s, p) = half();
    let out = place_on_dac(&mut s, Holder::PlayerCue(p));
    assert!(set_volumes(&out, p).is_empty());
    assert!(!fader_locked(&s, p));
    let e = entries(&s);
    let out = apply(&mut s, Command::CueEntry(p, e[1])).unwrap();
    assert!(
        out.iter()
            .any(|a| matches!(a, EngineAction::StartCue { .. })),
        "{out:?}"
    );
}

#[test]
fn bf8_apply_now_lists_the_faders_it_would_reset() {
    let mut s = fixture(3);
    let (p, q, r) = (s.players[0].id, s.players[1].id, s.players[2].id);
    // P1 at 50 % on dac, which runs shared and is wanted bit-perfect.
    let dac = bp_device();
    let shared = fp_model::device_settings(&s.config.outputs, &dac);
    for holder in [Holder::PlayerMain(p), Holder::PlayerMain(r)] {
        on_event(
            &mut s,
            EngineEvent::Placed {
                holder,
                route: Some(common::bp_route()),
                device: dac.clone(),
                running: shared,
            },
        );
    }
    for player in [p, r] {
        s.config
            .outputs
            .routes
            .iter_mut()
            .find(|x| x.player == player)
            .unwrap()
            .main = Some(common::bp_route());
    }
    apply(&mut s, Command::SetVolume(p, 0.5)).unwrap();
    s.config.outputs.bit_perfect = vec![dac.clone()];
    // Q at 70 % elsewhere, with a pending route to dac.
    apply(&mut s, Command::SetVolume(q, 0.7)).unwrap();
    s.config
        .outputs
        .routes
        .iter_mut()
        .find(|x| x.player == q)
        .unwrap()
        .main = Some(common::bp_route());
    let speakers = fp_model::OutputDevice {
        backend: "null".into(),
        device: "speakers".into(),
    };
    let running = fp_model::device_settings(&s.config.outputs, &speakers);
    on_event(
        &mut s,
        EngineEvent::Placed {
            holder: Holder::PlayerMain(q),
            route: None,
            device: speakers,
            running,
        },
    );
    // R is on dac at 100 %: nothing to reset.
    assert_eq!(fp_model::faders_reset_by(&s), vec![p, q]);
}

#[test]
fn bf8_a_locked_player_is_not_listed() {
    let (s, _, _) = on_air();
    assert!(fp_model::faders_reset_by(&s).is_empty());
}
