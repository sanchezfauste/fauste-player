# Marker und Mixen

Jeder Track hat bis zu fünf **Marker**, in Sekunden:

| Marker | Bedeutung | Wie er gesetzt wird |
|---|---|---|
| Cue-in | Wo die Wiedergabe beginnt | Automatisch: kurz vor dem ersten Ton über der Trim-Schwelle |
| Cue-out | Wo der Track endet | Automatisch: kurz nach dem letzten Ton über der Trim-Schwelle |
| MIX (Segue-Start) | Wo im CONT-Modus der nächste Track beginnt | Automatisch (siehe unten) |
| Outro-Anfang | Wo das Ende des Tracks beginnt | Automatisch (siehe unten) |
| Intro-Ende | Ende der besprochenen Einleitung | Von Hand oder aus einem `INTRO`-Tag in der Datei |

Von Hand gesetzte Marker gewinnen immer: Eine neue Analyse ersetzt sie nie.

## Marker bearbeiten {#editing-markers}

In der Wellenform eines Players oder in der Wellenform seines CUE-Fensters
(dasselbe Menü und dieselben Griffe; eine Änderung erscheint in beiden
gleichzeitig):

- **Rechtsklick** dorthin, wo ein Marker hin soll, und **Cue-in hier setzen**,
  **Intro-Ende hier setzen**, **Outro-Anfang hier setzen**, **MIX-Punkt hier
  setzen** oder **Cue-out hier setzen** wählen. **Marker auf automatisch
  zurücksetzen** entfernt die von dir gesetzten Marker, und der Track wird neu
  analysiert.
- **Alt halten** (Option unter macOS): An den Markern erscheinen Griffe. Ziehe
  einen, um ihn zu verschieben; beim Ziehen wird die Zeit angezeigt. Ein Ziehen
  bewegt nie die Abspielposition.

Cue-in muss vor Cue-out liegen. Die anderen Marker bleiben dazwischen. Änderungen
am Track, der on Air ist, gelten sofort für seinen nächsten Übergang.

## Der INTRO-Tag {#the-intro-tag}

Eine Datei kann ihre Intro-Zeit in einem `INTRO`-Tag tragen, als Sekunden
(`12.5`) oder `m:ss`. Der Name darf in beliebiger Groß- und Kleinschreibung
stehen (`INTRO`, `Intro`). Es kann ein ID3v2-Benutzertext-Frame sein (MP3, WAV,
AIFF, DSF), ein Vorbis-, Opus- oder FLAC-Kommentar, ein APE-Eintrag (WavPack,
Monkey's Audio) oder ein MP4-Freeform-Atom. Er wird bei der Analyse gelesen. Ein
von Hand gesetztes Intro-Ende gewinnt weiterhin.

## Wie die automatischen Marker gefunden werden {#how-the-automatic-markers-are-found}

Die Analyse misst die Spitzen des Tracks in 10-ms-Schritten und seine Lautheit in
kurzen Fenstern (standardmäßig 50 ms).

- **Cue-in / Cue-out:** Nur nahezu Stille am Anfang und Ende wird übersprungen:
  Alles, dessen Spitze auf einem der beiden Kanäle die *Trim-Schwelle*
  (standardmäßig −60 dBFS) erreicht, bleibt erhalten, mit einer *Trim-Reserve*
  (standardmäßig 20 ms) darum herum. Sanfte Einblendungen, leise Ausklänge und
  kurze Klänge werden nie abgeschnitten.
- **MIX:** Die Analyse findet den letzten Punkt, an dem der Track noch weniger
  als den *Pegelabfall für Segue* (standardmäßig 15 dB) unter seiner eigenen
  typischen Lautheit liegt, sodass laute und leise Master mit derselben
  Ausblendung gleich mixen. Dieser Punkt liegt nie weiter als die *maximale
  Mixlänge* (standardmäßig 4 s) vor dem Cue-out, damit die Überlappungen kurz
  bleiben.
- **Outro:** Die Analyse sucht vom Cue-out rückwärts und findet, wo der Pegel
  mehr als den *Pegelabfall für Outro* (standardmäßig 6 dB) unter die mediane
  Lautheit des Tracks fällt. Das Outro ist standardmäßig nie länger als 30 s.
- Tracks, die kürzer sind als die *Mindestlänge für Mix- und Outro-Marker*
  (standardmäßig 60 s), wie Jingles und Werbung, erhalten weder MIX noch Outro.

### Lange Aufnahmen {#long-recordings}

Ein ganzes Programm (eine, vier oder mehr Stunden) wird wie ein Song analysiert,
notfalls während es spielt: Eine 4-stündige FLAC- oder Opus-Datei braucht auf
einem aktuellen Computer weniger als eine Minute, und der Speicherbedarf wächst
nicht mit der Länge. Das Springen an eine beliebige Stelle, auch nahe dem Ende,
ist sofort möglich. Wellenform und Marker werden im Analyse-Cache bis zu etwa 16
Stunden Audio aufbewahrt; eine längere Datei funktioniert ebenfalls, wird aber
bei jedem Start von Fauste Player neu analysiert.

Alle diese Werte stehen unter **Einstellungen → Analyse**. Nach ihrer Änderung
werden die Tracks automatisch neu analysiert.

## Was der Player damit macht {#what-the-player-does-with-them}

- **CONT-Modus mit automatischem Mix:** Am MIX-Punkt startet der nächste Track
  mit vollem Pegel, während der aktuelle bis zu seinem Cue-out ausgeblendet wird.
  Die Überlappung ist samplegenau.
- **CONT-Modus ohne MIX-Punkt** oder mit ausgeschaltetem automatischem Mix: Der
  nächste Track startet genau am Cue-out, ohne Lücke.
- **SINGLE-Modus** oder **Stopp danach**: Der Player stoppt am Cue-out.
- **Play drücken, während on Air:** Der nächste Track startet sofort, und der
  aktuelle wird über die *Blendzeit* (standardmäßig 1 s) ausgeblendet.
- **Cue-in und Cue-out verwenden ausgeschaltet** (Einstellungen → Player): Jeder
  Player spielt jeden Track von 0 bis zum Ende der Datei. Cue-in und Cue-out,
  automatisch und manuell, bleiben erhalten, und die Wellenform zeichnet sie als
  blasse Linien. Der MIX-Punkt, das Intro und das Outro funktionieren weiterhin,
  innerhalb der ganzen Datei; der **Automatische Mix** ist ein eigener Schalter.
  Countdowns, die Spalte Dauer, die Playlist-Summen in den Einstellungen und die
  Zeiten der Remote-API folgen demselben Bereich. Carts verwenden immer ihr
  eigenes Cue-in und Cue-out. Das Ändern der Einstellung startet, springt in oder
  stoppt nie einen Track, der gerade spielt; der nächste Track wird neu
  vorbereitet.

Ein Track kann abgespielt werden, bevor seine Analyse fertig ist. Bis dahin
spielt er vom Anfang bis zum Ende der Datei, ohne MIX-Punkt.
