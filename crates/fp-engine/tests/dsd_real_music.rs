#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
//! Opt-in end-to-end checks of real DSD files (feedback 2 spec O25):
//! `cargo test --release -p fp-engine --test dsd_real_music -- --ignored --nocapture`.
//!
//! Every `.dsf` and `.dff` file of the real-music folder (`FAUSTE_TEST_MUSIC`,
//! else `test-music/`) is analysed, played through the engine on an Offline
//! device (never a real one) as DoP, native DSD and PCM, and compared with
//! the raw bytes read straight from the file. With no DSD file the tests
//! pass with a note. They never run in CI.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use fp_backends::dsd::{DsdStream, sample_to_word};
use fp_backends::exclusive::write_samples;
use fp_backends::{AudioBackend, OfflineBackend, OfflineDevice, SampleFormat};
use fp_decode::{DsdRawReader, FileDecoder};
use fp_engine::engine::{Engine, EngineSettings};
use fp_engine::worker::file_opener;
use fp_model::{
    AudioFormat, Config, DsdDevice, DsdMix, DsdOutput, EngineAction, EntryId, OutputDevice,
    PlayerId, PlayerRoutes, Route, SourceRequest, TrackId,
};

const P: PlayerId = PlayerId(1);
const BLOCK: usize = 480;
const DSD64: u32 = 2_822_400;
const SILENCE_MS: f64 = 10.0;
const IDLE: [u8; 2] = [0x69, 0x69];
/// Seconds of words that must match the file.
const MATCH_SECS: f64 = 2.0;
/// Seconds played in the DoP and native checks.
const PLAY_SECS: f64 = 3.0;

/// The DSD files of the real-music folder, sorted; empty (with a note) when
/// there is none.
fn dsd_files() -> Vec<PathBuf> {
    let root = std::env::var_os("FAUSTE_TEST_MUSIC").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-music"),
        PathBuf::from,
    );
    let mut out = Vec::new();
    let mut pending = vec![root.clone()];
    while let Some(d) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("dsf") || e.eq_ignore_ascii_case("dff"))
            {
                out.push(path);
            }
        }
    }
    out.sort();
    if out.is_empty() {
        eprintln!(
            "no DSD file in {} (see test-music/README.md); nothing to check",
            root.display()
        );
    }
    out
}

/// What the file says about itself, read like the analysis does.
struct Info {
    format: AudioFormat,
    dsd_rate: u32,
    duration: f64,
}

fn info(path: &Path) -> Info {
    let decoder = FileDecoder::open(path).unwrap();
    let dsd_rate = decoder.dsd_rate().expect("a DSD rate");
    let rate = decoder.sample_rate();
    let frames = decoder.frames_hint().expect("a length");
    Info {
        format: AudioFormat {
            sample_rate: rate,
            bits: None,
            channels: decoder.channels() as u32,
            dsd_rate: Some(dsd_rate),
        },
        dsd_rate,
        duration: frames as f64 / f64::from(rate),
    }
}

struct Rig {
    engine: Engine,
    dac: OfflineDevice,
    clock: Instant,
}

fn rig(mode: DsdOutput, native: bool) -> Rig {
    let backend = OfflineBackend::new();
    let dac = backend.add_device("dac", 2);
    dac.set_exclusive_capable(true);
    dac.set_sample_format(SampleFormat::I24);
    dac.set_native_dsd(native);
    let mut config = Config::default();
    config.outputs.backend = Some("offline".into());
    config.outputs.buffer_frames = BLOCK as u32;
    config.outputs.routes = vec![PlayerRoutes {
        player: P,
        main: Some(Route {
            backend: "offline".into(),
            device: "dac".into(),
            first_channel: 0,
        }),
        cue: None,
    }];
    config.outputs.bit_perfect = vec![OutputDevice {
        backend: "offline".into(),
        device: "dac".into(),
    }];
    config.outputs.dsd_output = vec![DsdDevice {
        backend: "offline".into(),
        device: "dac".into(),
        mode,
    }];
    config.outputs.dsd_mix = DsdMix::ConvertToPcm;
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
    Rig { engine, dac, clock }
}

impl Rig {
    /// Starts `path` and waits for the decode worker (never for audio).
    fn start(&mut self, path: &Path, format: AudioFormat) {
        let request = SourceRequest {
            entry: EntryId(1),
            track: TrackId(1),
            path: path.to_owned(),
            from_secs: 0.0,
            format: Some(format),
        };
        self.engine.execute(
            EngineAction::StartCurrent { player: P, request },
            self.clock,
        );
        let deadline = Instant::now() + Duration::from_secs(120);
        while self.engine.unsettled_sources() > 0 && Instant::now() < deadline {
            self.engine.tick(self.clock);
            std::thread::sleep(Duration::from_millis(1));
        }
        self.engine.tick(self.clock);
    }

    /// Renders `frames` stereo frames in blocks, ticking after each.
    fn run(&mut self, frames: usize) -> Vec<[f32; 2]> {
        let mut out = Vec::with_capacity(frames);
        for _ in 0..frames.div_ceil(BLOCK) {
            let rate = self.dac.config().unwrap().sample_rate;
            let samples = self.dac.render(BLOCK).unwrap();
            out.extend(samples.chunks(2).map(|f| [f[0], f[1]]));
            self.clock += Duration::from_secs_f64(BLOCK as f64 / f64::from(rate));
            self.engine.tick(self.clock);
            // The decode worker is a real thread: rendering faster than
            // real time would outrun it and underrun.
            std::thread::sleep(Duration::from_millis(2));
        }
        out
    }
}

/// The first `words` 16-bit words of each channel of the file, read with the
/// lowest-level reader.
fn raw_words(path: &Path, words: usize) -> [Vec<u8>; 2] {
    let mut reader = DsdRawReader::open(path).unwrap();
    assert_eq!(reader.channels(), 2, "a stereo file");
    let mut out = vec![Vec::new(), Vec::new()];
    while out[0].len() < words * 2 && reader.next_bytes(&mut out).unwrap() {}
    assert!(out[0].len() >= words * 2, "the file is long enough");
    let right = out.pop().unwrap();
    let left = out.pop().unwrap();
    [left, right]
}

fn word(bytes: &[u8], k: usize) -> [u8; 2] {
    [bytes[2 * k], bytes[2 * k + 1]]
}

/// The index of the first word where either channel is not the idle word.
fn first_sound_word(left: &[u8], right: &[u8]) -> usize {
    (0..left.len() / 2)
        .position(|k| word(left, k) != IDLE || word(right, k) != IDLE)
        .expect("a non-idle word in the read bytes")
}

/// A rendered frame as 24-bit words: marker and the two bytes per channel.
fn dop(frame: [f32; 2]) -> Option<(u8, [u8; 2], [u8; 2])> {
    let mut bytes = [0u8; 8];
    write_samples(SampleFormat::I24, &frame, &mut bytes);
    let w = |b: &[u8]| u32::from_le_bytes(b.try_into().unwrap()) >> 8;
    let (l, r) = (w(&bytes[0..4]), w(&bytes[4..8]));
    (l >> 16 == r >> 16).then_some((
        (l >> 16) as u8,
        [(l >> 8) as u8, l as u8],
        [(r >> 8) as u8, r as u8],
    ))
}

#[test]
#[ignore = "needs a real DSD file in the real-music corpus"]
fn real_dsd_files_are_analysed() {
    for path in dsd_files() {
        let i = info(&path);
        let k = (f64::from(i.dsd_rate) / f64::from(DSD64)).log2();
        eprintln!(
            "{}: DSD {} Hz ({:.0}x44.1k), {} channels, {:.1} s, PCM {} Hz",
            path.display(),
            i.dsd_rate,
            f64::from(i.dsd_rate) / 44_100.0,
            i.format.channels,
            i.duration,
            i.format.sample_rate
        );
        assert!(
            k >= 0.0 && (k - k.round()).abs() < 1e-9 && k <= 6.0,
            "{}: implausible DSD rate {}",
            path.display(),
            i.dsd_rate
        );
        assert!(i.duration > 0.0, "{}: no duration", path.display());
        assert_eq!(i.format.sample_rate, i.dsd_rate / 32);
    }
}

/// Plays `path` as DoP or native DSD and compares with the raw bytes.
fn check_words(path: &Path, native: bool) {
    let i = info(path);
    let word_rate = (i.dsd_rate / 16) as usize;
    let mut r = rig(
        if native {
            DsdOutput::Native
        } else {
            DsdOutput::Dop
        },
        native,
    );
    r.start(path, i.format);
    let config = r.dac.config().unwrap();
    assert_eq!(
        config.dsd,
        Some(if native {
            DsdStream::Native
        } else {
            DsdStream::Dop
        })
    );
    assert_eq!(config.sample_rate as usize, word_rate);
    let raw = r.run(word_rate * PLAY_SECS as usize + word_rate / 10);
    let need = (word_rate as f64 * (PLAY_SECS + 1.0)) as usize;
    let [left, right] = raw_words(path, need);
    let sound = first_sound_word(&left, &right);

    // The words of the stream: (left, right) per frame; `None` is idle.
    let mut previous = None;
    let mut stream: Vec<([u8; 2], [u8; 2])> = Vec::with_capacity(raw.len());
    for (n, f) in raw.iter().enumerate() {
        if native {
            stream.push((sample_to_word(f[0]), sample_to_word(f[1])));
        } else {
            let (marker, l, rr) = dop(*f).unwrap_or_else(|| panic!("frame {n}: two markers"));
            assert!(
                matches!(marker, 0x05 | 0xFA),
                "frame {n}: marker {marker:#x}"
            );
            assert_ne!(Some(marker), previous, "frame {n}: markers alternate");
            previous = Some(marker);
            stream.push((l, rr));
        }
    }
    let first = stream
        .iter()
        .position(|w| w.0 != IDLE || w.1 != IDLE)
        .expect("DSD data in the stream");
    let silence_frames = (SILENCE_MS / 1000.0 * word_rate as f64) as usize;
    // The file's own leading idle words are idle too: the stream has the
    // silence, then the file from its first word on, which starts at
    // `first - sound` at the latest.
    assert!(
        first + 1 >= silence_frames,
        "at least the silence time first: {first}"
    );
    let start = first - sound;
    let wanted = (word_rate as f64 * MATCH_SECS) as usize;
    let mut matched = 0;
    for (n, w) in stream.iter().enumerate().skip(start) {
        let k = n - start;
        assert_eq!(w.0, word(&left, k), "left, word {k}");
        assert_eq!(w.1, word(&right, k), "right, word {k}");
        matched += 1;
    }
    assert!(
        matched >= wanted,
        "{matched} words matched, {wanted} wanted"
    );
    eprintln!(
        "{} {}: {matched} words ({:.2} s) identical, file's first sound word {sound}, \
         stream starts it at frame {start} ({:.1} ms)",
        path.display(),
        if native { "native" } else { "DoP" },
        matched as f64 / word_rate as f64,
        start as f64 * 1000.0 / word_rate as f64
    );
    assert!(r.engine.telemetry(P).dsd);
}

#[test]
#[ignore = "needs a real DSD file in the real-music corpus"]
fn real_dsd_goes_out_as_dop_byte_for_byte() {
    for path in dsd_files() {
        check_words(&path, false);
    }
}

#[test]
#[ignore = "needs a real DSD file in the real-music corpus"]
fn real_dsd_goes_out_as_native_words_byte_for_byte() {
    for path in dsd_files() {
        check_words(&path, true);
    }
}

#[test]
#[ignore = "needs a real DSD file in the real-music corpus"]
fn real_dsd_converts_to_audible_pcm() {
    for path in dsd_files() {
        let i = info(&path);
        let rate = i.format.sample_rate as usize;
        let mut r = rig(DsdOutput::Pcm, false);
        r.start(&path, i.format);
        assert_eq!(r.dac.config().unwrap().dsd, None);
        assert_eq!(r.dac.config().unwrap().sample_rate as usize, rate);
        let secs = 10.0f64.min(i.duration) as usize;
        let raw = r.run(rate * secs);
        let mut loudest = f64::NEG_INFINITY;
        for (n, f) in raw.iter().enumerate() {
            for s in f {
                assert!(s.is_finite(), "frame {n}: {s}");
                assert!(s.abs() <= 1.0, "frame {n}: {s}");
            }
        }
        for window in raw.chunks(rate / 10) {
            let ms = window
                .iter()
                .map(|f| f64::from(f[0] * f[0] + f[1] * f[1]))
                .sum::<f64>()
                / (2.0 * window.len() as f64);
            loudest = loudest.max(10.0 * ms.max(1e-20).log10());
        }
        eprintln!(
            "{}: PCM {rate} Hz, loudest 100 ms window of the first {secs} s: {loudest:.1} dBFS RMS",
            path.display()
        );
        assert!(
            loudest > -60.0,
            "{}: only {loudest:.1} dBFS",
            path.display()
        );
    }
}
