#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Loudness normalization spec §6: the gain rule (N7), the device rule
//! (N8), requests (N9), edits (N10), fixed while sounding (N11), loaded
//! afterwards (N12) and crash recovery (N13).

mod common;

use std::path::PathBuf;

use common::{fixture, p0, place_on_dac};
use fp_model::{
    AppState, Command, DeviceSettings, EngineAction, EngineEvent, GainLimit, Holder,
    LoudnessConfig, LoudnessMeasurement, Normalization, NormalizationStatus as S, OutputDevice,
    Route, Track, TrackId, apply, bit_perfect_player, device_settings, holder_bit_perfect,
    normalization, normalization_for, on_event,
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
        gain(normalization(&LoudnessConfig::default(), &t, false)),
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
