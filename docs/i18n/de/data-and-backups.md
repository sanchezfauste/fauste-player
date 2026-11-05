# Daten und Backups

## Wo die Dateien liegen {#where-files-are-kept}

| | Linux | Windows | macOS |
|---|---|---|---|
| Einstellungen (`config.json`) | `~/.config/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\config\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Bibliothek, Playlists, Carts und Session (`fauste.db`) | `~/.local/share/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\data\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Analyse-Cache | `~/.cache/fausteplayer/` | `%LOCALAPPDATA%\Fauste\Fauste Player\cache\` | `~/Library/Caches/org.Fauste.Fauste-Player/` |
| Logs und Absturzberichte | `~/.local/share/fausteplayer/logs/` | `%LOCALAPPDATA%\Fauste\Fauste Player\data\logs\` | `~/Library/Application Support/org.Fauste.Fauste-Player/logs/` |

Die Linux-Pfade folgen den XDG-Variablen (`XDG_CONFIG_HOME` und so weiter),
sofern sie gesetzt sind.

Das Snap legt seine Dateien stattdessen in einem eigenen Ordner ab: die
Einstellungen in `~/snap/fauste-player/current/.config/fausteplayer/`, die
Datenbank und die Logs in
`~/snap/fauste-player/current/.local/share/fausteplayer/` und den Analyse-Cache
in `~/snap/fauste-player/common/.cache/fausteplayer/`.
`sudo snap remove fauste-player` löscht sie ebenfalls (es behält eine Weile einen
Snapshot; `--purge` nicht).

## Portabler Modus {#portable-mode}

Setze die Umgebungsvariable `FAUSTE_HOME` auf einen Ordner, und alles wird
stattdessen dort abgelegt, in `config/`, `data/`, `cache/` und `logs/`. Das
ist nützlich auf einem USB-Stick oder um getrennte Setups nebeneinander zu
führen.

## Dateien {#files}

| Datei | Inhalt |
|---|---|
| `config.json` | Einstellungen, Ausgänge, Analyseschwellen, erweiterte Feinabstimmung |
| `fauste.db` | Die Bibliothek (Tracks, ihre Kategorien, manuelle Marker, Tags, Lautheit), die Playlists mit ihren Gespielt-Markierungen, die Cart-Seiten und die Session: für jeden Player die angezeigte Playlist, aktueller und nächster Track, Modus, Position, Lautstärke und Spaltenbreiten (pro Spalte), sowie das Fensterlayout |
| `fauste.db.bak1`, `.bak2`, `.bak3` | Kopien der Datenbank (siehe unten) |
| `window.ron` | Größe und Position des Hauptfensters und ob es maximiert war (nur Computer; unter Wayland wählt das System die Position) |

## Automatisches Speichern und Backups {#autosave-and-backups}

- Änderungen werden etwa eine Sekunde, nachdem sie auftreten, gespeichert, und
  nur das Geänderte wird geschrieben. Solange etwas spielt, wird die Session
  (Positionen) im selben Takt aktualisiert.
- Die Datenbank wird in Transaktionen geschrieben, sodass ein Stromausfall nie
  eine halb gespeicherte Änderung hinterlässt.
- Beim ersten Speichern jeder Sitzung und danach etwa alle fünf Minuten, solange
  Änderungen gespeichert werden, kopiert die Anwendung die Datenbank nach
  `fauste.db.bak1` und schiebt die älteren Kopien nach oben, zu `.bak2` und
  `.bak3`. Mehrere Neustarts hintereinander verdrängen daher die älteren Backups.
- Lässt sich die Datenbank nicht lesen, wird das neueste intakte Backup
  verwendet. Die unlesbare Datei bleibt zur Untersuchung erhalten, umbenannt in
  `fauste.db.corrupt-<time>`. Schlägt jede Kopie fehl, startet die Anwendung
  leer. Sie startet immer.
- Auch eine Kopie, die sich nicht verwenden lässt, bleibt erhalten: Startet die
  Anwendung von einer anderen Kopie (oder einer leeren Datenbank), werden die
  unbrauchbaren in `fauste.db.bak1.corrupt-<time>` (oder `.newer-<time>`)
  umbenannt, damit das nächste Backup sie nie verdrängt.
- Verwendet eine andere Kopie der Anwendung die Datenbank, läuft diese ohne zu
  speichern und sagt es beim Start.
- Eine Datenbank, die eine neuere Version der Anwendung geschrieben hat, wird
  nicht geöffnet: Sie bleibt als `fauste.db.newer-<time>` erhalten, und die
  Anwendung startet vom neuesten Backup, das sie lesen kann, oder leer, wenn es
  keines gibt, sodass das Öffnen der Daten mit einer älteren Version sie nie
  zerstört.
- `config.json` wird wie bisher gespeichert: Eine temporäre Datei ersetzt die
  alte, und die vorherigen Versionen bleiben als `.bak1`, `.bak2` und `.bak3`
  daneben erhalten.
- Lässt sich die Datenbank in einer Sitzung gar nicht verwenden (zum Beispiel
  weil sie nicht umbenannt werden kann, da der Ordner schreibgeschützt ist), läuft
  die Anwendung bis zum nächsten Start ohne zu speichern, und das Log sagt es.

## Playlists und Session aus einer früheren Version {#playlists-and-session-from-an-earlier-version}

Frühere Versionen legten die Playlists in `playlists.json`, die Cart-Seiten in
`carts.json` und die Session in `session.json` ab. Der erste Start dieser Version
liest sie (mit ihren `.bak`-Kopien) einmal und legt alles in `fauste.db` ab: die
Bibliothek mit ihren Markern, Tags und ihrer Lautheit, die Playlists mit ihren
Gespielt-Markierungen, die Cart-Seiten und die Session. Danach werden die alten
Dateien mit `.imported` am Ende umbenannt (`playlists.json.imported`,
`playlists.json.bak1.imported` und so weiter). Sie werden nicht mehr verwendet,
und du kannst sie löschen, sobald du sicher bist, dass alles da ist.

So gehst du zu einer früheren Version zurück:

1. Schließe die Anwendung.
2. Verschiebe `fauste.db`, `fauste.db-wal`, `fauste.db-shm` und jede
   `fauste.db.bak*` in einen Ordner außerhalb des Datenordners. Lösche sie nicht.
3. Entferne `.imported` aus den Namen der alten Dateien.
4. Starte die ältere Version.

Alles, was seit dem Update gespeichert wurde, bleibt nur in dem Ordner, in den du
die Dateien verschoben hast: Die alten Dateien enthalten es nicht. Aktualisierst
du später erneut, werden die alten Dateien wieder importiert, solange im
Datenordner keine `fauste.db` oder `fauste.db.bak*` liegt.

Wird ein Start während des Imports unterbrochen, beendet ihn der nächste. Lässt
sich eine der alten Dateien nicht lesen (zu groß oder von einer neueren Version
geschrieben), importiert die Anwendung nichts und läuft ohne diese Session zu
speichern, sodass die alten Dateien dort bleiben, wo sie sind; das Log nennt die
Datei. Behebe das Problem oder entferne die Datei und starte erneut. Sobald
`fauste.db` existiert, werden alte JSON-Dateien nie wieder importiert.

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
