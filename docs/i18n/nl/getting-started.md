# Aan de slag

## Installeren {#install}

Download een pakket voor je systeem van de pagina **Releases** van het
project. Bij elk bestand hoort een `.sha256`-bestand. Om een download te
controleren voer je op Linux `sha256sum -c <bestand>.sha256` uit, en op macOS
`shasum -a 256 -c <bestand>.sha256`.

### Linux {#linux}

| Pakket | Installeren | Bijwerken | Verwijderen |
|---|---|---|---|
| `.deb` (Debian, Ubuntu) | `sudo apt install ./fauste-player_<version>_amd64.deb` | installeer de nieuwere `.deb` op dezelfde manier | `sudo apt remove fauste-player` |
| `.rpm` (Fedora, openSUSE) | `sudo dnf install ./fauste-player-<version>-1.x86_64.rpm` | installeer de nieuwere `.rpm` | `sudo dnf remove fauste-player` |
| AppImage (elke distributie) | `chmod +x fauste-player-<version>-x86_64.AppImage`, en voer het dan uit | vervang het bestand | verwijder het bestand |
| Flatpak-bundel | `flatpak install --user fauste-player-<version>-x86_64.flatpak` | installeer de nieuwere bundel | `flatpak uninstall org.fauste.FaustePlayer` |
| Archief | pak het uit en voer `fauste-player` uit | pak het nieuwere uit | verwijder de map |

- **Integratie met het bureaublad:** de pakketten voegen **Fauste Player** toe
  aan het programmamenu en laten je afspeellijsten van het type `.m3u`,
  `.m3u8` en `.pls` ermee openen, waarna ze als nieuwe playlists worden
  geïmporteerd.
- **Benodigde bibliotheken:** de ALSA- en D-Bus-bibliotheken. Elk bureaublad
  heeft ze, en de pakketten vermelden ze als afhankelijkheid. JACK wordt
  gebruikt als het geïnstalleerd is, maar is nooit vereist.
- **AppImage:** start het niet doordat FUSE ontbreekt, voer het dan uit met
  `--appimage-extract-and-run`.
- **Flatpak:**
  - De bundel heeft de freedesktop-runtime van Flathub nodig. Is de
    Flathub-remote niet ingesteld, voer dan eerst
    `flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo`
    uit.
  - De sandbox kan je homemap lezen (om je muziek af te spelen waar die staat).
  - Het speelt via PulseAudio, of rechtstreeks via ALSA voor bit-perfect
    apparaten.
- **Bureaubladbibliotheken:** het venster gebruikt libxkbcommon en EGL of
  OpenGL, met Wayland of X11. Elk bureaublad heeft ze, en de `.deb` en `.rpm`
  vermelden ze als afhankelijkheid. Installeer ze op een erg minimaal systeem
  voordat je de AppImage gebruikt.

### Windows {#windows}

Voer `fauste-player-<version>-x86_64-pc-windows-msvc.msi` uit. Het installeert
voor alle gebruikers onder *Program Files* en voegt een item toe aan het
Startmenu.
- **Bijwerken:** voer het nieuwere installatieprogramma uit. Het vervangt de
  geïnstalleerde versie.
- **Verwijderen:** gebruik **Instellingen → Apps**.
- **Niet-ondertekend installatieprogramma:** is de release niet ondertekend,
  dan waarschuwt SmartScreen voor een onbekende uitgever. Kies **Meer info →
  Toch uitvoeren**.

Het `.zip`-archief is een portable alternatief: pak het uit en voer
`fauste-player.exe` uit.

### macOS {#macos}

Open `fauste-player-<version>-macos-universal.dmg` en sleep **Fauste Player**
naar **Programma’s**. Dezelfde app draait op Apple silicon en Intel (macOS 11
of nieuwer).
- **Bijwerken:** vervang de app op dezelfde manier.
- **Verwijderen:** verplaats de app naar de prullenmand.
- **Niet-ondertekende app:** is de release niet ondertekend en genotariseerd,
  dan weigert macOS de eerste start.
  - macOS 15 en nieuwer: open **Systeeminstellingen → Privacy en beveiliging**,
    scrol naar het bericht over Fauste Player, kies **Open toch** en bevestig.
  - macOS 14 en ouder: klik met de rechtermuisknop op de app, kies **Open** en
    bevestig.
  - Of voer `xattr -dr com.apple.quarantine "/Applications/Fauste Player.app"`
    uit.
- **JACK op macOS:** een ondertekende en genotariseerde app kan alleen een
  JACK-bibliotheek laden die zelf ondertekend is. Anders staat JACK als niet
  beschikbaar; gebruik Core Audio.
- **Playlists:** op macOS worden ze geïmporteerd via Instellingen →
  Playlists. Een afspeellijstbestand vanuit de Finder met de app openen wordt
  niet ondersteund.

### Eén instantie tegelijk {#one-instance-at-a-time}

Per gegevensmap draait er maar één Fauste Player. Open je een playlist vanuit
de bestandsbeheerder terwijl het programma draait, dan wordt die playlist in
de draaiende applicatie geïmporteerd. Start je het opnieuw zonder playlist,
dan verschijnt een bericht dat het al draait. Wil je aparte instanties naast
elkaar draaien (bijvoorbeeld twee studio’s op één computer), geef dan elk een
eigen map met `FAUSTE_HOME`.

### Opdrachtregel {#command-line}

`fauste-player --version` toont de versie. De titelbalk van het venster toont
de naam en de versie, en de knop **Over Fauste Player** (een info-pictogram,
links van **Instellingen**) opent het venster Over Fauste Player, met het
copyright en de licentievermeldingen (**Licenties van derden** opent het
bestand met vermeldingen dat met de releasepakketten wordt geïnstalleerd). In
een taal die met AI is vertaald, zegt dat venster ook dat de vertaling fouten
kan bevatten. Sluit het met **Sluiten** of `Esc`. `fauste-player --help`
toont de opties. Afspeellijstbestanden die als argument worden meegegeven,
worden als nieuwe playlists geïmporteerd.

![Het venster Over Fauste Player: versie, copyright en de licenties van de meegeleverde onderdelen](../../images/guide/about.png)

Op Windows opent het releaseprogramma geen consolevenster: `--version`,
`--help` en opstartfouten verschijnen in plaats daarvan in een berichtvenster.

Heeft een instelling een herstart nodig, dan verschijnt in de bovenbalk een
label **Herstart nodig**: druk erop om te herstarten (zie
[Instellingen](settings.md#restart-pending)).

### Sluiten terwijl er audio on air is {#closing-while-audio-is-on-air}

Het venster sluiten terwijl er iets on air is, sluit het programma niet af.
Het venster komt naar voren (ook als het geminimaliseerd was) en een dialoog
**Er is audio on air** toont wat er klinkt: spelers die spelen of gepauzeerd
zijn (`P1 — titel`) en spelende carts met hun nummer op de pagina
(`Cart 3 — titel`). Een CUE van een speler of van de cartwall telt niet mee.
Kies **Annuleren** (of `Esc`, of klik buiten de dialoog) om door te blijven
spelen, of **Stoppen en sluiten** om elke speler die on air is en alle carts
te stoppen en dan af te sluiten. De sessie wordt opgeslagen zoals bij elke
andere manier van afsluiten. Is er niets on air, dan sluit het venster meteen.
De dialoog heeft voorrang op Instellingen en Over Fauste Player, en
sneltoetsen doen niets zolang hij open is (MIDI- en afstandsopdrachten werken
nog wel).

## Eerste start {#first-start}

Het venster opent met vier spelers, elk met een lege playlist. Er speelt niets
totdat je op Play drukt: dat geldt ook na een herstart of een crash.

## Muziek toevoegen {#add-music}

- Klik onderaan een speler op **+ Toevoegen** en kies bestanden, of
- sleep audiobestanden of mappen uit je bestandsbeheerder naar een
  nummerlijst. Bij mappen worden de audiobestanden direct in de map
  toegevoegd, niet die in de submappen.

Ondersteunde formaten: WAV, AIFF, CAF, FLAC, MP3 (en MP1/MP2), AAC/M4A, ALAC, Ogg Vorbis, Opus, WavPack, Monkey's Audio (APE), DSD (DSF en DFF) en Matroska-audio (MKA).
DSD-bestanden worden omgezet naar PCM afgespeeld, met hun frequentie gedeeld
door 32 (88,2 kHz voor DSD64), of ongewijzigd als DoP of native DSD op een
bit-perfect apparaat dat daarvoor is ingesteld (zie
[Bit-perfect uitgang](bit-perfect.md#dsd)). Opus speelt altijd op 48 kHz.
WavPack-bestanden moeten lossless en mono of stereo zijn; hybride en
meerkanaals WavPack worden als onleesbaar getoond, net als DFF-bestanden met
DST-compressie.

Elk bestand wordt op de achtergrond geanalyseerd. De analyse leest de titel,
artiest, het album en de hoes, tekent de golfvorm en bepaalt waar het geluid
begint en eindigt en waar in het volgende nummer gemixt moet worden. Je kunt
een nummer afspelen voordat de analyse klaar is.

## Afspelen {#play}

- Druk op **Play** (of op de cijfertoets van de speler, `1` voor P1) om het
  **volgende** nummer te starten, groen gemarkeerd in de lijst.
- Druk nogmaals op **Play** terwijl een nummer on air is om naar het volgende
  over te faden.
- **Dubbelklik** op een nummer om er het volgende nummer van te maken.

In de modus **CONT** (doorlopend) mixt de speler op het MIX-punt zelf naar het
volgende nummer. In de modus **SINGLE** stopt hij aan het einde van elk
nummer. Ga verder met [Spelers](players.md).
