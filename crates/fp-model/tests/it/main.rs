//! The integration tests of `fp-model`, gathered in one binary: every test
//! file is a module here, so the crate's tests link once instead of once
//! per file. Tests that install process-wide state (a panic hook, a global
//! tracing subscriber) keep their own binary next to this directory.

mod analysis;
mod availability;
mod bit_perfect_rules;
mod bitrate_pass;
mod cartwall;
mod cartwall_props;
mod column_widths;
mod common;
mod create_playlist_in;
mod cue_markers_off;
mod cue_on_air;
mod cue_output;
mod cue_window;
mod device_offer;
mod device_overrides;
mod dsd_config;
mod dsd_rules;
mod editing;
mod entry_flags;
mod entry_notice;
mod header_duration;
mod history;
mod intro_column;
mod layout;
mod live;
mod loudness;
mod loudness_config;
mod markers;
mod meter;
mod midi;
mod on_air;
mod pending_start;
mod playlist_props;
mod playout;
mod replay_next;
mod reset_played;
mod restore;
mod reuse_tracks;
mod review_fixes;
mod session;
mod set_values;
mod shortcuts;
mod single_stop_after;
mod table_columns;
mod tag_cover;
mod tag_sheet;
mod track_tags;
mod transitions;
mod transport;
mod volume;
