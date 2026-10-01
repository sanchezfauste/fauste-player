#![allow(clippy::unwrap_used)]
mod support;

use std::sync::Arc;
use std::sync::atomic::Ordering;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use fp_model::{Command, HttpRemoteConfig};
use fp_remote::control::{Playback, WaveformData};
use fp_remote::http::{Ctx, router};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use support::{FakeControl, demo_state};
use tower::ServiceExt;

pub fn ctx(fake: &Arc<FakeControl>) -> Ctx {
    Ctx::new(fake.clone(), Arc::new(HttpRemoteConfig::default()))
}

/// Sends a request with `Host: 127.0.0.1:7380` (the guard of Task 5 needs it).
pub async fn call(
    ctx: Ctx,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value, Vec<u8>) {
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
    let res = router(ctx).oneshot(req.body(body).unwrap()).await.unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes().to_vec();
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json, bytes)
}

#[tokio::test]
async fn get_state_returns_players_playlists_and_cartwall() {
    let fake = FakeControl::new(demo_state());
    fake.playback.store(Arc::new(Playback {
        revision: 3,
        ..Default::default()
    }));
    let (status, body, _) = call(ctx(&fake), "GET", "/api/v1/state", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["revision"], 3);
    assert_eq!(body["players"].as_array().unwrap().len(), 4);
    assert_eq!(body["playlists"][0]["entry_count"], 3);
    assert!(body["cartwall"]["pages"].is_array());
}

#[tokio::test]
async fn single_resources_are_found_by_id() {
    let s = demo_state();
    let p = s.players[0].id.0;
    let list = s.playlists.first_id().unwrap().0;
    let track = s.playlists.iter().next().unwrap().entries[0].track.0;
    let fake = FakeControl::new(s);
    for uri in [
        format!("/api/v1/players/{p}"),
        "/api/v1/players".to_owned(),
        "/api/v1/playlists".to_owned(),
        format!("/api/v1/playlists/{list}"),
        format!("/api/v1/tracks/{track}"),
        "/api/v1/cartwall".to_owned(),
    ] {
        let (status, _, _) = call(ctx(&fake), "GET", &uri, None).await;
        assert_eq!(status, StatusCode::OK, "{uri}");
    }
}

#[tokio::test]
async fn malformed_or_unknown_ids_and_paths_are_json_404s() {
    let fake = FakeControl::new(demo_state());
    for (method, uri) in [
        ("GET", "/api/v1/players/abc"),
        ("GET", "/api/v1/players/99999999999999999999999"),
        ("GET", "/api/v1/players/424242"),
        ("POST", "/api/v1/players/1/dance"),
        ("GET", "/api/v1/nothing"),
        ("GET", "/"),
    ] {
        let (status, body, _) = call(ctx(&fake), method, uri, None).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{uri}");
        assert_eq!(body["error"], "not_found", "{uri}");
    }
}

#[tokio::test]
async fn play_is_accepted_and_queued() {
    let s = demo_state();
    let p = s.players[0].id;
    let fake = FakeControl::new(s);
    fake.playback.store(Arc::new(Playback {
        revision: 9,
        ..Default::default()
    }));
    let (status, body, _) = call(
        ctx(&fake),
        "POST",
        &format!("/api/v1/players/{}/play", p.0),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(body["revision"], 9);
    assert_eq!(fake.take_sent(), vec![Command::Play(p)]);
}

#[tokio::test]
async fn every_transport_action_has_its_route() {
    let s = demo_state();
    let p = s.players[0].id.0;
    let fake = FakeControl::new(s);
    for action in ["pause", "stop", "fade-stop", "restart", "previous"] {
        let (status, body, _) = call(
            ctx(&fake),
            "POST",
            &format!("/api/v1/players/{p}/{action}"),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::CONFLICT, "{action}");
        assert_eq!(body["error"], "unavailable");
    }
    assert!(fake.take_sent().is_empty());
}

#[tokio::test]
async fn bodies_are_read_and_checked() {
    let s = demo_state();
    let p = s.players[0].id.0;
    let e = s.playlists.iter().next().unwrap().entries[2].id.0;
    let night = s.playlists.iter().nth(1).unwrap().id.0;
    let c = s.cartwall.pages[0].carts[0].id.0;
    let page = s.cartwall.pages[0].id.0;
    let fake = FakeControl::new(s);
    let ok = [
        (
            "PUT",
            format!("/api/v1/players/{p}/volume"),
            json!({"fader": 0.5}),
        ),
        (
            "PUT",
            format!("/api/v1/players/{p}/next"),
            json!({"entry": e}),
        ),
        (
            "POST",
            format!("/api/v1/players/{p}/cue-entry"),
            json!({"entry": e}),
        ),
        (
            "PUT",
            format!("/api/v1/players/{p}/mode"),
            json!({"mode": "single"}),
        ),
        (
            "PUT",
            format!("/api/v1/players/{p}/stop-after-current"),
            json!({"on": false}),
        ),
        (
            "PUT",
            format!("/api/v1/players/{p}/cue"),
            json!({"on": true}),
        ),
        (
            "PUT",
            format!("/api/v1/players/{p}/playlist"),
            json!({"playlist": night}),
        ),
        (
            "PUT",
            format!("/api/v1/entries/{e}/repeat"),
            json!({"on": true}),
        ),
        (
            "PUT",
            format!("/api/v1/entries/{e}/stop-after"),
            json!({"on": true}),
        ),
        ("PUT", format!("/api/v1/carts/{c}/cue"), json!({"on": true})),
        (
            "PUT",
            "/api/v1/cartwall/shown".to_owned(),
            json!({"page": page}),
        ),
    ];
    for (method, uri, body) in ok {
        let (status, out, _) = call(ctx(&fake), method, &uri, Some(body)).await;
        assert_eq!(status, StatusCode::ACCEPTED, "{uri}: {out}");
    }
    for uri in [
        format!("/api/v1/carts/{c}/fire"),
        format!("/api/v1/carts/{c}/stop"),
        "/api/v1/cartwall/stop-all".to_owned(),
    ] {
        let (status, _, _) = call(ctx(&fake), "POST", &uri, None).await;
        assert_eq!(status, StatusCode::ACCEPTED, "{uri}");
    }
}

#[tokio::test]
async fn broken_bodies_are_400s() {
    let s = demo_state();
    let p = s.players[0].id.0;
    let fake = FakeControl::new(s);
    let uri = format!("/api/v1/players/{p}/volume");
    for body in [
        json!({"fader": "x"}),
        json!({}),
        json!({"fader": 2.0}),
        json!([1]),
    ] {
        let (status, out, _) = call(ctx(&fake), "PUT", &uri, Some(body.clone())).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
        assert_eq!(out["error"], "bad_request");
    }
    let (status, _, _) = call(
        ctx(&fake),
        "PUT",
        &format!("/api/v1/players/{p}/mode"),
        Some(json!({"mode": "x"})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(fake.take_sent().is_empty());
}

#[tokio::test]
async fn a_body_that_is_not_json_is_415() {
    let s = demo_state();
    let p = s.players[0].id.0;
    let fake = FakeControl::new(s);
    let req = Request::put(format!("/api/v1/players/{p}/volume"))
        .header("host", "127.0.0.1:7380")
        .header("content-type", "text/plain")
        .body(Body::from(r#"{"fader":0.5}"#))
        .unwrap();
    let res = router(ctx(&fake)).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[tokio::test]
async fn a_full_queue_is_503() {
    let s = demo_state();
    let p = s.players[0].id.0;
    let fake = FakeControl::new(s);
    fake.accept.store(false, Ordering::SeqCst);
    let (status, body, _) = call(
        ctx(&fake),
        "POST",
        &format!("/api/v1/players/{p}/play"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["error"], "busy");
}

#[tokio::test]
async fn cover_and_peaks_come_from_the_control_once_analysed() {
    let mut s = demo_state();
    let t = s.playlists.iter().next().unwrap().entries[0].track;
    let fake = FakeControl::new(s.clone());
    let uri = format!("/api/v1/tracks/{}/peaks", t.0);
    let (status, body, _) = call(ctx(&fake), "GET", &uri, None).await;
    assert_eq!(
        (status, body["error"].clone()),
        (StatusCode::NOT_FOUND, json!("not_analyzed"))
    );

    s.library.get_mut(t).unwrap().analyzed = true;
    fake.state.store(Arc::new(s));
    let (status, body, _) = call(ctx(&fake), "GET", &uri, None).await;
    assert_eq!(
        (status, body["error"].clone()),
        (StatusCode::NOT_FOUND, json!("not_found"))
    );

    fake.peaks.lock().unwrap().insert(
        t,
        WaveformData {
            bucket_secs: 0.05,
            peaks: vec![[-10, 20, 5]],
        },
    );
    fake.covers
        .lock()
        .unwrap()
        .insert(t, vec![0x89, b'P', b'N', b'G']);
    let (status, body, _) = call(ctx(&fake), "GET", &uri, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body,
        json!({"bucket_secs": 0.05, "full_scale": 32767, "peaks": [[-10, 20, 5]]})
    );
    let (status, _, bytes) = call(
        ctx(&fake),
        "GET",
        &format!("/api/v1/tracks/{}/cover", t.0),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(bytes, vec![0x89, b'P', b'N', b'G']);
}

#[tokio::test]
async fn a_wrong_method_is_a_json_405() {
    let fake = FakeControl::new(demo_state());
    let (status, body, _) = call(ctx(&fake), "GET", "/api/v1/players/1/play", None).await;
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(body["error"], "method_not_allowed");
}
