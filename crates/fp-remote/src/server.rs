//! The remote thread (remote control spec §7): a tokio `current_thread`
//! runtime that follows `config.remote` in the model snapshot, (re)starts
//! the HTTP server when it changes, and publishes its status. It never
//! sends a command by itself (rule 10).

use std::net::SocketAddr;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use arc_swap::ArcSwap;
use fp_model::HttpRemoteConfig;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, oneshot, watch};

use crate::control::RemoteControl;
use crate::events::{self, Envelope, Event};
use crate::http::{Ctx, EVENT_BUFFER, router};

/// How often the configuration in the snapshot is looked at.
const CONFIG_POLL: Duration = Duration::from_millis(250);
/// How long open requests may take to finish when the server stops.
const SHUTDOWN_GRACE: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerError {
    /// `bind` is not an IP address.
    InvalidBind,
    /// Listening beyond loopback needs a token.
    TokenRequired,
    /// The address could not be bound (in use, no permission).
    Bind(String),
    /// The runtime could not start.
    Runtime(String),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ServerStatus {
    #[default]
    Off,
    Listening(SocketAddr),
    Error(ServerError),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RemoteStatus {
    pub http: ServerStatus,
}

/// Owns the remote thread; dropping it stops the servers and joins.
pub struct RemoteHandle {
    status: Arc<ArcSwap<RemoteStatus>>,
    stop: watch::Sender<bool>,
    thread: Option<JoinHandle<()>>,
}

impl RemoteHandle {
    pub fn status(&self) -> RemoteStatus {
        (**self.status.load()).clone()
    }
}

impl Drop for RemoteHandle {
    fn drop(&mut self) {
        let _ = self.stop.send(true);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Starts the remote thread. It listens only once `config.remote` asks for it.
pub fn spawn(control: Arc<dyn RemoteControl>) -> std::io::Result<RemoteHandle> {
    let status = Arc::new(ArcSwap::from_pointee(RemoteStatus::default()));
    let (stop, stopped) = watch::channel(false);
    let shared = status.clone();
    let thread = std::thread::Builder::new()
        .name("fp-remote".to_owned())
        .spawn(move || run(control, shared, stopped))?;
    Ok(RemoteHandle {
        status,
        stop,
        thread: Some(thread),
    })
}

fn run(
    control: Arc<dyn RemoteControl>,
    status: Arc<ArcSwap<RemoteStatus>>,
    stop: watch::Receiver<bool>,
) {
    match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt.block_on(supervise(control, status, stop)),
        Err(e) => {
            tracing::error!(error = %e, "remote control could not start");
            status.store(Arc::new(RemoteStatus {
                http: ServerStatus::Error(ServerError::Runtime(e.to_string())),
            }));
        }
    }
}

/// A server that is running, and how to stop it.
struct Running {
    shutdown: oneshot::Sender<()>,
    task: tokio::task::JoinHandle<()>,
    /// Ends the server's open event streams.
    streams: Option<Arc<watch::Sender<bool>>>,
}

impl Running {
    async fn stop(self) {
        if let Some(streams) = &self.streams {
            let _ = streams.send(true);
        }
        let _ = self.shutdown.send(());
        let mut task = self.task;
        if tokio::time::timeout(SHUTDOWN_GRACE, &mut task)
            .await
            .is_err()
        {
            task.abort();
        }
    }
}

async fn supervise(
    control: Arc<dyn RemoteControl>,
    status: Arc<ArcSwap<RemoteStatus>>,
    mut stop: watch::Receiver<bool>,
) {
    let (events, _) = broadcast::channel::<Arc<Envelope>>(EVENT_BUFFER);
    let publisher = tokio::spawn(publish(control.clone(), events.clone()));
    let mut applied: Option<HttpRemoteConfig> = None;
    let mut running: Option<Running> = None;
    loop {
        let wanted = control.model().config.remote.http.clone();
        if applied.as_ref() != Some(&wanted) {
            if let Some(server) = running.take() {
                server.stop().await;
            }
            let (http, server) = start(&control, &wanted, &events).await;
            status.store(Arc::new(RemoteStatus { http }));
            running = server;
            applied = Some(wanted);
        }
        tokio::select! {
            _ = stop.changed() => break,
            () = tokio::time::sleep(CONFIG_POLL) => {}
        }
    }
    if let Some(server) = running.take() {
        server.stop().await;
    }
    publisher.abort();
    status.store(Arc::new(RemoteStatus::default()));
}

/// How often the publisher looks for a new snapshot.
const PUBLISH_EVERY: Duration = Duration::from_millis(50);

/// Diffs snapshots into events, and reports positions every
/// `position_interval_ms` while something plays. Idle without listeners.
async fn publish(control: Arc<dyn RemoteControl>, events: broadcast::Sender<Arc<Envelope>>) {
    let mut last = control.model();
    let mut last_position = tokio::time::Instant::now();
    let mut tick = tokio::time::interval(PUBLISH_EVERY);
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tick.tick().await;
        let model = control.model();
        if events.receiver_count() == 0 {
            last = model;
            continue;
        }
        let playback = control.playback();
        if !Arc::ptr_eq(&last, &model) {
            for event in events::diff(&last, &model, &playback) {
                let _ = events.send(Arc::new(Envelope {
                    revision: playback.revision,
                    event,
                }));
            }
            last = model.clone();
        }
        let every = Duration::from_millis(model.config.remote.events.position_interval_ms.into());
        if last_position.elapsed() >= every {
            last_position = tokio::time::Instant::now();
            if let Some(p) = events::position(&model, &playback) {
                let _ = events.send(Arc::new(Envelope {
                    revision: playback.revision,
                    event: Event::Position(p),
                }));
            }
        }
    }
}

async fn start(
    control: &Arc<dyn RemoteControl>,
    config: &HttpRemoteConfig,
    events: &broadcast::Sender<Arc<Envelope>>,
) -> (ServerStatus, Option<Running>) {
    if !config.enabled {
        return (ServerStatus::Off, None);
    }
    let Some(ip) = config.bind_addr() else {
        return (ServerStatus::Error(ServerError::InvalidBind), None);
    };
    if !ip.is_loopback() && config.token.is_empty() {
        tracing::error!(bind = %ip, "remote HTTP not started: a token is required beyond this computer");
        return (ServerStatus::Error(ServerError::TokenRequired), None);
    }
    let listener = match TcpListener::bind((ip, config.port)).await {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!(bind = %ip, port = config.port, error = %e, "remote HTTP could not listen");
            return (ServerStatus::Error(ServerError::Bind(e.to_string())), None);
        }
    };
    let addr = listener
        .local_addr()
        .unwrap_or_else(|_| SocketAddr::new(ip, config.port));
    let ctx = Ctx::new(control.clone(), Arc::new(config.clone())).with_events(events.clone());
    let streams = ctx.stop.clone();
    let app = router(ctx);
    let (shutdown, signal) = oneshot::channel::<()>();
    let task = tokio::spawn(async move {
        let served = axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(async {
            let _ = signal.await;
        })
        .await;
        if let Err(e) = served {
            tracing::warn!(error = %e, "remote HTTP stopped with an error");
        }
    });
    tracing::info!(%addr, "remote HTTP listening");
    (
        ServerStatus::Listening(addr),
        Some(Running {
            shutdown,
            task,
            streams: Some(streams),
        }),
    )
}
