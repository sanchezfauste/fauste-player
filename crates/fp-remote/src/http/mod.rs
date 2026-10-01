//! The HTTP/JSON API (remote control spec §3).

mod handlers;
mod json;

use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use fp_model::HttpRemoteConfig;
use serde_json::json;

use crate::api::ApiError;
use crate::control::RemoteControl;

/// What every handler sees.
#[derive(Clone)]
pub struct Ctx {
    pub control: Arc<dyn RemoteControl>,
    pub config: Arc<HttpRemoteConfig>,
}

impl Ctx {
    pub fn new(control: Arc<dyn RemoteControl>, config: Arc<HttpRemoteConfig>) -> Self {
        Self { control, config }
    }
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
        .route("/playlists", get(h::playlists))
        .route("/playlists/{id}", get(h::playlist))
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
    Router::new()
        .nest("/api/v1", api)
        .fallback(h::not_found)
        .with_state(ctx)
}
