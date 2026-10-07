# Einstellungen

Öffne **Einstellungen** in der oberen Leiste. Schließe sie mit **Schließen**
oder `Esc`. Die meisten Änderungen gelten sofort und werden automatisch
gespeichert.

Das Fenster hat unabhängig vom Abschnitt eine feste Größe (900 × 640, auf einem
kleinen Bildschirm kleiner), und der Abschnitt scrollt darin. In jedem Abschnitt
sind die Beschriftungen in einer Spalte ausgerichtet.

Die Abschnitte Player, Pegelmesser, Analyse und Tastenkürzel haben in ihrer
Kopfzeile einen Button **Standardwerte wiederherstellen**. Er fragt nach einer
Bestätigung und setzt dann nur diesen Abschnitt zurück (Player behält die Anzahl
der Player und die Sprache; Tastenkürzel hat keinen weiteren Zurücksetzen-Button).
Audioausgänge, Playlists, Cartwall, MIDI und Fernsteuerung haben keinen.

## Neustart ausstehend {#restart-pending}

Einige Änderungen werden erst wirksam, wenn die Anwendung neu startet: das
Audiosystem, die Abtastrate, die Puffergröße (auch die eines einzelnen Geräts),
die Main- und Cue-Ausgänge (Player und Cartwall), die Bit-perfect-Geräte und die
DSD-Einstellungen. Die Anzahl der Player gehört nicht dazu: Sie gilt sofort.

Eine Rate oder ein Puffer, die einem Gerät gegeben werden, zählen nur, wenn sie
ändern, womit das Gerät geöffnet wird: Einem Gerät denselben Wert wie den
globalen zu geben oder einen solchen Wert zu leeren, ist nicht ausstehend.

Die Grenzwerte und die Engine-Feinabstimmung gelten ebenfalls ab dem nächsten
Start, werden aber bei geschlossener Anwendung in der Konfigurationsdatei
bearbeitet (siehe [Daten und Backups](data-and-backups.md)), sodass sie nie als
ausstehend erscheinen.

Solange eine dieser Änderungen wartet, steht in der Fußzeile der Einstellungen
„Einige Änderungen werden erst nach einem Neustart wirksam.“ mit dem Button
**Jetzt neu starten**, und die obere Leiste zeigt eine Pille **Neustart
ausstehend**. Fahre mit der Maus über die Pille, um zu sehen, was wartet. Eine
kurze Meldung (zum Beispiel, dass eine Einstellung gespeichert wurde) kann für
einen Moment den Text der Fußzeile ersetzen; **Jetzt neu starten** bleibt. Beide
tun dasselbe:

- Ist nichts on Air, startet **Jetzt neu starten** (oder die Pille) sofort neu.
- Ist etwas on Air, erscheint das Fenster, das auflistet, was gerade klingt, mit
  **Stoppen und neu starten** oder **Abbrechen**.

Zuerst wird die Session gespeichert, und das Audio und die MIDI-Steuerung
stoppen; dann startet die Anwendung mit demselben Datenordner (`FAUSTE_HOME`)
neu, und danach geht nichts von selbst on Air. Kann die Anwendung nicht neu
starten (bei einem Flatpak auch, wenn die neue nicht rechtzeitig startet),
meldet sie es; starte sie dann über dein Anwendungsmenü.

## Audioausgänge {#audio-outputs}

Änderungen in diesem Abschnitt warten auf einen Neustart: siehe
[Neustart ausstehend](#restart-pending).

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
| Puffergröße | Frames pro Audioblock, sofern ein Gerät keine eigene hat; die resultierende Latenz steht darunter |
| Ausgänge pro Player | Für jeden Player ein **Main**-Gerät (On Air) und ein **Cue**-Gerät (Vorhören), jeweils mit einem Kanalpaar. Eine Soundkarte, die mehrere Ausgabeprofile anbietet (ALSA listet Front, Surround, direkte Hardware…), zeigt jedes als *Karte — Profil*; zwei Einträge, die dennoch gleich lauten würden, erhalten ihre Geräte-ID in Klammern. Mehrkanal-Interfaces können mehrere Player auf verschiedenen Paaren tragen. |
| Main testen / Cue testen | Spielt einen kurzen Ton (1 kHz auf Main, 440 Hz auf Cue, 1,5 s, −18 dBFS) auf dem gewählten Ausgang ab, damit du die Verkabelung prüfen kannst, bevor du auf Sendung gehst |
| Cartwall | Die Main- und Cue-Ausgänge der Cartwall. Main ist standardmäßig der Systemausgang. Ohne Cue gibt es kein Cart-Vorhören. |
| Abtastrate: *Gerät* (Erweitert) | **Global (...)** verwendet die obige Abtastrate; ein Wert gibt diesem Gerät seine eigene Rate. Es werden nur die Raten angeboten, die das Gerät meldet; eine gespeicherte Rate, die es nicht mehr meldet, bleibt mit dem Hinweis gelistet, dass es sich eventuell nicht öffnen lässt (das Gerät fällt dann auf die globale Rate zurück). Die eigenen Werte gelten nur für Geräte, die ein Ausgang benennt, nicht für den Systemstandardausgang, außer wenn einer es tut. |
| Puffergröße: *Gerät* (Erweitert) | **Global (...)** verwendet die obige Puffergröße; ein Wert gibt diesem Gerät seine eigene, mit ihrer Latenz darunter. Ein Gerät, das keine eigene Puffergröße annimmt, fällt auf die globale zurück, und auch auf die globale Rate, wenn es keine eigene Rate annimmt. |
| Bit-perfect: *Gerät* (Erweitert) | Ein Bit-perfect-Gerät wird mit exklusivem Zugriff geöffnet und folgt der Abtastrate jeder Datei, solange nichts darauf spielt. Der Schalter ist deaktiviert, wo das Gerät keinen exklusiven Zugriff geben kann. Siehe [Bit-perfect-Ausgabe](bit-perfect.md). |
| DSD: *Gerät* (Erweitert) | **In PCM umwandeln** (Standard), **DoP** oder, unter Linux, **Natives DSD**. Jedes Gerät zeigt es; angeboten werden nur die Modi, die das Gerät annehmen kann, und eine Zeile darunter sagt, warum die anderen nicht. Siehe [DSD](bit-perfect.md#dsd). |
| Wenn eine andere Quelle einen DSD-Ausgang braucht (Erweitert) | **DSD-Track als PCM fortsetzen** (Standard) oder **DSD behalten und andere Quellen stummschalten**. Siehe [DSD](bit-perfect.md#dsd). |
| DSD-Stille (Erweitert) | Stille, die vor dem Start eines DSD-Streams, nach seinem Ende und beim Wechsel zu PCM gesendet wird, damit der Wandler ohne Knacken einrastet; standardmäßig 200 ms, 0 bis 2000. |

Ein Cue-Ausgang fällt nie auf den Ausgang zurück, den Main verwendet, sodass das
Vorhören nie on Air geht. Ein Cue, der ein Gerät an einem Audiosystem nennt, das
dieser Computer nicht hat, oder denselben Ausgang (Gerät und Kanäle) wie Main,
bedeutet „kein Cue“. Ist ein Cue-Ausgang derselbe wie sein Main-Ausgang, sagt
das eine Warnung darunter. Bei einem Player ohne Cue-Ausgang oder mit seinem Cue
auf seinem Main-Ausgang ist der **CUE**-Button abgeblendet; beim Darüberfahren
steht, dass du hier einen Cue-Ausgang wählen sollst. Dasselbe gilt für **Auf CUE
vorhören** der Cartwall.

Verschwindet ein Gerät während der Wiedergabe, behalten die Player ihre
Zeitachsen, und das Gerät wird wieder geöffnet, wenn es zurückkommt (siehe
[Fehlerbehebung](troubleshooting.md)).

## Player {#players}

![Einstellungen, Player: Anzahl der Player, Standardmodus, Blendzeit, automatischer Mix, Cue-in und Cue-out, Warnung vor Track-Ende und Sprache](../../images/guide/settings-players.png)

| Einstellung | Standard | Bedeutung |
|---|---|---|
| Sprache | System | Sprache der Oberfläche |
| Anzahl der Player | 4 | Spalten im Hauptfenster (ein Player on Air kann nicht entfernt werden) |
| Standardmodus | CONT | Der Modus, mit dem die Player starten |
| Blendzeit | 1000 ms | Wird von Play während on Air und von Ausblenden verwendet |
| Automatischer Mix am MIX-Punkt | An | Tracks im CONT-Modus überlappen |
| Cue-in und Cue-out verwenden | An | Aus: Die Player spielen jeden Track vom Anfang bis zum Ende der Datei; die Marker bleiben erhalten, und Carts verwenden weiterhin ihre eigenen. Dauern und Playlist-Summen folgen demselben Bereich |
| Warnung vor Track-Ende | 10 s | Wann der Countdown rot zu blinken beginnt |

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

Die Schwellen sind in [Marker und Mixen](markers-and-mixing.md) beschrieben.
**Alle Tracks neu analysieren** führt die Analyse für die ganze Bibliothek
erneut aus; manuelle Marker bleiben erhalten.

Nach einem Update, bei dem sich die Analyse geändert hat, behalten Tracks, die
von der früheren Version analysiert wurden, ihre Marker und Wellenformen, die
weiterhin funktionieren. Beim Start sagt Fauste Player, wie viele es sind, und
bietet **Jetzt analysieren** oder **Später** an; **Veraltete Tracks analysieren
(N)** hier tut jederzeit dasselbe. Die Tracks auf den Playern werden ohnehin auf
den neuesten Stand gebracht, sobald sie angezeigt werden, ebenso die Tracks von
Carts, für die kein Format erfasst ist (ein Cart spielt nur bit-perfect, wenn
sein Format bekannt ist). Tracks, deren Datei fehlt, werden nicht gezählt, bis
die Datei wieder da ist.

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

Englisch und Spanisch sind von Hand geschrieben. Die anderen Übersetzungen wurden
mit KI erstellt und können Fehler enthalten; wird eine davon verwendet, sagt
das Fenster „Über Fauste Player“ es. Korrekturen von Muttersprachlern sind als
Issues oder Pull Requests willkommen.

## Cartwall {#cartwall}

![Einstellungen, Cartwall: die Seiten, das Raster und der Editor des gewählten Carts](../../images/guide/settings-cartwall.png)

Seiten, Rastergröße, der Cart-Editor sowie Import und Export von Cart-Seiten.
Siehe [Cartwall](cartwall.md).

## Tastenkürzel {#keyboard-shortcuts}

![Einstellungen, Tastenkürzel: jede Player-Aktion mit ihrer Taste und Entfernen neben den gebundenen](../../images/guide/settings-shortcuts.png)

Siehe [Tastatur](keyboard.md).

## MIDI {#midi}

![Einstellungen, MIDI, bei ausgeschalteter MIDI-Steuerung](../../images/guide/settings-midi.png)

MIDI-Controller einschalten, die Eingangsports sehen und für jede Player-Aktion
ein Bedienelement lernen. Siehe [MIDI-Controller](midi.md).

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
