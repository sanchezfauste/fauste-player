# Android

Fauste Player läuft auch auf Android-Telefonen und -Tablets (Android 10 oder
neuer), auf demselben Playout-Kern wie auf dem Desktop: Die Player,
Playlists, die Cartwall, Marker, die Lautheitsnormalisierung und jede
Abspielregel funktionieren auf dieselbe Weise.

## Installation {#install}

Ein Release, das die App enthält, hat auf seiner Seite **Releases**
`fauste-player-<version>-android.apk` (mit seiner `.sha256`). Öffne die Datei
auf dem Telefon und erlaube deinem Browser oder Dateimanager, Apps zu
installieren, wenn Android danach fragt. Zum Aktualisieren installierst du die
neuere APK auf dieselbe Weise: Deine Playlists und Einstellungen bleiben
erhalten. Die App ist noch nicht in Google Play.

## Erster Start {#first-start}

- **Zugriff auf Musik.** Erlaube den Zugriff auf Musik (und auf Benachrichtigungen,
  ab Android 13). Ohne ihn bleibt der Musikbrowser leer.
- **Das Playout läuft weiter.** Solange ein Player oder ein Cart spielt, bleibt
  eine Benachrichtigung bestehen, und das Audio läuft bei ausgeschaltetem
  Bildschirm oder mit der App im Hintergrund weiter. Wie auf dem Desktop geht
  nach einem Start oder einem Absturz nichts von selbst auf Sendung.
- **Zurück und Schließen.** Zurück schließt zuerst, was offen ist (den
  Musikbrowser, die Einstellungen, Über, einen geöffneten Player, eine Seite des
  Tabs Bibliothek). Ist nichts mehr zu schließen, legt Zurück die App weg,
  solange etwas klingt (ein Player, ein Cart oder ein CUE-Vorhören), und das
  Abspielen geht weiter; klingt nichts, speichert und schließt es die App. Die
  App im App-Wechsler wegzuwischen tut dasselbe: Ein Wischen stoppt nie, was
  gerade spielt, und die App schließt sich, sobald sie verstummt.
- **Ton anderer Apps.** Übernimmt eine andere App das Audio, spielt Fauste
  Player weiter und sagt es in der Statusleiste unten.

## Der Bildschirm {#the-screen}

Auf einem **Telefon** hat die obere Leiste vier Tabs, **Live**, **Cartwall**,
**Playlists** und **Bibliothek**, als Symbole dargestellt (der gewählte auch
mit seinem Namen, wenn Platz ist), dann das **Vorhängeschloss** und **⋮**
(**Mehr**: Einstellungen und **Über**). Die Uhr erscheint, wenn Platz für sie
ist. Quer gehalten, setzt der Bildschirm Playlists seine Player-Chips (P1,
P2…) in die obere Leiste, und die Statusleiste unten erscheint nur, wenn sie
etwas zu sagen hat.

<p>
<img src="../../images/android/1-live.png" width="180" alt="Live auf einem Telefon: zwei Player als Karten, eine ganz und der Anfang der nächsten, und darunter die Cartwall">
<img src="../../images/android/2-carts.png" width="180" alt="Carts auf einem Telefon: die Seite General hochkant gedreht, zwei Carts pro Zeile">
<img src="../../images/android/3-playlists.png" width="180" alt="Playlists auf einem Telefon: die Chips P1 bis P4, die Playlist-Tabs, die Track-Liste und unten die Karte des Players">
<img src="../../images/android/4-music.png" width="180" alt="Musik auf dem Gerät auf einem Telefon: der Ordner Demo mit drei angehakten Songs und die Leiste, um sie zur Playlist Morning von P1 hinzuzufügen">
<img src="../../images/android/5-library.png" width="180" alt="Der Tab Bibliothek auf einem Telefon: die Tracks der Kategorie Music, jeder mit einem Kästchen zum Anhaken und einem Kopfhörer zum Vorhören, und darunter die Vorhör-Karte mit der Wellenform des Tracks und seinen Markern">
</p>

- *Live:* jeder Player als Karte, und die Cartwall. Jede Karte hat von oben nach
  unten den Namen des Players, den Status, Single/Cont und CUE; das Cover, den
  Titel und Interpreten und den nächsten Track; die Wellenform (ein Tipp
  springt); dann die verstrichene Zeit und die Länge des Tracks, mit der
  Restzeit in großen Ziffern über dem Transport (Abspielen/Weiter, Vorheriger
  Track, Pause, Stopp, Track neu starten, Ausblenden, Nach dem aktuellen Track
  stoppen) mit Buttons in Fingergröße, und einen schlanken Pegelmesser mit dem
  Lautstärke-Fader (der Pegel des Faders steht darunter). Die Restzeit endet,
  wo die Buttons enden, kleiner bei einem langen Track. Auf den meisten
  Telefonen zeigt der Pegelmesser links seine Skala, und auf einem breiteren
  (etwa 406 dp und mehr) zeigt auch der Fader seine Pegel; auf einem schmalen
  Telefon (unter etwa 384 dp) zeigt keiner von beiden sie, und die Buttons
  werden etwas kleiner, damit sie passen. Die Buttons funktionieren, während ein
  anderer Finger den Fader hält, und ein Tipp zählt, auch wenn der Finger ein
  wenig rollt oder einen Moment liegen bleibt. Hochkant liegen die Karten oben,
  eine ganze Karte und der Anfang der nächsten (scrolle für mehr), und die
  Cartwall darunter. Quer ist jeder Player eine Zeile links (sein Name und
  Status, der Titel, die Restzeit, Abspielen/Weiter, Pause und Stopp; tippe auf
  eine andere Stelle der Zeile, um den Player in Playlists zu öffnen), und die
  Cartwall liegt rechts. Tippe in ihrem Kopf auf **CARTWALL**, um sie
  einzuklappen: Nur der Kopf bleibt unten (die Seiten-Tabs und *Alle stoppen*
  funktionieren weiter), und die Player erhalten den Platz; tippe erneut, um sie
  auszuklappen. Der Tab Cartwall zeigt immer die ganze Cartwall. Eine Cart-Seite,
  die für den Platz zu breit ist, wird in weniger Spalten gezeigt, in
  derselben Reihenfolge.
- *Carts:* die angezeigte Cart-Seite, den Bildschirm füllend. Wische seitwärts,
  um die nächste oder vorherige Seite zu zeigen. Hochkant wird eine Seite, die
  breiter als hoch ist, gedreht (8 × 2 erscheint als 2 × 8), in derselben
  Reihenfolge.
- *Playlists:* die Chips oben (P1, P2…) wählen den Player; darunter stehen seine
  Playlist-Tabs (+ legt eine neue an), die Track-Liste und unten (rechts, wenn
  das Telefon quer liegt) die Karte des Players mit einer größeren Wellenform,
  um einen Track vorzubereiten (sie wird niedriger, wenn der Platz knapp ist,
  damit die Buttons immer zu sehen sind). Langes Drücken auf eine Zeile öffnet
  ihr Menü, das auch **Nach oben**, **Nach unten** und **Zu einem anderen
  Player verschieben** hat (der Track kommt direkt nach dem nächsten Track
  dieses Players, in der Playlist, die er zeigt; der Track auf Sendung bleibt).
  Ziehe eine Zeile an ihrem ≡-Griff, um sie zu verschieben. Die Fußzeile hat
  **Hinzufügen** und, hinter **⋯**, **Gespielt zurücksetzen**. Langes Drücken
  auf die Wellenform setzt an dieser Stelle einen Marker (Cue-in, Intro-Ende,
  Outro-Anfang, MIX, Cue-out) oder entfernt einen, den du gesetzt hast; Marker
  werden nur hier bearbeitet, nicht in Live.
- *Bibliothek:* die Track-Bibliothek und die Musik des Geräts (siehe
  [Bibliothek](#library)).

Zurück von einem anderen Bildschirm führt zu Live; in Bibliothek zuerst zurück
zur Liste der Kategorien (in Musik auf dem Gerät zuerst einen Ordner hoch).

Bewegt sich auf dem Bildschirm nichts, zeichnet die App ihn einmal pro Sekunde
(für die Uhr), um den Akku zu schonen; ein Pegelmesser, der nach einer Pause
oder einem Stopp noch fällt, hält das Zeichnen aufrecht, bis er unten ist.

**Das Vorhängeschloss** schützt vor versehentlichen Berührungen während einer
Sendung. Gesperrt (das Vorhängeschloss geschlossen und leuchtend), zeigen die
Lautstärke-Fader nur den Pegel, und die Wellenformen springen nicht und
bearbeiten keine Marker; sie sehen abgeblendet aus. Ein Zusammenziehen und ein
Ziehen zoomen und verschieben eine Wellenform weiterhin, und Abspielen, der
Transport, die Carts und die Playlists funktionieren immer. Die Sperre bleibt
nach einem Neustart an und wird mit der Sitzung gespeichert, sodass das Laden
einer anderen Sitzung deren Sperre übernimmt; die Remote-API und MIDI sind nicht
gesperrt.

Ein Ziehen, das auf einem Lautstärke-Fader beginnt, bewegt ihn; eines, das
anderswo beginnt, scrollt, sodass Scrollen nie einen Pegel auf Sendung ändert.
Ein Tipp auf eine Wellenform springt dorthin, ein Zusammenziehen mit zwei Fingern
zoomt um deine Finger, ein Ziehen mit einem Finger verschiebt eine gezoomte
Ansicht, und **Gesamtansicht** zeigt wieder den ganzen Track. In einer Playlist
scrollt ein Finger; um einen Track zu verschieben, nutze seinen ≡-Griff oder das
Zeilenmenü.

Auf einem **Tablet** ist das Layout das des Desktops (siehe [Layout](layout.md)),
mit leichter zu treffenden Zielen.

Langes Drücken öffnet, was ein Rechtsklick auf dem Desktop öffnet: das
Zeilenmenü, die Kopfmenüs, das Menü eines Carts (**Bearbeiten…** wählt die Datei
eines leeren Carts). Auf einem Telefon füllt Einstellungen den Bildschirm und
listet seine Bereiche auf; tippe auf einen, um ihn zu öffnen, mit dem Namen jeder
Einstellung über ihrem Bedienelement. Der Pfeil oder Zurück führt zur Liste
zurück, und erneut Zurück (oder **Schließen**) schließt die Einstellungen. Das
Bearbeiten eines Carts öffnet seinen Bereich direkt.

Über → **Drittanbieterlizenzen** öffnet die Hinweise der Bibliotheken, mit denen
die App gebaut ist, in einem Browser oder HTML-Betrachter.

## Bibliothek {#library}

Auf einem **Telefon** öffnet der Tab **Bibliothek** bei einer Liste von
Kategorien: **Alle Titel**, deine Kategorien (jede Unterkategorie unter ihrer
übergeordneten, mit ihrer Farbe und der Zahl ihrer Tracks), **Ohne Kategorie**
und **Musik auf dem Gerät**. Die Bibliothek ist dieselbe, die das
[Bibliotheksfenster](library.md) des Desktops zeigt: Jede Datei, die zu einer
Playlist, einem Cart oder der Bibliothek hinzugefügt wurde, ist darin.
Kategorien werden unter **Einstellungen → Bibliothek** angelegt, umbenannt,
umgefärbt und gelöscht.

**Die Tracks einer Kategorie.** Tippe auf eine Kategorie, um ihre Tracks zu
öffnen (mit denen ihrer Unterkategorien). Der Pfeil in ihrem Kopf oder Zurück
führt zu den Kategorien zurück. Ein anderer Tab und zurück findet die Liste so,
wie du sie verlassen hast. Das Öffnen einer Kategorie aus den Kategorien beginnt
mit einer leeren Suche. Die Suche filtert die Liste nach Titel, Interpret, Album
und den anderen Tags, die das Bibliotheksfenster durchsucht; **Ganze
Bibliothek** durchsucht jede Kategorie. Jede Zeile zeigt den Titel (oder den
Dateinamen), den Interpreten, die Dauer und die Markierungen des Fensters für
eine fehlende, nicht lesbare oder noch nicht analysierte Datei.

Tippe auf Zeilen, um sie anzuhaken. Solange etwas angehakt ist, erscheint unten
die Leiste von Musik auf dem Gerät: wie viele, der Player und die Playlist,
zu denen sie gehen (tippe darauf, um eine andere zu wählen), **Am Ende
hinzufügen** und **Nach dem nächsten hinzufügen**, wie in Musik auf dem Gerät.
Die Tracks gehen in der Reihenfolge, in der du sie angehakt hast, und das
Hinzufügen löscht die Haken. Auch die Rückkehr zu den Kategorien löscht sie.

**Das Zeilenmenü.** Langes Drücken auf eine Zeile öffnet ihr Menü: **Hinzufügen
zu ▸** (die Playlist eines Players, am Ende), **Vorhören**, **In Kategorie
verschieben ▸**, **Neu analysieren** und **Aus der Bibliothek entfernen…**, das
wie auf dem Desktop fragt (siehe [Tracks
entfernen](library.md#removing-tracks)). Solange Zeilen angehakt sind, hakt
langes Drücken auf eine andere Zeile sie ebenfalls an, und das Menü wirkt auf
jede angehakte Zeile, in der Reihenfolge, in der du sie angehakt hast;
**Vorhören** wird dann nicht angeboten. Die Haken bleiben nach der Aktion des
Menüs.

**Vorhören.** Das 🎧 rechts in einer Zeile spielt ihren Track ab seinem Cue-in
auf dem Vorhör-Ausgang der Bibliothek; das 🎧 einer anderen Zeile wechselt zu
diesem Track. Ein schlanker Streifen über der Liste zeigt den Titel, die
Restzeit, **Pause**/**Abspielen** und **Stopp**, solange er spielt (oder
pausiert ist), und sonst einen Hinweis: Er behält immer seinen Platz, sodass
sich die Zeilen nicht bewegen, wenn ein Vorhören startet oder stoppt. Das 🎧 ist
abgeblendet, wenn die Datei fehlt oder nicht gespielt werden kann.

**Die Vorhör-Karte.** Tippe auf den Streifen (neben seinen Schaltflächen),
um die Karte unter der Liste zu öffnen, mit der Wellenform des Tracks wie auf
der Karte eines Players in Playlists; der Pfeil rechts am Streifen zeigt, ob
sie offen ist. Tippe noch einmal auf den Streifen oder drücke Zurück, um sie
zu schließen; das Vorhören geht weiter. Der Pfeil in der Kopfzeile der Liste
geht sofort zu den Kategorien zurück und lässt die Karte für die nächste
Liste offen. Auf der Karte:

- ein Tipp auf die Wellenform springt dorthin oder legt, wenn gestoppt,
  fest, wo **Vorhören** beginnt;
- langes Drücken setzt an dieser Stelle einen Marker (Cue-in, Intro-Ende,
  Outro-Anfang, MIX, Cue-out) oder entfernt einen, den du gesetzt hast.
  Marker gehören zum Track, die Änderung zeigt sich also in jeder Playlist
  und auf dem Desktop;
- ein Zusammenziehen zoomt, ein Ziehen verschiebt eine gezoomte Ansicht und
  **Gesamtansicht** zeigt wieder den ganzen Track;
- unter der Wellenform stehen die Zeiten, **Vorhören**, **Pause**,
  **Stopp** und das Meter.

Die Liste bleibt über der Karte (links, wenn das Telefon quer liegt), sodass
das 🎧 einer anderen Zeile die Karte weiterhin auf diesen Track wechselt. Das
Vorhängeschloss stoppt die Sprünge und Markeränderungen der Wellenform, wie
bei den Playern.
Die Vorschau
stoppt, wenn ihr Track die Bibliothek verlässt und wenn du den Tab Bibliothek
verlässt (für einen anderen Tab oder mit Zurück); in den Kategorien oder in
Musik auf dem Gerät geht sie weiter, aber ihr Streifen erscheint nur in der
Track-Liste.

Anders als auf dem Desktop spielt Android (ein Telefon oder ein Tablet) das
Vorhören der Bibliothek auf dem Standardausgang, wenn dafür kein Ausgang
gewählt ist: Unter **Einstellungen → Audioausgänge → Bibliothek** lautet die
leere Auswahl *Systemstandard*, und ihr **Cue testen** spielt dort. Wähle dort
einen anderen Ausgang (zum Beispiel kabelgebundene Kopfhörer), um abseits der
Sendung vorzuhören. Der Standardausgang ist meist auch der der Player und der
Cartwall, sodass das Vorhören auf Sendung zu hören sein kann: Solange auf diesem
Ausgang etwas spielt, fragt das 🎧 zuerst (**Trotzdem vorhören** oder
**Abbrechen**), und ein Vorhören, das du nicht bestätigt hast, stoppt, sobald
dort etwas startet, wie das CUE eines Players (siehe [CUE
(Vorhören)](players.md#cue-pre-listen)).

**Musik des Geräts zu einer Kategorie hinzufügen.** Das **＋** im Kopf einer
Track-Liste öffnet Musik auf dem Gerät für diese Kategorie. Ihre Leiste hat dann
nur **Zu &lt;Kategorie&gt; hinzufügen** (**Zu „Ohne Kategorie“ hinzufügen** von
*Alle Titel* oder *Ohne Kategorie* aus), das die angehakten Dateien in dieser
Kategorie zur Bibliothek hinzufügt und zur Liste zurückkehrt; eine Datei, die
schon in der Bibliothek ist, tritt ihr bei, wenn sie keine Kategorie hat, und
behält sonst die, die sie hat, wie auf dem Desktop. Musik auf dem Gerät behält
ihre Haken zwischen Besuchen, sodass auch Dateien hinzugefügt werden, die bei
einem früheren Besuch angehakt wurden: Prüfe zuerst die Zahl in der Leiste.
**Zurück zur Liste** oben oder Zurück führt zur Liste zurück, ohne etwas
hinzuzufügen. Musik auf dem Gerät, aus den Kategorien geöffnet, hat oben
stattdessen **Zurück zu den Kategorien**.

**Der Geräteordner einer Kategorie.** Eine Kategorie kann ihre Dateien aus einem
Ordner der Musik des Geräts beziehen. In ihrem Menü (langes Drücken darauf unter
**Einstellungen → Bibliothek** oder im Bibliotheksfenster auf einem Tablet)
öffnet **Geräteordner verwenden…** Musik auf dem Gerät, um den Ordner zu wählen
(**Diesen Ordner verwenden**); das Menü der Kategorie nennt ihn dann über seinen
Ordner-Einträgen. Nichts überwacht den Ordner: Hast du neue Musik dorthin
kopiert, tippe auf das **↻** im Kopf der Track-Liste der Kategorie (oder auf
**Geräteordner neu einlesen** in ihrem Menü). Jede Datei, die Musik auf dem
Gerät im Ordner und seinen Unterordnern auflistet, wird der Kategorie mit den
obigen Regeln hinzugefügt (eine Datei, die schon in einer anderen Kategorie ist,
behält sie), und ein Hinweis sagt, wie viele Dateien gelesen wurden. Dateien, die
den Ordner verlassen haben, bleiben in der Bibliothek, als fehlend markiert, und werden wieder normal, wenn sie
zurückkehren (listet Musik auf dem Gerät nichts auf, etwa ohne
Berechtigung, wird nichts markiert). **Geräteordner ändern…** und
**Geräteordner vergessen** stehen im selben Menü; ein Ordner gehört zu einer
Kategorie.

### Musik auf dem Gerät {#device-music}

**Musik auf dem Gerät** zeigt die Audiodateien des Geräts nach **Ordner** (mit
dem Pfad über der Liste; tippe auf einen Teil davon, um wieder nach oben zu
gehen), **Interpreten** oder **Alben**, und eine Suche über Titel, Interpret,
Album und Dateiname. Jede Zeile zeigt den Titel, den Interpreten und die Dauer;
tippe auf Zeilen, um sie anzuhaken. Solange etwas angehakt ist, zeigt eine Leiste
unten, wie viele, den Player und die Playlist, zu denen sie gehen (tippe darauf,
um eine andere zu wählen), **Am Ende hinzufügen** und, wenn der nächste Track
dieses Players in dieser Playlist liegt, **Nach dem nächsten hinzufügen**. Die
Tracks gehen in der Reihenfolge, in der du sie angehakt hast. Eine Gerätedatei
hat kein Vorhören.

**Hinzufügen** in einer Playlist-Fußzeile, die Datei eines Carts und
Einstellungen → Playlists → Musikordner öffnen auf einem Telefon Musik auf dem
Gerät (auf einem Tablet das Fenster **Musik auf diesem Gerät**, nach Ordner).
Hake Dateien an und tippe auf **Hinzufügen**, tippe auf eine Datei für ein Cart
oder auf **Diesen Ordner verwenden**; **Abbrechen** oder Zurück führt dorthin
zurück, wo du warst. Kopiere Musik auf das Gerät (zum Beispiel in seinen Ordner
*Music*), um sie dort zu sehen. Fauste Player liest deine Musik nur: Tags und
Cover werden nicht geschrieben, und Playlists und Cart-Seiten werden unter
Android noch nicht importiert oder exportiert.

Auf einem **Tablet** ist die Bibliothek das [Bibliotheksfenster](library.md) des
Desktops, geöffnet über die obere Leiste.

## Ausgänge {#outputs}

Einstellungen → Audioausgänge listet die Ausgänge des Telefons nach Art auf: den
Lautsprecher, kabelgebundene Kopfhörer, ein USB-Interface, einen
Bluetooth-Lautsprecher („JBL Flip (Bluetooth)“). Jeder Player, die Cartwall und
ihre CUE wählen einen, wie auf dem Desktop, und ebenso das Vorhören der
Bibliothek (siehe [Bibliothek](#library)). Ein Bluetooth-Lautsprecher, der aus
und wieder eingeschaltet wird, wird über seine Route wiedergefunden; solange er
weg ist, verhalten sich seine Player wie bei einem fehlenden Gerät auf dem
Desktop. Ob zwei Ausgänge gleichzeitig klingen können, hängt vom Telefon ab.

## Nicht unter Android {#not-on-android}

MIDI-Controller, Fernsteuerung, das Bearbeiten von Tags und Covern sowie
Playlist- und Cart-Seiten-Dateien sind vorerst Desktop-Funktionen. Eine an ein
Tablet angeschlossene Tastatur funktioniert wie auf dem Desktop (siehe
[Tastatur](keyboard.md)).
