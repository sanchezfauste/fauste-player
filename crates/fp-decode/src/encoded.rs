//! Encoded bitstreams stored as PCM. A DTS-CD rip is a DTS stream written
//! into the samples of a 16-bit stereo PCM file (WAV, FLAC, AIFF): played as
//! PCM it is loud noise. This module recognises such a payload from the
//! first samples of a file so that it is refused instead. DTS and AC-3
//! decoding is out of scope.

/// Stereo frames examined when a file is opened (after any leading digital
/// silence): 256 KiB of 16-bit samples, a few DTS frames and three bursts
/// of an IEC 61937 stream with a period of up to 32768 frames.
pub(crate) const SNIFF_FRAMES: usize = 65_536;

/// Frames of leading digital silence skipped before the window starts, at
/// most (about six seconds at 44.1 kHz): a rip may begin with a gap.
pub(crate) const MAX_LEADING_SILENCE_FRAMES: usize = 262_144;

/// Syncs needed, at one spacing, to call a payload an encoded bitstream.
const MIN_SYNCS: usize = 3;

/// What the payload turned out to be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Encoded {
    /// DTS core frames, in any of the four stream forms.
    Dts,
    /// IEC 61937 bursts (compressed audio as carried over S/PDIF).
    Iec61937,
}

impl Encoded {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Dts => "DTS",
            Self::Iec61937 => "IEC 61937",
        }
    }
}

/// IEC 61937 data types (Pc, low 5 bits) a burst may carry: AC-3, pause,
/// the MPEG audio and AAC types, DTS, ATRAC, WMA, E-AC-3 and MAT. 0 (null)
/// and the reserved values are not accepted.
fn known_iec_type(pc: u16) -> bool {
    matches!(pc & 0x1F, 1 | 3..=22)
}

/// A sync found in the samples: where it starts and the least distance, in
/// 16-bit words, that the next sync may be at (the frame or burst it opens).
struct Candidate {
    at: usize,
    min_spacing: usize,
}

/// The four header words of a DTS core frame at the start of `w`, in the
/// 16-bit big-endian form whatever the stream form, and whether the stream
/// is 14-bit packed. `None` when `w` does not start with a DTS sync.
fn dts_header(w: &[i16]) -> Option<([u16; 4], bool)> {
    let word = |i: usize| w.get(i).map(|v| *v as u16);
    let swap = |v: u16| v.swap_bytes();
    let (a, b) = (word(0)?, word(1)?);
    match (a, b) {
        (0x7FFE, 0x8001) => Some(([a, b, word(2)?, word(3)?], false)),
        (0xFE7F, 0x0180) => Some(([swap(a), swap(b), swap(word(2)?), swap(word(3)?)], false)),
        (0x1FFF, 0xE800) | (0xFF1F, 0x00E8) => {
            let swapped = a == 0xFF1F;
            // Five 14-bit words hold the 64 bits of the four header words.
            let mut acc = 0u128;
            for i in 0..5 {
                let v = word(i)?;
                let v = if swapped { swap(v) } else { v };
                acc = acc << 14 | u128::from(v & 0x3FFF);
            }
            let bits = (acc >> 6) as u64;
            Some((
                [
                    (bits >> 48) as u16,
                    (bits >> 32) as u16,
                    (bits >> 16) as u16,
                    bits as u16,
                ],
                true,
            ))
        }
        _ => None,
    }
}

/// The DTS core header at the start of `w`, checked: sync, FTYPE = 1
/// (normal frame), SHORT = 31, at least 5 blocks and a frame size of 96
/// bytes or more. Returns the frame size in 16-bit words of the stream.
fn dts_frame_words(w: &[i16]) -> Option<usize> {
    let (h, packed14) = dts_header(w)?;
    let [s0, s1, w2, w3] = h;
    // FTYPE (1 bit) and SHORT (5 bits) are all ones in a normal frame.
    if s0 != 0x7FFE || s1 != 0x8001 || w2 >> 10 != 0x3F {
        return None;
    }
    let blocks = (w2 >> 2) & 0x7F;
    // FSIZE: 14 bits, the frame size in bytes minus one.
    let bytes = ((usize::from(w2) & 3) << 12 | usize::from(w3) >> 4) + 1;
    if blocks < 5 || bytes < 96 {
        return None;
    }
    Some(if packed14 {
        (bytes * 4).div_ceil(7)
    } else {
        bytes / 2
    })
}

/// The IEC 61937 burst at the start of `w` (Pa, Pb, Pc, Pd): the burst size
/// in words, when Pc names a known data type.
fn iec_burst_words(w: &[i16]) -> Option<usize> {
    let word = |i: usize| w.get(i).map(|v| *v as u16);
    let (a, b, pc, pd) = (word(0)?, word(1)?, word(2)?, word(3)?);
    let (pc, pd) = match (a, b) {
        (0xF872, 0x4E1F) => (pc, pd),
        (0x72F8, 0x1F4E) => (pc.swap_bytes(), pd.swap_bytes()),
        _ => return None,
    };
    // Pd is the payload length in bits (bytes for a few types: then this
    // bound is only looser).
    known_iec_type(pc).then(|| 4 + usize::from(pd).div_ceil(16))
}

/// Whether `samples` (interleaved 16-bit words, in file order) carry an
/// encoded bitstream: at least three syncs with valid headers, the last
/// three of them at the same spacing, which is no shorter than the frame
/// they open. Plain PCM does not do this by chance: a 32-bit pattern has to
/// repeat at an exact period and a header has to follow it.
pub(crate) fn detect(samples: &[i16]) -> Option<Encoded> {
    if has_regular_syncs(samples, dts_frame_words) {
        Some(Encoded::Dts)
    } else if has_regular_syncs(samples, iec_burst_words) {
        Some(Encoded::Iec61937)
    } else {
        None
    }
}

fn has_regular_syncs(samples: &[i16], header: fn(&[i16]) -> Option<usize>) -> bool {
    let mut found: Vec<Candidate> = Vec::new();
    for at in 0..samples.len() {
        let Some(rest) = samples.get(at..) else { break };
        if let Some(min_spacing) = header(rest) {
            found.push(Candidate { at, min_spacing });
        }
    }
    found.windows(MIN_SYNCS).any(|w| {
        matches!(w, [x, y, z]
            if y.at - x.at == z.at - y.at
                && y.at - x.at >= x.min_spacing.max(y.min_spacing).max(z.min_spacing))
    })
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::*;

    /// The header words of a DTS core frame of `bytes` bytes in one stream
    /// form: 16-bit or 14-bit packed, big- or little-endian.
    pub(crate) fn dts_words(packed14: bool, little: bool, bytes: usize) -> Vec<u16> {
        let fsize = (bytes - 1) as u16;
        // sync, FTYPE 1, SHORT 31, CPF 0, NBLKS 15, FSIZE, AMODE 2 (first bits)
        let w2 = 0xFC00 | (15 << 2) | (fsize >> 12);
        let w3 = (fsize << 4) | 0x2;
        let be = [0x7FFEu16, 0x8001, w2, w3];
        let mut out: Vec<u16> = if packed14 {
            // 64 bits as 14-bit payloads, sign-extended to 16 bits.
            let bits = be.iter().fold(0u128, |a, w| a << 16 | u128::from(*w)) << 6;
            (0..5)
                .map(|i| {
                    let v = ((bits >> (70 - 14 * (i + 1))) & 0x3FFF) as u16;
                    if v & 0x2000 != 0 { v | 0xC000 } else { v }
                })
                .collect()
        } else {
            be.to_vec()
        };
        if little {
            for w in &mut out {
                *w = w.swap_bytes();
            }
        }
        out
    }

    /// A payload with `head` every `period` words over varied filler.
    fn stream(head: &[u16], period: usize, count: usize) -> Vec<i16> {
        let mut v = vec![0x1234i16; period * count + 8];
        for n in 0..count {
            for (k, w) in head.iter().enumerate() {
                v[10 + n * period + k] = *w as i16;
            }
        }
        // Not silence in between: varied words.
        for (i, s) in v.iter_mut().enumerate() {
            if *s == 0x1234 {
                *s = (i * 37 % 20_000) as i16 + 100;
            }
        }
        v
    }

    #[test]
    fn each_dts_form_is_detected() {
        for (name, packed14, little) in [
            ("16-bit BE", false, false),
            ("16-bit LE", false, true),
            ("14-bit BE", true, false),
            ("14-bit LE", true, true),
        ] {
            // 4096 bytes: 2048 words of a 16-bit stream, 2341 of a packed one.
            let head = dts_words(packed14, little, 4096);
            assert_eq!(
                detect(&stream(&head, 2400, 8)),
                Some(Encoded::Dts),
                "{name}"
            );
        }
    }

    #[test]
    fn the_real_14_bit_header_is_valid() {
        // 1FFF E800 07Fx .. as found in a DTS-CD rip.
        let packed = dts_words(true, false, 3584);
        assert_eq!(packed.get(..2), Some(&[0x1FFF, 0xE800][..]));
        assert_eq!(packed.get(2).map(|w| w & 0xFFF0), Some(0x07F0));
        assert_eq!(dts_frame_words(&to_i16(&packed)), Some(2048));
    }

    fn to_i16(w: &[u16]) -> Vec<i16> {
        w.iter().map(|v| *v as i16).collect()
    }

    #[test]
    fn a_sync_closer_than_its_frame_size_is_not_detected() {
        // The header says 8192 bytes (4096 words) but the syncs are 2048
        // words apart.
        let head = dts_words(false, false, 8192);
        assert_eq!(detect(&stream(&head, 2048, 8)), None);
    }

    #[test]
    fn a_square_wave_with_sync_looking_edges_is_not_detected() {
        // L = R, +384 / -385, 100 frames per period: every -385 -> +384 edge
        // reads FE7F 0180 across R(n) and L(n+1), at an exact period.
        let mut v = Vec::new();
        for n in 0..SNIFF_FRAMES {
            let s: i16 = if n % 100 < 50 { 384 } else { -385 };
            v.extend([s, s]);
        }
        assert_eq!(detect(&v), None);
    }

    #[test]
    fn iec_61937_bursts_are_detected_in_both_byte_orders() {
        // Pc 1 (AC-3), Pd 24576 bits = 1536 words.
        for head in [
            [0xF872u16, 0x4E1F, 0x0001, 24576],
            [0x72F8, 0x1F4E, 0x0100, 24576u16.swap_bytes()],
        ] {
            assert_eq!(detect(&stream(&head, 3072, 6)), Some(Encoded::Iec61937));
        }
    }

    #[test]
    fn iec_61937_with_an_unknown_type_or_a_length_beyond_the_spacing_is_not_detected() {
        assert_eq!(
            detect(&stream(&[0xF872, 0x4E1F, 0x001F, 100], 3072, 6)),
            None
        );
        assert_eq!(
            detect(&stream(&[0xF872, 0x4E1F, 0x0001, 60000], 3072, 6)),
            None
        );
    }

    #[test]
    fn music_like_pcm_is_not_detected() {
        let sine: Vec<i16> = (0..SNIFF_FRAMES * 2)
            .map(|i| ((i as f32 * 0.05).sin() * 20_000.0) as i16)
            .collect();
        assert_eq!(detect(&sine), None);
        let mut x = 12345u32;
        let noise: Vec<i16> = (0..SNIFF_FRAMES * 2)
            .map(|_| {
                x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                (x >> 16) as i16
            })
            .collect();
        assert_eq!(detect(&noise), None);
        assert_eq!(detect(&vec![0; SNIFF_FRAMES * 2]), None);
        assert_eq!(detect(&[]), None);
        assert_eq!(detect(&[0x7FFE, -32767]), None);
    }

    #[test]
    fn sync_words_without_a_regular_spacing_are_not_detected() {
        // A single accidental sync word.
        let head = to_i16(&dts_words(false, false, 4096));
        let put = |v: &mut Vec<i16>, at: usize| v[at..at + 4].copy_from_slice(&head);
        let mut one = vec![5i16; 5000];
        put(&mut one, 100);
        assert_eq!(detect(&one), None);
        // Three syncs at irregular distances.
        let mut v = vec![5i16; 9000];
        for at in [100usize, 1000, 4000] {
            put(&mut v, at);
        }
        assert_eq!(detect(&v), None);
        // Two regular syncs are not enough.
        let mut v = vec![5i16; 9000];
        for at in [100usize, 2148] {
            put(&mut v, at);
        }
        assert_eq!(detect(&v), None);
    }
}
