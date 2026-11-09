# Problemen oplossen

## Geen geluid {#no-sound}

1. Open **Instellingen → Audio-uitgangen** en druk op **Main testen** voor de
   speler. Hoor je de toon, controleer dan de volumefader van de speler.
2. Hoor je niets, kies dan een ander apparaat of kanaalpaar. Een wijziging aan
   de uitgangen wordt toegepast zodra er niets op speelt; **Nu toepassen** in
   Instellingen past haar meteen toe.
3. Kies op Linux bij voorkeur **PipeWire** of **PulseAudio** in Instellingen →
   Audio-uitgangen → Audiosysteem. Ze delen de geluidskaart met andere
   programma’s. **ALSA** praat rechtstreeks met de kaart en kan hem bezet
   vinden.
4. **JACK** staat als niet beschikbaar (“no output device”) wanneer er geen
   JACK-server draait. Start de server (bijvoorbeeld met QjackCtl) en herstart
   de applicatie. Zet de JACK-server op de samplefrequentie uit Instellingen
   (standaard 48 kHz): JACK draait op één frequentie voor alle programma’s.
5. **PipeWire** wordt niet aangeboden door de downloadbare archieven; ze
   bereiken PipeWire via de PulseAudio-service, die op dezelfde manier werkt.
   Het is beschikbaar in builds die met de feature `pipewire` zijn gemaakt.

## Bit-perfect {#bit-perfect}

- **De BP-badge blijft uit.** Controleer elke voorwaarde in
  [Bit-perfect uitgang](bit-perfect.md#the-bp-badge): geen fade, niets
  anders op dezelfde uitgangen, een lossless bestand dat is geanalyseerd, en
  een apparaat dat op de frequentie van het bestand draait.
- **Een korte stilte vóór een nummer.** Het bit-perfect apparaat is opnieuw
  geopend op de samplefrequentie van het nummer. Het gebeurt alleen wanneer de frequentie
  verandert: nummers met dezelfde frequentie volgen elkaar zonder gat op. Houd
  de bibliotheek op één frequentie om het te voorkomen. Zie
  [Wat het kost](bit-perfect.md#what-it-costs).
- **Nummers mixen niet en Play fadet niet.** Op een speler waarvan de
  Main-uitgang bit-perfect is, speelt elk nummer tot het einde en begint dan het
  volgende, en Play / Volgende en Vorige schakelen meteen om: er zijn geen
  crossfades of segues op zo'n speler. Zet bit-perfect uit voor het apparaat, of
  verplaats de speler naar een andere uitgang, om weer te mixen. Zie
  [Wat het kost](bit-perfect.md#what-it-costs).
- **De volumefader beweegt niet.** De Main-uitgang van de speler is
  bit-perfect, wat het volume op 100 % houdt; de tooltip van de fader zegt dat.
  MIDI-faders en de API op afstand kunnen hem ook niet bewegen. Zie
  [Wat het kost](bit-perfect.md#what-it-costs).
- **Een nummer wordt geresampled afgespeeld en het logbestand zegt dat het
  apparaat bezet is (Linux).** Om van frequentie te wisselen sluit de
  applicatie het apparaat en opent ze het opnieuw. Op dat moment kan de
  geluidsserver (PipeWire) de kaart overnemen. De applicatie probeert het een
  paar keer opnieuw; is de kaart nog steeds bezet, dan speelt het nummer op de
  huidige frequentie van het apparaat, en vraagt het volgende nummer opnieuw om
  zijn frequentie. Om de applicatie de kaart voor zichzelf te geven, open je de
  geluidsinstellingen van het systeem en zet je het profiel van die kaart op
  **Uit** (of **Pro Audio**), zodat de geluidsserver zijn `hw:`-apparaat met
  rust laat. Het aantal pogingen en de wachttijd ertussen zijn
  `tuning.device_busy_retries` en `tuning.device_busy_retry_ms` in het
  configuratiebestand.
- **Het apparaat speelt, maar de BP-badge blijft uit (Windows of macOS).**
  Exclusieve toegang is geweigerd en het apparaat speelt gedeeld.
  - Windows: een ander programma kan het apparaat exclusief vasthouden, of
    exclusieve controle staat uit in de geavanceerde eigenschappen van het
    apparaat.
  - macOS: een ander programma kan het apparaat in hog-modus houden, of het
    apparaat biedt zijn frequenties alleen als continu bereik (de meeste
    interfaces tonen vaste frequenties).
- **Een `hw:`-apparaat kan niet worden geopend (Linux).**
  - Een geluidsserver kan de kaart vasthouden. Stop hem, of stel de server in
    om die kaart met rust te laten, en herstart de applicatie.
  - Sommige USB-DAC’s accepteren alleen gepakte 24-bit samples (`S24_3LE`), die
    de audiobibliotheek niet ondersteunt. Gebruik die kaart in plaats daarvan
    via `plughw:` (niet bit-perfect).

### DSD {#dsd}

- **Een DSD-nummer wordt omgezet afgespeeld hoewel het apparaat op DoP of
  native DSD staat.** Het logbestand zegt waarom (“DSD converted to PCM” en de
  reden) bij deze oorzaken: het volume van de speler staat niet op 100 %, er
  speelt iets anders op het apparaat, meer dan twee kanalen, of een apparaat
  dat de frequentie weigert (DoP vereist de DSD-frequentie gedeeld door 16,
  bijvoorbeeld 176,4 kHz voor DSD64) of geen 24- of 32-bit formaat heeft. Een
  nummer dat nog niet is geanalyseerd, wordt stilzwijgend omgezet, zonder
  logregel: analyseer het (Instellingen → Analyse) en speel het opnieuw af.
- **Alleen het eerste nummer van een DSD-album gaat als DSD naar buiten.** Dat
  is de standaard mixinstelling: de nummers die de speler zelf start, spelen
  omgezet. Kies **DSD behouden en de andere bronnen dempen** in Instellingen →
  Audio-uitgangen om ze DSD te houden. Zie [DSD](bit-perfect.md#dsd).
- **De kop toont DSD maar de converter speelt ruis of vergrendelt niet.** De
  converter herkent DoP (of het native formaat) niet. Zet het apparaat terug
  op **Naar PCM omzetten**.
- **Een klik wanneer een DSD-nummer begint, stopt of DSD verlaat.** De
  converter heeft meer DSD-stilte nodig: verhoog **DSD-stilte** (standaard
  200 ms) in Instellingen → Audio-uitgangen, Geavanceerd.
- **Andere spelers of carts zijn stil op het apparaat.** Er speelt een
  DSD-nummer met **DSD behouden en de andere bronnen dempen**; de badge
  **Andere gedempt** wordt getoond. Ze klinken weer wanneer het nummer
  eindigt.

## Melding “Uitgang kwijt” {#output-lost-alert}

De statusbalk toont **Uitgang kwijt: &lt;apparaat&gt;** wanneer een apparaat niet
meer reageert. De spelers blijven tellen en mixen op een interne klok, zodat de
automatisering niet vastloopt. Het apparaat wordt elke 2 seconden opnieuw
geprobeerd en neemt het weer over wanneer het terugkomt. Sluit de kabel weer
aan of zet de interface weer aan.

### “Uitgang kwijt” die nooit verdwijnt, met een directe `hw:`-uitgang {#output-lost-that-never-clears-with-a-direct-hw-output}

Een geluidskaart die via een directe ALSA-uitgang `hw:` wordt gebruikt (bijvoorbeeld
een bit-perfect uitgang), wordt door Fauste Player alleen vastgehouden: de
geluidsserver (PipeWire of PulseAudio) kan hem niet tegelijk gebruiken. Gaat
een andere uitgang via het standaardapparaat van de geluidsserver en is dat
standaardapparaat dezelfde kaart, dan start die uitgang nooit en blijft hij op
**Uitgang kwijt** staan. Het logbestand zegt eenmalig “output device opened
but never started”.

Gebruik één pad per kaart: leid elke uitgang van die kaart via hetzelfde
`hw:`-apparaat (zo nodig met andere kanalen), of kies in de geluidsinstellingen
van je systeem een andere kaart als standaarduitgang van de geluidsserver.

## Een nummer toont een waarschuwingspictogram of een bestand met een kruis {#a-track-shows-a-warning-icon-or-a-file-with-a-cross}

Het bestand ontbreekt (verplaatst, verwijderd, ontkoppeld: een bestand met een
kruis) of kan niet worden gedecodeerd (een waarschuwingsteken). De spelers
slaan het over. Beweeg de muisaanwijzer over de rij om te zien welk van de twee
het is, en het pad van het bestand.

Een ontbrekend bestand wordt elke 30 seconden opnieuw gezocht
(`tuning.missing_recheck_ms` in het configuratiebestand): wanneer de schijf
wordt gekoppeld of het bestand wordt teruggezet, wordt het nummer vanzelf weer
afspeelbaar. Een bestand dat niet kan worden gedecodeerd, wordt vanzelf met
dezelfde timer opnieuw gecontroleerd, op grootte en wijzigingstijd: het wordt
niet opnieuw gedecodeerd tenzij een van beide is veranderd, bijvoorbeeld
wanneer een kopie klaar is. Om het meteen te controleren gebruik je
**Opnieuw analyseren** in het rijmenu, of **Instellingen → Analyse → Alle
nummers opnieuw analyseren** voor de hele bibliotheek.

Een WAV of FLAC die als onleesbaar wordt getoond, of als harde ruis zou spelen,
kan een DTS-CD-rip zijn: het bestand bevat een DTS-bitstream in plaats van
audio. De applicatie weigert het (het log zegt “encoded bitstream stored as
PCM”) en decodeert geen DTS. Zet het eerst met een ander hulpmiddel om naar
gewone PCM.

## Een bewaakte map toont een waarschuwing {#a-watched-folder-shows-a-warning}

Een categorie die een map bewaakt, toont ⟳, in een andere kleur wanneer de map
niet wordt gevolgd; houd de muisaanwijzer erboven voor de reden. Zie [Bewaakte
mappen](library.md#watched-folders).

- **Niet gevonden of niet te lezen**: de schijf of share is niet gekoppeld of
  niet bereikbaar. Een lege map die nummers van de bibliotheek zou moeten
  bevatten, telt ook als niet gevonden, omdat een niet-gekoppelde share er vaak
  zo uitziet. Er wordt niets gemarkeerd of verwijderd; de map wordt elke 30
  seconden opnieuw gezocht en gescand zodra hij verschijnt. Heb je de map
  met opzet geleegd, wis dan zijn nummers door ze te selecteren in de
  bibliotheektabel en **Uit de bibliotheek verwijderen…** te kiezen, of door
  **Opnieuw analyseren** erop te kiezen, wat de nummers waarvan het bestand weg
  is als ontbrekend markeert zodat **Ontbrekende nummers verwijderen…** ze kan
  verwijderen.
- **Wijzigingen worden niet gevolgd**: het systeem wilde hem niet bewaken,
  meestal een map met heel veel submappen. De map wordt nog steeds bij elke
  start en bij elke nieuwe poging gescand, dus nieuwe bestanden verschijnen dan,
  niet meteen. Op Linux heeft een maptree die groter is dan de limiet van
  bewakingen per gebruiker een hogere `fs.inotify.max_user_watches` nodig
  (bijvoorbeeld `sudo sysctl fs.inotify.max_user_watches=524288`; maak het
  blijvend in `/etc/sysctl.d/`).
- **Geen volledig pad**: de opgeslagen map is met de hand bewerkt tot een pad
  dat niet volledig is. Kies **Bewaakte map wijzigen…** om hem opnieuw te
  kiezen.
- **Een netwerkshare toont geen waarschuwing, maar bestanden die vanaf een
  andere computer zijn toegevoegd verschijnen niet**: op een netwerkshare
  (SMB/CIFS, NFS) accepteert het systeem de bewaking meestal, maar meldt het
  alleen de wijzigingen die vanaf deze computer zijn gemaakt. Wijzigingen die
  elders zijn gemaakt, worden bij de volgende start gezien, niet terwijl de
  applicatie draait.
- **Een map met duizenden bestanden** heeft even nodig om bij de start te
  worden gescand; de voortgang van de analyse toont het.
- Nummers waarvan het bestand een bewaakte map heeft verlaten, blijven in de
  bibliotheek als ontbrekend gemarkeerd. Verwijder ze met **Ontbrekende nummers
  verwijderen…** in het menu van de categorie.

## Audio-onderbrekingen {#audio-dropouts}

De statusbalk waarschuwt 5 seconden lang na elke onderbreking die de
applicatie detecteert: **P1: audio-onderbrekingen (3)** wanneer het decoderen
van een speler de schijf niet bijhield (het aantal geldt voor het nummer dat nu
speelt), en **&lt;apparaat&gt;: onderbrekingen van het audioapparaat (2)** wanneer
het uitgangsapparaat een deadline miste (een xrun). Het logbestand registreert
ze ook, hooguit één regel per 10 seconden per soort, met hoeveel er zijn
gebeurd. Niet elk audiosysteem meldt xruns (PulseAudio niet; de exclusieve modus
van Windows niet).


- Verhoog de **buffergrootte** in Instellingen (ze wordt toegepast zodra er niets
  op dat apparaat speelt, of meteen met **Nu toepassen**).
- Via een geluidsserver (PulseAudio, het ALSA-apparaat `default`) vraagt elke
  uitgang al om minstens 50 ms, wat de buffergrootte ook is, zodat een drukke
  computer niet te horen is; Instellingen toont het naast de latentie. Verhoog
  op een oude of zwaarbelaste computer `shared_buffer_ms` in de sectie
  `outputs` van het configuratiebestand (bijvoorbeeld naar 100). Het systeem
  houdt ongeveer twee van die blokken in de wachtrij, dus cart- en
  play-knoppen klinken dan ongeveer twee keer zo veel van de verhoging later.
  Voor de on-air uitgang vermijdt het eigen hardware-item van het apparaat
  (`hw:` op Linux) de geluidsserver helemaal.
- Sta op Linux realtime scheduling toe. De applicatie vraagt het systeem erom
  via rtkit (D-Bus), en het log noteert een weigering. Veel bureaubladen
  weigeren het aan applicaties.
- Vermijd netwerkschijven voor muziek die on air speelt.

## De interface is traag op een oude computer {#the-interface-is-slow-on-an-old-computer}

Terwijl er iets speelt, tekent de interface hooguit 30 beelden per seconde
(`max_fps` in de sectie `ui` van het configuratiebestand), en klikken en
toetsen worden meteen beantwoord. Als de beelden meer kosten dan een deel van
de processor (`frame_budget_percent`, standaard 40% van één kern), tekent hij
vanzelf minder beelden, tot `min_fps` (standaard 8), en gaat weer omhoog als
ze goedkoper worden. Dat gebeurt vooral op computers zonder bruikbare
grafische kaart, waar de processor het tekenen doet: het logboek meldt het bij
het starten ("the interface draws in software"). Om nog meer van de processor
aan de audio over te laten, verlaag `max_fps` of `frame_budget_percent`; 0 in
`max_fps` tekent elk beeld dat het scherm toont. De meters meten hetzelfde bij
elke snelheid; ze worden alleen minder vaak getekend. Als er niets speelt,
tekent de interface één keer per seconde (voor de klok) en telkens wanneer er
iets verandert.

## Bestanden kunnen op Linux niet worden neergezet {#files-cannot-be-dropped-on-linux}

Het venster neemt bestanden die uit een bestandsbeheerder worden gesleept alleen
aan wanneer het op X11 draait. Standaard doet het dat, ook op een
Wayland-bureaublad (via XWayland, dat elk bureaublad heeft). Staat
`ui.window_system` in `config.json` op `"Wayland"`, dan is het venster een
native Wayland-venster, dat er bij fractionele schaling scherper uit kan zien,
maar bestanden kunnen er niet op worden neergezet: gebruik in plaats daarvan
**+ Toevoegen**. Zet het terug op `"Auto"` (de standaard) of `"X11"` en herstart
Fauste Player.

## Een MIDI-bedieningspaneel reageert niet {#a-midi-controller-does-not-respond}

Toont **Instellingen → MIDI** het bedieningspaneel als verbonden, maar doen zijn
knoppen en faders niets, druk daar dan op **Poorten opnieuw verbinden**: het
programma sluit de MIDI-poorten en opent ze opnieuw, alsof je het
bedieningspaneel loskoppelt en weer aansluit. Helpt dat niet, koppel het
bedieningspaneel dan los en sluit het weer aan.

In beide gevallen zegt het log wat er gebeurde. Het vermeldt de MIDI-poorten
telkens als ze veranderen, elke poort die wordt geopend en gesloten (met
waarom), een poort die niet kan worden geopend (eenmalig, totdat hij opent of
verdwijnt), het eerste bericht dat op elke verbinding is ontvangen, het eerste
bericht dat op elke verbinding is genegeerd met de reden (geen koppeling, een
fader die het volume nog niet heeft bereikt, een actie die nu niet kan werken)
en het eerste LED-bericht dat mislukte. Een verbonden bedieningspaneel dat nooit
een eerste bericht logt, stuurt niets dat het programma ontvangt. Start het
programma voor elk genegeerd bericht en elke mislukte nieuwe poging met
`RUST_LOG=info,fp_control=debug`.

## “De interface had een fout” {#the-interface-hit-an-error}

Er is een tekenfout opgevangen. De audio wordt er niet door beïnvloed. Druk op
**Interface herstarten**. Meld het alsjeblieft samen met de logbestanden.

## Logbestanden en crashrapporten {#logs-and-crash-reports}

Zie [Gegevens en back-ups](data-and-backups.md) voor de logmap. Er is één
logbestand per dag. Identieke regels die direct na elkaar herhalen, worden
eenmaal geschreven, gevolgd door een regel zoals `previous message repeated 12
times` (tijdens een lange reeks herhalingen wordt die regel elke minuut
geschreven). Regels die verschillen, bijvoorbeeld omdat ze een teller bevatten,
worden niet samengevoegd. De routineuze verbindingsberichten van de
PulseAudio-client worden standaard weggelaten. Zet `RUST_LOG=debug` in de
omgeving voor meer detail (het vervangt het standaardfilter,
`info,pulseaudio=warn`).

Het log is begrensd: het bestand van een dag groeit niet verder dan 10 MiB (de
laatste regel zegt dat; de rest van die dag wordt niet gelogd, wat een stroom
van verschillende regels veroorzaakt) en de laatste 14 bestanden blijven
bewaard. Wijzig ze in het configuratiebestand met `limits.max_log_file_mb` (1
tot 1024), `limits.max_log_files` (2 tot 365) en
`limits.log_repeat_summary_secs` (1 tot 3600, standaard 60, hoe vaak een lange
reeks herhalingen wordt gemeld); ze gelden vanaf de volgende start.
Crashrapporten worden in dezelfde map opgeslagen als `crash-<tijd>.txt` en
hebben een eigen limiet (`limits.max_crash_reports` per run).
Voeg beide bestanden bij wanneer je een bug meldt.
