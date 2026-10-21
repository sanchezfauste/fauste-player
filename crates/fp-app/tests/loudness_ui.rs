#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Loudness normalization in the interface (loudness normalization spec
//! §8): the gain badge (N18), the tooltip lines (N19), the Loudness… popup
//! (N20), the cart editor (N21) and the Settings tab (N22).

mod support;

use egui::accesskit::Role;
use egui::{Event, Modifiers, PointerButton};
use egui_kittest::kittest::Queryable;
use fp_model::{AppState, Command, LoudnessMeasurement, apply};
use support::{harness, state};

/// One player on air with −20 LUFS tracks: +4.0 dB at the default target.
fn on_air() -> AppState {
    let mut s = state(1, 3);
    s.config.loudness.normalize = true;
    for t in s.library.iter_mut() {
        t.analyzed = true;
        t.duration_secs = 180.0;
        t.loudness = Some(LoudnessMeasurement {
            integrated_lufs: Some(-20.0),
            true_peak_dbtp: -10.0,
        });
    }
    let p = s.players[0].id;
    apply(&mut s, Command::Play(p)).unwrap();
    s
}

/// Places player 1's Main on a bit-perfect device.
fn bit_perfect(s: &mut AppState) {
    let p = s.players[0].id;
    let dac = fp_model::OutputDevice {
        backend: "null".into(),
        device: "dac".into(),
    };
    s.config.outputs.bit_perfect = vec![dac.clone()];
    let running = fp_model::device_settings(&s.config.outputs, &dac);
    fp_model::on_event(
        s,
        fp_model::EngineEvent::Placed {
            holder: fp_model::Holder::PlayerMain(p),
            route: None,
            device: dac,
            running,
        },
    );
}

#[test]
fn n18_the_header_shows_the_gain_and_says_why() {
    let (mut h, _fake) = harness(on_air());
    h.run_steps(2);
    // The tile's accessible name is its tooltip, as for the BP badge.
    assert!(
        h.query_by_label("Gain +4.0 dB: brings the measured −20.0 LUFS to the −16.0 LUFS target.")
            .is_some()
    );
    assert!(
        h.query_by_label("+4.0 dB").is_none(),
        "named by its tooltip"
    );
}

#[test]
fn n18_a_track_not_measured_says_so() {
    let mut s = on_air();
    for t in s.library.iter_mut() {
        t.loudness = None;
    }
    // Start again so that the source on air is the unmeasured one.
    let p = s.players[0].id;
    apply(&mut s, Command::Stop(p)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    let (mut h, _fake) = harness(s);
    h.run_steps(2);
    assert!(
        h.query_by_label("Loudness not measured yet: this track plays unchanged.")
            .is_some()
    );
}

#[test]
fn n18_no_badge_when_stopped_or_off() {
    let mut s = on_air();
    let p = s.players[0].id;
    apply(&mut s, Command::Stop(p)).unwrap();
    let (mut h, _fake) = harness(s);
    h.run_steps(2);
    assert!(h.query_by_label_contains("brings the measured").is_none());

    let mut s = on_air();
    s.config.loudness.normalize = false;
    apply(&mut s, Command::Stop(p)).unwrap();
    apply(&mut s, Command::Play(p)).unwrap();
    let (mut h, _fake) = harness(s);
    h.run_steps(2);
    assert!(h.query_by_label_contains("brings the measured").is_none());
    assert!(h.query_by_label_contains("not measured").is_none());
}

#[test]
fn n18_a_bit_perfect_player_plays_at_unity_gain_and_says_so() {
    let mut s = on_air();
    let p = s.players[0].id;
    apply(&mut s, Command::Stop(p)).unwrap();
    bit_perfect(&mut s);
    apply(&mut s, Command::Play(p)).unwrap();
    let (mut h, _fake) = harness(s);
    h.run_steps(2);
    assert!(
        h.query_by_label("Bit-perfect output: tracks play unaltered.")
            .is_some()
    );
    assert!(h.query_by_label_contains("brings the measured").is_none());
}

#[test]
fn n18_a_paused_player_keeps_its_badge_and_gain() {
    let mut s = on_air();
    let p = s.players[0].id;
    apply(&mut s, Command::Pause(p)).unwrap();
    let mut config = s.config.clone();
    config.loudness.target_lufs = -10.0;
    apply(&mut s, Command::UpdateConfig(Box::new(config))).unwrap();
    let (mut h, _fake) = harness(s);
    h.run_steps(2);
    // The source on air keeps +4.0 dB; the tooltip says that the gain was
    // set at load, and quotes none of the new settings beside it.
    assert!(
        h.query_by_label_contains("Gain set when the track was loaded")
            .is_some()
    );
    assert!(h.query_by_label_contains("brings the measured").is_none());
}

#[test]
fn n19_the_row_tooltip_shows_the_loudness() {
    let (mut h, _) = harness(on_air());
    h.get_by_label("Song 3").hover();
    h.run_steps(40);
    assert!(
        h.query_by_label("Loudness: −20.0 LUFS, true peak −10.0 dBTP")
            .is_some()
    );
    assert!(h.query_by_label("Excluded from normalization").is_none());
}

#[test]
fn n19_the_row_tooltip_shows_the_edit() {
    let mut s = on_air();
    let playlist = s.playlists.first_id().unwrap();
    let track = s.playlists.get(playlist).unwrap().entries[2].track;
    apply(
        &mut s,
        Command::SetTrackLoudness {
            track,
            excluded: true,
            trim_db: 0.0,
        },
    )
    .unwrap();
    let (mut h, _) = harness(s);
    h.get_by_label("Song 3").hover();
    h.run_steps(40);
    assert!(h.query_by_label("Excluded from normalization").is_some());
}

/// Opens the waveform's context menu of player 1 and picks Loudness….
fn open_popup(h: &mut egui_kittest::Harness<'static, fp_app::ui::app::AppUi>) {
    let wave = h.get_by_label("Waveform: click to seek").rect();
    h.event(Event::PointerMoved(wave.center()));
    for pressed in [true, false] {
        h.event(Event::PointerButton {
            pos: wave.center(),
            button: PointerButton::Secondary,
            pressed,
            modifiers: Modifiers::NONE,
        });
    }
    h.run_steps(2);
    h.get_by_label("Loudness…").click();
    h.run_steps(2);
}

fn loudness_edits(sent: &[Command]) -> Vec<(bool, f32)> {
    sent.iter()
        .filter_map(|c| match c {
            Command::SetTrackLoudness {
                excluded, trim_db, ..
            } => Some((*excluded, *trim_db)),
            _ => None,
        })
        .collect()
}

#[test]
fn n20_the_popup_shows_the_measurement_and_excludes_the_track() {
    let (mut h, fake) = harness(on_air());
    h.run_steps(2);
    open_popup(&mut h);
    assert!(
        h.query_by_label("Loudness: −20.0 LUFS, true peak −10.0 dBTP")
            .is_some()
    );
    fake.take_sent();
    h.get_by_role_and_label(Role::CheckBox, "Exclude from normalization")
        .click();
    h.run_steps(2);
    assert_eq!(loudness_edits(&fake.take_sent()), vec![(true, 0.0)]);
}

#[test]
fn n20_the_trim_slider_sends_half_db_steps_and_resets_on_double_click() {
    let (mut h, fake) = harness(on_air());
    h.run_steps(2);
    open_popup(&mut h);
    fake.take_sent();
    h.get_by_role_and_label(Role::Slider, "Trim").focus();
    h.run_steps(1);
    h.key_press(egui::Key::ArrowRight);
    h.run_steps(2);
    assert_eq!(loudness_edits(&fake.take_sent()), vec![(false, 0.5)]);
    let rect = h.get_by_role_and_label(Role::Slider, "Trim").rect();
    let at = egui::pos2(rect.left() + 4.0, rect.center().y);
    h.event(Event::PointerMoved(at));
    // Two clicks a frame apart, within the double-click delay.
    for _ in 0..2 {
        for pressed in [true, false] {
            h.event(Event::PointerButton {
                pos: at,
                button: PointerButton::Primary,
                pressed,
                modifiers: Modifiers::NONE,
            });
        }
        h.run_steps(1);
    }
    h.run_steps(2);
    let sent = fake.take_sent();
    assert_eq!(loudness_edits(&sent).last(), Some(&(false, 0.0)));
    assert_eq!(
        fake.state
            .load()
            .library
            .iter()
            .next()
            .unwrap()
            .loudness_edit
            .trim_db,
        0.0
    );
}

#[test]
fn a_zero_maximum_trim_still_allows_excluding() {
    let mut s = on_air();
    s.config.loudness.max_trim_db = 0.0;
    let (mut h, fake) = harness(s);
    h.run_steps(2);
    open_popup(&mut h);
    fake.take_sent();
    h.get_by_role_and_label(Role::CheckBox, "Exclude from normalization")
        .click();
    h.run_steps(2);
    assert_eq!(loudness_edits(&fake.take_sent()), vec![(true, 0.0)]);
}

#[test]
fn excluding_sends_the_trim_in_use() {
    // A trim saved above a maximum lowered since: the switch saves the
    // trim the gain uses (N7), as the slider shows it.
    let mut s = on_air();
    s.config.loudness.max_trim_db = 6.0;
    for t in s.library.iter_mut() {
        t.loudness_edit.trim_db = 10.0;
    }
    let (mut h, fake) = harness(s);
    h.run_steps(2);
    open_popup(&mut h);
    fake.take_sent();
    h.get_by_role_and_label(Role::CheckBox, "Exclude from normalization")
        .click();
    h.run_steps(2);
    assert_eq!(loudness_edits(&fake.take_sent()), vec![(true, 6.0)]);
}

#[test]
fn n20_the_cue_window_waveform_menu_opens_the_popup_for_the_cued_track() {
    use fp_app::ui::controller::Controller;
    let s = on_air();
    let cued = s.players[0].next.unwrap();
    let track = s.playlists.entry(cued).unwrap().track;
    let (mut h, fake) = harness(s);
    fake.send(Command::ToggleCue(fake.player(0)));
    h.run_steps(2);
    let wave = h.get_by_label("CUE waveform: click to seek").rect();
    h.event(Event::PointerMoved(wave.center()));
    for pressed in [true, false] {
        h.event(Event::PointerButton {
            pos: wave.center(),
            button: PointerButton::Secondary,
            pressed,
            modifiers: Modifiers::NONE,
        });
    }
    h.run_steps(2);
    h.get_by_label("Loudness…").click();
    h.run_steps(2);
    fake.take_sent();
    h.get_by_role_and_label(Role::CheckBox, "Exclude from normalization")
        .click();
    h.run_steps(2);
    let sent: Vec<_> = fake
        .take_sent()
        .into_iter()
        .filter_map(|c| match c {
            Command::SetTrackLoudness { track, .. } => Some(track),
            _ => None,
        })
        .collect();
    assert_eq!(sent, vec![track], "the cued track, not the one on air");
}

#[test]
fn the_popup_keeps_its_track_and_closes_when_the_track_is_gone() {
    let (mut h, fake) = harness(on_air());
    h.run_steps(2);
    open_popup(&mut h);
    let track = fake.state.load().players[0]
        .current
        .and_then(|e| fake.state.load().track_for_entry(e).map(|t| t.id))
        .unwrap();
    // The player moves on: the popup still edits the track it was opened for.
    let p = fake.player(0);
    let other = fake.entries()[1];
    let mut next = (**fake.state.load()).clone();
    for c in [
        Command::Stop(p),
        Command::SetNext(p, other),
        Command::Play(p),
    ] {
        apply(&mut next, c).unwrap();
    }
    assert_ne!(
        next.players[0]
            .current
            .and_then(|e| next.track_for_entry(e))
            .map(|t| t.id),
        Some(track)
    );
    fake.state.store(std::sync::Arc::new(next));
    h.run_steps(2);
    fake.take_sent();
    h.get_by_role_and_label(Role::CheckBox, "Exclude from normalization")
        .click();
    h.run_steps(2);
    assert!(
        fake.take_sent().iter().any(|c| matches!(
            c,
            Command::SetTrackLoudness { track: t, excluded: true, .. } if *t == track
        )),
        "the popup's own track"
    );
    let mut next = (**fake.state.load()).clone();
    next.library.remove(track);
    fake.state.store(std::sync::Arc::new(next));
    h.run_steps(2);
    assert!(
        h.query_by_role_and_label(Role::CheckBox, "Exclude from normalization")
            .is_none()
    );
}

#[test]
fn the_popup_closes_with_its_button() {
    let (mut h, _fake) = harness(on_air());
    h.run_steps(2);
    open_popup(&mut h);
    h.get_by_label("Close").click();
    h.run_steps(2);
    assert!(
        h.query_by_role_and_label(Role::CheckBox, "Exclude from normalization")
            .is_none()
    );
}

#[test]
fn n21_the_cart_editor_has_the_same_controls() {
    let mut s = state(1, 0);
    let page = s.cartwall.pages[0].id;
    apply(
        &mut s,
        Command::AssignCartFile {
            page,
            index: 0,
            path: "/carts/a.flac".into(),
        },
    )
    .unwrap();
    let track = s.cartwall.pages[0].carts[0].track.unwrap();
    s.library.get_mut(track).unwrap().loudness = Some(LoudnessMeasurement {
        integrated_lufs: Some(-12.0),
        true_peak_dbtp: -1.5,
    });
    let (mut h, fake) = harness(s);
    h.get_by_label("a").click_secondary();
    h.run_steps(2);
    h.get_by_label("Edit…").click();
    h.run_steps(3);
    assert!(
        h.query_by_label("Loudness: −12.0 LUFS, true peak −1.5 dBTP")
            .is_some()
    );
    // The editor is below the grid: scroll to it first, as a user would.
    h.get_by_role_and_label(Role::CheckBox, "Exclude from normalization")
        .scroll_to_me();
    h.run_steps(3);
    fake.take_sent();
    h.get_by_role_and_label(Role::CheckBox, "Exclude from normalization")
        .click();
    h.run_steps(2);
    assert_eq!(loudness_edits(&fake.take_sent()), vec![(true, 0.0)]);
    assert!(
        h.query_by_role_and_label(Role::Slider, "Trim").is_some(),
        "the trim too"
    );
}

fn open_loudness_tab(
    s: AppState,
) -> (
    egui_kittest::Harness<'static, fp_app::ui::app::AppUi>,
    std::sync::Arc<support::Fake>,
) {
    let (mut h, fake) = harness(s);
    h.get_by_label("Settings").click();
    h.run_steps(2);
    h.get_by_role_and_label(Role::Button, "Loudness").click();
    h.run_steps(2);
    fake.take_sent();
    (h, fake)
}

#[test]
fn n22_the_toggle_turns_normalization_on() {
    let (mut h, fake) = open_loudness_tab(state(1, 1));
    assert!(
        !fake.state.load().config.loudness.normalize,
        "off by default"
    );
    h.get_by_role_and_label(Role::CheckBox, "Normalize loudness")
        .click();
    h.run_steps(2);
    assert!(fake.state.load().config.loudness.normalize);
}

#[test]
fn n22_the_sliders_keep_their_ranges() {
    let mut s = state(1, 1);
    s.config.loudness.target_lufs = -6.0;
    let (mut h, fake) = open_loudness_tab(s);
    let mut press = |key| {
        h.get_by_role_and_label(Role::Slider, "Target").focus();
        h.run_steps(1);
        h.key_press(key);
        h.run_steps(2);
        fake.state.load().config.loudness.target_lufs
    };
    assert_eq!(press(egui::Key::ArrowLeft), -6.5, "0.5 LUFS steps");
    assert_eq!(press(egui::Key::ArrowRight), -6.0);
    assert_eq!(press(egui::Key::ArrowRight), -6.0, "the top of the range");
    for label in ["True-peak ceiling", "Maximum boost", "Maximum trim"] {
        assert!(
            h.query_by_role_and_label(Role::Slider, label).is_some(),
            "{label}"
        );
    }
    assert!(
        h.query_by_label(
            "Changes apply to tracks loaded afterwards; what is playing keeps its gain."
        )
        .is_some()
    );
    assert!(
        h.query_by_label_contains("bit-perfect output always plays at unity gain")
            .is_some(),
        "a bit-perfect output is never normalized"
    );
}

#[test]
fn n22_it_counts_the_tracks_still_waiting_for_a_measurement() {
    let mut s = state(1, 3);
    for t in s.library.iter_mut() {
        t.analyzed = true;
    }
    let (mut h, _fake) = open_loudness_tab(s);
    h.run_steps(2);
    assert!(h.query_by_label("3 tracks not measured yet").is_some());
}

#[test]
fn n22_no_count_once_every_track_is_measured() {
    let mut s = state(1, 1);
    for t in s.library.iter_mut() {
        t.loudness_pass = Some(fp_model::LOUDNESS_VERSION);
    }
    let (mut h, _fake) = open_loudness_tab(s);
    h.run_steps(2);
    assert!(h.query_by_label_contains("not measured yet").is_none());
}

#[test]
fn n22_the_unmeasured_count_is_worked_out_once_per_snapshot() {
    use std::sync::Arc;
    let first = Arc::new(state(1, 3));
    let mut count = fp_app::services::UnmeasuredCount::default();
    for _ in 0..10 {
        assert_eq!(count.get(&first), 3);
    }
    assert_eq!(count.scans(), 1);
    let mut s = state(1, 3);
    s.library.iter_mut().next().unwrap().loudness_pass = Some(fp_model::LOUDNESS_VERSION);
    let second = Arc::new(s);
    assert_eq!(count.get(&second), 2);
    assert_eq!(count.scans(), 2);
}

#[test]
fn n20_the_row_menu_opens_the_popup_of_that_track() {
    let (mut h, fake) = harness(on_air());
    h.run_steps(2);
    let track = fake.state.load().playlists.iter().next().unwrap().entries[1].track;
    h.get_all_by_label("Song 2")
        .last()
        .unwrap()
        .click_secondary();
    h.run_steps(2);
    h.get_by_label("Loudness…").click();
    h.run_steps(2);
    assert!(
        h.query_by_label("Loudness: −20.0 LUFS, true peak −10.0 dBTP")
            .is_some()
    );
    fake.take_sent();
    h.get_by_role_and_label(Role::Slider, "Trim").focus();
    h.run_steps(1);
    h.key_press(egui::Key::ArrowRight);
    h.run_steps(2);
    let sent = fake.take_sent();
    assert!(
        sent.iter().any(|c| matches!(
            c,
            Command::SetTrackLoudness { track: t, trim_db, .. } if *t == track && *trim_db == 0.5
        )),
        "{sent:?}"
    );
}

#[test]
fn n18_the_gain_badge_is_painted_at_full_strength() {
    fn find(shape: &egui::Shape, out: &mut Vec<egui::Color32>) {
        match shape {
            egui::Shape::Text(t) if t.galley.text() == "+4.0 dB" => {
                out.push(t.galley.job.sections[0].format.color);
            }
            egui::Shape::Vec(v) => v.iter().for_each(|s| find(s, out)),
            _ => {}
        }
    }
    let (mut h, _fake) = harness(on_air());
    h.run_steps(2);
    let mut found = Vec::new();
    for clipped in &h.output().shapes {
        find(&clipped.shape, &mut found);
    }
    assert_eq!(found, vec![fp_app::ui::theme::ACCENT]);
}

#[test]
fn i4_the_popup_shows_the_automatic_gain_and_the_gain_with_trim() {
    let (mut h, fake) = harness(on_air());
    h.run_steps(2);
    open_popup(&mut h);
    assert!(
        h.query_by_label_contains("Automatic gain: +4.0 dB")
            .is_some()
    );
    assert!(h.query_by_label_contains("Gain with trim").is_none());
    fake.take_sent();
    h.get_by_role_and_label(Role::Slider, "Trim").focus();
    h.run_steps(1);
    h.key_press(egui::Key::ArrowRight);
    h.run_steps(3);
    assert!(
        h.query_by_label_contains("Automatic gain: +4.0 dB")
            .is_some()
    );
    assert!(
        h.query_by_label_contains("Gain with trim: +4.5 dB")
            .is_some()
    );
}

#[test]
fn i4_the_popup_says_why_there_is_no_automatic_gain() {
    let mut s = on_air();
    s.config.loudness.normalize = false;
    let (mut h, _fake) = harness(s);
    h.run_steps(2);
    open_popup(&mut h);
    assert!(
        h.query_by_label_contains("Automatic gain: none, normalization is off")
            .is_some()
    );
    assert!(
        h.query_by_label_contains("Normalization is off, so nothing changes")
            .is_some()
    );
}

#[test]
fn i4_the_cart_editor_shows_the_gain_lines() {
    let mut s = state(1, 0);
    s.config.loudness.normalize = true;
    let page = s.cartwall.pages[0].id;
    apply(
        &mut s,
        Command::AssignCartFile {
            page,
            index: 0,
            path: "/carts/a.flac".into(),
        },
    )
    .unwrap();
    let track = s.cartwall.pages[0].carts[0].track.unwrap();
    s.library.get_mut(track).unwrap().loudness = Some(LoudnessMeasurement {
        integrated_lufs: Some(-20.0),
        true_peak_dbtp: -10.0,
    });
    let (mut h, _fake) = harness(s);
    h.get_by_label("a").click_secondary();
    h.run_steps(2);
    h.get_by_label("Edit…").click();
    h.run_steps(3);
    assert!(
        h.query_by_label_contains("Automatic gain: +4.0 dB")
            .is_some()
    );
}

#[test]
fn i4_a_bit_perfect_player_shows_no_automatic_gain() {
    let mut s = on_air();
    bit_perfect(&mut s);
    let (mut h, _fake) = harness(s);
    h.run_steps(2);
    open_popup(&mut h);
    assert!(
        h.query_by_label_contains("No automatic gain: bit-perfect output plays tracks unaltered")
            .is_some()
    );
}

#[test]
fn i4_the_popup_says_a_sounding_track_changes_on_its_next_load() {
    let (mut h, _fake) = harness(on_air());
    h.run_steps(2);
    open_popup(&mut h);
    assert!(
        h.query_by_label_contains("a change applies from its next load")
            .is_some()
    );
    assert!(
        h.query_by_label_contains("Added to the automatic gain")
            .is_some()
    );
}
