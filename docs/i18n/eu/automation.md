# Automatizazioa

Automatizazioak estudioan inor ez dagoenean ere emisora airean mantentzen du.
Bertsio honek **musika-betegarria** dakar, musika bere kabuz aukeratzen duen
erreproduzitzailea, zuk ezarritako arauen arabera, eta inoiz zerrenda faltagatik
gelditzen ez dena, eta **Programazioa** zutabea, egunaren ordu-blokeak
exekutatzen dituena, haiek egiteko editorearekin batera.
Dagoeneko erabiltzen dituzun erreproduzitzaileak (P1 … Pn) ukitu gabe geratzen
dira: zureak izaten jarraitzen dute, prestatzen dituzun eta zuzenean jotzen
dituzun saioetarako.

## Programazioa zutabea {#the-schedule-column}

**Programazioa** zutabeak egunaren **blokeak** jotzen ditu: hasiera-ordua duten
elementu-taldeak (orduko identifikazio bat, iragarkiak, saio bat). Erakutsi
goiko barrako **Programazioa** lauzarekin, erreproduzitzaile bat erakusten duzun
bezala.

![Programazioa zutabea martxan, Musika zutabearen ondoan: egunaren blokeak hasiera-orduaren arabera, bakoitza bere emisoraren identifikazioarekin (erlojuak puntual gisa markatzen du), iragarkiekin eta blokearen ondoren musika-betegarriak jotzen duen errotazioarekin; betegarriaren airean dagoen abestia eta hurrengoak](../../images/guide/automation-columns.png)

Bere zerrenda programazioa da, blokeka taldekatuta: bloke bakoitzaren
hasiera-ordua, eta gero bere elementuak. Gaurkoa erakusten du, atzoko azken
blokea eta biharko lehena. Egun baterako aldatutako bloke batek hala dio.
Airean dagoen elementua eta hurrengoa erreproduzitzaile batean bezala markatzen
dira; dagoeneko jo diren elementuak ilunduta daude, eta jo ez zen elementu bat
(**Igorri gabea**) marratuta dago. Erloju batek elementu **puntual** bat
markatzen du, bere orduan bakarrik jotzen dena.

Zerrendaren gainean:

- **Hasi** botoiak programazioa abiarazten du: bloke bakoitza bere orduan
  jotzen da, eta musika-betegarriak blokeen artean jotzen du. Ezer ez da bere
  kabuz hasten: aplikazioaren abiarazte bakoitzaren ondoren, automatizazioa
  geldituta dago Hasi sakatu arte.
- **Gelditu unekoaren ondoren** botoiak airean dagoena amaitzen uzten du, eta
  gero ez da ezer bere kabuz jotzen.
- **Martxan** edo **Geldituta** testuak zein egoeratan dagoen esaten du.

Hurrengo zer jotzen den aldatzeko, egin klik bikoitza elementu batean, edo
egin eskuineko klik eta aukeratu **Hurrengo entzun dadila**; airean dagoen
abestiaren ondoren datorkio. **Itzuli erlojura**, menu berean, programazioak
berriro orduaren arabera aukeratzen uzten du.

## Programazioa editatzea {#editing-the-schedule}

Programazio zutabeko **Editatu programazioa** botoiak editorea irekitzen du.
Hiru fitxa ditu, eta **Errotazioak…** botoiak Ezarpenak → Automatizazioa
atalera eramaten zaitu, errotazioak egiten diren tokira.

![Programazio-editorea, Txantiloiak fitxa: Base eta Weekend txantiloiak ezkerrean; Base-ren blokeak eskuinean, bakoitza bere hasierarekin, bere elementuekin, Gehitu kategoriatik botoiarekin eta ondoren datorren musikaren errotazioarekin](../../images/guide/schedule-templates.png)

- **Txantiloiak.** Txantiloi bat bloke-egun bat da; instalazio berri batek bat
  du, "Base", egun guztietarako. Sortu, bikoiztu, aldatu izena edo ezabatu
  txantiloiak ezkerrean; aukeratutakoaren blokeak eskuinean daude. Idatzi
  blokearen hasiera `HH:MM:SS` gisa (existitzen ez den ordu bat oharrarekin
  geratzen da eremuan, eta ezer ez da aldatzen). Arrastatu liburutegiko pistak
  bloke batera gehitzeko; **Gehitu kategoriatik** aukerak kategoria horretako
  pista bat jotzen duen elementu bat gehitzen du, aldi bakoitzean desberdina;
  **Musika ondoren** aukerak blokearen ondoren musika-betegarriak jotzen duen
  errotazioa aukeratzen du (errotazio lehenetsia, bat aukeratzen ez bada).
  Elementu bakoitza **Puntuala** gisa marka daiteke, mugitu edo kendu. Blokearen
  elementuak hurrengo blokera arteko denbora baino luzeagoak direnean, blokeak
  zenbat gainditzen duen esaten du, anbarrez; hala ere horrela utz dezakezu.
- **Aste mota.** Asteko egun bakoitzak zein txantiloi erabiltzen duen.

  ![Aste mota fitxa: Base astelehenetik ostiralera, Weekend larunbatean eta igandean](../../images/guide/schedule-week.png)

- **Egunak.** Aukeratu data bat egutegian (aldaketak dituzten datak anbarrezkoak
  dira). Data batek bere asteguneko txantiloiaz bestelako bat erabil dezake
  (osteguneko jai batek asteburukoa erabil dezake). Bere blokeak data horretarako
  bakarrik alda daitezke: aldaketa markatuta dago, eta **Itzuli txantiloira**
  botoiak desegiten du. **Egun honetan ez** aukerak bloke bat data horretatik
  bakarrik kentzen du, eta **Gehitu blokea egun honetan** aukerak bat gehitzen
  du. Txantiloiaren geroko aldaketek data horretan aldatu ez dituzun blokeei
  eragiten diete.

  ![Egunak fitxa: egutegia gaur aukeratuta eta aldaketak dituzten datak anbarrez; gaurko lehen blokeak iragarki bat gehiago du, "egun honetarako aldatua" markatuta, Itzuli txantiloira botoiarekin](../../images/guide/schedule-days.png)

## Musika-betegarria {#the-music-filler}

Musika-betegarria **Musika** zutabea da. Ezkutuan dago erakutsi arte: egin klik
goiko barraren ezkerreko **Musika** lauzan. P1-en ezkerrean dago.

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

Bere irteerak Ezarpenak → Audio-irteerak atalean ezartzen dira, **Musika**
errenkadan, edozein erreproduzitzaile bezala. Ez du zenbakirik, laster-teklarik
edo MIDI loturarik, eta urruneko kontrolak ez du ikusten.

Sakatu **Play** hasteko eta **Gelditu unekoaren ondoren** airean dagoen abestia
amaitu eta gelditzeko. Erreproduzitzaile guztiak bezala, ez da inoiz bere kabuz
hasten, ezta berrabiarazi ondoren ere.

Bere zerrendaren gainean, erreproduzitzaile batek bere zerrenda-fitxak dituen
tokian, Musika zutabeak jotzen duen **errotazioa** erakusten du. Aukeratu hor
beste bat: ilaran zituen abestiak ordezkatzen dira, zuk gehitutakoak geratzen
dira. Errotazioak jotzeko ezer aurkitzen ez duenean (bere kategoriak hutsik
daude, edo fitxategi guztiak falta dira), barrak hala dio.

## Errotazioak {#rotations}

**Errotazioa** betegarriak txandaka jotzen dituen **urratsen** zerrenda labur bat
da, eta gero berriro lehenengotik. Urrats bakoitzak bere pistak nondik datozen
adierazten du (liburutegiko kategoria bat bere azpikategoriekin, edo mota
bateko pista guztiak) eta aldi bakoitzean zenbat jotzen dituen, tarte gisa:

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
bat du, **Music**, zure musika guztia jotzen duena.

Errotazioak Ezarpenak → Automatizazioa atalean egiten dira:

![Ezarpenak, Automatizazioa: Music (lehenetsia) eta Hits errotazioak, hautatutako Music bere urrats bakarrarekin musika guztiaren gainean, bere Gutxienez eta Gehienez kontagailuekin, eta musika-betegarriaren arauak](../../images/guide/settings-automation.png)

## Nola aukeratzen duen {#how-it-chooses}

Urrats baten barruan, betegarriak denbora gehien airean izan ez diren pistak
hobesten ditu, haien artean zorizko apur batekin, ordena ez dadin inoiz ziklo
finko bihur. Hauek ere bereizita mantentzen ditu:

- **pista bera:** berriro jo daiteke 3 ordu igaro direnean edo bere urratseko
  beste pisten erdia ordutik jo denean, lehenik gertatzen dena;
- **artista bera:** ez 30 minutuen barruan.

Airera atera den guztiak balio du, betegarriak, erreproduzitzaile batek edo
kartutxo-panelak jota; aurrez entzuteak ez. Kategoria txiki batek arauak bete
ezin dituenean, betegarriak amore ematen du isilik geratu baino lehen: lehenik
artistaren arauan, gero pistarenean. Ezarpenak → Automatizazioa atalak balio
guztiak ditu, eta errotazioak egin, izena aldatu eta aldatzen diren tokia da
(ikus [Ezarpenak](settings.md#automation)).

Programazioko bloke baten aurretik, betegarriak azken abestiak ere haien
luzeraren arabera aukeratzen ditu, musika garbi amaitzeko: blokea hastean
itzaltzen den abestiak gehienez 30 segundo galtzen ditu, eta ez da hasten
45 segundo baino gutxiago jotzeko abestirik. Errotazioko abestirik ez dagoenean
biak betetzeko gai, gutxien galtzen duena jotzen da. Bi balioak oraingoz
konfigurazio-fitxategian bakarrik alda daitezke
(`automation.max_fade_cut_secs` eta `automation.min_song_play_secs`).

**Orduko identifikazioa** pista-mota propio bat da (Liburutegia → Mota ▸
Orduko identifikazioa), emisoraren identifikaziorako ordu bakoitzaren hasieran.
