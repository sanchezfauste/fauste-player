#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Operator feedback 4, Q3 and Q12: what Settings offers a routed device.

use fp_model::DsdOutput::{Dop, Native, Pcm};
use fp_model::{
    DsdCaps, DsdNotOffered, dsd_not_offered, effective_dsd_mode, offered_buffers,
    offered_dsd_modes, offered_rates,
};

/// A bit-perfect, exclusive-capable Linux device that takes native DSD.
const FULL: DsdCaps = DsdCaps {
    bit_perfect: true,
    connected: true,
    exclusive_capable: true,
    native_dsd: true,
    linux: true,
};

#[test]
fn only_the_modes_the_device_can_take_are_offered() {
    assert_eq!(offered_dsd_modes(FULL, Pcm), vec![Pcm, Dop, Native]);
    let windows = DsdCaps {
        linux: false,
        ..FULL
    };
    assert_eq!(offered_dsd_modes(windows, Pcm), vec![Pcm, Dop]);
    let no_native = DsdCaps {
        native_dsd: false,
        ..FULL
    };
    assert_eq!(offered_dsd_modes(no_native, Pcm), vec![Pcm, Dop]);
    let shared = DsdCaps {
        exclusive_capable: false,
        native_dsd: false,
        ..FULL
    };
    assert_eq!(offered_dsd_modes(shared, Pcm), vec![Pcm]);
    let off = DsdCaps {
        bit_perfect: false,
        ..FULL
    };
    assert_eq!(offered_dsd_modes(off, Pcm), vec![Pcm], "bit-perfect off");
    let unplugged = DsdCaps {
        exclusive_capable: false,
        native_dsd: false,
        ..FULL
    };
    assert_eq!(
        offered_dsd_modes(unplugged, Native),
        vec![Pcm, Native],
        "a configured mode stays listed so it can be changed back"
    );
}

#[test]
fn every_mode_offered_needs_no_reason() {
    assert_eq!(dsd_not_offered(FULL), None);
}

#[test]
fn a_device_that_cannot_be_exclusive_says_so() {
    let caps = DsdCaps {
        bit_perfect: false,
        exclusive_capable: false,
        native_dsd: false,
        ..FULL
    };
    assert_eq!(dsd_not_offered(caps), Some(DsdNotOffered::NotExclusive));
}

#[test]
fn a_device_with_bit_perfect_off_says_so() {
    let caps = DsdCaps {
        bit_perfect: false,
        ..FULL
    };
    assert_eq!(dsd_not_offered(caps), Some(DsdNotOffered::BitPerfectOff));
}

#[test]
fn native_dsd_off_linux_says_it_needs_linux() {
    let caps = DsdCaps {
        linux: false,
        ..FULL
    };
    assert_eq!(dsd_not_offered(caps), Some(DsdNotOffered::NativeNeedsLinux));
}

#[test]
fn a_device_without_native_dsd_says_so() {
    let caps = DsdCaps {
        native_dsd: false,
        ..FULL
    };
    assert_eq!(dsd_not_offered(caps), Some(DsdNotOffered::NoNativeDsd));
}

const RATES: [u32; 6] = [44_100, 48_000, 88_200, 96_000, 176_400, 192_000];
const BUFFERS: [u32; 7] = [64, 128, 256, 512, 1024, 2048, 4096];

#[test]
fn only_the_rates_the_device_reports_are_offered() {
    assert_eq!(
        offered_rates(&[(44_100, 48_000)], &RATES, None),
        vec![44_100, 48_000]
    );
    assert_eq!(
        offered_rates(&[(44_100, 44_100), (96_000, 192_000)], &RATES, None),
        vec![44_100, 96_000, 176_400, 192_000]
    );
    assert_eq!(
        offered_rates(&[], &RATES, None),
        RATES.to_vec(),
        "nothing reported"
    );
    assert_eq!(
        offered_rates(&[(44_100, 48_000)], &RATES, Some(96_000)),
        vec![44_100, 48_000, 96_000],
        "the chosen rate stays listed"
    );
}

#[test]
fn only_the_buffers_the_device_reports_are_offered() {
    assert_eq!(
        offered_buffers(Some((128, 1024)), &BUFFERS, None),
        vec![128, 256, 512, 1024]
    );
    assert_eq!(offered_buffers(None, &BUFFERS, None), BUFFERS.to_vec());
    assert_eq!(
        offered_buffers(Some((128, 1024)), &BUFFERS, Some(4096)),
        vec![128, 256, 512, 1024, 4096]
    );
}

#[test]
fn a_configured_mode_the_device_cannot_take_shows_as_pcm() {
    assert_eq!(effective_dsd_mode(FULL, Dop), Dop);
    assert_eq!(effective_dsd_mode(FULL, Native), Native);
    let shared = DsdCaps {
        exclusive_capable: false,
        ..FULL
    };
    assert_eq!(effective_dsd_mode(shared, Dop), Pcm);
    let off = DsdCaps {
        bit_perfect: false,
        ..FULL
    };
    assert_eq!(effective_dsd_mode(off, Dop), Pcm);
    assert_eq!(effective_dsd_mode(off, Pcm), Pcm);
}

#[test]
fn an_unplugged_device_says_so_instead_of_not_exclusive() {
    let caps = DsdCaps {
        connected: false,
        exclusive_capable: false,
        native_dsd: false,
        ..FULL
    };
    assert_eq!(dsd_not_offered(caps), Some(DsdNotOffered::NotConnected));
}
