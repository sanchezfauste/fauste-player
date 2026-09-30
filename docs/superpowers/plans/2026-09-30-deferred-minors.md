# Deferred minors: settlement

**Goal:** close every minor that earlier final reviews deferred. Each one is fixed test-first, found already fixed, or kept with a ruling.

**Branch:** `fix/deferred-minors-1`. There is one commit per area: model/store/meter/waveform, engine, backends, decode, app, and release.

## Fixed (each with a test that failed first)

| Area | Minor | Test |
|---|---|---|
| Meter | The maximum did not restart on a start of the same entry within one tick, or on a segue into itself. It now restarts on every `StartCurrent`, `Crossfade` and `TransitionStarted`. | `the_meter_maximum_restarts_when_the_entry_restarts_within_one_tick` |
| Waveform | The memo kept the last track alive after the player unloaded. It now uses a stable id per player. | `a_player_that_unloads_lets_go_of_its_waveform` |
| Analysis | The cache sweep ran at start-up. It now runs on the analysis pool. | `the_pool_sweeps_old_cache_entries_off_the_callers_thread` |
| Model | Repeated playlist or entry ids in a hand-edited `playlists.json`. | `restore_gives_repeated_playlist_and_entry_ids_new_ones` |
| Model | A NaN volume silenced the player. It is now ignored. | `volume_is_clamped_and_nan_keeps_the_volume` |
| Model | `UpdateConfig` was not validated, and a lower `max_players` was not applied. | `a_config_update_is_validated_…`, `a_lower_player_limit_is_refused_…` |
| Store | Two quarantines in the same second overwrote each other. | `two_quarantines_in_the_same_second_keep_both_files` |
| Store | There was no retry on Windows sharing violations. | `a_briefly_locked_file_is_retried` |
| Engine | A transition to a next that was not buffered yet started as underrun silence. It now starts when ready. | `a_transition_to_a_next_not_ready_yet_starts_it_when_ready_without_underruns` |
| Engine | `CueEnded` had no entry and `CartCueEnded` no cart, so a stale end cleared a newer cue. | `a_stale_cue_end_does_not_end_a_newer_cue`, `a_stale_cart_cue_end_does_not_end_a_newer_cue` |
| Engine | A refused rate was retried on every idle start, and the shared fallback was taken on a refused rate. | `a_refused_rate_is_not_tried_again_on_every_start` |
| Engine | Volume smoothing and the ready threshold were sized at the configured rate. | `volume_smoothing_keeps_its_duration_when_the_rate_changes`, `the_ready_threshold_is_the_same_time_at_any_source_rate` |
| Engine | The meter's five sums were taken one by one. They are now taken in whole blocks. | `a_block_rendered_during_a_take_is_taken_whole`, `a_later_measurement_of_the_same_source_adds_its_frames` |
| Engine | `BusShared::consistent` could spin forever. | `a_render_that_never_finishes_does_not_hang_the_reader` |
| Engine | A failed reconnection must keep the virtual clock running (pinned). | `a_failed_reconnection_keeps_the_virtual_clock_running` |
| Engine | A dropped command logged its whole payload. | `a_dropped_command_is_logged_briefly` |
| Engine | Conductor tests slept 30 ms. They now settle on the workers. | (the tests themselves) |
| Backends | `OutputStream::config` reported the requested buffer size. The WASAPI `Initialize` reason was lost. | `the_reported_buffer_size_is_the_one_in_use` |
| Decode | Opus seeks had no 80 ms pre-roll, and Vorbis seeks landed a packet late. | `a_seek_pre_rolls_so_the_first_frame_is_already_right`, `a_vorbis_seek_lands_on_its_frame` |
| Decode | A seek past the end was an error with symphonia. | `seeking_past_the_end_is_the_end_for_every_decoder` |
| Decode | A WavPack seek assumed the first `block_index` was 0. | `seeking_counts_from_the_first_block_whatever_its_index` |
| Decode | DSD512 5.1 cost 41 % of a core and tiny DSF blocks were slow. Both now cost about 16 %. The per-chunk copies are gone. | `tiny_dsf_blocks_decode_like_the_usual_ones`, `dsd_surround_decoding_speed` (ignored measurement) |
| App | A shortcut key also acted on the focused widget (Space, Enter, repeats), and Tab or an arrow moved the focus. | `a_shortcut_key_does_not_also_press_the_focused_button`, `a_shortcut_on_tab_does_not_move_the_focus` |
| App | Import and export errors were shown in the system's English. | `an_import_failure_is_explained_in_the_interface_language` |
| App | The whole file was read before the size check. | `a_file_over_the_limit_is_refused_without_reading_it_all` |
| App | Empty carts had no context menu. | `an_empty_cart_can_be_edited_from_its_menu` |
| App | Every export button had the same accessible name. | `each_playlist_export_button_names_its_playlist` |
| App | `wave_menu` was never cleared. It is now forgotten once the menu closes (covered by the marker menu tests). | |
| Services | An analysis result lost to a panic left the track unanalysed until a restart. | `a_result_lost_to_a_panic_is_asked_for_again` |
| Analysis | The ID3 `TXXX:INTRO` lookup was case-sensitive. | `the_id3_intro_description_is_matched_without_regard_to_case` |
| Playlists | Latin-1 percent-encoding, `#EXTINF` attributes with commas, and the PLS `File1`/`File01` collision. | `a_file_url_percent_encoded_in_latin_1_is_read`, `extinf_attributes_may_hold_commas`, `pls_entries_numbered_1_and_01_are_both_kept` |
| Playlists | An exported relative path starting with `#` read back as a comment. | `an_exported_path_starting_with_a_hash_is_not_read_back_as_a_comment` |
| Fuzzing | There was no seed corpus. `fuzz/seeds/` is now versioned, and every target was run with it. | |
| Release | appimagetool and flatpak-cargo-generator were not pinned. There was no release note for unsigned installers. | release dry run in CI |

Tests that pin paths that already worked: `a_32_bit_float_wavpack_decodes_exactly` and `opus_in_matroska_drops_its_codec_delay_like_ogg`.

## Found already fixed

- The `fail_next_frame` hook is behind `test-hooks`.
- The services step runs in `catch_unwind`.
- `row_status` no longer searches playlists.
- Dropped folders expand on a helper thread.
- Esc cancels a playlist rename (`escape_cancels_a_playlist_rename`).
- `demo_session` refuses to run without `FAUSTE_HOME`.
- Opus `INTRO` is covered by `an_intro_tag_is_read_from_opus`.
- There is no `error-other` any more.
- The extension list is covered by the audio formats spec. The main spec §6 now points there.
- Marker order: `set_marker` keeps cue-in before cue-out, and `plan_for` ignores a segue outside the cue range.

## Rulings

Each ruling gives what was decided, why, and the cost if it is wrong.

- **Idle players may take an entry on air elsewhere as next.** No change. Spec §3 rule 22 allows the same entry on air on several players at once. Cost if wrong: a model rule.
- **The crossfade peak is the maximum of the sources, not the peak of their sum.** No change. The peak of the sum needs every meter (peak, true peak, integrator, K-weighting) per player on the real-time path, instead of per source. Cost if wrong: a meter reading about 3 dB low during an overlap of two loud, in-phase sources.
- **Opus is decoded through 16-bit PCM inside `opus-decoder`.** No change. The float decoder (`opus-pure`) uses `unsafe`, which the audio formats spec rules out for untrusted files. The quantisation noise (about −96 dBFS) is far below Opus's coding noise. Cost if wrong: audible only with a large positive header gain.
- **The conductor clones `AppState` once per tick, when something changed.** No change. The rate is bounded by the tick. Cost if wrong: a large library costs a copy per changed tick.
- **The test tone is synthesised on the conductor.** No change. 1.5 s takes a few milliseconds, even at 768 kHz, against a `schedule_lead_ms` of 200. The slot cannot leak: Attach and Start are sent only when both fit. Cost if wrong: a late tick while a tone starts.
- **One cartwall worker decodes every cart.** No change. Each cart's ring holds `tuning.prebuffer_secs` (5 s), which absorbs a slow read. Cost if wrong: looped carts on storage stalling longer than 5 s.
- **A marker change on a playing cart applies from its next fire.** Documented known limit. Cost if wrong: a running cart ends at its old cue-out.
- **The MSI Start menu component keeps an HKCU key path.** ICE38 requires HKCU for Start menu components, and the shortcut is not advertised, so no self-repair runs for other users. Cost if wrong: a second user triggers a repair.
- **Unit symbols (ms, dB, dBFS, LUFS) are not Fluent messages.** They are the same in every language. Decimal separators are a separate matter. Cost if wrong: a locale that writes a unit differently.
- **The validation ceilings `max_players` 256 and `fade_ms` 10 s stay.** Rule 4 requires every config field to have a documented range. Cost if wrong: an operator needing more has to edit the code.
- **A misrouted slot counts as audible for the BP badge.** Conservative: the badge turns off. Cost if wrong: a false "not bit-perfect".
- **Hog mode is reused if this process already holds it.** One bus per device makes a second guard impossible, and this is commented. Cost if wrong: none today.
- **`availability()` enumerates devices at start-up.** It does so on a helper thread, once. Cost if wrong: slower start-up with many devices.
- **The DIN test checks its own definition.** IEC 60268-10 is not available. Cost if wrong: a DIN meter off the standard.
- **The DIN −10 mark gives way to the alignment tick at the column size,** under the legibility rule. Cost if wrong: one mark fewer.
- **The fader sends one command per changed value,** with duplicates removed. This is intended. Cost if wrong: more commands during a drag.

## Final review (fresh reviewer) and its fix pass

- **Critical, fixed:** a next that was waiting for its file survived a stop, a seek or a pause, and went on air by itself once the file was read (rule 10). A transition taken back in that state promoted a next that had never been started, which left the air silent. `undispatch` now returns a waiting next to idle, and counts a transition as executed only once the next was sent its `Start`. A waiting next starts only while its transition stands. Tests: `a_next_waiting_for_its_file_never_starts_after_a_stop` and `a_pause_while_the_next_waits_takes_the_transition_back`.
- **Important, fixed:** a next that became ready before its frame started early, summed with the current track. It now starts at the dispatched frame at the earliest (`a_next_ready_before_its_frame_still_starts_on_it`).
- **Important, reverted:** stopping the virtual clock before each reconnection attempt stalled the timeline for as long as each attempt took. The clock runs again while the device opens.
  - Ruling: the stand-in and the reopened stream may both render one block, once per reconnection — a lost timeline for the whole open is worse — cost if wrong: the timeline runs one period fast after a reconnection.
- **Minors, fixed:**
  - playlist id repairs are reported as load warnings (`repeated_playlist_ids_are_repaired_with_a_warning`);
  - the sharing-violation retry runs on Windows only and covers errors 32 and 33 (`only_a_held_file_is_retried`);
  - volume smoothing no longer drifts over chains of rates;
  - the pre-roll needs a time base;
  - a seek to 0 after reading (or past the end) repositions the reader;
  - the empty-cart menu area has its own accessible name;
  - the shortcut filter matches the whole chord;
  - other I/O errors keep the system's message;
  - the macOS unsigned note also needs notarisation.
- Ruling: the bounded waits (50 ms) are per source when a device thread hangs mid-block — the watchdog declares the bus lost within `watchdog_timeout_ms` anyway — cost if wrong: a few slow conductor ticks before that.
- Ruling: the Latin-1 fallback of a `file:` URL is all-or-nothing — a URL that mixes UTF-8 and Latin-1 escapes is broken either way — cost if wrong: one garbled name.

