//! DSD reaching bit-perfect devices unchanged (feedback 2 spec O25), the
//! engine's side. A DSD track that starts on an idle device set to DoP or
//! native DSD reopens the device as a DSD stream at the word rate (the DSD
//! rate over 16), and the mixer copies the source's words into it. Any
//! other source that needs that device either plays muted until the DSD
//! stream ends (`DsdMix::HoldOthers`) or switches the device back to PCM
//! (`DsdMix::ConvertToPcm`), the DSD track going on as PCM. Every start,
//! end and switch sends the DSD silence first, so the converter locks
//! without a pop. All of this runs on the conductor thread.

use super::*;
use fp_backends::dsd::DsdStream;
use fp_model::{DsdFacts, DsdFallback, DsdMix, DsdOutput, DsdStreamMode, dsd_decision};

/// Where a bus carrying DSD is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DsdState {
    /// The DSD source in `slot` is on air, or about to be.
    Playing { slot: usize },
    /// The source is gone; the DSD silence runs until this bus frame, then
    /// the bus goes back to PCM (unless a DSD start at the same word rate
    /// takes the stream over first).
    Tail { until: u64 },
    /// Switching to PCM at this bus frame: the DSD silence (and, for
    /// native DSD, the hold until the device is reopened as PCM). The bus
    /// is not idle meanwhile, so no DSD start can slip in before the
    /// switch applies.
    Switching { until: u64 },
}

/// A bus carrying DSD unchanged, and whose stream it is.
pub(super) struct DsdBus {
    stream: DsdStream,
    word_rate: u32,
    player: PlayerId,
    entry: EntryId,
    state: DsdState,
    /// `DsdStarted` was reported for this stream.
    announced: bool,
    /// The PCM configuration the bus had before the DSD stream: a device
    /// that takes native DSD at a word rate need not take that rate as PCM.
    pcm: StreamConfig,
}

impl Engine {
    /// Whether `slot` on `bus` is a DSD source going out unchanged.
    pub(super) fn dsd_slot(&self, bus: &BusKey, slot: usize) -> bool {
        self.dsd_buses
            .get(bus)
            .is_some_and(|d| d.state == DsdState::Playing { slot })
    }

    /// Whether `p` goes out as DSD, unchanged.
    pub(super) fn is_dsd_direct(&self, p: &Playing) -> bool {
        !p.cue && self.dsd_slot(&p.bus, p.slot)
    }

    /// Whether `bus` runs a DSD stream's silence (its tail, or a switch to
    /// PCM): not quiet, so a device change waits for it (live settings L11).
    pub(super) fn dsd_silence_running(&self, bus: &BusKey) -> bool {
        self.dsd_buses
            .get(bus)
            .is_some_and(|d| !matches!(d.state, DsdState::Playing { .. }))
    }

    /// The pause and resume ramp of `slot`: none for a DSD stream (the
    /// mixer holds at once and the DSD silence keeps the stream valid).
    pub(super) fn pause_ramp_of(&self, bus: &BusKey, slot: usize) -> u32 {
        if self.dsd_slot(bus, slot) {
            0
        } else {
            self.ramp_frames(bus)
        }
    }

    /// The DSD silence in frames of `bus`.
    fn silence_frames(&self, bus: &BusKey) -> u64 {
        let ms = self
            .device_of(bus)
            .dsd_silence_ms
            .unwrap_or(self.settings.dsd.silence_ms);
        self.frames_on(bus, ms)
    }

    /// Before a track of `player` starts on `bus`: whether it can go out as
    /// DSD unchanged (`fp_model::dsd_decision`). If it can, the bus is made a
    /// DSD stream at the word rate (or keeps the one whose silence is still
    /// running) and the DSD source is returned, to start after the DSD
    /// silence; otherwise `None`, and the track plays converted, as before.
    pub(super) fn try_start_dsd(
        &mut self,
        player: PlayerId,
        bus: &BusKey,
        request: &SourceRequest,
    ) -> Option<Playing> {
        let mode = self.device_of(bus).dsd;
        let volume = self.players.get(&player)?.volume.load();
        let wanted = match mode {
            DsdOutput::Pcm => None,
            DsdOutput::Dop => Some(DsdStream::Dop),
            DsdOutput::Native => Some(DsdStream::Native),
        };
        let word_rate = request
            .format
            .and_then(|f| f.dsd_rate)
            .map(|r| r / fp_model::DSD_WORD_BITS);
        // A stream whose silence is still running carries the next track
        // straight on when it is the same kind at the same rate.
        let reuse = self.dsd_buses.get(bus).is_some_and(|d| {
            matches!(d.state, DsdState::Tail { .. })
                && Some(d.stream) == wanted
                && Some(d.word_rate) == word_rate
        });
        let device_idle = !self.bus_sounding(bus) && (reuse || !self.dsd_buses.contains_key(bus));
        let facts = DsdFacts {
            mode,
            format: request.format,
            volume,
            device_idle,
        };
        let target = match dsd_decision(&facts) {
            Ok(target) => target,
            Err(fallback) => {
                log_fallback(bus, fallback);
                return None;
            }
        };
        let stream = match target.mode {
            DsdStreamMode::Dop => DsdStream::Dop,
            DsdStreamMode::Native => DsdStream::Native,
        };
        let kept = self.dsd_buses.get(bus).filter(|_| reuse).map(|d| d.pcm);
        let pcm = match kept {
            Some(pcm) => pcm,
            None => match self.open_dsd_stream(bus, stream, target.word_rate) {
                Ok(pcm) => pcm,
                Err(fallback) => {
                    log_fallback(bus, fallback);
                    return None;
                }
            },
        };
        let now_frame = self.now_frame(bus);
        if !reuse {
            self.send(
                bus,
                BusCommand::DsdMode {
                    on: true,
                    at_frame: now_frame,
                },
            );
        }
        // The DSD source never comes from a PCM preload; one of the same
        // entry would only stay behind unused.
        let stale = self.players.get_mut(&player).and_then(|rt| {
            rt.preload
                .take_if(|p| p.entry == request.entry && p.start_secs == request.from_secs)
        });
        if let Some(stale) = stale {
            self.release(stale);
        }
        let opened = self.open_source(player, false, request, true);
        // A stream taken over: the one before it has ended.
        if let Some(old) = self.dsd_buses.remove(bus) {
            self.events.push(EngineEvent::DsdEnded {
                player: old.player,
                entry: old.entry,
            });
        }
        let silence = self.silence_frames(bus);
        let mut record = DsdBus {
            stream,
            word_rate: target.word_rate,
            player,
            entry: request.entry,
            state: DsdState::Tail {
                until: now_frame + silence,
            },
            announced: false,
            pcm,
        };
        match opened {
            Ok(mut source) => {
                source.not_before = if reuse {
                    now_frame
                } else {
                    now_frame + silence
                };
                record.state = DsdState::Playing { slot: source.slot };
                self.dsd_buses.insert(bus.clone(), record);
                Some(source)
            }
            Err(e) => {
                // An engine limitation, not a bad file: the stream just ends
                // (after its silence), and the track tries to play converted.
                tracing::error!(?player, error = ?e, "cannot open a DSD source");
                self.dsd_buses.insert(bus.clone(), record);
                None
            }
        }
    }

    /// Reopens `bus` as a `stream` at `word_rate` and checks the stream can
    /// carry it; returns the PCM configuration the bus had, to go back to.
    /// On a refusal the bus is back as it was (the device remembers the
    /// refusal).
    fn open_dsd_stream(
        &mut self,
        bus: &BusKey,
        stream: DsdStream,
        word_rate: u32,
    ) -> Result<StreamConfig, DsdFallback> {
        let now = self.now;
        let Some(b) = self.buses.get_mut(bus) else {
            return Err(DsdFallback::StreamRefused);
        };
        let previous = b.config();
        let wanted = StreamConfig {
            sample_rate: word_rate,
            dsd: Some(stream),
            exclusive: true,
            // A DSD stream takes the device's buffer when it does not take
            // the one asked for, as before: a refusal would lose the DSD.
            exact_buffer: false,
            ..previous
        };
        // Only on an idle device (`device_idle`): the conductor may wait,
        // once for the reopen and the way back below.
        let mut budget = b.busy_budget(true);
        if b.reopen_with(wanted, now, &mut budget).is_err() {
            return Err(self.refusal(bus, word_rate));
        }
        // Checked here, whatever the backend checked: DoP needs every bit of
        // a 24-bit word, and native DSD a stream the backend packs as DSD.
        if !b.dsd_fits(stream) {
            b.refuse_dsd(word_rate, stream);
            if let Err(error) = b.reopen_with(previous, now, &mut budget) {
                tracing::warn!(?bus, %error, "cannot reopen the PCM stream");
            }
            return Err(match stream {
                DsdStream::Dop => DsdFallback::FormatTooNarrow,
                DsdStream::Native => DsdFallback::StreamRefused,
            });
        }
        // A bus that carried DoP before keeps that configuration while it
        // plays PCM: the same rate, as PCM.
        let pcm = StreamConfig {
            dsd: None,
            ..previous
        };
        b.set_pcm_fallback(pcm);
        if previous.sample_rate != word_rate {
            self.reopen_waiting(bus);
        }
        Ok(pcm)
    }

    /// Why the device refused a DSD stream at `word_rate`: the rate, when
    /// the device does not list it, else the stream itself.
    fn refusal(&self, bus: &BusKey, word_rate: u32) -> DsdFallback {
        let listed = self
            .find_backend(&bus.backend)
            .and_then(|b| b.enumerate_devices().ok())
            .and_then(|devices| devices.into_iter().find(|d| d.id.0 == bus.device))
            .map(|d| {
                d.sample_rates
                    .iter()
                    .any(|(low, high)| (*low..=*high).contains(&word_rate))
            });
        match listed {
            Some(false) => DsdFallback::RateRefused(word_rate),
            _ => DsdFallback::StreamRefused,
        }
    }

    /// Cuts `player`'s current source if it is a DSD stream (a DSD stream
    /// cannot be faded): its silence starts.
    pub(super) fn cut_dsd_current(&mut self, player: PlayerId) {
        let direct = self
            .players
            .get(&player)
            .and_then(|rt| rt.current.as_ref())
            .is_some_and(|c| self.is_dsd_direct(c));
        if !direct {
            return;
        }
        if let Some(current) = self
            .players
            .get_mut(&player)
            .and_then(|rt| rt.current.take())
        {
            self.send(&current.bus, BusCommand::Cancel { slot: current.slot });
            self.release(current);
        }
    }

    /// `p` left its bus (it ended, failed, was cut or replaced): if it was a
    /// DSD stream's source, the stream's silence starts.
    pub(super) fn dsd_source_gone(&mut self, p: &Playing) {
        if !self.is_dsd_direct(p) {
            return;
        }
        // One block of margin: the engine's frame lags the mixer's.
        let until = self.now_frame(&p.bus)
            + u64::from(self.buffer_of(&p.bus))
            + self.silence_frames(&p.bus);
        if let Some(d) = self.dsd_buses.get_mut(&p.bus) {
            d.state = DsdState::Tail { until };
        }
    }

    /// A DSD stream's source was replaced by another in `to` (a seek).
    pub(super) fn dsd_source_moved(&mut self, bus: &BusKey, from: usize, to: usize) {
        if let Some(d) = self.dsd_buses.get_mut(bus)
            && d.state == (DsdState::Playing { slot: from })
        {
            d.state = DsdState::Playing { slot: to };
        }
    }

    /// The mixer started `slot` on `bus`. For a DSD stream's source this is
    /// when DSD reaches the device: reported, unless the player's volume
    /// left unity since the decision, which switches it to PCM instead.
    pub(super) fn dsd_slot_started(&mut self, bus: &BusKey, slot: usize) {
        let Some(d) = self.dsd_buses.get(bus) else {
            return;
        };
        if d.state != (DsdState::Playing { slot }) || d.announced {
            return;
        }
        let (player, entry) = (d.player, d.entry);
        let volume = self.players.get(&player).map_or(1.0, |rt| rt.volume.load());
        #[allow(clippy::float_cmp)] // exactly unity: any other gain changes the signal
        if volume != 1.0 {
            log_fallback(bus, DsdFallback::VolumeNotUnity);
            let now = self.now_frame(bus);
            self.switch_to_pcm(bus, now);
            return;
        }
        if let Some(d) = self.dsd_buses.get_mut(bus) {
            d.announced = true;
        }
        self.events.push(EngineEvent::DsdStarted {
            player,
            entry,
            hold_others: self.device_of(bus).dsd_mix == Some(DsdMix::HoldOthers),
        });
    }

    /// `EngineAction::LeaveDsd`: `player`'s DSD stream goes on as PCM.
    pub(super) fn leave_dsd(&mut self, player: PlayerId) {
        let bus = self
            .dsd_buses
            .iter()
            .find(|(_, d)| d.player == player && matches!(d.state, DsdState::Playing { .. }))
            .map(|(k, _)| k.clone());
        if let Some(bus) = bus {
            let now = self.now_frame(&bus);
            self.switch_to_pcm(&bus, now);
        }
    }

    /// Before a source that is not a DSD stream starts on `bus` at
    /// `at_frame`: returns the frame it may start at. A bus without DSD
    /// keeps `at_frame`. Over a DSD stream, the source starts muted
    /// (`HoldOthers`) or after the switch to PCM (`ConvertToPcm`); over a
    /// stream's silence, when the silence is over.
    pub(super) fn before_start_on(&mut self, bus: &BusKey, at_frame: u64) -> u64 {
        let Some(state) = self.dsd_buses.get(bus).map(|d| d.state) else {
            return at_frame;
        };
        match state {
            DsdState::Playing { .. } => {
                match self.device_of(bus).dsd_mix.unwrap_or(self.settings.dsd.mix) {
                    DsdMix::HoldOthers => at_frame,
                    DsdMix::ConvertToPcm => self.switch_to_pcm(bus, at_frame),
                }
            }
            DsdState::Tail { until } => self.end_tail_at(bus, at_frame.max(until)),
            DsdState::Switching { until } => at_frame.max(until),
        }
    }

    /// Switches `bus` from its DSD stream to PCM at `at_frame` (or now):
    /// everything holds for the DSD silence, then the bus is PCM and the DSD
    /// track goes on converted from where it held. Returns the frame the
    /// bus is PCM from. A native stream is reopened as PCM at that frame
    /// (`end_dsd_streams`), the mixer holding until then.
    fn switch_to_pcm(&mut self, bus: &BusKey, at_frame: u64) -> u64 {
        let from = at_frame.max(self.now_frame(bus));
        let pcm_at = from + self.silence_frames(bus);
        let Some(d) = self.dsd_buses.get_mut(bus) else {
            return at_frame;
        };
        let (player, entry, stream) = (d.player, d.entry, d.stream);
        d.state = DsdState::Switching { until: pcm_at };
        match stream {
            DsdStream::Dop => {
                self.send(
                    bus,
                    BusCommand::HoldAll {
                        from_frame: from,
                        until_frame: pcm_at,
                    },
                );
                self.send(
                    bus,
                    BusCommand::DsdMode {
                        on: false,
                        at_frame: pcm_at,
                    },
                );
            }
            DsdStream::Native => {
                self.send(
                    bus,
                    BusCommand::HoldAll {
                        from_frame: from,
                        until_frame: u64::MAX,
                    },
                );
            }
        }
        tracing::info!(?bus, ?player, "DSD switched to PCM");
        self.events.push(EngineEvent::DsdEnded { player, entry });
        pcm_at
    }

    /// Ends the silence of `bus`'s finished DSD stream at `at_frame` (or
    /// now), for a PCM start there; returns the frame the bus is PCM from.
    fn end_tail_at(&mut self, bus: &BusKey, at_frame: u64) -> u64 {
        let pcm_at = at_frame.max(self.now_frame(bus));
        let Some(d) = self.dsd_buses.get_mut(bus) else {
            return at_frame;
        };
        let (player, entry, stream) = (d.player, d.entry, d.stream);
        d.state = DsdState::Switching { until: pcm_at };
        match stream {
            DsdStream::Dop => {
                // Only a block straddling the switch is held, so that a
                // start on it lands where the bus is PCM.
                self.send(
                    bus,
                    BusCommand::HoldAll {
                        from_frame: pcm_at,
                        until_frame: pcm_at,
                    },
                );
                self.send(
                    bus,
                    BusCommand::DsdMode {
                        on: false,
                        at_frame: pcm_at,
                    },
                );
            }
            DsdStream::Native => {
                self.send(
                    bus,
                    BusCommand::HoldAll {
                        from_frame: pcm_at,
                        until_frame: u64::MAX,
                    },
                );
            }
        }
        self.events.push(EngineEvent::DsdEnded { player, entry });
        pcm_at
    }

    /// The device of `bus` came back from a loss as PCM (it could no longer
    /// carry the DSD stream): the DSD track goes on from its PCM conversion.
    pub(super) fn dsd_stream_lost(&mut self, bus: &BusKey) {
        let Some(d) = self.dsd_buses.remove(bus) else {
            return;
        };
        tracing::warn!(?bus, player = ?d.player, "DSD lost with the device; going on as PCM");
        match d.state {
            DsdState::Switching { .. } => {
                // `DsdEnded` was reported; end a native switch's hold.
                let now_frame = self.now_frame(bus);
                self.send(
                    bus,
                    BusCommand::HoldAll {
                        from_frame: 0,
                        until_frame: now_frame,
                    },
                );
            }
            DsdState::Playing { .. } | DsdState::Tail { .. } => {
                self.events.push(EngineEvent::DsdEnded {
                    player: d.player,
                    entry: d.entry,
                });
            }
        }
    }

    /// Called from `tick`: DSD streams whose silence is over go back to
    /// PCM (a DoP stream simply leaves DSD mode; a native one is reopened as
    /// PCM, see `reopen_native_as_pcm`), and native switches reach their
    /// reopen.
    pub(super) fn end_dsd_streams(&mut self) {
        let due: Vec<(BusKey, bool)> = self
            .dsd_buses
            .iter()
            .filter_map(|(bus, d)| {
                let now = self.now_frame(bus);
                match d.state {
                    DsdState::Tail { until } if now >= until => Some((bus.clone(), false)),
                    DsdState::Switching { until } if now >= until => Some((bus.clone(), true)),
                    _ => None,
                }
            })
            .collect();
        for (bus, switching) in due {
            let Some(d) = self.dsd_buses.remove(&bus) else {
                continue;
            };
            let now_frame = self.now_frame(&bus);
            match d.stream {
                // A DoP switch queued its `DsdMode` off already.
                DsdStream::Dop if switching => {}
                DsdStream::Dop => self.send(
                    &bus,
                    BusCommand::DsdMode {
                        on: false,
                        at_frame: now_frame,
                    },
                ),
                DsdStream::Native => self.reopen_native_as_pcm(&bus, now_frame, d.pcm),
            }
            if !switching {
                self.events.push(EngineEvent::DsdEnded {
                    player: d.player,
                    entry: d.entry,
                });
            }
        }
    }

    /// Reopens a native DSD bus as PCM at a rate the device takes. The
    /// stream closes first, so no PCM block ever reaches the DSD stream; the
    /// mixer leaves DSD mode and its hold on the new stream's first block.
    ///
    /// A device that takes native DSD at a word rate need not take that
    /// rate as PCM (DSD512 never: 1.4112 MHz). An idle bus goes back to the
    /// PCM configuration it had before the DSD (`pcm`); a bus with sources
    /// on it tries the word rate first, so their timelines stay as they
    /// are. Then the configured rate: the device's own, else the global
    /// one (operator feedback 4, Q12.5). When the rate changes, the sources on
    /// the bus are opened again at the new rate (`follow_forced_rate`). If
    /// nothing opens, the bus is `Lost` and the watchdog retries, falling
    /// back to `pcm` (`Bus::set_pcm_fallback`): never an impossible rate
    /// forever.
    fn reopen_native_as_pcm(&mut self, bus: &BusKey, now_frame: u64, pcm: StreamConfig) {
        let now = self.now;
        if let Some(b) = self.buses.get_mut(bus) {
            b.close_stream();
        }
        self.send(
            bus,
            BusCommand::DsdMode {
                on: false,
                at_frame: now_frame,
            },
        );
        self.send(
            bus,
            BusCommand::HoldAll {
                from_frame: 0,
                until_frame: now_frame,
            },
        );
        let sounding = self.bus_sounding(bus);
        let configured = StreamConfig {
            sample_rate: self.device_of(bus).sample_rate,
            ..pcm
        };
        let Some(b) = self.buses.get_mut(bus) else {
            return;
        };
        let word_rate = b.sample_rate();
        let word = StreamConfig {
            dsd: None,
            ..b.config()
        };
        let candidates = if sounding {
            [Some(word), Some(pcm), Some(configured)]
        } else {
            [Some(pcm), Some(configured), None]
        };
        let mut reopened = false;
        // A sounding bus never waits for a busy device: its stream is closed
        // and no virtual clock runs, so its timeline would stall.
        let mut budget = b.busy_budget(!sounding);
        for config in candidates.into_iter().flatten() {
            match b.reopen_with(config, now, &mut budget) {
                Ok(()) => {
                    reopened = true;
                    break;
                }
                Err(error) => {
                    tracing::warn!(?bus, rate = config.sample_rate, %error, "cannot reopen the native DSD device as PCM at this rate");
                }
            }
        }
        if !reopened {
            tracing::warn!(?bus, "cannot reopen the native DSD device as PCM; retrying");
        }
        self.follow_forced_rate(bus, word_rate);
    }
}

/// Logs why a DSD track plays converted, unless that is the ordinary case.
fn log_fallback(bus: &BusKey, fallback: DsdFallback) {
    if fallback.worth_logging() {
        tracing::info!(bus = ?bus, %fallback, "DSD converted to PCM");
    }
}
