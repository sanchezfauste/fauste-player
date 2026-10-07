# Problemen oplossen

## Geen geluid {#no-sound}

1. Open **Instellingen → Audio-uitgangen** en druk op **Main testen** voor de
   speler. Hoor je de toon, controleer dan de volumefader van de speler.
2. Hoor je niets, kies dan een ander apparaat of kanaalpaar. Wijzigingen aan
   uitgangen werken na een herstart: druk in Instellingen op **Nu herstarten**.
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
  [Bit-perfect uitgang](bit-perfect.md#the-bp-badge): volume op 100 %, geen
  fade, niets anders op dezelfde uitgangen, een lossless bestand dat is
  geanalyseerd, en een apparaat dat op de frequentie van het bestand draait.
- **Een korte stilte vóór een nummer.** Het bit-perfect apparaat is opnieuw
  geopend op de samplefrequentie van het nummer. Houd de bibliotheek op één
  frequentie om dat te voorkomen.
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

## Audio-onderbrekingen {#audio-dropouts}

De statusbalk waarschuwt 5 seconden lang na elke onderbreking die de
applicatie detecteert: **P1: audio-onderbrekingen (3)** wanneer het decoderen
van een speler de schijf niet bijhield (het aantal geldt voor het nummer dat nu
speelt), en **&lt;apparaat&gt;: onderbrekingen van het audioapparaat (2)** wanneer
het uitgangsapparaat een deadline miste (een xrun). Het logbestand registreert
ze ook, hooguit één regel per 10 seconden per soort, met hoeveel er zijn
gebeurd. Niet elk audiosysteem meldt xruns (PulseAudio niet; de exclusieve modus
van Windows niet).


- Verhoog de **buffergrootte** in Instellingen (en druk op **Nu herstarten**).
- Sta op Linux realtime scheduling toe. De applicatie vraagt het systeem erom
  via rtkit (D-Bus). Lidmaatschap van de groep `audio` met een `rtprio`-limiet
  werkt ook.
- Vermijd netwerkschijven voor muziek die on air speelt.

## “De interface had een fout” {#the-interface-hit-an-error}

Er is een tekenfout opgevangen. De audio wordt er niet door beïnvloed. Druk op
**Interface herstarten**. Meld het alsjeblieft samen met de logbestanden.

## Logbestanden en crashrapporten {#logs-and-crash-reports}

Zie [Gegevens en back-ups](data-and-backups.md) voor de logmap. Er is één
logbestand per dag, en de laatste 14 blijven bewaard. Crashrapporten worden
opgeslagen als `crash-<tijd>.txt`. Zet `RUST_LOG=debug` in de omgeving voor
meer detail. Voeg beide bestanden bij wanneer je een bug meldt.
