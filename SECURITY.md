# Security policy

## Reporting a vulnerability

Please report vulnerabilities **privately** through GitHub's
[*Report a vulnerability*](https://github.com/sanchezfauste/fauste-player/security/advisories/new)
form, not in a public issue. Include the version, the platform, and the
steps or file that trigger the problem. We aim to acknowledge reports within
a week.

Only the latest release is supported with fixes.

## Threat model

Fauste Player is a local desktop application:

- It makes **no network connections** and sends no telemetry.
- It reads audio files, tags and embedded images, and playlists chosen by the
  user. These are treated as untrusted input:
  - covers are size-limited before and during decoding;
  - tag parsing has allocation limits;
  - state files larger than the configured limits are refused;
  - a malformed file degrades to "not available", never to a crash.
- Paths from playlists are only opened for reading. Nothing is executed, and
  nothing is written outside the application's config, data, cache and log
  folders, or a path the user explicitly picks when exporting (from Phase 2).
- The code has no `unsafe` (the workspace lints forbid `unsafe_code`). Dependencies are
  checked by `cargo deny` (advisories, licences, sources) in CI.
