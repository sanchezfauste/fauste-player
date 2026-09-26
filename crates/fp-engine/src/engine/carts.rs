//! The cartwall on air (Phase 2 spec P2.4). Carts are ordinary sources on
//! the cartwall's Main (or Cue) bus, decoded by their own worker. Ends and
//! loops are exact because the worker bounds each source at its cue-out.

use super::*;
use crate::worker::LoadOptions;
use fp_model::{CartId, CartRequest};

/// What the UI shows for a playing cart.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CartTelemetry {
    /// Position in the file, wrapping within `[cue_in, cue_out)` when looped.
    pub position_secs: f64,
    pub peak: f32,
}

/// Carts on air with their telemetry, and the cart pre-listening with its
/// position.
pub type CartwallTelemetry = (Vec<(CartId, CartTelemetry)>, Option<(CartId, f64)>);

pub(super) struct CartSource {
    cart: CartId,
    key: SourceKey,
    bus: BusKey,
    slot: usize,
    from_secs: f64,
    looped: bool,
    shared: Arc<SourceShared>,
    start: StartState,
    /// The worker failed: report `CartFailed` when the buffered audio ends.
    failed: bool,
}

pub(super) struct CartwallRuntime {
    worker: PlayerWorker,
    main: (BusKey, u16),
    cue: Option<(BusKey, u16)>,
    volume: Arc<AtomicF32>,
    playing: Vec<CartSource>,
    cue_src: Option<CartSource>,
    /// Stopped with a de-click ramp; released when the mixer finishes them.
    stopping: Vec<CartSource>,
}

impl CartwallRuntime {
    fn all(&self) -> impl Iterator<Item = &CartSource> {
        self.playing
            .iter()
            .chain(self.cue_src.iter())
            .chain(self.stopping.iter())
    }
}

impl Engine {
    /// Creates the cartwall worker and opens its buses on first use.
    fn ensure_cartwall(&mut self, now: Instant) -> bool {
        if self.cartwall.is_none() {
            let routes = self.settings.cartwall_routes.clone();
            let main = match &routes.main {
                Some(route) => self.route_target(route),
                None => (self.default_output(), 0),
            };
            let cue = routes
                .cue
                .as_ref()
                .and_then(|route| self.cue_target(route, &main));
            let ready = self
                .settings
                .frames(self.settings.tuning.ready_threshold_ms) as usize;
            let worker = match PlayerWorker::spawn(
                "fp-cartwall",
                self.opener.clone(),
                self.settings.sample_rate,
                ready,
                self.failures_tx.clone(),
            ) {
                Ok(w) => w,
                Err(e) => {
                    tracing::error!("cannot spawn the cartwall worker: {e}");
                    return false;
                }
            };
            self.cartwall = Some(CartwallRuntime {
                worker,
                main,
                cue,
                volume: Arc::new(AtomicF32::new(1.0)),
                playing: Vec::new(),
                cue_src: None,
                stopping: Vec::new(),
            });
        }
        let keys: Vec<BusKey> = self
            .cartwall
            .iter()
            .flat_map(|c| {
                std::iter::once(c.main.0.clone()).chain(c.cue.iter().map(|k| k.0.clone()))
            })
            .collect();
        for key in keys {
            self.ensure_bus(&key, now);
        }
        true
    }

    /// Cart sources attached to `key` (for the derived mixer capacity).
    pub(super) fn cart_sources_on(&self, key: &BusKey) -> usize {
        self.cartwall.as_ref().map_or(0, |c| {
            let routed = &c.main.0 == key || c.cue.as_ref().is_some_and(|k| &k.0 == key);
            if !routed {
                return 0;
            }
            // Slots still held by released sources count until the mixer
            // returns them; one more makes room for the cart being fired.
            let held = self.buses.get(key).map_or(0, Bus::used_slots);
            held.max(c.all().filter(|s| &s.bus == key).count()) + 1
        })
    }

    fn cart_source(&mut self, request: &CartRequest, cue: bool) -> Result<CartSource, AttachError> {
        let c = self.cartwall.as_ref().ok_or(AttachError::NoPlayer)?;
        let (bus_key, channel) = if cue {
            c.cue.clone().ok_or(AttachError::NoRoute)?
        } else {
            c.main.clone()
        };
        let volume = c.volume.clone();
        let ring = ((self.settings.tuning.prebuffer_secs * f64::from(self.settings.sample_rate))
            as usize)
            .max(1024);
        let bus = self.buses.get_mut(&bus_key).ok_or(AttachError::NoRoute)?;
        if bus.command_room() < 2 {
            self.dropped_commands += 1;
            return Err(AttachError::QueueFull);
        }
        let Some(slot) = bus.alloc_slot() else {
            self.slot_exhaustions += 1;
            tracing::error!(cart = ?request.cart, bus = ?bus_key, "no free mixer slot");
            return Err(AttachError::NoSlot);
        };
        let (producer, consumer) = source_pair(ring);
        let shared = consumer.shared.clone();
        if !bus.send(BusCommand::Attach {
            slot,
            source: consumer,
            volume,
            first_channel: channel,
        }) {
            return Err(AttachError::QueueFull);
        }
        self.next_key += 1;
        let key = SourceKey(self.next_key);
        let options = LoadOptions {
            until_secs: Some(request.until_secs).filter(|u| u.is_finite()),
            looped: request.looped,
            rate: Some(self.rate_of(&bus_key)),
        };
        if let Some(c) = self.cartwall.as_ref() {
            c.worker.load_with(
                key,
                request.path.clone(),
                request.from_secs,
                producer,
                options,
            );
        }
        Ok(CartSource {
            cart: request.cart,
            key,
            bus: bus_key,
            slot,
            from_secs: request.from_secs,
            looped: request.looped,
            shared,
            start: StartState::WhenReady { fade_in: false },
            failed: false,
        })
    }

    pub(super) fn start_cart(&mut self, request: &CartRequest, now: Instant) {
        if !self.ensure_cartwall(now) {
            // Not the file's fault: the cart simply does not play.
            self.events
                .push(EngineEvent::CartEnded { cart: request.cart });
            return;
        }
        // A cart fired again while still on air restarts it.
        self.stop_cart(request.cart);
        match self.cart_source(request, false) {
            Ok(source) => {
                if let Some(c) = self.cartwall.as_mut() {
                    c.playing.push(source);
                }
            }
            Err(_) => self
                .events
                .push(EngineEvent::CartEnded { cart: request.cart }),
        }
    }

    pub(super) fn stop_cart(&mut self, cart: CartId) {
        let taken = self.cartwall.as_mut().and_then(|c| {
            c.playing
                .iter()
                .position(|s| s.cart == cart)
                .map(|i| c.playing.remove(i))
        });
        if let Some(source) = taken {
            self.stop_cart_source(source);
        }
    }

    pub(super) fn start_cart_cue(&mut self, request: &CartRequest, now: Instant) {
        if !self.ensure_cartwall(now) {
            self.events.push(EngineEvent::CartCueEnded);
            return;
        }
        self.stop_cart_cue();
        match self.cart_source(request, true) {
            Ok(source) => {
                if let Some(c) = self.cartwall.as_mut() {
                    c.cue_src = Some(source);
                }
            }
            // No Cue output: nothing can be heard, end it at once.
            Err(_) => self.events.push(EngineEvent::CartCueEnded),
        }
    }

    pub(super) fn stop_cart_cue(&mut self) {
        if let Some(source) = self.cartwall.as_mut().and_then(|c| c.cue_src.take()) {
            self.stop_cart_source(source);
        }
    }

    /// De-click and stop a started source (released when the mixer finishes
    /// it); a source that never started is released at once.
    fn stop_cart_source(&mut self, source: CartSource) {
        self.send(&source.bus, BusCommand::Cancel { slot: source.slot });
        // A requested start may already be audible (its Started event is
        // not polled yet): fade it like a started one.
        if !matches!(source.start, StartState::Started | StartState::Requested) {
            self.release_cart(&source);
            return;
        }
        let frames = self.frames_on(&source.bus, self.settings.tuning.declick_ms);
        let now = self.now_frame(&source.bus);
        self.send(
            &source.bus,
            BusCommand::Ramp {
                slot: source.slot,
                to: 0.0,
                frames: u32::try_from(frames).unwrap_or(u32::MAX),
                curve: Curve::Linear,
                at_frame: now,
            },
        );
        self.send(
            &source.bus,
            BusCommand::StopAt {
                slot: source.slot,
                at_frame: now + frames,
            },
        );
        if let Some(c) = self.cartwall.as_mut() {
            c.stopping.push(source);
        }
    }

    fn release_cart(&mut self, source: &CartSource) {
        self.send(&source.bus, BusCommand::Detach { slot: source.slot });
        if let Some(c) = self.cartwall.as_ref() {
            c.worker.drop_source(source.key);
        }
    }

    /// Handles a worker failure if it belongs to a cart source.
    pub(super) fn cart_failure(&mut self, failure: &WorkerFailure) -> bool {
        let Some(c) = self.cartwall.as_mut() else {
            return false;
        };
        if let Some(s) = c
            .playing
            .iter_mut()
            .chain(c.cue_src.iter_mut())
            .find(|s| s.key == failure.key)
        {
            tracing::warn!(cart = ?s.cart, error = %failure.error, "cart source failed");
            if !matches!(s.start, StartState::WhenReady { .. }) && !s.shared.is_drained() {
                // Let what is buffered play; report at the end.
                s.failed = true;
                return true;
            }
            let cart = s.cart;
            let is_cue = c.cue_src.as_ref().is_some_and(|q| q.key == failure.key);
            let taken = if is_cue {
                c.cue_src.take()
            } else {
                c.playing
                    .iter()
                    .position(|p| p.key == failure.key)
                    .map(|i| c.playing.remove(i))
            };
            if let Some(source) = taken {
                self.release_cart(&source);
            }
            self.events.push(if is_cue {
                EngineEvent::CartCueEnded
            } else {
                EngineEvent::CartFailed { cart }
            });
            return true;
        }
        c.stopping.iter().any(|s| s.key == failure.key)
    }

    /// Handles a mixer event if it belongs to a cart source.
    pub(super) fn cart_bus_event(&mut self, bus: &BusKey, event: &BusEvent) -> bool {
        let Some(c) = self.cartwall.as_mut() else {
            return false;
        };
        let (BusEvent::Started { slot, .. } | BusEvent::Finished { slot, .. }) = *event;
        let is = |s: &CartSource| &s.bus == bus && s.slot == slot;
        match event {
            BusEvent::Started { .. } => {
                for s in c.playing.iter_mut().chain(c.cue_src.iter_mut()) {
                    if is(s) {
                        s.start = StartState::Started;
                        return true;
                    }
                }
                c.stopping.iter().any(is)
            }
            BusEvent::Finished { .. } => {
                if let Some(i) = c.playing.iter().position(is) {
                    let source = c.playing.remove(i);
                    let event = if source.failed {
                        EngineEvent::CartFailed { cart: source.cart }
                    } else {
                        EngineEvent::CartEnded { cart: source.cart }
                    };
                    self.release_cart(&source);
                    self.events.push(event);
                    return true;
                }
                if c.cue_src.as_ref().is_some_and(is) {
                    if let Some(source) = c.cue_src.take() {
                        self.release_cart(&source);
                    }
                    self.events.push(EngineEvent::CartCueEnded);
                    return true;
                }
                if let Some(i) = c.stopping.iter().position(is) {
                    let source = c.stopping.remove(i);
                    self.release_cart(&source);
                    return true;
                }
                false
            }
        }
    }

    pub(super) fn start_ready_carts(&mut self) {
        let Some(c) = self.cartwall.as_mut() else {
            return;
        };
        let mut starts = Vec::new();
        for s in c.playing.iter_mut().chain(c.cue_src.iter_mut()) {
            if matches!(s.start, StartState::WhenReady { .. })
                && s.shared.is_ready()
                && !(s.shared.is_failed() && s.shared.is_drained())
            {
                s.start = StartState::Requested;
                starts.push((s.bus.clone(), s.slot));
            }
        }
        for (bus, slot) in starts {
            let now = self.now_frame(&bus);
            self.send(
                &bus,
                BusCommand::Start {
                    slot,
                    at_frame: now,
                },
            );
        }
    }

    pub(super) fn unsettled_carts(&self) -> usize {
        self.cartwall.as_ref().map_or(0, |c| {
            c.playing
                .iter()
                .chain(c.cue_src.iter())
                .filter(|s| matches!(s.start, StartState::WhenReady { .. }))
                .count()
        })
    }

    /// Positions of the playing carts, and of the cart pre-listening.
    pub fn cart_telemetry(&self) -> CartwallTelemetry {
        let Some(c) = self.cartwall.as_ref() else {
            return (Vec::new(), None);
        };
        let position = |s: &CartSource| {
            let rate = f64::from(self.rate_of(&s.bus));
            let played = s.shared.frames_played();
            let pass = s
                .shared
                .loop_frames
                .load(std::sync::atomic::Ordering::Acquire);
            if s.looped && pass > 0 {
                // The real pass length, as the worker measured it.
                s.from_secs + (played % pass) as f64 / rate
            } else {
                s.from_secs + played as f64 / rate
            }
        };
        let carts = c
            .playing
            .iter()
            .map(|s| {
                let peak = s.shared.peak_l.take().max(s.shared.peak_r.take());
                (
                    s.cart,
                    CartTelemetry {
                        position_secs: position(s),
                        peak,
                    },
                )
            })
            .collect();
        let cue = c.cue_src.as_ref().map(|s| (s.cart, position(s)));
        (carts, cue)
    }
}
