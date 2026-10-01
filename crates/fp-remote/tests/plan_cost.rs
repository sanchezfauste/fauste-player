#![allow(clippy::unwrap_used)]
//! The cost of the dry run on a large state (plan 4, item 9). Local only:
//! `cargo test --release -p fp-remote --test plan_cost -- --ignored --nocapture`.

use std::path::PathBuf;
use std::time::Instant;

use fp_model::{AppState, Command, Config};
use fp_remote::api::{Edit, Operation, plan, plan_edit};

#[test]
#[ignore]
fn plan_on_5000_entries_and_16_players() {
    let mut c = Config::default();
    c.players.count = 16;
    let mut s = AppState::new(c, "Main");
    let list = s.playlists.first_id().unwrap();
    let paths: Vec<PathBuf> = (0..5_000)
        .map(|i| {
            PathBuf::from(format!(
                "/music/a-fairly-long-folder-name/track-{i:05}.flac"
            ))
        })
        .collect();
    fp_model::apply(
        &mut s,
        Command::InsertPaths {
            playlist: list,
            index: 0,
            paths,
        },
    )
    .unwrap();
    let ids: Vec<_> = s.library.iter().map(|t| t.id).collect();
    for id in ids {
        let t = s.library.get_mut(id).unwrap();
        t.duration_secs = 200.0;
        t.title = format!("Title of track {}", id.0);
        t.artist = "Some Artist".into();
    }
    assert_eq!(s.players.len(), 16);
    let p = s.players[0].id;
    let e = s.playlists.get(list).unwrap().entries[10].id;
    let n = 200;
    let t0 = Instant::now();
    for _ in 0..n {
        std::hint::black_box(s.clone());
    }
    let clone = t0.elapsed() / n;
    let t0 = Instant::now();
    for _ in 0..n {
        std::hint::black_box(plan(&s, Operation::Play(p)).unwrap());
    }
    let play = t0.elapsed() / n;
    let t0 = Instant::now();
    for _ in 0..n {
        std::hint::black_box(
            plan_edit(
                &s,
                Edit::MoveEntry {
                    entry: e,
                    playlist: list,
                    index: 4_000,
                },
            )
            .unwrap(),
        );
    }
    let mv = t0.elapsed() / n;
    println!("clone {clone:?}  plan(Play) {play:?}  plan_edit(Move) {mv:?}");
}
