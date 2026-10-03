//! DSD carried through the engine's f32 path (feedback 2 spec O25). A DSD
//! stream is a stream of 16-bit words (16 DSD bits per channel, the first
//! byte first in time, each byte most significant bit first) at the word
//! rate, DSD rate ÷ 16. Each word travels exactly as one f32 sample. Pure
//! and real-time safe: no allocation, no panics.

/// The DSD idle pattern: a decoder turns it into silence.
pub const DSD_SILENCE: u8 = 0x69;
/// DoP 1.1 markers, alternating frame by frame, the same in every channel
/// of one frame.
pub const DOP_MARKERS: [u8; 2] = [0x05, 0xFA];

/// How a stream carries DSD (`StreamConfig::dsd`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DsdStream {
    /// DSD over PCM (DoP 1.1) in a 24- or 32-bit integer PCM stream.
    Dop,
    /// Raw DSD in one of the device's DSD formats.
    Native,
}

/// Native DSD sample formats (ALSA `DSD_U32_BE` … `DSD_U8`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NativeDsdFormat {
    U32Be,
    U32Le,
    U16Be,
    U16Le,
    U8,
}

/// The order in which native formats are tried.
pub const NATIVE_PREFERENCE: [NativeDsdFormat; 5] = [
    NativeDsdFormat::U32Be,
    NativeDsdFormat::U32Le,
    NativeDsdFormat::U16Be,
    NativeDsdFormat::U16Le,
    NativeDsdFormat::U8,
];

impl NativeDsdFormat {
    /// DSD bytes per channel in one device frame.
    pub fn bytes(self) -> usize {
        match self {
            NativeDsdFormat::U32Be | NativeDsdFormat::U32Le => 4,
            NativeDsdFormat::U16Be | NativeDsdFormat::U16Le => 2,
            NativeDsdFormat::U8 => 1,
        }
    }

    /// The device's frame rate for a word stream at `word_rate`.
    pub fn device_rate(self, word_rate: u32) -> u32 {
        word_rate.saturating_mul(2) / self.bytes() as u32
    }
}

/// The first native format, in preference order, that `accepts` takes.
pub fn choose_native_format(
    mut accepts: impl FnMut(NativeDsdFormat) -> bool,
) -> Option<NativeDsdFormat> {
    NATIVE_PREFERENCE.into_iter().find(|f| accepts(*f))
}

/// One word as an f32 sample, exactly (an `i16` over 32768).
pub fn word_to_sample(first: u8, second: u8) -> f32 {
    f32::from(i16::from_be_bytes([first, second])) / 32768.0
}

/// The word an f32 sample carries (the inverse of `word_to_sample`).
pub fn sample_to_word(sample: f32) -> [u8; 2] {
    // Saturating float-to-int conversion; NaN reads as 0.
    let v = (sample * 32768.0).round().clamp(-32768.0, 32767.0) as i16;
    v.to_be_bytes()
}

/// The DSD idle word as a sample.
pub fn silence_sample() -> f32 {
    word_to_sample(DSD_SILENCE, DSD_SILENCE)
}

/// One DoP sample: `marker` above the word's two bytes, as 24-bit PCM in an
/// f32 (exact: 24 bits fit the mantissa, and the integer conversions scale
/// by powers of two).
pub fn dop_sample(marker: u8, word: f32) -> f32 {
    let [a, b] = sample_to_word(word);
    let v = (u32::from(marker) << 16) | (u32::from(a) << 8) | u32::from(b);
    // Sign-extend the 24-bit word.
    let signed = ((v << 8) as i32) >> 8;
    signed as f32 / 8_388_608.0
}

/// Turns word frames into DoP frames, keeping the marker alternation across
/// calls (one encoder per open stream).
#[derive(Debug, Clone, Copy, Default)]
pub struct DopEncoder {
    odd: bool,
}

impl DopEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Replaces each frame of words in `buf` (`channels` per frame) by its
    /// DoP samples. Real-time safe.
    pub fn encode_in_place(&mut self, buf: &mut [f32], channels: usize) {
        let [even, odd] = DOP_MARKERS;
        for frame in buf.chunks_exact_mut(channels.max(1)) {
            let marker = if self.odd { odd } else { even };
            for s in frame.iter_mut() {
                *s = dop_sample(marker, *s);
            }
            self.odd = !self.odd;
        }
    }
}

/// Packs word frames (`channels` per frame) into device bytes for `format`;
/// returns the bytes written. 32-bit formats take two word frames per
/// device frame (an odd last word frame is left out: streams render an
/// even number of frames). Stops where `out` ends. Real-time safe.
pub fn pack_native(
    format: NativeDsdFormat,
    words: &[f32],
    channels: usize,
    out: &mut [u8],
) -> usize {
    let channels = channels.max(1);
    let mut written = 0;
    let mut put = |byte: u8, written: &mut usize| {
        if let Some(o) = out.get_mut(*written) {
            *o = byte;
            *written += 1;
        }
    };
    match format {
        NativeDsdFormat::U16Be | NativeDsdFormat::U16Le => {
            for s in words {
                let [a, b] = sample_to_word(*s);
                let (x, y) = if format == NativeDsdFormat::U16Be {
                    (a, b)
                } else {
                    (b, a)
                };
                put(x, &mut written);
                put(y, &mut written);
            }
        }
        NativeDsdFormat::U8 => {
            for frame in words.chunks_exact(channels) {
                for half in 0..2 {
                    for s in frame {
                        let [a, b] = sample_to_word(*s);
                        put(if half == 0 { a } else { b }, &mut written);
                    }
                }
            }
        }
        NativeDsdFormat::U32Be | NativeDsdFormat::U32Le => {
            for pair in words.chunks_exact(channels * 2) {
                let (first, second) = pair.split_at(channels);
                for (s1, s2) in first.iter().zip(second) {
                    let [a, b] = sample_to_word(*s1);
                    let [c, d] = sample_to_word(*s2);
                    let bytes = if format == NativeDsdFormat::U32Be {
                        [a, b, c, d]
                    } else {
                        [d, c, b, a]
                    };
                    for byte in bytes {
                        put(byte, &mut written);
                    }
                }
            }
        }
    }
    written
}
