# Automation

Automation keeps the station on air when nobody is in the studio. This
version has its first part, the **music filler**: a player that chooses
music by itself, from rules you set, and never stops for lack of a list.
The players you already use (P1 … Pn) are untouched: they stay yours for
shows you prepare and play live.

## The music filler

The music filler is the **Music** column. It is hidden until you show it:
click the **Music** tile on the left of the top bar. It sits left of P1.

It looks and works like a player: the big clock, the waveform, the
transport buttons, the fader and the track list. The difference is the
list:

- it **fills itself** from a **rotation** (below), always about half an
  hour ahead (*look-ahead*, in Settings → Automation);
- it **trims itself**: it keeps the last 50 tracks it played and removes
  older ones, so it can run for weeks;
- you can still **change it by hand**: drag tracks from the library,
  reorder rows, remove a song you do not want. A row you add is marked
  with a hand and is never replaced; a row you remove does not count as
  played.

Its outputs are set in Settings → Audio outputs, on the **Music** row, as
for any player. It has no number, no keyboard shortcuts or MIDI bindings,
and remote control does not see it.

Press **Play** to start it and **Stop after current** to let the song on air
end and stop. Like every player, it never starts by itself, not even after
a restart.

Above its list, where a player has its playlist tabs, the Music column
shows the **rotation** it plays. Choose another one there: the songs it had
queued are replaced, the ones you added stay. When the rotation can find
nothing to play (its categories are empty, or every file is missing), the
bar says so.

## Rotations

A **rotation** is a short list of **steps** the filler plays in turn, then
again from the first. Each step names where its tracks come from (a
category of the library with its subcategories, or every track of a kind)
and how many it plays each time, as a range:

| Step | Plays |
|---|---|
| Current hits | 2–3 |
| Station IDs | 1 |
| Gold | 1 |

With a range, a station ID comes after two songs one time and after three
the next. One rotation is the **default**: a new installation has one,
**Music**, which plays all your music.

## How it chooses

Within a step, the filler prefers the tracks that have gone longest
without airing, with a little chance among them so the order never becomes
a fixed cycle. It also keeps apart:

- **the same track:** it may play again once 3 hours have passed or half
  of the other tracks of its step have aired since, whichever comes first;
- **the same artist:** not within 30 minutes.

Everything that went on air counts, from the filler, a player or the
cartwall; pre-listening does not. When a small category cannot keep the
rules, the filler gives way rather than fall silent: first on the artist,
then on the track. Settings → Automation has every value, and is where
you make, rename and change rotations (see
[Settings](settings.md#automation)).

An **Hourly ID** is a track kind of its own (Library → Kind ▸ Hourly ID),
for the station's identification at the top of the hour.
