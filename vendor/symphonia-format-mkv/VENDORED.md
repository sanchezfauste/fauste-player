# Vendored `symphonia-format-mkv` 0.6.1

Source: crates.io `symphonia-format-mkv` 0.6.1 (MPL-2.0,
<https://github.com/pdeljanov/Symphonia>), used through `[patch.crates-io]`
in the workspace `Cargo.toml`. Every change is marked `VENDORED CHANGE` in
the source.

## Local changes

- `src/demuxer.rs`, `seek_track_by_ts`: a seek through a cue point set the
  cluster's timestamp from the cue's time divided by the timestamp scale
  (the time is already in Segment ticks), which made it 0, and jumped to the
  cue's relative position, past the cluster's `Timestamp` element. Every
  block of the cluster then seemed to come before the target, and the seek
  landed on the first block of the next cluster: about 4.7 s late at 10 s in
  a file with 5 s clusters. The seek now reads the cluster from its start,
  so its timestamp comes from its own element (the cue's time is the cued
  block's, not the cluster's), and clears the frames buffered before it.
  `MatroskaTicks::into_segment_ticks` (`src/segment.rs`) is no longer used.
- `src/demuxer.rs`, `seek_track_by_ts`: the seek read from the last cue
  point at or before the target. In a file with video, cue points index the
  video's frames (often every keyframe, several per cluster), whose
  keyframes also start the clusters, and the audio frame that holds the
  target can start in a cluster before, even several clusters before when
  clusters are shorter than an audio frame: the seek landed on the cued
  cluster's first audio frame, up to a frame late. The clusters of the cue
  points at or before the target, from the last cue point of the sought
  track (or the first cue point) on, are now tried from the last back until
  one reaches a frame of the track at or before the target (or skips one):
  usually the cued cluster or the one before it.
- `src/demuxer.rs`, `seek_track_by_ts_forward` (which now also tells
  whether it skipped a frame of the track, `Landing`): a frame with no duration
  (no `BlockDuration`, no `DefaultDuration`, as muxers write audio) counted
  as ending where it starts, so the seek landed on the first block starting
  at or after the target, up to a block late. The frames of the last block
  of the track skipped are kept, and when the next one starts past the
  target (or the stream ends) the seek lands on them instead.
- `src/codecs.rs`, `get_codec_id`: an unknown codec was logged with its
  codec ID, a value of the track element, which also holds the track's UID;
  code scanning flags it as clear-text logging of the UID. The line now
  logs a constant message. The other log lines of the crate print element
  types, byte counts, track numbers and timestamps, nothing read from a
  track element's text or UID.

Tested in `crates/fp-decode/tests/matroska.rs` with PCM files built in the
test. Drop this copy once a release upstream fixes the seeking defects
(still present on its `dev-0.6` branch on 2026-10-15).

## Keeping the patch applied

`[patch.crates-io]` replaces only the version it matches: when symphonia is
updated to a release that requires a newer `symphonia-format-mkv` (say
`^0.6.2`), cargo uses the registry crate and only warns that the patch is
unused. The version is not pinned (that would take a direct dependency
fp-decode does not use). Watch for that warning when updating symphonia:
`tests/matroska.rs` fails without these changes, so the suite catches a
dropped patch; then port the changes to the new version or drop the copy.

## Known, not changed

- The cue lookup converts the target to cue time with the track time
  base's numerator only (`SignedTrackTicks::try_into_matroska_ticks`),
  which is right for the usual timestamp scales (1 ms ticks reduce to a
  numerator of 1) but not for an unusual `TimestampScale` or
  `TrackTimestampScale`. The cue time is also typed `MatroskaTicks`
  (nanoseconds) though it holds Segment ticks; the changes above no longer
  use it as a cluster timestamp, and the type is left as it is.
- A malformed file whose blocks all share one timestamp and carry no
  duration makes a seek keep every skipped frame of that timestamp: memory
  bounded by the file's size, during the seek only.
