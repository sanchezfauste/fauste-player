# Erreproduzitzaileak

Zutabe bakoitza erreproduzitzaile bat da. Erreproduzitzaileak independenteak
dira: bakoitzak bere zerrenda-fitxak, garraio-kontrolak, bolumena eta
irteerak ditu.

![1. erreproduzitzailea airean: goiburua, azala, izenburua, hurrengo pista, garraioa, atzerako kontaketa, neurgailua, faderra eta uhin-forma](../../images/guide/player.png)

## Goiburua {#header}

| Elementua | Esanahia |
|---|---|
| `P1` … `Pn` | Erreproduzitzailearen zenbakia (hura erreproduzitzen duen zenbaki-tekla) |
| Egoera-puntua eta etiketa | **Airean** (gorria), **Pausan** (anbarra), **Geldituta** (grisa) |
| **Nahasten** / **Itzaltzen** ikurra | Hurrengo pistarako nahasketa gurutzatu bat edo itzaltzearekin gelditze bat abian dago |
| **Gelditu amaieran** ikurra | Erreproduzitzailea gelditu egingo da uneko pista amaitzean |
| **Errepikatu** / **Gelditu pistaren ondoren** ikurra | Uneko pista errepikatu egiten da, edo erreproduzitzailea gelditzen du amaitzean, zerrendaren menuko bere markagatik. Pasatu sagua gainetik esaldi osoa ikusteko. Erreproduzitzailearen beraren **Gelditu amaieran** botoiak irabazten du: aktibo dagoen bitartean, haren ikurra bakarrik agertzen da |
| **BP** / **DSD** | **BP** piztuta dago uneko pista bere Main gailura aldaketarik gabe iristen den bitartean. **DSD** agertzen da haren ordez DSD pista bat DSD gisa ateratzen den bitartean, aldaketarik gabe (ikus [Bit perfect irteera](bit-perfect.md)). **Besteak isilik** agertzen da ondoan DSD fluxu horrek beste iturriak irteeratik kanpo uzten dituenean |
| **SINGLE** \| **CONT** | Erreprodukzio-modua (ikus behean): kontrol bateratu bat; piztutako erdia da modu aktiboa |
| **CUE** | Aurrez entzun hurrengo pista CUE irteeran |

## Informazio-errenkada {#info-row}

- Airean dagoen pistaren **azala**, edo binilo baten irudia ordezko gisa.
- Airean dagoen pistaren **izenburua eta artista**. Geldituta dagoen
  erreproduzitzaile batek Play-k abiaraziko duen pista erakusten du (bere
  hurrengoa), bere azalarekin, iraupenarekin eta uhin-formarekin, cue-in
  puntuan prest, edo bere uhin-forman klik egin duzun lekuan.
- Analisia amaitu aurretik erreproduzitutako edo kargatutako pista batek
  badu jada bere iraupena, fitxategiaren goiburuak ematen duenean: atzerako
  kontaketak, `igarotakoa / guztira` eta klik eginez jauzi egiteak berehala
  funtzionatzen dute, lerro lau baten gainean uhin-forma prest egon arte.
  Goiburuak gordetzen ez duenean (AAC fitxategi gordinak, iraupen-framerik
  gabeko MP3 fitxategiak, Matroska eta WebM), guztizkoak "—" erakusten du
  eta ezin da uhin-forman klik egin analisia amaitu arte.
- **Neurgailu estereoa** (erreproduzitzailearen eskuineko zutabean,
  faderraren ondoan; informazio-errenkada eta garraioa hartzen ditu):
  erreproduzitzaileak ateratzen duen maila, bere bolumenaren ondoren.
  - Barra jarraitu bat kanal bakoitzeko, neurgailu motaren arauaren
    eskalan (lehenespenez gailur digitaleko neurgailua: −60tik 0 dBFSra,
    goiko 20 dB-ek altueraren erdia hartzen dutela). Eskala erregela bat
    da barren alde bakoitzean, neurgailuaren beraren unitateetan etiketatua
    bi aldeetan. Goialdea eta behealdea (neurgailu digitalean, eskalaren
    behealdea) beti markatuta daude; etiketa bakoitzak marra bat du
    erregela bakoitzean, eta etiketen arteko marra laburragoek
    neurketa-erregela batekoek bezala funtzionatzen dute: balio borobiletan tarte
    berdinetan, dB bakoitzean EBU neurgailuan eta 5 dB-tik behin −20tik
    behera digitalean neurgailua behar bezain altua denean, eta 2, 2,5, 5
    edo 10 dB-tik behin (edo bat ere ez) haientzat laburregia den lekuan.
    Neurgailu digitalean (eta pertsonalizatuan), neurgailu altu batek balio
    gehiago etiketatzen ditu: dB bakoitzean −20tik 0ra, eta 5 dB-tik behin
    −20tik beherako 10 dB-ko marken artean (−45, −55), beti tarte berdinetan
    eta etiketak elkar ukitu gabe sartzen diren lekuetan bakarrik; beste
    neurgailu motek beren arauak ematen dituen etiketak mantentzen dituzte.
    Lerrokatze-maila (−18 dBFS neurgailu digitalean) marra zuri lodiago bat
    da bi erregeletan. Ez da ezer marrazten barren gainean ezta haien
    artean ere, beraz barretan ikusten duzuna maila, gailurrari eustea eta
    koloreak baino ez dira.
  - Barra berdea da, horia abisu-mailatik aurrera (−9 dBFS), eta gorria
    arrisku-mailatik aurrera (−3 dBFS). Beste neurgailu motak beren eskalak
    gorri jartzen duen lekuan jartzen dira gorri (0 VU-tik aurrera,
    baimendutako gehienekotik aurrera PPM batean).
  - K-System neurgailuek bi atal erakusten dituzte: barra betea batez
    besteko maila (RMS) da, eta haren gaineko zati ilunagoa gailurreraino
    iristen da. Haien koloreak K-Systemenak dira: berdea 0tik behera,
    anbarra 0tik +4ra, gorria hortik gora.
  - Gailur digitaleko, K-System eta neurgailu pertsonalizatuek mailarik
    altuena piztuta mantentzen dute une batez lerro gisa (gailurrari
    eustea; bere iraupena **Gailurrari eutsi** da
    [Ezarpenak → Neurgailuak](settings.md#meters) atalean, eta 0 balioak
    desaktibatzen du). EBU PPM, DIN PPM eta VU neurgailuek ez dute
    eusterik.
  - Goiko zenbakia sarrera hasi zenetik izandako mailarik altuena da,
    dBFS-tan, gorriz arrisku-eremuan. Gelditu ondoren mantendu egiten da,
    eta berriro hasten da sarrera bat jotzen denean (hurrengoa edo bera
    berriro), edo haren gainean klik egiten duzunean.
  - Azpiko zenbakia ozentasuna da LUFS-etan (EBU R128), berdea helburuaren
    ±1 LU-ren barruan (−23 LUFS).
  - Neurgailu mota eta maila guztiak
    [Ezarpenak → Neurgailuak](settings.md#meters) atalean alda daitezke.
  - **0 dBFS-tik gorako irakurketak.** Neurgailuak erreproduzitzaileak
    ateratzen duena erakusten du, eta hori eskala osotik gorakoa izan
    daiteke. Barra eskalaren goialdean gelditzen da, beraz gorri berak
    erakusten ditu 0 dBFS eta hortik gorako guztia; goiko zenbakiak bakarrik
    adierazten du zenbateraino, bere zeinuarekin (adibidez `+3.5`).
    - Fitxategi batek berak eskala osotik gorako mailak izan ditzake (koma
      mugikorreko fitxategi bat, edo galeradun fitxategi bat, deskodetutako
      gailurrak eskala osotik gorakoak dituena).
    - Maiztasuna bihurtzeak laginen arteko gailurrak sor ditzake: 0 dBFS
      ukitzen duen seinale batek +3 dBFS inguru irakurtzen du 44,1 → 48 kHz
      bihurketaren ondoren. True peak aukerak gailur horiek ere irakurtzen
      ditu.
    - Neurgailuak erreproduzitzaile bakoitza bakarrik irakurtzen du, ez
      gailuko batura: irteera bakarreko bi erreproduzitzailek eskala osotik
      gora batu dezakete, bi neurgailuetako batek ere erakutsi gabe.
    - Erreproduzitzaileko ezerk ez du irabazirik gehitzen % 100etik gora.
      Osoko zenbakizko gailu batek eskala osoan mozten du; koma mugikorreko
      gailu batek maila dagoen bezala jasotzen du, eta soinu-sistemak edo
      kontrolatzaileak mozten du.
- **Bolumen-faderra** (neurgailuaren eskuinean, bera bezain altua):
  arrastatu edo erabili saguaren gurpila, urrats bat koska bakoitzeko.
  Argibideak maila dB-tan erakusten du; goialdea 0 dB da eta behealdea
  isiltasuna.
- **Izenburua, artista** eta **hurrengo** lerroa, lauki berde batekin. CUEa
  aktibo dagoen bitartean, aurrez entzutearen posizioa urdinez agertzen da.

## Garraioa {#transport}

| Botoia | Ekintza |
|---|---|
| **Play / Hurrengoa** (handia) | Geldituta: hurrengo pista abiarazten du. Airean: hurrengo pistara pasatzen da itzaltzearekin (itzaltze-denbora [Ezarpenak](settings.md) atalean ezartzen da). Pausan: jarraitzen du. Airean dagoen pista hurrengo gisa dagoela, Play-k pista hori berriro hasten du ohiko itzaltzearekin. |
| **Gelditu** | Berehala gelditzen da (klikak saihesteko arrapala labur batekin) |
| **Itzaltzearekin gelditu** | Bolumena itzaltzen du eta gelditzen da |
| **Pausatu** | Pausatu edo jarraitu; anbarrez keinuka pausan dagoen bitartean |
| **Gelditu uneko pistaren ondoren** (play triangelu bat eta lauki bat) | Uneko pista amaitzean gelditzen da, behin. SINGLE moduan uneko pista errepikatzen den bitartean bakarrik dago erabilgarri: errepikapena amaitzen du jotzen ari den itzulia amaitzean. Pista bat jotzen den bakoitzean haren ondoren gelditzeko, edo pista bat errepikatzeko, erabili bere menua zerrendan (ikus [Zerrendak](playlists.md)) |
| **Aurreko pista** (barra bat eta bi triangelu) | Airean dagoela: erreproduzitzaile honek lehenago jo zuen pistara itzultzen da itzaltzearekin, Hurrengoak egiten duen bezala. Sakatu berriro atzera egiten jarraitzeko. Utzi duzun pista hurrengoa bihurtzen da. |
| **Berrabiarazi pista** (barra bat eta triangelu bat) | Uneko pistaren hasierara itzultzen da (bere cue-in puntura). Pausan dagoen erreproduzitzaile batek pausan jarraitzen du. |

Une honetan ezer egin ezin duten botoiak ilunduta agertzen dira: Gelditu
eta Berrabiarazi ezer kargatu gabe, Pausatu eta Itzaltzearekin gelditu
erreproduzitzailea geldituta dagoela, eta Aurreko pista aurreko pistarik
gabe edo itzaltze batean zehar. Erreproduzitzaile batek jo dituen azken 50
pistak gogoratzen ditu (`players.history_len` `config.json` fitxategian,
0tik 1000ra).

## Moduak {#modes}

- **CONT (jarraitua):** MIX puntuan erreproduzitzaileak hurrengo pista
  abiarazten du eta unekoaren amaierarekin gainjartzen du. Ikus
  [Markatzaileak eta nahasketa](markers-and-mixing.md).
- **SINGLE:** pista bakoitza bere amaieran gelditzen da. *Gelditu uneko
  pistaren ondoren* ez dago erabilgarri modu honetan, pista bakoitza jada
  gelditzen delako, uneko pista errepikatzen den bitartean izan ezik:
  orduan errepikapena amaitzen du jotzen ari den itzulia amaitzean.

## Atzerako kontaketa {#countdown}

Zenbaki handia pistaren amaierara (bere cue-out puntura) falta den denbora
da, hamarrenekin. Igarotako denbora eta guztizkoa uhin-formaren azpiko
errenkadan daude, eskuinean. Amaieraren aurreko azken segundoetan
(lehenespenez 10, Ezarpenetan ezartzen da) atzerako kontaketa gorriz
keinuka hasten da.

## Uhin-forma {#waveform}

- Jada jo den zatia uhin-formaren kolorean marrazten da; gainerakoa
  ilunagoa da.
- Ingeradak gailurrak erakusten ditu, ahul; barruko gorputz beteak batez
  besteko maila (RMS) erakusten du. Pista ozen batean gailurrek altuera
  osoa betetzen dute, eta gorputzak oraindik erakusten du pista non den
  isilagoa edo ozenagoa.
- Hasierako eremu itzaldun urdin batek **introa** markatzen du, eta ikur
  batek atzerako kontaketa egiten du. Introa ezarri denean bakarrik
  agertzen da.
- Amaierako eremu itzaldun laranja batek **outroa** markatzen du, bere
  atzerako kontaketarekin.
- **MIX** etiketa duen marra eten anbar batek modu jarraituan hurrengo pista
  non hasten den markatzen du. Ilunduta agertzen da modu bakarrean.
- Fitxategi osoa marrazten da. Erreprodukzioak saltatzen dituen hasiera eta
  amaiera isilak (cue-in aurretik eta cue-out ondoren) ilunago marrazten
  dira, erreprodukzioa hasten eta amaitzen den lekuan marra fin batekin.
  **Erabili cue-in eta cue-out** desaktibatuta dagoenean (Ezarpenak →
  Erreproduzitzaileak) ezer ez da ilunagoa eta bi marrak ilunduta daude:
  erreprodukzioa fitxategiaren hasieratik amaierara doa, eta gida honetako
  cue-in eta cue-out bi mutur horiek dira.
- Pasatu sagua gainetik erakuslearen azpiko denbora ikusteko. **Egin klik
  jauzi egiteko** hara. Geldituta dagoen erreproduzitzaile batean, klik
  batek **Play**-k hurrengo pista non abiaraziko duen aukeratzen du:
  erreprodukzio-buruak eta atzerako kontaketak hara mugitzen dira, eta ez da
  ezer entzuten Play sakatu arte. Beste hurrengo pista bat aukeratzeak,
  hura mugitzeak edo kentzeak, edo Gelditu sakatzeak cue-in puntura
  itzultzen du; baita pista bat abiarazteko beste edozein modutan ere, eta
  Berrabiarazi pistak, Aurreko pistak eta aurrerapen automatikoak beti
  cue-in erabiltzen dute. Cue-in aurreko klik batek (hasiera ilunagoan)
  cue-in aukeratzen du. Cue-out puntuan edo ondoren egindako klik batek
  (isats ilunagoan) aurreko aukera bat bertan behera uzten du: Play-k cue-in
  puntuan hasten du. Klik bat sakatze eta askatze bat da, erakuslea pixel
  gutxi batzuk baino gehiago mugitu gabe.
- **Sakatu eta arrastatu** zoom-ikuspegia pistan zehar mugitzen du, heldu
  izan bazenu bezala. Arrastatze batek ez du inoiz jauzirik egiten, eta
  zoomik gabe ez du ezer egiten. Alt-arrastatzeak markatzaileak editatzen
  jarraitzen du.
- **Saguaren gurpila** uhin-formaren gainean: handitu eta txikitu
  erakuslearen inguruan, analisiak duen xehetasunik finenaraino.
  **Shift+gurpila** (edo alboko gurpil batek) pistan zehar mugitzen du.
  Zoomarekin dagoela, ikuspegiak jotzen ari den posizioa jarraitzen du,
  zooma egin edo mugitu ondorengo 10 segundoetan izan ezik
  (`ui.follow_current_grace_secs`; 0 balioak jarraipena desaktibatzen du).
  **Ikuspegi osoa** botoiak, goiko eskuineko izkinan, guztiz txikitzeak edo
  pista berri batek pista osoa erakusten dute berriro.

## CUE (aurrez entzutea) {#cue-pre-listen}

**CUE** sakatzeak (edo pista baten menuko **Aurrez entzun CUEan**) pista
erreproduzitzailearen CUE irteeran jotzen du, adibidez entzungailuetan,
airean dagoen irteera ukitu gabe, eta erreproduzitzaile horren **CUE leiho**
txiki bat irekitzen du. Hainbat leiho egon daitezke irekita, bat
erreproduzitzaile bakoitzeko. Ikus [Ezarpenak](settings.md) CUE gailua
aukeratzeko. Erreproduzitzaile batek bere Main irteera ez den Cue irteera
bat behar du: halakorik gabe, **CUE** eta **Aurrez entzun CUEan** ilunduta
daude, eta sagua gainetik pasatzean hala adierazten da.

![4. erreproduzitzailearen CUE leihoa, bere hurrengo pista aurrez entzuten](../../images/guide/cue-window.png)

Leihoak hau erakusten du:

- izenburua eta artista;
- fitxategi osoaren uhin-forma, CUEaren posizioarekin. Erreproduzitzailearena
  bezala funtzionatzen du: egin klik hara jauzi egiteko, handitu gurpilarekin,
  arrastatu mugitzeko, **Ikuspegi osoa**, introaren eta outroaren atzerako
  kontaketak, eta intro, outro eta MIX markatzaileak, hemen
  erreproduzitzailean bezala editatzen dituzunak (ikus
  [Markatzaileak eta nahasketa](markers-and-mixing.md)). CUE batek fitxategi
  osoa jotzen du, beraz ez da ezer ilunago marrazten, cue-in eta cue-out
  marra ilunduak dira, eta outroak fitxategiaren amaieraraino egiten du
  atzerako kontaketa. Bere zooma berea da: erreproduzitzailearen uhin-forma
  ez da mugitzen. CUEa jotzen ari den bitartean, zoom-ikuspegi batek bere
  posizioa jarraitzen du erreproduzitzailearenak bezala; pausan dagoen CUE
  batek ezarri duzun ikuspegia mantentzen du, zooma egin eta markatzaileak
  jar ditzazun. Bere leihoa itxi ondoren edo beste pista batean
  abiarazitako CUE batek fitxategi osoa erakusten du;
- igarotako denbora eta fitxategiaren amaierara falta den denbora (CUE batek
  fitxategi osoak jotzen ditu);
- **Pausatu CUEa** / **Jarraitu CUEa**, **Gelditu CUEa** eta **Ezarri
  hurrengo gisa**. **Ezarri hurrengo gisa** aukerak CUEan dagoen pista
  erreproduzitzailearen hurrengo bihurtzen du eta CUEak jotzen jarraitzen
  du. Ilunduta dago pista jada hurrengoa denean. CUEan dagoen pista airean
  dagoena bada, beste behin joko da uneko itzulia amaitzean.

CUEa pausan dagoen bitartean, bere **Pausatu** botoia (**Jarraitu** gisa
erakutsia) anbarrez keinuka dago, erreproduzitzailearena bezala.

Pausan dagoen CUE batean egindako jauzi batek pausan mantentzen du. Leihoa
ixteko botoiak, edo **Gelditu CUEa** botoiak, CUEa gelditzen du.

CUE bat abian dagoen bitartean, hurrengo bat ezartzeak (klik bikoitza) edo
errenkada batean klik bakarra egiteak CUEa pista horretara eramaten du, bere
cue-in puntutik; pausan bazegoen, berriro jotzen du. Fitxategia falta duen
edo irakurtezina den pista batek CUEa dagoen lekuan uzten du.
