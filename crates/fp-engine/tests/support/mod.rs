#![allow(
    dead_code,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Shared test helpers: WAV fixtures and synthetic sources.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use fp_engine::worker::{SampleSource, SourceOpener};

/// Writes a 16-bit WAV whose sample `i` (per channel) is `(i % 20_000) as i16`,
/// so any position can be identified from its value.
pub fn indexed_wav(dir: &Path, name: &str, rate: u32, channels: u16, frames: usize) -> PathBuf {
    let path = dir.join(name);
    let spec = hound::WavSpec {
        channels,
        sample_rate: rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut w = hound::WavWriter::create(&path, spec).unwrap();
    for i in 0..frames {
        for _ in 0..channels {
            w.write_sample((i % 20_000) as i16).unwrap();
        }
    }
    w.finalize().unwrap();
    path
}

/// The sample index encoded by `indexed_wav`, recovered from a decoded value.
pub fn index_of(sample: f32) -> i64 {
    (f64::from(sample) * 32_768.0).round() as i64
}

/// Emits `total` frames whose left sample is the frame number and right is its negation.
pub struct Counting {
    pub next: u64,
    pub total: u64,
    pub block: u64,
}

impl SampleSource for Counting {
    fn next_block(&mut self, out: &mut Vec<f32>) -> Result<bool, String> {
        if self.next >= self.total {
            return Ok(false);
        }
        let end = (self.next + self.block).min(self.total);
        for i in self.next..end {
            out.push(i as f32);
            out.push(-(i as f32));
        }
        self.next = end;
        Ok(true)
    }
}

/// Opener producing `Counting` sources of `total` frames. The path is ignored.
pub fn counting_opener(total: u64) -> SourceOpener {
    Arc::new(move |_path, from_secs, rate| {
        let start = (from_secs * f64::from(rate)).round() as u64;
        Ok(Box::new(Counting {
            next: start,
            total,
            block: 64,
        }) as Box<dyn SampleSource>)
    })
}

/// A source that panics on its first block.
pub struct Exploding;

impl SampleSource for Exploding {
    fn next_block(&mut self, _out: &mut Vec<f32>) -> Result<bool, String> {
        panic!("boom");
    }
}

/// Opener for paths named `track<N>`: sample `i` (from the start of the file)
/// of track N is `N * 100_000 + i` on both channels, and every track is
/// `frames` long. `broken<N>` plays like `track<N>` but ends with a read
/// error instead of a clean end. Any other path fails to open.
pub fn tagged_opener(frames: u64) -> SourceOpener {
    Arc::new(move |path, from_secs, rate| {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let (digits, fails) = match name.strip_prefix("track") {
            Some(d) => (Some(d), false),
            None => (name.strip_prefix("broken"), true),
        };
        let n: u64 = digits
            .and_then(|d| d.parse().ok())
            .ok_or_else(|| format!("cannot open {name}"))?;
        let start = (from_secs * f64::from(rate)).round() as u64;
        Ok(Box::new(Tagged {
            base: n * 100_000,
            next: start,
            total: frames,
            fails,
        }) as Box<dyn SampleSource>)
    })
}

pub struct Tagged {
    base: u64,
    next: u64,
    total: u64,
    fails: bool,
}

impl SampleSource for Tagged {
    fn next_block(&mut self, out: &mut Vec<f32>) -> Result<bool, String> {
        if self.next >= self.total {
            return if self.fails {
                Err("read error".to_owned())
            } else {
                Ok(false)
            };
        }
        let end = (self.next + 512).min(self.total);
        for i in self.next..end {
            let v = (self.base + i) as f32;
            out.push(v);
            out.push(v);
        }
        self.next = end;
        Ok(true)
    }
}

/// Like `tagged_opener`, but opening a path named `slow<N>` (played as
/// `track<N>`) waits until `gate` is set: a preload that is not ready yet.
pub fn gated_opener(frames: u64, gate: Arc<std::sync::atomic::AtomicBool>) -> SourceOpener {
    let tagged = tagged_opener(frames);
    Arc::new(move |path, from_secs, rate| {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        match name.strip_prefix("slow") {
            Some(n) => {
                while !gate.load(std::sync::atomic::Ordering::Acquire) {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                tagged(&PathBuf::from(format!("track{n}")), from_secs, rate)
            }
            None => tagged(path, from_secs, rate),
        }
    })
}

pub const DSD64: u32 = 2_822_400;

/// A stereo DSF file at DSD64 whose channels hold `left` and `right`
/// (bytes most significant bit first in time, as the engine carries them).
pub fn dsf_file(dir: &Path, name: &str, left: &[u8], right: &[u8]) -> PathBuf {
    const BLOCK: usize = 4096;
    let len = left.len().min(right.len());
    let blocks = len.div_ceil(BLOCK);
    let mut data = Vec::new();
    for g in 0..blocks {
        for ch in [left, right] {
            let mut block = vec![0x69u8.reverse_bits(); BLOCK];
            for (i, byte) in ch
                .iter()
                .skip(g * BLOCK)
                .take(BLOCK.min(len - g * BLOCK))
                .enumerate()
            {
                block[i] = byte.reverse_bits();
            }
            data.extend(block);
        }
    }
    let mut f = Vec::new();
    let total = 28 + 52 + 12 + data.len() as u64;
    f.extend(b"DSD ");
    f.extend(28u64.to_le_bytes());
    f.extend(total.to_le_bytes());
    f.extend(0u64.to_le_bytes());
    f.extend(b"fmt ");
    f.extend(52u64.to_le_bytes());
    f.extend(1u32.to_le_bytes()); // version
    f.extend(0u32.to_le_bytes()); // DSD raw
    f.extend(2u32.to_le_bytes()); // stereo
    f.extend(2u32.to_le_bytes()); // channels
    f.extend(DSD64.to_le_bytes());
    f.extend(1u32.to_le_bytes()); // LSB first
    f.extend((len as u64 * 8).to_le_bytes()); // samples per channel
    f.extend((BLOCK as u32).to_le_bytes());
    f.extend(0u32.to_le_bytes());
    f.extend(b"data");
    f.extend((12 + data.len() as u64).to_le_bytes());
    f.extend(data);
    let path = dir.join(name);
    std::fs::write(&path, f).unwrap();
    path
}
