#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! The main screen, driven headlessly (spec §8.3).

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use arc_swap::ArcSwap;
use egui::Key;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use fp_app::i18n::I18n;
use fp_app::services::MediaCache;
use fp_app::ui::app::AppUi;
use fp_app::ui::controller::Controller;
use fp_app::ui::files::audio_paths;
use fp_engine::conductor::Telemetry;
use fp_model::{AppState, Command, Config, EntryId, ModelError, PlayerId, Route};

struct Fake {
    state: ArcSwap<AppState>,
    telemetry: ArcSwap<Telemetry>,
    sent: Mutex<Vec<Command>>,
}

impl Fake {
    fn new(state: AppState) -> Arc<Self> {
        Arc::new(Self {
            state: ArcSwap::from_pointee(state),
            telemetry: ArcSwap::from_pointee(Telemetry::default()),
            sent: Mutex::new(Vec::new()),
        })
    }
}

impl Controller for Fake {
    fn model(&self) -> Arc<AppState> {
        self.state.load_full()
    }
    fn telemetry(&self) -> Arc<Telemetry> {
        self.telemetry.load_full()
    }
    fn send(&self, command: Command) -> bool {
        self.sent.lock().unwrap().push(command);
        true
    }
    fn test_tone(&self, _route: Route, _frequency_hz: f32) -> bool {
        true
    }
    fn take_rejection(&self) -> Option<ModelError> {
        None
    }
}

fn state(players: usize, tracks: usize) -> AppState {
    let mut config = Config::default();
    config.players.count = players;
    let mut state = AppState::new(config, "Main");
    let playlist = state.playlists.first_id().unwrap();
    let paths = (1..=tracks)
        .map(|n| PathBuf::from(format!("/music/Song {n}.mp3")))
        .collect();
    fp_model::apply(
        &mut state,
        Command::InsertPaths {
            playlist,
            index: 0,
            paths,
        },
    )
    .unwrap();
    state
}

fn harness(state: AppState) -> (Harness<'static, AppUi>, Arc<Fake>) {
    let fake = Fake::new(state);
    let ui = AppUi::new(
        fake.clone(),
        I18n::new(Some("en-US")),
        MediaCache::default(),
    );
    // Short frames, so that two clicks fall within the double-click delay.
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1000.0, 700.0))
        .with_step_dt(0.02)
        .build_ui_state(|ui, app: &mut AppUi| app.ui(ui), ui);
    harness.run_steps(2);
    (harness, fake)
}

fn sent(fake: &Fake) -> Vec<Command> {
    std::mem::take(&mut *fake.sent.lock().unwrap())
}

fn entries(fake: &Fake) -> Vec<EntryId> {
    let s = fake.state.load();
    s.playlists
        .iter()
        .next()
        .unwrap()
        .entries
        .iter()
        .map(|e| e.id)
        .collect()
}

fn player(fake: &Fake, n: usize) -> PlayerId {
    fake.state.load().players[n].id
}

#[test]
fn clicking_play_sends_play() {
    let (mut h, fake) = harness(state(2, 3));
    h.get_all_by_label("Play").next().unwrap().click();
    h.run_steps(2);
    assert!(sent(&fake).contains(&Command::Play(player(&fake, 0))));
}

#[test]
fn double_clicking_a_row_sets_next() {
    let (mut h, fake) = harness(state(1, 3));
    h.get_by_label("Song 3").click();
    h.step();
    h.get_by_label("Song 3").click();
    h.run_steps(2);
    let e = entries(&fake);
    assert!(sent(&fake).contains(&Command::SetNext(player(&fake, 0), e[2])));
}

#[test]
fn the_context_menu_removes_an_entry() {
    let (mut h, fake) = harness(state(1, 3));
    h.get_by_label("Song 2").click_secondary();
    h.run_steps(2);
    h.get_by_label("Remove from playlist").click();
    h.run_steps(2);
    let e = entries(&fake);
    assert!(sent(&fake).contains(&Command::RemoveEntry(e[1])));
}

#[test]
fn number_keys_play_players() {
    let (mut h, fake) = harness(state(2, 3));
    h.key_press(Key::Num2);
    h.run_steps(2);
    assert_eq!(sent(&fake), vec![Command::Play(player(&fake, 1))]);
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
