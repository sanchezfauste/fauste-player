//! Connects the remote server (`fp-remote`) to the conductor and to the
//! caches that hold covers and waveforms.

use std::path::PathBuf;
use std::sync::Arc;

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
}

impl Bridge {
    /// The cached analysis of `track`, read from disk (blocking).
    fn analysis(&self, track: TrackId) -> Option<fp_analysis::Analysis> {
        let model = self.conductor.model.load_full();
        let path = &model.library.get(track)?.path;
        self.cache.load(path, &model.config.analysis)
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
    };
    match fp_remote::spawn(Arc::new(bridge)) {
        Ok(handle) => Some(handle),
        Err(error) => {
            tracing::warn!(%error, "remote control could not start");
            None
        }
    }
}
