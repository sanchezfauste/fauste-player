//! Core Audio hog mode (Phase 4 plan 2), through coreaudio-rs's safe
//! helpers: exclusive access to a device, its nominal rate set to the
//! stream's, and its physical format set to the widest integer format at
//! that rate. The cpal stream then plays through it; `HogGuard` gives the
//! device back when the stream is dropped.

use coreaudio::audio_unit::macos_helpers::{
    get_audio_device_ids, get_device_name, get_hogging_pid, get_supported_physical_stream_formats,
    set_device_physical_stream_format, set_device_sample_rate, toggle_hog_mode,
};
use objc2_core_audio_types::{kAudioFormatFlagIsFloat, kAudioFormatFlagIsSignedInteger};

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

/// Takes hog mode on `name` and prepares it for `rate`. Returns the guard
/// and the sample format the hardware now runs in.
pub(crate) fn take(name: &str, rate: u32) -> Result<(HogGuard, SampleFormat), BackendError> {
    let device = find(name)?;
    let me = this_process();
    match get_hogging_pid(device).map_err(|e| unsupported("hog mode", e))? {
        NOBODY => {
            let owner = toggle_hog_mode(device).map_err(|e| unsupported("hog mode", e))?;
            if owner != me {
                return Err(BackendError::Unsupported(
                    "another application holds the device".to_owned(),
                ));
            }
        }
        owner if owner == me => {}
        _ => {
            return Err(BackendError::Unsupported(
                "another application holds the device".to_owned(),
            ));
        }
    }
    // From here on the guard gives the device back on any failure.
    let guard = HogGuard { device };
    set_device_sample_rate(device, f64::from(rate)).map_err(|e| unsupported("sample rate", e))?;
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
                min_rate: r.mSampleRateRange.mMinimum,
                max_rate: r.mSampleRateRange.mMaximum,
            }
        })
        .collect();
    let format = match choose_physical_format(&offered, rate) {
        Some(chosen) => {
            let index = offered.iter().position(|f| *f == chosen).unwrap_or(0);
            if let Some(r) = ranged.get(index) {
                let mut asbd = r.mFormat;
                asbd.mSampleRate = f64::from(rate);
                set_device_physical_stream_format(device, asbd)
                    .map_err(|e| unsupported("physical format", e))?;
            }
            if chosen.bits >= 24 {
                SampleFormat::I24
            } else {
                SampleFormat::I16
            }
        }
        // Float hardware only: the HAL passes cpal's f32 through.
        None => SampleFormat::F32,
    };
    Ok((guard, format))
}
