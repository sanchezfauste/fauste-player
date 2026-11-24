# Automatisering

Automatisering houdt het station on air wanneer er niemand in de studio is.
Deze versie heeft de **muziekvuller**, een speler die zelf muziek kiest, op
grond van regels die je instelt, en die nooit stopt bij gebrek aan een lijst,
en de kolom **Programmering**, die de getimede blokken van de dag uitvoert,
gemaakt van sjablonen in Instellingen.
De spelers die je al gebruikt (P1 … Pn) blijven ongemoeid: ze blijven van jou,
voor shows die je voorbereidt en live afspeelt.

## De kolom Programmering {#the-log-column}

De kolom **Programmering** speelt de **blokken** van de dag af: groepen items
met een starttijd (een uurmelding, spots, een programma). Toon hem met de tegel
**Programmering** in de bovenbalk, zoals je een speler toont.

![De kolom Programmering in bedrijf, naast de kolom Muziekvuller: de blokken van de dag op starttijd, elk met zijn stationsmelding (de klok markeert die met een vaste starttijd), spots en de rotatie die de muziekvuller daarna speelt; het nummer van de muziekvuller on air en de volgende](../../images/guide/automation-columns.png)

Zijn lijst ziet eruit als een playlist, gegroepeerd per blok. Elk blok begint
met een kopregel: zijn starttijd, hoeveel items het heeft en hoe lang ze
duren, "gewijzigd voor deze dag" wanneer je het vandaag hebt gewijzigd, en, in
amber, hoeveel het over het volgende blok heen loopt. Dan komen zijn items,
met de **Tijd** waarop elk moet beginnen. De lijst toont vandaag, het laatste
blok van gisteren en het eerste van morgen. Het item on air en het volgende
worden gemarkeerd zoals bij een speler; items die al zijn uitgezonden zijn
gedimd, en een item dat niet is uitgezonden (**gemist**) is doorgestreept. Een
klok markeert een item met een **vaste starttijd**, dat alleen op zijn tijd
speelt. Een **categorienummer** toont het nummer dat ervoor gekozen is zodra
het het volgende is (of in zijn blok na het volgende komt), met de categorie
erachter: dat is het nummer dat speelt. Heeft de categorie geen nummer dat kan
spelen (er is er geen aan toegewezen, of hun bestanden ontbreken), dan zegt de
regel dat in amber, en wordt het item gemist. Een item waarvan het bestand
onleesbaar blijkt wanneer het moet gaan spelen, wordt overgeslagen en ook als
gemist gemarkeerd. Nummers wijs je in de bibliotheek aan een categorie toe. De
regel van de **muziekvuller** zegt tot wanneer hij speelt, en staat als on air
zolang de muziekvuller voor de Programmering speelt. Een volgende waarvan de
tijd voorbij is, toont zijn tijd in amber. Sleep de lijn tussen twee
kolomkoppen om de kolommen breder of smaller te maken, zoals bij een playlist;
de sjabloon- en dagvensters gebruiken dezelfde breedtes.

### De programmering van vandaag wijzigen {#changing-todays-log}

Je wijzigt de Programmering zoals een playlist, en de wijziging geldt alleen
voor die dag: het sjabloon blijft zoals het is.

- De knoppen onder in de kolom: **+ Nummer…** opent de bibliotheek, om
  nummers naar de lijst te slepen; **+ Categorienummer…** en
  **+ Muziekvuller (rotatie)…** vragen welke en voegen het direct toe onder de
  regel die je hebt geselecteerd (klik op een regel om hem te selecteren;
  selecteer een blokkop om aan het begin van dat blok toe te voegen), of,
  zonder selectie of onder een regel die niet meer kan veranderen, aan het
  eind van het komende blok; **+ Blok…** vraagt om een tijd, met de eerste
  vrije al ingevuld. Wanneer de kolom te smal is voor allemaal, staan de knoppen
  die niet passen achter **⋯**.
- Sleep nummers uit de bibliotheek, of audiobestanden uit je bestandsbeheer,
  op een regel om ze ervoor te zetten (of erachter, op de onderste helft van de
  regel, zoals bij een playlist: zo komen ze aan het eind van een blok), en
  sleep regels binnen dezelfde dag om ze te verplaatsen. De regel die je
  toevoegt wordt geselecteerd en in beeld gebracht, zodat de volgende die je
  toevoegt eronder komt.
- Klik met rechts op een regel voor **Vaste starttijd** en **Uit de
  programmering van vandaag halen**. Klik (of klik met rechts) op een blokkop
  om aan het eind van dat blok een **Nummer…**, een **Categorienummer…** of een
  **Muziekvuller (rotatie)…** toe te voegen, of voor **Tijd wijzigen…**,
  **Blok verwijderen** en, bij een gewijzigd blok, **Terug naar het sjabloon**.
- Terwijl je over een blokkop sleept, toont de lijn eronder dat het
  neerzetten aan het begin van dat blok gebeurt; om regels aan het eind van het
  blok erboven te zetten, zet je ze neer op de onderste helft van zijn laatste
  regel.
- Wat al gespeeld heeft, het item on air en voorbije dagen kunnen niet worden
  gewijzigd. Een wijziging van het blok on air werkt na het item on air.

De kolom Programmering heeft de kop en de knoppen van een speler, en die
werken zoals bij een speler:

- **Afspelen** speelt het als volgende gemarkeerde item en zet de
  automatisering aan: elk blok speelt dan op zijn tijd, en de muziekvuller
  speelt tussen de blokken. Wanneer de klok het volgende heeft gekozen en zijn
  blok later begint, speelt tot dan de muziek. Niets start vanzelf: na elke
  start van de applicatie staat de automatisering stil tot je op Afspelen
  drukt.
- **Pauze**, **Stop** en **Uitfaden** werken op wat voor de Programmering on
  air is (zijn item, of tussen blokken het nummer van de muziekvuller) en
  stoppen de automatisering. Afspelen na Pauze gaat verder waar het was;
  tijdens de pauze verschuift het volgende niet, en bij het hervatten houdt het
  volgende blok zijn tijd. Afspelen na een Stop tijdens de muziek speelt weer
  muziek, en het volgende blok begint op zijn tijd.
- **Stop na** laat uitspelen wat on air is, en dan stopt de automatisering.
  Speelt de muziek en komt eerst de tijd van een blok, dan begint het blok op
  tijd en stopt de automatisering na zijn eerste item.
- **Vorig nummer** doet niets bij de Programmering.
- **SINGLE** maakt de Programmering handmatig: Afspelen speelt het volgende
  item, en stopt daarna; geen blok begint vanzelf.

Terwijl de muziekvuller tussen blokken speelt, staat in de kop van de
Programmering **Muziekvuller · *rotatie*** (de rotatie die hij speelt) en
blijven de knoppen werken; het nummer, zijn meter en zijn golfvorm staan
alleen in de kolom Muziekvuller.

Wanneer de automatisering na een item stopt, blijft het volgende op het item
onder het item dat speelde, en Afspelen speelt het, hoe laat het ook is: jij
kiest. Wanneer ze tijdens de muziek stopt, speelt Afspelen weer muziek tot het
volgende blok, tenzij je een ander volgende hebt gekozen. Om te wijzigen wat
hierna speelt, dubbelklik je op een item, of klik je er met rechts op en kies
je **Hierna spelen**. **Terug naar de klok**, in hetzelfde menu, zet als
volgende wat de tijd zegt.

## Sjablonen en dagen {#templates-and-days}

De Programmering speelt elke dag vanuit een **sjabloon**: een dag met
blokken. Een nieuwe installatie heeft er één, "Basis", voor elke dag.
Sjablonen maak je in Instellingen → **Automatisering**, onder **Sjablonen en
dagen** (niet op Android); de kolom Programmering wijzigt alleen vandaag, zoals
hierboven.

![Instellingen, Automatisering, Sjablonen en dagen: de sjablonen Basis en Weekend met Bewerken…, Nieuw sjabloon, Dupliceren, Verwijderen en de naam; het weekplan, Basis van maandag tot vrijdag en Weekend op zaterdag en zondag; de maand met vandaag gekozen, de datums met wijzigingen in amber, het sjabloon van de datum en Dag bewerken…](../../images/guide/settings-plans.png)

- **Sjablonen.** Kies er een en druk op **Bewerken…** (of dubbelklik erop) om
  het te openen; **Nieuw sjabloon**, **Dupliceren** en **Verwijderen** staan
  ernaast, en de naam eronder.
- **Weekplan.** Welk sjabloon elke weekdag gebruikt.
- **Dagen.** Kies een datum in de kalender (datums met wijzigingen zijn
  amber). Een datum kan een ander sjabloon gebruiken dan dat van zijn weekdag
  (een feestdag op donderdag kan dat van het weekend gebruiken). **Dag
  bewerken…** opent die datum, om zijn blokken alleen voor die datum te
  wijzigen; voorbije datums kunnen niet worden gewijzigd.

Een sjabloon of een datum opent in een eigen venster, dat Instellingen sluit.
Het is dezelfde lijst als de kolom Programmering, en je wijzigt hem op
dezelfde manier: de knoppen onderaan (Nummer…, Categorienummer…, Muziekvuller
(rotatie)…, Blok…, die toevoegen onder de regel die je selecteert), nummers
die je uit de bibliotheek (of bestanden die je uit je bestandsbeheer) op een
regel sleept, regels die je sleept om ze te verplaatsen, het rechtsklikmenu
van de regels en het menu van de blokkoppen. Het speelt nooit, en kan open
blijven naast de bibliotheek. Het venster van vandaag toont wat speelde en wat
speelt, zoals de Programmering; het venster van een dag sluit om middernacht,
wanneer die dag niet meer kan veranderen.

![Een sjabloon geopend in zijn venster: de blokken van het sjabloon Basis op starttijd, elk met zijn stationsmelding, zijn spots en daarna de muziekvuller, en Toevoegen onderaan](../../images/guide/template-popup.png)

Een **categorienummer** in een sjabloon speelt een nummer uit die categorie,
elke keer een ander. Het item **Muziekvuller** kiest de rotatie die de
muziekvuller na het blok speelt (de standaardrotatie wanneer het blok er geen
heeft). Wanneer de items van een blok langer duren dan de tijd tot het
volgende blok, zegt zijn kop in amber hoeveel; je kunt het toch zo laten.

Een wijziging van een datum is op zijn blok gemarkeerd met "gewijzigd voor
deze dag", en **Terug naar het sjabloon**, in het menu van de blokkop, maakt
haar ongedaan. Latere wijzigingen van het sjabloon bereiken nog steeds de
blokken die je op die datum niet hebt gewijzigd. Een wijziging van een sjabloon
terwijl een van zijn blokken on air is, laat dat blok voor vandaag zoals het
is (het toont "gewijzigd voor deze dag"); de wijziging bereikt het vanaf de
volgende dag.

## De muziekvuller {#the-music-fill}

De muziekvuller is de kolom **Muziekvuller**. Hij is verborgen tot je hem
toont: klik op de tegel **Muziekvuller** links in de bovenbalk. Hij staat links
van P1.

Hij ziet eruit en werkt als een speler: de grote klok, de golfvorm, de
transportknoppen, de fader en de nummerlijst. Het verschil is de lijst:

- hij **vult zichzelf** vanuit een **rotatie** (hieronder), altijd ongeveer
  een half uur vooruit (*vooruit*, in Instellingen → Automatisering);
- hij **kort zichzelf in**: hij bewaart de laatste 50 nummers die hij speelde
  en verwijdert oudere, zodat hij weken kan draaien;
- vóór een blok van de programmering kan hij **nummers die hij zelf koos
  ruilen** voor andere waarvan de lengte past in de resterende tijd (zie
  hieronder); de nummers die jij toevoegde worden nooit aangeraakt;
- je kunt hem nog steeds **met de hand wijzigen**: nummers uit de bibliotheek
  slepen, regels herschikken, een nummer verwijderen dat je niet wilt. Een
  regel die je toevoegt, wordt met een hand gemarkeerd en nooit vervangen; een
  regel die je verwijdert, telt niet als gespeeld.

Zijn uitgangen stel je in bij Instellingen → Audio-uitgangen, op de regel
**Muziekvuller**, zoals voor elke speler. Hij heeft geen nummer, geen
sneltoetsen en geen MIDI-koppelingen, en afstandsbediening ziet hem niet.

Druk op **Afspelen** om hem te starten en op **Stop na** om het nummer on air
te laten uitspelen en te stoppen. Zoals elke speler start hij nooit vanzelf,
ook niet na een herstart. Terwijl hij tussen blokken voor de Programmering
speelt, stopt hem stoppen of pauzeren ook de automatisering, en zijn Stop na
stopt de automatisering wanneer het nummer eindigt; komt eerst de tijd van een
blok, dan speelt het blok en wordt de markering gewist. Nadat de
Programmering tijdens de muziek is gepauzeerd, speelt Afspelen in de kolom
Muziekvuller het nummer op zichzelf: Afspelen van de Programmering hervat het
niet meer, en zijn Stop stopt het niet meer.

Boven zijn lijst, waar een speler zijn playlisttabbladen heeft, toont de kolom
Muziekvuller de **rotatie** die hij speelt. Kies daar een andere: de nummers
die hij in de wachtrij had worden vervangen, die je zelf toevoegde blijven.
Wanneer de rotatie niets kan vinden om te spelen (zijn categorieën zijn leeg,
of elk bestand ontbreekt), zegt de balk dat.

## Rotaties {#rotations}

Een **rotatie** is een korte lijst **stappen** die de muziekvuller om de beurt
speelt, en dan weer vanaf de eerste. Elke stap noemt waar zijn nummers vandaan
komen (een categorie van de bibliotheek met haar subcategorieën, of elk nummer
van een soort) en hoeveel hij er elke keer speelt, als bereik:

| Stap | Speelt |
|---|---|
| Current hits | 2–3 |
| Station IDs | 1 |
| Gold | 0–1 |

Met een bereik komt een stationsmelding de ene keer na twee nummers en de
volgende keer na drie. Een bereik vanaf **0** laat een stap sommige beurten
overslaan: met Gold op 0–1 speelt er soms een gouden oudje na de
stationsmelding en soms niet. Het maximum is altijd minstens 1. Eén rotatie is
de **standaard**: een nieuwe installatie heeft er één, **Muziek**, die al je
muziek speelt.

Een stap waarvan de categorie geen nummer heeft dat kan spelen (een lege
categorie, of bestanden die ontbreken) speelt op zijn beurt een nummer van de
standaardrotatie (willekeurige muziek, als de stap van de standaardrotatie
zelf is), zodat de andere stappen hun plaats houden: een stationsmelding komt
nog steeds tussen de nummers.

Rotaties maak je in Instellingen → Automatisering:

![Instellingen, Automatisering: de rotaties Music (de standaard) en Hits, de geselecteerde Music met zijn ene stap over alle muziek, zijn aantallen Minstens en Hoogstens, en het begin van Sjablonen en dagen](../../images/guide/settings-automation.png)

## Hoe hij kiest {#how-it-chooses}

Binnen een stap geeft de muziekvuller de voorkeur aan de nummers die het
langst niet zijn uitgezonden, met een beetje toeval ertussen zodat de volgorde
nooit een vaste cyclus wordt. Hij houdt ook uit elkaar:

- **hetzelfde nummer:** het mag weer spelen zodra 3 uur voorbij is of zodra
  de helft van de andere nummers van zijn stap sindsdien is uitgezonden, wat
  het eerst komt;
- **dezelfde artiest:** niet binnen 30 minuten.

Alles wat on air ging telt mee, van de muziekvuller, een speler of de
cartwall; voorbeluisteren niet. Wanneer een kleine categorie de regels niet kan
volhouden, geeft de muziekvuller toe in plaats van te verstommen: eerst bij de
artiest, dan bij het nummer. Instellingen → Automatisering heeft alle waarden,
en is waar je rotaties maakt, hernoemt en wijzigt (zie
[Instellingen](settings.md#automation)).

Vóór een blok van de programmering kiest de muziekvuller ook zijn laatste
nummers op lengte, zodat de muziek netjes eindigt: het nummer dat uitfaadt
wanneer het blok begint, speelt op de tijd van het blok nog en verliest daarna
hoogstens 30 seconden, en er begint geen nummer dat korter dan 45 seconden zou
spelen. Ook de regels voor artiest en nummer houdt hij aan wanneer hij kan.
Wanneer geen nummer van de rotatie beide kan, speelt het nummer dat het minst
verliest, zodat er soms een paar seconden van een nummer te horen kunnen zijn.
Een pauze van de muziek verschuift de nummers erna, en de muziekvuller kiest
ze opnieuw. De muziekvuller vermijdt ook de nummers, en de artiesten, die een
blok kort daarna speelt. Beide waarden kunnen voorlopig alleen in het
configuratiebestand worden gewijzigd (`automation.max_fade_cut_secs` en
`automation.min_song_play_secs`).

Een **Uurmelding** is een eigen soort nummer (Bibliotheek → Soort ▸
Uurmelding), voor de identificatie van het station op het hele uur.
