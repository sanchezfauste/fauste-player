#![allow(clippy::unwrap_used, clippy::expect_used)]
mod support;

use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use fp_model::HttpRemoteConfig;
use fp_remote::events::{Envelope, Event, RemovedDto};
use fp_remote::http::{Ctx, router};
use http_body_util::BodyExt;
use support::{FakeControl, demo_state};
use tokio::sync::broadcast;
use tower::ServiceExt;

const TOKEN: &str = "0123456789abcdef0123";

fn ctx_with(config: HttpRemoteConfig, capacity: usize) -> (Ctx, broadcast::Sender<Arc<Envelope>>) {
    let (tx, _) = broadcast::channel(capacity);
    let ctx = Ctx::new(FakeControl::new(demo_state()), Arc::new(config)).with_events(tx.clone());
    (ctx, tx)
}

fn get(uri: &str) -> Request<Body> {
    Request::get(uri)
        .header("host", "127.0.0.1:7380")
        .body(Body::empty())
        .unwrap()
}

/// The next chunk of an event stream as text, or `None` when it ended.
async fn next(body: &mut Body) -> Option<String> {
    let frame = tokio::time::timeout(Duration::from_secs(5), body.frame())
        .await
        .expect("timed out waiting for an event")?;
    let data = frame.unwrap().into_data().ok()?;
    Some(String::from_utf8(data.to_vec()).unwrap())
}

fn removed(id: u64) -> Arc<Envelope> {
    Arc::new(Envelope::new(
        5,
        Event::PlaylistRemoved(RemovedDto {
            id: fp_model::PlaylistId(id),
        }),
    ))
}

#[tokio::test]
async fn a_stream_starts_with_the_state_then_carries_events() {
    let (ctx, tx) = ctx_with(HttpRemoteConfig::default(), 16);
    let res = router(ctx).oneshot(get("/api/v1/events")).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(res.headers()["content-type"], "text/event-stream");
    let mut body = res.into_body();
    let first = next(&mut body).await.unwrap();
    assert!(first.contains("event: state"), "{first}");
    assert!(first.contains("\"players\""), "{first}");
    tx.send(removed(77)).unwrap();
    let second = next(&mut body).await.unwrap();
    assert!(second.contains("event: playlist-removed"), "{second}");
    assert!(second.contains("id: 5"), "{second}");
    assert!(second.contains("{\"id\":77}"), "{second}");
}

#[tokio::test]
async fn topics_filter_the_stream() {
    let (ctx, tx) = ctx_with(HttpRemoteConfig::default(), 16);
    let res = router(ctx)
        .oneshot(get("/api/v1/events?topics=player,position"))
        .await
        .unwrap();
    let mut body = res.into_body();
    assert!(next(&mut body).await.unwrap().contains("event: state"));
    tx.send(removed(1)).unwrap();
    tx.send(Arc::new(Envelope::new(
        6,
        Event::Position(fp_remote::events::PositionDto {
            players: vec![],
            carts: vec![],
        }),
    )))
    .unwrap();
    let got = next(&mut body).await.unwrap();
    assert!(got.contains("event: position"), "{got}");
}

#[tokio::test]
async fn a_client_that_falls_behind_gets_a_resync() {
    let (ctx, tx) = ctx_with(HttpRemoteConfig::default(), 1);
    let res = router(ctx).oneshot(get("/api/v1/events")).await.unwrap();
    let mut body = res.into_body();
    assert!(next(&mut body).await.unwrap().contains("event: state"));
    for n in 0..4 {
        tx.send(removed(n)).unwrap();
    }
    let got = next(&mut body).await.unwrap();
    assert!(got.contains("event: resync"), "{got}");
    assert!(got.contains("\"players\""), "{got}");
}

#[tokio::test]
async fn the_client_limit_is_enforced_and_freed_on_disconnect() {
    let config = HttpRemoteConfig {
        max_event_clients: 1,
        ..Default::default()
    };
    let (ctx, _tx) = ctx_with(config, 16);
    let first = router(ctx.clone())
        .oneshot(get("/api/v1/events"))
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::OK);
    let second = router(ctx.clone())
        .oneshot(get("/api/v1/events"))
        .await
        .unwrap();
    assert_eq!(second.status(), StatusCode::SERVICE_UNAVAILABLE);
    drop(first);
    let third = router(ctx).oneshot(get("/api/v1/events")).await.unwrap();
    assert_eq!(third.status(), StatusCode::OK);
}

#[tokio::test]
async fn stopping_ends_open_streams() {
    let (ctx, _tx) = ctx_with(HttpRemoteConfig::default(), 16);
    let stop = ctx.stop.clone();
    let res = router(ctx).oneshot(get("/api/v1/events")).await.unwrap();
    let mut body = res.into_body();
    assert!(next(&mut body).await.unwrap().contains("event: state"));
    stop.send(true).unwrap();
    assert_eq!(next(&mut body).await, None);
}

#[tokio::test]
async fn the_token_may_come_in_the_query_only_for_events() {
    let config = HttpRemoteConfig {
        token: TOKEN.into(),
        ..Default::default()
    };
    let (ctx, _tx) = ctx_with(config, 16);
    let ok = router(ctx.clone())
        .oneshot(get(&format!("/api/v1/events?token={TOKEN}")))
        .await
        .unwrap();
    assert_eq!(ok.status(), StatusCode::OK);
    let refused = router(ctx.clone())
        .oneshot(get(&format!("/api/v1/state?token={TOKEN}")))
        .await
        .unwrap();
    assert_eq!(refused.status(), StatusCode::UNAUTHORIZED);
    let wrong = router(ctx)
        .oneshot(get("/api/v1/events?token=nope"))
        .await
        .unwrap();
    assert_eq!(wrong.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_stream_opened_after_a_stop_ends_after_the_state() {
    let (ctx, _tx) = ctx_with(HttpRemoteConfig::default(), 16);
    // As `Running::stop` does, with no stream open yet.
    ctx.stop.send_replace(true);
    let res = router(ctx).oneshot(get("/api/v1/events")).await.unwrap();
    let mut body = res.into_body();
    assert!(next(&mut body).await.unwrap().contains("event: state"));
    assert_eq!(next(&mut body).await, None);
}

#[tokio::test]
async fn after_a_resync_old_buffered_events_are_skipped() {
    let (ctx, tx) = ctx_with(HttpRemoteConfig::default(), 2);
    let res = router(ctx).oneshot(get("/api/v1/events")).await.unwrap();
    let mut body = res.into_body();
    assert!(next(&mut body).await.unwrap().contains("event: state"));
    for n in 0..5 {
        tx.send(removed(n)).unwrap();
    }
    assert!(next(&mut body).await.unwrap().contains("event: resync"));
    tx.send(removed(99)).unwrap();
    let after = next(&mut body).await.unwrap();
    assert!(after.contains("{\"id\":99}"), "{after}");
}
