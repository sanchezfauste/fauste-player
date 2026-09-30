# Real-music corpus

Put audio files here (any format the player reads) to run the opt-in tests
and the marker report on real music. Everything in this folder except this
README is git-ignored: the files never leave your machine.

A useful corpus has a few dozen tracks of varied genres and masters: loud
and quiet, hard endings and long fades, soft or silent intros, spoken word.
Sub-folders are read too. `FAUSTE_TEST_MUSIC=<dir>` points the tools at
another folder instead.

```sh
# Invariants on every file (trimming never cuts audio, overlaps stay short)
cargo test --release -p fp-analysis --test real_music -- --ignored
# One row per file: cue in/out, segue, overlap, outro; tune with --set
cargo run --release -p fp-analysis --example marker_report
cargo run --release -p fp-analysis --example marker_report -- --set segue_drop_db=12 --set segue_max_secs=3
```
