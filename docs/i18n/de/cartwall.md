# Cartwall

Die Cartwall ist das Panel aus Buttons unter den Playern oder neben ihnen
(siehe [Layout](layout.md#move-and-resize-the-cartwall)). Jeder Button, ein
**Cart**, spielt sofort einen Sound ab: Jingles, Effekte, Spots. Carts laufen
auf eigenen Ausgängen, unabhängig von den Playern.

![Die Cartwall mit einem laufenden Cart](../../images/guide/cartwall.png)

## Verwendung {#using-it}

- **Klicke auf ein Cart**, um es zu starten. **Klicke erneut darauf**, um es
  zu stoppen.
- **Tracks aus dem [Bibliotheksfenster](library.md) ziehen**: Lege sie auf ein
  Cart, um den ersten darin abzulegen (er ersetzt die Datei, wenn das Cart
  gestoppt ist). Ein Cart, das spielt oder auf CUE läuft, lehnt das Ablegen ab.
- **Alle stoppen** (am rechten Ende der Leiste) stoppt jedes laufende Cart auf
  allen Seiten. Die Beschriftung zeigt, wie viele laufen, etwa **Alle stoppen
  (2)**; läuft keines, ist der Button abgeblendet und zeigt keine Zahl.
- Solange ein Cart spielt, nehmen sein Rand und ein Balken, der von links mit
  der Wiedergabe wächst, die [Farbe](#setting-up-carts) des Carts an (Rot bei
  einem Cart ohne Farbe), eine dünne Linie markiert über der Wellenform den
  bereits gespielten Rand, und seine Zeit läuft rückwärts. Die Linie bewegt
  sich fließend, auch bei einem kurzen Cart.
- Sobald die Datei eines Carts analysiert ist, erscheint ihre Wellenform
  schwach hinter seinem Namen, sodass du einen Sound, der sofort einsetzt,
  von einem unterscheidest, der einblendet. Sie deckt ab, was das Cart
  spielt, von seinem Cue-in bis zu seinem Cue-out.
- Carts **überlagern sich** standardmäßig: Das Starten eines zweiten stoppt das
  erste nicht. Ein Cart mit der Option **Beim Start andere Carts stoppen**
  stoppt zuerst alle anderen Carts, die laufen, auf jeder Seite.
- Ein Cart mit der Option **Schleife** beginnt am Ende ohne Lücke wieder bei
  seinem Cue-in, bis du es stoppst.
- Mit **Rechtsklick** auf ein Cart erscheinen weitere Optionen:

| Eintrag | Aktion |
|---|---|
| Auf CUE vorhören | Auf dem CUE-Ausgang der Cartwall abspielen (abgeblendet, wenn die Cartwall keinen Cue-Ausgang hat). Auf dem Sendeausgang wird zuerst gefragt, solange dort etwas spielt (siehe [CUE](players.md#cue-pre-listen)) |
| Stoppen | Stoppen |
| Bearbeiten… | In den Einstellungen öffnen |

Das Menü eines leeren Carts enthält nur **Bearbeiten…**, um seine Datei zu
wählen.

- **Seiten:** Die Tabs neben **CARTWALL** wechseln die Seiten. Ein roter Punkt
  zeigt, dass auf dieser Seite ein Cart läuft.
- Klicke auf **CARTWALL**, um die Cartwall einzuklappen oder wieder
  auszuklappen. Neben den Playern eingeklappt, ist sie ein schmaler Streifen
  mit einem Pfeil zum Ausklappen und, solange ein Cart spielt, einem roten
  Punkt.
- Unter den Playern, so hoch wie ihre Zeilen es brauchen, schrumpfen die
  Buttons bei niedrigem Fenster (bis zu einer Mindesthöhe), damit alle
  konfigurierten Zeilen passen. Sobald du die Cartwall in der Größe änderst
  oder neben den Playern, wachsen die Buttons, um sie zu füllen, und Namen und
  Zeiten behalten ihre Größe in der Mitte jedes Buttons. In beiden Fällen
  scrollt die Cartwall nur, wenn selbst die kleinsten Buttons nicht passen.

| Aussehen des Buttons | Bedeutung |
|---|---|
| Violetter Punkt | Jingle |
| Bernsteinfarbener Punkt | Effekt |
| Grauer Punkt | Spot (Werbung) |
| Farbiger Rand, Tönung und Punkt | Die Farbe des Carts, in den Einstellungen gewählt: Der Punkt nimmt die Farbe statt der des Typs an |
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

Das Kürzel, das die Cartwall ein- oder ausklappt, bewirkt nichts, solange die
Cartwall in der oberen Leiste ausgeblendet ist (siehe
[Layout](layout.md#show-and-hide-panels)).

## Carts einrichten {#setting-up-carts}

Gehe zu **Einstellungen → Cartwall**:

- **Seiten:** erstellen, umbenennen, löschen (die letzte Seite lässt sich
  nicht löschen) und die Rastergröße festlegen (Zeilen × Spalten). Ein
  kleineres Raster wird abgelehnt, wenn dadurch Carts mit Datei wegfielen.
  Eine in das Feld neben einem Schieberegler getippte Größe gilt, wenn du
  Enter drückst oder das Feld verlässt; Esc bricht sie ab.
- **Importieren… / Exportieren…** speichern eine Seite in einer
  `.cartpage.json`-Datei und laden sie wieder, zum Beispiel um sie zwischen
  Studios zu teilen. Relative Dateipfade werden relativ zum Ordner der Datei
  aufgelöst. Farben wandern mit der Seite; eine Datei aus einer älteren
  Version wird ohne sie importiert.
- **Carts:** Klicke im Raster auf ein Cart und lege dann seinen Namen, seine
  Datei (**Auswählen…** oder **Leeren**), den Typ, **Farbe**, **Schleife** und **Beim
  Start andere Carts stoppen** fest. **Farbe** bietet keine Farbe und acht
  Farben (Rot, Orange, Bernstein, Grün, Türkis, Blau, Violett, Rosa): Gib
  allen Spots eine Farbe und den Senderkennungen eine andere, um sie auf einen
  Blick zu finden. Ein leeres Cart behält seine Farbe, und das Leeren der Datei
  eines Carts entfernt sie nicht. Ein Cart mit Datei hat außerdem **Von der
  Normalisierung ausnehmen** und **Trim**, die
  [Lautheitsnormalisierungs](loudness.md#per-track-exclude-and-trim)-Einstellungen
  seines Tracks, mit der Messung unter der Beschriftung.

Die eigenen Main- und Cue-Ausgänge der Cartwall findest du unter
**Einstellungen → Audioausgänge** (Zeile **Cartwall**).
