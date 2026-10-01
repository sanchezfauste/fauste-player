//! The OSC socket (remote control spec §4, §5.3): checks the source,
//! decodes, plans and queues commands, and pushes changed values to
//! subscribers after every event and subscription.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use fp_model::{CartPageId, OscRemoteConfig};
use rosc::OscPacket;
use tokio::net::UdpSocket;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{broadcast, oneshot};

use crate::api;
use crate::control::RemoteControl;
use crate::events::Envelope;
use crate::osc::{self, OscRequest, Subscribers};
use crate::throttle::Throttle;

/// The largest UDP payload.
const MAX_PACKET: usize = 65_536;
/// How often expired subscriptions are dropped.
const EXPIRY_CHECK: Duration = Duration::from_secs(1);

pub(crate) async fn run(
    socket: UdpSocket,
    control: Arc<dyn RemoteControl>,
    config: OscRemoteConfig,
    mut events: broadcast::Receiver<Arc<Envelope>>,
    mut stop: oneshot::Receiver<()>,
) {
    let mut buf = vec![0u8; MAX_PACKET];
    let ttl = Duration::from_secs(config.subscription_ttl_secs.into());
    let mut subs = Subscribers::new(config.max_subscribers as usize, ttl);
    let mut shape: Option<(usize, Option<CartPageId>)> = None;
    let log = Throttle::default();
    let mut expiry = tokio::time::interval(EXPIRY_CHECK);
    loop {
        tokio::select! {
            _ = &mut stop => break,
            received = socket.recv_from(&mut buf) => {
                let Ok((len, from)) = received else { continue };
                if !config.allows(from.ip()) {
                    log.note(Some(from.ip()), "OSC source not allowed");
                    continue;
                }
                let Some(packet) = buf.get(..len) else { continue };
                if handle(packet, from, &*control, &mut subs, &log) {
                    push(&socket, &*control, &mut subs, &mut shape, &log).await;
                }
            }
            event = events.recv() => {
                if matches!(event, Err(RecvError::Closed)) {
                    break;
                }
                push(&socket, &*control, &mut subs, &mut shape, &log).await;
            }
            _ = expiry.tick() => subs.expire(Instant::now()),
        }
    }
}

/// Acts on one packet; true when a subscriber was added or renewed.
fn handle(
    packet: &[u8],
    from: SocketAddr,
    control: &dyn RemoteControl,
    subs: &mut Subscribers,
    log: &Throttle,
) -> bool {
    let source = Some(from.ip());
    let messages = match osc::messages(packet) {
        Ok(m) => m,
        Err(why) => {
            log.note(source, why);
            return false;
        }
    };
    let mut subscribed = false;
    for message in messages {
        let model = control.model();
        let to = |port: Option<u16>| SocketAddr::new(from.ip(), port.unwrap_or(from.port()));
        match osc::parse(&message, &model) {
            Ok(OscRequest::Op(op)) => match api::plan(&model, op) {
                Ok(commands) => {
                    for c in commands {
                        if !control.send(c) {
                            log.note(source, "command queue full");
                        }
                    }
                }
                Err(e) => log.note(source, e.code()),
            },
            Ok(OscRequest::Subscribe(port)) => {
                if subs.subscribe(to(port), Instant::now()) {
                    subscribed = true;
                } else {
                    log.note(source, "too many OSC subscribers");
                }
            }
            Ok(OscRequest::Unsubscribe(port)) => subs.unsubscribe(to(port)),
            Err(why) => log.note(source, why),
        }
    }
    subscribed
}

/// Sends every subscriber the values that changed since it was last sent
/// them; a new player count or page shown sends everything again.
async fn push(
    socket: &UdpSocket,
    control: &dyn RemoteControl,
    subs: &mut Subscribers,
    shape: &mut Option<(usize, Option<CartPageId>)>,
    log: &Throttle,
) {
    if subs.is_empty() {
        return;
    }
    let model = control.model();
    let now = (
        model.players.len(),
        model.cartwall.shown_page().map(|p| p.id),
    );
    if *shape != Some(now) {
        subs.reset();
        *shape = Some(now);
    }
    let values = osc::values(&model, &control.playback());
    for (to, messages) in subs.changes(&values) {
        for m in messages {
            let Ok(bytes) = rosc::encoder::encode(&OscPacket::Message(m)) else {
                continue;
            };
            if socket.send_to(&bytes, to).await.is_err() {
                log.note(Some(to.ip()), "OSC send failed");
            }
        }
    }
}
