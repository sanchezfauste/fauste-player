# Spelers

Elke kolom is een speler. Spelers zijn onafhankelijk: elk heeft zijn eigen
playlisttabbladen, transport, volume en uitgangen.

![Speler 1 on air: kop, hoes, titel, volgend nummer, transport, aftelling, meter, fader en golfvorm](../../images/guide/player.png)

## Kop {#header}

| Element | Betekenis |
|---|---|
| `P1` … `Pn` | Spelernummer (de cijfertoets die hem laat spelen) |
| Statusstip en -label | **On air** (rood), **Gepauzeerd** (amber), **Gestopt** (grijs) |
| Badge **Mixen** / **Faden** | Er loopt een crossfade naar het volgende nummer, of een uitfade tot stop |
| Badge **Stop na** | De speler stopt wanneer het huidige nummer eindigt |
| Badge **Herhalen** / **Stop na nummer** | Het huidige nummer wordt herhaald, of stopt de speler wanneer het eindigt, vanwege zijn eigen markering in het playlistmenu. Houd de muisaanwijzer erboven voor de volledige zin. De eigen knop **Stop na** van de speler gaat voor: zolang die aanstaat, wordt alleen zijn badge getoond |
| **BP** / **DSD** | **BP** brandt zolang het huidige nummer zijn Main-apparaat ongewijzigd bereikt. **DSD** vervangt het zolang een DSD-nummer als DSD ongewijzigd naar buiten gaat (zie [Bit-perfect uitgang](bit-perfect.md)). **Andere gedempt** verschijnt ernaast wanneer die DSD-stream andere bronnen van de uitgang weghoudt |
| **SINGLE** \| **CONT** | Afspeelmodus (zie hieronder): één samengevoegd bedieningselement, de verlichte helft is de actieve modus |
| **CUE** | Het volgende nummer voorbeluisteren op de CUE-uitgang |

## Infokolom {#info-row}

- **Hoes** van het nummer dat on air is, of een vinylplaatje als plaatsvervanger.
- **Titel en artiest** van het nummer dat on air is. Een gestopte speler toont
  het nummer dat Play zal starten (zijn volgende), met zijn hoes, zijn lengte
  en zijn golfvorm, klaar bij de cue-in, of waar je op zijn golfvorm hebt
  geklikt.
- Een nummer dat is afgespeeld of geladen voordat zijn analyse klaar is, heeft
  al zijn lengte wanneer de header van het bestand die geeft: de aftelling,
  `verstreken / totaal` en klikken om te springen werken meteen, boven een
  vlakke lijn totdat de golfvorm klaar is. Slaat de header de lengte niet op
  (ruwe AAC-bestanden, MP3-bestanden zonder lengteframe, Matroska en WebM),
  dan staat bij het totaal “—” en kan er niet op de golfvorm worden geklikt
  totdat de analyse klaar is.
- **Stereometer** (in de kolom rechts van de speler, naast de fader; hij
  beslaat de infokolom en het transport): het niveau dat de speler uitstuurt,
  na zijn volume.
  - Eén doorlopende balk per kanaal, op de schaal van de standaard van het
    metertype (standaard de digitale piekmeter: −60 tot 0 dBFS, waarbij de
    bovenste 20 dB de helft van de hoogte innemen). De schaal is een liniaal
    aan weerszijden van de balken, aan beide kanten gelabeld in de eigen
    eenheden van de meter. De bovenkant en de onderkant (voor de digitale
    meter de ondergrens van de schaal) zijn altijd gemarkeerd; elk label heeft
    een streepje op elke liniaal, en kortere streepjes tussen de labels werken
    als die van een meetlint: gelijkmatig verdeeld op ronde waarden, elke 1 dB
    op de EBU-meter en elke 5 dB onder −20 op de digitale wanneer de meter
    hoog genoeg is, en elke 2, 2,5, 5 of 10 dB (of geen) waar hij daarvoor te
    laag is. Op de digitale (en aangepaste) meter labelt een hoge meter meer
    waarden: elke 1 dB van −20 tot 0, en elke 5 dB tussen de 10 dB-markeringen
    onder −20 (−45, −55), altijd gelijkmatig verdeeld en alleen waar ze passen
    zonder dat de labels elkaar raken; de andere metertypen houden de labels
    die hun standaard geeft. Het uitlijnniveau (−18 dBFS op de digitale meter)
    is een dikker wit streepje op beide linialen. Er wordt niets over de
    balken of ertussen getekend, dus wat je in de balken ziet is alleen het
    niveau, de piekvasthouding en de kleuren.
  - De balk is groen, geel vanaf het waarschuwingsniveau (−9 dBFS) en rood
    vanaf het gevarenniveau (−3 dBFS). De andere metertypen worden rood waar
    hun schaal dat doet (vanaf 0 VU, vanaf het toegestane maximum bij een PPM).
  - K-System-meters tonen twee delen: de volle balk is het gemiddelde (RMS)
    niveau en het gedimde deel erboven reikt tot de piek. Hun kleuren zijn die
    van het K-System: groen onder 0, amber van 0 tot +4, rood erboven.
  - Digitale piek-, K-System- en aangepaste meters houden het hoogste niveau
    even verlicht als een lijn (de piekvasthouding; de duur ervan is
    **Piekvasthouding** in [Instellingen → Meters](settings.md#meters), en 0
    zet hem uit). EBU PPM-, DIN PPM- en VU-meters hebben geen vasthouding.
  - Het getal erboven is het hoogste niveau sinds het item begon, in dBFS, rood
    in de gevarenzone. Het blijft staan na een stop en begint opnieuw wanneer
    een item speelt (het volgende of hetzelfde opnieuw), of wanneer je erop
    klikt.
  - Het getal eronder is de loudness in LUFS (EBU R128), groen binnen ±1 LU van
    het doel (−23 LUFS).
  - Het metertype en elk niveau kunnen worden gewijzigd in
    [Instellingen → Meters](settings.md#meters).
  - **Metingen boven 0 dBFS.** De meter toont wat de speler uitstuurt, en dat
    kan boven full scale uitkomen. De balk stopt bovenaan de schaal, dus
    hetzelfde rood toont 0 dBFS en alles erboven; alleen het getal erboven
    zegt hoeveel, met zijn teken (bijvoorbeeld `+3.5`).
    - Een bestand kan zelf niveaus boven full scale bevatten (een
      float-bestand, of een lossy bestand waarvan de gedecodeerde pieken
      erboven uitkomen).
    - Het omzetten van de frequentie kan pieken tussen de samples creëren: een
      signaal dat 0 dBFS raakt, leest na 44,1 → 48 kHz ongeveer +3 dBFS. De
      optie true peak leest die pieken ook.
    - De meter leest elke speler afzonderlijk, niet de som op het apparaat: twee
      spelers op één uitgang kunnen boven full scale optellen zonder dat een van
      beide meters het toont.
    - Niets in de speler voegt versterking boven 100 % toe. Een
      integer-apparaat clipt op full scale; een float-apparaat ontvangt het
      niveau zoals het is en het geluidssysteem of stuurprogramma clipt.
- **Volumefader** (rechts van de meter, even hoog): sleep hem of gebruik het
  muiswiel, één stap per tik. De tooltip toont het niveau in dB; bovenaan is 0
  dB en onderaan is stilte.
- **Titel, artiest** en de regel **volgende**, met een groen vierkantje.
  Terwijl CUE aanstaat, wordt de voorbeluisterpositie in blauw getoond.

## Transport {#transport}

| Knop | Actie |
|---|---|
| **Play / Volgende** (groot) | Gestopt: het volgende nummer starten. On air: overfaden naar het volgende nummer (de fadetijd stel je in bij [Instellingen](settings.md)). Gepauzeerd: hervatten. Als het nummer dat on air is ook het volgende is, start Play dat nummer opnieuw met de gebruikelijke fade. |
| **Stop** | Meteen stoppen (met een korte de-click-helling) |
| **Uitfaden** | Uitfaden en stoppen |
| **Pauze** | Pauzeren of hervatten; knippert amber terwijl gepauzeerd |
| **Stop na** (een afspeeldriehoek en dan een vierkant) | Eenmalig stoppen wanneer het huidige nummer eindigt. In SINGLE-modus is het alleen beschikbaar zolang het huidige nummer herhaalt: het beëindigt de herhaling wanneer de lopende ronde eindigt. Om elke keer dat een nummer speelt erna te stoppen, of om een nummer te herhalen, gebruik je het menu ervan in de playlist (zie [Playlists](playlists.md)) |
| **Vorig nummer** (een streep en twee driehoeken) | Terwijl on air: terugfaden naar het nummer dat deze speler eerder speelde, zoals Volgende doet. Druk nogmaals om verder terug te gaan. Het nummer dat je verliet, wordt het volgende. |
| **Nummer opnieuw starten** (een streep en één driehoek) | Terug naar het begin van het huidige nummer (zijn cue-in). Een gepauzeerde speler blijft gepauzeerd. |

Knoppen die nu niets kunnen doen, zijn gedimd: Stop en Opnieuw starten zonder
geladen nummer, Pauze en Uitfaden terwijl gestopt, Vorige zonder eerder nummer
of tijdens een fade. Een speler onthoudt de laatste 50 nummers die hij speelde
(`players.history_len` in `config.json`, 0 tot 1000).

## Modi {#modes}

- **CONT (doorlopend):** op het MIX-punt start de speler het volgende nummer en
  laat het het einde van het huidige overlappen. Zie
  [Markers en mixen](markers-and-mixing.md).
- **SINGLE:** elk nummer stopt aan zijn einde. *Stop na* is in deze modus niet
  beschikbaar, omdat elk nummer al stopt, behalve zolang het huidige nummer
  herhaalt: dan beëindigt het de herhaling wanneer de lopende ronde eindigt.

## Aftelling {#countdown}

Het grote getal is de tijd die resteert tot het einde van het nummer (zijn
cue-out), met tienden. De verstreken tijd en het totaal staan op de regel onder
de golfvorm, rechts. Tijdens de laatste seconden vóór het einde (standaard 10,
in te stellen in Instellingen) knippert de aftelling rood.

## Golfvorm {#waveform}

- Het deel dat al is gespeeld, is getekend in de kleur van de golfvorm; de rest
  is gedimder.
- De omtrek toont de pieken, vaag; het volle lichaam erin is het gemiddelde
  (RMS) niveau. Bij een luid nummer vullen de pieken de hoogte, en het lichaam
  toont nog steeds waar het nummer zachter of luider is.
- Een blauw gearceerd gebied aan het begin markeert de **intro**, en een badge
  telt hem af. De intro wordt alleen getoond wanneer hij is ingesteld.
- Een oranje gearceerd gebied aan het einde markeert de **outro**, met een eigen
  aftelling.
- Een gestippelde amberkleurige lijn met een label **MIX** markeert waar het
  volgende nummer begint in doorlopende modus. In de modus SINGLE is hij gedimd.
- Het hele bestand wordt getekend. Het stille begin en einde dat bij het
  afspelen wordt overgeslagen (vóór de cue-in en na de cue-out) is donkerder
  getekend, met een dun lijntje waar het afspelen begint en eindigt. Met
  **Cue-in en cue-out gebruiken** uit (Instellingen → Spelers) is niets
  donkerder en zijn de twee lijnen gedimd: het afspelen loopt van het begin tot
  het einde van het bestand, en de cue-in en cue-out in deze handleiding
  betekenen die twee uiteinden.
- Beweeg de muisaanwijzer om de tijd onder de aanwijzer te zien. **Klik om
  erheen te springen.** Op een gestopte speler kiest een klik waar **Play** het
  volgende nummer start: de afspeelkop en de aftelling verplaatsen daarheen, en
  er speelt niets totdat je op Play drukt. Een ander volgend nummer kiezen, het
  verplaatsen of verwijderen, of Stop gaat terug naar de cue-in; dat doet ook
  elke andere manier om een nummer te starten, en Opnieuw starten, Vorige en de
  automatische doorgang gebruiken altijd de cue-in. Een klik vóór de cue-in (in
  het donkerdere begin) kiest de cue-in. Een klik op of na de cue-out (in de
  donkerdere staart) annuleert een eerdere keuze: Play start bij de cue-in. Een
  klik is indrukken en loslaten zonder de aanwijzer meer dan een paar pixels te
  bewegen.
- **Indrukken en slepen** verplaatst de ingezoomde weergave langs het nummer,
  alsof je het vastpakt. Slepen springt nooit, en zonder zoom doet het niets.
  Alt-slepen bewerkt nog steeds markers.
- **Muiswiel** boven de golfvorm: in- en uitzoomen rond de aanwijzer, tot het
  fijnste detail dat de analyse heeft. **Shift+wiel** (of een zijwaarts wiel)
  verplaatst langs het nummer. Terwijl ingezoomd, volgt de weergave de
  afspeelpositie, behalve gedurende 10 seconden nadat je hebt gezoomd of
  verplaatst (`ui.follow_current_grace_secs`; 0 zet het volgen uit). **Volledig
  beeld**, in de rechterbovenhoek, helemaal uitzoomen of een nieuw nummer tonen
  weer het hele nummer.

## CUE (voorbeluisteren) {#cue-pre-listen}

**CUE** indrukken (of **Voorbeluisteren op CUE** in het menu van een nummer)
speelt het nummer af op de CUE-uitgang van de speler, bijvoorbeeld een
koptelefoon, zonder de uitgang die on air is aan te raken, en opent een klein
**CUE-venster** voor die speler. Er kunnen meerdere vensters open zijn, één per
speler. Zie [Instellingen](settings.md) om het CUE-apparaat te kiezen. Een
speler heeft een Cue-uitgang nodig die niet zijn Main-uitgang is: zonder die
zijn **CUE** en **Voorbeluisteren op CUE** gedimd, en de muisaanwijzer erboven
zegt dat.

![Het CUE-venster van speler 4, dat zijn volgende nummer voorbeluistert](../../images/guide/cue-window.png)

Het venster toont:

- de titel en de artiest;
- de golfvorm van het hele bestand met de CUE-positie. Hij werkt zoals die van
  de speler: klik om erheen te springen, zoom met het wiel, sleep om langs het
  nummer te bewegen, **Volledig beeld**, de aftellingen van intro en outro, en
  de markers voor intro, outro en MIX, die je hier net als op de speler
  bewerkt (zie [Markers en mixen](markers-and-mixing.md)). Een CUE speelt het
  hele bestand, dus niets is donkerder getekend, de cue-in en cue-out zijn
  gedimde lijnen, en de outro telt af tot het einde van het bestand. De zoom is
  zijn eigen: de golfvorm van de speler beweegt niet. Terwijl de CUE speelt,
  volgt een ingezoomde weergave zijn positie zoals die van de speler; een
  gepauzeerde CUE behoudt de weergave die je hebt ingesteld, zodat je kunt
  inzoomen en markers plaatsen. Een CUE die wordt gestart nadat zijn venster
  was gesloten, of op een ander nummer, toont het hele bestand;
- de verstreken tijd en de tijd die resteert tot het einde van het bestand (een
  CUE speelt hele bestanden);
- **CUE pauzeren** / **CUE hervatten**, **CUE stoppen** en **Als volgende
  instellen**. **Als volgende instellen** maakt van het gecuede nummer het
  volgende van de speler en laat de CUE doorspelen. Het is gedimd wanneer het
  nummer al het volgende is. Is het gecuede nummer het nummer dat on air is,
  dan speelt het nog een keer wanneer de huidige ronde eindigt.

Terwijl de CUE gepauzeerd is, knippert zijn knop **CUE pauzeren** (dan getoond
als **CUE hervatten**) amber, zoals die van de speler zelf.

Een sprong op een gepauzeerde CUE houdt hem gepauzeerd. De sluitknop van het
venster, of **CUE stoppen**, stopt de CUE.

Terwijl een CUE loopt, verplaatst het instellen van een volgend nummer
(dubbelklik) of een enkele klik op een rij de CUE naar dat nummer, vanaf zijn
cue-in; was hij gepauzeerd, dan speelt hij weer. Een nummer waarvan het bestand
ontbreekt of onleesbaar is, laat de CUE waar hij is.
