# Android

Fauste Player Android telefonoetan eta tabletetan ere dabil (Android 10 edo
berriagoa), mahaigainaren emisio-nukleo berean: erreproduzitzaileek,
zerrendek, kartutxo-panelak, markatzaileek, ozentasun-normalizazioak eta
erreprodukzio-arau guztiek berdin funtzionatzen dute.

## Instalazioa {#install}

Aplikazioa daraman argitalpen batek `fauste-player-<version>-android.apk`
du **Releases** orrian (bere `.sha256` fitxategiarekin). Ireki telefonoan
eta baimendu zure nabigatzaileak edo fitxategi-kudeatzaileak aplikazioak
instalatzea Androidek eskatzen duenean. Eguneratzeko, instalatu APK berriagoa
modu berean: zure zerrendak eta ezarpenak mantentzen dira. Aplikazioa ez dago
oraindik Google Play-n.

## Lehen abiaraztea {#first-start}

- **Musikarako sarbidea.** Baimendu musikarako sarbidea (eta jakinarazpenak,
  Android 13 eta berriagoetan). Hura gabe, musika-arakatzailea hutsik
  geratzen da.
- **Emisioak aurrera jarraitzen du.** Erreproduzitzaile bat edo kartutxo bat
  jotzen ari den bitartean, jakinarazpen bat ageri da eta audioak aurrera
  jarraitzen du pantaila itzalita edo aplikazioa atzeko planoan dagoela.
  Mahaigainean bezala, abiarazte edo hutsegite baten ondoren ez da ezer
  airera ateratzen bere kabuz.
- **Atzera eta ixtea.** Atzera botoiak lehenik irekita dagoena ixten du
  (musika-arakatzailea, Ezarpenak, Honi buruz, irekitako erreproduzitzaile
  bat, Liburutegia fitxako orri bat). Ixteko ezer ez dagoenean, Atzerak
  aplikazioa gordetzen du zerbait entzuten den bitartean (erreproduzitzaile
  bat, kartutxo bat edo CUE aurrez entzute bat) eta jotzen jarraitzen du;
  ezer entzuten ez denean, gorde eta aplikazioa ixten du. Aplikazioa
  aplikazio-aldatzailetik hatzaz kentzeak gauza bera egiten du: hatzaldi batek
  ez du inoiz jotzen ari dena gelditzen, eta aplikazioa itxi egiten da
  isilik geratzen denean.
- **Beste aplikazioen soinua.** Beste aplikazio batek audioa hartzen duenean,
  Fauste Player-ek jotzen jarraitzen du eta behean egoera-barran esaten du.

## Pantaila {#the-screen}

**Telefono** batean goiko barrak lau fitxa ditu, **Zuzenean**,
**Kartutxoak**, **Zerrendak** eta **Liburutegia**, ikono gisa erakutsiak
(hautatutakoak bere izenarekin ere, lekua dagoenean), eta gero **giltzarrapoa**
eta **⋮** (Gehiago: Ezarpenak eta Honi buruz). Erlojua lekua dagoenean
ageri da. Alboan jarrita, Zerrendak pantailak bere erreproduzitzaile-chipak
(P1, P2…) goiko barran jartzen ditu, eta beheko egoera-barra esateko zerbait
duenean bakarrik ageri da.

<p>
<img src="../../images/android/1-live.png" width="180" alt="Zuzenean telefono batean: bi erreproduzitzaile txartel gisa, bat osorik eta hurrengoaren hasiera, eta kartutxo-panela azpian">
<img src="../../images/android/2-carts.png" width="180" alt="Kartutxoak telefono batean: Orokorra orria zutik jarrita, errenkada bakoitzean bi kartutxo">
<img src="../../images/android/3-playlists.png" width="180" alt="Zerrendak telefono batean: P1etik P4rako chipak, zerrenda-fitxak, pista-zerrenda eta erreproduzitzailearen txartela behean">
<img src="../../images/android/4-music.png" width="180" alt="Gailuko musika telefono batean: Demo karpeta hiru abestirekin markatuta eta P1eko Goizeko zerrendara gehitzeko barra">
<img src="../../images/android/5-library.png" width="180" alt="Liburutegia fitxa telefono batean: Music kategoriako pistak, bakoitza markatzeko lauki batekin eta aurrez entzuteko entzungailu batzuekin, eta azpian aurrez entzuteko txartela pistaren uhin-formarekin eta bere markatzaileekin">
</p>

- *Zuzenean:* erreproduzitzaile bakoitza txartel gisa, eta kartutxo-panela.
  Txartel bakoitzak, goitik behera, erreproduzitzailearen izena, egoera,
  Single/Cont eta CUE ditu; azala, izenburua eta artista eta hurrengo pista;
  uhin-forma (ukitze batek jauzi egiten du); gero igarotako denbora eta
  pistaren iraupena, geratzen den denbora zifra handietan garraioaren gainean
  (Play/Hurrengoa, Aurrekoa, Pausatu, Gelditu, Berrabiarazi, Itzaltzearekin
  gelditu, Gelditu uneko pistaren ondoren), hatz baten tamainako botoiekin, eta
  neurgailu mehe bat bolumen-faderrarekin (faderraren maila haren azpian
  ageri da). Geratzen den denbora botoiak amaitzen diren lekuan amaitzen da,
  txikiagoa pista luze batean. Telefono gehienetan neurgailuak bere eskala
  ezkerrean erakusten du, eta zabalago batean (406 dp inguru eta gehiago)
  faderrak bere mailak ere erakusten ditu; telefono estu batean (384 dp
  inguru azpitik) bietako bat ere ez, eta botoiak pixka bat txikiagoak
  dira sar daitezen. Botoiek funtzionatzen dute beste hatz batek faderrari
  eusten dion bitartean, eta ukitze batek balio du hatza pixka bat mugitzen
  bada edo une batez behean geratzen bada ere. Zutik, txartelak goian daude,
  txartel oso bat eta hurrengoaren hasiera (korritu gehiagorako), eta
  kartutxo-panela azpian. Alboan, erreproduzitzaile bakoitza errenkada bat da
  ezkerrean (bere izena eta egoera, izenburua, geratzen den denbora,
  Play/Hurrengoa, Pausatu eta Gelditu; ukitu errenkada beste nonbait
  erreproduzitzailea Zerrendak atalean irekitzeko) eta kartutxo-panela
  eskuinean dago. Ukitu **KARTUTXOAK** bere goiburuan tolesteko: goiburua
  bakarrik geratzen da, behean (orri-fitxek eta *Gelditu dena* botoiak
  funtzionatzen jarraitzen dute), eta erreproduzitzaileek lekua hartzen dute;
  ukitu berriro zabaltzeko. Kartutxoak fitxak beti kartutxo-panel osoa
  erakusten du. Lekurako zabalegia den kartutxo-orri bat zutabe gutxiagotan
  erakusten da, ordena berean.
- *Kartutxoak:* erakutsitako kartutxo-orria, pantaila betez. Irristatu
  alboetara hurrengo edo aurreko orria erakusteko. Zutik, altuera baino
  zabalera handiagoa duen orri bat biratuta dago (8 × 2 2 × 8 gisa ageri da),
  ordena berean.
- *Zerrendak:* goiko chipek (P1, P2…) erreproduzitzailea aukeratzen dute;
  haien azpian bere zerrenda-fitxak (+ batek berri bat sortzen du), pista-zerrenda
  eta, behean (eskuinean telefonoa alboan dagoenean), erreproduzitzailearen
  txartela, pista bat prestatzeko uhin-forma handiagoarekin (lekua falta
  denean laburragoa da, botoiak beti ikus daitezen). Errenkada bati sakatuta
  eutsiz bere menua irekitzen da, **Igo**, **Jaitsi** eta **Eraman beste
  erreproduzigailu batera** ere dituena (pista erreproduzitzaile horren
  hurrengo pistaren ondoren doa, erakusten duen zerrendan; airean dagoen
  pistak bere lekuan jarraitzen du). Arrastatu errenkada bat bere ≡
  heldulekutik mugitzeko. Oinak **Gehitu** du eta, **⋯** atzean,
  **Berrezarri**. Uhin-formari sakatuta eusteak markatzaile bat jartzen du
  puntu horretan (cue-in, introaren amaiera, outroaren hasiera, MIX, cue-out)
  edo zuk jarritako bat kentzen du; markatzaileak hemen bakarrik editatzen
  dira, ez Zuzenean atalean.
- *Liburutegia:* pisten liburutegia eta gailuko musika (ikus
  [Liburutegia](#library)).

Beste pantaila batetik Atzera Zuzenean atalera doa; Liburutegian lehenik
kategorien zerrendara itzultzen da (Gailuko musikan, lehenik karpeta bat
gora).

Pantailan ezer mugitzen ez denean, aplikazioak segundoko behin marrazten du
(erlojurako) bateria aurrezteko; pausa edo gelditze baten ondoren oraindik
erortzen ari den neurgailu batek marrazten jarraitzen du neurgailua erori arte.

**Giltzarrapoak** ustekabeko ukituetatik babesten du ikuskizun batean.
Blokeatuta (giltzarrapoa itxita eta piztuta), bolumen-faderrek maila bakarrik
erakusten dute eta uhin-formek ez dute jauzirik egiten ezta markatzaileak
editatzen ere; ilundu egiten dira. Atximurkada batek eta arrastatze batek
uhin-forma bat handitzen eta mugitzen jarraitzen dute, eta Play, garraioak,
kartutxoek eta zerrendek beti funtzionatzen dute. Blokeoak berrabiarazi
ondoren aktibo jarraitzen du eta saioarekin gordetzen da, beraz beste saio bat
kargatzeak saio horren blokeoa hartzen du; urruneko APIa eta MIDI ez dira
blokeatzen.

Bolumen-fader batean hasten den arrastatze batek mugitzen du; beste edonon
hasten denak korritzen du, beraz korritzeak ez du inoiz airean dagoen maila
aldatzen. Uhin-forma batean ukitzeak hara jauzi egiten du, bi hatzeko
atximurkada batek zure hatzen inguruan handitzen du, hatz bakarreko
arrastatze batek handitutako ikuspegi bat mugitzen du eta **Ikuspegi osoa**
botoiak pista osoa berriro erakusten du. Zerrenda batean hatz batek korritzen
du; pista bat mugitzeko, erabili bere ≡ heldulekua edo errenkadaren menua.

**Tableta** batean diseinua mahaigainekoa da (ikus [Diseinua](layout.md)),
helburuak jotzeko errazagoak eginda.

Sakatuta eusteak mahaigainean eskuineko klikak irekitzen duena irekitzen du:
errenkadaren menua, goiburuko menuak, kartutxo baten menua (**Editatu…**
aukerak kartutxo huts baten fitxategia aukeratzen du). Telefono batean,
Ezarpenek pantaila osoa hartzen dute eta bere atalak zerrendatzen dituzte;
ukitu bat irekitzeko, ezarpen bakoitzaren izena bere kontrolaren gainean
dagoela. Geziak edo Atzerak zerrendara itzultzen du, eta Atzerak berriro (edo
**Itxi**) Ezarpenak ixten ditu. Kartutxo bat editatzeak bere atala zuzenean
irekitzen du.

Honi buruz → **Hirugarrenen lizentziak** aplikazioa eraikita dagoen
liburutegien oharrak irekitzen ditu nabigatzaile edo HTML ikusle batean.

## Liburutegia {#library}

**Telefono** batean **Liburutegia** fitxa kategorien zerrenda batean irekitzen
da: **Pista guztiak**, zure kategoriak (azpikategoria bakoitza bere gurasoaren
azpian, bere koloreaz eta zenbat pista dituen), **Kategoriarik gabe** eta
**Gailuko musika**. Liburutegia mahaigaineko [liburutegiaren leihoak](library.md)
erakusten duen bera da: zerrenda batera, kartutxo batera edo liburutegira
gehitutako fitxategi guztiak bertan daude. Kategoriak **Ezarpenak →
Liburutegia** atalean sortzen, izenez aldatzen, koloreztatzen eta ezabatzen
dira.

**Kategoria baten pistak.** Ukitu kategoria bat bere pistak irekitzeko (bere
azpikategorietakoekin). Bere goiburuko geziak, edo Atzerak, kategorietara
itzultzen du. Beste fitxa batera eta atzera joateak zerrenda utzi zenuen
bezala aurkitzen du. Kategorien artetik kategoria bat irekitzeak bilaketa
hutsik hasten du. Bilaketak zerrenda izenburuaren, artistaren, albumaren eta
liburutegiaren leihoak bilatzen dituen beste etiketen arabera iragazten du;
**Liburutegi osoa** aukerak kategoria guztietan bilatzen du. Errenkada
bakoitzak izenburua (edo fitxategi-izena), artista, iraupena eta leihoak
fitxategi falta, irakurezin edo oraindik aztertu gabe batentzat erabiltzen
dituen markak erakusten ditu.

Ukitu errenkadak markatzeko. Zerbait markatuta dagoen bitartean, Gailuko
musikaren barra behean ageri da: zenbat, zein erreproduzitzaile eta
zerrendatara doazen (ukitu beste bat aukeratzeko), **Gehitu amaieran** eta
**Gehitu hurrengoaren ondoren**, Gailuko musikan bezala. Pistak markatu
dituzun ordenan doaz, eta gehitzeak markak garbitzen ditu. Kategorietara
itzultzeak ere garbitzen ditu.

**Errenkadaren menua.** Errenkada bati sakatuta eutsiz bere menua irekitzen da:
**Gehitu hona ▸** (erreproduzitzaile baten zerrenda, amaieran), **Aurrez
entzun**, **Eraman kategoriara ▸**, **Berriro aztertu** eta **Kendu
liburutegitik…**, mahaigainean bezala galdetzen duena (ikus
[Pistak kentzea](library.md#removing-tracks)). Errenkadak markatuta dauden
bitartean, beste errenkada bati sakatuta eutsiz hura ere markatzen da eta
menuak markatutako errenkada guztiei eragiten die, markatu dituzun ordenan;
orduan **Aurrez entzun** ez da eskaintzen. Markak menuko ekintzaren ondoren
geratzen dira.

**Aurrez entzutea.** Errenkada baten eskuineko 🎧 ikonoak bere pista cue-in
puntutik jotzen du liburutegiaren aurrez entzuteko irteeran; beste errenkada
baten 🎧 ikonoak pista horretara pasatzen du. Zerrendaren gaineko zerrenda
mehe batek izenburua, geratzen den denbora, **Pausatu**/**Aurrez entzun** eta
**Gelditu** erakusten ditu jotzen ari den bitartean (edo pausan), eta
bestela iradokizun bat: beti bere lekua mantentzen du, beraz errenkadak ez dira
mugitzen aurrez entzute bat hasi edo gelditzen denean. 🎧 ilunduta dago
fitxategia falta denean edo jo ezin denean.

**Aurrez entzuteko txartela.** Ukitu zerrenda (bere botoietatik kanpo)
txartela zerrendaren azpian irekitzeko, pistaren uhin-formarekin,
Zerrendak ataleko erreproduzitzaile baten txartelean bezala; zerrendaren
eskuineko geziak irekita dagoen erakusten du. Ukitu zerrenda berriro, edo
sakatu Atzera, ixteko; aurrez entzuteak jarraitzen du. Zerrendaren
goiburuko geziak berehala itzultzen du kategorietara eta txartela irekita
uzten du hurrengo zerrendarako. Txartelean:

- uhin-forma ukitzeak puntu horretara jauzi egiten du edo, geldirik
  dagoenean, **Aurrez entzun** non hasten den finkatzen du;
- sakatuta eustean markatzaile bat jartzen da puntu horretan (cue-in,
  introaren amaiera, outroaren hasiera, MIX, cue-out) edo zuk jarritako
  bat kentzen da. Markatzaileak pistarenak dira, beraz aldaketa zerrenda
  guztietan eta mahaigainean ikusten da;
- atximurkatzeak handitu egiten du, arrastatzeak handitutako ikuspegia
  mugitzen du eta **Ikuspegi osoa** aukerak pista osoa erakusten du
  berriro;
- uhin-formaren azpian daude denborak, **Aurrez entzun**, **Pausatu**,
  **Gelditu** eta neurgailua.

Zerrenda txartelaren gainean geratzen da (ezkerrean telefonoa etzanda
dagoenean), beraz beste errenkada baten 🎧 ikonoak txartela pista horretara
aldatzen jarraitzen du. Giltzarrapoak uhin-formaren jauziak eta
markatzaileen aldaketak gelditzen ditu, erreproduzitzaileetan bezala.
Aurrebista gelditu egiten da bere
pista liburutegitik ateratzen denean eta Liburutegia fitxa uzten duzunean
(beste fitxa batera, edo Atzerarekin); kategorietan edo Gailuko musikan
jarraitzen du, baina bere zerrenda pisten zerrendan bakarrik ageri da.

Mahaigainean ez bezala, Androidek (telefono batek edo tableta batek)
liburutegiaren aurrez entzutea irteera lehenetsian jotzen du harentzat irteerarik
aukeratu ez denean: **Ezarpenak → Audio-irteerak → Liburutegia** atalean
aukera hutsak *Sistemaren lehenetsia* dio, eta bere **Probatu Cue** botoiak
hor jotzen du. Aukeratu hor beste irteera bat (adibidez, entzungailu
kableatuak) airetik urrun aurrez entzuteko. Irteera lehenetsia normalean
erreproduzitzaileena eta kartutxo-panelarena ere izaten da, beraz aurrez
entzutea airean entzun daiteke: irteera horretan zerbait jotzen ari den
bitartean, 🎧 ikonoak lehenik galdetzen du (**Aurrez entzun hala ere** edo
**Utzi**), eta berretsi ez duzun aurrez entzute bat hor zerbait hasten
denean gelditzen da, erreproduzitzaile baten CUEk bezala (ikus [CUE (aurrez
entzutea)](players.md#cue-pre-listen)).

**Gailuko musika kategoria batera gehitzea.** Pista-zerrenda baten goiburuko
**＋** botoiak Gailuko musika irekitzen du kategoria horretarako. Bere barrak
orduan **Gehitu hona: &lt;kategoria&gt;** bakarrik du (**Gehitu hona: Kategoriarik
gabe** *Pista guztiak* edo *Kategoriarik gabe* ataletik), markatutako
fitxategiak kategoria horretan liburutegira gehitzen dituena eta zerrendara
itzultzen dena; liburutegian dagoeneko dagoen fitxategi bat kategoriarik
ez badu horretara sartzen da eta bestela daukana mantentzen du, mahaigainean
bezala. Gailuko musikak bere markak mantentzen ditu bisiten artean, beraz
aurreko bisita batean markatutako fitxategiak ere gehitzen dira: egiaztatu
barrako kopurua lehenik. Goiko **Itzuli zerrendara** botoiak, edo Atzerak,
zerrendara itzultzen du gehitu gabe. Kategorietatik irekitako Gailuko
musikak goian **Itzuli kategorietara** du horren ordez.

**Kategoria baten gailuko karpeta.** Kategoria batek bere fitxategiak
gailuko musikaren karpeta batetik har ditzake. Bere menuan (sakatuta eutsi
**Ezarpenak → Liburutegia** atalean, edo liburutegiaren leihoan tableta
batean), **Erabili gailuko karpeta bat…** aukerak Gailuko musika irekitzen du
karpeta aukeratzeko (**Erabili karpeta hau**); kategoriaren menuak orduan
izena ematen dio, bere karpeta-elementuen gainean. Ezerk ez du karpeta
zaintzen: han musika berria kopiatu duzunean, ukitu kategoriaren pista-zerrendaren
goiburuko **↻** (edo bere menuko **Irakurri berriro gailuko karpeta**).
Gailuko musikak karpetan eta bere azpikarpetetan zerrendatzen duen fitxategi
guztia kategoriara gehitzen da, goiko arauekin (beste kategoria batean dagoen
fitxategi batek hura mantentzen du), eta ohar batek zenbat fitxategi irakurri
diren esaten du. Karpetatik atera diren fitxategiak liburutegian geratzen
dira, falta gisa markatuta, eta normaltasunera itzultzen dira karpetara
itzultzen direnean (Gailuko musikak ezer zerrendatzen ez duenean, adibidez
baimenik gabe, ez da ezer markatzen). **Aldatu gailuko karpeta…** eta **Ahaztu gailuko karpeta**
menu berean daude; karpeta bat kategoria bakar batena da.

### Gailuko musika {#device-music}

**Gailuko musikak** gailuaren audio-fitxategiak **Karpetak** (bide-izena
zerrendaren gainean; ukitu bere zati bat gora joateko), **Artistak** edo
**Albumak** bidez erakusten ditu, eta izenburuan, artistan, albumean eta
fitxategi-izenean bilatzeko aukera. Errenkada bakoitzak izenburua, artista eta
iraupena erakusten ditu; ukitu errenkadak markatzeko. Zerbait markatuta
dagoen bitartean, beheko barrak zenbat, zein erreproduzitzaile eta zerrendatara
doazen (ukitu beste bat aukeratzeko), **Gehitu amaieran** eta, erreproduzitzaile
horren hurrengo pista zerrenda horretan dagoenean, **Gehitu hurrengoaren
ondoren** erakusten ditu. Pistak markatu dituzun ordenan doaz. Gailuko fitxategi
batek ez du aurrez entzuterik.

Zerrenda baten oineko **Gehitu** botoiak, kartutxo baten fitxategiak eta
Ezarpenak → Zerrendak → Musika-karpetak Gailuko musika irekitzen dute telefono
batean (tableta batean, **musika-arakatzailea** leihoa, karpetaka). Markatu
fitxategiak eta ukitu **Gehitu**, ukitu fitxategi bat kartutxo baterako, edo
**Erabili karpeta hau**; **Utzi** edo Atzerak lehen zeunden lekura itzultzen
du. Kopiatu musika gailura (adibidez bere *Music* karpetara) han ikusteko.
Fauste Player-ek zure musika irakurtzen baino ez du egiten: etiketak eta
azalak ez dira idazten, eta zerrendak eta kartutxo-orriak ez dira inportatzen
edo esportatzen oraindik Androiden.

**Tableta** batean liburutegia mahaigaineko [liburutegiaren
leihoa](library.md) da, goiko barratik irekia.

## Irteerak {#outputs}

Ezarpenak → Audio-irteerak atalak telefonoaren irteerak motaren arabera
zerrendatzen ditu: bozgorailua, entzungailu kableatuak, USB interfaze bat,
Bluetooth bozgorailu bat ("JBL Flip (Bluetooth)"). Erreproduzitzaile
bakoitzak, kartutxo-panelak eta haien CUEak bat aukeratzen dute, mahaigainean
bezala, eta liburutegiaren aurrez entzuteak ere (ikus [Liburutegia](#library)).
Itzali eta piztutako Bluetooth bozgorailu bat berriro aurkitzen da bere
bidearen bidez; kanpoan dagoen bitartean, bere erreproduzitzaileek mahaigainean
gailu falta batekin bezala jokatzen dute. Bi irteerak aldi berean soinua eman
dezaketen ala ez telefonoaren araberakoa da.

## Androiden ez {#not-on-android}

MIDI kontrol-gainazalak, urruneko kontrola, etiketa eta azalen edizioa, eta
zerrenda eta kartutxo-orri fitxategiak oraingoz mahaigaineko eginbideak dira.
Tableta batera konektatutako teklatu batek mahaigainean bezala funtzionatzen du
(ikus [Teklatua](keyboard.md)).
