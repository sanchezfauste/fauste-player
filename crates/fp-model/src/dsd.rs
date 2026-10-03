//! DSD output (feedback 2 spec O25): the settings, and (Task 4) the pure
//! rules that decide when DSD reaches a device unchanged.

use serde::{Deserialize, Serialize};

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
