# Bit perfect irteera

**Bit perfect** gailu batek fitxategi bakoitzaren laginak fitxategian dauden
bezala jasotzen ditu: lagintze-maiztasun bera, balio berberak, birlaginketarik,
bolumen-aldaketarik edo nahasketarik gabe. Erabilgarria da
monitorizazio-kateetarako eta lotura digitaletarako, non ordenagailuko edozein prozesamendu
saihestu behar den.

## Gailu bat bit perfect bihurtzea {#setting-a-device-bit-perfect}

1. **Ezarpenak → Audio-irteerak** atalean, aukeratu gailua esplizituki
   erreproduzitzaile baten Main irteerarako (edo kartutxo-panelarenerako).
   Sistemaren lehenetsian utzitako erreproduzitzaile bat ezin da bit
   perfect bihurtu. Irteera batek ere erabiltzen ez duen gailu batek bere
   bit perfect etengailua eta DSD modua galtzen ditu aplikazioa hurrengoz
   abiarazten denean.
2. Aukeratu **Aurreratua** atalaren goialdean. **Gailuko ezarpenak**
   azpian, aktibatu **Bit perfect** gailuaren ondoan.
3. Berrabiarazi aplikazioa.

Orduan gailua bere lagintze-maiztasunean hasten da, leku berean bat eman
badiozu, eta bestela lagintze-maiztasun orokorrean, eta hortik aurrera
fitxategi bakoitza jarraitzen du.

Etengailua desaktibatuta dago gailuak sarbide esklusiborik eman ezin
duenean.
- **Linux:** aukeratu izena `hw:` batekin hasten den ALSA gailu bat.
  Soinu-txartela bera da. PulseAudio, PipeWire, JACK eta ALSAren `default`
  edo `plughw:` gailuek nahastu edo bihurtu egiten dute, beraz ez dira inoiz
  bit perfect.
- **Windows:** aukeratu gailua **WASAPI** sisteman. Modu esklusiboan
  irekitzen da.
  - Windowsen soinu-ezarpenetan, gailuaren propietate **aurreratuetan**
    *aplikazioei gailu honen kontrol esklusiboa hartzen uztea* (*Allow
    applications to take exclusive control of this device*) aktibatuta
    egon behar du (lehenespenez aktibatuta dago).
  - Jotzen ari den bitartean, beste programa batek ezin du gailua erabili.
- **macOS:** aukeratu gailua **Core Audio** sisteman. Hog moduan irekitzen
  da.
  - Gailuaren lagintze-maiztasuna pistarena bihurtzen da, eta bere formatua
    maiztasun horretan eskaintzen duen osoko formaturik zabalena (Audio
    MIDI Setup-ek erakusten dituen ezarpenak).
  - Aplikazioak gailua erabiltzeari uzten dionean itzultzen dira.
  - Izen berbera duten bi gailu ezin dira bit perfect bihurtu.

## Zer gertatzen den bit perfect gailu batean {#what-happens-on-a-bit-perfect-device}

- **Sarbide esklusiboa.** Ordenagailuko beste ezerk ezin du gailuan jo
  aplikazioak erabiltzen duen bitartean. Sarbide esklusiboa ukatzen bada,
  gailuak jotzen jarraitzen du, partekatuta, eta BP ikurra itzalita
  geratzen da.
- **Maiztasunak fitxategia jarraitzen du.** Gailuan ezer jotzen ari ez
  denean eta beste lagintze-maiztasun bateko pista bat hasten denean, gailua
  maiztasun horretan berriro irekitzen da.
  - Hori gertatzen da pista bat jotzen duzunean, pausan kargatutako bat
    berriro abiarazten duzunean, aurrez entzuten duzunean edo kartutxo bat
    jaurtitzen duzunean. Zain baino ez dauden pistak (erreproduzitzaile
    bakoitzaren hurrengo pista) berriro prestatzen dira maiztasun berrian.
  - Berriro irekitzeak gailuak abiarazteko behar duen beste denbora
    hartzen du (normalean milisegundo hamarkada batzuk). Hasiera hainbeste
    atzeratzen da.
  - Gailuan zerbait jotzen ari den bitartean, maiztasuna ez da inoiz
    aldatzen. Orduan hasten den beste maiztasun bateko pista bat (adibidez,
    44,1 kHz-eko baten ondoren nahastutako 48 kHz-eko pista bat, edo beste
    erreproduzitzaile bat edo kartutxo bat gailu berean jotzen ari dela
    hasitako pista bat) bere iraupen osoan bihurtzen da, eta ez da bit
    perfect.
  - Gailuak maiztasun bat ukatzen badu, aurrekoa mantentzen du eta pista
    bihurtu egiten da.
- **Prozesamendurik ez, ezerk eskatzen ez duenean.** Laginak aldaketarik
  gabe igarotzen dira honako hauek guztiak betetzen diren bitartean:
  - erreproduzitzailearen bolumena % 100ean dago;
  - ez dago itzaltzerik abian;
  - irteera berberetan ez da beste ezer jotzen (beste erreproduzitzaile bat,
    kartutxo bat, proba-tonu bat).

## DSD {#dsd}

DSD fitxategi bat normalean PCM bihurtuta jotzen da, beste edozein
fitxategi bezala. Bit perfect gailu batek, ordea, DSD fluxua aldaketarik
gabe jaso dezake.

**Hiru moduak.** Ezarpenak → Audio-irteerak ataleko ikuspegi aurreratuan,
irteera batek erabiltzen duen gailu bakoitzak **DSD** aukera bat du bere bit
perfect etengailuaren azpian:
- **Bihurtu PCM** (lehenetsia): DSDa bihurtu egiten da, beste edozein
  gailutan bezala.
- **DoP** (DSD over PCM): DSD bitak 24 biteko PCM laginen barruan doaz, DSD
  onartzen duten bihurgailu gehienek ezagutzen dutena. Sistema guztietan
  funtzionatzen du.
- **DSD natiboa** (Linuxen bakarrik): DSD gordina, kontrolatzaileak DSD
  lagin-formatu bat adierazten duen ALSA `hw:` gailuetarako.

Gailuak onar ditzakeen moduak bakarrik eskaintzen dira, eta aukeraren
azpiko lerro batek besteak zergatik ez diren adierazten du: gailua ez dago
konektatuta, bit perfect desaktibatuta dago, gailua ezin da modu esklusiboan
ireki, DSD natiboak Linux behar du, edo gailuak ez du DSD natiborik
onartzen. Orain onartu ezin duen gailu baterako gordetako modu bat PCM
gisa agertzen da, hori baita jotzen dena; gordetako modua itzuli egiten da
gailuak berriro onar dezakeenean. Modu bat, nahasketa-ezarpena edo DSD
isiltasuna aldatzeak berrabiaraztea behar du, beste irteera-ezarpenek
bezala.

**DSDa aldaketarik gabe noiz ateratzen den.** Honako hauek guztiak bete
behar dira pista hasten denean:
- gailua bit perfect da, sarbide esklusiboarekin, eta bere modua DoP edo
  DSD natiboa da;
- pista DSD da (DSF edo DFF), mono edo estereo, eta aztertuta dago (horrela
  ezagutzen da bere DSD maiztasuna);
- erreproduzitzailearen bolumena % 100ean dago;
- gailuan ez da beste ezer jotzen (beste erreproduzitzaile bat, kartutxo
  bat, proba-tonu bat);
- gailuak fluxua onartzen du. DoP-ek DSD maiztasuna 16z zatituta ematen
  duen gailu-maiztasuna behar du (176,4 kHz DSD64rako, 352,8 kHz
  DSD128rako, 705,6 kHz DSD256rako) eta 24 edo 32 biteko formatu bat. DSD
  natiboak maiztasun horretan DSD formatua onartzen duen gailu bat behar du.

DSD natiboa amaitzean, gailua PCMra itzultzen da DSD pistaren aurretik zuen
maiztasunean, bihurgailu askok DSD natiboa onartzen baitute PCM gisa jo
ezin dituzten maiztasunetan (bihurgailu batek ere ez du PCM jotzen DSD512k
erabiltzen duen maiztasunean). PCM gisa jarraitzen duen pista batek DSD
fluxuaren maiztasuna mantentzen du gailuak PCM gisa onartzen duenean, eta
bestela aurreko maiztasunean jarraitzen du zegoen lekutik, gailu horretan
jotzen ari den beste guztia bezala; bertan abian dagoen itzaltze bat
berehala amaitzen da. Gailua ez da inoiz ukatzen duen maiztasun batean
geratzen: maiztasun batek ere irekitzen ez badu (adibidez, une horretan
gailua deskonektatu zen), irteera galdu egiten da berriro saiatze
automatikoak aurreko maiztasunean berriro ireki arte.

Bestela, pista PCM bihurtzen da eta erregistroak zergatik adierazten du
(adibidez "something else plays on the device" edo "the device refused
705600 Hz"). Aurrez entzutea eta kartutxoak beti bihurtzen dira.

DSDa aldaketarik gabe ateratzen den bitartean:
- goiburuko ikurrak **DSD** dio **BP** ordez;
- neurgailuek pista beraren PCM bihurketaren maila erakusten dute, beraz
  ohi bezala funtzionatzen dute;
- bolumenak % 100ean egon behar du: faderraren argibideak hala dio.
  Mugitzeak pista PCMra aldatzen du (ikus behean);
- Gelditu eta itzaltzearekin gelditzeak pista berehala gelditzen dute,
  itzaltzerik gabe, DSD fluxu bat ezin baita itzali. Jotzen ari den bitartean
  beste pista batean Play sakatzeak modu berean mozten du, nahasketa
  gurutzatua egin ordez;
- pausatzeak eta jarraitzeak ere berehala eragiten dute, arrapalarik gabe.

**Isiltasuna ertzetan.** Hasiera, amaiera eta PCMrako aldaketa bakoitzak
DSD isiltasuna bidaltzen du lehenik (lehenespenez 200 ms), bihurgailua
klikik gabe sinkroniza dadin. Salbuespena da mota eta DSD maiztasun
bereko fluxu bat jarraitzen duen DSD pista bat, haren isiltasuna oraindik
abian dagoela: bihurgailua oraindik sinkronizatuta dago, beraz isiltasun
gehigarririk gabe hasten da. Beraz, pista bat hainbeste atzerago hasten da,
eta PCMrako aldaketa batek luzera horretako tarte bat uzten du. **DSD
isiltasuna** da, Ezarpenak → Audio-irteerak, Aurreratua atalean (0tik
2000 ms-ra).

**Beste iturri batek gailua behar duenean.** Ezarpenak → Audio-irteerak
ataleko **Beste iturri batek DSD irteera bat behar duenean** aukerak
zehazten du zer gertatzen den beste erreproduzitzaile bat, kartutxo bat edo
proba-tonu bat gailu berean hasten denean (erreproduzitzailearen beraren
faderra mugitzea da salbuespena: beti aldatzen du pista PCMra):
- **Jarraitu DSD pista PCM gisa** (lehenetsia). Fluxua PCMra aldatzen da
  DSD isiltasunaren ondoren, eta pistak aurrera jarraitzen du, bihurtuta,
  zegoen lekutik. Gauza bera gertatzen zaio berez jarraitzen duen pistari
  (ikus behean).
- **Mantendu DSDa eta isilarazi beste iturriak.** Ezerk ez du DSD fluxua
  eteten. Gailura bideratutako beste iturriak isilarazita daude DSD pista
  amaitu arte, eta erreproduzitzaileak **Besteak isilik** ikurra erakusten
  du bitartean. Erreproduzitzailearen beraren hurrengo pista ez da
  gainjartzen: DSD pista amaitzean hasten da, nahasketa gurutzaturik edo
  segue-rik gabe. PCM pista batek DSD isiltasunaren zain egoten da; mota eta
  DSD maiztasun bereko DSD pista batek fluxua jarraitzen du isiltasunik
  gabe. Faderra mugitzeak pista PCMra aldatzen jarraitzen du.

**Album bat ez da DSD gisa geratzen ezarpen lehenetsiarekin.** *Jarraitu
DSD pista PCM gisa* aukerarekin, gailu inaktibo batean hasten den DSD pista
bat bakarrik ateratzen da DSD gisa. Erreproduzitzaileak ondoren berez
hasten dituen pistak (pista baten amaieran, segue batean edo nahasketa
gurutzatu batean) aurrekarga batetik hasten dira, beti PCM dena; beraz,
gailua PCMra aldatzen da eta bihurtuta jotzen dira. Zuk zeuk hasten duzun
pista bat (Play, klik bikoitza) DSD gisa ateratzen da berriro gailua
inaktibo dagoenean edo aurreko DSD fluxua DSD maiztasun bereko bere
isiltasunean dagoenean oraindik. DSD album oso bat DSD gisa mantentzeko,
aukeratu *Mantendu DSDa eta isilarazi beste iturriak*. Orduan
erreproduzitzailearen pista bakoitza DSD gisa ateratzen da, eta hurrengoa
aurrekoa amaitzean hasten da.

DSDa jotzen ari dela gailua galtzen bada eta hura eramateko gai ez dela
itzultzen bada (adibidez, sarbide esklusiborik gabe), pistak PCM gisa
jarraitzen du.

## BP ikurra {#the-bp-badge}

Erreproduzitzailearen goiburuko **BP** ikurra piztu egiten da uneko pista
bere Main gailura aldaketarik gabe iristen den bitartean. Honako hauek
guztiak bete behar dira:

- gailua bit perfect da eta sarbide esklusiboarekin irekita dago;
- gailua pistaren lagintze-maiztasunean dabil;
- pista galerarik gabeko osoko PCM da (WAV, AIFF, FLAC, ALAC, WavPack edo
  Monkey's Audio), mono edo estereo, eta gehienez 24 bitekoa, eta gailuaren
  formatuak bere lagin-tamaina hartzen du (16 biteko gailu batean 24 biteko
  fitxategi bat ez da bit perfect). DSDa bihurtu egiten da, beraz ez du
  inoiz BP pizten; aldaketarik gabe ateratzen denean (ikus [DSD](#dsd))
  ikurrak **DSD** dio horren ordez;
- pista aztertuta dago, horrela ezagutzen baitira bere maiztasuna eta
  lagin-tamaina. Aurreko bertsio batek aztertutako pistek beren formatua
  jasotzen dute berriro aztertzen direnean (abiarazteko oharra, edo
  Ezarpenak → Analisia), edo erreproduzitzaile batek erakusten dituenean
  edo kartutxo batek hartzen dituenean;
- bolumena % 100 da, ez dago itzaltzerik abian, eta irteera berberetan ez
  da beste ezer jotzen.

Fitxategi batzuk ez dira inoiz bit perfect gisa erakusten:
- **Galeradun fitxategiak** (MP3, AAC, Ogg Vorbis, Opus): beren lagin
  deskodetuak ez dira gailu batek onartzen dituen osoko balioak.
- **24 bitetik gorako fitxategiak:** nahasgailuak 32 biteko koma
  mugikorrean lan egiten du, eta horrek 24 bit zehazki eramaten ditu.
- **Bi kanal baino gehiagoko fitxategiak:** estereora nahasten dira.

## Zeuk egiaztatzea {#checking-it-yourself}

Kate bat muturretik muturrera egiaztatzeko:

1. Konektatu gailuaren irteera digitala (S/PDIF, AES edo USB loopback)
   bitez bit grabatzen duen grabagailu batera.
2. Jo galerarik gabeko proba-fitxategi bat % 100ean, beste ezer jotzen ari
   ez dela.
3. Grabatu.
4. Konparatu grabazioa fitxategiarekin. Adibidez, SoX-ekin, alderantzikatu
   bat eta nahastu biak: `sox -m -v 1 file.wav -v -1 recording.wav diff.wav`,
   haien hasierak lerrokatu ondoren. Diferentziaren lagin guztiek zero izan
   behar dute.

Proiektuaren proba automatikoek propietate bera egiaztatzen dute
aplikazioaren barruan, simulatutako gailu batean.

### DSDa bihurgailu erreal batean {#dsd-on-a-real-converter}

Proba automatikoek DSDa simulatutako gailuetan bakarrik egiaztatzen dute.
Proiektuak ez ditu DoP eta DSD natiboa bihurgailu erreal batean probatu.
Bat egiaztatzeko:
1. Ezarri gailua **DoP** moduan (edo **DSD natiboa** Linuxen), berrabiarazi
   eta jo DSD fitxategi bat % 100ean, beste ezer jotzen ari ez dela.
   Goiburuak **DSD** erakutsi behar du, eta bihurgailuaren pantailak DSD
   maiztasuna (adibidez DSD64) erakutsi beharko luke, PCM maiztasun baten
   ordez. PCM maiztasun bat erakusten duen edo zarata jotzen duen
   bihurgailu batek ez du fluxua ezagutzen: itzuli **Bihurtu PCM** aukerara.
2. Entzun klik bat edo zarata-zaparrada bat hasieran, Gelditu sakatzean,
   pistaren amaieran eta faderra mugitzean. Klik batek esan nahi du
   bihurgailuak **DSD isiltasuna** luzeagoa behar duela (Ezarpenak →
   Audio-irteerak, Aurreratua).
3. Hasi kartutxo bat edo beste erreproduzitzaile bat gailu berean, behin
   nahasketa-ezarpen bakoitzarekin, eta egiaztatu goian deskribatutako
   portaera.
4. Linuxen, DSD natiboa aplikaziorik gabe egiaztatzeko, exekutatu
   `FAUSTE_NATIVE_DSD_DEVICE=alsa:hw:CARD=<card>,DEV=0 cargo test -p fp-backends --test conformance -- --ignored native_dsd_on_a_real_device`.
   Gailua DSD natiboan irekitzen du DSD64an eta segundo bateko DSD
   isiltasuna jotzen du. Gainditu egin behar du, eta bihurgailua DSD64ra
   sinkronizatu beharko litzateke.
5. `test-music/` karpetan DSD fitxategiak daudela,
   `cargo test --release -p fp-engine --test dsd_real_music -- --ignored`
   komandoak motorraren bidez jotzen ditu simulatutako gailu batean eta
   hitzak fitxategiaren byteekin konparatzen ditu (ikus [Probak](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/testing.md)).
