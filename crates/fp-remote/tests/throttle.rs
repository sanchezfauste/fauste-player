#![allow(clippy::unwrap_used)]

use std::net::{IpAddr, Ipv6Addr};

use fp_remote::throttle::Throttle;

#[test]
fn a_flood_from_spoofed_sources_stays_bounded() {
    // Thousands of sources within one second: the table must not grow
    // past its bound (each would otherwise log a line and cost a scan).
    let log = Throttle::default();
    for n in 0..5000u128 {
        log.note(Some(IpAddr::V6(Ipv6Addr::from(n))), "test");
    }
    assert!(log.tracked() <= 256, "{}", log.tracked());
}
