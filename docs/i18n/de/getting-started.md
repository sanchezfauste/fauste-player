# Erste Schritte

## Installation {#install}

Lade auf der **Releases**-Seite des Projekts ein Paket für dein System
herunter. Neben jeder Datei liegt eine `.sha256`-Datei. Um einen Download zu
prüfen, führe unter Linux `sha256sum -c <file>.sha256` aus, unter macOS
`shasum -a 256 -c <file>.sha256`.

### Linux {#linux}

| Paket | Installation | Aktualisierung | Entfernen |
|---|---|---|---|
| `.deb` (Debian, Ubuntu) | `sudo apt install ./fauste-player_<version>_amd64.deb` | die neuere `.deb` auf dieselbe Weise installieren | `sudo apt remove fauste-player` |
| `.rpm` (Fedora, openSUSE) | `sudo dnf install ./fauste-player-<version>-1.x86_64.rpm` | die neuere `.rpm` installieren | `sudo dnf remove fauste-player` |
| AppImage (jede Distribution) | `chmod +x fauste-player-<version>-x86_64.AppImage`, dann ausführen | die Datei ersetzen | die Datei löschen |
| Flatpak-Bundle | `flatpak install --user fauste-player-<version>-x86_64.flatpak` | das neuere Bundle installieren | `flatpak uninstall org.fauste.FaustePlayer` |
| Archiv | entpacken und `fauste-player` ausführen | das neuere entpacken | den Ordner löschen |

- **Desktop-Integration:** Die Pakete tragen **Fauste Player** ins
  Anwendungsmenü ein und erlauben es, `.m3u`-, `.m3u8`- und `.pls`-Playlists
  damit zu öffnen; sie werden dann als neue Playlists importiert.
- **Benötigte Bibliotheken:** die ALSA- und D-Bus-Bibliotheken. Jeder Desktop
  hat sie, und die Pakete deklarieren sie. JACK wird verwendet, wenn es
  installiert ist, und ist nie erforderlich.
- **AppImage:** Startet es nicht, weil FUSE fehlt, führe es mit
  `--appimage-extract-and-run` aus.
- **Flatpak:**
  - Das Bundle benötigt die freedesktop-Runtime von Flathub. Ist das
    Flathub-Remote nicht eingerichtet, führe zuerst
    `flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo`
    aus.
  - Die Sandbox kann deinen Home-Ordner lesen (um deine Musik dort
    abzuspielen, wo sie liegt).
  - Die Wiedergabe läuft über PulseAudio, bei Bit-perfect-Geräten direkt über
    ALSA.
- **Desktop-Bibliotheken:** Das Fenster nutzt libxkbcommon und EGL oder
  OpenGL, mit Wayland oder X11. Jeder Desktop hat sie, und die `.deb` und
  `.rpm` deklarieren sie. Installiere sie auf einem sehr minimalen System,
  bevor du das AppImage verwendest.

### Windows {#windows}

Führe `fauste-player-<version>-x86_64-pc-windows-msvc.msi` aus. Es
installiert für alle Benutzer unter *Programme* (Program Files) und legt
einen Eintrag im Startmenü an.
- **Aktualisieren:** Führe das neuere Installationsprogramm aus. Es ersetzt
  die installierte Version.
- **Entfernen:** Verwende **Einstellungen → Apps**.
- **Nicht signiertes Installationsprogramm:** Ist das Release nicht signiert,
  warnt SmartScreen vor einem unbekannten Herausgeber. Wähle **Weitere
  Informationen → Trotzdem ausführen**.

Das `.zip`-Archiv ist eine portable Alternative: entpacken und
`fauste-player.exe` ausführen.

### macOS {#macos}

Öffne `fauste-player-<version>-macos-universal.dmg` und ziehe **Fauste
Player** in **Programme**. Dieselbe App läuft auf Apple Silicon und Intel
(macOS 11 oder neuer).
- **Aktualisieren:** Ersetze die App auf dieselbe Weise.
- **Entfernen:** Lege sie in den Papierkorb.
- **Nicht signierte App:** Ist das Release nicht signiert und notarisiert,
  verweigert macOS den ersten Start.
  - macOS 15 und neuer: Öffne **Systemeinstellungen → Datenschutz &
    Sicherheit**, scrolle zur Meldung über Fauste Player, wähle **Dennoch
    öffnen** und bestätige.
  - macOS 14 und älter: Klicke mit der rechten Maustaste auf die App, wähle
    **Öffnen** und bestätige.
  - Oder führe `xattr -dr com.apple.quarantine "/Applications/Fauste Player.app"` aus.
- **JACK unter macOS:** Eine signierte und notarisierte App kann nur eine
  JACK-Bibliothek laden, die selbst signiert ist. Andernfalls wird JACK als
  nicht verfügbar angezeigt; verwende Core Audio.
- **Playlists:** Unter macOS werden sie über Einstellungen → Playlists
  importiert. Eine Playlist-Datei im Finder mit der App zu öffnen, wird nicht
  unterstützt.

### Jeweils nur eine Instanz {#one-instance-at-a-time}

Pro Datenordner läuft nur ein Fauste Player. Öffnest du eine Playlist im
Dateimanager, während er läuft, wird sie in die laufende Anwendung
importiert. Startest du ihn erneut ohne Playlist, erscheint eine Meldung,
dass er bereits läuft. Um getrennte Instanzen nebeneinander zu betreiben (zum
Beispiel zwei Studios auf einem Computer), gib jeder mit `FAUSTE_HOME` einen
eigenen Ordner.

### Kommandozeile {#command-line}

`fauste-player --version` gibt die Version aus. Die Titelleiste des Fensters
zeigt den Namen und die Version, und der Button **Über Fauste Player** (ein
Info-Symbol, links von **Einstellungen**) öffnet das Fenster „Über Fauste
Player“ mit dem Copyright und den Lizenzhinweisen (**Drittanbieterlizenzen**
öffnet die mit den Release-Paketen installierte Hinweisdatei). In einer mit
KI übersetzten Sprache weist das Fenster außerdem darauf hin, dass die
Übersetzung Fehler enthalten kann. Schließe es mit **Schließen** oder `Esc`.
`fauste-player --help` listet die Optionen auf. Als Argumente übergebene
Playlist-Dateien werden als neue Playlists importiert.

![Das Fenster „Über Fauste Player“: Version, Copyright und die Lizenzen der enthaltenen Komponenten](../../images/guide/about.png)

Unter Windows öffnet das Release-Programm kein Konsolenfenster: `--version`,
`--help` und Startfehler erscheinen stattdessen in einem Meldungsfenster.

Erfordert eine Einstellung einen Neustart, erscheint in der oberen Leiste
eine Pille **Neustart ausstehend**: Drücke darauf, um neu zu starten (siehe
[Einstellungen](settings.md#restart-pending)).

### Schließen, während Audio on Air ist {#closing-while-audio-is-on-air}

Wird das Fenster geschlossen, während etwas on Air ist, beendet sich das
Programm nicht. Das Fenster kommt in den Vordergrund (auch wenn es
minimiert war), und ein Dialog **Audio ist on Air** listet auf, was gerade
klingt: Player, die spielen oder pausiert sind (`P1 — Titel`), und laufende
Carts mit ihrer Nummer auf der Seite (`Cart 3 — Titel`). Ein CUE eines
Players oder der Cartwall zählt nicht. Wähle **Abbrechen** (oder `Esc`, oder
klicke außerhalb des Dialogs), um weiterzuspielen, oder **Stoppen und
schließen**, um jeden Player on Air und alle Carts zu stoppen und dann zu
beenden. Die Session wird wie bei jedem anderen Beenden gespeichert. Ist
nichts on Air, schließt sich das Fenster sofort. Der Dialog hat Vorrang vor
Einstellungen und „Über Fauste Player“, und Tastenkürzel bewirken nichts,
solange er offen ist (MIDI- und Fernsteuerungsbefehle wirken weiterhin).

## Erster Start {#first-start}

Das Fenster öffnet sich mit vier Playern, die jeweils eine leere Playlist
zeigen. Nichts spielt, bis du Play drückst: Das gilt auch nach einem Neustart
oder einem Absturz.

## Musik hinzufügen {#add-music}

- Klicke unten an einem Player auf **+ Hinzufügen** und wähle Dateien aus, oder
- ziehe Audiodateien oder Ordner aus deinem Dateimanager auf eine Track-Liste.
  Ordner fügen die direkt in ihnen liegenden Audiodateien hinzu, nicht ihre
  Unterordner.

Unterstützte Formate: WAV, AIFF, CAF, FLAC, MP3 (und MP1/MP2), AAC/M4A, ALAC, Ogg Vorbis, Opus, WavPack, Monkey's Audio (APE), DSD (DSF und DFF) und Matroska-Audio (MKA).
DSD-Dateien werden in PCM umgewandelt mit ihrer Rate geteilt durch 32
abgespielt (88,2 kHz bei DSD64) oder unverändert als DoP oder natives DSD auf
einem Bit-perfect-Gerät, das dafür eingestellt ist (siehe
[Bit-perfect-Ausgabe](bit-perfect.md#dsd)). Opus spielt immer mit 48 kHz.
WavPack-Dateien müssen verlustfrei und mono oder stereo sein; hybride und
mehrkanalige WavPack-Dateien erscheinen als nicht lesbar, ebenso DST-komprimierte
DFF-Dateien.

Jede Datei wird im Hintergrund analysiert. Die Analyse liest Titel,
Interpret, Album und Cover, zeichnet die Wellenform und findet heraus, wo der
Ton beginnt und endet und wo in den nächsten Track gemixt werden soll. Du
kannst einen Track abspielen, bevor seine Analyse fertig ist.

## Abspielen {#play}

- Drücke **Play** (oder die Zifferntaste des Players, `1` für P1), um den
  **nächsten** Track zu starten, in der Liste grün markiert.
- Drücke **Play** erneut, während ein Track on Air ist, um in den nächsten
  Track überzublenden.
- Per **Doppelklick** auf einen Track machst du ihn zum nächsten.

Im Modus **CONT** (durchgehend) mixt der Player am MIX-Punkt von selbst in den
nächsten Track. Im Modus **SINGLE** stoppt er am Ende jedes Tracks. Weiter
mit [Player](players.md).
