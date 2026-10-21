#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Loudness normalization spec §6: the gain rule (N7), the device rule
//! (N8), requests (N9), edits (N10), fixed while sounding (N11), loaded
//! afterwards (N12) and crash recovery (N13).

mod common;

use std::path::PathBuf;

use common::{entries, fixture, p0, place_on_dac};
use fp_model::{
    AppState, Command, DeviceSettings, EngineAction, EngineEvent, GainLimit, Holder,
    LoudnessConfig, LoudnessEdit, LoudnessMeasurement, ModelError, Normalization,
    NormalizationStatus as S, OutputDevice, Route, Track, TrackId, apply, bit_perfect_player,
    device_settings, holder_bit_perfect, normalization, normalization_for, on_event,
    wanted_gain_db,
};

fn on() -> LoudnessConfig {
    LoudnessConfig {
        normalize: true,
        ..LoudnessConfig::default()
    }
}

fn track(lufs: Option<f32>, tp: f32) -> Track {
    let mut t = Track::new(TrackId(1), PathBuf::from("/a.flac"));
    t.loudness = Some(LoudnessMeasurement {
        integrated_lufs: lufs,
        true_peak_dbtp: tp,
    });
    t
}

fn gain(n: Normalization) -> (f32, S) {
    (n.gain_db, n.status)
}

#[test]
fn n7_bit_perfect_wins() {
    let t = track(Some(-20.0), -10.0);
    assert_eq!(gain(normalization(&on(), &t, true)), (0.0, S::BitPerfect));
}

#[test]
fn n7_off_gives_no_gain() {
    let t = track(Some(-20.0), -10.0);
    assert_eq!(
        gain(normalization(
            &LoudnessConfig {
                normalize: false,
                ..LoudnessConfig::default()
            },
            &t,
            false
        )),
        (0.0, S::Off)
    );
}

#[test]
fn n7_an_excluded_track_gets_no_gain() {
    let mut t = track(Some(-20.0), -10.0);
    t.loudness_edit.excluded = true;
    assert_eq!(gain(normalization(&on(), &t, false)), (0.0, S::Excluded));
}

#[test]
fn n7_a_track_not_measured_plays_at_0_db() {
    let mut t = track(Some(-20.0), -10.0);
    t.loudness = None;
    assert_eq!(gain(normalization(&on(), &t, false)), (0.0, S::NotMeasured));
}

#[test]
fn n7_a_silent_track_gets_no_gain() {
    let t = track(None, -80.0);
    assert_eq!(gain(normalization(&on(), &t, false)), (0.0, S::Silent));
}

#[test]
fn n7_a_measured_track_is_brought_to_the_target() {
    let t = track(Some(-20.0), -10.0);
    assert_eq!(
        gain(normalization(&on(), &t, false)),
        (4.0, S::Applied { limited: None })
    );
}

#[test]
fn n7_a_boost_is_limited_by_the_true_peak_ceiling() {
    // Wants +4; the peak may rise only from −3 to −1 dBTP.
    let t = track(Some(-20.0), -3.0);
    assert_eq!(
        gain(normalization(&on(), &t, false)),
        (
            2.0,
            S::Applied {
                limited: Some(GainLimit::TruePeak)
            }
        )
    );
}

#[test]
fn n7_a_boost_is_limited_by_the_maximum_boost() {
    let t = track(Some(-40.0), -30.0);
    assert_eq!(
        gain(normalization(&on(), &t, false)),
        (
            12.0,
            S::Applied {
                limited: Some(GainLimit::MaxBoost)
            }
        )
    );
}

#[test]
fn n7_on_a_tie_the_true_peak_is_named() {
    // Wants +24; the ceiling allows +12 and so does the maximum boost.
    let t = track(Some(-40.0), -13.0);
    assert_eq!(
        gain(normalization(&on(), &t, false)),
        (
            12.0,
            S::Applied {
                limited: Some(GainLimit::TruePeak)
            }
        )
    );
}

#[test]
fn n7_a_boost_with_the_peak_already_above_the_ceiling_gets_0_db() {
    // Spec §10: −18 LUFS, +0.5 dBTP, target −16 → +0 dB.
    let t = track(Some(-18.0), 0.5);
    assert_eq!(
        gain(normalization(&on(), &t, false)),
        (
            0.0,
            S::Applied {
                limited: Some(GainLimit::TruePeak)
            }
        )
    );
}

#[test]
fn n7_a_cut_is_never_limited() {
    // Spec §10: −10 LUFS, +0.5 dBTP keeps its −6 dB.
    let t = track(Some(-10.0), 0.5);
    assert_eq!(
        gain(normalization(&on(), &t, false)),
        (-6.0, S::Applied { limited: None })
    );
    // Nor by the maximum boost.
    let mut c = on();
    c.max_boost_db = 0.0;
    let t = track(Some(0.0), -0.1);
    assert_eq!(
        gain(normalization(&c, &t, false)),
        (-16.0, S::Applied { limited: None })
    );
}

#[test]
fn n7_the_trim_is_added_before_the_limits() {
    let mut t = track(Some(-20.0), -10.0);
    t.loudness_edit.trim_db = 3.0; // wants +7, ceiling allows +9
    assert_eq!(
        gain(normalization(&on(), &t, false)),
        (7.0, S::Applied { limited: None })
    );
    t.loudness_edit.trim_db = 6.0; // wants +10, ceiling allows +9
    assert_eq!(
        gain(normalization(&on(), &t, false)),
        (
            9.0,
            S::Applied {
                limited: Some(GainLimit::TruePeak)
            }
        )
    );
    t.loudness_edit.trim_db = -6.0; // wants −2: a cut
    assert_eq!(
        gain(normalization(&on(), &t, false)),
        (-2.0, S::Applied { limited: None })
    );
}

#[test]
fn n7_a_stored_trim_is_limited_by_the_current_maximum() {
    let mut t = track(Some(-20.0), -20.0);
    t.loudness_edit.trim_db = 10.0;
    let mut c = on();
    c.max_trim_db = 2.0;
    assert_eq!(gain(normalization(&c, &t, false)).0, 6.0);
    t.loudness_edit.trim_db = f32::NAN;
    assert_eq!(gain(normalization(&c, &t, false)).0, 4.0);
}

#[test]
fn n7_the_gain_is_rounded_to_a_hundredth_of_a_db() {
    let t = track(Some(-20.004_7), -10.0);
    assert_eq!(normalization(&on(), &t, false).gain_db, 4.0);
    let t = track(Some(-20.016), -10.0);
    assert_eq!(normalization(&on(), &t, false).gain_db, 4.02);
}

#[test]
fn n7_rounding_never_takes_a_boost_above_the_true_peak_ceiling() {
    // The ceiling leaves +2.006 dB: rounding to the nearest hundredth
    // (+2.01) would pass it, so a gain the ceiling limits rounds down.
    let config = on();
    for tp in [-3.006, -3.009, -2.995, -4.9951] {
        let t = track(Some(-20.0), tp);
        let n = normalization(&config, &t, false);
        assert_eq!(
            n.status,
            S::Applied {
                limited: Some(GainLimit::TruePeak)
            },
            "{tp}"
        );
        let room = config.true_peak_ceiling_dbtp - tp;
        assert!(n.gain_db <= room, "{tp}: {} > {room}", n.gain_db);
        assert!(room - n.gain_db < 0.01, "{tp}: {} too low", n.gain_db);
    }
    // A room already on a hundredth keeps it.
    let t = track(Some(-20.0), -3.01);
    assert_eq!(normalization(&config, &t, false).gain_db, 2.01);
}

#[test]
fn n7_rounding_never_takes_any_boost_above_the_true_peak_ceiling() {
    // Unlimited (wanted +2.006, room +2.008) and limited by the maximum
    // boost (+2.006 allowed, room +2.008): the nearest hundredth (+2.01)
    // passes the ceiling, so the gain steps down to +2.00.
    let config = on();
    let t = track(Some(-18.006), -3.008);
    let n = normalization(&config, &t, false);
    assert_eq!(n.status, S::Applied { limited: None });
    assert_eq!(n.gain_db, 2.0);
    let config = LoudnessConfig {
        max_boost_db: 2.006,
        ..on()
    };
    let t = track(Some(-20.0), -3.008);
    let n = normalization(&config, &t, false);
    assert_eq!(
        n.status,
        S::Applied {
            limited: Some(GainLimit::MaxBoost)
        }
    );
    assert_eq!(n.gain_db, 2.0);
}

#[test]
fn n7_a_room_on_a_hundredth_keeps_it_and_no_boost_passes_the_room() {
    let config = on();
    // Every peak on a hundredth leaves a room on a hundredth: the gain the
    // ceiling limits is exactly that room, never 0.01 dB less.
    for k in 101..=2000 {
        let tp = -(k as f32) / 100.0;
        let n = normalization(&config, &track(Some(-40.0), tp), false);
        let room = (k - 100) as f32 / 100.0;
        let expected = if room > config.max_boost_db {
            config.max_boost_db
        } else {
            room
        };
        assert_eq!(n.gain_db, expected, "peak {tp}");
    }
    // Whatever the peak and the loudness, a boost never passes the room by
    // more than f32 noise, and never stays a hundredth or more below it when
    // the ceiling limits it.
    for i in 0..4000 {
        let tp = -1.0 - i as f32 * 0.002_7;
        for lufs in [-40.0, -18.0 - i as f32 * 0.001_3] {
            let n = normalization(&config, &track(Some(lufs), tp), false);
            let room = config.true_peak_ceiling_dbtp - tp;
            assert!(
                n.gain_db <= room + 1e-4,
                "{tp} {lufs}: {} > {room}",
                n.gain_db
            );
            if n.status
                == (S::Applied {
                    limited: Some(GainLimit::TruePeak),
                })
            {
                assert!(room - n.gain_db < 0.01 + 1e-5, "{tp}: {} low", n.gain_db);
            }
        }
    }
}

#[test]
fn unity_is_exactly_one() {
    assert_eq!(Normalization::default().linear_gain(), 1.0);
    let six = Normalization {
        gain_db: 6.02,
        status: S::Applied { limited: None },
    };
    assert!((six.linear_gain() - 2.0).abs() < 1e-3);
}

/// A state normalizing 3 tracks of −20 LUFS / −10 dBTP: +4 dB each.
fn normalizing(tracks: usize) -> AppState {
    let mut s = fixture(tracks);
    s.config.loudness.normalize = true;
    for t in s.library.iter_mut() {
        t.analyzed = true;
        t.loudness = Some(LoudnessMeasurement {
            integrated_lufs: Some(-20.0),
            true_peak_dbtp: -10.0,
        });
    }
    // The fixture preloaded the first entry at 0 dB; one `apply` preloads
    // it again with its gain, so each test sees only its own actions.
    let p = p0(&s);
    apply(&mut s, Command::SetVolume(p, 1.0)).unwrap();
    s
}

fn route(device: &str) -> Route {
    Route {
        backend: "null".into(),
        device: device.into(),
        first_channel: 0,
    }
}

fn device(name: &str) -> OutputDevice {
    OutputDevice {
        backend: "null".into(),
        device: name.into(),
    }
}

#[test]
fn n8_the_default_output_is_not_bit_perfect() {
    let s = normalizing(1);
    assert!(!holder_bit_perfect(&s, Holder::PlayerMain(p0(&s))));
}

#[test]
fn n8_before_the_engine_reports_the_configured_route_decides() {
    let mut s = normalizing(1);
    let p = p0(&s);
    s.config.outputs.routes[0].main = Some(route("dac"));
    s.config.outputs.bit_perfect.push(device("dac"));
    assert!(holder_bit_perfect(&s, Holder::PlayerMain(p)));
    assert!(
        !holder_bit_perfect(&s, Holder::PlayerCue(p)),
        "phones are not"
    );
}

#[test]
fn n8_once_placed_the_running_device_decides() {
    let mut s = normalizing(1);
    let p = p0(&s);
    // Configured bit-perfect, but the engine runs the device without it.
    s.config.outputs.routes[0].main = Some(route("dac"));
    s.config.outputs.bit_perfect.push(device("dac"));
    let mut running: DeviceSettings = device_settings(&s.config.outputs, &device("dac"));
    running.bit_perfect = false;
    on_event(
        &mut s,
        EngineEvent::Placed {
            holder: Holder::PlayerMain(p),
            route: Some(route("dac")),
            device: device("dac"),
            running,
        },
    );
    assert!(!holder_bit_perfect(&s, Holder::PlayerMain(p)));
}

#[test]
fn n8_a_bit_perfect_player_gets_exactly_unity_gain() {
    let mut s = normalizing(1);
    let p = p0(&s);
    place_on_dac(&mut s, Holder::PlayerMain(p));
    assert!(bit_perfect_player(&s, p));
    assert!(holder_bit_perfect(&s, Holder::PlayerMain(p)));
    let track = s.library.iter().next().unwrap().clone();
    let n = normalization_for(&s, Holder::PlayerMain(p), &track);
    assert_eq!(gain(n), (0.0, S::BitPerfect));
    assert_eq!(n.linear_gain().to_bits(), 1.0f32.to_bits());
}

#[test]
fn n8_the_cartwall_follows_its_own_device() {
    let mut s = normalizing(1);
    place_on_dac(&mut s, Holder::CartwallMain);
    assert!(holder_bit_perfect(&s, Holder::CartwallMain));
    assert!(!holder_bit_perfect(&s, Holder::CartwallCue));
}

#[test]
fn n7_a_configuration_that_was_not_validated_never_panics() {
    let mut t = track(Some(-20.0), -10.0);
    t.loudness_edit.trim_db = 3.0;
    let mut c = on();
    c.max_trim_db = f32::NAN;
    assert_eq!(gain(normalization(&c, &t, false)).0, 4.0, "no trim allowed");
    c.max_trim_db = -5.0;
    assert_eq!(gain(normalization(&c, &t, false)).0, 4.0);
    c.max_trim_db = f32::INFINITY;
    assert_eq!(gain(normalization(&c, &t, false)).0, 4.0, "infinite: none");
    // A maximum boost that is not a number or below 0 allows no boost.
    let t = track(Some(-30.0), -20.0);
    let mut c = on();
    c.max_boost_db = f32::NAN;
    assert_eq!(gain(normalization(&c, &t, false)).0, 0.0, "NaN boost");
    c.max_boost_db = -5.0;
    assert_eq!(gain(normalization(&c, &t, false)).0, 0.0, "negative boost");
}

fn request_gain(actions: &[EngineAction]) -> Vec<(&'static str, f32)> {
    actions
        .iter()
        .filter_map(|a| match a {
            EngineAction::Preload {
                request: Some(r), ..
            } => Some(("preload", r.normalization.gain_db)),
            EngineAction::StartCurrent { request, .. } => {
                Some(("start", request.normalization.gain_db))
            }
            EngineAction::Crossfade { request, .. } => {
                Some(("crossfade", request.normalization.gain_db))
            }
            EngineAction::StartCue { request, .. } => Some(("cue", request.normalization.gain_db)),
            EngineAction::LoadPaused { request, .. } => {
                Some(("load", request.normalization.gain_db))
            }
            EngineAction::StartCart(r) => Some(("cart", r.normalization.gain_db)),
            EngineAction::StartCartCue(r) => Some(("cart-cue", r.normalization.gain_db)),
            _ => None,
        })
        .collect()
}

#[test]
fn n9_player_requests_carry_the_main_gain() {
    let mut s = normalizing(3);
    let p = p0(&s);
    let start = apply(&mut s, Command::Play(p)).unwrap();
    assert!(request_gain(&start).contains(&("start", 4.0)), "{start:?}");
    assert!(
        request_gain(&start).contains(&("preload", 4.0)),
        "{start:?}"
    );
    let fade = apply(&mut s, Command::Play(p)).unwrap();
    assert!(
        request_gain(&fade).contains(&("crossfade", 4.0)),
        "{fade:?}"
    );
}

#[test]
fn n9_the_cue_is_normalized_like_on_air() {
    let mut s = normalizing(3);
    let p = p0(&s);
    let cue = apply(&mut s, Command::ToggleCue(p)).unwrap();
    assert_eq!(request_gain(&cue), vec![("cue", 4.0)]);
}

#[test]
fn n9_a_bit_perfect_cue_device_gets_no_gain_while_main_does() {
    let mut s = normalizing(3);
    let p = p0(&s);
    s.config.outputs.bit_perfect.push(device("phones"));
    let cue = apply(&mut s, Command::ToggleCue(p)).unwrap();
    assert_eq!(request_gain(&cue), vec![("cue", 0.0)]);
    let start = apply(&mut s, Command::Play(p)).unwrap();
    assert!(request_gain(&start).contains(&("start", 4.0)));
}

/// Gives the first cart of the first page the file `path`, measured like
/// the others (+4 dB).
fn with_cart(s: &mut AppState) -> fp_model::CartId {
    let page = s.cartwall.pages[0].id;
    apply(
        s,
        Command::AssignCartFile {
            page,
            index: 0,
            path: PathBuf::from("/carts/jingle.flac"),
        },
    )
    .unwrap();
    let cart = s.cartwall.pages[0].carts[0].clone();
    let t = s.library.get_mut(cart.track.unwrap()).unwrap();
    t.analyzed = true;
    t.loudness = Some(LoudnessMeasurement {
        integrated_lufs: Some(-20.0),
        true_peak_dbtp: -10.0,
    });
    cart.id
}

#[test]
fn n9_carts_and_their_pre_listen_carry_the_gain() {
    let mut s = normalizing(1);
    let cart = with_cart(&mut s);
    let fire = apply(&mut s, Command::FireCart(cart)).unwrap();
    assert_eq!(request_gain(&fire), vec![("cart", 4.0)]);
    let cue = apply(&mut s, Command::CueCart(cart)).unwrap();
    assert_eq!(request_gain(&cue), vec![("cart-cue", 4.0)]);
}

#[test]
fn n9_with_normalization_off_every_request_is_at_0_db() {
    let mut s = normalizing(3);
    s.config.loudness.normalize = false;
    let p = p0(&s);
    let start = apply(&mut s, Command::Play(p)).unwrap();
    assert!(request_gain(&start).iter().all(|(_, g)| *g == 0.0));
}

#[test]
fn n10_the_edit_is_stored_with_the_track() {
    let mut s = normalizing(1);
    let track = s.library.iter().next().unwrap().id;
    apply(
        &mut s,
        Command::SetTrackLoudness {
            track,
            excluded: true,
            trim_db: -2.5,
        },
    )
    .unwrap();
    assert_eq!(
        s.library.get(track).unwrap().loudness_edit,
        LoudnessEdit {
            excluded: true,
            trim_db: -2.5
        }
    );
}

#[test]
fn n10_a_trim_out_of_range_or_not_a_number_is_refused() {
    let mut s = normalizing(1);
    let track = s.library.iter().next().unwrap().id;
    for trim_db in [12.5, -12.5, f32::NAN, f32::INFINITY] {
        let before = s.clone();
        assert_eq!(
            apply(
                &mut s,
                Command::SetTrackLoudness {
                    track,
                    excluded: false,
                    trim_db
                }
            ),
            Err(ModelError::InvalidTrim),
            "{trim_db}"
        );
        assert_eq!(s, before, "a refused command changes nothing");
    }
    // The limits themselves are allowed.
    apply(
        &mut s,
        Command::SetTrackLoudness {
            track,
            excluded: false,
            trim_db: 12.0,
        },
    )
    .unwrap();
}

#[test]
fn n10_a_maximum_not_validated_allows_the_trim_the_gain_rule_uses() {
    // The edit is refused exactly when N7 would not use it: a maximum that
    // is not a number, infinite or below 0 allows no trim at all.
    for max in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -3.0] {
        let mut s = normalizing(1);
        s.config.loudness.max_trim_db = max;
        let track = s.library.iter().next().unwrap().id;
        let set = |s: &mut AppState, trim_db| {
            apply(
                s,
                Command::SetTrackLoudness {
                    track,
                    excluded: false,
                    trim_db,
                },
            )
        };
        assert_eq!(set(&mut s, 3.0), Err(ModelError::InvalidTrim), "{max}");
        assert!(set(&mut s, 0.0).is_ok(), "{max}");
    }
}

#[test]
fn n10_an_unknown_track_is_refused() {
    let mut s = normalizing(1);
    assert_eq!(
        apply(
            &mut s,
            Command::SetTrackLoudness {
                track: TrackId(999),
                excluded: true,
                trim_db: 0.0
            }
        ),
        Err(ModelError::UnknownTrack(TrackId(999)))
    );
}

#[test]
fn n10_the_track_on_air_can_be_edited() {
    let mut s = normalizing(2);
    let p = p0(&s);
    apply(&mut s, Command::Play(p)).unwrap();
    let track = s.playlists.entry(entries(&s)[0]).unwrap().track;
    assert!(
        apply(
            &mut s,
            Command::SetTrackLoudness {
                track,
                excluded: true,
                trim_db: 0.0
            }
        )
        .is_ok()
    );
}

#[test]
fn n10_the_edit_survives_a_library_round_trip() {
    let mut s = normalizing(1);
    let track = s.library.iter().next().unwrap().id;
    apply(
        &mut s,
        Command::SetTrackLoudness {
            track,
            excluded: true,
            trim_db: 1.5,
        },
    )
    .unwrap();
    let json = serde_json::to_string(&s.library).unwrap();
    let back: fp_model::Library = serde_json::from_str(&json).unwrap();
    assert_eq!(
        back.get(track).unwrap().loudness_edit,
        s.library.get(track).unwrap().loudness_edit
    );
}

fn measure(s: &mut AppState, entry: fp_model::EntryId, lufs: f32) {
    let track = s.playlists.entry(entry).unwrap().track;
    s.library.get_mut(track).unwrap().loudness = Some(LoudnessMeasurement {
        integrated_lufs: Some(lufs),
        true_peak_dbtp: -20.0,
    });
}

/// The gains of the preloads sent to `player` (every player shows the same
/// playlist, so the stopped ones preload too).
fn preloads(actions: &[EngineAction], player: fp_model::PlayerId) -> Vec<f32> {
    actions
        .iter()
        .filter_map(|a| match a {
            EngineAction::Preload {
                player: to,
                request: Some(r),
            } if *to == player => Some(r.normalization.gain_db),
            _ => None,
        })
        .collect()
}

/// Only `Preload`s: nothing reaches what is sounding.
fn only_preloads(actions: &[EngineAction]) -> bool {
    actions
        .iter()
        .all(|a| matches!(a, EngineAction::Preload { .. }))
}

#[test]
fn n11_the_gain_on_air_is_recorded_and_kept() {
    let mut s = normalizing(3);
    let (p, e) = (p0(&s), entries(&s));
    apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(s.player(p).unwrap().on_air.map(|n| n.gain_db), Some(4.0));
    // An analysis result, a loudness-only result, a configuration change
    // and an edit of the track on air: nothing on air changes.
    let track = s.playlists.entry(e[0]).unwrap().track;
    let a = apply(
        &mut s,
        Command::ApplyLoudness {
            track,
            loudness: Some(LoudnessMeasurement {
                integrated_lufs: Some(-30.0),
                true_peak_dbtp: -20.0,
            }),
            version: fp_model::LOUDNESS_VERSION,
        },
    )
    .unwrap();
    let mut config = s.config.clone();
    config.loudness.target_lufs = -12.0;
    let b = apply(&mut s, Command::UpdateConfig(Box::new(config))).unwrap();
    let c = apply(
        &mut s,
        Command::SetTrackLoudness {
            track,
            excluded: true,
            trim_db: 0.0,
        },
    )
    .unwrap();
    for actions in [&a, &b, &c] {
        assert!(only_preloads(actions), "{actions:?}");
    }
    assert_eq!(s.player(p).unwrap().on_air.map(|n| n.gain_db), Some(4.0));
}

#[test]
fn n11_a_seek_sends_no_new_gain() {
    let mut s = normalizing(3);
    let p = p0(&s);
    apply(&mut s, Command::Play(p)).unwrap();
    s.config.loudness.target_lufs = -10.0;
    let seek = apply(&mut s, Command::Seek(p, 30.0)).unwrap();
    assert!(seek.iter().any(|a| matches!(a, EngineAction::Seek { .. })));
    assert!(
        request_gain(&seek)
            .iter()
            .all(|(kind, _)| *kind == "preload")
    );
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    let cue_seek = apply(&mut s, Command::SeekCue(p, 10.0)).unwrap();
    assert!(
        request_gain(&cue_seek)
            .iter()
            .all(|(kind, _)| *kind == "preload")
    );
}

/// The normalization `gain_db` the engine reports it started with.
fn started_with(gain_db: f32) -> Normalization {
    Normalization {
        gain_db,
        status: S::Applied { limited: None },
    }
}

#[test]
fn n11_a_scheduled_transition_takes_the_gain_the_engine_started_with() {
    let mut s = normalizing(3);
    let (p, e) = (p0(&s), entries(&s));
    apply(&mut s, Command::Play(p)).unwrap();
    measure(&mut s, e[1], -22.0); // the next now wants +6
    let pre = apply(&mut s, Command::SetVolume(p, 1.0)).unwrap();
    assert_eq!(preloads(&pre, p), vec![6.0]);
    // The model changes its mind after the engine already started it.
    s.library
        .get_mut(s.playlists.entry(e[1]).unwrap().track)
        .unwrap()
        .loudness = None;
    on_event(
        &mut s,
        EngineEvent::TransitionStarted {
            player: p,
            entry: e[1],
            normalization: started_with(6.0),
        },
    );
    assert_eq!(s.player(p).unwrap().on_air.map(|n| n.gain_db), Some(6.0));
}

#[test]
fn n11_a_re_preload_racing_a_committed_transition_keeps_the_engines_gain() {
    // The engine committed the transition with the +6 dB preload; the model
    // had meanwhile re-preloaded the same entry at +2 dB. The engine plays
    // +6 dB, and `on_air` must say so.
    let mut s = normalizing(3);
    let (p, e) = (p0(&s), entries(&s));
    apply(&mut s, Command::Play(p)).unwrap();
    measure(&mut s, e[1], -22.0);
    apply(&mut s, Command::SetVolume(p, 1.0)).unwrap();
    measure(&mut s, e[1], -18.0); // the next now wants +2: re-preloaded
    let pre = apply(&mut s, Command::SetVolume(p, 1.0)).unwrap();
    assert_eq!(preloads(&pre, p), vec![2.0]);
    on_event(
        &mut s,
        EngineEvent::TransitionStarted {
            player: p,
            entry: e[1],
            normalization: started_with(6.0),
        },
    );
    assert_eq!(s.player(p).unwrap().on_air.map(|n| n.gain_db), Some(6.0));
}

#[test]
fn n11_a_transition_to_another_entry_takes_the_engines_gain() {
    let mut s = normalizing(3);
    let (p, e) = (p0(&s), entries(&s));
    apply(&mut s, Command::Play(p)).unwrap();
    measure(&mut s, e[2], -24.0);
    // The engine reports an entry that was not the one preloaded.
    on_event(
        &mut s,
        EngineEvent::TransitionStarted {
            player: p,
            entry: e[2],
            normalization: started_with(8.0),
        },
    );
    assert_eq!(s.player(p).unwrap().on_air.map(|n| n.gain_db), Some(8.0));
}

#[test]
fn n11_the_record_is_cleared_when_the_source_ends() {
    let mut s = normalizing(3);
    let p = p0(&s);
    apply(&mut s, Command::Play(p)).unwrap();
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    assert!(s.player(p).unwrap().cue_normalization.is_some());
    apply(&mut s, Command::Stop(p)).unwrap();
    apply(&mut s, Command::ToggleCue(p)).unwrap();
    let player = s.player(p).unwrap();
    assert_eq!((player.on_air, player.cue_normalization), (None, None));
}

#[test]
fn n11_carts_keep_their_gain() {
    let mut s = normalizing(1);
    let cart = with_cart(&mut s);
    apply(&mut s, Command::FireCart(cart)).unwrap();
    apply(&mut s, Command::CueCart(cart)).unwrap();
    assert_eq!(s.cartwall.playing[0].normalization.gain_db, 4.0);
    assert_eq!(s.cartwall.cue_normalization.map(|n| n.gain_db), Some(4.0));
    s.config.loudness.target_lufs = -10.0;
    let p = p0(&s);
    apply(&mut s, Command::SetVolume(p, 1.0)).unwrap();
    assert_eq!(s.cartwall.playing[0].normalization.gain_db, 4.0);
    apply(&mut s, Command::CueCart(cart)).unwrap();
    assert_eq!(s.cartwall.cue_normalization, None);
}

#[test]
fn n12_a_measurement_of_the_next_preloads_it_again() {
    let mut s = normalizing(3);
    let (p, e) = (p0(&s), entries(&s));
    apply(&mut s, Command::Play(p)).unwrap();
    let track = s.playlists.entry(e[1]).unwrap().track;
    let a = apply(
        &mut s,
        Command::ApplyLoudness {
            track,
            loudness: Some(LoudnessMeasurement {
                integrated_lufs: Some(-18.0),
                true_peak_dbtp: -20.0,
            }),
            version: fp_model::LOUDNESS_VERSION,
        },
    )
    .unwrap();
    assert_eq!(preloads(&a, p), vec![2.0]);
}

#[test]
fn n12_a_configuration_change_preloads_the_next_again() {
    let mut s = normalizing(3);
    let p = p0(&s);
    apply(&mut s, Command::Play(p)).unwrap();
    let mut config = s.config.clone();
    config.loudness.target_lufs = -18.0;
    let a = apply(&mut s, Command::UpdateConfig(Box::new(config))).unwrap();
    assert_eq!(preloads(&a, p), vec![2.0]);
    // Unchanged settings preload nothing.
    let same = s.config.clone();
    assert!(
        preloads(
            &apply(&mut s, Command::UpdateConfig(Box::new(same))).unwrap(),
            p
        )
        .is_empty()
    );
}

#[test]
fn n12_an_edit_of_the_next_preloads_it_again() {
    let mut s = normalizing(3);
    let (p, e) = (p0(&s), entries(&s));
    apply(&mut s, Command::Play(p)).unwrap();
    let track = s.playlists.entry(e[1]).unwrap().track;
    let a = apply(
        &mut s,
        Command::SetTrackLoudness {
            track,
            excluded: false,
            trim_db: -1.5,
        },
    )
    .unwrap();
    assert_eq!(preloads(&a, p), vec![2.5]);
}

#[test]
fn n12_a_placement_on_a_bit_perfect_device_preloads_the_next_again() {
    let mut s = normalizing(3);
    let p = p0(&s);
    apply(&mut s, Command::Play(p)).unwrap();
    let mut running = device_settings(&s.config.outputs, &device("dac"));
    running.bit_perfect = true;
    let a = on_event(
        &mut s,
        EngineEvent::Placed {
            holder: Holder::PlayerMain(p),
            route: Some(route("dac")),
            device: device("dac"),
            running,
        },
    );
    assert_eq!(preloads(&a, p), vec![0.0]);
    assert_eq!(
        s.player(p).unwrap().on_air.map(|n| n.gain_db),
        Some(4.0),
        "N17"
    );
}

#[test]
fn n12_a_repeating_entry_takes_the_new_gain_on_its_next_pass() {
    let mut s = normalizing(3);
    let (p, e) = (p0(&s), entries(&s));
    apply(&mut s, Command::SetEntryRepeat(e[0], true)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    let mut config = s.config.clone();
    config.loudness.target_lufs = -14.0;
    let a = apply(&mut s, Command::UpdateConfig(Box::new(config))).unwrap();
    assert_eq!(
        preloads(&a, p),
        vec![6.0],
        "the repeat pass is preloaded again"
    );
    assert_eq!(s.player(p).unwrap().on_air.map(|n| n.gain_db), Some(4.0));
}

#[test]
fn n13_crash_recovery_computes_the_gain_again() {
    let mut s = normalizing(3);
    let p = p0(&s);
    apply(&mut s, Command::Play(p)).unwrap();
    s.config.loudness.target_lufs = -14.0;
    let parts = fp_model::RestoreParts {
        config: s.config.clone(),
        library: s.library.clone(),
        playlists: s.playlists.clone(),
        cart_pages: s.cartwall.pages.clone(),
        cartwall_session: s.cartwall.session(),
        layout: s.layout.clone(),
        ids: s.ids.clone(),
    };
    let (restored, actions) = AppState::restore(parts, &s.sessions(|_| 20.0), "Main");
    assert!(
        request_gain(&actions).contains(&("load", 6.0)),
        "{actions:?}"
    );
    assert_eq!(
        restored.player(p).unwrap().on_air.map(|n| n.gain_db),
        Some(6.0)
    );
    assert!(
        !actions.iter().any(|a| matches!(
            a,
            EngineAction::StartCurrent { .. } | EngineAction::Resume { .. }
        )),
        "nothing goes on air by itself"
    );
}

#[test]
fn n9_a_bit_perfect_player_sends_unity_gain() {
    let mut s = normalizing(3);
    let p = p0(&s);
    place_on_dac(&mut s, Holder::PlayerMain(p));
    let start = apply(&mut s, Command::Play(p)).unwrap();
    let gains = request_gain(&start);
    assert!(gains.contains(&("start", 0.0)), "{start:?}");
    assert!(gains.iter().all(|(_, g)| *g == 0.0), "{gains:?}");
    assert_eq!(
        s.player(p).unwrap().on_air.map(|n| n.status),
        Some(S::BitPerfect)
    );
}

#[test]
fn n7_the_wanted_gain_is_the_target_minus_the_measurement_plus_the_trim() {
    assert_eq!(wanted_gain_db(&on(), -20.0, 0.0), 4.0);
    assert_eq!(wanted_gain_db(&on(), -10.0, 1.5), -4.5);
}
