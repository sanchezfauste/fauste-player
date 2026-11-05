# Bit perfect irteera

**Bit perfect** gailu batek fitxategi bakoitzaren laginak fitxategian dauden
bezala jasotzen ditu: lagintze-maiztasun bera, balio berberak, birlaginketarik,
bolumen-aldaketarik edo nahasketarik gabe.

Bit perfect irteera **audiofiloen erreprodukziorako modu berezi bat da**:
artxibo bati entzutea, hi-fi edo monitorizazio-kate bat elikatzea, lotura
digital bat egiaztatzea, ordenagailuko edozein prozesamendu saihestu behar den
edonon. Berez desaktibatuta dago, eta ez dago musika-programa nahastu
baterako pentsatuta. Irakurri [Zer kostatzen den](#what-it-costs) aktibatu
aurretik.

## Zer kostatzen den {#what-it-costs}

Pista guztiak ukitu gabe mantentzeko, Main irteera bit perfect gailu batean
duen erreproduzitzaile batek besteek baino modu desberdinean jotzen du:

- **Gurutzaketarik eta segue-rik ez.** Pista bakoitza bere amaierara arte
  jotzen da (bere cue-out puntura, edo fitxategiaren amaierara **Erabili
  cue-in eta cue-out** desaktibatuta dagoenean), eta gero hurrengoa hasten da.
  **Nahasketa automatikoa MIX puntuan** eta MIX puntua ez dira erabiltzen:
  MIX markatzailea ilunduta marrazten da eta **Nahasten** ikurra ez da inoiz
  pizten. Pista bat airean dagoela **Play / Hurrengoa** eta **Aurreko pista**
  sakatzeak berehala aldatzen du, klikik gabeko gelditze labur batekin,
  itzali ordez.
- **Isilune labur bat lagintze-maiztasuna aldatzen denean.** Lagintze-maiztasun
  desberdineko bi pistaren artean, gailuan beste ezer entzuten ez denean,
  gailuak lehenik bufferrean oraindik duena jotzen du (pistaren amaiera), eta
  gero maiztasun berrian berriro irekitzen da, eta horrek isilune labur bat
  uzten du (normalean milisegundo hamarkada batzuk, gailu batzuetan gehiago).
  Maiztasun bereko pistak tarterik eta gainjartzerik gabe jarraitzen dute
  elkarri. Maiztasuna jarrai ezin denean (ikus [Zer gertatzen den bit
  perfect gailu batean](#what-happens-on-a-bit-perfect-device)), hurrengo
  pista berehala hasten da, gailuaren maiztasunera bihurtuta.
- **Ozentasun-normalizaziorik ez.** Bit perfect gailu batean dagoen pista
  batek beti irabazi unitarioarekin jotzen du,
  [ozentasun-normalizazioa](loudness.md#bit-perfect-outputs) aktibo egonda
  ere: goiburuak **0.0 dB** dio anbarrez, eta bere argibideak zergatik dioen.
  Beste erreproduzitzaile eta kartutxoak ohi bezala normalizatzen dira.
- **Bolumena % 100ean finkatuta dago.** Erreproduzitzailearen bolumen-faderra
  goialdean blokeatuta dago: pantailan, [MIDI](midi.md) kontrolagailuetan eta
  [urruneko APIaren eta OSCren](remote-control.md) bidez. Gailuan bit perfect
  aktibatzean % 100era doa, eta bertan geratzen da desaktibatzean.
- **Funtzionatzen jarraitzen duena:** **Itzaltzearekin gelditu** (itzaltze-denboran
  itzaltzen da), **Pausatu**, **Gelditu**, **Berrabiarazi**, errepikapen- eta
  gelditze-markak, Single eta Continuous moduak, Cue (aurrez entzutea, faderrik
  ez duena eta eragiten ez diona), kartutxoak, [DSD irteera](#dsd), eta Main
  irteera beste gailu batean duten erreproduzitzaile guztiak, ohi bezala
  nahasten direnak. Kartutxo bat gailu berean jo daiteke: entzuten den
  bitartean, erreproduzitzailearen hurrengo pista gailuaren uneko
  maiztasunean hasten da, bihurtuta (beraz ez da bit perfect), eta ez da
  isiluneik txertatzen.
- **Noiz erabili.** Aktibatu pista guztiek bihurgailura ukitu gabe iritsi behar
  dutenean. Mantendu desaktibatuta trantsizio nahastuak edo maila-aldaketak
  behar dituen aireko programa batean. Modua gailuko ezartzen da, beraz
  erreproduzitzaile nahastuek beste gailu batean jarrai dezakete.

## Gailu bat bit perfect bihurtzea {#setting-a-device-bit-perfect}

1. **Ezarpenak → Audio-irteerak** atalean, aukeratu gailua esplizituki
   erreproduzitzaile baten Main irteerarako (edo kartutxo-panelarenerako).
   Sistemaren lehenetsian utzitako erreproduzitzaile bat ezin da bit
   perfect bihurtu. Irteera batek ere erabiltzen ez duen gailu batek bere
   bit perfect etengailua eta DSD modua galtzen ditu aplikazioa hurrengoz
   abiarazten denean.
2. Aukeratu **Aurreratua** atalaren goialdean. **Gailuko ezarpenak**
   azpian, aktibatu **Bit perfect** gailuaren ondoan.
3. Gailua bit perfect bihurtzen da bertan ezer jotzen ez denean berehala (ikus
   [Zain dauden ezarpenak](settings.md#settings-pending)).

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
- **Maiztasunak fitxategia jarraitzen du.** Gailua pista baten lagintze-maiztasunean
  berriro irekitzen da pista hasten denean:
  - **Isiltasunetik:** pista bat jotzen duzunean, pausan kargatutako bat
    berriro abiarazten duzunean, aurrez entzuten duzunean, edo kartutxo bat
    jaurtitzen duzunean gailuan ezer jotzen ez den bitartean. Zain baino ez
    dauden pistak (erreproduzitzaile bakoitzaren hurrengo pista) berriro
    prestatzen dira maiztasun berrian.
  - **Erreproduzitzaile baten bi pistaren artean:** airean dagoen pista
    gelditzen da (bere amaieran, edo berehala **Play / Hurrengoa** eta
    **Aurreko pista** sakatzean), gailua berriro irekitzen da eta hurrengo
    pista hasten da. [Zer kostatzen den](#what-it-costs) ataleko isilune
    laburra da.
  - Gailua bufferrean zuen guztia jo ondoren bakarrik irekitzen da berriro,
    beraz aurreko pistaren amaiera ez da inoiz moztuta geratzen. Gero berriro
    irekitzeak gailuak abiarazteko behar duen beste denbora hartzen du.
    Elkarrekin, normalean milisegundo hamarkada batzuk dira; hasiera
    hainbeste atzeratzen da. Bufferra itxaroteko denbora ez da inoiz
    erdi segundo baino luzeagoa (`tuning.play_out_max_ms`
    konfigurazio-fitxategian, 0tik 2000 ms-ra), gailuak dioena edozein dela
    ere.
- **Maiztasuna jarrai ezin denean.** Gailuak bere maiztasuna mantentzen du,
  eta beste maiztasun bateko pista bat bere iraupen osoan bihurtzen da (ez da
  bit perfect), honako kasuetan:
  - beste zerbaitek jotzen du gailuan pista hasten den unean: beste
    erreproduzitzaile batek (edo pista batekin pausan dagoen batek), kartutxo
    batek edo aurrez entzute batek;
  - gailuak maiztasun hori lehenago ukatu zuen; aurrekoa mantentzen du;
  - pista ez dago oraindik aztertuta, beraz bere maiztasuna ez da ezagutzen.

  Bi pistaren arteko aldaketa gurutzaketarik gabe egiten da hala ere.
- **Prozesamendurik ez, ezerk eskatzen ez duenean.** Laginak aldaketarik
  gabe igarotzen dira itzaltzerik abian ez dagoen bitartean (Itzaltzearekin
  gelditu, pausatu eta jarraitzeko arrapalak, pista baten klikik gabeko
  gelditze laburra) eta irteera berberetan ez den beste ezer jotzen ari ez
  denean (beste erreproduzitzaile bat, kartutxo bat, proba-tonu bat).
  Bolumena ez da kontua: bit perfect irteera batek % 100ean mantentzen du.

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
isiltasuna aldatzea gailuan ezer jotzen ez denean aplikatzen da, beste
irteera-ezarpenak bezala.

**DSDa aldaketarik gabe noiz ateratzen den.** Honako hauek guztiak bete
behar dira pista hasten denean:
- gailua bit perfect da, sarbide esklusiboarekin, eta bere modua DoP edo
  DSD natiboa da;
- pista DSD da (DSF edo DFF), mono edo estereo, eta aztertuta dago (horrela
  ezagutzen da bere DSD maiztasuna);
- gailuan ez da beste ezer jotzen (beste erreproduzitzaile bat, kartutxo
  bat, proba-tonu bat);
- pistak ez du [ozentasun-normalizazioaren](loudness.md#bit-perfect-outputs)
  irabazirik: gailua bit perfect bihurtu aurretik irabazi batekin kargatutako
  (adibidez, pausan) pista batek mantentzen du, beraz bihurtuta jotzen du,
  **DSD** edo **BP** gabe;
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
- Gelditu eta itzaltzearekin gelditzeak pista berehala gelditzen dute,
  itzaltzerik gabe, DSD fluxu bat ezin baita itzali;
- pausatzeak eta jarraitzeak ere berehala eragiten dute, arrapalarik gabe.

**Isiltasuna ertzetan.** Hasiera, amaiera eta PCMrako aldaketa bakoitzak
DSD isiltasuna bidaltzen du lehenik (lehenespenez 200 ms), bihurgailua
klikik gabe sinkroniza dadin. Salbuespena da mota eta DSD maiztasun
bereko fluxu bat jarraitzen duen DSD pista bat, haren isiltasuna oraindik
abian dagoela: bihurgailua oraindik sinkronizatuta dago, beraz isiltasun
gehigarririk gabe hasten da. Beraz, pista bat hainbeste atzerago hasten da,
eta PCMrako aldaketa batek luzera horretako tarte bat uzten du (DSD natiboko
gailu batek bufferrean duena ere jotzen du PCM gisa berriro ireki aurretik,
milisegundo batzuk gehiago). **DSD isiltasuna** da, Ezarpenak →
Audio-irteerak, Aurreratua atalean (0tik 2000 ms-ra).

**Beste iturri batek gailua behar duenean.** Ezarpenak → Audio-irteerak
ataleko **Beste iturri batek DSD irteera bat behar duenean** aukerak
zehazten du zer gertatzen den beste erreproduzitzaile bat, kartutxo bat edo
proba-tonu bat gailu berean hasten denean:
- **Jarraitu DSD pista PCM gisa** (lehenetsia). Fluxua PCMra aldatzen da
  DSD isiltasunaren ondoren, eta pistak aurrera jarraitzen du, bihurtuta,
  zegoen lekutik. Gauza bera gertatzen zaio berez jarraitzen duen pistari
  (ikus behean).
- **Mantendu DSDa eta isilarazi beste iturriak.** Ezerk ez du DSD fluxua
  eteten. Gailura bideratutako beste iturriak isilarazita daude DSD pista
  amaitu arte, eta erreproduzitzaileak **Besteak isilik** ikurra erakusten
  du bitartean. Erreproduzitzailearen beraren hurrengo pista DSD pista
  amaitzean hasten da: PCM pista bat DSD isiltasunaren ondoren, mota eta DSD
  maiztasun bereko DSD pista bat berehala, fluxua jarraituz.

**Album bat ez da DSD gisa geratzen ezarpen lehenetsiarekin.** *Jarraitu
DSD pista PCM gisa* aukerarekin, gailu inaktibo batean hasten den DSD pista
bat bakarrik ateratzen da DSD gisa. Erreproduzitzaileak ondoren berez
hasten dituen pistak (pista baten amaieran) aurrekarga batetik hasten dira,
beti PCM dena; beraz, gailua PCMra aldatzen da eta bihurtuta jotzen dira.
Zuk zeuk hasten duzun pista bat (Play, klik bikoitza) DSD gisa ateratzen da
berriro gailua inaktibo dagoenean edo aurreko DSD fluxua DSD maiztasun
bereko bere isiltasunean dagoenean oraindik. DSD album oso bat DSD gisa
mantentzeko, aukeratu *Mantendu DSDa eta isilarazi beste iturriak*. Orduan
erreproduzitzailearen pista bakoitza DSD gisa ateratzen da, eta hurrengoa
aurrekoa amaitzean hasten da.

DSDa jotzen ari dela gailua galtzen bada eta hura eramateko gai ez dela
itzultzen bada (adibidez, sarbide esklusiborik gabe), pistak PCM gisa
jarraitzen du.

## BP ikurra {#the-bp-badge}

Erreproduzitzailearen goiburuko **BP** ikurra Main gailua bit perfect dabilen
erreproduzitzaile batean bakarrik dago. Piztu egiten da (ertz eta testu
berdea; gris ilun itzalita dagoenean) uneko pista bere Main gailura
aldaketarik gabe iristen den bitartean. Honako hauek guztiak bete behar dira:

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
- pistak irabazi unitarioarekin jotzen du: bit perfect irteera bat ez da
  inoiz normalizatzen, baina gailua bit perfect bihurtu aurretik
  normalizazio-irabazi batekin hasi zen pista batek irabazi hori mantentzen
  du gelditu arte, eta BP itzalita geratzen da harentzat;
- bolumena % 100ean dago bit perfect irteera batean, ez dago itzaltzerik
  abian (Itzaltzearekin gelditu, pausatu eta jarraitzeko arrapalak eta pista
  baten klikik gabeko gelditze laburra dira erreproduzitzaile horretan
  geratzen diren itzaltze bakarrak), eta irteera berberetan ez da beste
  ezer jotzen.

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
2. Jo galerarik gabeko proba-fitxategi bat, beste ezer jotzen ari ez dela.
3. Grabatu.
4. Konparatu grabazioa fitxategiarekin. Adibidez, SoX-ekin, alderantzikatu
   bat eta nahastu biak: `sox -m -v 1 file.wav -v -1 recording.wav diff.wav`,
   haien hasierak lerrokatu ondoren. Diferentziaren lagin guztiek zero izan
   behar dute.
5. Egiaztatu maiztasun-aldaketa bat: Continuous moduan, **Nahasketa automatikoa
   MIX puntuan** aktibatuta, jo MIX puntua duen 44,1 kHz-eko pista bat eta
   ondoren 96 kHz-eko bat. Lehenengoak bere cue-out punturaino jotzen du
   itzaltzerik gabe; isilune labur baten ondoren, bihurgailuak 96 kHz
   erakusten du bigarrenean. BP ikurra bietan pizten da.
6. Gero jo maiztasun bereko bi pista: bigarrenak lehenengoari tarterik eta
   gainjartzerik gabe jarraitzen dio.

Proiektuaren proba automatikoek propietate bera egiaztatzen dute
aplikazioaren barruan, simulatutako gailu batean.

### DSDa bihurgailu erreal batean {#dsd-on-a-real-converter}

Proba automatikoek DSDa simulatutako gailuetan bakarrik egiaztatzen dute.
Proiektuak ez ditu DoP eta DSD natiboa bihurgailu erreal batean probatu.
Bat egiaztatzeko:
1. Ezarri gailua **DoP** moduan (edo **DSD natiboa** Linuxen) eta jo DSD
   fitxategi bat, beste ezer jotzen ari ez dela. Goiburuak **DSD** erakutsi
   behar du, eta bihurgailuaren pantailak DSD maiztasuna (adibidez DSD64)
   erakutsi beharko luke, PCM maiztasun baten ordez. PCM maiztasun bat
   erakusten duen edo zarata jotzen duen bihurgailu batek ez du fluxua
   ezagutzen: itzuli **Bihurtu PCM** aukerara.
2. Entzun klik bat edo zarata-zaparrada bat hasieran, Gelditu sakatzean,
   pistaren amaieran. Klik batek esan nahi du bihurgailuak **DSD isiltasuna**
   luzeagoa behar duela (Ezarpenak → Audio-irteerak, Aurreratua).
3. Hasi kartutxo bat edo beste erreproduzitzaile bat gailu berean, behin
   nahasketa-ezarpen bakoitzarekin, eta egiaztatu goian deskribatutako
   portaera.
4. Linuxen, DSD natiboa aplikaziorik gabe egiaztatzeko, exekutatu
   `FAUSTE_NATIVE_DSD_DEVICE=alsa:hw:CARD=<card>,DEV=0 cargo test -p fp-backends --test it -- --ignored conformance::native_dsd_on_a_real_device`.
   Gailua DSD natiboan irekitzen du DSD64an eta segundo bateko DSD
   isiltasuna jotzen du. Gainditu egin behar du, eta bihurgailua DSD64ra
   sinkronizatu beharko litzateke.
5. `test-music/` karpetan DSD fitxategiak daudela,
   `cargo test --release -p fp-engine --test it dsd_real_music:: -- --ignored`
   komandoak motorraren bidez jotzen ditu simulatutako gailu batean eta
   hitzak fitxategiaren byteekin konparatzen ditu (ikus [Probak](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/testing.md)).
