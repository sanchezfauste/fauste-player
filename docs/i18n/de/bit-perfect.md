# Bit-perfect-Ausgabe

Ein **Bit-perfect**-Gerät erhält die Samples jeder Datei genau so, wie sie in der
Datei stehen: dieselbe Abtastrate, dieselben Werte, ohne Resampling,
Lautstärkeänderung oder Mischen. Das ist nützlich für Abhörketten und digitale
Verbindungen, bei denen jede Verarbeitung auf dem Computer vermieden werden
soll.

## Ein Gerät bit-perfect machen {#setting-a-device-bit-perfect}

1. Wähle unter **Einstellungen → Audioausgänge** das Gerät ausdrücklich für den
   Main-Ausgang eines Players (oder den der Cartwall). Ein Player, der auf dem
   Systemstandard bleibt, lässt sich nicht bit-perfect machen. Ein Gerät, das
   kein Ausgang mehr verwendet, verliert seinen Bit-perfect-Schalter und seinen
   DSD-Modus beim nächsten Start der Anwendung.
2. Wähle oben im Abschnitt **Erweitert**. Schalte unter **Einstellungen pro
   Gerät** neben dem Gerät **Bit-perfect** ein.
3. Starte die Anwendung neu.

Das Gerät startet dann mit seiner eigenen Abtastrate, wenn du ihm an derselben
Stelle eine gegeben hast, sonst mit der globalen Abtastrate, und folgt von dort
aus jeder Datei.

Der Schalter ist deaktiviert, wenn das Gerät keinen exklusiven Zugriff geben
kann.
- **Linux:** Wähle ein ALSA-Gerät, dessen Name mit `hw:` beginnt. Es ist die
  Soundkarte selbst. PulseAudio, PipeWire, JACK und die ALSA-Geräte `default`
  oder `plughw:` mischen oder wandeln um, sind also nie bit-perfect.
- **Windows:** Wähle das Gerät im System **WASAPI**. Es wird im exklusiven Modus
  geöffnet.
  - In den Windows-Soundeinstellungen muss in den **Erweitert**-Eigenschaften
    des Geräts *Anwendungen haben alleinige Kontrolle über dieses Gerät*
    eingeschaltet sein (standardmäßig ist es das).
  - Solange es spielt, kann kein anderes Programm das Gerät verwenden.
- **macOS:** Wähle das Gerät unter **Core Audio**. Es wird im Hog-Modus
  geöffnet.
  - Die Abtastrate des Geräts wird auf die des Tracks gesetzt und sein Format
    auf das breiteste Integer-Format, das es bei dieser Rate anbietet (die
    Einstellungen, die das Audio-MIDI-Setup zeigt).
  - Sie werden zurückgegeben, wenn die Anwendung das Gerät nicht mehr verwendet.
  - Zwei Geräte mit exakt demselben Namen lassen sich nicht bit-perfect machen.

## Was auf einem Bit-perfect-Gerät geschieht {#what-happens-on-a-bit-perfect-device}

- **Exklusiver Zugriff.** Nichts anderes auf dem Computer kann auf dem Gerät
  spielen, solange die Anwendung es verwendet. Wird der exklusive Zugriff
  verweigert, spielt das Gerät dennoch, geteilt, und das BP-Symbol bleibt aus.
- **Die Rate folgt der Datei.** Spielt nichts auf dem Gerät und startet ein Track
  mit einer anderen Abtastrate, wird das Gerät mit dieser Rate neu geöffnet.
  - Das geschieht, wenn du einen Track abspielst, einen pausiert geladenen
    fortsetzt, vorhörst oder ein Cart startest. Tracks, die nur warten (der
    nächste Track jedes Players), werden mit der neuen Rate neu vorbereitet.
  - Das erneute Öffnen dauert so lange, wie das Gerät zum Starten braucht
    (meist einige zehn Millisekunden). Der Start verzögert sich um diese Zeit.
  - Solange etwas auf dem Gerät spielt, ändert sich die Rate nie. Ein Track mit
    einer anderen Rate, der dann startet (zum Beispiel ein 48-kHz-Track, in den
    nach einem 44,1-kHz-Track gemixt wird, oder ein Track, der startet, während
    ein anderer Player oder ein Cart auf demselben Gerät spielt), wird über seine
    ganze Länge umgewandelt und ist nicht bit-perfect.
  - Verweigert das Gerät eine Rate, behält es die vorherige, und der Track wird
    umgewandelt.
- **Keine Verarbeitung, wenn nichts sie verlangt.** Die Samples laufen
  unverändert durch, solange alles Folgende zutrifft:
  - die Lautstärke des Players beträgt 100 %;
  - es läuft keine Blende;
  - nichts anderes spielt auf denselben Ausgängen (ein anderer Player, ein Cart,
    ein Testton).

## DSD {#dsd}

Eine DSD-Datei wird normalerweise wie jede andere Datei in PCM umgewandelt
abgespielt. Ein Bit-perfect-Gerät kann stattdessen den DSD-Stream unverändert
erhalten.

**Die drei Modi.** In der Ansicht Erweitert unter Einstellungen → Audioausgänge
hat jedes Gerät, das ein Ausgang verwendet, unter seinem Bit-perfect-Schalter
eine **DSD**-Auswahl:
- **In PCM umwandeln** (Standard): DSD wird umgewandelt, wie auf jedem anderen
  Gerät.
- **DoP** (DSD over PCM): Die DSD-Bits reisen innerhalb von 24-Bit-PCM-Samples,
  die die meisten DSD-fähigen Wandler erkennen. Es funktioniert auf jedem System.
- **Natives DSD** (nur Linux): rohes DSD, für ALSA-`hw:`-Geräte, deren Treiber
  ein DSD-Sampleformat meldet.

Es werden nur die Modi angeboten, die das Gerät annehmen kann, und eine Zeile
unter der Auswahl sagt, warum die anderen nicht: Das Gerät ist nicht verbunden,
Bit-perfect ist aus, das Gerät lässt sich nicht exklusiv öffnen, natives DSD
erfordert Linux, oder das Gerät nimmt kein natives DSD an. Ein für ein Gerät
gespeicherter Modus, den es jetzt nicht annehmen kann, wird als PCM angezeigt,
was auch das ist, was spielt; der gespeicherte Modus kehrt zurück, wenn das Gerät
ihn wieder annehmen kann. Das Ändern eines Modus, der Mischeinstellung oder der
DSD-Stille erfordert einen Neustart, wie die anderen Ausgangseinstellungen.

**Wann DSD unverändert ausgegeben wird.** Alles Folgende muss beim Start des
Tracks zutreffen:
- das Gerät ist bit-perfect, mit exklusivem Zugriff, und sein Modus ist DoP oder
  natives DSD;
- der Track ist DSD (DSF oder DFF), mono oder stereo, und wurde analysiert (so
  ist seine DSD-Rate bekannt);
- die Lautstärke des Players beträgt 100 %;
- nichts anderes spielt auf dem Gerät (ein anderer Player, ein Cart, ein
  Testton);
- das Gerät akzeptiert den Stream. DoP erfordert eine Geräterate von der DSD-Rate
  geteilt durch 16 (176,4 kHz für DSD64, 352,8 kHz für DSD128, 705,6 kHz für
  DSD256) und ein 24- oder 32-Bit-Format. Natives DSD erfordert ein Gerät, das
  das DSD-Format bei dieser Rate annimmt.

Wenn natives DSD endet, geht das Gerät zu PCM mit der Rate zurück, die es vor dem
DSD-Track hatte, weil viele Wandler natives DSD mit Raten annehmen, die sie als
PCM nicht abspielen können (kein Wandler spielt PCM mit der Rate, mit der DSD512
läuft). Ein Track, der als PCM weitergeht, behält die Rate des DSD-Streams, wenn
das Gerät sie als PCM annimmt, und läuft sonst mit der früheren Rate von der
Stelle aus weiter, an der er war, wie alles andere, was auf diesem Gerät spielt;
eine dort laufende Blende endet sofort. Das Gerät bleibt nie auf einer Rate, die
es verweigert: Öffnet sich keine Rate (zum Beispiel weil das Gerät in diesem
Moment abgezogen wurde), ist der Ausgang verloren, bis der automatische neue
Versuch ihn mit der früheren Rate wieder öffnet.

Andernfalls wird der Track in PCM umgewandelt, und das Log sagt, warum (zum
Beispiel „something else plays on the device“ oder „the device refused 705600
Hz“). Vorhören und Carts werden immer umgewandelt.

Solange DSD unverändert ausgegeben wird:
- steht im Symbol der Kopfzeile **DSD** statt **BP**;
- zeigen die Pegelmesser den Pegel der PCM-Umwandlung desselben Tracks, sodass
  sie wie gewohnt funktionieren;
- muss die Lautstärke bei 100 % bleiben: Der Tooltip des Faders sagt es. Ihn zu
  bewegen schaltet den Track auf PCM um (siehe unten);
- stoppen Stopp und Ausblenden den Track sofort, ohne Blende, da sich ein
  DSD-Stream nicht ausblenden lässt. Wird Play auf einem anderen Track gedrückt,
  während er spielt, schneidet es ihn auf dieselbe Weise ab, statt überzublenden;
- wirken auch Pause und Fortsetzen sofort, ohne Rampe.

**Stille an den Rändern.** Jeder Start, jedes Ende und jeder Wechsel zu PCM
sendet zuerst DSD-Stille (standardmäßig 200 ms), damit der Wandler ohne Knacken
einrastet. Die Ausnahme ist ein DSD-Track, der einen Stream derselben Art und
DSD-Rate fortsetzt, dessen Stille noch läuft: Der Wandler ist noch eingerastet,
also startet er ohne zusätzliche Stille. Ein Track startet daher um diese Zeit
später, und ein Wechsel zu PCM hinterlässt eine Lücke dieser Länge. Sie heißt
**DSD-Stille** unter Einstellungen → Audioausgänge, Erweitert (0 bis 2000 ms).

**Wenn eine andere Quelle das Gerät braucht.** **Wenn eine andere Quelle einen
DSD-Ausgang braucht** unter Einstellungen → Audioausgänge wählt, was geschieht,
wenn ein anderer Player, ein Cart oder ein Testton auf demselben Gerät startet
(den Fader des Players selbst zu bewegen ist die Ausnahme: Das schaltet den Track
immer auf PCM um):
- **DSD-Track als PCM fortsetzen** (Standard). Der Stream wechselt nach der
  DSD-Stille zu PCM, und der Track läuft umgewandelt von der Stelle aus weiter, an
  der er war. Dasselbe geschieht mit dem Track, der von selbst folgt (siehe
  unten).
- **DSD behalten und andere Quellen stummschalten.** Nichts unterbricht den
  DSD-Stream. Andere Quellen, die auf das Gerät geroutet sind, sind stumm, bis
  der DSD-Track endet, und der Player zeigt währenddessen ein Symbol **Andere
  stumm**. Der nächste Track des Players überlappt nicht: Er startet, wenn der
  DSD-Track endet, ohne Crossfade oder Segue. Ein PCM-Track wartet auf die
  DSD-Stille; ein DSD-Track derselben Art und DSD-Rate setzt den Stream ohne sie
  fort. Das Bewegen des Faders schaltet den Track weiterhin auf PCM um.

**Ein Album bleibt unter der Standardeinstellung nicht DSD.** Mit *DSD-Track als
PCM fortsetzen* wird nur ein DSD-Track, der auf einem ruhenden Gerät startet, als
DSD ausgegeben. Die Tracks, die der Player danach von selbst startet (am Ende
eines Tracks, bei einem Segue oder einem Crossfade), starten aus einem Preload,
der immer PCM ist, sodass das Gerät auf PCM wechselt und sie umgewandelt spielen.
Ein Track, den du selbst startest (Play, Doppelklick), wird wieder als DSD
ausgegeben, wenn das Gerät ruht oder der vorherige DSD-Stream mit derselben
DSD-Rate noch in seiner Stille ist. Um ein ganzes DSD-Album als DSD zu behalten,
wähle *DSD behalten und andere Quellen stummschalten*. Dann wird jeder Track des
Players als DSD ausgegeben, und der nächste startet, wenn der vorherige endet.

Geht das Gerät verloren, während DSD spielt, und kommt es zurück, ohne es tragen
zu können (zum Beispiel ohne exklusiven Zugriff), läuft der Track als PCM weiter.

## Das BP-Symbol {#the-bp-badge}

Das Symbol **BP** in der Kopfzeile des Players leuchtet, solange der aktuelle
Track sein Main-Gerät unverändert erreicht. Alles Folgende muss zutreffen:

- das Gerät ist bit-perfect und mit exklusivem Zugriff geöffnet;
- das Gerät läuft mit der Abtastrate des Tracks;
- der Track ist verlustfreies Integer-PCM (WAV, AIFF, FLAC, ALAC, WavPack oder
  Monkey's Audio), mono oder stereo, und höchstens 24 Bit, und das Format des
  Geräts fasst seine Samplegröße (eine 24-Bit-Datei auf einem 16-Bit-Gerät ist
  nicht bit-perfect). DSD wird umgewandelt und leuchtet daher nie BP; wird es
  unverändert ausgegeben (siehe [DSD](#dsd)), steht stattdessen **DSD** im
  Symbol;
- der Track wurde analysiert, da so seine Rate und Samplegröße bekannt sind.
  Tracks, die eine frühere Version analysiert hat, erhalten ihr Format, sobald
  sie erneut analysiert wurden (die Meldung beim Start oder Einstellungen →
  Analyse) oder sobald ein Player sie anzeigt oder ein Cart sie enthält;
- die Lautstärke beträgt 100 %, es läuft keine Blende, und nichts anderes spielt
  auf denselben Ausgängen.

Einige Dateien werden nie als bit-perfect angezeigt:
- **Verlustbehaftete Dateien** (MP3, AAC, Ogg Vorbis, Opus): Ihre dekodierten
  Samples sind nicht die Integer-Werte, die ein Gerät annimmt.
- **Dateien über 24 Bit:** Der Mixer arbeitet mit 32-Bit-Gleitkomma, das 24 Bit
  exakt trägt.
- **Dateien mit mehr als zwei Kanälen:** Sie werden auf Stereo heruntergemischt.

## Selbst prüfen {#checking-it-yourself}

So prüfst du eine Signalkette von Ende zu Ende:

1. Verbinde den digitalen Ausgang des Geräts (S/PDIF, AES oder USB-Loopback) mit
   einem Recorder, der bitgenau aufnimmt.
2. Spiele eine verlustfreie Testdatei bei 100 % ab, ohne dass etwas anderes
   spielt.
3. Nimm sie auf.
4. Vergleiche die Aufnahme mit der Datei. Zum Beispiel mit SoX: eine invertieren
   und beide mischen: `sox -m -v 1 file.wav -v -1 recording.wav diff.wav`,
   nachdem ihre Anfänge aufeinander ausgerichtet wurden. Jedes Sample der
   Differenz muss null sein.

Die automatisierten Tests des Projekts prüfen dieselbe Eigenschaft innerhalb der
Anwendung, auf einem simulierten Gerät.

### DSD auf einem echten Wandler {#dsd-on-a-real-converter}

Die automatisierten Tests prüfen DSD nur auf simulierten Geräten. DoP und
natives DSD wurden vom Projekt nicht auf einem echten Wandler ausprobiert. So
prüfst du einen:
1. Stelle das Gerät auf **DoP** (oder **Natives DSD** unter Linux), starte neu
   und spiele eine DSD-Datei bei 100 % ab, ohne dass etwas anderes spielt. Die
   Kopfzeile muss **DSD** zeigen, und die eigene Anzeige des Wandlers sollte die
   DSD-Rate (zum Beispiel DSD64) statt einer PCM-Rate zeigen. Ein Wandler, der
   eine PCM-Rate zeigt oder Rauschen spielt, erkennt den Stream nicht: Gehe
   zurück zu **In PCM umwandeln**.
2. Achte auf ein Knacken oder einen Rauschstoß beim Start, bei Stopp, am Ende
   des Tracks und beim Bewegen des Faders. Ein Knacken bedeutet, dass der Wandler
   eine längere **DSD-Stille** braucht (Einstellungen → Audioausgänge,
   Erweitert).
3. Starte ein Cart oder einen anderen Player auf demselben Gerät, einmal mit
   jeder Mischeinstellung, und prüfe das oben beschriebene Verhalten.
4. Um unter Linux natives DSD ohne die Anwendung zu prüfen, führe
   `FAUSTE_NATIVE_DSD_DEVICE=alsa:hw:CARD=<card>,DEV=0 cargo test -p fp-backends --test conformance -- --ignored native_dsd_on_a_real_device`
   aus. Es öffnet das Gerät in nativem DSD bei DSD64 und spielt eine Sekunde
   DSD-Stille. Es muss bestehen, und der Wandler sollte auf DSD64 einrasten.
5. Mit DSD-Dateien in `test-music/` spielt
   `cargo test --release -p fp-engine --test dsd_real_music -- --ignored` sie
   durch die Engine auf einem simulierten Gerät ab und vergleicht die Wörter mit
   den Bytes der Datei (siehe [Testing](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/testing.md)).
