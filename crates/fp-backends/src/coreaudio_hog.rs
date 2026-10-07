//! Core Audio hog mode (Phase 4 plan 2), through coreaudio-rs's safe
//! helpers: exclusive access to a device, its nominal rate set to the
//! stream's, and its physical format set to the widest integer format at
//! that rate. The cpal stream then plays through it; `HogGuard` gives the
//! device back when the stream is dropped.

use coreaudio::audio_unit::macos_helpers::{
    get_audio_device_ids, get_device_name, get_hogging_pid, get_supported_physical_stream_formats,
    set_device_physical_stream_format, set_device_sample_rate, toggle_hog_mode,
};
use objc2_core_audio_types::{
    kAudioFormatFlagIsFloat, kAudioFormatFlagIsSignedInteger, kAudioFormatLinearPCM,
};

use crate::exclusive::{PhysicalFormat, choose_physical_format};
use crate::{BackendError, SampleFormat};

/// No process holds the device.
const NOBODY: i32 = -1;

fn this_process() -> i32 {
    i32::try_from(std::process::id()).unwrap_or(i32::MAX)
}

fn unsupported(what: &str, e: impl std::fmt::Debug) -> BackendError {
    BackendError::Unsupported(format!("{what}: {e:?}"))
}

/// Holds hog mode on a device while the stream lives.
pub(crate) struct HogGuard {
    device: u32,
}

impl Drop for HogGuard {
    fn drop(&mut self) {
        if get_hogging_pid(self.device).is_ok_and(|pid| pid == this_process()) {
            let _ = toggle_hog_mode(self.device);
        }
    }
}

/// The Core Audio device named `name`. Refused when several share the name,
/// since hog mode on the wrong one would silence another program.
fn find(name: &str) -> Result<u32, BackendError> {
    let ids = get_audio_device_ids().map_err(|e| unsupported("listing devices", e))?;
    let mut named = ids
        .into_iter()
        .filter(|id| get_device_name(*id).is_ok_and(|n| n == name));
    match (named.next(), named.next()) {
        (Some(id), None) => Ok(id),
        (None, _) => Err(BackendError::Unsupported(format!(
            "no Core Audio device named {name}"
        ))),
        (Some(_), Some(_)) => Err(BackendError::Unsupported(format!(
            "several devices are named {name}; hog mode needs a unique name"
        ))),
    }
}

/// Takes hog mode on `name`. The guard gives it back when dropped.
pub(crate) fn take(name: &str) -> Result<HogGuard, BackendError> {
    let device = find(name)?;
    let me = this_process();
    match get_hogging_pid(device).map_err(|e| unsupported("hog mode", e))? {
        NOBODY => {
            let owner = toggle_hog_mode(device).map_err(|e| unsupported("hog mode", e))?;
            if owner != me {
                return Err(BackendError::Busy(
                    "another application holds the device".to_owned(),
                ));
            }
        }
        // Only one bus opens a device, so this process holding it already
        // means a previous stream of that bus; its guard is gone.
        owner if owner == me => {}
        _ => {
            return Err(BackendError::Busy(
                "another application holds the device".to_owned(),
            ));
        }
    }
    Ok(HogGuard { device })
}

impl HogGuard {
    /// Sets the nominal rate and the widest integer linear-PCM physical
    /// format at that rate with at least `channels` channels. Called after
    /// the stream is built, since building sets a physical format of its
    /// own. Returns the format the hardware now runs in.
    pub(crate) fn prepare(&self, rate: u32, channels: u32) -> Result<SampleFormat, BackendError> {
        let device = self.device;
        set_device_sample_rate(device, f64::from(rate))
            .map_err(|e| unsupported("sample rate", e))?;
        let ranged = get_supported_physical_stream_formats(device)
            .map_err(|e| unsupported("physical formats", e))?;
        let offered: Vec<PhysicalFormat> = ranged
            .iter()
            .map(|r| {
                let flags = r.mFormat.mFormatFlags;
                PhysicalFormat {
                    bits: r.mFormat.mBitsPerChannel,
                    integer: flags & kAudioFormatFlagIsSignedInteger != 0
                        && flags & kAudioFormatFlagIsFloat == 0,
                    linear_pcm: r.mFormat.mFormatID == kAudioFormatLinearPCM,
                    channels: r.mFormat.mChannelsPerFrame,
                    min_rate: r.mSampleRateRange.mMinimum,
                    max_rate: r.mSampleRateRange.mMaximum,
                }
            })
            .collect();
        let chosen = choose_physical_format(&offered, rate, channels)
            .and_then(|i| Some((ranged.get(i)?, offered.get(i)?)));
        let Some((r, format)) = chosen else {
            // Float hardware only: the HAL passes cpal's f32 through.
            return Ok(SampleFormat::F32);
        };
        let mut asbd = r.mFormat;
        asbd.mSampleRate = f64::from(rate);
        set_device_physical_stream_format(device, asbd)
            .map_err(|e| unsupported("physical format", e))?;
        Ok(if format.bits >= 24 {
            SampleFormat::I24
        } else {
            SampleFormat::I16
        })
    }
}
