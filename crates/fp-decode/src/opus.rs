//! Opus for symphonia (audio formats spec F1): a decoder built on
//! `opus-decoder`, registered for Opus tracks in Ogg and Matroska. It
//! always decodes at 48 kHz and applies the output gain of the Opus header.
//! Like symphonia's own decoders, it removes the pre-skip and end padding the
//! container marks on each packet when gapless decoding is on (the default).

use std::sync::OnceLock;

use symphonia::core::audio::layouts::{CHANNEL_LAYOUT_MONO, CHANNEL_LAYOUT_STEREO};
use symphonia::core::audio::{
    AsGenericAudioBufferRef, AudioBuffer, AudioMut, AudioSpec, GenericAudioBufferRef,
};
use symphonia::core::codecs::CodecInfo;
use symphonia::core::codecs::audio::well_known::CODEC_ID_OPUS;
use symphonia::core::codecs::audio::{
    AudioCodecParameters, AudioDecoder, AudioDecoderOptions, FinalizeResult,
};
use symphonia::core::codecs::registry::{
    CodecRegistry, RegisterableAudioDecoder, SupportedAudioCodec,
};
use symphonia::core::errors::{Error, Result, decode_error, unsupported_error};
use symphonia::core::packet::PacketRef;

/// Opus always decodes at 48 kHz.
const RATE: u32 = 48_000;
/// The longest Opus packet: 120 ms at 48 kHz.
const MAX_FRAMES: usize = 5760;

const INFO: CodecInfo = CodecInfo {
    short_name: "opus",
    long_name: "Opus",
    profiles: &[],
};

const SUPPORTED: &[SupportedAudioCodec] = &[SupportedAudioCodec {
    id: CODEC_ID_OPUS,
    info: INFO,
}];

/// symphonia's codecs plus Opus.
pub(crate) fn codecs() -> &'static CodecRegistry {
    static REGISTRY: OnceLock<CodecRegistry> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut registry = CodecRegistry::new();
        symphonia::default::register_enabled_codecs(&mut registry);
        registry.register_audio_decoder::<OpusDecoder>();
        registry
    })
}

pub(crate) struct OpusDecoder {
    params: AudioCodecParameters,
    gapless: bool,
    decoder: opus_decoder::OpusDecoder,
    channels: usize,
    /// Output gain from the Opus header, as a factor.
    gain: f32,
    pcm: Vec<f32>,
    buf: AudioBuffer<f32>,
}

/// Channel count and output gain from an `OpusHead` (RFC 7845 §5.1). Only
/// mapping family 0 (mono or stereo) is supported.
fn parse_head(head: &[u8]) -> Result<(usize, f32)> {
    if head.get(0..8) != Some(b"OpusHead") {
        return decode_error("opus: missing OpusHead");
    }
    let channels = head.get(9).copied().map(usize::from);
    let gain = head
        .get(16..18)
        .and_then(|b| b.try_into().ok())
        .map(i16::from_le_bytes);
    let family = head.get(18).copied();
    match (channels, gain, family) {
        (Some(channels @ 1..=2), Some(gain), Some(0)) => {
            // Q7.8 decibels.
            let db = f32::from(gain) / 256.0;
            Ok((channels, 10f32.powf(db / 20.0)))
        }
        (_, _, Some(family)) if family != 0 => {
            unsupported_error("opus: multichannel (mapping family other than 0)")
        }
        _ => decode_error("opus: malformed OpusHead"),
    }
}

impl OpusDecoder {
    fn try_new(params: &AudioCodecParameters, opts: &AudioDecoderOptions) -> Result<Self> {
        let head = params
            .extra_data
            .as_deref()
            .ok_or(Error::DecodeError("opus: no OpusHead"))?;
        let (channels, gain) = parse_head(head)?;
        let decoder = opus_decoder::OpusDecoder::new(RATE, channels)
            .map_err(|_| Error::DecodeError("opus: decoder rejected the stream"))?;
        let layout = if channels == 1 {
            CHANNEL_LAYOUT_MONO
        } else {
            CHANNEL_LAYOUT_STEREO
        };
        Ok(Self {
            params: params.clone(),
            gapless: opts.gapless,
            decoder,
            channels,
            gain,
            pcm: vec![0.0; MAX_FRAMES * channels],
            buf: AudioBuffer::new(AudioSpec::new(RATE, layout), MAX_FRAMES),
        })
    }

    fn decode_inner(&mut self, packet: &PacketRef<'_>) -> Result<()> {
        let frames = self
            .decoder
            .decode_float(packet.data, &mut self.pcm, false)
            .map_err(|_| Error::DecodeError("opus: invalid packet"))?
            .min(MAX_FRAMES);
        self.buf.clear();
        self.buf.render_uninit(Some(frames));
        for c in 0..self.channels {
            if let Some(plane) = self.buf.plane_mut(c) {
                let samples = self.pcm.iter().skip(c).step_by(self.channels);
                for (out, s) in plane.iter_mut().zip(samples) {
                    *out = s * self.gain;
                }
            }
        }
        if self.gapless {
            let start = usize::try_from(packet.trim_start.get()).unwrap_or(usize::MAX);
            let end = usize::try_from(packet.trim_end.get()).unwrap_or(usize::MAX);
            self.buf.trim(start, end);
        }
        Ok(())
    }
}

impl AudioDecoder for OpusDecoder {
    fn reset(&mut self) {
        self.decoder.reset();
    }

    fn codec_info(&self) -> &CodecInfo {
        &INFO
    }

    fn codec_params(&self) -> &AudioCodecParameters {
        &self.params
    }

    fn decode_ref(&mut self, packet: &PacketRef<'_>) -> Result<GenericAudioBufferRef<'_>> {
        if let Err(e) = self.decode_inner(packet) {
            self.buf.clear();
            return Err(e);
        }
        Ok(self.buf.as_generic_audio_buffer_ref())
    }

    fn finalize(&mut self) -> FinalizeResult {
        FinalizeResult::default()
    }

    fn last_decoded(&self) -> GenericAudioBufferRef<'_> {
        self.buf.as_generic_audio_buffer_ref()
    }
}

impl RegisterableAudioDecoder for OpusDecoder {
    fn try_registry_new(
        params: &AudioCodecParameters,
        opts: &AudioDecoderOptions,
    ) -> Result<Box<dyn AudioDecoder>> {
        Ok(Box::new(Self::try_new(params, opts)?))
    }

    fn supported_codecs() -> &'static [SupportedAudioCodec] {
        SUPPORTED
    }
}
