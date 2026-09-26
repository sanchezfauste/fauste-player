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
use egui_kittest::kittest::Queryable;
use fp_app::i18n::I18n;
use fp_app::services::MediaCache;
use fp_app::ui::app::AppUi;
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
    assert_eq!(h.query_all_by_label("Unknown artist").count(), 3);
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
fn opus_and_video_files_are_not_accepted() {
    use fp_app::ui::files::is_audio;
    use std::path::Path;
    for yes in [
        "a.wav", "a.aiff", "a.flac", "a.mp3", "a.ogg", "a.m4a", "a.AAC",
    ] {
        assert!(is_audio(Path::new(yes)), "{yes}");
    }
    for no in ["a.opus", "a.mkv", "a.webm", "a.mp4", "a.mka", "a.txt"] {
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
