#![no_main]
//! Any bytes read as a cart page file must parse (or be refused) without
//! panicking, and a parsed page must import into the model.

use std::path::Path;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let limits = fp_model::Limits::default();
    if let Ok(import) =
        fp_store::playlist_io::parse_cart_page(data, Path::new("/pages/fuzz.cartpage.json"), &limits)
    {
        let mut state = fp_model::AppState::new(fp_model::Config::default(), "Main");
        let _ = fp_model::apply(
            &mut state,
            fp_model::Command::ImportCartPage(Box::new(import)),
        );
    }
});
