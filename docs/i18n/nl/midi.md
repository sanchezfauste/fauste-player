# MIDI-bedieningspanelen

Fauste Player kan worden bespeeld met MIDI-controllers: pad- en
fadercontrollers, bedieningspanelen in dj-stijl of keyboards. De
transportknoppen en de volumefader van elke speler kunnen aan een knop, een
toets of een fader worden gekoppeld, en knoppen met lampjes tonen wat elke
speler doet.

## Aanzetten {#turning-it-on}

![Instellingen, MIDI: de schakelaar om MIDI aan te zetten en de lijst met acties met elk een knop Leren](../../images/guide/settings-midi.png)

Open **Instellingen → MIDI** en vink **MIDI-bedieningspanelen gebruiken** aan.
De lijst onder **Invoerpoorten** toont elke MIDI-ingang van de computer en of
die verbonden is. Alleen de controllers die je hebt gekoppeld worden geopend
(sommige systemen geven een poort aan één programma tegelijk), plus elke
ingang terwijl je een bedieningselement leert; het programma luistert nooit
naar zijn eigen poorten. Controllers kunnen worden aan- of losgekoppeld terwijl
het programma draait: om de paar seconden wordt opnieuw naar de poorten
gezocht, en een controller die terugkomt wordt op naam verbonden, met zijn
lampjes opnieuw ingesteld.

Op Linux loopt MIDI via ALSA: je gebruiker moet de sequencer mogen openen
(`/dev/snd/seq`, meestal doordat hij in de groep `audio` zit).

## Een bedieningselement koppelen {#binding-a-control}

Voor elke speler is er een rij per actie: **Play / Volgende**, **Pauze**,
**Stop**, **Uitfaden**, **Opnieuw starten**, **Vorige**, **CUE** en
**Volume**.

1. Klik op **Leren** in de rij.
2. Druk op de knop of beweeg de fader die je wilt (intussen staat er **Beweeg
   een bedieningselement…**). Een knop accepteert een toets, een pad of een
   knop die een control change verstuurt; **Volume** accepteert een fader of
   draaiknop (een control change) of een pitch-bend-fader.
3. De rij toont het apparaat en het bedieningselement, bijvoorbeeld
   `APC mini · Noot 36, kanaal 1`.

Was dat bedieningselement al aan een andere actie gekoppeld, dan gaat het naar
deze. `Esc` of nogmaals op de knop klikken annuleert het leren. **Wissen**
verwijdert een koppeling.

## Hoe de bedieningselementen zich gedragen {#how-the-controls-behave}

- Een knop werkt zodra hij wordt ingedrukt (een toets of pad dat omlaag gaat,
  of een control change die door het midden van zijn bereik omhoog gaat),
  precies als de knop van de speler op het scherm. Een knop die op het scherm
  gedimd is, doet niets.
- Een fader verandert het volume op dezelfde schaal als de fader op het
  scherm. Om sprongen te vermijden neemt hij pas over wanneer hij het huidige
  volume bereikt of passeert (soft takeover); wordt het volume op het scherm
  gewijzigd, dan moet de fader het weer inhalen. Er verandert niets totdat je
  een bedieningselement beweegt: het programma starten brengt nooit iets on
  air.
- Met **Knoppen laten oplichten (LED-feedback)** aan lichten gekoppelde
  knoppen op: Play terwijl de speler on air is, Pauze knipperend terwijl hij
  gepauzeerd is, CUE tijdens het voorbeluisteren, en Stop, Uitfaden, Opnieuw
  starten en Vorige zolang ze kunnen werken. De lampjes gaan naar de
  uitgangspoort van de controller met dezelfde naam; een andere poort kan
  worden ingesteld in `config.json` (`midi.devices`).
