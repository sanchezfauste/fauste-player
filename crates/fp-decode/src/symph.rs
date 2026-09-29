//! Everything symphonia decodes: WAV, AIFF, CAF, FLAC, MP1/2/3, AAC, ALAC,
//! Vorbis and Matroska/WebM.

use std::fs::File;
use std::path::Path;

use symphonia::core::codecs::audio::{AudioDecoder, AudioDecoderOptions};
use symphonia::core::errors::Error;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo, TrackType};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::units::Time;

use crate::downmix;

pub(crate) struct SymphoniaDecoder {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn AudioDecoder>,
    track_id: u32,
    sample_rate: u32,
    bits_per_sample: Option<u32>,
    channels: usize,
    frames_hint: Option<u64>,
    scratch: Vec<f32>,
    /// Frames still to drop after a seek (the decoder lands before the target).
    skip_frames: u64,
}

impl SymphoniaDecoder {
    pub(crate) fn open(path: &Path) -> Result<Self, String> {
        let file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }
        let format = symphonia::default::get_probe()
            .probe(
                &hint,
                mss,
                FormatOptions::default(),
                MetadataOptions::default(),
            )
            .map_err(|e| format!("{}: {e}", path.display()))?;
        let track = format
            .default_track(TrackType::Audio)
            .ok_or_else(|| format!("{}: no audio track", path.display()))?;
        let params = track
            .codec_params
            .as_ref()
            .and_then(|p| p.audio())
            .ok_or_else(|| format!("{}: no audio parameters", path.display()))?;
        let sample_rate = params
            .sample_rate
            .ok_or_else(|| format!("{}: unknown sample rate", path.display()))?;
        let decoder = crate::opus::codecs()
            .make_audio_decoder(params, &AudioDecoderOptions::default())
            .map_err(|e| format!("{}: {e}", path.display()))?;
        let channels = params.channels.as_ref().map_or(0, |c| c.count());
        // Lossy codecs have no sample size: their output is not integer PCM.
        let bits_per_sample = params.bits_per_sample.filter(|b| *b > 0);
        let frames_hint = track.num_frames;
        let track_id = track.id;
        Ok(Self {
            format,
            decoder,
            track_id,
            sample_rate,
            bits_per_sample,
            channels,
            frames_hint,
            scratch: Vec::new(),
            skip_frames: 0,
        })
    }

    pub(crate) fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Bits per sample of integer PCM (lossless codecs); `None` for lossy.
    pub(crate) fn bits_per_sample(&self) -> Option<u32> {
        self.bits_per_sample
    }

    /// Channels in the file (0 if the container does not say).
    pub(crate) fn channels(&self) -> usize {
        self.channels
    }

    /// Total frames, when the container knows it without decoding.
    pub(crate) fn frames_hint(&self) -> Option<u64> {
        self.frames_hint
    }

    /// Positions the stream so that the next frame produced is at `secs`.
    pub(crate) fn seek(&mut self, secs: f64) -> Result<(), String> {
        if secs <= 0.0 {
            return Ok(());
        }
        let time = Time::try_from_secs_f64(secs).ok_or("seek position out of range")?;
        let seeked = self
            .format
            .seek(
                SeekMode::Accurate,
                SeekTo::Time {
                    time,
                    track_id: Some(self.track_id),
                },
            )
            .map_err(|e| e.to_string())?;
        self.decoder.reset();
        let time_base = self
            .format
            .tracks()
            .iter()
            .find(|t| t.id == self.track_id)
            .and_then(|t| t.time_base);
        let landed = time_base.map_or(secs, |tb| {
            tb.calc_time_saturating(seeked.actual_ts).as_secs_f64()
        });
        let behind = (secs - landed).max(0.0);
        self.skip_frames = (behind * f64::from(self.sample_rate)).round() as u64;
        Ok(())
    }

    /// Appends the next decoded packet as interleaved stereo. Returns `false`
    /// at the end of the stream. Corrupt packets are skipped.
    pub(crate) fn next_block(&mut self, out: &mut Vec<f32>) -> Result<bool, String> {
        loop {
            let packet = match self.format.next_packet() {
                Ok(Some(packet)) => packet,
                Ok(None) => return Ok(false),
                // A truncated file (interrupted download, damaged tail) ends
                // where its data ends instead of failing.
                Err(Error::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    return Ok(false);
                }
                Err(e) => return Err(e.to_string()),
            };
            if packet.track_id != self.track_id {
                continue;
            }
            let buf = match self.decoder.decode(&packet) {
                Ok(buf) => buf,
                Err(Error::DecodeError(_)) => continue,
                Err(e) => return Err(e.to_string()),
            };
            let channels = buf.spec().channels().count().max(1);
            let frames = buf.frames();
            self.scratch.resize(frames * channels, 0.0);
            buf.copy_to_slice_interleaved(self.scratch.as_mut_slice());
            let skip = usize::try_from(self.skip_frames)
                .unwrap_or(usize::MAX)
                .min(frames);
            self.skip_frames -= skip as u64;
            for frame in self.scratch.chunks_exact(channels).skip(skip) {
                let (l, r) = downmix(frame);
                out.push(l);
                out.push(r);
            }
            return Ok(true);
        }
    }
}
