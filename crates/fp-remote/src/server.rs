//! The remote thread (remote control spec §7): a tokio `current_thread`
//! runtime that follows `config.remote` in the model snapshot, (re)starts
//! the HTTP server when it changes, and publishes its status. It never
//! sends a command by itself (rule 10).

use std::net::SocketAddr;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use arc_swap::ArcSwap;
use fp_model::{HttpRemoteConfig, OscRemoteConfig};
use tokio::net::{TcpListener, UdpSocket};
use tokio::sync::{broadcast, oneshot, watch};

use crate::control::RemoteControl;
use crate::events::{self, Envelope, Event};
use crate::http::{Ctx, EVENT_BUFFER, router};

/// How often the configuration in the snapshot is looked at.
const CONFIG_POLL: Duration = Duration::from_millis(250);
/// How often a server whose address could not be bound tries again.
const BIND_RETRY: Duration = Duration::from_secs(2);
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
    pub osc: ServerStatus,
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

    /// Where the status is published, for a reader that outlives a borrow
    /// of the handle (the interface).
    pub fn status_cell(&self) -> Arc<ArcSwap<RemoteStatus>> {
        self.status.clone()
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
            let failed = ServerStatus::Error(ServerError::Runtime(e.to_string()));
            status.store(Arc::new(RemoteStatus {
                http: failed.clone(),
                osc: failed,
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
            // `send_replace` keeps the value even with no stream open, so a
            // stream that starts now sees it.
            streams.send_replace(true);
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
    let mut applied_osc: Option<OscRemoteConfig> = None;
    let mut osc: Option<Running> = None;
    let mut current = RemoteStatus::default();
    let mut tried = tokio::time::Instant::now();
    loop {
        let remote = control.model().config.remote.clone();
        // A port that was busy may have been freed: try it again.
        if tried.elapsed() >= BIND_RETRY {
            tried = tokio::time::Instant::now();
            if matches!(current.http, ServerStatus::Error(ServerError::Bind(_))) {
                applied = None;
            }
            if matches!(current.osc, ServerStatus::Error(ServerError::Bind(_))) {
                applied_osc = None;
            }
        }
        if applied.as_ref() != Some(&remote.http) {
            if let Some(server) = running.take() {
                server.stop().await;
            }
            let (http, server) = start(&control, &remote.http, &events).await;
            current.http = http;
            status.store(Arc::new(current.clone()));
            running = server;
            applied = Some(remote.http);
        }
        if applied_osc.as_ref() != Some(&remote.osc) {
            if let Some(server) = osc.take() {
                server.stop().await;
            }
            let (state, server) = start_osc(&control, &remote.osc, &events).await;
            current.osc = state;
            status.store(Arc::new(current.clone()));
            osc = server;
            applied_osc = Some(remote.osc);
        }
        tokio::select! {
            _ = stop.changed() => break,
            () = tokio::time::sleep(CONFIG_POLL) => {}
        }
    }
    for server in [running.take(), osc.take()].into_iter().flatten() {
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
                let _ = events.send(Arc::new(Envelope::new(playback.revision, event)));
            }
            last = model.clone();
        }
        let every = Duration::from_millis(model.config.remote.events.position_interval_ms.into());
        if last_position.elapsed() >= every {
            // Keep the cadence: advance by the interval, unless far behind.
            last_position += every;
            if last_position.elapsed() >= every {
                last_position = tokio::time::Instant::now();
            }
            if let Some(p) = events::position(&model, &playback) {
                let _ = events.send(Arc::new(Envelope::new(
                    playback.revision,
                    Event::Position(p),
                )));
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

async fn start_osc(
    control: &Arc<dyn RemoteControl>,
    config: &OscRemoteConfig,
    events: &broadcast::Sender<Arc<Envelope>>,
) -> (ServerStatus, Option<Running>) {
    if !config.enabled {
        return (ServerStatus::Off, None);
    }
    let Some(ip) = config.bind_addr() else {
        return (ServerStatus::Error(ServerError::InvalidBind), None);
    };
    let socket = match UdpSocket::bind((ip, config.port)).await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(bind = %ip, port = config.port, error = %e, "remote OSC could not listen");
            return (ServerStatus::Error(ServerError::Bind(e.to_string())), None);
        }
    };
    let addr = socket
        .local_addr()
        .unwrap_or_else(|_| SocketAddr::new(ip, config.port));
    let (shutdown, signal) = oneshot::channel();
    let task = tokio::spawn(crate::osc_server::run(
        socket,
        control.clone(),
        config.clone(),
        events.subscribe(),
        signal,
    ));
    tracing::info!(%addr, "remote OSC listening");
    (
        ServerStatus::Listening(addr),
        Some(Running {
            shutdown,
            task,
            streams: None,
        }),
    )
}
