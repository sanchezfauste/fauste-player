# Einstellungen

Öffne **Einstellungen** in der oberen Leiste. Schließe sie mit **Schließen**
oder `Esc`. Die meisten Änderungen gelten sofort und werden automatisch
gespeichert.

Das Fenster hat unabhängig vom Abschnitt eine feste Größe (900 × 640, auf einem
kleinen Bildschirm kleiner), und der Abschnitt scrollt darin. In jedem Abschnitt
sind die Beschriftungen in einer Spalte ausgerichtet.

Die Abschnitte Player, Pegelmesser, Analyse, Lautheit, Automatisierung und Tastenkürzel haben in ihrer
Kopfzeile einen Button **Standardwerte wiederherstellen**. Er fragt nach einer
Bestätigung und setzt dann nur diesen Abschnitt zurück (Player behält die Anzahl
der Player und die Sprache; Tastenkürzel hat keinen weiteren Zurücksetzen-Button).
Audioausgänge, Playlists, Cartwall, Bibliothek, MIDI und Fernsteuerung haben keinen.

## Einstellungen ausstehend {#settings-pending}

Jede Einstellung gilt, während die Anwendung läuft. Das Audiosystem, die
Abtastrate, die Puffergröße (auch die eines einzelnen Geräts), die Main- und
Cue-Ausgänge (Player und Cartwall), die Bit-perfect-Geräte und die
DSD-Einstellungen ändern, wie ein Ausgabegerät geöffnet wird, daher gelten sie,
sobald das sicher ist:

- Die Einstellungen eines Geräts gelten, wenn nichts auf diesem Gerät spielt:
  Kein darauf geroutetes Player spielt, blendet aus oder hört vor (CUE, auch
  angehalten), und kein Cart spielt darauf. Ein pausierter Player oder einer mit
  nur geladenem Track hält ein Gerät nicht fest: Sein Track bleibt pausiert, wo
  er war.
- Der Ausgang eines Players oder der Cartwall wechselt, wenn dieser Player (oder
  die Cartwall) nicht spielt; andere Player auf denselben Geräten werden nicht
  unterbrochen.
- Ein neues Audiosystem gilt, wenn nirgends etwas spielt.

Eine wartende Änderung läuft im ersten Moment, in dem nichts spielt, was sie
betrifft. Das kann die Lücke zwischen dem Ende eines Tracks und dem Start des
nächsten sein: Der nächste Track startet dann um die Zeit später, die das Gerät
zum erneuten Öffnen braucht. **Jetzt anwenden** (unten) erzwingt die Änderung
stattdessen sofort.

Eine Rate oder ein Puffer, die einem Gerät gegeben werden, zählen nur, wenn sie
ändern, womit das Gerät geöffnet wird: Einem Gerät denselben Wert wie den
globalen zu geben oder einen solchen Wert zu leeren, ändert nichts.

Bis dahin zeigt die obere Leiste **Einstellungen ausstehend**. Fahre mit der Maus
darüber, um zu sehen, was worauf wartet; klicke darauf für die Liste, die auch
jede Änderung zeigt, die ein Gerät oder ein Ausgang abgelehnt hat (das Gerät
behält seine aktuellen Einstellungen; die Änderung wird erneut versucht, wenn du
sie änderst oder **Jetzt anwenden** drückst). Die Fußzeile der Einstellungen
sagt „Einige Änderungen warten, bis die betroffenen Ausgänge frei sind.“ und
bietet **Jetzt anwenden** an; eine kurze Meldung (zum Beispiel, dass eine
Einstellung gespeichert wurde) kann für einen Moment diesen Text ersetzen.

**Jetzt anwenden** wendet jede wartende Ausgangsänderung sofort an. Würde das
etwas unterbrechen, listet es zuerst die Geräte und was darauf spielt auf;
**Unterbrechen und anwenden** unterbricht dann kurz das Audio auf diesen Geräten:
Was spielte, läuft nach einer kurzen Lücke dort weiter, wo es war, und nichts
Pausiertes oder Gestopptes startet. **Abbrechen** (oder `Esc`) wartet weiter.

Die Grenzwerte und die Engine-Feinabstimmung werden bei geschlossener Anwendung
in der Konfigurationsdatei bearbeitet (siehe [Daten und
Backups](data-and-backups.md)).

## Audioausgänge {#audio-outputs}

Änderungen in diesem Abschnitt gelten, während die Anwendung läuft, sobald nichts
spielt, was sie betrifft: siehe [Einstellungen ausstehend](#settings-pending).

![Einstellungen, Audioausgänge, Ansicht Einfach: die Auswahl, das Audiosystem, die Abtastrate, die Puffergröße und die Main- und Cue-Ausgänge jedes Players (hier das stumme System)](../../images/guide/settings-outputs.png)

Oben wählt **Anzeigen** zwischen **Einfach** und **Erweitert**. Einfach zeigt das
Audiosystem, die Abtastrate, die Puffergröße und die Ausgänge. Erweitert fügt
für jedes Gerät, das ein Ausgang verwendet, seine eigene Rate und seinen eigenen
Puffer, den Bit-perfect-Schalter und den DSD-Modus hinzu, und danach die
DSD-Einstellungen. Das Umschalten der Ansichten blendet nur Zeilen ein oder aus:
Nichts wird geändert oder zurückgesetzt. Blendet Einfach eine verwendete
Einstellung aus, weist eine Zeile darauf hin. Wird ein Gerät von keinem Ausgang
mehr verwendet, werden seine eigene Rate und sein eigener Puffer, sein
Bit-perfect-Schalter und sein DSD-Modus beim nächsten Start der Anwendung
vergessen: Verwendet ein Ausgang es danach wieder, beginnt es mit den globalen
Werten. Bis dahin behält es sie, wenn du es erneut wählst (zum Beispiel nach dem
Tausch zweier Geräte).

| Einstellung | Bedeutung |
|---|---|
| Audiosystem | Die letzte Wahl, **Keine Ausgabe (stumm)**, spielt nichts ab: Die Zeitachsen laufen in Echtzeit ohne Soundkarte (für eine Maschine ohne eine oder zum Proben). Linux: PipeWire (in Builds, die es enthalten), PulseAudio, JACK oder ALSA. Windows: WASAPI, ASIO (in Builds, die es enthalten) oder JACK. macOS: Core Audio oder JACK. Systeme, die auf diesem Computer fehlen oder kein Ausgabegerät haben (ein JACK-Server, der nicht läuft), werden als nicht verfügbar angezeigt. „Systemstandard“ verwendet in dieser Reihenfolge das erste verfügbare. |
| Abtastrate | Die Rate, mit der jeder Ausgang läuft, sofern ein Gerät keine eigene hat (Erweitert); Dateien werden mit hochwertigem Resampling in sie umgewandelt. Bit-perfect-Geräte starten mit ihrer Rate und folgen dann den Dateien. |
| Puffergröße | Frames pro Audioblock, sofern ein Gerät keine eigene hat; die resultierende Latenz steht darunter. Ausgänge über einen Soundserver oder einen System-Mixer (PulseAudio, das ALSA-Gerät `default`) verlangen mindestens 50 ms, egal welche Größe hier steht, oder so viel, wie das Gerät annimmt, damit ein ausgelasteter Computer nicht zu hören ist; die Latenz sagt es dann (`shared_buffer_ms` in der Konfigurationsdatei) |
| Ausgänge pro Player | Für jeden Player und für die Spalte **Musik** (den [Musikfüller](automation.md)) ein **Main**-Gerät (On Air) und ein **Cue**-Gerät (Vorhören), jeweils mit einem Kanalpaar. Eine Soundkarte, die mehrere Ausgabeprofile anbietet (ALSA listet Front, Surround, direkte Hardware…), zeigt jedes als *Karte — Profil*; zwei Einträge, die dennoch gleich lauten würden, erhalten ihre Geräte-ID in Klammern. Mehrkanal-Interfaces können mehrere Player auf verschiedenen Paaren tragen. Ein Cue-Ausgang, der einen Sendeausgang teilt (den Main eines Players oder den der Cartwall), ist erlaubt: Ein Hinweis darunter sagt, dass sein Vorhören auf Sendung zu hören ist und dass zuerst gefragt wird, solange dort etwas spielt. |
| Main testen / Cue testen | Spielt einen kurzen Ton (1 kHz auf Main, 440 Hz auf Cue, 1,5 s, −18 dBFS) auf dem gewählten Ausgang ab, damit du die Verkabelung prüfen kannst, bevor du auf Sendung gehst |
| Bibliothek | Der Cue-Ausgang der Bibliothek, mit eigenem **Cue testen**: wo das [Vorhören](library.md#pre-listening) des Bibliotheksfensters (auf einem Telefon das des Tabs Bibliothek) zu hören ist. Ohne ihn ist Vorhören in der Vorhörleiste abgeblendet. Unter Android lautet seine leere Auswahl *Systemstandard* statt *Keiner*: Das Vorhören und **Cue testen** spielen dann auf dem Standardausgang (siehe [Android → Bibliothek](android.md#library)). Ein Ausgang Bibliothek auf einem Sendeausgang funktioniert wie der Cue eines Players: Ein Hinweis darunter sagt, dass sein Vorhören auf Sendung zu hören ist und dass zuerst gefragt wird, solange dort etwas spielt. Er steht in der Ansicht Einfach und Erweitert, auch auf einem Telefon |
| Cartwall | Die Main- und Cue-Ausgänge der Cartwall. Main ist standardmäßig der Systemausgang. Ohne Cue gibt es kein Cart-Vorhören; ein Cue auf einem Sendeausgang funktioniert wie bei den Playern. |
| Abtastrate: *Gerät* (Erweitert) | **Global (...)** verwendet die obige Abtastrate; ein Wert gibt diesem Gerät seine eigene Rate. Es werden nur die Raten angeboten, die das Gerät meldet; eine gespeicherte Rate, die es nicht mehr meldet, bleibt mit dem Hinweis gelistet, dass es sich eventuell nicht öffnen lässt (das Gerät fällt dann auf die globale Rate zurück). Die eigenen Werte gelten nur für Geräte, die ein Ausgang benennt, nicht für den Systemstandardausgang, außer wenn einer es tut. |
| Puffergröße: *Gerät* (Erweitert) | **Global (...)** verwendet die obige Puffergröße (mit dem Mindestwert, den ein gemeinsam genutzter Ausgang verlangt, wie oben); ein Wert gibt diesem Gerät seine eigene, unverändert verwendet, mit ihrer Latenz darunter. Ein Gerät, das keine eigene Puffergröße annimmt, fällt auf die globale zurück, und auch auf die globale Rate, wenn es keine eigene Rate annimmt. |
| Bit-perfect: *Gerät* (Erweitert) | Standardmäßig aus. Ein Sondermodus für audiophile Wiedergabe, nicht für gemischte Programme: Das Gerät wird mit exklusivem Zugriff geöffnet und folgt der Abtastrate jedes Tracks. Die Player, deren Main-Ausgang darauf liegt, überblenden oder mischen nie, ein Ratenwechsel hinterlässt eine kurze Stille zwischen zwei Tracks, und ihre Lautstärke ist auf 100 % festgehalten. Der Schalter ist deaktiviert, wo das Gerät keinen exklusiven Zugriff geben kann. Siehe [Was es kostet](bit-perfect.md#what-it-costs). |
| DSD: *Gerät* (Erweitert) | **In PCM umwandeln** (Standard), **DoP** oder, unter Linux, **Natives DSD**. Jedes Gerät zeigt es; angeboten werden nur die Modi, die das Gerät annehmen kann, und eine Zeile darunter sagt, warum die anderen nicht. Siehe [DSD](bit-perfect.md#dsd). |
| Wenn eine andere Quelle einen DSD-Ausgang braucht (Erweitert) | **DSD-Track als PCM fortsetzen** (Standard) oder **DSD behalten und andere Quellen stummschalten**. Siehe [DSD](bit-perfect.md#dsd). |
| DSD-Stille (Erweitert) | Stille, die vor dem Start eines DSD-Streams, nach seinem Ende und beim Wechsel zu PCM gesendet wird, damit der Wandler ohne Knacken einrastet; standardmäßig 200 ms, 0 bis 2000. |

Ein Cue-Ausgang fällt nie auf den Ausgang zurück, den Main verwendet. Ein Cue,
der ein Gerät an einem Audiosystem nennt, das dieser Computer nicht hat,
bedeutet „kein Cue“. Ein Cue auf einem Sendeausgang (dasselbe Gerät und dieselben
Kanäle wie ein Main, oder sich mit ihnen überschneidend) ist erlaubt: Ein Hinweis
darunter sagt, dass sein Vorhören auf Sendung zu hören ist und dass zuerst
gefragt wird, solange dort etwas spielt. Bei einem Player ohne Cue-Ausgang ist
der **CUE**-Button abgeblendet; beim Darüberfahren steht, dass du hier einen
Cue-Ausgang wählen sollst. Dasselbe gilt für **Auf CUE vorhören** der Cartwall.

Verschwindet ein Gerät während der Wiedergabe, behalten die Player ihre
Zeitachsen, und das Gerät wird wieder geöffnet, wenn es zurückkommt (siehe
[Fehlerbehebung](troubleshooting.md)).

## Player {#players}

![Einstellungen, Player: Anzahl der Player, Standardmodus, Überblendzeit, Ausblendzeit, automatischer Mix, Cue-in und Cue-out, Warnung vor Track-Ende und Sprache](../../images/guide/settings-players.png)

| Einstellung | Standard | Bedeutung |
|---|---|---|
| Sprache | System | Sprache der Oberfläche |
| Anzahl der Player | 4 | Spalten im Hauptfenster (ein Player on Air kann nicht entfernt werden). Die Zahl gilt, wenn du den Schieberegler loslässt; eine in das Feld getippte Zahl gilt, wenn du Enter drückst oder das Feld verlässt, und Esc bricht ab |
| Standardmodus | CONT | Der Modus, mit dem die Player starten |
| Überblendzeit | 1000 ms | Wie lange der Track on Air ausgeblendet wird, wenn du Play (Weiter) drückst, während er spielt, und bei Zurück. Gilt nicht für einen Player, dessen Main-Ausgang bit-perfect ist |
| Ausblendzeit | 2000 ms | Wie lange **Ausblenden** den Track ausblendet. Gilt für jeden Player, auch für Bit-perfect-Player |
| Automatischer Mix am MIX-Punkt | An | Tracks im CONT-Modus überlappen. Gilt nicht für einen Player, dessen Main-Ausgang bit-perfect ist |
| Cue-in und Cue-out verwenden | An | Aus: Die Player spielen jeden Track vom Anfang bis zum Ende der Datei; die Marker bleiben erhalten, und Carts verwenden weiterhin ihre eigenen. Dauern und Playlist-Summen folgen demselben Bereich |
| Warnung vor Track-Ende | 10 s | Wann der Countdown rot zu blinken beginnt |

Solange der Main-Ausgang eines Players bit-perfect ist, sagt ein Hinweis unter
**Automatischer Mix am MIX-Punkt**, dass solche Player nie mischen oder
überblenden, die Überblendzeit und der automatische Mix für sie also nicht
gelten. Beide Einstellungen bleiben bearbeitbar: Sie gelten weiterhin für
die anderen Player. Siehe [Was es kostet](bit-perfect.md#what-it-costs).

## Pegelmesser {#meters}

![Einstellungen, Pegelmesser, mit gewähltem digitalem Spitzenpegelmesser](../../images/guide/settings-meters.png)

Änderungen gelten sofort. Die Einstellungen zeigen nur, was der gewählte
Messertyp verwendet: Die EBU-, DIN- und VU-Messer haben die Skala, den roten
Bereich und das Verhalten, die ihre Norm festlegt (nur der Abgleichpegel wird
eingestellt), und der Abgleich eines K-System-Messers ist sein eigenes 0. Ein
von dir eingestellter Wert bleibt erhalten, für den Fall, dass du diesen Typ
erneut wählst.

| Einstellung | Standard | Bedeutung |
|---|---|---|
| Messertyp | Digitaler Spitzenpegel | Wie der Balken steigt und fällt, und seine Skala, nach einer Norm (siehe unten) |
| Anstiegszeit, Rücklaufrate | 5 ms, 11,8 dB/s | Nur für **Benutzerdefiniert**. Die Anstiegszeit ist eine Integrationszeit: Ein Tonburst dieser Länge wird 2 dB zu niedrig angezeigt; 0 zeigt jede Spitze. |
| True Peak | Aus | Nur digitaler Spitzenpegel, benutzerdefiniert und K-System. Misst zwischen den Samples, mit dem 4×-Oversampling-Filter, den ITU-R BS.1770 veröffentlicht. Es zeigt Spitzen, die nach der Umwandlung 0 dBFS überschreiten und die ein Sample-Peak-Messer übersieht. Wie die Norm es erlaubt, kann ein einzelner Ein-Sample-Klick bis zu etwa 0,3 dB unter seinem Sample-Wert angezeigt werden. |
| Skalenuntergrenze | −60 dBFS | Das untere Ende der digitalen Skala (digitaler Spitzenpegel und benutzerdefiniert). Die anderen Messer zeigen den Bereich, den ihre Norm vorgibt. |
| Spitzenwert halten | 2 s | Nur digitaler Spitzenpegel, benutzerdefiniert und K-System: wie lange der höchste Pegel aufleuchtet; 0 schaltet es aus. Programmmesser und das VU halten nichts. |
| Abgleichpegel | −18 dBFS | Alle außer K-System. Auf der Skala markiert (EBU R68). Dort liegen auch die EBU-TEST-Marke, die DIN-Marke −9 und 0 VU. |
| Warnung ab | −9 dBFS | Gelb ab hier (EBU-Höchstpegel), für den digitalen Spitzenpegelmesser und den benutzerdefinierten |
| Gefahr ab | −3 dBFS | Rot ab hier, für den digitalen Spitzenpegelmesser und den benutzerdefinierten. Die anderen werden dort rot, wo es ihre Skala vorsieht: VU ab 0 VU, EBU- und DIN-PPM ab dem erlaubten Höchstpegel (EBU +9, DIN 0), das K-System ab +4. |
| Lautheitsanzeige | Kurzzeit | Die Lautheit unter dem Messer: aus, momentan (letzte 400 ms) oder Kurzzeit (letzte 3 s), EBU R128 |
| Lautheitsziel | −23 LUFS | Die Anzeige ist innerhalb von ±1 LU grün (EBU R128) |

| Messertyp | Norm | Verhalten |
|---|---|---|
| Digitaler Spitzenpegel | IEC 60268-18 | Zeigt jede Spitze sofort; fällt in 1,7 s um 20 dB |
| EBU-PPM | IEC 60268-10 Typ IIb | Spitzen, die kürzer als etwa 10 ms sind, werden niedriger angezeigt (ein 10-ms-Tonburst etwa 1,6 dB zu niedrig, ein 0,5-ms-Burst etwa 18 dB zu niedrig), innerhalb der Toleranzen von EBU Tech 3205; fällt in 2,8 s um 24 dB |
| DIN-PPM | IEC 60268-10 Typ I | Dasselbe mit 5 ms Integrationszeit; fällt in 1,5 s um 20 dB |
| VU | IEC 60268-17 | Der Durchschnittspegel, mit der Zeigerbewegung eines VU-Messers: 99 % in 300 ms, mit leichtem Überschwingen; ein Sinus zeigt seinen Spitzenpegel |
| K-20, K-14, K-12 | K-System | Zwei Abschnitte: der Durchschnitt (RMS, 600 ms) als voller Balken und die Spitze (fällt in 3 s um 26 dB) heller darüber. 0 liegt 20, 14 oder 12 dB unter dem Vollausschlag; grün unter 0, bernsteinfarben von 0 bis +4, rot darüber. K-12 eignet sich für Rundfunk, K-14 und K-20 für dynamischeres Programm. |
| Benutzerdefiniert | — | Deine Anstiegszeit und Rücklaufrate |

Jeder Messer verwendet die Skala seiner Norm, mit den Marken zwischen den Kanälen:

| Messer | Skala |
|---|---|
| Digitaler Spitzenpegel, benutzerdefiniert | −60 … 0 dBFS, Marken alle 10 dB bis −40 und alle 5 dB darüber; die obersten 20 dB nehmen die halbe Höhe ein |
| EBU-PPM | −12 … +12 um den Abgleichpegel (TEST), alle 4 dB; leisere Pegel ruhen am unteren Ende |
| DIN-PPM | −50 … +5, wobei 0 9 dB über dem Abgleichpegel liegt (standardmäßig −9 dBFS) |
| VU | −20 … +3 VU, 0 VU am Abgleichpegel; der Balken bewegt sich proportional zur Spannung, wie der Zeiger |
| K-System | von +20, +14 oder +12 (0 dBFS) hinunter bis −60; gleichmäßig in dB bis −24 |

## Analyse {#analysis}

![Einstellungen, Analyse: die Schwellen der automatischen Marker](../../images/guide/settings-analysis.png)

Die Schwellen sind in [Marker und Mixen](markers-and-mixing.md) beschrieben;
jede Zeile hat unter ihrem Namen einen einzeiligen Hinweis, was der Wert bewirkt.
**Alle Tracks neu analysieren** führt die Analyse für die ganze Bibliothek
erneut aus und liest dabei jede Datei erneut; manuelle Marker bleiben erhalten.
Solange sie läuft, ist der Button abgeblendet und zeigt, wie weit sie ist
(**Analysiere… 120 von 340**), sodass ein erneutes Drücken nicht von vorn
beginnt; der Tooltip von **Analysiere N Titel** in der Statusleiste nennt
denselben Fortschritt. Auch das Ändern einer Schwelle hier analysiert die
Bibliothek erneut und startet einen bereits laufenden Durchlauf neu, damit kein
Track Marker der alten Werte behält.

Nach einem Update, bei dem sich die Analyse geändert hat, behalten Tracks, die
von der früheren Version analysiert wurden, ihre Marker und Wellenformen, die
weiterhin funktionieren. Beim Start sagt Fauste Player, wie viele es sind, und
bietet **Jetzt analysieren** oder **Später** an; **Veraltete Tracks analysieren
(N)** hier tut jederzeit dasselbe. Die Tracks auf den Playern werden ohnehin auf
den neuesten Stand gebracht, sobald sie angezeigt werden, ebenso die Tracks von
Carts, für die kein Format erfasst ist (ein Cart spielt nur bit-perfect, wenn
sein Format bekannt ist). Tracks, deren Datei fehlt, werden nicht gezählt, bis
die Datei wieder da ist.

Die Analyse misst außerdem die Lautheit jeder Datei für die
[Lautheitsnormalisierung](loudness.md). Dateien, die analysiert wurden, bevor die
Lautheit gemessen wurde, werden im Hintergrund gemessen, wenn die Analyse nichts
anderes zu tun hat, ob die Normalisierung an oder aus ist; ihre Marker werden
nicht angetastet. **Neu analysieren** (im Playlist-Menü) und **Alle Tracks neu
analysieren** messen erneut.

**Nach geänderten Dateien suchen** prüft jeden Track der Bibliothek auf eine
Datei, die ein anderes Programm geändert hat, während Fauste Player lief, und
analysiert die geänderten neu; ein Hinweis sagt, wie viele es gefunden hat. Der
Button ist abgeblendet, solange eine Suche läuft. Siehe [Wenn sich eine Datei
ändert](library.md#when-a-file-changes).

**Nach geänderten Dateien suchen** prüft jeden Track der Bibliothek auf eine
Datei, die ein anderes Programm geändert hat, während Fauste Player lief, und
analysiert die geänderten neu; ein Hinweis sagt, wie viele es gefunden hat. Der
Button ist abgeblendet, solange eine Suche läuft. Siehe [Wenn sich eine Datei
ändert](library.md#when-a-file-changes).

## Lautheit {#loudness}

![Einstellungen, Lautheit: Normalisierung an, mit Ziel, True-Peak-Obergrenze, maximaler Anhebung und maximalem Trim auf ihren Standardwerten](../../images/guide/settings-loudness.png)

Spielt jeden Track und jedes Cart mit derselben Lautheit, mit einer festen
Verstärkung pro Track. Die [Lautheitsnormalisierung](loudness.md) erklärt, wie die
Verstärkung berechnet wird, das Badge in der Player-Kopfzeile und die Einstellungen
pro Track.

| Einstellung | Standard | Bereich | Bedeutung |
|---|---|---|---|
| Lautheit normalisieren | An | | Normalisierung an oder aus |
| Ziel | −16 LUFS | −36 … −6 LUFS, Schritte von 0,5 | Die integrierte Lautheit, auf die jeder Track gebracht wird |
| True-Peak-Obergrenze | −1 dBTP | −12 … 0 dBTP, Schritte von 0,1 | Eine Anhebung bringt den True Peak des Tracks nie über diesen Wert. Absenkungen sind nicht begrenzt |
| Maximale Anhebung | 12 dB | 0 … 24 dB | Die größte Verstärkung, die ein leiser Track erhalten kann. Absenkungen sind nicht begrenzt |
| Maximaler Trim | 12 dB | 0 … 24 dB | Wie weit der eigene Trim jedes Tracks gehen kann, nach oben oder unten |

Änderungen gelten für Tracks, die danach geladen werden: Was spielt, behält seine
Verstärkung, und der nächste Track, der auf jedem Player wartet, wird mit der
neuen neu vorbereitet. Ein Hinweis unter den Einstellungen sagt, wie viele Tracks
noch auf ihre Messung warten; sie spielen unverändert, bis sie fertig ist. Ein
Player oder Cart an einem Bit-perfect-Ausgang wird nie normalisiert.
**Standardwerte wiederherstellen** setzt nur diesen Abschnitt zurück
(Normalisierung an). Die Normalisierung ist bei neuen Installationen an; eine
bestehende Konfiguration behält ihre aktuelle Einstellung (jede gespeicherte
Konfiguration enthält den Wert bereits).

## Playlists {#playlists}

![Einstellungen, Playlists: der Musikordner, die Playlists und die Tabellenspalten](../../images/guide/settings-playlists.png)

- **Musikordner:** wo die Dateidialoge beginnen.
- **Neue Playlist**, **umbenennen** (den Namen bearbeiten und Enter drücken; Esc
  bricht ab) und **löschen** (Papierkorb-Symbol).
- **M3U / PLS importieren…** erstellt aus einer Playlist-Datei eine neue
  Playlist. **M3U** in jeder Zeile exportiert sie als M3U8. Siehe
  [Playlists](playlists.md).
- **Tabellenspalten:** welche Spalten die Track-Tabellen zeigen und in welcher
  Reihenfolge, für jeden Player: ein Kontrollkästchen pro Spalte (Titel und Dauer
  lassen sich nicht ausschalten), Pfeile nach oben und unten für die angezeigten
  und **Standardspalten**. Siehe [Playlists](playlists.md).

**Sprache:** eine Dropdown-Liste: **System** (dem Betriebssystem folgen), dann
jede Sprache, in der die Oberfläche verfügbar ist, jeweils unter ihrem eigenen
Namen (zuerst Englisch, dann alphabetisch: zum Beispiel Español). Die
Oberfläche wechselt sofort. Eine Systemsprache ohne eigene Übersetzung verwendet
die nächstliegende (kanadisches Französisch verwendet Französisch, brasilianisches
Portugiesisch verwendet Portugiesisch), sonst Englisch. Eine Sprache in der
Einstellungsdatei, die die Oberfläche nicht hat, wird als **System** angezeigt
und folgt dem Betriebssystem.

Die Oberfläche in allen Sprachen, auch Englisch und Spanisch, wurde mit KI
erstellt und kann Fehler enthalten. Das Fenster „Über Fauste Player“ trägt einen
allgemeinen Haftungsausschluss, der alle abdeckt. Korrekturen sind als Issues
oder Pull Requests willkommen.

## Cartwall {#cartwall}

![Einstellungen, Cartwall: die Seiten, das Raster und der Editor des gewählten Carts](../../images/guide/settings-cartwall.png)

Seiten, Rastergröße, der Cart-Editor sowie Import und Export von Cart-Seiten.
Siehe [Cartwall](cartwall.md).

## Bibliothek {#library}

Lege die Kategorien an, die deine Tracks gruppieren, benenne sie um, ändere ihre
Farbe und Art, verschiebe sie (durch Ziehen) und lösche sie, überwache einen
Ordner mit einer Kategorie und entferne ihre fehlenden Tracks, ohne das
Bibliotheksfenster zu öffnen. Siehe [Bibliothek](library.md#categories) und
[Überwachte Ordner](library.md#watched-folders).

**Externer Editor** ist der Audio-Editor, den *In externem Editor öffnen* mit
der Datei eines Tracks startet: der Pfad des Programms (oder *Keiner*),
**Auswählen…**, um es zu wählen, und **Entfernen**, um es zu entfernen. Unter
macOS kann ein Anwendungspaket (`.app`) gewählt werden. Lässt sich das Programm
nicht starten, sagt es ein Hinweis. Die Zeile wird unter Android nicht
angezeigt, ebenso wenig in den Flatpak- und Snap-Builds. Siehe [Wenn sich eine
Datei ändert](library.md#when-a-file-changes).

## Automatisierung {#automation}

Die Rotationen und Regeln des [Musikfüllers](automation.md) (nicht unter
Android).

![Einstellungen, Automatisierung: die Rotationen Music (der Standard) und Hits, die gewählte Music mit ihrem einen Schritt über die gesamte Musik, und die Regeln des Musikfüllers](../../images/guide/settings-automation.png)

- **Rotationen:** die Liste, der Standard ist markiert. **Neue Rotation** fügt
  eine hinzu, die die gesamte Musik spielt, **Zum Standard machen** macht die
  gewählte zum Standard, **Rotation löschen** entfernt sie (die letzte kann
  nicht gelöscht werden).
- **Die gewählte Rotation:** ihr **Name**, dann eine Zeile pro **Schritt**:
  woher seine Tracks kommen (eine Kategorie mit ihren Unterkategorien oder
  *Alle* einer Art), wie viele er jedes Mal spielt (**Mindestens** 0–99,
  **Höchstens** 1–99; bei Mindestens 0 setzt der Schritt manche Durchgänge
  aus), **Nach oben**, **Nach unten** und **Entfernen**. **Schritt
  hinzufügen** fügt am Ende einen hinzu. Ein Schritt, dessen Kategorie gelöscht
  wurde, sagt es; der Füller überspringt ihn.
- **Musikfüller:** wie lange es dauert, bis derselbe Track wieder spielen darf
  (3 Stunden oder sobald die Hälfte der anderen Tracks seines Schritts gelaufen
  ist, was zuerst eintritt; 0 schaltet jeweils eines ab), derselbe Interpret
  (30 Minuten), der **Vorlauf** (30 Minuten Musik in der Warteschlange) und die
  **behaltenen gespielten Titel** in seiner Liste (50).

**Standardwerte wiederherstellen** setzt die Regeln zurück; die Rotationen
bleiben.

## Tastenkürzel {#keyboard-shortcuts}

![Einstellungen, Tastenkürzel: jede Player-Aktion mit ihrer Taste und Entfernen neben den gebundenen](../../images/guide/settings-shortcuts.png)

Siehe [Tastatur](keyboard.md).

## MIDI {#midi}

![Einstellungen, MIDI, bei ausgeschalteter MIDI-Steuerung](../../images/guide/settings-midi.png)

MIDI-Controller einschalten, die Eingangsports sehen (**Ports neu verbinden**
öffnet sie erneut, als würden die Controller aus- und wieder eingesteckt) und für
jede Player-Aktion ein Bedienelement lernen. Siehe [MIDI-Controller](midi.md).

## Fernsteuerung {#remote}

![Einstellungen, Fernsteuerung, mit der HTTP-API, die auf diesem Computer lauscht](../../images/guide/settings-remote.png)

Fernsteuerung über das Netzwerk, für Webseiten, Smartphone-Apps, Automatisierung
und Bedienoberflächen. Siehe [Fernsteuerung](remote-control.md).

- **Fernsteuerung über HTTP erlauben**, ihre **Adresse** und ihr **Port** sowie
  eine Zeile, die sagt, ob sie lauscht.
- **Token**, über diesen Computer hinaus erforderlich. **Erzeugen** macht ein
  zufälliges, **Anzeigen** zeigt es, und **Kopieren** legt es in die
  Zwischenablage. Eine Warnung erscheint, wenn die Adresse über diesen Computer
  hinausreicht und es kein Token gibt.
- **Webseiten, die die API nutzen dürfen**: ein Origin pro Zeile.
- **Steuerung über OSC erlauben**, ihre **Adresse** und ihr **Port** sowie die
  **erlaubten Absender** (Adressen oder Subnetze, eine pro Zeile).
- **Zeiten senden alle**: wie oft verstrichene und verbleibende Zeiten gesendet
  werden, solange etwas spielt.

Textfelder und Zahlen gelten, wenn du sie verlässt, was das Öffnen eines anderen
Abschnitts oder das Schließen der Einstellungen einschließt; Esc bricht ab, was
du gerade getippt hast. Ein ungültiger Wert wird korrigiert, und das Feld zeigt,
was beibehalten wurde.
