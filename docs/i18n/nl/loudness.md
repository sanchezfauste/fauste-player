# Loudness-normalisatie

Nummers komen uit veel bronnen en zijn op heel verschillende niveaus gemasterd:
een recente popsingle kan twee keer zo hard klinken als een opname uit de jaren
zeventig of een gesproken promo. **Loudness-normalisatie** speelt elk nummer en
elke cart af op dezelfde waargenomen luidheid, zodat de operator de fader niet
van het ene nummer naar het volgende hoeft te sturen.

Ze staat **aan voor nieuwe installaties**; een bestaande configuratie behoudt
haar huidige instelling (elke opgeslagen configuratie bewaart haar al). Wijzig
haar in [Instellingen → Loudness](settings.md#loudness).

![Instellingen, Loudness: normalisatie aan, met het doel, het true-peakplafond, de maximale versterking en de maximale trim op hun standaardwaarden](../../images/guide/settings-loudness.png)

## Hoe het werkt {#how-it-works}

**Een vaste gain per nummer.** Elk nummer krijgt één gain, in dB, die wordt
bepaald wanneer het nummer wordt geladen en blijft staan totdat het stopt. Er
wordt niets gecomprimeerd of begrensd: het hele nummer wordt met hetzelfde
bedrag luider of zachter gemaakt, zodat zijn eigen dynamiek blijft zoals hij is
gemasterd. De fader van de speler werkt daar bovenop, zoals voorheen.

**De meting.** De analyse op de achtergrond meet elk bestand één keer:

- zijn **geïntegreerde loudness**, in LUFS: hoe luid het hele bestand klinkt,
  zoals ITU-R BS.1770 en EBU R128 het definiëren (stille passages en stilte
  worden eruit gepoort, zodat een lange uitfade of een pauze hem niet verlaagt);
- zijn **true peak**, in dBTP: het hoogste niveau dat het signaal tussen de
  samples bereikt zodra het naar analoog wordt omgezet.

Nieuwe bestanden worden gemeten bij hun eerste analyse. Bestanden die zijn
toegevoegd voordat de loudness werd gemeten, worden gemeten door een **pass op
de achtergrond** die loopt terwijl de normale analyse niets te doen heeft, zonder
hun markers aan te raken. Ze loopt of normalisatie nu aan of uit staat, zodat de
metingen klaar zijn wanneer je haar aanzet. **Opnieuw analyseren** (in het
playlistmenu) meet een bestand opnieuw, waarbij het wordt gelezen ook als de
grootte en datum niet veranderden. Een bestand korter dan 0,4 s wordt nooit
gemeten, en een stil bestand heeft geen loudness: beide spelen ongewijzigd.

**De gain.** De gain brengt de gemeten loudness naar het **Doel**, plus de eigen
**Trim** van het nummer (zie [Per nummer](#per-track-exclude-and-trim)):

> gain = doel − gemeten loudness + trim

Een nummer dat luider is dan het doel wordt zo veel zachter gezet als nodig is.
Een zachter nummer wordt luider gezet, maar nooit met meer dan de **Maximale
versterking**, en nooit zoveel dat zijn true peak boven het **True-peakplafond**
zou komen: er is geen limiter, dus het plafond houdt een versterkt nummer van
clippen af. Een nummer waarvan de true peak al boven het plafond ligt, wordt
helemaal niet versterkt.

Voorbeelden, met de standaardwaarden (doel −16 LUFS, plafond −1 dBTP, maximale
versterking 12 dB):

| Nummer | Gemeten | True peak | Gain |
|---|---|---|---|
| Een luide moderne master | −9,0 LUFS | +0,5 dBTP | −7,0 dB |
| Een zachtere oudere master | −20,0 LUFS | −10,0 dBTP | +4,0 dB |
| Een zacht nummer met hoge pieken | −22,0 LUFS | −3,0 dBTP | +2,0 dB (begrensd door het plafond) |
| Een erg zachte opname | −35,0 LUFS | −20,0 dBTP | +12,0 dB (begrensd door de maximale versterking) |

**Wanneer de gain wordt bepaald.** Wanneer een nummer wordt geladen: voorbereid
als volgende van een speler, gestart, gepauzeerd geladen, voorbeluisterd op CUE
of gestart als cart. Een nummer dat speelt of gepauzeerd is, **behoudt zijn
gain** totdat het stopt, over zoeken en pauzes heen. Het wijzigen van de
instellingen, de meting van een nummer of zijn trim werkt de volgende keer dat
het nummer wordt geladen; het volgende nummer dat al op een speler wacht, wordt
opnieuw voorbereid met de nieuwe gain, zodat het ermee begint.

**Voorbeluisteren.** CUE en het voorbeluisteren van carts spelen een nummer af
met dezelfde gain die het on air krijgt, zodat wat je in de koptelefoon hoort
overeenkomt met het programma.

**Meters.** De meters en hun loudness-uitlezing meten na de gain, dus ze tonen
het genormaliseerde niveau. Het eigen **Loudnessdoel** van de meters
(Instellingen → Meters) kleurt alleen de uitlezing; het wijzigt geen enkele
gain.

## Bit-perfect uitgangen {#bit-perfect-outputs}

Een speler of cart waarvan de uitgang [bit-perfect](bit-perfect.md) is, wordt
**nooit genormaliseerd**: hij speelt altijd met eenheidsversterking (0 dB), wat
de instellingen ook zijn, omdat elke gain de samples zou veranderen die het
apparaat ontvangt. Met normalisatie aan toont zijn kop een amberkleurige
badge **0.0 dB** waarvan de tooltip zegt “Bit-perfect uitgang: nummers spelen
ongewijzigd.” Hetzelfde geldt voor voorbeluisteren op een bit-perfect
Cue-apparaat, terwijl de Main-uitgang van de speler, als die niet bit-perfect
is, de gain wel krijgt.

Een nummer dat al speelt (of gepauzeerd is) wanneer zijn apparaat bit-perfect
wordt gemaakt, behoudt de gain waarmee het begon, en de badge **BP** blijft
ervoor uit. Het volgende nummer speelt op 0 dB en bit-perfect. Een DSD-nummer in
die situatie speelt naar PCM omgezet met zijn gain, nooit als ongewijzigde
DSD-stream.

## De gain in de spelerkop {#the-gain-in-the-player-header}

Terwijl een speler speelt of gepauzeerd is, toont een badge links van **BP** de
gain van het nummer dat on air is. De badge heeft een rand in zijn kleur en is
altijd even breed. Houd de muisaanwijzer erboven voor de reden.

| Badge | Betekenis |
|---|---|
| **+4.0 dB**, **−7.0 dB** (blauw) | Het nummer wordt genormaliseerd met deze gain. De tooltip is de hieronder beschreven gain-uitsplitsing. Zijn de instellingen, de meting of de trim veranderd nadat het nummer was geladen, dan zegt hij alleen dat het nummer de gain behoudt die bij het laden is vastgelegd, en toont hij die gain. |
| **0.0 dB** (amber) | Het nummer speelt ongewijzigd. De tooltip is dezelfde uitsplitsing, en zijn regel **Automatische versterking** zegt waarom: het heeft nog geen meting (of is te kort om te meten), de uitgang is bit-perfect, of het nummer is van normalisatie uitgesloten. |
| Geen badge | Normalisatie stond uit toen het nummer begon, het nummer is stil, of de speler is gestopt. Een bit-perfect speler toont er ook geen terwijl normalisatie uit staat. |

De badge toont de gain die de speler vroeg toen het nummer begon.

### De gain-uitsplitsing {#the-gain-breakdown}

Overal waar het programma de gain van een nummer uitlegt, toont het hetzelfde
blok regels, met het label links en de waarde rechts: de tooltip van de
gain-badge, de loudness-pop-up (onder zijn bedieningselementen), de cart-editor
en de nummertooltip van de playlisttabel.

| Regel | Betekenis |
|---|---|
| **Geïntegreerde loudness** | De gemeten loudness van het nummer, in LUFS (of “niet gemeten”). |
| **True peak** | Zijn hoogste true peak, in dBTP (alleen wanneer gemeten). |
| **Doel** | De loudness waarop elk nummer wordt gebracht (Instellingen → Loudness). |
| **Automatische versterking** | Doel min gemeten loudness, vóór de trim en elke begrenzing, of “geen” en waarom: uitgesloten, bit-perfect uitgang, normalisatie uit, nog niet gemeten, of stil. |
| **Trim** | De gebruikte handmatige trim van het nummer, wanneer die niet 0 is (met de opgeslagen waarde en het maximum wanneer een verlaagde **Maximale trim** haar begrenst). |
| **Gewenste versterking** en **Beperkt door** | Alleen wanneer het true-peakplafond of de maximale versterking de gain tegenhield: de automatische versterking plus de trim (getoond wanneer er een trim is) en de grens die haar stopte. |
| **Resulterende versterking** | Altijd laatst, onder een lijn en vetgedrukt: de gain waarmee het nummer speelt. 0.0 dB wanneer het niet wordt genormaliseerd. |

Met normalisatie uit zegt een opmerking boven de resulterende versterking dat er
niets verandert in de uitzending en dat de trim werkt zodra ze aan staat.

## Per nummer: uitsluiten en bijstellen {#per-track-exclude-and-trim}

Sommige nummers moeten het doel niet volgen: een zender-ID die met opzet luid is
gemasterd, een testtoon, een nummer waarvan de meting niet overeenkomt met hoe
het klinkt (spraak klinkt vaak luider dan muziek bij dezelfde LUFS). Elk nummer
heeft daarvoor twee instellingen:

- **Uitsluiten van normalisatie:** het nummer speelt altijd ongewijzigd.
- **Trim:** dB die bij de gain van het nummer wordt opgeteld, van − tot + de
  **Maximale trim** (Instellingen → Loudness), in stappen van 0,5 dB. Dubbelklik
  op de schuifregelaar om hem terug op 0 te zetten. De trim is gedimd zolang het
  nummer is uitgesloten.

De schuifregelaar is een **trim**: een offset bovenop de automatische gain, dus
hij leest 0 dB voor elk nummer dat je niet hebt bewerkt. De pop-up en de
cart-editor tonen daarom de [gain-uitsplitsing](#the-gain-breakdown) bij hun
bedieningselementen: de regel **Resulterende versterking** volgt de
schuifregelaar terwijl je hem beweegt. In de pop-up staan de bedieningselementen
eerst en de uitsplitsing eronder, zodat de schuifregelaar blijft waar hij is
terwijl de uitsplitsing groeit of krimpt. Terwijl het nummer klinkt, voegt de
pop-up toe dat een wijziging geldt vanaf de volgende keer laden.

Waar je ze wijzigt:

- **Op een golfvorm:** klik met de rechtermuisknop op de golfvorm van een speler
  (of die van zijn CUE-venster) en kies **Loudness…**. De pop-up noemt het
  nummer en toont zijn meting. Hij blijft dat nummer bewerken ook als de speler
  naar een ander nummer gaat; sluit hem met zijn knop **×**. Hij sluit vanzelf
  als het nummer wordt verwijderd.
- **Op een playlistrij:** klik met de rechtermuisknop op de rij en kies
  **Loudness…**: dezelfde pop-up, voor het nummer van die rij.
- **In de cart-editor:** Instellingen → Cartwall, klik op een cart: dezelfde twee
  bedieningselementen staan onder **Andere carts stoppen bij starten**, met de
  meting onder het label.
- **Op afstand:** `PUT /api/v1/tracks/<id>/loudness` (zie
  [Bediening op afstand](remote-control.md)).

De instellingen horen bij het nummer en worden ermee opgeslagen. Een wijziging
werkt de volgende keer dat het nummer wordt geladen, zoals elke andere wijziging.

**De nummertooltip** van de playlisttabel toont de tags van het nummer en, onder
een lijn, dezelfde [gain-uitsplitsing](#the-gain-breakdown), berekend voor de
speler waartoe die lijst behoort (een bit-perfect speler speelt het
ongewijzigd).

## De instellingen kiezen {#choosing-the-settings}

- **Doel.** −16 LUFS past bij de meeste muziekprogramma’s on air en online.
  −23 LUFS (EBU R128) of −24 LUFS (ATSC A/85) laten meer headroom, maar zachte
  nummers hebben dan grotere versterkingen nodig, die het plafond en de maximale
  versterking kunnen tegenhouden. Kies een doel dat dicht bij hoe luid je
  materiaal al is ligt.
- **True-peakplafond.** −1 dBTP laat ruimte voor de conversie en voor lossy
  encoders verderop. Verlaag het wanneer een streamencoder clipt.
- **Maximale versterking.** Voorkomt dat een erg zachte opname (of een
  grotendeels stille) tot ruis wordt opgetild. 0 dB zet alleen luide nummers
  zachter.
- **Maximale trim.** Hoe ver de trim per nummer mag gaan. Haar verlagen begrenst
  al ingestelde trims zonder ze te wijzigen, totdat je de loudness van dat
  nummer opnieuw bewerkt: de bewerking slaat dan de begrensde trim op.

## Normen achter de standaardwaarden {#standards-behind-the-defaults}

- **Meting.** Geïntegreerde loudness en true peak zoals gedefinieerd door
  ITU-R BS.1770 (K-weging, gepoort).
- **Doel −16 LUFS.** Het volgt de AES-aanbevelingen voor streaming en online
  distributie. AES TD1008 (2021) vervangt de eerdere AES TD1004.1.15-10, die −16
  tot −20 LUFS voor streaming aanbeval. Het is geen waarde voor omroep: EBU R128
  gebruikt −23 LUFS en ATSC A/85 gebruikt −24 LKFS. Gebruik een van die twee
  wanneer een leveringsspecificatie het vereist (EBU R128 voor Europese
  programma-uitwisseling in de omroep, ATSC A/85 voor Amerikaanse digitale
  televisie), en −16 voor een stream of online programma.
- **True-peakplafond −1 dBTP.** De maximale true peak van EBU R128, ook gebruikt
  door de AES-aanbevelingen. Lossy encoders kunnen bij het decoderen overschieten,
  dus een stream die door een encoder met lage bitrate gaat, wil misschien een
  lager plafond zoals −2 dBTP (een gangbare praktijk, geen onderdeel van die
  normen).
- **Maximale versterking en maximale trim, 12 dB.** Productkeuzes zonder norm
  erachter: genoeg om een zachte opname op te tillen en een nummer op het gehoor
  te corrigeren, niet zoveel dat ruis wordt opgetild of de trim het doel
  tenietdoet.
- **Normalisatie staat aan voor nieuwe installaties.** De applicatie bijwerken
  behoudt de instelling die je configuratie al heeft, dus een installatie waar ze
  uit stond, blijft uit totdat je haar aanzet.
- **Het loudnessdoel van de meter** (Instellingen → Meters) is een aparte
  instelling, standaard de R128-waarde van −23 LUFS. Normalisatie gebruikt het
  niet.
