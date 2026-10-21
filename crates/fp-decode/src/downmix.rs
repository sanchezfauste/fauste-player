//! Stereo from any channel layout (ITU-R BS.775 coefficients).
//!
//! The layout comes from the channel positions the container reports
//! (symphonia's `Channels`); when it reports none, from the channel count,
//! assuming the usual WAV/SMPTE order for it. Front left/right keep their
//! side; the centre and the back centre go to both sides at -3 dB; surround,
//! side and rear channels go to their side at -3 dB; the LFE is dropped. A
//! layout with any surround channel is scaled so that its loudest side
//! cannot clip (front-only layouts are not: 3.0 and 3.1 sum above full
//! scale by design).

use symphonia::core::audio::{ChannelLabel, Channels, Position};

use crate::MINUS_3DB;

/// Most channels with a position of their own; the rest are dropped.
const MAX: usize = 32;

/// Per-channel weights `(left, right)` to the stereo pair.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Downmix {
    gains: [(f32, f32); MAX],
    len: usize,
}

/// The weights of one channel position, and whether it is a surround
/// channel (one that calls for normalisation).
fn weights(pos: Position) -> Option<((f32, f32), bool)> {
    let m = MINUS_3DB;
    Some(match pos {
        Position::FRONT_LEFT
        | Position::FRONT_LEFT_CENTER
        | Position::FRONT_LEFT_WIDE
        | Position::BOTTOM_FRONT_LEFT => ((1.0, 0.0), false),
        Position::FRONT_RIGHT
        | Position::FRONT_RIGHT_CENTER
        | Position::FRONT_RIGHT_WIDE
        | Position::BOTTOM_FRONT_RIGHT => ((0.0, 1.0), false),
        Position::FRONT_CENTER | Position::BOTTOM_FRONT_CENTER => ((m, m), false),
        Position::REAR_LEFT
        | Position::SIDE_LEFT
        | Position::TOP_FRONT_LEFT
        | Position::TOP_REAR_LEFT
        | Position::TOP_SIDE_LEFT => ((m, 0.0), true),
        Position::REAR_RIGHT
        | Position::SIDE_RIGHT
        | Position::TOP_FRONT_RIGHT
        | Position::TOP_REAR_RIGHT
        | Position::TOP_SIDE_RIGHT => ((0.0, m), true),
        Position::REAR_CENTER
        | Position::TOP_CENTER
        | Position::TOP_FRONT_CENTER
        | Position::TOP_REAR_CENTER => ((m, m), true),
        // LFE1, LFE2 and anything unknown.
        _ => return None,
    })
}

impl Downmix {
    /// The mix for channels at `positions`, in the order of the frame.
    fn from_positions(positions: &[Position]) -> Self {
        let mut mix = Downmix {
            gains: [(0.0, 0.0); MAX],
            len: positions.len().min(MAX),
        };
        if positions.len() == 1 {
            // Mono, whatever it is called: duplicated.
            mix.gains[0] = (1.0, 1.0);
            return mix;
        }
        let (mut left, mut right, mut surround) = (0.0f32, 0.0f32, false);
        for (slot, pos) in mix.gains.iter_mut().zip(positions) {
            if let Some(((l, r), s)) = weights(*pos) {
                *slot = (l, r);
                left += l;
                right += r;
                surround |= s;
            }
        }
        if surround {
            let norm = 1.0 / left.max(right);
            for (l, r) in mix.gains.iter_mut().take(mix.len) {
                *l *= norm;
                *r *= norm;
            }
        }
        mix
    }

    /// The mix for `count` channels in the usual order for their number
    /// (WAV default masks; beyond eight, the first eight).
    pub(crate) fn for_count(count: usize) -> Self {
        use Position as P;
        let order: &[Position] = match count {
            0 => &[],
            1 => &[P::FRONT_CENTER],
            2 => &[P::FRONT_LEFT, P::FRONT_RIGHT],
            3 => &[P::FRONT_LEFT, P::FRONT_RIGHT, P::FRONT_CENTER],
            4 => &[P::FRONT_LEFT, P::FRONT_RIGHT, P::REAR_LEFT, P::REAR_RIGHT],
            5 => &[
                P::FRONT_LEFT,
                P::FRONT_RIGHT,
                P::FRONT_CENTER,
                P::REAR_LEFT,
                P::REAR_RIGHT,
            ],
            6 => &[
                P::FRONT_LEFT,
                P::FRONT_RIGHT,
                P::FRONT_CENTER,
                P::LFE1,
                P::REAR_LEFT,
                P::REAR_RIGHT,
            ],
            7 => &[
                P::FRONT_LEFT,
                P::FRONT_RIGHT,
                P::FRONT_CENTER,
                P::LFE1,
                P::REAR_CENTER,
                P::SIDE_LEFT,
                P::SIDE_RIGHT,
            ],
            _ => &[
                P::FRONT_LEFT,
                P::FRONT_RIGHT,
                P::FRONT_CENTER,
                P::LFE1,
                P::REAR_LEFT,
                P::REAR_RIGHT,
                P::SIDE_LEFT,
                P::SIDE_RIGHT,
            ],
        };
        let mut mix = Self::from_positions(order);
        // Channels past the table are dropped (zero weight).
        mix.len = count.min(MAX);
        mix
    }

    /// The mix for a decoded buffer's channels; `count` is the number of
    /// channels in its frames, used when the positions are not known or do
    /// not match.
    pub(crate) fn for_channels(channels: &Channels, count: usize) -> Self {
        let positions: Option<Vec<Position>> = match channels {
            // The decoded order is the order of the position bits.
            Channels::Positioned(set) => Some(set.iter().collect()),
            Channels::Custom(labels) => labels
                .iter()
                .map(|l| match l {
                    ChannelLabel::Positioned(p) => Some(*p),
                    _ => None,
                })
                .collect(),
            _ => None,
        };
        match positions {
            Some(p) if p.len() == count && count <= MAX => Self::from_positions(&p),
            _ => Self::for_count(count),
        }
    }

    /// One frame to stereo. Channels beyond the mix are ignored.
    pub(crate) fn apply(&self, frame: &[f32]) -> (f32, f32) {
        let (mut l, mut r) = (0.0, 0.0);
        for ((gl, gr), x) in self.gains.iter().take(self.len).zip(frame) {
            l += gl * x;
            r += gr * x;
        }
        (l, r)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::indexing_slicing)]

    use symphonia::core::audio::{ChannelLabel, Channels, Position as P};

    use super::Downmix;
    use crate::MINUS_3DB;

    /// The stereo of one channel at full scale.
    fn only(mix: &Downmix, channels: usize, at: usize) -> (f32, f32) {
        let mut frame = vec![0.0; channels];
        frame[at] = 1.0;
        mix.apply(&frame)
    }

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-6
    }

    fn positioned(list: &[P]) -> Downmix {
        let mask = list.iter().fold(P::empty(), |a, p| a | *p);
        Downmix::for_channels(&Channels::Positioned(mask), list.len())
    }

    #[test]
    fn five_one_keeps_its_coefficients() {
        let norm = 1.0 / (1.0 + 2.0 * MINUS_3DB);
        for mix in [
            Downmix::for_count(6),
            positioned(&[
                P::FRONT_LEFT,
                P::FRONT_RIGHT,
                P::FRONT_CENTER,
                P::LFE1,
                P::REAR_LEFT,
                P::REAR_RIGHT,
            ]),
            // The side variant of 5.1 weighs the same.
            positioned(&[
                P::FRONT_LEFT,
                P::FRONT_RIGHT,
                P::FRONT_CENTER,
                P::LFE1,
                P::SIDE_LEFT,
                P::SIDE_RIGHT,
            ]),
        ] {
            let (l, r) = only(&mix, 6, 0);
            assert!(near(l, norm) && r == 0.0);
            let (l, r) = only(&mix, 6, 2);
            assert!(near(l, MINUS_3DB * norm) && near(r, l));
            assert_eq!(only(&mix, 6, 3), (0.0, 0.0), "LFE");
            let (l, r) = only(&mix, 6, 4);
            assert!(near(l, MINUS_3DB * norm) && r == 0.0);
            let (l, r) = only(&mix, 6, 5);
            assert!(l == 0.0 && near(r, MINUS_3DB * norm));
        }
    }

    #[test]
    fn seven_one_counts_its_side_channels() {
        let mix = Downmix::for_count(8);
        let norm = 1.0 / (1.0 + 3.0 * MINUS_3DB);
        let (l, r) = only(&mix, 8, 6);
        assert!(near(l, MINUS_3DB * norm) && r == 0.0, "SL {l} {r}");
        let (l, r) = only(&mix, 8, 7);
        assert!(l == 0.0 && near(r, MINUS_3DB * norm), "SR {l} {r}");
        let (l, r) = only(&mix, 8, 4);
        assert!(near(l, MINUS_3DB * norm) && r == 0.0, "BL {l} {r}");
        assert_eq!(only(&mix, 8, 3), (0.0, 0.0));
        // Everything at full scale in phase cannot clip.
        let (l, r) = mix.apply(&[1.0; 8]);
        assert!(l <= 1.0 + 1e-6 && r <= 1.0 + 1e-6, "{l} {r}");
    }

    #[test]
    fn six_one_puts_the_back_centre_on_both_sides() {
        let mix = positioned(&[
            P::FRONT_LEFT,
            P::FRONT_RIGHT,
            P::FRONT_CENTER,
            P::LFE1,
            P::REAR_CENTER,
            P::SIDE_LEFT,
            P::SIDE_RIGHT,
        ]);
        let norm = 1.0 / (1.0 + 3.0 * MINUS_3DB);
        let (l, r) = only(&mix, 7, 4);
        assert!(near(l, MINUS_3DB * norm) && near(r, l), "{l} {r}");
        let (l, r) = only(&mix, 7, 5);
        assert!(near(l, MINUS_3DB * norm) && r == 0.0);
        // By count, the seven channels are the same layout.
        let (l, r) = only(&Downmix::for_count(7), 7, 4);
        assert!(near(l, MINUS_3DB * norm) && near(r, l));
    }

    #[test]
    fn quad_has_no_centre_or_lfe() {
        for mix in [
            Downmix::for_count(4),
            positioned(&[P::FRONT_LEFT, P::FRONT_RIGHT, P::REAR_LEFT, P::REAR_RIGHT]),
        ] {
            let norm = 1.0 / (1.0 + MINUS_3DB);
            let (l, r) = only(&mix, 4, 2);
            assert!(near(l, MINUS_3DB * norm) && r == 0.0, "RL {l} {r}");
            let (l, r) = only(&mix, 4, 3);
            assert!(l == 0.0 && near(r, MINUS_3DB * norm), "RR {l} {r}");
            let (l, r) = only(&mix, 4, 0);
            assert!(near(l, norm) && r == 0.0);
        }
    }

    #[test]
    fn two_one_drops_the_lfe_and_three_zero_keeps_the_centre() {
        let mix = positioned(&[P::FRONT_LEFT, P::FRONT_RIGHT, P::LFE1]);
        assert_eq!(only(&mix, 3, 2), (0.0, 0.0));
        assert_eq!(only(&mix, 3, 0), (1.0, 0.0));
        let mix = positioned(&[P::FRONT_LEFT, P::FRONT_RIGHT, P::FRONT_CENTER]);
        let (l, r) = only(&mix, 3, 2);
        assert!(near(l, MINUS_3DB) && near(r, l));
        // 3.1 by positions: the LFE is dropped, nothing is scaled.
        let mix = positioned(&[P::FRONT_LEFT, P::FRONT_RIGHT, P::FRONT_CENTER, P::LFE1]);
        assert_eq!(only(&mix, 4, 3), (0.0, 0.0));
        assert_eq!(only(&mix, 4, 0), (1.0, 0.0));
    }

    #[test]
    fn mono_and_stereo_pass_through() {
        assert_eq!(Downmix::for_count(1).apply(&[0.5]), (0.5, 0.5));
        assert_eq!(Downmix::for_count(2).apply(&[0.25, -0.5]), (0.25, -0.5));
        let mono = Downmix::for_channels(&Channels::Positioned(P::FRONT_CENTER), 1);
        assert_eq!(mono.apply(&[0.5]), (0.5, 0.5));
        assert_eq!(Downmix::for_count(0).apply(&[]), (0.0, 0.0));
    }

    #[test]
    fn unknown_layouts_fall_back_to_the_channel_count() {
        let by_count = Downmix::for_count(8);
        let discrete = Downmix::for_channels(&Channels::Discrete(8), 8);
        let none = Downmix::for_channels(&Channels::None, 8);
        let frame = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];
        assert_eq!(by_count.apply(&frame), discrete.apply(&frame));
        assert_eq!(by_count.apply(&frame), none.apply(&frame));
        // Positions that do not match the frame are not trusted.
        let wrong = Downmix::for_channels(&Channels::Positioned(P::FRONT_LEFT | P::FRONT_RIGHT), 6);
        assert_eq!(wrong.apply(&frame), Downmix::for_count(6).apply(&frame));
    }

    #[test]
    fn custom_labels_are_read_in_frame_order() {
        // R then L: the order of the labels, not of the bits.
        let labels = vec![
            ChannelLabel::Positioned(P::FRONT_RIGHT),
            ChannelLabel::Positioned(P::FRONT_LEFT),
        ];
        let mix = Downmix::for_channels(&Channels::from(labels), 2);
        assert_eq!(mix.apply(&[1.0, 0.0]), (0.0, 1.0));
    }

    #[test]
    fn more_than_eight_channels_keep_the_first_eight() {
        let mix = Downmix::for_count(10);
        assert_eq!(only(&mix, 10, 8), (0.0, 0.0));
        assert_eq!(only(&mix, 10, 9), (0.0, 0.0));
        assert!(only(&mix, 10, 6).0 > 0.0);
    }
}
