# Automatizazioa

Automatizazioak estudioan inor ez dagoenean ere emisora airean mantentzen du.
Bertsio honek **musika-betegarria** dakar, musika bere kabuz aukeratzen duen
erreproduzitzailea, zuk ezarritako arauen arabera, eta inoiz zerrenda faltagatik
gelditzen ez dena, eta **Programazioa** zutabea, egunaren ordu-blokeak
exekutatzen dituena, Ezarpenetako txantiloietatik eginak.
Dagoeneko erabiltzen dituzun erreproduzitzaileak (P1 … Pn) ukitu gabe geratzen
dira: zureak izaten jarraitzen dute, prestatzen dituzun eta zuzenean jotzen
dituzun saioetarako.

## Programazioa zutabea {#the-log-column}

**Programazioa** zutabeak egunaren **blokeak** jotzen ditu: hasiera-ordua duten
elementu-taldeak (orduko identifikazio bat, iragarkiak, saio bat). Erakutsi
goiko barrako **Programazioa** lauzarekin, erreproduzitzaile bat erakusten duzun
bezala.

![Programazioa zutabea martxan, Musika-betegarria zutabearen ondoan: egunaren blokeak hasiera-orduaren arabera, bakoitza bere emisoraren identifikazioarekin (erlojuak ordu finkokotzat markatzen du), iragarkiekin eta blokearen ondoren musika-betegarriak jotzen duen errotazioarekin; musika-betegarriaren airean dagoen abestia eta hurrengoak](../../images/guide/automation-columns.png)

Bere zerrendak erreprodukzio-zerrenda baten itxura du, blokeka taldekatuta.
Bloke bakoitza goiburu-errenkada batekin hasten da: bere hasiera-ordua, zenbat
elementu dituen eta zenbat irauten duten, "egun honetarako aldatua" gaur
aldatu baduzu, eta, anbarrez, hurrengo blokean zenbat sartzen den. Gero bere
elementuak datoz, bakoitza hastea aurreikusita dagoen **Ordua**rekin.
Zerrendak gaurkoa erakusten du, atzoko azken blokea eta biharko lehena. Airean
dagoen elementua eta hurrengoa erreproduzitzaile batean bezala markatzen dira;
dagoeneko jo diren elementuak ilunduta daude, eta jo ez zen elementu bat
(**igorri gabea**) marratuta dago. Erloju batek **ordu finkoko** elementu bat
markatzen du, bere orduan bakarrik jotzen dena. **Kategoriako pista** batek
harentzat aukeratutako pista erakusten du hurrengoa bihurtu orduko (edo bere
blokean hurrengoaren ondoren datorrenean), kategoria atzean duela: hori da
jotzen den pista. Kategoriak jo dezakeen pistarik ez badu (ez du bat ere
esleituta, edo haren fitxategiak falta dira), errenkadak anbarrez dio, eta
elementua ez da igorriko. Jotzear dagoenean fitxategia irakurtezina duela
ikusten den elementua saltatu egiten da, eta igorri gabe gisa markatzen da
hura ere. Pistak liburutegian esleitzen zaizkio kategoria bati.
**Musika-betegarriaren** errenkadak noiz arte jotzen duen dio, eta airean
agertzen da musika-betegarria Programazioarentzat jotzen ari den bitartean.
Ordua igaro zaion hurrengo batek ordua anbarrez erakusten du. Arrastatu bi
zutabe-goiburuen arteko marra zutabeen zabalera aldatzeko,
erreprodukzio-zerrenda batean bezala; txantiloi- eta egun-leihoek zabalera
berak erabiltzen dituzte.

### Gaurko programazioa aldatzea {#changing-todays-log}

Programazioa erreprodukzio-zerrenda bat bezala aldatzen da, eta aldaketa egun
horretarako bakarrik da: txantiloia dagoen bezala geratzen da.

- Zutabearen beheko botoiak: **+ Pista…** liburutegia irekitzen du, pistak
  zerrendara arrastatzeko; **+ Kategoriako pista…** eta
  **+ Musika-betegarria (errotazioa)…** zein den galdetzen dute eta hautatu
  duzun errenkadaren azpian gehitzen dute (egin klik errenkada batean hura
  hautatzeko; hautatu bloke-goiburu bat bloke horren hasieran gehitzeko), edo,
  ezer hautatu gabe edo jada alda ezin den errenkada baten azpian, hurrengo
  blokearen amaieran; **+ Ordu-blokea…** ordu bat eskatzen du, lehen ordu librea
  jarrita duela. Zutabea guztientzat estuegia denean, sartzen ez direnak
  **⋯**-ren atzean daude.
- Arrastatu pistak liburutegitik, edo audio-fitxategiak zure
  fitxategi-kudeatzailetik, errenkada batera haren aurretik jartzeko (edo
  atzetik, errenkadaren beheko erdian, erreprodukzio-zerrenda batean bezala:
  horrela joaten dira bloke baten amaierara), eta arrastatu errenkadak egun
  beraren barruan haiek mugitzeko. Gehitzen duzun errenkada hautatuta eta
  ikusgai geratzen da, beraz gehitzen duzun hurrengoa haren azpian doa.
- Egin eskuineko klik errenkada batean **Ordu finkoa** eta **Kendu gaurko
  programaziotik** aukeretarako. Egin klik (edo eskuineko klik) bloke-goiburu
  batean bloke horren amaieran **Pista…**, **Kategoriako pista…** edo
  **Musika-betegarria (errotazioa)…** gehitzeko, edo **Aldatu ordua…**,
  **Kendu blokea** eta, aldatutako bloke batean, **Itzuli txantiloira**
  aukeretarako.
- Bloke-goiburu baten gainean arrastatzen duzun bitartean, azpiko marrak
  adierazten du bloke horren hasieran askatuko duzula; goiko blokearen
  amaieran errenkadak jartzeko, askatu haren azken errenkadaren beheko erdian.
- Jo dena, airean dagoen elementua eta iraganeko egunak ezin dira aldatu.
  Airean dagoen blokeko aldaketa airean dagoen elementuaren ondoren sartzen da
  indarrean.

Programazioa zutabeak erreproduzitzaile baten goiburua eta botoiak ditu, eta
erreproduzitzaile batean bezala dabiltza:

- **Erreproduzitu** botoiak hurrengo gisa markatutako elementua jotzen du eta
  automatizazioa martxan jartzen du: bloke bakoitza bere orduan jotzen da
  orduan, eta musika-betegarriak blokeen artean jotzen du. Hurrengoa erlojuak
  aukeratu duenean eta bere blokea beranduago hasten denean, musikak jotzen du
  ordura arte. Ezer ez da bere kabuz hasten: aplikazioa abiarazten den
  bakoitzean, automatizazioa geldirik dago Erreproduzitu sakatu arte.
- **Pausatu**, **Gelditu** eta **Itzaltzearekin gelditu** botoiek
  Programazioarentzat airean dagoenaren gainean jarduten dute (bere elementua,
  edo blokeen artean musika-betegarriaren abestia) eta automatizazioa
  gelditzen dute. Pausatu ondoren Erreproduzitu sakatzean zegoen tokitik
  jarraitzen du; pausan dagoen bitartean hurrengoa ez da mugitzen, eta
  berrekitean hurrengo blokeak bere ordua mantentzen du. Musikaren bitartean
  Gelditu ondoren Erreproduzitu sakatzean musika jartzen du berriro, eta
  hurrengo blokea bere orduan hasten da.
- **Gelditu amaieran** airean dagoena amaitzen uzten du, eta gero
  automatizazioa gelditzen da. Musika jotzen ari bada eta lehenago bloke baten
  ordua iristen bada, blokea puntual hasten da eta automatizazioa bere lehen
  elementuaren ondoren gelditzen da.
- **Aurreko pista** botoiak ez du ezer egiten Programazioan.
- **SINGLE** moduak Programazioa eskuzko bihurtzen du: Erreproduzitu botoiak
  hurrengo elementua jotzen du, eta haren ondoren gelditzen da; ez da blokerik
  bere kabuz hasten.

Musika-betegarriak blokeen artean jotzen duen bitartean, Programazioaren
goiburuak **Musika-betegarria · *errotazioa*** dio (jotzen duen errotazioa) eta
bere botoiek funtzionatzen jarraitzen dute; abestia, bere neurgailua eta bere
uhin-forma Musika-betegarria zutabean bakarrik daude.

Automatizazioa elementu baten ondoren gelditzen denean, hurrengoa jo zuenaren
azpiko elementuan geratzen da, eta Erreproduzitu botoiak hura jotzen du,
ordua edozein dela ere: zuk aukeratzen duzu. Musikaren bitartean gelditzen
denean, Erreproduzitu botoiak musika jartzen du berriro hurrengo blokera arte,
beste hurrengo bat aukeratu ez baduzu behintzat. Hurrengo zer entzungo den
aldatzeko, egin klik bikoitza elementu batean, edo egin eskuineko klik haren
gainean eta aukeratu **Hurrengo entzun dadila**. **Itzuli erlojura** aukerak,
menu berean, orduak dioena jartzen du hurrengo gisa.

## Txantiloiak eta egunak {#templates-and-days}

Programazioak egun bakoitza **txantiloi** batetik jotzen du: bloke-egun bat.
Instalazio berri batek bat du, "Oinarria", egun guztietarako. Txantiloiak
Ezarpenak → **Automatizazioa** atalean egiten dira, **Txantiloiak eta egunak**
azpian (ez Androiden); Programazioa zutabeak gaurkoa bakarrik aldatzen du,
goian bezala.

![Ezarpenak, Automatizazioa, Txantiloiak eta egunak: Oinarria eta Weekend txantiloiak Editatu…, Txantiloi berria, Bikoiztu, Ezabatu eta izenarekin; aste mota, Oinarria astelehenetik ostiralera eta Weekend larunbatean eta igandean; hilabetea gaur aukeratuta, aldaketak dituzten datak anbarrez, dataren txantiloia eta Editatu eguna…](../../images/guide/settings-plans.png)

- **Txantiloiak.** Aukeratu bat eta sakatu **Editatu…** (edo egin klik
  bikoitza haren gainean) irekitzeko; **Txantiloi berria**, **Bikoiztu** eta
  **Ezabatu** ondoan daude, eta izena azpian.
- **Aste mota.** Asteko egun bakoitzak zein txantiloi erabiltzen duen.
- **Egunak.** Aukeratu data bat egutegian (aldaketak dituzten datak anbarrez
  daude). Data batek bere asteko egunarena ez den beste txantiloi bat erabil
  dezake (ostegun bateko jaiegunak asteburukoa erabil dezake). **Editatu
  eguna…** aukerak data hori irekitzen du, bere blokeak data horretarako
  bakarrik aldatzeko; iraganeko datak ezin dira aldatu.

Txantiloi bat edo data bat bere leihoan irekitzen da, eta Ezarpenak ixten
ditu. Programazioa zutabearen zerrenda bera da, eta berdin aldatzen da: bere
beheko botoiak (Pista…, Kategoriako pista…, Musika-betegarria (errotazioa)…,
Ordu-blokea…, hautatzen duzun errenkadaren azpian gehitzen dutenak),
liburutegitik (edo fitxategi-kudeatzailetik fitxategiak) errenkada batera
arrastatutako pistak, mugitzeko arrastatutako errenkadak, errenkaden eskuineko
klikeko menua eta bloke-goiburuen menua. Ez du inoiz jotzen, eta irekita gera
daiteke liburutegiaren ondoan. Gaurko leihoak jo dena eta jotzen ari dena
erakusten du, Programazioak bezala; egun baten leihoa gauerdian ixten da, egun
hori jada aldatu ezin denean.

![Txantiloi bat bere leihoan irekita: Oinarria txantiloiaren blokeak hasiera-orduaren arabera, bakoitza bere emisoraren identifikazioarekin, bere iragarkiekin eta ondoren musika-betegarriarekin, eta Gehitu behean](../../images/guide/template-popup.png)

Txantiloi bateko **kategoriako pista** batek kategoria horretako pista bat
jotzen du, aldi bakoitzean desberdina. **Musika-betegarria** elementuak
blokearen ondoren musika-betegarriak jotzen duen errotazioa aukeratzen du
(errotazio lehenetsia, blokeak bat ere ez duenean). Bloke baten elementuek
hurrengo blokera arteko denbora baino gehiago irauten dutenean, bere goiburuak
zenbat dioen anbarrez; hala ere horrela utz dezakezu.

Data bateko aldaketa "egun honetarako aldatua" gisa markatzen da bere blokean,
eta **Itzuli txantiloira** aukerak, bloke-goiburuaren menuan, desegiten du.
Txantiloiaren geroko aldaketak data horretan aldatu ez dituzun blokeetara
iristen jarraitzen dute. Txantiloi batean bere bloke bat airean dagoela
egindako aldaketak bloke hori dagoen bezala uzten du gaurko ("egun honetarako
aldatua" erakusten du); aldaketa hurrengo egunetik aurrera iristen zaio.

## Musika-betegarria {#the-music-fill}

Musika-betegarria **Musika-betegarria** zutabea da. Ezkutuan dago erakutsi
arte: egin klik goiko barraren ezkerreko **Musika-betegarria** lauzan. P1-en
ezkerrean dago.

Erreproduzitzaile baten itxura eta funtzionamendua ditu: erloju handia,
uhin-forma, garraio-botoiak, faderra eta pisten zerrenda. Aldea zerrenda da:

- **bere kabuz betetzen da** **errotazio** batetik (jarraian), beti ordu erdi
  inguru aurrera (*aurrerako tartea*, Ezarpenak → Automatizazioa atalean);
- **bere kabuz mozten da**: jotako azken 50 pistak gordetzen ditu eta zaharragoak
  kentzen ditu, beraz asteetan zehar exekuta daiteke;
- programazioko bloke baten aurretik, **berak aukeratutako abestiak** beste
  batzuekin **trukatu** ditzake, geratzen den denborara egokitzen den
  luzerakoekin (ikus behean); zuk gehitutako abestiei ez zaie inoiz ukitzen;
- oraindik **eskuz alda dezakezu**: arrastatu pistak liburutegitik, berrordenatu
  errenkadak, kendu nahi ez duzun abesti bat. Gehitzen duzun errenkada esku
  batekin markatzen da eta ez da inoiz ordezkatzen; kentzen duzun errenkada ez
  da jotzat hartzen.

Bere irteerak Ezarpenak → Audio-irteerak atalean ezartzen dira,
**Musika-betegarria** errenkadan, edozein erreproduzitzaile bezala. Ez du
zenbakirik, laster-teklarik edo MIDI loturarik, eta urruneko kontrolak ez du
ikusten.

Sakatu **Erreproduzitu** hasteko eta **Gelditu amaieran** airean dagoen abestia
amaitu eta gelditzeko. Erreproduzitzaile guztiak bezala, ez da inoiz bere kabuz
hasten, ezta berrabiarazi ondoren ere. Blokeen artean Programazioarentzat
jotzen ari den bitartean, gelditzeak edo pausatzeak automatizazioa ere
gelditzen du, eta bere Gelditu amaieran aukerak automatizazioa gelditzen du
abestia amaitzean; lehenago bloke baten ordua iristen bada, blokeak jotzen du
eta marka kentzen da. Musikaren bitartean Programazioa pausatu ondoren,
Musika-betegarria zutabeko Erreproduzitu botoiak abestia bere kabuz jotzen du:
Programazioaren Erreproduzitu botoiak ez du jada berrekiten, eta bere Gelditu
botoiak ez du jada gelditzen.

Bere zerrendaren gainean, erreproduzitzaile batek bere zerrenda-fitxak dituen
tokian, Musika-betegarria zutabeak jotzen duen **errotazioa** erakusten du.
Aukeratu hor beste bat: ilaran zituen abestiak ordezkatzen dira, zuk
gehitutakoak geratzen dira. Errotazioak jotzeko ezer aurkitzen ez duenean (bere
kategoriak hutsik daude, edo fitxategi guztiak falta dira), barrak hala dio.

## Errotazioak {#rotations}

**Errotazioa** musika-betegarriak txandaka jotzen dituen **urratsen** zerrenda
labur bat da, eta gero berriro lehenengotik. Urrats bakoitzak bere pistak
nondik datozen adierazten du (liburutegiko kategoria bat bere
azpikategoriekin, edo mota bateko pista guztiak) eta aldi bakoitzean zenbat
jotzen dituen, tarte gisa:

| Urratsa | Jotzen du |
|---|---|
| Current hits | 2–3 |
| Station IDs | 1 |
| Gold | 0–1 |

Tarte batekin, emisoraren identifikazio bat bi abestiren ondoren dator aldi
batean eta hiruren ondoren hurrengoan. **0**tik hasten den tarte batek urratsak
txanda batzuk saltatzen uzten ditu: Gold 0–1 izanda, abesti klasiko bat
identifikazioaren ondoren jotzen da batzuetan eta besteetan ez. Gehienezkoa
beti da gutxienez 1. Errotazio bat **lehenetsia** da: instalazio berri batek
bat du, **Musika**, zure musika guztia jotzen duena.

Jo dezakeen pistarik ez duen kategoriako urrats batek (kategoria hutsa, edo
falta diren fitxategiak) errotazio lehenetsiko abesti bat jotzen du bere
txandan (edozein musika, urratsa errotazio lehenetsiarena bera bada), beste
urratsek beren tokia mantentzeko: emisoraren identifikazio bat abestien
artean etortzen da oraindik.

Errotazioak Ezarpenak → Automatizazioa atalean egiten dira:

![Ezarpenak, Automatizazioa: Music (lehenetsia) eta Hits errotazioak, hautatutako Music bere urrats bakarrarekin musika guztiaren gainean, bere Gutxienez eta Gehienez kontagailuekin, eta Txantiloiak eta egunak ataleko hasiera](../../images/guide/settings-automation.png)

## Nola aukeratzen duen {#how-it-chooses}

Urrats baten barruan, musika-betegarriak denbora gehien airean izan ez diren
pistak hobesten ditu, haien artean zorizko apur batekin, ordena ez dadin inoiz
ziklo finko bihur. Hauek ere bereizita mantentzen ditu:

- **pista bera:** berriro jo daiteke 3 ordu igaro direnean edo bere urratseko
  beste pisten erdia ordutik jo denean, lehenik gertatzen dena;
- **artista bera:** ez 30 minutuen barruan.

Airera atera den guztiak balio du, musika-betegarriak, erreproduzitzaile batek
edo kartutxo-panelak jota; aurrez entzuteak ez. Kategoria txiki batek arauak
bete ezin dituenean, musika-betegarriak amore ematen du isilik geratu baino
lehen: lehenik artistaren arauan, gero pistarenean. Ezarpenak →
Automatizazioa atalak balio guztiak ditu, eta errotazioak egin, izena aldatu
eta aldatzen diren tokia da (ikus [Ezarpenak](settings.md#automation)).

Programazioko bloke baten aurretik, musika-betegarriak azken abestiak ere haien
luzeraren arabera aukeratzen ditu, musika garbi amaitzeko: blokea hastean
itzaltzen den abestia oraindik jotzen ari da blokearen orduan eta gehienez 30
segundo galtzen ditu haren ondoren, eta ez da hasten 45 segundo baino gutxiago
jotzeko abestirik. Artistaren eta pistaren arauak ere betetzen ditu ahal duen
guztietan. Errotazioko abestirik ez dagoenean biak betetzeko gai, gutxien
galtzen duena jotzen da, beraz batzuetan abesti baten segundo batzuk entzun
daitezke. Musikaren pausa batek ondorengo abestiak mugitzen ditu, eta
musika-betegarriak berriro aukeratzen ditu. Musika-betegarriak bloke batek
handik gutxira jotzen dituen abestiak eta artistak ere saihesten ditu. Bi
balioak oraingoz konfigurazio-fitxategian bakarrik alda daitezke
(`automation.max_fade_cut_secs` eta `automation.min_song_play_secs`).

**Orduko identifikazioa** pista-mota propio bat da (Liburutegia → Mota ▸
Orduko identifikazioa), emisoraren identifikaziorako ordu bakoitzaren hasieran.
