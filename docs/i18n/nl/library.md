# Bibliotheek

De bibliotheek bestaat uit alle nummers die Fauste Player kent: elk bestand dat
je aan een playlist, een cart of de bibliotheek zelf hebt toegevoegd is één
nummer, met zijn tags, markers en loudness-instellingen. Het bibliotheekvenster
toont ze in een lijst, groepeert ze in *categorieën*, zoekt ze op tekst en
stuurt ze naar de playlists en de cartwall. Het is een venster op de desktop en
op een tablet; een telefoon toont de bibliotheek in plaats daarvan in zijn
tabblad **Bibliotheek** (zie [Op een telefoon](#on-a-phone)).

![Het bibliotheekvenster: de voorbeluisterstrook met de golfvorm van het geselecteerde nummer, de meter en de luisterfader, de categorieboom met Music (met de subcategorieën Pop en Evening) geselecteerd en Te controleren markers aan het eind, en de tabel van zijn nummers met hun categorie en soort](../../images/guide/library.png)

## Het venster openen {#opening-the-window}

Klik op de tegel **Bibliotheek** in de bovenbalk. Hij is verlicht terwijl het
venster open is; klik er nogmaals op, of op de sluitknop van het venster, om het
te sluiten. Het venster zweeft boven de spelers en de cartwall en blokkeert ze
niet: niets erin stopt de audio of de sneltoetsen van het hoofdvenster.

Verplaats het met zijn titelbalk en wijzig de grootte vanaf zijn randen en
hoeken. De titelbalk heeft drie knoppen: **Bibliotheek minimaliseren** (alleen de
titelbalk blijft over, waar het venster was, of bovenaan het hoofdgebied als het
venster gemaximaliseerd is; klik er nogmaals op om uit te klappen),
**Bibliotheek maximaliseren** (het venster vult het hoofdgebied;
**Bibliotheek herstellen** brengt het terug) en **Bibliotheek sluiten**. Een
venster dat niet meer past, nadat het hoofdvenster krimpt of naar een kleiner
scherm verhuist, wordt weer binnen het hoofdgebied gebracht.

De positie en grootte van het venster, of het geminimaliseerd of gemaximaliseerd
is, de breedte van de boom, de kolommen, de sorteervolgorde en de geselecteerde
categorie worden met de sessie opgeslagen en komen bij de volgende start terug.
Of het venster open is wordt ook opgeslagen, maar er speelt daardoor niets
vanzelf. De tekst in het zoekveld wordt niet opgeslagen.

## Categorieën {#categories}

De boom links groepeert de nummers. Bovenaan staat **Alle nummers**; dan je
categorieën, die subcategorieën kunnen bevatten; dan **Zonder
categorie**, de nummers die tot geen enkele categorie behoren; en, alleen
zolang een nummer het nodig heeft, **Te controleren markers** (zie [Als een
bestand verandert](#when-a-file-changes)). Elk item toont
zijn kleur en een aantal: een categorie telt haar eigen nummers en die van haar
subcategorieën. Een nummer hoort bij hoogstens één categorie.

- **Klik** op een item om zijn nummers in de tabel te tonen. Een categorie toont
  ook de nummers van haar subcategorieën.
- De **pijl** voor een categorie opent of sluit haar subcategorieën. Welke open
  zijn, wordt opgeslagen.
- **Nieuwe categorie** (＋ boven de boom) vraagt om een naam, een kleur en een
  *soort*. Een nieuwe categorie neemt haar kleur en soort over van de categorie
  waarin ze wordt gemaakt; de soort is wat een nummer overneemt wanneer het in de
  categorie komt, en hem later wijzigen laat de nummers die er al in zitten
  ongewijzigd. Een naam mag niet leeg zijn en mag de naam van een zusje niet
  herhalen; categorieën nesten maar tot een limiet.
- **Klik met de rechtermuisknop** op een categorie (of houd ingedrukt op een
  aanraakscherm) voor **Nieuwe subcategorie…**, **Naam wijzigen…**, **Kleur**,
  **Soort** en **Verwijderen…**. Op de desktop heeft het menu ook **Map
  bewaken…** (**Bewaakte map wijzigen…** en **Map niet meer bewaken** zodra ze
  een map bewaakt) en **Ontbrekende nummers verwijderen…**; zie [Bewaakte
  mappen](#watched-folders) en [Nummers verwijderen](#removing-tracks).
- **Sleep** een categorie om haar te verplaatsen: naar een andere categorie om
  haar erin te nesten, of in de ruimte tussen twee categorieën om haar daar te
  plaatsen. Een categorie kan niet in zichzelf. Op een aanraakscherm neemt de
  boom alleen tikken aan en scrolt hij onder een vinger; een categorie kan daar
  niet worden gesleept.

**Verwijderen…** noemt wat met de categorie meegaat: haar subcategorieën en elk
nummer erin worden uit de bibliotheek gehaald. De bestanden worden nooit
verwijderd. Is een van die nummers in gebruik, dan zegt een tweede vraag
hoeveel playlistitems en carts ze gebruiken: **Toch verwijderen** haalt de items
uit hun playlists en maakt de carts leeg. Er wordt niets verwijderd zolang een
ervan on air is; een melding zegt wat er speelt.

Dezelfde boom staat in **Instellingen → Bibliotheek**, om categorieën aan te
maken, te hernoemen, een andere kleur te geven, van soort te veranderen, te
verplaatsen en te verwijderen zonder het venster te openen (hij heeft geen *Alle
nummers*, *Zonder categorie* of selectie).

![Instellingen, Bibliotheek: de categorieboom met Music, de subcategorieën Pop en Evening, en Jingles, elk met zijn aantal](../../images/guide/settings-library.png)

## Bewaakte mappen {#watched-folders}

Op de desktop kan een categorie één map bewaken. Klik er met de rechtermuisknop
op en kies **Map bewaken…**, en kies dan de map. De categorie toont een ⟳-teken
voor haar aantal; houd de muisaanwijzer erboven om de map te lezen.

- Elk audiobestand in de map en haar submappen komt in de categorie. Submappen
  maken geen subcategorieën.
- Een nummer dat al in een andere categorie zit, blijft daar. Een nummer dat in
  geen enkele categorie zit, komt in deze. Een bewaking instellen, wijzigen of
  stoppen verplaatst nooit een nummer: de nummers van een map die niet meer wordt
  bewaakt, blijven in hun categorie.
- Terwijl de applicatie draait, wordt de map gevolgd: een nieuw bestand komt erin
  zodra het even stil is geweest (`library.watch_debounce_ms`, standaard 1
  seconde, zodat een bestand dat nog wordt gekopieerd niet half geschreven wordt
  gelezen). De map wordt ook bij elke start van de applicatie gescand, om op te
  pikken wat veranderde terwijl ze gesloten was.
- Een bestand dat uit de map verdwijnt (verwijderd, verplaatst of hernoemd) laat
  zijn nummer in de bibliotheek, als ontbrekend gemarkeerd, met zijn markers.
  Komt het bestand terug, dan is het nummer vanzelf weer normaal. Een hernoemd
  bestand komt als nieuw nummer binnen onder zijn nieuwe naam.
- Bewaakte mappen kunnen genest zijn. Een bestand onder twee ervan gaat naar de
  categorie van de binnenste; er een stoppen laat de andere gevolgd. Dezelfde map
  kan niet door twee categorieën worden bewaakt: de applicatie zegt welke
  categorie hem al heeft.
- Niets wat een bewaakte map doet, laadt of speelt een nummer.

Kan de map niet worden gevolgd, dan verandert het ⟳-teken van kleur en zegt zijn
tooltip waarom: *Bewaakte map niet gevonden: …. Er wordt opnieuw naar gezocht.*
(bijvoorbeeld een schijf die niet is gekoppeld), *Bewaakte map kan niet worden
gelezen: …. Het wordt opnieuw geprobeerd.*, of *Wijzigingen in … worden niet
gevolgd; de map wordt bij het starten doorzocht en opnieuw geprobeerd.* (het
systeem weigerde hem te bewaken). Een map die is opgeslagen als een pad dat niet
volledig is (met de hand bewerkt) wordt helemaal niet bewaakt, en het teken zegt
dat.

Elke 30 seconden (`tuning.missing_recheck_ms`) controleert de applicatie of elke
bewaakte map er nog is, zodat ook een schijf die losgekoppeld wordt terwijl ze
draait wordt gemeld. Een map in problemen wordt in dat tempo opnieuw geprobeerd
en gescand zodra hij terug is. Een map die er niet is, niet kan worden gelezen,
of leeg is terwijl de bibliotheek nog nummers ervan heeft (een schijf of share
die niet is gekoppeld ziet er vaak uit als een lege map) telt als niet gevonden.
De bewaking markeert dan geen van zijn nummers als ontbrekend en verwijdert niets,
ook wanneer de map binnen een andere bewaakte map ligt.

Leeg je een bewaakte map met opzet, dan blijft de waarschuwing totdat er weer
bestanden in staan of je ophoudt hem te bewaken, en blijven zijn nummers in de
bibliotheek zonder als ontbrekend te worden gemarkeerd. Om ze te wissen,
selecteer je ze in de tabel en kies je **Uit de bibliotheek verwijderen…**, of
kies je **Opnieuw analyseren** erop: de analyse markeert de nummers waarvan het
bestand weg is als ontbrekend, en **Ontbrekende nummers verwijderen…** verwijdert
ze dan (wordt de map nog bewaakt, dan verdwijnt zijn waarschuwing bij de
volgende controle).

Eén uitzondering op de regel hierboven: een nummer dat nooit is geanalyseerd
wordt nog steeds als ontbrekend gemarkeerd wanneer de analyse zijn bestand niet
kan vinden, waar het ook staat.

Een submap die niet kan worden gelezen wordt overgeslagen en het log zegt dat;
maplinks (symbolische links naar mappen) worden niet gevolgd.

Nieuwe bestanden gaan naar de analyse achter alles wat een speler of een cart
nodig heeft, dus een map met duizenden bestanden duurt even; de voortgang van de
analyse toont het.

Hetzelfde menu en teken staan in **Instellingen → Bibliotheek**. Ze staan niet op
Android, dat geen mappen bewaakt: daar kan een categorie haar bestanden uit een
apparaatmap nemen die je opnieuw inleest wanneer je dat kiest (zie [Android →
Bibliotheek](android.md#library)).

## De nummertabel {#the-track-table}

De tabel toont de nummers van het geselecteerde item, in dezelfde kolommen als de
playlisttabellen (die gekozen in **Instellingen → Playlists → Tabelkolommen**,
zonder de positie `#`) gevolgd door **Categorie** en **Soort**. Een smalle cel
links markeert een nummer waarvan het bestand ontbreekt of niet kan worden
gelezen. Een nummer dat op zijn analyse wacht, wordt gemarkeerd rechts van zijn
cel **Titel** (dus alleen wanneer de kolom Titel wordt getoond).

- Sleep een kolomscheiding om de breedte van een kolom te wijzigen; klik op een
  kop om erop te sorteren en nogmaals om om te keren. Breedtes en
  sorteervolgorde worden opgeslagen.
- **Klik** selecteert een rij, **Ctrl** (**Cmd** op macOS) **klik** voegt er een
  toe of haalt er een weg, **Shift-klik** selecteert de rijen tussen de laatste
  klik en deze. De selectie behoudt alleen rijen die in de lijst staan: een rij
  die de tabel verlaat, door een zoekopdracht, een andere categorie of een
  verwijdering, verlaat de selectie. Elke actie geldt voor de geselecteerde
  rijen in tabelvolgorde.
- **Bestanden toevoegen** (boven de tabel) kiest audiobestanden en voegt ze toe
  aan de bibliotheek, in de geselecteerde categorie (zonder categorie wanneer
  *Alle nummers* of *Zonder categorie* is geselecteerd).

Op een aanraakscherm nemen de rijen alleen tikken aan en scrollen ze onder een
vinger; ingedrukt houden opent het menu, en nummers worden met de menu’s
verstuurd (hieronder), niet door te slepen.

## Zoeken {#search}

Het veld boven de tabel zoekt nummers op tekst. Het kijkt in de titel, artiest,
het album, de albumartiest, componist, het genre, de opmerking en de
bestandsnaam, zonder hoofdletters of accenten te onderscheiden, en een nummer
wordt getoond wanneer het elk woord bevat dat je typte (in willekeurige volgorde
en in elk van die velden). Het zoekt in het geselecteerde item; zet **Hele
bibliotheek** aan om elk nummer te doorzoeken, wat de selectie ook is. Het bereik
wordt opgeslagen; de tekst niet. Wis het veld om het hele item weer te zien.

## Het rijmenu {#the-row-menu}

Klik met de rechtermuisknop op een rij. Is de rij niet geselecteerd, dan wordt
ze de selectie; anders geldt het menu voor elke geselecteerde rij. De eerste
regel toont de titel van het nummer wanneer het menu voor één nummer is, en hoeveel
nummers er geselecteerd zijn wanneer het voor meerdere is.

| Item | Actie |
|---|---|
| Toevoegen aan ▸ | Een playlist: de nummers worden aan het einde toegevoegd |
| Toevoegen aan cart ▸ | Een cart van de getoonde pagina van de cartwall: de nummers vullen de gekozen cart en de volgende lege carts van de pagina, in rastervolgorde. De nummers die niet passen, blijven weg en een melding telt ze |
| Verplaatsen naar categorie ▸ | Een categorie, of Zonder categorie |
| Soort ▸ | Muziek, Jingle, Effect, Reclame, Stem of Uurmelding |
| Tags bewerken… | De tag-editor (één nummer) |
| Openen in externe editor | Je audio-editor starten met het bestand van het nummer (één nummer; zie [Als een bestand verandert](#when-a-file-changes)) |
| Markeren als gecontroleerd | De vlag *markers moeten worden gecontroleerd* wissen (getoond wanneer een geselecteerd nummer die heeft) |
| Loudness… | De loudness-pop-up (één nummer) |
| Opnieuw analyseren | De nummers opnieuw analyseren |
| Markers terugzetten op automatisch | De handmatige markers laten vallen |
| Bestandslocatie tonen | De map van het bestand openen (één nummer) |
| Uit de bibliotheek verwijderen… | De nummers uit de bibliotheek verwijderen, na een vraag (hieronder) |

## Als een bestand verandert {#when-a-file-changes}

Materiaal voorbereiden betekent vaak een bestand inkorten of repareren in een
audio-editor. Fauste Player helpt bij beide helften.

**Openen in externe editor** (het rijmenu van de bibliotheek en van
playlistrijen) start het programma dat je koos in [Instellingen →
Bibliotheek](settings.md#library) met het bestand van het nummer. Het werkt op
één nummer. Het item is gedimd, met de reden als tooltip, wanneer er geen editor
is ingesteld, wanneer het bestand ontbreekt of onleesbaar is, of wanneer het
nummer *klinkt*: het huidige nummer van een speler (spelend of gepauzeerd), een
nummer dat wacht om te starten of in CUE staat, het uitgaande nummer van een
fade, een spelende cart of de CUE-cart van de cartwall, of het voorbeluisteren
van de bibliotheek terwijl het speelt of gepauzeerd is. De editor wordt niet
aangeboden op Android, noch in de Flatpak- en snap-builds, die geen andere
programma's kunnen starten; de rest van dit onderdeel geldt daar nog steeds.

Fauste Player merkt wanneer een bestand dat het kent is veranderd (zijn grootte
of wijzigingstijd verschilt van de laatste analyse), door wie dan ook, en
analyseert het uit zichzelf opnieuw: duur, markers, loudness, golfvorm, formaat
en tags. Het kijkt:

- één keer bij de start, bij elk nummer in de bibliotheek;
- wanneer een bewaakte map een wijziging van een bekend nummer meldt;
- in de map van elk bestand dat je in de editor opende, tot je afsluit, zodat
  een opslag meteen wordt opgemerkt;
- wanneer een nummer in een speler, een cart of het voorbeluisteren wordt
  geladen, en wanneer zijn markers worden geopend om te bewerken of op de
  golfvorm worden gesleept;
- wanneer je op **Gewijzigde bestanden zoeken** drukt in [Instellingen →
  Analyse](settings.md#analysis), wat de hele bibliotheek controleert en in een
  melding zegt hoeveel bestanden zijn veranderd.

Er is geen periodiek pollen. Een bestand buiten de bewaakte mappen, door een
ander programma gewijzigd terwijl Fauste Player draait en sindsdien niet
gebruikt, wordt gevonden bij zijn volgende gebruik of door **Gewijzigde
bestanden zoeken**.

Een gewijzigd nummer wordt nooit opnieuw geanalyseerd terwijl het klinkt: het
wacht tot het stopt, en zijn markers en loudness verschuiven nooit onder een
spelende bron. (**Alle nummers opnieuw analyseren** is je eigen verzoek en wacht
niet.) Editors slaan vaak op door het bestand te verwijderen en opnieuw aan te
maken, dus een bestand dat verdwijnt wordt twee keer bekeken voordat het als
ontbrekend wordt gemarkeerd. Een bestand dat verandert terwijl het wordt
geanalyseerd (een editor die het nog opslaat) wordt niet gebruikt: het wordt
kort daarna opnieuw geanalyseerd, of bij zijn volgende wijziging, gebruik of
controle. Een ontbrekend nummer waarvan het bestand terugkomt in een map die
alleen wordt gevolgd vanwege de editor, wordt niet uit zichzelf hersteld.

**Te controleren markers.** Als de *audio* van een bestand is veranderd en het
nummer markers heeft die je met de hand hebt gezet, toont het nummer een oranje
waarschuwingsicoon op zijn rij (de tooltip noemt eventuele markers die zijn
vervallen). Handmatige markers die nu voorbij het einde van de audio liggen
vervallen en de automatische waarde neemt hun plaats in; de andere blijven zoals
ze zijn (wanneer de cue-in of cue-out is vervallen, worden de intro-, outro- en
MIX-markers binnen het nieuwe cuebereik verplaatst). De vlag verhindert nooit
dat het nummer wordt geladen, in CUE gezet of afgespeeld. Hij verdwijnt wanneer
je **Markeren als gecontroleerd** kiest in het menu van het nummer (bij meerdere
geselecteerde rijen verdwijnt hij bij allemaal) of een marker van het nummer
wijzigt. Het item **Te controleren markers** van de boom, na *Zonder categorie*,
toont de gemarkeerde nummers zolang er zijn (niet op een telefoon). Een
wijziging van alleen de tags of de cover vernieuwt de tags en zet geen vlag,
wat ook gebeurt nadat je tags in Fauste Player bewerkt. De eerste analyse na een
update zet de vlag alleen als ze een marker voorbij het einde van de audio moest
laten vallen.

## Voorbeluisteren {#pre-listening}

De balk onder de titelbalk beluistert het geselecteerde nummer vooraf op de
eigen **Bibliotheek**-uitgang van de bibliotheek (te kiezen in [Instellingen →
Audio-uitgangen](settings.md#audio-outputs)), bijvoorbeeld een koptelefoon. Een
telefoon beluistert vooraf vanuit zijn nummerlijst (zie
[Op een telefoon](#on-a-phone)).

De balk toont:

- **Voorbeluisteren**, **Voorbeluisteren pauzeren** en **Voorbeluisteren
  stoppen** (Play, Pause en Stop);
- de grote resterende tijd van het nummer, zijn titel, artiest en categorie;
- rechts een meter en een volumefader, getekend zoals bij een speler. De meter
  leest alleen dit voorbeluisteren, nooit de meter van een speler of van de
  cartwall; klik erop om zijn maximum opnieuw te beginnen. De fader is je
  **luisterniveau** in de koptelefoon of de monitors: hij bepaalt hoe luid het
  voorbeluisteren klinkt en wijzigt nooit een nummer (zijn bestand, zijn
  loudness-gain of zijn markers), een speler of de cartwall. Hij blijft staan
  zoals je hem achterliet wanneer het voorbeluisteren naar een ander nummer
  gaat en wordt bewaard voor de volgende start; zijn tooltip en de uitlezing
  eronder tonen het niveau in dB. Deelt de Bibliotheek-uitgang kanalen met een
  on-air uitgang, dan bepaalt de fader ook hoe luid het voorbeluisteren daar
  klinkt. De voorbeluisterkaart van een telefoon heeft geen fader;
- de golfvorm van het hele bestand, even hoog als die van een speler, met zijn
  markers, en een regel `verstreken / totaal` eronder.

Play start het geselecteerde nummer vanaf zijn cue-in en speelt het tot het
einde van het bestand. Play op een gepauzeerde voorbeluistering hervat hem, en
Play op een spelende doet niets: Stop, daarna Play, begint opnieuw vanaf de
cue-in. De loudness-normalisatie geldt zoals bij de CUE van een speler.

De golfvorm werkt zoals in het CUE-venster van een speler: klik erop om te
zoeken (een klik of het loslaten van de muis zoekt; slepen niet), zoom en
verschuif hem met het wiel, en bewerk de markers en open **Loudness…** vanuit
zijn menu (zie [Markers bewerken](markers-and-mixing.md#editing-markers)). Terwijl
de voorbeluistering gestopt is, kiest een klik in plaats daarvan waar de
volgende Play begint.

**De voorbeluistering volgt de selectie.** Met meerdere geselecteerde rijen
gebruikt hij de laatste waarop je klikte.

- Terwijl hij speelt, schakelt een andere rij selecteren over naar dat nummer
  vanaf zijn cue-in en blijft hij spelen, zonder te vragen.
- Terwijl hij gepauzeerd is, wordt het nieuwe nummer gepauzeerd geladen.
- Terwijl hij gestopt is, blijft hij gestopt en toont de balk het nieuwe nummer.
- Niets selecteren (geen rij geselecteerd, of de geselecteerde rij weggefilterd)
  laat een lopende voorbeluistering zoals hij is.
- Een nummer selecteren dat niet kan worden afgespeeld (zijn bestand ontbreekt of
  is onleesbaar) stopt de voorbeluistering, en de balk zegt dat het bestand niet
  kan worden afgespeeld.

**Wat hem stopt.** Stop; zijn nummer uit de bibliotheek verwijderen; het venster
sluiten (minimaliseren niet). Hij speelt nooit na een herstart.

**Zonder Bibliotheek-uitgang** (op de desktop; Android speelt in plaats daarvan
op de standaarduitgang, zie [Android → Bibliotheek](android.md#library)) is Play
gedimd (houd de muisaanwijzer erboven voor de reden) en zegt een regel ernaast
dat voorbeluisteren een CUE-uitgang van de bibliotheek nodig heeft, met een link
die Instellingen → Audio-uitgangen opent bij de rij **Bibliotheek**. De balk toont
nog steeds het nummer, zijn golfvorm en zijn markers.

**Op een on-air uitgang.** De Bibliotheek-uitgang mag een on-air uitgang zijn
(hetzelfde apparaat en dezelfde kanalen als de Main van een speler of van de
cartwall). Hij werkt dan zoals de CUE van de spelers en van de cartwall (zie
[CUE (voorbeluisteren)](players.md#cue-pre-listen)): terwijl daar iets speelt,
vraagt Play eerst (**Toch voorbeluisteren** of **Annuleren**), en een
voorbeluistering die je niet hebt bevestigd stopt wanneer er iets op die uitgang
begint.

## Slepen naar playlists en carts {#dragging-to-playlists-and-carts}

Sleep de geselecteerde rijen (of de rij waarmee je begint te slepen, wanneer die
niet geselecteerd is) met de muis:

- naar een **playlisttabblad** van een speler: de nummers gaan naar het einde van
  die playlist;
- naar een **playlisttabel**: de nummers gaan waar de violette lijn laat zien;
- naar een **cart**: de cart krijgt het eerste nummer (een cart bevat één
  bestand), en vervangt zijn bestand wanneer hij gestopt is. Een cart die speelt
  of op CUE staat weigert de drop, omdat een nieuw bestand hem on air zou stoppen.

## Nummers verwijderen {#removing-tracks}

**Uit de bibliotheek verwijderen…** vraagt eerst: de bestanden worden niet
verwijderd. Wordt een van de nummers gebruikt, dan telt een tweede vraag de
playlistitems en carts die ze kwijtraken. Een nummer dat on air is (speelt in een
speler of op de cartwall) kan niet worden verwijderd: er wordt niets verwijderd
en een melding zegt wat er speelt.

**Ontbrekende nummers verwijderen…** (in het menu van de categorie, op elke
categorie) verwijdert de nummers van die categorie en haar subcategorieën waarvan
het bestand ontbreekt en die geen playlistitem of cart gebruikt. Het vraagt
eerst, met hun aantal; er wordt geen bestand aangeraakt. Nummers in gebruik
blijven, en een categorie zonder zulke nummers zegt dat in een melding. Gebruik
het om op te ruimen nadat bestanden een bewaakte map hebben verlaten: de
applicatie verwijdert ze nooit zelf.

## Vanuit een playlist {#from-a-playlist}

Op de desktop heeft het nummermenu van een playlistrij **Tonen in bibliotheek** en
**Verplaatsen naar categorie ▸**, na **Loudness…**. *Tonen in bibliotheek* opent
het venster op de categorie van het nummer en selecteert zijn rij; *Verplaatsen
naar categorie* brengt het nummer onder in een categorie zonder het venster te
openen. Voor een nummer dat niet in de bibliotheek staat, wordt *Tonen in
bibliotheek* gedimd getoond en wordt *Verplaatsen naar categorie* niet getoond.

## Op een telefoon {#on-a-phone}

Een telefoon heeft geen bibliotheekvenster: zijn tabblad **Bibliotheek** opent
op de categorieën (*Alle nummers*, je categorieën, *Zonder categorie* en **Muziek
op het apparaat**), en een tik op een ervan toont haar nummers met een zoekveld
en **Hele bibliotheek**. Vink nummers aan om ze aan een playlist toe te voegen,
houd een rij ingedrukt voor haar menu, beluister een nummer vooraf met zijn 🎧,
en voeg de muziek van het apparaat toe aan een categorie met de **＋** van haar
lijst. Op Android speelt het voorbeluisteren zonder Bibliotheek-uitgang op de
standaarduitgang. Zie [Android → Bibliotheek](android.md#library).
