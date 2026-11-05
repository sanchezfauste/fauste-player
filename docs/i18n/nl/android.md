# Android

Fauste Player draait ook op Android-telefoons en -tablets (Android 10 of
nieuwer), op dezelfde playout-kern als de desktop: de spelers, playlists, de
cartwall, markers, loudness-normalisatie en elke afspeelregel werken op dezelfde
manier.

## Installeren {#install}

Een release die de app bevat, heeft `fauste-player-<version>-android.apk` op zijn
pagina **Releases** (met zijn `.sha256`). Open het op de telefoon en sta je
browser of bestandsbeheerder toe apps te installeren wanneer Android erom vraagt.
Installeer om bij te werken de nieuwere APK op dezelfde manier: je playlists en
instellingen blijven. De app staat nog niet in Google Play.

## Eerste start {#first-start}

- **Toegang tot muziek.** Sta toegang tot muziek toe (en tot meldingen, op
  Android 13 en nieuwer). Zonder dat blijft de muziekbrowser leeg.
- **De playout gaat door.** Terwijl een speler of een cart speelt, blijft er een
  melding staan en gaat de audio door met het scherm uit of de app op de
  achtergrond. Zoals op de desktop gaat er na een start of een crash niets uit
  zichzelf on air.
- **Terug en sluiten.** Terug sluit eerst wat open is (de muziekbrowser,
  Instellingen, Over, een geopende speler, een pagina van het tabblad
  Bibliotheek). Is er niets meer te sluiten, dan legt Terug de app weg terwijl er
  iets klinkt (een speler, een cart of een CUE-voorbeluistering) en blijft het
  spelen; klinkt er niets, dan slaat het op en sluit het de app. De app wegvegen
  in de appwisselaar doet hetzelfde: een veegbeweging stopt nooit wat speelt, en
  de app sluit zodra ze stil valt.
- **Geluid van andere apps.** Wanneer een andere app de audio overneemt, blijft
  Fauste Player spelen en zegt dat in de statusbalk onderaan.

## Het scherm {#the-screen}

Op een **telefoon** heeft de bovenbalk vier tabbladen, **Live**, **Cartwall**,
**Playlists** en **Bibliotheek**, getoond als pictogrammen (het gekozen ook met
zijn naam wanneer er ruimte is), dan het **hangslot** en **⋮** (Meer:
Instellingen en Over). De klok verschijnt wanneer er ruimte voor is. Zijwaarts
gedraaid zet het scherm Playlists zijn spelerchips (P1, P2…) in de bovenbalk, en
toont de statusbalk onderaan zich alleen wanneer hij iets te melden heeft.

<p>
<img src="../../images/android/1-live.png" width="180" alt="Live op een telefoon: twee spelers als kaarten, een hele en het begin van de volgende, en de cartwall eronder">
<img src="../../images/android/2-carts.png" width="180" alt="Cartwall op een telefoon: de pagina General rechtop gedraaid, twee carts per rij">
<img src="../../images/android/3-playlists.png" width="180" alt="Playlists op een telefoon: de chips P1 tot P4, de playlisttabbladen, de nummerlijst en de kaart van de speler onderaan">
<img src="../../images/android/4-music.png" width="180" alt="Muziek op het apparaat op een telefoon: de map Demo met drie aangevinkte nummers en de balk om ze toe te voegen aan de playlist Morning van P1">
<img src="../../images/android/5-library.png" width="180" alt="Het tabblad Bibliotheek op een telefoon: de nummers van de categorie Music, elk met een vakje om aan te vinken en een koptelefoon om voor te beluisteren, en daaronder de voorbeluisterkaart met de golfvorm van het nummer en zijn markers">
</p>

- *Live:* elke speler als kaart, en de cartwall. Elke kaart heeft, van boven naar
  beneden, de naam van de speler, de status, Single/Cont en CUE; de hoes, de
  titel en artiest en het volgende nummer; de golfvorm (een tik zoekt); dan de
  verstreken tijd en de lengte van het nummer met de resterende tijd in grote
  cijfers boven het transport (Play/Volgende, Vorige, Pauze, Stop, Opnieuw
  starten, Uitfaden, Stop na huidige) met knoppen ter grootte van een vinger, en
  een slanke meter met de volumefader (het niveau van de fader staat eronder). De
  resterende tijd eindigt waar de knoppen eindigen, kleiner voor een lang nummer.
  Op de meeste telefoons toont de meter zijn schaal links ervan, en op een
  bredere (ongeveer 406 dp en meer) toont de fader ook zijn niveaus; op een
  smalle telefoon (onder ongeveer 384 dp) doet geen van beide dat, en worden de
  knoppen iets kleiner zodat ze passen. De knoppen werken terwijl een andere
  vinger de fader vasthoudt, en een tik telt ook als de vinger een beetje rolt of
  even blijft liggen. Rechtop gehouden staan de kaarten bovenaan, een hele kaart
  en het begin van de volgende (scroll voor meer), en de cartwall eronder.
  Zijwaarts gedraaid is elke speler een rij links (zijn naam en status, de titel,
  de resterende tijd, Play/Volgende, Pauze en Stop; tik elders op de rij om de
  speler te openen op Playlists) en staat de cartwall rechts. Tik op
  **CARTWALL** in zijn kop om hem op te vouwen: alleen de kop blijft, onderaan
  (de paginatabbladen en *Alles stoppen* werken nog), en de spelers krijgen de
  ruimte; tik er nogmaals op om uit te vouwen. Het tabblad Cartwall toont altijd
  de hele cartwall. Een cartpagina die te breed is voor de ruimte wordt in minder
  kolommen getoond, in dezelfde volgorde.
- *Cartwall:* de getoonde cartpagina, het scherm vullend. Veeg zijwaarts om de
  volgende of de vorige pagina te tonen. Rechtop gehouden wordt een pagina die
  breder is dan hoog gedraaid (8 × 2 wordt als 2 × 8 getoond), in dezelfde
  volgorde.
- *Playlists:* de chips bovenaan (P1, P2…) kiezen de speler; eronder staan zijn
  playlisttabbladen (+ maakt een nieuwe), de nummerlijst en, onderaan (rechts als
  de telefoon op zijn zij ligt), de kaart van de speler met een grotere golfvorm
  om een nummer voor te bereiden (ze wordt korter als er weinig ruimte is, zodat
  de knoppen altijd te zien zijn). Ingedrukt houden op een rij opent zijn menu,
  dat ook **Omhoog**, **Omlaag** en **Verplaatsen naar een andere speler** heeft
  (het nummer komt direct na het volgende nummer van die speler, in de playlist
  die hij toont; het nummer dat on air is blijft). Sleep een rij aan zijn
  ≡-handvat om hem te verplaatsen. De voettekst heeft **Toevoegen** en, achter
  **⋯**, **Gespeeld wissen**. Ingedrukt houden op de golfvorm zet op dat punt een
  marker (cue-in, einde intro, begin outro, MIX, cue-out) of verwijdert er een
  die je zette; markers worden alleen hier bewerkt, niet op Live.
- *Bibliotheek:* de nummerbibliotheek en de muziek van het apparaat (zie
  [Bibliotheek](#library)).

Terug vanaf een ander scherm gaat naar Live; op Bibliotheek gaat het eerst terug
naar de lijst met categorieën (op Muziek op het apparaat eerst een map omhoog).

Wanneer er niets beweegt op het scherm, tekent de app het eenmaal per seconde
(voor de klok) om de batterij te sparen; een meter die na een pauze of een stop
nog daalt, houdt het tekenen gaande totdat de meter beneden is.

**Het hangslot** beschermt tegen verdwaalde aanrakingen tijdens een show.
Vergrendeld (het hangslot dicht en verlicht) tonen de volumefaders alleen het
niveau en zoeken de golfvormen niet en bewerken geen markers; ze zien er gedimd
uit. Samenknijpen en slepen zoomen en verplaatsen een golfvorm nog steeds, en
Play, het transport, de carts en de playlists werken altijd. De vergrendeling
blijft aan na een herstart en wordt met de sessie opgeslagen, dus een andere
sessie laden neemt de vergrendeling van die sessie over; de API op afstand en
MIDI zijn niet vergrendeld.

Een sleepbeweging die op een volumefader begint, verplaatst hem; een die elders
begint, scrolt, zodat scrollen nooit een niveau on air verandert. Een tik op een
golfvorm zoekt daar, knijpen met twee vingers erop zoomt rond je vingers, slepen
met één vinger verplaatst een ingezoomde weergave en **Volledig beeld** toont het
hele nummer weer. In een playlist scrolt een vinger; om een nummer te verplaatsen
gebruik je zijn ≡-handvat of het rijmenu.

Op een **tablet** is de indeling die van de desktop (zie [Indeling](layout.md)),
met doelen die makkelijker te raken zijn.

Ingedrukt houden opent wat een rechtsklik op de desktop opent: het rijmenu, de
kopmenu’s, het menu van een cart (**Bewerken…** kiest het bestand van een lege
cart). Op een telefoon vult Instellingen het scherm en toont het zijn secties in
een lijst; tik op een sectie om haar te openen, met de naam van elke instelling
boven zijn bedieningselement. De pijl of Terug gaat terug naar de lijst, en
nogmaals Terug (of **Sluiten**) sluit Instellingen. Een cart bewerken opent zijn
sectie rechtstreeks.

Over → **Licenties van derden** opent de vermeldingen van de bibliotheken
waarmee de app is gebouwd in een browser of HTML-viewer.

## Bibliotheek {#library}

Op een **telefoon** opent het tabblad **Bibliotheek** op een lijst met
categorieën: **Alle nummers**, je categorieën (elke subcategorie onder haar
bovenliggende, met haar kleur en hoeveel nummers ze bevat), **Zonder categorie**
en **Muziek op het apparaat**. De bibliotheek is dezelfde als die het
[bibliotheekvenster](library.md) van de desktop toont: elk bestand dat aan een
playlist, een cart of de bibliotheek is toegevoegd, staat erin. Categorieën
worden aangemaakt, hernoemd, van kleur veranderd en verwijderd in **Instellingen
→ Bibliotheek**.

**De nummers van een categorie.** Tik op een categorie om haar nummers te openen
(met die van haar subcategorieën). De pijl in haar kop, of Terug, gaat terug naar
de categorieën. Een ander tabblad en terug vindt de lijst zoals je haar liet. Een
categorie openen vanuit de categorieën begint met een lege zoekopdracht. Zoeken
filtert de lijst op titel, artiest, album en de andere tags die het
bibliotheekvenster doorzoekt; **Hele bibliotheek** doorzoekt elke categorie. Elke
rij toont de titel (of de bestandsnaam), de artiest, de duur en de markeringen
van het venster voor een ontbrekend, onleesbaar of nog niet geanalyseerd bestand.

Tik op rijen om ze aan te vinken. Zolang er iets is aangevinkt, verschijnt onderaan
de balk van Muziek op het apparaat: hoeveel, de speler en playlist waar ze naartoe
gaan (tik erop om een andere te kiezen), **Aan het einde toevoegen** en **Na de
volgende toevoegen**, zoals op Muziek op het apparaat. De nummers gaan in de
volgorde waarin je ze aanvinkte, en toevoegen wist de vinkjes. Terug naar de
categorieën wist ze ook.

**Het rijmenu.** Ingedrukt houden op een rij opent zijn menu: **Toevoegen aan ▸**
(de playlist van een speler, aan het einde), **Voorbeluisteren**, **Verplaatsen
naar categorie ▸**, **Opnieuw analyseren** en **Uit de bibliotheek verwijderen**,
dat vraagt zoals op de desktop (zie [Nummers
verwijderen](library.md#removing-tracks)). Terwijl rijen zijn aangevinkt, vinkt
ingedrukt houden op een andere rij die ook aan en geldt het menu voor elke
aangevinkte rij, in de volgorde waarin je ze aanvinkte; **Voorbeluisteren** wordt
dan niet aangeboden. De vinkjes blijven na de actie van het menu.

**Voorbeluisteren.** De 🎧 rechts van een rij speelt zijn nummer vanaf zijn cue-in
op de voorbeluisteruitgang van de bibliotheek; de 🎧 van een andere rij schakelt
over naar dat nummer. Een slanke strook boven de lijst toont de titel, de
resterende tijd, **Pauze**/**Play** en **Stop** terwijl het speelt (of gepauzeerd
is), en anders een hint: hij behoudt altijd zijn plaats, zodat de rijen niet
verspringen wanneer een voorbeluistering begint of stopt. De 🎧 is gedimd wanneer
het bestand ontbreekt of niet kan worden afgespeeld.

**De voorbeluisterkaart.** Tik op de strook (naast zijn knoppen) om de
kaart onder de lijst te openen, met de golfvorm van het nummer zoals op de
kaart van een speler in Playlists; de pijl rechts op de strook toont of hij
open is. Tik nog eens op de strook, of druk op Terug, om hem te sluiten; het
voorbeluisteren gaat door. De pijl in de kop van de lijst gaat meteen terug
naar de categorieën en laat de kaart open voor de volgende lijst. Op de
kaart:

- een tik op de golfvorm springt daarheen of legt, gestopt, vast waar
  **Voorbeluisteren** begint;
- ingedrukt houden zet op dat punt een marker (cue-in, einde intro, begin
  outro, MIX, cue-out) of verwijdert er een die je zette. Markers horen bij
  het nummer, dus de wijziging is zichtbaar in elke playlist en op de
  desktop;
- knijpen zoomt, slepen verschuift een ingezoomd beeld en **Volledig
  beeld** toont weer het hele nummer;
- onder de golfvorm staan de tijden, **Voorbeluisteren**, **Pauze**,
  **Stop** en de meter.

De lijst blijft boven de kaart (links met de telefoon gekanteld), zodat de
🎧 van een andere rij de kaart nog steeds naar dat nummer schakelt. Het
hangslot stopt de sprongen en markerwijzigingen van de golfvorm, zoals bij de
spelers.
De voorbeluistering stopt
wanneer zijn nummer de bibliotheek verlaat en wanneer je het tabblad Bibliotheek
verlaat (voor een ander tabblad, of met Terug); op de categorieën of op Muziek op
het apparaat gaat hij door, maar zijn strook verschijnt alleen op de nummerlijst.

In tegenstelling tot de desktop speelt Android (een telefoon of een tablet) het
voorbeluisteren van de bibliotheek op de standaarduitgang wanneer er geen uitgang
voor is gekozen: in **Instellingen → Audio-uitgangen → Bibliotheek** luidt de
lege keuze *Systeemstandaard*, en zijn **Cue testen** speelt daar. Kies daar een
andere uitgang (bijvoorbeeld bedrade koptelefoon) om weg van de uitzending te
beluisteren. De standaarduitgang is meestal ook die van de spelers en de
cartwall, dus het voorbeluisteren kan on air te horen zijn: terwijl er iets op
die uitgang speelt, vraagt de 🎧 eerst (**Toch voorbeluisteren** of
**Annuleren**), en een voorbeluistering die je niet hebt bevestigd stopt wanneer
er iets begint, zoals de CUE van een speler doet (zie [CUE
(voorbeluisteren)](players.md#cue-pre-listen)).

**Muziek van het apparaat aan een categorie toevoegen.** De **＋** in de kop van
een nummerlijst opent Muziek op het apparaat voor die categorie. Zijn balk heeft
dan alleen **Toevoegen aan &lt;categorie&gt;** (**Toevoegen aan Zonder categorie**
vanuit *Alle nummers* of *Zonder categorie*), dat de aangevinkte bestanden in die
categorie aan de bibliotheek toevoegt en terugkeert naar de lijst; een bestand
dat al in de bibliotheek staat, komt erin als het geen categorie heeft en
behoudt anders de categorie die het heeft, zoals op de desktop. Muziek op het
apparaat bewaart zijn vinkjes tussen bezoeken, dus bestanden die je bij een
eerder bezoek aanvinkte worden ook toegevoegd: controleer eerst het aantal op de
balk. **Terug naar de lijst**, bovenaan, of Terug keert terug naar de lijst
zonder toe te voegen. Muziek op het apparaat die vanuit de categorieën is
geopend, heeft bovenaan in plaats daarvan **Terug naar de categorieën**.

**De apparaatmap van een categorie.** Een categorie kan haar bestanden uit een map
van de muziek van het apparaat nemen. In haar menu (ingedrukt houden in
**Instellingen → Bibliotheek**, of in het bibliotheekvenster op een tablet) opent
**Een apparaatmap gebruiken…** Muziek op het apparaat om de map te kiezen
(**Deze map gebruiken**); het menu van de categorie noemt hem dan, boven haar
mapitems. Niets bewaakt de map: wanneer je er nieuwe muziek naartoe hebt gekopieerd,
tik je op de **↻** in de kop van de nummerlijst van de categorie (of **Apparaatmap
opnieuw inlezen** in haar menu). Elk bestand dat Muziek op het apparaat in de map
en haar submappen toont, wordt aan de categorie toegevoegd, met de regels
hierboven (een bestand dat al in een andere categorie zit, behoudt die), en een
melding zegt hoeveel bestanden zijn gelezen. Bestanden die de map hebben verlaten,
blijven in de bibliotheek, gemarkeerd als ontbrekend, en worden weer normaal wanneer ze
terugkomen (als Muziek op het apparaat niets toont, bijvoorbeeld zonder
toestemming, wordt niets gemarkeerd). **Apparaatmap wijzigen…** en **Apparaatmap vergeten** staan
in hetzelfde menu; één map hoort bij één categorie.

### Muziek op het apparaat {#device-music}

**Muziek op het apparaat** toont de audiobestanden van het apparaat per **Mappen**
(met het pad boven de lijst; tik op een deel ervan om terug omhoog te gaan),
**Artiesten** of **Albums**, en een zoekopdracht op titel, artiest, album en
bestandsnaam. Elke rij toont de titel, de artiest en de duur; tik op rijen om ze
aan te vinken. Zolang er iets is aangevinkt, toont een balk onderaan hoeveel, de
speler en playlist waar ze naartoe gaan (tik erop om een andere te kiezen), **Aan
het einde toevoegen** en, wanneer het volgende nummer van die speler in die
playlist staat, **Na de volgende toevoegen**. De nummers gaan in de volgorde
waarin je ze aanvinkte. Een apparaatbestand heeft geen voorbeluistering.

**Toevoegen** in een playlistvoettekst, het bestand van een cart en Instellingen →
Playlists → Muziekmap openen Muziek op het apparaat op een telefoon (op een
tablet het venster **muziekbrowser**, per map). Vink bestanden aan en tik op
**Toevoegen**, tik op één bestand voor een cart, of op **Deze map gebruiken**;
**Annuleren** of Terug gaat terug naar waar je was. Kopieer muziek naar het
apparaat (bijvoorbeeld naar zijn map *Music*) om haar daar te zien. Fauste Player
leest je muziek alleen: tags en hoezen worden niet geschreven, en playlists en
cartpagina’s worden op Android nog niet geïmporteerd of geëxporteerd.

Op een **tablet** is de bibliotheek het [bibliotheekvenster](library.md) van de
desktop, geopend vanuit de bovenbalk.

## Uitgangen {#outputs}

Instellingen → Audio-uitgangen toont de uitgangen van de telefoon per soort: de
luidspreker, bedrade koptelefoon, een USB-interface, een Bluetooth-luidspreker
(“JBL Flip (Bluetooth)”). Elke speler, de cartwall en hun CUE kiezen er een, zoals
op de desktop, en het voorbeluisteren van de bibliotheek ook (zie
[Bibliotheek](#library)). Een Bluetooth-luidspreker die uit en weer aan wordt
gezet, wordt via zijn route teruggevonden; terwijl hij weg is, gedragen zijn
spelers zich zoals bij een ontbrekend apparaat op de desktop. Of twee uitgangen
tegelijk kunnen klinken, hangt van de telefoon af.

## Niet op Android {#not-on-android}

MIDI-bedieningspanelen, bediening op afstand, het bewerken van tags en hoezen, en
playlist- en cartpaginabestanden zijn voorlopig functies van de desktop. Een
toetsenbord dat op een tablet is aangesloten, werkt zoals op de desktop (zie
[Toetsenbord](keyboard.md)).
