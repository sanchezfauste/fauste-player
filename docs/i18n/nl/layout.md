# Indeling

Richt het hoofdvenster in voor je dienst: geef de spelers en de cartwall de
formaten die je wilt, zet de cartwall onder de spelers of ernaast, verberg de
panelen die je niet gebruikt en maximaliseer er even een. De indeling wordt met
de sessie opgeslagen en komt bij de volgende start terug; een nieuwe
installatie begint met alle spelers even breed en de cartwall eronder.

## Panelen tonen en verbergen {#show-and-hide-panels}

De tegels links in de bovenbalk, **P1** … **Pn** en **Cartwall**, laten zien
welke panelen op het scherm staan: een verlichte tegel is getoond. Klik op een
tegel om zijn paneel te verbergen of te tonen. Bij veel spelers scrollen de
tegels zijwaarts (scroll met het muiswiel erboven) en blijft de
indelingsknop in beeld. De andere panelen nemen de ruimte in die vrijkomt, en
een speler die opnieuw wordt getoond, komt terug op zijn breedte.

Verbergen haalt een paneel alleen van het scherm. Een verborgen speler blijft
spelen, en zijn cijfertoets, MIDI, de API op afstand, zijn CUE-venster en
automatisering bedienen hem nog steeds. Een tegel toont de toestand van zijn
paneel, of het paneel nu getoond of verborgen is: een rode stip terwijl een
speler speelt (of de cartwall terwijl een cart speelt) en twee amberkleurige
balkjes, zoals een Pauze-knop, terwijl een speler gepauzeerd is.

Een speler verbergen wist zijn geselecteerde nummer, zodat **Delete** geen
nummer kan verwijderen dat je niet ziet. Bestanden die buiten een lijst op het
venster worden neergezet, gaan naar de speler die je het laatst gebruikte, of
naar de eerste getoonde speler; als elke speler verborgen is, worden ze
geweigerd en zegt de statusbalk dat.

De tegel **Bibliotheek**, erna, opent en sluit het [bibliotheekvenster](library.md),
dat boven de panelen zweeft; het is geen paneel, dus de regel hieronder over de
laatste verlichte tegel geldt er niet voor. Op een telefoon wordt hij niet
getoond.

Minstens één paneel blijft getoond: op de laatste verlichte tegel kan niet
worden geklikt. Zolang de cartwall verborgen is, doet de sneltoets die hem
in- of uitklapt niets; klik op **Cartwall** om hem te tonen.

## De spelers van grootte veranderen {#resize-the-players}

Sleep de ruimte tussen twee spelers om breedte van de ene naar de andere te
verplaatsen. De andere spelers bewegen niet, en geen speler wordt smaller dan
zijn bedieningselementen nodig hebben. Dubbelklik op de ruimte om elke speler
weer even breed te maken.

Is het venster te smal voor elke getoonde speler op zijn smalst, dan scrolt de
rij met spelers zijwaarts, zoals altijd.

## De cartwall verplaatsen en van grootte veranderen {#move-and-resize-the-cartwall}

De cartwall staat onder de spelers, of links of rechts in het venster op volle
hoogte: kies **Cartwall onderaan**, **Cartwall links** of **Cartwall rechts** in
[het indelingsmenu](#the-layout-menu). Onder de spelers is hij zo hoog als zijn
rijen nodig hebben, hoogstens zes tienden van de hoogte tussen de bovenbalk en
de statusbalk; ernaast neemt hij eerst drie tienden van de breedte in.

De cartwall is nooit smaller dan zijn kop (**Alles stoppen**, zijn titel en een
paginatabblad) of dan zijn kolommen met knoppen op hun kleinst. Is het venster
te smal voor hem en elke getoonde speler op zijn smalst, dan scrolt de rij met
spelers zijwaarts.

Sleep de ruimte tussen de cartwall en de spelers om hem groter of kleiner te
maken: omhoog of omlaag wanneer hij onder de spelers staat, zijwaarts wanneer
hij ernaast staat. De spelers behouden de ruimte die hun bedieningselementen
nodig hebben. Dubbelklik op de ruimte om terug te gaan naar het beginformaat:
zo hoog als zijn rijen nodig hebben onder de spelers, drie tienden van de
breedte ernaast. Het formaat wordt met de indeling opgeslagen; de cartwall
verplaatsen laat hem opnieuw op zijn beginformaat beginnen.

Zodra je de cartwall van grootte verandert, of terwijl hij naast de spelers
staat, groeien zijn knoppen mee om hem te vullen; de namen en tijden behouden
hun grootte, in het midden van elke knop. Een naam breekt af tussen zijn
woorden; een woord dat te lang is voor zijn knop eindigt op een weglatingsteken
(…) in plaats van over regels te breken. Is de cartwall kleiner dan zijn rijen
en kolommen nodig hebben bij de kleinste knoppen, dan scrolt hij.

Klik op **CARTWALL** om de cartwall in te klappen. Onder de spelers blijft alleen
zijn kop staan; ernaast wordt hij een smalle strook zo hoog als het venster, met
een pijl om hem weer uit te klappen en, terwijl een cart speelt, een rode stip.
Uitgeklapt heeft hij zijn formaat terug. Zolang hij ingeklapt is, doet de ruimte
ernaast niets.

## Een paneel maximaliseren {#maximize-a-panel}

De kop van elke speler, en die van de cartwall, eindigt met twee kleine knoppen:
**Verbergen** (een min) en **Maximaliseren** (hoeken die naar buiten wijzen;
**Herstellen**, hoeken die naar binnen wijzen, op het gemaximaliseerde paneel).
Houd de muisaanwijzer erboven voor de naam. Ze zijn gedimd terwijl de indeling
vergrendeld is, **Verbergen** op het laatste getoonde paneel, en
**Maximaliseren** met één getoond paneel en op een ingeklapte cartwall. Dezelfde
twee acties staan in het menu van een rechtsklik op de achtergrond van de kop
(niet op een van zijn knoppen):

- **Maximaliseren** geeft dat paneel alle ruimte: de andere panelen krimpen tot
  hun kleinst. Het is alleen voor even en wordt niet opgeslagen. **Herstellen**
  in hetzelfde menu beëindigt het, en dat doen ook een paneel verbergen of
  tonen, de indeling vergrendelen, de cartwall verplaatsen, het aantal spelers
  wijzigen en een ruimte tussen panelen slepen of dubbelklikken; de formaten van
  daarvoor komen terug. Een ingeklapte cartwall kan niet worden gemaximaliseerd,
  en een gemaximaliseerde cartwall inklappen beëindigt het.
- **Verbergen** verbergt het paneel, zoals zijn tegel in de bovenbalk doet.

## Het indelingsmenu {#the-layout-menu}

De indelingsknop, rechts van de tegels, opent een menu met:

- **Cartwall onderaan**, **Cartwall links**, **Cartwall rechts**: waar de
  cartwall staat;
- **Indeling vergrendelen**: zolang dit is aangevinkt, verandert niets de
  indeling per ongeluk: de tegels, de andere items van dit menu, de ruimtes
  tussen de panelen en **Maximaliseren** en **Verbergen** van de koppen doen
  niets. Afspelen, en de cartwall in- of uitklappen, werken zoals gewoonlijk. De
  vergrendeling wordt met de indeling opgeslagen;
- **Spelers even breed**: elke speler even breed;
- **Indeling terugzetten**: terug naar de indeling van een nieuwe installatie.
