#![no_main]
//! Any bytes read as config.json, playlists.json, session.json or
//! carts.json must load (or be refused) and restore without panicking.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    fp_store::fuzz_documents(data);
});
