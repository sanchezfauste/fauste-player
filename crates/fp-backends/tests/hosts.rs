#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Phase 3: one backend per audio system, and the preferred default.

use std::collections::HashSet;

use fp_backends::{Availability, display_name, preferred_backend, system_backends};

#[test]
fn each_host_is_a_backend_with_its_own_id() {
    let backends = system_backends();
    assert!(!backends.is_empty());
    let ids: HashSet<String> = backends.iter().map(|b| b.id().0).collect();
    assert_eq!(ids.len(), backends.len(), "unique ids");
    assert!(
        ids.iter()
            .all(|id| id.chars().all(|c| c.is_ascii_lowercase()))
    );
    for b in &backends {
        if let Availability::Unavailable(reason) = b.availability() {
            assert!(!reason.is_empty(), "{} gives a reason", b.id().0);
        }
    }
}

#[test]
fn the_default_backend_follows_the_preferred_order() {
    let linux = [
        ("alsa", true),
        ("pulseaudio", true),
        ("pipewire", false),
        ("jack", true),
    ];
    assert_eq!(preferred_backend(&linux, "linux"), Some("pulseaudio"));
    let linux_pw = [("alsa", true), ("pipewire", true), ("pulseaudio", true)];
    assert_eq!(preferred_backend(&linux_pw, "linux"), Some("pipewire"));
    assert_eq!(preferred_backend(&[("alsa", true)], "linux"), Some("alsa"));
    assert_eq!(
        preferred_backend(&[("asio", true), ("wasapi", true)], "windows"),
        Some("wasapi")
    );
    assert_eq!(
        preferred_backend(&[("jack", true), ("coreaudio", true)], "macos"),
        Some("coreaudio")
    );
    assert_eq!(preferred_backend(&[("alsa", false)], "linux"), None);
    // An unknown system is still usable when it is the only one.
    assert_eq!(
        preferred_backend(&[("other", true)], "linux"),
        Some("other")
    );
}

#[test]
fn backends_have_readable_names() {
    assert_eq!(display_name("pipewire"), "PipeWire");
    assert_eq!(display_name("pulseaudio"), "PulseAudio");
    assert_eq!(display_name("jack"), "JACK");
    assert_eq!(display_name("coreaudio"), "Core Audio");
    assert_eq!(display_name("mystery"), "mystery");
}
