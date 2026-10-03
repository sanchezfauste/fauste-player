//! Raw DSD bytes for output without conversion (feedback 2 spec O25).

use std::fs::File;
use std::io::Read;
use std::path::Path;

use super::{ChunkReader, open_reader};
use crate::Kind;

/// Reads a DSF or uncompressed DSDIFF file as per-channel DSD bytes.
pub struct DsdRawReader {
    reader: ChunkReader,
    /// Bytes per channel still to skip at the front of the next chunk (a
    /// seek inside a DSF block).
    skip: usize,
    /// Each output's length before a read (reused).
    lens: Vec<usize>,
}

impl DsdRawReader {
    /// Opens a DSF or uncompressed DSDIFF file for its raw DSD bytes.
    pub fn open(path: &Path) -> Result<Self, String> {
        let mut head = [0u8; 16];
        let mut file = File::open(path).map_err(|e| e.to_string())?;
        let mut filled = 0;
        while let Some(rest) = head.get_mut(filled..).filter(|r| !r.is_empty()) {
            match file.read(rest).map_err(|e| e.to_string())? {
                0 => break,
                n => filled += n,
            }
        }
        let dff = match crate::probe(head.get(..filled).unwrap_or_default(), None) {
            Kind::Dsf => false,
            Kind::Dff => true,
            _ => return Err("not a DSD file".to_owned()),
        };
        Ok(Self {
            reader: open_reader(path, dff)?,
            skip: 0,
            lens: Vec::new(),
        })
    }

    /// The DSD sample rate in Hz.
    pub fn dsd_rate(&self) -> u32 {
        self.reader.layout.rate
    }

    pub fn channels(&self) -> usize {
        self.reader.layout.channels
    }

    /// Bytes per channel of audio in the file.
    pub fn len_bytes(&self) -> u64 {
        self.reader.layout.bytes
    }

    /// Positions at byte round(secs x rate / 8) of each channel, rounded down
    /// to an even byte (one 16-bit word), clamped to the end.
    pub fn seek(&mut self, secs: f64) -> Result<(), String> {
        let rate = f64::from(self.dsd_rate());
        // Saturating float-to-int conversion (NaN is 0).
        let byte = (secs.max(0.0) * rate / 8.0).round() as u64;
        let byte = (byte - byte % 2).min(self.len_bytes());
        let start = self.reader.aligned(byte);
        self.reader.set_read_pos(start);
        self.skip = usize::try_from(byte - start).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Appends the next bytes of every channel to `out[c]` (most significant
    /// bit first in time, whatever the container). `out.len()` must equal
    /// `channels()`. Returns `false` at the end of the audio.
    pub fn next_bytes(&mut self, out: &mut [Vec<u8>]) -> Result<bool, String> {
        if out.len() != self.channels() || out.is_empty() {
            return Err("one output per channel".to_owned());
        }
        self.lens.clear();
        self.lens.extend(out.iter().map(Vec::len));
        let more = self.reader.read_chunk(out)?;
        if self.skip > 0 {
            let fresh = out
                .first()
                .zip(self.lens.first())
                .map_or(0, |(v, before)| v.len().saturating_sub(*before));
            let drop = self.skip.min(fresh);
            for (v, before) in out.iter_mut().zip(&self.lens) {
                let end = (*before + drop).min(v.len());
                v.drain((*before).min(end)..end);
            }
            self.skip -= drop;
        }
        Ok(more)
    }
}
