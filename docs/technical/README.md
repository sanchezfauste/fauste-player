# Fauste Player — Technical documentation

For developers and maintainers. The binding design is the spec,
[`docs/superpowers/specs/2026-09-25-fauste-player-design.md`](../superpowers/specs/2026-09-25-fauste-player-design.md).
These pages describe what is **implemented** and how, and point out where
the implementation differs from the spec or defers part of it.

| Document | Content |
|---|---|
| [Architecture](architecture.md) | Crates, dependency direction, data flow, design principles |
| [Threading and real time](threading-and-realtime.md) | Every thread, its priority, what it may do, how threads talk |
| [Audio engine](audio-engine.md) | Sources, mixer, buses, scheduling, worker threads, the conductor, device loss |
| [Backends](backends.md) | The backend trait, the implemented backends, the roadmap |
| [Audio path audit](audio-path-audit.md) | The file-to-device audit of feedback 2 (O26): findings A1–A22 with their evidence, and readings above 0 dBFS (O34) |
| [Decoding](decoding.md) | Formats, the backends behind `FileDecoder`, DSD to PCM, seeking |
| [Analysis](analysis.md) | Tags, covers, peaks, markers, the cache and the analysis pool |
| [Persistence and configuration](persistence.md) | Files, atomic writes, backups, migrations, every configuration field |
| [Remote control API](remote-api.md) | The HTTP/JSON API: resources, routes, errors, security, the remote thread |
| [User interface](ui.md) | The egui app: view model, controller, services, panic isolation, i18n |
| [Testing](testing.md) | Test layers, the Offline backend, stress and soak, UI tests |
| [Release process](release-process.md) | Versioning, commits, release-please, artefacts, rebuilding a release, the website and the translated guide |

The implementation plans in [`docs/superpowers/plans`](../superpowers/plans)
record how each part was built, and the rulings taken on the way.
