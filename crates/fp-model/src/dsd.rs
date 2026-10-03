//! DSD output (feedback 2 spec O25): the settings, and (Task 4) the pure
//! rules that decide when DSD reaches a device unchanged.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::ids::{EntryId, PlayerId};
use crate::player::Transport;
use crate::state::AppState;
use crate::track::AudioFormat;

/// What a bit-perfect device receives from a DSD track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum DsdOutput {
    /// Converted to PCM, as for any other device.
    #[default]
    Pcm,
    /// DSD over PCM (DoP 1.1): 24-bit samples at the DSD rate ÷ 16, with the
    /// alternating 0x05/0xFA markers.
    Dop,
    /// Raw DSD; only on Linux, through ALSA, on hardware devices that report
    /// a DSD sample format.
    Native,
}

/// What happens when another source needs an output that carries DSD.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum DsdMix {
    /// The DSD track goes on as PCM from that moment, mixed as usual.
    #[default]
    ConvertToPcm,
    /// Nothing interrupts the DSD stream: the player's next track waits for
    /// its end, and other sources are muted on that output meanwhile.
    HoldOthers,
}

/// The DSD mode of one output device.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DsdDevice {
    pub backend: String,
    pub device: String,
    pub mode: DsdOutput,
}

/// Default DSD silence sent at a DSD stream's start, end and switch to PCM.
pub const DEFAULT_DSD_SILENCE_MS: f64 = 200.0;

pub const DSD_WORD_BITS: u32 = 16;

/// What the engine knows when a DSD track is about to start on a device.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DsdFacts {
    /// The device's mode (`OutputsConfig::dsd_output_for`).
    pub mode: DsdOutput,
    pub format: Option<AudioFormat>,
    /// The player's volume (linear gain).
    pub volume: f32,
    /// Nothing sounds on the device.
    pub device_idle: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DsdStreamMode {
    Dop,
    Native,
}

/// How DSD goes out: the stream mode, and the rate of its 16-bit word
/// stream (the device rate for DoP).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DsdTarget {
    pub mode: DsdStreamMode,
    pub dsd_rate: u32,
    pub word_rate: u32,
}

/// Why a DSD track is converted to PCM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DsdFallback {
    ModeIsPcm,
    NotDsd,
    Multichannel,
    VolumeNotUnity,
    DeviceBusy,
    RateRefused(u32),
    /// DoP needs a 24- or 32-bit integer device format.
    FormatTooNarrow,
    /// The device or the system refused the DSD stream.
    StreamRefused,
}

impl DsdFallback {
    /// Whether the conversion deserves a log line (not for the default
    /// mode, nor for a file that is not DSD).
    pub fn worth_logging(self) -> bool {
        !matches!(self, DsdFallback::ModeIsPcm | DsdFallback::NotDsd)
    }
}

impl fmt::Display for DsdFallback {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DsdFallback::ModeIsPcm => write!(f, "the device converts DSD to PCM"),
            DsdFallback::NotDsd => write!(f, "the track is not DSD"),
            DsdFallback::Multichannel => write!(f, "the track has more than two channels"),
            DsdFallback::VolumeNotUnity => write!(f, "the player's volume is not 100 %"),
            DsdFallback::DeviceBusy => write!(f, "something else plays on the device"),
            DsdFallback::RateRefused(rate) => write!(f, "the device refused {rate} Hz"),
            DsdFallback::FormatTooNarrow => {
                write!(f, "DoP needs a 24- or 32-bit integer device format")
            }
            DsdFallback::StreamRefused => write!(f, "the device refused the DSD stream"),
        }
    }
}

/// Whether a DSD track can reach its device unchanged, before the device is
/// asked (spec O25). The engine then opens the stream; a refusal there is
/// `RateRefused`, `FormatTooNarrow` or `StreamRefused`.
pub fn dsd_decision(facts: &DsdFacts) -> Result<DsdTarget, DsdFallback> {
    let mode = match facts.mode {
        DsdOutput::Pcm => return Err(DsdFallback::ModeIsPcm),
        DsdOutput::Dop => DsdStreamMode::Dop,
        DsdOutput::Native => DsdStreamMode::Native,
    };
    let format = facts.format.ok_or(DsdFallback::NotDsd)?;
    let dsd_rate = format
        .dsd_rate
        .filter(|r| *r > 0)
        .ok_or(DsdFallback::NotDsd)?;
    if format.channels > 2 {
        return Err(DsdFallback::Multichannel);
    }
    #[allow(clippy::float_cmp)] // exactly unity: any other gain changes the signal
    if facts.volume != 1.0 {
        return Err(DsdFallback::VolumeNotUnity);
    }
    if !facts.device_idle {
        return Err(DsdFallback::DeviceBusy);
    }
    Ok(DsdTarget {
        mode,
        dsd_rate,
        word_rate: dsd_rate / DSD_WORD_BITS,
    })
}

/// A DSD stream the engine reported going out for a player's entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DsdOnAir {
    pub entry: EntryId,
    /// The mix policy in force: the engine holds other sources off the
    /// output (`DsdMix::HoldOthers`).
    pub hold_others: bool,
}

/// The player header's badge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BpBadge {
    Off,
    /// "BP": the PCM samples reach the device unchanged.
    Pcm,
    /// "DSD": the DSD stream reaches the device unchanged.
    Dsd,
}

pub fn bp_badge(bit_perfect: bool, dsd: bool) -> BpBadge {
    match (bit_perfect, dsd) {
        (_, true) => BpBadge::Dsd,
        (true, false) => BpBadge::Pcm,
        (false, false) => BpBadge::Off,
    }
}

/// Whether `player` plays DSD that holds the other sources off its output,
/// for the notice that says they are muted.
pub fn dsd_holds_others(state: &AppState, player: PlayerId) -> bool {
    state.player(player).is_ok_and(|p| {
        p.transport != Transport::Stopped
            && p.dsd
                .is_some_and(|d| d.hold_others && p.current == Some(d.entry))
    })
}
