//! The HTTP/JSON API (remote control spec §3).

mod events;
pub mod guard;
mod handlers;
mod json;

use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::time::Duration;

use axum::Json;
use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::middleware;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, patch, post, put};
use fp_model::HttpRemoteConfig;
use serde_json::json;
use tokio::sync::{broadcast, watch};
use tower::limit::ConcurrencyLimitLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::timeout::TimeoutLayer;

use crate::api::ApiError;
use crate::control::RemoteControl;
use crate::events::Envelope;
use crate::throttle::Throttle;

/// Events buffered per stream before a slow client gets a `resync`.
pub const EVENT_BUFFER: usize = 256;

pub(crate) const EVENTS_PATH: &str = "/api/v1/events";

/// What every handler sees.
#[derive(Clone)]
pub struct Ctx {
    pub control: Arc<dyn RemoteControl>,
    pub config: Arc<HttpRemoteConfig>,
    pub rejections: Arc<Throttle>,
    pub events: broadcast::Sender<Arc<Envelope>>,
    pub event_clients: Arc<AtomicUsize>,
    /// Set to true to end every open event stream.
    pub stop: Arc<watch::Sender<bool>>,
}

impl Ctx {
    pub fn new(control: Arc<dyn RemoteControl>, config: Arc<HttpRemoteConfig>) -> Self {
        Self {
            control,
            config,
            rejections: Arc::default(),
            events: broadcast::channel(EVENT_BUFFER).0,
            event_clients: Arc::default(),
            stop: Arc::new(watch::channel(false).0),
        }
    }

    /// Streams events from the server's publisher.
    pub fn with_events(mut self, events: broadcast::Sender<Arc<Envelope>>) -> Self {
        self.events = events;
        self
    }
}

/// The value of `name` in a query string, percent-decoded.
pub(crate) fn param(query: &str, name: &str) -> Option<String> {
    query.split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        (key == name).then(|| percent_decode(value))
    })
}

fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while let Some(&b) = bytes.get(i) {
        let hex = (b == b'%')
            .then(|| bytes.get(i + 1..i + 3))
            .flatten()
            .and_then(|h| std::str::from_utf8(h).ok())
            .and_then(|h| u8::from_str_radix(h, 16).ok());
        match (b, hex) {
            (_, Some(v)) => {
                out.push(v);
                i += 3;
            }
            (b'+', None) => {
                out.push(b' ');
                i += 1;
            }
            (b, None) => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status()).unwrap_or(StatusCode::BAD_REQUEST);
        let body = json!({ "error": self.code(), "message": self.message() });
        (status, Json(body)).into_response()
    }
}

pub fn router(ctx: Ctx) -> Router {
    use handlers as h;
    let api = Router::new()
        .route("/state", get(h::state))
        .route("/events", get(events::events))
        .route("/players", get(h::players))
        .route("/players/{id}", get(h::player))
        .route("/players/{id}/{action}", post(h::player_action))
        .route("/players/{id}/cue", put(h::set_cue))
        .route("/players/{id}/next", put(h::set_next))
        .route("/players/{id}/cue-entry", post(h::cue_entry))
        .route("/players/{id}/seek", post(h::seek))
        .route("/players/{id}/volume", put(h::volume))
        .route("/players/{id}/mode", put(h::mode))
        .route(
            "/players/{id}/stop-after-current",
            put(h::stop_after_current),
        )
        .route("/players/{id}/playlist", put(h::show_playlist))
        .route("/playlists", get(h::playlists).post(h::create_playlist))
        .route(
            "/playlists/{id}",
            get(h::playlist)
                .patch(h::rename_playlist)
                .delete(h::delete_playlist),
        )
        .route("/playlists/{id}/entries", post(h::insert_entry))
        .route("/entries/{id}", delete(h::remove_entry))
        .route("/entries/{id}/move", post(h::move_entry))
        .route("/entries/{id}/duplicate", post(h::duplicate_entry))
        .route("/cartwall/pages", post(h::create_cart_page))
        .route(
            "/cartwall/pages/{id}",
            patch(h::edit_cart_page).delete(h::delete_cart_page),
        )
        .route("/cartwall/pages/{id}/carts/{index}", put(h::set_cart))
        .route("/tracks/{id}/markers/{kind}", put(h::set_marker))
        .route("/tracks/{id}/markers/reset", post(h::reset_markers))
        .route("/entries/{id}/repeat", put(h::entry_repeat))
        .route("/entries/{id}/stop-after", put(h::entry_stop_after))
        .route("/tracks/{id}", get(h::track))
        .route("/tracks/{id}/cover", get(h::cover))
        .route("/tracks/{id}/peaks", get(h::peaks))
        .route("/cartwall", get(h::cartwall))
        .route("/cartwall/stop-all", post(h::stop_all_carts))
        .route("/cartwall/shown", put(h::show_cart_page))
        .route("/carts/{id}/fire", post(h::fire_cart))
        .route("/carts/{id}/stop", post(h::stop_cart))
        .route("/carts/{id}/cue", put(h::cart_cue));
    let config = ctx.config.clone();
    Router::new()
        .nest("/api/v1", api)
        .fallback(h::not_found)
        .layer(middleware::from_fn_with_state(ctx.clone(), guard::guard))
        .layer(DefaultBodyLimit::max(config.max_body_bytes as usize))
        .layer(cors(&config))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_millis(config.request_timeout_ms.into()),
        ))
        .layer(ConcurrencyLimitLayer::new(IN_FLIGHT_REQUESTS))
        .with_state(ctx)
}

/// Requests served at once; more wait their turn. An engineering bound
/// that keeps a flood from growing memory, not an operator setting.
const IN_FLIGHT_REQUESTS: usize = 64;

/// How long a browser may cache a preflight answer.
const PREFLIGHT_MAX_AGE: Duration = Duration::from_secs(600);

fn cors(config: &HttpRemoteConfig) -> CorsLayer {
    let layer = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .max_age(PREFLIGHT_MAX_AGE);
    if config.cors_origins.iter().any(|o| o == "*") {
        layer.allow_origin(AllowOrigin::any())
    } else {
        layer.allow_origin(AllowOrigin::list(
            config
                .cors_origins
                .iter()
                .filter_map(|o| HeaderValue::from_str(o).ok()),
        ))
    }
}
