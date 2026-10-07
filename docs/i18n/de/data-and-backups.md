# Daten und Backups

## Wo die Dateien liegen {#where-files-are-kept}

| | Linux | Windows | macOS |
|---|---|---|---|
| Einstellungen (`config.json`) | `~/.config/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\config\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Playlists und Session | `~/.local/share/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\data\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Analyse-Cache | `~/.cache/fausteplayer/` | `%LOCALAPPDATA%\Fauste\Fauste Player\cache\` | `~/Library/Caches/org.Fauste.Fauste-Player/` |
| Logs und Absturzberichte | `~/.local/share/fausteplayer/logs/` | `%LOCALAPPDATA%\Fauste\Fauste Player\data\logs\` | `~/Library/Application Support/org.Fauste.Fauste-Player/logs/` |

Die Linux-Pfade folgen den XDG-Variablen (`XDG_CONFIG_HOME` und so weiter),
sofern sie gesetzt sind.

## Portabler Modus {#portable-mode}

Setze die Umgebungsvariable `FAUSTE_HOME` auf einen Ordner, und alles wird
stattdessen dort abgelegt, in `config/`, `data/`, `cache/` und `logs/`. Das
ist nützlich auf einem USB-Stick oder um getrennte Setups nebeneinander zu
führen.

## Dateien {#files}

| Datei | Inhalt |
|---|---|
| `config.json` | Einstellungen, Ausgänge, Analyseschwellen, erweiterte Feinabstimmung |
| `playlists.json` | Playlists, Tracks, Gespielt-Markierungen, manuelle Marker |
| `session.json` | Für jeden Player: angezeigte Playlist, aktueller und nächster Track, Modus, Position, Lautstärke, Spaltenbreiten (pro Spalte) |

## Automatisches Speichern und Backups {#autosave-and-backups}

- Änderungen werden etwa eine Sekunde, nachdem sie auftreten, gespeichert.
  Solange etwas spielt, wird die Session (Positionen) im selben Takt
  aktualisiert.
- Jedes Speichern schreibt eine temporäre Datei und ersetzt dann die alte,
  sodass ein Stromausfall nie eine halb geschriebene Datei hinterlässt.
- Die vorherigen Versionen bleiben als `.bak1`, `.bak2` und `.bak3` erhalten.
- Lässt sich eine Datei nicht lesen, wird das neueste intakte Backup
  verwendet. Die unlesbare Datei bleibt zur Untersuchung erhalten, umbenannt
  in `*.corrupt-<time>`. Die Anwendung startet immer.

## Wiederherstellung nach einem Absturz {#crash-recovery}

Nach einem Absturz oder Neustart kommt jeder Player mit seiner Playlist,
seinem aktuellen und nächsten Track und seiner Position zurück, aber
**pausiert oder gestoppt**. Nichts geht von selbst on Air. Ein Player, der ganz
am Ende seines Tracks stand, kommt stattdessen am Anfang dieses Tracks
zurück, sodass Play ihn abspielt, statt ihn sofort zu beenden.

## `config.json` von Hand bearbeiten {#editing-configjson-by-hand}

Schließe zuerst die Anwendung (ist Audio on Air, fragt sie vor dem Schließen
nach). Unbekannte oder außerhalb des Bereichs liegende Werte werden beim
Laden der Datei auf den nächsten gültigen Wert korrigiert, und die Korrekturen
werden protokolliert. Die Abschnitte `limits` und `tuning` enthalten
erweiterte Werte (Ressourcengrenzen, Engine-Timing), die nicht im
Einstellungsfenster stehen.
