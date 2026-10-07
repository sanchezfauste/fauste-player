# Markers en mixen

Elk nummer heeft maximaal vijf **markers**, in seconden:

| Marker | Betekenis | Hoe het wordt ingesteld |
|---|---|---|
| Cue-in | Waar het afspelen begint | Automatisch: vlak vóór het eerste geluid boven de trimdrempel |
| Cue-out | Waar het nummer eindigt | Automatisch: vlak na het laatste geluid boven de trimdrempel |
| MIX (begin segue) | Waar het volgende nummer begint in doorlopende modus | Automatisch (zie hieronder) |
| Begin outro | Waar het slot van het nummer begint | Automatisch (zie hieronder) |
| Einde intro | Einde van de inleiding waaroverheen gesproken wordt | Met de hand, of uit een `INTRO`-tag in het bestand |

Markers die met de hand zijn gezet, gaan altijd voor: een nieuwe analyse
vervangt ze nooit.

## Markers bewerken {#editing-markers}

Op de golfvorm van een speler, of op de golfvorm van het CUE-venster ervan
(hetzelfde menu en dezelfde handvatten; een wijziging is in beide meteen
zichtbaar):

- **Klik met de rechtermuisknop** waar je een marker wilt en kies **Cue-in hier
  zetten**, **Einde intro hier zetten**, **Begin outro hier zetten**,
  **MIX-punt hier zetten** of **Cue-out hier zetten**. **Markers terugzetten op
  automatisch** verwijdert de markers die je hebt geplaatst, en het nummer
  wordt opnieuw geanalyseerd.
- **Houd Alt ingedrukt** (Option op macOS): er verschijnen handvatten op de
  markers. Sleep er een om hem te verplaatsen; de tijd wordt getoond tijdens
  het slepen. Slepen verplaatst de afspeelkop nooit.

De cue-in moet vóór de cue-out blijven. De andere markers blijven daartussen.
Wijzigingen aan het nummer dat on air is, gelden meteen voor zijn volgende
overgang.

## De INTRO-tag {#the-intro-tag}

Een bestand kan zijn introtijd bevatten in een `INTRO`-tag, als seconden
(`12.5`) of `m:ss`. De naam mag in elke schrijfwijze staan (`INTRO`, `Intro`).
Het kan een ID3v2-gebruikerstekstframe zijn (MP3, WAV, AIFF, DSF), een
Vorbis-, Opus- of FLAC-opmerking, een APE-item (WavPack, Monkey's Audio) of
een MP4-freeform-atoom. Het wordt tijdens de analyse gelezen. Een handmatig
einde van de intro gaat nog steeds voor.

## Hoe de automatische markers worden gevonden {#how-the-automatic-markers-are-found}

De analyse meet de pieken van het nummer in stappen van 10 ms en de luidheid in
korte vensters (standaard 50 ms).

- **Cue-in / cue-out:** alleen bijna-stilte aan het begin en einde wordt
  overgeslagen: alles waarvan de piek de *trimdrempel* (standaard −60 dBFS)
  bereikt, op een van beide kanalen, blijft behouden, met een *trimmarge*
  (standaard 20 ms) eromheen. Zachte fade-ins, stille uitloop en korte
  geluiden worden nooit afgesneden.
- **MIX:** de analyse zoekt het laatste punt waar het nummer nog minder dan de
  *niveaudaling voor segue* (standaard 15 dB) onder zijn eigen typische
  luidheid ligt, zodat luide en stille masters met dezelfde fade op dezelfde
  manier mixen. Dat punt ligt nooit meer dan de *maximale mixduur* (standaard
  4 s) vóór de cue-out, zodat overlappingen kort blijven.
- **Outro:** de analyse scant achterwaarts vanaf de cue-out en vindt waar het
  niveau meer dan de *niveaudaling outro* (standaard 6 dB) onder de mediane
  luidheid van het nummer zakt. De outro is standaard nooit langer dan 30 s.
- Nummers korter dan de *minimale duur voor mix- en outromarkers* (standaard
  60 s), zoals jingles en reclame, krijgen geen MIX en geen outro.

### Lange opnamen {#long-recordings}

Een heel programma (een, vier of meer uur) wordt geanalyseerd als een liedje,
zo nodig terwijl het speelt: een FLAC- of Opus-bestand van 4 uur kost op een
huidige computer minder dan een minuut, en het geheugengebruik groeit niet met
de lengte mee. Naar elk punt zoeken, zelfs vlak bij het einde, gaat meteen. De
golfvorm en markers worden in de analysecache bewaard tot ongeveer 16 uur
audio; een langer bestand werkt ook, maar wordt elke keer dat Fauste Player
start opnieuw geanalyseerd.

Al deze waarden staan in **Instellingen → Analyse**. Nadat je ze hebt
gewijzigd, worden de nummers automatisch opnieuw geanalyseerd.

## Wat de speler ermee doet {#what-the-player-does-with-them}

- **Doorlopende modus met automatisch mixen aan:** op het MIX-punt begint het
  volgende nummer op vol niveau terwijl het huidige uitfade tot zijn cue-out.
  De overlap is samplenauwkeurig.
- **Doorlopende modus zonder MIX-punt,** of met automatisch mixen uit: het
  volgende nummer begint precies op de cue-out, zonder gat.
- **SINGLE-modus**, of **Stop na**: de speler stopt op de cue-out.
- **Op Play drukken terwijl er iets on air is:** het volgende nummer begint
  meteen en het huidige fadet uit gedurende de *fadetijd* (standaard 1 s).
- **Cue-in en cue-out gebruiken uit** (Instellingen → Spelers): elke speler
  speelt elk nummer van 0 tot het einde van het bestand. Cue-in en cue-out,
  automatisch en handmatig, blijven bewaard, en de golfvorm tekent ze als
  gedimde lijnen. Het MIX-punt, de intro en de outro werken nog steeds,
  binnen het hele bestand; **Automatisch mixen op het MIX-punt** is een aparte
  schakelaar. Aftellingen, de kolom met de duur, de playlisttotalen in
  Instellingen en de tijden van de API op afstand volgen hetzelfde bereik.
  Carts gebruiken altijd hun eigen cue-in en cue-out. De instelling wijzigen
  herstart, verspringt of stopt nooit een nummer dat speelt; het volgende
  nummer wordt opnieuw voorbereid.

Een nummer kan worden afgespeeld voordat zijn analyse klaar is. Tot dan speelt
het van het begin tot het einde van het bestand, zonder MIX-punt.
