//! File decoding. Output is interleaved stereo `f32` at the file's own rate:
//! mono is duplicated, more channels are downmixed with ITU-R BS.775
//! coefficients. symphonia decodes most formats; DSD, WavPack and Monkey's
//! Audio have backends of their own (audio formats spec F2).

#![deny(clippy::indexing_slicing)]

use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

mod ape;
mod dsd;
mod opus;
pub mod priority;
mod symph;
mod wavpack;

use ape::ApeFileDecoder;
use dsd::DsdDecoder;
pub use dsd::DsdRawReader;
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

/// Extensions of the formats that have a backend of their own.
fn own_format_extension(extension: &str) -> bool {
    ["dsf", "dff", "wv", "ape"]
        .iter()
        .any(|e| e.eq_ignore_ascii_case(extension))
}

/// The length of an ID3v2 tag at the start of `head`, header and footer
/// included.
fn id3v2_len(head: &[u8]) -> Option<u64> {
    if head.get(0..3)? != b"ID3" {
        return None;
    }
    let flags = *head.get(5)?;
    let size = head.get(6..10)?.iter().try_fold(0u64, |acc, b| {
        (b & 0x80 == 0).then_some(acc << 7 | u64::from(*b))
    })?;
    let footer = if flags & 0x10 != 0 { 10 } else { 0 };
    Some(10 + size + footer)
}

/// Reads until `buf` is full or the file ends.
fn read_up_to(file: &mut std::fs::File, buf: &mut [u8]) -> std::io::Result<usize> {
    let mut filled = 0;
    while let Some(rest) = buf.get_mut(filled..).filter(|r| !r.is_empty()) {
        match file.read(rest)? {
            0 => break,
            n => filled += n,
        }
    }
    Ok(filled)
}

/// Seconds of `frames` frames at `rate`: `None` when the count is unknown
/// or zero, or the rate is zero (operator feedback 4, Q1).
pub fn duration_from_frames(frames: Option<u64>, rate: u32) -> Option<f64> {
    let frames = frames.filter(|f| *f > 0)?;
    if rate == 0 {
        return None;
    }
    let secs = frames as f64 / f64::from(rate);
    (secs.is_finite() && secs > 0.0).then_some(secs)
}

enum Backend {
    Symphonia(Box<SymphoniaDecoder>),
    Dsd(Box<DsdDecoder>),
    WavPack(Box<WavPackDecoder>),
    Ape(Box<ApeFileDecoder>),
}

/// Decodes one file, whatever its format.
pub struct FileDecoder {
    backend: Backend,
    /// The file, for the log line.
    path: String,
    /// Set once non-finite samples were logged for this file.
    logged_non_finite: bool,
}

impl FileDecoder {
    pub fn open(path: &Path) -> Result<Self, String> {
        let io = |e: std::io::Error| format!("{}: {e}", path.display());
        let mut file = std::fs::File::open(path).map_err(io)?;
        let mut head = [0u8; 16];
        let read = read_up_to(&mut file, &mut head).map_err(io)?;
        // A leading ID3v2 tag hides the signature of WavPack and Monkey's
        // Audio files; look past it.
        let start = id3v2_len(head.get(..read).unwrap_or_default()).unwrap_or(0);
        let read = if start > 0 {
            file.seek(SeekFrom::Start(start)).map_err(io)?;
            read_up_to(&mut file, &mut head).map_err(io)?
        } else {
            read
        };
        let extension = path.extension().and_then(|e| e.to_str());
        let backend = match probe(head.get(..read).unwrap_or_default(), extension) {
            Kind::Dsf => Backend::Dsd(Box::new(DsdDecoder::open_dsf(path)?)),
            Kind::Dff => Backend::Dsd(Box::new(DsdDecoder::open_dff(path)?)),
            Kind::WavPack => Backend::WavPack(Box::new(WavPackDecoder::open(path, start)?)),
            Kind::Ape => Backend::Ape(Box::new(ApeFileDecoder::open(path)?)),
            // symphonia can take almost anything for MPEG and play noise, so
            // a file whose extension promises one of our own formats is
            // refused rather than handed to it.
            Kind::Symphonia if extension.is_some_and(own_format_extension) => {
                return Err(format!(
                    "{}: not a valid {} file",
                    path.display(),
                    extension.unwrap_or_default()
                ));
            }
            Kind::Symphonia => Backend::Symphonia(Box::new(SymphoniaDecoder::open(path)?)),
        };
        Ok(Self {
            backend,
            path: path.display().to_string(),
            logged_non_finite: false,
        })
    }

    pub fn sample_rate(&self) -> u32 {
        match &self.backend {
            Backend::Symphonia(d) => d.sample_rate(),
            Backend::WavPack(d) => d.sample_rate(),
            Backend::Ape(d) => d.sample_rate(),
            Backend::Dsd(d) => d.sample_rate(),
        }
    }

    /// Bits per sample of integer PCM (lossless codecs); `None` for lossy
    /// codecs and for DSD, which is converted.
    pub fn bits_per_sample(&self) -> Option<u32> {
        match &self.backend {
            Backend::Symphonia(d) => d.bits_per_sample(),
            Backend::WavPack(d) => d.bits_per_sample(),
            Backend::Ape(d) => d.bits_per_sample(),
            Backend::Dsd(_) => None,
        }
    }

    /// The DSD sample rate of a DSD file; `None` for every other format.
    pub fn dsd_rate(&self) -> Option<u32> {
        match &self.backend {
            Backend::Dsd(d) => Some(d.dsd_rate()),
            _ => None,
        }
    }

    /// Channels in the file (0 if the container does not say).
    pub fn channels(&self) -> usize {
        match &self.backend {
            Backend::Symphonia(d) => d.channels(),
            Backend::WavPack(d) => d.channels(),
            Backend::Ape(d) => d.channels(),
            Backend::Dsd(d) => d.channels(),
        }
    }

    /// Total frames, when the container knows it without decoding.
    pub fn frames_hint(&self) -> Option<u64> {
        match &self.backend {
            Backend::Symphonia(d) => d.frames_hint(),
            Backend::WavPack(d) => d.frames_hint(),
            Backend::Ape(d) => d.frames_hint(),
            Backend::Dsd(d) => d.frames_hint(),
        }
    }

    /// The length the file stores, in seconds, without decoding (operator
    /// feedback 4, Q1): what a track shows until its analysis. `None`
    /// unless the format is known to store it: an estimate (a raw AAC
    /// stream, an MPEG stream with no Xing, Info or VBRI frame) can end a
    /// VBR track early. WavPack sums its block headers, Monkey's Audio and
    /// DSD read their headers' sample counts. For MPEG audio this reads the
    /// file's first frame again.
    pub fn duration_hint_secs(&self) -> Option<f64> {
        let frames = match &self.backend {
            Backend::Symphonia(d) => d.declared_frames(),
            Backend::WavPack(d) => d.frames_hint(),
            Backend::Ape(d) => d.frames_hint(),
            Backend::Dsd(d) => d.frames_hint(),
        };
        duration_from_frames(frames, self.sample_rate())
    }

    /// Positions the stream so that the next frame produced is at `secs`.
    pub fn seek(&mut self, secs: f64) -> Result<(), String> {
        match &mut self.backend {
            Backend::Symphonia(d) => d.seek(secs),
            Backend::WavPack(d) => d.seek(secs),
            Backend::Ape(d) => d.seek(secs),
            Backend::Dsd(d) => d.seek(secs),
        }
    }

    /// Appends the next decoded block as interleaved stereo. Returns `false`
    /// at the end of the stream.
    ///
    /// NaN and infinite samples (a float file with bad data) are replaced
    /// by silence, and the first of a file is logged.
    pub fn next_block(&mut self, out: &mut Vec<f32>) -> Result<bool, String> {
        let start = out.len();
        let more = match &mut self.backend {
            Backend::Symphonia(d) => d.next_block(out),
            Backend::WavPack(d) => d.next_block(out),
            Backend::Ape(d) => d.next_block(out),
            Backend::Dsd(d) => d.next_block(out),
        }?;
        if let Some(fresh) = out.get_mut(start..)
            && silence_non_finite(fresh)
            && !self.logged_non_finite
        {
            self.logged_non_finite = true;
            tracing::warn!(
                file = %self.path,
                "non-finite samples in the file were replaced by silence"
            );
        }
        Ok(more)
    }
}

/// Replaces NaN and infinite samples by 0. Returns whether any was found.
fn silence_non_finite(samples: &mut [f32]) -> bool {
    let mut found = false;
    for s in samples.iter_mut().filter(|s| !s.is_finite()) {
        *s = 0.0;
        found = true;
    }
    found
}

/// Stereo from any channel count. Order follows the WAV/SMPTE convention
/// (L, R, C, LFE, Ls, Rs, …); the LFE channel is dropped.
pub(crate) fn downmix(frame: &[f32]) -> (f32, f32) {
    match *frame {
        [m] => (m, m),
        [l, r] => (l, r),
        [l, r, c] => (l + MINUS_3DB * c, r + MINUS_3DB * c),
        [l, r, c, _lfe] => (l + MINUS_3DB * c, r + MINUS_3DB * c),
        // 5.0: L R C Ls Rs.
        [l, r, c, ls, rs] => {
            let norm = 1.0 / (1.0 + 2.0 * MINUS_3DB);
            (
                (l + MINUS_3DB * (c + ls)) * norm,
                (r + MINUS_3DB * (c + rs)) * norm,
            )
        }
        [l, r, c, _lfe, ls, rs, ..] => {
            let norm = 1.0 / (1.0 + 2.0 * MINUS_3DB);
            (
                (l + MINUS_3DB * (c + ls)) * norm,
                (r + MINUS_3DB * (c + rs)) * norm,
            )
        }
        [] => (0.0, 0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::{downmix, silence_non_finite};

    #[test]
    fn non_finite_samples_become_silence_and_are_reported() {
        let mut s = [0.5, f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -0.25];
        assert!(silence_non_finite(&mut s));
        assert_eq!(s, [0.5, 0.0, 0.0, 0.0, -0.25]);
        assert!(!silence_non_finite(&mut s));
    }

    #[test]
    fn downmix_duplicates_mono_and_folds_centre_into_both_sides() {
        assert_eq!(downmix(&[0.5]), (0.5, 0.5));
        let (l, r) = downmix(&[0.0, 0.0, 1.0]);
        assert!((l - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6 && (r - l).abs() < 1e-6);
    }

    #[test]
    fn downmix_keeps_the_centre_of_five_channels() {
        // L R C Ls Rs (5.0): the centre reaches both sides.
        let (l, r) = downmix(&[0.0, 0.0, 1.0, 0.0, 0.0]);
        assert!(l > 0.2 && (l - r).abs() < 1e-6, "{l} {r}");
        let (l, r) = downmix(&[0.0, 0.0, 0.0, 1.0, 0.0]);
        assert!(
            l > 0.2 && r.abs() < 1e-6,
            "left surround stays left: {l} {r}"
        );
    }
}
