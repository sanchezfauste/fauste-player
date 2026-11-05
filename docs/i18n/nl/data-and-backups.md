# Gegevens en back-ups

## Waar bestanden staan {#where-files-are-kept}

| | Linux | Windows | macOS |
|---|---|---|---|
| Instellingen (`config.json`) | `~/.config/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\config\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Bibliotheek, playlists, carts en sessie (`fauste.db`) | `~/.local/share/fausteplayer/` | `%APPDATA%\Fauste\Fauste Player\data\` | `~/Library/Application Support/org.Fauste.Fauste-Player/` |
| Analysecache | `~/.cache/fausteplayer/` | `%LOCALAPPDATA%\Fauste\Fauste Player\cache\` | `~/Library/Caches/org.Fauste.Fauste-Player/` |
| Logbestanden en crashrapporten | `~/.local/share/fausteplayer/logs/` | `%LOCALAPPDATA%\Fauste\Fauste Player\data\logs\` | `~/Library/Application Support/org.Fauste.Fauste-Player/logs/` |

De Linux-paden volgen de XDG-variabelen (`XDG_CONFIG_HOME` enzovoort) wanneer
die zijn ingesteld.

De snap bewaart zijn bestanden in plaats daarvan in een eigen map: de
instellingen in `~/snap/fauste-player/current/.config/fausteplayer/`, de
database en logbestanden in
`~/snap/fauste-player/current/.local/share/fausteplayer/`, en de analysecache
in `~/snap/fauste-player/common/.cache/fausteplayer/`.
`sudo snap remove fauste-player` verwijdert ze ook (het bewaart een tijdje een
snapshot; `--purge` niet).

## Portable-modus {#portable-mode}

Stel de omgevingsvariabele `FAUSTE_HOME` in op een map, en alles wordt daar
bewaard, in `config/`, `data/`, `cache/` en `logs/`. Dat is handig op een
USB-stick, of om aparte opstellingen naast elkaar te houden.

## Bestanden {#files}

| Bestand | Inhoud |
|---|---|
| `config.json` | Instellingen, uitgangen, analysedrempels, geavanceerde afstelling |
| `fauste.db` | De bibliotheek (nummers, hun categorieën, handmatige markers, tags, loudness), de playlists met hun gespeeld-markeringen, de cartpagina's en de sessie: per speler de getoonde playlist, het huidige en volgende nummer, modus, positie, volume en kolombreedtes (per kolom), en de vensterindeling |
| `fauste.db.bak1`, `.bak2`, `.bak3` | Kopieën van de database (zie hieronder) |
| `window.ron` | De grootte en positie van het hoofdvenster en of het gemaximaliseerd was (alleen computer; op Wayland kiest het systeem de positie) |

## Automatisch opslaan en back-ups {#autosave-and-backups}

- Wijzigingen worden ongeveer een seconde nadat ze plaatsvinden opgeslagen, en
  alleen wat veranderd is wordt geschreven. Terwijl er iets speelt, wordt de
  sessie (posities) in hetzelfde tempo vernieuwd.
- De database wordt in transacties geschreven, zodat een stroomstoring nooit
  een half opgeslagen wijziging achterlaat.
- Bij de eerste keer opslaan van elke sessie, en daarna ongeveer elke vijf
  minuten zolang er wijzigingen worden opgeslagen, kopieert de applicatie de
  database naar `fauste.db.bak1` en schuift de oudere kopieën op naar `.bak2`
  en `.bak3`. Meerdere keren achter elkaar herstarten duwt de oudere back-ups
  dus weg.
- Kan de database niet worden gelezen, dan wordt de nieuwste goede back-up
  gebruikt. Het onleesbare bestand blijft bewaard, hernoemd naar
  `fauste.db.corrupt-<tijd>`, om te onderzoeken. Falen alle kopieën, dan start
  de applicatie leeg. Ze start altijd.
- Ook een kopie die niet bruikbaar is, blijft bewaard: wanneer de applicatie
  start vanaf een andere kopie (of een lege database), worden de onbruikbare
  hernoemd naar `fauste.db.bak1.corrupt-<tijd>` (of `.newer-<tijd>`), zodat de
  volgende back-up ze nooit wegduwt.
- Gebruikt een andere kopie van de applicatie de database, dan draait deze
  zonder op te slaan en zegt dat bij het starten.
- Een database die door een nieuwere versie van de applicatie is geschreven,
  wordt niet geopend: hij blijft bewaard als `fauste.db.newer-<tijd>`, en de
  applicatie start vanaf de nieuwste back-up die ze kan lezen, of leeg als er
  geen is, zodat de gegevens met een oudere versie openen ze nooit vernietigt.
- `config.json` wordt op dezelfde manier opgeslagen als voorheen: een tijdelijk
  bestand vervangt het oude, en de vorige versies blijven ernaast bewaard als
  `.bak1`, `.bak2` en `.bak3`.
- Kan de database in een sessie helemaal niet worden gebruikt (bijvoorbeeld
  omdat hij niet kan worden hernoemd doordat de map alleen-lezen is), dan draait
  de applicatie tot de volgende start zonder op te slaan, en zegt het log dat.

## Playlists en sessie van een eerdere versie {#playlists-and-session-from-an-earlier-version}

Eerdere versies bewaarden de playlists in `playlists.json`, de cartpagina's in
`carts.json` en de sessie in `session.json`. De eerste start van deze versie
leest ze (met hun `.bak`-kopieën) eenmalig en zet alles in `fauste.db`: de
bibliotheek met haar markers, tags en loudness, de playlists met hun
gespeeld-markeringen, de cartpagina's en de sessie. Daarna worden de oude
bestanden hernoemd met `.imported` aan het eind (`playlists.json.imported`,
`playlists.json.bak1.imported`, enzovoort). Ze worden niet meer gebruikt en je
kunt ze verwijderen zodra je zeker weet dat alles er is.

Om terug te gaan naar een eerdere versie:

1. Sluit de applicatie.
2. Verplaats `fauste.db`, `fauste.db-wal`, `fauste.db-shm` en elke
   `fauste.db.bak*` naar een map buiten de gegevensmap. Verwijder ze niet.
3. Haal `.imported` uit de namen van de oude bestanden.
4. Start de oudere versie.

Alles wat sinds de update is opgeslagen, staat alleen in de map waarheen je de
bestanden hebt verplaatst: de oude bestanden hebben het niet. Als je later weer
bijwerkt, worden de oude bestanden opnieuw geïmporteerd, zolang er geen
`fauste.db` of `fauste.db.bak*` in de gegevensmap staat.

Wordt een start tijdens het importeren onderbroken, dan maakt de volgende hem
af. Kan een van de oude bestanden niet worden gelezen (te groot, of geschreven
door een nieuwere versie), dan importeert de applicatie niets en draait ze
zonder die sessie op te slaan, zodat de oude bestanden blijven staan waar ze
staan; het log noemt het bestand. Herstel of verwijder het en start opnieuw.
Zodra `fauste.db` bestaat, worden oude JSON-bestanden nooit meer geïmporteerd.

## Herstel na een crash {#crash-recovery}

Na een crash of een herstart komt elke speler terug met zijn playlist, zijn
huidige en volgende nummer en zijn positie, maar **gepauzeerd of gestopt**. Er
gaat niets uit zichzelf on air. Een speler die helemaal aan het einde van zijn
nummer was, komt in plaats daarvan terug aan het begin van dat nummer, zodat
Play het afspeelt in plaats van het meteen te beëindigen.

## `config.json` met de hand bewerken {#editing-configjson-by-hand}

Sluit eerst de applicatie (als er audio on air is, vraagt ze dat eerst te
bevestigen). Onbekende of buiten het bereik vallende waarden worden bij het
laden van het bestand gecorrigeerd naar de dichtstbijzijnde geldige waarde, en
de correcties worden gelogd. De secties `limits` en `tuning` bevatten
geavanceerde waarden (resourcelimieten, engine-timing) die niet in het venster
Instellingen staan.
