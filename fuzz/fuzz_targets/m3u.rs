#![no_main]
//! Any bytes read as a .m3u playlist must parse without panicking.

use std::path::Path;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = fp_store::playlist_io::parse_playlist(
        data,
        Path::new("/lists/fuzz.m3u"),
        &fp_model::Limits::default(),
    );
});
