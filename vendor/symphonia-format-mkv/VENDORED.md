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
  video's keyframes, which also start the clusters, and the audio frame
  that holds the target can start in the cluster before: the seek landed on
  the cued cluster's first audio frame, up to a frame late. It now reads
  from the later of the last cue point of the sought track and the cue
  point before the last one (one extra cluster at most).
- `src/demuxer.rs`, `seek_track_by_ts_forward`: a frame with no duration
  (no `BlockDuration`, no `DefaultDuration`, as muxers write audio) counted
  as ending where it starts, so the seek landed on the first block starting
  at or after the target, up to a block late. The frames of the last block
  of the track skipped are kept, and when the next one starts past the
  target (or the stream ends) the seek lands on them instead.

Tested in `crates/fp-decode/tests/matroska.rs` with PCM files built in the
test. Drop this copy once a release upstream fixes both (still present on
its `dev-0.6` branch on 2026-10-15).
