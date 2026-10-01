#![allow(clippy::unwrap_used)]
mod support;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use fp_model::{CartEdit, CartKind, Command, HttpRemoteConfig, MarkerKind, TrackId};
use fp_remote::api::{ApiError, Edit, plan_edit};
use fp_remote::http::{Ctx, router};
use serde_json::{Value, json};
use support::{FakeControl, demo_state};
use tower::ServiceExt;

#[test]
fn names_are_trimmed_and_must_not_be_empty() {
    let s = demo_state();
    assert_eq!(
        plan_edit(&s, Edit::CreatePlaylist("  Late ".into())).unwrap(),
        vec![Command::CreatePlaylist {
            name: "Late".into()
        }]
    );
    assert_eq!(
        plan_edit(&s, Edit::CreatePlaylist("  ".into()))
            .unwrap_err()
            .status(),
        400
    );
}

#[test]
fn a_track_is_inserted_by_id_with_the_index_clamped() {
    let s = demo_state();
    let night = s.playlists.iter().nth(1).unwrap().id;
    let t = s.playlists.iter().next().unwrap().entries[0].track;
    assert_eq!(
        plan_edit(
            &s,
            Edit::InsertTrack {
                playlist: night,
                index: 99,
                track: t
            }
        )
        .unwrap(),
        vec![Command::InsertTracks {
            playlist: night,
            index: 0,
            tracks: vec![t]
        }]
    );
    assert_eq!(
        plan_edit(
            &s,
            Edit::InsertTrack {
                playlist: night,
                index: 0,
                track: TrackId(999_999)
            }
        )
        .unwrap_err(),
        ApiError::NotFound
    );
}

#[test]
fn the_playlist_on_air_cannot_be_deleted() {
    let mut s = demo_state();
    let p = s.players[0].id;
    fp_model::apply(&mut s, Command::Play(p)).unwrap();
    let main = s.playlists.first_id().unwrap();
    let e = plan_edit(&s, Edit::DeletePlaylist(main)).unwrap_err();
    assert_eq!(e.status(), 409, "{e:?}");
    let current = s.players[0].current.unwrap();
    assert_eq!(
        plan_edit(&s, Edit::RemoveEntry(current))
            .unwrap_err()
            .status(),
        409
    );
}

#[test]
fn moving_clamps_into_the_target_playlist() {
    let s = demo_state();
    let e = s.playlists.iter().next().unwrap().entries[2].id;
    let night = s.playlists.iter().nth(1).unwrap().id;
    assert_eq!(
        plan_edit(
            &s,
            Edit::MoveEntry {
                entry: e,
                playlist: night,
                index: 50
            }
        )
        .unwrap(),
        vec![Command::MoveEntry {
            entry: e,
            to: night,
            index: 0
        }]
    );
}

#[test]
fn cart_pages_are_renamed_and_resized_within_limits() {
    let s = demo_state();
    let page = s.cartwall.pages[0].id;
    let (rows, cols) = (s.cartwall.pages[0].rows, s.cartwall.pages[0].cols);
    assert_eq!(
        plan_edit(
            &s,
            Edit::EditCartPage {
                page,
                name: Some("Jingles".into()),
                rows: None,
                cols: None
            }
        )
        .unwrap(),
        vec![Command::RenameCartPage {
            page,
            name: "Jingles".into()
        }]
    );
    assert_eq!(
        plan_edit(
            &s,
            Edit::EditCartPage {
                page,
                name: None,
                rows: Some(rows + 1),
                cols: None
            }
        )
        .unwrap(),
        vec![Command::ResizeCartPage {
            page,
            rows: rows + 1,
            cols
        }]
    );
    let too_big = s.config.limits.max_cart_rows + 1;
    assert_eq!(
        plan_edit(
            &s,
            Edit::EditCartPage {
                page,
                name: None,
                rows: Some(too_big),
                cols: None
            }
        )
        .unwrap_err()
        .status(),
        400
    );
    assert_eq!(
        plan_edit(
            &s,
            Edit::EditCartPage {
                page,
                name: None,
                rows: None,
                cols: None
            }
        )
        .unwrap_err()
        .status(),
        400
    );
}

#[test]
fn a_cart_edit_keeping_its_track_does_not_reassign_it() {
    let s = demo_state();
    let page = s.cartwall.pages[0].id;
    let t = s.cartwall.pages[0].carts[0].track.unwrap();
    let edit = CartEdit {
        name: "Top".into(),
        kind: CartKind::Spot,
        looped: false,
        exclusive: true,
    };
    assert_eq!(
        plan_edit(
            &s,
            Edit::SetCart {
                page,
                index: 0,
                edit: edit.clone(),
                track: Some(t)
            }
        )
        .unwrap(),
        vec![Command::SetCart {
            page,
            index: 0,
            edit: edit.clone()
        }]
    );
    let other = s.playlists.iter().next().unwrap().entries[1].track;
    assert_eq!(
        plan_edit(
            &s,
            Edit::SetCart {
                page,
                index: 0,
                edit: edit.clone(),
                track: Some(other)
            }
        )
        .unwrap(),
        vec![
            Command::SetCart {
                page,
                index: 0,
                edit: edit.clone()
            },
            Command::AssignCartTrack {
                page,
                index: 0,
                track: other
            },
        ]
    );
    assert_eq!(
        plan_edit(
            &s,
            Edit::SetCart {
                page,
                index: 0,
                edit: edit.clone(),
                track: None
            }
        )
        .unwrap(),
        vec![
            Command::SetCart {
                page,
                index: 0,
                edit: edit.clone()
            },
            Command::ClearCartFile { page, index: 0 }
        ]
    );
    assert_eq!(
        plan_edit(
            &s,
            Edit::SetCart {
                page,
                index: 999,
                edit,
                track: None
            }
        )
        .unwrap_err(),
        ApiError::NotFound
    );
}

#[test]
fn markers_take_finite_seconds() {
    let s = demo_state();
    let t = s.playlists.iter().next().unwrap().entries[0].track;
    assert_eq!(
        plan_edit(
            &s,
            Edit::SetMarker {
                track: t,
                kind: MarkerKind::CueOut,
                secs: Some(170.0)
            }
        )
        .unwrap(),
        vec![Command::SetMarker {
            track: t,
            kind: MarkerKind::CueOut,
            secs: Some(170.0)
        }]
    );
    assert_eq!(
        plan_edit(
            &s,
            Edit::SetMarker {
                track: t,
                kind: MarkerKind::CueOut,
                secs: Some(f64::NAN)
            }
        )
        .unwrap_err()
        .status(),
        400
    );
    assert_eq!(
        plan_edit(
            &s,
            Edit::SetMarker {
                track: t,
                kind: MarkerKind::CueOut,
                secs: Some(-1.0)
            }
        )
        .unwrap_err()
        .status(),
        400
    );
    assert_eq!(
        plan_edit(&s, Edit::ResetMarkers(t)).unwrap(),
        vec![Command::ResetMarkers { track: t }]
    );
}

async fn call(fake: &Arc<FakeControl>, method: &str, uri: &str, body: Option<Value>) -> StatusCode {
    let ctx = Ctx::new(fake.clone(), Arc::new(HttpRemoteConfig::default()));
    let mut req = Request::builder()
        .method(method)
        .uri(uri)
        .header("host", "127.0.0.1:7380");
    let body = match body {
        Some(v) => {
            req = req.header("content-type", "application/json");
            Body::from(v.to_string())
        }
        None => Body::empty(),
    };
    router(ctx)
        .oneshot(req.body(body).unwrap())
        .await
        .unwrap()
        .status()
}

#[tokio::test]
async fn every_editing_route_answers() {
    let s = demo_state();
    let main = s.playlists.first_id().unwrap().0;
    let night = s.playlists.iter().nth(1).unwrap().id.0;
    let e = s.playlists.iter().next().unwrap().entries[2].id.0;
    let t = s.playlists.iter().next().unwrap().entries[0].track.0;
    let page = s.cartwall.pages[0].id.0;
    let fake = FakeControl::new(s);
    let accepted = [
        (
            "POST",
            "/api/v1/playlists".to_owned(),
            Some(json!({"name": "Late"})),
        ),
        (
            "PATCH",
            format!("/api/v1/playlists/{night}"),
            Some(json!({"name": "Overnight"})),
        ),
        (
            "POST",
            format!("/api/v1/playlists/{night}/entries"),
            Some(json!({"track": t, "index": 0})),
        ),
        (
            "POST",
            format!("/api/v1/entries/{e}/move"),
            Some(json!({"playlist": main, "index": 0})),
        ),
        ("POST", format!("/api/v1/entries/{e}/duplicate"), None),
        ("DELETE", format!("/api/v1/entries/{e}"), None),
        (
            "POST",
            "/api/v1/cartwall/pages".to_owned(),
            Some(json!({"name": "B"})),
        ),
        (
            "PATCH",
            format!("/api/v1/cartwall/pages/{page}"),
            Some(json!({"name": "A"})),
        ),
        (
            "PUT",
            format!("/api/v1/cartwall/pages/{page}/carts/1"),
            Some(
                json!({"name": "Bed", "kind": "effect", "looped": true, "exclusive": false, "track": t}),
            ),
        ),
        (
            "PUT",
            format!("/api/v1/tracks/{t}/markers/intro-end"),
            Some(json!({"secs": 4.0})),
        ),
        (
            "PUT",
            format!("/api/v1/tracks/{t}/markers/intro-end"),
            Some(json!({"secs": null})),
        ),
        ("POST", format!("/api/v1/tracks/{t}/markers/reset"), None),
        ("DELETE", format!("/api/v1/playlists/{night}"), None),
    ];
    for (method, uri, body) in accepted {
        assert_eq!(
            call(&fake, method, &uri, body).await,
            StatusCode::ACCEPTED,
            "{method} {uri}"
        );
    }
    assert_eq!(
        call(
            &fake,
            "PUT",
            &format!("/api/v1/tracks/{t}/markers/middle"),
            Some(json!({"secs": 1.0}))
        )
        .await,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(&fake, "PUT", &format!("/api/v1/cartwall/pages/{page}/carts/0"), Some(json!({"name": "x", "kind": "song", "looped": false, "exclusive": false, "track": null}))).await,
        StatusCode::BAD_REQUEST
    );
}
