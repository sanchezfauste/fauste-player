//! Connects the remote server (`fp-remote`) to the conductor and to the
//! caches that hold covers and waveforms.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};

use fp_analysis::cache::AnalysisCache;
use fp_engine::conductor::ConductorHandle;
use fp_model::{AppState, Command, Limits, TrackId};
use fp_remote::RemoteHandle;
use fp_remote::control::{Playback, RemoteControl, WaveformData};

use crate::services::MediaCache;

struct Bridge {
    conductor: Arc<ConductorHandle>,
    media: MediaCache,
    cache: AnalysisCache,
    /// Held while a track is analysed for a request: one at a time, so that
    /// a client asking for many tracks cannot take every core.
    on_demand: Mutex<()>,
}

impl Bridge {
    /// The analysis of `track` (blocking). It is read from the cache; a
    /// track the cache has nothing for (one an earlier version analysed
    /// that no player shows, or an entry that was removed) is analysed now
    /// and cached, so that the next request, and the analysis the services
    /// thread would run, find it.
    fn analysis(&self, track: TrackId) -> Option<fp_analysis::Analysis> {
        let model = self.conductor.model.load_full();
        let path = &model.library.get(track)?.path;
        let settings = &model.config.analysis;
        if let Some(analysis) = self.cache.load(path, settings) {
            return Some(analysis);
        }
        let _one = self
            .on_demand
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        // A request that waited for the lock finds the other one's result.
        if let Some(analysis) = self.cache.load(path, settings) {
            return Some(analysis);
        }
        let analysis = fp_analysis::analyze_file(path, settings, &model.config.limits).ok()?;
        if let Err(error) = self.cache.store(path, settings, &analysis) {
            tracing::warn!(path = %path.display(), %error, "cannot cache the analysis");
        }
        Some(analysis)
    }
}

impl RemoteControl for Bridge {
    fn model(&self) -> Arc<AppState> {
        self.conductor.model.load_full()
    }

    fn playback(&self) -> Playback {
        let t = self.conductor.telemetry.load();
        Playback {
            revision: t.model_version,
            players: t
                .players
                .iter()
                .filter_map(|(id, p)| p.position_secs.map(|s| (*id, s)))
                .collect(),
            carts: t
                .carts
                .iter()
                .map(|(id, c)| (*id, c.position_secs))
                .collect(),
        }
    }

    fn send(&self, command: Command) -> bool {
        self.conductor.send(command)
    }

    fn cover(&self, track: TrackId) -> Option<Vec<u8>> {
        match self.media.get(track) {
            Some(media) => media.cover_png.as_deref().map(<[u8]>::to_vec),
            None => self.analysis(track)?.cover_png,
        }
    }

    fn peaks(&self, track: TrackId) -> Option<WaveformData> {
        let (peaks, bucket_secs) = match self.media.get(track) {
            Some(media) => (media.peaks.clone(), media.peak_bucket_secs),
            None => {
                let a = self.analysis(track)?;
                (a.peaks, a.peak_bucket_secs)
            }
        };
        Some(WaveformData {
            bucket_secs,
            peaks: peaks.iter().map(|p| [p.min, p.max, p.rms]).collect(),
        })
    }
}

/// Starts the remote thread. It listens only when `config.remote` asks for
/// it. A failure is logged and the application runs without it.
pub fn start(
    conductor: Arc<ConductorHandle>,
    media: MediaCache,
    analysis_dir: PathBuf,
    limits: &Limits,
) -> Option<RemoteHandle> {
    let bridge = Bridge {
        conductor,
        media,
        cache: AnalysisCache::new(analysis_dir, limits),
        on_demand: Mutex::new(()),
    };
    match fp_remote::spawn(Arc::new(bridge)) {
        Ok(handle) => Some(handle),
        Err(error) => {
            tracing::warn!(%error, "remote control could not start");
            None
        }
    }
}

/// A new API token: 32 random bytes in base64url without padding (43
/// characters). `None` if the system has no random source.
pub fn new_token() -> Option<String> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).ok()?;
    Some(base64url(&bytes))
}

fn base64url(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk.first().copied().unwrap_or(0),
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        let chars = chunk.len() + 1;
        for i in 0..chars {
            let index = ((n >> (18 - 6 * i)) & 63) as usize;
            if let Some(c) = ALPHABET.get(index) {
                out.push(char::from(*c));
            }
        }
    }
    out
}
