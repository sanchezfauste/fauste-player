# Arazoen konponbidea

## Soinurik ez {#no-sound}

1. Ireki **Ezarpenak → Audio-irteerak** eta sakatu **Probatu Main**
   erreproduzitzailearentzat. Tonua entzuten baduzu, egiaztatu
   erreproduzitzailearen bolumen-faderra.
2. Ezer entzuten ez baduzu, aukeratu beste gailu bat edo beste kanal-bikote
   bat. Irteeren aldaketa bat ezer jotzen ez denean aplikatzen da irteera
   horietan; Ezarpenetako **Aplikatu orain** botoiak berehala aplikatzen du.
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
  bakoitza: itzaltzerik ez, irteera berberetan beste ezer ez, aztertuta
  dagoen galerarik gabeko fitxategi bat, eta fitxategiaren maiztasunean
  dabilen gailu bat.
- **Isiltasun labur bat pista baten aurretik.** Bit perfect gailua pistaren
  lagintze-maiztasunean berriro ireki da. Maiztasuna aldatzen denean bakarrik
  gertatzen da: maiztasun berean dauden pistak tarterik gabe doaz batetik
  bestera. Saihesteko, mantendu liburutegia maiztasun bakarrean. Ikus
  [Zer kostatzen den](bit-perfect.md#what-it-costs).
- **Pistak ez dira nahasten, eta Play-k ez du itzaltzen.** Main irteera bit
  perfect duen erreproduzitzaile batean, pista bakoitza amaierara arte jotzen
  da eta gero hurrengoa hasten da, eta Play / Hurrengoa eta Aurrekoa
  berehala aldatzen dira: erreproduzitzaile horretan ez dago gurutzaketarik
  ez segue-rik. Desaktibatu bit perfect gailuarentzat, edo eraman
  erreproduzitzailea beste irteera batera, berriro nahasteko. Ikus
  [Zer kostatzen den](bit-perfect.md#what-it-costs).
- **Bolumen-faderra ez da mugitzen.** Erreproduzitzailearen Main irteera bit
  perfect da, eta horrek bolumena % 100ean finkatzen du; faderraren
  argibideak hala dio. MIDI faderrek eta urruneko APIak ere ezin dute
  mugitu. Ikus [Zer kostatzen den](bit-perfect.md#what-it-costs).
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

Irakurtezin gisa agertzen den, edo zarata ozen gisa joko litzatekeen WAV edo
FLAC bat DTS-CD ripping bat izan daiteke: fitxategiak audioaren ordez DTS
bitstream bat dauka. Aplikazioak baztertu egiten du (erregistroak "encoded
bitstream stored as PCM" dio) eta ez du DTS deskodetzen. Bihurtu lehenik PCM
arrunt batera beste tresna batekin.

## Zaindutako karpeta batek abisu bat erakusten du {#a-watched-folder-shows-a-warning}

Karpeta bat zaintzen duen kategoria batek ⟳ erakusten du, beste kolore
batean karpeta jarraitzen ez denean; pasatu sagua gainetik arrazoia ikusteko.
Ikus [Zaindutako karpetak](library.md#watched-folders).

- **Ez da aurkitu edo ezin da irakurri**: unitatea edo partekatutako
  baliabidea ez dago muntatuta edo ez dago eskuragarri. Liburutegiko pistak
  izan beharko lituzkeen karpeta huts bat ere ez aurkitutzat hartzen da,
  muntatu gabeko partekatutako baliabide batek halakoa dirudielako. Ez da
  ezer markatzen edo kentzen; karpeta 30 segundotik behin bilatzen da berriro
  eta agertzen denean arakatzen da. Karpeta nahita hustu baduzu, garbitu
  bere pistak liburutegiko taulan hautatuz eta **Kendu liburutegitik…**
  aukeratuz, edo haien gainean **Berriro aztertu** aukeratuz; horrek
  fitxategia galdu duten pistak falta gisa markatzen ditu, **Kendu falta
  diren pistak…** aukerak kendu ahal ditzan.
- **Aldaketak ez dira jarraitzen**: sistemak ez luke zainduko, normalean
  azpikarpeta oso asko dituen karpeta bat delako. Karpeta abiarazte
  bakoitzean eta saiakera bakoitzean arakatzen da, beraz fitxategi berriak
  orduan agertzen dira, ez berehala. Linuxen, erabiltzaileko zaintza-mugak
  gainditzen duen karpeta-zuhaitz batek `fs.inotify.max_user_watches` altuago
  bat behar du (adibidez `sudo sysctl fs.inotify.max_user_watches=524288`;
  egin iraunkor `/etc/sysctl.d/` karpetan).
- **Ez da bide-izen osoa**: gordetako karpeta eskuz osatu gabeko bide-izen
  batera editatu da. Aukeratu **Aldatu zaindutako karpeta…** berriro
  aukeratzeko.
- **Sareko partekatutako baliabide batek ez du abisurik erakusten, baina
  beste ordenagailu batetik gehitutako fitxategiak ez dira agertzen**: sareko
  partekatutako baliabide batean (SMB/CIFS, NFS) sistemak normalean
  zaintza onartzen du, baina ordenagailu honetatik egindako aldaketak
  bakarrik jakinarazten ditu. Beste nonbaitetik egindako aldaketak
  hurrengo abiaraztean ikusten dira, ez aplikazioa abian dagoen bitartean.
- **Milaka fitxategiko karpeta batek** denbora pixka bat behar du
  abiaraztean arakatzeko; analisiaren aurrerapenak erakusten du.
- Zaindutako karpeta batetik kanpo geratu den fitxategiaren pistak
  liburutegian geratzen dira falta gisa markatuta. Kendu itzazu kategoriaren
  menuko **Kendu falta diren pistak…** aukerarekin.

## Audio-etenak {#audio-dropouts}

Egoera-barrak 5 segundoz ohartarazten du aplikazioak detektatzen duen
eten bakoitzaren ondoren: **P1: audio-etenak (3)** erreproduzitzaile baten
deskodetzeak diskoaren erritmoari eutsi ez dionean (zenbaketa orain jotzen
ari den pistarena da), eta **&lt;gailua&gt;: audio-gailuaren etenak (2)**
irteera-gailuak epe-muga bat galdu duenean (xrun bat). Erregistroak ere
bakoitza jasotzen du, gehienez lerro bat 10 segundotik behin mota
bakoitzeko, zenbat gertatu diren adieraziz. Audio-sistema guztiek ez dituzte
xrunak adierazten (PulseAudiok ez; Windowsen modu esklusiboak ez).


- Handitu **bufferraren tamaina** Ezarpenetan (gailu horretan ezer jotzen ez
  denean aplikatzen da, edo berehala **Aplikatu orain** botoiarekin).
- Soinu-zerbitzari baten bidez (PulseAudio, ALSA `default` gailua), irteera
  guztiek gutxienez 50 ms eskatzen dituzte bufferraren tamaina edozein dela
  ere, ordenagailu lanpetu bat entzun ez dadin; Ezarpenek latentziaren ondoan
  erakusten dute. Ordenagailu zahar edo oso kargatu batean, igo
  `shared_buffer_ms` konfigurazio-fitxategiaren `outputs` atalean (adibidez
  100era). Sistemak bloke horietako bi inguru ilaran mantentzen ditu, beraz
  kartutxo eta play botoiek orduan igoeraren bikoitz inguru geroago
  jotzen dute. Aireko irteerarako, gailuaren hardware-sarrera propioak
  (`hw:` Linuxen) soinu-zerbitzaria saihesten du guztiz.
- Linuxen, baimendu denbora errealeko planifikazioa. Aplikazioak sistemari
  eskatzen dio rtkit (D-Bus) bidez, eta erregistroak uko egitea jasotzen du.
  Mahaigain askok uko egiten diete aplikazioei.
- Saihestu sare-unitateak airean jotzen den musikarako.

## Interfazea mantsoa da ordenagailu zahar batean {#the-interface-is-slow-on-an-old-computer}

Zerbait jotzen ari den bitartean, interfazeak gehienez 30 frame segundoko
marrazten ditu (`max_fps`, konfigurazio-fitxategiaren `ui` atalean), eta
klikei eta teklei berehala erantzuten zaie. Ordenagailu zahar edo lanpetu
batean jaitsi (adibidez 20ra) prozesadorearen zati handiagoa audiorako uzteko;
0 balioak pantailak erakusten duen frame guztiak marrazten ditu. Neurgailuek
berdin neurtzen dute edozein maiztasunetan; gutxiagotan marrazten dira soilik.

## Fitxategiak ezin dira jaregin Linuxen {#files-cannot-be-dropped-on-linux}

Leihoak fitxategi-kudeatzaile batetik arrastatutako fitxategiak X11n
exekutatzen denean bakarrik hartzen ditu. Berez hala egiten du, Wayland
mahaigain batean ere (XWayland bidez, mahaigain guztiek dutena). `config.json`
fitxategiko `ui.window_system` `"Wayland"` balioan ezarrita badago, leihoa
Wayland natibo bat da, eskalatze zatikatuarekin zorrotzagoa ager daitekeena,
baina fitxategiak ezin dira bertan jaregin: erabili **+ Gehitu** horren
ordez. Ezarri berriro `"Auto"` (lehenetsia) edo `"X11"` eta berrabiarazi
Fauste Player.

## MIDI kontrolagailu batek ez du erantzuten {#a-midi-controller-does-not-respond}

**Ezarpenak → MIDI** atalak kontrolagailua konektatuta erakusten badu baina
bere botoiek eta faderrek ez badute ezer egiten, sakatu han **Konektatu
berriro atakak**: programak MIDI atakak itxi eta berriro irekitzen ditu,
kontrolagailua deskonektatu eta berriro konektatuko bazenu bezala. Horrek
laguntzen ez badu, deskonektatu kontrolagailua eta konektatu berriro.

Edonola ere, erregistroak zer gertatu den dio. MIDI atakak zerrendatzen ditu
aldatzen diren bakoitzean, irekitako eta itxitako ataka bakoitza (zergatiarekin),
ireki ezin den ataka bat (behin, ireki arte edo desagertu arte), konexio
bakoitzean jasotako lehen mezua, konexio bakoitzean ez ikusi egindako lehen
mezua arrazoiarekin (loturarik ez, bolumenera oraindik iritsi ez den fader
bat, orain eragin ezin duen ekintza bat) eta huts egin duen lehen LED mezua.
Lehen mezurik inoiz erregistratzen ez duen kontrolagailu konektatu batek
programak jasotzen ez duen ezer ez du bidaltzen. Ez ikusi egindako mezu
bakoitzerako eta huts egindako berriro saiakera bakoitzerako, abiarazi
programa `RUST_LOG=info,fp_control=debug` erabilita.

## "Interfazeak errore bat izan du" {#the-interface-hit-an-error}

Marrazketa-errore bat harrapatu da. Audioari ez dio eragiten. Sakatu
**Berrabiarazi interfazea**. Mesedez, jakinarazi erregistroekin.

## Erregistroak eta hutsegite-txostenak {#logs-and-crash-reports}

Ikus [Datuak eta babeskopiak](data-and-backups.md) erregistroen
karpetarako. Egun bakoitzeko erregistro-fitxategi bat dago. Segidan errepikatzen
diren lerro berdinak behin idazten dira, eta ondoren
`previous message repeated 12 times` bezalako lerro bat (errepikapen-sail
luze batean lerro hori minuturo idazten da). Desberdinak diren lerroak,
adibidez kontagailu bat dutelako, ez dira biltzen. PulseAudio bezeroaren
konexio-mezu arruntak lehenespenez kanpoan uzten dira. Ezarri
`RUST_LOG=debug` ingurunean xehetasun gehiago lortzeko (lehenetsitako iragazkia
ordezten du, `info,pulseaudio=warn`).

Erregistroak mugak ditu: egun bateko fitxategia 10 MiB-tan uzten da hazteari
(bere azken lerroak hala dio; egun horren gainerakoa ez da erregistratzen,
hori da lerro desberdinen uholde batek eragiten duena) eta azken 14
fitxategiak gordetzen dira. Aldatu itzazu konfigurazio-fitxategian
`limits.max_log_file_mb` (1etik 1024ra), `limits.max_log_files` (2tik 365era)
eta `limits.log_repeat_summary_secs` (1etik 3600ra, lehenetsia 60,
errepikapen-sail luze bat zenbatero jakinarazten den) bidez; hurrengo
abiaraztean aplikatzen dira. Hutsegite-txostenak `crash-<time>.txt` gisa
gordetzen dira karpeta berean, eta bereizita mugatzen dira
(`limits.max_crash_reports` exekuzio bakoitzeko). Erantsi bi fitxategiak akats
bat jakinaraztean.
