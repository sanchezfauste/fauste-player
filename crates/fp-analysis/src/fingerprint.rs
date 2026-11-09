//! The audio fingerprint (external editor spec EE8): FNV-1a 64 over the bits
//! of every decoded sample, so a tag-only edit keeps it and an audio edit
//! changes it.

const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const PRIME: u64 = 0x0100_0000_01b3;

/// An incremental fingerprint of decoded samples.
pub(crate) struct Fingerprint(u64);

impl Fingerprint {
    pub(crate) fn new() -> Self {
        Self(OFFSET)
    }

    /// Feeds `samples` (in order, as the decoder delivers them).
    pub(crate) fn push(&mut self, samples: &[f32]) {
        for s in samples {
            for byte in s.to_bits().to_le_bytes() {
                self.0 = (self.0 ^ u64::from(byte)).wrapping_mul(PRIME);
            }
        }
    }

    pub(crate) fn finish(&self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_is_the_offset_basis() {
        assert_eq!(Fingerprint::new().finish(), 0xcbf2_9ce4_8422_2325);
    }

    #[test]
    fn matches_fnv1a_of_the_little_endian_bits() {
        // 0.0f32 is four zero bytes: four rounds of (h ^ 0) * prime.
        let mut expected = OFFSET;
        for _ in 0..4 {
            expected = expected.wrapping_mul(0x0100_0000_01b3);
        }
        let mut f = Fingerprint::new();
        f.push(&[0.0]);
        assert_eq!(f.finish(), expected);
    }

    #[test]
    fn pushing_in_pieces_equals_pushing_at_once() {
        let mut a = Fingerprint::new();
        a.push(&[0.5, -0.25, 1.0]);
        let mut b = Fingerprint::new();
        b.push(&[0.5]);
        b.push(&[-0.25, 1.0]);
        assert_eq!(a.finish(), b.finish());
    }
}
