# Arazoen konponbidea

## Soinurik ez {#no-sound}

1. Ireki **Ezarpenak → Audio-irteerak** eta sakatu **Probatu Main**
   erreproduzitzailearentzat. Tonua entzuten baduzu, egiaztatu
   erreproduzitzailearen bolumen-faderra.
2. Ezer entzuten ez baduzu, aukeratu beste gailu bat edo beste kanal-bikote
   bat. Irteeren aldaketek berrabiarazi ondoren dute eragina: sakatu
   **Berrabiarazi orain** Ezarpenetan.
3. Linuxen, hobetsi **PipeWire** edo **PulseAudio** Ezarpenak →
   Audio-irteerak → Audio-sistema atalean. Soinu-txartela beste programa
   batzuekin partekatzen dute. **ALSA**k zuzenean hitz egiten du
   txartelarekin eta lanpetuta aurki dezake.
4. **JACK** erabilgarri ez gisa agertzen da ("irteera-gailurik ez") JACK
   zerbitzaririk abian ez dagoenean. Abiarazi zerbitzaria (adibidez
   QjackCtl-ekin) eta berrabiarazi aplikazioa. Ezarri JACK zerbitzaria
   Ezarpenetako lagintze-maiztasunean (lehenespenez 48 kHz): JACK-ek
   maiztasun bakarrean lan egiten du programa guztientzat.
5. Deskarga daitezkeen artxiboek ez dute **PipeWire** eskaintzen; PipeWire-ra
   bere PulseAudio zerbitzuaren bidez iristen dira, modu berean
   funtzionatzen duena. `pipewire` ezaugarriarekin egindako konpilazioetan
   dago erabilgarri.

## Bit perfect {#bit-perfect}

- **BP ikurra itzalita geratzen da.** Egiaztatu
  [Bit perfect irteera](bit-perfect.md#the-bp-badge) ataleko baldintza
  bakoitza: bolumena % 100ean, itzaltzerik ez, irteera berberetan beste
  ezer ez, aztertuta dagoen galerarik gabeko fitxategi bat, eta
  fitxategiaren maiztasunean dabilen gailu bat.
- **Isiltasun labur bat pista baten aurretik.** Bit perfect gailua pistaren
  lagintze-maiztasunean berriro ireki da. Saihesteko, mantendu liburutegia
  maiztasun bakarrean.
- **Pista bat birlaginduta jotzen da, eta erregistroak gailua lanpetuta
  dagoela dio (Linux).** Maiztasuna aldatzeko, aplikazioak gailua itxi eta
  berriro irekitzen du. Une horretan soinu-zerbitzariak (PipeWire)
  txartela har dezake. Aplikazioa hainbat aldiz saiatzen da berriro;
  txartela oraindik lanpetuta badago, pista gailuaren uneko maiztasunean
  jotzen da, eta hurrengo pistak berriro eskatzen du bere maiztasuna.
  Aplikazioari txartela berarentzat bakarrik emateko, ireki sistemaren
  soinu-ezarpenak eta ezarri txartel horren profila **Off** (itzalita) edo
  **Pro Audio** gisa, soinu-zerbitzariak bere `hw:` gailua bakean utz
  dezan. Saiakera kopurua eta haien arteko itxaronaldia
  `tuning.device_busy_retries` eta `tuning.device_busy_retry_ms` dira
  konfigurazio-fitxategian.
- **Gailuak jotzen du, baina BP ikurra itzalita geratzen da (Windows edo
  macOS).** Sarbide esklusiboa ukatu da, eta gailuak partekatuta jotzen
  du.
  - Windows: beste programa batek gailua modu esklusiboan har dezake, edo
    kontrol esklusiboa desaktibatuta dago gailuaren propietate
    aurreratuetan.
  - macOS: beste programa batek gailua hog moduan har dezake, edo gailuak
    bere maiztasunak tarte jarraitu gisa bakarrik eskaintzen ditu
    (interfaze gehienek maiztasun finkoak zerrendatzen dituzte).
- **`hw:` gailu bat ezin da ireki (Linux).**
  - Soinu-zerbitzari batek txartela hartuta izan dezake. Gelditu, edo
    konfiguratu zerbitzaria txartel hori bakean utz dezan, eta berrabiarazi
    aplikazioa.
  - USB DAC batzuek 24 biteko lagin paketatuak (`S24_3LE`) bakarrik onartzen
    dituzte, audio-liburutegiak onartzen ez dituenak. Erabili txartel hori
    `plughw:` bidez (ez da bit perfect).

### DSD {#dsd}

- **DSD pista bat bihurtuta jotzen da, gailua DoP edo DSD natibo gisa
  konfiguratuta egon arren.** Erregistroak zergatik adierazten du ("DSD
  converted to PCM" eta arrazoia) kausa hauetarako: erreproduzitzailearen
  bolumena ez dago % 100ean, beste zerbait jotzen ari da gailuan, bi kanal
  baino gehiago daude, edo gailuak maiztasuna ukatzen du (DoP-ek DSD
  maiztasuna 16z zatituta behar du, adibidez 176,4 kHz DSD64rako) edo ez du
  24 edo 32 biteko formaturik. Oraindik aztertu ez den pista bat isilean
  bihurtzen da, erregistro-lerrorik gabe: aztertu (Ezarpenak → Analisia) eta
  jo berriro.
- **DSD album bateko lehen pista bakarrik ateratzen da DSD gisa.** Hori da
  nahasketa-ezarpen lehenetsia: erreproduzitzaileak berez hasten dituen
  pistak bihurtuta jotzen dira. Aukeratu **Mantendu DSDa eta isilarazi beste
  iturriak** Ezarpenak → Audio-irteerak atalean DSD gisa mantentzeko. Ikus
  [DSD](bit-perfect.md#dsd).
- **Goiburuak DSD erakusten du, baina bihurgailuak zarata jotzen du edo ez
  da sinkronizatzen.** Bihurgailuak ez du DoP (edo formatu natiboa)
  ezagutzen. Itzuli gailua **Bihurtu PCM** aukerara.
- **Klik bat DSD pista bat hasten, gelditzen edo DSDtik irteten denean.**
  Bihurgailuak DSD isiltasun gehiago behar du: handitu **DSD isiltasuna**
  (lehenespenez 200 ms) Ezarpenak → Audio-irteerak, Aurreratua atalean.
- **Beste erreproduzitzaile edo kartutxo batzuk isilik daude gailuan.** DSD
  pista bat jotzen ari da **Mantendu DSDa eta isilarazi beste iturriak**
  aukerarekin; **Besteak isilik** ikurra agertzen da. Pista amaitzean
  berriro entzuten dira.

## "Irteera galdu da" abisua {#output-lost-alert}

Egoera-barrak **Irteera galdu da: &lt;gailua&gt;** erakusten du gailu
batek erantzuteari uzten dionean. Erreproduzitzaileek barne-erloju batean
zenbatzen eta nahasten jarraitzen dute, automatizazioa geldi ez dadin.
2 segundotik behin gailua berriro irekitzen saiatzen da, eta itzultzen
denean berriro hartzen du lana. Konektatu berriro kablea edo piztu berriro interfazea.

### Inoiz desagertzen ez den "Irteera galdu da", `hw:` irteera zuzen batekin {#output-lost-that-never-clears-with-a-direct-hw-output}

ALSA `hw:` irteera zuzen baten bidez erabilitako soinu-txartel bat (adibidez
bit perfect irteera bat) Fauste Player-ek bakarrik hartzen du:
soinu-zerbitzariak (PipeWire edo PulseAudio) ezin du aldi berean erabili.
Beste irteera bat soinu-zerbitzariaren gailu lehenetsitik igarotzen bada eta
gailu lehenetsi hori txartel bera bada, irteera hori ez da inoiz abiarazten
eta **Irteera galdu da** egoeran geratzen da. Erregistroak "output device
opened but never started" dio behin.

Erabili bide bakarra txartel bakoitzeko: bideratu txartel horren irteera
guztiak `hw:` gailu beretik (kanal desberdinekin, behar izanez gero), edo
aukeratu beste txartel bat soinu-zerbitzariaren irteera lehenetsi gisa
zure sistemaren soinu-ezarpenetan.

## Pista batek abisu-ikono bat edo gurutzea duen fitxategi bat erakusten du {#a-track-shows-a-warning-icon-or-a-file-with-a-cross}

Fitxategia falta da (mugituta, ezabatuta, desmuntatuta: gurutzea duen
fitxategia) edo ezin da deskodetu (abisu-ikurra). Erreproduzitzaileek
saltatu egiten dute. Pasatu sagua errenkadaren gainetik zein den eta
fitxategiaren bide-izena ikusteko.

Falta den fitxategi bat berriro bilatzen da 30 segundotik behin
(`tuning.missing_recheck_ms` konfigurazio-fitxategian): unitatea
muntatzen denean edo fitxategia bere lekura itzultzen denean, pista berez
bihurtzen da erreproduzigarri. Deskodetu ezin den fitxategi bat berez
egiaztatzen da berriro tenporizadore berarekin, tamainaren eta
aldaketa-dataren arabera: ez da berriro deskodetzen horietako bat aldatu ez bada,
adibidez kopia bat amaitzen denean. Berehala egiaztatzeko, erabili
**Berriro aztertu** bere errenkadaren menuan, edo **Ezarpenak → Analisia →
Berriro aztertu pista guztiak** liburutegi osorako.

## Audio-etenak {#audio-dropouts}

Egoera-barrak 5 segundoz ohartarazten du aplikazioak detektatzen duen
eten bakoitzaren ondoren: **P1: audio-etenak (3)** erreproduzitzaile baten
deskodetzeak diskoaren erritmoari eutsi ez dionean (zenbaketa orain jotzen
ari den pistarena da), eta **&lt;gailua&gt;: audio-gailuaren etenak (2)**
irteera-gailuak epe-muga bat galdu duenean (xrun bat). Erregistroak ere
bakoitza jasotzen du, gehienez lerro bat 10 segundotik behin mota
bakoitzeko, zenbat gertatu diren adieraziz. Audio-sistema guztiek ez dituzte
xrunak adierazten (PulseAudiok ez; Windowsen modu esklusiboak ez).


- Handitu **bufferraren tamaina** Ezarpenetan (eta sakatu **Berrabiarazi
  orain**).
- Linuxen, baimendu denbora errealeko planifikazioa. Aplikazioak sistemari
  eskatzen dio rtkit (D-Bus) bidez. `audio` taldeko kide izateak `rtprio`
  muga batekin ere funtzionatzen du.
- Saihestu sare-unitateak airean jotzen den musikarako.

## "Interfazeak errore bat izan du" {#the-interface-hit-an-error}

Marrazketa-errore bat harrapatu da. Audioari ez dio eragiten. Sakatu
**Berrabiarazi interfazea**. Mesedez, jakinarazi erregistroekin.

## Erregistroak eta hutsegite-txostenak {#logs-and-crash-reports}

Ikus [Datuak eta babeskopiak](data-and-backups.md) erregistroen
karpetarako. Egun bakoitzeko erregistro-fitxategi bat dago, eta azken 14ak
gordetzen dira. Hutsegite-txostenak `crash-<time>.txt` gisa gordetzen dira.
Ezarri `RUST_LOG=debug` ingurunean xehetasun gehiago lortzeko. Erantsi bi
fitxategiak akats bat jakinaraztean.
