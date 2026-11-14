# Automation rework — Design Spec

Date: 2026-11-13. Status: approved by the maintainer (2026-11-13).

This spec **revises** the automation spec
(`2026-11-11-automation-design.md`, "the automation spec"). It replaces
its terms (§3), how the schedule is started, stopped and pointed (§5.1),
its screens (§8) and the editor's place. Everything else there stands:
rotations and selection (§4.1, §6.2), blocks, templates and overrides as
data (§4.2–§4.3), timing on air (§5.2–§5.3), the filler's list rules
(§6.1), the as-run log (§7), running for weeks (§9), persistence (§10),
configuration (§11). When this spec is built, the automation spec is
edited to match it (its rules rewritten in place, as other specs here
are), so there is again one binding text.

Rules are numbered **AR1…** here; `AUn` refers to the automation spec.

## 1. Intent

Automation was built and tried. The maintainer's verdict: it works, but it
does not feel like part of Fauste Player. "It looks like a patch on top, as
if two teams that don't talk had programmed it." The schedule column has
its own buttons (Start, Stop after current, a Running label) instead of a
player's; its list is not a playlist; the editor is a separate window with
its own look; the filler has two names ("Music" in the top bar and the
column, "Musicalizador" in Settings).

The operator's real night, as the maintainer described it:

1. They arrive; automation is on air: the filler plays music, the schedule
   waits for the next block.
2. At the hour the hourly ID plays. They mark **Stop at end** on the
   schedule column, the same Stop at end every player has.
3. After the ID automation is silent; they do their show on the players
   (show opener, beds, tracks…).
4. Time for the spots: they select the first item of the spot block in the
   schedule and press **Play** on the schedule column. The block plays; they
   stop again.
5. When the show ends they press Play and the station runs by itself again.

**The principle:** the schedule column and the filler column **are
players**. Everything an operator knows from a player (the transport
buttons, Stop at end, next, the playlist table, drag from the library, the
row menu) works the same there. Nothing automation-specific is learnt
except what has no player equivalent (blocks and their start times,
category tracks, music fill items).

### What success looks like

- An operator who knows the players runs automation the first time without
  being told how.
- Steps 2–5 above take one or two clicks each.
- The schedule column, the filler column, the template popup and the
  settings section are visually indistinguishable in style from the rest of
  the app (same table, row heights, colours, menus, buttons).

### Anti-goals (added to the automation spec's)

- Automation-only buttons or states where a player's would do.
- A second look: any widget, colour, row style or window the rest of the
  app does not already use, unless nothing existing fits.
- Two names for one thing anywhere in the UI or the docs.

## 2. Scope

**Goals:** §3–§7 on the desktop.

**Non-goals:** Android screens, remote API and MIDI commands for automation
(still the automation spec's non-goals; the transport now being a player's
makes mapping them later easier), changes to selection, timing, the as-run
log or persistence formats beyond what §6 needs.

## 3. Terms (replaces the automation spec §3)

The maintainer asked to follow the radio-automation sector's usual terms.
Proposed in the brainstorming and accepted. UI strings use these; code
identifiers may keep their current names (`schedule`, `filler`,
`time_critical`) *(my call: renaming code is churn with no user value)*,
but docs use the UI terms.

| Concept | English UI | Español UI | Was |
|---|---|---|---|
| The whole system | Automation | Automatización | same |
| A date's programme, played by its column | Log | Parrilla | Schedule / Parrilla |
| A reusable day | Template | Plantilla | Day template / Plantilla |
| Which template each weekday uses | Week plan | Semana tipo | same |
| A date's changes to its template | Date changes | Cambios del día | Date override / Cambio del día |
| A group of items with a start time | Block | Bloque | same |
| Menu entry to add one | Block… | Bloque horario… | Añadir bloque |
| One row of a block | Item | Elemento | same |
| A fixed track | Track | Pista | same |
| One track chosen from a category when it airs | Category track | Pista de categoría | "Una de {cat}" |
| "Music until the next block" item | Music fill | Musicalizador | Rotation item / "Música: {name}" |
| The player that plays music between blocks | Music fill | Musicalizador | Music (column, top bar) / Musicalizador (Settings) |
| Ordered category steps | Rotation | Rotación | same |
| Must air at its time or not at all | Hard start | Hora fija | Time-critical / Puntual |
| Did not air | Missed | No emitido | same |
| What aired | As-run log | Registro de emisión | same |

- **AR1.** The filler has one name everywhere: the top bar's toggle, its
  column header, Settings, the user guide, error messages
  (`error-filler-playlist` says "Musicalizador", not "columna Música").
- **AR2.** The en-US "Log" may read ambiguously next to "As-run log"; the
  two never share a screen today. If a plan finds they do, the plan asks the
  maintainer instead of choosing another word.

## 4. Transport: the Log column is a player (replaces AU15–AU17, AU29)

The Log column shows a **player header exactly like P1–P4** (title, clock,
waveform, transport buttons, Stop at end, status), driven by the same code
*(builder's call how; the column is already a `PlayerState`)*. Its strip
under the header loses Start, Stop after current, the Running/Stopped label
and the Edit button.

- **AR3. Running = the Log column's player is Playing.** Automation runs
  exactly while the Log's transport is Playing (a music fill row on air
  counts as Playing), and is stopped otherwise, **paused included**
  *(maintainer: pause, stop, fade stop and Stop at end all stop
  automation)*. While paused, blocks whose start passes are missed as when
  stopped (AU21). There is no separate running flag the operator sees;
  the column's status (playing, paused, stopped, the "Stop at end" badge)
  is the automation's state.
- **AR4. Play** on the Log plays the row marked next (AR8) and starts
  automation. If that row is a block item, the block plays from it; if it
  is a music fill row, the filler starts with that rotation and the next
  block starts on time (AU17's "Start at 10:58 → two minutes of music"
  case). Play while something is on air does what Play does on a player
  (go to next with the crossfade) *(my call: it is the operator's explicit
  action; AU26's "never cut a scheduled item" is about automatic cuts)*.
- **AR5. Pause, Stop, Fade stop** do what they do on a player to whatever
  is on air **for the Log**, and stop automation. "On air for the Log"
  includes the filler's song while a music fill row is current: Stop on the
  Log while music plays stops the music. Pause then Play resumes the
  paused audio and runs automation again; while paused, next stays where
  it is (AR10), and the Play that resumes gives a next the clock had
  chosen back to the clock if it is still next, so the next block keeps
  its time (AR9) *(my call, final review of the minors)*. Play during the Log's fade stop does nothing, as on a
  player *(my call, deferred minor of plan 3)*.
- **AR6. Stop at end** is the player's own toggle and badge. On the Log it
  means: what is on air for the Log (an item, or the filler's current song)
  ends, then automation stops and nothing else plays. Unmarking it before
  the end cancels it. **A block start comes first** *(maintainer: "unless
  automation is stopped by 23:00, automation goes on")*: if a block's
  hand-over (AU24) arrives while the filler's song for the Log is still on
  air, the block starts on time as usual (the song fades) and Stop at end
  moves to the block's first item: automation stops when that item ends.
  If the song ended before the hand-over, automation had already stopped
  and the block is missed as usual.
- **AR6b. SINGLE and CONT on the Log** *(maintainer)*. The Log has the
  player's SINGLE/CONT selector. CONT is automation as described here.
  SINGLE is **manual**: Play plays the next row, it ends, next moves to the
  row below and automation stops; no block starts by itself and the filler
  is not started for a music fill row reached by order (Play on a music
  fill row starts it). Stop at end follows the players' SINGLE rule.
- **AR7. The Music fill column is a player too**, with the same header.
  Its Play starts the filler on its own with the rotation chosen in its
  strip (AU29), also after the Log paused it during music: the song is
  then no longer the Log's to resume or stop *(my call)*; its Pause, Stop, Fade stop and Stop at end act like a
  player's. **Stopping the filler leaves silence**: nothing else starts by
  itself. If the filler was playing for the Log (a music fill row
  current), stopping it also stops automation *(my call: otherwise the
  next block would start later by itself out of a silence the operator
  chose, which is the surprise this rework removes)*. **Stop at end on the
  Music fill column never passes to the Log** *(maintainer)*: it stops the
  filler when its song ends, like a player. If a block's hand-over comes
  first while the filler plays for the Log, the song fades as usual, the
  filler's Stop at end mark is cleared and automation goes on with the
  block. Its strip keeps only
  the rotation selector.
- **AR7b. The two columns together** *(my calls)*. Play on the Log while
  the filler plays on its own hands over as before a block: the filler's
  song fades over `automation.block_fade_ms` and the Log's next starts.
  Play on the Music fill column while a Log item is on air starts the
  filler at once and the two mix, as any two players do (AU41); it does not
  stop the Log.
- Unchanged: automation is stopped after every start, restart or crash and
  nothing goes on air until the operator presses Play (AU15, rule 10).
- Shortcuts, the remote API and MIDI keep addressing P1–P4 only (non-goal);
  the Log and Music fill players are not numbered.

## 5. Next: what the operator selected (replaces AU16 and part of AU18)

- **AR8.** The Log has one row marked **next**, shown and set exactly as on
  a player's playlist (the same marker, the same "Set as next" menu entry
  and gesture). Play plays it, whatever its time *(maintainer: "lo que tenga
  seleccionado")*.
- **AR9.** When a row ends, next moves to the row below, as in a playlist
  in continuous mode. At a block's end the automation spec's AU19–AU20 still
  apply while running (music fill, then the next block by the clock;
  blocks in between missed).
- **AR10. While stopped, next stays where it is.** It does not follow the
  clock (this removes AU16's "next follows the clock while stopped" and
  `next_by_hand`). Example: stopped after the 23:00 ID at 23:00:20, Play at
  23:58 plays the row after the ID.
- **AR11.** A next whose block's start is more than the late tolerance in
  the past is shown as **late** (its time in the warning colour the players
  use for the end warning) *(my call)*. It still plays if the operator
  presses Play; its hard-start items play too (AU18: a block played by hand
  plays in full).
- **AR11b.** Next stays on its row even when that row becomes missed while
  stopped. The row menu keeps a **Back to the clock** entry (today's
  `schedule-clear-next`) that sets next to what the clock gives, so after
  a long show the operator reaches "what is due now" in one action
  *(my call)*.
- **AR12.** At application start, next is what the clock gives (the old
  AU16 rule, once) *(my call)*; nothing plays until Play.
- **AR13. The filler column's next** behaves as a player's playlist next
  too: selecting the **last** row as next must still let the filler
  generate more rows after it, and generated rows must appear **after**
  next without moving the next marker or taking it *(maintainer's bug
  report: "al seleccionar la última pista no genera más hasta que no
  termina, y cuando lo hace no marca correctamente como siguiente las
  nuevas")*. A test reproduces both before the fix.

## 6. The Log column's list is a playlist (replaces AU23's display and §8.1)

- **AR14.** The list uses **the playlists' table** (the same widget, row
  height, cells, column set, status colours, drag feedback, footer) with
  one addition: a **block header row** showing the block's start time and
  its total length, plus the overrun error of AU37 when it applies. Past
  rows look like played rows on a player; missed rows are struck through
  (AU23); a hard-start item shows a small clock icon *(my call; from the
  icon set already in use)*.
- **AR14b. Mock-ups approved** *(maintainer, 2026-11-13; Claude Design
  `Automatización v2.html`)*, with: a **Time** column (each row's planned
  start) on the Log only, not on the Music fill, which also runs on its
  own without automation and keeps its rotation selector (AU29); no
  "Today's changes" button: live editing is enough for today.
- **AR15. Category tracks are visible** *(maintainer: "¿Dónde se ve este
  track?")*. A category track row shows the category until its track is
  chosen, then the chosen track like any track row with the category as a
  secondary line or cell; after airing it shows what aired. The track is
  chosen when the block is loaded for play *(today's behaviour, kept)*;
  the builder may choose it earlier (for example when the block becomes
  next) if that is cheap, so the operator sees it before Play.
- **AR16.** A music fill row shows "Music fill · *rotation name*"; while
  it is current it is shown as on air (the filler plays for it), and the
  filler column's header shows the song.
- **AR17. Live editing** *(maintainer)*. The Log's list is edited like a
  playlist: drag tracks from the library (files dropped from the system
  are a later step: plan 4 left them out), reorder by drag, remove, and an
  **Add** menu with *Track…*, *Category track…*, *Music fill
  (rotation)…*, *Block…*. The row menu also
  has the item's *Hard start* toggle and, on a block header, *Change
  time…* and *Remove block*. Every edit is saved at once as **date
  changes** of the date the row belongs to (AU11–AU12; the template is not
  touched). Editing a template is only done in Settings (§7).
- **AR18.** A block may be added at any time not taken by another block of
  that date, from the Add menu anywhere in the list; its time is asked in
  a small popup with the time field the app already uses. *(Maintainer's
  bug report: after adding a 23:00 block there was no way to add 23:30 or
  23:45.)* A test covers adding blocks after the last one of a day.
- **AR19.** What may not be edited live *(my call)*: rows of past dates
  (yesterday), the row on air (as on a player), and rows already played.
  A Music fill item stays the last item of its block (the automation
  spec's model rule); adding one where another exists replaces it.
- **AR19b. Where a drop or an added row goes** *(my calls; plan 4
  ruling)*. Before the row it lands on, in that row's block, or after it
  when it lands on the row's lower half (as on a playlist), so a block's
  end takes a drop too; on a block header: at the block's start; never
  after a Music fill row. A row dragged onto another date's rows does not
  move (no insertion line is shown there). Add puts its item at the end
  of next's block (before its Music fill row), never in a block that
  already played. A refused edit changes nothing. Editing the
  upcoming rows of the block on air takes effect at once (the Log
  player's queue is reloaded after the row on air). A live edit stores
  the whole block as changed for that date (AU12, unchanged): later
  template edits no longer reach that block on that date, which is
  acceptable because the change is for one date only.

## 7. Templates and dates in Settings (replaces §8.2)

- **AR20.** **Settings → Automation** holds everything that is not today's
  Log: the rotations (as today), the **templates** list (new, duplicate,
  rename, delete), the **week plan** (a template per weekday), and the
  **dates** with changes or another template (a calendar, as today's
  Days tab), where a date can use another template. The floating schedule
  editor window and the Log column's "Edit log" button are removed.
- **AR21.** Selecting a template, or a date, opens a **popup window that
  is the same playlist editor as the Log column** (§6, AR14–AR19): block
  headers, items, the Add menu, drag from the library (files from the
  system later, as on the Log, AR17), reorder, remove. For a date it edits
  that date's changes and shows changed blocks as "changed for this day"
  with "Back to the template" in the block header's menu; the date's
  template is chosen in Settings (AR20). Settings is a modal
  dialog, so opening the popup **closes Settings** and the popup is a
  non-modal floating window like the library, which can stay open while
  tracks are dragged into it; closing it leaves the operator where they
  were *(my call)*.
- **AR22.** The popup and the Log column share the list code *(builder's
  call how)*; anything that works in one works in the other, except
  playing (the popup does not play; its rows have no next). Today's
  popup shows its rows as the Log does (aired, on air, next, missed);
  a template's and another day's rows are all to come. A date's popup
  closes when its date has passed (midnight), and a popup's open dialog
  closes with it *(my calls, final review of plan 5)*.
- **AR22b.** A template edit to a block that is on air today, while today
  follows the template for that block, first keeps that block as it was
  loaded as a change of today (AU12); the template's change reaches it
  from the next day. So the Log's rows always match what plays
  *(my call, from the final review of plan 5)*.

## 8. Look and feel

- **AR23.** Before the UI plans, mock-ups of the Log column, the Music
  fill column and the template popup are made in the Claude Design project
  (Nocturne design system, `Reproductor v3.dc.html` as reference) and
  approved by the maintainer. The mock-up shows them next to a player, so
  any difference is visible.
- **AR24.** No new colours, fonts or widget styles; existing theme
  constants and widgets are reused. Every new or changed string is a
  Fluent message in every locale (en-US and es-ES written with the change).

## 9. Unchanged

Rotations and selection (AU1–AU4, AU30–AU35), blocks, items, templates,
week plan and date changes as data (AU5–AU14), timing (AU19–AU26),
the filler's list rules (AU27–AU28), the as-run log (AU36), running for
weeks (AU38–AU41), persistence (§10; date changes made live use the same
tables), configuration (§11; no new fields expected — if one is needed it
follows rule 4).

## 10. Testing

- `fp-model`: one test per rule AR3–AR13, AR17–AR19 (transport, next,
  live edits as date changes), including the two reported bugs (AR13,
  AR18) reproduced first.
- `egui_kittest`: the Log and Music fill columns show a player header and
  the playlist table; the Add menu; the template popup opens from Settings
  and accepts a dropped track.
- The engine's timing tests and the long-run test (AU39) still pass
  unchanged.

## 11. Documentation

The automation spec is rewritten to match (§ intro); the user guide's
automation pages, `docs/technical/automation.md`, README features and the
screenshots (the screenshot scene adds the two columns only if it already
shows them) follow. Locales: the renamed strings in every file.

## 12. Left to the builder

How the two columns reuse the player header and the playlist table; how
the list code is shared between the column and the popup; the order of the
plans. A suggested order: (1) the two bugs (AR13, AR18) on the current UI,
small and shippable alone; (2) terms (§3) in strings and docs; (3)
transport and next (§4–§5) in the model, then the headers; (4) the
playlist-table list with live editing (§6); (5) Settings and the popup
(§7), after the mock-ups (AR23). Plans 1–3 may be written and built
before the mock-ups exist; plans 4–5 are written once they are approved
*(my call)*.
