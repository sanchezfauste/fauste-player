# Analysis (`fp-analysis`)

## Pipeline

`analyze_file_cancellable(path, settings, limits, cancelled)` decodes the
file once and returns an `Analysis`. Cancellation is checked between decode
blocks. The result holds:

- `TrackAnalysis`: title, artist, album, duration, the automatic markers
  (`cue_in`, `cue_out`, `segue_start`, `outro_start`), and the file's
  `format`: its sample rate, its bits per sample for lossless codecs (`None`
  for lossy ones), and its channel count. The track keeps the format, and source and cart
  requests carry it, so bit-perfect buses can follow the file rate without
  reading the file;
- `peaks`: one `WavePeak { min, max, rms }` (`i16`, full scale =
  `i16::MAX`) per `analysis.peak_bucket_ms` (10 ms) of the mono sum. The
  waveform draws the peaks dimmed and the RMS level as a solid body, one
  mesh of 1 px columns whose reduction is kept per player, track, span and
  width, and dropped when the player unloads;
- `cover_png`: a PNG thumbnail of the embedded cover (`analysis.cover_thumb_px`,
  128 px).

A truncated file is analysed up to where its data ends. A decode error mid-way
keeps what was decoded.

## Tags and covers (`metadata.rs`)

- lofty reads the tags. Its allocation limit is set per thread from
  `limits.max_cover_bytes`. If tags with artwork fail to parse, the read is
  retried without artwork, so a broken picture never hides the title.
- If there is no title, `Artist - Title` is parsed from the file name.
- Covers larger than `limits.max_cover_bytes` or
  `limits.max_cover_pixels` are rejected, and decoding uses explicit
  `image::Limits`. Every failure degrades to "no cover".

## Markers (`signal.rs`)

The envelope holds an RMS level per `analysis.rms_window_ms` window and,
per `peak_bucket_ms` bucket, the mono waveform peaks and the **stereo peak**
(`peak_db`: the largest absolute sample of either channel, so one-sided or
antiphase audio is not missed). They give:

| Marker | Rule |
|---|---|
| `cue_in` | start of the first bucket with `peak_db` ≥ `trim_threshold_db`, minus `trim_margin_ms` (≥ 0); else 0 |
| `cue_out` | end of the last such bucket, plus `trim_margin_ms` (≤ the duration); else the duration |
| `segue_start` | the end of the last body window (RMS windows overlapping `[cue_in, cue_out]`) ≥ median body RMS − `segue_drop_db`, clamped to `[cue_out − segue_max_secs, cue_out]` and `≥ cue_in` |
| `outro_start` | scanning back from `cue_out`: the end of the first window ≥ median RMS − `outro_drop_db`, clamped to `≥ cue_out − outro_max_secs`, and `< cue_out` |

Tracks shorter than `markers_min_duration_secs` get neither `segue_start`
nor `outro_start`. `intro_end` is never detected from audio. It comes from
an `INTRO` tag (`read_intro`), clamped to the cue range and stored as an
automatic marker so a manual one wins. The tag can be:

- an ID3v2 `TXXX:INTRO` (MP3, WAV, AIFF, FLAC, Monkey's Audio, DSF);
- a Vorbis, Opus or FLAC comment `INTRO`;
- an APE item `INTRO` (WavPack, Monkey's Audio, MP3);
- an MP4 freeform `----:com.apple.iTunes:INTRO`.

lofty reads the tags of every format except DSF. For DSF, `read_tags` reads
the ID3v2 chunk the DSF header points to (at most the cover limit), wraps it
in an in-memory WAV and hands that to lofty. DFF has no standard tags, so the
file name is used.

Its value is seconds (`12.5`) or `m:ss(.f)`. The model's
`Track::apply_analysis` stores automatic markers without touching manual ones.

## Cache (`cache.rs`)

There is one postcard file per track in `<cache>/analysis/`. The key is an
FNV-1a hash of the canonical path, size, mtime, `ANALYSIS_VERSION` (4 since
the format was added; the services thread analyses again, once, every track
that is analysed but has no format), the
analysis settings and the cover limits. The key is taken *before* analysing,
and the result is only stored if the file did not change meanwhile. Writes
use unique temporary files. Corrupt entries are ignored and recomputed.
File names are `v<ANALYSIS_VERSION>-<hash>.bin`. Before its first job, one
pool worker sweeps entries of any other version (they can never match a key
again) and leftover temporary files (`AnalysisCache::sweep`), so start-up
never waits on the cache directory.

## Pool (`analyzer.rs`)

`Analyzer::spawn(threads, settings, limits, cache)` starts the workers.
Every `submit(track, path)` gets a generation number. A job only runs, and
its result is only delivered, while it is the latest submission of its
track. That single rule covers:

- duplicates;
- `cancel(track)`, which also stops a running job through its cancel flag;
- resubmission.

Dropping the analyzer does not wait for queued jobs. A panicking job is
reported as `Unreadable`.

## How the app uses it (`fp-app/src/services.rs`)

The services thread submits tracks when either:

- they are not analysed yet (or were forced by *Re-analyse all* or by a
  settings change, which also cancels running jobs); or
- they are **shown** (current, next or cue on any player) and their peaks are
  not in memory.

Results go to the model as `ApplyAnalysis` or `SetFileState`. Peaks and covers
are kept in the `MediaCache` only for shown tracks. The disk cache brings
them back cheaply when a track is shown again.
