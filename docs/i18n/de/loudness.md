# Lautheitsnormalisierung

Tracks stammen aus vielen Quellen und sind mit sehr unterschiedlichen Pegeln
gemastert: Eine aktuelle Pop-Single kann doppelt so laut klingen wie eine
Aufnahme aus den 1970ern oder ein gesprochener Promo. Die
**Lautheitsnormalisierung** spielt jeden Track und jedes Cart mit derselben
empfundenen Lautheit ab, sodass die Bedienung den Fader nicht von Track zu
Track nachziehen muss.

Sie ist **bei neuen Installationen an**; eine bestehende Konfiguration behält
ihre aktuelle Einstellung (jede gespeicherte Konfiguration enthält sie
bereits). Ändere sie unter [Einstellungen → Lautheit](settings.md#loudness).

![Einstellungen, Lautheit: Normalisierung an, mit Ziel, True-Peak-Obergrenze, maximaler Anhebung und maximalem Trim auf ihren Standardwerten](../../images/guide/settings-loudness.png)

## So funktioniert es {#how-it-works}

**Eine feste Verstärkung pro Track.** Jeder Track erhält eine Verstärkung in
dB, die beim Laden des Tracks festgelegt und bis zu seinem Stopp beibehalten
wird. Es wird nichts komprimiert oder begrenzt: Der ganze Track wird um
denselben Betrag lauter oder leiser, sodass seine Dynamik so bleibt, wie sie
gemastert wurde. Der Fader des Players wirkt wie bisher zusätzlich darauf.

**Die Messung.** Die Hintergrundanalyse misst jede Datei einmal:

- ihre **integrierte Lautheit** in LUFS: wie laut die ganze Datei klingt, wie
  ITU-R BS.1770 und EBU R128 sie definieren (leise Passagen und Stille werden
  ausgeblendet, sodass ein langes Ausblenden oder eine Pause sie nicht
  senken);
- ihren **True Peak** in dBTP: der höchste Pegel, den das Signal zwischen den
  Samples erreicht, sobald es in analog umgewandelt wird.

Neue Dateien werden mit ihrer ersten Analyse gemessen. Dateien, die vor der
Lautheitsmessung hinzugefügt wurden, misst ein **Hintergrunddurchlauf**, der
läuft, solange die normale Analyse nichts zu tun hat, ohne ihre Marker
anzutasten. Er läuft bei eingeschalteter wie bei ausgeschalteter
Normalisierung, sodass die Messwerte bereitstehen, wenn du sie einschaltest.
**Neu analysieren** (im Playlist-Menü) misst eine Datei erneut und liest sie
auch dann, wenn sich ihre Größe und ihr Datum nicht geändert haben. Eine Datei
unter 0,4 s wird nie gemessen, und eine stille Datei hat keine Lautheit: Beide
werden unverändert gespielt.

**Die Verstärkung.** Die Verstärkung bringt die gemessene Lautheit auf das
**Ziel**, plus den eigenen **Trim** des Tracks (siehe [Pro
Track](#per-track-exclude-and-trim)):

> Verstärkung = Ziel − gemessene Lautheit + Trim

Ein Track, der lauter als das Ziel ist, wird so weit abgesenkt, wie nötig. Ein
leiserer Track wird angehoben, aber nie um mehr als die **Maximale Anhebung**
und nie so weit, dass sein True Peak über die **True-Peak-Obergrenze**
stiege: Es gibt keinen Limiter, die Obergrenze verhindert also, dass ein
angehobener Track übersteuert. Ein Track, dessen True Peak schon über der
Obergrenze liegt, wird gar nicht angehoben.

Beispiele mit den Standardwerten (Ziel −16 LUFS, Obergrenze −1 dBTP, maximale
Anhebung 12 dB):

| Track | Gemessen | True Peak | Verstärkung |
|---|---|---|---|
| Ein lautes modernes Master | −9.0 LUFS | +0.5 dBTP | −7.0 dB |
| Ein leiseres älteres Master | −20.0 LUFS | −10.0 dBTP | +4.0 dB |
| Ein leiser Track mit hohen Spitzen | −22.0 LUFS | −3.0 dBTP | +2.0 dB (durch die Obergrenze begrenzt) |
| Eine sehr leise Aufnahme | −35.0 LUFS | −20.0 dBTP | +12.0 dB (durch die maximale Anhebung begrenzt) |

**Wann die Verstärkung festgelegt wird.** Wenn ein Track geladen wird: als
nächster eines Players vorbereitet, gestartet, pausiert geladen, auf CUE
vorgehört oder als Cart gestartet. Ein Track, der spielt oder pausiert ist,
**behält seine Verstärkung** bis zu seinem Stopp, über Sprünge und Pausen
hinweg. Eine Änderung der Einstellungen, der Messung eines Tracks oder seines
Trims gilt beim nächsten Laden des Tracks; der nächste Track, der schon auf
einem Player wartet, wird mit der neuen Verstärkung neu vorbereitet und
startet daher mit ihr.

**Vorhören.** CUE und das Vorhören von Carts spielen einen Track mit derselben
Verstärkung ab, die er auf Sendung erhält, sodass das, was du im Kopfhörer
hörst, zum Programm passt.

**Pegelmesser.** Die Pegelmesser und ihre Lautheitsanzeige messen nach der
Verstärkung und zeigen daher den normalisierten Pegel. Das **Lautheitsziel**
der Pegelmesser selbst (Einstellungen → Pegelmesser) färbt nur die Anzeige; es
ändert keine Verstärkung.

## Bit-perfect-Ausgänge {#bit-perfect-outputs}

Ein Player oder Cart, dessen Ausgang [bit-perfect](bit-perfect.md) ist, wird
**nie normalisiert**: Er spielt immer mit Einheitsverstärkung (0 dB), egal
welche Einstellungen gelten, weil jede Verstärkung die Samples verändern
würde, die das Gerät erhält. Bei eingeschalteter Normalisierung zeigt sein
Kopf ein bernsteinfarbenes Badge **0.0 dB**, dessen Tooltip sagt:
„Bit-perfect-Ausgang: Tracks werden unverändert gespielt.“ Dasselbe gilt für
das Vorhören auf einem Bit-perfect-Cue-Gerät, während der Main-Ausgang des
Players, wenn er nicht bit-perfect ist, die Verstärkung weiterhin erhält.

Ein Track, der schon spielt (oder pausiert ist), wenn sein Gerät auf
bit-perfect umgestellt wird, behält die Verstärkung, mit der er gestartet ist,
und das Badge **BP** bleibt für ihn aus. Der nächste Track spielt mit 0 dB und
bit-perfect. Ein DSD-Track in dieser Lage wird mit seiner Verstärkung in PCM
umgewandelt gespielt, nie als unveränderter DSD-Stream.

## Die Verstärkung im Player-Kopf {#the-gain-in-the-player-header}

Solange ein Player spielt oder pausiert ist, zeigt ein Badge links von **BP**
die Verstärkung des Tracks auf Sendung. Das Badge hat einen Rand in seiner
Farbe und immer dieselbe Breite. Fahre mit der Maus darüber, um den Grund zu
sehen.

| Badge | Bedeutung |
|---|---|
| **+4.0 dB**, **−7.0 dB** (blau) | Der Track wird mit dieser Verstärkung normalisiert. Der Tooltip ist die unten beschriebene Aufschlüsselung der Verstärkung. Haben sich die Einstellungen, die Messung oder der Trim nach dem Laden des Tracks geändert, sagt er nur, dass der Track die beim Laden festgelegte Verstärkung behält, und zeigt diese Verstärkung. |
| **0.0 dB** (bernsteinfarben) | Der Track wird unverändert gespielt. Der Tooltip ist dieselbe Aufschlüsselung, und ihre Zeile **Automatische Verstärkung** nennt den Grund: Er hat noch keine Messung (oder ist zu kurz zum Messen), der Ausgang ist bit-perfect, oder der Track ist von der Normalisierung ausgenommen. |
| Kein Badge | Die Normalisierung war aus, als der Track startete, der Track ist stumm, oder der Player ist gestoppt. Ein Bit-perfect-Player zeigt bei ausgeschalteter Normalisierung ebenfalls keines. |

Das Badge zeigt die Verstärkung, die der Player beim Start des Tracks
angefordert hat.

### Die Aufschlüsselung der Verstärkung {#the-gain-breakdown}

Überall, wo das Programm die Verstärkung eines Tracks erklärt, zeigt es
denselben Block aus Zeilen, Beschriftung links und Wert rechts: der Tooltip
des Verstärkungs-Badges, das Lautheits-Popup (unter seinen Reglern), der
Cart-Editor und der Track-Tooltip der Playlist-Tabelle.

| Zeile | Bedeutung |
|---|---|
| **Integrierte Lautheit** | Die gemessene Lautheit des Tracks in LUFS (oder „nicht gemessen“). |
| **True Peak** | Sein höchster True Peak in dBTP (nur wenn gemessen). |
| **Ziel** | Die Lautheit, auf die jeder Track gebracht wird (Einstellungen → Lautheit). |
| **Automatische Verstärkung** | Ziel minus gemessene Lautheit, vor dem Trim und jeder Begrenzung, oder „keine“ und der Grund: ausgenommen, Bit-perfect, Normalisierung aus, noch nicht gemessen oder stumm. |
| **Trim** | Der verwendete Trim des Tracks, wenn er nicht 0 ist (mit dem gespeicherten Wert und dem Maximum, wenn ein gesenkter **Maximaler Trim** ihn begrenzt). |
| **Gewünschte Verstärkung** und **Begrenzt durch** | Nur wenn die True-Peak-Obergrenze oder die maximale Anhebung die Verstärkung zurückgehalten hat: die automatische Verstärkung plus der Trim (angezeigt, wenn es einen Trim gibt) und die Grenze, die sie gestoppt hat. |
| **Resultierende Verstärkung** | Immer zuletzt, unter einer Linie und fett: die Verstärkung, mit der der Track spielt. 0.0 dB, wenn er nicht normalisiert wird. |

Bei ausgeschalteter Normalisierung sagt ein Hinweis über der resultierenden
Verstärkung, dass sich auf Sendung nichts ändert und dass der Trim gilt,
sobald sie eingeschaltet wird.

## Pro Track: ausnehmen und Trim {#per-track-exclude-and-trim}

Manche Tracks sollen dem Ziel nicht folgen: eine absichtlich laut gemasterte
Senderkennung, ein Testton, ein Track, dessen Messung nicht dazu passt, wie er
klingt (Sprache klingt bei gleichen LUFS oft lauter als Musik). Jeder Track hat
dafür zwei Einstellungen:

- **Von der Normalisierung ausnehmen:** Der Track wird immer unverändert
  gespielt.
- **Trim:** dB, die zur Verstärkung des Tracks addiert werden, von − bis + dem
  **Maximalen Trim** (Einstellungen → Lautheit), in Schritten von 0,5 dB.
  Doppelklicke auf den Schieberegler, um ihn auf 0 zurückzusetzen. Der Trim
  ist abgeblendet, solange der Track ausgenommen ist.

Der Schieberegler ist ein **Trim**: ein Versatz auf der automatischen
Verstärkung, er zeigt also für jeden Track, den du nicht bearbeitet hast,
0 dB. Das Popup und der Cart-Editor zeigen deshalb die
[Aufschlüsselung der Verstärkung](#the-gain-breakdown) mit ihren Reglern: Die
Zeile **Resultierende Verstärkung** folgt dem Schieberegler, während du ihn
bewegst. Im Popup stehen die Regler zuerst und die Aufschlüsselung darunter,
sodass der Schieberegler an seinem Platz bleibt, während die Aufschlüsselung
wächst oder schrumpft. Solange der Track klingt, ergänzt das Popup, dass eine
Änderung ab dem nächsten Laden gilt.

Wo du sie änderst:

- **An einer Wellenform:** Rechtsklick auf die Wellenform eines Players (oder
  die seines CUE-Fensters) und **Lautheit…** wählen. Das Popup nennt den Track
  und zeigt seine Messung. Es bearbeitet weiter diesen Track, auch wenn der
  Player zu einem anderen übergeht; schließe es mit seinem **×**-Button. Es
  schließt sich von selbst, wenn der Track entfernt wird.
- **An einer Playlist-Zeile:** Rechtsklick auf die Zeile und **Lautheit…**
  wählen: dasselbe Popup, für den Track dieser Zeile.
- **Im Cart-Editor:** Einstellungen → Cartwall, auf ein Cart klicken: Dieselben
  zwei Regler stehen unter **Beim Start andere Carts stoppen**, mit der
  Messung unter der Beschriftung.
- **Per Fernsteuerung:** `PUT /api/v1/tracks/<id>/loudness` (siehe
  [Fernsteuerung](remote-control.md)).

Die Einstellungen gehören zum Track und werden mit ihm gespeichert. Eine
Änderung gilt beim nächsten Laden des Tracks, wie jede andere Änderung.

**Der Track-Tooltip** der Playlist-Tabelle zeigt die Tags des Tracks und,
unter einer Linie, dieselbe [Aufschlüsselung der Verstärkung](#the-gain-breakdown),
berechnet für den Player, zu dem diese Liste gehört (ein Bit-perfect-Player
spielt ihn unverändert).

## Die Einstellungen wählen {#choosing-the-settings}

- **Ziel.** −16 LUFS passt für die meisten Musikprogramme auf Sendung und
  online. −23 LUFS (EBU R128) oder −24 LUFS (ATSC A/85) lassen mehr Headroom,
  aber leise Tracks brauchen dann größere Anhebungen, die die Obergrenze und
  die maximale Anhebung womöglich zurückhalten. Wähle ein Ziel nahe an der
  Lautheit, die dein Material ohnehin hat.
- **True-Peak-Obergrenze.** −1 dBTP lässt Spielraum für die Umwandlung und für
  verlustbehaftete Encoder weiter hinten in der Kette. Senke sie, wenn ein
  Stream-Encoder übersteuert.
- **Maximale Anhebung.** Verhindert, dass eine sehr leise Aufnahme (oder eine
  größtenteils stille) ins Rauschen hochgezogen wird. 0 dB senkt nur laute
  Tracks ab.
- **Maximaler Trim.** Wie weit der Trim pro Track gehen darf. Ihn zu senken
  begrenzt bereits gesetzte Trims, ohne sie zu ändern, bis du die Lautheit
  dieses Tracks das nächste Mal bearbeitest: Die Bearbeitung speichert dann
  den begrenzten Trim.

## Die Normen hinter den Standardwerten {#standards-behind-the-defaults}

- **Messung.** Integrierte Lautheit und True Peak, wie ITU-R BS.1770 sie
  definiert (K-Gewichtung, gegatet).
- **Ziel −16 LUFS.** Es folgt den AES-Empfehlungen für Streaming und
  Online-Verbreitung. AES TD1008 (2021) ersetzt die frühere AES TD1004.1.15-10,
  die −16 bis −20 LUFS für Streaming empfahl. Es ist kein Rundfunkwert: EBU
  R128 verwendet −23 LUFS und ATSC A/85 −24 LKFS. Verwende einen davon, wenn
  eine Lieferspezifikation es verlangt (EBU R128 für den europäischen
  Programmaustausch im Rundfunk, ATSC A/85 für das digitale Fernsehen in den
  USA), und −16 für einen Stream oder ein Online-Programm.
- **True-Peak-Obergrenze −1 dBTP.** Der maximale True Peak der EBU R128, den
  auch die AES-Empfehlungen verwenden. Verlustbehaftete Encoder können beim
  Dekodieren überschwingen, daher kann ein Stream, der durch einen Encoder mit
  niedriger Bitrate läuft, eine niedrigere Obergrenze wie −2 dBTP wünschen
  (gängige Praxis, nicht Teil dieser Normen).
- **Maximale Anhebung und maximaler Trim, 12 dB.** Produktentscheidungen ohne
  Norm dahinter: genug, um eine leise Aufnahme anzuheben und einen Track nach
  Gehör zu korrigieren, nicht so viel, dass Rauschen angehoben wird oder der
  Trim das Ziel aushebelt.
- **Die Normalisierung ist bei neuen Installationen an.** Ein Update der
  Anwendung behält die Einstellung, die deine Konfiguration schon hat, sodass
  eine Installation, bei der sie aus war, aus bleibt, bis du sie einschaltest.
- **Das Lautheitsziel der Pegelmesser** (Einstellungen → Pegelmesser) ist eine
  getrennte Einstellung, standardmäßig der R128-Wert von −23 LUFS. Die
  Normalisierung verwendet es nicht.
