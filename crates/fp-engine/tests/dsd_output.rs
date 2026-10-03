#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! DSD reaching bit-perfect devices unchanged (feedback 2 spec O25), end to
//! end on an Offline device: DoP frames byte for byte, silence, mixing
//! policies and fallbacks.

mod support;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use fp_backends::dsd::{DsdStream, sample_to_word, silence_sample};
use fp_backends::exclusive::write_samples;
use fp_backends::{AudioBackend, OfflineBackend, OfflineDevice, SampleFormat};
use fp_engine::engine::{Engine, EngineSettings};
use fp_engine::worker::{dsd_file_opener, file_opener};
use fp_model::{
    AudioFormat, Config, DsdDevice, DsdMix, DsdOutput, EngineAction, EngineEvent, EntryId,
    OutputDevice, PlayerId, PlayerRoutes, Route, SourceRequest, TrackId, TransitionPlan,
};
use support::{DSD64, dsf_file, indexed_wav};

const P: PlayerId = PlayerId(1);
const Q: PlayerId = PlayerId(2);
const BLOCK: usize = 480;
const WORD_RATE: u32 = DSD64 / 16; // 176 400
const SILENCE_MS: f64 = 10.0;
const SILENCE_FRAMES: usize = 1_764; // 10 ms at 176.4 kHz
/// The rate of the PCM conversion of a DSD64 file.
const PCM_RATE: u32 = DSD64 / 32;

struct Rig {
    engine: Engine,
    dac: OfflineDevice,
    clock: Instant,
    dir: tempfile::TempDir,
    /// Everything `Engine::tick` returned so far.
    seen: Vec<EngineEvent>,
}

fn rig(mode: DsdOutput, mix: DsdMix, format: SampleFormat) -> Rig {
    rig_with(mode, mix, format, |_| {})
}

fn rig_with(
    mode: DsdOutput,
    mix: DsdMix,
    format: SampleFormat,
    device: impl FnOnce(&OfflineDevice),
) -> Rig {
    let backend = OfflineBackend::new();
    let dac = backend.add_device("dac", 2);
    dac.set_exclusive_capable(true);
    dac.set_sample_format(format);
    device(&dac);
    let route = |player| PlayerRoutes {
        player,
        main: Some(Route {
            backend: "offline".into(),
            device: "dac".into(),
            first_channel: 0,
        }),
        cue: None,
    };
    let mut config = Config::default();
    config.outputs.backend = Some("offline".into());
    config.outputs.buffer_frames = BLOCK as u32;
    config.outputs.routes = vec![route(P), route(Q)];
    config.outputs.bit_perfect = vec![OutputDevice {
        backend: "offline".into(),
        device: "dac".into(),
    }];
    config.outputs.dsd_output = vec![DsdDevice {
        backend: "offline".into(),
        device: "dac".into(),
        mode,
    }];
    config.outputs.dsd_mix = mix;
    config.outputs.dsd_silence_ms = SILENCE_MS;
    config.tuning.gain_smoothing_ms = 0.0;
    config.tuning.prebuffer_secs = 4.0;
    config.tuning.ready_threshold_ms = 1_500.0;
    let backends: Vec<Arc<dyn AudioBackend>> = vec![Arc::new(backend)];
    let mut engine = Engine::new(
        backends,
        EngineSettings::from_config(&config),
        file_opener(),
    );
    let clock = Instant::now();
    engine.execute(EngineAction::AddPlayer { player: P }, clock);
    engine.execute(EngineAction::AddPlayer { player: Q }, clock);
    Rig {
        engine,
        dac,
        clock,
        dir: tempfile::tempdir().unwrap(),
        seen: Vec::new(),
    }
}

fn dsd64() -> Option<AudioFormat> {
    Some(AudioFormat {
        sample_rate: PCM_RATE,
        bits: None,
        channels: 2,
        dsd_rate: Some(DSD64),
    })
}

fn pcm16(rate: u32) -> Option<AudioFormat> {
    Some(AudioFormat {
        sample_rate: rate,
        bits: Some(16),
        channels: 2,
        dsd_rate: None,
    })
}

/// Two seconds of recognisable bytes that never contain the idle byte.
fn pattern() -> (Vec<u8>, Vec<u8>) {
    pattern_mod(253)
}

/// Like `pattern`, with another period, so two files can be told apart.
fn pattern_mod(period: usize) -> (Vec<u8>, Vec<u8>) {
    let len = DSD64 as usize / 8 * 2;
    let left: Vec<u8> = (0..len)
        .map(|i| match (i % period) as u8 {
            0x69 => 0x6A,
            b => b,
        })
        .collect();
    let right: Vec<u8> = left
        .iter()
        .map(|b| match !b {
            0x69 => 0x6B,
            b => b,
        })
        .collect();
    (left, right)
}

fn request(n: u64, path: PathBuf, format: Option<AudioFormat>) -> SourceRequest {
    SourceRequest {
        entry: EntryId(n),
        track: TrackId(n),
        path,
        from_secs: 0.0,
        format,
    }
}

/// One rendered stereo frame as the 24-bit words the device receives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Frame {
    marker: u8,
    left: [u8; 2],
    right: [u8; 2],
}

const IDLE: [u8; 2] = [0x69, 0x69];

impl Frame {
    fn idle(&self) -> bool {
        self.left == IDLE && self.right == IDLE
    }
}

/// A rendered frame as 24-bit words: `None` when its two markers differ
/// (it cannot be a DoP frame).
fn as_frame([l, r]: [f32; 2]) -> Option<Frame> {
    let mut bytes = [0u8; 8];
    write_samples(SampleFormat::I24, &[l, r], &mut bytes);
    let w = |b: &[u8]| u32::from_le_bytes(b.try_into().unwrap()) >> 8;
    let (l, r) = (w(&bytes[0..4]), w(&bytes[4..8]));
    (l >> 16 == r >> 16).then_some(Frame {
        marker: (l >> 16) as u8,
        left: [(l >> 8) as u8, l as u8],
        right: [(r >> 8) as u8, r as u8],
    })
}

/// Every frame as DoP 24-bit words, asserting one marker per frame.
fn dop_frames(raw: &[[f32; 2]]) -> Vec<Frame> {
    raw.iter()
        .map(|f| as_frame(*f).expect("one marker per frame"))
        .collect()
}

/// A valid idle DoP frame (DoP marker, idle bytes on both channels).
fn idle_dop(raw: [f32; 2]) -> bool {
    as_frame(raw).is_some_and(|f| matches!(f.marker, 0x05 | 0xFA) && f.idle())
}

impl Rig {
    fn act(&mut self, action: EngineAction) {
        self.engine.execute(action, self.clock);
    }

    fn start(&mut self, player: PlayerId, request: SourceRequest) {
        self.act(EngineAction::StartCurrent { player, request });
        self.settle();
    }

    /// As in `bit_perfect.rs` (waits for the decode worker, never for audio),
    /// keeping what `tick` returns. DSD decoding is slow in debug builds
    /// with every test of the file running at once: the deadline is long.
    fn settle(&mut self) {
        let deadline = Instant::now() + Duration::from_secs(60);
        while self.engine.unsettled_sources() > 0 && Instant::now() < deadline {
            let events = self.engine.tick(self.clock);
            self.seen.extend(events);
            std::thread::sleep(Duration::from_millis(1));
        }
        let events = self.engine.tick(self.clock);
        self.seen.extend(events);
    }

    fn rate(&self) -> u32 {
        self.dac.config().unwrap().sample_rate
    }

    /// Renders `frames` frames in blocks, ticking after each; stereo frames.
    fn run_raw(&mut self, frames: usize) -> Vec<[f32; 2]> {
        let mut out = Vec::new();
        for _ in 0..frames.div_ceil(BLOCK) {
            let rate = self.rate();
            let samples = self.dac.render(BLOCK).unwrap();
            out.extend(samples.chunks(2).map(|f| [f[0], f[1]]));
            self.clock += Duration::from_secs_f64(BLOCK as f64 / f64::from(rate));
            let events = self.engine.tick(self.clock);
            self.seen.extend(events);
        }
        out
    }

    /// Renders `frames` frames and returns them decoded as DoP 24-bit words.
    fn run_dop(&mut self, frames: usize) -> Vec<Frame> {
        let raw = self.run_raw(frames);
        dop_frames(&raw)
    }

    fn events(&mut self) -> Vec<EngineEvent> {
        std::mem::take(&mut self.seen)
    }
}

fn assert_valid_dop(frames: &[Frame]) {
    for pair in frames.windows(2) {
        assert!(matches!(pair[0].marker, 0x05 | 0xFA), "{:?}", pair[0]);
        assert_ne!(pair[0].marker, pair[1].marker, "markers alternate");
    }
}

/// Asserts that `frames[from..]`, up to the first idle frame, are the
/// words of `left`/`right` from word `word` on; returns how many matched.
fn assert_words(frames: &[Frame], from: usize, left: &[u8], right: &[u8], word: usize) -> usize {
    let mut k = 0;
    while let Some(f) = frames.get(from + k)
        && !f.idle()
    {
        let w = word + k;
        assert_eq!(f.left, [left[2 * w], left[2 * w + 1]], "word {w}");
        assert_eq!(f.right, [right[2 * w], right[2 * w + 1]], "word {w}");
        k += 1;
    }
    k
}

fn first_data(frames: &[Frame]) -> usize {
    frames.iter().position(|f| !f.idle()).expect("DSD data")
}

/// Whether the frames are a PCM stream rather than DoP: some marker is not
/// a DoP marker, or two neighbours do not alternate.
fn not_dop(raw: &[[f32; 2]]) -> bool {
    raw.windows(2)
        .any(|w| match (as_frame(w[0]), as_frame(w[1])) {
            (Some(a), Some(b)) => !matches!(a.marker, 0x05 | 0xFA) || a.marker == b.marker,
            _ => true,
        })
}

/// The index of the first frame after `from` that is not an idle DoP frame.
fn end_of_idle(raw: &[[f32; 2]], from: usize) -> usize {
    raw[from..]
        .iter()
        .position(|f| !idle_dop(*f))
        .map_or(raw.len(), |p| from + p)
}

#[test]
fn a_dsd_track_goes_out_as_dop_byte_for_byte_after_the_silence() {
    let mut r = rig(DsdOutput::Dop, DsdMix::ConvertToPcm, SampleFormat::I24);
    let (left, right) = pattern();
    let path = dsf_file(r.dir.path(), "a.dsf", &left, &right);
    r.start(P, request(1, path, dsd64()));
    assert_eq!(r.rate(), WORD_RATE);
    assert_eq!(r.dac.config().unwrap().dsd, Some(DsdStream::Dop));
    let frames = r.run_dop(WORD_RATE as usize / 4);
    assert_valid_dop(&frames);
    let first = first_data(&frames);
    assert!(
        first >= SILENCE_FRAMES,
        "at least the silence time first: {first}"
    );
    assert!(frames[..first].iter().all(Frame::idle));
    for (k, f) in frames[first..].iter().enumerate() {
        assert_eq!(f.left, [left[2 * k], left[2 * k + 1]], "frame {k}");
        assert_eq!(f.right, [right[2 * k], right[2 * k + 1]], "frame {k}");
    }
    assert!(r.engine.telemetry(P).dsd);
    assert!(!r.engine.telemetry(P).bit_perfect);
    assert!(r.events().contains(&EngineEvent::DsdStarted {
        player: P,
        entry: EntryId(1),
        hold_others: false
    }));
}

#[test]
fn the_meter_reads_the_pcm_conversion_not_the_words() {
    // All-idle bytes: the words read -1.7 dBFS as samples, the conversion is silence.
    let mut r = rig(DsdOutput::Dop, DsdMix::ConvertToPcm, SampleFormat::I24);
    let idle = vec![0x69u8; DSD64 as usize / 8];
    let path = dsf_file(r.dir.path(), "idle.dsf", &idle, &idle);
    r.start(P, request(1, path, dsd64()));
    assert!(r.engine.telemetry(P).dsd);
    r.run_dop(WORD_RATE as usize / 4);
    let input = r.engine.take_meter_input(P);
    assert!(input.frames > 0);
    assert!(input.peak.iter().all(|p| *p < 1e-3), "{:?}", input.peak);
}

#[test]
fn the_end_sends_silence_then_leaves_dsd_mode() {
    let mut r = rig(DsdOutput::Dop, DsdMix::ConvertToPcm, SampleFormat::I24);
    let (left, right) = pattern();
    let len = DSD64 as usize / 8 / 10; // 0.1 s
    let path = dsf_file(r.dir.path(), "short.dsf", &left[..len], &right[..len]);
    r.start(P, request(1, path, dsd64()));
    let raw = r.run_raw(WORD_RATE as usize / 2);
    // The DoP part: silence, the file, silence.
    let pcm_at = raw
        .iter()
        .position(|f| as_frame(*f).is_none_or(|f| f.marker == 0))
        .expect("the stream leaves DSD mode");
    let frames = dop_frames(&raw[..pcm_at]);
    assert_valid_dop(&frames);
    let first = first_data(&frames);
    assert!(first >= SILENCE_FRAMES);
    let words = assert_words(&frames, first, &left, &right, 0);
    assert_eq!(words, len / 2, "the whole file");
    let after = first + words;
    assert!(frames[after..].iter().all(Frame::idle));
    assert!(
        pcm_at - after >= SILENCE_FRAMES,
        "at least the silence time after the end: {}",
        pcm_at - after
    );
    // Then plain PCM: the stream is idle and silent.
    assert!(raw[pcm_at..].iter().all(|f| *f == [0.0, 0.0]));
    let events = r.events();
    assert!(events.contains(&EngineEvent::DsdEnded {
        player: P,
        entry: EntryId(1)
    }));
    assert!(events.contains(&EngineEvent::ReachedEnd {
        player: P,
        entry: EntryId(1)
    }));
    assert!(!r.engine.telemetry(P).dsd);
}

#[test]
fn pause_and_seek_keep_every_frame_valid_dop() {
    let mut r = rig(DsdOutput::Dop, DsdMix::ConvertToPcm, SampleFormat::I24);
    let (left, right) = pattern();
    let path = dsf_file(r.dir.path(), "a.dsf", &left, &right);
    r.start(P, request(1, path, dsd64()));
    let mut all = Vec::new();

    let before = r.run_dop(WORD_RATE as usize / 20);
    let first = first_data(&before);
    let played = assert_words(&before, first, &left, &right, 0);
    assert_eq!(first + played, before.len(), "no gap before the pause");
    all.extend(before);

    r.act(EngineAction::Pause { player: P });
    let paused = r.run_dop(WORD_RATE as usize / 20);
    assert!(paused.iter().all(Frame::idle), "a pause holds at once");
    all.extend(paused);

    r.act(EngineAction::Resume { player: P });
    let resumed = r.run_dop(WORD_RATE as usize / 20);
    assert!(!resumed[0].idle(), "a resume continues at once");
    let more = assert_words(&resumed, 0, &left, &right, played);
    assert_eq!(more, resumed.len(), "it continues from where it paused");
    all.extend(resumed);

    r.act(EngineAction::Seek {
        player: P,
        secs: 1.0,
    });
    r.settle();
    let seeked = r.run_dop(WORD_RATE as usize / 10);
    let at = first_data(&seeked);
    let target = (DSD64 as usize / 8) / 2; // byte round(1.0 × DSD64 / 8), as a word
    let matched = assert_words(&seeked, at, &left, &right, target);
    assert_eq!(at + matched, seeked.len(), "the seek target onwards");
    all.extend(seeked);

    assert_valid_dop(&all);
    assert!(r.engine.telemetry(P).dsd);
}

#[test]
fn hold_others_mutes_a_second_player_on_the_same_device() {
    let mut r = rig(DsdOutput::Dop, DsdMix::HoldOthers, SampleFormat::I24);
    let (left, right) = pattern();
    let path = dsf_file(r.dir.path(), "a.dsf", &left, &right);
    r.start(P, request(1, path, dsd64()));
    let mut all = r.run_dop(WORD_RATE as usize / 20);

    let wav = indexed_wav(r.dir.path(), "q.wav", WORD_RATE, 2, WORD_RATE as usize * 2);
    r.start(Q, request(2, wav, pcm16(WORD_RATE)));
    let q_before = r.engine.telemetry(Q).position_secs.unwrap();
    all.extend(r.run_dop(WORD_RATE as usize / 10));

    assert_valid_dop(&all);
    let first = first_data(&all);
    let words = assert_words(&all, first, &left, &right, 0);
    assert_eq!(first + words, all.len(), "the DSD stream is untouched");
    let q_after = r.engine.telemetry(Q).position_secs.unwrap();
    assert!(
        q_after > q_before + 0.05,
        "the muted player keeps time: {q_before} -> {q_after}"
    );
    assert!(r.engine.telemetry(P).dsd);
    assert!(r.events().contains(&EngineEvent::DsdStarted {
        player: P,
        entry: EntryId(1),
        hold_others: true
    }));
}

#[test]
fn convert_to_pcm_switches_after_the_silence_when_another_source_starts() {
    let mut r = rig(DsdOutput::Dop, DsdMix::ConvertToPcm, SampleFormat::I24);
    let (left, right) = pattern();
    let path = dsf_file(r.dir.path(), "a.dsf", &left, &right);
    r.start(P, request(1, path, dsd64()));
    let before = r.run_dop(WORD_RATE as usize / 20);
    let first = first_data(&before);
    assert_eq!(
        first + assert_words(&before, first, &left, &right, 0),
        before.len()
    );
    let p_before = r.engine.telemetry(P).position_secs.unwrap();

    let wav = indexed_wav(r.dir.path(), "q.wav", WORD_RATE, 2, WORD_RATE as usize * 2);
    r.start(Q, request(2, wav, pcm16(WORD_RATE)));
    let rendered = WORD_RATE as usize / 4;
    let after = r.run_raw(rendered);

    // Any DSD words left, then the silence, then PCM.
    let words = after.iter().take_while(|f| !idle_dop(**f)).count();
    let pcm_at = end_of_idle(&after, words);
    assert!(
        pcm_at - words >= SILENCE_FRAMES,
        "at least the silence time: {}",
        pcm_at - words
    );
    let mut dop = before.clone();
    dop.extend(dop_frames(&after[..pcm_at]));
    assert_valid_dop(&dop);
    assert!(not_dop(&after[pcm_at..]), "PCM after the switch");
    assert!(!r.engine.telemetry(P).dsd);
    assert!(r.events().contains(&EngineEvent::DsdEnded {
        player: P,
        entry: EntryId(1)
    }));
    // P held during the switch and went on (as PCM) from there.
    let p_after = r.engine.telemetry(P).position_secs.unwrap();
    let expected = p_before
        + (rendered.div_ceil(BLOCK) * BLOCK - (pcm_at - words)) as f64 / f64::from(WORD_RATE);
    assert!(
        (p_after - expected).abs() < 0.01,
        "{p_before} -> {p_after}, expected about {expected}"
    );
}

#[test]
fn leave_dsd_switches_to_pcm() {
    let mut r = rig(DsdOutput::Dop, DsdMix::ConvertToPcm, SampleFormat::I24);
    let (left, right) = pattern();
    let path = dsf_file(r.dir.path(), "a.dsf", &left, &right);
    r.start(P, request(1, path, dsd64()));
    let before = r.run_dop(WORD_RATE as usize / 20);
    assert!(r.engine.telemetry(P).dsd);
    r.act(EngineAction::LeaveDsd { player: P });
    let after = r.run_raw(WORD_RATE as usize / 4);
    let words = after.iter().take_while(|f| !idle_dop(**f)).count();
    let pcm_at = end_of_idle(&after, words);
    assert!(pcm_at - words >= SILENCE_FRAMES);
    let mut dop = before;
    dop.extend(dop_frames(&after[..pcm_at]));
    assert_valid_dop(&dop);
    assert!(not_dop(&after[pcm_at..]), "P goes on as PCM");
    assert!(!r.engine.telemetry(P).dsd);
    assert!(r.events().contains(&EngineEvent::DsdEnded {
        player: P,
        entry: EntryId(1)
    }));
}

#[test]
fn a_busy_device_plays_dsd_converted() {
    let mut r = rig(DsdOutput::Dop, DsdMix::ConvertToPcm, SampleFormat::I24);
    let wav = indexed_wav(r.dir.path(), "q.wav", 44_100, 2, 44_100 * 3);
    r.start(Q, request(2, wav, pcm16(44_100)));
    r.run_raw(BLOCK * 4);
    assert_eq!(r.rate(), 44_100);
    let (left, right) = pattern();
    let path = dsf_file(r.dir.path(), "a.dsf", &left, &right);
    r.start(P, request(1, path, dsd64()));
    r.engine.take_meter_input(P);
    r.run_raw(44_100 / 4);
    assert_eq!(r.rate(), 44_100, "the rate stays");
    assert_eq!(r.dac.config().unwrap().dsd, None);
    assert!(!r.engine.telemetry(P).dsd);
    assert!(r.engine.telemetry(P).position_secs.unwrap() > 0.1);
    let input = r.engine.take_meter_input(P);
    assert!(
        input.peak.iter().any(|p| *p > 1e-3),
        "P is audible: {:?}",
        input.peak
    );
    assert!(
        !r.events()
            .iter()
            .any(|e| matches!(e, EngineEvent::DsdStarted { .. }))
    );
}

#[test]
fn a_refused_word_rate_plays_converted_and_is_not_asked_again() {
    let mut r = rig(DsdOutput::Dop, DsdMix::ConvertToPcm, SampleFormat::I24);
    r.dac.refuse_rate(WORD_RATE);
    let (left, right) = pattern();
    let path = dsf_file(r.dir.path(), "a.dsf", &left, &right);
    r.start(P, request(1, path.clone(), dsd64()));
    assert_eq!(
        r.rate(),
        PCM_RATE,
        "the PCM conversion's rate, as before O25"
    );
    assert_eq!(r.dac.config().unwrap().dsd, None);
    r.act(EngineAction::StopNow { player: P });
    // Let the stop finish, so the device is idle for the next start.
    r.run_raw(BLOCK * 4);
    let attempts = r.dac.open_attempts();
    r.start(P, request(2, path, dsd64()));
    assert_eq!(
        r.dac.open_attempts(),
        attempts,
        "the refused pair is not asked again"
    );
    assert_eq!(r.rate(), PCM_RATE);
}

#[test]
fn a_sixteen_bit_device_plays_dsd_converted() {
    // The Offline open refuses DoP on a 16-bit device.
    let mut r = rig(DsdOutput::Dop, DsdMix::ConvertToPcm, SampleFormat::I16);
    let (left, right) = pattern();
    let path = dsf_file(r.dir.path(), "a.dsf", &left, &right);
    r.start(P, request(1, path, dsd64()));
    assert_eq!(r.rate(), PCM_RATE);
    assert_eq!(r.dac.config().unwrap().dsd, None);
    assert!(!r.engine.telemetry(P).dsd);
}

#[test]
fn the_engine_checks_the_dop_format_whatever_the_backend_accepts() {
    // A backend that opens DoP on 16 bits: the engine refuses it itself.
    let mut r = rig_with(
        DsdOutput::Dop,
        DsdMix::ConvertToPcm,
        SampleFormat::I16,
        |dac| dac.set_dop_any_format(true),
    );
    let (left, right) = pattern();
    let path = dsf_file(r.dir.path(), "a.dsf", &left, &right);
    r.start(P, request(1, path, dsd64()));
    assert_eq!(r.rate(), PCM_RATE);
    assert_eq!(r.dac.config().unwrap().dsd, None);
    assert!(!r.engine.telemetry(P).dsd);
    let raw = r.run_raw(BLOCK * 8);
    assert!(raw.iter().all(|f| !idle_dop(*f)), "never a DoP frame");
}

#[test]
fn native_on_a_device_without_it_plays_converted() {
    let mut r = rig(DsdOutput::Native, DsdMix::ConvertToPcm, SampleFormat::I24);
    let (left, right) = pattern();
    let path = dsf_file(r.dir.path(), "a.dsf", &left, &right);
    r.start(P, request(1, path, dsd64()));
    assert_eq!(r.rate(), PCM_RATE);
    assert_eq!(r.dac.config().unwrap().dsd, None);
    assert!(!r.engine.telemetry(P).dsd);
}

#[test]
fn native_dsd_goes_out_as_the_raw_words() {
    let mut r = rig_with(
        DsdOutput::Native,
        DsdMix::ConvertToPcm,
        SampleFormat::I24,
        |dac| dac.set_native_dsd(true),
    );
    let (left, right) = pattern();
    let path = dsf_file(r.dir.path(), "a.dsf", &left, &right);
    r.start(P, request(1, path, dsd64()));
    assert_eq!(r.dac.config().unwrap().dsd, Some(DsdStream::Native));
    assert_eq!(r.rate(), WORD_RATE);
    let raw = r.run_raw(WORD_RATE as usize / 4);
    let first = raw
        .iter()
        .position(|f| *f != [silence_sample(), silence_sample()])
        .unwrap();
    assert!(first >= SILENCE_FRAMES);
    for (k, f) in raw[first..].iter().enumerate() {
        assert_eq!(sample_to_word(f[0]), [left[2 * k], left[2 * k + 1]], "{k}");
        assert_eq!(
            sample_to_word(f[1]),
            [right[2 * k], right[2 * k + 1]],
            "{k}"
        );
    }
    assert!(r.engine.telemetry(P).dsd);
}

#[test]
fn native_dsd_reopens_as_pcm_after_the_end() {
    let mut r = rig_with(
        DsdOutput::Native,
        DsdMix::ConvertToPcm,
        SampleFormat::I24,
        |dac| dac.set_native_dsd(true),
    );
    let (left, right) = pattern();
    let len = DSD64 as usize / 8 / 10;
    let path = dsf_file(r.dir.path(), "short.dsf", &left[..len], &right[..len]);
    r.start(P, request(1, path, dsd64()));
    assert_eq!(r.dac.config().unwrap().dsd, Some(DsdStream::Native));
    r.run_raw(WORD_RATE as usize / 2);
    let config = r.dac.config().unwrap();
    assert_eq!(config.dsd, None, "reopened as PCM");
    assert_eq!(config.sample_rate, WORD_RATE, "at the same rate");
    let raw = r.run_raw(BLOCK * 2);
    assert!(raw.iter().all(|f| *f == [0.0, 0.0]), "PCM silence");
    assert!(r.events().contains(&EngineEvent::DsdEnded {
        player: P,
        entry: EntryId(1)
    }));
}

#[test]
fn a_multichannel_dsd_file_plays_converted() {
    let mut r = rig(DsdOutput::Dop, DsdMix::ConvertToPcm, SampleFormat::I24);
    let (left, right) = pattern();
    let path = dsf_file(r.dir.path(), "a.dsf", &left, &right);
    let format = dsd64().map(|f| AudioFormat { channels: 6, ..f });
    r.start(P, request(1, path, format));
    assert_eq!(r.dac.config().unwrap().dsd, None);
    assert!(!r.engine.telemetry(P).dsd);
}

#[test]
fn a_volume_below_unity_plays_converted() {
    let mut r = rig(DsdOutput::Dop, DsdMix::ConvertToPcm, SampleFormat::I24);
    r.act(EngineAction::SetVolume {
        player: P,
        volume: 0.5,
    });
    let (left, right) = pattern();
    let path = dsf_file(r.dir.path(), "a.dsf", &left, &right);
    r.start(P, request(1, path, dsd64()));
    assert_eq!(r.rate(), PCM_RATE);
    assert_eq!(r.dac.config().unwrap().dsd, None);
    assert!(!r.engine.telemetry(P).dsd);
}

#[test]
fn a_volume_lowered_before_the_dsd_slot_starts_switches_to_pcm() {
    let mut r = rig(DsdOutput::Dop, DsdMix::ConvertToPcm, SampleFormat::I24);
    let (left, right) = pattern();
    let path = dsf_file(r.dir.path(), "a.dsf", &left, &right);
    r.act(EngineAction::StartCurrent {
        player: P,
        request: request(1, path, dsd64()),
    });
    // Decided at unity; the fader moves before the slot starts.
    assert_eq!(r.rate(), WORD_RATE);
    r.act(EngineAction::SetVolume {
        player: P,
        volume: 0.5,
    });
    r.settle();
    let raw = r.run_raw(WORD_RATE as usize / 4);
    assert!(not_dop(&raw[raw.len() - BLOCK..]), "PCM in the end");
    assert!(!r.engine.telemetry(P).dsd);
    assert!(
        !r.events()
            .iter()
            .any(|e| matches!(e, EngineEvent::DsdStarted { .. }))
    );
}

#[test]
fn pcm_mode_plays_converted_as_before() {
    let mut r = rig(DsdOutput::Pcm, DsdMix::ConvertToPcm, SampleFormat::I24);
    let (left, right) = pattern();
    let path = dsf_file(r.dir.path(), "a.dsf", &left, &right);
    r.start(P, request(1, path, dsd64()));
    r.run_raw(BLOCK * 4);
    assert_eq!(r.rate(), PCM_RATE);
    assert_eq!(r.dac.config().unwrap().dsd, None);
    assert!(
        !r.events()
            .iter()
            .any(|e| matches!(e, EngineEvent::DsdStarted { .. }))
    );
}

#[test]
fn a_device_lost_during_dop_comes_back_as_dop() {
    let mut r = rig(DsdOutput::Dop, DsdMix::ConvertToPcm, SampleFormat::I24);
    let (left, right) = pattern();
    let path = dsf_file(r.dir.path(), "a.dsf", &left, &right);
    r.start(P, request(1, path, dsd64()));
    r.run_dop(BLOCK * 8);
    r.dac.unplug();
    r.clock += Duration::from_millis(50);
    r.engine.tick(r.clock);
    assert!(r.dac.config().is_none());
    r.dac.replug();
    r.clock += Duration::from_secs(10);
    r.engine.tick(r.clock);
    assert_eq!(r.dac.config().unwrap().dsd, Some(DsdStream::Dop));
    assert_eq!(r.rate(), WORD_RATE);
    let frames = r.run_dop(BLOCK * 8);
    assert_valid_dop(&frames);
    assert!(frames.iter().any(|f| !f.idle()), "the track goes on");
}

#[test]
fn a_dsd_track_loaded_paused_starts_direct_on_resume() {
    let mut r = rig(DsdOutput::Dop, DsdMix::ConvertToPcm, SampleFormat::I24);
    let (left, right) = pattern();
    let path = dsf_file(r.dir.path(), "a.dsf", &left, &right);
    let rate_before = r.rate();
    r.act(EngineAction::LoadPaused {
        player: P,
        request: SourceRequest {
            from_secs: 0.5,
            ..request(1, path, dsd64())
        },
    });
    r.settle();
    let raw = r.run_raw(BLOCK * 4);
    assert!(raw.iter().all(|f| *f == [0.0, 0.0]), "nothing on air");
    assert_eq!(r.rate(), rate_before, "no stream change yet");
    r.act(EngineAction::Resume { player: P });
    r.settle();
    assert_eq!(r.rate(), WORD_RATE);
    let frames = r.run_dop(WORD_RATE as usize / 4);
    assert_valid_dop(&frames);
    let first = first_data(&frames);
    assert!(first >= SILENCE_FRAMES);
    let word = (DSD64 as usize / 8 / 2) / 2; // byte round(0.5 × DSD64 / 8), as a word
    let matched = assert_words(&frames, first, &left, &right, word);
    assert_eq!(first + matched, frames.len());
    assert!(r.engine.telemetry(P).dsd);
}

#[test]
fn start_current_on_a_held_dsd_player_ends_the_stream_first() {
    let mut r = rig(DsdOutput::Dop, DsdMix::HoldOthers, SampleFormat::I24);
    let (left, right) = pattern();
    let (left2, right2) = pattern_mod(241);
    let a = dsf_file(r.dir.path(), "a.dsf", &left, &right);
    let b = dsf_file(r.dir.path(), "b.dsf", &left2, &right2);
    r.start(P, request(1, a, dsd64()));
    let before = r.run_dop(WORD_RATE as usize / 20);
    let first = first_data(&before);
    let played = assert_words(&before, first, &left, &right, 0);
    assert_eq!(first + played, before.len());

    r.start(P, request(2, b, dsd64()));
    let after = r.run_dop(WORD_RATE as usize / 10);
    let mut all = before;
    all.extend(after.iter().copied());
    assert_valid_dop(&all);
    // The first stream is cut, and the second goes out unchanged.
    let next = first_data(&after);
    let matched = assert_words(&after, next, &left2, &right2, 0);
    assert_eq!(next + matched, after.len());
    assert!(r.engine.telemetry(P).dsd);
    let events = r.events();
    assert!(events.contains(&EngineEvent::DsdEnded {
        player: P,
        entry: EntryId(1)
    }));
    assert!(events.contains(&EngineEvent::DsdStarted {
        player: P,
        entry: EntryId(2),
        hold_others: true
    }));
}

#[test]
fn a_repeat_pass_reports_the_dsd_end_before_the_transition() {
    // ConvertToPcm: the next pass is a PCM preload, so the DSD stream
    // switches to PCM first and the model hears of it before the pass.
    let mut r = rig(DsdOutput::Dop, DsdMix::ConvertToPcm, SampleFormat::I24);
    let (left, right) = pattern();
    let path = dsf_file(r.dir.path(), "a.dsf", &left, &right);
    r.start(P, request(1, path.clone(), dsd64()));
    r.run_dop(WORD_RATE as usize / 20);
    r.act(EngineAction::Preload {
        player: P,
        request: Some(request(1, path, dsd64())),
    });
    r.settle();
    r.act(EngineAction::Schedule {
        player: P,
        plan: Some(TransitionPlan::StartNextAt {
            at_secs: 0.1,
            fade_current_until_secs: None,
        }),
    });
    r.run_raw(WORD_RATE as usize / 4);
    let events = r.events();
    let ended = events
        .iter()
        .position(|e| {
            *e == EngineEvent::DsdEnded {
                player: P,
                entry: EntryId(1),
            }
        })
        .expect("DsdEnded");
    let started = events
        .iter()
        .position(|e| {
            *e == EngineEvent::TransitionStarted {
                player: P,
                entry: EntryId(1),
            }
        })
        .expect("TransitionStarted");
    assert!(ended < started, "{events:?}");
    assert!(!r.engine.telemetry(P).dsd);
}

/// Index of the first sample of `pcm` (left channel) louder than `level`.
fn first_loud(pcm: &[f32], level: f32) -> usize {
    pcm.chunks(2).position(|f| f[0].abs() > level).unwrap()
}

#[test]
fn after_a_seek_the_pcm_conversion_stays_aligned_with_the_words() {
    // One second of idle bytes, then the pattern: the PCM conversion turns
    // loud a fixed number of frames after the words change, from the start
    // of the file or from a seek.
    let dir = tempfile::tempdir().unwrap();
    let (loud_l, loud_r) = pattern();
    let second = DSD64 as usize / 8;
    let mut left = vec![0x69u8; second];
    let mut right = vec![0x69u8; second];
    left.extend(&loud_l[..second]);
    right.extend(&loud_r[..second]);
    let path = dsf_file(dir.path(), "step.dsf", &left, &right);
    let opener = dsd_file_opener();
    let lag_from = |from_secs: f64| {
        let mut source = opener(&path, from_secs, WORD_RATE).unwrap();
        let (mut pcm, mut words) = (Vec::new(), Vec::new());
        while pcm.len() < WORD_RATE as usize * 2 * 2
            && source.next_pair(&mut pcm, &mut words).unwrap()
        {}
        let change = words
            .chunks(2)
            .position(|f| f[0] != silence_sample())
            .unwrap();
        let expected = ((1.0 - from_secs) * f64::from(WORD_RATE)).round() as usize;
        assert_eq!(change, expected, "the words change on their frame");
        first_loud(&pcm, 0.01) as i64 - change as i64
    };
    let from_start = lag_from(0.0);
    let from_seek = lag_from(0.5);
    assert_eq!(from_start, from_seek, "no extra offset after a seek");
}

// ---- Fix round 1 ----

/// `count` frames from `from` carry the indexed WAV from its first frame
/// (`support::indexed_wav`): the next source's head was not lost.
fn assert_head(raw: &[[f32; 2]], from: usize, count: usize) {
    for k in 0..count {
        assert_eq!(
            support::index_of(raw[from + k][0]),
            k as i64,
            "frame {k} of the next source (at {from})"
        );
    }
}

fn wav_request(r: &Rig, n: u64) -> SourceRequest {
    let wav = indexed_wav(
        r.dir.path(),
        &format!("w{n}.wav"),
        WORD_RATE,
        2,
        WORD_RATE as usize * 2,
    );
    request(n, wav, pcm16(WORD_RATE))
}

#[test]
fn a_device_back_without_exclusive_access_plays_pcm() {
    let mut r = rig(DsdOutput::Dop, DsdMix::HoldOthers, SampleFormat::I24);
    let (left, right) = pattern();
    let path = dsf_file(r.dir.path(), "a.dsf", &left, &right);
    r.start(P, request(1, path, dsd64()));
    r.run_dop(BLOCK * 8);
    assert!(r.engine.telemetry(P).dsd);
    r.dac.unplug();
    r.clock += Duration::from_millis(50);
    r.engine.tick(r.clock);
    r.dac.set_exclusive_capable(false);
    r.dac.replug();
    r.clock += Duration::from_secs(10);
    let events = r.engine.tick(r.clock);
    r.seen.extend(events);
    let config = r.dac.config().expect("the device is never left closed");
    assert_eq!(config.dsd, None);
    let raw = r.run_raw(BLOCK * 8);
    assert!(not_dop(&raw), "PCM");
    assert!(
        raw.iter().any(|f| f[0] != 0.0),
        "the track goes on converted"
    );
    assert!(!r.engine.telemetry(P).dsd);
    assert!(r.events().contains(&EngineEvent::DsdEnded {
        player: P,
        entry: EntryId(1)
    }));
}

#[test]
fn a_device_back_with_a_narrow_format_plays_pcm() {
    let mut r = rig(DsdOutput::Dop, DsdMix::ConvertToPcm, SampleFormat::I24);
    let (left, right) = pattern();
    let path = dsf_file(r.dir.path(), "a.dsf", &left, &right);
    r.start(P, request(1, path, dsd64()));
    r.run_dop(BLOCK * 8);
    r.dac.unplug();
    r.clock += Duration::from_millis(50);
    r.engine.tick(r.clock);
    // Another device behind the same name: 16 bits, and a backend that
    // opens DoP on it anyway.
    r.dac.set_sample_format(SampleFormat::I16);
    r.dac.set_dop_any_format(true);
    r.dac.replug();
    r.clock += Duration::from_secs(10);
    let events = r.engine.tick(r.clock);
    r.seen.extend(events);
    assert_eq!(r.dac.config().unwrap().dsd, None, "reopened as PCM");
    let raw = r.run_raw(BLOCK * 8);
    assert!(raw.iter().all(|f| !idle_dop(*f)), "never a DoP frame");
    assert!(!r.engine.telemetry(P).dsd);
    assert!(r.events().contains(&EngineEvent::DsdEnded {
        player: P,
        entry: EntryId(1)
    }));
}

#[test]
fn a_dsd_start_inside_a_pending_switch_plays_converted() {
    let mut r = rig(DsdOutput::Dop, DsdMix::ConvertToPcm, SampleFormat::I24);
    let (left, right) = pattern();
    let a = dsf_file(r.dir.path(), "a.dsf", &left, &right);
    r.start(P, request(1, a.clone(), dsd64()));
    r.act(EngineAction::Pause { player: P });
    r.run_raw(WORD_RATE as usize / 10);
    // Q's start switches the bus to PCM after the silence…
    let q = wav_request(&r, 3);
    r.start(Q, q);
    // …and before it applies, everything stops and a DSD track starts.
    r.act(EngineAction::StopNow { player: Q });
    r.act(EngineAction::StopNow { player: P });
    r.start(P, request(2, a, dsd64()));
    let raw = r.run_raw(WORD_RATE as usize / 4);
    let tail = &raw[raw.len() - BLOCK..];
    assert!(not_dop(tail), "the bus is PCM");
    assert!(!r.engine.telemetry(P).dsd, "and the engine says so");
    assert!(!r.events().contains(&EngineEvent::DsdStarted {
        player: P,
        entry: EntryId(2),
        hold_others: false
    }));
}

#[test]
fn a_scheduled_transition_waits_for_the_switch_and_keeps_the_next_head() {
    let mut r = rig(DsdOutput::Dop, DsdMix::ConvertToPcm, SampleFormat::I24);
    let (left, right) = pattern();
    let a = dsf_file(r.dir.path(), "a.dsf", &left, &right);
    r.start(P, request(1, a, dsd64()));
    let next = wav_request(&r, 2);
    r.act(EngineAction::Preload {
        player: P,
        request: Some(next),
    });
    r.settle();
    r.act(EngineAction::Schedule {
        player: P,
        plan: Some(TransitionPlan::StartNextAt {
            at_secs: 0.3,
            fade_current_until_secs: None,
        }),
    });
    let raw = r.run_raw(WORD_RATE as usize * 6 / 10);
    let frames_dop: Vec<Option<Frame>> = raw.iter().map(|f| as_frame(*f)).collect();
    let first = frames_dop
        .iter()
        .position(|f| f.is_some_and(|f| !f.idle()))
        .unwrap();
    let data = frames_dop[first..]
        .iter()
        .take_while(|f| f.is_some_and(|f| !f.idle() && matches!(f.marker, 0x05 | 0xFA)))
        .count();
    let cue = (0.3 * f64::from(WORD_RATE)) as usize;
    assert!(
        data <= cue && data + BLOCK >= cue,
        "the current plays to its transition: {data} of {cue}"
    );
    let pcm_at = end_of_idle(&raw, first + data);
    assert!(pcm_at - (first + data) >= SILENCE_FRAMES);
    assert_head(&raw, pcm_at, 200);
    assert!(!r.engine.telemetry(P).dsd);
}

#[test]
fn a_dsd_track_ending_into_its_next_keeps_the_next_head() {
    let mut r = rig(DsdOutput::Dop, DsdMix::ConvertToPcm, SampleFormat::I24);
    let (left, right) = pattern();
    let len = DSD64 as usize / 8 / 10;
    let a = dsf_file(r.dir.path(), "short.dsf", &left[..len], &right[..len]);
    r.start(P, request(1, a, dsd64()));
    let next = wav_request(&r, 2);
    r.act(EngineAction::Preload {
        player: P,
        request: Some(next),
    });
    r.settle();
    r.act(EngineAction::Schedule {
        player: P,
        plan: Some(TransitionPlan::StartNextAt {
            at_secs: fp_model::SOURCE_END,
            fade_current_until_secs: None,
        }),
    });
    let raw = r.run_raw(WORD_RATE as usize / 2);
    let dop: Vec<Frame> = raw
        .iter()
        .map_while(|f| as_frame(*f).filter(|f| matches!(f.marker, 0x05 | 0xFA)))
        .collect();
    let first = first_data(&dop);
    let words = assert_words(&dop, first, &left, &right, 0);
    assert_eq!(words, len / 2);
    let pcm_at = end_of_idle(&raw, first + words);
    assert!(pcm_at - (first + words) >= SILENCE_FRAMES);
    assert_head(&raw, pcm_at, 200);
}

#[test]
fn a_pcm_start_during_the_dsd_silence_keeps_its_head() {
    let mut r = rig(DsdOutput::Dop, DsdMix::ConvertToPcm, SampleFormat::I24);
    let (left, right) = pattern();
    let len = DSD64 as usize / 8 / 10;
    let a = dsf_file(r.dir.path(), "short.dsf", &left[..len], &right[..len]);
    r.start(P, request(1, a, dsd64()));
    let mut raw = Vec::new();
    while r.engine.telemetry(P).position_secs.is_some() {
        raw.extend(r.run_raw(BLOCK));
        assert!(raw.len() < WORD_RATE as usize, "the track ends");
    }
    // In the silence after the end: Q starts.
    let q = wav_request(&r, 3);
    r.start(Q, q);
    raw.extend(r.run_raw(WORD_RATE as usize / 4));
    let dop: Vec<Frame> = raw
        .iter()
        .map_while(|f| as_frame(*f).filter(|f| matches!(f.marker, 0x05 | 0xFA)))
        .collect();
    let first = first_data(&dop);
    let words = assert_words(&dop, first, &left, &right, 0);
    let pcm_at = end_of_idle(&raw, first + words);
    assert!(pcm_at - (first + words) >= SILENCE_FRAMES);
    assert_head(&raw, pcm_at, 200);
}

#[test]
fn a_cart_over_a_dsd_stream_switches_it_to_pcm() {
    let mut r = rig(DsdOutput::Dop, DsdMix::ConvertToPcm, SampleFormat::I24);
    let (left, right) = pattern();
    let a = dsf_file(r.dir.path(), "a.dsf", &left, &right);
    r.start(P, request(1, a, dsd64()));
    r.act(EngineAction::Pause { player: P });
    r.run_raw(BLOCK * 8);
    let jingle = indexed_wav(r.dir.path(), "jingle.wav", WORD_RATE, 2, WORD_RATE as usize);
    r.act(EngineAction::StartCart(fp_model::CartRequest {
        cart: fp_model::CartId(1),
        track: TrackId(9),
        path: jingle,
        from_secs: 0.0,
        until_secs: f64::INFINITY,
        looped: false,
        format: pcm16(WORD_RATE),
    }));
    r.settle();
    let raw = r.run_raw(WORD_RATE as usize / 4);
    let pcm_at = end_of_idle(&raw, 0);
    assert!(pcm_at >= SILENCE_FRAMES);
    assert_head(&raw, pcm_at, 200);
    assert!(!r.engine.telemetry(P).dsd);
}

#[test]
fn a_test_tone_over_a_dsd_stream_switches_it_to_pcm() {
    let mut r = rig(DsdOutput::Dop, DsdMix::ConvertToPcm, SampleFormat::I24);
    let (left, right) = pattern();
    let a = dsf_file(r.dir.path(), "a.dsf", &left, &right);
    r.start(P, request(1, a, dsd64()));
    r.act(EngineAction::Pause { player: P });
    r.run_raw(BLOCK * 8);
    let route = Route {
        backend: "offline".into(),
        device: "dac".into(),
        first_channel: 0,
    };
    r.engine
        .play_test_tone(&route, 1_000.0, 0.5, -20.0, r.clock);
    let raw = r.run_raw(WORD_RATE as usize / 4);
    let pcm_at = end_of_idle(&raw, 0);
    assert!(pcm_at >= SILENCE_FRAMES);
    assert!(
        raw[pcm_at..].iter().any(|f| f[0].abs() > 0.01),
        "the tone is heard"
    );
    assert!(!r.engine.telemetry(P).dsd);
}
