# Erste Schritte

## Installation {#install}

Unter Windows ist Fauste Player im **Microsoft Store**
(<https://apps.microsoft.com/detail/9NH4KCL73463>), und unter Linux im
**Snap Store** (<https://snapcraft.io/fauste-player>). Oder lade auf der
**Releases**-Seite des Projekts ein Paket für dein System herunter. Neben jeder Datei liegt eine `.sha256`-Datei. Um einen Download zu
prüfen, führe unter Linux `sha256sum -c <file>.sha256` aus, unter macOS
`shasum -a 256 -c <file>.sha256`.

### Linux {#linux}

| Paket | Installation | Aktualisierung | Entfernen |
|---|---|---|---|
| Snap (Snap Store) | `sudo snap install fauste-player` | automatisch (`sudo snap refresh fauste-player`, um sofort zu aktualisieren) | `sudo snap remove fauste-player` |
| `.deb` (Debian, Ubuntu) | `sudo apt install ./fauste-player_<version>_amd64.deb` | die neuere `.deb` auf dieselbe Weise installieren | `sudo apt remove fauste-player` |
| `.rpm` (Fedora, openSUSE) | `sudo dnf install ./fauste-player-<version>-1.x86_64.rpm` | die neuere `.rpm` installieren | `sudo dnf remove fauste-player` |
| AppImage (jede Distribution) | `chmod +x fauste-player-<version>-x86_64.AppImage`, dann ausführen | die Datei ersetzen | die Datei löschen |
| Flatpak-Bundle | `flatpak install --user fauste-player-<version>-x86_64.flatpak` | das neuere Bundle installieren | `flatpak uninstall com.sanchezfauste.FaustePlayer` |
| Archiv | entpacken und `fauste-player` ausführen | das neuere entpacken | den Ordner löschen |

- **Desktop-Integration:** Die Pakete tragen **Fauste Player** ins
  Anwendungsmenü ein und erlauben es, `.m3u`-, `.m3u8`- und `.pls`-Playlists
  damit zu öffnen; sie werden dann als neue Playlists importiert.
- **Benötigte Bibliotheken:** die ALSA- und D-Bus-Bibliotheken. Jeder Desktop
  hat sie, und die Pakete deklarieren sie. JACK wird verwendet, wenn es
  installiert ist, und ist nie erforderlich.
- **AppImage:** Startet es nicht, weil FUSE fehlt, führe es mit
  `--appimage-extract-and-run` aus.
- **Snap:** Das Snap läuft in einer Sandbox. Von Haus aus liest es deinen
  Home-Ordner (außer versteckten Dateien und Ordnern), spielt über PulseAudio
  oder PipeWire und darf das Netzwerk für die Fernsteuerungs-API und OSC nutzen
  (beide aus, bis du sie einschaltest). Zwei Funktionen brauchen eine
  Berechtigung, die du einmal in einem Terminal erteilst:
  - Bit-perfect-Ausgabe auf ALSA-`hw:`-Geräten und MIDI-Controller:
    `sudo snap connect fauste-player:alsa`.
  - Musik auf USB-Laufwerken und anderen Datenträgern (`/media`, `/run/media`,
    `/mnt`): `sudo snap connect fauste-player:removable-media`.

  `snap connections fauste-player` zeigt, was verbunden ist. Starte Fauste
  Player nach dem Verbinden neu. Musik an anderen Orten (außerhalb deines
  Home-Ordners und dieser drei) kann das Snap nicht lesen: Nutze dafür ein
  anderes Paket. JACK ist im Snap nicht verfügbar. Seine Einstellungen und
  Playlists liegen in `~/snap/fauste-player/` (siehe [Daten und
  Backups](data-and-backups.md)).
- **Flatpak:**
  - Das Bundle benötigt die freedesktop-Runtime von Flathub. Ist das
    Flathub-Remote nicht eingerichtet, führe zuerst
    `flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo`
    aus.
  - Die Sandbox kann deinen Home-Ordner lesen (um deine Musik dort
    abzuspielen, wo sie liegt).
  - Die Wiedergabe läuft über PulseAudio, bei Bit-perfect-Geräten direkt über
    ALSA.
  - Es darf das Netzwerk nutzen, für die Fernsteuerungs-API und OSC (beide aus,
    bis du sie einschaltest).
  - Bundles bis 0.10 wurden als `org.fauste.FaustePlayer` installiert, und die
    neueren installieren sich als `com.sanchezfauste.FaustePlayer`, eine eigene
    Anwendung. Um deine Einstellungen, Playlists und Carts zu behalten, schließe
    die App und führe
    `mv ~/.var/app/org.fauste.FaustePlayer ~/.var/app/com.sanchezfauste.FaustePlayer`
    aus, bevor du die neue startest, danach
    `flatpak uninstall org.fauste.FaustePlayer`.
- **Desktop-Bibliotheken:** Das Fenster nutzt libxkbcommon und EGL oder
  OpenGL, mit Wayland oder X11. Jeder Desktop hat sie, und die `.deb` und
  `.rpm` deklarieren sie. Installiere sie auf einem sehr minimalen System,
  bevor du das AppImage verwendest.

### Windows {#windows}

Am einfachsten geht es über den **Microsoft Store**: Suche nach *Fauste
Player* oder öffne <https://apps.microsoft.com/detail/9NH4KCL73463> und wähle
**Abrufen**. Der Store hält die App aktuell, und **Einstellungen → Apps**
entfernt sie.

Oder führe `fauste-player-<version>-x86_64-pc-windows-msvc.msi` aus. Es
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
Player“ mit dem Autor, dem Copyright, der Lizenz (GNU GPL Version 3 oder später; **Lizenz
online lesen** öffnet ihren Text), einem kurzen Haftungsausschluss und den
Lizenzhinweisen (**Drittanbieterlizenzen** öffnet die mit den Release-Paketen
installierte Hinweisdatei). Schließe es mit **Schließen** oder `Esc`.
`fauste-player --help` listet die Optionen auf. Als Argumente übergebene
Playlist-Dateien werden als neue Playlists importiert.

![Das Fenster „Über Fauste Player“: Version, Copyright und die Lizenzen der enthaltenen Komponenten](../../images/guide/about.png)

Unter Windows öffnet das Release-Programm kein Konsolenfenster: `--version`,
`--help` und Startfehler erscheinen stattdessen in einem Meldungsfenster.

Wartet eine Änderung an den Ausgängen auf etwas, das on Air ist, erscheint in
der oberen Leiste eine Pille **Einstellungen ausstehend**; sie sagt, was wartet
(siehe [Einstellungen](settings.md#settings-pending)).

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
DFF-Dateien. Eine WAV- oder FLAC-Datei, die statt Audio einen DTS-Bitstream
(oder anderen kodierten Surround-Bitstream) enthält, wie bei manchen
DTS-CD-Rips, erscheint ebenfalls als nicht lesbar (das Log nennt den Grund):
Konvertiere sie zuerst in gewöhnliches PCM.

Surround-Dateien (Quad, 5.1, 6.1, 7.1 und andere Anordnungen mit bis zu acht
Kanälen) werden nach der Position jedes Kanals auf Stereo heruntergemischt:
Front links und rechts bleiben auf ihrer Seite, die Mitte und die hintere Mitte
gehen auf beide Seiten, die Seiten- und Rückkanäle gehen auf ihre Seite, und der
LFE-Kanal (Subwoofer) entfällt. Mehrkanalige Opus- und AAC-Dateien können nicht
gelesen werden und erscheinen als nicht lesbar.

Jede Datei wird im Hintergrund analysiert. Die Analyse liest Titel,
Interpret, Album und Cover, zeichnet die Wellenform und findet heraus, wo der
Ton beginnt und endet und wo in den nächsten Track gemixt werden soll. Du
kannst einen Track abspielen, bevor seine Analyse fertig ist. Während Dateien
analysiert werden, sagt der rechte Teil der Statusleiste, wie viele
(**Analysiere 12 Titel**); fahre mit der Maus darüber, um die Dateien zu sehen,
die gerade analysiert werden, wie viele warten und, während **Alle Tracks neu
analysieren**, wie weit es ist. Die Farblegende steht rechts davon; ist das
Fenster schmal, werden zuerst Legendeneinträge ausgeblendet, dann dieser
Eintrag, damit eine Warnung links in der Statusleiste lesbar bleibt. Laufen nur
Lautheitsmessungen (siehe [Lautheit](loudness.md)), steht dort **Messe die
Lautheit von N Titeln**.

## Abspielen {#play}

- Drücke **Play** (oder die Zifferntaste des Players, `1` für P1), um den
  **nächsten** Track zu starten, in der Liste grün markiert.
- Drücke **Play** erneut, während ein Track on Air ist, um in den nächsten
  Track überzublenden.
- Per **Doppelklick** auf einen Track machst du ihn zum nächsten.

Im Modus **CONT** (durchgehend) mixt der Player am MIX-Punkt von selbst in den
nächsten Track. Im Modus **SINGLE** stoppt er am Ende jedes Tracks. Weiter
mit [Player](players.md).
