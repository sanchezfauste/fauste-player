# Lehen urratsak

## Instalazioa {#install}

Deskargatu zure sistemarako pakete bat proiektuaren **Releases** orritik.
Fitxategi bakoitzak `.sha256` bat du ondoan. Deskarga bat egiaztatzeko,
exekutatu `sha256sum -c <file>.sha256` Linuxen, edo
`shasum -a 256 -c <file>.sha256` macOSen.

### Linux {#linux}

| Paketea | Instalatu | Eguneratu | Kendu |
|---|---|---|---|
| `.deb` (Debian, Ubuntu) | `sudo apt install ./fauste-player_<version>_amd64.deb` | instalatu `.deb` berriagoa modu berean | `sudo apt remove fauste-player` |
| `.rpm` (Fedora, openSUSE) | `sudo dnf install ./fauste-player-<version>-1.x86_64.rpm` | instalatu `.rpm` berriagoa | `sudo dnf remove fauste-player` |
| AppImage (edozein banaketa) | `chmod +x fauste-player-<version>-x86_64.AppImage`, eta exekutatu | ordeztu fitxategia | ezabatu fitxategia |
| Flatpak paketea | `flatpak install --user fauste-player-<version>-x86_64.flatpak` | instalatu pakete berriagoa | `flatpak uninstall org.fauste.FaustePlayer` |
| Artxibo konprimitua | erauzi eta exekutatu `fauste-player` | erauzi berriagoa | ezabatu karpeta |

- **Mahaigainarekiko integrazioa:** paketeek **Fauste Player** gehitzen
  diote aplikazioen menuari, eta `.m3u`, `.m3u8` eta `.pls` zerrendak
  harekin irekitzeko aukera ematen dute; zerrenda berri gisa inportatzen
  dira.
- **Beharrezko liburutegiak:** ALSA eta D-Bus liburutegiak. Mahaigain
  guztiek dituzte, eta paketeek adierazten dituzte. JACK instalatuta
  dagoenean erabiltzen da, eta ez da inoiz derrigorrezkoa.
- **AppImage:** FUSE falta delako abiarazten ez bada, exekutatu
  `--appimage-extract-and-run` aukerarekin.
- **Flatpak:**
  - Paketeak Flathub-eko freedesktop runtimea behar du. Flathub urruneko
    biltegia konfiguratuta ez badago, exekutatu lehenik
    `flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo`.
  - Sandboxak zure karpeta pertsonala irakur dezake (zure musika dagoen
    lekuan erreproduzitzeko).
  - PulseAudio bidez entzuten da, edo zuzenean ALSA bidez bit perfect
    gailuetarako.
- **Mahaigaineko liburutegiak:** leihoak libxkbcommon eta EGL edo OpenGL
  erabiltzen ditu, Wayland edo X11rekin. Mahaigain guztiek dituzte, eta
  `.deb` eta `.rpm` paketeek adierazten dituzte. Sistema oso minimo batean,
  instalatu itzazu AppImage erabili aurretik.

### Windows {#windows}

Exekutatu `fauste-player-<version>-x86_64-pc-windows-msvc.msi`. Erabiltzaile
guztientzat instalatzen da *Program Files* karpetan, eta sarrera bat
gehitzen dio Hasiera menuari.
- **Eguneratzea:** exekutatu instalatzaile berriagoa. Instalatutako bertsioa
  ordezten du.
- **Kentzea:** erabili **Ezarpenak → Aplikazioak** (**Settings → Apps**).
- **Sinatu gabeko instalatzailea:** argitalpena sinatuta ez badago,
  SmartScreen-ek argitaratzaile ezezagun bati buruz ohartarazten du.
  Aukeratu **Informazio gehiago → Exekutatu hala ere** (**More info → Run
  anyway**).

`.zip` artxiboa aukera eramangarri bat da: erauzi eta exekutatu
`fauste-player.exe`.

### macOS {#macos}

Ireki `fauste-player-<version>-macos-universal.dmg` eta arrastatu **Fauste
Player** **Applications** karpetara. Aplikazio bera Apple silicon eta Intel
prozesadoreetan dabil (macOS 11 edo berriagoa).
- **Eguneratzea:** ordeztu aplikazioa modu berean.
- **Kentzea:** eraman Zakarrontzira.
- **Sinatu gabeko aplikazioa:** argitalpena sinatuta eta notarizatuta ez
  badago, macOSek uko egiten dio lehen abiarazteari.
  - macOS 15 eta berriagoak: ireki **System Settings → Privacy & Security**,
    korritu Fauste Player-i buruzko mezuraino, aukeratu **Open Anyway** eta
    berretsi.
  - macOS 14 eta zaharragoak: egin eskuineko klik aplikazioan, aukeratu
    **Open** eta berretsi.
  - Edo exekutatu `xattr -dr com.apple.quarantine "/Applications/Fauste Player.app"`.
- **JACK macOSen:** sinatutako eta notarizatutako aplikazio batek bera ere
  sinatuta dagoen JACK liburutegi bat bakarrik karga dezake. Bestela, JACK
  erabilgarri ez gisa agertzen da; erabili Core Audio.
- **Zerrendak:** macOSen Ezarpenak → Zerrendak ataletik inportatzen dira.
  Ez da onartzen zerrenda-fitxategi bat Finder-etik aplikazioarekin
  irekitzea.

### Instantzia bakarra aldi berean {#one-instance-at-a-time}

Datu-karpeta bakoitzeko Fauste Player bakarra exekutatzen da. Abian
dagoela fitxategi-kudeatzailetik zerrenda bat irekitzen baduzu, zerrenda
hori abian dagoen aplikaziora inportatzen da. Zerrendarik gabe berriro
abiarazten baduzu, abian dagoela dioen mezu bat erakusten du. Instantzia
bereiziak bata bestearen ondoan exekutatzeko (adibidez, bi estudio
ordenagailu bakarrean), eman bakoitzari bere karpeta `FAUSTE_HOME` bidez.

### Komando-lerroa {#command-line}

`fauste-player --version` komandoak bertsioa inprimatzen du. Leihoaren
titulu-barrak izena eta bertsioa erakusten ditu, eta **Fauste Player-i
buruz** botoiak (informazio-ikono bat, **Ezarpenak** botoiaren ezkerrean)
**Fauste Player-i buruz** leihoa irekitzen du, copyrightarekin eta lizentzia-
oharrekin (**Hirugarrenen lizentziak** botoiak argitalpen-paketeekin
instalatzen den oharren fitxategia irekitzen du). AArekin itzulitako
hizkuntza batean, leiho horrek itzulpenak akatsak izan ditzakeela ere
adierazten du. Itxi **Itxi** botoiarekin edo `Esc` teklarekin.
`fauste-player --help` komandoak aukerak zerrendatzen ditu. Argumentu gisa
emandako zerrenda-fitxategiak zerrenda berri gisa inportatzen dira.

![Fauste Player-i buruz leihoa: bertsioa, copyrighta eta barneratutako osagaien lizentziak](../../images/guide/about.png)

Windowsen, argitalpeneko programak ez du kontsola-leihorik irekitzen:
`--version`, `--help` eta abiaraztean gertatutako erroreak mezu-koadro
batean agertzen dira.

Ezarpen batek berrabiaraztea behar duenean, **Berrabiaraztea falta da**
pilula bat agertzen da goiko barran: sakatu berrabiarazteko (ikus
[Ezarpenak](settings.md#restart-pending)).

### Audioa airean dagoela ixtea {#closing-while-audio-is-on-air}

Zerbait airean dagoela leihoa ixteak ez du aplikaziotik irteten. Leihoa
aurrealdera dator (minimizatuta egon arren) eta **Audioa airean dago**
elkarrizketa-koadro batek entzuten ari dena zerrendatzen du: jotzen edo
pausan dauden erreproduzitzaileak (`P1 — izenburua`), eta jotzen ari diren
kartutxoak, orriko beren zenbakiarekin (`3. kartutxoa — izenburua`).
Erreproduzitzaile baten edo kartutxo-panelaren CUEa ez da kontuan hartzen.
Aukeratu **Utzi** (edo `Esc`, edo egin klik elkarrizketa-koadrotik kanpo)
jotzen jarraitzeko, edo **Gelditu eta itxi** airean dauden erreproduzitzaile
guztiak eta kartutxo guztiak gelditu eta irteteko. Saioa beste edozein
irteeratan bezala gordetzen da. Airean ezer ez dagoenean, leihoa berehala
ixten da. Elkarrizketa-koadroak lehentasuna du Ezarpenen eta Fauste
Player-i buruz leihoaren gainetik, eta laster-teklek ez dute ezer egiten
irekita dagoen bitartean (MIDI eta urruneko aginduek eragina dute oraindik).

## Lehen abiaraztea {#first-start}

Leihoa lau erreproduzitzailerekin irekitzen da, bakoitza zerrenda huts
batekin. Ez da ezer entzuten Play sakatu arte: hori bera gertatzen da
berrabiarazte edo hutsegite baten ondoren ere.

## Gehitu musika {#add-music}

- Egin klik erreproduzitzaile baten beheko **+ Gehitu** botoian eta
  aukeratu fitxategiak, edo
- arrastatu audio-fitxategiak edo karpetak fitxategi-kudeatzailetik pista-
  zerrenda batera. Karpetek zuzenean barruan dituzten audio-fitxategiak
  gehitzen dituzte, ez azpikarpetetakoak.

Onartutako formatuak: WAV, AIFF, CAF, FLAC, MP3 (eta MP1/MP2), AAC/M4A, ALAC, Ogg Vorbis, Opus, WavPack, Monkey's Audio (APE), DSD (DSF eta DFF) eta Matroska audioa (MKA).
DSD fitxategiak PCM bihurtuta entzuten dira, beren maiztasuna 32z zatituta
(88,2 kHz DSD64rako), edo aldaketarik gabe DoP edo DSD natibo gisa, horretarako
konfiguratutako bit perfect gailu batean (ikus
[Bit perfect irteera](bit-perfect.md#dsd)). Opus beti 48 kHz-ean entzuten
da. WavPack fitxategiek galerarik gabekoak eta mono edo estereo izan behar
dute; WavPack hibridoak eta kanal anitzekoak irakurtezin gisa agertzen dira,
baita DST bidez konprimitutako DFF fitxategiak ere.

Fitxategi bakoitza atzeko planoan aztertzen da. Analisiak izenburua,
artista, albuma eta azaleko irudia irakurtzen ditu, uhin-forma marrazten
du, eta soinua non hasten eta amaitzen den eta hurrengo pistarekin non
nahastu aurkitzen du. Pista bat erreproduzi dezakezu bere analisia amaitu
aurretik.

## Erreproduzitu {#play}

- Sakatu **Play** (edo erreproduzitzailearen zenbaki-tekla, `1` P1erako)
  **hurrengo** pista abiarazteko, zerrendan berdez markatuta dagoena.
- Sakatu berriro **Play** pista bat airean dagoela, hurrengoarekin
  itzaltzearekin pasatzeko.
- Egin **klik bikoitza** pista batean hurrengoa izan dadin.

**CONT** (jarraitua) moduan, erreproduzitzaileak berak nahasten du
hurrengo pistarekin MIX puntuan. **SINGLE** moduan, pista bakoitzaren
amaieran gelditzen da. Jarraitu [Erreproduzitzaileak](players.md) atalarekin.
