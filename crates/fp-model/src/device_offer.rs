//! What Settings → Audio outputs offers a routed device (operator feedback
//! 4, Q3 and Q12): the DSD modes it can take and why the others are not
//! offered, and the rates and buffers it reports.

use crate::dsd::DsdOutput;

/// What Settings knows about a routed device when it offers DSD modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DsdCaps {
    /// The device is plugged in (reported by the backend).
    pub connected: bool,
    /// The device is in `outputs.bit_perfect`.
    pub bit_perfect: bool,
    /// The device can be opened exclusively; `false` when it is not plugged in.
    pub exclusive_capable: bool,
    /// The device reports a DSD sample format.
    pub native_dsd: bool,
    /// The application runs on Linux (native DSD goes through ALSA).
    pub linux: bool,
}

/// Why DoP or native DSD is not offered (Q3.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DsdNotOffered {
    /// Neither: the device is not plugged in, so nothing is known of it.
    NotConnected,
    /// Neither: the device cannot be opened exclusively.
    NotExclusive,
    /// Neither: the device is not bit-perfect.
    BitPerfectOff,
    /// Native DSD: not on this system.
    NativeNeedsLinux,
    /// Native DSD: the device reports no DSD format.
    NoNativeDsd,
}

/// The DSD modes a device can be given: PCM always; DoP when it is
/// bit-perfect and exclusive-capable; native DSD when it is bit-perfect, on
/// Linux, and reports a DSD format. The configured mode stays listed so it
/// can be changed back.
pub fn offered_dsd_modes(caps: DsdCaps, configured: DsdOutput) -> Vec<DsdOutput> {
    let mut modes = vec![DsdOutput::Pcm];
    if caps.bit_perfect && caps.exclusive_capable {
        modes.push(DsdOutput::Dop);
    }
    if caps.bit_perfect && caps.linux && caps.native_dsd {
        modes.push(DsdOutput::Native);
    }
    if !modes.contains(&configured) {
        modes.push(configured);
        modes.sort_by_key(|m| match m {
            DsdOutput::Pcm => 0,
            DsdOutput::Dop => 1,
            DsdOutput::Native => 2,
        });
    }
    modes
}

/// The mode a device really plays: `configured` when this device can take
/// it, PCM otherwise. The configuration is not rewritten, so the mode comes
/// back when the device can take it again.
pub fn effective_dsd_mode(caps: DsdCaps, configured: DsdOutput) -> DsdOutput {
    if offered_dsd_modes(caps, DsdOutput::Pcm).contains(&configured) {
        configured
    } else {
        DsdOutput::Pcm
    }
}

/// Why a mode is missing from [`offered_dsd_modes`]: one reason, the most
/// fundamental first, since it is the one the operator must act on first;
/// `None` when every mode is offered.
pub fn dsd_not_offered(caps: DsdCaps) -> Option<DsdNotOffered> {
    if !caps.connected {
        Some(DsdNotOffered::NotConnected)
    } else if !caps.exclusive_capable {
        Some(DsdNotOffered::NotExclusive)
    } else if !caps.bit_perfect {
        Some(DsdNotOffered::BitPerfectOff)
    } else if !caps.linux {
        Some(DsdNotOffered::NativeNeedsLinux)
    } else if !caps.native_dsd {
        Some(DsdNotOffered::NoNativeDsd)
    } else {
        None
    }
}

/// Whether the device reports `rate` (it does when it reports no ranges).
pub fn rate_is_reported(reported: &[(u32, u32)], rate: u32) -> bool {
    reported.is_empty() || reported.iter().any(|(lo, hi)| (*lo..=*hi).contains(&rate))
}

/// Whether the device reports `frames` (it does when it reports no range).
pub fn buffer_is_reported(reported: Option<(u32, u32)>, frames: u32) -> bool {
    reported.is_none_or(|(lo, hi)| (lo..=hi).contains(&frames))
}

/// The `candidates` within the device's reported rate ranges (all of them
/// when it reports none), with `current` kept, in ascending order. A device
/// asked to open at a rate it refuses stays silent, so only what it reports
/// is offered.
pub fn offered_rates(
    reported: &[(u32, u32)],
    candidates: &[u32],
    current: Option<u32>,
) -> Vec<u32> {
    let fits = |r: u32| rate_is_reported(reported, r);
    with_current(
        candidates.iter().copied().filter(|r| fits(*r)).collect(),
        current,
    )
}

/// The `candidates` within the device's reported buffer range (all of them
/// when it reports none), with `current` kept, in ascending order.
pub fn offered_buffers(
    reported: Option<(u32, u32)>,
    candidates: &[u32],
    current: Option<u32>,
) -> Vec<u32> {
    let fits = |b: u32| buffer_is_reported(reported, b);
    with_current(
        candidates.iter().copied().filter(|b| fits(*b)).collect(),
        current,
    )
}

fn with_current(mut values: Vec<u32>, current: Option<u32>) -> Vec<u32> {
    if let Some(c) = current
        && !values.contains(&c)
    {
        values.push(c);
        values.sort_unstable();
    }
    values
}
