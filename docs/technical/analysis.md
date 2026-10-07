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
- `Tags` also carries the recording date (ISO 8601 text, as the standards
  store it: ID3v2.4 `TDRC`, Vorbis `DATE`, MP4 `©day`, APE `Year`), the
  genre, the album artist, the composer and the comment.
- `display_picture` is the one rule for which picture stands for a file: the
  front cover, else the first picture. The library's thumbnail and the tag
  editor share it.

## Track tags (`tags.rs`)

Reading degrades to "nothing"; writing never touches the original until the
new file is complete.

### The summary API

- `read_track_tags(path, limits)` gives the `TrackTags` the library keeps
  (title, artist, album, album artist, date, genre, composer, comment). The
  file name stands in for a missing title (and its `Artist - ` for a missing
  artist). Every text is trimmed and cut to `limits.max_tag_chars`. A panic in
  the parser costs only the tags.
- `can_write_tags(path)` judges the format from the extension alone, from
  lofty's `FileType::tag_support` (no I/O, so the interface can ask).
- `write_tags(path, before, after, limits)` writes only the fields that
  differ; an empty one removes the tag. The editor does not use it: it is
  the summary-level write, used by `TagJob::Write` and its tests.

### The tag sheet

- `TagField` (in `fp-model`) has 36 fields in the shown order; the first 10
  are always shown. `TagField::slug` names the field's locale key
  (`tag-field-<slug>`).
- `storable_fields(tag_type)` lists what a format can store: for each field,
  the first `ItemKey` of its candidate list that `ItemKey::supported_keys`
  lists for that tag type. A track or disc number also needs the total's key
  to store a total.
- `read_tag_sheet(path, limits)` reads the tag that `read_tags` reads (the
  primary one, else the first). A field's values are the items of its key
  with an empty description. A number and total pair splits a `3/12` written
  into one key. The sheet is clamped to `limits.max_tag_chars` and
  `limits.max_tag_values`; a field that lost text or values is marked
  (`TagSheet::is_cut`) and is never written. `other_kept` counts the other
  items and the pictures that are not the front cover; `other_kept_more` says
  the format also holds frames lofty cannot count. The sheet is `None` for a
  format with no writable tags or a file that does not parse.
- `write_tag_sheet(path, before, after, limits)` clamps `after` to the limits
  and refuses, before it touches the disk, a field that
  `fp_model::invalid_fields` reports (`field_problem`: `InvalidValue`,
  `NotStorable`, or `TooLongToEdit` for a cut field), a cover change in a
  format with no pictures and a new cover that is not usable. It replaces each
  changed field with `retain` and `push`, one item per value, and never
  touches an item it does not own (other keys, items with a description,
  custom frames, other pictures).

### The cover

- `can_store_pictures(tag_type)`: ID3v2, Vorbis comments, MP4 and APE. RIFF
  INFO, AIFF text and ID3v1 have no place for one, and lofty would drop it
  silently.
- `load_cover_file` and `check_cover`: the image must be JPEG or PNG by its
  content, at most `limits.max_cover_bytes`, and decode through the same
  `thumbnail_png` as the library's thumbnails under `limits.max_cover_pixels`
  (bounded decode, panics contained). The file must be a regular file. The
  result is a `CoverArt` with a thumbnail of at most `analysis.cover_thumb_px`.
- `with_cover_thumbnail` decodes the thumbnail of an existing cover. One that
  does not decode (or a GIF, BMP or WebP, since the decoder is built with JPEG
  and PNG only) keeps no thumbnail: the editor says it cannot be shown and the
  picture stays as it is.
- `change_cover` removes every `CoverFront` picture and adds the new one with
  its MIME type; other pictures are untouched. The sheet's cover is the front
  cover, or the first picture when there is none (`display_picture`);
  `CoverArt::is_front` tells which, and a picture that is only displayed is
  never removed or counted as the cover that was kept.

### The safe write

`safe_edit` is the one helper behind both writes:

1. canonicalise the path, so a symlink is written through and not replaced;
2. check the extension (`can_write_tags`);
3. copy the file to `name.fptag-<pid>.ext` in the same folder;
4. parse the copy with its covers, or fail (saving a tag read without them
   would drop them);
5. `urls_as_text`, then the edit;
6. `save_to_path`, fsync, rename over the original.

The copy is removed on any error, so a failure leaves the original and no
stray file. `TagWriteError` maps what can go wrong: `Unsupported`,
`NotFound` and `Denied` (from the I/O kind), `InvalidDate`,
`InvalidField(field)`, `CoverNotStorable`, `InvalidCover(CoverError)` and
`Other` (with the system's or lofty's description).

Two lofty 0.25.4 defects shape it:

- ID3v2 URL frames come out of a parsed file as locators, and lofty drops a
  locator when it builds the frames of a saved tag. `urls_as_text` re-adds
  them as text so they are written back as URL frames. Guarded by
  `a_summary_write_keeps_the_url_frames_too` and
  `custom_items_and_pictures_survive_an_edit_untouched`.
- `Tag::take_filter` (and `Tag::take`) swap items around and reorder the ones
  they leave, so items are removed with `retain` or `remove_key` and pushed
  again. Guarded by `a_save_keeps_the_order_of_the_values_it_did_not_change`.

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

`submit_urgent` queues a job on a second queue that workers always take
from first; `promote` moves a normal job still waiting there. A track a
player shows gets its waveform ahead of a library being analysed. An
urgent job already waiting or running is never duplicated, and a job
whose result was already sent is not promoted again.

Dropping the analyzer does not wait for queued jobs. A panicking job is
reported as `Unreadable`.

## How the app uses it (`fp-app/src/services.rs`)

The services thread submits tracks when either:

- they are not analysed yet (or were forced by *Re-analyse all* or by a
  settings change, which also cancels running jobs); or
- an earlier version analysed them (no format recorded, or an older
  `analysis_version`, and a readable file: `services::outdated`) **and**
  the operator asked for it (`ServiceRequest::AnalyseOutdated`, from the
  start-up notice or
  Settings → Analysis). Until then they keep that analysis, which still
  works; re-analysing a library costs the processor for a while. The
  exception is a track on a cart that has no format recorded: the cart bus
  opens bit-perfect only for a known format, so it is analysed at once
  (`Services::cart_tracks`); or
- they are **shown** (current, next or cue on any player) and their peaks are
  not in memory. Shown tracks go on the urgent queue, and one already
  queued with the library is promoted.
- their file was not found (`Missing`) and the probe thread
  (`fp-file-probe`, `Services::recheck_missing`) found it again. It looks
  every `tuning.missing_recheck_ms` (30 s), one look at a time, each
  folder once from the root down (a drive not mounted costs one look), so
  a share that is offline never holds up the services thread or the pool.
  A drive mounted after the start brings its tracks back by themselves,
  from the cache when they were analysed before. A file still missing
  sends nothing to the model, so nothing is saved every interval.
  `Unreadable` files are not decoded again by themselves (operator
  feedback 4, Q10). The same probe thread `stat`s them (it never opens
  them) and answers with what it saw; `Services::stamps` records each file's
  size and modification time (`Seen`). For a failed analysis it is the stat
  the worker took just before decoding (`AnalysisResult::stamp`), so a file
  that changed while it was analysed (a copy that completed) is analysed
  again at the next look. For a playback failure, or when that stat failed,
  the first look at the newly unreadable file is not waited for the timer
  and its stat is recorded. A changed size or time clears the failure and queues
  an analysis; an unchanged file does nothing (no analysis, no save); a
  file that is gone becomes `Missing` (sent once: until a snapshot shows it, the track is neither looked at nor reported again); any other `stat` error is ignored. A
  change seen while the track is already being analysed is dropped and
  reported again at the next look. `ServiceRequest::ReanalyseTrack` (the row
  menu's **Re-analyse**) clears the failure and queues the track on the
  urgent queue whatever its state, through the same analyser and cache as
  *Re-analyse all*, so an unchanged file whose analysis is cached comes back
  playable and fails again on the next playback.

A **tag-only pass** reads the tags of tracks that were analysed but not read
since: `Track::needs_tag_read` (analysed, readable, `tags_read` false) selects
them and `Services::tag_pass` sends a `TagJob::Read` for each to the `fp-tags`
worker (`fp-app/src/tags.rs`, one thread). The worker has five jobs: `Read`,
`Write`, `ReadSheet`, `WriteSheet` and `LoadCover`. A panic inside a job is
contained and answered as a failure. At shutdown queued reads are skipped,
while the job in progress and queued writes finish. The answer becomes
`Command::ApplyTags`, which sets `tags_read`. Every `ApplyAnalysis` resets
`tags_read`, so a re-analysed track is read again. The pass changes no
analysis version and no cache key: tracks from an earlier version get their
date, genre and other tags without a re-analysis. The editor's own jobs
(`ReadSheet`, `WriteSheet`, `LoadCover`) come from the UI; see
[User interface](ui.md).

Results go to the model as `ApplyAnalysis` or `SetFileState`. Peaks and covers
are kept in the `MediaCache` only for shown tracks. The disk cache brings
them back cheaply when a track is shown again.
