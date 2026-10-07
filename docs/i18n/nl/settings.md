# Instellingen

Open **Instellingen** in de bovenbalk. Sluit het met **Sluiten** of `Esc`. De
meeste wijzigingen werken meteen en worden automatisch opgeslagen.

Het venster heeft één formaat (900 × 640, kleiner op een klein scherm), welke
sectie ook, en de sectie scrolt erin. Elke sectie lijnt haar labels uit in één
kolom.

De secties Spelers, Meters, Analyse en Sneltoetsen hebben in hun kop een knop
**Standaardwaarden herstellen**. Die vraagt om bevestiging en zet dan alleen die
sectie terug (Spelers behoudt het aantal spelers en de taal; Sneltoetsen heeft
geen andere resetknop). Audio-uitgangen, Playlists, Cartwall, MIDI en Op
afstand hebben er geen.

## Herstart nodig {#restart-pending}

Sommige wijzigingen werken pas wanneer de applicatie opnieuw start: het
audiosysteem, de samplefrequentie, de buffergrootte (ook die van een apparaat
zelf), de Main- en Cue-uitgangen (spelers en cartwall), de bit-perfect
apparaten en de DSD-instellingen. Het aantal spelers hoort daar niet bij: het
werkt meteen.

Een frequentie of buffer die aan een apparaat wordt gegeven, telt alleen mee
wanneer ze verandert waarmee het apparaat opent: een apparaat dezelfde waarde
geven als de globale, of zo’n waarde wissen, is niet in afwachting.

De limieten en de engine-afstelling werken ook bij de volgende start, maar ze
worden bewerkt in het configuratiebestand terwijl de applicatie gesloten is
(zie [Gegevens en back-ups](data-and-backups.md)), dus ze verschijnen nooit als
in afwachting.

Terwijl een van deze wacht, zegt de voettekst van Instellingen “Sommige
wijzigingen werken pas na een herstart.” en biedt **Nu herstarten** aan, en
toont de bovenbalk een label **Herstart nodig**. Houd de muisaanwijzer boven het
label om te zien wat er wacht. Een korte melding (bijvoorbeeld dat een
instelling is opgeslagen) kan een moment de plaats van de voettekst innemen;
**Nu herstarten** blijft. Beide doen hetzelfde:

- Wanneer niets on air is, herstart **Nu herstarten** (of het label) meteen.
- Wanneer er iets on air is, verschijnt het venster dat opsomt wat er klinkt,
  met **Stoppen en herstarten** of **Annuleren**.

De sessie wordt eerst opgeslagen en de audio en MIDI-besturing stoppen, daarna
start de applicatie opnieuw met dezelfde gegevensmap (`FAUSTE_HOME`), en er
gaat daarna niets uit zichzelf on air. Kan de applicatie niet opnieuw starten
(in een Flatpak ook wanneer de nieuwe niet op tijd start), dan zegt ze dat;
start haar dan vanuit je programmamenu.

## Audio-uitgangen {#audio-outputs}

Wijzigingen in deze sectie wachten op een herstart: zie
[Herstart nodig](#restart-pending).

![Instellingen, Audio-uitgangen, weergave Basis: de keuzeschakelaar, het audiosysteem, de samplefrequentie, de buffergrootte en de Main- en Cue-uitgangen van elke speler (hier het stille systeem)](../../images/guide/settings-outputs.png)

Bovenaan kiest **Tonen** tussen **Basis** en **Geavanceerd**. Basis toont het
audiosysteem, de samplefrequentie, de buffergrootte en de uitgangen.
Geavanceerd voegt voor elk apparaat dat een uitgang gebruikt zijn eigen
frequentie en buffer, de bit-perfect-schakelaar en de DSD-modus toe, en daarna
de DSD-instellingen. Van weergave wisselen toont of verbergt alleen rijen: er
wordt niets gewijzigd of teruggezet. Wanneer Basis een instelling verbergt die
in gebruik is, zegt een regel dat. Wanneer geen enkele uitgang een apparaat nog
gebruikt, worden zijn eigen frequentie en buffer, zijn bit-perfect-schakelaar en
zijn DSD-modus vergeten de volgende keer dat de applicatie start: gebruikt een
uitgang het daarna weer, dan begint het met de globale waarden. Tot dan behoudt
het opnieuw kiezen ervan (bijvoorbeeld na het omwisselen van twee apparaten) ze.

| Instelling | Betekenis |
|---|---|
| Audiosysteem | De laatste keuze, **Geen uitgang (stil)**, speelt niets af: tijdlijnen lopen in realtime zonder geluidskaart (voor een machine zonder kaart, of om te oefenen). Linux: PipeWire (in builds die het bevatten), PulseAudio, JACK of ALSA. Windows: WASAPI, ASIO (in builds die het bevatten) of JACK. macOS: Core Audio of JACK. Systemen die op deze computer ontbreken, of zonder uitvoerapparaat (een JACK-server die niet draait), worden als niet beschikbaar getoond. “Systeemstandaard” gebruikt de eerste beschikbare in die volgorde. |
| Samplefrequentie | De frequentie waarop elke uitgang draait, tenzij een apparaat een eigen heeft (Geavanceerd); bestanden worden ernaar omgezet met hoogwaardige resampling. Bit-perfect apparaten starten op hun frequentie en volgen daarna de bestanden. |
| Buffergrootte | Frames per audioblok, tenzij een apparaat een eigen heeft; de resulterende latentie wordt eronder getoond |
| Uitgangen per speler | Voor elke speler een **Main**-apparaat (on air) en een **Cue**-apparaat (voorbeluisteren), elk met een kanaalpaar. Een geluidskaart die meerdere uitvoerprofielen biedt (ALSA toont front, surround, direct hardware…) toont elk als *kaart — profiel*; twee items die nog steeds hetzelfde zouden lezen, krijgen hun apparaat-id tussen haakjes. Meerkanaals interfaces kunnen meerdere spelers op verschillende paren dragen. |
| Main testen / Cue testen | Speelt een korte toon (1 kHz op Main, 440 Hz op Cue, 1,5 s, −18 dBFS) af op de gekozen uitgang, zodat je de bedrading kunt controleren voordat je on air gaat |
| Cartwall | De Main- en Cue-uitgangen van de cartwall. Main staat standaard op de systeemuitgang. Zonder Cue is er geen voorbeluisteren van carts. |
| Samplefrequentie: *apparaat* (Geavanceerd) | **Globaal (...)** gebruikt de samplefrequentie hierboven; een waarde geeft dit apparaat een eigen frequentie. Alleen de frequenties die het apparaat meldt worden aangeboden; een opgeslagen frequentie die het niet meer meldt, blijft in de lijst met een opmerking dat hij misschien niet opent (het apparaat valt dan terug op de globale frequentie). De eigen waarden gelden alleen voor apparaten die een uitgang noemt, niet voor de systeemstandaarduitgang tenzij een uitgang hem noemt. |
| Buffergrootte: *apparaat* (Geavanceerd) | **Globaal (...)** gebruikt de buffergrootte hierboven; een waarde geeft dit apparaat een eigen, met zijn latentie eronder. Een apparaat dat geen eigen buffergrootte aanneemt, valt terug op de globale, en ook op de globale frequentie wanneer het geen eigen frequentie aanneemt. |
| Bit-perfect: *apparaat* (Geavanceerd) | Een bit-perfect apparaat wordt met exclusieve toegang geopend en volgt de samplefrequentie van elk bestand zolang er niets op speelt. De schakelaar is uitgeschakeld waar het apparaat geen exclusieve toegang kan geven. Zie [Bit-perfect uitgang](bit-perfect.md). |
| DSD: *apparaat* (Geavanceerd) | **Naar PCM omzetten** (de standaard), **DoP** of, op Linux, **Natieve DSD**. Elk apparaat toont het; alleen de modi die het apparaat aankan worden aangeboden, en een regel eronder zegt waarom de andere niet. Zie [DSD](bit-perfect.md#dsd). |
| Als een andere bron een DSD-uitgang nodig heeft (Geavanceerd) | **Het DSD-nummer als PCM voortzetten** (de standaard), of **DSD behouden en de andere bronnen dempen**. Zie [DSD](bit-perfect.md#dsd). |
| DSD-stilte (Geavanceerd) | Stilte die wordt verzonden voordat een DSD-stream begint, nadat hij eindigt en bij een overgang naar PCM, zodat de converter zonder klik vergrendelt; standaard 200 ms, 0 tot 2000. |

Een Cue-uitgang valt nooit terug op de uitgang die Main gebruikt, zodat
voorbeluisteren nooit on air gaat. Een Cue die een apparaat noemt op een
audiosysteem dat deze computer niet heeft, of dezelfde uitgang (apparaat en
kanalen) als Main, betekent “geen cue”. Wanneer een Cue-uitgang dezelfde is als
zijn Main-uitgang, zegt een waarschuwing eronder dat. Een speler zonder
Cue-uitgang, of met zijn Cue op zijn Main-uitgang, heeft zijn knop **CUE**
gedimd; de muisaanwijzer erboven zegt dat je hier een Cue-uitgang moet kiezen.
Hetzelfde geldt voor **Voorbeluisteren op CUE** van de cartwall.

Verdwijnt een apparaat tijdens het afspelen, dan behouden de spelers hun
tijdlijnen, en wordt het apparaat heropend wanneer het terugkomt (zie
[Problemen oplossen](troubleshooting.md)).

## Spelers {#players}

![Instellingen, Spelers: aantal spelers, standaardmodus, fadetijd, automatisch mixen, cue-in en cue-out, waarschuwing einde nummer en taal](../../images/guide/settings-players.png)

| Instelling | Standaard | Betekenis |
|---|---|---|
| Taal | Systeem | Interfacetaal |
| Aantal spelers | 4 | Kolommen op het hoofdscherm (een speler die on air is, kan niet worden verwijderd) |
| Standaardmodus | CONT | De modus waarin spelers starten |
| Fadetijd | 1000 ms | Gebruikt door Play terwijl on air en door Uitfaden |
| Automatisch mixen op het MIX-punt | Aan | Nummers laten overlappen in doorlopende modus |
| Cue-in en cue-out gebruiken | Aan | Uit: spelers spelen elk nummer van het begin tot het einde van het bestand; de markers blijven bewaard en carts gebruiken de hunne nog steeds. Duur en playlisttotalen volgen hetzelfde bereik |
| Waarschuwing einde nummer | 10 s | Wanneer de aftelling rood begint te knipperen |

## Meters {#meters}

![Instellingen, Meters, met de digitale piekmeter gekozen](../../images/guide/settings-meters.png)

Wijzigingen werken meteen. Instellingen toont alleen wat het gekozen metertype
gebruikt: de EBU-, DIN- en VU-meters hebben de schaal, rode zone en het gedrag
die hun standaard vastlegt (alleen het uitlijnniveau wordt ingesteld), en de
uitlijning van een K-System-meter is zijn eigen 0. Een waarde die je instelt,
blijft bewaard voor wanneer je dat type opnieuw kiest.

| Instelling | Standaard | Betekenis |
|---|---|---|
| Metertype | Digitale piek | Hoe de balk stijgt en daalt, en zijn schaal, volgens een standaard (zie hieronder) |
| Stijgtijd, Daalsnelheid | 5 ms, 11,8 dB/s | Alleen voor **Aangepast**. De stijgtijd is een integratietijd: een toonburst van die lengte leest 2 dB te laag; 0 toont elke piek. |
| True peak | Uit | Alleen digitale piek, aangepast en K-System. Meet tussen samples, met het 4×-oversamplingfilter dat ITU-R BS.1770 publiceert. Het toont pieken die na conversie 0 dBFS overschrijden, die een samplepiekmeter mist. Zoals de standaard toestaat, kan een geïsoleerde klik van één sample tot ongeveer 0,3 dB onder zijn samplewaarde lezen. |
| Ondergrens schaal | −60 dBFS | De onderkant van de digitale schaal (digitale piek en aangepast). De andere meters tonen het bereik dat hun standaard geeft. |
| Piekvasthouding | 2 s | Alleen digitale piek, aangepast en K-System: hoe lang het hoogste niveau verlicht blijft; 0 zet het uit. Programmameters en de VU hebben geen vasthouding. |
| Uitlijnniveau | −18 dBFS | Alle behalve het K-System. Gemarkeerd op de schaal (EBU R68). Daar staan ook de EBU TEST-markering, de DIN −9-markering en 0 VU. |
| Waarschuwing vanaf | −9 dBFS | Geel vanaf hier (EBU toegestaan maximum), voor de digitale piek- en aangepaste meters |
| Gevaar vanaf | −3 dBFS | Rood vanaf hier, voor de digitale piek- en aangepaste meters. De andere worden rood waar hun schaal dat doet: VU vanaf 0 VU, EBU en DIN PPM vanaf het toegestane maximum (EBU +9, DIN 0), het K-System vanaf +4. |
| Loudnessweergave | Korte termijn | De loudness onder de meter: uit, momentaan (laatste 400 ms) of korte termijn (laatste 3 s), EBU R128 |
| Loudnessdoel | −23 LUFS | De weergave is groen binnen ±1 LU (EBU R128) |

| Metertype | Norm | Gedrag |
|---|---|---|
| Digitale piek | IEC 60268-18 | Toont elke piek meteen; valt 20 dB in 1,7 s |
| EBU PPM | IEC 60268-10 type IIb | Pieken korter dan ongeveer 10 ms lezen lager (een toonburst van 10 ms leest ongeveer 1,6 dB te laag, een van 0,5 ms ongeveer 18 dB te laag), binnen de toleranties van EBU Tech 3205; valt 24 dB in 2,8 s |
| DIN PPM | IEC 60268-10 type I | Hetzelfde met een integratietijd van 5 ms; valt 20 dB in 1,5 s |
| VU | IEC 60268-17 | Het gemiddelde niveau, met de naaldbeweging van een VU-meter: 99 % in 300 ms, met een lichte overshoot; een sinus leest zijn piekniveau |
| K-20, K-14, K-12 | K-System | Twee delen: het gemiddelde (RMS, 600 ms) als volle balk en de piek (valt 26 dB in 3 s) gedimd erboven. 0 is 20, 14 of 12 dB onder full scale; groen onder 0, amber van 0 tot +4, rood erboven. K-12 past bij broadcast, K-14 en K-20 bij dynamischer programma’s. |
| Aangepast | — | Jouw stijgtijd en daalsnelheid |

Elke meter gebruikt de schaal van zijn standaard, met zijn markeringen tussen
de kanalen:

| Meter | Schaal |
|---|---|
| Digitale piek, aangepast | −60 … 0 dBFS, markeringen elke 10 dB tot −40 en elke 5 dB erboven; de bovenste 20 dB nemen de helft van de hoogte in |
| EBU PPM | −12 … +12 rond het uitlijnniveau (TEST), elke 4 dB; zachtere niveaus rusten onderaan |
| DIN PPM | −50 … +5, waar 0 gelijk is aan 9 dB boven het uitlijnniveau (standaard −9 dBFS) |
| VU | −20 … +3 VU, 0 VU op het uitlijnniveau; de balk beweegt evenredig met de spanning, zoals de naald |
| K-System | van +20, +14 of +12 (0 dBFS) tot −60; gelijkmatig in dB tot −24 |

## Analyse {#analysis}

![Instellingen, Analyse: de drempels van de automatische markers](../../images/guide/settings-analysis.png)

De drempels beschreven in [Markers en mixen](markers-and-mixing.md). **Alle
nummers opnieuw analyseren** voert de analyse opnieuw uit voor de hele
bibliotheek; handmatige markers blijven bewaard.

Na een update waarvan de analyse is veranderd, behouden nummers die de vorige
versie heeft geanalyseerd hun markers en golfvormen, die nog steeds werken.
Bij het opstarten zegt Fauste Player hoeveel het er zijn en biedt **Nu
analyseren** of **Later** aan; **Verouderde nummers analyseren (N)** hier doet
hetzelfde op elk moment. De nummers op de spelers worden hoe dan ook bijgewerkt,
zoals ze worden getoond, en dat geldt ook voor de nummers van carts die geen
vastgelegd formaat hebben (een cart speelt alleen bit-perfect wanneer zijn
formaat bekend is). Nummers waarvan het bestand ontbreekt, worden niet geteld
totdat het bestand terug is.

## Playlists {#playlists}

![Instellingen, Playlists: de muziekmap, de playlists en de tabelkolommen](../../images/guide/settings-playlists.png)

- **Muziekmap:** waar de bestandsdialogen beginnen.
- **Nieuwe playlist**, **hernoemen** (bewerk de naam en druk op Enter; Esc
  annuleert) en **verwijderen** (prullenbakpictogram).
- **M3U / PLS importeren…** maakt een nieuwe playlist van een
  afspeellijstbestand. **M3U** op elke rij exporteert hem als M3U8. Zie
  [Playlists](playlists.md).
- **Tabelkolommen:** welke kolommen de nummertabellen tonen en in welke
  volgorde, voor elke speler: een selectievakje per kolom (Titel en Duur kunnen
  niet worden uitgezet), pijlen omhoog en omlaag voor de getoonde, en
  **Standaardkolommen**. Zie [Playlists](playlists.md).

**Taal:** een keuzelijst: **Systeem** (het besturingssysteem volgen), daarna
elke taal waarin de interface beschikbaar is, elk onder zijn eigen naam
(Engels eerst, daarna alfabetisch: bijvoorbeeld Español). De interface
schakelt meteen. Een systeemtaal zonder eigen vertaling gebruikt de dichtstbijzijnde
(Canadees Frans gebruikt Frans, Braziliaans Portugees gebruikt Portugees), en
anders Engels. Een taal in het instellingenbestand die de interface niet heeft,
wordt als **Systeem** getoond en volgt het besturingssysteem.

Engels en Spaans zijn met de hand geschreven. De andere vertalingen zijn met AI
gemaakt en kunnen fouten bevatten; wanneer een ervan in gebruik is, zegt
**Over Fauste Player** dat. Correcties van moedertaalsprekers zijn welkom als
issue of pull request.

## Cartwall {#cartwall}

![Instellingen, Cartwall: de pagina’s, het raster en de editor van de geselecteerde cart](../../images/guide/settings-cartwall.png)

Pagina’s, rastergrootte, de cart-editor, en het importeren en exporteren van
cartpagina’s. Zie [Cartwall](cartwall.md).

## Sneltoetsen {#keyboard-shortcuts}

![Instellingen, Sneltoetsen: elke spelersactie met zijn toets, en Ontkoppelen naast de gekoppelde](../../images/guide/settings-shortcuts.png)

Zie [Toetsenbord](keyboard.md).

## MIDI {#midi}

![Instellingen, MIDI, met MIDI-besturing uit](../../images/guide/settings-midi.png)

Zet MIDI-bedieningspanelen aan, bekijk de invoerpoorten en leer een
bedieningselement voor elke spelersactie. Zie
[MIDI-bedieningspanelen](midi.md).

## Op afstand {#remote}

![Instellingen, Op afstand, met de HTTP-API luisterend op deze computer](../../images/guide/settings-remote.png)

Bediening op afstand via het netwerk, voor webpagina’s, telefoon-apps,
automatisering en bedieningspanelen. Zie
[Bediening op afstand](remote-control.md).

- **Bediening op afstand via HTTP toestaan**, het **adres** en de **poort** ervan,
  en een regel die zegt of er geluisterd wordt.
- **Token**, vereist buiten deze computer. **Genereren** maakt een willekeurig
  token, **Tonen** onthult het en **Kopiëren** zet het op het klembord. Er
  verschijnt een waarschuwing wanneer het adres verder reikt dan deze computer
  en er geen token is.
- **Webpagina’s die de API mogen gebruiken**: één origin per regel.
- **Bediening via OSC toestaan**, het **adres** en de **poort** ervan, en de
  **toegestane afzenders** (adressen of subnetten, één per regel).
- **Tijden publiceren elke**: hoe vaak verstreken en resterende tijden worden
  verzonden terwijl er iets speelt.

Tekstvelden en getallen worden toegepast wanneer je ze verlaat, wat ook
inhoudt dat je een andere sectie opent of Instellingen sluit; Esc annuleert wat
je typte. Een ongeldige waarde wordt gecorrigeerd, en het veld toont wat is
behouden.
