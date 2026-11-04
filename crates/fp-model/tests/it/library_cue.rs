#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Library plan 4 (LB32): the library CUE, a holder of its own.

use crate::common::fixture;
use fp_model::library_cue::LibraryCue;
use fp_model::{
    AppState, BusyCause, Config, EngineEvent, Holder, LibraryRoutes, OutputDevice, Route, TrackId,
    Transport, causes, configured_route, holders, on_event,
};

fn route(device: &str, first_channel: u16) -> Route {
    Route {
        backend: "null".to_owned(),
        device: device.to_owned(),
        first_channel,
    }
}

#[test]
fn lb32_the_library_cue_is_the_last_holder() {
    let s = fixture(1);
    assert_eq!(holders(&s).last(), Some(&Holder::LibraryCue));
}

#[test]
fn lb32_its_route_is_outputs_library_cue() {
    let mut config = Config::default();
    assert_eq!(configured_route(&config, Holder::LibraryCue), None);
    assert!(!config.outputs.library_has_cue());
    config.outputs.library.cue = Some(route("lib", 2));
    assert_eq!(
        configured_route(&config, Holder::LibraryCue),
        Some(route("lib", 2))
    );
    assert!(config.outputs.library_has_cue());
}

#[test]
fn lb32_a_running_preview_is_a_cause() {
    let mut s = fixture(1);
    assert!(causes(&s, Holder::LibraryCue).is_empty());
    for transport in [Transport::Playing, Transport::Paused] {
        s.library_cue.transport = transport;
        assert_eq!(
            causes(&s, Holder::LibraryCue),
            vec![BusyCause::LibraryCue],
            "{transport:?}"
        );
    }
    s.library_cue.transport = Transport::Stopped;
    assert!(causes(&s, Holder::LibraryCue).is_empty());
}

#[test]
fn lb32_routed_devices_and_cue_shares_main_count_the_library_cue() {
    let mut s = fixture(1);
    let main = route("desk", 0);
    s.config.outputs.routes[0].main = Some(main);
    s.config.outputs.library.cue = Some(route("desk", 1));
    assert!(s.config.outputs.cue_shares_main(Holder::LibraryCue));
    s.config.outputs.library.cue = Some(route("monitor", 0));
    assert!(!s.config.outputs.cue_shares_main(Holder::LibraryCue));
    assert!(s.config.outputs.routed_devices().contains(&OutputDevice {
        backend: "null".to_owned(),
        device: "monitor".to_owned(),
    }));
}

#[test]
fn lb32_the_config_round_trips() {
    let mut config = Config::default();
    config.outputs.library.cue = Some(route("lib", 2));
    let json = serde_json::to_string(&config).unwrap();
    let back: Config = serde_json::from_str(&json).unwrap();
    assert_eq!(back, config);

    let mut value = serde_json::to_value(Config::default()).unwrap();
    value["outputs"].as_object_mut().unwrap().remove("library");
    let old: Config = serde_json::from_value(value).unwrap();
    assert_eq!(old.outputs.library.cue, None);
}

#[test]
fn rule10_a_fresh_state_has_the_library_cue_stopped() {
    let s = AppState::new(Config::default(), "Main");
    assert_eq!(s.library_cue, LibraryCue::default());
    assert_eq!(s.library_cue.transport, Transport::Stopped);
    assert_eq!(s.library_cue.track, None);
}

#[test]
fn lb36_the_engines_end_stops_the_preview() {
    let (a, b) = (self::a(), self::b());
    let mut s = fixture(2);
    s.library_cue = LibraryCue {
        track: Some(a),
        transport: Transport::Playing,
        start_at: Some(3.0),
    };
    let before = s.library_cue.clone();
    let out = on_event(&mut s, EngineEvent::LibraryCueEnded { track: b });
    assert!(out.is_empty(), "{out:?}");
    assert_eq!(s.library_cue, before, "another track's end changes nothing");
    let out = on_event(&mut s, EngineEvent::LibraryCueEnded { track: a });
    assert!(out.is_empty(), "{out:?}");
    assert_eq!(
        s.library_cue,
        LibraryCue {
            track: Some(a),
            transport: Transport::Stopped,
            start_at: None,
        }
    );
}

// ---- Plan 4 Task 4: the preview's rules (LB32-LB36) ----

use fp_model::{
    Command, EngineAction, FileState, LibraryView, Marker, MarkerSource, Normalization, TrackKind,
    apply, command_available, normalization_for,
};

/// The two library tracks of `fixture(2)`; its ids are deterministic.
fn a() -> TrackId {
    fixture(2).library.ids()[0]
}

fn b() -> TrackId {
    fixture(2).library.ids()[1]
}

/// A library route and two analysed tracks with cue-ins at 1.5 s and 2.0 s,
/// the bar on `a`, stopped.
fn bar() -> AppState {
    let mut s = fixture(2);
    s.config.outputs.library.cue = Some(route("lib", 0));
    for (id, cue_in) in [(a(), 1.5), (b(), 2.0)] {
        let t = s.library.get_mut(id).unwrap();
        t.markers.cue_in = Some(Marker {
            secs: cue_in,
            source: MarkerSource::Manual,
        });
    }
    s.config.players.use_cue_markers = true;
    apply(&mut s, Command::SelectLibraryCue(Some(a()))).unwrap();
    s
}

fn run(s: &mut AppState, c: Command) -> Vec<EngineAction> {
    apply(s, c).unwrap()
}

fn started(out: &[EngineAction]) -> (TrackId, f64, bool) {
    match out {
        [
            EngineAction::StartLibraryCue {
                request, paused, ..
            },
        ] => (request.track, request.from_secs, *paused),
        other => panic!("not a single start: {other:?}"),
    }
}

#[test]
fn lb32_without_a_route_play_does_nothing() {
    let mut s = bar();
    s.config.outputs.library.cue = None;
    let out = run(&mut s, Command::PlayLibraryCue);
    assert!(out.is_empty(), "{out:?}");
    assert_eq!(s.library_cue.transport, Transport::Stopped);
}

#[test]
fn lb32_a_removed_route_refuses_play_but_keeps_a_running_preview() {
    let mut s = bar();
    run(&mut s, Command::PlayLibraryCue);
    let mut config = s.config.clone();
    config.outputs.library.cue = None;
    let out = run(&mut s, Command::UpdateConfig(Box::new(config)));
    assert!(!out.contains(&EngineAction::StopLibraryCue), "{out:?}");
    assert_eq!(s.library_cue.transport, Transport::Playing);
    assert!(command_available(&s, &Command::StopLibraryCue));

    run(&mut s, Command::SetLibraryCuePaused(true));
    assert!(command_available(&s, &Command::PlayLibraryCue), "release");
    run(&mut s, Command::StopLibraryCue);
    assert!(!command_available(&s, &Command::PlayLibraryCue));
    assert!(command_available(&s, &Command::StopLibraryCue));
}

#[test]
fn lb32_play_needs_a_playable_track() {
    let mut s = bar();
    assert!(command_available(&s, &Command::PlayLibraryCue));
    s.library_cue.track = None;
    assert!(!command_available(&s, &Command::PlayLibraryCue));
    s.library_cue.track = Some(a());
    s.library.get_mut(a()).unwrap().file_state = FileState::Missing;
    assert!(!command_available(&s, &Command::PlayLibraryCue));
}

#[test]
fn lb33_play_starts_the_selected_track_from_its_cue_in() {
    let mut s = bar();
    let out = run(&mut s, Command::PlayLibraryCue);
    assert_eq!(started(&out), (a(), 1.5, false));
    assert_eq!(s.library_cue.transport, Transport::Playing);
}

#[test]
fn lb33_play_while_playing_does_nothing() {
    let mut s = bar();
    run(&mut s, Command::PlayLibraryCue);
    let out = run(&mut s, Command::PlayLibraryCue);
    assert!(out.is_empty(), "{out:?}");
    assert_eq!(s.library_cue.transport, Transport::Playing);
}

#[test]
fn lb33_pause_holds_and_play_releases() {
    let mut s = bar();
    run(&mut s, Command::PlayLibraryCue);
    let out = run(&mut s, Command::SetLibraryCuePaused(true));
    assert_eq!(
        out,
        vec![EngineAction::SetLibraryCuePaused { paused: true }]
    );
    assert_eq!(s.library_cue.transport, Transport::Paused);
    let out = run(&mut s, Command::SetLibraryCuePaused(true));
    assert!(out.is_empty(), "{out:?}");
    let out = run(&mut s, Command::PlayLibraryCue);
    assert_eq!(
        out,
        vec![EngineAction::SetLibraryCuePaused { paused: false }]
    );
    assert_eq!(s.library_cue.transport, Transport::Playing);
}

#[test]
fn lb33_stop_ends_it() {
    let mut s = bar();
    run(&mut s, Command::PlayLibraryCue);
    let out = run(&mut s, Command::StopLibraryCue);
    assert_eq!(out, vec![EngineAction::StopLibraryCue]);
    assert_eq!(s.library_cue.transport, Transport::Stopped);
    let out = run(&mut s, Command::StopLibraryCue);
    assert!(out.is_empty(), "{out:?}");
}

#[test]
fn lb33_a_seek_moves_a_running_preview() {
    let mut s = bar();
    run(&mut s, Command::PlayLibraryCue);
    let out = run(&mut s, Command::SeekLibraryCue(30.0));
    assert_eq!(out, vec![EngineAction::SeekLibraryCue { secs: 30.0 }]);
    assert_eq!(s.library_cue.transport, Transport::Playing);
}

#[test]
fn lb33_a_seek_while_stopped_sets_where_play_starts() {
    let mut s = bar();
    let out = run(&mut s, Command::SeekLibraryCue(30.0));
    assert!(out.is_empty(), "{out:?}");
    assert_eq!(s.library_cue.start_at, Some(30.0));
    let out = run(&mut s, Command::PlayLibraryCue);
    assert_eq!(started(&out), (a(), 30.0, false));
    assert_eq!(s.library_cue.start_at, None);
}

#[test]
fn lb33_a_seek_outside_the_file_is_clamped_and_nan_is_ignored() {
    let mut s = bar();
    run(&mut s, Command::SeekLibraryCue(-5.0));
    assert_eq!(s.library_cue.start_at, Some(0.0));
    run(&mut s, Command::SeekLibraryCue(1e9));
    assert_eq!(s.library_cue.start_at, Some(180.0));
    let out = run(&mut s, Command::SeekLibraryCue(f64::NAN));
    assert!(out.is_empty(), "{out:?}");
    assert_eq!(s.library_cue.start_at, Some(180.0));
    run(&mut s, Command::PlayLibraryCue);
    let out = run(&mut s, Command::SeekLibraryCue(-5.0));
    assert_eq!(out, vec![EngineAction::SeekLibraryCue { secs: 0.0 }]);
}

#[test]
fn lb33_a_missing_file_does_not_start() {
    let mut s = bar();
    apply(
        &mut s,
        Command::SetFileState {
            track: a(),
            state: FileState::Missing,
        },
    )
    .unwrap();
    let out = run(&mut s, Command::PlayLibraryCue);
    assert!(out.is_empty(), "{out:?}");
    assert_eq!(s.library_cue.transport, Transport::Stopped);
}

#[test]
fn lb34_selecting_another_track_switches_a_playing_preview_from_its_cue_in() {
    let mut s = bar();
    run(&mut s, Command::PlayLibraryCue);
    let out = run(&mut s, Command::SelectLibraryCue(Some(b())));
    assert_eq!(started(&out), (b(), 2.0, false));
    assert_eq!(s.library_cue.track, Some(b()));
    assert_eq!(s.library_cue.transport, Transport::Playing);
}

#[test]
fn lb34_a_paused_preview_switches_held() {
    let mut s = bar();
    run(&mut s, Command::PlayLibraryCue);
    run(&mut s, Command::SetLibraryCuePaused(true));
    let out = run(&mut s, Command::SelectLibraryCue(Some(b())));
    assert_eq!(started(&out), (b(), 2.0, true));
    assert_eq!(s.library_cue.transport, Transport::Paused);
}

#[test]
fn lb34_a_stopped_preview_stays_stopped() {
    let mut s = bar();
    run(&mut s, Command::SeekLibraryCue(30.0));
    let out = run(&mut s, Command::SelectLibraryCue(Some(b())));
    assert!(out.is_empty(), "{out:?}");
    assert_eq!(s.library_cue.track, Some(b()));
    assert_eq!(s.library_cue.start_at, None);
    assert_eq!(s.library_cue.transport, Transport::Stopped);
}

#[test]
fn lb34_selecting_the_same_track_changes_nothing() {
    let mut s = bar();
    run(&mut s, Command::PlayLibraryCue);
    run(&mut s, Command::SeekLibraryCue(30.0));
    let out = run(&mut s, Command::SelectLibraryCue(Some(a())));
    assert!(out.is_empty(), "{out:?}");
    assert_eq!(s.library_cue.transport, Transport::Playing);
}

#[test]
fn lb34_selecting_nothing_leaves_a_running_preview() {
    let mut s = bar();
    run(&mut s, Command::PlayLibraryCue);
    let out = run(&mut s, Command::SelectLibraryCue(None));
    assert!(out.is_empty(), "{out:?}");
    assert_eq!(s.library_cue.track, Some(a()));
    assert_eq!(s.library_cue.transport, Transport::Playing);
    run(&mut s, Command::StopLibraryCue);
    run(&mut s, Command::SelectLibraryCue(None));
    assert_eq!(s.library_cue.track, None);
}

#[test]
fn lb34_selecting_a_track_that_cannot_play_stops_the_preview() {
    let mut s = bar();
    run(&mut s, Command::PlayLibraryCue);
    s.library.get_mut(b()).unwrap().file_state = FileState::Unreadable;
    let out = run(&mut s, Command::SelectLibraryCue(Some(b())));
    assert_eq!(out, vec![EngineAction::StopLibraryCue]);
    assert_eq!(s.library_cue.track, Some(b()));
    assert_eq!(s.library_cue.transport, Transport::Stopped);
}

#[test]
fn lb35_removing_the_track_stops_the_preview() {
    let mut s = bar();
    run(&mut s, Command::PlayLibraryCue);
    let out = run(&mut s, Command::RemoveTracks(vec![a()]));
    assert!(out.contains(&EngineAction::StopLibraryCue), "{out:?}");
    assert_eq!(s.library_cue.track, None);
    assert_eq!(s.library_cue.transport, Transport::Stopped);
}

#[test]
fn lb35_deleting_its_category_stops_the_preview() {
    let mut s = bar();
    run(
        &mut s,
        Command::CreateCategory {
            parent: None,
            name: "Jingles".into(),
            color: None,
            kind: TrackKind::Music,
        },
    );
    let category = s.categories.iter().map(|c| c.id).max().unwrap();
    run(
        &mut s,
        Command::MoveTracksToCategory {
            tracks: vec![a()],
            category: Some(category),
        },
    );
    run(&mut s, Command::PlayLibraryCue);
    let out = run(&mut s, Command::DeleteCategory(category));
    assert!(out.contains(&EngineAction::StopLibraryCue), "{out:?}");
    assert_eq!(s.library_cue.track, None);
    assert_eq!(s.library_cue.transport, Transport::Stopped);
}

#[test]
fn lb35_closing_the_window_stops_it_and_minimising_does_not() {
    let mut s = bar();
    let mut view = LibraryView {
        open: true,
        ..LibraryView::default()
    };
    run(&mut s, Command::SetLibraryView(Box::new(view.clone())));
    run(&mut s, Command::PlayLibraryCue);
    view.minimised = true;
    let out = run(&mut s, Command::SetLibraryView(Box::new(view.clone())));
    assert!(out.is_empty(), "{out:?}");
    assert_eq!(s.library_cue.transport, Transport::Playing);
    view.open = false;
    let out = run(&mut s, Command::SetLibraryView(Box::new(view)));
    assert_eq!(out, vec![EngineAction::StopLibraryCue]);
    assert_eq!(s.library_cue.transport, Transport::Stopped);
}

#[test]
fn lb36_the_request_carries_the_library_cue_normalization() {
    let mut s = bar();
    s.config.loudness.normalize = true;
    s.library.get_mut(a()).unwrap().loudness = Some(fp_model::LoudnessMeasurement {
        integrated_lufs: Some(-30.0),
        true_peak_dbtp: -20.0,
    });
    let want = normalization_for(&s, Holder::LibraryCue, s.library.get(a()).unwrap());
    let request = fp_model::library_cue::request(&s, a(), None).unwrap();
    assert_eq!(request.normalization, want);
    assert_ne!(want, Normalization::default());
    s.config.loudness.normalize = false;
    let request = fp_model::library_cue::request(&s, a(), None).unwrap();
    assert_eq!(request.normalization, Normalization::default());
    assert_eq!(request.from_secs, 1.5);
    assert!(fp_model::library_cue::request(&s, TrackId(99), None).is_none());
}

#[test]
fn rule10_a_restored_state_has_the_library_cue_stopped() {
    let mut s = bar();
    run(&mut s, Command::PlayLibraryCue);
    let parts = fp_model::RestoreParts {
        library_view: LibraryView {
            open: true,
            ..LibraryView::default()
        },
        config: s.config.clone(),
        library: s.library.clone(),
        categories: Default::default(),
        playlists: s.playlists.clone(),
        cart_pages: s.cartwall.pages.clone(),
        cartwall_session: s.cartwall.session(),
        layout: s.layout.clone(),
        ids: s.ids.clone(),
    };
    let (restored, actions) = AppState::restore(parts, &s.sessions(|_| 10.0), "Main");
    assert_eq!(restored.library_cue, LibraryCue::default());
    assert!(
        !actions
            .iter()
            .any(|a| matches!(a, EngineAction::StartLibraryCue { .. })),
        "{actions:?}"
    );
}

// ---- LB46: the default output as the library CUE's fallback ----

#[test]
fn lb46_the_default_output_gives_the_library_a_cue() {
    let mut config = Config::default();
    config.outputs.library = LibraryRoutes {
        cue: None,
        default_output: true,
    };
    assert!(config.outputs.library_has_cue());
    config.outputs.library.default_output = false;
    assert!(!config.outputs.library_has_cue());
}

#[test]
fn lb46_an_old_config_without_the_field_loads_the_platform_default() {
    let routes: LibraryRoutes = serde_json::from_str(r#"{"cue": null}"#).unwrap();
    assert_eq!(routes.default_output, cfg!(target_os = "android"));
    assert_eq!(routes, LibraryRoutes::default());
}

#[test]
fn lb46_play_is_available_without_a_route_on_the_default_output() {
    let mut s = bar();
    s.config.outputs.library = LibraryRoutes {
        cue: None,
        default_output: true,
    };
    assert!(command_available(&s, &Command::PlayLibraryCue));
    s.config.outputs.library.default_output = false;
    assert!(!command_available(&s, &Command::PlayLibraryCue));
}
