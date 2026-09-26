//! Every audio system this build supports, as backends (Phase 3). Each
//! cpal host compiled in becomes one backend; a system that cannot start
//! here (library missing, server not running) reports `Unavailable`.

use std::sync::Arc;

use crate::{AudioBackend, Availability, CpalBackend};

/// One backend per audio system compiled into this build, in the order
/// they are preferred on this OS. Nothing connects to a system here: each
/// backend opens its host on first use.
pub fn system_backends() -> Vec<Arc<dyn AudioBackend>> {
    let order = preference(std::env::consts::OS);
    let rank = |h: &cpal::HostId| {
        let id = h.name().to_lowercase();
        order.iter().position(|p| *p == id).unwrap_or(order.len())
    };
    let mut hosts: Vec<cpal::HostId> = cpal::ALL_HOSTS.to_vec();
    hosts.sort_by_key(rank);
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

/// The system the default output uses: the configured one when it is
/// available here, otherwise the preferred available one (a config from
/// another machine, or a JACK server that is not running, must not leave
/// the default output silent).
pub fn choose_default_backend<'a>(
    configured: Option<&str>,
    backends: &[(&'a str, bool)],
    os: &str,
) -> Option<&'a str> {
    configured
        .and_then(|id| backends.iter().find(|(b, ok)| *b == id && *ok))
        .map(|(b, _)| *b)
        .or_else(|| preferred_backend(backends, os))
}

/// Whether a system can be used: its host opened (`Err` carries why not)
/// and it has at least one output device. A JACK host opens even when no
/// server runs, but then has no ports.
pub fn host_availability(opened: Result<bool, String>) -> Availability {
    match opened {
        Ok(true) => Availability::Available,
        Ok(false) => {
            Availability::Unavailable("no output device (is its server running?)".to_owned())
        }
        Err(reason) => Availability::Unavailable(reason),
    }
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
