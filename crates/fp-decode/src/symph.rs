//! Everything symphonia decodes: WAV, AIFF, CAF, FLAC, MP1/2/3, AAC, ALAC,
//! Vorbis and Matroska/WebM.

use std::fs::File;
use std::path::{Path, PathBuf};

use symphonia::core::codecs::audio::{AudioDecoder, AudioDecoderOptions, well_known};
use symphonia::core::errors::{Error, SeekErrorKind};
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::well_known::{
    FORMAT_ID_AIFF, FORMAT_ID_CAF, FORMAT_ID_FLAC, FORMAT_ID_ISOMP4, FORMAT_ID_MP1, FORMAT_ID_MP2,
    FORMAT_ID_MP3, FORMAT_ID_OGG, FORMAT_ID_WAVE,
};
use symphonia::core::formats::{
    FormatId, FormatOptions, FormatReader, SeekMode, SeekTo, TrackType,
};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::units::{Time, TimeBase};

use crate::downmix;

pub(crate) struct SymphoniaDecoder {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn AudioDecoder>,
    track_id: u32,
    sample_rate: u32,
    bits_per_sample: Option<u32>,
    channels: usize,
    frames_hint: Option<u64>,
    /// The file, read again for an MPEG stream's length frame.
    path: PathBuf,
    /// How the container's reader knows `frames_hint`.
    stored_length: StoredLength,
    scratch: Vec<f32>,
    /// The track's time base, to place each decoded packet in time.
    time_base: Option<TimeBase>,
    /// After a seek, the time (seconds) the output must start at: frames of
    /// a packet placed before it are dropped.
    target_secs: Option<f64>,
    /// How far before a seek target decoding restarts, so the decoder has
    /// converged (and produces output) by the target: 80 ms for Opus
    /// (RFC 7845 §4.6), the longest block for Vorbis (its first packet
    /// after a reset only primes the overlap), none for the rest.
    preroll_secs: f64,
    /// A seek went past the end: nothing more to play (as for the other
    /// decoders), instead of an error.
    at_end: bool,
    /// Whether anything was read yet: a seek to 0 before that is a no-op.
    read_any: bool,
    /// Seconds the container's timestamps run ahead of the audio: an Ogg
    /// Opus granule counts the pre-skip (RFC 7845 §4), which symphonia
    /// keeps as the track's delay instead of trimming it. Matroska already
    /// subtracts its codec delay, so it is 0 there.
    ts_offset_secs: f64,
    /// Frames still to drop from the start of the stream: Opus's pre-skip,
    /// read from its header so it is exact in every container.
    drop_frames: usize,
}

/// Opus needs this much decoded audio before a seek target (RFC 7845 §4.6).
const OPUS_PREROLL_SECS: f64 = 0.08;
/// The longest Vorbis block, in frames (Vorbis I §4.2.2).
const VORBIS_MAX_BLOCK: f64 = 8192.0;

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
        let is_opus = params.codec == well_known::CODEC_ID_OPUS;
        // Opus starts with its pre-skip, which is dropped (RFC 7845 §4.2).
        let pre_skip = if is_opus {
            params
                .extra_data
                .as_deref()
                .and_then(|head| head.get(10..12))
                .and_then(|b| b.try_into().ok())
                .map_or(0, u16::from_le_bytes)
        } else {
            0
        };
        let stored_length = stored_length(format.format_info().format);
        let delay = if is_opus { track.delay.unwrap_or(0) } else { 0 };
        let frames_hint = track.num_frames.map(|n| n.saturating_sub(u64::from(delay)));
        let ts_offset_secs = f64::from(delay) / f64::from(sample_rate.max(1));
        let track_id = track.id;
        let preroll_secs = match params.codec {
            well_known::CODEC_ID_OPUS => OPUS_PREROLL_SECS,
            well_known::CODEC_ID_VORBIS => VORBIS_MAX_BLOCK / f64::from(sample_rate.max(1)),
            _ => 0.0,
        };
        let time_base = track.time_base;
        Ok(Self {
            format,
            decoder,
            track_id,
            sample_rate,
            bits_per_sample,
            channels,
            frames_hint,
            path: path.to_path_buf(),
            stored_length,
            scratch: Vec::new(),
            time_base,
            target_secs: None,
            preroll_secs,
            at_end: false,
            read_any: false,
            ts_offset_secs,
            drop_frames: usize::from(pre_skip),
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

    /// `frames_hint` when the file stores it, `None` when it is only the
    /// reader's estimate or the reader is not known to read a stored count
    /// (operator feedback 4, Q1). For MPEG audio this reads the file's
    /// first frame again, so it is called only for a track's length.
    pub(crate) fn declared_frames(&self) -> Option<u64> {
        let stored = match self.stored_length {
            StoredLength::Stored => true,
            StoredLength::MpegLengthFrame => mpeg_length_frame(&self.path),
            StoredLength::Unknown => false,
        };
        self.frames_hint.filter(|_| stored)
    }

    /// Positions the stream so that the next frame produced is at `secs`.
    pub(crate) fn seek(&mut self, secs: f64) -> Result<(), String> {
        if secs <= 0.0 && !self.read_any && !self.at_end {
            return Ok(());
        }
        let secs = secs.max(0.0);
        // The pre-roll is dropped by placing packets in time, which needs
        // the time base; without it, decoding starts at the target.
        let preroll = if self.time_base.is_some() {
            self.preroll_secs
        } else {
            0.0
        };
        let from = (secs - preroll).max(0.0) + self.ts_offset_secs;
        let time = Time::try_from_secs_f64(from).ok_or("seek position out of range")?;
        match self.format.seek(
            SeekMode::Accurate,
            SeekTo::Time {
                time,
                track_id: Some(self.track_id),
            },
        ) {
            Ok(_) => {}
            Err(Error::SeekError(SeekErrorKind::OutOfRange)) => {
                self.at_end = true;
                return Ok(());
            }
            Err(e) => return Err(e.to_string()),
        }
        self.at_end = false;
        self.drop_frames = 0;
        self.decoder.reset();
        // Each packet is placed by its timestamp, so a codec that yields
        // nothing for its first packet after a reset still starts on time.
        // Without a time base the landing is taken as the target.
        self.target_secs = self.time_base.map(|_| secs);
        Ok(())
    }

    /// Appends the next decoded packet as interleaved stereo. Returns `false`
    /// at the end of the stream. Corrupt packets are skipped.
    pub(crate) fn next_block(&mut self, out: &mut Vec<f32>) -> Result<bool, String> {
        if self.at_end {
            return Ok(false);
        }
        self.read_any = true;
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
            let skip = match (self.target_secs, self.time_base) {
                (Some(target), Some(tb)) => {
                    let rate = f64::from(self.sample_rate.max(1));
                    let secs = |ts| tb.calc_time_saturating(ts).as_secs_f64() - self.ts_offset_secs;
                    // The decoded frames end where the packet's valid frames
                    // end; a container that gives no duration (Matroska
                    // blocks) places them from the packet's start instead.
                    let (start, end) = if packet.dur.get() > 0 {
                        let end = secs(packet.pts.saturating_add(packet.dur));
                        (end - frames as f64 / rate, end)
                    } else {
                        let start = secs(packet.pts);
                        (start, start + frames as f64 / rate)
                    };
                    if end > target {
                        self.target_secs = None;
                    }
                    let behind = ((target - start) * rate).round().max(0.0);
                    (behind as usize).min(frames)
                }
                _ => 0,
            };
            let skip = skip.max(self.drop_frames.min(frames));
            self.drop_frames = self.drop_frames.saturating_sub(frames);
            for frame in self.scratch.chunks_exact(channels).skip(skip) {
                let (l, r) = downmix(frame);
                out.push(l);
                out.push(r);
            }
            return Ok(true);
        }
    }
}

/// How far after any ID3v2 tag the first MPEG frame is looked for, and how
/// much of it is read: room for junk before the sync and for the tag.
const MPEG_LENGTH_FRAME_SPAN: usize = 4096;

/// The size of an MPEG audio frame header.
const MPEG_HEADER_LEN: usize = 4;

/// Whether an MPEG audio file's first frame declares the stream's length
/// (a Xing, Info or VBRI frame). Without one symphonia estimates the length
/// from the first frames' bitrate. The tags are looked for where symphonia
/// reads them: Xing and Info right after the side information, VBRI 32
/// bytes after the header.
fn mpeg_length_frame(path: &Path) -> bool {
    use std::io::{Seek, SeekFrom};
    let Ok(mut file) = File::open(path) else {
        return false;
    };
    let mut head = [0u8; 10];
    let Ok(read) = crate::read_up_to(&mut file, &mut head) else {
        return false;
    };
    let start = crate::id3v2_len(head.get(..read).unwrap_or_default()).unwrap_or(0);
    if file.seek(SeekFrom::Start(start)).is_err() {
        return false;
    }
    let mut span = vec![0u8; MPEG_LENGTH_FRAME_SPAN];
    let Ok(read) = crate::read_up_to(&mut file, &mut span) else {
        return false;
    };
    has_length_frame(span.get(..read).unwrap_or_default())
}

/// Whether the first Layer III frame header in `span` starts a Xing, Info
/// or VBRI frame.
fn has_length_frame(span: &[u8]) -> bool {
    let Some((at, side_info_len)) = (0..span.len())
        .find_map(|i| layer3_side_info_len(span.get(i..i + MPEG_HEADER_LEN)?).map(|len| (i, len)))
    else {
        return false;
    };
    let tag_at = |offset: usize| span.get(at + offset..at + offset + 4);
    matches!(
        tag_at(MPEG_HEADER_LEN + side_info_len),
        Some(b"Xing" | b"Info")
    ) || tag_at(MPEG_HEADER_LEN + 32) == Some(b"VBRI")
}

/// The side information length of a Layer III frame starting with
/// `header` (ISO/IEC 11172-3 and 13818-3): 17 or 32 bytes for MPEG-1, 9 or
/// 17 for MPEG-2 and 2.5, mono or not. `None` when `header` is not the
/// header of a Layer III frame.
fn layer3_side_info_len(header: &[u8]) -> Option<usize> {
    let [0xFF, b1, b2, b3] = *header else {
        return None;
    };
    let version = (b1 >> 3) & 0b11;
    let layer = (b1 >> 1) & 0b11;
    let bitrate = b2 >> 4;
    let rate = (b2 >> 2) & 0b11;
    // Sync (11 bits), a version that is not reserved, Layer III, a bitrate
    // index that is neither free nor bad, a sample rate that is not
    // reserved.
    if b1 & 0xE0 != 0xE0 || version == 0b01 || layer != 0b01 || bitrate == 0 || bitrate == 0xF {
        return None;
    }
    if rate == 0b11 {
        return None;
    }
    let mono = b3 >> 6 == 0b11;
    Some(match (version == 0b11, mono) {
        (true, true) => 17,
        (true, false) => 32,
        (false, true) => 9,
        (false, false) => 17,
    })
}

/// How a container's frame count is known (operator feedback 4, Q1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StoredLength {
    /// The reader takes the count from the file.
    Stored,
    /// Only when the stream's first frame is a length frame.
    MpegLengthFrame,
    /// An estimate, or no count at all.
    Unknown,
}

/// An allowlist: only readers known to read a stored count are trusted.
/// FLAC (STREAMINFO), WAV and AIFF (the data chunk), CAF, MP4/M4A (the
/// track's duration) and Ogg (the last page's granule, so Vorbis and Opus)
/// store it. An MPEG stream stores it in a Xing, Info or VBRI frame;
/// without one, as for ADTS, symphonia estimates it from a sample of frame
/// sizes, which on a VBR stream can end a track early.
fn stored_length(format: FormatId) -> StoredLength {
    if [
        FORMAT_ID_FLAC,
        FORMAT_ID_WAVE,
        FORMAT_ID_AIFF,
        FORMAT_ID_CAF,
        FORMAT_ID_ISOMP4,
        FORMAT_ID_OGG,
    ]
    .contains(&format)
    {
        StoredLength::Stored
    } else if [FORMAT_ID_MP1, FORMAT_ID_MP2, FORMAT_ID_MP3].contains(&format) {
        StoredLength::MpegLengthFrame
    } else {
        StoredLength::Unknown
    }
}

#[cfg(test)]
mod tests {
    use symphonia::core::common::FourCc;
    use symphonia::core::formats::well_known::*;
    use symphonia::core::formats::{FORMAT_ID_NULL, FormatId};

    use super::{StoredLength, stored_length};

    #[test]
    fn only_readers_that_read_a_stored_count_are_trusted() {
        for f in [
            FORMAT_ID_FLAC,
            FORMAT_ID_WAVE,
            FORMAT_ID_AIFF,
            FORMAT_ID_CAF,
            FORMAT_ID_ISOMP4,
            FORMAT_ID_OGG,
        ] {
            assert_eq!(stored_length(f), StoredLength::Stored, "{f:?}");
        }
        for f in [FORMAT_ID_MP1, FORMAT_ID_MP2, FORMAT_ID_MP3] {
            assert_eq!(stored_length(f), StoredLength::MpegLengthFrame, "{f:?}");
        }
        // ADTS estimates from sampled frame sizes; the rest, and any
        // reader added later, are not known to store a count.
        for f in [
            FORMAT_ID_ADTS,
            FORMAT_ID_MKV,
            FORMAT_ID_AVI,
            FORMAT_ID_FLV,
            FORMAT_ID_WAVPACK,
            FormatId::new(FourCc::new(*b"zzzz")),
            FORMAT_ID_NULL,
        ] {
            assert_eq!(stored_length(f), StoredLength::Unknown, "{f:?}");
        }
    }
}
