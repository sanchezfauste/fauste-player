# Fehlerbehebung

## Kein Ton {#no-sound}

1. Öffne **Einstellungen → Audioausgänge** und drücke für den Player auf **Main
   testen**. Hörst du den Ton, prüfe den Lautstärke-Fader des Players.
2. Hörst du nichts, wähle ein anderes Gerät oder Kanalpaar. Eine Änderung an den
   Ausgängen gilt, sobald nichts darauf spielt; **Jetzt anwenden** in den
   Einstellungen wendet sie sofort an.
3. Bevorzuge unter Linux in Einstellungen → Audioausgänge → Audiosystem
   **PipeWire** oder **PulseAudio**. Sie teilen die Soundkarte mit anderen
   Programmen. **ALSA** spricht die Karte direkt an und findet sie womöglich
   belegt.
4. **JACK** wird als nicht verfügbar angezeigt („no output device“), wenn kein
   JACK-Server läuft. Starte den Server (zum Beispiel mit QjackCtl) und starte
   die Anwendung neu. Stelle den JACK-Server auf die Abtastrate in den
   Einstellungen ein (standardmäßig 48 kHz): JACK läuft für alle Programme mit
   einer Rate.
5. **PipeWire** wird von den herunterladbaren Archiven nicht angeboten; sie
   erreichen PipeWire über dessen PulseAudio-Dienst, der auf dieselbe Weise
   funktioniert. Es ist in Builds verfügbar, die mit dem Feature `pipewire`
   erstellt wurden.

## Bit-perfect {#bit-perfect}

- **Das BP-Symbol bleibt aus.** Prüfe jede Bedingung unter
  [Bit-perfect-Ausgabe](bit-perfect.md#the-bp-badge): keine Blende, nichts anderes
  auf denselben Ausgängen, eine verlustfreie Datei, die analysiert wurde, und ein
  Gerät, das mit der Rate der Datei läuft.
- **Eine kurze Stille vor einem Track.** Das Bit-perfect-Gerät wurde mit der
  Abtastrate des Tracks neu geöffnet. Das geschieht nur, wenn sich die Rate ändert:
  Tracks mit derselben Rate folgen ohne Lücke aufeinander. Halte die Bibliothek
  auf einer Rate, um das zu vermeiden. Siehe [Was es
  kostet](bit-perfect.md#what-it-costs).
- **Tracks mischen nicht, und Play blendet nicht aus.** Bei einem Player, dessen
  Main-Ausgang bit-perfect ist, spielt jeder Track bis zu seinem Ende, und dann
  startet der nächste, und Play / Weiter und Vorheriger Track wechseln sofort: Auf
  einem solchen Player gibt es keine Crossfades oder Segues. Schalte Bit-perfect
  für das Gerät aus oder verlege den Player auf einen anderen Ausgang, um wieder
  zu mischen. Siehe [Was es kostet](bit-perfect.md#what-it-costs).
- **Der Lautstärke-Fader bewegt sich nicht.** Der Main-Ausgang des Players ist
  bit-perfect, was die Lautstärke bei 100 % festhält; der Tooltip des Faders sagt
  es. Auch MIDI-Fader und die Remote-API können ihn nicht bewegen. Siehe [Was es
  kostet](bit-perfect.md#what-it-costs).
- **Ein Track wird resampelt abgespielt, und das Log sagt, das Gerät sei belegt
  (Linux).** Um die Rate zu ändern, schließt die Anwendung das Gerät und öffnet
  es erneut. In diesem Moment kann der Soundserver (PipeWire) die Karte
  übernehmen. Die Anwendung versucht es einige Male erneut; ist die Karte noch
  belegt, spielt der Track mit der aktuellen Rate des Geräts, und der nächste
  Track fragt wieder nach seiner Rate. Damit die Anwendung die Karte für sich
  allein hat, öffne die Soundeinstellungen des Systems und setze das Profil dieser
  Karte auf **Aus** (oder **Pro Audio**), damit der Soundserver ihr `hw:`-Gerät
  in Ruhe lässt. Die Anzahl der Versuche und die Wartezeit dazwischen sind
  `tuning.device_busy_retries` und `tuning.device_busy_retry_ms` in der
  Konfigurationsdatei.
- **Das Gerät spielt, aber das BP-Symbol bleibt aus (Windows oder macOS).** Der
  exklusive Zugriff wurde verweigert, und das Gerät spielt geteilt.
  - Windows: Ein anderes Programm hält das Gerät womöglich exklusiv, oder die
    alleinige Kontrolle ist in den erweiterten Eigenschaften des Geräts
    ausgeschaltet.
  - macOS: Ein anderes Programm hält das Gerät womöglich im Hog-Modus, oder das
    Gerät bietet seine Raten nur als kontinuierlichen Bereich an (die meisten
    Interfaces listen feste Raten).
- **Ein `hw:`-Gerät lässt sich nicht öffnen (Linux).**
  - Womöglich hält ein Soundserver die Karte. Stoppe ihn oder stelle den Server
    so ein, dass er diese Karte in Ruhe lässt, und starte die Anwendung neu.
  - Einige USB-DACs akzeptieren nur gepackte 24-Bit-Samples (`S24_3LE`), die die
    Audiobibliothek nicht unterstützt. Verwende diese Karte stattdessen über
    `plughw:` (nicht bit-perfect).

### DSD {#dsd}

- **Ein DSD-Track wird umgewandelt abgespielt, obwohl das Gerät auf DoP oder
  natives DSD eingestellt ist.** Das Log sagt, warum („DSD converted to PCM“ und
  der Grund), bei diesen Ursachen: Die Lautstärke des Players liegt nicht bei
  100 %, etwas anderes spielt auf dem Gerät, mehr als zwei Kanäle, oder ein Gerät,
  das die Rate verweigert (DoP erfordert die DSD-Rate geteilt durch 16, zum
  Beispiel 176,4 kHz für DSD64) oder kein 24- oder 32-Bit-Format hat. Ein Track,
  der noch nicht analysiert wurde, wird stillschweigend umgewandelt, ohne
  Log-Zeile: Analysiere ihn (Einstellungen → Analyse) und spiele ihn erneut ab.
- **Nur der erste Track eines DSD-Albums wird als DSD ausgegeben.** Das ist die
  Standard-Mischeinstellung: Die Tracks, die der Player von selbst startet,
  spielen umgewandelt. Wähle in Einstellungen → Audioausgänge **DSD behalten und
  andere Quellen stummschalten**, um sie als DSD zu behalten. Siehe
  [DSD](bit-perfect.md#dsd).
- **Die Kopfzeile zeigt DSD, aber der Wandler spielt Rauschen oder rastet nicht
  ein.** Der Wandler erkennt DoP (oder das native Format) nicht. Stelle das Gerät
  zurück auf **In PCM umwandeln**.
- **Ein Knacken, wenn ein DSD-Track startet, stoppt oder DSD verlässt.** Der
  Wandler braucht mehr DSD-Stille: Erhöhe **DSD-Stille** (standardmäßig 200 ms)
  in Einstellungen → Audioausgänge, Erweitert.
- **Andere Player oder Carts sind auf dem Gerät stumm.** Ein DSD-Track spielt mit
  **DSD behalten und andere Quellen stummschalten**; das Symbol **Andere stumm**
  wird angezeigt. Sie klingen wieder, wenn der Track endet.

## Meldung „Ausgang verloren“ {#output-lost-alert}

Die Statusleiste zeigt **Ausgang verloren: &lt;Gerät&gt;**, wenn ein Gerät nicht
mehr reagiert. Die Player zählen und mixen weiter auf einer internen Uhr, sodass
die Automatisierung nicht stehen bleibt. Das Gerät wird alle 2 Sekunden erneut
versucht und übernimmt wieder, sobald es zurückkehrt. Stecke das Kabel wieder an
oder schalte das Interface wieder ein.

### „Ausgang verloren“ verschwindet nie, bei einem direkten `hw:`-Ausgang {#output-lost-that-never-clears-with-a-direct-hw-output}

Eine Soundkarte, die über einen direkten ALSA-`hw:`-Ausgang verwendet wird (zum
Beispiel ein Bit-perfect-Ausgang), wird von Fauste Player allein gehalten: Der
Soundserver (PipeWire oder PulseAudio) kann sie nicht gleichzeitig verwenden.
Geht ein anderer Ausgang über das Standardgerät des Soundservers und ist dieses
Standardgerät dieselbe Karte, startet dieser Ausgang nie und bleibt bei **Ausgang
verloren**. Das Log sagt einmal „output device opened but never started“.

Verwende einen Weg pro Karte: Leite jeden Ausgang dieser Karte über dasselbe
`hw:`-Gerät (bei Bedarf mit verschiedenen Kanälen), oder wähle in den
Soundeinstellungen deines Systems eine andere Karte als Standardausgang des
Soundservers.

## Ein Track zeigt ein Warnsymbol oder eine Datei mit Kreuz {#a-track-shows-a-warning-icon-or-a-file-with-a-cross}

Die Datei fehlt (verschoben, gelöscht, ausgehängt: eine Datei mit Kreuz) oder
lässt sich nicht dekodieren (ein Warnzeichen). Die Player überspringen sie. Fahre
mit der Maus über die Zeile, um zu sehen, welcher Fall vorliegt, und den Pfad der
Datei.

Eine fehlende Datei wird alle 30 Sekunden erneut gesucht
(`tuning.missing_recheck_ms` in der Konfigurationsdatei): Wird das Laufwerk
eingehängt oder die Datei wieder abgelegt, wird der Track von selbst spielbar.
Eine Datei, die sich nicht dekodieren lässt, wird mit demselben Timer von selbst
erneut geprüft, anhand von Größe und Änderungszeit: Sie wird nicht erneut
dekodiert, es sei denn, eines von beiden hat sich geändert, zum Beispiel wenn
ein Kopiervorgang endet. Um sie sofort zu prüfen, nutze **Neu analysieren** im
Zeilenmenü oder **Einstellungen → Analyse → Alle Tracks neu analysieren** für die
ganze Bibliothek.

Eine WAV- oder FLAC-Datei, die als nicht lesbar erscheint oder als lautes Rauschen
spielen würde, kann ein DTS-CD-Rip sein: Die Datei enthält statt Audio einen
DTS-Bitstream. Die Anwendung lehnt sie ab (das Log sagt „encoded bitstream stored
as PCM“) und dekodiert kein DTS. Konvertiere sie zuerst mit einem anderen Werkzeug
in gewöhnliches PCM.

## Ein überwachter Ordner zeigt eine Warnung {#a-watched-folder-shows-a-warning}

Eine Kategorie, die einen Ordner überwacht, zeigt ⟳, in einer anderen Farbe, wenn
der Ordner nicht verfolgt wird; fahre mit der Maus darüber für den Grund. Siehe
[Überwachte Ordner](library.md#watched-folders).

- **Nicht gefunden oder nicht lesbar**: Das Laufwerk oder die Freigabe ist nicht
  eingehängt oder nicht erreichbar. Ein leerer Ordner, der Tracks der Bibliothek
  enthalten sollte, gilt ebenfalls als nicht gefunden, da eine nicht eingehängte
  Freigabe oft wie einer aussieht. Nichts wird markiert oder entfernt; der Ordner
  wird alle 30 Sekunden erneut gesucht und durchsucht, wenn er erscheint. Hast du
  den Ordner mit Absicht geleert, räume seine Tracks auf, indem du sie in der
  Bibliothekstabelle auswählst und **Aus der Bibliothek entfernen…** wählst, oder
  indem du **Neu analysieren** für sie wählst, was die Tracks, deren Datei fehlt,
  als fehlend markiert, sodass **Fehlende Titel entfernen…** sie entfernen kann.
- **Änderungen werden nicht verfolgt**: Das System wollte ihn nicht überwachen,
  meist ein Ordner mit sehr vielen Unterordnern. Der Ordner wird weiterhin bei
  jedem Start und bei jedem erneuten Versuch durchsucht, sodass neue Dateien dann
  erscheinen, nicht sofort. Unter Linux braucht ein Ordnerbaum, der größer als die
  Grenze der Überwachungen pro Benutzer ist, ein höheres
  `fs.inotify.max_user_watches` (zum Beispiel `sudo sysctl
  fs.inotify.max_user_watches=524288`; mache es dauerhaft in `/etc/sysctl.d/`).
- **Kein vollständiger Pfad**: Der gespeicherte Ordner wurde von Hand in einen
  Pfad geändert, der nicht vollständig ist. Wähle **Überwachten Ordner ändern…**,
  um ihn erneut zu wählen.
- **Eine Netzwerkfreigabe zeigt keine Warnung, aber Dateien, die von einem anderen
  Computer hinzugefügt wurden, erscheinen nicht**: Auf einer Netzwerkfreigabe
  (SMB/CIFS, NFS) akzeptiert das System die Überwachung meist, meldet aber nur die
  Änderungen, die von diesem Computer aus gemacht wurden. Anderswo gemachte
  Änderungen werden beim nächsten Start gesehen, nicht, solange die Anwendung
  läuft.
- **Ein Ordner mit tausenden Dateien** braucht beim Start eine Weile zum
  Durchsuchen; der Analysefortschritt zeigt es.
- Tracks, deren Datei einen überwachten Ordner verlassen hat, bleiben in der
  Bibliothek als fehlend markiert. Entferne sie mit **Fehlende Titel entfernen…**
  im Menü der Kategorie.

## Audioaussetzer {#audio-dropouts}

Die Statusleiste warnt 5 Sekunden lang nach jedem Aussetzer, den die Anwendung
erkennt: **P1: Audioaussetzer (3)**, wenn die Dekodierung eines Players nicht mit
der Festplatte mitgekommen ist (die Zahl gilt für den Track, der gerade spielt),
und **&lt;Gerät&gt;: Aussetzer des Audiogeräts (2)**, wenn das Ausgabegerät eine
Frist verpasst hat (ein Xrun). Auch das Log hält jeden fest, höchstens eine Zeile
alle 10 Sekunden pro Art, mit der Anzahl der Fälle. Nicht jedes Audiosystem
meldet Xruns (PulseAudio nicht; der exklusive Modus von Windows nicht).


- Erhöhe die **Puffergröße** in den Einstellungen (sie gilt, sobald nichts auf
  diesem Gerät spielt, oder sofort mit **Jetzt anwenden**).
- Über einen Soundserver (PulseAudio, das ALSA-Gerät `default`) verlangt jeder
  Ausgang ohnehin mindestens 50 ms, egal welche Puffergröße gilt, damit ein
  ausgelasteter Computer nicht zu hören ist; die Einstellungen zeigen es neben der
  Latenz. Erhöhe auf einem alten oder stark ausgelasteten Computer
  `shared_buffer_ms` im Abschnitt `outputs` der Konfigurationsdatei (zum Beispiel
  auf 100). Das System hält etwa zwei dieser Blöcke in der Warteschlange, sodass
  Cart- und Play-Buttons dann etwa das Doppelte der Erhöhung später klingen. Für
  den Sendeausgang vermeidet der eigene Hardware-Eintrag des Geräts (`hw:` unter
  Linux) den Soundserver ganz.
- Erlaube unter Linux Echtzeit-Scheduling. Die Anwendung fragt das System über
  rtkit (D-Bus) danach, und das Log hält eine Ablehnung fest. Viele Desktops lehnen
  es für Anwendungen ab.
- Vermeide Netzlaufwerke für Musik, die on Air gespielt wird.

## Die Oberfläche ist auf einem alten Computer langsam {#the-interface-is-slow-on-an-old-computer}

Solange etwas spielt, zeichnet die Oberfläche höchstens 30 Bilder pro Sekunde
(`max_fps` im Abschnitt `ui` der Konfigurationsdatei), und Klicks und Tasten
werden sofort beantwortet. Senke den Wert auf einem alten oder ausgelasteten
Computer (zum Beispiel auf 20), um dem Audio mehr vom Prozessor zu lassen; 0
zeichnet jedes Bild, das die Anzeige zeigt. Die Pegelmesser messen bei jeder Rate
dasselbe; sie werden nur seltener gezeichnet.

## Dateien lassen sich unter Linux nicht ablegen {#files-cannot-be-dropped-on-linux}

Das Fenster nimmt aus einem Dateimanager gezogene Dateien nur an, wenn es unter
X11 läuft. Standardmäßig tut es das, auch auf einem Wayland-Desktop (über
XWayland, das jeder Desktop hat). Ist `ui.window_system` in `config.json` auf
`"Wayland"` gesetzt, ist das Fenster ein natives Wayland-Fenster, das bei
gebrochener Bildschirmskalierung schärfer aussehen kann, auf dem sich aber keine
Dateien ablegen lassen: Nutze stattdessen **+ Hinzufügen**. Setze es zurück auf
`"Auto"` (der Standard) oder `"X11"` und starte Fauste Player neu.

## Ein MIDI-Controller reagiert nicht {#a-midi-controller-does-not-respond}

Zeigt **Einstellungen → MIDI** den Controller als verbunden, aber seine Tasten und
Fader tun nichts, drücke dort **Ports neu verbinden**: Das Programm schließt die
MIDI-Ports und öffnet sie erneut, als würde der Controller abgezogen und wieder
eingesteckt. Hilft das nicht, ziehe den Controller ab und stecke ihn wieder ein.

In beiden Fällen sagt das Log, was geschehen ist. Es listet die MIDI-Ports auf,
sobald sie sich ändern, jeden geöffneten und geschlossenen Port (mit dem Grund),
einen Port, der sich nicht öffnen lässt (einmal, bis er sich öffnet oder
verschwindet), die erste empfangene Nachricht jeder Verbindung, die erste
ignorierte Nachricht jeder Verbindung mit dem Grund (keine Bindung, ein Fader, der
die Lautstärke noch nicht erreicht hat, eine Aktion, die jetzt nicht wirken kann)
und die erste LED-Nachricht, die fehlgeschlagen ist. Ein verbundener Controller,
der nie eine erste Nachricht protokolliert, sendet nichts, was das Programm
empfängt. Starte das Programm für jede ignorierte Nachricht und jeden
fehlgeschlagenen erneuten Versuch mit `RUST_LOG=info,fp_control=debug`.

## „In der Oberfläche ist ein Fehler aufgetreten“ {#the-interface-hit-an-error}

Ein Zeichenfehler wurde abgefangen. Das Audio ist nicht betroffen. Drücke auf
**Oberfläche neu starten**. Bitte melde ihn zusammen mit den Logs.

## Logs und Absturzberichte {#logs-and-crash-reports}

Den Log-Ordner findest du unter [Daten und Backups](data-and-backups.md). Es gibt
pro Tag eine Log-Datei. Identische Zeilen, die direkt hintereinander wiederholt
werden, werden einmal geschrieben, gefolgt von einer Zeile wie
`previous message repeated 12 times` (bei einer langen Reihe von Wiederholungen
wird diese Zeile jede Minute geschrieben). Zeilen, die sich unterscheiden, zum
Beispiel weil sie einen Zähler tragen, werden nicht zusammengefasst. Die
routinemäßigen Verbindungsmeldungen des PulseAudio-Clients bleiben standardmäßig
weg. Setze `RUST_LOG=debug` in der Umgebung für mehr Details (es ersetzt den
Standardfilter `info,pulseaudio=warn`).

Das Log ist begrenzt: Die Datei eines Tages wächst nicht über 10 MiB hinaus (ihre
letzte Zeile sagt es; der Rest dieses Tages wird nicht protokolliert, was bei einer
Flut sich unterscheidender Zeilen passiert), und die letzten 14 Dateien werden
aufbewahrt. Ändere sie in der Konfigurationsdatei mit `limits.max_log_file_mb` (1
bis 1024), `limits.max_log_files` (2 bis 365) und `limits.log_repeat_summary_secs`
(1 bis 3600, Standard 60, wie oft eine lange Reihe von Wiederholungen gemeldet
wird); sie gelten ab dem nächsten Start. Absturzberichte werden als
`crash-<time>.txt` im selben Ordner gespeichert und getrennt begrenzt
(`limits.max_crash_reports` pro Lauf). Hänge beim Melden eines Fehlers beide
Dateien an.
