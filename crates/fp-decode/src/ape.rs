//! Monkey's Audio (audio formats spec F2), decoded with `ape-decoder` one
//! frame at a time. Frames are independent, so a seek decodes from the
//! frame that holds the target and skips to it.

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use ape_decoder::ApeDecoder;

/// Most channels accepted (the downmix folds up to 5.1 and keeps the front
/// pair of anything wider).
const MAX_CHANNELS: u16 = 8;

pub(crate) struct ApeFileDecoder {
    decoder: ApeDecoder<BufReader<File>>,
    next: u32,
    skip: usize,
    channels: usize,
    bytes_per_sample: usize,
    frame: Vec<f32>,
}

fn err(e: ape_decoder::ApeError) -> String {
    format!("Monkey's Audio: {e}")
}

impl ApeFileDecoder {
    pub(crate) fn open(path: &Path) -> Result<Self, String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let decoder = ApeDecoder::new(BufReader::new(file)).map_err(err)?;
        let info = decoder.info();
        let bytes_per_sample = usize::from(info.bits_per_sample / 8);
        let channels = usize::from(info.channels);
        if info.channels == 0
            || info.channels > MAX_CHANNELS
            || info.sample_rate == 0
            || !matches!(info.bits_per_sample, 8 | 16 | 24 | 32)
            || usize::from(info.block_align) != channels * bytes_per_sample
        {
            return Err("Monkey's Audio: unsupported stream parameters".into());
        }
        if info.total_frames == 0 || info.total_samples == 0 {
            return Err("Monkey's Audio: no audio".into());
        }
        Ok(Self {
            decoder,
            next: 0,
            skip: 0,
            channels,
            bytes_per_sample,
            frame: vec![0.0; channels],
        })
    }

    pub(crate) fn sample_rate(&self) -> u32 {
        self.decoder.info().sample_rate
    }

    pub(crate) fn bits_per_sample(&self) -> Option<u32> {
        let info = self.decoder.info();
        (!info.is_floating_point).then_some(u32::from(info.bits_per_sample))
    }

    pub(crate) fn channels(&self) -> usize {
        self.channels
    }

    pub(crate) fn frames_hint(&self) -> Option<u64> {
        Some(self.decoder.info().total_samples)
    }

    pub(crate) fn seek(&mut self, secs: f64) -> Result<(), String> {
        // Saturating float-to-int conversion.
        let target = (secs.max(0.0) * f64::from(self.sample_rate())).round() as u64;
        if target >= self.decoder.info().total_samples {
            self.next = self.decoder.total_frames();
            self.skip = 0;
            return Ok(());
        }
        let at = self.decoder.seek(target).map_err(err)?;
        self.next = at.frame_index;
        self.skip = usize::try_from(at.skip_samples).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// One sample from its little- or big-endian bytes, scaled to ±1.
    fn sample(&self, bytes: &[u8]) -> f32 {
        let info = self.decoder.info();
        let mut b = [0u8; 4];
        for (i, byte) in bytes.iter().enumerate() {
            let at = if info.is_big_endian {
                bytes.len() - 1 - i
            } else {
                i
            };
            if let Some(slot) = b.get_mut(at) {
                *slot = *byte;
            }
        }
        match bytes.len() {
            1 if info.is_signed_8bit => f32::from(b[0] as i8) / 128.0,
            1 => (f32::from(b[0]) - 128.0) / 128.0,
            2 => f32::from(i16::from_le_bytes([b[0], b[1]])) / 32_768.0,
            3 => (i32::from_le_bytes([0, b[0], b[1], b[2]]) >> 8) as f32 / 8_388_608.0,
            4 if info.is_floating_point => f32::from_le_bytes(b),
            4 => i32::from_le_bytes(b) as f32 / 2_147_483_648.0,
            _ => 0.0,
        }
    }

    pub(crate) fn next_block(&mut self, out: &mut Vec<f32>) -> Result<bool, String> {
        if self.next >= self.decoder.total_frames() {
            return Ok(false);
        }
        let pcm = self.decoder.decode_frame(self.next).map_err(err)?;
        self.next += 1;
        let skip = std::mem::take(&mut self.skip);
        let frame_bytes = self.channels * self.bytes_per_sample;
        for frame in pcm.chunks_exact(frame_bytes).skip(skip) {
            for (c, bytes) in frame.chunks_exact(self.bytes_per_sample).enumerate() {
                let value = self.sample(bytes);
                if let Some(slot) = self.frame.get_mut(c) {
                    *slot = value;
                }
            }
            let (l, r) = crate::downmix(&self.frame);
            out.push(l);
            out.push(r);
        }
        Ok(true)
    }
}
