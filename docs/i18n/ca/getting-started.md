# Primers passos

## Instal·lació {#install}

Baixa un paquet per al teu sistema des de la pàgina **Releases** del
projecte. Cada fitxer té al costat un `.sha256`. Per comprovar una
baixada, executa `sha256sum -c <file>.sha256` a Linux, o
`shasum -a 256 -c <file>.sha256` a macOS.

### Linux {#linux}

| Paquet | Instal·lar | Actualitzar | Desinstal·lar |
|---|---|---|---|
| `.deb` (Debian, Ubuntu) | `sudo apt install ./fauste-player_<version>_amd64.deb` | instal·la el `.deb` més nou de la mateixa manera | `sudo apt remove fauste-player` |
| `.rpm` (Fedora, openSUSE) | `sudo dnf install ./fauste-player-<version>-1.x86_64.rpm` | instal·la el `.rpm` més nou | `sudo dnf remove fauste-player` |
| AppImage (qualsevol distribució) | `chmod +x fauste-player-<version>-x86_64.AppImage` i executa'l | substitueix el fitxer | esborra el fitxer |
| Paquet Flatpak | `flatpak install --user fauste-player-<version>-x86_64.flatpak` | instal·la el paquet més nou | `flatpak uninstall org.fauste.FaustePlayer` |
| Arxiu comprimit | extreu-lo i executa `fauste-player` | extreu el més nou | esborra la carpeta |

- **Integració amb l'escriptori:** els paquets afegeixen **Fauste Player**
  al menú d'aplicacions i et permeten obrir-hi llistes `.m3u`, `.m3u8` i
  `.pls`, que s'importen com a llistes noves.
- **Biblioteques necessàries:** les biblioteques d'ALSA i D-Bus. Tots els
  escriptoris les tenen, i els paquets les declaren. JACK s'utilitza quan
  està instal·lat, i mai no és obligatori.
- **AppImage:** si no s'inicia perquè falta FUSE, executa'l amb
  `--appimage-extract-and-run`.
- **Flatpak:**
  - El paquet necessita el runtime de freedesktop de Flathub. Si el remot
    de Flathub no està configurat, executa abans
    `flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo`.
  - El sandbox pot llegir la teva carpeta personal (per reproduir la teva
    música allà on és).
  - Sona a través de PulseAudio, o directament a través d'ALSA per als
    dispositius bit perfect.
- **Biblioteques d'escriptori:** la finestra utilitza libxkbcommon i EGL o
  OpenGL, amb Wayland o X11. Tots els escriptoris les tenen, i el `.deb` i el
  `.rpm` les declaren. En un sistema molt mínim, instal·la-les abans
  d'utilitzar l'AppImage.

### Windows {#windows}

Executa `fauste-player-<version>-x86_64-pc-windows-msvc.msi`. S'instal·la
per a tots els usuaris a *Program Files* i afegeix una entrada al menú
Inici.
- **Actualitzar:** executa l'instal·lador més nou. Substitueix la versió
  instal·lada.
- **Desinstal·lar:** utilitza **Configuració → Aplicacions**.
- **Instal·lador sense signar:** si la release no està signada, SmartScreen
  avisa d'un editor desconegut. Tria **Més informació → Executa igualment**.

L'arxiu `.zip` és una alternativa portàtil: extreu-lo i executa
`fauste-player.exe`.

### macOS {#macos}

Obre `fauste-player-<version>-macos-universal.dmg` i arrossega **Fauste
Player** a **Aplicacions**. La mateixa aplicació funciona en Apple silicon i
Intel (macOS 11 o posterior).
- **Actualitzar:** substitueix l'aplicació de la mateixa manera.
- **Desinstal·lar:** mou-la a la paperera.
- **Aplicació sense signar:** si la release no està signada ni notaritzada,
  macOS rebutja el primer inici.
  - macOS 15 i posteriors: obre **Configuració del Sistema → Privacitat i
    seguretat**, desplaça't fins al missatge sobre Fauste Player, tria
    **Obre igualment** i confirma.
  - macOS 14 i anteriors: fes clic dret a l'aplicació, tria **Obre** i
    confirma.
  - O bé executa `xattr -dr com.apple.quarantine "/Applications/Fauste Player.app"`.
- **JACK a macOS:** una aplicació signada i notaritzada només pot carregar
  una biblioteca JACK que també estigui signada. Si no, JACK apareix com a
  no disponible; utilitza Core Audio.
- **Llistes:** a macOS s'importen des de Configuració → Llistes. No es pot
  obrir un fitxer de llista amb l'aplicació des del Finder.

### Una sola instància alhora {#one-instance-at-a-time}

Només s'executa un Fauste Player per carpeta de dades. Si obres una llista
des del gestor de fitxers mentre s'executa, aquesta llista s'importa a
l'aplicació en marxa. Si l'inicies de nou sense cap llista, mostra un
missatge que diu que ja s'està executant. Per executar instàncies separades
una al costat de l'altra (per exemple, dos estudis en un sol ordinador),
dona a cadascuna la seva pròpia carpeta amb `FAUSTE_HOME`.

### Línia d'ordres {#command-line}

`fauste-player --version` mostra la versió. La barra de títol de la finestra
mostra el nom i la versió, i el botó **Quant a Fauste Player** (una icona
d'informació, a l'esquerra de **Configuració**) obre la finestra **Quant a
Fauste Player**, amb el copyright i els avisos de llicència (**Llicències de
tercers** obre el fitxer d'avisos instal·lat amb els paquets de release).
En un idioma traduït amb IA, aquesta finestra també diu que la traducció pot
contenir errors. Tanca-la amb **Tancar** o amb `Esc`. `fauste-player --help`
llista les opcions. Els fitxers de llista donats com a arguments
s'importen com a llistes noves.

![La finestra Quant a Fauste Player: versió, copyright i llicències dels components inclosos](../../images/guide/about.png)

A Windows el programa de release no obre cap finestra de consola:
`--version`, `--help` i els errors d'inici apareixen en un quadre de
missatge.

Quan un ajust necessita un reinici, a la barra superior apareix una píndola
**Reinici pendent**: prem-la per reiniciar (vegeu
[Configuració](settings.md#restart-pending)).

### Tancar amb àudio en antena {#closing-while-audio-is-on-air}

Tancar la finestra mentre alguna cosa és en antena no tanca l'aplicació. La
finestra passa a primer pla (fins i tot si estava minimitzada) i un diàleg
**Hi ha àudio en antena** llista el que sona: els reproductors que estan
reproduint o en pausa (`P1 — títol`) i els cartutxos que sonen amb el seu
número a la pàgina (`Cartutx 3 — títol`). El CUE d'un reproductor o de la
cartutxera no compta. Tria **Cancel·lar** (o `Esc`, o fes clic fora del
diàleg) per continuar reproduint, o **Aturar i tancar** per aturar tots els
reproductors en antena i tots els cartutxos i després sortir. La sessió es
desa com en qualsevol altra sortida. Si no hi ha res en antena, la finestra
es tanca a l'instant. El diàleg té prioritat sobre Configuració i Quant a
Fauste Player, i les dreceres de teclat no fan res mentre és obert (les
ordres MIDI i remotes continuen actuant).

## Primer inici {#first-start}

La finestra s'obre amb quatre reproductors, cadascun amb una llista buida.
No sona res fins que prems Play: això també és així després d'un reinici o
d'una fallada.

## Afegir música {#add-music}

- Fes clic a **Afegir** a la part inferior d'un reproductor i tria fitxers, o
- arrossega fitxers d'àudio o carpetes des del gestor de fitxers a una llista
  de pistes. Les carpetes afegeixen els fitxers d'àudio que contenen
  directament, no les seves subcarpetes.

Formats compatibles: WAV, AIFF, CAF, FLAC, MP3 (i MP1/MP2), AAC/M4A, ALAC, Ogg Vorbis, Opus, WavPack, Monkey's Audio (APE), DSD (DSF i DFF) i àudio Matroska (MKA).
Els fitxers DSD es reprodueixen convertits a PCM a la seva freqüència
dividida per 32 (88,2 kHz per a DSD64), o sense canvis com a DoP o DSD natiu
en un dispositiu bit perfect configurat per fer-ho (vegeu
[Sortida bit perfect](bit-perfect.md#dsd)). L'Opus sempre es reprodueix a
48 kHz. Els fitxers WavPack han de ser sense pèrdua i mono o estèreo; el
WavPack híbrid i multicanal apareix com a il·legible, igual que els fitxers
DFF comprimits amb DST.

Cada fitxer s'analitza en segon pla. L'anàlisi llegeix el títol, l'artista,
l'àlbum i la caràtula, dibuixa la forma d'ona i troba on comença i acaba el
so i on cal mesclar amb la pista següent. Pots reproduir una pista abans que
acabi la seva anàlisi.

## Reproduir {#play}

- Prem **Play** (o la tecla numèrica del reproductor, `1` per a P1) per
  iniciar la pista **següent**, marcada en verd a la llista.
- Prem **Play** de nou mentre una pista és en antena per fer un fos cap a la
  següent.
- Fes **doble clic** en una pista per fer-ne la següent.

En mode **CONT** (continu) el reproductor mescla per si sol amb la pista
següent al punt MIX. En mode **SINGLE** s'atura al final de cada pista.
Continua amb [Reproductors](players.md).
