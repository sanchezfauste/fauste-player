#![allow(clippy::unwrap_used)]
mod support;

use fp_model::{Command, MarkerKind, Transport};
use fp_remote::control::Playback;
use fp_remote::events::{Event, diff, position};
use support::demo_state;

fn names(events: &[Event]) -> Vec<&'static str> {
    events.iter().map(Event::name).collect()
}

#[test]
fn an_unchanged_snapshot_has_no_events() {
    let s = demo_state();
    assert!(diff(&s, &s.clone(), &Playback::default()).is_empty());
}

#[test]
fn playing_changes_the_player_and_its_playlist() {
    let old = demo_state();
    let mut new = old.clone();
    let p = new.players[0].id;
    fp_model::apply(&mut new, Command::Play(p)).unwrap();
    let events = diff(&old, &new, &Playback::default());
    assert!(names(&events).contains(&"player"));
    assert!(names(&events).contains(&"playlist"));
    let Some(Event::Player(dto)) = events.iter().find(|e| e.name() == "player") else {
        panic!("{events:?}")
    };
    assert_eq!(dto.id, p);
    assert_eq!(dto.transport, "playing");
}

#[test]
fn a_moving_position_alone_is_not_a_player_change() {
    let mut old = demo_state();
    let p = old.players[0].id;
    fp_model::apply(&mut old, Command::Play(p)).unwrap();
    let new = old.clone();
    let pb = Playback {
        players: vec![(p, 42.0)],
        ..Default::default()
    };
    assert!(diff(&old, &new, &pb).is_empty());
}

#[test]
fn a_deleted_playlist_is_announced() {
    let old = demo_state();
    let mut new = old.clone();
    let night = new.playlists.iter().nth(1).unwrap().id;
    fp_model::apply(&mut new, Command::DeletePlaylist(night)).unwrap();
    let events = diff(&old, &new, &Playback::default());
    assert!(
        matches!(events.as_slice(), [Event::PlaylistRemoved(r)] if r.id == night),
        "{events:?}"
    );
}

#[test]
fn a_marker_change_announces_the_track() {
    let old = demo_state();
    let mut new = old.clone();
    let t = new.playlists.iter().next().unwrap().entries[0].track;
    fp_model::apply(
        &mut new,
        Command::SetMarker {
            track: t,
            kind: MarkerKind::IntroEnd,
            secs: Some(9.0),
        },
    )
    .unwrap();
    assert!(names(&diff(&old, &new, &Playback::default())).contains(&"track"));
}

#[test]
fn firing_a_cart_changes_the_cartwall() {
    let old = demo_state();
    let mut new = old.clone();
    let c = new.cartwall.pages[0].carts[0].id;
    fp_model::apply(&mut new, Command::FireCart(c)).unwrap();
    assert_eq!(
        names(&diff(&old, &new, &Playback::default())),
        vec!["cartwall"]
    );
}

#[test]
fn a_different_player_count_sends_the_whole_state() {
    let old = demo_state();
    let mut new = old.clone();
    fp_model::apply(&mut new, Command::SetPlayerCount(2)).unwrap();
    assert_eq!(
        names(&diff(&old, &new, &Playback::default())),
        vec!["state"]
    );
}

#[test]
fn position_only_while_something_plays() {
    let mut s = demo_state();
    assert!(position(&s, &Playback::default()).is_none());
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    assert_eq!(s.players[0].transport, Transport::Playing);
    let pos = position(
        &s,
        &Playback {
            players: vec![(p, 20.0)],
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(pos.players[0].elapsed_secs, Some(20.0));
    assert_eq!(pos.players[0].remaining_secs, Some(160.0));
    assert!(pos.carts.is_empty());
}

#[test]
fn events_serialise_to_json() {
    let old = demo_state();
    let mut new = old.clone();
    let c = new.cartwall.pages[0].carts[0].id;
    fp_model::apply(&mut new, Command::FireCart(c)).unwrap();
    let json: serde_json::Value =
        serde_json::from_str(&diff(&old, &new, &Playback::default())[0].json()).unwrap();
    assert!(json["playing"].is_array());
}
