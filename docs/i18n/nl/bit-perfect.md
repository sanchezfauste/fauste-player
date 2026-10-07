# Bit-perfect uitgang

Een **bit-perfect** apparaat ontvangt de samples van elk bestand precies zoals
ze in het bestand staan: dezelfde samplefrequentie, dezelfde waarden, zonder
resampling, volumewijziging of mixen. Dat is nuttig voor monitoringketens en
digitale verbindingen, waar elke bewerking op de computer vermeden moet
worden.

## Een apparaat bit-perfect maken {#setting-a-device-bit-perfect}

1. Kies in **Instellingen → Audio-uitgangen** het apparaat expliciet voor de
   Main-uitgang van een speler (of die van de cartwall). Een speler die op de
   systeemstandaard staat, kan niet bit-perfect worden gemaakt. Een apparaat
   dat geen enkele uitgang meer gebruikt, verliest zijn bit-perfect-schakelaar
   en zijn DSD-modus de volgende keer dat de applicatie start.
2. Kies bovenaan de sectie **Geavanceerd**. Zet onder **Instellingen per
   apparaat** de schakelaar **Bit-perfect** naast het apparaat aan.
3. Herstart de applicatie.

Het apparaat start dan op zijn eigen samplefrequentie als je daar op dezelfde
plek een hebt ingesteld, anders op de globale samplefrequentie, en volgt vanaf
daar elk bestand.

De schakelaar is uitgeschakeld wanneer het apparaat geen exclusieve toegang
kan geven.
- **Linux:** kies een ALSA-apparaat waarvan de naam met `hw:` begint. Dat is
  de geluidskaart zelf. PulseAudio, PipeWire, JACK en de ALSA-apparaten
  `default` of `plughw:` mixen of converteren, dus ze zijn nooit bit-perfect.
- **Windows:** kies het apparaat onder het systeem **WASAPI**. Het wordt in
  exclusieve modus geopend.
  - In de geluidsinstellingen van Windows moet bij de eigenschappen **Geavanceerd**
    van het apparaat *Toepassingen toestaan exclusief beheer over dit apparaat
    te nemen* aanstaan (standaard staat dat aan).
  - Terwijl het speelt, kan geen ander programma het apparaat gebruiken.
- **macOS:** kies het apparaat onder **Core Audio**. Het wordt in hog-modus
  geopend.
  - De samplefrequentie van het apparaat wordt op die van het nummer gezet, en
    zijn formaat op het breedste integerformaat dat het bij die frequentie
    biedt (de instellingen die Audio-MIDI-configuratie toont).
  - Ze worden teruggegeven wanneer de applicatie het apparaat niet meer
    gebruikt.
  - Twee apparaten met precies dezelfde naam kunnen niet bit-perfect worden
    gemaakt.

## Wat er gebeurt op een bit-perfect apparaat {#what-happens-on-a-bit-perfect-device}

- **Exclusieve toegang.** Niets anders op de computer kan op het apparaat
  spelen zolang de applicatie het gebruikt. Wordt exclusieve toegang
  geweigerd, dan speelt het apparaat nog steeds, gedeeld, en blijft de
  BP-badge uit.
- **De frequentie volgt het bestand.** Speelt er niets op het apparaat en
  begint een nummer met een andere samplefrequentie, dan wordt het apparaat
  opnieuw geopend op die frequentie.
  - Dit gebeurt wanneer je een nummer afspeelt, er een hervat dat gepauzeerd
    was geladen, voorbeluistert, of een cart start. Nummers die alleen wachten
    (het volgende nummer van elke speler) worden opnieuw voorbereid op de
    nieuwe frequentie.
  - Het heropenen duurt zo lang als het apparaat nodig heeft om te starten
    (meestal enkele tientallen milliseconden). De start komt zoveel later.
  - Terwijl er iets op het apparaat speelt, verandert de frequentie nooit. Een
    nummer met een andere frequentie dat dan begint (bijvoorbeeld een
    48 kHz-nummer waarnaar gemixt wordt na een 44,1 kHz-nummer, of een nummer
    dat start terwijl een andere speler of een cart op hetzelfde apparaat
    speelt) wordt over zijn hele lengte geconverteerd en is niet bit-perfect.
  - Weigert het apparaat een frequentie, dan behoudt het de vorige en wordt het
    nummer geconverteerd.
- **Geen bewerking, als niets erom vraagt.** De samples gaan ongewijzigd door
  zolang dit allemaal geldt:
  - het volume van de speler staat op 100 %;
  - er loopt geen fade;
  - er speelt niets anders op dezelfde uitgangen (een andere speler, een cart,
    een testtoon).

## DSD {#dsd}

Een DSD-bestand wordt normaal naar PCM omgezet afgespeeld, zoals elk ander
bestand. Een bit-perfect apparaat kan in plaats daarvan de DSD-stream
ongewijzigd ontvangen.

**De drie modi.** In de weergave Geavanceerd van Instellingen →
Audio-uitgangen heeft elk apparaat dat een uitgang gebruikt onder zijn
bit-perfect-schakelaar een keuze **DSD**:
- **Naar PCM omzetten** (de standaard): DSD wordt omgezet, zoals op elk ander
  apparaat.
- **DoP** (DSD over PCM): de DSD-bits reizen binnen 24-bit PCM-samples, die de
  meeste DSD-geschikte converters herkennen. Het werkt op elk systeem.
- **Natieve DSD** (alleen Linux): ruwe DSD, voor ALSA-apparaten `hw:` waarvan
  het stuurprogramma een DSD-sampleformaat meldt.

Alleen de modi die het apparaat aankan worden aangeboden, en een regel onder
de keuze zegt waarom de andere niet: het apparaat is niet aangesloten,
bit-perfect staat uit, het apparaat kan niet exclusief worden geopend, native
DSD vereist Linux, of het apparaat aanvaardt geen native DSD. Een modus die
is opgeslagen voor een apparaat dat hem nu niet aankan, staat als PCM, en dat
is wat er speelt; de opgeslagen modus komt terug wanneer het apparaat hem weer
aankan. Een modus, de mixinstelling of de DSD-stilte wijzigen vereist een
herstart, net als de andere uitgangsinstellingen.

**Wanneer DSD ongewijzigd naar buiten gaat.** Dit moet allemaal gelden
wanneer het nummer begint:
- het apparaat is bit-perfect, met exclusieve toegang, en zijn modus is DoP of
  native DSD;
- het nummer is DSD (DSF of DFF), mono of stereo, en is geanalyseerd (zo is
  zijn DSD-frequentie bekend);
- het volume van de speler staat op 100 %;
- er speelt niets anders op het apparaat (een andere speler, een cart, een
  testtoon);
- het apparaat aanvaardt de stream. DoP vereist een apparaatfrequentie van de
  DSD-frequentie gedeeld door 16 (176,4 kHz voor DSD64, 352,8 kHz voor
  DSD128, 705,6 kHz voor DSD256) en een 24- of 32-bit formaat. Native DSD
  vereist een apparaat dat het DSD-formaat bij die frequentie aanvaardt.

Wanneer native DSD eindigt, gaat het apparaat terug naar PCM op de frequentie
die het had vóór het DSD-nummer, omdat veel converters native DSD aannemen op
frequenties die ze niet als PCM kunnen afspelen (geen enkele converter speelt
PCM op de frequentie waarop DSD512 loopt). Een nummer dat als PCM doorgaat,
behoudt de frequentie van de DSD-stream wanneer het apparaat die als PCM
aanvaardt, en gaat anders verder op de eerdere frequentie vanaf waar het was,
zoals al het andere dat op dat apparaat speelt; een fade die daar bezig is,
eindigt meteen. Het apparaat blijft nooit op een frequentie staan die het
weigert: opent geen enkele frequentie (bijvoorbeeld doordat het apparaat op
dat moment is losgekoppeld), dan is de uitgang kwijt totdat de automatische
nieuwe poging hem heropent op de eerdere frequentie.

Anders wordt het nummer naar PCM omgezet en zegt het logbestand waarom
(bijvoorbeeld “something else plays on the device” of “the device refused
705600 Hz”). Voorbeluisteren en carts worden altijd omgezet.

Terwijl DSD ongewijzigd naar buiten gaat:
- staat in de badge in de kop **DSD** in plaats van **BP**;
- tonen de meters het niveau van de PCM-conversie van hetzelfde nummer, dus ze
  werken zoals gewoonlijk;
- moet het volume op 100 % blijven: de tooltip van de fader zegt dat. De fader
  verschuiven schakelt het nummer over op PCM (zie hieronder);
- stoppen Stop en Uitfaden het nummer meteen, zonder fade, omdat een
  DSD-stream niet kan worden gefade. Op Play drukken voor een ander nummer
  terwijl het speelt, kapt het op dezelfde manier af in plaats van over te
  faden;
- werken pauzeren en hervatten ook meteen, zonder helling.

**Stilte aan de randen.** Elke start, elk einde en elke overschakeling naar PCM
verstuurt eerst DSD-stilte (standaard 200 ms), zodat de converter zonder klik
vergrendelt. De uitzondering is een DSD-nummer dat een stream van dezelfde
soort en DSD-frequentie voortzet waarvan de stilte nog loopt: de converter is
nog vergrendeld, dus het begint zonder extra stilte. Een nummer begint dus
zoveel later, en een overschakeling naar PCM laat een gat van die lengte. Het
is **DSD-stilte** in Instellingen → Audio-uitgangen, Geavanceerd (0 tot
2000 ms).

**Wanneer een andere bron het apparaat nodig heeft.** **Als een andere bron
een DSD-uitgang nodig heeft** in Instellingen → Audio-uitgangen kiest wat er
gebeurt wanneer een andere speler, een cart of een testtoon op hetzelfde
apparaat begint (de eigen fader van de speler verschuiven is de uitzondering:
dat schakelt het nummer altijd over op PCM):
- **Het DSD-nummer als PCM voortzetten** (de standaard). De stream schakelt na
  de DSD-stilte over op PCM, en het nummer gaat omgezet verder vanaf waar het
  was. Hetzelfde gebeurt met het nummer dat vanzelf volgt (zie hieronder).
- **DSD behouden en de andere bronnen dempen.** Niets onderbreekt de
  DSD-stream. Andere bronnen die naar het apparaat gaan, worden gedempt
  totdat het DSD-nummer eindigt, en de speler toont intussen een badge
  **Andere gedempt**. Het volgende nummer van de speler zelf overlapt niet: het
  begint wanneer het DSD-nummer eindigt, zonder crossfade of segue. Een
  PCM-nummer wacht op de DSD-stilte; een DSD-nummer van dezelfde soort en
  DSD-frequentie zet de stream zonder die voort. De fader verschuiven schakelt
  het nummer nog steeds over op PCM.

**Een album blijft onder de standaardinstelling geen DSD.** Met *Het DSD-nummer
als PCM voortzetten* gaat alleen een DSD-nummer dat op een inactief apparaat
begint als DSD naar buiten. De nummers die de speler daarna zelf start (aan
het einde van een nummer, bij een segue of een crossfade) beginnen vanuit een
preload, die altijd PCM is, dus het apparaat schakelt over op PCM en ze spelen
omgezet. Een nummer dat je zelf start (Play, dubbelklik) gaat weer als DSD
naar buiten wanneer het apparaat inactief is of de vorige DSD-stream nog in
zijn stilte zit met dezelfde DSD-frequentie. Om een heel DSD-album als DSD te
houden, kies je *DSD behouden en de andere bronnen dempen*. Dan gaat elk nummer
van de speler als DSD naar buiten, en begint het volgende wanneer het vorige
eindigt.

Raakt het apparaat kwijt terwijl DSD speelt en komt het terug zonder het te
kunnen dragen (bijvoorbeeld zonder exclusieve toegang), dan gaat het nummer
als PCM verder.

## De BP-badge {#the-bp-badge}

De badge **BP** in de kop van de speler brandt zolang het huidige nummer zijn
Main-apparaat ongewijzigd bereikt. Dit moet allemaal gelden:

- het apparaat is bit-perfect en geopend met exclusieve toegang;
- het apparaat draait op de samplefrequentie van het nummer;
- het nummer is lossless integer-PCM (WAV, AIFF, FLAC, ALAC, WavPack of
  Monkey's Audio), mono of stereo, en hoogstens 24-bit, en het formaat van het
  apparaat omvat zijn samplegrootte (een 24-bit bestand op een 16-bit apparaat
  is niet bit-perfect). DSD wordt omgezet, dus laat BP nooit branden; gaat het
  ongewijzigd naar buiten (zie [DSD](#dsd)), dan staat in de badge in plaats
  daarvan **DSD**;
- het nummer is geanalyseerd, omdat daaruit zijn frequentie en samplegrootte
  bekend zijn. Nummers die een eerdere versie heeft geanalyseerd, krijgen hun
  formaat zodra ze opnieuw zijn geanalyseerd (de melding bij het opstarten, of
  Instellingen → Analyse), of zodra een speler ze toont of een cart ze bevat;
- het volume is 100 %, er loopt geen fade en er speelt niets anders op
  dezelfde uitgangen.

Sommige bestanden worden nooit als bit-perfect getoond:
- **Lossy bestanden** (MP3, AAC, Ogg Vorbis, Opus): hun gedecodeerde samples
  zijn niet de integerwaarden die een apparaat aanneemt.
- **Bestanden boven 24 bits:** de mixer werkt in 32-bit floating point, dat
  24 bits exact draagt.
- **Bestanden met meer dan twee kanalen:** ze worden naar stereo gemixt.

## Het zelf controleren {#checking-it-yourself}

Om een keten van begin tot eind te controleren:

1. Verbind de digitale uitgang van het apparaat (S/PDIF, AES of USB-loopback)
   met een recorder die bit-exact opneemt.
2. Speel een lossless testbestand af op 100 % terwijl er niets anders speelt.
3. Neem het op.
4. Vergelijk de opname met het bestand. Met SoX bijvoorbeeld keer je er een om
   en mix je ze: `sox -m -v 1 file.wav -v -1 recording.wav diff.wav`, nadat je
   hun begin hebt uitgelijnd. Elke sample van het verschil moet nul zijn.

De geautomatiseerde tests van het project controleren dezelfde eigenschap in
de applicatie, op een gesimuleerd apparaat.

### DSD op een echte converter {#dsd-on-a-real-converter}

De geautomatiseerde tests controleren DSD alleen op gesimuleerde apparaten.
DoP en native DSD zijn door het project niet op een echte converter
uitgeprobeerd. Om er een te controleren:
1. Zet het apparaat op **DoP** (of **Natieve DSD** op Linux), herstart en
   speel een DSD-bestand af op 100 % terwijl er niets anders speelt. De kop
   moet **DSD** tonen, en het eigen display van de converter zou de
   DSD-frequentie (bijvoorbeeld DSD64) moeten tonen in plaats van een
   PCM-frequentie. Een converter die een PCM-frequentie toont of ruis speelt,
   herkent de stream niet: ga terug naar **Naar PCM omzetten**.
2. Luister naar een klik of een stoot ruis bij het begin, bij Stop, aan het
   einde van het nummer en bij het verschuiven van de fader. Een klik betekent
   dat de converter een langere **DSD-stilte** nodig heeft (Instellingen →
   Audio-uitgangen, Geavanceerd).
3. Start een cart of een andere speler op hetzelfde apparaat, één keer met
   elke mixinstelling, en controleer het hierboven beschreven gedrag.
4. Om native DSD op Linux zonder de applicatie te controleren, voer je uit:
   `FAUSTE_NATIVE_DSD_DEVICE=alsa:hw:CARD=<card>,DEV=0 cargo test -p fp-backends --test conformance -- --ignored native_dsd_on_a_real_device`.
   Het opent het apparaat in native DSD op DSD64 en speelt één seconde
   DSD-stilte. Het moet slagen, en de converter zou op DSD64 moeten
   vergrendelen.
5. Met DSD-bestanden in `test-music/` speelt `cargo test --release -p fp-engine --test dsd_real_music -- --ignored`
   ze via de engine af op een gesimuleerd apparaat en vergelijkt de woorden
   met de bytes van het bestand (zie [Testen](https://github.com/sanchezfauste/fauste-player/blob/master/docs/technical/testing.md)).
