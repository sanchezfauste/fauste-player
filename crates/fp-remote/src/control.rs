//! What the remote server needs from the application (remote control spec
//! §7). `fp-app` implements it over the conductor and the caches, so this
//! crate depends on nothing but the model.

use std::sync::Arc;

use fp_model::{AppState, CartId, Command, PlayerId, TrackId};

/// Engine telemetry the API reports.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Playback {
    /// The conductor's model version: it rises with every model change.
    pub revision: u64,
    /// Position of each player's current source, in track seconds.
    pub players: Vec<(PlayerId, f64)>,
    /// Carts on air and their position in the file, in firing order.
    pub carts: Vec<(CartId, f64)>,
}

impl Playback {
    pub fn player_position(&self, player: PlayerId) -> Option<f64> {
        self.players
            .iter()
            .find(|(id, _)| *id == player)
            .map(|(_, s)| *s)
    }

    pub fn cart_position(&self, cart: CartId) -> Option<f64> {
        self.carts
            .iter()
            .find(|(id, _)| *id == cart)
            .map(|(_, s)| *s)
    }
}

/// A track's waveform: min, max and RMS per bucket, full scale `i16::MAX`.
#[derive(Debug, Clone, PartialEq)]
pub struct WaveformData {
    pub bucket_secs: f64,
    pub peaks: Vec<[i16; 3]>,
}

impl WaveformData {
    /// At most `max` buckets (0: all), each merging a whole group of them:
    /// the lowest minimum, the highest maximum and the RMS of their RMS, as
    /// the interface draws a column. A programme recording has hundreds of
    /// thousands of buckets; a client drawing it needs one per pixel.
    pub fn reduced(&self, max: usize) -> WaveformData {
        if max == 0 || self.peaks.len() <= max {
            return self.clone();
        }
        let group = self.peaks.len().div_ceil(max);
        let peaks = self
            .peaks
            .chunks(group)
            .map(|g| {
                let min = g.iter().map(|p| p[0]).min().unwrap_or(0);
                let max = g.iter().map(|p| p[1]).max().unwrap_or(0);
                let mean_square =
                    g.iter().map(|p| f64::from(p[2]).powi(2)).sum::<f64>() / g.len() as f64;
                [min, max, mean_square.sqrt().round() as i16]
            })
            .collect();
        WaveformData {
            bucket_secs: self.bucket_secs * group as f64,
            peaks,
        }
    }
}

pub trait RemoteControl: Send + Sync + 'static {
    /// The current model snapshot.
    fn model(&self) -> Arc<AppState>;
    fn playback(&self) -> Playback;
    /// Queues a command; never blocks. False when the queue is full.
    fn send(&self, command: Command) -> bool;
    /// The cover thumbnail (PNG). May read the disk, and for a track with
    /// no cached analysis analyse the whole file (minutes for a long one,
    /// one track at a time): call it off the runtime's thread
    /// (`spawn_blocking`).
    fn cover(&self, track: TrackId) -> Option<Vec<u8>>;
    /// The waveform. May read the disk, like `cover`.
    fn peaks(&self, track: TrackId) -> Option<WaveformData>;
}
