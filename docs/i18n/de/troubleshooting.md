# Fehlerbehebung

## Kein Ton {#no-sound}

1. Öffne **Einstellungen → Audioausgänge** und drücke für den Player auf **Main
   testen**. Hörst du den Ton, prüfe den Lautstärke-Fader des Players.
2. Hörst du nichts, wähle ein anderes Gerät oder Kanalpaar. Änderungen an den
   Ausgängen werden nach einem Neustart wirksam: Drücke in den Einstellungen auf
   **Jetzt neu starten**.
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
  [Bit-perfect-Ausgabe](bit-perfect.md#the-bp-badge): Lautstärke bei 100 %, keine
  Blende, nichts anderes auf denselben Ausgängen, eine verlustfreie Datei, die
  analysiert wurde, und ein Gerät, das mit der Rate der Datei läuft.
- **Eine kurze Stille vor einem Track.** Das Bit-perfect-Gerät wurde mit der
  Abtastrate des Tracks neu geöffnet. Halte die Bibliothek auf einer Rate, um das
  zu vermeiden.
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

## Audioaussetzer {#audio-dropouts}

Die Statusleiste warnt 5 Sekunden lang nach jedem Aussetzer, den die Anwendung
erkennt: **P1: Audioaussetzer (3)**, wenn die Dekodierung eines Players nicht mit
der Festplatte mitgekommen ist (die Zahl gilt für den Track, der gerade spielt),
und **&lt;Gerät&gt;: Aussetzer des Audiogeräts (2)**, wenn das Ausgabegerät eine
Frist verpasst hat (ein Xrun). Auch das Log hält jeden fest, höchstens eine Zeile
alle 10 Sekunden pro Art, mit der Anzahl der Fälle. Nicht jedes Audiosystem
meldet Xruns (PulseAudio nicht; der exklusive Modus von Windows nicht).


- Erhöhe die **Puffergröße** in den Einstellungen (und drücke **Jetzt neu
  starten**).
- Erlaube unter Linux Echtzeit-Scheduling. Die Anwendung fragt das System über
  rtkit (D-Bus) danach. Auch die Mitgliedschaft in der Gruppe `audio` mit einem
  `rtprio`-Limit funktioniert.
- Vermeide Netzlaufwerke für Musik, die on Air gespielt wird.

## „In der Oberfläche ist ein Fehler aufgetreten“ {#the-interface-hit-an-error}

Ein Zeichenfehler wurde abgefangen. Das Audio ist nicht betroffen. Drücke auf
**Oberfläche neu starten**. Bitte melde ihn zusammen mit den Logs.

## Logs und Absturzberichte {#logs-and-crash-reports}

Den Log-Ordner findest du unter [Daten und Backups](data-and-backups.md). Es gibt
pro Tag eine Log-Datei, und die letzten 14 werden aufbewahrt. Absturzberichte
werden als `crash-<time>.txt` gespeichert. Setze `RUST_LOG=debug` in der Umgebung
für mehr Details. Hänge beim Melden eines Fehlers beide Dateien an.
