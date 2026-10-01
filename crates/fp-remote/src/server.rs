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
use tokio::sync::{oneshot, watch};

use crate::control::RemoteControl;
use crate::http::{Ctx, router};

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
    task: tokio::task::JoinHandle<std::io::Result<()>>,
}

impl Running {
    async fn stop(self) {
        let _ = self.shutdown.send(());
        let _ = tokio::time::timeout(SHUTDOWN_GRACE, self.task).await;
    }
}

async fn supervise(
    control: Arc<dyn RemoteControl>,
    status: Arc<ArcSwap<RemoteStatus>>,
    mut stop: watch::Receiver<bool>,
) {
    let mut applied: Option<HttpRemoteConfig> = None;
    let mut running: Option<Running> = None;
    loop {
        let wanted = control.model().config.remote.http.clone();
        if applied.as_ref() != Some(&wanted) {
            if let Some(server) = running.take() {
                server.stop().await;
            }
            let (http, server) = start(&control, &wanted).await;
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
    status.store(Arc::new(RemoteStatus::default()));
}

async fn start(
    control: &Arc<dyn RemoteControl>,
    config: &HttpRemoteConfig,
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
    let app = router(Ctx::new(control.clone(), Arc::new(config.clone())));
    let (shutdown, signal) = oneshot::channel::<()>();
    let task = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(async {
            let _ = signal.await;
        })
        .await
    });
    tracing::info!(%addr, "remote HTTP listening");
    (
        ServerStatus::Listening(addr),
        Some(Running { shutdown, task }),
    )
}
