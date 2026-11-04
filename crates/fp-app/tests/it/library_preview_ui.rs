#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::float_cmp
)]
//! Track library spec §8 (LB32–LB35): the preview bar of the library
//! window, which pre-listens the selected track on the library CUE.

use std::path::PathBuf;
use std::sync::Arc;

use crate::support;

use egui::accesskit::Role;
use egui::{Event, Modifiers, PointerButton, Pos2, Rect, pos2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use fp_app::services::{MediaCache, TrackMedia};
use fp_app::ui::app::AppUi;
use fp_backends::{AudioBackend, OfflineBackend};
use fp_model::library_view::LibraryView;
use fp_model::player::Transport;
use fp_model::{AppState, CategoryId, Command, FileState, MarkerKind, TrackId, TrackKind};
use support::{Fake, harness_sized, harness_with_media, state};

type H = Harness<'static, AppUi>;

const PLAY: &str = "Pre-listen";
const PAUSE: &str = "Pause pre-listen";
const STOP: &str = "Stop pre-listen";
const EMPTY: &str = "Select a track to pre-listen to it";
const NO_ROUTE: &str = "Pre-listen needs a library CUE output";
const OPEN_SETTINGS: &str = "Choose it in Settings › Audio outputs";
const NOT_AVAILABLE: &str = "The file cannot be played";
const WAVE: &str = "Library CUE waveform: click to seek";
/// The window's title bar and the preview bar under it.
const TITLE_BAR: f32 = 28.0;
const BAR: f32 = 112.0;

fn create(s: &mut AppState, name: &str) -> CategoryId {
    fp_model::apply(
        s,
        Command::CreateCategory {
            parent: None,
            name: name.into(),
            color: None,
            kind: TrackKind::Music,
        },
    )
    .unwrap();
    s.categories.iter().find(|c| c.name == name).unwrap().id
}

/// The library track titled `title`.
fn id_of(s: &AppState, title: &str) -> TrackId {
    s.library
        .ids()
        .into_iter()
        .find(|id| s.library.get(*id).unwrap().title == title)
        .unwrap()
}

/// One player with an empty playlist, three library tracks of 180 s
/// (Alpha, Bravo, Charlie; Bravo by Artist B in Jingles), the library CUE
/// routed when `routed`, and the library window open.
fn base(routed: bool) -> AppState {
    let mut s = state(1, 0);
    if routed {
        support::give_library_cue_route(&mut s);
    }
    let jingles = create(&mut s, "Jingles");
    fp_model::apply(
        &mut s,
        Command::AddToLibrary {
            paths: ["Alpha", "Bravo", "Charlie"]
                .iter()
                .map(|n| PathBuf::from(format!("/lib/{n}.mp3")))
                .collect(),
            category: None,
        },
    )
    .unwrap();
    for id in s.library.ids() {
        let t = s.library.get_mut(id).unwrap();
        t.duration_secs = 180.0;
        if t.title == "Bravo" {
            t.artist = "Artist B".into();
            t.category = Some(jingles);
        }
    }
    s.library_view = LibraryView {
        open: true,
        rect: Some([20.0, 60.0, 900.0, 600.0]),
        whole_library: true,
        ..LibraryView::default()
    };
    s
}

fn open(s: AppState) -> (H, Arc<Fake>) {
    let (mut h, fake) = harness_sized(s, vec2(1200.0, 800.0), |ui| ui);
    h.run_steps(2);
    fake.take_sent();
    (h, fake)
}

/// The preview bar's rectangle, under the window's title bar.
fn bar(h: &H) -> Rect {
    let w = h.state().library_rect().unwrap();
    Rect::from_min_size(pos2(w.left(), w.top() + TITLE_BAR), vec2(w.width(), BAR))
}

/// Whether a node labelled `label` is drawn inside the preview bar.
fn in_bar(h: &H, label: &str) -> bool {
    let b = bar(h).expand(1.0);
    h.query_all_by_label(label)
        .any(|n| b.contains_rect(n.rect()))
}

/// Clicks the row of `title` in the table (below the bar).
fn click_row(h: &mut H, title: &str) {
    let b = bar(h);
    let row = h
        .query_all_by_label(title)
        .find(|n| n.rect().top() > b.bottom())
        .expect("the row")
        .rect();
    h.event(Event::PointerMoved(row.center()));
    h.run_steps(1);
    press(h, row.center(), PointerButton::Primary);
    h.run_steps(2);
}

fn press(h: &mut H, at: Pos2, button: PointerButton) {
    for pressed in [true, false] {
        h.event(Event::PointerButton {
            pos: at,
            button,
            pressed,
            modifiers: Modifiers::NONE,
        });
    }
}

fn selects(sent: &[Command]) -> Vec<Option<TrackId>> {
    sent.iter()
        .filter_map(|c| match c {
            Command::SelectLibraryCue(t) => Some(*t),
            _ => None,
        })
        .collect()
}

/// The library CUE commands among `sent`.
fn preview_commands(sent: &[Command]) -> Vec<Command> {
    sent.iter()
        .filter(|c| {
            matches!(
                c,
                Command::SelectLibraryCue(_)
                    | Command::PlayLibraryCue
                    | Command::SetLibraryCuePaused(_)
                    | Command::StopLibraryCue
                    | Command::SeekLibraryCue(_)
            )
        })
        .cloned()
        .collect()
}

#[test]
fn selecting_a_row_sends_select_library_cue() {
    let s = base(true);
    let t2 = id_of(&s, "Bravo");
    let (mut h, fake) = open(s);
    click_row(&mut h, "Bravo");
    assert_eq!(selects(&fake.take_sent()), vec![Some(t2)]);
    h.run_steps(5);
    assert_eq!(selects(&fake.take_sent()), vec![], "sent once");
    assert_eq!(fake.state.load().library_cue.track, Some(t2));
}

#[test]
fn the_bar_plays_pauses_and_stops_the_selected_track() {
    let s = base(true);
    let t2 = id_of(&s, "Bravo");
    let (mut h, fake) = open(s);
    click_row(&mut h, "Bravo");
    fake.take_sent();
    h.get_by_label(PLAY).click();
    h.run_steps(2);
    assert_eq!(
        preview_commands(&fake.take_sent()),
        vec![Command::PlayLibraryCue]
    );
    let cue = fake.state.load().library_cue.clone();
    assert_eq!((cue.track, cue.transport), (Some(t2), Transport::Playing));
    h.get_by_label(PAUSE).click();
    h.run_steps(2);
    assert_eq!(
        preview_commands(&fake.take_sent()),
        vec![Command::SetLibraryCuePaused(true)]
    );
    assert_eq!(fake.state.load().library_cue.transport, Transport::Paused);
    h.get_by_label(STOP).click();
    h.run_steps(2);
    assert_eq!(
        preview_commands(&fake.take_sent()),
        vec![Command::StopLibraryCue]
    );
    assert_eq!(fake.state.load().library_cue.transport, Transport::Stopped);
}

#[test]
fn the_bar_shows_title_artist_category_and_the_remaining_time() {
    let (mut h, _fake) = open(base(true));
    assert!(in_bar(&h, EMPTY), "an empty bar says what to do");
    click_row(&mut h, "Bravo");
    for label in ["Bravo", "Artist B", "Jingles", "-03:00.0", "00:00 / 03:00"] {
        assert!(in_bar(&h, label), "{label}");
    }
    assert!(!in_bar(&h, EMPTY));
    // A track without a category reads Uncategorised.
    click_row(&mut h, "Alpha");
    assert!(in_bar(&h, "Alpha"));
    assert!(in_bar(&h, "Uncategorised"));
}

#[test]
fn a_click_on_the_waveform_seeks_the_preview() {
    let s = base(true);
    let t2 = id_of(&s, "Bravo");
    let media = MediaCache::default();
    media.seed(
        t2,
        TrackMedia {
            peaks: vec![
                fp_analysis::WavePeak {
                    min: -16_000,
                    max: 16_000,
                    rms: 8_000,
                };
                1800
            ],
            peak_bucket_secs: 0.1,
            cover_png: None,
        },
    );
    let (mut h, fake) = harness_with_media(s, media);
    h.run_steps(2);
    click_row(&mut h, "Bravo");
    h.get_by_label(PLAY).click();
    h.run_steps(2);
    fake.take_sent();
    let wave = h.get_by_label(WAVE).rect();
    // A drag across the waveform seeks nothing on its way.
    let from = pos2(wave.left() + wave.width() * 0.2, wave.center().y);
    h.event(Event::PointerMoved(from));
    h.run_steps(1);
    h.event(Event::PointerButton {
        pos: from,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(1);
    for step in 1..=10 {
        let x = from.x + wave.width() * 0.04 * step as f32;
        h.event(Event::PointerMoved(pos2(x, from.y)));
        h.run_steps(1);
    }
    h.event(Event::PointerButton {
        pos: pos2(from.x + wave.width() * 0.4, from.y),
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(2);
    let seeks = |sent: Vec<Command>| -> Vec<f64> {
        sent.into_iter()
            .filter_map(|c| match c {
                Command::SeekLibraryCue(secs) => Some(secs),
                _ => None,
            })
            .collect()
    };
    assert!(seeks(fake.take_sent()).len() <= 1, "never one per frame");
    // A click seeks once.
    let at = pos2(wave.center().x, wave.center().y);
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    press(&mut h, at, PointerButton::Primary);
    h.run_steps(2);
    let sent = seeks(fake.take_sent());
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert!(sent[0] > 0.0 && sent[0] < 180.0, "{sent:?}");
    assert!((sent[0] - 90.0).abs() < 3.0, "{sent:?}");
}

#[test]
fn the_waveform_menu_sets_a_marker_on_the_selected_track() {
    let s = base(true);
    let t2 = id_of(&s, "Bravo");
    let (mut h, fake) = open(s);
    click_row(&mut h, "Bravo");
    fake.take_sent();
    let wave = h.get_by_label(WAVE).rect();
    let at = pos2(wave.left() + wave.width() * 0.25, wave.center().y);
    h.event(Event::PointerMoved(at));
    h.run_steps(1);
    press(&mut h, at, PointerButton::Secondary);
    h.run_steps(2);
    h.get_by_label("Set cue in here").click();
    h.run_steps(2);
    let set: Vec<_> = fake
        .take_sent()
        .into_iter()
        .filter_map(|c| match c {
            Command::SetMarker { track, kind, secs } => Some((track, kind, secs)),
            _ => None,
        })
        .collect();
    assert_eq!(set.len(), 1, "{set:?}");
    assert_eq!((set[0].0, set[0].1), (t2, MarkerKind::CueIn));
    assert!((set[0].2.unwrap() - 45.0).abs() < 3.0, "{set:?}");
}

#[test]
fn without_a_route_the_bar_says_why_and_opens_settings_on_outputs() {
    let backend = OfflineBackend::new();
    backend.add_device("speakers", 2);
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let (mut h, fake) = harness_sized(base(false), vec2(1200.0, 800.0), |ui| {
        ui.with_backends(backends)
    });
    h.run_steps(2);
    click_row(&mut h, "Bravo");
    fake.take_sent();
    assert!(in_bar(&h, NO_ROUTE), "the bar says why");
    let play = h.get_by_label(PLAY);
    assert!(play.accesskit_node().is_disabled(), "Play is dimmed");
    // The reason is Play's tooltip.
    let before = h.query_all_by_label(NO_ROUTE).count();
    h.get_by_label(PLAY).hover();
    h.run_steps(40);
    assert_eq!(h.query_all_by_label(NO_ROUTE).count(), before + 1);
    // The bar still shows the track and its waveform.
    assert!(in_bar(&h, "Bravo"));
    assert!(h.query_by_label(WAVE).is_some());
    assert_eq!(
        h.query_all_by_role_and_label(Role::Label, "Library")
            .count(),
        0,
        "Settings is shut"
    );
    h.get_by_label(OPEN_SETTINGS).click();
    for _ in 0..200 {
        h.run_steps(1);
        if h.query_all_by_label("Test Main").next().is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(
        h.query_all_by_role_and_label(Role::Label, "Library")
            .count(),
        1,
        "Settings shows the library CUE row of Outputs"
    );
    assert_eq!(preview_commands(&fake.take_sent()), vec![]);
}

#[test]
fn a_missing_track_shows_not_available_and_play_is_dimmed() {
    let mut s = base(true);
    let t2 = id_of(&s, "Bravo");
    s.library.get_mut(t2).unwrap().file_state = FileState::Missing;
    let (mut h, fake) = open(s);
    click_row(&mut h, "Bravo");
    assert!(in_bar(&h, NOT_AVAILABLE));
    assert!(h.get_by_label(PLAY).accesskit_node().is_disabled());
    // The reason is Play's tooltip.
    let before = h.query_all_by_label(NOT_AVAILABLE).count();
    h.get_by_label(PLAY).hover();
    h.run_steps(40);
    assert_eq!(h.query_all_by_label(NOT_AVAILABLE).count(), before + 1);
    h.get_by_label(PLAY).click();
    h.run_steps(2);
    assert!(!fake.take_sent().contains(&Command::PlayLibraryCue));
}

#[test]
fn filtering_the_row_out_keeps_the_preview_playing() {
    let s = base(true);
    let t2 = id_of(&s, "Bravo");
    let (mut h, fake) = open(s);
    click_row(&mut h, "Bravo");
    h.get_by_label(PLAY).click();
    h.run_steps(2);
    fake.take_sent();
    h.get_by_label("Search the library").click();
    h.run_steps(1);
    h.get_by_label("Search the library").type_text("zzz");
    h.run_steps(5);
    let sent = fake.take_sent();
    assert!(!sent.contains(&Command::StopLibraryCue), "{sent:?}");
    // Selecting nothing leaves the running preview alone (LB34).
    assert!(
        selects(&sent).iter().all(Option::is_none),
        "no other track: {sent:?}"
    );
    let cue = fake.state.load().library_cue.clone();
    assert_eq!((cue.track, cue.transport), (Some(t2), Transport::Playing));
    assert!(in_bar(&h, "Bravo"), "the bar keeps showing it");
    assert!(h.query_by_label(STOP).is_some());
}

#[test]
fn a_minimised_window_draws_no_bar_and_sends_nothing() {
    let mut s = base(true);
    let t2 = id_of(&s, "Bravo");
    fp_model::apply(&mut s, Command::SelectLibraryCue(Some(t2))).unwrap();
    fp_model::apply(&mut s, Command::PlayLibraryCue).unwrap();
    s.library_view.minimised = true;
    let (mut h, fake) = harness_sized(s, vec2(1200.0, 800.0), |ui| ui);
    h.run_steps(5);
    assert_eq!(preview_commands(&fake.take_sent()), vec![]);
    for label in [PLAY, PAUSE, STOP, EMPTY, WAVE] {
        assert!(h.query_by_label(label).is_none(), "{label}");
    }
    assert_eq!(
        fake.state.load().library_cue.transport,
        Transport::Playing,
        "minimising keeps the preview (LB35)"
    );
}

#[test]
fn the_player_countdown_is_unchanged() {
    let mut s = state(1, 3);
    for id in s.library.ids() {
        s.library.get_mut(id).unwrap().duration_secs = 180.0;
    }
    let (h, _fake) = support::harness(s);
    let countdown = h.get_by_label("-03:00.0");
    assert_eq!(countdown.accesskit_node().role(), Role::Label);
}
