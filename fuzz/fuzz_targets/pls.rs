#![no_main]
//! Any bytes read as a .pls playlist must parse without panicking.

use std::path::Path;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = fp_store::playlist_io::parse_playlist(
        data,
        Path::new("/lists/fuzz.pls"),
        &fp_model::Limits::default(),
    );
});
