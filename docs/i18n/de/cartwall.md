# Cartwall

Die Cartwall ist der Streifen aus Buttons unter den Playern. Jeder Button, ein
**Cart**, spielt sofort einen Sound ab: Jingles, Effekte, Spots. Carts laufen
auf eigenen Ausgängen, unabhängig von den Playern.

![Die Cartwall mit einem laufenden Cart](../../images/guide/cartwall.png)

## Verwendung {#using-it}

- **Klicke auf ein Cart**, um es zu starten. **Klicke erneut darauf**, um es
  zu stoppen.
- **Alle stoppen** (am rechten Ende der Leiste) stoppt jedes laufende Cart auf
  allen Seiten. Die Beschriftung zeigt, wie viele laufen, etwa **Alle stoppen
  (2)**; läuft keines, ist der Button abgeblendet und zeigt keine Zahl.
- Solange ein Cart spielt, wird sein Rand rot, ein roter Balken schrumpft
  mit der Wiedergabe, und seine Zeit läuft rückwärts.
- Carts **überlagern sich** standardmäßig: Das Starten eines zweiten stoppt das
  erste nicht. Ein Cart mit der Option **Beim Start andere Carts stoppen**
  stoppt zuerst alle anderen Carts, die laufen, auf jeder Seite.
- Ein Cart mit der Option **Schleife** beginnt am Ende ohne Lücke wieder bei
  seinem Cue-in, bis du es stoppst.
- Mit **Rechtsklick** auf ein Cart erscheinen weitere Optionen:

| Eintrag | Aktion |
|---|---|
| Auf CUE vorhören | Auf dem CUE-Ausgang der Cartwall abspielen (abgeblendet, wenn die Cartwall neben ihrem Main-Ausgang keinen Cue-Ausgang hat) |
| Stoppen | Stoppen |
| Bearbeiten… | In den Einstellungen öffnen |

Das Menü eines leeren Carts enthält nur **Bearbeiten…**, um seine Datei zu
wählen.

- **Seiten:** Die Tabs neben **CARTWALL** wechseln die Seiten. Ein roter Punkt
  zeigt, dass auf dieser Seite ein Cart läuft.
- Klicke auf **CARTWALL**, um den Streifen einzuklappen oder wieder
  auszuklappen.
- Ist das Fenster niedrig, schrumpfen die Buttons (bis zu einer Mindesthöhe),
  damit alle konfigurierten Zeilen passen; die Cartwall scrollt nur, wenn
  selbst die kleinsten Buttons nicht passen.

| Aussehen des Buttons | Bedeutung |
|---|---|
| Violetter Punkt | Jingle |
| Bernsteinfarbener Punkt | Effekt |
| Grauer Punkt | Spot (Werbung) |
| ↻ nach dem Typ | Läuft in einer Schleife |
| ✋ nach dem Typ | Stoppt beim Start die anderen Carts |
| Datei mit Kreuz / Warnzeichen | Die Datei fehlt / kann nicht dekodiert werden; fahre mit der Maus über das Cart, um den Grund und den Pfad zu sehen. Eine fehlende Datei wird alle 30 s erneut gesucht (`tuning.missing_recheck_ms`). |
| „Leer“, abgeblendet | Keine Datei zugewiesen |

Carts verwenden dieselben Marker wie Tracks. Sie beginnen an ihrem Cue-in und
enden an ihrem Cue-out, die du in der Wellenform eines Players bearbeiten
kannst, wenn die Datei dort geladen ist.

## Tastatur {#keyboard}

Standardmäßig starten **F1**…**F12** die Carts 1–12 der angezeigten Seite, und
**Ctrl+Space** stoppt jedes Cart (dasselbe wie **Alle stoppen**; es stoppt
auch ein Cart, das du auf CUE vorhörst, selbst wenn kein Cart läuft). Siehe
[Tastatur](keyboard.md), um sie zu ändern.

## Carts einrichten {#setting-up-carts}

Gehe zu **Einstellungen → Cartwall**:

- **Seiten:** erstellen, umbenennen, löschen (die letzte Seite lässt sich
  nicht löschen) und die Rastergröße festlegen (Zeilen × Spalten). Ein
  kleineres Raster wird abgelehnt, wenn dadurch Carts mit Datei wegfielen.
- **Importieren… / Exportieren…** speichern eine Seite in einer
  `.cartpage.json`-Datei und laden sie wieder, zum Beispiel um sie zwischen
  Studios zu teilen. Relative Dateipfade werden relativ zum Ordner der Datei
  aufgelöst.
- **Carts:** Klicke im Raster auf ein Cart und lege dann seinen Namen, seine
  Datei (**Auswählen…** oder **Leeren**), den Typ, **Schleife** und **Beim
  Start andere Carts stoppen** fest.

Die eigenen Main- und Cue-Ausgänge der Cartwall findest du unter
**Einstellungen → Audioausgänge** (Zeile **Cartwall**).
