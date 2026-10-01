//! Who may call the API (remote control spec §6.2): a bearer token when one
//! is set, no foreign `Origin`, and on a loopback bind a loopback `Host`, so
//! a web page in the operator's browser cannot drive the local server.

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use axum::extract::{ConnectInfo, Request, State};
use axum::http::{HeaderMap, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use fp_model::HttpRemoteConfig;

use super::Ctx;
use crate::api::ApiError;

/// One log line per source and per this long, so an attack cannot flood the log.
const LOG_EVERY: Duration = Duration::from_secs(1);
/// Sources remembered for log throttling before old ones are forgotten.
const LOG_SOURCES: usize = 256;

pub fn check(config: &HttpRemoteConfig, headers: &HeaderMap) -> Result<(), ApiError> {
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
        let given = headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));
        if !given.is_some_and(|t| same(t.as_bytes(), config.token.as_bytes())) {
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

/// Throttles the log lines of refused requests.
#[derive(Default)]
pub struct RejectLog {
    last: Mutex<HashMap<Option<IpAddr>, Instant>>,
}

impl RejectLog {
    fn note(&self, source: Option<IpAddr>, error: &ApiError) {
        let now = Instant::now();
        let mut last = self.last.lock().unwrap_or_else(PoisonError::into_inner);
        if last.len() >= LOG_SOURCES {
            last.retain(|_, at| now.duration_since(*at) < LOG_EVERY);
        }
        let due = last
            .get(&source)
            .is_none_or(|at| now.duration_since(*at) >= LOG_EVERY);
        if due {
            last.insert(source, now);
            tracing::warn!(?source, code = error.code(), "remote request refused");
        }
    }
}

pub async fn guard(State(ctx): State<Ctx>, req: Request, next: Next) -> Response {
    match check(&ctx.config, req.headers()) {
        Ok(()) => next.run(req).await,
        Err(error) => {
            let source = req
                .extensions()
                .get::<ConnectInfo<SocketAddr>>()
                .map(|c| c.0.ip());
            ctx.rejections.note(source, &error);
            error.into_response()
        }
    }
}
