# Ezarpenak

Ireki **Ezarpenak** goiko barran. Itxi **Itxi** botoiarekin edo `Esc`
teklarekin. Aldaketa gehienak berehala aplikatzen dira eta automatikoki
gordetzen dira.

Leihoak tamaina bakarra du (900 × 640, txikiagoa pantaila txiki batean),
atala edozein dela ere, eta atalak barruan korritzen du. Atal bakoitzak
bere etiketak zutabe bakarrean lerrokatzen ditu.

Erreproduzitzaileak, Neurgailuak, Analisia, Ozentasuna eta Laster-teklak atalek
**Leheneratu balio lehenetsiak** botoi bat dute goiburuan. Berrespena
eskatzen du, eta ondoren atal hori bakarrik berrezartzen du
(Erreproduzitzaileak atalak erreproduzitzaile kopurua eta hizkuntza
mantentzen ditu; Laster-teklak atalak ez du beste berrezartze-botoirik).
Audio-irteerak, Zerrendak, Kartutxoak, Liburutegia, MIDI eta Urrunekoa atalek ez dute.

## Zain dauden ezarpenak {#settings-pending}

Ezarpen guztiak aplikazioa abian dagoela aplikatzen dira. Audio-sistemak,
lagintze-maiztasunak, bufferraren tamainak (baita gailu batena ere), Main eta
Cue irteerek (erreproduzitzaileenak eta kartutxo-panelarenak), bit perfect
gailuek eta DSD ezarpenek irteera-gailu bat nola irekitzen den aldatzen
dute, beraz segurua den ahalik eta lasterren aplikatzen dira:

- Gailu baten ezarpenak gailu horretan ezer jotzen ez denean aplikatzen dira:
  harantz bideratutako erreproduzitzailerik ez dago jotzen, itzaltzen edo
  aurrez entzuten (CUE, eusten ari denean ere), eta ez dago kartutxorik
  jotzen. Pausan dagoen erreproduzitzaile batek, edo pista bat kargatuta
  baino ez duenak, ez du gailurik mantentzen: bere pista pausan geratzen da
  zegoen lekuan.
- Erreproduzitzaile baten edo kartutxo-panelaren irteera mugitu egiten da
  erreproduzitzaile hori (edo kartutxo-panela) jotzen ari ez denean; gailu
  berberetako beste erreproduzitzaileak ez dira etetzen.
- Audio-sistema berri bat inon ezer jotzen ez denean aplikatzen da.

Zain dagoen aldaketa bat eragiten dion ezerk jotzen ez duen lehen unean
exekutatzen da. Hori pista bat amaitu eta hurrengoa hasi arteko tartea izan
daiteke: orduan hurrengo pista gailua berriro irekitzeko behar den denbora
beranduago hasten da. **Aplikatu orain** botoiak (behean) aldaketa
berehala behartzen du.

Gailu bati emandako maiztasun edo buffer batek gailua zerekin irekitzen den
aldatzen duenean bakarrik du eragina: gailu bati orokorraren balio bera
ematea, edo halako balio bat garbitzea, ez du ezer aldatzen.

Ordura arte, goiko barrak **Zain dauden ezarpenak** erakusten du. Pasatu
sagua gainetik zer dagoen zain eta zeren zain ikusteko; egin klik zerrenda
ikusteko, gailu edo irteera batek baztertu duen edozein aldaketa ere
erakusten duena (gailuak bere uneko ezarpenak mantentzen ditu; aldaketa
berriro saiatzen da aldatzen duzunean edo **Aplikatu orain** sakatzen
duzunean). Ezarpenen oinak "Aldaketa batzuek ukitzen dituzten irteerak libre
egon arte itxaroten dute." dio eta **Aplikatu orain** eskaintzen du; ohar
labur batek (adibidez, ezarpen bat gorde dela) testu horren lekua har dezake
une batez.

**Aplikatu orain** botoiak zain dauden irteera-aldaketa guztiak berehala
aplikatzen ditu. Horrek zerbait eteingo balu, lehenik gailuak eta haietan
jotzen duena zerrendatzen ditu; **Eten eta aplikatu** botoiak orduan gailu
horietako audioa laburki eteten du: jotzen ari zena zegoen lekutik jarraitzen
du tarte labur baten ondoren, eta pausan edo geldituta zegoen ezer ez da
hasten. **Utzi** botoiak (edo `Esc` teklak) itxaroten jarraitzen du.

Mugak eta motorraren doikuntza konfigurazio-fitxategian editatzen dira
aplikazioa itxita dagoela (ikus [Datuak eta babeskopiak](data-and-backups.md)).

## Audio-irteerak {#audio-outputs}

Atal honetako aldaketak aplikazioa abian dagoela aplikatzen dira, eragiten
dieten ezerk jotzen ez duen bezain laster: ikus
[Zain dauden ezarpenak](#settings-pending).

![Ezarpenak, Audio-irteerak, Oinarrizko ikuspegia: hautatzailea, audio-sistema, lagintze-maiztasuna, bufferraren tamaina eta erreproduzitzaile bakoitzaren Main eta Cue irteerak (hemen, sistema isila)](../../images/guide/settings-outputs.png)

Goialdean, **Erakutsi** aukerak **Oinarrizkoa** edo **Aurreratua**
aukeratzen du. Oinarrizkoak audio-sistema, lagintze-maiztasuna,
bufferraren tamaina eta irteerak erakusten ditu. Aurreratuak, irteera batek
erabiltzen duen gailu bakoitzerako, bere maiztasuna eta bufferra, bit
perfect etengailua eta DSD modua gehitzen ditu, eta ondoren DSD ezarpenak.
Ikuspegiz aldatzeak errenkadak erakutsi edo ezkutatu baino ez ditu egiten:
ez da ezer aldatzen edo berrezartzen. Oinarrizkoak erabiltzen ari den
ezarpen bat ezkutatzen duenean, lerro batek hala adierazten du. Irteera
batek ere gailu bat erabiltzen ez duenean, bere maiztasuna eta bufferra,
bit perfect etengailua eta DSD modua ahaztu egiten dira aplikazioa
hurrengoz abiarazten denean: ondoren irteera batek berriro erabiltzen
badu, balio orokorretatik hasten da. Ordura arte, berriro aukeratzeak
(adibidez, bi gailu trukatu ondoren) mantendu egiten ditu.

| Ezarpena | Esanahia |
|---|---|
| Audio-sistema | Azken aukerak, **Irteerarik ez (isilik)**, ez du ezer jotzen: denbora-lerroak denbora errealeko abiaduran doaz soinu-txartelik gabe (txartelik gabeko makina baterako, edo entseatzeko). Linux: PipeWire (hura barne duten konpilazioetan), PulseAudio, JACK edo ALSA. Windows: WASAPI, ASIO (hura barne duten konpilazioetan) edo JACK. macOS: Core Audio edo JACK. Ordenagailu honetan falta diren sistemak, edo irteera-gailurik ez dutenak (abian ez dagoen JACK zerbitzari bat), erabilgarri ez gisa agertzen dira. "Sistemaren lehenetsia" aukerak ordena horretan erabilgarri dagoen lehena erabiltzen du. |
| Lagintze-maiztasuna | Irteera guztiek erabiltzen duten maiztasuna, gailu batek berea ez badu (Aurreratua); fitxategiak maiztasun horretara bihurtzen dira kalitate handiko birlaginketarekin. Bit perfect gailuak beren maiztasunean hasten dira eta ondoren fitxategiak jarraitzen dituzte. |
| Bufferraren tamaina | Audio-bloke bakoitzeko frameak, gailu batek berea ez badu; ondoriozko latentzia azpian erakusten da. Soinu-zerbitzari baten edo sistemaren nahasgailu baten bidezko irteerek (PulseAudio, ALSA `default` gailua) gutxienez 50 ms eskatzen dituzte tamaina honek dioena edozein dela ere, edo gailuak hartzen duen bezainbeste, ordenagailu lanpetu bat entzun ez dadin; latentziak orduan hala dio (`shared_buffer_ms` konfigurazio-fitxategian) |
| Irteerak erreproduzitzaileko | Erreproduzitzaile bakoitzeko, **Main** gailu bat (airerakoa) eta **Cue** gailu bat (aurrez entzuteko), bakoitza kanal-bikote batekin. Hainbat irteera-profil eskaintzen dituen soinu-txartel batek (ALSAk front, surround, direct hardware… zerrendatzen ditu) bakoitza *txartela — profila* gisa erakusten du; berdin irakurtzen jarraituko luketen bi sarrerek beren gailu-identifikatzailea jasotzen dute kortxete artean. Kanal anitzeko interfazeek hainbat erreproduzitzaile eraman ditzakete bikote desberdinetan. Aireko irteera bat (edozein erreproduzitzaileren Main edo kartutxo-panelarena) partekatzen duen Cue irteera bat baimenduta dago: azpiko ohar batek dio bere aurrez entzutea airean entzuten dela eta han zerbait jotzen ari den bitartean lehenik galdetzen duela. |
| Probatu Main / Probatu Cue | Tonu labur bat jotzen du (1 kHz Main irteeran, 440 Hz Cue irteeran, 1,5 s, −18 dBFS) aukeratutako irteeran, airera atera aurretik kableatua egiaztatu ahal izateko |
| Liburutegia | Liburutegiaren Cue irteera, bere **Probatu Cue** botoiarekin: liburutegiaren leihoko [aurrez entzutea](library.md#pre-listening) (telefono batean, Liburutegi fitxakoa) non entzuten den. Hura gabe, aurrez entzuteko barrako Play ilunduta dago. Androiden, bere aukera hutsak *Sistemaren lehenetsia* dio *Bat ere ez* ordez: aurrez entzuteak eta **Probatu Cue** botoiak irteera lehenetsian jotzen dute (ikus [Android → Liburutegia](android.md#library)). Aireko irteera batean dagoen Liburutegiko irteera batek erreproduzitzaile baten Cue-k bezala jokatzen du: azpiko ohar batek dio bere aurrez entzutea airean entzuten dela eta han zerbait jotzen ari den bitartean lehenik galdetzen duela. Oinarrizko eta Aurreratua ikuspegietan dago, telefono batean ere |
| Kartutxo-panela | Kartutxo-panelaren Main eta Cue irteerak. Main lehenespenez sistemaren irteera da. Cue irteerarik gabe, ez dago kartutxoak aurrez entzuterik; aireko irteera batean dagoen Cue batek erreproduzitzaileetan bezala jokatzen du. |
| Lagintze-maiztasuna: *gailua* (Aurreratua) | **Orokorra (...)** aukerak goiko lagintze-maiztasuna erabiltzen du; balio batek gailu honi bere maiztasuna ematen dio. Gailuak adierazten dituen maiztasunak bakarrik eskaintzen dira; gordeta dagoen eta gailuak jada adierazten ez duen maiztasun bat zerrendan geratzen da, agian ez dela irekiko dioen ohar batekin (orduan gailuak maiztasun orokorra erabiltzen du). Balio propioak irteera batek izendatzen dituen gailuei bakarrik aplikatzen zaizkie, ez sistemaren irteera lehenetsiari, irteera batek hura izendatzen ez badu. |
| Bufferraren tamaina: *gailua* (Aurreratua) | **Orokorra (...)** aukerak goiko bufferraren tamaina erabiltzen du (partekatutako irteera batek eskatzen duen gutxienekoarekin, goian bezala); balio batek gailu honi berea ematen dio, dagoen bezala erabilia, bere latentzia azpian duela. Bere bufferraren tamaina onartzen ez duen gailu batek orokorra erabiltzen du, eta baita maiztasun orokorra ere bere maiztasuna onartzen ez duenean. |
| Bit perfect: *gailua* (Aurreratua) | Berez desaktibatuta. Audiofiloen erreprodukziorako modu berezi bat, ez programa nahasietarako: gailua sarbide esklusiboarekin irekitzen da eta pista bakoitzaren lagintze-maiztasuna jarraitzen du. Bere Main irteera horretan duten erreproduzitzaileek ez dute inoiz gurutzatzen ezta nahasten ere, maiztasun-aldaketa batek isilune labur bat uzten du bi pistaren artean, eta haien bolumena % 100ean finkatuta dago. Etengailua desaktibatuta dago gailuak sarbide esklusiborik eman ezin duenean. Ikus [Zer kostatzen den](bit-perfect.md#what-it-costs). |
| DSD: *gailua* (Aurreratua) | **Bihurtu PCM** (lehenetsia), **DoP** edo, Linuxen, **DSD natiboa**. Gailu guztiek erakusten dute; gailuak onar ditzakeen moduak bakarrik eskaintzen dira, eta azpiko lerro batek besteak zergatik ez diren adierazten du. Ikus [DSD](bit-perfect.md#dsd). |
| Beste iturri batek DSD irteera bat behar duenean (Aurreratua) | **Jarraitu DSD pista PCM gisa** (lehenetsia), edo **Mantendu DSDa eta isilarazi beste iturriak**. Ikus [DSD](bit-perfect.md#dsd). |
| DSD isiltasuna (Aurreratua) | DSD fluxu bat hasi aurretik, amaitu ondoren eta PCMra aldatzean bidaltzen den isiltasuna, bihurgailua klikik gabe sinkroniza dadin; lehenespenez 200 ms, 0tik 2000ra. |

Cue irteera bat ez da inoiz Main-ek erabiltzen duen irteerara pasatzen.
Ordenagailu honek ez duen audio-sistema bateko gailu bat izendatzen duen Cue
batek "cuerik ez" esan nahi du. Aireko irteera batean dagoen Cue bat (Main
baten gailu eta kanal berberak, edo haiekin gainjarriak) baimenduta dago:
azpiko ohar batek dio bere aurrez entzutea airean entzuten dela eta han
zerbait jotzen ari den bitartean lehenik galdetzen duela. Cue irteerarik ez
duen erreproduzitzaile batek **CUE** botoia ilunduta du; gainetik pasatzean
Cue irteera bat hemen aukeratzeko esaten dizu. Gauza bera gertatzen da
kartutxo-panelaren **Aurrez entzun CUEan** aukerarekin.

Gailu bat jotzen ari dela desagertzen bada, erreproduzitzaileek beren
denbora-lerroak mantentzen dituzte, eta gailua berriro irekitzen da
itzultzen denean (ikus [Arazoen konponbidea](troubleshooting.md)).

## Erreproduzitzaileak {#players}

![Ezarpenak, Erreproduzitzaileak: erreproduzitzaile kopurua, modu lehenetsia, itzaltze-denbora, nahasketa automatikoa, cue-in eta cue-out, pista-amaierako abisua eta hizkuntza](../../images/guide/settings-players.png)

| Ezarpena | Lehenetsia | Esanahia |
|---|---|---|
| Hizkuntza | Sistema | Interfazearen hizkuntza |
| Erreproduzitzaile kopurua | 4 | Pantaila nagusiko zutabeak (ezin da kendu airean dagoen erreproduzitzaile bat). Zenbakia graduatzailea askatzean aplikatzen da; koadroan idatzitako zenbaki bat Enter sakatzean edo koadrotik irtetean aplikatzen da, eta Escape-k bertan behera uzten du |
| Modu lehenetsia | CONT | Erreproduzitzaileek hasieran duten modua |
| Itzaltze-denbora | 1000 ms | Play-k airean dagoela eta Itzaltzearekin gelditu aukerak erabiltzen dute. Main irteera bit perfect duen erreproduzitzaile batean, Itzaltzearekin gelditu aukerak bakarrik |
| Nahasketa automatikoa MIX puntuan | Aktibo | Pistak gainjartzen ditu modu jarraituan. Ez zaio aplikatzen Main irteera bit perfect duen erreproduzitzaile bati |
| Erabili cue-in eta cue-out | Aktibo | Desaktibatuta: erreproduzitzaileek pista bakoitza fitxategiaren hasieratik amaierara jotzen dute; markatzaileak gorde egiten dira eta kartutxoek beren markatzaileak erabiltzen jarraitzen dute. Iraupenek eta zerrenden guztizkoek tarte bera jarraitzen dute |
| Pista-amaierako abisua | 10 s | Atzerako kontaketa gorriz keinuka noiz hasten den |

Erreproduzitzaile baten Main irteera bit perfect denean, **Nahasketa automatikoa
MIX puntuan** aukeraren azpiko ohar batek dio erreproduzitzaile horiek ez
dutela inoiz nahasten edo gurutzatzen. Bi ezarpenak editagarri geratzen dira:
beste erreproduzitzaileei aplikatzen jarraitzen diete. Ikus
[Zer kostatzen den](bit-perfect.md#what-it-costs).

## Neurgailuak {#meters}

![Ezarpenak, Neurgailuak, gailur digitaleko neurgailua aukeratuta](../../images/guide/settings-meters.png)

Aldaketak berehala aplikatzen dira. Ezarpenek aukeratutako neurgailu motak
erabiltzen duena bakarrik erakusten dute: EBU, DIN eta VU neurgailuek beren
arauak finkatzen dituen eskala, eremu gorria eta portaera dituzte
(lerrokatze-maila bakarrik ezartzen da), eta K-System neurgailu baten
lerrokatzea bere 0 da. Ezartzen duzun balio bat gorde egiten da mota hori
berriro aukeratzen duzunerako.

| Ezarpena | Lehenetsia | Esanahia |
|---|---|---|
| Neurgailu mota | Gailur digitala | Barra nola igotzen eta jaisten den, eta bere eskala, arau baten arabera (ikus behean) |
| Igoera-denbora, Jaitsiera-abiadura | 5 ms, 11,8 dB/s | **Pertsonalizatua** motarako bakarrik. Igoera-denbora integrazio-denbora bat da: iraupen horretako tonu-zaparrada batek 2 dB gutxiago irakurtzen du; 0 balioak gailur guztiak erakusten ditu. |
| True peak | Desaktibatuta | Gailur digitala, pertsonalizatua eta K-System bakarrik. Laginen artean neurtzen du, ITU-R BS.1770 arauak argitaratzen duen 4× gainlagintze-iragazkiarekin. Bihurketaren ondoren 0 dBFS gainditzen duten gailurrak erakusten ditu, lagin-gailurreko neurgailu batek ikusten ez dituenak. Arauak baimentzen duen bezala, lagin bakarreko klik isolatu batek bere lagin-balioa baino 0,3 dB inguru gutxiago irakur dezake. |
| Eskalaren behealdea | −60 dBFS | Eskala digitalaren behealdea (gailur digitala eta pertsonalizatua). Beste neurgailuek beren arauak ematen duen tartea erakusten dute. |
| Gailurrari eutsi | 2 s | Gailur digitala, pertsonalizatua eta K-System bakarrik: mailarik altuena zenbat denboraz dagoen piztuta; 0 balioak desaktibatzen du. Programa-neurgailuek eta VUak ez dute eusterik. |
| Lerrokatze-maila | −18 dBFS | Guztiak K-System izan ezik. Eskalan markatuta (EBU R68). Hor daude baita EBUren TEST marka, DINen −9 marka eta 0 VU ere. |
| Abisua hemendik | −9 dBFS | Horia hemendik aurrera (EBUk baimendutako gehienekoa), gailur digitaleko eta neurgailu pertsonalizatuetarako |
| Arriskua hemendik | −3 dBFS | Gorria hemendik aurrera, gailur digitaleko eta neurgailu pertsonalizatuetarako. Besteak beren eskalak gorri jartzen duen lekuan jartzen dira gorri: VU 0 VU-tik aurrera, EBU eta DIN PPM baimendutako gehienekotik aurrera (EBU +9, DIN 0), K-System +4tik aurrera. |
| Ozentasunaren irakurketa | Epe laburrekoa | Neurgailuaren azpiko ozentasuna: desaktibatuta, momentukoa (azken 400 ms) edo epe laburrekoa (azken 3 s), EBU R128 |
| Ozentasun-helburua | −23 LUFS | Irakurketa berdea da ±1 LU-ren barruan (EBU R128) |

| Neurgailu mota | Araua | Portaera |
|---|---|---|
| Gailur digitala | IEC 60268-18 | Gailur guztiak berehala erakusten ditu; 20 dB jaisten da 1,7 s-tan |
| EBU PPM | IEC 60268-10 IIb mota | 10 ms inguru baino laburragoak diren gailurrek gutxiago irakurtzen dute (10 ms-ko tonu-zaparrada batek 1,6 dB inguru gutxiago, 0,5 ms-ko batek 18 dB inguru gutxiago), EBU Tech 3205en tolerantzien barruan; 24 dB jaisten da 2,8 s-tan |
| DIN PPM | IEC 60268-10 I mota | Gauza bera 5 ms-ko integrazio-denborarekin; 20 dB jaisten da 1,5 s-tan |
| VU | IEC 60268-17 | Batez besteko maila, VU neurgailu baten orratzaren mugimenduarekin: % 99 300 ms-tan, gainditze txiki batekin; sinusoide batek bere gailur-maila irakurtzen du |
| K-20, K-14, K-12 | K-System | Bi atal: batez bestekoa (RMS, 600 ms) barra bete gisa eta gailurra (26 dB jaisten da 3 s-tan) ilunduta haren gainean. 0 eskala osoaren azpitik 20, 14 edo 12 dB-ra dago; berdea 0tik behera, anbarra 0tik +4ra, gorria hortik gora. K-12 irrati-emisiorako egokia da, K-14 eta K-20 programa dinamikoagoetarako. |
| Pertsonalizatua | — | Zure igoera-denbora eta jaitsiera-abiadura |

Neurgailu bakoitzak bere arauaren eskala erabiltzen du, bere markak
kanalen artean dituela:

| Neurgailua | Eskala |
|---|---|
| Gailur digitala, pertsonalizatua | −60 … 0 dBFS, markak 10 dB-tik behin −40raino eta 5 dB-tik behin hortik gora; goiko 20 dB-ek altueraren erdia hartzen dute |
| EBU PPM | −12 … +12 lerrokatze-mailaren inguruan (TEST), 4 dB-tik behin; maila isilagoak behealdean geratzen dira |
| DIN PPM | −50 … +5, non 0 lerrokatze-mailaren gainetik 9 dB-ra dagoen (lehenespenez −9 dBFS) |
| VU | −20 … +3 VU, 0 VU lerrokatze-mailan; barra tentsioaren proportzioan mugitzen da, orratza bezala |
| K-System | +20, +14 edo +12 (0 dBFS) mailatik −60raino; dB-tan uniformea −24raino |

## Analisia {#analysis}

![Ezarpenak, Analisia: markatzaile automatikoen atalaseak](../../images/guide/settings-analysis.png)

[Markatzaileak eta nahasketa](markers-and-mixing.md) atalean deskribatutako
atalaseak; errenkada bakoitzak bere izenaren azpian lerro bateko iradokizun
bat du, balioak zer egiten duen esaten duena. **Berriro aztertu pista
guztiak** botoiak analisia berriro exekutatzen du liburutegi osorako,
fitxategi guztiak berriro irakurriz; eskuzko markatzaileak gorde egiten dira.
Abian dagoen bitartean botoia ilunduta dago eta zenbat egin duen erakusten du
(**Aztertzen… 120 / 340**), beraz berriro sakatzeak ez du hasieratik hasten;
egoera-barrako **N pista aztertzen** elementuaren argibideak aurrerapen bera
ematen du. Hemen atalase bat aldatzeak ere liburutegia berriro aztertzen du,
eta dagoeneko abian dagoen exekuzio bat berrabiarazten du, pistarik ez dadin
balio zaharren markatzaileekin geratu.

Analisia aldatu duen eguneratze baten ondoren, aurreko bertsioak aztertutako
pistek beren markatzaileak eta uhin-formak mantentzen dituzte, eta
funtzionatzen jarraitzen dute. Abiaraztean, Fauste Player-ek zenbat diren
esaten du eta **Aztertu orain** edo **Geroago** eskaintzen du; hemengo
**Aztertu zaharkitutako pistak (N)** botoiak gauza bera egiten du edozein
unetan. Erreproduzitzaileetan dauden pistak eguneratu egiten dira hala ere,
erakusten diren heinean, baita formatu erregistraturik ez duten kartutxoen
pistak ere (kartutxo bat bit perfect jotzen da bere formatua ezagutzen
denean bakarrik). Fitxategia falta duten pistak ez dira zenbatzen
fitxategia itzuli arte.

Analisiak fitxategi bakoitzaren ozentasuna ere neurtzen du
[ozentasun-normalizaziorako](loudness.md). Ozentasuna neurtu aurretik
aztertutako fitxategiak atzeko planoan neurtzen dira, analisiak beste
ezer egiteko ez duenean, normalizazioa aktibo egon edo ez; haien markatzaileei
ez zaie ukitzen. **Berriro aztertu** (zerrendaren menuan) eta **Berriro
aztertu pista guztiak** aukerek berriro neurtzen dute.

## Ozentasuna {#loudness}

![Ezarpenak, Ozentasuna: normalizazioa aktibo, helburuarekin, true peak sabaiarekin, gehieneko igoerarekin eta gehieneko doikuntzarekin, beren balio lehenetsietan](../../images/guide/settings-loudness.png)

Pista eta kartutxo guztiak ozentasun berean jotzen ditu, pista bakoitzeko
irabazi finko batekin. [Ozentasun-normalizazioak](loudness.md) azaltzen du
irabazia nola kalkulatzen den, erreproduzitzailearen goiburuko ikurra eta
pista bakoitzeko ezarpenak.

| Ezarpena | Lehenetsia | Tartea | Esanahia |
|---|---|---|---|
| Ozentasuna normalizatu | Aktibo | | Normalizazioa aktibo edo ez |
| Helburua | −16 LUFS | −36 … −6 LUFS, 0,5eko urratsak | Pista guztiak eramaten diren ozentasun integratua |
| True peak sabaia | −1 dBTP | −12 … 0 dBTP, 0,1eko urratsak | Igoera batek ez du inoiz pistaren true peak hau gainditzen. Jaitsierak ez dira mugatzen |
| Gehieneko igoera | 12 dB | 0 … 24 dB | Pista baxu batek jaso dezakeen irabazirik handiena. Jaitsierak ez dira mugatzen |
| Gehieneko doikuntza | 12 dB | 0 … 24 dB | Pista bakoitzaren doikuntza manualak noraino irits daitekeen, gora edo behera |

Aldaketak geroago kargatzen diren pistei aplikatzen zaizkie: jotzen ari dena
bere irabaziarekin jarraitzen du, eta erreproduzitzaile bakoitzean zain
dagoen hurrengo pista berriro prestatzen da balio berriarekin. Ezarpenen
azpiko ohar batek esaten du zenbat pista dauden oraindik beren neurketaren
zain; aldatu gabe jotzen dira hura amaitu arte. Bit perfect irteera batean
dagoen erreproduzitzaile edo kartutxo bat ez da inoiz normalizatzen.
**Leheneratu balio lehenetsiak** botoiak atal hau bakarrik berrezartzen du
(normalizazioa aktibo). Normalizazioa aktibo dago instalazio berrietan;
lehendik dagoen konfigurazio batek bere uneko ezarpena mantentzen du
(gordetako konfigurazio guztiek balioa gordeta dute jada).

## Zerrendak {#playlists}

![Ezarpenak, Zerrendak: musika-karpeta, zerrendak eta taulako zutabeak](../../images/guide/settings-playlists.png)

- **Musika-karpeta:** fitxategi-elkarrizketak non hasten diren.
- **Zerrenda berria**, **Aldatu izena** (editatu izena eta sakatu Enter;
  Esc teklak bertan behera uzten du) eta **Ezabatu** (zakarrontzi-ikonoa).
- **Inportatu M3U / PLS…** botoiak zerrenda berri bat sortzen du
  zerrenda-fitxategi batetik. Errenkada bakoitzeko **M3U** botoiak M3U8 gisa
  esportatzen du. Ikus [Zerrendak](playlists.md).
- **Taulako zutabeak:** pisten taulek zein zutabe erakusten dituzten eta
  zein ordenatan, erreproduzitzaile guztientzat: kontrol-lauki bat zutabe
  bakoitzeko (Izenburua eta Iraup. ezin dira desaktibatu), gora eta behera
  geziak erakutsitakoentzat, eta **Zutabe lehenetsiak**. Ikus
  [Zerrendak](playlists.md).

**Hizkuntza:** goitibeherako zerrenda bat: **Sistema** (sistema eragilea
jarraitu), eta ondoren interfazea eskuragarri dagoen hizkuntza guztiak,
bakoitza bere izenarekin (ingelesa lehenik, eta ondoren alfabetikoki:
adibidez Español). Interfazea berehala aldatzen da. Bere itzulpenik ez duen
sistema-hizkuntza batek hurbilena erabiltzen du (Kanadako frantsesak
frantsesa erabiltzen du, Brasilgo portugesak portugesa), eta bestela
ingelesa. Interfazeak ez duen ezarpen-fitxategiko hizkuntza bat **Sistema**
gisa agertzen da eta sistema eragilea jarraitzen du.

Hizkuntza guztietako interfazea, ingelesa eta gaztelania barne, AArekin sortu
da eta akatsak izan ditzake. **Fauste Player-i buruz** leihoak hizkuntza
guztiak hartzen dituen erantzukizun-ukapen orokor bat dauka. Zuzenketak ongi
etorriak dira, issue edo pull request gisa.

## Kartutxoak {#cartwall}

![Ezarpenak, Kartutxoak: orriak, sareta eta hautatutako kartutxoaren editorea](../../images/guide/settings-cartwall.png)

Orriak, saretaren tamaina, kartutxo-editorea, eta kartutxo-orrien
inportazioa eta esportazioa. Ikus [Kartutxo-panela](cartwall.md).

## Liburutegia {#library}

Sortu, aldatu izena, aldatu kolorea, aldatu mota, mugitu (arrastatuz) eta
ezabatu zure pistak taldekatzen dituzten kategoriak, kategoria batekin
karpeta bat zaindu eta bere falta diren pistak kendu, liburutegiaren leihoa
ireki gabe. Ikus [Liburutegia](library.md#categories) eta
[Zaindutako karpetak](library.md#watched-folders).

## Laster-teklak {#keyboard-shortcuts}

![Ezarpenak, Laster-teklak: erreproduzitzailearen ekintza bakoitza bere teklarekin, eta Kendu esleituta daudenen ondoan](../../images/guide/settings-shortcuts.png)

Ikus [Teklatua](keyboard.md).

## MIDI {#midi}

![Ezarpenak, MIDI, MIDI kontrola desaktibatuta](../../images/guide/settings-midi.png)

Aktibatu MIDI kontrol-gainazalak, ikusi sarrera-atakak (**Konektatu berriro
atakak** botoiak berriro irekitzen ditu, kontrolagailuak berriro konektatuko
bazenitu bezala), eta ikasi kontrol bat erreproduzitzailearen ekintza
bakoitzerako. Ikus
[MIDI kontrol-gainazalak](midi.md).

## Urrunekoa {#remote}

![Ezarpenak, Urrunekoa, HTTP APIa ordenagailu honetan entzuten](../../images/guide/settings-remote.png)

Sareko urruneko kontrola, web-orrietarako, telefono-aplikazioetarako,
automatizaziorako eta kontrol-gainazaletarako. Ikus
[Urruneko kontrola](remote-control.md).

- **Baimendu urruneko kontrola HTTP bidez**, bere **Helbidea** eta
  **Ataka**, eta entzuten ari den ala ez dioen lerro bat.
- **Tokena**, ordenagailu honetatik kanpo derrigorrezkoa. **Sortu** botoiak
  ausazko bat sortzen du, **Erakutsi** botoiak agerian uzten du, eta
  **Kopiatu** botoiak arbelean jartzen du. Abisu bat agertzen da helbidea
  ordenagailu honetatik haratago iristen denean eta tokenik ez dagoenean.
- **APIa erabil dezaketen web-orriak**: jatorri bat lerro bakoitzeko.
- **Baimendu OSC bidezko kontrola**, bere **Helbidea** eta **Ataka**, eta
  **Baimendutako igorleak** (helbideak edo azpisareak, bat lerro
  bakoitzeko).
- **Argitaratu denborak maiztasun honekin**: zerbait jotzen ari den bitartean
  igarotako eta falta den denbora zenbatero bidaltzen diren.

Testu-eremuak eta zenbakiak uztean aplikatzen dira, eta horrek beste atal bat
irekitzea edo Ezarpenak ixtea barne hartzen du; Esc teklak idazten ari
zinena bertan behera uzten du. Balio baliogabe bat zuzendu egiten da, eta
eremuak gorde dena erakusten du.
