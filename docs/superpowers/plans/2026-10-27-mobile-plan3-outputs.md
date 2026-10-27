# Mobile Plan 3 — Outputs and Devices Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** On Android, each player, the cartwall and their Cue routes pick
an output among the system's audio devices, and a saved route finds its
device again after the device goes away and comes back (Bluetooth off
and on), as the desktop does.

**Architecture:** The desktop already has everything MB5 and MB6 ask for
except stable device ids. Routes store the backend's device id string.
cpal's AAudio host uses Android's volatile device id (`aaudio:<n>`). On
the AAudio host, `fp-backends` now gives every listed output a stable id
built from its interface, its type and its product name, plus an ordinal
for twins, and finds a device by that id. The labels name the kind of
device: the type, or else the interface (Bluetooth A2DP has no cpal
type). The missing-device rules, the Cue routes and the on-air
confirmation (CA) are unchanged.

**Tech Stack:** Rust 1.98, cpal 0.18.2 (`DeviceType`, `InterfaceType`,
`DeviceDescription`).

**Spec:** `docs/superpowers/specs/2026-10-25-mobile-design.md` (MB5, MB6,
§11).

## Global Constraints

- Desktop hosts are unchanged: their ids, labels and lookups stay
  byte-for-byte the same.
- The identity logic is pure and tested on the host; the cpal glue is
  `#[cfg(target_os = "android")]`.
- Nothing re-routes silently: an id that matches no listed device is
  `DeviceNotFound`, as on the desktop (MB5).
- No new `Config` field and no new UI string.

## Review Focus

- Two outputs with the same type and name (the built-in speaker and the
  "speaker safe" path both map to cpal `Speaker`): they get different
  ids, and the ids do not swap while both stay listed.
- A device that reconnects with a new Android id keeps its stable id.
- A route saved with the default id `aaudio:-1` still opens the default
  device (plan 2).
- A Bluetooth A2DP device (cpal type `Unknown`, interface `Bluetooth`) is
  labelled as Bluetooth, not by its product name alone.
- A name containing `:` or `#` cannot collide with another device's id.

---

### Task 1: Stable AAudio identities (pure)

**Files:**
- Create: `crates/fp-backends/src/aaudio_ids.rs`
- Modify: `crates/fp-backends/src/lib.rs` (module, `pub` for tests)
- Test: `crates/fp-backends/tests/it/aaudio_ids.rs` (+ `mod` line)

**Interfaces:**
- Produces: `pub struct ListedOutput { pub name: String, pub device_type: cpal::DeviceType, pub interface: cpal::InterfaceType }`
  and `pub fn identities(outputs: &[ListedOutput]) -> Vec<(DeviceId, String)>`:
  the stable id and the label of each output, in order.

- [ ] **Step 1: Write the failing tests:**
  - `the_speaker_and_the_earpiece_get_different_ids`
  - `an_id_survives_a_reconnect`: the same three fields give the same id
  - `twins_get_an_ordinal`: two equal outputs give `…` and `… #2`, in list order
  - `a_bluetooth_a2dp_device_is_labelled_bluetooth`: name "JBL Flip", `Unknown`/`Bluetooth` gives the label "JBL Flip (Bluetooth)"
  - `a_typed_device_keeps_cpals_name`: "Pixel (Speaker)" stays the label
  - `ids_start_with_the_aaudio_host`: every id starts with `aaudio:`, never `aaudio:-1`
  - `separators_in_names_do_not_collide`: names chosen to collide under naive joining get different ids
- [ ] **Step 2:** Run `cargo test -p fp-backends --test it aaudio_ids`. Expected: FAIL (module missing).
- [ ] **Step 3:** Implement. Id: `aaudio:` followed by the interface, the type and the name joined with `/`, percent-escaping `%`, `/` and `#` in the name, then `#<n>` for the n-th twin (n ≥ 2). Label: cpal's name, plus ` (<interface>)` when the type is `Unknown` and the interface is not.
- [ ] **Step 4:** Run the tests. Expected: PASS.
- [ ] **Step 5:** Commit `feat(backends): stable ids for AAudio outputs`.

### Task 2: The AAudio host lists and finds outputs by stable id

**Files:**
- Modify: `crates/fp-backends/src/cpal_backend.rs` (`enumerate_devices`, `find`)

**Interfaces:**
- Consumes: Task 1's `identities`.

- [ ] **Step 1:** Write the failing test `a_stable_id_is_found_among_the_listed_devices` for a pure helper
  `fn pick<D>(wanted: &str, listed: Vec<(DeviceId, D)>) -> Option<D>` (the unit tests in `cpal_backend.rs`), plus
  `an_unknown_stable_id_is_not_found`.
- [ ] **Step 2:** Run them. Expected: FAIL.
- [ ] **Step 3:** Implement `pick`. On `cpal::HostId::AAudio`
  (`#[cfg(target_os = "android")]`), `enumerate_devices` takes ids and
  labels from `identities` over the listed outputs' descriptions, and
  `find` lists the outputs, builds the same ids and picks from them,
  before the existing `resolve` (which keeps `aaudio:-1` and raw ids).
- [ ] **Step 4:** Run the fp-backends tests and the Android clippy
  (`cargo ndk -P 29 -t arm64-v8a clippy --locked -p fp-backends -p fp-mobile -p fp-app --lib -- -D warnings`). Expected: PASS.
- [ ] **Step 5:** Commit `feat(backends): AAudio routes follow their device across reconnects`.

### Task 3: Docs and device checks

**Files:**
- Modify: `docs/technical/mobile.md` (outputs section; drop "outputs" from the interim list)

- [ ] **Step 1:** Write the outputs section: ids, labels, what happens when a device leaves and comes back, Cue routes as on the desktop (MB6).
- [ ] **Step 2:** Emulator: Settings → Audio outputs lists the outputs with their kinds; a route on the speaker plays; the saved config holds the stable id.
- [ ] **Step 3:** Commit `docs: describe Android outputs`.
