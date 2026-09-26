//! The platform-independent parts of exclusive output (Phase 4 plan 2):
//! writing samples as device bytes, the WASAPI format order, and choosing a
//! Core Audio physical format. Pure, so every OS tests them.

use cpal::FromSample;

use crate::SampleFormat;

/// Bytes per sample in the device buffer for `format`.
pub fn bytes_per_sample(format: SampleFormat) -> usize {
    match format {
        SampleFormat::I16 => 2,
        SampleFormat::F32 | SampleFormat::I32 | SampleFormat::I24 => 4,
    }
}

/// Writes `samples` into `out` as little-endian device samples, with the
/// same exact scaling as the shared-mode path (powers of two, saturating).
/// `I24` is 24 valid bits, most-significant-aligned in a 32-bit container.
/// Stops at whichever of `samples` or `out` ends first. Real-time safe.
pub fn write_samples(format: SampleFormat, samples: &[f32], out: &mut [u8]) {
    match format {
        SampleFormat::I16 => {
            for (s, o) in samples.iter().zip(out.as_chunks_mut::<2>().0.iter_mut()) {
                *o = i16::from_sample_(*s).to_le_bytes();
            }
        }
        SampleFormat::I24 | SampleFormat::I32 => {
            for (s, o) in samples.iter().zip(out.as_chunks_mut::<4>().0.iter_mut()) {
                let v = i32::from_sample_(*s);
                let v = if format == SampleFormat::I24 {
                    v & !0xff
                } else {
                    v
                };
                *o = v.to_le_bytes();
            }
        }
        SampleFormat::F32 => {
            for (s, o) in samples.iter().zip(out.as_chunks_mut::<4>().0.iter_mut()) {
                *o = s.to_le_bytes();
            }
        }
    }
}

/// One device format to try in exclusive mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Candidate {
    pub format: SampleFormat,
    pub container_bits: u16,
    pub valid_bits: u16,
}

/// The exclusive-mode formats in order of preference: 24-bit integer (holds
/// every source the mixer can pass unchanged), 32-bit integer (devices that
/// take only full 32-bit words), 16-bit integer, then float.
const CANDIDATES: [Candidate; 4] = [
    Candidate {
        format: SampleFormat::I24,
        container_bits: 32,
        valid_bits: 24,
    },
    Candidate {
        format: SampleFormat::I32,
        container_bits: 32,
        valid_bits: 32,
    },
    Candidate {
        format: SampleFormat::I16,
        container_bits: 16,
        valid_bits: 16,
    },
    Candidate {
        format: SampleFormat::F32,
        container_bits: 32,
        valid_bits: 32,
    },
];

/// The first candidate the device accepts.
pub fn negotiate(mut supported: impl FnMut(Candidate) -> bool) -> Option<Candidate> {
    CANDIDATES.into_iter().find(|c| supported(*c))
}

/// A physical (hardware) format a Core Audio device offers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicalFormat {
    pub bits: u32,
    /// Signed integer PCM (as opposed to float).
    pub integer: bool,
    /// Linear PCM, as opposed to an encoded format (AC-3, IEC 60958).
    pub linear_pcm: bool,
    pub channels: u32,
    pub min_rate: f64,
    pub max_rate: f64,
}

/// The index of the widest signed-integer linear-PCM physical format that
/// runs at `rate` with at least `channels` channels. Float and encoded
/// formats are skipped: the point is to run the hardware at the file's
/// integer sample size, never to change what it decodes.
pub fn choose_physical_format(
    formats: &[PhysicalFormat],
    rate: u32,
    channels: u32,
) -> Option<usize> {
    let rate = f64::from(rate);
    formats
        .iter()
        .enumerate()
        .filter(|(_, f)| {
            f.integer
                && f.linear_pcm
                && f.channels >= channels
                && f.min_rate <= rate
                && rate <= f.max_rate
        })
        .max_by_key(|(_, f)| (f.bits, std::cmp::Reverse(f.channels)))
        .map(|(i, _)| i)
}
