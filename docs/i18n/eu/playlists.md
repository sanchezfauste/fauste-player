# Zerrendak

## Fitxak {#tabs}

Erreproduzitzaile bakoitzak fitxa-errenkada bat du, zerrenda bakoitzeko
fitxa bat. Erreproduzitzaile guztiek zerrenda berberak ikusten dituzte;
bakoitzak aukeratzen du zein erakutsi. Fitxa bateko puntu batek
erreproduzitzailearen pistak non dauden erakusten du: **gorria** airean
dagoen pistarentzat, **berdea** hurrengoarentzat.

Fitxek erreproduzitzailearen zabalera partekatzen dute. Sartzen ez den izen
bat "…" batekin amaitzen da; pasatu sagua fitxaren gainetik osorik
irakurtzeko. Zerrenda asko daudenean, fitxak gutxieneko zabalera batean
uzten diote txikitzeari, errenkadaren muturretan geziak agertzen dira, eta
fitxen gaineko saguaren gurpilak korritu egiten ditu. Aukeratzen duzun
fitxa, eta erreproduzitzaile batek erakusten duena, ikusgai jartzen dira.

Fitxaz aldatzeak ez du inoiz aldatzen airean dagoena edo hurrengoa dena.
Pista bat amaitzean, erreproduzitzaileak pista hori duen zerrendan
jarraitzen du.

Zerrendak [Ezarpenak](settings.md) atalean sortzen, izenez aldatzen eta
ezabatzen dira. Ezin dira ezabatu azken zerrenda eta airean pista bat duen
zerrenda.

## Pisten taula {#the-track-table}

![Zerrenda bat: jotako pistak ilunduta, airean dagoena gorriz, hurrengoa berdez, eta oina falta den denborarekin](../../images/guide/playlist.png)

| Zutabea | Edukia |
|---|---|
| `#` | Posizioa, ezkerreko zeroekin; ikono batek ordezten du uneko eta hurrengo pistetan |
| Izenburua | Etiketetatik, edo fitxategiaren izenetik (`Artista - Izenburua.mp3` zatitu egiten da). Pista baten errepikapen- eta gelditze-ikonoak haren aurrean daude |
| Artista | Etiketetatik; "Artista ezezaguna" ez dagoenean |
| Albuma | Etiketetatik |
| Data | Grabazio-data, fitxategiak gordetzen duen bezala (`2019`, `2019-05` edo `2019-05-14`, ordua badago ordu eta guzti) |
| Generoa | Etiketetatik |
| Iraup. | Erreprodukzio-iraupena, cue-in puntutik cue-out puntura (fitxategi osoa **Erabili cue-in eta cue-out** desaktibatuta dagoenean) |
| Intro | Introak zenbat irauten duen, pista jotzen hasten den lekutik bere intro-markatzaileraino; hutsik pistak intro-markatzailerik ez duenean |
| Fitxategia | Fitxategiaren izena, bere luzapenarekin |

Instalazio berri batek `#`, Izenburua, Artista eta Iraup. erakusten ditu.
Beste zutabeak aukerakoak dira; ikus behean **Zutabeak aukeratzea**. Balio
bat ez duen pista batek gelaxka huts bat erakusten du, Artistan izan ezik,
non "Artista ezezaguna" agertzen den.

Zutabeek taula betetzen dute eta beren proportzioak mantentzen dituzte
leihoaren tamaina aldatzean; testu-zutabeek dute lekurik gehien. Arrastatu
goiburuko bereizleak proportzioak aldatzeko: bereizlearen eskuineko zutabeek
erakuslea jarraitzen dute frame bakoitzean (geratzen dena beren
zabaleraren proportzioan banatzen dute), ezkerrekoak dauden lekuan geratzen
dira, eta zabalerak askatzean gordetzen dira. Zutaberik ez da bere
gutxienekoa baino estuagoa izaten. Zabalerak erreproduzitzaile bakoitzeko
gogoratzen dira; geroago erakusten duzun zutabe bat bere zabalera
lehenetsiarekin hasten da, eta besteek beren proportzioak mantentzen
dituzte.

### Zutabeak aukeratzea {#choosing-the-columns}

Izenburua eta Iraup. beti erakusten dira. Beste zutabe guztiak erakutsi
edo ezkutatu daitezke, eta edozein zutabe mugi daiteke, bi horiek barne.
Zerrenda bera da erreproduzitzaile eta zerrenda guztietarako, eta
`config.json` fitxategian gordetzen da `ui.table_columns` gisa. Aldatzeko
hiru modu:

- **Ezarpenak → Zerrendak → Taulako zutabeak:** markatu erakutsi beharreko
  zutabeak; geziek erakutsitako zutabe bat gora edo behera mugitzen dute
  (tauletan ezkerretik eskuinera irakurtzen dira). **Zutabe lehenetsiak**
  botoiak `#`, Izenburua, Artista eta Iraup. zutabeetara itzultzen du.
- **Egin eskuineko klik goiburu batean:** aukerako zutabe bakoitzeko
  kontrol-lauki bat duen menu bat. Erakusten duzun zutabe bat eskuineko
  muturrean agertzen da; arrastatu hortik.
- **Arrastatu goiburu bat** beste baten gainera: jaregin goiburu baten
  ezkerreko erdian zutabea haren aurrean jartzeko, eskuineko erdian haren
  ondoren jartzeko. Beste edozein lekutan jaregiteak ez du ezer egiten.

Bertsio honek ezagutzen ez duen `ui.table_columns` bateko izen bat ez da
kontuan hartzen, eta Izenburua edo Iraup. falta badira, berriro gehitzen
dira.

Aplikazioa irekitzean, taula bakoitza korritu egiten da bere
erreproduzitzailearen hurrengo pista taularen erdian egon dadin (zerrendaren
muturrek uzten duten neurrian). Hori behin gertatzen da, abiaraztean, eta
hurrengo pista taulak erakusten duen zerrendan dagoenean bakarrik.

Erreproduzitzaile bat beste pista batera pasatzen denean, bere taulak
pista horren zerrenda erakusten du eta haren errenkada goialdera korritzen
du, azken 10 segundoetan taula erabili ez baduzu behintzat (korritu,
pista bat arrastatu, pista baten menua ireki edo fitxa batean klik egin):
kasu horretan, taula denbora horretan ukitu gabe utzi arte itxaroten
du. Denbora `ui.follow_current_grace_secs` da `config.json`
fitxategian; 0 balioak jarraipena desaktibatzen du.

Erreproduzitzaileak independenteak dira: hainbat erreproduzitzailek zerrenda
bera erakuts dezakete, bakoitzak bere hurrengo pista, bere jotako markak eta
oineko bere denborak dituela. Erreproduzitzaile batean jotzeak, gelditzeak
edo saltatzeak ez du inoiz beste erreproduzitzaile baten hurrengoa
mugitzen. Pista bera airean egon daiteke bi erreproduzitzailetan aldi
berean. Zerrenda editatzeak (sarrerak gehitu, mugitu edo kendu) edo
fitxategi bat irakurtezin bihurtzeak, ordea, hura erakusten duen edozein
erreproduzitzaileren hurrengoa alda dezake.

Errenkaden koloreak:

| Errenkada | Esanahia |
|---|---|
| **Gorria**, bozgorailu (edo pausa) ikono batekin | Airean erreproduzitzaile honetan. Gezi berdea ere erakuts dezake: airean dagoen pista hurrengoa ere bada, beraz beste behin joko da |
| **P2** gorria (edo beste zenbaki bat) zenbaki-zutabean | Airean erreproduzitzaile horretan |
| **Berdea**, gezi batekin | Erreproduzitzaile honen hurrengo pista |
| Ilunduta | Jada jota erreproduzitzaile honetan |
| Gurutze / abisu-ikonoa duen fitxategia | Fitxategia falta da / irakurtezina da (saltatu egiten da); pasatu sagua errenkadaren gainetik: leiho txikia arrazoiarekin hasten da, anbarrez, eta ondoren ohiko eremuak. Falta den fitxategi bat berriro bilatzen da 30 s-ro (`tuning.missing_recheck_ms`). |
| Birkargatze-geziak izenburuaren eskuinean | Aurreko bertsio batek aztertua; analisi horrekin jotzen da oraindik. **Ezarpenak → Analisia → Aztertu zaharkitutako pistak** aukerak eguneratzen du (erreproduzitzaile batean dauden pistak eguneratzen dira hala ere) |
| Harea-erlojua izenburuaren eskuinean | Pista bere analisiaren zain dago (pasatu sagua harea-erlojuaren gainetik: *Analisia egiteke*). Hala ere jotzen da, eta harea-erlojua desagertu egiten da analisia amaitzean |
| Morea | Hautatuta |

**Pistaren argibidea.** Pasatu sagua errenkada baten gainetik une batez
haren izenburua, artista, albuma, data, generoa, iraupena, formatua (mota,
lagintze-maiztasuna eta bit-sakonera, ezagutzen direnean) eta fitxategiaren
bide-izena ikusteko. Fitxategiak ez duen eremu bat ez da agertzen. Leiho
txiki hori da errenkada bateko gainetik pasatzeko informazio bakarra, eta
ez da inoiz mugitzen erakusten den bitartean. Falta den edo irakurtezina den
fitxategi batean, arrazoiarekin hasten da.

## Sagua {#mouse}

- **Klik** batek pista bat hautatzen du. **Klik bikoitzak** erreproduzitzaile
  honen hurrengo pista bihurtzen du. Airean dagoen pistan, uneko itzulia
  amaitzean beste behin jo dezan egiten du.
- **Eskuineko klikak** testuinguru-menua irekitzen du:

![Pista baten testuinguru-menua](../../images/guide/track-menu.png)

| Elementua | Ekintza |
|---|---|
| Erreproduzitu orain | Pista hau berehala abiarazten du (nahastuz, erreproduzitzailea airean badago) |
| Ezarri hurrengo gisa | Klik bikoitzaren berdina. Airean dagoen pistan, beste behin jotzen da, hasieratik, uneko itzulia amaitzean (Errepikatu bezala nahastuz, tarterik gabe), eta ondoren erreproduzitzaileak aurrera jarraitzen du. Behin bakarrik eragiten du. Gelditu uneko pistaren ondoren, SINGLE moduak eta Gelditu pistaren ondoren markak erreproduzitzailea amaitzen dute lehenik. CUE bat abian dagoen bitartean, hurrengo berrira mugitzen da |
| Aurrez entzun CUEan | CUE irteeran jotzen du (CUE leihoa irekitzen du). Ilunduta, erreproduzitzaileak bere Main irteeraz bestelako Cue irteerarik ez duenean |
| Editatu etiketak… | Pista honen etiketa-editorea irekitzen du. **Gorde** botoiak aldaketak audio-fitxategian idazten ditu; **Utzi** botoiak (edo Esc teklak, gordetzerik abian ez dagoenean) idatzi gabe ixten du. Elementua ilunduta dago, gainetik pasatzean arrazoia erakutsiz, pista airean, CUEan edo jotzen ari den kartutxo batean dagoen bitartean, bere etiketak oraindik irakurri ez diren bitartean, fitxategia falta denean, eta etiketak idatzi ezin zaizkien formatuetan (adibidez DSD) |
| Berriro aztertu | Pista hau berriro aztertzen du orain, bere egoera edozein dela ere. Irakurtezina zen eta konpondu den fitxategi bat ere berez hartzen da (ikus [Arazoen konponbidea](troubleshooting.md)). Eskuzko markatzaileak gorde egiten dira |
| Gehitu pistak azpian… | Pista honen ondoren txertatzeko fitxategiak aukeratzen ditu |
| Bikoiztu | Jo gabeko kopia bat txertatzen du azpian (bere errepikapen- eta gelditze-markekin) |
| Errepikatu pista hau | Markatu behin eta berriro jotzeko, tarterik gabe, Play (hurrengoa), Aurreko pista, Gelditu edo Itzaltzearekin gelditu sakatu arte, edo Gelditu uneko pistaren ondoren aktibatu arte. Pausatu sakatuta ere, errepikatzen jarraitzen du. Errepikapen-ikono bat agertzen da izenburuaren aurrean |
| Gelditu pista honen ondoren | Markatu pista hau amaitzean erreproduzitzailea gelditzeko, jotzen den bakoitzean (edozein modutan). Erreproduzitzailearen **Gelditu uneko pistaren ondoren** botoiak ez bezala, marka pistarekin geratzen da eta zerrendarekin gordetzen da. Gelditze-ikonoa izenburuaren aurrean agertzen da. Errepikatu aukeraren gainetik dago |
| Eraman hona ▸ | Beste zerrenda baten amaierara eramaten du |
| Kendu zerrendatik | Kentzen du; ezin da airean dagoen bitartean |

## Etiketak editatzea {#editing-tags}

**Editatu etiketak…** aukerak pista baterako leiho bat irekitzen du.
Irekita dagoen bitartean ez du laster-teklarik eragiten, eta aplikazioaren
leihoan jaregindako fitxategiei ez zaie jaramonik egiten.

![FLAC fitxategi baten etiketa-editorea, bere azala, izenburua, artista, albuma, data eta generoarekin](../../images/guide/tag-editor.png)

- **Zer ikusten duzun.** Editoreak fitxategia irekitzean irakurtzen du
  (bitartean "Etiketak irakurtzen…" erakusten du). Beti erakusten dira:
  izenburua, artista, albuma, albumaren artista, data, pista-zenbakia eta
  guztizkoa, disko-zenbakia eta guztizkoa, generoa, konpositorea eta
  iruzkina. Fitxategiak dituenean erakusten dira: azpititulua,
  taldekatzea, BPM, hasierako tonua, aldartea, ISRC, argitaletxea,
  katalogo-zenbakia, copyrighta, jatorrizko artista, jatorrizko albuma,
  jatorrizko argitaratze-data, hitzen egilea, zuzendaria, remixerra,
  moldatzailea, interpretea, hizkuntza, kodetzailea, hitzak, ordenatzeko
  izenburua, ordenatzeko artista, ordenatzeko albuma, ordenatzeko
  albumaren artista, ordenatzeko konpositorea eta artistaren webgunea.
- **Gehitu eremua.** Eremuen azpiko menuak beste eremuak zerrendatzen ditu.
  Fitxategiaren etiketa-formatuak gorde dezakeena bakarrik eskaintzen du
  (RIFF INFO duen WAV batek, AIFF batek edo ID3v1 etiketa zahar batek
  ID3v2, FLAC edo MP4 batek baino eremu gutxiago gordetzen dituzte), eta
  ilunduta dago gehitzeko ezer geratzen ez denean. Formatuak gorde ezin
  duen beti erakutsitako eremu bat grisez agertzen da ohar batekin. Eremu
  bat garbitzeak fitxategitik kentzen du; hutsik utzitako eremu gehitu bat
  ez da idazten.
- **Hainbat balio.** Hainbat balio izan ditzaketen eremuek (artista,
  albumaren artista, generoa, konpositorea, aldartea eta kredituak, hala
  nola hitzen egilea, zuzendaria, remixerra, moldatzailea eta interpretea,
  eta hizkuntza) balio bat erakusten dute lerro bakoitzeko; **Gorde**
  botoiak balio bat idazten du lerro bakoitzeko, formatuaren beraren
  moduan. Iruzkina eta hitzak hainbat lerrotako testu librea dira.
- **Egiaztapenak.** Data eta jatorrizko argitaratze-data ISO 8601 dira
  (`2019`, `2019-05` edo `2019-05-14`, aukeran orduarekin); pista- eta
  disko-zenbakia, haien guztizkoak eta BPMa zenbaki osoak dira, eta
  guztizko batek bere zenbakia behar du. Balio baliogabea duen eremu bat
  markatuta agertzen da eta **Gorde** desaktibatuta geratzen da. Fitxategiak
  jada zuen eta ukitu ez duzun balio bat dagoen bezala gordetzen da.
- **Luzeegiak diren eremuak.** Testua `limits.max_tag_chars` baino luzeagoa
  duen eremu bat, edo `limits.max_tag_values` baino balio gehiago dituena,
  irakurtzeko soilik erakusten da "Luzeegia hemen editatzeko; fitxategian
  dagoen bezala gordetzen da" oharrarekin. Ez da inoiz berridazten, beraz
  gordetze batek ezin du moztu.
- **Zer gordetzen den.** Editoreak erakusten ez duen guztia (beste gako
  estandar batzuk, gako pertsonalizatuak, aurreko azala ez diren irudiak,
  frame bitarrak) fitxategian geratzen da balio berberekin. Editoreak
  adierazten du zenbat etiketa gordetzen diren horrela (eta "beste batzuk
  ere" formatuak zenbatu ezin diren frameak dituenean). Gordetzeak
  editoreak lotzen dituen elementuak berriro kodetzen ditu, beraz gordetako
  elementu bat bere byteetan desberdina izan daiteke (testu-kodeketa,
  frameen ordena), baina ez bere balioan.
- **Azala.** Editoreak aurreko azala erakusten du, edo fitxategiaren lehen
  irudia aurreko azalik ez dagoenean, miniatura gisa.
  - **Aldatu…** botoiak fitxategi-elkarrizketa bat irekitzen du JPEG edo
    PNG irudi baterako (gehienez `limits.max_cover_bytes`, eta deskodetu
    egin behar da). Hala ez bada, editoreak zergatik adierazten du eta ez da
    ezer aldatzen.
  - **Kendu** botoiak aurreko azala garbitzen du. Desaktibatuta dago
    fitxategiak aurreko azalik ez duenean: aurreko azalik ez dagoelako
    bakarrik erakusten den irudi bat erakusteko baino ez da, eta dagoen
    bezala gordetzen da.
  - Fitxategian dagoen baina erakutsi ezin den azal bat (deskodetzen ez den
    irudi bat, edo GIF, BMP edo WebP bat) "Azal hau ezin da erakutsi; dagoen
    bezala gordetzen da" mezuarekin iragartzen da. **Aldatu…** eta
    **Kendu** botoiek funtzionatzen jarraitzen dute.
  - Aldaketa **Gorde** botoiak idazten du eta **Utzi** botoiak baztertzen.
    Atzeko azalak eta beste irudi guztiak ez dira inoiz ukitzen. Irudientzako
    lekurik ez duen formatu batek (RIFF INFO duen WAV, AIFF, ID3v1) eremua
    desaktibatuta erakusten du. Gorde ondoren, erreproduzitzailearen azalak
    azal berria erakusten du.
- **Nola gordetzen den.** Fitxategia jatorrizkoaren ondoan kopiatzen da,
  kopiak etiketak jasotzen ditu, sinkronizatu egiten da eta jatorrizkoa
  ordezten du; beraz, hutsegite batek fitxategia lehen zegoen bezala uzten
  du. Arrazoia editorean agertzen da, berriro saiatzeko irekita geratzen
  dena, eta egoera-barran. Aldatu dituzun eremuak bakarrik idazten dira.
  Gorde ondoren taulak etiketa berriak erakusten ditu berehala, eta
  markatzaileak eta uhin-forma gorde egiten dira. Fitxategiak aldatu duzun
  eremu bat gorde ez badu, egoera-barrak haren izena ematen du.
- **Eguneratze baten ondoren.** Aurreko bertsio bateko pistek beren data,
  generoa eta beste etiketak isilean betetzen dituzte atzeko planoan
  (analisi osorik gabe).

## Arrastatu eta jaregin {#drag-and-drop}

- Arrastatu pista bat zerrendaren barruan berrordenatzeko. Lerro more batek
  non geratuko den erakusten du: erakuslearen errenkada-mugarik hurbilenean
  dago, eta erakuslearen azpiko zerrendan bakarrik. Goiburuaren, zutabe-ertz
  baten, korritze-barraren edo zerrenda estaltzen duen leiho baten (CUE
  leihoa) gainean askatzeak ez du ezer jaregiten.
- Pista bat arrastatzen ari zarela, eutsi erakuslea zerrenda baten goiko edo
  beheko ertzetik gertu korritzeko: zenbat eta ertzetik gertuago, orduan eta
  azkarrago doa, eta zerrendaren muturretan edo ertzetik urruntzean
  gelditzen da. Saguaren gurpilak ere korritzen du zerrenda arrastatzean.
  Lerro moreak erakuslea jarraitzen du zerrenda mugitzen den heinean.
  Fitxategi-kudeatzailetik fitxategiak zerrenda baten gainera arrastatzeak
  modu berean korritzen du, sistemak erakuslearen posizioa adierazten duen
  lekuetan, erakuslea zerrendaren gainera mugitu ondoren.
- Arrastatu beste erreproduzitzaile baten zerrendara hara eramateko.
- Arrastatu fitxa batera zerrenda horren amaieran gehitzeko.
- Jaregin fitxategiak edo karpetak fitxategi-kudeatzailetik zerrenda batean
  jaregite-posizioan txertatzeko. Goiburuaren, zutabe-ertz baten,
  korritze-barraren edo zerrenda estaltzen duen leiho baten gainean ez da
  ezer txertatzen. Sistemak posizioa adierazten ez badu, erakutsitako
  zerrendaren amaierara doaz.

## Oina {#footer}

**+ Gehitu** botoiak fitxategi-elkarrizketa bat irekitzen du, Ezarpenetan
ezarritako musika-karpetan hasita. **Berrezarri** botoiak (ondoko gezi-
ikonoa) zerrendako pista guztien "jada jota" marka ilundua garbitzen du,
erreproduzitzaile guztientzat, "Garbitu zerrenda honetako pista guztien
erreproduzitutako marka?" galdetu ondoren (**Utzi**, Esc edo kanpoko klik
batek markak mantentzen dituzte). Airean dagoen pistak bere egoera
mantentzen du, eta erreproduzitzaileak uztean markatzen da. Botoia ilunduta
dago garbitzeko ezer ez dagoenean. Oinak pista kopurua, zerrendan falta den
denbora eta guztizko iraupena ere erakusten ditu.

## Zerrenda-fitxategiak {#playlist-files}

- **Inportatu:** Ezarpenak → Zerrendak → **Inportatu M3U / PLS…**, edo
  jaregin `.m3u`, `.m3u8` edo `.pls` fitxategi bat leihoan. Zerrenda berri
  bihurtzen da, fitxategiaren izena duena.
  - Bide-izen erlatiboak zerrenda-fitxategiaren karpetaren arabera
    ebazten dira.
  - `file://` helbideak ulertzen dira.
  - Aurkitu ezin diren fitxategiak hala ere gehitzen dira, erabilgarri ez
    gisa markatuta.
  - Interneteko streamak saltatu egiten dira; mezu batek zenbat diren
    adierazten du.
- **Esportatu:** Ezarpenetako zerrenda bakoitzaren **M3U** botoiak M3U8
  fitxategi gisa gordetzen du, izenburu, iraupen eta bide-izen osoekin.
