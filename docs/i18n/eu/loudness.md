# Ozentasun-normalizazioa

Pistak iturri askotatik datoz eta maila oso desberdinetan masterizatuta
daude: pop single berri batek 1970eko hamarkadako grabazio batek edo
ahozko promo batek baino bi aldiz ozenago jo dezake. **Ozentasun-normalizazioak**
pista eta kartutxo guztiak ozentasun hautemandu berean jotzen ditu, beraz
operadoreak ez du faderra mugitu behar pista batetik besterako.

**Instalazio berrietan aktibo dago**; lehendik dagoen konfigurazio batek bere
uneko ezarpena mantentzen du (gordetako konfigurazio guztiek gordeta dute
jada). Aldatu [Ezarpenak → Ozentasuna](settings.md#loudness) atalean.

![Ezarpenak, Ozentasuna: normalizazioa aktibo, helburuarekin, true peak sabaiarekin, gehieneko igoerarekin eta gehieneko doikuntzarekin, beren balio lehenetsietan](../../images/guide/settings-loudness.png)

## Nola dabilen {#how-it-works}

**Pista bakoitzeko irabazi finko bat.** Pista bakoitzak irabazi bakar bat
jasotzen du, dB-tan, pista kargatzean erabakia eta gelditu arte mantentzen
dena. Ez da ezer konprimitzen edo mugatzen: pista osoa kopuru berean egiten da
ozenago edo isilago, beraz bere dinamika masterizatu zen bezala geratzen da.
Erreproduzitzailearen faderrak horren gainean jarraitzen du lanean, lehen
bezala.

**Neurketa.** Atzeko planoko analisiak fitxategi bakoitza behin neurtzen du:

- bere **ozentasun integratua**, LUFS-etan: fitxategi osoak zein ozen
  entzuten den, ITU-R BS.1770 eta EBU R128 arauek definitzen duten bezala
  (pasarte isilak eta isiltasuna kanpoan uzten dira, beraz amaierako
  fade-out luze batek edo pausa batek ez du jaisten);
- bere **true peak**, dBTP-etan: seinalea analogiko bihurtzen denean laginen
  artean iristen duen mailarik altuena.

Fitxategi berriak lehen analisiarekin neurtzen dira. Ozentasuna neurtu aurretik
gehitutako fitxategiak **atzeko planoko pase batek** neurtzen ditu, ohiko
analisiak beste ezer egiteko ez duenean, beren markatzaileei ukitu gabe.
Normalizazioa aktibo egon edo ez exekutatzen da, beraz neurketak prest daude
aktibatzen duzunerako. **Berriro aztertu** (zerrendaren menuan) aukerak
fitxategi bat berriro neurtzen du, bere tamaina eta data aldatu ez badira ere
irakurriz. 0,4 s baino laburragoa den fitxategi bat ez da inoiz neurtzen,
eta fitxategi isil batek ez du ozentasunik: biak aldatu gabe jotzen dira.

**Irabazia.** Irabaziak neurtutako ozentasuna **Helburura** eramaten du, gehi
pistaren **Doikuntza** propioa (ikus [Pista bakoitza](#per-track-exclude-and-trim)):

> irabazia = helburua − neurtutako ozentasuna + doikuntza

Helburua baino ozenagoa den pista bat behar adina jaisten da. Isilagoa den
pista bat igo egiten da, baina inoiz ez **Gehieneko igoera** baino gehiago,
eta inoiz ez bere true peak **True peak sabaia** gainditzeko adina: ez dago
limitadorerik, beraz sabaiak du igotako pista bat mozten ez uzten. True peak-a
dagoeneko sabaiaren gainetik duen pista bat ez da batere igotzen.

Adibideak, balio lehenetsiekin (helburua −16 LUFS, sabaia −1 dBTP, gehieneko
igoera 12 dB):

| Pista | Neurtua | True peak | Irabazia |
|---|---|---|---|
| Master moderno ozen bat | −9,0 LUFS | +0,5 dBTP | −7,0 dB |
| Master zahar isilago bat | −20,0 LUFS | −10,0 dBTP | +4,0 dB |
| Gailur altuak dituen pista isil bat | −22,0 LUFS | −3,0 dBTP | +2,0 dB (sabaiak mugatua) |
| Oso grabazio isil bat | −35,0 LUFS | −20,0 dBTP | +12,0 dB (gehieneko igoerak mugatua) |

**Irabazia noiz erabakitzen den.** Pista bat kargatzean: erreproduzitzaile
baten hurrengo gisa prestatzean, abiaraztean, pausan kargatzean, CUEan aurrez
entzuten hastean, edo kartutxo gisa jaurtitzean. Jotzen edo pausan dagoen
pista batek **bere irabazia mantentzen du** gelditu arte, jauzien eta
pausen bidez. Ezarpenak, pistaren neurketa edo bere doikuntza aldatzeak
pista hurrengoan kargatzean aplikatzen da; erreproduzitzaile batean zain
dagoen hurrengo pista irabazi berriarekin berriro prestatzen da, beraz harekin
hasten da.

**Aurrez entzutea.** CUEak eta kartutxoen aurrez entzuteak pista airean izango
lukeen irabazi berarekin jotzen dute, beraz entzungailuetan entzuten duzuna
programarekin bat dator.

**Neurgailuak.** Neurgailuek eta haien ozentasun-irakurketak irabaziaren
ondoren neurtzen dute, beraz maila normalizatua erakusten dute. Neurgailuen
beraien **Ozentasun-helburuak** (Ezarpenak → Neurgailuak) irakurketa koloreztatzen
baino ez du egiten; ez du inongo irabazirik aldatzen.

## Bit perfect irteerak {#bit-perfect-outputs}

Irteera [bit perfect](bit-perfect.md) duen erreproduzitzaile edo kartutxo bat
**ez da inoiz normalizatzen**: beti irabazi unitarioarekin (0 dB) jotzen du,
ezarpenak edozein direla ere, edozein irabazik gailuak jasotzen dituen laginak
aldatuko lituzkeelako. Normalizazioa aktibo dagoela, bere goiburuak **0.0 dB**
ikur anbar bat erakusten du, eta bere argibideak "Bit perfect irteera: pistak
aldatu gabe jotzen dira." dio. Gauza bera gertatzen da bit perfect den Cue
gailu batean aurrez entzutean, erreproduzitzailearen Main irteerak, bit
perfect ez bada, irabazia jasotzen jarraitzen duen bitartean.

Bere gailua bit perfect bihurtzen den unean jotzen (edo pausan) ari den pista
batek hasi zen irabazia mantentzen du, eta **BP** ikurra itzalita geratzen da
harentzat. Hurrengo pistak 0 dB-tan eta bit perfect jotzen du. Egoera
horretako DSD pista bat PCM bihurtuta jotzen da bere irabaziarekin, inoiz ez
aldatu gabeko DSD fluxu gisa.

## Irabazia erreproduzitzailearen goiburuan {#the-gain-in-the-player-header}

Erreproduzitzaile bat jotzen edo pausan dagoen bitartean, **BP** ikurraren
ezkerreko ikur batek airean dagoen pistaren irabazia erakusten du. Ikurrak
bere koloreko ertza du eta beti zabalera bera. Pasatu sagua gainetik
arrazoia ikusteko.

| Ikurra | Esanahia |
|---|---|
| **+4.0 dB**, **−7.0 dB** (urdina) | Pista irabazi honekin normalizatzen da. Argibidea behean azaltzen den irabazi-banaketa da. Pista kargatu ondoren ezarpenak, neurketa edo doikuntza aldatu badira, pistak kargatzean ezarritako irabazia mantentzen duela bakarrik dio, eta irabazi hori erakusten du. |
| **0.0 dB** (anbarra) | Pista aldatu gabe jotzen da. Argibidea banaketa bera da, eta bere **Irabazi automatikoa** lerroak zergatik dioen: oraindik neurketarik ez du (edo neurtzeko laburregia da), irteera bit perfect da, edo pista normalizaziotik kanpo utzi da. |
| Ikurrik ez | Normalizazioa desaktibatuta zegoen pista hasi zenean, pista isila da, edo erreproduzitzailea geldituta dago. Bit perfect erreproduzitzaile batek ere ez du ikurrik erakusten normalizazioa desaktibatuta dagoen bitartean. |

Ikurrak pista hasi zenean erreproduzitzaileak eskatu zuen irabazia erakusten du.

### Irabazi-banaketa {#the-gain-breakdown}

Programak pista baten irabazia azaltzen duen toki guztietan lerro-bloke bera
erakusten du, etiketa ezkerrean eta balioa eskuinean: irabazi-ikurraren
argibidean, ozentasun-leiho txikian (bere kontrolen azpian), kartutxo-editorean
eta zerrenda-taulako pistaren argibidean.

| Lerroa | Esanahia |
|---|---|
| **Ozentasun integratua** | Pistaren neurtutako ozentasuna, LUFS-etan (edo "neurtu gabe"). |
| **True peak** | Bere true peak gorena, dBTP-etan (neurtuta dagoenean bakarrik). |
| **Helburua** | Pista guztiak eramaten diren ozentasuna (Ezarpenak → Ozentasuna). |
| **Irabazi automatikoa** | Helburua ken neurtutako ozentasuna, doikuntzaren eta edozein mugaren aurretik, edo "bat ere ez" eta zergatik: baztertuta, bit perfect irteera, normalizazioa desaktibatuta, oraindik neurtu gabe, edo isila. |
| **Doikuntza** | Pistaren erabiltzen ari den doikuntza manuala, 0 ez denean (gordetako balioarekin eta gehienekoarekin, jaitsitako **Gehieneko doikuntzak** mugatzen duenean). |
| **Nahi zen irabazia** eta **Honek mugatua** | True peak sabaiak edo gehieneko igoerak irabazia atzera egin duenean bakarrik: irabazi automatikoa gehi doikuntza (doikuntza dagoenean erakutsia) eta gelditu zuen muga. |
| **Irabazi emaitza** | Beti azkena, marra baten azpian eta lodiz: pistak jotzen duen irabazia. 0.0 dB normalizatzen ez denean. |

Normalizazioa desaktibatuta dagoenean, irabazi emaitzaren gaineko ohar batek
dio airean ez dela ezer aldatzen eta doikuntza aktibatzean aplikatuko dela.

## Pista bakoitza: baztertu eta doitu {#per-track-exclude-and-trim}

Pista batzuek ez lukete helburua jarraitu behar: nahita ozen masterizatutako
estazio-identifikatzaile bat, proba-tonu bat, neurketa entzuten den moduarekin
bat ez datorren pista bat (hizkera sarritan musika baino ozenagoa entzuten da
LUFS berean). Pista bakoitzak bi ezarpen ditu horretarako:

- **Normalizaziotik kanpo utzi:** pistak beti aldatu gabe jotzen du.
- **Doikuntza:** pistaren irabazian gehitutako dB-ak, **Gehieneko doikuntza**
  (Ezarpenak → Ozentasuna) negatibotik positiborako, 0,5 dB-ko urratsetan.
  Egin klik bikoitza graduatzailean 0ra itzultzeko. Doikuntza ilunduta
  dago pista baztertuta dagoen bitartean.

Graduatzailea **doikuntza** bat da: irabazi automatikoaren gaineko desplazamendu
bat, beraz 0 dB irakurtzen du editatu ez duzun pista guztientzat. Beraz,
leiho txikiak eta kartutxo-editoreak [irabazi-banaketa](#the-gain-breakdown)
erakusten dute beren kontrolekin: **Irabazi emaitza** lerroak graduatzailea
jarraitzen du mugitzen duzun heinean. Leiho txikian kontrolak lehenik daude eta
banaketa haien azpian, beraz graduatzailea dagoen tokian geratzen da banaketa
hazten edo txikitzen den bitartean. Pista entzuten ari den bitartean, leiho
txikiak aldaketa hurrengo kargatik aurrera aplikatzen dela gehitzen du.

Non aldatu:

- **Uhin-forma batean:** egin eskuineko klik erreproduzitzaile baten
  uhin-forman (edo bere CUE leihoarenean) eta aukeratu **Ozentasuna…**.
  Leiho txikiak pistaren izena ematen du eta bere neurketa erakusten du.
  Pista hori editatzen jarraitzen du erreproduzitzaileak beste batera pasatzen
  denean ere; itxi bere **×** botoiarekin. Pista kentzen bada, bere kabuz
  ixten da.
- **Zerrendako errenkada batean:** egin eskuineko klik errenkadan eta aukeratu
  **Ozentasuna…**: leiho txiki bera, errenkada horretako pistarako.
- **Kartutxo-editorean:** Ezarpenak → Kartutxoak, egin klik kartutxo batean:
  bi kontrol berberak **Gelditu beste kartutxoak jaurtitzean** aukeraren azpian
  daude, neurketa etiketaren azpian dagoela.
- **Urrunetik:** `PUT /api/v1/tracks/<id>/loudness` (ikus
  [Urruneko kontrola](remote-control.md)).

Ezarpenak pistarenak dira eta harekin gordetzen dira. Aldaketa pista
hurrengoan kargatzean aplikatzen da, beste edozein aldaketa bezala.

**Pistaren argibideak** zerrenda-taulan pistaren etiketak erakusten ditu eta,
marra baten azpian, [irabazi-banaketa](#the-gain-breakdown) bera, zerrenda
horrek dagokion erreproduzitzailearentzat kalkulatuta (bit perfect
erreproduzitzaile batek aldatu gabe jotzen du).

## Ezarpenak aukeratzea {#choosing-the-settings}

- **Helburua.** −16 LUFS egokia da aireko eta lineako musika-programa
  gehienetarako. −23 LUFS (EBU R128) edo −24 LUFS (ATSC A/85) tarte gehiago
  uzten dute, baina pista isilek orduan igoera handiagoak behar dituzte, eta
  sabaiak eta gehieneko igoerak atzera egin dezakete. Aukeratu zure
  materiala dagoeneko zein ozen dagoenaren hurbileko helburu bat.
- **True peak sabaia.** −1 dBTP-k lekua uzten du bihurketarako eta
  hurrengo etapetako kodetzaile galeradunentzat. Jaitsi stream-kodetzaile bat
  mozten denean.
- **Gehieneko igoera.** Oso grabazio isil bat (edo ia isila dena) zaratara
  igo ez dadin. 0 dB-k pista ozenak bakarrik jaisten ditu.
- **Gehieneko doikuntza.** Pista bakoitzaren doikuntza noraino irits daitekeen.
  Jaisteak dagoeneko ezarritako doikuntzak mugatzen ditu aldatu gabe, pista
  horren ozentasuna hurrengoan editatu arte: editatzeak orduan mugatutako
  doikuntza gordetzen du.

## Balio lehenetsien atzean dauden arauak {#standards-behind-the-defaults}

- **Neurketa.** Ozentasun integratua eta true peak, ITU-R BS.1770 arauak
  definitzen duen bezala (K-weighting, gated).
- **−16 LUFS helburua.** AES gomendioak jarraitzen ditu streaming eta lineako
  banaketarako. AES TD1008 (2021) aurreko AES TD1004.1.15-10 ordezten du,
  streamingerako −16 eta −20 LUFS artean gomendatzen zituena. Ez da
  emisio-balio bat: EBU R128-k −23 LUFS erabiltzen ditu eta ATSC A/85-ek −24
  LKFS. Erabili horietako bat entrega-espezifikazio batek eskatzen duenean
  (EBU R128 Europako emisio-programen trukerako, ATSC A/85 AEBetako telebista
  digitalerako), eta −16 stream edo lineako programa batetarako.
- **−1 dBTP true peak sabaia.** EBU R128-ren true peak gehienekoa, AES
  gomendioek ere erabiltzen dutena. Kodetzaile galeradunek gain egin dezakete
  deskodetzean, beraz bit-tasa baxuko kodetzaile batetik igarotzen den stream
  batek sabai baxuago bat nahi lezake, adibidez −2 dBTP (praktika arrunta,
  arau horien parte ez dena).
- **12 dB-ko gehieneko igoera eta gehieneko doikuntza.** Atzean arau
  gabeko produktu-aukerak: grabazio isil bat igotzeko eta pista bat belarriz
  zuzentzeko adina, zarata igo ez dadin edo doikuntzak helburua baliogabetu ez
  dezan.
- **Normalizazioa aktibo dago instalazio berrietan.** Aplikazioa
  eguneratzeak zure konfigurazioak dagoeneko duen ezarpena mantentzen du, beraz
  desaktibatuta zuen instalazio batek desaktibatuta jarraitzen du zuk
  aktibatu arte.
- **Neurgailuaren ozentasun-helburua** (Ezarpenak → Neurgailuak) ezarpen
  bereizi bat da, berez R128 balioa, −23 LUFS. Normalizazioak ez du
  erabiltzen.
