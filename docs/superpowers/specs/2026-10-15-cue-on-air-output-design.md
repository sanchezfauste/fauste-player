# Pre-listen on an on-air output — Design Spec

- **Date:** 2026-10-15.
- **Status:** Draft, awaiting the maintainer's review.
- **Amends:** [the main spec](2026-09-25-fauste-player-design.md) §4.6
  ("a pre-listen never goes on air"; a Cue route equal to Main means no
  CUE) and rule 26 (Cue availability),
  [the cartwall spec](2026-09-26-phase2-cartwall-settings-design.md) C9
  (cart pre-listen refused without a usable Cue route), and
  [the remote control spec](2026-10-01-remote-control-design.md) (the CUE
  operations).
- **Numbering:** rules are `CA<n>`. Each rule is testable and gets at least
  one test (§7).
- **Line numbers** are those of `master` at the date above (`8f1813b`).

---

## 1. Goal

A station with one sound card often routes Main and Cue to the same
output. Today that means **no CUE at all**: `cue_is_usable` refuses a Cue
route equal to the Main route, the CUE button is dimmed and cart
pre-listen is refused. The operator cannot even check a track while
nothing is on air.

Meanwhile a Cue route equal to **another** player's Main route (or the
cartwall's) is accepted today, and its pre-listen goes on air without any
warning.

From now on a pre-listen may use an output that carries a Main route
(an **on-air output**):

1. while nothing sounds on that output, the CUE plays at once;
2. while something sounds there, the CUE does not start by itself: the
   operator is told why and chooses **Pre-listen anyway** or **Cancel**;
3. when something starts on that output while an unconfirmed CUE plays
   there, the CUE stops at once and the operator is told why.

Nothing goes on air by itself: a pre-listen reaches a sounding on-air
output only through an explicit confirmation.

## 2. Terms

- **Holders** are as in the live settings spec: `PlayerMain(p)`,
  `PlayerCue(p)`, `CartwallMain`, `CartwallCue`.
- A holder's **output** is the device the engine placed it on
  (`live.placement`) and the channel pair of its configured route (a
  missing Main route plays on the default output, channels 1–2).
- Two outputs **overlap** when they are on the same device and their
  channel pairs share at least one channel.
- A Cue holder is **on air** when its output overlaps the output of any
  Main holder (`PlayerMain` of any player, or `CartwallMain`).
- An on-air output **sounds** when any holder whose output overlaps it has
  a cause to be busy (`live::causes`, non-empty) or is a player paused with
  a current track (the definition BR3.5 already uses). The Cue holder
  being decided is left out.

## 3. Rules (in `fp-model`)

- **CA1. On-air Cue route.** A Cue route that overlaps a Main route is
  usable. `cue_is_usable` no longer refuses a Cue route equal to Main; a
  missing Cue route, or one to a backend this machine does not have, still
  means no CUE (§4.6 otherwise unchanged). Rule 26's Cue availability and
  C9 follow: the CUE button and cart pre-listen are available.
- **CA2. Quiet output.** Starting a CUE (`ToggleCue`, `SetCue(true)`,
  `CueEntry`, cart pre-listen) on an on-air Cue holder whose output does
  not sound starts it as today.
- **CA3. Sounding output.** Starting a CUE on an on-air Cue holder whose
  output sounds starts nothing. `apply` records a **confirmation request**
  (the holder, what was asked, and the holders that sound) that the
  interface shows (§5). The request is runtime only and is dropped when
  the output stops sounding, the operator answers, or another CUE request
  replaces it.
- **CA4. Forced start.** The same commands with `force: true` start the
  CUE even on a sounding on-air output. Such a CUE is **confirmed**.
- **CA5. Something starts.** When an `apply` or `on_event` would make an
  on-air output sound (play, next, automatic transition, cart fire,
  restart, a paused player resuming) while an **unconfirmed** CUE plays on
  an overlapping Cue holder, the same step stops that CUE and records a
  **notice** naming the holder that started. A confirmed CUE keeps
  playing.
- **CA6. Remote and MIDI.** The remote API and MIDI cannot answer a
  confirmation: CA3 from them is refused with the reason `cue_on_air`
  (remote: `409`). The remote API accepts `"force": true` on the CUE
  operations (CA4); MIDI has no force.
- **CA7. Bit-perfect.** An on-air CUE on a bit-perfect device is a source
  like any other: it is resampled to the running rate and never moves the
  rate (BR3.5 already counts the player's own CUE as sounding).

## 4. Engine (`fp-engine`)

- **CA8.** The engine no longer drops a Cue route that resolves to the
  Main output (the default-output case of §4.6). It places the Cue holder
  on its device like any other holder and reports the placement in
  `live`, so CA1–CA5 also see a missing Main route that resolves to the
  same device as the Cue route.
- Stopping a CUE under CA5 is an ordinary plan step; nothing new runs on
  the device callback.

## 5. Interface (`fp-app`)

- **Confirmation.** A modal popup for the pending CA3 request: *"This
  pre-listen would go out on the on-air output, where Player 2 is
  playing. It would be heard on air."* with **Pre-listen anyway** and
  **Cancel**. Cancel is the default (Escape and Enter cancel); Pre-listen
  anyway sends the request again with `force: true`. The same popup serves
  a player's CUE button, the cue window and cart pre-listen.
- **Notice.** CA5's notice shows as a short non-modal notice, like the
  existing ones: *"Pre-listen stopped: Player 2 started on the same
  output."*
- **CUE button.** No longer dimmed for a Cue route equal to Main. On an
  on-air Cue holder its tooltip says the pre-listen goes out on the on-air
  output.
- **Settings › Audio outputs.** The warning under a Cue route that overlaps
  a Main route no longer says the CUE is off: it says the pre-listen goes
  out on an on-air output and asks for confirmation while something plays
  there.
- Every new or changed string goes into all locale files.

## 6. Out of scope

- A per-route switch to forbid on-air pre-listen.
- Lowering the on-air level while a confirmed CUE plays (talkback-style
  ducking).

## 7. Testing

- `fp-model`: one test per rule CA1–CA7, including another player's Main
  on the Cue output, overlapping but unequal channel pairs (no overlap: no
  rule applies), a paused player counting as sounding, and a confirmed
  CUE surviving CA5.
- `fp-engine`: a Cue route resolving to the default output that Main uses
  is placed and reported (CA8), with the `Offline` backend.
- `fp-remote`: `409 cue_on_air` and `force: true`.
- `fp-app`: `egui_kittest` tests of the popup (both buttons, Escape), the
  notice, and the tooltip.

## 8. Plans

1. **Model:** CA1–CA7, the `force` field, the confirmation request and
   the notice.
2. **Engine:** CA8.
3. **Interface, MIDI, remote and docs:** §5, CA6, the user guide (audio
   outputs, CUE, cartwall), the remote API reference, and the technical
   docs.
