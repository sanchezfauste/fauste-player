//! WavPack (audio formats spec F2), decoded with wavicle one block at a
//! time: every WavPack block carries its own starting state, so each is
//! handed to the decoder on its own. Opening indexes the block headers, and
//! seeking starts at the block that holds the target frame.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use wavicle::BlockHeader;
use wavicle::block::HEADER_LEN;

struct BlockRef {
    offset: u64,
    len: usize,
    first: u64,
    frames: u64,
}

pub(crate) struct WavPackDecoder {
    file: File,
    blocks: Vec<BlockRef>,
    next: usize,
    skip: u64,
    rate: u32,
    channels: usize,
    bits: u32,
    float: bool,
    buf: Vec<u8>,
}

fn read_at(file: &mut File, offset: u64, buf: &mut [u8]) -> std::io::Result<()> {
    file.seek(SeekFrom::Start(offset))?;
    file.read_exact(buf)
}

/// The headers of the audio blocks, up to the end of the file or the first
/// thing that is not a block (a trailing tag, or damage).
fn index(file: &mut File, start: u64) -> Result<Vec<BlockRef>, String> {
    let file_len = file.metadata().map_err(|e| e.to_string())?.len();
    let mut blocks = Vec::new();
    let mut offset = start;
    let mut header = [0u8; HEADER_LEN];
    while offset + HEADER_LEN as u64 <= file_len {
        if read_at(file, offset, &mut header).is_err() {
            break;
        }
        // wavicle adds to the size field unchecked; refuse a lying size first.
        let size = header
            .get(4..8)
            .and_then(|b| b.try_into().ok())
            .map(u32::from_le_bytes)
            .unwrap_or(u32::MAX);
        let parsed = if size > wavicle::format::MAX_BLOCK_SIZE {
            Err(format!("block size {size} is too large"))
        } else {
            BlockHeader::parse(&header).map_err(|e| e.to_string())
        };
        let h = match parsed {
            Ok(h) => h,
            Err(e) if offset == start => return Err(format!("WavPack: {e}")),
            Err(_) => break,
        };
        let len = h.block_len();
        if offset + len as u64 > file_len {
            if offset == start {
                return Err("WavPack: the first block is truncated".into());
            }
            break;
        }
        if h.block_samples > 0 {
            if !(h.flags.initial_block() && h.flags.final_block()) {
                return Err("WavPack with more than two channels is not supported".into());
            }
            blocks.push(BlockRef {
                offset,
                len,
                first: h.block_index,
                frames: u64::from(h.block_samples),
            });
        }
        offset += len as u64;
    }
    if blocks.is_empty() {
        return Err("WavPack: no audio blocks".into());
    }
    Ok(blocks)
}

impl WavPackDecoder {
    /// Opens the stream that starts `start` bytes into the file (after a
    /// leading tag).
    pub(crate) fn open(path: &Path, start: u64) -> Result<Self, String> {
        let mut file = File::open(path).map_err(|e| e.to_string())?;
        let blocks = index(&mut file, start)?;
        let mut decoder = Self {
            file,
            blocks,
            next: 0,
            skip: 0,
            rate: 0,
            channels: 0,
            bits: 0,
            float: false,
            buf: Vec::new(),
        };
        let first = decoder.decode(0)?;
        decoder.rate = first.sample_rate;
        decoder.channels = usize::try_from(first.channels).map_err(|e| e.to_string())?;
        decoder.bits = first.bits_per_sample;
        decoder.float = first.is_float;
        if decoder.rate == 0 || !(1..=2).contains(&decoder.channels) {
            return Err("WavPack: bad stream parameters".into());
        }
        Ok(decoder)
    }

    fn decode(&mut self, block: usize) -> Result<wavicle::DecodedStream, String> {
        let b = self.blocks.get(block).ok_or("WavPack: no such block")?;
        self.buf.resize(b.len, 0);
        read_at(&mut self.file, b.offset, &mut self.buf).map_err(|e| format!("WavPack: {e}"))?;
        wavicle::decode_stream(&self.buf).map_err(|e| format!("WavPack: {e}"))
    }

    pub(crate) fn sample_rate(&self) -> u32 {
        self.rate
    }

    pub(crate) fn bits_per_sample(&self) -> Option<u32> {
        (!self.float).then_some(self.bits)
    }

    pub(crate) fn channels(&self) -> usize {
        self.channels
    }

    pub(crate) fn frames_hint(&self) -> Option<u64> {
        Some(self.blocks.iter().map(|b| b.frames).sum())
    }

    pub(crate) fn seek(&mut self, secs: f64) -> Result<(), String> {
        // Frames count from the first block, which need not start at 0 (a
        // file cut out of a longer stream). Saturating float-to-int.
        let origin = self.blocks.first().map_or(0, |b| b.first);
        let target = origin.saturating_add((secs.max(0.0) * f64::from(self.rate)).round() as u64);
        let at = self
            .blocks
            .partition_point(|b| b.first + b.frames <= target);
        self.next = at;
        self.skip = self
            .blocks
            .get(at)
            .map(|b| target.saturating_sub(b.first))
            .unwrap_or(0);
        Ok(())
    }

    pub(crate) fn next_block(&mut self, out: &mut Vec<f32>) -> Result<bool, String> {
        if self.next >= self.blocks.len() {
            return Ok(false);
        }
        let decoded = self.decode(self.next)?;
        self.next += 1;
        let channels = usize::try_from(decoded.channels).map_err(|e| e.to_string())?;
        if channels != self.channels {
            return Err("WavPack: the channel count changes mid-stream".into());
        }
        let scale = 1.0 / (1u64 << self.bits.clamp(1, 32).saturating_sub(1)) as f32;
        let skip = usize::try_from(std::mem::take(&mut self.skip)).unwrap_or(usize::MAX);
        let mut frame = [0f32; 2];
        for samples in decoded.samples.chunks_exact(channels).skip(skip) {
            for (slot, &s) in frame.iter_mut().zip(samples) {
                *slot = if self.float {
                    f32::from_bits(s as u32)
                } else {
                    s as f32 * scale
                };
            }
            let (l, r) = crate::downmix(frame.get(..channels).unwrap_or_default());
            out.push(l);
            out.push(r);
        }
        Ok(true)
    }
}
