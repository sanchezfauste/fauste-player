# Markatzaileak eta nahasketa

Pista bakoitzak bost **markatzaile** izan ditzake gehienez, segundotan:

| Markatzailea | Esanahia | Nola ezartzen den |
|---|---|---|
| Cue-in | Erreprodukzioa non hasten den | Automatikoki: mozketa-atalasetik gorako lehen soinuaren aurretxoan |
| Cue-out | Pista non amaitzen den | Automatikoki: mozketa-atalasetik gorako azken soinuaren ondotxoan |
| MIX (segue-aren hasiera) | Modu jarraituan hurrengo pista non hasten den | Automatikoki (ikus behean) |
| Outroaren hasiera | Pistaren amaiera non hasten den | Automatikoki (ikus behean) |
| Introaren amaiera | Gainetik hitz egiteko sarreraren amaiera | Eskuz, edo fitxategiko `INTRO` etiketa batetik |

Eskuz ezarritako markatzaileek irabazten dute beti: analisi berri batek ez
ditu inoiz ordezten.

## Markatzaileak editatzea {#editing-markers}

Erreproduzitzaile baten uhin-forman, edo bere CUE leihoaren uhin-forman
(menu eta helduleku berberak; aldaketa bat bietan agertzen da aldi berean):

- **Egin eskuineko klik** markatzaile bat nahi duzun lekuan eta aukeratu
  **Ezarri cue-in hemen**, **Ezarri introaren amaiera hemen**, **Ezarri
  outroaren hasiera hemen**, **Ezarri MIX puntua hemen** edo **Ezarri
  cue-out hemen**. **Berrezarri markatzaile automatikoak** aukerak jarri
  dituzun markatzaileak kentzen ditu, eta pista berriro aztertzen da.
- **Eutsi Alt** (Option macOSen): heldulekuak agertzen dira markatzaileetan.
  Arrastatu bat mugitzeko; denbora erakusten da arrastatzen duzun bitartean.
  Arrastatze batek ez du inoiz erreprodukzio-burua mugitzen.

Cue-in puntuak cue-out puntuaren aurretik egon behar du beti. Beste
markatzaileak bien artean mantentzen dira. Airean dagoen pistaren
aldaketak berehala aplikatzen zaizkio bere hurrengo trantsizioari.

## INTRO etiketa {#the-intro-tag}

Fitxategi batek bere intro-denbora `INTRO` etiketa batean eraman dezake,
segundotan (`12.5`) edo `m:ss` gisa. Izena maiuskulaz edo minuskulaz idatz
daiteke (`INTRO`, `Intro`). ID3v2 erabiltzaile-testuko frame bat (MP3, WAV,
AIFF, DSF), Vorbis, Opus edo FLAC iruzkin bat, APE elementu bat (WavPack,
Monkey's Audio) edo MP4 atomo libre bat izan daiteke. Analisian irakurtzen
da. Eskuzko introaren amaierak irabazten du hala ere.

## Markatzaile automatikoak nola aurkitzen diren {#how-the-automatic-markers-are-found}

Analisiak pistaren gailurrak neurtzen ditu 10 ms-ko urratsetan, eta bere
ozentasuna leiho laburretan (lehenespenez 50 ms).

- **Cue-in / cue-out:** hasierako eta amaierako ia-isiltasuna bakarrik
  saltatzen da: gailurra *mozketa-atalasera* (lehenespenez −60 dBFS)
  iristen den guztia gordetzen da, edozein kanaletan, inguruan *mozketa-
  tarte* batekin (lehenespenez 20 ms). Hasierako itzaltze leunak, isats
  isilak eta soinu laburrak ez dira inoiz mozten.
- **MIX:** analisiak pista oraindik bere ohiko ozentasunaren azpitik
  *segue-rako maila-jaitsiera* (lehenespenez 15 dB) baino gutxiago dagoen
  azken puntua aurkitzen du; horrela, itzaltze bera duten master ozenak eta
  isilak modu berean nahasten dira. Puntu hori ez dago inoiz cue-out
  puntuaren aurretik *nahasketaren gehieneko iraupena* (lehenespenez 4 s)
  baino gehiagora, gainjartzeak laburrak izan daitezen.
- **Outroa:** analisiak cue-out puntutik atzerantz miatzen du, eta maila
  pistaren mediana-ozentasunaren azpitik *outroaren maila-jaitsiera*
  (lehenespenez 6 dB) baino gehiago jaisten den lekua aurkitzen du. Outroa
  ez da inoiz lehenespenez 30 s baino luzeagoa.
- *Nahasketa- eta outro-markatzaileen gutxieneko iraupena* (lehenespenez
  60 s) baino laburragoak diren pistek, jingleek eta iragarkiek esaterako,
  ez dute ez MIX ez outrorik jasotzen.

### Grabazio luzeak {#long-recordings}

Programa oso bat (ordu bat, lau edo gehiago) abesti bat bezala aztertzen
da, behar bada jotzen ari den bitartean: 4 orduko FLAC edo Opus fitxategi
batek minutu bat baino gutxiago behar du gaur egungo ordenagailu batean, eta
memoria ez da hazten iraupenarekin. Edozein puntutara jauzi egitea,
amaieratik gertu ere, berehalakoa da. Uhin-forma eta markatzaileak
analisiaren cachean gordetzen dira 16 ordu inguruko audioraino; fitxategi
luzeago batek ere funtzionatzen du, baina berriro aztertzen da Fauste
Player abiarazten den bakoitzean.

Balio horiek guztiak **Ezarpenak → Analisia** atalean daude. Aldatu
ondoren, pistak berriro aztertzen dira automatikoki.

## Erreproduzitzaileak zer egiten duen haiekin {#what-the-player-does-with-them}

- **Modu jarraitua nahasketa automatikoa aktibo dagoela:** MIX puntuan
  hurrengo pista maila osoan hasten da, eta unekoa itzaltzen doa bere
  cue-out puntura arte. Gainjartzea laginaren zehaztasunekoa da.
- **Modu jarraitua MIX punturik gabe,** edo nahasketa automatikoa
  desaktibatuta dagoela: hurrengo pista zehazki cue-out puntuan hasten da,
  tarterik gabe.
- **Modu bakarra**, edo **Gelditu uneko pistaren ondoren**: erreproduzitzailea
  cue-out puntuan gelditzen da.
- **Play sakatzea airean dagoela:** hurrengo pista berehala hasten da, eta
  unekoa *itzaltze-denboran* (lehenespenez 1 s) itzaltzen da.
- **Erabili cue-in eta cue-out desaktibatuta** (Ezarpenak →
  Erreproduzitzaileak): erreproduzitzaile guztiek pista bakoitza 0tik
  fitxategiaren amaierara jotzen dute. Cue-in eta cue-out, automatikoak eta
  eskuzkoak, gorde egiten dira, eta uhin-formak marra ilun gisa marrazten
  ditu. MIX puntuak, introak eta outroak funtzionatzen jarraitzen dute,
  fitxategi osoaren barruan; **Nahasketa automatikoa MIX puntuan** aparteko
  etengailu bat da. Atzerako kontaketek, iraupen-zutabeak, Ezarpenetako
  zerrenden guztizkoek eta urruneko APIaren denborek tarte bera jarraitzen
  dute. Kartutxoek beti beren cue-in eta cue-out erabiltzen dituzte.
  Ezarpena aldatzeak ez du inoiz jotzen ari den pista bat berrabiarazten,
  hartan jauzi egiten edo gelditzen; hurrengo pista berriro prestatzen da.

Pista bat bere analisia amaitu aurretik jo daiteke. Ordura arte,
fitxategiaren hasieratik amaierara jotzen da, MIX punturik gabe.
