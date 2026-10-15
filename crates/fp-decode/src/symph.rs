//! Everything symphonia decodes: WAV, AIFF, CAF, FLAC, MP1/2/3, AAC, ALAC,
//! Vorbis and Matroska/WebM.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use symphonia::core::codecs::audio::{AudioCodecId, AudioDecoder, AudioDecoderOptions, well_known};
use symphonia::core::errors::{Error, SeekErrorKind};
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::well_known::{
    FORMAT_ID_AIFF, FORMAT_ID_CAF, FORMAT_ID_FLAC, FORMAT_ID_ISOMP4, FORMAT_ID_MP1, FORMAT_ID_MP2,
    FORMAT_ID_MP3, FORMAT_ID_OGG, FORMAT_ID_WAVE,
};
use symphonia::core::formats::{
    FormatId, FormatOptions, FormatReader, SeekMode, SeekTo, TrackType,
};
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::units::{Duration, Time, TimeBase, Timestamp};

use crate::downmix;

pub(crate) struct SymphoniaDecoder {
    /// The file, to open it again for a restart from the start.
    path: PathBuf,
    /// Whether the file is probed as unseekable (see `open`).
    hide_seeking: bool,
    format: Box<dyn FormatReader>,
    decoder: Box<dyn AudioDecoder>,
    track_id: u32,
    /// The track's codec, checked when the file is opened again.
    codec: AudioCodecId,
    sample_rate: u32,
    bits_per_sample: Option<u32>,
    channels: usize,
    frames_hint: Option<u64>,
    /// Whether the file stores `frames_hint` (otherwise it is the reader's
    /// estimate, if any).
    length_stored: bool,
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
    /// Whether the reader is still where opening left it (nothing read, no
    /// seek since): a restart from the start needs no new reader then.
    at_start: bool,
    /// The lowest timestamp the reader can still reach by reading on: the
    /// end of the last packet read, or where the last seek landed. A seek
    /// below it goes backwards. `None` at the start of the stream.
    resume_ts: Option<Timestamp>,
    /// Seconds the container's timestamps run ahead of the audio: an Ogg
    /// Opus granule counts the pre-skip (RFC 7845 §4), which symphonia
    /// keeps as the track's delay instead of trimming it. Matroska already
    /// subtracts its codec delay, so it is 0 there.
    ts_offset_secs: f64,
    /// Frames still to drop from the start of the stream: Opus's pre-skip,
    /// read from its header so it is exact in every container, plus the
    /// target of a seek that restarted from the start.
    drop_frames: usize,
    /// Opus's pre-skip (0 for other codecs).
    pre_skip: usize,
}

/// Opus needs this much decoded audio before a seek target (RFC 7845 §4.6).
const OPUS_PREROLL_SECS: f64 = 0.08;
/// The longest Vorbis block, in frames (Vorbis I §4.2.2).
const VORBIS_MAX_BLOCK: f64 = 8192.0;

impl SymphoniaDecoder {
    pub(crate) fn open(path: &Path) -> Result<Self, String> {
        let mut format = probe(path, false)?;
        let mut hide_seeking = false;
        let length_stored = match stored_length(format.format_info().format) {
            StoredLength::Stored => true,
            StoredLength::MpegLengthFrame if mpeg_length_frame(path) => true,
            // symphonia's MPEG reader estimates the length of a seekable
            // stream from its first frames and takes it as the end: past
            // it, every packet is trimmed to nothing and seeks are out of
            // range, so a VBR file ends early. Probed as unseekable, it
            // makes no estimate and reads to the end of the stream.
            StoredLength::MpegLengthFrame => {
                format = probe(path, true)?;
                hide_seeking = true;
                false
            }
            StoredLength::Unknown => false,
        };
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
        let delay = if is_opus { track.delay.unwrap_or(0) } else { 0 };
        let frames_hint = track.num_frames.map(|n| n.saturating_sub(u64::from(delay)));
        let ts_offset_secs = f64::from(delay) / f64::from(sample_rate.max(1));
        let track_id = track.id;
        let codec = params.codec;
        let preroll_secs = match params.codec {
            well_known::CODEC_ID_OPUS => OPUS_PREROLL_SECS,
            well_known::CODEC_ID_VORBIS => VORBIS_MAX_BLOCK / f64::from(sample_rate.max(1)),
            _ => 0.0,
        };
        let time_base = track.time_base;
        Ok(Self {
            path: path.to_owned(),
            hide_seeking,
            format,
            decoder,
            track_id,
            codec,
            sample_rate,
            bits_per_sample,
            channels,
            frames_hint,
            length_stored,
            scratch: Vec::new(),
            time_base,
            target_secs: None,
            preroll_secs,
            at_end: false,
            at_start: true,
            resume_ts: None,
            ts_offset_secs,
            drop_frames: usize::from(pre_skip),
            pre_skip: usize::from(pre_skip),
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
    /// (operator feedback 4, Q1).
    pub(crate) fn declared_frames(&self) -> Option<u64> {
        self.frames_hint.filter(|_| self.length_stored)
    }

    /// Positions the stream so that the next frame produced is at `secs`.
    pub(crate) fn seek(&mut self, secs: f64) -> Result<(), String> {
        let secs = secs.max(0.0);
        // The pre-roll is dropped by placing packets in time, which needs
        // the time base; without it, decoding starts at the target.
        let preroll = if self.time_base.is_some() {
            self.preroll_secs
        } else {
            0.0
        };
        if secs - preroll <= 0.0 {
            return self.restart(secs);
        }
        let from = secs - preroll + self.ts_offset_secs;
        let time = Time::try_from_secs_f64(from).ok_or("seek position out of range")?;
        let track_id = Some(self.track_id);
        let to = || SeekTo::Time { time, track_id };
        self.at_start = false;
        let mut seeked = self.format.seek(SeekMode::Accurate, to());
        // A reader that can only seek forward (Matroska without cues)
        // lands past a target behind what it has read: seek again from
        // the start. Landing past the target is not enough to tell, since
        // the Matroska reader lands on the first block at or after it.
        let behind = |s: &symphonia::core::formats::SeekedTo| {
            s.actual_ts > s.required_ts && self.resume_ts.is_some_and(|r| s.required_ts < r)
        };
        if seeked.as_ref().is_ok_and(behind) {
            let mut format = self.reopen()?;
            seeked = format.seek(SeekMode::Accurate, to());
            self.format = format;
        }
        match seeked {
            Ok(landed) => self.resume_ts = Some(landed.actual_ts),
            Err(Error::SeekError(SeekErrorKind::OutOfRange)) => {
                self.at_end = true;
                return Ok(());
            }
            // A reader that seeks by reading on (Matroska without cues)
            // runs into the end of a file whose size it does not know (a
            // segment written to a pipe) when the target lies past it.
            Err(Error::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
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

    /// Seeks to `secs` when decoding restarts from the start of the stream
    /// anyway (`secs` within the pre-roll): the frames before `secs` are
    /// dropped by count, exactly as on the first play, from a new reader
    /// unless this one has not moved yet. Timestamps would place them only
    /// to the container's precision (Matroska's are whole milliseconds),
    /// and a Matroska file without cues cannot seek backwards at all.
    fn restart(&mut self, secs: f64) -> Result<(), String> {
        if !self.at_start {
            self.format = self.reopen()?;
            self.decoder.reset();
            self.at_start = true;
            self.resume_ts = None;
        }
        self.at_end = false;
        self.target_secs = None;
        let rate = f64::from(self.sample_rate);
        self.drop_frames = self.pre_skip + (secs * rate).round() as usize;
        Ok(())
    }

    /// Opens the file again, at its start. A file that no longer holds the
    /// same track (replaced on disk) is an error rather than being read
    /// with this decoder.
    fn reopen(&self) -> Result<Box<dyn FormatReader>, String> {
        let format = probe(&self.path, self.hide_seeking)?;
        let same = format.default_track(TrackType::Audio).is_some_and(|track| {
            let params = track.codec_params.as_ref().and_then(|p| p.audio());
            track.id == self.track_id
                && params.is_some_and(|p| {
                    p.codec == self.codec && p.sample_rate == Some(self.sample_rate)
                })
        });
        if same {
            Ok(format)
        } else {
            Err(format!("{}: the file changed", self.path.display()))
        }
    }

    /// Appends the next decoded packet as interleaved stereo. Returns `false`
    /// at the end of the stream. Corrupt packets are skipped.
    pub(crate) fn next_block(&mut self, out: &mut Vec<f32>) -> Result<bool, String> {
        if self.at_end {
            return Ok(false);
        }
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
            self.at_start = false;
            if packet.track_id != self.track_id {
                continue;
            }
            // Reading on reaches no frame before the packet's end: a target
            // inside it lies behind. Without a duration, the end is found
            // from the frames it decodes to.
            self.resume_ts = Some(if packet.dur.get() > 0 {
                packet.pts.saturating_add(packet.dur)
            } else {
                packet.pts.saturating_add(Duration::new(1))
            });
            let buf = match self.decoder.decode(&packet) {
                Ok(buf) => buf,
                Err(Error::DecodeError(_)) => continue,
                Err(e) => return Err(e.to_string()),
            };
            let channels = buf.spec().channels().count().max(1);
            let frames = buf.frames();
            if packet.dur.get() == 0 {
                let rate = f64::from(self.sample_rate.max(1));
                let len = self.time_base.and_then(|tb| {
                    tb.calc_timestamp(Time::try_from_secs_f64(frames as f64 / rate)?)
                });
                if let Some(len) = len.and_then(|l| u64::try_from(l.get()).ok()) {
                    self.resume_ts = Some(packet.pts.saturating_add(Duration::new(len.max(1))));
                }
            }
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

/// Opens `path` and probes its format. With `hide_seeking`, the stream
/// reports itself unseekable until the probe is done, so a reader makes no
/// estimate that needs seeking (see `SymphoniaDecoder::open`); seeking works
/// afterwards.
fn probe(path: &Path, hide_seeking: bool) -> Result<Box<dyn FormatReader>, String> {
    #[cfg(test)]
    OPENS.with(|n| n.set(n.get() + 1));
    let file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let seekable = Arc::new(AtomicBool::new(!hide_seeking));
    let source = ProbeSource {
        len: file.metadata().ok().map(|m| m.len()),
        file,
        seekable: Arc::clone(&seekable),
    };
    let mss = MediaSourceStream::new(Box::new(source), Default::default());
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
    seekable.store(true, Ordering::Relaxed);
    Ok(format)
}

/// A file whose seekability can be hidden while it is probed.
struct ProbeSource {
    file: File,
    len: Option<u64>,
    seekable: Arc<AtomicBool>,
}

impl Read for ProbeSource {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.file.read(buf)
    }
}

impl Seek for ProbeSource {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.file.seek(pos)
    }
}

impl MediaSource for ProbeSource {
    fn is_seekable(&self) -> bool {
        self.seekable.load(Ordering::Relaxed)
    }

    fn byte_len(&self) -> Option<u64> {
        self.len
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
#[path = "../tests/support/opus_files.rs"]
mod opus_files;

#[cfg(test)]
thread_local! {
    /// How many times this thread opened a file for symphonia.
    static OPENS: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
mod reopen_tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    #![allow(clippy::indexing_slicing)]

    use std::path::{Path, PathBuf};

    use super::opus_files::{
        Layout, RATE, encode, marker, matroska_with, packets, pcm_matroska, stereo,
    };
    use super::{OPENS, SymphoniaDecoder};

    fn opens() -> u32 {
        OPENS.with(std::cell::Cell::get)
    }

    /// An Ogg Opus file and Matroska files with and without cues, 1 s each.
    fn files(dir: &Path) -> Vec<PathBuf> {
        let pcm = stereo(1.0, marker);
        let (packets, pre_skip) = packets(&pcm);
        vec![
            encode(dir, "a.opus", &pcm),
            matroska_with(dir, "cues.mka", &packets, pre_skip, true),
            matroska_with(dir, "bare.mka", &packets, pre_skip, false),
        ]
    }

    fn rest(d: &mut SymphoniaDecoder) -> Vec<f32> {
        let mut out = Vec::new();
        while d.next_block(&mut out).unwrap() {}
        out
    }

    /// A fresh decoder seeking forward opens the file once: symphonia's
    /// Matroska reader lands on the first block at or after the target
    /// (its blocks have no duration), which is not a seek backwards.
    #[test]
    fn a_fresh_forward_seek_does_not_open_the_file_again() {
        let dir = tempfile::tempdir().unwrap();
        for path in files(dir.path()) {
            for secs in [0.37, 0.5] {
                let before = opens();
                let mut d = SymphoniaDecoder::open(&path).unwrap();
                let opened = opens() - before;
                d.seek(secs).unwrap();
                let _ = rest(&mut d);
                assert_eq!(opens() - before, opened, "{}: {secs} s", path.display());
            }
        }
    }

    /// A fresh decoder seeking within the pre-roll drops the frames by
    /// count from where it already is, without opening the file again.
    #[test]
    fn a_fresh_seek_within_the_pre_roll_does_not_open_the_file_again() {
        let dir = tempfile::tempdir().unwrap();
        for path in files(dir.path()) {
            let full = rest(&mut SymphoniaDecoder::open(&path).unwrap());
            let before = opens();
            let mut d = SymphoniaDecoder::open(&path).unwrap();
            let opened = opens() - before;
            d.seek(0.05).unwrap();
            let out = rest(&mut d);
            assert_eq!(opens() - before, opened, "{}", path.display());
            let skip = 2 * RATE * 5 / 100;
            assert!(out[..] == full[skip..], "{}", path.display());
        }
    }

    /// A seek forward from the end of the block read last, or past it, does
    /// not open the file again: only a target inside or before that block
    /// lies behind.
    #[test]
    fn a_seek_forward_after_reading_does_not_open_the_file_again() {
        let dir = tempfile::tempdir().unwrap();
        let layout = Layout::BARE;
        // 100 ms blocks.
        let (path, _) = pcm_matroska(dir.path(), "bare.mka", 1.0, 4800, layout);
        for secs in [0.3, 0.35] {
            let mut d = SymphoniaDecoder::open(&path).unwrap();
            let mut out = Vec::new();
            for _ in 0..3 {
                assert!(d.next_block(&mut out).unwrap());
            }
            let before = opens();
            d.seek(secs).unwrap();
            let _ = rest(&mut d);
            assert_eq!(opens(), before, "{secs} s");
        }
    }

    /// A seek back still works after a seek forward that read nothing.
    #[test]
    fn a_seek_back_after_a_seek_that_read_nothing_lands_right() {
        let dir = tempfile::tempdir().unwrap();
        for path in files(dir.path()) {
            let full = rest(&mut SymphoniaDecoder::open(&path).unwrap());
            let mut d = SymphoniaDecoder::open(&path).unwrap();
            d.seek(0.6).unwrap();
            d.seek(0.0).unwrap();
            assert!(rest(&mut d) == full, "{}", path.display());
        }
    }

    /// A file replaced on disk by one of another kind is not read with the
    /// decoder of the first one: the seek that opens it again fails.
    #[test]
    fn a_file_replaced_by_another_kind_fails_the_reopening_seek() {
        let dir = tempfile::tempdir().unwrap();
        let path = files(dir.path()).remove(0);
        let mut d = SymphoniaDecoder::open(&path).unwrap();
        let mut out = Vec::new();
        while out.len() < RATE && d.next_block(&mut out).unwrap() {}
        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: 44_100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut w = hound::WavWriter::create(&path, spec).unwrap();
        for _ in 0..44_100 {
            w.write_sample(0i16).unwrap();
        }
        w.finalize().unwrap();
        assert!(d.seek(0.0).is_err());
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
