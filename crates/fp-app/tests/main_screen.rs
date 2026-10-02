#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The main screen, driven headlessly (spec §8.3).

mod support;

use egui::Key;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use fp_app::i18n::I18n;
use fp_app::services::MediaCache;
use fp_app::ui::app::AppUi;
use fp_app::ui::controller::Controller;
use fp_app::ui::files::audio_paths;
use fp_model::Command;
use support::{Fake, harness, state};

fn sent(fake: &Fake) -> Vec<Command> {
    fake.take_sent()
}

#[test]
fn clicking_play_sends_play() {
    let (mut h, fake) = harness(state(2, 3));
    h.get_all_by_label("Play").next().unwrap().click();
    h.run_steps(2);
    assert!(sent(&fake).contains(&Command::Play(fake.player(0))));
}

#[test]
fn double_clicking_a_row_sets_next() {
    let (mut h, fake) = harness(state(1, 3));
    h.get_by_label("Song 3").click();
    h.step();
    h.get_by_label("Song 3").click();
    h.run_steps(2);
    let e = fake.entries();
    assert!(sent(&fake).contains(&Command::SetNext(fake.player(0), e[2])));
}

#[test]
fn the_context_menu_removes_an_entry() {
    let (mut h, fake) = harness(state(1, 3));
    let e = fake.entries();
    h.get_by_label("Song 2").click_secondary();
    h.run_steps(2);
    h.get_by_label("Remove from playlist").click();
    h.run_steps(2);
    assert!(sent(&fake).contains(&Command::RemoveEntry(e[1])));
}

#[test]
fn number_keys_play_players() {
    let (mut h, fake) = harness(state(2, 3));
    h.key_press(Key::Num2);
    h.run_steps(2);
    assert_eq!(sent(&fake), vec![Command::Play(fake.player(1))]);
}

#[test]
fn shortcuts_are_ignored_while_editing_text() {
    let fake = Fake::new(state(2, 3));
    let app = AppUi::new(
        fake.clone(),
        I18n::new(Some("en-US")),
        MediaCache::default(),
    );
    let mut h = Harness::builder()
        .with_size(egui::vec2(1000.0, 700.0))
        .build_ui_state(
            |ui, (app, text): &mut (AppUi, String)| {
                ui.add(egui::TextEdit::singleline(text).id_salt("editor"));
                app.ui(ui);
            },
            (app, String::new()),
        );
    h.run_steps(2);
    h.get_by_role(egui::accesskit::Role::TextInput).focus();
    h.run_steps(1);
    h.key_press(Key::Num1);
    h.key_press(Key::Delete);
    h.run_steps(2);
    assert!(sent(&fake).is_empty());
}

#[test]
fn large_playlists_build_only_visible_rows() {
    let (mut h, _fake) = harness(state(1, 5_000));
    h.run_steps(1);
    let built = h.state().rows_built();
    assert!((1..100).contains(&built), "built {built} rows");
}

#[test]
fn dropped_paths_keep_only_audio_files() {
    let dir = tempfile::tempdir().unwrap();
    let folder = dir.path().join("folder");
    std::fs::create_dir_all(folder.join("nested")).unwrap();
    for name in ["b.flac", "a.MP3", "notes.txt", "nested/deep.wav"] {
        std::fs::write(folder.join(name), b"x").unwrap();
    }
    let song = dir.path().join("song.ogg");
    let image = dir.path().join("cover.jpg");
    std::fs::write(&song, b"x").unwrap();
    std::fs::write(&image, b"x").unwrap();
    let got = audio_paths(&[
        song.clone(),
        image,
        folder.clone(),
        dir.path().join("gone.mp3"),
    ]);
    assert_eq!(got, vec![song, folder.join("a.MP3"), folder.join("b.flac")]);
}

#[test]
fn tracks_without_an_artist_show_unknown_artist() {
    let (h, _fake) = harness(state(1, 3));
    // Three rows, and the stopped player's info row, which shows its next.
    assert_eq!(h.query_all_by_label("Unknown artist").count(), 4);
}

#[test]
fn holding_a_number_key_plays_only_once() {
    let (mut h, fake) = harness(state(1, 3));
    h.key_down(Key::Num1);
    h.step();
    h.key_down(Key::Num1);
    h.step();
    h.key_down(Key::Num1);
    h.run_steps(2);
    h.key_up(Key::Num1);
    h.run_steps(1);
    let plays = sent(&fake)
        .iter()
        .filter(|c| matches!(c, Command::Play(_)))
        .count();
    assert_eq!(plays, 1, "key repeats must not press Play again");
}

#[test]
fn one_wheel_notch_moves_the_fader_one_step() {
    let (mut h, fake) = harness(state(1, 1));
    let at = h.get_by_label_contains("Volume").rect().center();
    h.hover_at(at);
    h.step();
    h.event(egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Line,
        delta: egui::vec2(0.0, -1.0),
        phase: egui::TouchPhase::Move,
        modifiers: egui::Modifiers::NONE,
    });
    h.run_steps(20);
    let volume = fake.state.load().players[0].volume;
    let pos = fp_app::ui::view::fader_from_gain(volume);
    assert!((0.94..0.96).contains(&pos), "fader at {pos}");
}

#[derive(Debug)]
struct Dropped(std::path::PathBuf);

impl egui::DroppedFile for Dropped {
    fn path(&self) -> &std::path::Path {
        &self.0
    }
    fn bytes(&self) -> Result<Vec<u8>, String> {
        std::fs::read(&self.0).map_err(|e| e.to_string())
    }
}

fn drop_file(h: &mut egui_kittest::Harness<'static, AppUi>, path: &std::path::Path) {
    h.input_mut()
        .dropped_files
        .push(std::sync::Arc::new(Dropped(path.to_path_buf())));
    // Dropped paths are read on a helper thread: give it time to answer.
    for _ in 0..40 {
        h.run_steps(1);
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[test]
fn files_dropped_on_a_row_are_inserted_there() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("new.wav");
    std::fs::write(&file, b"x").unwrap();
    let (mut h, fake) = harness(state(1, 3));
    let row = h.get_by_label("Song 2").rect();
    h.hover_at(egui::pos2(row.center().x, row.top() + 3.0));
    h.step();
    drop_file(&mut h, &file);
    assert!(sent(&fake).iter().any(|c| matches!(
        c,
        Command::InsertPaths { index: 1, paths, .. } if paths == &vec![file.clone()]
    )));
}

#[test]
fn files_dropped_without_a_pointer_go_to_the_end_of_a_shown_list() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("new.wav");
    std::fs::write(&file, b"x").unwrap();
    let (mut h, fake) = harness(state(1, 3));
    h.remove_cursor();
    h.step();
    drop_file(&mut h, &file);
    assert!(
        sent(&fake)
            .iter()
            .any(|c| matches!(c, Command::InsertPaths { index: 3, .. }))
    );
}

#[test]
fn play_now_on_a_paused_player_starts_the_chosen_track() {
    let (mut h, fake) = harness(state(1, 3));
    let p = fake.player(0);
    let e = fake.entries();
    fp_app::ui::controller::Controller::send(fake.as_ref(), Command::Play(p));
    fp_app::ui::controller::Controller::send(fake.as_ref(), Command::Pause(p));
    h.run_steps(2);
    h.get_by_label("Song 3").click_secondary();
    h.run_steps(2);
    h.get_by_label("Play now").click();
    h.run_steps(2);
    let s = fake.state.load();
    let player = s.player(p).unwrap();
    assert_eq!(player.current, Some(e[2]));
    assert_eq!(player.transport, fp_model::Transport::Playing);
}

#[test]
fn delete_after_switching_tabs_does_not_remove_a_hidden_entry() {
    let mut s = state(1, 3);
    fp_model::apply(
        &mut s,
        Command::CreatePlaylist {
            name: "Other".into(),
        },
    )
    .unwrap();
    let (mut h, fake) = harness(s);
    h.get_by_label("Song 2").click();
    h.run_steps(2);
    h.get_by_label("Other").click();
    h.run_steps(2);
    h.key_press(Key::Delete);
    h.run_steps(2);
    assert!(
        !sent(&fake)
            .iter()
            .any(|c| matches!(c, Command::RemoveEntry(_)))
    );
}

#[test]
fn background_faults_show_an_alert() {
    let fake = support::Fake::new(state(1, 1));
    let faults = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(1));
    let app = AppUi::new(
        fake.clone(),
        I18n::new(Some("en-US")),
        MediaCache::default(),
    )
    .with_service_faults(faults);
    let mut h = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1000.0, 700.0))
        .build_ui_state(|ui, app: &mut AppUi| app.ui(ui), app);
    h.run_steps(3);
    assert!(h.query_by_label_contains("Background services").is_some());
}

#[test]
fn new_formats_are_accepted_as_audio_files() {
    use fp_app::ui::files::is_audio;
    use std::path::Path;
    for yes in [
        "a.wav", "a.aiff", "a.flac", "a.mp3", "a.ogg", "a.m4a", "a.AAC", "a.opus", "a.wv", "a.ape",
        "a.dsf", "a.DFF", "a.caf", "a.mka", "a.mp2", "a.weba",
    ] {
        assert!(is_audio(Path::new(yes)), "{yes}");
    }
    // Video containers stay out: a folder scan must not pick up films.
    for no in ["a.mkv", "a.webm", "a.mp4", "a.txt", "a.wvc"] {
        assert!(!is_audio(Path::new(no)), "{no}");
    }
}

#[test]
fn a_dropped_folder_inserts_its_audio_files() {
    let dir = tempfile::tempdir().unwrap();
    for name in ["b.flac", "a.mp3", "notes.txt"] {
        std::fs::write(dir.path().join(name), b"x").unwrap();
    }
    let (mut h, fake) = harness(state(1, 1));
    h.remove_cursor();
    h.step();
    drop_file(&mut h, dir.path());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    let mut inserted = None;
    while inserted.is_none() && std::time::Instant::now() < deadline {
        h.run_steps(1);
        inserted = sent(&fake).into_iter().find_map(|c| match c {
            Command::InsertPaths { paths, .. } => Some(paths),
            _ => None,
        });
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(
        inserted,
        Some(vec![dir.path().join("a.mp3"), dir.path().join("b.flac")])
    );
}

#[test]
fn function_keys_fire_carts_of_the_page_shown() {
    let (mut h, fake) = harness(state(1, 1));
    h.key_press(Key::F2);
    h.run_steps(2);
    let second = fake.state.load().cartwall.pages[0].carts[1].id;
    assert_eq!(sent(&fake), vec![Command::FireCart(second)]);
}

#[test]
fn ctrl_space_stops_all_carts() {
    let (mut h, fake) = harness(state(1, 1));
    h.key_press_modifiers(egui::Modifiers::CTRL, Key::Space);
    h.run_steps(2);
    assert_eq!(sent(&fake), vec![Command::StopAllCarts]);
}

#[test]
fn stop_all_follows_a_rebound_shortcut() {
    let mut s = state(1, 1);
    fp_model::apply(
        &mut s,
        Command::SetShortcut {
            action: fp_model::ShortcutAction::StopAllCarts,
            chord: Some(fp_model::KeyChord::key("Q")),
        },
    )
    .unwrap();
    let (mut h, fake) = harness(s);
    h.key_press(Key::Q);
    h.run_steps(2);
    assert_eq!(sent(&fake), vec![Command::StopAllCarts]);
    h.key_press_modifiers(egui::Modifiers::CTRL, Key::Space);
    h.run_steps(2);
    assert!(sent(&fake).is_empty(), "Ctrl+Space lost its binding");
}

#[test]
fn an_unbound_stop_all_shortcut_does_nothing() {
    let mut s = state(1, 1);
    fp_model::apply(
        &mut s,
        Command::SetShortcut {
            action: fp_model::ShortcutAction::StopAllCarts,
            chord: None,
        },
    )
    .unwrap();
    let (mut h, fake) = harness(s);
    h.key_press_modifiers(egui::Modifiers::CTRL, Key::Space);
    h.run_steps(2);
    assert!(sent(&fake).is_empty());
}

#[test]
fn a_rebound_key_plays_the_new_target() {
    let mut s = state(2, 1);
    fp_model::apply(
        &mut s,
        Command::SetShortcut {
            action: fp_model::ShortcutAction::PlayPlayer(2),
            chord: Some(fp_model::KeyChord::key("Q")),
        },
    )
    .unwrap();
    let (mut h, fake) = harness(s);
    h.key_press(Key::Q);
    h.run_steps(2);
    assert_eq!(sent(&fake), vec![Command::Play(fake.player(1))]);
    h.key_press(Key::Num2);
    h.run_steps(2);
    assert!(
        sent(&fake).is_empty(),
        "2 lost its binding when it moved to Q"
    );
}

#[test]
fn the_bp_badge_follows_telemetry() {
    let (mut h, fake) = harness(state(1, 1));
    assert!(h.query_by_label("Bit-perfect: off").is_some());
    let player = fake.player(0);
    fake.telemetry
        .store(std::sync::Arc::new(fp_engine::conductor::Telemetry {
            players: vec![(
                player,
                fp_engine::engine::PlayerTelemetry {
                    bit_perfect: true,
                    ..Default::default()
                },
            )],
            ..Default::default()
        }));
    h.run_steps(2);
    assert!(h.query_by_label("Bit-perfect: on").is_some());
}

#[test]
fn a_shortcut_key_does_not_also_press_the_focused_button() {
    let mut s = state(1, 1);
    fp_model::apply(
        &mut s,
        Command::SetShortcut {
            action: fp_model::ShortcutAction::PausePlayer(1),
            chord: Some(fp_model::KeyChord::key("Space")),
        },
    )
    .unwrap();
    // Pause is only available while the player plays (R28).
    let playing = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(playing)).unwrap();
    let p = s.players[0].id;
    let (mut h, fake) = harness(s);
    h.get_by_label("Stop").focus();
    h.run_steps(1);
    fake.take_sent();
    h.key_press(Key::Space);
    h.run_steps(2);
    assert_eq!(sent(&fake), vec![Command::Pause(p)], "the shortcut only");
    // Holding the key: the first press fires, its repeats reach nothing.
    let space = |pressed: bool| egui::Event::Key {
        key: Key::Space,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    };
    h.event(space(true));
    h.run_steps(1);
    assert_eq!(sent(&fake), vec![Command::Pause(p)]);
    for _ in 0..3 {
        h.event(space(true)); // egui marks it a repeat: the key is down
        h.run_steps(1);
    }
    h.event(space(false));
    h.run_steps(2);
    assert_eq!(sent(&fake), vec![], "the repeats press nothing");
}

#[test]
fn a_shortcut_on_tab_does_not_move_the_focus() {
    let mut s = state(1, 1);
    fp_model::apply(
        &mut s,
        Command::SetShortcut {
            action: fp_model::ShortcutAction::PausePlayer(1),
            chord: Some(fp_model::KeyChord::key("Tab")),
        },
    )
    .unwrap();
    // Pause is only available while the player plays (R28).
    let playing = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(playing)).unwrap();
    let (mut h, fake) = harness(s);
    h.get_by_label("Stop").focus();
    h.run_steps(1);
    let before = h.ctx.memory(|m| m.focused());
    assert!(before.is_some());
    fake.take_sent();
    h.key_press(Key::Tab);
    h.run_steps(2);
    assert_eq!(h.ctx.memory(|m| m.focused()), before, "the focus stays");
    assert_eq!(sent(&fake).len(), 1, "the shortcut fired");
}

#[test]
fn single_and_cont_are_one_joined_control() {
    let (mut h, fake) = harness(state(1, 3));
    let single = h.get_by_label("Stop after every track").rect();
    let cont = h
        .get_by_label("Continuous: chain tracks at the mix point")
        .rect();
    // Side by side with no gap, SINGLE first.
    assert!(
        (single.right() - cont.left()).abs() < 0.5,
        "{single:?} {cont:?}"
    );
    assert_eq!(single.top(), cont.top());
    // Continuous is the default: clicking it does nothing, SINGLE switches.
    h.get_by_label("Continuous: chain tracks at the mix point")
        .click();
    h.run_steps(2);
    assert!(sent(&fake).is_empty());
    h.get_by_label("Stop after every track").click();
    h.run_steps(2);
    assert_eq!(
        sent(&fake),
        vec![Command::SetMode(fake.player(0), fp_model::PlayMode::Single)]
    );
}

#[test]
fn the_meter_and_fader_form_a_column_right_of_the_transport() {
    let (h, _) = harness(quiet_meter(state(1, 3)));
    let play = h.get_by_label("Play").rect();
    let meter = h.get_by_label("Level meter").rect();
    let fader = h
        .query_all_by_label_contains("Volume")
        .next()
        .unwrap()
        .rect();
    let wave = h.get_by_label("Waveform: click to seek").rect();
    let countdown = h.get_by_label("-00:00.0").rect();
    // The column spans the info row and the transport row.
    assert!(meter.top() < play.top() - 40.0, "{meter:?} {play:?}");
    assert!(
        (meter.bottom() - play.bottom()).abs() <= 1.0,
        "{meter:?} {play:?}"
    );
    assert!(meter.left() > countdown.right(), "{meter:?} {countdown:?}");
    assert!(fader.left() >= meter.right());
    assert!((fader.bottom() - meter.bottom()).abs() <= 1.0);
    // The waveform keeps the player's full width.
    assert!(wave.right() >= fader.right() - 1.0, "{wave:?} {fader:?}");
    // Elapsed / total sits under the waveform, right-aligned.
    let time = h.get_by_label("00:00 / 00:00").rect();
    assert!(time.top() >= wave.bottom(), "{time:?} {wave:?}");
    assert!(
        (time.right() - wave.right()).abs() <= 1.0,
        "{time:?} {wave:?}"
    );
}

#[test]
fn at_the_minimum_player_width_the_countdown_still_fits() {
    let (h, _) =
        support::harness_sized(quiet_meter(state(1, 3)), egui::vec2(380.0, 700.0), |ui| ui);
    let countdown = h.get_by_label("-00:00.0").rect();
    let meter = h.get_by_label("Level meter").rect();
    let grid = h.get_by_label("Stop after the current track").rect();
    assert!(countdown.right() <= meter.left(), "{countdown:?} {meter:?}");
    assert!(countdown.left() >= grid.right(), "{countdown:?} {grid:?}");
}

#[test]
fn an_hour_long_countdown_fits_between_the_grid_and_the_meter() {
    let mut state = quiet_meter(state(1, 3));
    for track in state.library.iter_mut() {
        track.duration_secs = 3700.0;
    }
    let p = state.players[0].id;
    fp_model::apply(&mut state, Command::Play(p)).unwrap();
    let (h, _) = support::harness_sized(state, egui::vec2(380.0, 700.0), |ui| ui);
    let countdown = h.get_by_label_contains("-1:01:40").rect();
    let meter = h.get_by_label("Level meter").rect();
    let grid = h.get_by_label("Stop after the current track").rect();
    assert!(countdown.right() <= meter.left(), "{countdown:?} {meter:?}");
    assert!(countdown.left() >= grid.right(), "{countdown:?} {grid:?}");
}

/// The state with the loudness line off, so the meter is named "Level meter".
fn quiet_meter(mut state: fp_model::AppState) -> fp_model::AppState {
    state.config.meter.loudness = fp_model::LoudnessReadout::Off;
    state
}

/// `state` with `action` bound to `key`.
fn bound(
    mut s: fp_model::AppState,
    action: fp_model::ShortcutAction,
    key: &str,
) -> fp_model::AppState {
    fp_model::apply(
        &mut s,
        Command::SetShortcut {
            action,
            chord: Some(fp_model::KeyChord::key(key)),
        },
    )
    .unwrap();
    s
}

#[test]
fn a_restart_shortcut_restarts_a_playing_player() {
    let mut s = bound(state(1, 3), fp_model::ShortcutAction::RestartPlayer(1), "R");
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    let (mut h, fake) = harness(s);
    h.key_press(Key::R);
    h.run_steps(2);
    assert_eq!(sent(&fake), vec![Command::Restart(p)]);
}

#[test]
fn shortcuts_for_unavailable_actions_do_nothing() {
    let s = bound(state(1, 3), fp_model::ShortcutAction::RestartPlayer(1), "R");
    let s = bound(s, fp_model::ShortcutAction::PreviousPlayer(1), "P");
    let s = bound(s, fp_model::ShortcutAction::StopPlayer(1), "S");
    let (mut h, fake) = harness(s);
    for key in [Key::R, Key::P, Key::S] {
        h.key_press(key);
        h.run_steps(2);
    }
    assert!(
        sent(&fake).is_empty(),
        "a stopped player cannot restart, go back or stop"
    );
}

#[test]
fn the_grid_has_previous_and_restart_in_its_first_column() {
    let (h, _) = harness(state(1, 3));
    let previous = h.get_by_label("Previous track").rect();
    let restart = h.get_by_label("Restart the track").rect();
    let stop = h.get_by_label("Stop").rect();
    let fade = h.get_by_label("Fade stop").rect();
    assert!(previous.right() < stop.left(), "{previous:?} {stop:?}");
    assert!((previous.top() - stop.top()).abs() < 0.5);
    assert!(restart.right() < fade.left(), "{restart:?} {fade:?}");
    assert!((restart.top() - fade.top()).abs() < 0.5);
    assert!(restart.top() > previous.bottom());
}

#[test]
fn unavailable_buttons_are_dimmed_and_inert() {
    let (mut h, fake) = harness(state(1, 3));
    for label in [
        "Previous track",
        "Restart the track",
        "Stop",
        "Pause",
        "Fade stop",
    ] {
        let node = h.get_by_label(label);
        assert!(node.accesskit_node().is_disabled(), "{label}");
        node.click();
        h.run_steps(2);
    }
    assert!(sent(&fake).is_empty());
    assert!(!h.get_by_label("Play").accesskit_node().is_disabled());
}

#[test]
fn restart_and_previous_send_their_commands_when_available() {
    let mut s = state(1, 3);
    let p = s.players[0].id;
    for _ in 0..2 {
        fp_model::apply(&mut s, Command::Play(p)).unwrap();
        fp_model::on_event(&mut s, fp_model::EngineEvent::FadeCompleted { player: p });
    }
    let (mut h, fake) = harness(s);
    h.get_by_label("Previous track").click();
    h.run_steps(2);
    h.get_by_label("Restart the track").click();
    h.run_steps(2);
    assert_eq!(sent(&fake), vec![Command::Previous(p), Command::Restart(p)]);
}

#[test]
fn the_row_menu_toggles_repeat_and_stop_after() {
    let (mut h, fake) = harness(state(1, 3));
    let e = fake.entries();
    h.get_by_label("Song 2").click_secondary();
    h.run_steps(2);
    h.get_by_label("Repeat this track").click();
    h.run_steps(2);
    h.get_by_label("Song 2").click_secondary();
    h.run_steps(2);
    h.get_by_label("Stop after this track").click();
    h.run_steps(2);
    assert_eq!(
        sent(&fake),
        vec![
            Command::ToggleEntryRepeat(e[1]),
            Command::ToggleEntryStopAfter(e[1])
        ]
    );
}

#[test]
fn flagged_entries_show_their_icons() {
    let mut s = state(1, 3);
    let e: Vec<_> = s
        .playlists
        .iter()
        .next()
        .unwrap()
        .entries
        .iter()
        .map(|x| x.id)
        .collect();
    fp_model::apply(&mut s, Command::ToggleEntryRepeat(e[0])).unwrap();
    fp_model::apply(&mut s, Command::ToggleEntryStopAfter(e[2])).unwrap();
    let (h, _) = harness(s);
    assert_eq!(h.query_all_by_label("Repeats").count(), 1);
    assert_eq!(h.query_all_by_label("Stops after").count(), 1);
    let repeat = h.get_by_label("Repeats").rect();
    assert!(
        h.query_all_by_label("Song 1").any(|n| {
            let song = n.rect();
            repeat.left() > song.left() && (repeat.center().y - song.center().y).abs() < 4.0
        }),
        "the icon sits in the row of its entry"
    );
}

fn shares(w: [f32; 4]) -> [f32; 4] {
    let sum: f32 = w.iter().sum();
    w.map(|x| x / sum)
}

#[test]
fn columns_fill_the_table_and_keep_their_shares_when_the_window_grows() {
    let (mut h, fake) = harness(state(1, 3));
    let p = fake.player(0);
    let before = h.state().column_widths(p).unwrap();
    assert!(
        before[1] > before[2],
        "Title gets the most room: {before:?}"
    );
    h.set_size(egui::vec2(1400.0, 700.0));
    h.run_steps(3);
    let after = h.state().column_widths(p).unwrap();
    let (a, b) = (shares(before), shares(after));
    for i in 1..3 {
        assert!((a[i] - b[i]).abs() < 0.03, "{before:?} → {after:?}");
    }
    assert!(after.iter().sum::<f32>() > before.iter().sum::<f32>() + 300.0);
}

#[test]
fn stored_fractions_are_applied() {
    let mut s = state(1, 3);
    let p = s.players[0].id;
    fp_model::apply(
        &mut s,
        Command::SetColumnWidths(
            p,
            fp_model::ColumnWidths {
                fractions: Some([0.1, 0.3, 0.5, 0.1]),
            },
        ),
    )
    .unwrap();
    let (h, _) = harness(s);
    let w = shares(h.state().column_widths(p).unwrap());
    assert!(w[2] > w[1], "Artist wider than Title as stored: {w:?}");
}

#[test]
fn resizing_the_window_does_not_store_column_widths() {
    let (mut h, fake) = harness(state(1, 3));
    fake.take_sent();
    h.set_size(egui::vec2(1400.0, 700.0));
    h.run_steps(3);
    h.set_size(egui::vec2(1200.0, 700.0));
    h.run_steps(3);
    assert!(
        !sent(&fake)
            .iter()
            .any(|c| matches!(c, Command::SetColumnWidths(..))),
        "only a handle release stores widths"
    );
}

/// A programme recording longer than ten hours still fits its countdown
/// at the minimum player width (plan 4, item 15).
#[test]
fn a_ten_hour_countdown_fits_between_the_grid_and_the_meter() {
    let mut state = quiet_meter(state(1, 3));
    for track in state.library.iter_mut() {
        track.duration_secs = 36_100.0;
    }
    let p = state.players[0].id;
    fp_model::apply(&mut state, Command::Play(p)).unwrap();
    let (h, _) = support::harness_sized(state, egui::vec2(380.0, 700.0), |ui| ui);
    let countdown = h.get_by_label_contains("-10:01:40").rect();
    let meter = h.get_by_label("Level meter").rect();
    let grid = h.get_by_label("Stop after the current track").rect();
    assert!(countdown.right() <= meter.left(), "{countdown:?} {meter:?}");
    assert!(countdown.left() >= grid.right(), "{countdown:?} {grid:?}");
    assert!(h.query_by_label("00:00 / 10:01:40").is_some());
}

#[test]
fn hovering_an_unavailable_row_says_why() {
    let mut s = state(1, 3);
    let playlist = s.playlists.first_id().unwrap();
    let track = s.playlists.get(playlist).unwrap().entries[1].track;
    fp_model::apply(
        &mut s,
        Command::SetFileState {
            track,
            state: fp_model::FileState::Unreadable,
        },
    )
    .unwrap();
    let (mut h, _fake) = harness(s);
    h.get_by_label_contains(egui_phosphor::regular::WARNING)
        .hover();
    h.run_steps(40);
    assert!(
        h.query_by_label_contains("Cannot read the file: /music/Song 2.mp3")
            .is_some(),
        "the tooltip gives the reason and the path"
    );
}

#[test]
fn a_missing_file_and_an_unreadable_one_have_their_own_icons() {
    use egui_phosphor::regular::{FILE_X, WARNING};
    let mut s = state(1, 3);
    let playlist = s.playlists.first_id().unwrap();
    let entries = s.playlists.get(playlist).unwrap().entries.clone();
    for (entry, file_state) in [
        (&entries[1], fp_model::FileState::Missing),
        (&entries[2], fp_model::FileState::Unreadable),
    ] {
        fp_model::apply(
            &mut s,
            Command::SetFileState {
                track: entry.track,
                state: file_state,
            },
        )
        .unwrap();
    }
    let (h, _fake) = harness(s);
    assert!(h.query_by_label(&format!("{FILE_X}02")).is_some());
    assert!(h.query_by_label(&format!("{WARNING}03")).is_some());
}

#[test]
fn a_track_an_earlier_version_analysed_shows_the_reload_flag() {
    let mut s = state(1, 2);
    let playlist = s.playlists.first_id().unwrap();
    let track = s.playlists.get(playlist).unwrap().entries[1].track;
    // No format and version 0: an earlier version's analysis.
    fp_model::apply(
        &mut s,
        Command::ApplyAnalysis {
            track,
            analysis: Box::default(),
        },
    )
    .unwrap();
    let (h, _fake) = harness(s);
    assert_eq!(
        h.query_all_by_label_contains("Analysed by an earlier version")
            .count(),
        1
    );
}

#[test]
fn clicking_a_row_moves_a_running_cue_to_it() {
    let (mut h, fake) = harness(state(1, 3));
    let p = fake.player(0);
    let e = fake.entries();
    fake.send(Command::ToggleCue(p));
    fake.take_sent();
    h.run_steps(2);
    h.get_by_label("Song 3").click();
    h.run_steps(2);
    assert!(sent(&fake).contains(&Command::CueEntry(p, e[2])));
}

#[test]
fn clicking_the_cued_row_a_missing_file_or_without_a_cue_sends_nothing() {
    let (mut h, fake) = harness(state(1, 3));
    let p = fake.player(0);
    let e = fake.entries();
    // No CUE: a click only selects.
    h.get_by_label("Song 2").click();
    h.run_steps(2);
    assert!(
        !sent(&fake)
            .iter()
            .any(|c| matches!(c, Command::CueEntry(..)))
    );
    // The cued row (the next, Song 1): nothing to move.
    fake.send(Command::ToggleCue(p));
    fake.take_sent();
    h.run_steps(2);
    // The player column and the CUE window show the title too: the table
    // row is the match lowest on the screen.
    let row = h
        .get_all_by_label("Song 1")
        .max_by(|a, b| a.rect().min.y.total_cmp(&b.rect().min.y))
        .unwrap();
    row.click();
    h.run_steps(2);
    assert!(
        !sent(&fake)
            .iter()
            .any(|c| matches!(c, Command::CueEntry(..)))
    );
    // A missing file.
    let track = fake.state.load().playlists.entry(e[2]).unwrap().track;
    fake.send(Command::SetFileState {
        track,
        state: fp_model::FileState::Missing,
    });
    fake.take_sent();
    h.run_steps(2);
    h.get_by_label("Song 3").click();
    h.run_steps(2);
    assert!(
        !sent(&fake)
            .iter()
            .any(|c| matches!(c, Command::CueEntry(..)))
    );
}

#[test]
fn a_double_click_moves_the_cue_once() {
    let (mut h, fake) = harness(state(1, 3));
    let p = fake.player(0);
    let e = fake.entries();
    fake.send(Command::ToggleCue(p));
    fake.take_sent();
    h.run_steps(2);
    h.get_by_label("Song 3").click();
    h.step();
    h.get_by_label("Song 3").click();
    h.run_steps(2);
    let commands = sent(&fake);
    assert!(commands.contains(&Command::SetNext(p, e[2])));
    let moves = commands
        .iter()
        .filter(|c| matches!(c, Command::CueEntry(..)))
        .count();
    assert_eq!(moves, 1, "{commands:?}");
    assert_eq!(fake.state.load().players[0].cue.unwrap().entry, e[2]);
}
