//! Every audio system this build supports, as backends (Phase 3). Each
//! cpal host compiled in becomes one backend; a system that cannot start
//! here (library missing, server not running) reports `Unavailable`.

use std::sync::Arc;

use crate::{AudioBackend, CpalBackend};

/// One backend per audio system compiled into this build, the platform
/// default first.
pub fn system_backends() -> Vec<Arc<dyn AudioBackend>> {
    let default = cpal::default_host().id();
    let mut hosts: Vec<cpal::HostId> = cpal::ALL_HOSTS.to_vec();
    hosts.sort_by_key(|h| *h != default);
    hosts
        .into_iter()
        .map(|h| Arc::new(CpalBackend::for_host(h)) as Arc<dyn AudioBackend>)
        .collect()
}

/// The order in which systems are preferred when none is configured.
fn preference(os: &str) -> &'static [&'static str] {
    match os {
        "linux" => &["pipewire", "pulseaudio", "jack", "alsa"],
        "windows" => &["wasapi", "asio", "jack"],
        "macos" => &["coreaudio", "jack"],
        _ => &[],
    }
}

/// The preferred available system among `(id, available)` on `os`
/// (`std::env::consts::OS`). Unknown systems come after the known ones.
pub fn preferred_backend<'a>(backends: &[(&'a str, bool)], os: &str) -> Option<&'a str> {
    let order = preference(os);
    let rank = |id: &str| order.iter().position(|p| *p == id).unwrap_or(order.len());
    backends
        .iter()
        .filter(|(_, available)| *available)
        .min_by_key(|(id, _)| rank(id))
        .map(|(id, _)| *id)
}

/// A readable name for a backend id.
pub fn display_name(id: &str) -> &str {
    match id {
        "alsa" => "ALSA",
        "pulseaudio" => "PulseAudio",
        "pipewire" => "PipeWire",
        "jack" => "JACK",
        "wasapi" => "WASAPI",
        "asio" => "ASIO",
        "coreaudio" => "Core Audio",
        "null" => "Null",
        "offline" => "Offline",
        other => other,
    }
}
