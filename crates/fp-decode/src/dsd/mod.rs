//! DSD files (DSF and uncompressed DSDIFF), converted to PCM at the DSD rate
//! divided by 32 (audio formats spec F3).
//!
//! Output frame `k` is the filter centred on DSD byte `4k`, so its window
//! covers bytes `4k - 50 .. 4k + 50`. Bytes outside the stream read as the
//! idle pattern. Seeking re-reads from the start of the window, so a seek
//! produces exactly the frames a decode from the start would.

mod convert;
mod dff;
mod dsf;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use convert::{BYTES_PER_FRAME, DECIMATION, IDLE, WINDOW_BYTES};

/// Most channels in a DSD file (5.1).
const MAX_CHANNELS: usize = 6;
/// DSD rates accepted: DSD64 (2.8224 MHz) to DSD512 and the 48 kHz family.
const MIN_RATE: u32 = 2_822_400;
const MAX_RATE: u32 = 24_576_000;
/// Bytes per channel read at a time.
const CHUNK_BYTES: usize = 4096;

/// Where a DSD file's audio is and how it is laid out.
#[derive(Debug, Clone)]
pub(crate) struct Layout {
    channels: usize,
    rate: u32,
    /// DSD samples per channel.
    samples: u64,
    /// Bytes per channel that hold audio.
    bytes: u64,
    data_start: u64,
    /// DSF: bytes per channel in each block. DSDIFF: `None` (interleaved).
    block: Option<u64>,
    lsb_first: bool,
    /// Where each channel goes in the frame `downmix` reads for more than
    /// two channels: L, R, C, LFE, Ls, Rs (`SLOTS`). `None` drops it.
    positions: Vec<Option<usize>>,
}

/// Frame size for more than two channels: L, R, C, LFE, Ls, Rs.
const SLOTS: usize = 6;
pub(crate) const L: usize = 0;
pub(crate) const R: usize = 1;
pub(crate) const C: usize = 2;
pub(crate) const LFE: usize = 3;
pub(crate) const LS: usize = 4;
pub(crate) const RS: usize = 5;

/// The positions of `channels` channels stored in the usual order for their
/// count, when the file says nothing more precise.
pub(crate) fn default_positions(channels: usize) -> Vec<Option<usize>> {
    let order: &[usize] = match channels {
        3 => &[L, R, C],
        4 => &[L, R, C, LFE],
        5 => &[L, R, C, LS, RS],
        _ => &[L, R, C, LFE, LS, RS],
    };
    (0..channels).map(|c| order.get(c).copied()).collect()
}

impl Layout {
    fn validate(&self) -> Result<(), String> {
        if self.channels == 0 || self.channels > MAX_CHANNELS {
            return Err(format!("{} channels is not a DSD layout", self.channels));
        }
        if !(MIN_RATE..=MAX_RATE).contains(&self.rate)
            || !self.rate.is_multiple_of(DECIMATION as u32)
        {
            return Err(format!("{} Hz is not a DSD rate", self.rate));
        }
        if self.bytes == 0 {
            return Err("no DSD audio in the file".into());
        }
        Ok(())
    }

    fn frames(&self) -> u64 {
        self.samples.min(self.bytes * 8).div_ceil(DECIMATION)
    }
}

fn read_at(file: &mut File, offset: u64, buf: &mut [u8]) -> Result<(), String> {
    file.seek(SeekFrom::Start(offset))
        .and_then(|_| file.read_exact(buf))
        .map_err(|e| format!("DSD read: {e}"))
}

fn le_u32(b: &[u8], at: usize) -> Option<u32> {
    b.get(at..at + 4)?.try_into().ok().map(u32::from_le_bytes)
}

fn le_u64(b: &[u8], at: usize) -> Option<u64> {
    b.get(at..at + 8)?.try_into().ok().map(u64::from_le_bytes)
}

fn be_u16(b: &[u8], at: usize) -> Option<u16> {
    b.get(at..at + 2)?.try_into().ok().map(u16::from_be_bytes)
}

fn be_u32(b: &[u8], at: usize) -> Option<u32> {
    b.get(at..at + 4)?.try_into().ok().map(u32::from_be_bytes)
}

fn be_u64(b: &[u8], at: usize) -> Option<u64> {
    b.get(at..at + 8)?.try_into().ok().map(u64::from_be_bytes)
}

pub(crate) struct DsdDecoder {
    file: File,
    layout: Layout,
    /// Per channel, the bytes read so far from byte `base`.
    bytes: Vec<Vec<u8>>,
    base: u64,
    /// The next byte (per channel) to read from the file.
    read_pos: u64,
    /// The next output frame.
    next: u64,
    raw: Vec<u8>,
    frame: Vec<f32>,
}

impl DsdDecoder {
    pub(crate) fn open_dsf(path: &Path) -> Result<Self, String> {
        let mut file = File::open(path).map_err(|e| e.to_string())?;
        let layout = dsf::layout(&mut file)?;
        Self::new(file, layout)
    }

    pub(crate) fn open_dff(path: &Path) -> Result<Self, String> {
        let mut file = File::open(path).map_err(|e| e.to_string())?;
        let layout = dff::layout(&mut file)?;
        Self::new(file, layout)
    }

    fn new(file: File, layout: Layout) -> Result<Self, String> {
        layout.validate()?;
        Ok(Self {
            file,
            bytes: vec![Vec::new(); layout.channels],
            base: 0,
            read_pos: 0,
            next: 0,
            raw: Vec::new(),
            frame: vec![
                0.0;
                if layout.channels > 2 {
                    SLOTS
                } else {
                    layout.channels
                }
            ],
            layout,
        })
    }

    pub(crate) fn sample_rate(&self) -> u32 {
        self.layout.rate / DECIMATION as u32
    }

    pub(crate) fn channels(&self) -> usize {
        self.layout.channels
    }

    pub(crate) fn frames_hint(&self) -> Option<u64> {
        Some(self.layout.frames())
    }

    pub(crate) fn seek(&mut self, secs: f64) -> Result<(), String> {
        let target = (secs.max(0.0) * f64::from(self.sample_rate())).round();
        // Saturating float-to-int conversion; clamped to the end.
        let target = (target as u64).min(self.layout.frames());
        let window_start = (target * BYTES_PER_FRAME).saturating_sub(WINDOW_BYTES / 2);
        let start = match self.layout.block {
            Some(block) => window_start - window_start % block,
            None => window_start,
        };
        self.next = target;
        self.base = start;
        self.read_pos = start;
        for ch in &mut self.bytes {
            ch.clear();
        }
        Ok(())
    }

    /// Reads the next chunk of every channel. `false` at the end of the audio.
    /// DSF blocks are read several at a time (about `CHUNK_BYTES` per
    /// channel), so a file with tiny blocks costs no more reads than one
    /// with the usual 4096-byte blocks.
    fn read_chunk(&mut self) -> Result<bool, String> {
        let layout = &self.layout;
        if self.read_pos >= layout.bytes {
            return Ok(false);
        }
        let channels = layout.channels as u64;
        let remaining = layout.bytes - self.read_pos;
        // (file offset, bytes per channel in each group, groups).
        let (offset, per_group, groups) = match layout.block {
            Some(block) => {
                let block = block.max(1);
                let groups = (CHUNK_BYTES as u64 / block)
                    .max(1)
                    .min(remaining.div_ceil(block));
                let group = self.read_pos / block;
                (layout.data_start + group * block * channels, block, groups)
            }
            None => {
                let len = remaining.min(CHUNK_BYTES as u64);
                (layout.data_start + self.read_pos * channels, len, 1)
            }
        };
        let per_group = usize::try_from(per_group).map_err(|e| e.to_string())?;
        let groups = usize::try_from(groups).map_err(|e| e.to_string())?;
        self.raw.resize(per_group * layout.channels * groups, 0);
        read_at(&mut self.file, offset, &mut self.raw)?;
        let reverse = layout.lsb_first;
        let mut read = 0u64;
        for g in 0..groups {
            let valid = (per_group as u64).min(remaining - read) as usize;
            let group = self
                .raw
                .get(g * per_group * layout.channels..(g + 1) * per_group * layout.channels)
                .unwrap_or_default();
            for (c, out) in self.bytes.iter_mut().enumerate() {
                let bytes: &mut dyn Iterator<Item = &u8> = match layout.block {
                    // DSF: one block per channel, one after the other.
                    Some(_) => &mut group
                        .get(c * per_group..c * per_group + valid)
                        .unwrap_or_default()
                        .iter(),
                    // DSDIFF: bytes interleaved by channel.
                    None => &mut group.iter().skip(c).step_by(layout.channels).take(valid),
                };
                if reverse {
                    out.extend(bytes.map(|b| b.reverse_bits()));
                } else {
                    out.extend(bytes);
                }
            }
            read += valid as u64;
        }
        self.read_pos += read;
        Ok(true)
    }

    /// The whole window of channel `c` starting at byte `start`, when it lies
    /// inside the stream and has been read (the usual case).
    fn window(&self, c: usize, start: i64) -> Option<&[u8]> {
        let start = u64::try_from(start).ok()?;
        if start + WINDOW_BYTES > self.layout.bytes {
            return None;
        }
        let at = usize::try_from(start.checked_sub(self.base)?).ok()?;
        self.bytes.get(c)?.get(at..at + WINDOW_BYTES as usize)
    }

    /// The byte at absolute index `i` of channel `c`, if it has been read.
    fn byte(&self, c: usize, i: i64) -> Option<u8> {
        if i < 0 || i as u64 >= self.layout.bytes {
            return Some(IDLE);
        }
        let at = (i as u64).checked_sub(self.base)?;
        self.bytes.get(c)?.get(at as usize).copied()
    }

    pub(crate) fn next_block(&mut self, out: &mut Vec<f32>) -> Result<bool, String> {
        let frames = self.layout.frames();
        loop {
            if self.next >= frames {
                return Ok(false);
            }
            let more = self.read_chunk()?;
            let available = self.read_pos.min(self.layout.bytes);
            let mut produced = false;
            while self.next < frames {
                let start = (self.next * BYTES_PER_FRAME) as i64 - (WINDOW_BYTES / 2) as i64;
                let end = start + WINDOW_BYTES as i64;
                // Until the file is exhausted, wait for the whole window.
                if more && end as u64 > available {
                    break;
                }
                let surround = self.layout.channels > 2;
                self.frame.fill(0.0);
                for c in 0..self.layout.channels {
                    let sample = match self.window(c, start) {
                        Some(window) => convert::filter(window.iter().copied()),
                        // At the edges of the stream: idle bytes outside it.
                        None => {
                            convert::filter((start..end).map(|i| self.byte(c, i).unwrap_or(IDLE)))
                        }
                    };
                    let at = if surround {
                        self.layout.positions.get(c).copied().flatten()
                    } else {
                        Some(c)
                    };
                    if let Some(slot) = at.and_then(|at| self.frame.get_mut(at)) {
                        *slot += sample;
                    }
                }
                let (l, r) = crate::downmix(&self.frame);
                out.push(l);
                out.push(r);
                self.next += 1;
                produced = true;
            }
            self.trim();
            if produced {
                return Ok(true);
            }
            if !more {
                return Ok(self.next < frames);
            }
        }
    }

    /// Drops the bytes no future window needs.
    fn trim(&mut self) {
        let keep_from = (self.next * BYTES_PER_FRAME).saturating_sub(WINDOW_BYTES / 2);
        if keep_from > self.base {
            let drop = ((keep_from - self.base) as usize)
                .min(self.bytes.first().map(Vec::len).unwrap_or(0));
            for ch in &mut self.bytes {
                ch.drain(..drop.min(ch.len()));
            }
            self.base += drop as u64;
        }
    }
}
