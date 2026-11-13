# Automation — Design Spec

- **Date:** 2026-11-11.
- **Status:** Approved (2026-11-11).
- **Amends:** [the main spec](2026-09-25-fauste-player-design.md) §1
  non-goals ("scheduling / clock-based automation": now a goal); [the track
  library spec](2026-11-01-track-library-design.md) §12 (this is the phase it
  prepared) and its track kinds (a new kind, AU30).
- **Revised by:** [the automation rework spec](2026-11-13-automation-rework-design.md)
  (rules `AR<n>`, 2026-11-13): its terms (§3), how the Log is started,
  stopped and pointed (§5.1) and its screens (§8) replace the ones first
  written here; those sections now point to it. The rest stands.
- **Numbering:** rules are `AU<n>`. Each rule is testable and gets at least
  one test (§12).
- Marks: *(maintainer)* is a decision taken in the brainstorming;
  *(my call)* is the designer's default, open to change in review;
  *(builder's call)* is left to the plan.

---

## 1. Intent

A station is staffed some hours and empty others. Today Fauste Player only
plays what an operator starts. This phase adds **automation**: a daily
**schedule** of timed blocks (hourly IDs, spots, recorded shows, music) that
plays itself once the operator starts it, and a **music filler** that keeps
music going between blocks from rules the station sets.

The existing players are untouched and become the **auxiliary** players: an
operator prepares a show on them and plays it live, without automation.
Automation and live work alternate as the operator decides, many times a day:
let the hourly ID and the spots play, stop the automation after the current
item, do the show on the players, start the automation again for the next
spots, or fire those spots by hand from a player.

The design is for **any station**, not one: some have live shows at
weekends, some none; some schedule every hour, some only a few blocks a day.

### Why now

The track library (categories, kinds, a SQLite database) was built for this
phase (track library spec §12).

### What success looks like

- A station with no operator overnight runs for weeks on a weekly plan
  without anyone touching it, and every block starts on the second it is due.
- An operator stops and starts automation at will and always knows what
  plays next and why.
- A small station gets value on day one from the music filler alone, with no
  schedule configured.

## 2. Scope

**Goals:** §3–§10 on the desktop (Linux, Windows, macOS).

**Non-goals (later, or never in this spec)** *(my call unless marked)*:

- Android. The model and store work there (shared core), but the phone gets
  no automation screens in this phase.
- Automation commands on the remote API and MIDI. They are the obvious next
  step (start, stop after current, set next); §11 keeps the door open.
- Exporting the as-run log (CSV, PDF) and any spot billing.
- Validity dates for spots (a spot that stops airing after a date), live
  inserts or network feeds, voice tracking, satellite or stream relays.
- Rules beyond those in §6 (tempo, energy, gender, mood, era).

**Anti-goals (a failure even if it works):**

- **Anything going on air by itself** after a start, a restart or a crash
  (rule 10). Automation starts only by an operator's action, every time
  *(maintainer)*.
- **Cutting a scheduled item.** An item of the schedule always plays to its
  end; only the music filler is faded *(maintainer)*.
- **An hourly ID at the wrong time** (the "it's one o'clock" ID at 13:10)
  *(maintainer)*.
- **A few seconds of a song** before a block: the filler never starts a song
  only to fade it a moment later *(maintainer)*.
- **Degrading over time.** Six days, or sixty, of continuous play must not
  grow memory, lists, files or database tables without bound, nor slow the
  UI *(maintainer)*.
- **Silence because a rule could not be met** (a small category, missing
  files). Rules relax before the music stops.
- Automation slowing what is on air (rule 5, rule 8): selection, schedule
  computation and database queries run off the UI thread and off the device
  callback.

## 3. Terms

Replaced by the rework spec §3 (the sector's usual terms): **Log**
(*Parrilla*) for a date's programme and its column, **Template**, **Week
plan**, **Date changes**, **Block**, **Item**, **Category track**, **Music
fill** (*Musicalizador*) for both the player between blocks and the item
that hands over to it, **Rotation**, **Hard start** (*Hora fija*),
**Missed**, **As-run log**, and **Hourly ID** (*Indicativo horario*, AU30)
and **Late tolerance** (*Margen de retraso*) as before. Code identifiers
keep their first names (`schedule`, `filler`, `time_critical`); this spec's
rules below still say "schedule", "filler" and "time-critical" for them.

## 4. Model

### 4.1 Rotations

- **AU1.** A **rotation** has a name and an ordered list of **steps**. A step
  is a category (with its subcategories, *my call*) and a count given as a
  range `min–max` (`min` ≥ 0, `max` ≥ 1, `min ≤ max`). Example:
  `Current 2–3 · Station IDs 1 · Gold 0–1`. A minimum of 0 lets a step
  sit out some turns *(maintainer, 2026-11-12: "between 0 and 1", so a
  track of that category may or may not be added; 0–0 makes no sense)*.
- **AU2.** The filler plays the steps in order and starts again at the first
  after the last. Each time a step comes up its count is drawn afresh in its
  range, so a station ID falls after two songs one time and three the next
  *(maintainer: variety without a separate random mechanism)*. A step that
  draws 0 is passed over. When every step of a whole pass draws 0, the
  next pass draws at least 1 for each step, so a rotation always plays and
  never falls back on the default for that reason *(my call)*.
- **AU3.** A step may use a category of any kind (music, jingles, promos,
  spots). Selection inside the step follows §6.
- **AU4.** One rotation is the **default rotation** (chosen in the
  automation settings; a fresh install creates one, "Music", with a single
  step over all music *(my call)*).

### 4.2 Blocks and items

- **AU5.** A **block** has a start time of day (`HH:MM:SS`, local time) and
  an ordered list of items. Blocks are not tied to the hour: 07:30 is as good
  as 08:00. A block lasts until the next block's start.
- **AU6.** An **item** is one of:
  1. **a track** of the library, any kind, music included *(maintainer: no
     limits)*;
  2. **one track from a category**, chosen by §6 each time the item airs
     (alternates station IDs, spots, show promos) *(maintainer)*;
  3. **a rotation**: from here to the end of the block, the music filler
     plays that rotation. It has no length of its own.
- **AU7.** An item is **time-critical** when its track's kind is Hourly ID
  (AU30), when it is a category item whose category's kind is Hourly ID
  *(my call: known at edit time, before a track is chosen)*, or when the
  operator marks it so. Other items are not.
- **AU8.** If a block has no rotation item and its items end before the next
  block, the filler plays the **default rotation** until then
  *(maintainer)*.

### 4.3 Templates, week plan, date overrides

- **AU9.** A **day template** is a named list of blocks. The **week plan**
  gives each weekday a template; a fresh install has one template, "Base",
  used by all seven days.
- **AU10.** A **date** may name a different template than its weekday's (a
  public holiday on a Thursday uses "Weekend").
- **AU11.** A **date override** changes blocks of one date only
  *(maintainer)*. It stores only the blocks the operator changed on that
  date; every other block of that date keeps following its template, and
  later edits to the template reach them *(maintainer)*.
- **AU12.** Template blocks have a stable id, and a block override refers
  to the template block it replaces by that id *(my call)*: "the 17:00 block
  of 22 Dec is this". If the template block later moves to 17:30, the
  override moves with it; if the template block is deleted, the override
  becomes a block of that date alone, at its own time. Adding a block on a
  date, and removing a template block from a date, are overrides too. An
  overridden block keeps its own content even if the template's block
  changes later *(my call: the operator chose it for that day)*; the editor
  shows it as "changed for this day" with "back to the template".
- **AU13.** The **schedule** of a date is computed: its template (AU9–AU10)
  with its overrides (AU11–AU12). It is a pure function of the stored data
  and the date.
- **AU14.** Local time and daylight saving: a block whose time does not
  exist on a date (02:30 on the spring change) is missed; a time that occurs
  twice (the autumn change) plays once, at its first occurrence *(my call)*.

## 5. Running the schedule

### 5.1 Start, stop, next

Replaced by the rework spec §4–§5. In short: the Log column is a player
with a player's transport (AR3–AR10): **Play** plays what the operator
selected as next and runs automation; **Pause**, **Stop**, **Fade stop**
and **Stop at end** stop it; **SINGLE** makes it manual. Automation is
stopped after every start of the application, restart or crash, and
starts only by the operator's Play (rule 10; was AU15). Next is what the
operator selected, else what the clock says (AR11–AR13; was AU16): a next
set by hand plays at once on Play, one from the clock waits for its block
with the music fill until then (was AU17). **AU18** stands: while running,
items play in order from next to the end of their block, each after the
previous one; a hand-set next is always obeyed, and a past block played by
hand plays in full.

### 5.2 After a block

- **AU19.** When a block ends (its last item before a rotation, or its last
  item), next moves to the **next block in time order that has not started
  yet by the clock**, across midnight into the next date. Blocks in between
  are **missed** *(maintainer: the 10:00 block played at 11:00 ends at
  11:55 → music until 12:00, then the 12:00 block; the 11:00 block is
  missed)*.
- **AU20.** Until that block's start, the music filler plays the ending
  block's rotation item, or the default rotation (AU8) *(maintainer)*. The
  first item of the coming block is shown as next while the filler plays.
- **AU21.** A block is **missed** when its start time passes while
  automation is stopped, or while a block chosen by hand away from its time
  is on air (AU19). A block whose start time passes while **the block
  before it, started at its own time or itself reached in order, is still
  on air** (it overran) is not missed: it plays as soon as the previous item
  ends, minus its time-critical items beyond tolerance (AU22)
  *(maintainer)*. Reached in order is a chain: an overrun can delay several
  blocks, each losing only its late time-critical items, until a rotation
  absorbs the delay.

- **AU22.** A **time-critical** item is skipped and marked missed when it
  would start later than its block's start plus the **late tolerance**
  (`automation.late_tolerance_secs`). Skipping never cuts anything: the next
  item plays instead *(maintainer)*.
- **AU23.** Every scheduled item that did not air (skipped, in a missed
  block, or passed while stopped) shows as **missed** in the schedule column,
  and the operator may still set it as next and play it.

### 5.3 Starting a block on time

- **AU24.** The filler hands over so that the block's first item starts
  **exactly at the block's start**: the song on air fades out over
  `automation.block_fade_ms`, starting that long before the start. If the
  hand-over is a crossfade, the timing uses the crossfade's real overlap and
  the song's segue or cue-out point *(maintainer)*. "Exactly" is to one
  conductor tick (`tuning.conductor_tick_ms`, 5 ms by default), not to the
  sample: the hand-over is between two players, and a few milliseconds are
  inaudible and below the system clock's own accuracy *(maintainer,
  2026-11-12: "5 ms es totalmente aceptable")*.
- **AU25.** **No fragments.** When the time left before a block is short, the
  filler chooses the last songs so that the faded part is small (at most
  `automation.max_fade_cut_secs`) and never starts a song that would play
  for less than `automation.min_song_play_secs` before its fade
  *(maintainer: the goal; the selection method is the builder's call within
  §6's classic methods)*. If no choice satisfies both, the one that cuts least
  wins and a line is logged. *(My call, plan 5: a greedy fit of known
  lengths on the filler's timeline, segues included; the queue after the
  loaded next is checked again once per song, since it drifts; see
  `docs/technical/automation.md`.)*
- **AU26.** A schedule item is never faded or cut to keep the next block on
  time *(maintainer)*. Overruns are shown at edit time (AU37) and absorbed at
  play time by AU21–AU22.

## 6. Music filler

### 6.1 The list

- **AU27.** The filler is a **list** like a player's playlist. It generates
  tracks ahead of what plays, enough to cover
  `automation.filler_lookahead_mins` or the next block, whichever comes first
  *(my call)*; before a block it queues up to the start of its fade, and a
  song still audible when the fade starts is the last one (AU25, plan 5),
  and keeps only the last `automation.filler_played_kept` played
  rows; older played rows are removed from the list (they stay in the as-run
  log, AU35) *(maintainer)*.
- **AU28.** The operator may **remove, reorder and add** tracks (drag from
  the library) in the filler's list *(maintainer)*. What the operator adds or
  moves is never removed or reordered by the filler; when the filler
  regenerates (the rotation changes, next is set by hand, the schedule is
  edited) it replaces only the rows it generated that have not played. A
  removed track does not count as played. A song added by hand is still
  music: it fades before a block like any other, and a row that cannot end
  before the coming block is flagged in the list *(my call)*.
- **AU29.** The filler can run **on its own**, without the schedule
  *(maintainer)*: its column has a player's transport (Play, Stop at end…)
  and a rotation selector, and it then plays that rotation without end.
  Starting automation while it runs hands it to the schedule (it plays
  until the next block, AU20). While it plays for the Log, stopping it
  stops automation (rework AR8–AR9). Like the Log, it never starts by
  itself.

### 6.2 Selection

Only classic, publicly documented methods: category steps, least recently
played order, simple separation rules (track library spec §12, patents).

- **AU30.** A new track kind, **Hourly ID**, joins Music, Jingle, Effect, Ad
  and Voice (track library spec §3). It changes nothing outside automation
  except where kinds are listed (menus, filters).
- **AU31.** **Track separation.** A track is eligible again when either
  `automation.track_separation_mins` have passed since it last aired **or**
  `automation.track_separation_percent` of the other tracks of its step's
  category have aired since *(maintainer: e.g. 3 h or 50 %; a 35-track
  current list repeats after about 17 songs, an archive of 3 000 is ruled by
  time)*.
- **AU32.** **Artist separation.** A track whose artist (compared trimmed and
  case-insensitively) aired within `automation.artist_separation_mins` is
  not eligible. Tracks without an artist are exempt.
- **AU33.** **Fair choice.** Among eligible tracks, the filler prefers those
  that have gone longest without airing, with some randomness among the
  oldest so the order does not become a fixed cycle *(maintainer: "not truly
  random")*. The exact weighting is the builder's call within classic
  methods.
- **AU34.** **Relaxing.** If no track is eligible, artist separation is
  relaxed first, then track separation (least recently played wins); a step
  whose category has no playable track at all is skipped with a log line. If
  the whole rotation yields nothing, the default rotation is used; if that
  yields nothing either, the filler stops after the current song and shows an
  alert in its column *(my call)*. Missing or unreadable files are never
  chosen (rule 9).
- **AU35.** **What counts as aired:** a track that started on an on-air
  output from the schedule, the filler **or any auxiliary player** or the
  cartwall *(maintainer: what really sounded)*. Pre-listen does not count.
  The library keeps one **last aired time per track** (bounded by the
  library's size); the percentage rule of AU31 is computed from those times,
  so no unbounded history is needed *(my call)*.

## 7. As-run log

- **AU36.** Every scheduled item and filler track that aired, and every
  scheduled item that was missed, is recorded with its date, scheduled time,
  actual start and outcome. Entries older than `automation.as_run_keep_days`
  are deleted *(my call: the "missed" marks need it, and a station needs to
  prove a spot aired; viewing it beyond the schedule column and exporting it
  are non-goals)*.

## 8. Screens (desktop)

Replaced by the rework spec §6–§8: the **Log** and **Music fill** columns
are players (header, transport, list, footer); the Log's list is a
playlist with a Time column and block header rows, edited live as changes
of its date (AR14–AR19); templates, the week plan and dates are managed in
Settings → Automation, and a template or a date opens in a popup with the
same list (AR20–AR22). The floating schedule editor first built here is
gone.

- **AU37.** A block whose items' total length (cue-in to cue-out) exceeds
  the time to the next block shows a **visible error** on its header, with
  by how much *(maintainer)*. It does not prevent saving *(my call)*. A
  template's last block runs to midnight, a date's to the next day's first
  block.

## 9. Running for weeks

- **AU38.** Everything that grows with time is bounded: the filler's played
  rows (AU27), the schedule held in memory (yesterday, today, tomorrow
  computed on demand), the as-run log (AU36), date overrides older than
  `automation.overrides_keep_days` (deleted), last-aired times (one per
  track). Nothing else accumulates.
- **AU39.** A long-run test drives the model and the engine (Offline
  backend, explicit time) through at least 30 simulated days with a weekly
  plan, overrides and the filler, and checks that list sizes, table row
  counts and allocations stay flat, and that every block started on time.
- **AU40.** A jump of the system clock (a correction, a manual change) makes
  the schedule recompute next and the filler's timing at once; it never plays
  the blocks the jump skipped over; after a backward jump, nothing that
  already aired plays again (the as-run log decides) *(my call)*.
- **AU41.** Automation does not watch the auxiliary players or the
  cartwall: a block starts at its time whatever they play. An operator going
  live stops automation first (AU17); a cart fired over the filler simply
  mixes, as today *(my call: the workflow the maintainer described)*.

## 10. Persistence

Rotations, templates, the week plan, date overrides, last-aired times and the
as-run log live in the SQLite database (track library spec §4), referring
to tracks and categories by id. The filler's list is saved like a playlist,
and comes back after a restart with the automation stopped (AU15). Untrusted
or damaged rows degrade to "not available" with a log line (rule 9).

## 11. Configuration

New `Config` fields (rule 4), each with a default, a range in
`Config::validate` and lenient loading. Defaults are *(my call)* unless they
came from the maintainer.

| Field | Default | Range | Meaning |
|---|---|---|---|
| `automation.late_tolerance_secs` | 120 | 0–1 800 | AU22 |
| `automation.block_fade_ms` | 2 000 | 0–10 000 | AU24 |
| `automation.max_fade_cut_secs` | 30 | 0–300 | AU25 |
| `automation.min_song_play_secs` | 45 | 0–600 | AU25 |
| `automation.filler_lookahead_mins` | 30 | 5–240 | AU27 |
| `automation.filler_played_kept` | 50 | 10–1 000 | AU27 *(maintainer: 50–100)* |
| `automation.track_separation_mins` | 180 | 0–10 080 | AU31 *(maintainer)* |
| `automation.track_separation_percent` | 50 | 0–95 | AU31 *(maintainer: 50–60)* |
| `automation.artist_separation_mins` | 30 | 0–1 440 | AU32 |
| `automation.as_run_keep_days` | 90 | 1–3 650 | AU36 |
| `automation.overrides_keep_days` | 30 | 1–3 650 | AU38 |

The remote API and MIDI (non-goals) will reach automation through the same
commands the UI sends (start, stop after current, set next), so adding them
later is a mapping, not a redesign.

## 12. Testing

- One test per rule at the lowest level that can see it: `fp-model` for the
  pure rules (AU1–AU23, AU27–AU35), with time passed in explicitly; `fp-store`
  for persistence and pruning (AU36, AU38, §10); the engine with the Offline
  backend for timing (AU24–AU26: the block's first item starts within one
  conductor tick of its start); `egui_kittest` for the columns and the editor.
- AU39's long-run test runs in CI in fast simulated time. *(My call, plan
  8: it drives the model and the store with a small fake engine, not the
  engine's real-time loop: engine timing is covered by the conductor's
  test, and thirty days at a 5 ms tick cannot run in CI time. "Allocations
  stay flat" is checked through what holds memory: every list, mark set,
  the in-memory log and the growing tables are counted and bounded, not
  the allocator itself.)*
- A forward jump of the clock does not delete the stored as-run log: the
  store's prune cutoff moves no faster than the time that really passed
  since its first save of the session. *(My call, plan 8 review.)*

## 13. Left to the builder, and open points

**Builder's calls:** how the two columns reuse the player machinery
(sources, playlist, segue); the selection weighting (AU33) and the fitting
method (AU25) within classic methods; the editor's layout; how far the
schedule column scrolls; how plans split the work. A suggested order: (1)
rotations, selection and the filler on its own (AU1–AU4, AU27–AU35), which
is useful alone; (2) the schedule model, templates and overrides
(AU5–AU14); (3) running the schedule and timing (AU15–AU26); (4) the editor
(§8.2); (5) as-run log and the long-run test.

**Patents:** the design uses only classic, publicly documented methods
(category steps, least recently played order, simple separation rules,
timed blocks). The maintainer judged that no further freedom-to-operate
review is needed (2026-11-11), replacing that requirement of track library
spec §12.

**Open points:** none.
