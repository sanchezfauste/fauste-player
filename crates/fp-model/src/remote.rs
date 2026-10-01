//! Network remote control configuration (remote control spec §6.1).

use std::net::IpAddr;

use serde::{Deserialize, Serialize};

use crate::config::{ConfigWarning, clamp_to};

/// Remote control over the network: off until the operator turns it on.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RemoteConfig {
    pub http: HttpRemoteConfig,
    pub events: RemoteEventsConfig,
}

/// The HTTP/JSON API (remote control spec §3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HttpRemoteConfig {
    pub enabled: bool,
    /// An IPv4 or IPv6 address literal; "0.0.0.0" or "::" listen everywhere.
    pub bind: String,
    pub port: u16,
    /// Bearer token; mandatory when `bind` is not a loopback address.
    pub token: String,
    /// Web origins allowed to call the API from a browser.
    pub cors_origins: Vec<String>,
    /// Event streams open at once (remote control spec §5.2).
    pub max_event_clients: u32,
    pub request_timeout_ms: u32,
    pub max_body_bytes: u32,
}

impl Default for HttpRemoteConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bind: "127.0.0.1".to_owned(),
            port: 7380,
            token: String::new(),
            cors_origins: Vec::new(),
            max_event_clients: 16,
            request_timeout_ms: 10_000,
            max_body_bytes: 65_536,
        }
    }
}

/// What remote clients are told, and how often.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RemoteEventsConfig {
    /// How often elapsed and remaining times are published while playing.
    pub position_interval_ms: u32,
}

impl Default for RemoteEventsConfig {
    fn default() -> Self {
        Self {
            position_interval_ms: 250,
        }
    }
}

/// Shortest token accepted: guessing 16 random characters is out of reach.
const MIN_TOKEN_CHARS: usize = 16;

impl HttpRemoteConfig {
    /// The address to listen on, if `bind` is an IP literal.
    pub fn bind_addr(&self) -> Option<IpAddr> {
        self.bind.parse().ok()
    }
}

/// `scheme://host[:port]` with an http or https scheme and nothing after.
fn is_origin(origin: &str) -> bool {
    let rest = origin
        .strip_prefix("https://")
        .or_else(|| origin.strip_prefix("http://"));
    rest.is_some_and(|host| {
        !host.is_empty() && !host.contains('/') && !host.chars().any(char::is_whitespace)
    })
}

impl RemoteConfig {
    pub(crate) fn validate(&mut self, w: &mut Vec<ConfigWarning>) {
        let h = &mut self.http;
        if h.bind_addr().is_none() {
            w.push(ConfigWarning {
                field: "remote.http.bind",
                message: format!("{:?} is not an IP address; using 127.0.0.1", h.bind),
            });
            h.bind = "127.0.0.1".to_owned();
        }
        clamp_to(&mut h.port, 1024, u16::MAX, "remote.http.port", w);
        if !h.token.is_empty() && h.token.chars().count() < MIN_TOKEN_CHARS {
            h.token.clear();
            w.push(ConfigWarning {
                field: "remote.http.token",
                message: format!("shorter than {MIN_TOKEN_CHARS} characters; ignored"),
            });
        }
        // "*" lets any web page call the API, so it needs the token.
        let wildcard_ok = !h.token.is_empty();
        h.cors_origins.retain(|o| {
            let ok = if o == "*" { wildcard_ok } else { is_origin(o) };
            if !ok {
                w.push(ConfigWarning {
                    field: "remote.http.cors_origins",
                    message: format!("{o:?} cannot be used; dropped"),
                });
            }
            ok
        });
        clamp_to(
            &mut h.max_event_clients,
            1,
            256,
            "remote.http.max_event_clients",
            w,
        );
        clamp_to(
            &mut h.request_timeout_ms,
            1000,
            120_000,
            "remote.http.request_timeout_ms",
            w,
        );
        clamp_to(
            &mut h.max_body_bytes,
            1024,
            1_048_576,
            "remote.http.max_body_bytes",
            w,
        );
        clamp_to(
            &mut self.events.position_interval_ms,
            50,
            5000,
            "remote.events.position_interval_ms",
            w,
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::Config;

    #[test]
    fn remote_control_is_off_and_local_by_default() {
        let c = Config::default();
        assert!(!c.remote.http.enabled);
        assert_eq!(c.remote.http.bind, "127.0.0.1");
        assert_eq!(c.remote.http.port, 7380);
        assert!(c.remote.http.token.is_empty());
        assert!(c.remote.http.cors_origins.is_empty());
        assert_eq!(c.remote.http.max_event_clients, 16);
        assert_eq!(c.remote.http.request_timeout_ms, 10_000);
        assert_eq!(c.remote.http.max_body_bytes, 65_536);
        assert_eq!(c.remote.events.position_interval_ms, 250);
        assert!(Config::default().validate().is_empty());
    }

    #[test]
    fn numbers_are_brought_into_range() {
        let mut c = Config::default();
        c.remote.http.port = 80;
        c.remote.http.max_event_clients = 0;
        c.remote.http.request_timeout_ms = 999_999;
        c.remote.http.max_body_bytes = 10;
        c.remote.events.position_interval_ms = 1;
        let w = c.validate();
        assert_eq!(c.remote.http.port, 1024);
        assert_eq!(c.remote.http.max_event_clients, 1);
        assert_eq!(c.remote.http.request_timeout_ms, 120_000);
        assert_eq!(c.remote.http.max_body_bytes, 1024);
        assert_eq!(c.remote.events.position_interval_ms, 50);
        assert_eq!(w.len(), 5);
    }

    #[test]
    fn a_bind_that_is_not_an_ip_address_falls_back_to_loopback() {
        let mut c = Config::default();
        c.remote.http.bind = "localhost".into();
        let w = c.validate();
        assert_eq!(c.remote.http.bind, "127.0.0.1");
        assert_eq!(w[0].field, "remote.http.bind");
        c.remote.http.bind = "::".into();
        assert!(c.validate().is_empty());
        assert!(c.remote.http.bind_addr().is_some());
    }

    #[test]
    fn a_short_token_is_dropped() {
        let mut c = Config::default();
        c.remote.http.token = "short".into();
        let w = c.validate();
        assert!(c.remote.http.token.is_empty());
        assert_eq!(w[0].field, "remote.http.token");
        c.remote.http.token = "0123456789abcdef".into();
        assert!(c.validate().is_empty());
    }

    #[test]
    fn invalid_cors_origins_are_dropped() {
        let mut c = Config::default();
        c.remote.http.cors_origins = vec![
            "https://studio.example".into(),
            "http://10.0.0.5:8080".into(),
            "ftp://x".into(),
            "https://a.example/path".into(),
            "nonsense".into(),
            "https://".into(),
        ];
        let w = c.validate();
        assert_eq!(
            c.remote.http.cors_origins,
            vec!["https://studio.example", "http://10.0.0.5:8080"]
        );
        assert_eq!(w.len(), 4);
    }

    #[test]
    fn a_wildcard_origin_needs_a_token() {
        // Without a token, "*" would let any web page drive the station.
        let mut c = Config::default();
        c.remote.http.cors_origins = vec!["*".into()];
        assert_eq!(c.validate()[0].field, "remote.http.cors_origins");
        assert!(c.remote.http.cors_origins.is_empty());
        c.remote.http.token = "0123456789abcdef".into();
        c.remote.http.cors_origins = vec!["*".into()];
        assert!(c.validate().is_empty());
        assert_eq!(c.remote.http.cors_origins, vec!["*"]);
        c.remote.http.bind = "0.0.0.0".into();
        assert!(c.validate().is_empty());
        assert_eq!(c.remote.http.cors_origins, vec!["*"]);
    }

    #[test]
    fn missing_remote_fields_take_their_defaults() {
        let c: Config = serde_json::from_str(r#"{"remote":{"http":{"enabled":true}}}"#).unwrap();
        assert!(c.remote.http.enabled);
        assert_eq!(c.remote.http.port, 7380);
        assert_eq!(c.remote.events.position_interval_ms, 250);
    }
}
