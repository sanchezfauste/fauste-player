//! The library CUE (library plan 4, LB32-LB37): the library window
//! pre-listens one track at a time on a holder of its own, with its own
//! worker, on the cart pre-listen pattern (`carts.rs`). The preview plays
//! from where it was started to the end of its file.

use super::*;
use crate::worker::LoadOptions;
use fp_model::{LibraryCueRequest, TrackId};

pub(super) struct LibrarySource {
    key: SourceKey,
    bus: BusKey,
    slot: usize,
    from_secs: f64,
    shared: Arc<SourceShared>,
    start: StartState,
    /// The worker failed: report `LibraryCueEnded` when the buffered audio
    /// ends.
    failed: bool,
    /// What it was opened from, to open it again.
    request: LibraryCueRequest,
    /// Held by Pause: the mixer never evaluates a stop on a paused slot, so
    /// it is released directly.
    held: bool,
}

pub(super) struct LibraryCueRuntime {
    worker: PlayerWorker,
    /// Where it plays: the bus and first channel of its route.
    cue: Option<(BusKey, u16)>,
    /// The route the engine holds for it (L3, L16).
    route: Option<Route>,
    volume: Arc<AtomicF32>,
    source: Option<LibrarySource>,
    /// Stopped with a de-click ramp; released when the mixer finishes them.
    outgoing: Vec<LibrarySource>,
    /// Pause holds the preview; a new start sets it, a stop clears it.
    paused: bool,
}

impl LibraryCueRuntime {
    fn all(&self) -> impl Iterator<Item = &LibrarySource> {
        self.source.iter().chain(self.outgoing.iter())
    }
}

impl Engine {
    /// Creates the library CUE worker on first use and opens its bus.
    fn ensure_library(&mut self, now: Instant) -> bool {
        let mut created = None;
        if self.library.is_none() {
            let route = self.settings.library_routes.cue.clone();
            let cue = route.as_ref().and_then(|r| self.cue_target(r));
            let ready = self
                .settings
                .frames(self.settings.tuning.ready_threshold_ms) as usize;
            let worker = match PlayerWorker::spawn(
                "fp-library-cue",
                self.opener.clone(),
                self.settings.sample_rate,
                ready,
                self.failures_tx.clone(),
            ) {
                Ok(w) => w,
                Err(e) => {
                    tracing::error!("cannot spawn the library CUE worker: {e}");
                    return false;
                }
            };
            created = Some((route.clone(), cue.as_ref().map(|c| c.0.clone())));
            self.library = Some(LibraryCueRuntime {
                worker,
                cue,
                route,
                volume: Arc::new(AtomicF32::new(1.0)),
                source: None,
                outgoing: Vec::new(),
                paused: false,
            });
        }
        if let Some(key) = self
            .library
            .as_ref()
            .and_then(|l| l.cue.as_ref().map(|c| c.0.clone()))
        {
            self.ensure_bus(&key, now);
        }
        if let Some((route, key)) = created {
            self.report_placement(Holder::LibraryCue, route, key.as_ref());
        }
        true
    }

    /// Library CUE sources attached to `key` (for the derived mixer
    /// capacity): the preview and every tail still ramping down there,
    /// plus one for the next (a seek attaches before its old source
    /// ramps out), as the cartwall counts its own.
    pub(super) fn library_sources_on(&self, key: &BusKey) -> usize {
        let Some(l) = self.library.as_ref().filter(|_| self.library_uses(key)) else {
            return 0;
        };
        // Slots still held by released sources count until the mixer
        // returns them.
        let held = self.buses.get(key).map_or(0, Bus::used_slots);
        held.max(l.all().filter(|s| &s.bus == key).count()).max(1) + 1
    }

    /// Whether the library CUE plays on `key`.
    pub(super) fn library_uses(&self, key: &BusKey) -> bool {
        self.library
            .as_ref()
            .is_some_and(|l| l.cue.as_ref().is_some_and(|c| &c.0 == key))
    }

    /// Whether a library CUE source on `bus` is audible or about to be: one
    /// started, requested or waiting only to be ready (a preview opened
    /// paused merely waits).
    pub(super) fn library_sounding(&self, bus: &BusKey) -> bool {
        self.library.as_ref().is_some_and(|l| {
            l.all()
                .any(|s| &s.bus == bus && s.start != StartState::Idle)
        })
    }

    /// Whether nothing of the library CUE on `bus` is audible: a held or
    /// idle preview is quiet, one ramping down is not.
    pub(super) fn library_quiet_on(&self, bus: &BusKey) -> bool {
        self.library.as_ref().is_none_or(|l| {
            l.source
                .iter()
                .all(|s| &s.bus != bus || s.start == StartState::Idle || s.held)
                && l.outgoing.iter().all(|s| &s.bus != bus)
        })
    }

    /// L11: whether the library CUE is silent: no preview sounding and none
    /// de-clicking.
    pub(super) fn library_path_quiet(&self) -> bool {
        self.library.as_ref().is_none_or(|l| {
            l.source
                .iter()
                .all(|s| s.start == StartState::Idle || s.held)
                && l.outgoing.is_empty()
        })
    }

    fn library_source(
        &mut self,
        request: &LibraryCueRequest,
    ) -> Result<LibrarySource, AttachError> {
        let l = self.library.as_ref().ok_or(AttachError::NoPlayer)?;
        let (bus_key, channel) = l.cue.clone().ok_or(AttachError::NoRoute)?;
        let volume = l.volume.clone();
        // The preview's fixed normalization gain (N14).
        let gain = request.normalization.linear_gain();
        let (slot, producer, shared) = self
            .attach_worker_source(&bus_key, channel, volume, gain)
            .inspect_err(|e| {
            if *e == AttachError::NoSlot {
                tracing::error!(track = ?request.track, bus = ?bus_key, "no free mixer slot");
            }
        })?;
        self.next_key += 1;
        let key = SourceKey(self.next_key);
        let rate = self.rate_of(&bus_key);
        let options = LoadOptions {
            until_secs: None,
            looped: false,
            rate: Some(rate),
            fade_out_frames: self.frames_on(&bus_key, self.settings.tuning.declick_ms),
            dsd: false,
            ready_frames: Some(frames_at(rate, self.settings.tuning.ready_threshold_ms) as usize),
        };
        if let Some(l) = self.library.as_ref() {
            l.worker.load_with(
                key,
                request.path.clone(),
                request.from_secs,
                producer,
                options,
            );
        }
        Ok(LibrarySource {
            key,
            bus: bus_key,
            slot,
            from_secs: request.from_secs,
            shared,
            start: StartState::WhenReady { fade_in: false },
            failed: false,
            request: request.clone(),
            held: false,
        })
    }

    /// LB32: pre-listens `request` on the library CUE route, replacing any
    /// preview without an end event. `paused` opens it held. Without a
    /// route nothing can be heard: it ends at once.
    pub(super) fn start_library_cue(
        &mut self,
        request: &LibraryCueRequest,
        paused: bool,
        now: Instant,
    ) {
        if !self.ensure_library(now) {
            self.events.push(EngineEvent::LibraryCueEnded {
                track: request.track,
            });
            return;
        }
        self.stop_library_source();
        if let Some(l) = self.library.as_mut() {
            l.paused = paused;
        }
        // B3.2 follows the file's rate on an idle bit-perfect bus, but an
        // on-air pre-listen never moves the rate (CA7).
        if let Some(cue) = self
            .library
            .as_ref()
            .and_then(|l| l.cue.clone())
            .filter(|cue| !self.cue_on_air(cue))
        {
            self.prepare_start(&cue.0, request.format);
        }
        match self.library_source(request) {
            Ok(mut source) => {
                if let Some(l) = self.library.as_mut() {
                    if l.paused {
                        source.start = StartState::Idle;
                        source.held = true;
                    }
                    l.source = Some(source);
                }
            }
            Err(_) => self.events.push(EngineEvent::LibraryCueEnded {
                track: request.track,
            }),
        }
    }

    /// Stops the preview with a de-click ramp; no end event.
    pub(super) fn stop_library_cue(&mut self) {
        if let Some(l) = self.library.as_mut() {
            l.paused = false;
        }
        self.stop_library_source();
    }

    fn stop_library_source(&mut self) {
        let Some(source) = self.library.as_mut().and_then(|l| l.source.take()) else {
            return;
        };
        self.retire_library_source(source);
    }

    /// Stops `source`, no longer the preview, with a de-click ramp.
    fn retire_library_source(&mut self, source: LibrarySource) {
        // A requested start may already be audible (its Started event is
        // not polled yet): it keeps its start and fades like a started one,
        // so the mixer reports its end either way (a cancelled start would
        // never finish). A held one is silent, and the mixer never stops a
        // paused slot.
        if source.start != StartState::Requested {
            self.send(&source.bus, BusCommand::Cancel { slot: source.slot });
        }
        if source.held || !matches!(source.start, StartState::Started | StartState::Requested) {
            self.release_library(&source);
            return;
        }
        self.declick_stop(&source.bus, source.slot);
        if let Some(l) = self.library.as_mut() {
            l.outgoing.push(source);
        }
    }

    fn release_library(&mut self, source: &LibrarySource) {
        self.send(&source.bus, BusCommand::Detach { slot: source.slot });
        if let Some(l) = self.library.as_ref() {
            l.worker.drop_source(source.key);
        }
    }

    /// L16 for the library CUE. Before its first preview only the placement
    /// is reported: it opens with the current configuration (L13). A
    /// preview on a route that moves ends (`LibraryCueEnded`). Returns
    /// `None` while the change must wait (L11).
    pub(super) fn move_library_cue(
        &mut self,
        route: Option<Route>,
        force: bool,
    ) -> Option<Result<(), String>> {
        if self.library.is_none() {
            self.report_library_route(route);
            return Some(Ok(()));
        }
        let cue = route.as_ref().and_then(|r| self.cue_target(r));
        let outcome = match &cue {
            Some(target) => self.make_room(target, force)?,
            None => Ok(()),
        };
        let now = self.now;
        let l = self.library.as_mut()?;
        let old = std::mem::replace(&mut l.cue, cue.clone());
        l.route = route.clone();
        if let Some((key, _)) = &cue {
            self.ensure_bus(key, now);
        }
        if old != cue
            && let Some((old_key, _)) = &old
        {
            self.orphans.insert(old_key.clone());
            self.end_library_cue();
        }
        self.report_placement(Holder::LibraryCue, route, cue.as_ref().map(|c| &c.0));
        Some(outcome)
    }

    /// The preview ends because its output went away (L16): cut, with what
    /// still de-clicks there.
    fn end_library_cue(&mut self) {
        let (source, outgoing) = match self.library.as_mut() {
            Some(l) => (l.source.take(), std::mem::take(&mut l.outgoing)),
            None => return,
        };
        for s in &outgoing {
            self.send(&s.bus, BusCommand::Cancel { slot: s.slot });
            self.release_library(s);
        }
        if let Some(s) = source {
            self.send(&s.bus, BusCommand::Cancel { slot: s.slot });
            self.release_library(&s);
            self.events.push(EngineEvent::LibraryCueEnded {
                track: s.request.track,
            });
        }
    }

    /// Handles a worker failure if it belongs to a library CUE source.
    pub(super) fn library_failure(&mut self, failure: &WorkerFailure) -> bool {
        let Some(l) = self.library.as_mut() else {
            return false;
        };
        if let Some(s) = l.source.as_mut().filter(|s| s.key == failure.key) {
            tracing::warn!(track = ?s.request.track, error = %failure.error, "library CUE source failed");
            if !matches!(s.start, StartState::WhenReady { .. } | StartState::Idle)
                && !s.shared.is_drained()
            {
                // Let what is buffered play; report at the end.
                s.failed = true;
                return true;
            }
            if let Some(source) = l.source.take() {
                self.release_library(&source);
                self.events.push(EngineEvent::LibraryCueEnded {
                    track: source.request.track,
                });
            }
            return true;
        }
        l.outgoing.iter().any(|s| s.key == failure.key)
    }

    /// Handles a mixer event if it belongs to a library CUE source.
    pub(super) fn library_bus_event(&mut self, bus: &BusKey, event: &BusEvent) -> bool {
        let Some(l) = self.library.as_mut() else {
            return false;
        };
        let (BusEvent::Started { slot, .. } | BusEvent::Finished { slot, .. }) = *event;
        let is = |s: &LibrarySource| &s.bus == bus && s.slot == slot;
        match event {
            BusEvent::Started { .. } => {
                if let Some(s) = l.source.as_mut().filter(|s| is(s)) {
                    s.start = StartState::Started;
                    return true;
                }
                l.outgoing.iter().any(is)
            }
            BusEvent::Finished { .. } => {
                if l.source.as_ref().is_some_and(is) {
                    if let Some(source) = l.source.take() {
                        if source.failed {
                            tracing::warn!(track = ?source.request.track, "library CUE ended early: the file failed");
                        }
                        self.release_library(&source);
                        self.events.push(EngineEvent::LibraryCueEnded {
                            track: source.request.track,
                        });
                    }
                    return true;
                }
                if let Some(i) = l.outgoing.iter().position(is) {
                    let source = l.outgoing.remove(i);
                    self.release_library(&source);
                    return true;
                }
                false
            }
        }
    }

    /// Starts the preview once its worker has buffered enough.
    pub(super) fn start_ready_library(&mut self) {
        let Some(s) = self.library.as_mut().and_then(|l| l.source.as_mut()) else {
            return;
        };
        if matches!(s.start, StartState::WhenReady { .. })
            && s.shared.is_ready()
            && !(s.shared.is_failed() && s.shared.is_drained())
        {
            s.start = StartState::Requested;
            let (bus, slot, fade_in) = (s.bus.clone(), s.slot, s.from_secs > 0.0);
            self.start_on_now(&bus, slot, fade_in);
        }
    }

    /// A preview still waiting for its worker: to start, or (opened
    /// paused) to be filled.
    pub(super) fn unsettled_library(&self) -> usize {
        self.library
            .as_ref()
            .and_then(|l| l.source.as_ref())
            .filter(|s| match s.start {
                StartState::WhenReady { .. } => true,
                StartState::Idle => !s.shared.is_ready() && !s.shared.is_failed(),
                StartState::Requested | StartState::Started => false,
            })
            .map_or(0, |_| 1)
    }

    /// Library CUE sources attached to a mixer (a leak shows up here).
    pub(super) fn attached_library(&self) -> usize {
        self.library.as_ref().map_or(0, |l| l.all().count())
    }

    /// The track the library CUE plays and its position in the file.
    pub fn library_cue_position(&self) -> Option<(TrackId, f64)> {
        let s = self.library.as_ref()?.source.as_ref()?;
        let rate = f64::from(self.rate_of(&s.bus));
        Some((
            s.request.track,
            s.from_secs + s.shared.frames_played() as f64 / rate,
        ))
    }

    /// LB33: replaces the preview by one at `secs` of its file, as the
    /// player CUE's seek does: the old source ramps down. A held preview
    /// stays held: the new source waits idle until it is released. Without
    /// a preview nothing happens.
    pub(super) fn seek_library_cue(&mut self, secs: f64) {
        if !secs.is_finite() {
            return;
        }
        let Some((mut request, bus)) = self
            .library
            .as_ref()
            .and_then(|l| l.source.as_ref())
            .map(|s| (s.request.clone(), s.bus.clone()))
        else {
            return;
        };
        request.from_secs = secs.max(0.0);
        // Tails still ramping keep their slots: the bus makes room for
        // one more (never a cut tail, which would click).
        let now = self.now;
        self.ensure_bus(&bus, now);
        let Ok(mut next) = self.library_source(&request) else {
            return;
        };
        let Some(l) = self.library.as_mut() else {
            return;
        };
        if l.paused {
            next.start = StartState::Idle;
            next.held = true;
        }
        let old = l.source.replace(next);
        if let Some(old) = old {
            self.retire_library_source(old);
        }
    }

    /// LB33: holds or releases the preview, as the player CUE's pause does.
    /// A source that has not started yet is held back idle; one that is
    /// audible ramps down with the pause ramp. Without a preview only the
    /// flag changes.
    pub(super) fn set_library_cue_paused(&mut self, paused: bool) {
        let Some(l) = self.library.as_mut() else {
            return;
        };
        l.paused = paused;
        let Some(s) = l.source.as_mut() else {
            return;
        };
        s.held = paused;
        let (bus, slot) = (s.bus.clone(), s.slot);
        if let Some(pause) = hold_transition(&mut s.start, paused) {
            self.send_hold(&bus, slot, pause);
        }
    }

    /// The library CUE part of `reopen_on_bus` (and of a B3.2 rate change
    /// under a held preview): a preview on `bus` is opened again at its
    /// position measured at `old_rate`, held if it was; one ramping down
    /// there is cut.
    pub(super) fn follow_forced_rate_library(&mut self, bus: &BusKey, old_rate: u32) {
        self.cut_library_outgoing(|s| &s.bus == bus);
        let Some(source) = self
            .library
            .as_mut()
            .and_then(|l| l.source.take_if(|s| &s.bus == bus))
        else {
            return;
        };
        let mut request = source.request.clone();
        request.from_secs =
            source.from_secs + source.shared.frames_played() as f64 / f64::from(old_rate.max(1));
        self.send(&source.bus, BusCommand::Cancel { slot: source.slot });
        self.release_library(&source);
        match self.library_source(&request) {
            Ok(mut fresh) => {
                if source.held {
                    fresh.start = StartState::Idle;
                    fresh.held = true;
                }
                if let Some(l) = self.library.as_mut() {
                    l.source = Some(fresh);
                }
            }
            Err(_) => self.events.push(EngineEvent::LibraryCueEnded {
                track: request.track,
            }),
        }
    }

    /// Cuts and releases the sources ramping down that `on` selects.
    fn cut_library_outgoing(&mut self, on: impl Fn(&LibrarySource) -> bool) {
        let Some(l) = self.library.as_mut() else {
            return;
        };
        let (gone, kept): (Vec<_>, Vec<_>) = std::mem::take(&mut l.outgoing)
            .into_iter()
            .partition(|s| on(s));
        l.outgoing = kept;
        for s in &gone {
            self.send(&s.bus, BusCommand::Cancel { slot: s.slot });
            self.release_library(s);
        }
    }

    /// What the library CUE's sources (the preview and its tails ramping down)
    /// measured since the last call; the measurement is taken, not copied.
    pub fn take_library_cue_meter_input(&self) -> crate::meter::MeterInput {
        let mut input = crate::meter::MeterInput::default();
        let Some(l) = self.library.as_ref() else {
            return input;
        };
        for p in l.all() {
            input.merge(self.take_source_measurement(&p.bus, &p.shared));
        }
        input
    }
}
