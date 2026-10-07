# Playlists

## Tabbladen {#tabs}

Elke speler heeft een rij tabbladen, één per playlist. Alle spelers zien
dezelfde playlists; elke speler kiest welke hij toont. Een stip op een tabblad
laat zien waar de nummers van de speler staan: **rood** voor het nummer dat on
air is, **groen** voor het volgende.

De tabbladen delen de breedte van de speler. Een naam die niet past, eindigt op
“…”; houd de muisaanwijzer boven het tabblad om hem helemaal te lezen. Bij veel
playlists houden de tabbladen op met krimpen bij een minimale breedte,
verschijnen er pijlen aan de uiteinden van de rij en scrolt het muiswiel boven
de tabbladen ze. Het tabblad dat je kiest, en het tabblad dat een speler toont,
wordt in beeld gebracht.

Van tabblad wisselen verandert nooit wat on air is of wat het volgende is.
Wanneer een nummer eindigt, gaat de speler verder in de playlist die dat
nummer bevat.

Playlists worden aangemaakt, hernoemd en verwijderd in
[Instellingen](settings.md). De laatste playlist, en een playlist met een
nummer on air, kunnen niet worden verwijderd.

## De nummertabel {#the-track-table}

![Een playlist: gespeelde nummers gedimd, het nummer on air in rood, het volgende nummer in groen, en de voettekst met de resterende tijd](../../images/guide/playlist.png)

| Kolom | Inhoud |
|---|---|
| `#` | Positie, met voorloopnullen; een pictogram vervangt het voor het huidige en het volgende nummer |
| Titel | Uit de tags, of de bestandsnaam (`Artist - Title.mp3` wordt gesplitst). De pictogrammen voor herhalen en stoppen na staan voor de titel |
| Artiest | Uit de tags; “Onbekende artiest” wanneer die er niet is |
| Album | Uit de tags |
| Datum | De opnamedatum zoals het bestand die opslaat (`2019`, `2019-05` of `2019-05-14`, met een tijd als die er is) |
| Genre | Uit de tags |
| Duur | Speelduur, van cue-in tot cue-out (het hele bestand met **Cue-in en cue-out gebruiken** uit) |
| Intro | Hoe lang de intro duurt, vanaf waar het nummer begint te spelen tot zijn intromarker; leeg wanneer het nummer geen intromarker heeft |
| Bestandsnaam | De naam van het bestand, met zijn extensie |

Een nieuwe installatie toont `#`, Titel, Artiest en Duur. De andere kolommen
zijn optioneel; zie **De kolommen kiezen** hieronder. Een nummer dat een waarde
mist, toont een lege cel, behalve Artiest, die “Onbekende artiest” toont.

De kolommen vullen de tabel en behouden hun verhoudingen wanneer het venster
van grootte verandert; de tekstkolommen krijgen de meeste ruimte. Sleep de
scheidingen in de kop om de verhoudingen te wijzigen: de kolommen rechts van de
scheiding volgen de aanwijzer bij elk beeld (ze delen wat er over is naar
verhouding van hun breedte), die links ervan blijven, en de breedtes worden
opgeslagen wanneer je loslaat. Geen kolom wordt smaller dan zijn minimum. De
breedtes worden per speler onthouden; een kolom die je later toont, begint met
zijn standaardbreedte en de andere behouden hun verhoudingen.

### De kolommen kiezen {#choosing-the-columns}

Titel en Duur worden altijd getoond. Elke andere kolom kan worden getoond of
verborgen, en elke kolom, die twee inbegrepen, kan worden verplaatst. De lijst
is voor elke speler en playlist hetzelfde, en wordt opgeslagen in `config.json`
als `ui.table_columns`. Drie manieren om hem te wijzigen:

- **Instellingen → Playlists → Tabelkolommen:** vink de kolommen aan die je
  wilt tonen; de pijlen verplaatsen een getoonde kolom omhoog of omlaag (ze
  lezen van links naar rechts in de tabellen). **Standaardkolommen** gaat terug
  naar `#`, Titel, Artiest en Duur.
- **Klik met de rechtermuisknop op een kolomkop:** een menu met een selectievakje
  voor elke optionele kolom. Een kolom die je toont, verschijnt aan het
  rechteruiteinde; sleep hem vanaf daar.
- **Sleep een kolomkop** naar een andere: zet hem neer op de linkerhelft van een
  kop om de kolom ervoor te zetten, op de rechterhelft om hem erna te zetten.
  Ergens anders neerzetten doet niets.

Een naam in `ui.table_columns` die deze versie niet kent, wordt genegeerd, en
een ontbrekende Titel of Duur wordt weer toegevoegd.

Wanneer de applicatie opent, scrolt elke tabel zo dat het volgende nummer van
zijn speler in het midden van de tabel staat (zo dicht als de uiteinden van de
lijst toestaan). Dat gebeurt eenmalig, bij het opstarten, en alleen wanneer het
volgende nummer in de playlist staat die de tabel toont.

Wanneer een speler naar een ander nummer doorgaat, toont zijn tabel de playlist
van dat nummer en scrolt zijn rij naar boven, tenzij je de tabel de laatste 10
seconden hebt gebruikt (hem gescrold, een nummer gesleept, het menu van een
nummer geopend of op een tabblad geklikt): dan wacht hij totdat je hem zo lang
met rust laat. De tijd is `ui.follow_current_grace_secs` in `config.json`; 0
zet het volgen uit.

Spelers zijn onafhankelijk: meerdere spelers kunnen dezelfde playlist tonen,
elk met zijn eigen volgende nummer, zijn eigen gespeeld-markeringen en zijn
eigen tijden in de voettekst. Afspelen, stoppen of overslaan op de ene speler
verplaatst nooit het volgende van een andere speler. Hetzelfde nummer kan zelfs
tegelijk on air zijn op twee spelers. De playlist bewerken (items toevoegen,
verplaatsen of verwijderen) of een bestand dat onleesbaar wordt, kan nog steeds
het volgende van elke speler die hem toont wijzigen.

Rijkleuren:

| Rij | Betekenis |
|---|---|
| **Rood**, met een luidspreker- (of pauze)pictogram | On air op deze speler. Het kan ook de groene pijl tonen: het nummer on air is ook het volgende, dus het speelt nog een keer |
| Rode **P2** (of een ander nummer) in de nummerkolom | On air op die speler |
| **Groen**, met een pijl | Het volgende nummer van deze speler |
| Gedimd | Al gespeeld op deze speler |
| Bestand met een kruis / waarschuwingspictogram | Bestand ontbreekt / onleesbaar (het wordt overgeslagen); houd de muisaanwijzer boven de rij: de pop-up begint met de reden, in amber, en toont dan de gebruikelijke velden. Een ontbrekend bestand wordt elke 30 s opnieuw gezocht (`tuning.missing_recheck_ms`). |
| Herlaadpijlen rechts van de titel | Geanalyseerd door een eerdere versie; het speelt nog steeds met die analyse. **Instellingen → Analyse → Verouderde nummers analyseren** brengt het bij (nummers op een speler worden hoe dan ook bijgewerkt) |
| Zandloper rechts van de titel | Het nummer wacht op zijn analyse (houd de muisaanwijzer boven de zandloper: *Analyse in afwachting*). Het speelt toch, en de zandloper verdwijnt wanneer de analyse klaar is |
| Violet | Geselecteerd |

**Tooltip van een nummer.** Houd de muisaanwijzer even boven een rij om de
titel, artiest, het album, de datum, het genre, de lengte, het formaat (type,
samplefrequentie en bitdiepte wanneer bekend) en het pad van het bestand te
zien. Een veld dat het bestand niet heeft, wordt weggelaten. De pop-up is de
enige hoverinformatie op een rij, en hij beweegt nooit zolang hij getoond
wordt. Voor een ontbrekend of onleesbaar bestand begint hij met de reden.

## Muis {#mouse}

- **Klikken** selecteert een nummer. **Dubbelklikken** maakt er het volgende
  nummer van deze speler van. Op het nummer dat on air is, laat het dat nog een
  keer spelen wanneer de huidige ronde eindigt.
- **Rechtsklikken** opent het contextmenu:

![Het contextmenu van een nummer](../../images/guide/track-menu.png)

| Item | Actie |
|---|---|
| Nu afspelen | Dit nummer meteen starten (met mixen als de speler on air is) |
| Als volgende instellen | Hetzelfde als dubbelklikken. Op het nummer dat on air is, speelt het nog een keer, vanaf zijn begin, wanneer de huidige ronde eindigt (mixend zoals Herhalen, zonder gat), waarna de speler verdergaat. Het werkt eenmalig. Stop na, de modus SINGLE en een markering Stop na beëindigen de speler nog steeds eerst. Terwijl een CUE loopt, verplaatst hij naar het nieuwe volgende |
| Voorbeluisteren op CUE | Afspelen op de CUE-uitgang (het opent het CUE-venster). Gedimd wanneer de speler naast zijn Main-uitgang geen Cue-uitgang heeft |
| Tags bewerken… | Open de tag-editor voor dit nummer. **Opslaan** schrijft de wijzigingen in het audiobestand; **Annuleren** (of Esc, wanneer er geen opslag loopt) sluit zonder te schrijven. Het item is gedimd, met de reden wanneer je de muisaanwijzer erboven houdt, zolang het nummer on air is, op CUE staat of op een spelende cart, zolang de tags nog niet zijn gelezen, wanneer het bestand ontbreekt, en voor formaten waarvan de tags niet kunnen worden geschreven (bijvoorbeeld DSD) |
| Opnieuw analyseren | Dit nummer nu opnieuw analyseren, wat zijn toestand ook is. Een gerepareerd bestand dat onleesbaar was, wordt ook vanzelf opgepakt (zie [Problemen oplossen](troubleshooting.md)). Handmatige markers blijven bewaard |
| Nummers hieronder toevoegen… | Kies bestanden om na dit nummer in te voegen |
| Dupliceren | Een ongespeelde kopie eronder invoegen (met zijn markeringen voor herhalen en stoppen na) |
| Dit nummer herhalen | Vink aan om het steeds opnieuw af te spelen, zonder gat, totdat je op Play (volgende), Vorige, Stop of Uitfaden drukt, of Stop na aanzet. Pauze houdt het herhalend. Een herhaalpictogram staat voor de titel |
| Stoppen na dit nummer | Vink aan om de speler te stoppen wanneer dit nummer eindigt, elke keer dat het speelt (in elke modus). In tegenstelling tot de knop **Stop na** van de speler blijft de markering bij het nummer en wordt ze met de playlist opgeslagen. Het pictogram voor stoppen na staat voor de titel. Het gaat voor Herhalen |
| Verplaatsen naar ▸ | Het naar het einde van een andere playlist verplaatsen |
| Uit playlist verwijderen | Het verwijderen; niet mogelijk zolang het on air is |

## Tags bewerken {#editing-tags}

**Tags bewerken…** opent een venster voor één nummer. Zolang het open is, werkt
geen enkele sneltoets en worden bestanden die op het applicatievenster worden
neergezet genegeerd.

![De tag-editor van een FLAC-bestand, met hoes, titel, artiest, album, datum en genre](../../images/guide/tag-editor.png)

- **Wat je ziet.** De editor leest het bestand wanneer hij opent (intussen toont
  hij “Tags lezen…”). Altijd getoond: titel, artiest, album, albumartiest,
  datum, tracknummer en totaal, schijfnummer en totaal, genre, componist en
  opmerking. Getoond wanneer het bestand ze heeft: ondertitel, groepering, BPM,
  begintoonsoort, stemming, ISRC, uitgever, catalogusnummer, copyright,
  oorspronkelijke artiest, oorspronkelijk album, oorspronkelijke releasedatum,
  tekstschrijver, dirigent, remixer, arrangeur, uitvoerende, taal, gecodeerd
  door, songtekst, sorteertitel, sorteerartiest, sorteeralbum,
  sorteeralbumartiest, sorteercomponist en website van artiest.
- **Veld toevoegen.** Het menu onder de velden somt de andere velden op. Het
  biedt alleen wat het tagformaat van het bestand kan opslaan (een WAV met
  RIFF INFO, een AIFF of een oude ID3v1-tag slaan minder velden op dan ID3v2,
  FLAC of MP4), en het is gedimd wanneer er niets meer is om toe te voegen. Een
  van de altijd getoonde velden die het formaat niet kan opslaan, is grijs
  weergegeven met een opmerking. Een veld leegmaken verwijdert het uit het
  bestand; een toegevoegd veld dat leeg blijft, wordt niet geschreven.
- **Meerdere waarden.** Velden die meerdere waarden kunnen bevatten (artiest,
  albumartiest, genre, componist, stemming en de vermeldingen zoals
  tekstschrijver, dirigent, remixer, arrangeur en uitvoerende, en taal) tonen
  één waarde per regel; **Opslaan** schrijft één waarde per regel op de manier
  van het formaat zelf. Opmerking en songtekst zijn vrije tekst over meerdere
  regels.
- **Controles.** Datum en oorspronkelijke releasedatum zijn ISO 8601 (`2019`,
  `2019-05` of `2019-05-14`, eventueel met een tijd); track- en schijfnummer,
  hun totalen en de BPM zijn hele getallen, en een totaal heeft zijn nummer
  nodig. Een veld met een ongeldige waarde is gemarkeerd en **Opslaan** blijft
  uit. Een waarde die het bestand al had en die je niet hebt aangeraakt, blijft
  zoals ze is.
- **Velden die te lang zijn.** Een veld waarvan de tekst langer is dan
  `limits.max_tag_chars`, of dat meer waarden bevat dan `limits.max_tag_values`,
  wordt alleen-lezen getoond met de opmerking “Te lang om hier te bewerken;
  blijft zoals het in het bestand staat”. Het wordt nooit teruggeschreven, dus
  een opslag kan het niet afkappen.
- **Wat bewaard blijft.** Alles wat de editor niet toont (andere standaardsleutels,
  eigen sleutels, andere afbeeldingen dan de voorhoes, binaire frames) blijft
  met dezelfde waarden in het bestand. De editor zegt hoeveel van zulke tags
  bewaard blijven (en “meer” wanneer het formaat frames bevat die niet geteld
  kunnen worden). Opslaan codeert de items die de editor afbeeldt opnieuw, dus
  een bewaard item kan in zijn bytes verschillen (tekstcodering, volgorde van
  frames) maar niet in zijn waarde.
- **De hoes.** De editor toont de voorhoes, of de eerste afbeelding van het
  bestand wanneer er geen voorhoes is, als miniatuur.
  - **Wijzigen…** opent een bestandsdialoog voor een JPEG- of PNG-afbeelding
    (hoogstens `limits.max_cover_bytes`, en ze moet te decoderen zijn). Is dat
    niet zo, dan zegt de editor waarom en verandert er niets.
  - **Verwijderen** wist de voorhoes. Het staat uit wanneer het bestand geen
    voorhoes heeft: een afbeelding die alleen wordt getoond omdat er geen
    voorhoes is, is alleen ter weergave en blijft zoals ze is.
  - Een hoes die in het bestand zit maar niet kan worden getoond (een
    afbeelding die niet decodeert, of een GIF, BMP of WebP) wordt aangekondigd
    met “Deze hoes kan niet worden getoond; hij blijft zoals hij is”.
    **Wijzigen…** en **Verwijderen** werken nog steeds.
  - De wijziging wordt geschreven door **Opslaan** en verworpen door
    **Annuleren**. Achterhoezen en elke andere afbeelding worden nooit
    aangeraakt. Een formaat zonder plek voor afbeeldingen (WAV met RIFF INFO,
    AIFF, ID3v1) toont het gebied uitgeschakeld. Na een opslag toont de hoes van
    de speler de nieuwe hoes.
- **Hoe een opslag werkt.** Het bestand wordt naast het origineel gekopieerd, de
  kopie krijgt de tags, wordt gesynchroniseerd en vervangt het origineel, zodat
  een mislukking het bestand laat zoals het was. De reden wordt getoond in de
  editor, die open blijft om het opnieuw te proberen, en in de statusbalk.
  Alleen de velden die je hebt gewijzigd, worden geschreven. Na een opslag toont
  de tabel meteen de nieuwe tags, en markers en de golfvorm blijven bewaard.
  Heeft het bestand een veld dat je hebt gewijzigd niet behouden, dan noemt de
  statusbalk het.
- **Na een update.** Nummers van een eerdere versie krijgen hun datum, genre en
  andere tags stilletjes op de achtergrond ingevuld (geen volledige analyse).

## Slepen en neerzetten {#drag-and-drop}

- Sleep een nummer binnen de lijst om het te herschikken. Een violette lijn
  toont waar het terechtkomt: hij staat op de rijgrens die het dichtst bij de
  aanwijzer ligt, en alleen in de lijst onder de aanwijzer. Loslaten boven de
  kop, een kolomrand, de schuifbalk of een venster dat de lijst bedekt (het
  CUE-venster) zet niets neer.
- Houd terwijl je een nummer sleept de aanwijzer bij de boven- of onderrand van
  een lijst om hem te scrollen: hoe dichter bij de rand, hoe sneller hij gaat,
  en hij stopt aan de uiteinden van de lijst of wanneer je van de rand af
  beweegt. Het muiswiel scrolt de lijst ook tijdens het slepen. De violette
  lijn blijft de aanwijzer volgen terwijl de lijst beweegt. Bestanden uit de
  bestandsbeheerder over een lijst slepen scrolt hem op dezelfde manier waar
  het systeem de aanwijzerpositie meldt, zodra je de aanwijzer over de lijst
  beweegt.
- Sleep het naar de lijst van een andere speler om het daarheen te verplaatsen.
- Sleep het naar een tabblad om het aan die playlist toe te voegen.
- Zet bestanden of mappen uit de bestandsbeheerder neer op een lijst om ze op
  de plek van het neerzetten in te voegen. Boven de kop, een kolomrand, de
  schuifbalk of een venster dat de lijst bedekt wordt niets ingevoegd. Meldt het
  systeem de positie niet, dan gaan ze naar het einde van de getoonde lijst.

## Voettekst {#footer}

**+ Toevoegen** opent een bestandsdialoog, beginnend in de muziekmap die is
ingesteld in Instellingen. **Gespeeld wissen** (het pijlpictogram ernaast) wist
de gedimde markering “al gespeeld” van elk nummer van de playlist, voor elke
speler, nadat is gevraagd “De gespeeld-markering van elk nummer in deze
playlist wissen?” (**Annuleren**, Esc of een klik erbuiten behouden de
markeringen). Het nummer dat on air is, behoudt zijn toestand en wordt
gemarkeerd wanneer de speler het verlaat. De knop is gedimd wanneer er niets te
wissen is. De voettekst toont ook het aantal nummers, de tijd die in de
playlist resteert en zijn totale lengte.

## Playlistbestanden {#playlist-files}

- **Importeren:** Instellingen → Playlists → **M3U / PLS importeren…**, of zet
  een `.m3u`-, `.m3u8`- of `.pls`-bestand neer op het venster. Het wordt een
  nieuwe playlist met de naam van het bestand.
  - Relatieve paden worden opgelost vanaf de map van het playlistbestand.
  - `file://`-adressen worden begrepen.
  - Bestanden die niet kunnen worden gevonden, worden toch toegevoegd, als niet
    beschikbaar gemarkeerd.
  - Internetstreams worden overgeslagen; een bericht zegt hoeveel.
- **Exporteren:** de knop **M3U** bij elke playlist in Instellingen slaat hem op
  als M3U8-bestand met titels, lengtes en volledige paden.
