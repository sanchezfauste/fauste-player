//! Tracks (audio files with metadata and cue markers) and the library that owns them.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::ids::TrackId;

/// What a track is used for on air.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TrackKind {
    #[default]
    Music,
    Jingle,
    Effect,
    Ad,
    Voice,
}

/// Whether the file behind a track can currently be played.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FileState {
    #[default]
    Ok,
    Missing,
    Unreadable,
}

impl FileState {
    pub fn is_playable(self) -> bool {
        matches!(self, FileState::Ok)
    }
}

/// Whether a marker was detected by analysis or placed by the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkerSource {
    Auto,
    Manual,
}

/// A cue point, in seconds from the start of the file.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Marker {
    pub secs: f64,
    pub source: MarkerSource,
}

/// The standard broadcast cue points of a track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkerKind {
    CueIn,
    IntroEnd,
    OutroStart,
    SegueStart,
    CueOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Markers {
    pub cue_in: Option<Marker>,
    pub intro_end: Option<Marker>,
    pub outro_start: Option<Marker>,
    pub segue_start: Option<Marker>,
    pub cue_out: Option<Marker>,
}

impl Markers {
    pub fn get(&self, kind: MarkerKind) -> Option<Marker> {
        match kind {
            MarkerKind::CueIn => self.cue_in,
            MarkerKind::IntroEnd => self.intro_end,
            MarkerKind::OutroStart => self.outro_start,
            MarkerKind::SegueStart => self.segue_start,
            MarkerKind::CueOut => self.cue_out,
        }
    }

    fn slot_mut(&mut self, kind: MarkerKind) -> &mut Option<Marker> {
        match kind {
            MarkerKind::CueIn => &mut self.cue_in,
            MarkerKind::IntroEnd => &mut self.intro_end,
            MarkerKind::OutroStart => &mut self.outro_start,
            MarkerKind::SegueStart => &mut self.segue_start,
            MarkerKind::CueOut => &mut self.cue_out,
        }
    }

    /// Stores an analysis result unless the user placed this marker manually.
    pub fn set_auto(&mut self, kind: MarkerKind, secs: Option<f64>) {
        let slot = self.slot_mut(kind);
        if matches!(
            slot,
            Some(Marker {
                source: MarkerSource::Manual,
                ..
            })
        ) {
            return;
        }
        *slot = secs.map(|secs| Marker {
            secs,
            source: MarkerSource::Auto,
        });
    }

    /// Removes every marker the user placed; automatic ones stay.
    pub fn clear_manual(&mut self) {
        for kind in [
            MarkerKind::CueIn,
            MarkerKind::IntroEnd,
            MarkerKind::OutroStart,
            MarkerKind::SegueStart,
            MarkerKind::CueOut,
        ] {
            let slot = self.slot_mut(kind);
            if slot.is_some_and(|m| m.source == MarkerSource::Manual) {
                *slot = None;
            }
        }
    }

    /// Places (or clears, with `None`) a user marker.
    pub fn set_manual(&mut self, kind: MarkerKind, secs: Option<f64>) {
        *self.slot_mut(kind) = secs.map(|secs| Marker {
            secs,
            source: MarkerSource::Manual,
        });
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub id: TrackId,
    pub path: PathBuf,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_secs: f64,
    pub kind: TrackKind,
    pub file_state: FileState,
    pub markers: Markers,
    /// True once analysis (tags, peaks, automatic markers) has completed.
    pub analyzed: bool,
}

impl Track {
    /// A not-yet-analysed track; the title is the file stem until tags are read.
    pub fn new(id: TrackId, path: PathBuf) -> Self {
        let title = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        Self {
            id,
            path,
            title,
            artist: String::new(),
            album: String::new(),
            duration_secs: 0.0,
            kind: TrackKind::default(),
            file_state: FileState::default(),
            markers: Markers::default(),
            analyzed: false,
        }
    }

    pub fn cue_in_secs(&self) -> f64 {
        self.markers.cue_in.map_or(0.0, |m| m.secs)
    }

    pub fn cue_out_secs(&self) -> f64 {
        self.markers.cue_out.map_or(self.duration_secs, |m| m.secs)
    }

    /// Cue-out when it is actually known: a marker, or a duration from
    /// analysis. `None` for a track whose duration is still unknown.
    pub fn known_cue_out_secs(&self) -> Option<f64> {
        match self.markers.cue_out {
            Some(m) => Some(m.secs),
            None if self.duration_secs > 0.0 => Some(self.duration_secs),
            None => None,
        }
    }

    pub fn segue_start_secs(&self) -> Option<f64> {
        self.markers.segue_start.map(|m| m.secs)
    }

    pub fn intro_end_secs(&self) -> Option<f64> {
        self.markers.intro_end.map(|m| m.secs)
    }

    pub fn outro_start_secs(&self) -> Option<f64> {
        self.markers.outro_start.map(|m| m.secs)
    }

    /// Audible length between cue-in and cue-out.
    pub fn play_length_secs(&self) -> f64 {
        (self.cue_out_secs() - self.cue_in_secs()).max(0.0)
    }
}

/// What analysis learned about a file (spec §6). Metadata fields are
/// `None` when the file did not provide them.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TrackAnalysis {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration_secs: f64,
    pub cue_in: Option<f64>,
    pub cue_out: Option<f64>,
    pub segue_start: Option<f64>,
    pub outro_start: Option<f64>,
    /// From an `INTRO` tag (Phase 2 spec P2.8); never detected from audio.
    #[serde(default)]
    pub intro_end: Option<f64>,
}

impl Track {
    /// Stores an analysis result. Known metadata is only replaced by values
    /// the analysis actually found, and manual markers always win.
    pub fn apply_analysis(&mut self, analysis: &TrackAnalysis) {
        if let Some(title) = analysis.title.as_ref().filter(|t| !t.is_empty()) {
            self.title.clone_from(title);
        }
        if let Some(artist) = analysis.artist.as_ref().filter(|a| !a.is_empty()) {
            self.artist.clone_from(artist);
        }
        if let Some(album) = analysis.album.as_ref().filter(|a| !a.is_empty()) {
            self.album.clone_from(album);
        }
        if analysis.duration_secs.is_finite() && analysis.duration_secs > 0.0 {
            self.duration_secs = analysis.duration_secs;
        }
        self.markers.set_auto(MarkerKind::CueIn, analysis.cue_in);
        self.markers.set_auto(MarkerKind::CueOut, analysis.cue_out);
        self.markers
            .set_auto(MarkerKind::SegueStart, analysis.segue_start);
        self.markers
            .set_auto(MarkerKind::OutroStart, analysis.outro_start);
        self.markers
            .set_auto(MarkerKind::IntroEnd, analysis.intro_end);
        self.analyzed = true;
        self.file_state = FileState::Ok;
    }
}

/// All known tracks, keyed by id. Serialised as a JSON array.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(from = "Vec<Track>", into = "Vec<Track>")]
pub struct Library {
    tracks: BTreeMap<TrackId, Track>,
}

impl From<Vec<Track>> for Library {
    fn from(tracks: Vec<Track>) -> Self {
        Self {
            tracks: tracks.into_iter().map(|t| (t.id, t)).collect(),
        }
    }
}

impl From<Library> for Vec<Track> {
    fn from(library: Library) -> Self {
        library.tracks.into_values().collect()
    }
}

impl Library {
    pub fn insert(&mut self, track: Track) {
        self.tracks.insert(track.id, track);
    }

    pub fn get(&self, id: TrackId) -> Option<&Track> {
        self.tracks.get(&id)
    }

    pub fn get_mut(&mut self, id: TrackId) -> Option<&mut Track> {
        self.tracks.get_mut(&id)
    }

    pub fn remove(&mut self, id: TrackId) -> Option<Track> {
        self.tracks.remove(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Track> {
        self.tracks.values()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Track> {
        self.tracks.values_mut()
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    pub fn is_playable(&self, id: TrackId) -> bool {
        self.get(id).is_some_and(|t| t.file_state.is_playable())
    }

    pub fn max_raw_id(&self) -> u64 {
        self.tracks.keys().map(|id| id.0).max().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track() -> Track {
        let mut t = Track::new(TrackId(1), PathBuf::from("/music/Artist - Song.flac"));
        t.duration_secs = 200.0;
        t
    }

    #[test]
    fn new_track_takes_its_title_from_the_file_stem() {
        assert_eq!(track().title, "Artist - Song");
    }

    #[test]
    fn cue_points_default_to_the_whole_file() {
        let t = track();
        assert_eq!(t.cue_in_secs(), 0.0);
        assert_eq!(t.cue_out_secs(), 200.0);
        assert_eq!(t.segue_start_secs(), None);
        assert_eq!(t.play_length_secs(), 200.0);
    }

    #[test]
    fn auto_markers_never_overwrite_manual_ones() {
        let mut t = track();
        t.markers.set_manual(MarkerKind::SegueStart, Some(190.0));
        t.markers.set_auto(MarkerKind::SegueStart, Some(185.0));
        assert_eq!(t.segue_start_secs(), Some(190.0));
        t.markers.set_auto(MarkerKind::CueOut, Some(198.0));
        assert_eq!(t.cue_out_secs(), 198.0);
        assert_eq!(
            t.markers.get(MarkerKind::CueOut).map(|m| m.source),
            Some(MarkerSource::Auto)
        );
    }

    #[test]
    fn manual_marker_can_be_cleared() {
        let mut t = track();
        t.markers.set_manual(MarkerKind::IntroEnd, Some(12.0));
        t.markers.set_manual(MarkerKind::IntroEnd, None);
        assert_eq!(t.intro_end_secs(), None);
    }

    #[test]
    fn only_ok_files_are_playable() {
        let mut lib = Library::default();
        let mut t = track();
        lib.insert(t.clone());
        assert!(lib.is_playable(TrackId(1)));
        t.file_state = FileState::Missing;
        lib.insert(t);
        assert!(!lib.is_playable(TrackId(1)));
        assert!(!lib.is_playable(TrackId(99)));
    }

    #[test]
    fn library_round_trips_through_json_as_an_array() {
        let mut lib = Library::default();
        lib.insert(track());
        let json = serde_json::to_string(&lib).unwrap();
        assert!(json.starts_with('['));
        let back: Library = serde_json::from_str(&json).unwrap();
        assert_eq!(back, lib);
        assert_eq!(back.max_raw_id(), 1);
    }
}
