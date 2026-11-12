//! Writes a demo state under `FAUSTE_HOME`: four players and three
//! playlists filled with the audio files of a folder, with the first
//! players paused mid-track some way into their playlist. The carts come
//! from the shortest files, or from a second folder when one is given.
//! Automation gets a second rotation and a schedule: a block every hour,
//! a weekend template and two dates changed.
//!
//! `FAUSTE_HOME=/tmp/fp-demo cargo run -p fp-app --example demo_session -- <music folder> [cart folder]`

use std::path::PathBuf;
use std::process::ExitCode;

use fp_app::bootstrap;
use fp_app::ui::files::audio_paths;
use fp_model::library_view::LibrarySelection;
use fp_model::{
    AppState, CartColor, CartEdit, CartKind, CategoryId, Command, Config, Limits, PlaylistId,
    TrackKind,
};
use fp_store::Store;

fn main() -> ExitCode {
    // Never write the demo over a real installation.
    if std::env::var_os(bootstrap::HOME_VAR).is_none() {
        eprintln!(
            "demo_session: set {} to a scratch directory first",
            bootstrap::HOME_VAR
        );
        return ExitCode::FAILURE;
    }
    let (Some(paths), Some(folder)) = (bootstrap::paths(), std::env::args().nth(1)) else {
        eprintln!("usage: FAUSTE_HOME=<dir> demo_session <music folder> [cart folder]");
        return ExitCode::FAILURE;
    };
    let files = audio_paths(&[PathBuf::from(folder)]);
    if files.is_empty() {
        eprintln!(
            "demo_session: no audio files directly inside that folder (sub-folders are not read)"
        );
        return ExitCode::FAILURE;
    }
    let mut config = Config::default();
    config.players.count = 4;
    // An editor set, so the track menu offers "Open in external editor".
    config.library.external_editor = Some(PathBuf::from("/usr/bin/audio-editor"));
    let mut state = AppState::new(config, "Morning");
    let names = ["Afternoon", "Night"];
    for name in names {
        let _ = fp_model::apply(&mut state, Command::CreatePlaylist { name: name.into() });
    }
    let lists: Vec<PlaylistId> = state.playlists.iter().map(|p| p.id).collect();
    for (n, list) in lists.iter().enumerate() {
        let mut chosen = files.clone();
        let len = chosen.len();
        chosen.rotate_left((n * 3) % len);
        let _ = fp_model::apply(
            &mut state,
            Command::InsertPaths {
                playlist: *list,
                index: 0,
                paths: chosen,
            },
        );
    }
    let players: Vec<_> = state.players.iter().map(|p| p.id).collect();
    for (n, player) in players.iter().enumerate() {
        if let Some(list) = lists.get(n % lists.len()) {
            let _ = fp_model::apply(&mut state, Command::ShowPlaylist(*player, *list));
            let first = state
                .playlists
                .get(*list)
                .and_then(|p| p.entries.first())
                .map(|e| e.id);
            if let Some(entry) = first.filter(|_| n < 3) {
                let _ = fp_model::apply(&mut state, Command::SetNext(*player, entry));
            }
        }
        // Some way into the hour: player 1 on its 5th track, player 2 on
        // its 3rd, player 3 on its 2nd (the played rows show dimmed);
        // player 4 waits. Each stop leaves a played track behind.
        let plays = [5, 3, 2].get(n).copied().unwrap_or(0);
        for k in 0..plays {
            if k > 0 {
                let _ = fp_model::apply(&mut state, Command::Stop(*player));
            }
            let _ = fp_model::apply(&mut state, Command::Play(*player));
        }
    }
    // Some carts on the first page, from the cart folder or else from the
    // shortest files.
    let mut short = match std::env::args().nth(2) {
        Some(carts) => audio_paths(&[PathBuf::from(carts)]),
        None => files.clone(),
    };
    short.sort_by_key(|p| std::fs::metadata(p).map(|m| m.len()).unwrap_or(u64::MAX));
    let page = state.cartwall.pages[0].id;
    let _ = fp_model::apply(
        &mut state,
        Command::RenameCartPage {
            page,
            name: "General".into(),
        },
    );
    let kinds = [CartKind::Jingle, CartKind::Effect, CartKind::Spot];
    // A colour for most carts, so the screenshots show them (cart colour
    // spec CC6, CC7); the plain ones keep the default look.
    let colors = [
        Some(CartColor::Teal),
        None,
        Some(CartColor::Amber),
        Some(CartColor::Teal),
        None,
        Some(CartColor::Violet),
    ];
    for (index, path) in short.iter().take(6).enumerate() {
        let _ = fp_model::apply(
            &mut state,
            Command::AssignCartFile {
                page,
                index,
                path: path.clone(),
            },
        );
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let edit = CartEdit {
            name,
            kind: kinds[index % kinds.len()],
            looped: index == 5,
            exclusive: index == 0,
            color: colors.get(index).copied().flatten(),
        };
        let _ = fp_model::apply(&mut state, Command::SetCart { page, index, edit });
    }
    let _ = fp_model::apply(
        &mut state,
        Command::CreateCartPage {
            name: "Effects".into(),
        },
    );
    library(&mut state, &files, &short);
    automation(&mut state);
    let mut store = Store::new(paths, Limits::default());
    let saved = store.save_config(&state).and_then(|()| {
        store.save_state(
            &state,
            |p| 8.0 + f64::from(p.0 as u32 % 4) * 3.0,
            std::time::Instant::now(),
        )
    });
    match saved {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("demo_session: {e}");
            ExitCode::FAILURE
        }
    }
}

/// The track library of the guide's screenshot: the songs in two
/// categories under Music, the cart files in Jingles, and the window on
/// Music with its subcategories shown.
fn library(state: &mut AppState, songs: &[PathBuf], carts: &[PathBuf]) {
    fn category(
        state: &mut AppState,
        parent: Option<CategoryId>,
        name: &str,
        color: Option<CartColor>,
        kind: TrackKind,
    ) -> Option<CategoryId> {
        let created = Command::CreateCategory {
            parent,
            name: name.into(),
            color,
            kind,
        };
        fp_model::apply(state, created).ok()?;
        state
            .categories
            .iter()
            .find(|c| c.name == name && c.parent == parent)
            .map(|c| c.id)
    }
    let music = category(
        state,
        None,
        "Music",
        Some(CartColor::Teal),
        TrackKind::Music,
    );
    let pop = category(state, music, "Pop", None, TrackKind::Music);
    let evening = category(
        state,
        music,
        "Evening",
        Some(CartColor::Violet),
        TrackKind::Music,
    );
    let jingles = category(
        state,
        None,
        "Jingles",
        Some(CartColor::Amber),
        TrackKind::Jingle,
    );
    let (first, second) = songs.split_at(songs.len() / 2);
    let carts = carts.get(..6.min(carts.len())).unwrap_or_default();
    for (paths, category) in [(carts, jingles), (first, pop), (second, evening)] {
        let _ = fp_model::apply(
            state,
            Command::AddToLibrary {
                paths: paths.to_vec(),
                category,
            },
        );
    }
    // One effect whose audio changed under its manual outro: the library
    // tree shows "Markers to review". An effect, not a song: the songs are
    // on every playlist of the README and Google Play screenshots, and the
    // flag would show on their rows.
    let flagged = state
        .library
        .iter()
        .find(|t| {
            t.path
                .file_stem()
                .is_some_and(|s| s.to_string_lossy() == "Effect – Applause")
        })
        .map(|t| t.id);
    if let Some(track) = flagged.and_then(|id| state.library.get_mut(id)) {
        track.markers_review = Some(fp_model::MarkersReview {
            dropped: vec![fp_model::MarkerKind::OutroStart],
        });
    }
    let mut view = state.library_view.clone();
    if let Some(music) = music {
        view.selection = LibrarySelection::Category(music);
        view.expanded = vec![music];
    }
    let _ = fp_model::apply(state, Command::SetLibraryView(Box::new(view)));
}

/// The guide's automation screenshots: a "Hits" rotation beside the
/// default one, a "Base" template with a block every hour (the hourly ID,
/// spots, and the Hits rotation in the daytime), a "Weekend" template for
/// Saturday and Sunday, and two dates changed: today's midnight block gets
/// a third spot, and the date a week on uses the weekend template.
fn automation(state: &mut AppState) {
    use chrono::{Datelike, Duration, Local, Weekday};
    use fp_model::local_time::TimeOfDay;
    use fp_model::rotation::{RotationStep, StepSource};
    use fp_model::schedule::{Block, Item};

    let track = |state: &AppState, stem: &str| {
        state
            .library
            .iter()
            .find(|t| {
                t.path
                    .file_stem()
                    .is_some_and(|s| s.to_string_lossy() == stem)
            })
            .map(|t| t.id)
    };
    let category = |state: &AppState, name: &str| {
        state
            .categories
            .iter()
            .find(|c| c.name == name)
            .map(|c| c.id)
    };
    let station_id = track(state, "Jingle – Station ID");
    let spots: Vec<_> = ["Spot – Riverside Bakery", "Spot – City Theatre"]
        .iter()
        .filter_map(|s| track(state, s))
        .collect();
    if let Some(id) = station_id {
        let _ = fp_model::apply(
            state,
            Command::SetTracksKind {
                tracks: vec![id],
                kind: TrackKind::HourlyId,
            },
        );
    }
    let _ = fp_model::apply(
        state,
        Command::SetTracksKind {
            tracks: spots.clone(),
            kind: TrackKind::Ad,
        },
    );

    fp_model::rotation::ensure_default(state, "Music");
    let step = |category: Option<CategoryId>, min, max| {
        category.map(|c| RotationStep {
            source: StepSource::Category(c),
            min,
            max,
        })
    };
    let steps: Vec<RotationStep> = [
        step(category(state, "Pop"), 2, 3),
        step(category(state, "Jingles"), 1, 1),
        step(category(state, "Evening"), 1, 1),
    ]
    .into_iter()
    .flatten()
    .collect();
    let _ = fp_model::apply(
        state,
        Command::CreateRotation {
            name: "Hits".into(),
            steps,
        },
    );
    let hits = state
        .rotations
        .iter()
        .find(|r| r.name == "Hits")
        .map(|r| r.id);

    fp_model::schedule::ensure_default(state, "Base");
    let Some(base) = state.schedule.templates().first().map(|t| t.id) else {
        return;
    };
    for hour in 0..24 {
        let Some(start) = TimeOfDay::hms(hour, 0, 0) else {
            continue;
        };
        let _ = fp_model::apply(
            state,
            Command::AddBlock {
                template: base,
                start,
            },
        );
        let Some(id) = state
            .schedule
            .template(base)
            .and_then(|t| t.blocks.iter().find(|b| b.start == start))
            .map(|b| b.id)
        else {
            continue;
        };
        let mut items = Vec::new();
        if let Some(track) = station_id {
            items.push(Item::Track {
                track,
                time_critical: false,
            });
        }
        if hour % 2 == 0 {
            items.extend(spots.iter().map(|&track| Item::Track {
                track,
                time_critical: false,
            }));
        }
        // The Hits rotation in the daytime, the default one at night.
        let rotation = if (7..20).contains(&hour) {
            hits
        } else {
            state.rotations.default_id()
        };
        if let Some(rotation) = rotation {
            items.push(Item::Rotation { rotation });
        }
        let _ = fp_model::apply(
            state,
            Command::UpdateBlock {
                template: base,
                block: Block { id, start, items },
            },
        );
    }
    let _ = fp_model::apply(
        state,
        Command::DuplicateTemplate {
            template: base,
            name: "Weekend".into(),
        },
    );
    if let Some(weekend) = state
        .schedule
        .templates()
        .iter()
        .find(|t| t.name == "Weekend")
        .map(|t| t.id)
    {
        for weekday in [Weekday::Sat, Weekday::Sun] {
            let _ = fp_model::apply(
                state,
                Command::SetWeekTemplate {
                    weekday,
                    template: weekend,
                },
            );
        }
        let mut holiday = Local::now().date_naive() + Duration::days(7);
        if matches!(holiday.weekday(), Weekday::Sat | Weekday::Sun) {
            holiday += Duration::days(2);
        }
        let _ = fp_model::apply(
            state,
            Command::SetDateTemplate {
                date: holiday,
                template: Some(weekend),
            },
        );
    }
    // Today's midnight block with one more spot, for that date only: the
    // editor's Days tab opens on today and shows it first.
    let today = Local::now().date_naive();
    let midnight = TimeOfDay::hms(0, 0, 0);
    let block = state
        .schedule
        .day(today)
        .into_iter()
        .find(|b| Some(b.start) == midnight);
    if let (Some(block), Some(&spot)) = (block, spots.first()) {
        let mut items = block.items.clone();
        let at = items.len().saturating_sub(1);
        items.insert(
            at,
            Item::Track {
                track: spot,
                time_critical: false,
            },
        );
        let _ = fp_model::apply(
            state,
            Command::ChangeBlockOnDate {
                date: today,
                block: Block {
                    id: block.id,
                    start: block.start,
                    items,
                },
            },
        );
    }
}
