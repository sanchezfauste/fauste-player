# External editor and changed files — design

Status: implemented (approved 2026-11-09), with the maintainer's decisions on markers (EE10) and
sandboxed packages (§6) included. Extends the track library spec
(`2026-11-01-track-library-design.md`: the row menus §7.4 and §10, watched
folders §6) and the analysis rules of the main design spec (§6).

## 1. Why

Operators prepare material ahead of the show: trimming a silence, adding a
fade, normalising a jingle by ear. They do it in an audio editor (Audacity,
for example) and today they have to find the file by hand. The library is
where they prepare material, so they want to open a track in their editor
straight from its menu.

Looking at it showed a wider gap: **Fauste Player never notices that a
file it knows has changed.** A watched folder's event on a known track is
dropped (the scan only adds new files and brings back missing ones). A
track outside a watched folder is never looked at again. The analysis
cache is keyed by size and modification time, but an analysed track is
never sent to the analysis again, so the cache is not even asked. The
duration, the automatic markers and the loudness of an edited file stay
stale until the operator presses Re-analyse. The one exception is a track
marked unreadable, whose size and time are already followed.

So this design has two parts that work together:

- **A. Open in external editor**: a menu entry and a setting. Launching
  the program is all it does.
- **B. Changed files**: Fauste Player notices that a library file changed
  and analyses it again by itself. When the audio changed and the track
  has manual markers, the operator is told to review them.

Part B works whoever changed the file: the external editor, another
program, a file copied over the old one, or the player's own tag editor.

## 2. Goals, non-goals, anti-goals

Goals:

- One click from a track's menu (library and playlist rows) opens the
  file in the editor the operator chose in Settings.
- A library file whose content changes is analysed again by itself:
  duration, automatic markers, loudness, waveform, format and tags.
- A change of the **audio** on a track with **manual** markers leaves
  them in place and flags the track "audio changed: review the markers"
  until the operator says they are fine.
- A change of the **tags** alone (or the cover) refreshes the tags and
  raises no flag.

Non-goals:

- Android. The menu entry and the setting do not exist there (the
  platform has no way to launch an arbitrary program, and the music is
  read-only, mobile spec §2). Part B's detection may still run there, but
  nothing is designed for it.
- Editing audio inside Fauste Player.
- Arguments or templates for the editor's command line (see EE3).
- Opening several tracks at once (see EE2).
- A remote API or OSC entry for any of this.

Anti-goals (a failure even if it works):

- **Anything on air changes.** A track that is sounding cannot be opened
  in the editor, and a changed file is never analysed again while it is
  sounding. Its markers and gain never move under a playing source.
- A flood of false flags: after updating to this version, after a
  decoder upgrade, or after the player's own tag editor saved a file.
- The interface blocking: no `stat`, no process launch, no hashing on
  the UI thread (rule 8).
- Inotify exhaustion on a large library: no per-track or per-folder
  system watch beyond the watched folders the operator set up.

## 3. Rules

### A. Open in external editor

- **EE1. Setting.** `Config` gets an optional path to the editor's
  program, empty by default (rule 4: a `Config` field with a documented
  default and lenient loading). Settings shows it in the Library section
  as a path field with a **Choose…** button (the native file picker,
  through `Platform::pick_file`, off the UI thread) and a **Clear**
  button. A path that is not absolute is ignored and logged when loaded,
  as for watched folders. Not shown on Android.
- **EE2. Menu entry.** "Open in external editor" sits in the row menu of
  the library (window) and of playlist rows, next to Edit tags…. It acts
  on **one** track: with several rows selected it is not shown, like
  Edit tags…. It is not shown on Android.
- **EE3. Launch.** The program is started with the track's path as its
  only argument, on a helper thread (never the UI thread), detached:
  Fauste Player does not wait for it or track it. On macOS, a chosen
  `.app` bundle is launched with `open -a <bundle> <file>`; a plain
  executable is run directly. On Windows and Linux the program is run
  directly. If the launch fails (not found, not executable), the status
  bar shows a notice naming the program and the log gets the error.
- **EE4. When it is disabled.** The entry is shown dimmed, with the
  reason as its tooltip, when:
  - no editor is set (the tooltip points to Settings ▸ Library);
  - the file is missing or unreadable;
  - the track is **sounding** (EE5).
- **EE5. Sounding.** A track is sounding when it is the current entry of
  a player (playing or paused), the CUE entry of a player, on a playing
  cart, or in the library pre-listen while it plays or is paused (a stopped
  preview bar holds no source and does not count). This is the same
  rule as the tag editor's block (`fp_model::tag_edit_block`: `OnAir`,
  `Cued`, `OnCart`) plus the library pre-listen. It is a pure function in
  `fp-model`, so the menu and the change detection (EE9) agree. The
  check happens when the menu opens. Starting the track on air after the
  editor opened it is the operator's business; EE9 still protects it.

### B. Changed files

- **EE6. The file stamp.** Each library track stores the **size and
  modification time** of the file its analysis read (the stamp), and the
  **audio fingerprint** that analysis computed (EE8). Both are saved in
  the database. A track analysed before this version has neither.
- **EE7. Noticing a change.** A track's file has changed when its current
  size or modification time differs from the stored stamp. It is looked
  at:
  1. **at start**, for every library track, in the background;
  2. **in a watched folder**, when the system reports an event on a file
     that is a known track (today these events are dropped). The existing
     debounce applies, so a file still being written is not read half
     way;
  3. **after "Open in external editor"**: the folder of the opened file
     is followed through the same system notifications (not recursive)
     until the application ends, so a save in the editor is noticed at
     once. Only the folders of files actually opened are followed; one
     already inside a watched folder needs nothing more;
  4. **at the point of use**: when a track is loaded into a player, onto
     a cart or into the library pre-listen, or its markers are opened for
     editing, its stamp is checked on a helper thread. A changed file is
     analysed again at once, ahead of everything else; playing is never
     held back for it, and if the track starts sounding first, EE9 makes
     the analysis wait;
  5. **on demand**: **Look for changed files**, in Settings ▸ Analysis
     next to Re-analyse all, runs one pass over every track for files
     changed outside the application while it runs. The changed ones go
     to the analysis (whose progress shows as usual), and a notice says
     how many were found.

  There is **no periodic polling** of the library. The one case not
  noticed by itself (a file outside any watched folder, changed by
  another program while the application runs, and not used since) is
  caught at its next use (4) or by the action (5).

  A file found missing follows the existing missing-file rules, not
  these.
- **EE8. The audio fingerprint.** While the analysis decodes the file it
  computes a hash of the decoded samples (a fast non-cryptographic hash
  such as xxHash or FNV over the PCM as decoded; FNV-1a 64 over the downmixed
  stereo stream the analysis decodes, not the file's own channels). Tags and
  the cover are not part of it, so a tag-only edit leaves it the same.
  It is stored with the analysis version that computed it. Two
  fingerprints are compared only when both come from the same analysis
  version: a decoder upgrade (which bumps the version) cannot raise
  false flags.
- **EE9. Analysing again.** A changed track is analysed again with the
  same machinery as Re-analyse on its row (it goes ahead of the library
  pass). It is **never analysed again while it is sounding** (EE5): the
  change waits and is acted on once the track stops sounding. The new
  result then updates the track the usual way (`apply_analysis`: manual
  markers always win) plus the stamp and the fingerprint.
- **EE10. Audio changed or not.** When the new analysis is applied:
  - same fingerprint (same version): only tags or cover changed. No
    flag. The tags refresh as after any analysis;
  - different fingerprint (same version), and the track has **at least
    one manual marker**: the track is flagged **"markers need review"**.
    A manual marker that no longer fits the new audio (past the new
    duration) is **dropped**: the automatic value of the new analysis
    takes its place, or none. The flag remembers which markers were
    dropped, and the tooltip names them. The other manual markers stay as
    they are. A track therefore never reaches air with a marker past its
    end;
  - different fingerprint, no manual markers: no flag (the automatic
    markers were recalculated);
  - no stored fingerprint, or one from another version: the new one is
    stored and **no flag** is raised (the first pass after updating, or
    after a decoder upgrade, never flags).

  This rule is a pure function in `fp-model`, with one test per case
  (including a dropped marker and its automatic replacement).
- **EE11. The flag.** "Markers need review" is saved with the track. It
  shows:
  - as a warning icon on the track's row in the library table and on
    playlist rows, with a tooltip ("The audio of this file changed after
    its markers were set by hand. Check them, then mark the track as
    reviewed.");
  - as a filter in the library ("Markers to review"), so the operator can
    find all of them. Where the filter lives (the tree's fixed entries or
    the search) is builder's choice, following the library window's
    existing patterns.

  The flag never blocks loading, cueing or playing the track.
- **EE12. Clearing the flag.** The flag goes away when:
  - the operator picks **Mark as reviewed** in the track's menu (library
    and playlist rows; shown only on flagged tracks; on several selected
    rows it clears all of them), or
  - the operator **changes any marker** of the track (sets, moves or
    clears a manual marker, in any editor of markers).

  A later change of the audio flags it again (EE10).
- **EE13. Own writes.** When the player's tag editor writes a file, the
  stamp changes. EE7 notices it, EE10 finds the same fingerprint, and no
  flag is raised. Builder's choice: either let that happen (one extra
  analysis per tag save) or update the stored stamp right after the
  player's own write so no analysis follows. The second is cheaper, but
  it must not hide a change made by someone else at the same moment.

## 4. Configuration

New `Config` fields, each with a default, a range in `Config::validate` and
lenient loading (rule 4). Names are the builder's, following the existing
sections:

| What | Default | Notes |
|---|---|---|
| External editor program | empty (none) | absolute path; desktop only |

Nothing else is configurable: no polling interval exists (EE7).

## 5. Where things go

- `fp-model`: the config fields; the stamp, fingerprint, fingerprint
  version and review flag on `Track`; the "sounding" rule (EE5); the
  "audio changed" rule (EE10); clearing the flag on a marker change and
  on a new `Command` (Mark as reviewed). One test per rule.
- `fp-store`: the new track columns, with a migration. Old rows load with
  no stamp, no fingerprint and no flag.
- `fp-analysis`: the fingerprint while decoding. `ANALYSIS_VERSION` goes
  up so that cache entries without it are not reused, but tracks analysed
  by the previous version stay current (`analysis_is_current`): **no
  library-wide analysis follows the update.** A track gets its
  fingerprint at its next analysis (by EE10 that one never flags), and a
  track without a stamp gets the stamp of its file as it is at the first
  start (EE7.1), taken as unchanged.
- `fp-app`: the services thread does the stamp checks (start, point of
  use, the on-demand pass) on a helper thread, the folder watch passes events on
  known tracks to them, and changed tracks are sent to the analysis
  (deferred while sounding). The UI adds the menu entries, the row icon,
  the filter and the setting. The launch goes through the platform seam
  (`platform.rs`), so Android simply has none.

## 6. Sandboxed packages

The Flatpak build reads home read-only (`--filesystem=home:ro`) and
cannot start programs on the host. The snap is strictly confined. In
both, an external editor cannot be launched, and the editor could not
save over the file anyway. **Decided (maintainer, 2026-11-09):** like
Android, these builds show neither the setting nor the menu entry; no
extra sandbox permissions are asked for. They are detected at
run time (`FLATPAK_ID`, `SNAP` in the environment). Part B still works
there for files changed outside the sandbox.

The `.deb`, `.rpm`, AppImage, AUR, Windows (installer and MSIX) and macOS
builds get the full feature. MSIX packages run with full trust and can
start other programs.

## 7. Documentation and strings

- User guide (`docs/user/`): the library chapter (menu entry, flag,
  filter) and the settings chapter (the editor field). A short section on
  "when a file changes" explains EE7 to EE12 in operator terms.
- Technical docs (`docs/technical/`): the change detection and the
  fingerprint.
- UI strings in en-US and es-ES with the change; the other locales in
  bulk. Keys shipped: `menu-open-in-editor` (and `-no-editor`, `-unavailable`,
  `-sounding`), `menu-mark-reviewed`, `track-markers-review-tip` and
  `track-markers-review-tip-dropped`, the five `marker-name-*` keys
  (`cue-in`, `intro`, `outro`, `mix`, `cue-out`), `library-tree-markers-review`,
  `settings-external-editor`, `settings-external-editor-none`,
  `settings-hint-external-editor`, `settings-choose`, `settings-clear`,
  `settings-look-for-changes`, `settings-hint-look-for-changes`,
  `notice-changed-files`, `notice-editor-failed`; the analysis activity has a
  `looking` state.
- README feature list: one line.

## 8. Testing

- `fp-model`: EE5, EE10 (every case), EE12 (both ways to clear, and
  re-flagging), config validation.
- `fp-analysis`: the same audio with different tags gives the same
  fingerprint; changed audio gives a different one (WAV files written in
  the test, as in the existing tests).
- `fp-store`: migration of an old database; round trip of the new
  fields.
- `fp-app`: services tests with a scratch file: change it, the track is
  analysed again; change it while it sounds, nothing happens until it
  stops; a watched-folder event on a known track triggers it. UI tests
  (`egui_kittest`) for the menu entry (shown, dimmed with each reason,
  absent without an editor on Android), Mark as reviewed and the row
  icon. The launch itself is behind the platform seam and recorded by
  the fake.

## 9. Calls made by the designer (not discussed)

- One track at a time in "Open in external editor" (EE2).
- No command-line arguments or `{file}` template (EE3).
- Detection by system notifications and checks at start and at the point
  of use, with an on-demand pass; no periodic polling (EE7, maintainer's
  request). Only the folders of files opened in the editor get extra
  system watches, which keeps clear of inotify limits.
- The flag is raised only when manual markers exist (EE10, as asked) and
  the first pass after updating never flags.
- The flag is shown, never blocking (EE11).
- "Sounding" (EE5) is read cautiously for the change detection: a stopped
  player that still holds the track, the outgoing track of a crossfade
  and a paused pre-listen all count. Too cautious only delays an
  analysis.
- A file that looks missing during a save (editors that delete and
  recreate it) is checked twice before it is marked missing, so the
  library's automatic removal of gone files never fires on a save in
  progress.
