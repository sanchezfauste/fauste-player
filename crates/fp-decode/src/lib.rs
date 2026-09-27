//! File decoding. Output is interleaved stereo `f32` at the file's own rate:
//! mono is duplicated, more channels are downmixed with ITU-R BS.775
//! coefficients. symphonia decodes most formats; DSD, WavPack and Monkey's
//! Audio have backends of their own (audio formats spec F2).

#![deny(clippy::indexing_slicing)]

use std::io::Read;
use std::path::Path;

mod dsd;
mod symph;
mod wavpack;

use dsd::DsdDecoder;
use symph::SymphoniaDecoder;
use wavpack::WavPackDecoder;

/// -3 dB, the ITU-R BS.775 weight for centre and surround channels.
const MINUS_3DB: f32 = std::f32::consts::FRAC_1_SQRT_2;

/// Which backend decodes a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Symphonia,
    Dsf,
    Dff,
    WavPack,
    Ape,
}

/// The backend for a file, from its first bytes (the content decides; the
/// extension is only a hint for formats without a signature).
pub fn probe(head: &[u8], _extension: Option<&str>) -> Kind {
    match head {
        [b'D', b'S', b'D', b' ', ..] => Kind::Dsf,
        [
            b'F',
            b'R',
            b'M',
            b'8',
            _,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
            b'D',
            b'S',
            b'D',
            b' ',
            ..,
        ] => Kind::Dff,
        [b'w', b'v', b'p', b'k', ..] => Kind::WavPack,
        [b'M', b'A', b'C', b' ', ..] => Kind::Ape,
        _ => Kind::Symphonia,
    }
}

enum Backend {
    Symphonia(Box<SymphoniaDecoder>),
    Dsd(Box<DsdDecoder>),
    WavPack(Box<WavPackDecoder>),
}

/// Decodes one file, whatever its format.
pub struct FileDecoder {
    backend: Backend,
}

impl FileDecoder {
    pub fn open(path: &Path) -> Result<Self, String> {
        let mut head = [0u8; 16];
        let read = std::fs::File::open(path)
            .and_then(|mut f| f.read(&mut head))
            .map_err(|e| format!("{}: {e}", path.display()))?;
        let extension = path.extension().and_then(|e| e.to_str());
        let backend = match probe(head.get(..read).unwrap_or_default(), extension) {
            Kind::Dsf => Backend::Dsd(Box::new(DsdDecoder::open_dsf(path)?)),
            Kind::Dff => Backend::Dsd(Box::new(DsdDecoder::open_dff(path)?)),
            Kind::WavPack => Backend::WavPack(Box::new(WavPackDecoder::open(path)?)),
            // Monkey's Audio arrives with its decoder; until then symphonia
            // tries it, as before.
            Kind::Symphonia | Kind::Ape => {
                Backend::Symphonia(Box::new(SymphoniaDecoder::open(path)?))
            }
        };
        Ok(Self { backend })
    }

    pub fn sample_rate(&self) -> u32 {
        match &self.backend {
            Backend::Symphonia(d) => d.sample_rate(),
            Backend::WavPack(d) => d.sample_rate(),
            Backend::Dsd(d) => d.sample_rate(),
        }
    }

    /// Bits per sample of integer PCM (lossless codecs); `None` for lossy
    /// codecs and for DSD, which is converted.
    pub fn bits_per_sample(&self) -> Option<u32> {
        match &self.backend {
            Backend::Symphonia(d) => d.bits_per_sample(),
            Backend::WavPack(d) => d.bits_per_sample(),
            Backend::Dsd(_) => None,
        }
    }

    /// Channels in the file (0 if the container does not say).
    pub fn channels(&self) -> usize {
        match &self.backend {
            Backend::Symphonia(d) => d.channels(),
            Backend::WavPack(d) => d.channels(),
            Backend::Dsd(d) => d.channels(),
        }
    }

    /// Total frames, when the container knows it without decoding.
    pub fn frames_hint(&self) -> Option<u64> {
        match &self.backend {
            Backend::Symphonia(d) => d.frames_hint(),
            Backend::WavPack(d) => d.frames_hint(),
            Backend::Dsd(d) => d.frames_hint(),
        }
    }

    /// Positions the stream so that the next frame produced is at `secs`.
    pub fn seek(&mut self, secs: f64) -> Result<(), String> {
        match &mut self.backend {
            Backend::Symphonia(d) => d.seek(secs),
            Backend::WavPack(d) => d.seek(secs),
            Backend::Dsd(d) => d.seek(secs),
        }
    }

    /// Appends the next decoded block as interleaved stereo. Returns `false`
    /// at the end of the stream.
    pub fn next_block(&mut self, out: &mut Vec<f32>) -> Result<bool, String> {
        match &mut self.backend {
            Backend::Symphonia(d) => d.next_block(out),
            Backend::WavPack(d) => d.next_block(out),
            Backend::Dsd(d) => d.next_block(out),
        }
    }
}

/// Stereo from any channel count. Order follows the WAV/SMPTE convention
/// (L, R, C, LFE, Ls, Rs, …); the LFE channel is dropped.
pub(crate) fn downmix(frame: &[f32]) -> (f32, f32) {
    match *frame {
        [m] => (m, m),
        [l, r] => (l, r),
        [l, r, c] => (l + MINUS_3DB * c, r + MINUS_3DB * c),
        [l, r, c, _lfe] => (l + MINUS_3DB * c, r + MINUS_3DB * c),
        [l, r, c, _lfe, ls, rs, ..] => {
            let norm = 1.0 / (1.0 + 2.0 * MINUS_3DB);
            (
                (l + MINUS_3DB * (c + ls)) * norm,
                (r + MINUS_3DB * (c + rs)) * norm,
            )
        }
        [l, r, ..] => (l, r),
        [] => (0.0, 0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::downmix;

    #[test]
    fn downmix_duplicates_mono_and_folds_centre_into_both_sides() {
        assert_eq!(downmix(&[0.5]), (0.5, 0.5));
        let (l, r) = downmix(&[0.0, 0.0, 1.0]);
        assert!((l - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6 && (r - l).abs() < 1e-6);
    }
}
