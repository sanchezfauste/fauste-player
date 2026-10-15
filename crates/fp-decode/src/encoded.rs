//! Encoded bitstreams stored as PCM. A DTS-CD rip is a DTS stream written
//! into the samples of a 16-bit stereo PCM file (WAV, FLAC, AIFF): played as
//! PCM it is loud noise. This module recognises such a payload from the
//! first samples of a file so that it is refused instead. DTS and AC-3
//! decoding is out of scope.

/// Stereo frames examined when a file is opened: 128 KiB of 16-bit samples,
/// a few frames of a high-bitrate DTS stream.
pub(crate) const SNIFF_FRAMES: usize = 32_768;

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

/// A sync word as two 16-bit words, and the check of the word after them.
struct Sync {
    first: u16,
    second: u16,
    third: fn(u16) -> bool,
}

fn any(_: u16) -> bool {
    true
}

/// The four DTS core sync words (16-bit and 14-bit, big- and little-endian
/// streams). The set is closed under byte swapping, so a file whose samples
/// are big-endian (AIFF) matches the same way.
const DTS: [Sync; 4] = [
    Sync {
        first: 0x7FFE,
        second: 0x8001,
        third: any,
    },
    Sync {
        first: 0xFE7F,
        second: 0x0180,
        third: any,
    },
    Sync {
        first: 0x1FFF,
        second: 0xE800,
        third: |w| w & 0xFFF0 == 0x07F0,
    },
    Sync {
        first: 0xFF1F,
        second: 0x00E8,
        third: |w| w & 0xF0FF == 0xF007,
    },
];

/// IEC 61937 preamble Pa, Pb, in both byte orders.
const IEC: [Sync; 2] = [
    Sync {
        first: 0xF872,
        second: 0x4E1F,
        third: any,
    },
    Sync {
        first: 0x72F8,
        second: 0x1F4E,
        third: any,
    },
];

/// Whether `samples` (interleaved 16-bit words, in file order) carry an
/// encoded bitstream: at least three syncs of one family, the last three
/// of them at the same spacing. Plain PCM does not do this by chance: a
/// 32-bit pattern has to repeat at an exact period.
pub(crate) fn detect(samples: &[i16]) -> Option<Encoded> {
    if has_regular_syncs(samples, &DTS) {
        Some(Encoded::Dts)
    } else if has_regular_syncs(samples, &IEC) {
        Some(Encoded::Iec61937)
    } else {
        None
    }
}

fn has_regular_syncs(samples: &[i16], family: &[Sync]) -> bool {
    let mut at: Vec<usize> = Vec::new();
    for (i, w) in samples.windows(3).enumerate() {
        let (Some(&a), Some(&b), Some(&c)) = (w.first(), w.get(1), w.get(2)) else {
            continue;
        };
        let (a, b, c) = (a as u16, b as u16, c as u16);
        if family
            .iter()
            .any(|s| s.first == a && s.second == b && (s.third)(c))
        {
            at.push(i);
        }
    }
    if at.len() < MIN_SYNCS {
        return false;
    }
    at.windows(MIN_SYNCS)
        .any(|w| matches!(*w, [x, y, z] if y > x && y - x == z - y))
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::*;

    /// A payload of `frames` words with a sync every `period` words.
    fn stream(sync: &[u16; 3], period: usize, count: usize) -> Vec<i16> {
        let mut v = vec![0x1234i16; period * count + 8];
        for n in 0..count {
            for (k, w) in sync.iter().enumerate() {
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
        for (name, sync) in [
            ("16-bit BE", [0x7FFE, 0x8001, 0xFC00]),
            ("16-bit LE", [0xFE7F, 0x0180, 0x00FC]),
            ("14-bit BE", [0x1FFF, 0xE800, 0x07F1]),
            ("14-bit LE", [0xFF1F, 0x00E8, 0xF107]),
        ] {
            assert_eq!(
                detect(&stream(&sync, 2048, 8)),
                Some(Encoded::Dts),
                "{name}"
            );
        }
    }

    #[test]
    fn iec_61937_bursts_are_detected_in_both_byte_orders() {
        for sync in [[0xF872, 0x4E1F, 0x0001], [0x72F8, 0x1F4E, 0x0100]] {
            assert_eq!(detect(&stream(&sync, 3072, 6)), Some(Encoded::Iec61937));
        }
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
        let mut one = vec![5i16; 5000];
        one[100] = 0x7FFE;
        one[101] = 0x8001u16 as i16;
        assert_eq!(detect(&one), None);
        // Three syncs at irregular distances.
        let mut v = vec![5i16; 9000];
        for at in [100usize, 1000, 4000] {
            v[at] = 0x7FFE;
            v[at + 1] = 0x8001u16 as i16;
        }
        assert_eq!(detect(&v), None);
        // Two regular syncs are not enough.
        let mut v = vec![5i16; 9000];
        for at in [100usize, 2148] {
            v[at] = 0x7FFE;
            v[at + 1] = 0x8001u16 as i16;
        }
        assert_eq!(detect(&v), None);
    }
}
