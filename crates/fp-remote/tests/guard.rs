#![allow(clippy::unwrap_used)]
mod support;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{HeaderMap, HeaderValue, Request, StatusCode};
use fp_model::HttpRemoteConfig;
use fp_remote::api::ApiError;
use fp_remote::http::guard::check;
use fp_remote::http::{Ctx, router};
use support::{FakeControl, demo_state};
use tower::ServiceExt;

const TOKEN: &str = "0123456789abcdef0123";

fn headers(pairs: &[(&'static str, &str)]) -> HeaderMap {
    let mut h = HeaderMap::new();
    for (k, v) in pairs {
        h.insert(*k, HeaderValue::from_str(v).unwrap());
    }
    h
}

fn local() -> HttpRemoteConfig {
    HttpRemoteConfig::default()
}

fn lan() -> HttpRemoteConfig {
    HttpRemoteConfig {
        bind: "0.0.0.0".into(),
        token: TOKEN.into(),
        ..Default::default()
    }
}

#[test]
fn locally_without_a_token_any_loopback_host_passes() {
    for host in [
        "127.0.0.1:7380",
        "localhost:7380",
        "LOCALHOST",
        "[::1]:7380",
        "127.1.2.3",
    ] {
        assert_eq!(
            check(&local(), &headers(&[("host", host)])),
            Ok(()),
            "{host}"
        );
    }
}

#[test]
fn locally_a_foreign_host_is_refused() {
    for host in ["evil.example", "evil.example:7380", "192.168.1.10:7380", ""] {
        assert_eq!(
            check(&local(), &headers(&[("host", host)])),
            Err(ApiError::ForbiddenOrigin),
            "{host:?}"
        );
    }
    assert_eq!(
        check(&local(), &HeaderMap::new()),
        Err(ApiError::ForbiddenOrigin)
    );
}

#[test]
fn a_foreign_origin_is_refused_whatever_the_bind() {
    let h = headers(&[
        ("host", "127.0.0.1:7380"),
        ("origin", "https://evil.example"),
    ]);
    assert_eq!(check(&local(), &h), Err(ApiError::ForbiddenOrigin));
    let h = headers(&[
        ("host", "10.0.0.2:7380"),
        ("origin", "https://evil.example"),
        ("authorization", &format!("Bearer {TOKEN}")),
    ]);
    assert_eq!(check(&lan(), &h), Err(ApiError::ForbiddenOrigin));
}

#[test]
fn a_listed_origin_and_the_wildcard_pass() {
    let mut c = local();
    c.cors_origins = vec!["https://studio.example".into()];
    let h = headers(&[
        ("host", "127.0.0.1:7380"),
        ("origin", "https://studio.example"),
    ]);
    assert_eq!(check(&c, &h), Ok(()));
    c.cors_origins = vec!["*".into()];
    let h = headers(&[
        ("host", "127.0.0.1:7380"),
        ("origin", "https://any.example"),
    ]);
    assert_eq!(check(&c, &h), Ok(()));
}

#[test]
fn with_a_token_the_bearer_must_match_and_any_host_is_fine() {
    let ok = headers(&[
        ("host", "studio-pc:7380"),
        ("authorization", &format!("Bearer {TOKEN}")),
    ]);
    assert_eq!(check(&lan(), &ok), Ok(()));
    for auth in [
        "",
        "Bearer",
        "Bearer wrong",
        "Basic abc",
        &format!("bearer {TOKEN}x"),
    ] {
        let h = headers(&[("host", "studio-pc:7380"), ("authorization", auth)]);
        assert_eq!(check(&lan(), &h), Err(ApiError::Unauthorized), "{auth:?}");
    }
    let h = headers(&[("host", "studio-pc:7380")]);
    assert_eq!(check(&lan(), &h), Err(ApiError::Unauthorized));
}

#[test]
fn a_local_server_with_a_token_also_checks_host_and_token() {
    let mut c = local();
    c.token = TOKEN.into();
    let h = headers(&[("host", "127.0.0.1:7380")]);
    assert_eq!(check(&c, &h), Err(ApiError::Unauthorized));
    let h = headers(&[
        ("host", "evil.example"),
        ("authorization", &format!("Bearer {TOKEN}")),
    ]);
    assert_eq!(check(&c, &h), Err(ApiError::ForbiddenOrigin));
}

fn app(config: HttpRemoteConfig) -> axum::Router {
    router(Ctx::new(FakeControl::new(demo_state()), Arc::new(config)))
}

#[tokio::test]
async fn the_router_enforces_the_guard() {
    let req = Request::get("/api/v1/state")
        .header("host", "evil.example")
        .body(Body::empty());
    let res = app(local()).oneshot(req.unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    let req = Request::get("/api/v1/state")
        .header("host", "pc:7380")
        .body(Body::empty());
    let res = app(lan()).oneshot(req.unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn an_oversized_body_is_413() {
    let mut c = local();
    c.max_body_bytes = 1024;
    let body = format!(r#"{{"fader":0.5,"pad":"{}"}}"#, "x".repeat(4096));
    let req = Request::put("/api/v1/players/1/volume")
        .header("host", "127.0.0.1:7380")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();
    let res = app(c).oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn a_preflight_from_a_listed_origin_is_answered() {
    let mut c = local();
    c.cors_origins = vec!["https://studio.example".into()];
    let req = Request::options("/api/v1/players/1/volume")
        .header("host", "127.0.0.1:7380")
        .header("origin", "https://studio.example")
        .header("access-control-request-method", "PUT")
        .header("access-control-request-headers", "content-type")
        .body(Body::empty())
        .unwrap();
    let res = app(c).oneshot(req).await.unwrap();
    assert!(res.status().is_success());
    assert_eq!(
        res.headers().get("access-control-allow-origin").unwrap(),
        "https://studio.example"
    );
}
