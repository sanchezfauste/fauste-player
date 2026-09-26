#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Phase 3: one backend per audio system, and the preferred default.

use std::collections::HashSet;

use fp_backends::{
    Availability, choose_default_backend, display_name, host_availability, preferred_backend,
    system_backends,
};

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

#[test]
fn a_system_with_no_output_device_is_unavailable() {
    // A JACK host opens even with no server running, but has no ports.
    let none = host_availability(Ok(false));
    let Availability::Unavailable(reason) = none else {
        panic!("a system with no output device must not be chosen")
    };
    assert!(reason.contains("no output device"), "{reason}");
    assert_eq!(host_availability(Ok(true)), Availability::Available);
}

#[test]
fn an_unavailable_reason_is_not_prefixed_twice() {
    let Availability::Unavailable(reason) = host_availability(Err("library missing".into())) else {
        panic!("a host that fails to open is unavailable")
    };
    assert_eq!(reason, "library missing");
}

#[test]
fn the_configured_system_is_used_only_when_available() {
    let listed = [("jack", false), ("pulseaudio", true), ("alsa", true)];
    assert_eq!(
        choose_default_backend(Some("jack"), &listed, "linux"),
        Some("pulseaudio"),
        "a configured system missing here falls back to the preferred one"
    );
    assert_eq!(
        choose_default_backend(Some("alsa"), &listed, "linux"),
        Some("alsa")
    );
    assert_eq!(
        choose_default_backend(None, &listed, "linux"),
        Some("pulseaudio")
    );
    assert_eq!(
        choose_default_backend(Some("mystery"), &listed, "linux"),
        Some("pulseaudio")
    );
    assert_eq!(
        choose_default_backend(None, &[("alsa", false)], "linux"),
        None
    );
}

#[test]
fn systems_are_listed_in_the_preferred_order() {
    let ids: Vec<String> = system_backends().iter().map(|b| b.id().0).collect();
    let listed: Vec<(&str, bool)> = ids.iter().map(|id| (id.as_str(), true)).collect();
    // The first listed is the one preferred when every system is available.
    assert_eq!(
        listed.first().map(|(id, _)| *id),
        preferred_backend(&listed, std::env::consts::OS)
    );
}
