//! `GET /events` (remote control spec §5.2): a full `state`, then resource
//! events; `resync` with a full state when the client fell behind. The
//! publisher never waits for a client.

use std::convert::Infallible;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use axum::extract::{RawQuery, State};
use axum::response::sse::{Event as SseEvent, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use futures_util::stream;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{broadcast, watch};

use super::Ctx;
use crate::api::ApiError;
use crate::dto;
use crate::events::Envelope;

/// A comment line this often keeps proxies and NAT from closing the stream.
const KEEP_ALIVE: Duration = Duration::from_secs(15);

/// One of `max_event_clients`; given back when the stream is dropped.
struct Slot(Arc<AtomicUsize>);

impl Drop for Slot {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

fn take_slot(ctx: &Ctx) -> Option<Slot> {
    let taken = ctx.event_clients.fetch_add(1, Ordering::AcqRel);
    let slot = Slot(ctx.event_clients.clone());
    (taken < ctx.config.max_event_clients as usize).then_some(slot)
}

struct Feed {
    ctx: Ctx,
    rx: broadcast::Receiver<Arc<Envelope>>,
    stop: watch::Receiver<bool>,
    topics: Option<Vec<String>>,
    started: bool,
    _slot: Slot,
}

impl Feed {
    fn wants(&self, name: &str) -> bool {
        self.topics
            .as_ref()
            .is_none_or(|t| t.iter().any(|x| x == name))
    }

    /// The whole state, as `state` or `resync`.
    fn full(&self, name: &'static str) -> SseEvent {
        let playback = self.ctx.control.playback();
        let state = dto::state(&self.ctx.control.model(), &playback);
        SseEvent::default()
            .event(name)
            .id(playback.revision.to_string())
            .data(serde_json::to_string(&state).unwrap_or_default())
    }
}

pub async fn events(State(ctx): State<Ctx>, RawQuery(query): RawQuery) -> Response {
    let Some(slot) = take_slot(&ctx) else {
        return ApiError::Busy.into_response();
    };
    let topics = query
        .as_deref()
        .and_then(|q| super::param(q, "topics"))
        .map(|t| t.split(',').map(str::to_owned).collect());
    let feed = Feed {
        rx: ctx.events.subscribe(),
        stop: ctx.stop.subscribe(),
        topics,
        started: false,
        _slot: slot,
        ctx,
    };
    let stream = stream::unfold(feed, |mut f| async move {
        if !f.started {
            f.started = true;
            let first = f.full("state");
            return Some((Ok::<_, Infallible>(first), f));
        }
        loop {
            tokio::select! {
                _ = f.stop.changed() => return None,
                received = f.rx.recv() => match received {
                    Ok(env) => {
                        if f.wants(env.event.name()) {
                            let e = SseEvent::default()
                                .event(env.event.name())
                                .id(env.revision.to_string())
                                .data(env.json.as_str());
                            return Some((Ok(e), f));
                        }
                    }
                    Err(RecvError::Lagged(_)) => {
                        let e = f.full("resync");
                        return Some((Ok(e), f));
                    }
                    Err(RecvError::Closed) => return None,
                },
            }
        }
    });
    Sse::new(stream)
        .keep_alive(KeepAlive::new().interval(KEEP_ALIVE).text("keepalive"))
        .into_response()
}
