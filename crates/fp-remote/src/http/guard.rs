//! Who may call the API (remote control spec §6.2): a bearer token when one
//! is set, no foreign `Origin`, and on a loopback bind a loopback `Host`, so
//! a web page in the operator's browser cannot drive the local server.

use std::net::{IpAddr, SocketAddr};

use axum::extract::{ConnectInfo, Request, State};
use axum::http::{HeaderMap, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use fp_model::HttpRemoteConfig;

use super::Ctx;
use crate::api::ApiError;

/// `query_token` is the `token` query parameter, offered only on the event
/// stream (browsers' `EventSource` cannot set headers).
pub fn check(
    config: &HttpRemoteConfig,
    headers: &HeaderMap,
    query_token: Option<&str>,
) -> Result<(), ApiError> {
    if let Some(origin) = headers.get(header::ORIGIN) {
        let allowed = origin
            .to_str()
            .is_ok_and(|o| config.cors_origins.iter().any(|c| c == "*" || c == o));
        if !allowed {
            return Err(ApiError::ForbiddenOrigin);
        }
    }
    if config.bind_addr().is_some_and(|ip| ip.is_loopback()) {
        let host = headers.get(header::HOST).and_then(|h| h.to_str().ok());
        if !host.is_some_and(host_is_loopback) {
            return Err(ApiError::ForbiddenOrigin);
        }
    }
    if !config.token.is_empty() {
        let token = config.token.as_bytes();
        let given = headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));
        let header_ok = given.is_some_and(|t| same(t.as_bytes(), token));
        let query_ok = query_token.is_some_and(|t| same(t.as_bytes(), token));
        if !(header_ok || query_ok) {
            return Err(ApiError::Unauthorized);
        }
    }
    Ok(())
}

/// `localhost` or a loopback address, with or without a port.
fn host_is_loopback(host: &str) -> bool {
    let name = match host.strip_prefix('[') {
        Some(rest) => rest.split(']').next().unwrap_or_default(),
        None => host.rsplit_once(':').map_or(host, |(name, _)| name),
    };
    name.eq_ignore_ascii_case("localhost")
        || name.parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback())
}

/// Compares without stopping at the first difference.
fn same(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

pub async fn guard(State(ctx): State<Ctx>, req: Request, next: Next) -> Response {
    let query_token = (req.uri().path() == super::EVENTS_PATH)
        .then(|| req.uri().query().and_then(|q| super::param(q, "token")))
        .flatten();
    match check(&ctx.config, req.headers(), query_token.as_deref()) {
        Ok(()) => next.run(req).await,
        Err(error) => {
            let source = req
                .extensions()
                .get::<ConnectInfo<SocketAddr>>()
                .map(|c| c.0.ip());
            ctx.rejections.note(source, error.code());
            error.into_response()
        }
    }
}
